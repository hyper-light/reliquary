//! Races (plan RQ0.5): two clients moving one tag while a third reads it give only reads some
//! linearization allows (T-RQ0.5.4); the checker refuses a read that goes back after a move was
//! answered and allows one that overlaps it, lost answers included (AC-RQ0.5.3); and concurrent
//! histories of two to eight clients interleave, lose answers and check, as the counters show
//! (AC-RQ0.5.2).

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

use std::collections::BTreeSet;

use reliquary_api::answer::{Answer, Body};
use reliquary_api::check::{self, Budget, Verdict};
use reliquary_api::generate::{self, Mix, Race, Shape};
use reliquary_api::harness::{self, Gap, Interleaving};
use reliquary_api::history::{Client, History, Operation, Outcome};
use reliquary_api::model::Model;
use reliquary_api::request::{Op, Principal, Request};

const SEED: u64 = 0x5eed_7a65;

/// Two writers moving one tag among three manifests while a third client reads it: in every
/// interleaving, every read is a value some linearization allows, with answers lost too; and the
/// race is real: reads saw the tag at more than one manifest, and all three clients' operations
/// overlapped (T-RQ0.5.4).
#[test]
fn two_writers_moving_a_tag_and_a_reader_stay_linearizable() {
    let mut seen = BTreeSet::new();
    let mut all_three_overlapped = 0;
    let mut lost = 0;
    for (lose_one_in, histories) in [(0, 150), (6, 150)] {
        let race = Race {
            writers: 2,
            readers: 1,
            manifests: 3,
            moves: 5,
            reads: 8,
            lose_one_in,
        };
        for index in 0..histories {
            let seed = harness::history_seed(SEED, index);
            let (config, plan) = generate::tag_race(seed, &race).unwrap();
            let history = plan.run(&mut Model::new(config.clone())).unwrap();
            let verdict = check::check(&Model::new(config), &history, Budget(1_000_000));
            assert!(
                matches!(verdict, Verdict::Linearizable { .. }),
                "seed {seed:#x}: {verdict:?}"
            );
            for op in &history.operations {
                if let (
                    Op::GetManifest { .. } | Op::HeadManifest { .. },
                    Outcome::Answered {
                        answer:
                            Answer {
                                body: Body::Manifest { digest, .. },
                                ..
                            },
                        ..
                    },
                ) = (&op.request.op, &op.outcome)
                {
                    seen.insert(digest.clone());
                }
            }
            if history.interleaved_clients() == 3 {
                all_three_overlapped += 1;
            }
            lost += history.lost();
        }
    }
    assert!(seen.len() >= 2, "reads saw only {seen:?}");
    assert!(all_three_overlapped > 0, "the clients never overlapped");
    assert!(lost > 0, "no answer was lost");
}

