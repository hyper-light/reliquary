//! `Upstream`: another registry, which a pull-through cache or a mirror reads from (plan RQ0.4,
//! RQ8; architecture D17). A manifest by tag or digest, its head alone (the cheapest request a tag
//! is revalidated with: Docker Hub counts a manifest `GET` as a pull and a `HEAD` as none,
//! research/05 F31 to F36), and a blob by digest, whole or by range (RFC 9110 §14.1), each body in
//! pieces into its caller's buffer. No product's `-api` crate names this, so the trait is
//! Reliquary's own; [`SimUpstream`] is a registry in memory, its budget of pulls included.
//!
//! An upstream is not trusted: a body asked by its digest is held to it as its bytes arrive, and
//! one that does not hash to it ends refused [`Refusal::Mismatch`], never as a whole body.

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::Mutex;

use crate::seams::BoxFuture;
use crate::seams::bytes::{Hasher, Sha256};
use crate::seams::clock::{self, Clock, SimClock};
use crate::seams::conformance::{Case, Failure, Fixture, ensure, fine, refused, same};
use crate::seams::fault::{
    self, CUT, Counts, DELAY, Fault, Faults, Kind, LOSE, Unanswered, Unarmed, VANISH,
};
use crate::seams::range::{self, Range, Span, Unresolved};

/// Another registry.
pub trait Upstream: Send + Sync + 'static {
    /// The manifest `reference` names in `repository`: its facts and its body.
    fn manifest<'a>(
        &'a self,
        repository: &'a str,
        reference: &'a Reference,
        deadline: Instant,
    ) -> BoxFuture<'a, Result<Fetched, Refusal>>;

    /// The manifest's facts alone, its body not sent.
    fn head<'a>(
        &'a self,
        repository: &'a str,
        reference: &'a Reference,
        deadline: Instant,
    ) -> BoxFuture<'a, Result<Described, Refusal>>;

    /// The blob `digest` in `repository`, or the bytes of `range` of it.
    fn blob<'a>(
        &'a self,
        repository: &'a str,
        digest: &'a Sha256,
        range: Option<Range>,
        deadline: Instant,
    ) -> BoxFuture<'a, Result<Fetched, Refusal>>;
}

/// The bytes a fetch gives, in pieces.
pub trait Body: Send {
    /// The fetch's next bytes, into `into`: how many, zero at its end. A whole body that does not
    /// hash to its digest ends refused [`Refusal::Mismatch`] in place of its zero.
    fn read<'a>(
        &'a mut self,
        into: &'a mut [u8],
        deadline: Instant,
    ) -> BoxFuture<'a, Result<usize, Refusal>>;
}

/// What names a manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reference {
    Tag(String),
    Digest(Sha256),
}

/// A manifest's or a blob's facts, as the upstream says them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Described {
    pub digest: Sha256,
    pub media_type: String,
    pub length: u64,
}

/// A fetch: its facts, the bytes the range selected, and a body that gives them.
pub struct Fetched {
    pub described: Described,
    pub span: Span,
    pub body: Box<dyn Body>,
}

impl std::fmt::Debug for Fetched {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Fetched")
            .field("described", &self.described)
            .field("span", &self.span)
            .finish_non_exhaustive()
    }
}

/// Why a call was refused, or ended without its answer.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Refusal {
    /// The upstream has no such repository, manifest or blob.
    #[error("the upstream has no {0}")]
    NotFound(String),
    /// The upstream asks for a credential, or refused the one it was given (401).
    #[error("the upstream wants a credential it takes: {0}")]
    Unauthorized(String),
    /// The upstream refuses this holder (403).
    #[error("the upstream refuses it: {0}")]
    Denied(String),
    /// The upstream's limit on requests is reached (429), with its `Retry-After` if it said one.
    #[error("the upstream's limit is reached{}", retry_after.map(|d| format!("; ask again after {d:?}")).unwrap_or_default())]
    TooMany { retry_after: Option<Duration> },
    #[error("the range selects nothing of the body's {length} bytes")]
    Unsatisfiable { length: u64 },
    #[error("{what} isn't valid: {why}")]
    Invalid { what: &'static str, why: String },
    /// A body that does not hash to the digest it was asked by.
    #[error("the body's digest is {got:?}, not the {want:?} it was asked by")]
    Mismatch { want: Sha256, got: Sha256 },
    /// The body's stream was cut at byte `at`: the bytes before it were given.
    #[error("the stream was cut at byte {at}")]
    Cut { at: u64 },
    #[error("the deadline passed before the call was answered")]
    Late,
    #[error("the answer was lost: the request may have counted against the upstream's limit")]
    Lost,
    #[error("the upstream is unavailable: {0}")]
    Unavailable(String),
}

