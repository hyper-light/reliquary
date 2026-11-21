//! Contenders that keep retrying (plan RQ0.6): two clients moving one tag, move after move, their
//! writes conflicting at the log, both finish every move within a counted number of attempts; and
//! a node that turns whoever comes second away starves one of them, which the run reports
//! (T-RQ0.6.4).

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
use reliquary_api::generate::{self, Race};
use reliquary_api::harness;
use reliquary_sim::client::Ended;
use reliquary_sim::model_node::ModelNode;
use reliquary_sim::nemesis::Schedule;
use reliquary_sim::verdict::{self, Stuck};
use reliquary_sim::world::End;

use support::{Shape, budgets, clients, policy, run, spec, spec_with};

const SEED: u64 = 0x5eed_0604;

/// Two writers each moving one tag among three manifests forty times, with no reader and nothing
/// lost, after a setup that pushed the manifests.
fn race(index: u64) -> (reliquary_api::model::Config, reliquary_api::history::Plan) {
    let seed = harness::history_seed(SEED, index);
    generate::tag_race(
        seed,
        &Race {
            writers: 2,
            readers: 0,
            manifests: 3,
            moves: 40,
            reads: 0,
            lose_one_in: 0,
        },
    )
    .unwrap()
}

/// Two writers moving one tag, both on one node and each on a node of its own: their commits
/// conflict at the log (the stores refused commits `Conflict`), yet every move completes, none
/// taking more than its client's counted attempts and none starving, and the history is one the
/// model allows (T-RQ0.6.4).
#[test]
fn two_clients_retrying_conflicting_tag_moves_both_finish_within_their_attempts() {
    let mut conflicts = 0;
    for shape in [Shape::Laptop, Shape::Fleet(2)] {
        for index in 0..4 {
            let (config, plan) = race(index);
            let spec = spec(
                &config,
                shape,
                clients(&plan, shape),
                Schedule::default(),
                None,
            );
            let outcome = run(&spec, harness::history_seed(SEED, index));
            assert_eq!(outcome.end, End::Finished, "{:#?}", outcome.tail);
            conflicts += outcome
                .answers
                .get("refused Conflict")
                .copied()
                .unwrap_or(0);
            let most = outcome.attempts.iter().map(|a| a.attempt).max().unwrap();
            assert!(most <= policy().attempts);
            let ops: std::collections::BTreeSet<(u16, u64)> = outcome
                .attempts
                .iter()
                .map(|a| (a.client.0, a.op))
                .collect();
            let done: std::collections::BTreeSet<(u16, u64)> = outcome
                .attempts
                .iter()
                .filter(|a| matches!(a.ended, Ended::Answered { .. }))
                .map(|a| (a.client.0, a.op))
                .collect();
            assert_eq!(ops, done, "{shape:?} {index}: an operation never completed");
            let verdict = verdict::judge(&outcome, &config, Budget(10_000_000));
            assert!(verdict.passed(), "{shape:?} {index}: {verdict}");
        }
    }
    assert!(conflicts > 0, "the writers never conflicted");
}

/// The same race against one node, a LAN away from its stores, that refuses `Busy` every request
/// arriving while another is in flight and holds each answer back for eight more reads of its
/// records: the writer that comes second is turned away attempt after attempt while the other
/// keeps the node busy, uses every attempt, and the run fails, naming it starved (T-RQ0.6.4).
#[test]
fn a_node_that_turns_the_second_contender_away_starves_it_and_the_run_fails() {
    let shape = Shape::Fleet(1);
    let mut starved = 0;
    for index in 0..4 {
        let (config, plan) = race(index);
        let node = config.clone();
        let spec = spec_with(
            shape,
            clients(&plan, shape),
            Schedule::default(),
            Rc::new(move || {
                mutants::OneAtATime::new(
                    ModelNode::new(node.clone(), budgets(None)),
                    8,
                    budgets(None).wait,
                )
            }),
        );
        let outcome = run(&spec, harness::history_seed(SEED, index));
        let verdict = verdict::judge(&outcome, &config, Budget(10_000_000));
        starved += verdict
            .stuck
            .iter()
            .filter(|s| matches!(s, Stuck::Starved { .. }))
            .count();
    }
    assert!(starved > 0, "no run found a contender starved");
}
