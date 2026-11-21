//! What every seam's conformance suite is made of, in kiln-seams' shape (kiln's
//! `crates/kiln-seams/src/conformance.rs`): cases, each one clause of a seam's contract, its id and
//! its words as `docs/design/seams.md` says them, each run against a fresh fixture of its own.
//!
//! A fixture is an implementation and what a case needs around it: the simulator's controls (its
//! clock, the faults it can be made to inject), or, for a real implementation, the same controls
//! over the real thing. A fixture runs a case's work as its implementation runs
//! ([`Fixture::drive`]): a simulator by moving its simulated clock to the next sleep whenever
//! nothing else can go on, a real one on its runtime. A case's work never waits on the wall clock.

use std::future::poll_fn;
use std::task::Poll;

use crate::seams::BoxFuture;
use crate::seams::fault::Counts;

/// What a case found wrong, in words.
pub type Failure = String;

/// One clause of a seam's contract, checked against what fixture `F` gives.
pub struct Case<F> {
    /// The clause's id in `docs/design/seams.md` (`BYTES-3`).
    pub id: &'static str,
    /// What the clause says, as the doc says it.
    pub says: &'static str,
    pub check: for<'a> fn(&'a F) -> BoxFuture<'a, Result<(), Failure>>,
}

/// An implementation under test and the controls a case needs around it.
pub trait Fixture: Sync {
    /// Runs `work` to its end as the implementation runs. A simulator's fixture moves its simulated
    /// clock to the next sleep whenever nothing else can go on, within a budget of steps its maker
    /// states, and fails `work` that waits on nothing its clock can end.
    fn drive<'a, T>(&'a self, work: BoxFuture<'a, T>) -> Result<T, Failure>;

    /// The faults injected into the implementation so far, by kind.
    fn injected(&self) -> Counts;
}

/// What a suite's run found.
#[derive(Debug, Default)]
pub struct Report {
    pub seam: &'static str,
    pub passed: Vec<&'static str>,
    pub failed: Vec<(&'static str, Failure)>,
    /// What every case's fixture injected, by kind: the non-vacuity counts (T-RQ0.4.9).
    pub injected: Counts,
}

impl Report {
    /// Every case passed, and there was at least one.
    #[must_use]
    pub fn is_ok(&self) -> bool {
        self.failed.is_empty() && !self.passed.is_empty()
    }

    /// How many cases passed, or each failure by its clause.
    pub fn into_result(self) -> Result<usize, String> {
        if self.is_ok() {
            return Ok(self.passed.len());
        }
        let lines: Vec<String> = self
            .failed
            .iter()
            .map(|(id, why)| format!("{id}: {why}"))
            .collect();
        Err(format!(
            "{}: {} of {} cases failed\n{}",
            self.seam,
            self.failed.len(),
            self.passed.len().saturating_add(self.failed.len()),
            lines.join("\n")
        ))
    }
}

/// Runs each case, in order, against a fixture of its own (`make` makes one per case), and gathers
/// what each fixture injected.
pub fn run<F: Fixture>(
    seam: &'static str,
    cases: &[Case<F>],
    mut make: impl FnMut() -> Result<F, Failure>,
) -> Report {
    let mut report = Report {
        seam,
        ..Report::default()
    };
    for case in cases {
        let outcome = match make() {
            Ok(fixture) => {
                let outcome = fixture.drive((case.check)(&fixture)).and_then(|o| o);
                report.injected.merge(&fixture.injected());
                outcome
            }
            Err(why) => Err(format!("its fixture wasn't made: {why}")),
        };
        match outcome {
            Ok(()) => report.passed.push(case.id),
            Err(why) => report.failed.push((case.id, why)),
        }
    }
    report
}

/// `Ok` when `cond` holds, else the failure `why` says.
pub fn ensure(cond: bool, why: impl FnOnce() -> String) -> Result<(), Failure> {
    if cond { Ok(()) } else { Err(why()) }
}

/// `Ok` when `got == want`, else both, named.
pub fn same<T: PartialEq + std::fmt::Debug>(what: &str, got: T, want: T) -> Result<(), Failure> {
    if got == want {
        Ok(())
    } else {
        Err(format!("{what}: got {got:?}, want {want:?}"))
    }
}

/// `v`'s value, or its refusal said as a failure.
pub fn fine<T, E: std::fmt::Display>(what: &str, v: Result<T, E>) -> Result<T, Failure> {
    v.map_err(|e| format!("{what}: refused: {e}"))
}

/// The refusal `v` should have been, or a failure saying it was accepted.
pub fn refused<T: std::fmt::Debug, E>(what: &str, v: Result<T, E>) -> Result<E, Failure> {
    match v {
        Ok(t) => Err(format!(
            "{what}: accepted ({t:?}), should have been refused"
        )),
        Err(e) => Ok(e),
    }
}

/// `work`'s output from one poll: how a fixture whose implementation never waits (an entropy
/// source, the records simulator) drives a case. Work that waits fails, named.
pub fn at_once<T>(mut work: BoxFuture<'_, T>) -> Result<T, Failure> {
    match work
        .as_mut()
        .poll(&mut std::task::Context::from_waker(std::task::Waker::noop()))
    {
        Poll::Ready(out) => Ok(out),
        Poll::Pending => Err("the work waited, and nothing it calls ever waits".into()),
    }
}

/// Both futures, polled in turn until each has its output: two calls under way at once.
pub async fn both<A: Send, B: Send>(mut a: BoxFuture<'_, A>, mut b: BoxFuture<'_, B>) -> (A, B) {
    let (mut got_a, mut got_b) = (None, None);
    poll_fn(|cx| {
        if got_a.is_none()
            && let Poll::Ready(v) = a.as_mut().poll(cx)
        {
            got_a = Some(v);
        }
        if got_b.is_none()
            && let Poll::Ready(v) = b.as_mut().poll(cx)
        {
            got_b = Some(v);
        }
        match (got_a.take(), got_b.take()) {
            (Some(a), Some(b)) => Poll::Ready((a, b)),
            (a, b) => {
                got_a = a;
                got_b = b;
                Poll::Pending
            }
        }
    })
    .await
}