impl Kind for Refusal {
    fn kind(&self) -> &'static str {
        match self {
            Self::NotFound(_) => "NotFound",
            Self::Unauthorized(_) => "Unauthorized",
            Self::Denied(_) => "Denied",
            Self::TooMany { .. } => "TooMany",
            Self::Unsatisfiable { .. } => "Unsatisfiable",
            Self::Invalid { .. } => "Invalid",
            Self::Mismatch { .. } => "Mismatch",
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
        Self::Unavailable(format!("the caller's clock: {refusal}"))
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

/// The faults an upstream simulator injects (`docs/design/seams.md`, "Upstream"): each refusal,
/// an answer delayed past its caller's deadline, a request that never arrives, an answer lost after
/// the upstream counted it, and a body cut at any byte.
pub const FAULTS: &[&str] = &[
    "NotFound",
    "Unauthorized",
    "Denied",
    "TooMany",
    "Unsatisfiable",
    "Invalid",
    "Mismatch",
    "Cut",
    "Late",
    "Lost",
    "Unavailable",
    DELAY,
    VANISH,
    LOSE,
    CUT,
];

/// The operations a fault can be armed for: each method of [`Upstream`], and a body's read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Op {
    Manifest,
    Head,
    Blob,
    Read,
}

fn fits(op: Op, fault: &Fault<Refusal>) -> bool {
    match fault {
        Fault::Cut(_) => op == Op::Read,
        Fault::Lose => op != Op::Read,
        Fault::Refuse(_) | Fault::Delay(_) | Fault::Vanish => true,
    }
}

struct Content {
    media_type: String,
    bytes: Arc<[u8]>,
}

struct Registry {
    /// Each repository's tags, and the digests it holds.
    tags: BTreeMap<(String, String), Sha256>,
    held: BTreeMap<(String, Sha256), ()>,
    contents: BTreeMap<Sha256, Content>,
    /// Pulls left before the upstream says TooMany: a manifest's get costs one, its head none.
    budget: Option<u64>,
    retry_after: Option<Duration>,
    faults: Faults<Op, Refusal>,
}

/// A registry in memory: its repositories' tags, manifests and blobs, and its budget of pulls.
pub struct SimUpstream {
    clock: Arc<dyn Clock>,
    registry: Arc<Mutex<Registry>>,
}

impl Registry {
    fn content(&self, repository: &str, digest: &Sha256) -> Result<&Content, Refusal> {
        if !self.held.contains_key(&(repository.to_owned(), *digest)) {
            return Err(Refusal::NotFound(format!("{digest:?} in {repository}")));
        }
        self.contents
            .get(digest)
            .ok_or_else(|| Refusal::NotFound(format!("{digest:?}")))
    }

    fn resolve(&self, repository: &str, reference: &Reference) -> Result<Sha256, Refusal> {
        match reference {
            Reference::Digest(digest) => Ok(*digest),
            Reference::Tag(tag) => self
                .tags
                .get(&(repository.to_owned(), tag.clone()))
                .copied()
                .ok_or_else(|| Refusal::NotFound(format!("tag {tag:?} in {repository}"))),
        }
    }
}

impl SimUpstream {
    /// An empty registry on `clock`, with no limit on pulls.
    #[must_use]
    pub fn new(clock: Arc<dyn Clock>) -> Self {
        Self {
            clock,
            registry: Arc::new(Mutex::new(Registry {
                tags: BTreeMap::new(),
                held: BTreeMap::new(),
                contents: BTreeMap::new(),
                budget: None,
                retry_after: None,
                faults: Faults::default(),
            })),
        }
    }

    /// Pushes `bytes` to `repository` as content of `media_type`: its digest.
    pub fn push(&self, repository: &str, media_type: &str, bytes: &[u8]) -> Sha256 {
        let digest = Sha256::of(bytes);
        let mut registry = self.registry.lock();
        registry.contents.insert(
            digest,
            Content {
                media_type: media_type.to_owned(),
                bytes: Arc::from(bytes),
            },
        );
        registry.held.insert((repository.to_owned(), digest), ());
        digest
    }

