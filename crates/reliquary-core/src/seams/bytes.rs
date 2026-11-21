//! `Bytes`: objects' bytes, kept under keys of their own on Mantle (architecture D3, D4; plan
//! RQ0.4). A put streams in pieces into an upload that resumes from its durable offset after a cut,
//! and becomes an object only whole, at its finish, held to its length and SHA-256; a get reads a
//! range (RFC 9110 §14.1) in pieces into its caller's buffer; head, delete, create-once; and a
//! grant another party redeems at Mantle to get or put one object without Reliquary relaying its
//! bytes. Mantle has no `-api` crate yet, so the trait is Reliquary's own (CLAUDE.md §9);
//! [`sim::SimBytes`] is its simulator, Mantle's side of a grant included, and [`suite`] its
//! conformance suite.
//!
//! No method carries a whole object: a piece is at most the upload's plan ([`Plan`], the store's
//! own figures, as Mantle's gateway gives its client its plan), and a read fills its caller's
//! buffer and no more, so moving an object costs memory that does not grow with it (AC-RQ0.4.4).
//! Every method that waits takes a deadline its caller derives (AC-RQ0.4.5).

pub mod sim;
pub mod suite;

use std::fmt;
use std::time::{Duration, Instant, SystemTime};

use aws_lc_rs::digest::{self, SHA256, SHA256_OUTPUT_LEN};

use crate::seams::BoxFuture;
use crate::seams::clock;
use crate::seams::fault::{CUT, DELAY, Kind, LOSE, Unanswered, VANISH};
use crate::seams::range::{Range, Span, Unresolved};

/// Objects' bytes.
pub trait Bytes: Send + Sync + 'static {
    /// Opens an upload to `key`. With [`Put::create_once`], a key that holds an object is refused
    /// [`Refusal::Exists`] here, and again at the finish.
    fn begin<'a>(
        &'a self,
        key: &'a str,
        put: Put,
        deadline: Instant,
    ) -> BoxFuture<'a, Result<Upload, Refusal>>;

    /// Sends `piece`, at most the upload's [`Plan::piece`] bytes, at byte `at` of the upload, which
    /// must be the upload's next ([`Progress::received`]). Answers where the upload stands after
    /// it.
    fn send<'a>(
        &'a self,
        upload: &'a UploadId,
        at: u64,
        piece: &'a [u8],
        deadline: Instant,
    ) -> BoxFuture<'a, Result<Progress, Refusal>>;

    /// Where the upload stands: what a resumed put goes on from.
    fn status<'a>(
        &'a self,
        upload: &'a UploadId,
        deadline: Instant,
    ) -> BoxFuture<'a, Result<Progress, Refusal>>;

    /// Ends the upload: its bytes become the object at its key, if they are `whole` (its length
    /// and SHA-256) and its condition holds. A finish refused leaves the upload open.
    fn finish<'a>(
        &'a self,
        upload: &'a UploadId,
        whole: Whole,
        deadline: Instant,
    ) -> BoxFuture<'a, Result<Meta, Refusal>>;

    /// Gives the upload up: its bytes go, and nothing appears.
    fn abort<'a>(
        &'a self,
        upload: &'a UploadId,
        deadline: Instant,
    ) -> BoxFuture<'a, Result<(), Refusal>>;

    /// Opens a reading of the object at `key`, of `range` or the whole: its facts, the bytes it
    /// selects, and a body that gives them in pieces.
    fn get<'a>(
        &'a self,
        key: &'a str,
        range: Option<Range>,
        deadline: Instant,
    ) -> BoxFuture<'a, Result<Reading, Refusal>>;

    /// The facts of the object at `key`.
    fn head<'a>(&'a self, key: &'a str, deadline: Instant) -> BoxFuture<'a, Result<Meta, Refusal>>;

    /// Removes the object at `key`; a key with none is answered [`Deleted::Absent`].
    fn delete<'a>(
        &'a self,
        key: &'a str,
        deadline: Instant,
    ) -> BoxFuture<'a, Result<Deleted, Refusal>>;

    /// A grant that lets `holder` do `access` to `key` at Mantle until `expires`, a wall time its
    /// caller derives (architecture D4: the transfer's bytes over the path's learned rate).
    fn grant<'a>(
        &'a self,
        key: &'a str,
        access: Access,
        holder: &'a str,
        expires: SystemTime,
        deadline: Instant,
    ) -> BoxFuture<'a, Result<Grant, Refusal>>;
}

/// The bytes a reading gives, in pieces.
pub trait Body: Send {
    /// The reading's next bytes, into `into`: how many, zero at its end. At most `into.len()`.
    fn read<'a>(
        &'a mut self,
        into: &'a mut [u8],
        deadline: Instant,
    ) -> BoxFuture<'a, Result<usize, Refusal>>;
}

