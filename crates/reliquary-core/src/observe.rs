//! What Reliquary tells an operator about itself (plan RQ0.9): its metrics, its events, and the
//! ring its events wait in for their sink when it runs on its own.
//!
//! - [`catalog`]: every metric and event, with what each means. `docs/observability.md` lists the
//!   same, and `tests/observe.rs` holds the two to each other (AC-RQ0.9.1, AC-RQ0.9.4) and holds
//!   each name to the conventions `docs/design/observability.md` decides (AC-RQ0.9.2).
//! - [`openmetrics`]: the metrics a component reports, as the OpenMetrics 1.0 text a scrape reads.
//! - [`ring`]: the event ring, which drops its oldest line rather than make an emitter wait
//!   (T-RQ0.9.5), and [`layer`], which feeds it what is emitted through `tracing` on Reliquary's
//!   targets.
//!
//! Embedded in kiln, kiln's subscriber takes Reliquary's events and its scrape reads the metrics;
//! the ring is for a process that has no such host.

pub mod catalog;
pub mod layer;
mod line;
pub mod openmetrics;
pub mod ring;