    /// Points `tag` in `repository` at `digest`.
    pub fn tag(&self, repository: &str, tag: &str, digest: Sha256) {
        self.registry
            .lock()
            .tags
            .insert((repository.to_owned(), tag.to_owned()), digest);
    }

    /// Serves `digest` as `bytes` that are not its own, as a hostile or broken upstream would.
    pub fn corrupt(&self, digest: &Sha256, bytes: &[u8]) {
        if let Some(content) = self.registry.lock().contents.get_mut(digest) {
            content.bytes = Arc::from(bytes);
        }
    }

    /// Limits the pulls the upstream allows to `budget`, and says `retry_after` once they are
    /// spent.
    pub fn limit(&self, budget: u64, retry_after: Option<Duration>) {
        let mut registry = self.registry.lock();
        registry.budget = Some(budget);
        registry.retry_after = retry_after;
    }

    /// The pulls the upstream allows still, if it limits them.
    #[must_use]
    pub fn budget(&self) -> Option<u64> {
        self.registry.lock().budget
    }

    /// Arms `fault` for the next call of `op` (a cut, for the next read that reaches its byte).
    pub fn arm(&self, op: Op, fault: Fault<Refusal>) -> Result<(), Unarmed> {
        self.registry.lock().faults.arm(op, fault, fits)
    }

    /// How many calls of `op` were made.
    #[must_use]
    pub fn attempts(&self, op: Op) -> u64 {
        self.registry.lock().faults.attempts(op)
    }

    /// What it injected, by kind.
    #[must_use]
    pub fn injected(&self) -> Counts {
        self.registry.lock().faults.injected().clone()
    }

    /// A fetch of `digest`'s content in `repository`, of `range` or the whole: held to the digest
    /// when it is whole.
    fn fetch(
        &self,
        registry: &Registry,
        repository: &str,
        digest: Sha256,
        range: Option<Range>,
    ) -> Result<Fetched, Refusal> {
        let content = registry.content(repository, &digest)?;
        let length = u64::try_from(content.bytes.len()).unwrap_or(u64::MAX);
        let span = match range {
            Some(range) => range::resolve(range, length)?,
            None => Span::whole(length),
        };
        let whole = span == Span::whole(length);
        Ok(Fetched {
            described: Described {
                digest,
                media_type: content.media_type.clone(),
                length,
            },
            span,
            body: Box::new(SimBody {
                clock: Arc::clone(&self.clock),
                registry: Arc::clone(&self.registry),
                bytes: Arc::clone(&content.bytes),
                at: span.first,
                end: span.end(),
                check: whole.then(|| (digest, Hasher::default())),
                ended: None,
            }),
        })
    }
}

impl Upstream for SimUpstream {
    fn manifest<'a>(
        &'a self,
        repository: &'a str,
        reference: &'a Reference,
        deadline: Instant,
    ) -> BoxFuture<'a, Result<Fetched, Refusal>> {
        Box::pin(async move {
            let fault = self.registry.lock().faults.call(Op::Manifest);
            fault::answer(self.clock.as_ref(), fault, deadline, || {
                let mut registry = self.registry.lock();
                // A manifest's get is a pull: it costs the budget, whatever its answer.
                if let Some(left) = registry.budget {
                    let spent = left.checked_sub(1).ok_or(Refusal::TooMany {
                        retry_after: registry.retry_after,
                    })?;
                    registry.budget = Some(spent);
                }
                let digest = registry.resolve(repository, reference)?;
                self.fetch(&registry, repository, digest, None)
            })
            .await
        })
    }

    fn head<'a>(
        &'a self,
        repository: &'a str,
        reference: &'a Reference,
        deadline: Instant,
    ) -> BoxFuture<'a, Result<Described, Refusal>> {
        Box::pin(async move {
            let fault = self.registry.lock().faults.call(Op::Head);
            fault::answer(self.clock.as_ref(), fault, deadline, || {
                let registry = self.registry.lock();
                let digest = registry.resolve(repository, reference)?;
                self.fetch(&registry, repository, digest, None)
                    .map(|fetched| fetched.described)
            })
            .await
        })
    }

    fn blob<'a>(
        &'a self,
        repository: &'a str,
        digest: &'a Sha256,
        range: Option<Range>,
        deadline: Instant,
    ) -> BoxFuture<'a, Result<Fetched, Refusal>> {
        Box::pin(async move {
            let fault = self.registry.lock().faults.call(Op::Blob);
            fault::answer(self.clock.as_ref(), fault, deadline, || {
                let registry = self.registry.lock();
                self.fetch(&registry, repository, *digest, range)
            })
            .await
        })
    }
}

