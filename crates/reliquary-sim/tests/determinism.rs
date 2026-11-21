//! Every run is a function of its seed (plan RQ0.6): one seed run twice gives the same history, the
//! same event log and the same verdict, and run again from the first run's trace, without its
//! seed, the same digest (AC-RQ0.6.1); and the event log of one seed is the same bytes on every
//! target, held to the digest recorded below, which the suite checks on each platform it runs on
//! (T-RQ0.6.8).

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

mod support;

use hyper_sim::net::Path;
use hyper_sim::{Random, Source};
use reliquary_api::check::Budget;
use reliquary_api::generate::{self, Shape as PlanShape};
use reliquary_api::harness;
use reliquary_sim::model_node::ModelNode;
use reliquary_sim::nemesis::{Envelope, Schedule};
use reliquary_sim::verdict;
use reliquary_sim::world::{self, End, Spec};

use support::{MS, Shape, clients_at_once, mix, spec};

const SEED: u64 = 0x5eed_0608;

/// The run every test here makes: a three-node fleet, a generated plan of a setup and three clients
/// all at work at once, grants living 5 ms, a tenth of all deliveries made twice, and faults of
/// every kind drawn from the run's own stream.
fn the_run() -> (reliquary_api::model::Config, Spec<ModelNode>) {
    let shape = Shape::Fleet(3);
    let (config, plan) = generate::concurrent(
        harness::history_seed(SEED, 0),
        &mix(),
        PlanShape {
            clients: 3,
            setup: 15,
            per_client: 40,
            lose_one_in: 0,
        },
    )
    .unwrap();
    let mut spec = spec(
        &config,
        shape,
        clients_at_once(&plan, shape),
        Schedule::default(),
        Some(MS * 5),
    );
    spec.duplicate_ppm = 100_000;
    let (records, bytes) = shape.stores();
    spec.drawn = Some(Envelope {
        machines: shape.servers(),
        stores: vec![records, bytes],
        each: 1,
        calm: MS * 60,
        slow_path: Path::reordering(2_000_000, 1_500_000),
        latency: MS * 2,
        jump_ns: 1_000_000_000,
        lasts: MS * 15,
    });
    (config, spec)
}

/// One seed run twice gives the same digest of every decision, event and observation, the same
/// event log, the same attempts and the same verdict; and run from the first run's trace, without
/// its seed, the same digest again (AC-RQ0.6.1).
#[test]
fn one_seed_gives_one_history_one_log_and_one_verdict() {
    let (config, spec) = the_run();
    let first = support::run(&spec, SEED);
    let second = support::run(&spec, SEED);
    assert_eq!(first.end, End::Finished);
    assert_eq!(first.record, second.record);
    assert_eq!(first.log, second.log);
    assert_eq!(first.attempts, second.attempts);
    let v1 = verdict::judge(&first, &config, Budget(20_000_000));
    let v2 = verdict::judge(&second, &config, Budget(20_000_000));
    assert_eq!(v1, v2);
    assert!(v1.passed(), "{v1}");
    // hyper-sim's check: the seed twice and its trace once, every digest equal.
    let record = hyper_sim::twice(SEED, |source| {
        world::run(&spec, source, &mut Random).map(|outcome| outcome.record)
    })
    .unwrap();
    assert_eq!(record, first.record);
    let replayed = world::run(
        &spec,
        Source::Trace(first.record.trace.clone()),
        &mut Random,
    )
    .unwrap();
    assert_eq!(replayed.log, first.log);
    assert_eq!(replayed.attempts, first.attempts);
    // Another seed is another run.
    let other = support::run(&spec, SEED + 1);
    assert_ne!(other.log, first.log);
}

/// The event log of the run above, its verdict, and the digest of every decision, as this
/// repository's suite gives them on macOS arm64, Linux arm64 and x86_64 and Windows x86_64: every
/// target the suite runs on gives these bytes (T-RQ0.6.8). A change to what the world does changes
/// them; such a change records the new values here, from any one target, and the others must agree.
const LOG: &str =
    "sha256:b61d6f21fc6ed6fb9bbdeea7aad3e940495602ea1de7d29153c0de9edad2a9ba over 2133 lines";
const DIGEST: u64 = 7_139_588_381_552_911_525;

/// One seed's event log, verdict and digest on this target are the ones recorded (T-RQ0.6.8).
#[test]
fn one_seed_gives_the_same_event_log_on_every_target() {
    let (config, spec) = the_run();
    let outcome = support::run(&spec, SEED);
    let verdict = verdict::judge(&outcome, &config, Budget(20_000_000));
    assert!(verdict.passed(), "{verdict}");
    assert_eq!(
        (outcome.log.to_string().as_str(), outcome.record.digest.0),
        (LOG, DIGEST),
        "the event log or the digest differs from the one recorded on every target"
    );
}
