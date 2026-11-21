//! A node cut off from its records (plan RQ0.6): while the partition holds, the node refuses every
//! request typed, and once it heals every operation completes, the history one the model allows
//! (T-RQ0.6.3).

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

use reliquary_api::check::Budget;
use reliquary_sim::client::Ended;
use reliquary_sim::nemesis::{Fault, Schedule};
use reliquary_sim::verdict;
use reliquary_sim::wire::RefusedKind;
use reliquary_sim::world::End;

use support::{MS, Shape, client, content, owned, push, read_back, run, spec};

const SEED: u64 = 0x5eed_0603;

/// Three pushes and their reads against one node in a fleet, the node's machine cut off from the
/// records store's from 5 ms to 300 ms: every attempt the node answered while the partition held
/// was refused `Unavailable` or `Unknown`, never answered as if it had reached its records or left
/// unanswered; after the heal every operation completes on a later attempt, and the history,
/// refusals and all, is one the model allows (T-RQ0.6.3).
#[test]
fn a_partition_between_a_node_and_its_records_refuses_typed_then_completes_after_the_heal() {
    let shape = Shape::Fleet(1);
    let (records, _) = shape.stores();
    let node = shape.nodes()[0];
    let mut script = Vec::new();
    for (k, name) in (0u8..).zip(["one", "two", "three"]) {
        let layer = content(10 + k, 5000);
        let config = content(20 + k, 200);
        script.extend(push(name, 1 + u32::from(k), &layer, &config, "v1"));
        script.extend(read_back(name, &layer, &config, "v1"));
    }
    let config = owned(&["one", "two", "three"]);
    let cut = MS * 5;
    let calm = MS * 300;
    let nemesis = Schedule {
        faults: vec![(
            cut,
            Fault::Partition {
                a: vec![node],
                b: vec![records],
                lasts: None,
            },
        )],
        calm,
    };
    let ops = script.len();
    let spec = spec(
        &config,
        shape,
        vec![client(script, shape, 0, None)],
        nemesis,
        None,
    );
    let outcome = run(&spec, SEED);
    assert_eq!(outcome.end, End::Finished, "{:#?}", outcome.tail);
    assert!(outcome.net.dropped_partition > 0, "{:?}", outcome.net);
    let (from, to) = (nanos(cut), nanos(calm));
    let mut refused = 0;
    for a in &outcome.attempts {
        let at = match &a.ended {
            Ended::Answered { at, .. } | Ended::Refused { at, .. } | Ended::Lost { at } => *at,
        };
        if a.invoked < from || at > to {
            continue;
        }
        match &a.ended {
            Ended::Refused { refused: r, .. } => {
                assert!(
                    matches!(r.kind, RefusedKind::Unavailable | RefusedKind::Unknown),
                    "{a:?}"
                );
                refused += 1;
            }
            other => panic!(
                "op {}.{} ended {other:?} inside the partition",
                a.op, a.attempt
            ),
        }
    }
    assert!(refused > 0, "no attempt fell inside the partition");
    // Every operation completed, after the heal for those the partition refused.
    let completed: std::collections::BTreeSet<u64> = outcome
        .attempts
        .iter()
        .filter(|a| matches!(a.ended, Ended::Answered { .. }))
        .map(|a| a.op)
        .collect();
    assert_eq!(completed.len(), ops);
    let verdict = verdict::judge(&outcome, &config, Budget(1_000_000));
    assert!(verdict.passed(), "{verdict}");
}

fn nanos(d: Duration) -> u64 {
    u64::try_from(d.as_nanos()).unwrap()
}
