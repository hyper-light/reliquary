//! A core's calls on its seams, as data (docs/design/sim.md, W2): what a node asks of its records
//! and its bytes, and what it is answered. A core never calls a seam itself; it names the call, its
//! shell makes it (in the world: a message to the store's process, answered by the store's
//! simulator through the seam's own trait), and the answer comes back as an event. So every call is
//! a point where the world can run anything else, and a crash can fall between any two.
//!
//! The types inside are the seams' own: Amphora's `Tx`, `Scan` and `Refusal`, and the `Bytes` seam's
//! `Put`, `Whole`, `Grant` and refusal (CLAUDE.md §9: other products are reached through their own
//! `-api` crates; docs/design/seams.md S11).

use std::sync::Arc;
use std::time::SystemTime;

use amphora_api::{Consistency, Page, Refusal as RecordsRefusal, Rev, Row, Scan, Tx};
use reliquary_core::seams::bytes::{
    Access, Deleted, Grant, Meta, Progress, Put, Refusal as BytesRefusal, Upload, UploadId, Whole,
};
use reliquary_core::seams::range::{Range, Span};

/// A reading a bytes store keeps open for a caller, by the store's number for it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReadingId(pub u64);

/// A call on a seam.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Call {
    Records(RecordsCall),
    Bytes(BytesCall),
}

/// A call on `Records` (amphora-api), one variant for each method a core makes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecordsCall {
    Get {
        family: String,
        key: Vec<u8>,
        at: Consistency,
    },
    Scan {
        family: String,
        scan: Scan,
    },
    Commit {
        tx: Tx,
    },
    Head,
}

/// A call on `Bytes`, one variant for each method, and a reading's reads by the store's number for
/// the reading.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BytesCall {
    Begin {
        key: String,
        put: Put,
    },
    Send {
        upload: UploadId,
        at: u64,
        piece: Arc<[u8]>,
    },
    Status {
        upload: UploadId,
    },
    Finish {
        upload: UploadId,
        whole: Whole,
    },
    Abort {
        upload: UploadId,
    },
    /// Opens a reading, which the store keeps until it is read to its end or closed.
    Get {
        key: String,
        range: Option<Range>,
    },
    /// The next bytes of a reading, at most `length`.
    Read {
        reading: ReadingId,
        length: usize,
    },
    /// Lets a reading go before its end.
    Close {
        reading: ReadingId,
    },
    Head {
        key: String,
    },
    Delete {
        key: String,
    },
    Grant {
        key: String,
        access: Access,
        holder: String,
        expires: SystemTime,
    },
}

/// What a call was answered.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Answered {
    Records(RecordsAnswer),
    Bytes(BytesAnswer),
}

/// What a records call was answered.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecordsAnswer {
    Row(Result<Option<Row>, RecordsRefusal>),
    Page(Result<Page, RecordsRefusal>),
    Rev(Result<Rev, RecordsRefusal>),
}

/// A reading as opened: the object's facts, what the range selected, and the store's number for it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Opened {
    pub meta: Meta,
    pub span: Span,
    pub reading: ReadingId,
}

/// What a bytes call was answered.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BytesAnswer {
    Upload(Result<Upload, BytesRefusal>),
    Progress(Result<Progress, BytesRefusal>),
    Meta(Result<Meta, BytesRefusal>),
    Done(Result<(), BytesRefusal>),
    Opened(Result<Opened, BytesRefusal>),
    Read(Result<Arc<[u8]>, BytesRefusal>),
    Deleted(Result<Deleted, BytesRefusal>),
    Grant(Result<Grant, BytesRefusal>),
}

/// The seam a call is on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Seam {
    Records,
    Bytes,
}

impl Call {
    /// The seam it is on.
    #[must_use]
    pub const fn seam(&self) -> Seam {
        match self {
            Self::Records(_) => Seam::Records,
            Self::Bytes(_) => Seam::Bytes,
        }
    }

    /// Its method's name, as the event log and the write ledger say it.
    #[must_use]
    pub const fn method(&self) -> &'static str {
        match self {
            Self::Records(call) => match call {
                RecordsCall::Get { .. } => "records.get",
                RecordsCall::Scan { .. } => "records.scan",
                RecordsCall::Commit { .. } => "records.commit",
                RecordsCall::Head => "records.head",
            },
            Self::Bytes(call) => match call {
                BytesCall::Begin { .. } => "bytes.begin",
                BytesCall::Send { .. } => "bytes.send",
                BytesCall::Status { .. } => "bytes.status",
                BytesCall::Finish { .. } => "bytes.finish",
                BytesCall::Abort { .. } => "bytes.abort",
                BytesCall::Get { .. } => "bytes.get",
                BytesCall::Read { .. } => "bytes.read",
                BytesCall::Close { .. } => "bytes.close",
                BytesCall::Head { .. } => "bytes.head",
                BytesCall::Delete { .. } => "bytes.delete",
                BytesCall::Grant { .. } => "bytes.grant",
            },
        }
    }

    /// Whether it writes what its store keeps: a commit, and every call that makes, changes or
    /// removes an upload or an object. These are the writes a crash sweep cuts power before and
    /// after (plan AC-RQ0.6.3).
    #[must_use]
    pub const fn writes(&self) -> bool {
        matches!(
            self,
            Self::Records(RecordsCall::Commit { .. })
                | Self::Bytes(
                    BytesCall::Begin { .. }
                        | BytesCall::Send { .. }
                        | BytesCall::Finish { .. }
                        | BytesCall::Abort { .. }
                        | BytesCall::Delete { .. }
                )
        )
    }

    /// The bytes it carries, which the network counts.
    #[must_use]
    pub fn carried(&self) -> usize {
        match self {
            Self::Bytes(BytesCall::Send { piece, .. }) => piece.len(),
            _ => 0,
        }
    }

    /// The answer a caller is given when its deadline passes first: Amphora's `Timeout`, or the
    /// bytes seam's `Late`. Either says the call may have applied (docs/design/seams.md S7).
    #[must_use]
    pub fn late(&self) -> Answered {
        match self {
            Self::Records(call) => {
                Answered::Records(records_refused(call, RecordsRefusal::Timeout))
            }
            Self::Bytes(call) => Answered::Bytes(bytes_refused(call, BytesRefusal::Late)),
        }
    }
}