/// A fetch's body: the bytes from `at` to `end` of the content it holds, a whole one held to its
/// digest as it goes.
struct SimBody {
    clock: Arc<dyn Clock>,
    registry: Arc<Mutex<Registry>>,
    bytes: Arc<[u8]>,
    at: u64,
    end: u64,
    check: Option<(Sha256, Hasher)>,
    /// How the fetch ended, if it was cut, went unanswered or did not hash to its digest: every
    /// later read is refused the same.
    ended: Option<Refusal>,
}

impl SimBody {
    fn take(&mut self, into: &mut [u8]) -> Result<usize, Refusal> {
        let left = self.end.saturating_sub(self.at);
        let mut n = u64::try_from(into.len()).unwrap_or(u64::MAX).min(left);
        let to = self.at.saturating_add(n);
        if let Some(at) = self.registry.lock().faults.cut(Op::Read, self.at, to) {
            self.ended = Some(Refusal::Cut { at });
            if at == self.at {
                return Err(Refusal::Cut { at });
            }
            n = at.saturating_sub(self.at);
        }
        if n == 0 {
            // The end: a whole body is held to its digest here.
            if let Some((want, hasher)) = self.check.take() {
                let got = hasher.finish();
                if got != want {
                    let mismatch = Refusal::Mismatch { want, got };
                    self.ended = Some(mismatch.clone());
                    return Err(mismatch);
                }
            }
            return Ok(0);
        }
        let start = usize::try_from(self.at).unwrap_or(usize::MAX);
        let want = usize::try_from(n).unwrap_or(usize::MAX);
        let from = self
            .bytes
            .get(start..)
            .and_then(|rest| rest.get(..want))
            .ok_or_else(|| Refusal::Unavailable("a body shorter than its span".into()))?;
        let into = into
            .get_mut(..want)
            .ok_or_else(|| Refusal::Unavailable("a read past its buffer".into()))?;
        for (to, byte) in into.iter_mut().zip(from) {
            *to = *byte;
        }
        if let Some((_, hasher)) = &mut self.check {
            hasher.update(from);
        }
        self.at = self.at.saturating_add(n);
        Ok(want)
    }
}

impl Body for SimBody {
    fn read<'a>(
        &'a mut self,
        into: &'a mut [u8],
        deadline: Instant,
    ) -> BoxFuture<'a, Result<usize, Refusal>> {
        Box::pin(async move {
            if let Some(ended) = &self.ended {
                return Err(ended.clone());
            }
            let fault = self.registry.lock().faults.call(Op::Read);
            // A read's answer is its bytes: held, they arrive late, and past the deadline they do
            // not arrive at all, and the stream is gone.
            let held = match fault {
                Some(Fault::Refuse(refusal)) => return Err(refusal),
                Some(Fault::Vanish) => Some(None),
                Some(Fault::Delay(by)) => Some(self.clock.now().checked_add(by)),
                Some(Fault::Lose | Fault::Cut(_)) | None => None,
            };
            if let Some(at) = held
                && !fault::held(self.clock.as_ref(), at, deadline)
                    .await
                    .map_err(Refusal::clock)?
            {
                self.ended = Some(Refusal::Late);
                return Err(Refusal::Late);
            }
            self.take(into)
        })
    }
}

// -------------------------------------------------------------------------------- conformance