/// How a put goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Put {
    /// Only if no object is at the key when it finishes (`If-None-Match: *`).
    pub create_once: bool,
    /// The operation's identity, from its caller's [`Entropy`](crate::seams::entropy::Entropy),
    /// as Mantle's client gives each mutation one: a begin asked again with it, its answer lost,
    /// is answered with the upload it made, so a lost answer leaves no upload its caller cannot
    /// name.
    pub identity: u128,
}

/// An upload's name, as the store gave it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UploadId(pub String);

/// The store's plan for an upload: the largest piece a send carries, and the run its bytes become
/// durable in. Both are the store's (Mantle's gateway gives its client its plan), never the
/// caller's.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Plan {
    pub piece: usize,
    pub run: u64,
}

/// Where an upload stands: the bytes it has received, and of those, the bytes that survive a cut.
/// A resumed put sends from `received`; after a cut, `received` is `durable`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Progress {
    pub received: u64,
    pub durable: u64,
}

/// An upload, as begun.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Upload {
    pub id: UploadId,
    pub plan: Plan,
    pub progress: Progress,
}

/// A SHA-256.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Sha256(pub [u8; SHA256_OUTPUT_LEN]);

impl Sha256 {
    /// The SHA-256 of `bytes`.
    #[must_use]
    pub fn of(bytes: &[u8]) -> Self {
        let mut hasher = Hasher::default();
        hasher.update(bytes);
        hasher.finish()
    }
}

impl fmt::Debug for Sha256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "sha256:")?;
        for byte in self.0 {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

/// A SHA-256 taken as bytes go by.
#[derive(Clone)]
pub struct Hasher(digest::Context);

impl Default for Hasher {
    fn default() -> Self {
        Self(digest::Context::new(&SHA256))
    }
}

impl Hasher {
    pub fn update(&mut self, bytes: &[u8]) {
        self.0.update(bytes);
    }

    #[must_use]
    pub fn finish(self) -> Sha256 {
        let digest = self.0.finish();
        let mut out = [0; SHA256_OUTPUT_LEN];
        for (into, from) in out.iter_mut().zip(digest.as_ref()) {
            *into = *from;
        }
        Sha256(out)
    }
}

/// What a finish holds its upload to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Whole {
    pub length: u64,
    pub sha256: Sha256,
}

/// An object's facts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Meta {
    pub key: String,
    pub length: u64,
    pub sha256: Sha256,
}

/// A reading of an object: its facts, the bytes the range selected, and a body that gives them.
pub struct Reading {
    pub meta: Meta,
    pub span: Span,
    pub body: Box<dyn Body>,
}

impl fmt::Debug for Reading {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Reading")
            .field("meta", &self.meta)
            .field("span", &self.span)
            .finish_non_exhaustive()
    }
}

/// What a grant redeemed at Mantle's side gives its holder.
#[derive(Debug)]
pub enum Redeemed {
    /// A get's reading of the object.
    Reading(Reading),
    /// A put's upload, of exactly the grant's length, which the holder sends and finishes.
    Upload(Upload),
}

/// What a delete did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Deleted {
    Removed,
    /// No object was at the key: deleted already.
    Absent,
}

/// What a grant lets its holder do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Get,
    Put,
}

/// A grant's access: its method, and the length it moves (a get: the object's; a put: the body's
/// exact length).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Access {
    pub method: Method,
    pub length: u64,
}

/// What lets one holder do one thing to one object at Mantle until one time: its key, access,
/// holder and expiry, and the store's tag over them. Any of them changed, the tag does not hold.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grant {
    pub key: String,
    pub access: Access,
    pub holder: String,
    pub expires: SystemTime,
    pub tag: Vec<u8>,
}

