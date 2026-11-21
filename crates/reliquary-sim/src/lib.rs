#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing,
        clippy::arithmetic_side_effects,
        clippy::disallowed_macros
    )
)]
//! Reliquary's simulated world (plan RQ0.6; docs/design/sim.md): nodes that run a core against the
//! simulated seams, clients that run histories, a network that delays, drops, duplicates, reorders
//! and partitions, stores that lose power between any two writes, clocks that drift and jump, and a
//! nemesis that schedules faults. Every run is a function of its seed.
//!
//! - [`world`]: the machines, their processes and the run's loop, on hyper-sim's world and network.
//! - [`node`]: the [`node::Core`] a node runs, and its shell; [`call`], a core's seam calls as data.
//! - [`store`]: the records and bytes stores, RQ0.4's simulators behind the network.
//! - [`client`]: clients that run scripts and record every attempt.
//! - [`nemesis`]: the faults, written out or drawn, and the calm after them.
//! - [`verdict`]: a run's history against the reference model, its liveness, and its leases and
//!   waits; [`sweep`], power cut before and after every write.
//! - [`model_node`]: the reference model run as a node over the seams, which the world's nodes run
//!   until the registry's own cores land (plan RQ1); [`entry`], its log's entries.
//! - [`clock`], [`log`], [`ledger`], [`wire`]: each process's clocks, the event log, what a run
//!   tallies, and what travels.

pub mod call;
pub mod client;
pub mod clock;
mod ctx;
pub mod entry;
pub mod ledger;
pub mod log;
pub mod model_node;
pub mod nemesis;
pub mod node;
pub mod store;
pub mod sweep;
pub mod verdict;
pub mod wire;
pub mod world;