/// What an upstream's conformance suite runs against.
pub trait UpstreamFixture: Fixture {
    fn upstream(&self) -> &dyn Upstream;
    /// The clock the caller's deadlines are on.
    fn clock(&self) -> &dyn Clock;
    /// Arms `fault` for the upstream's next call of `op`.
    fn arm(&self, op: Op, fault: Fault<Refusal>) -> Result<(), Failure>;
    /// How many calls of `op` the upstream has had.
    fn attempts(&self, op: Op) -> u64;
    /// How long a case's caller waits for an answer: each deadline is the clock's now plus this.
    fn patience(&self) -> Duration;
    /// Pushes `bytes` to `repository` as content of `media_type`, tagged `tag` if it names one:
    /// its digest.
    fn push(&self, repository: &str, tag: Option<&str>, media_type: &str, bytes: &[u8]) -> Sha256;
    /// Makes the upstream serve `bytes` for `digest`, which are not its.
    fn corrupt(&self, digest: &Sha256, bytes: &[u8]);
    /// Limits the upstream to `budget` pulls more, saying `retry_after` once they are spent.
    fn limit(&self, budget: u64, retry_after: Option<Duration>);
    /// The pulls the upstream says it allows still.
    fn budget(&self) -> Option<u64>;
    /// How long a case's bodies are: several of its reads' buffers, and at least two bytes.
    fn length(&self) -> u64;
    /// The buffer a case reads bodies into.
    fn buffer(&self) -> usize;
}

/// The fixture for a [`SimUpstream`]: the clock it was made with, and what the test that makes it
/// states.
pub struct SimUpstreamFixture {
    pub upstream: SimUpstream,
    pub clock: SimClock,
    pub patience: Duration,
    pub steps: u64,
    pub length: u64,
    pub buffer: usize,
}

impl Fixture for SimUpstreamFixture {
    fn drive<'a, T>(&'a self, work: BoxFuture<'a, T>) -> Result<T, Failure> {
        self.clock
            .run(work, self.steps)
            .map_err(|stalled| stalled.to_string())
    }

    fn injected(&self) -> Counts {
        self.upstream.injected()
    }
}

impl UpstreamFixture for SimUpstreamFixture {
    fn upstream(&self) -> &dyn Upstream {
        &self.upstream
    }

    fn clock(&self) -> &dyn Clock {
        &self.clock
    }

    fn arm(&self, op: Op, fault: Fault<Refusal>) -> Result<(), Failure> {
        self.upstream.arm(op, fault).map_err(|e| e.to_string())
    }

    fn attempts(&self, op: Op) -> u64 {
        self.upstream.attempts(op)
    }

    fn patience(&self) -> Duration {
        self.patience
    }

    fn push(&self, repository: &str, tag: Option<&str>, media_type: &str, bytes: &[u8]) -> Sha256 {
        let digest = self.upstream.push(repository, media_type, bytes);
        if let Some(tag) = tag {
            self.upstream.tag(repository, tag, digest);
        }
        digest
    }

    fn corrupt(&self, digest: &Sha256, bytes: &[u8]) {
        self.upstream.corrupt(digest, bytes);
    }

    fn limit(&self, budget: u64, retry_after: Option<Duration>) {
        self.upstream.limit(budget, retry_after);
    }

    fn budget(&self) -> Option<u64> {
        self.upstream.budget()
    }

    fn length(&self) -> u64 {
        self.length
    }

    fn buffer(&self) -> usize {
        self.buffer
    }
}

const MANIFEST: &str = "application/vnd.oci.image.manifest.v1+json";
const LAYER: &str = "application/vnd.oci.image.layer.v1.tar";

fn deadline<F: UpstreamFixture>(f: &F) -> Result<Instant, Failure> {
    f.clock()
        .now()
        .checked_add(f.patience())
        .ok_or_else(|| "a deadline past what the platform's time can say".to_owned())
}

/// `length` bytes drawn for `name`.
fn content(name: &str, length: u64) -> Result<Vec<u8>, Failure> {
    let mut seed = [0u8; size_of::<u64>()];
    for (into, from) in seed.iter_mut().zip(Sha256::of(name.as_bytes()).0) {
        *into = from;
    }
    let mut bytes = vec![0u8; usize::try_from(length).map_err(|e| e.to_string())?];
    crate::seams::entropy::Stream::new(u64::from_le_bytes(seed))
        .fill(&mut bytes)
        .map_err(|e| e.to_string())?;
    Ok(bytes)
}