/// After a move of the tag is answered, a read that returns the old manifest is refused, and the
/// read is named; the same read overlapping the move is allowed; and with the move's answer lost,
/// either manifest is allowed (AC-RQ0.5.3, T-RQ0.5.4).
#[test]
fn the_checker_refuses_a_read_that_goes_back_and_allows_one_that_overlaps() {
    let setup_only = Race {
        writers: 0,
        readers: 0,
        manifests: 2,
        moves: 0,
        reads: 0,
        lose_one_in: 0,
    };
    let (config, plan) = generate::tag_race(SEED, &setup_only).unwrap();
    let setup = plan.run(&mut Model::new(config.clone())).unwrap();
    let mut model = Model::new(config.clone());
    for op in &setup.operations {
        model.step(&op.request);
    }
    // The setup pushes two manifests by digest, then puts the tag on the first.
    let pushes: Vec<&Request> = setup
        .operations
        .iter()
        .map(|op| &op.request)
        .filter(|r| matches!(&r.op, Op::PutManifest { reference, .. } if reference.contains(':')))
        .collect();
    let Op::PutManifest {
        manifest: second, ..
    } = &pushes[1].op
    else {
        panic!("the setup's second push is not a manifest's");
    };
    let owner = Principal::Named(0);
    let move_tag = Request {
        principal: owner,
        op: Op::PutManifest {
            name: "race".to_owned(),
            reference: "latest".to_owned(),
            manifest: second.clone(),
        },
    };
    let read = Request {
        principal: owner,
        op: Op::GetManifest {
            name: "race".to_owned(),
            reference: "latest".to_owned(),
        },
    };
    let old = model.clone().step(&read);
    let mut moved = model.clone();
    let move_answer = moved.step(&move_tag);
    let new = moved.step(&read);
    assert_ne!(old, new);

    let start = setup.operations.last().unwrap().completed().unwrap() + 1;
    let history = |move_ends: Option<u64>, read_starts: u64, read_answer: &Answer| {
        let mut operations = setup.operations.clone();
        operations.push(Operation {
            client: Client(1),
            request: move_tag.clone(),
            invoked: start,
            outcome: match move_ends {
                Some(at) => Outcome::Answered {
                    at,
                    answer: move_answer.clone(),
                },
                None => Outcome::Lost,
            },
        });
        operations.push(Operation {
            client: Client(2),
            request: read.clone(),
            invoked: read_starts,
            outcome: Outcome::Answered {
                at: start + 10,
                answer: read_answer.clone(),
            },
        });
        History { operations }
    };
    let initial = Model::new(config);
    let budget = Budget(100_000);
    let read_index = setup.operations.len() + 1;

    // The move answered before the read began: the old manifest goes back.
    let back = history(Some(start + 1), start + 2, &old);
    assert_eq!(
        check::check(&initial, &back, budget),
        Verdict::NotLinearizable {
            operation: read_index
        }
    );
    // The new manifest after the move: allowed.
    assert!(matches!(
        check::check(&initial, &history(Some(start + 1), start + 2, &new), budget),
        Verdict::Linearizable { .. }
    ));
    // The read overlapping the move: either is allowed.
    for answer in [&old, &new] {
        assert!(matches!(
            check::check(
                &initial,
                &history(Some(start + 5), start + 2, answer),
                budget
            ),
            Verdict::Linearizable { .. }
        ));
    }
    // The move's answer lost: it may have happened or not, so either is allowed.
    for answer in [&old, &new] {
        assert!(matches!(
            check::check(&initial, &history(None, start + 2, answer), budget),
            Verdict::Linearizable { .. }
        ));
    }
}

/// Concurrent histories of two to eight clients, run against the model and checked: the counters
/// show histories in which each of two to eight clients interleaved and answers lost, and every
/// history was linearizable (AC-RQ0.5.2).
#[test]
fn interleavings_of_two_to_many_clients_are_produced_and_counted() {
    let mix = Mix {
        repositories: 2,
        principals: 1,
        tags: 3,
        contents: 4,
        largest_blob: 16,
        invalid_one_in: 4,
        manifest_bound: None,
        staging_budget: None,
    };
    let hunt = Interleaving {
        seed: SEED,
        histories: 49,
        budget: Budget(2_000_000),
        shrink_tries: 0,
    };
    let plans = |seed: u64, index: u64| {
        let clients = 2 + u16::try_from(index % 7).unwrap();
        generate::concurrent(
            seed,
            &mix,
            Shape {
                clients,
                setup: 12,
                per_client: 3,
                lose_one_in: 10,
            },
        )
    };
    let coverage = harness::hunt_concurrent(&hunt, plans, |c| Model::new(c.clone()))
        .unwrap_or_else(|unexplained| panic!("{unexplained:?}"));
    let missing: Vec<Gap> = coverage
        .gaps(&[2, 3, 4, 5, 6, 7, 8])
        .into_iter()
        .filter(|gap| matches!(gap, Gap::Interleaved(_)))
        .collect();
    assert!(missing.is_empty(), "no history interleaved {missing:?}");
    assert!(coverage.lost > 0, "no answer was lost");
    assert_eq!(coverage.histories, 49);
}
