//! The nemesis (plan RQ0.6): each fault it can inject, written out, takes effect where it says
//! (AC-RQ0.6.2); and a schedule drawn from the run's own stream within an envelope injects every
//! kind, each counted above zero, while the run stays one the model allows and live once the faults
//! stop (T-RQ0.6.7).

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

use std::time::Duration;

use hyper_sim::net::{Loss, Path};
use reliquary_api::check::Budget;
use reliquary_api::generate::{self, Shape as PlanShape};
use reliquary_api::harness;
use reliquary_sim::ledger::Side;
use reliquary_sim::nemesis::{self, Envelope, Fault, Schedule};
use reliquary_sim::verdict;
use reliquary_sim::world::End;

use support::{MS, Shape, clients, clients_at_once, mix, run, spec};

const SEED: u64 = 0x5eed_0607;

/// A setup of 20 requests and three clients of 80 each: work that, with no faults, lasts past the
/// window the drawn faults fall in.
fn plan(index: u64) -> (reliquary_api::model::Config, reliquary_api::history::Plan) {
    generate::concurrent(
        harness::history_seed(SEED, index),
        &mix(),
        PlanShape {
            clients: 3,
            setup: 20,
            per_client: 80,
            lose_one_in: 0,
        },
    )
    .unwrap()
}

/// A fleet of two nodes under one fault of every kind, written out: the records store and a node
/// partitioned from the bytes store and the other node for 30 ms; two of the records store's
/// answers lost; a link slowed; the bytes store's disk slowed; its clock stepped back a second; a
/// node's machine crashed; and the records store's power cut after a write and, once that node
/// is back, its power cut before one. Every client works from the start, so every armed fault lands at its machine's next write
/// or answer. Each takes
/// effect (counted where it lands, messages cut by the partition, the bytes store's wall clock a
/// second behind its monotonic one), and the run is one the model allows and live once they stop
/// (AC-RQ0.6.2).
#[test]
fn the_nemesis_crashes_cuts_power_partitions_slows_jumps_and_loses_answers() {
    let shape = Shape::Fleet(2);
    let (records, bytes) = shape.stores();
    let nodes = shape.nodes();
    let (config, plan) = plan(0);
    let second = Duration::from_secs(1);
    // Faults that land at the next answer or write come while every client is at work; those
    // that stall a client (a crash, a power cut) come last. The calm is late enough for each to
    // land however long the work takes.
    let nemesis = Schedule {
        faults: vec![
            (
                MS * 5,
                Fault::Partition {
                    a: vec![records, nodes[0]],
                    b: vec![bytes, nodes[1]],
                    lasts: Some(MS * 30),
                },
            ),
            (MS * 40, Fault::LoseAnswer { machine: records }),
            (MS * 41, Fault::LoseAnswer { machine: records }),
            (
                MS * 45,
                Fault::SlowLink {
                    from: nodes[1],
                    to: records,
                    path: Path::in_order(5_000_000, 1_000_000),
                },
            ),
            (
                MS * 47,
                Fault::SlowDisk {
                    machine: bytes,
                    latency: MS * 3,
                },
            ),
            (
                MS * 50,
                Fault::JumpClock {
                    machine: bytes,
                    by_ns: -i64::try_from(second.as_nanos()).unwrap(),
                },
            ),
            (MS * 55, Fault::Crash { machine: nodes[0] }),
            (
                MS * 75,
                Fault::PowerCut {
                    machine: records,
                    side: Side::After,
                },
            ),
            (
                MS * 77,
                Fault::PowerCut {
                    machine: nodes[0],
                    side: Side::Before,
                },
            ),
        ],
        calm: MS * 150,
    };
    let spec = spec(&config, shape, clients_at_once(&plan, shape), nemesis, None);
    let outcome = run(&spec, SEED);
    assert_eq!(outcome.end, End::Finished, "{:#?}", outcome.tail);
    let count = |kind: &str| outcome.faults.get(kind).copied().unwrap_or(0);
    assert_eq!(count(nemesis::CRASH), 1);
    assert_eq!(count(nemesis::POWER_CUT), 2, "{:?}", outcome.faults);
    assert_eq!(count(nemesis::PARTITION), 1);
    assert_eq!(count(nemesis::SLOW_LINK), 1);
    assert_eq!(count(nemesis::SLOW_DISK), 1);
    assert_eq!(count(nemesis::JUMP_CLOCK), 1);
    assert_eq!(count(nemesis::LOSE_ANSWER), 2, "{:?}", outcome.faults);
    assert!(outcome.unlanded.is_empty(), "{:?}", outcome.unlanded);
    assert!(outcome.net.dropped_partition > 0, "{:?}", outcome.net);
    // A crashed machine refuses connections and resets those open; a machine with no power drops
    // what reaches it.
    assert!(outcome.attempts.iter().any(|a| matches!(
        &a.ended,
        reliquary_sim::client::Ended::Refused { refused, .. } if refused.why.starts_with("the connection was")
    )));
    assert!(
        outcome
            .let_go
            .get("to a machine with no power")
            .copied()
            .unwrap_or(0)
            > 0,
        "{:?}",
        outcome.let_go
    );
    // The bytes store's wall clock is a second behind where its monotonic clock has taken it.
    let offset = |name: &str| {
        let (_, monotonic, wall) = outcome.clocks.iter().find(|(n, ..)| n == name).unwrap();
        i128::from(*wall) - i128::from(*monotonic)
    };
    assert_eq!(
        offset(&format!("records{records}")) - offset(&format!("bytes{bytes}")),
        i128::try_from(second.as_nanos()).unwrap()
    );
    let verdict = verdict::judge(&outcome, &config, Budget(20_000_000));
    assert!(verdict.passed(), "{verdict}");
}

