//! A history that takes every pair of resources in both orders (plan RQ0.6): two clients, each
//! naming two repositories, and two blobs, in the order the other names them reversed. The model
//! node takes no lock, so no run ends with work waiting and nothing moving; a node that locks what
//! a request names, a lock at a time, ends with two requests each holding what the other waits
//! for, which the run reports (T-RQ0.6.5).

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

mod mutants;
mod support;

use std::rc::Rc;

use reliquary_api::check::Budget;
use reliquary_api::request::{Op, Request, SessionTag};
use reliquary_sim::client::Ended;
use reliquary_sim::model_node::ModelNode;
use reliquary_sim::nemesis::Schedule;
use reliquary_sim::verdict::{self, Stuck};
use reliquary_sim::world::{End, Waiter};

use support::{
    Shape, budgets, client, content, digest, image, owned, request, run, spec, spec_with,
};

const SEED: u64 = 0x5eed_0605;

fn mount(name: &str, from: &str, blob: &[u8], session: u32) -> Request {
    request(Op::MountBlob {
        name: name.to_owned(),
        digest: digest(blob).to_string(),
        from: Some(from.to_owned()),
        session: SessionTag(session),
    })
}

fn manifest(name: &str, tag: &str, config: &[u8], layers: &[&[u8]]) -> Request {
    request(Op::PutManifest {
        name: name.to_owned(),
        reference: tag.to_owned(),
        manifest: image(config, layers),
    })
}

/// The setup (a config and blobs x and y in repositories a and b), and two scripts: what one names
/// as (b, a), (a, b), (x, y) and (y, x), the other names reversed, step for step.
fn crossing() -> (Vec<Request>, Vec<Request>, Vec<Request>) {
    let (x, y, c) = (content(1, 900), content(2, 1300), content(3, 120));
    let mut setup = Vec::new();
    for name in ["a", "b"] {
        for blob in [&x, &y, &c] {
            setup.push(request(Op::PostBlob {
                name: name.to_owned(),
                digest: digest(blob).to_string(),
                body: blob.clone().into(),
            }));
        }
    }
    let one = vec![
        mount("b", "a", &x, 101),
        mount("a", "b", &y, 102),
        manifest("a", "one", &c, &[&x, &y]),
        manifest("b", "two", &c, &[&y, &x]),
        manifest("a", "two", &c, &[&y, &x]),
        manifest("b", "one", &c, &[&x, &y]),
    ];
    let two = vec![
        mount("a", "b", &y, 201),
        mount("b", "a", &x, 202),
        manifest("a", "one", &c, &[&y, &x]),
        manifest("b", "two", &c, &[&x, &y]),
        manifest("a", "two", &c, &[&x, &y]),
        manifest("b", "one", &c, &[&y, &x]),
    ];
    (setup, one, two)
}

/// The crossing history on one node, on two nodes with a client each, and on one node a LAN away
/// from its stores: every operation completes, nothing waits at the end, and the history is one the
/// model allows (T-RQ0.6.5).
#[test]
fn a_history_taking_every_pair_of_resources_in_both_orders_ends_with_nothing_waiting() {
    let config = owned(&["a", "b"]);
    for shape in [Shape::Laptop, Shape::Fleet(2), Shape::Fleet(1)] {
        for seed in 0..6 {
            let (setup, one, two) = crossing();
            let clients = vec![
                client(setup, shape, 0, None),
                client(one, shape, 0, Some(0)),
                client(two, shape, 1, Some(0)),
            ];
            let spec = spec(&config, shape, clients, Schedule::default(), None);
            let outcome = run(&spec, SEED + seed);
            assert_eq!(outcome.end, End::Finished, "{shape:?}: {:#?}", outcome.tail);
            assert!(
                outcome.waiters.is_empty(),
                "{shape:?}: {:?}",
                outcome.waiters
            );
            assert!(
                outcome
                    .attempts
                    .iter()
                    .all(|a| matches!(a.ended, Ended::Answered { .. })),
                "{shape:?}"
            );
            let verdict = verdict::judge(&outcome, &config, Budget(10_000_000));
            assert!(verdict.passed(), "{shape:?}: {verdict}");
        }
    }
}

/// The crossing history against a node that locks what each request names, in the order it names
/// them, reading its records between one lock and the next: the two clients' first requests each
/// take one repository's lock and wait for the other's. The run ends with the node holding
/// requests it waits on nothing for, and with attempts made once the faults stopped (there were
/// none) that were never answered: a deadlock, found and named (T-RQ0.6.5).
#[test]
fn a_node_that_locks_in_request_order_deadlocks_and_the_run_names_its_waiters() {
    let config = owned(&["a", "b"]);
    let shape = Shape::Fleet(1);
    let mut found = 0;
    for seed in 0..6 {
        let (setup, one, two) = crossing();
        let clients = vec![
            client(setup, shape, 0, None),
            client(one, shape, 0, Some(0)),
            client(two, shape, 0, Some(0)),
        ];
        let node = config.clone();
        let spec = spec_with(
            shape,
            clients,
            Schedule::default(),
            Rc::new(move || {
                mutants::Locking::new(
                    ModelNode::new(node.clone(), budgets(None)),
                    budgets(None).wait,
                )
            }),
        );
        let outcome = run(&spec, SEED + seed);
        let verdict = verdict::judge(&outcome, &config, Budget(10_000_000));
        let held = verdict.stuck.iter().any(
            |s| matches!(s, Stuck::Waiting(Waiter::Node { jobs, calls: 0, .. }) if *jobs >= 2),
        );
        let lost = verdict
            .stuck
            .iter()
            .any(|s| matches!(s, Stuck::Lost { .. }));
        if held && lost {
            found += 1;
        }
    }
    assert!(found > 0, "no run of the locking node deadlocked");
}
