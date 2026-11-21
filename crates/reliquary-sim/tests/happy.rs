//! Runs with no faults (plan RQ0.6): every operation of a generated history completes on its first
//! attempt with the answer the model gives, on one node and on three (T-RQ0.6.1).

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

use reliquary_api::check::Budget;
use reliquary_api::generate::{self, Shape as PlanShape};
use reliquary_api::harness;
use reliquary_api::model::Model;
use reliquary_sim::client::Ended;
use reliquary_sim::nemesis::Schedule;
use reliquary_sim::verdict;
use reliquary_sim::world::End;

use support::{Shape, client, clients, mix, run, spec};

const SEED: u64 = 0x5eed_0601;

/// One client's generated history, run with no faults on one node and on three: every operation
/// completes on its first attempt, and each answer is the one the model gives the same requests in
/// the same order (T-RQ0.6.1).
#[test]
fn a_sequential_history_with_no_faults_completes_and_matches_the_model() {
    let mut answered = 0;
    for shape in [Shape::Laptop, Shape::Fleet(3)] {
        for index in 0..8 {
            let seed = harness::history_seed(SEED, index);
            let (config, requests) = generate::sequential(seed, &mix(), 60);
            let spec = spec(
                &config,
                shape,
                vec![client(requests.clone(), shape, 0, None)],
                Schedule::default(),
                None,
            );
            let outcome = run(&spec, seed);
            assert_eq!(
                outcome.end,
                End::Finished,
                "seed {seed:#x}: {:#?}",
                outcome.tail
            );
            assert_eq!(outcome.attempts.len(), requests.len());
            let mut model = Model::new(config.clone());
            for (attempt, request) in outcome.attempts.iter().zip(&requests) {
                assert_eq!(attempt.attempt, 1, "seed {seed:#x}: {attempt:?}");
                let expected = model.step(request);
                match &attempt.ended {
                    Ended::Answered { answer, .. } => {
                        assert_eq!(answer, &expected, "seed {seed:#x}: {request}");
                        answered += 1;
                    }
                    other => panic!("seed {seed:#x}: {request} ended {other:?}"),
                }
            }
            let verdict = verdict::judge(&outcome, &config, Budget(1_000_000));
            assert!(verdict.passed(), "seed {seed:#x}: {verdict}");
        }
    }
    assert_eq!(answered, 2 * 8 * 60);
}

/// A generated plan of a setup and three clients, run with no faults on one node and on three:
/// every operation completes on its first attempt, and the history is linearizable against the
/// model (T-RQ0.6.1).
#[test]
fn a_concurrent_history_with_no_faults_completes_and_is_linearizable() {
    for shape in [Shape::Laptop, Shape::Fleet(3)] {
        for index in 0..6 {
            let seed = harness::history_seed(SEED ^ 0xc0c, index);
            let (config, plan) = generate::concurrent(
                seed,
                &mix(),
                PlanShape {
                    clients: 3,
                    setup: 12,
                    per_client: 8,
                    lose_one_in: 0,
                },
            )
            .unwrap();
            let spec = spec(
                &config,
                shape,
                clients(&plan, shape),
                Schedule::default(),
                None,
            );
            let outcome = run(&spec, seed);
            assert_eq!(
                outcome.end,
                End::Finished,
                "seed {seed:#x}: {:#?}",
                outcome.tail
            );
            assert_eq!(outcome.attempts.len(), plan.operations().len());
            assert!(
                outcome
                    .attempts
                    .iter()
                    .all(|a| a.attempt == 1 && matches!(a.ended, Ended::Answered { .. })),
                "seed {seed:#x}"
            );
            let verdict = verdict::judge(&outcome, &config, Budget(5_000_000));
            assert!(verdict.passed(), "seed {seed:#x}: {verdict}");
        }
    }
}