impl Call {
    /// The answer a caller is given when the connection its call went on is reset by its store's
    /// crash: Amphora's `Unavailable`, the bytes seam's `Lost`. Either says the call may have
    /// applied.
    #[must_use]
    pub fn reset(&self) -> Answered {
        match self {
            Self::Records(call) => Answered::Records(records_refused(
                call,
                RecordsRefusal::Unavailable("the connection was reset".to_owned()),
            )),
            Self::Bytes(call) => Answered::Bytes(bytes_refused(call, BytesRefusal::Lost)),
        }
    }

    /// The answer a caller is given when its store's machine refuses the connection: the call
    /// never reached the store.
    #[must_use]
    pub fn refused(&self) -> Answered {
        let why = || "the connection was refused".to_owned();
        match self {
            Self::Records(call) => {
                Answered::Records(records_refused(call, RecordsRefusal::Unavailable(why())))
            }
            Self::Bytes(call) => {
                Answered::Bytes(bytes_refused(call, BytesRefusal::Unavailable(why())))
            }
        }
    }
}

/// `refusal` as the answer to `call`.
#[must_use]
pub fn records_refused(call: &RecordsCall, refusal: RecordsRefusal) -> RecordsAnswer {
    match call {
        RecordsCall::Get { .. } => RecordsAnswer::Row(Err(refusal)),
        RecordsCall::Scan { .. } => RecordsAnswer::Page(Err(refusal)),
        RecordsCall::Commit { .. } | RecordsCall::Head => RecordsAnswer::Rev(Err(refusal)),
    }
}

/// `refusal` as the answer to `call`.
#[must_use]
pub fn bytes_refused(call: &BytesCall, refusal: BytesRefusal) -> BytesAnswer {
    match call {
        BytesCall::Begin { .. } => BytesAnswer::Upload(Err(refusal)),
        BytesCall::Send { .. } | BytesCall::Status { .. } => BytesAnswer::Progress(Err(refusal)),
        BytesCall::Finish { .. } | BytesCall::Head { .. } => BytesAnswer::Meta(Err(refusal)),
        BytesCall::Abort { .. } | BytesCall::Close { .. } => BytesAnswer::Done(Err(refusal)),
        BytesCall::Get { .. } => BytesAnswer::Opened(Err(refusal)),
        BytesCall::Read { .. } => BytesAnswer::Read(Err(refusal)),
        BytesCall::Delete { .. } => BytesAnswer::Deleted(Err(refusal)),
        BytesCall::Grant { .. } => BytesAnswer::Grant(Err(refusal)),
    }
}

impl Answered {
    /// Its words for the event log: the method's answer kind and, for a refusal, its kind.
    #[must_use]
    pub fn summary(&self) -> String {
        use reliquary_core::seams::fault::Kind;
        let refused = |kind: &str| format!("refused {kind}");
        match self {
            Self::Records(answer) => match answer {
                RecordsAnswer::Row(Ok(Some(row))) => format!("row at {}", row.rev),
                RecordsAnswer::Row(Ok(None)) => "no row".to_owned(),
                RecordsAnswer::Page(Ok(page)) => {
                    format!("page of {} rows, more {}", page.rows.len(), page.more)
                }
                RecordsAnswer::Rev(Ok(rev)) => format!("at {rev}"),
                RecordsAnswer::Row(Err(e))
                | RecordsAnswer::Page(Err(e))
                | RecordsAnswer::Rev(Err(e)) => refused(e.kind()),
            },
            Self::Bytes(answer) => match answer {
                BytesAnswer::Upload(Ok(upload)) => format!("upload {}", upload.id.0),
                BytesAnswer::Progress(Ok(p)) => {
                    format!("received {} durable {}", p.received, p.durable)
                }
                BytesAnswer::Meta(Ok(meta)) => format!("object {} of {}", meta.key, meta.length),
                BytesAnswer::Done(Ok(())) => "done".to_owned(),
                BytesAnswer::Opened(Ok(o)) => {
                    format!("reading {} of {}", o.reading.0, o.span.length)
                }
                BytesAnswer::Read(Ok(bytes)) => format!("{} bytes", bytes.len()),
                BytesAnswer::Deleted(Ok(d)) => format!("{d:?}"),
                BytesAnswer::Grant(Ok(g)) => format!("grant for {}", g.key),
                BytesAnswer::Upload(Err(e))
                | BytesAnswer::Progress(Err(e))
                | BytesAnswer::Meta(Err(e))
                | BytesAnswer::Done(Err(e))
                | BytesAnswer::Opened(Err(e))
                | BytesAnswer::Read(Err(e))
                | BytesAnswer::Deleted(Err(e))
                | BytesAnswer::Grant(Err(e)) => refused(e.kind()),
            },
        }
    }
}