/// Every byte `fetched` gives, or what it ended in and the bytes before.
async fn read_all<F: UpstreamFixture>(
    f: &F,
    fetched: &mut Fetched,
) -> Result<Result<Vec<u8>, (Vec<u8>, Refusal)>, Failure> {
    let mut out = Vec::new();
    let mut into = vec![0u8; f.buffer()];
    loop {
        let n = match fetched.body.read(&mut into, deadline(f)?).await {
            Ok(n) => n,
            Err(refusal) => return Ok(Err((out, refusal))),
        };
        if n == 0 {
            return Ok(Ok(out));
        }
        out.extend_from_slice(
            into.get(..n)
                .ok_or_else(|| format!("a read said {n} bytes into a buffer of {}", f.buffer()))?,
        );
        ensure(
            u64::try_from(out.len()).unwrap_or(u64::MAX) <= fetched.span.length,
            || "a fetch gave more bytes than its span".into(),
        )?;
    }
}

async fn whole<F: UpstreamFixture>(f: &F, mut fetched: Fetched) -> Result<Vec<u8>, Failure> {
    fine(
        "a body",
        read_all(f, &mut fetched).await?.map_err(|(_, r)| r),
    )
}

fn u1<F: UpstreamFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        let bytes = content("u1", f.length())?;
        let digest = f.push("u1", Some("latest"), MANIFEST, &bytes);
        for reference in [Reference::Tag("latest".into()), Reference::Digest(digest)] {
            let fetched = fine(
                "a manifest",
                f.upstream().manifest("u1", &reference, deadline(f)?).await,
            )?;
            same("its digest", fetched.described.digest, digest)?;
            same(
                "its media type",
                fetched.described.media_type.as_str(),
                MANIFEST,
            )?;
            let body = whole(f, fetched).await?;
            same("its bytes' digest", Sha256::of(&body), digest)?;
        }
        Ok(())
    })
}

fn u2<F: UpstreamFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        let bytes = content("u2", f.length())?;
        let digest = f.push("u2", Some("latest"), MANIFEST, &bytes);
        f.limit(f.length(), None);
        let described = fine(
            "a head",
            f.upstream()
                .head("u2", &Reference::Tag("latest".into()), deadline(f)?)
                .await,
        )?;
        same(
            "its facts",
            described,
            Described {
                digest,
                media_type: MANIFEST.into(),
                length: f.length(),
            },
        )?;
        same("the budget after a head", f.budget(), Some(f.length()))?;
        let fetched = fine(
            "a manifest",
            f.upstream()
                .manifest("u2", &Reference::Tag("latest".into()), deadline(f)?)
                .await,
        )?;
        drop(fetched);
        same(
            "the budget after a get",
            f.budget(),
            Some(f.length().saturating_sub(1)),
        )
    })
}

fn u3<F: UpstreamFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        let bytes = content("u3", f.length())?;
        let digest = f.push("u3", None, LAYER, &bytes);
        let fetched = fine(
            "a blob",
            f.upstream().blob("u3", &digest, None, deadline(f)?).await,
        )?;
        ensure(whole(f, fetched).await? == bytes, || {
            "a blob's bytes are not those pushed".into()
        })?;
        let length = f.length();
        let last = length
            .checked_sub(1)
            .filter(|last| *last > 0)
            .ok_or_else(|| "the fixture's bodies must be at least two bytes".to_owned())?;
        let byte = |at: u64| -> Result<Vec<u8>, Failure> {
            let at = usize::try_from(at).map_err(|e| e.to_string())?;
            Ok(bytes.get(at..=at).map(<[u8]>::to_vec).unwrap_or_default())
        };
        for (range, want) in [
            (
                Range::From {
                    first: 0,
                    last: Some(0),
                },
                byte(0)?,
            ),
            (
                Range::From {
                    first: last,
                    last: None,
                },
                byte(last)?,
            ),
            (Range::Suffix(1), byte(last)?),
        ] {
            let fetched = fine(
                "a ranged blob",
                f.upstream()
                    .blob("u3", &digest, Some(range), deadline(f)?)
                    .await,
            )?;
            ensure(whole(f, fetched).await? == want, || {
                format!("the bytes of {range:?}")
            })?;
        }
        for range in [
            Range::From {
                first: length,
                last: None,
            },
            Range::Suffix(0),
        ] {
            same(
                &format!("a blob of {range:?}"),
                f.upstream()
                    .blob("u3", &digest, Some(range), deadline(f)?)
                    .await
                    .map(|fetched| fetched.span),
                Err(Refusal::Unsatisfiable { length }),
            )?;
        }
        Ok(())
    })
}

