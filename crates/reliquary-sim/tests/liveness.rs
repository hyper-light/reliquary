//! The liveness check (plan RQ0.6): once the faults stop, every operation a client invokes
//! completes or is refused. A run under faults whose lost answers all fall while the faults last
//! passes; a run with a node that never answers one kind of request fails, naming the attempts it
//! left unanswered and the requests it holds with nothing to wait on (AC-RQ0.6.5).

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
use reliquary_api::request::{Op, Request};
use reliquary_sim::client::Ended;
use reliquary_sim::model_node::ModelNode;
use reliquary_sim::nemesis::{Fault, Schedule};
use reliquary_sim::verdict::{self, Stuck};
use reliquary_sim::world::{End, Waiter};

use support::{
    MS, Shape, budgets, client, content, owned, push, read_back, request, run, spec, spec_with,
};

const SEED: u64 = 0x5eed_0608;

/// A push, a version check, and the push's reads.
fn script() -> Vec<Request> {
    let layer = content(7, 2000);
    let config = content(8, 100);
    let mut script = push("app", 1, &layer, &config, "v1");
    script.push(request(Op::ApiVersion));
    script.extend(read_back("app", &layer, &config, "v1"));
    script
}

/// The node's machine loses power at its first write after 1 ms and is down 20 ms; the faults stop
/// at 50 ms. The attempt the power cut left unanswered was given up while the faults lasted, and
/// every attempt after them was answered: the run passes (AC-RQ0.6.5).
#[test]
fn answers_lost_while_the_faults_last_leave_the_run_live() {
    let shape = Shape::Fleet(1);
    let config = owned(&["app"]);
    let nemesis = Schedule {
        faults: vec![(
            MS,
            Fault::PowerCut {
                machine: shape.nodes()[0],
                side: reliquary_sim::ledger::Side::After,
            },
        )],
        calm: MS * 50,
    };
    let spec = spec(
        &config,
        shape,
        vec![client(script(), shape, 0, None)],
        nemesis,
        None,
    );
    let outcome = run(&spec, SEED);
    assert_eq!(outcome.end, End::Finished);
    let quiet = outcome.quiet_from.unwrap();
    assert!(
        outcome
            .attempts
            .iter()
            .any(|a| matches!(a.ended, Ended::Lost { .. }) && a.invoked < quiet)
    );
    let verdict = verdict::judge(&outcome, &config, Budget(1_000_000));
    assert!(verdict.passed(), "{verdict}");
}

/// The same script against a node that never answers a version check, with no faults at all: each
/// attempt at it waits out the client's patience unanswered, and the run fails, naming those
/// attempts, the operation they starved, and the node holding requests it waits on nothing for
/// (AC-RQ0.6.5).
#[test]
fn an_operation_neither_completed_nor_refused_once_the_faults_stop_fails_the_run() {
    let shape = Shape::Fleet(1);
    let config = owned(&["app"]);
    let node = config.clone();
    let spec = spec_with(
        shape,
        vec![client(script(), shape, 0, None)],
        Schedule::default(),
        Rc::new(move || {
            mutants::Silent::new(
                ModelNode::new(node.clone(), budgets(None)),
                |r: &Request| matches!(r.op, Op::ApiVersion),
            )
        }),
    );
    let outcome = run(&spec, SEED);
    let verdict = verdict::judge(&outcome, &config, Budget(1_000_000));
    assert!(!verdict.passed());
    let version = 6;
    let lost: Vec<u32> = verdict
        .stuck
        .iter()
        .filter_map(|s| match s {
            Stuck::Lost { op, attempt, .. } if *op == version => Some(*attempt),
            _ => None,
        })
        .collect();
    assert_eq!(lost, (1..=support::policy().attempts).collect::<Vec<_>>());
    assert!(
        verdict
            .stuck
            .iter()
            .any(|s| matches!(s, Stuck::Starved { op, .. } if *op == version))
    );
    assert!(
        verdict
            .stuck
            .iter()
            .any(|s| matches!(s, Stuck::Waiting(Waiter::Node { calls: 0, .. })))
    );
    // Every other operation completed: the check names only what was stuck.
    assert!(
        outcome
            .attempts
            .iter()
            .filter(|a| a.op != version)
            .all(|a| matches!(a.ended, Ended::Answered { .. }))
    );
}
