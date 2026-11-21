//! How a round's operations are offered and timed (AC-RQ0.7.5): open loop with each operation's
//! intended start recorded and its latency measured from it, or closed loop as a control.
//!
//! A closed loop sends its next operation when the last answers, so a stall stops the offered load and
//! hides from the tail: "coordinated omission" (wrk2's note, research/06 F48). Open loop, each
//! operation's intended start is drawn ahead from a Poisson process (hyper-measure's `Arrivals`), and its
//! latency runs from that intended start, never from when the driver got to it, so an operation queued
//! behind a stall is charged the stall (Schroeder, Wierman and Harchol-Balter, NSDI '06, research/07).
//! The driver's own lateness, its start less the intended, is recorded beside, so a run in which the
//! generator was the bottleneck shows it (Mantle's design §10).
//!
//! The loop is one function over a [`Clock`]: the worker drives it on the monotonic clock, and the tests
//! on a simulated one, whose server stalls when told to.

use hyper_measure::open_loop::{Arrivals, ArrivalsError};
use serde::{Deserialize, Serialize};

/// One operation's times on the driver's clock, in nanoseconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Op {
    /// When it should have started: the schedule's time open loop, the last answer's closed loop.
    pub intended: u64,
    /// When the driver started it.
    pub start: u64,
    /// When it answered.
    pub end: u64,
}

impl Op {
    /// Its latency, from its intended start.
    #[must_use]
    pub fn latency(&self) -> u64 {
        self.end.saturating_sub(self.intended)
    }

    /// How late the driver started it.
    #[must_use]
    pub fn late(&self) -> u64 {
        self.start.saturating_sub(self.intended)
    }
}

/// How operations are offered.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Load {
    /// The next operation when the last answers: a control, and the shape of a closed system.
    Closed,
    /// Poisson arrivals at `per_second`, drawn from `seed`.
    Open { per_second: f64, seed: u64 },
}

/// A clock the loop reads and waits on, in nanoseconds from an origin of its own.
pub trait Clock {
    /// Now.
    fn now(&mut self) -> u64;
    /// Returns once `at` has passed; at once if it already has.
    fn wait_until(&mut self, at: u64);
}

/// Why a round could not be driven.
#[derive(Debug, Clone, PartialEq)]
pub enum DriveError<E> {
    /// The schedule could not continue: a rate refused, or a time past the clock.
    Schedule(ArrivalsError),
    /// An operation failed; the round stops there.
    Op { index: usize, error: E },
}

impl<E: std::fmt::Display> std::fmt::Display for DriveError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Schedule(e) => write!(f, "the open loop's schedule: {e}"),
            Self::Op { index, error } => write!(f, "operation {index} failed: {error}"),
        }
    }
}

/// Runs `count` operations under `load`, appending each one's times to `out`. Open loop, the schedule
/// starts at the round's own start and its seed is the round's, so time spent between rounds is never
/// charged to an operation.
pub fn round<C: Clock, E>(
    clock: &mut C,
    load: Load,
    round: u64,
    count: usize,
    out: &mut Vec<Op>,
    mut op: impl FnMut(&mut C) -> Result<(), E>,
) -> Result<(), DriveError<E>> {
    let begin = clock.now();
    let mut arrivals = match load {
        Load::Closed => None,
        Load::Open { per_second, seed } => Some(
            Arrivals::poisson(per_second, round_seed(seed, round), begin)
                .map_err(DriveError::Schedule)?,
        ),
    };
    let mut last_end = begin;
    for index in 0..count {
        let intended = match arrivals.as_mut() {
            Some(a) => a.next_arrival().map_err(DriveError::Schedule)?,
            None => last_end,
        };
        clock.wait_until(intended);
        let start = clock.now();
        op(clock).map_err(|error| DriveError::Op { index, error })?;
        let end = clock.now();
        out.push(Op {
            intended,
            start,
            end,
        });
        last_end = end;
    }
    Ok(())
}

/// Round `round`'s seed: the point's seed and the round's index through one SplitMix64 step, so every
/// round draws a schedule of its own and replays it.
#[must_use]
pub fn round_seed(seed: u64, round: u64) -> u64 {
    let mut r = crate::rng::SplitMix64::new(seed ^ round.wrapping_mul(crate::rng::GOLDEN));
    r.next_u64()
}

