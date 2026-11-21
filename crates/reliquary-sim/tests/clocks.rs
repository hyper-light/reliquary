//! Clocks that jump backward (plan RQ0.6): the bytes store's wall clock stepped back by more than
//! any grant in the run lives, while slow and duplicated deliveries present grants again after the
//! store refused them as expired; no grant is honored after it was refused as expired, and no
//! process sets a deadline before its own now. A node that measures elapsed time by its wall clock
//! does set one, which the run reports (T-RQ0.6.6).

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
use std::time::Duration;

use hyper_sim::net::Path;
use reliquary_api::check::Budget;
use reliquary_api::request::{Op, Request};
use reliquary_sim::model_node::ModelNode;
use reliquary_sim::nemesis::{Fault, Schedule};
use reliquary_sim::verdict;
use reliquary_sim::world::End;

use support::{
    MS, Shape, budgets, client, content, digest, owned, push, request, run, spec, spec_with,
};

const SEED: u64 = 0x5eed_0606;

/// How long each grant lives: every jump below is ten times it.
const LIFE: Duration = Duration::from_millis(4);

/// A push, then forty whole reads of its layer, each by a grant.
fn reads() -> Vec<Request> {
    let layer = content(5, 3000);
    let config = content(6, 100);
    let mut script = push("app", 1, &layer, &config, "v1");
    for _ in 0..40 {
        script.push(request(Op::GetBlob {
            name: "app".to_owned(),
            digest: digest(&layer).to_string(),
            range: None,
        }));
    }
    script
}

/// Faults for a fleet of one node: the client's link to the bytes store slowed past a grant's life,
/// and the clocks of the bytes store and of the node each stepped back by ten lives, three times.
fn jumps(shape: Shape, client: usize) -> Schedule {
    let (_, bytes) = shape.stores();
    let node = shape.nodes()[0];
    let back = -i64::try_from((LIFE * 10).as_nanos()).unwrap();
    let mut faults = vec![(
        Duration::ZERO,
        Fault::SlowLink {
            from: client,
            to: bytes,
            path: Path::in_order(3_000_000, 2_000_000),
        },
    )];
    for at in [30, 60, 90] {
        faults.push((
            MS * at,
            Fault::JumpClock {
                machine: bytes,
                by_ns: back,
            },
        ));
        faults.push((
            MS * (at + 15),
            Fault::JumpClock {
                machine: node,
                by_ns: back,
            },
        ));
    }
    Schedule {
        faults,
        calm: MS * 150,
    }
}

/// Whole reads by grants living 4 ms, over a client link slow enough that some arrive after their
/// grant expired, with half of all deliveries made twice, while the bytes store's and the node's
/// wall clocks are stepped back 40 ms three times: grants are honored and refused as expired, some
/// expired grant is presented again, and none is honored after it was refused; no deadline is set
/// before its setter's now; and the history is one the model allows (T-RQ0.6.6).
#[test]
fn a_clock_jumping_back_past_every_grant_honors_none_twice_and_makes_no_wait_negative() {
    let shape = Shape::Fleet(1);
    let config = owned(&["app"]);
    let client_machine = 3;
    let (mut honored, mut expired, mut again, mut jumped) = (0, 0, 0, 0);
    for seed in 0..8 {
        let mut spec = spec(
            &config,
            shape,
            vec![client(reads(), shape, 0, None)],
            jumps(shape, client_machine),
            Some(LIFE),
        );
        spec.duplicate_ppm = 500_000;
        let outcome = run(&spec, SEED + seed);
        assert_eq!(outcome.end, End::Finished, "{:#?}", outcome.tail);
        honored += outcome.grants.honored;
        expired += outcome.grants.expired;
        again += outcome.grants.again;
        jumped += outcome.faults.get("jump a clock").copied().unwrap_or(0);
        let verdict = verdict::judge(&outcome, &config, Budget(10_000_000));
        assert!(
            verdict.breaches.is_empty(),
            "seed {seed}: {:?}",
            verdict.breaches
        );
        assert!(verdict.waits.is_empty(), "seed {seed}: {:?}", verdict.waits);
        assert!(verdict.passed(), "seed {seed}: {verdict}");
    }
    assert!(
        honored > 0 && expired > 0 && again > 0,
        "honored {honored}, expired {expired}, again {again}"
    );
    assert_eq!(jumped, 8 * 6);
}

/// The same run against a node that measures how long its calls may take by its wall clock since
/// it started: once its clock is stepped back past where it started, it sets deadlines before its
/// own now, and the run names each (T-RQ0.6.6).
#[test]
fn a_node_that_measures_waits_by_its_wall_clock_sets_negative_waits_and_the_run_names_them() {
    let shape = Shape::Fleet(1);
    let config = owned(&["app"]);
    let node = config.clone();
    let mut spec = spec_with(
        shape,
        vec![client(reads(), shape, 0, None)],
        jumps(shape, 3),
        Rc::new(move || {
            mutants::WallWaits::new(
                ModelNode::new(node.clone(), budgets(Some(LIFE))),
                budgets(None).wait,
            )
        }),
    );
    spec.duplicate_ppm = 500_000;
    let outcome = run(&spec, SEED);
    let verdict = verdict::judge(&outcome, &config, Budget(10_000_000));
    assert!(!verdict.waits.is_empty(), "no negative wait found");
    assert!(
        verdict
            .waits
            .iter()
            .all(|w| w.process == "node2" && w.short_by_ns > 0)
    );
}