fn u4<F: UpstreamFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        let bytes = content("u4", f.length())?;
        let digest = f.push("u4", Some("latest"), MANIFEST, &bytes);
        let other = content("u4-other", f.length())?;
        f.corrupt(&digest, &other);
        let mut fetched = fine(
            "a manifest a hostile upstream serves",
            f.upstream()
                .manifest("u4", &Reference::Digest(digest), deadline(f)?)
                .await,
        )?;
        let refusal = match read_all(f, &mut fetched).await? {
            Ok(_) => return Err("a body that is not its digest's ended as a whole body".into()),
            Err((_, refusal)) => refusal,
        };
        same(
            "its end",
            refusal,
            Refusal::Mismatch {
                want: digest,
                got: Sha256::of(&other),
            },
        )
    })
}

fn u5<F: UpstreamFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        let bytes = content("u5", f.length())?;
        let digest = f.push("u5", None, LAYER, &bytes);
        let refusal = refused(
            "a manifest under a tag the upstream lacks",
            f.upstream()
                .manifest("u5", &Reference::Tag("missing".into()), deadline(f)?)
                .await,
        )?;
        ensure(matches!(refusal, Refusal::NotFound(_)), || {
            format!("refused {refusal:?}, not NotFound")
        })?;
        let refusal = refused(
            "a blob from a repository that does not hold it",
            f.upstream()
                .blob("u5-elsewhere", &digest, None, deadline(f)?)
                .await,
        )?;
        ensure(matches!(refusal, Refusal::NotFound(_)), || {
            format!("refused {refusal:?}, not NotFound")
        })
    })
}

fn u6<F: UpstreamFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        let bytes = content("u6", f.length())?;
        f.push("u6", Some("latest"), MANIFEST, &bytes);
        f.limit(0, Some(f.patience()));
        let before = f.attempts(Op::Manifest);
        same(
            "a manifest past the upstream's limit",
            f.upstream()
                .manifest("u6", &Reference::Tag("latest".into()), deadline(f)?)
                .await
                .map(|fetched| fetched.described),
            Err(Refusal::TooMany {
                retry_after: Some(f.patience()),
            }),
        )?;
        same(
            "attempts of it",
            f.attempts(Op::Manifest),
            before.saturating_add(1),
        )
    })
}

fn u7<F: UpstreamFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        let bytes = content("u7", f.length())?;
        let digest = f.push("u7", None, LAYER, &bytes);
        for cut in 0..=f.length() {
            f.arm(Op::Read, Fault::Cut(cut))?;
            let mut fetched = fine(
                "a blob",
                f.upstream().blob("u7", &digest, None, deadline(f)?).await,
            )?;
            let (got, refusal) = match read_all(f, &mut fetched).await? {
                Err(ended) => ended,
                Ok(_) => return Err(format!("a body cut at {cut} was not cut")),
            };
            same("its refusal", refusal, Refusal::Cut { at: cut })?;
            let before = usize::try_from(cut).map_err(|e| e.to_string())?;
            ensure(bytes.get(..before) == Some(got.as_slice()), || {
                format!("cut at {cut}, the bytes before it are not the blob's")
            })?;
            if cut < f.length() {
                let rest = Range::From {
                    first: cut,
                    last: None,
                };
                let fetched = fine(
                    "the rest",
                    f.upstream()
                        .blob("u7", &digest, Some(rest), deadline(f)?)
                        .await,
                )?;
                let tail = whole(f, fetched).await?;
                ensure(bytes.get(before..) == Some(tail.as_slice()), || {
                    format!("cut at {cut}, the bytes from it are not the blob's")
                })?;
            }
        }
        Ok(())
    })
}

fn u8_<F: UpstreamFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        let bytes = content("u8", f.length())?;
        let digest = f.push("u8", None, LAYER, &bytes);
        let past = f
            .patience()
            .checked_add(f.patience())
            .ok_or_else(|| "a patience as long as time".to_owned())?;
        for fault in [Fault::Delay(past), Fault::Vanish] {
            f.arm(Op::Blob, fault.clone())?;
            let due = deadline(f)?;
            same(
                &format!("a blob {fault:?}"),
                f.upstream()
                    .blob("u8", &digest, None, due)
                    .await
                    .map(|fetched| fetched.span),
                Err(Refusal::Late),
            )?;
            ensure(f.clock().now() >= due, || {
                "told late before its deadline".into()
            })?;
        }
        f.arm(Op::Head, Fault::Delay(f.patience()))?;
        fine(
            "a head answered at its deadline",
            f.upstream()
                .head("u8", &Reference::Digest(digest), deadline(f)?)
                .await,
        )
        .map(drop)
    })
}