/// The monotonic clock, from when it was made, and how late its parks have woken.
#[derive(Debug)]
pub struct Monotonic {
    origin: std::time::Instant,
    /// The most a park has overslept its wake in this process: learned from the waits themselves, as
    /// a platform's timer may wake late by a millisecond or by a scheduler's tick (Windows's default
    /// timer; macOS's coalesced timers, Mantle's design §12).
    park_late: u64,
}

impl Default for Monotonic {
    fn default() -> Self {
        Self {
            origin: std::time::Instant::now(),
            park_late: 0,
        }
    }
}

impl Clock for Monotonic {
    fn now(&mut self) -> u64 {
        u64::try_from(self.origin.elapsed().as_nanos()).unwrap_or(u64::MAX)
    }

    fn wait_until(&mut self, at: u64) {
        // Parks until as long before the deadline as a park has woken late, then yields until it, so a
        // coarse timer does not make every start late; the loop ends on the deadline, and a start still
        // late is the lateness each operation records.
        loop {
            let now = self.now();
            if now >= at {
                return;
            }
            if at.saturating_sub(now) > self.park_late {
                let wake = at.saturating_sub(self.park_late);
                std::thread::park_timeout(std::time::Duration::from_nanos(
                    wake.saturating_sub(now),
                ));
                let woke = self.now();
                self.park_late = self.park_late.max(woke.saturating_sub(wake));
            } else {
                std::thread::yield_now();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::SplitMix64;
    use crate::tails::{Counts, tail};
    use hyper_measure::quantile::Level;

    /// A clock that moves only when told: an operation advances it by its service time.
    struct Sim {
        now: u64,
    }

    impl Clock for Sim {
        fn now(&mut self) -> u64 {
            self.now
        }
        fn wait_until(&mut self, at: u64) {
            self.now = self.now.max(at);
        }
    }

    const MS: u64 = 1_000_000;
    const SECOND: u64 = 1_000_000_000;

    /// A server that answers in 100 µs, but stalls once, for a second, from `stall_at`: an operation
    /// started in the stall waits it out.
    fn stalling(stall_at: u64) -> impl FnMut(&mut Sim) -> Result<(), ()> {
        move |c: &mut Sim| {
            let wait = if (stall_at..stall_at + SECOND).contains(&c.now) {
                stall_at + SECOND - c.now
            } else {
                0
            };
            c.now += wait + 100_000;
            Ok(())
        }
    }

    fn p(ops: &[Op], q: f64) -> u64 {
        let mut l: Vec<u64> = ops.iter().map(Op::latency).collect();
        l.sort_unstable();
        l[((l.len() as f64 * q).ceil() as usize).max(1) - 1]
    }

    /// T-RQ0.7.5: a server that stalls for a second mid-run under open-loop load → the stall appears in
    /// the tail; under a closed-loop control run it does not, and the difference shows.
    #[test]
    fn a_stall_shows_in_the_open_loops_tail_and_hides_from_the_closed_loops() {
        let count = 10_000;
        // Open loop at 1,000 a second for ten seconds, the stall at five.
        let mut open = Vec::new();
        let mut c = Sim { now: 0 };
        let load = Load::Open {
            per_second: 1_000.0,
            seed: 7,
        };
        round(&mut c, load, 0, count, &mut open, stalling(5 * SECOND)).unwrap();
        // Closed loop: back to back, the stall at its own midpoint.
        let mut closed = Vec::new();
        let mut c = Sim { now: 0 };
        let midpoint = count as u64 / 2 * 100_000;
        round(
            &mut c,
            Load::Closed,
            0,
            count,
            &mut closed,
            stalling(midpoint),
        )
        .unwrap();
        let (open_p99, closed_p99) = (p(&open, 0.99), p(&closed, 0.99));
        assert!(open_p99 > SECOND / 2, "open p99 {open_p99} ns");
        assert!(closed_p99 < MS, "closed p99 {closed_p99} ns");
        // The closed loop saw the stall exactly once, in its maximum.
        let stalled = closed.iter().filter(|o| o.latency() > MS).count();
        assert_eq!(stalled, 1);
        let open_stalled = open.iter().filter(|o| o.latency() > MS).count();
        assert!(open_stalled > 900, "{open_stalled}");
        // Each open-loop operation's latency runs from its intended start, which the record keeps.
        assert!(open.iter().all(|o| o.latency() == o.end - o.intended));
        assert!(open.iter().any(|o| o.late() > 0));
    }

    /// AC-RQ0.7.5: open loop records each intended start from the schedule, never from the driver.
    #[test]
    fn intended_starts_follow_the_schedule_whatever_the_server_does() {
        let mut ops = Vec::new();
        let mut c = Sim { now: 3 };
        let load = Load::Open {
            per_second: 500.0,
            seed: 1,
        };
        round(&mut c, load, 4, 1_000, &mut ops, stalling(0)).unwrap();
        let mut a = Arrivals::poisson(500.0, round_seed(1, 4), 3).unwrap();
        for o in &ops {
            assert_eq!(o.intended, a.next_arrival().unwrap());
            assert!(o.start >= o.intended && o.end > o.start);
        }
        // Rounds draw schedules of their own.
        assert_ne!(round_seed(1, 4), round_seed(1, 5));
    }

    /// T-RQ0.7.1: a synthetic operation with a known latency distribution → the reported quantiles'
    /// intervals contain the true values, at the coverage they state.
    #[test]
    fn quantile_intervals_contain_the_true_quantiles_at_their_coverage() {
        // Exponential service with a mean of 1 ms plus 50 µs: the q-quantile is 50 µs − 1 ms·ln(1 − q).
        let truth = |q: f64| 50_000.0 - 1_000_000.0 * (1.0 - q).ln();
        let levels = [(Level::P50, 0.5), (Level::P99, 0.99), (Level::P999, 0.999)];
        let runs = 200;
        let mut missed = [0usize; 3];
        let mut stated = [0usize; 3];
        let mut rng = SplitMix64::new(13);
        for run in 0..runs {
            let mut ops = Vec::new();
            let mut c = Sim { now: 0 };
            round(&mut c, Load::Closed, run, 4_000, &mut ops, |c: &mut Sim| {
                c.now += (truth(rng.unit())).round() as u64;
                Ok::<(), ()>(())
            })
            .unwrap();
            let mut counts = Counts::default();
            for o in &ops {
                counts.record(o.latency());
            }
            for (i, (level, q)) in levels.iter().enumerate() {
                if let Some(t) = tail(&counts, *level) {
                    stated[i] += 1;
                    let v = truth(*q);
                    if !(t.lower as f64 <= v && v <= t.upper as f64) {
                        missed[i] += 1;
                    }
                }
            }
        }
        // 4,000 samples resolve p50, p99 and p99.9 (3,688 needed); each interval covers at 95% or more,
        // so misses stay within the one-sided 5% binomial bound for 200 runs at 5% (15).
        assert_eq!(stated, [runs as usize; 3]);
        assert!(missed.iter().all(|&m| m <= 15), "{missed:?}");
    }

    /// AC-RQ0.7.5: the monotonic clock waits until its deadline and no less.
    #[test]
    fn the_monotonic_clock_waits_until_its_deadline() {
        let mut c = Monotonic::default();
        let at = c.now() + 2 * MS;
        c.wait_until(at);
        assert!(c.now() >= at);
        // A deadline already past returns at once.
        let before = c.now();
        c.wait_until(0);
        assert!(c.now() >= before);
    }

    /// AC-RQ0.7.5: a refused rate and a failed operation are typed refusals.
    #[test]
    fn a_bad_rate_or_a_failed_operation_is_refused_typed() {
        let mut c = Sim { now: 0 };
        let mut ops = Vec::new();
        let bad = Load::Open {
            per_second: 0.0,
            seed: 1,
        };
        let e = round(&mut c, bad, 0, 10, &mut ops, |_| Ok::<(), ()>(())).unwrap_err();
        assert!(matches!(e, DriveError::Schedule(ArrivalsError::Rate(_))));
        let mut n = 0;
        let e = round(&mut c, Load::Closed, 0, 10, &mut ops, |_| {
            n += 1;
            if n == 4 { Err("refused") } else { Ok(()) }
        })
        .unwrap_err();
        assert_eq!(
            e,
            DriveError::Op {
                index: 3,
                error: "refused"
            }
        );
    }
}
