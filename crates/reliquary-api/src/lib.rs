//! Reliquary's API (architecture D24): what a request to Reliquary is and what it is answered, the
//! reference model that gives every request its one right answer, and the differential harness that
//! holds an implementation to the model (plan RQ0.5; docs/design/model.md).
//!
//! - [`request`] and [`answer`]: the distribution spec's endpoints as requests, and their statuses,
//!   error codes and what a client reads; [`answer::ANSWERS`] lists every answer the model gives.
//! - [`grammar`]: names, tags, digests, references and manifest media types, as the specs write them.
//! - [`model`]: the registry's meaning: a state, a pure decision for each request, and its
//!   invariants.
//! - [`generate`]: seeded histories, sequential and concurrent, valid and broken one way each.
//! - [`history`]: histories and the plans that make them, run against any [`harness::Registry`].
//! - [`check`]: linearizability against the model, on a stated budget.
//! - [`shrink`]: a failing input made 1-minimal.
//! - [`harness`]: the registry trait, coverage counters, comparisons, and hunts that report a seed
//!   to replay from.

pub mod answer;
pub mod check;
pub mod generate;
pub mod grammar;
pub mod harness;
pub mod history;
mod json;
pub mod model;
pub mod request;
pub mod shrink;