/// Why a call was refused, or ended without its answer.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Refusal {
    #[error("no object at {0:?}")]
    NotFound(String),
    #[error("no upload {0:?}: never begun, finished, or given up")]
    NoUpload(String),
    /// A create-once put to a key that holds an object.
    #[error("an object is at {0:?} already")]
    Exists(String),
    /// A send at a byte other than the upload's next.
    #[error("the upload's next byte is {expected}")]
    Offset { expected: u64 },
    /// A finish whose length or SHA-256 isn't the upload's, or a grant whose length isn't the
    /// object's.
    #[error("{0}")]
    Mismatch(String),
    /// A range that selects nothing of the object (RFC 9110 §14.1.2): 416, with its length.
    #[error("the range selects nothing of the object's {length} bytes")]
    Unsatisfiable { length: u64 },
    #[error("{what} isn't valid: {why}")]
    Invalid { what: &'static str, why: String },
    #[error("{what} is {size}, over its limit of {limit}")]
    TooLarge {
        what: &'static str,
        size: u64,
        limit: u64,
    },
    /// The store holds its most of something: objects, uploads, bytes.
    #[error("the store holds its most {what} ({limit})")]
    Full { what: &'static str, limit: u64 },
    /// The store admits no more work now (architecture D9), with when to ask again if it says.
    #[error("the store is busy{}", retry_after.map(|d| format!("; ask again after {d:?}")).unwrap_or_default())]
    Busy { retry_after: Option<Duration> },
    /// A grant whose tag does not hold for what it says, or presented by another than its holder.
    #[error("the grant isn't valid here: {0}")]
    Forbidden(String),
    #[error("the grant has expired")]
    Expired,
    /// The stream was cut. A put's goes on from `at`, its durable offset; a reading's gave the
    /// bytes before `at`.
    #[error("the stream was cut at byte {at}")]
    Cut { at: u64 },
    /// The caller's deadline passed with no answer: what the call did, the caller learns by
    /// reading.
    #[error("the deadline passed before the call was answered")]
    Late,
    /// The request went whole and its answer was lost: it may have applied, and the caller
    /// learns which by reading.
    #[error("the answer was lost: the call may have applied")]
    Lost,
    #[error("the store is unavailable: {0}")]
    Unavailable(String),
}

impl Kind for Refusal {
    fn kind(&self) -> &'static str {
        match self {
            Self::NotFound(_) => "NotFound",
            Self::NoUpload(_) => "NoUpload",
            Self::Exists(_) => "Exists",
            Self::Offset { .. } => "Offset",
            Self::Mismatch(_) => "Mismatch",
            Self::Unsatisfiable { .. } => "Unsatisfiable",
            Self::Invalid { .. } => "Invalid",
            Self::TooLarge { .. } => "TooLarge",
            Self::Full { .. } => "Full",
            Self::Busy { .. } => "Busy",
            Self::Forbidden(_) => "Forbidden",
            Self::Expired => "Expired",
            Self::Cut { .. } => "Cut",
            Self::Late => "Late",
            Self::Lost => "Lost",
            Self::Unavailable(_) => "Unavailable",
        }
    }
}

impl Unanswered for Refusal {
    fn late() -> Self {
        Self::Late
    }

    fn lost() -> Self {
        Self::Lost
    }

    fn clock(refusal: clock::Refusal) -> Self {
        Self::Unavailable(format!("the store's clock: {refusal}"))
    }
}

impl From<Unresolved> for Refusal {
    fn from(unresolved: Unresolved) -> Self {
        match unresolved {
            // Said in fixed words: no formatting runs here, so nothing a future polls reaches
            // a formatter's writer (the recursion gate reads a writer's call as any method at
            // its slot).
            Unresolved::Invalid { .. } => Self::Invalid {
                what: "a range",
                why: String::from("its last byte is before its first"),
            },
            Unresolved::Unsatisfiable { length } => Self::Unsatisfiable { length },
        }
    }
}

/// The faults a bytes simulator injects (`docs/design/seams.md`, "Bytes"): each refusal, an
/// answer delayed past its caller's deadline, a request that never arrives, an answer lost after
/// its effect, and a put's or a reading's stream cut at any byte.
pub const FAULTS: &[&str] = &[
    "NotFound",
    "NoUpload",
    "Exists",
    "Offset",
    "Mismatch",
    "Unsatisfiable",
    "Invalid",
    "TooLarge",
    "Full",
    "Busy",
    "Forbidden",
    "Expired",
    "Cut",
    "Late",
    "Lost",
    "Unavailable",
    DELAY,
    VANISH,
    LOSE,
    CUT,
];

/// The operations a fault can be armed for: each method of [`Bytes`], a body's read, and Mantle's
/// side of a grant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Op {
    Begin,
    Send,
    Status,
    Finish,
    Abort,
    Get,
    Read,
    Head,
    Delete,
    Grant,
    Redeem,
}

/// What a grant's tag is taken over, in an encoding no two grants share: its method, its length,
/// its expiry in nanoseconds since the Unix epoch, and its key and holder, each after its length.
pub fn signed(
    key: &str,
    access: Access,
    holder: &str,
    expires: SystemTime,
) -> Result<Vec<u8>, Refusal> {
    let since = expires
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|_| Refusal::Invalid {
            what: "a grant's expiry",
            why: "before the Unix epoch".into(),
        })?;
    let width = |s: &str| u64::try_from(s.len()).unwrap_or(u64::MAX).to_le_bytes();
    let mut message = Vec::new();
    message.extend_from_slice(match access.method {
        Method::Get => b"GET",
        Method::Put => b"PUT",
    });
    message.extend_from_slice(&access.length.to_le_bytes());
    message.extend_from_slice(&since.as_nanos().to_le_bytes());
    message.extend_from_slice(&width(key));
    message.extend_from_slice(key.as_bytes());
    message.extend_from_slice(&width(holder));
    message.extend_from_slice(holder.as_bytes());
    Ok(message)
}
