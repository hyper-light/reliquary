//! `reliquary bench`: a measurement that states its intervals, compares two builds, gates instruction
//! counts in CI and finds where a nightly series changed (plan RQ0.7; the design is
//! `docs/design/measurement.md`, its sources `docs/research/07-measurement.md`).
//!
//! The pure cores, each sans-IO: [`rounds`] judges a point's units (Mantle's measurement design §2 to
//! §5), [`levels`] says how many builds, process starts and rounds a point needs (Kalibera and Jones),
//! [`ab`] orders and compares two arms, [`drive`] offers and times a round's operations, [`tails`] states
//! a state's latency quantiles, [`load`] holds a record to the load rule, and [`changes`] finds change
//! points. The shell runs them: [`worker`] is the process that measures, [`driver`] the one that plans,
//! starts workers, interleaves arms and writes the [`record`].

#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing,
        clippy::arithmetic_side_effects,
        clippy::disallowed_macros,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_possible_wrap
    )
)]

pub mod ab;
pub mod bench;
pub mod changes;
pub mod cli;
pub mod drive;
pub mod driver;
pub mod gate;
pub mod levels;
pub mod load;
pub mod protocol;
pub mod record;
pub mod report;
pub mod rng;
pub mod rounds;
pub mod tails;
pub mod worker;
