//! Crash injection at every write (plan RQ0.6): a push and the reads of everything it made, run
//! once for each write it makes to records and to bytes with power cut before that write and once
//! with power cut after it; after each restart nothing acknowledged is lost and nothing partial is
//! visible (T-RQ0.6.2, AC-RQ0.6.3). A node that names its bytes before they are made fails the
//! same sweep, so the sweep is shown to find what it is for.

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

use std::collections::BTreeSet;
use std::rc::Rc;

use hyper_sim::Random;
use reliquary_api::check::Budget;
use reliquary_api::model::Config;
use reliquary_api::request::Request;
use reliquary_sim::call::Seam;
use reliquary_sim::ledger::Side;
use reliquary_sim::model_node::ModelNode;
use reliquary_sim::nemesis::Schedule;
use reliquary_sim::sweep;

use support::{Shape, budgets, client, content, owned, push, read_back, spec, spec_with};

const SEED: u64 = 0x5eed_0602;

/// A push of a 6000-byte layer in two chunks, a config and a tagged manifest, then the reads of
/// everything it made.
fn push_history() -> (Config, Vec<Request>) {
    let layer = content(1, 6000);
    let config = content(2, 300);
    let mut script = push("app", 1, &layer, &config, "v1");
    script.extend(read_back("app", &layer, &config, "v1"));
    (owned(&["app"]), script)
}

/// Power cut before and after each write of a push, one run per cut, on a laptop (the node and
/// both stores on the machine that loses power) and in a fleet (a store's machine loses power, the
/// node does not): after each restart the history, the push's reads included, is one the model
/// allows with every acknowledged operation in it, and every operation once the machine is back
/// completes. Every write is cut on both sides, and the cuts fall on records and on bytes
/// (T-RQ0.6.2, AC-RQ0.6.3).
#[test]
fn power_cut_at_each_write_of_a_push_loses_nothing_acknowledged_and_shows_nothing_partial() {
    for shape in [Shape::Laptop, Shape::Fleet(1)] {
        let (config, script) = push_history();
        let spec = spec(
            &config,
            shape,
            vec![client(script, shape, 0, None)],
            Schedule::default(),
            None,
        );
        let sweep = sweep::sweep(&spec, SEED, || Random, &config, Budget(1_000_000)).unwrap();
        assert!(sweep.uncut.passed(), "{shape:?}: {}", sweep.uncut);
        let methods: BTreeSet<&str> = sweep.writes.iter().map(|w| w.method).collect();
        for method in [
            "records.commit",
            "bytes.begin",
            "bytes.send",
            "bytes.finish",
        ] {
            assert!(
                methods.contains(method),
                "{shape:?}: no {method} in {methods:?}"
            );
        }
        assert_eq!(sweep.cuts.len(), 2 * sweep.writes.len());
        let failures: Vec<String> = sweep
            .failures()
            .map(|c| {
                format!(
                    "{:?} at {} ({}): landed {}\n{}",
                    c.cut.side, c.cut.write, c.write.method, c.landed, c.verdict
                )
            })
            .collect();
        assert!(failures.is_empty(), "{shape:?}:\n{}", failures.join("\n"));
        let covered = sweep.covered();
        for seam in [Seam::Records, Seam::Bytes] {
            for side in [Side::Before, Side::After] {
                assert!(
                    covered.get(&(seam, side)).copied().unwrap_or(0) > 0,
                    "{shape:?}: no cut {side:?} a write to {seam:?}: {covered:?}"
                );
            }
        }
    }
}

/// The same sweep of a node whose bytes answer a finish before they make the object, and make it
/// only once the commit naming it is answered: power cut between the two leaves the log naming an
/// object that is not there, and the runs cut there fail (T-RQ0.6.2, AC-RQ0.6.3).
#[test]
fn the_sweep_finds_a_node_that_names_its_bytes_before_they_are_made() {
    let (config, script) = push_history();
    let shape = Shape::Laptop;
    let node_config = config.clone();
    let spec = spec_with(
        shape,
        vec![client(script, shape, 0, None)],
        Schedule::default(),
        Rc::new(move || {
            mutants::EagerFinish::new(ModelNode::new(node_config.clone(), budgets(None)))
        }),
    );
    let sweep = sweep::sweep(&spec, SEED, || Random, &config, Budget(1_000_000)).unwrap();
    assert!(sweep.uncut.passed(), "{}", sweep.uncut);
    let failed: Vec<_> = sweep.failures().collect();
    assert!(!failed.is_empty(), "no cut found the planted defect");
    assert!(failed.iter().all(|c| c.landed));
    // Each failure is a run cut between a commit and the finish of the object it names.
    assert!(
        failed
            .iter()
            .any(|c| c.cut.side == Side::After && c.write.method == "records.commit"),
        "{:?}",
        failed
            .iter()
            .map(|c| (c.cut, c.write.method))
            .collect::<Vec<_>>()
    );
}