/// Runs of a three-node fleet under faults drawn from each run's own stream, two of every kind
/// within 100 ms, among the servers (power cuts and lost answers among the stores, which write and
/// answer while any client works), each partition lasting 20 ms: every kind lands and is counted
/// above zero in every run, and each run is one the model allows and live once the faults stop
/// (T-RQ0.6.7).
#[test]
fn a_drawn_nemesis_injects_every_kind_and_counts_each() {
    let shape = Shape::Fleet(3);
    for index in 0..4 {
        let (config, plan) = plan(index);
        let mut spec = spec(
            &config,
            shape,
            clients(&plan, shape),
            Schedule::default(),
            None,
        );
        let (records, bytes) = shape.stores();
        spec.drawn = Some(Envelope {
            machines: shape.servers(),
            stores: vec![records, bytes],
            each: 2,
            calm: MS * 100,
            slow_path: Path::in_order(2_000_000, 1_000_000),
            latency: MS * 2,
            jump_ns: 1_000_000_000,
            lasts: MS * 20,
        });
        let outcome = run(&spec, harness::history_seed(SEED ^ 0xd, index));
        assert_eq!(outcome.end, End::Finished, "{index}: {:#?}", outcome.tail);
        assert!(
            outcome.unlanded.is_empty(),
            "run {index}: {:?}",
            outcome.unlanded
        );
        for kind in nemesis::KINDS {
            assert!(
                outcome.faults.get(kind).copied().unwrap_or(0) > 0,
                "run {index}: no {kind}: {:?}",
                outcome.faults
            );
        }
        let verdict = verdict::judge(&outcome, &config, Budget(20_000_000));
        assert!(verdict.passed(), "run {index}: {verdict}");
    }
}

/// A fleet of two nodes whose every link between machines delays each message 0.5 ms ± 0.4 ms,
/// letting later messages overtake earlier ones and losing one in twenty, for the first 200 ms, while
/// every link delivers one message in twenty twice: the network dropped, duplicated and reordered
/// messages (reorders read from the event log, a later send arriving before an earlier one on one
/// link), and the run is one the model allows and live once the faults stop (AC-RQ0.6.2, T-RQ0.6.7).
#[test]
fn a_network_that_delays_drops_duplicates_and_reorders_leaves_the_history_right() {
    let shape = Shape::Fleet(2);
    let (config, plan) = plan(1);
    let servers = shape.servers();
    let rough = Path::reordering(500_000, 400_000).with_loss(Loss::random(50_000));
    let mut faults = Vec::new();
    for from in &servers {
        for to in &servers {
            if from != to {
                faults.push((
                    Duration::ZERO,
                    Fault::SlowLink {
                        from: *from,
                        to: *to,
                        path: rough,
                    },
                ));
            }
        }
    }
    let nemesis = Schedule {
        faults,
        calm: MS * 200,
    };
    let mut spec = spec(&config, shape, clients_at_once(&plan, shape), nemesis, None);
    spec.duplicate_ppm = 50_000;
    spec.keep = 1 << 20;
    let outcome = run(&spec, SEED ^ 0x0e7);
    assert_eq!(outcome.end, End::Finished, "{:#?}", outcome.tail);
    assert!(outcome.net.dropped_loss > 0, "{:?}", outcome.net);
    assert!(outcome.net.duplicated > 0, "{:?}", outcome.net);
    // Each line `T P send Q … arrives A`: on one link, a later send arriving first is a reorder.
    let mut last: std::collections::BTreeMap<(String, String), u64> =
        std::collections::BTreeMap::new();
    let mut reordered = 0;
    for line in &outcome.tail {
        let words: Vec<&str> = line.split(' ').collect();
        if words.get(2) != Some(&"send") || words.get(words.len() - 2) != Some(&"arrives") {
            continue;
        }
        let at: u64 = words[words.len() - 1].parse().unwrap();
        let link = (words[1].to_owned(), words[3].to_owned());
        if last.get(&link).is_some_and(|before| at < *before) {
            reordered += 1;
        }
        last.insert(link, at);
    }
    assert!(reordered > 0, "no message overtook another");
    let verdict = verdict::judge(&outcome, &config, Budget(20_000_000));
    assert!(verdict.passed(), "{verdict}");
}
