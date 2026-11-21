//! What travels in the world: the events hyper-sim's queue holds ([`Ev`]) and the messages its
//! network carries between processes ([`Msg`]), with a node's reply to a client ([`Reply`]).

use std::sync::Arc;
use std::time::Duration;

use hyper_sim::NodeId;
use hyper_sim::net::Ticket;
use reliquary_api::answer::{Answer, Body};
use reliquary_api::grammar::Digest;
use reliquary_api::request::{Op, Request};
use reliquary_core::seams::bytes::{Grant, Refusal as BytesRefusal};

use crate::call::{Answered, Call};

/// A call's number, which its node gives it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CallId(pub u64);

/// A request in flight on a node, by its node's number for it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct JobId(pub u64);

/// An event of the world.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ev {
    /// A message arrives.
    Arrive(Ticket),
    /// The nemesis's action of this number is due.
    Nemesis(usize),
    /// The machine of this number comes back after a crash or a power cut.
    Restart(usize),
    /// The partition the nemesis's action of this number made heals.
    Heal(usize),
}

/// What a node replies to a request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reply {
    /// The registry's answer.
    Answer(Answer),
    /// A blob's bytes are to be read at the node's bytes store with this grant (architecture D4).
    Grant(Offer),
    /// Refused, typed: nothing more is said by the registry's own answers.
    Refused(Refused),
}

/// A grant for a blob, and what the blob is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Offer {
    pub grant: Grant,
    pub digest: Digest,
    pub size: u64,
}

/// A refusal of a node's own, outside the registry's answers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Refused {
    pub kind: RefusedKind,
    pub why: String,
}

/// What a refusal says of the request's effect.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RefusedKind {
    /// Too much is in flight: nothing applied, and the request can be sent again.
    Busy,
    /// A seam could not be reached or refused: nothing applied.
    Unavailable,
    /// The request may have applied, and the node could not learn whether: a client records it as
    /// an answer lost (docs/design/model.md, H12).
    Unknown,
}

impl RefusedKind {
    /// Its name in the event log.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Busy => "Busy",
            Self::Unavailable => "Unavailable",
            Self::Unknown => "Unknown",
        }
    }
}

/// A message between processes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Msg {
    /// A client's request: attempt `attempt` of its operation `op`.
    Request {
        op: u64,
        attempt: u32,
        request: Request,
    },
    /// A node's reply to it, and the bytes store a grant in it is redeemed at.
    Reply {
        op: u64,
        attempt: u32,
        reply: Reply,
        store: NodeId,
    },
    /// A node's call on a store, by its incarnation, with what is left of the caller's patience.
    Call {
        id: CallId,
        incarnation: u32,
        call: Call,
        remaining: Duration,
    },
    /// A store's answer to it.
    Answer {
        id: CallId,
        incarnation: u32,
        answer: Answered,
    },
    /// A client redeeming a grant at a bytes store, Mantle's side of a grant.
    Redeem {
        op: u64,
        attempt: u32,
        grant: Grant,
        holder: String,
        remaining: Duration,
    },
    /// What the redemption gave: the object's bytes, or a refusal.
    Redeemed {
        op: u64,
        attempt: u32,
        content: Result<Arc<[u8]>, BytesRefusal>,
    },
}

/// The bytes a request carries.
#[must_use]
pub fn request_bytes(request: &Request) -> usize {
    match &request.op {
        Op::PostBlob { body, .. } | Op::PatchUpload { body, .. } => body.len(),
        Op::PutUpload { chunk, .. } => chunk.as_ref().map_or(0, |chunk| chunk.body.len()),
        Op::PutManifest { manifest, .. } => manifest.bytes.len(),
        _ => 0,
    }
}

impl Msg {
    /// The bytes it carries, as the network counts them.
    #[must_use]
    pub fn carried(&self) -> usize {
        match self {
            Self::Request { request, .. } => request_bytes(request),
            Self::Reply {
                reply: Reply::Answer(answer),
                ..
            } => match &answer.body {
                Body::Blob {
                    content: Some(content),
                    ..
                }
                | Body::Manifest {
                    content: Some(content),
                    ..
                }
                | Body::Part { content, .. } => content.len(),
                _ => 0,
            },
            Self::Call { call, .. } => call.carried(),
            Self::Answer {
                answer: Answered::Bytes(crate::call::BytesAnswer::Read(Ok(bytes))),
                ..
            } => bytes.len(),
            Self::Redeemed {
                content: Ok(bytes), ..
            } => bytes.len(),
            _ => 0,
        }
    }
}