fn u9<F: UpstreamFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        let bytes = content("u9", f.length())?;
        f.push("u9", Some("latest"), MANIFEST, &bytes);
        f.limit(f.length(), None);
        f.arm(Op::Manifest, Fault::Lose)?;
        same(
            "a manifest whose answer is lost",
            f.upstream()
                .manifest("u9", &Reference::Tag("latest".into()), deadline(f)?)
                .await
                .map(|fetched| fetched.described),
            Err(Refusal::Lost),
        )?;
        same(
            "the budget after it",
            f.budget(),
            Some(f.length().saturating_sub(1)),
        )
    })
}

/// One refusal of every kind an upstream gives.
fn every_refusal<F: UpstreamFixture>(f: &F) -> Vec<Refusal> {
    let digest = Sha256::of(&[]);
    vec![
        Refusal::NotFound("n".into()),
        Refusal::Unauthorized("u".into()),
        Refusal::Denied("d".into()),
        Refusal::TooMany {
            retry_after: Some(f.patience()),
        },
        Refusal::Unsatisfiable { length: f.length() },
        Refusal::Invalid {
            what: "a reference",
            why: "w".into(),
        },
        Refusal::Mismatch {
            want: digest,
            got: digest,
        },
        Refusal::Cut { at: f.length() },
        Refusal::Late,
        Refusal::Lost,
        Refusal::Unavailable("u".into()),
    ]
}

fn u10<F: UpstreamFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        let bytes = content("u10", f.length())?;
        let digest = f.push("u10", None, LAYER, &bytes);
        for refusal in every_refusal(f) {
            let before = f.attempts(Op::Head);
            f.arm(Op::Head, Fault::Refuse(refusal.clone()))?;
            same(
                "a head the upstream refuses",
                f.upstream()
                    .head("u10", &Reference::Digest(digest), deadline(f)?)
                    .await,
                Err(refusal.clone()),
            )?;
            same(
                &format!("attempts of a head refused {refusal:?}"),
                f.attempts(Op::Head),
                before.saturating_add(1),
            )?;
        }
        Ok(())
    })
}

/// The `Upstream` contract, clause by clause (`docs/design/seams.md`, "Upstream").
#[must_use]
pub fn cases<F: UpstreamFixture>() -> Vec<Case<F>> {
    vec![
        Case {
            id: "UPSTREAM-1",
            says: "a manifest by tag or by digest gives its digest, its media type and its bytes, which hash to its digest",
            check: u1::<F>,
        },
        Case {
            id: "UPSTREAM-2",
            says: "a manifest's head gives its facts without its body and costs none of the upstream's budget; its get costs one",
            check: u2::<F>,
        },
        Case {
            id: "UPSTREAM-3",
            says: "a blob by digest gives its bytes in pieces into its caller's buffer, and a range is answered as RFC 9110 has it",
            check: u3::<F>,
        },
        Case {
            id: "UPSTREAM-4",
            says: "a body that does not hash to the digest it was asked by ends refused Mismatch, never as a whole body",
            check: u4::<F>,
        },
        Case {
            id: "UPSTREAM-5",
            says: "what the upstream does not hold is refused NotFound, a blob in a repository that does not hold it included",
            check: u5::<F>,
        },
        Case {
            id: "UPSTREAM-6",
            says: "the upstream's limit reached is refused TooMany with its Retry-After, after one attempt: the seam never asks again",
            check: u6::<F>,
        },
        Case {
            id: "UPSTREAM-7",
            says: "a body cut at any byte gives every byte before the cut, then is refused Cut there; a range from there gives the rest",
            check: u7::<F>,
        },
        Case {
            id: "UPSTREAM-8",
            says: "an answer delayed past its caller's deadline, or a request that never arrives, is refused Late at the deadline",
            check: u8_::<F>,
        },
        Case {
            id: "UPSTREAM-9",
            says: "a manifest whose answer is lost still cost the upstream's budget: the budget it states is one lower",
            check: u9::<F>,
        },
        Case {
            id: "UPSTREAM-10",
            says: "every refusal the upstream gives reaches its caller as it is, after one attempt: the seam never asks again",
            check: u10::<F>,
        },
    ]
}
