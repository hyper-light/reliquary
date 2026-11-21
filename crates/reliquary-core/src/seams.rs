//! The seams Reliquary reaches other products through (plan RQ0.4; CLAUDE.md §9): its records
//! through Amphora's `Records` ([`records`]), its events through Quiver's `Topics` ([`topics`]),
//! and, where no product's `-api` crate gives one yet, traits of its own: [`bytes::Bytes`] (Mantle),
//! [`secrets::Secrets`] (Veil), [`upstream::Upstream`] (another registry), [`clock::Clock`] and
//! [`entropy::Entropy`].
//!
//! Each trait is sans-IO: it names no socket, file, runtime or clock of its own. Time comes into a
//! call as a deadline its caller derives from a [`clock::Clock`], and randomness from an
//! [`entropy::Entropy`]; no method takes a timeout as a number. A body moves as bounded pieces
//! into and out of its caller's own buffer, so no trait carries a whole object in memory.
//!
//! Each simulator is in memory and deterministic, and bounded by what its maker states: past a
//! bound it refuses, typed. Each can be made to inject every fault its trait can meet ([`fault`]):
//! each typed refusal, an answer delayed past its caller's deadline, a request that never arrives,
//! an answer lost after its effect applied, and a stream cut at any byte. Each counts what it
//! injected, so a suite run shows that every kind was exercised.
//!
//! Each conformance suite ([`conformance`]) is a list of cases, one per clause of the trait's
//! contract in `docs/design/seams.md`, run against a fresh fixture apiece. The simulator passes
//! it now; the real implementation passes the same suite when it lands.

use std::future::Future;
use std::pin::Pin;

pub mod bytes;
pub mod clock;
pub mod conformance;
pub mod entropy;
pub mod fault;
pub mod range;
pub mod records;
pub mod secrets;
pub mod topics;
pub mod upstream;

/// A future a seam's method answers with: boxed, sendable, and borrowing what the call borrows.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;
