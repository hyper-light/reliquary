//! `Clock`: time, as Reliquary's code takes it (plan RQ0.4). A monotonic now, which every deadline
//! is derived from; the wall clock, which records are stamped with and grants and leases end by;
//! and sleeps until a deadline. Code that takes them from a `Clock` runs on real time in a node and
//! on a [`SimClock`] in simulation, where time moves only when it is moved, so a test of a long
//! wait takes no time and states every gap exactly.
//!
//! A deadline is an [`Instant`] its caller derives (its own now, plus what it learned that path's
//! answers take: architecture D8); no method of any seam takes a timeout as a number
//! (AC-RQ0.4.5). A trait that takes no deadline of its own (Quiver's `Topics`) is held to its
//! caller's by [`within`].

use std::collections::BTreeSet;
use std::future::{Future, poll_fn};
use std::pin::{Pin, pin};
use std::sync::Arc;
use std::task::{Context, Poll, Waker};
use std::time::{Duration, Instant, SystemTime};

use parking_lot::Mutex;

use crate::seams::BoxFuture;
use crate::seams::conformance::{Case, Failure, Fixture, ensure, refused, same};
use crate::seams::fault::{CUT, Counts, DELAY, Fault, Faults, Kind, LOSE, Unarmed};

/// A sleep: completes once its clock's now reaches its deadline, or is refused. Dropping it
/// cancels it.
pub type Sleep = Pin<Box<dyn Future<Output = Result<(), Refusal>> + Send + 'static>>;

/// Time.
pub trait Clock: Send + Sync + 'static {
    /// Monotonic time: never goes back. What deadlines are derived from and durations measured by.
    fn now(&self) -> Instant;
    /// The wall clock. It moves with the time that passes, but a real one can be set back or
    /// forward, so durations come from [`Clock::now`].
    fn wall(&self) -> SystemTime;
    /// Completes once [`Clock::now`] reaches `deadline`; at once for a deadline already past.
    fn sleep_until(&self, deadline: Instant) -> Sleep;
}

/// Why a sleep was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Refusal {
    /// The clock holds its most sleeps waiting: this one was refused at once, never completed
    /// early.
    #[error("the clock holds its most sleeps waiting ({limit})")]
    Full { limit: u64 },
    /// The clock has ended (its runtime closed): no sleep will complete on time.
    #[error("the clock has ended")]
    Ended,
    /// The sleep was armed and its answer lost: its place is held until its deadline, and its
    /// caller arms another.
    #[error("the sleep was armed and its answer lost")]
    Lost,
}

impl Kind for Refusal {
    fn kind(&self) -> &'static str {
        match self {
            Self::Full { .. } => "Full",
            Self::Ended => "Ended",
            Self::Lost => "Lost",
        }
    }
}

/// The faults a clock's simulator injects (`docs/design/seams.md`, "Clock"): each refusal of a
/// sleep, a sleep woken late past its deadline, a sleep whose answer is lost after it was armed, and
/// the clock's stream of wakes cut at any instant.
pub const FAULTS: &[&str] = &["Full", "Ended", "Lost", DELAY, LOSE, CUT];

/// Why a call held to a deadline by [`within`] did not end with its work's answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Past {
    /// The deadline came first: the work was dropped there, and what it did by then is the
    /// caller's to learn by reading.
    #[error("the deadline passed before the call was answered")]
    Deadline,
    /// The clock refused the sleep the deadline is kept by.
    #[error("the clock: {0}")]
    Clock(Refusal),
}

/// `work`'s answer, unless `deadline` passes on `clock` first: how a caller holds a call of a trait
/// that takes no deadline (Quiver's `Topics`) to the deadline it derived. The work is dropped at
/// the deadline.
pub async fn within<T>(
    clock: &dyn Clock,
    deadline: Instant,
    mut work: BoxFuture<'_, T>,
) -> Result<T, Past> {
    let mut timer = clock.sleep_until(deadline);
    poll_fn(|cx| {
        if let Poll::Ready(answer) = work.as_mut().poll(cx) {
            return Poll::Ready(Ok(answer));
        }
        match timer.as_mut().poll(cx) {
            Poll::Ready(Ok(())) => Poll::Ready(Err(Past::Deadline)),
            Poll::Ready(Err(refusal)) => Poll::Ready(Err(Past::Clock(refusal))),
            Poll::Pending => Poll::Pending,
        }
    })
    .await
}

// ---------------------------------------------------------------------------------- simulator

/// A simulated clock's one operation that faults are armed for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Op {
    Sleep,
}

/// Why a simulated clock could not move as asked: past the farthest instant or wall time the
/// platform can say.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("the simulated clock cannot move past the farthest time the platform can say")]
pub struct Overflow;

/// Why [`SimClock::run`] gave up on its work.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Stalled {
    /// The work waits, and no sleep is waiting that moving the clock could end: it waits on
    /// something nothing will do.
    #[error("the work waits on nothing the clock can end")]
    Waiting,
    /// The work had its budget of steps (a poll and a move of the clock each) and did not end.
    #[error("the work did not end within its {0} steps")]
    Steps(u64),
    /// The clock could not move to its next sleep.
    #[error("{0}")]
    Time(Overflow),
}

/// A clock whose time moves only when it is moved ([`SimClock::advance`]): its sleeps complete in
/// deadline order, ties in the order they started. Clones share one time.
///
/// Its sleeps keep no waker: the work that awaits them is polled again by whoever moves the clock
/// ([`SimClock::run`] polls its work after every move). A waker kept in the clock would be dropped
/// with it, and a waker's drop runs its executor's code through a function pointer, which could
/// drop the clock again: a cycle the recursion gate refuses (CLAUDE.md §11).
#[derive(Clone)]
pub struct SimClock {
    inner: Arc<Mutex<Ticking>>,
}

struct Ticking {
    /// The real instant the clock was made at: a simulated instant is this plus `elapsed`, as an
    /// `Instant` cannot be made from a number.
    base: Instant,
    elapsed: Duration,
    wall: SystemTime,
    /// Sleeps waiting, by when they complete and the order they started.
    sleepers: BTreeSet<(Instant, u64)>,
    /// Sleeps armed whose answers were lost: each holds its place until its deadline.
    ghosts: BTreeSet<(Instant, u64)>,
    next: u64,
    /// The most sleeps waiting at once, as the clock's maker states.
    limit: usize,
    /// When the clock's stream of wakes ended, if it has: at its `end`, or at a cut's instant.
    ended: Option<Instant>,
    faults: Faults<Op, Refusal>,
}

impl Ticking {
    fn now(&self) -> Instant {
        // `advance` moves time only where the sum exists, so this is always `Some`.
        self.base.checked_add(self.elapsed).unwrap_or(self.base)
    }

    fn limit(&self) -> u64 {
        u64::try_from(self.limit).unwrap_or(u64::MAX)
    }

    /// Whether the clock holds its most sleeps: those waiting and those whose answers were lost.
    fn full(&self) -> bool {
        self.sleepers.len().saturating_add(self.ghosts.len()) >= self.limit
    }

    /// A new sleep's key: when it completes, and the order it started.
    fn key(&mut self, wake: Instant) -> Option<(Instant, u64)> {
        let id = self.next;
        self.next = id.checked_add(1)?;
        Some((wake, id))
    }
}

/// A span's length in nanoseconds, or `u64::MAX` past what that holds.
fn nanos(d: Duration) -> u64 {
    u64::try_from(d.as_nanos()).unwrap_or(u64::MAX)
}

fn fits(_: Op, fault: &Fault<Refusal>) -> bool {
    !matches!(fault, Fault::Vanish)
}

impl SimClock {
    /// A clock whose wall clock starts at `wall`, holding at most `sleeps` sleeps waiting at once.
    #[must_use]
    pub fn new(wall: SystemTime, sleeps: usize) -> Self {
        let ticking = Ticking {
            base: Instant::now(),
            elapsed: Duration::ZERO,
            wall,
            sleepers: BTreeSet::new(),
            ghosts: BTreeSet::new(),
            next: 0,
            limit: sleeps,
            ended: None,
            faults: Faults::default(),
        };
        Self {
            inner: Arc::new(Mutex::new(ticking)),
        }
    }

    /// How long the clock has run: the sum of every move.
    #[must_use]
    pub fn elapsed(&self) -> Duration {
        self.inner.lock().elapsed
    }

    /// Moves time and the wall clock forward by `d`: every sleep due by then completes when it is
    /// next polled.
    pub fn advance(&self, d: Duration) -> Result<(), Overflow> {
        let mut t = self.inner.lock();
        let elapsed = t.elapsed.checked_add(d).ok_or(Overflow)?;
        t.base.checked_add(elapsed).ok_or(Overflow)?;
        t.wall = t.wall.checked_add(d).ok_or(Overflow)?;
        t.elapsed = elapsed;
        // A cut armed at an instant this move reaches ends the clock's stream of wakes there.
        if t.ended.is_none()
            && let Some(at) = t.faults.cut(Op::Sleep, 0, nanos(elapsed))
        {
            t.ended = Some(
                t.base
                    .checked_add(Duration::from_nanos(at))
                    .unwrap_or(t.base),
            );
        }
        // A sleep whose answer was lost holds its place until its deadline, and no longer.
        let now = t.now();
        t.ghosts.retain(|(at, _)| *at > now);
        Ok(())
    }

    /// Moves time forward to `t`; an instant already past moves nothing.
    pub fn advance_to(&self, t: Instant) -> Result<(), Overflow> {
        let d = t.saturating_duration_since(self.now());
        self.advance(d)
    }

    /// When the next sleep waiting completes, if one waits.
    #[must_use]
    pub fn next_wake(&self) -> Option<Instant> {
        self.inner.lock().sleepers.first().map(|(at, _)| *at)
    }

    /// Ends the clock, as a runtime closing: every sleep waiting, and every later one, completes
    /// refused [`Refusal::Ended`] when it is next polled.
    pub fn end(&self) {
        let mut t = self.inner.lock();
        if t.ended.is_none() {
            t.ended = Some(t.now());
        }
    }

    /// Arms `fault` for the next sleep: a refusal, a delay that wakes it that much late, or its
    /// answer lost once it is armed. A cut ([`Fault::Cut`], in nanoseconds since the clock was
    /// made) ends the clock's stream of wakes at that instant, once time reaches it.
    pub fn arm(&self, fault: Fault<Refusal>) -> Result<(), Unarmed> {
        self.inner.lock().faults.arm(Op::Sleep, fault, fits)
    }

    /// Arms a cut of the clock's stream of wakes at `at`: once time reaches it, every sleep due by
    /// then completes, and every later one is refused [`Refusal::Ended`].
    pub fn cut_at(&self, at: Instant) -> Result<(), Unarmed> {
        let mut t = self.inner.lock();
        let since = nanos(at.saturating_duration_since(t.base));
        t.faults.arm(Op::Sleep, Fault::Cut(since), fits)
    }

    /// What the clock injected, by kind.
    #[must_use]
    pub fn injected(&self) -> Counts {
        self.inner.lock().faults.injected().clone()
    }

    /// Runs `work` to its end: polls it, and whenever it waits, moves the clock to the next sleep
    /// due and polls it again, at most `steps` times. Work that waits with no sleep waiting waits
    /// on something no move of the clock can end, and is given up ([`Stalled::Waiting`]): a
    /// deadlock, found rather than waited on.
    pub fn run<F: Future>(&self, work: F, steps: u64) -> Result<F::Output, Stalled> {
        let mut work = pin!(work);
        let mut cx = Context::from_waker(Waker::noop());
        let mut left = steps;
        loop {
            if let Poll::Ready(out) = work.as_mut().poll(&mut cx) {
                return Ok(out);
            }
            left = left.checked_sub(1).ok_or(Stalled::Steps(steps))?;
            let at = self.next_wake().ok_or(Stalled::Waiting)?;
            self.advance_to(at).map_err(Stalled::Time)?;
        }
    }
}

impl Clock for SimClock {
    fn now(&self) -> Instant {
        self.inner.lock().now()
    }

    fn wall(&self) -> SystemTime {
        self.inner.lock().wall
    }

    fn sleep_until(&self, deadline: Instant) -> Sleep {
        let mut t = self.inner.lock();
        let after_end = t.ended.is_some();
        let fault = t.faults.call(Op::Sleep);
        let (wake, refused) = match fault {
            Some(Fault::Refuse(refusal)) => (deadline, Some(refusal)),
            Some(Fault::Delay(late)) => (
                // A wake later than the platform can say is a sleep that never wakes, which the
                // clock's end still completes.
                deadline.checked_add(late).unwrap_or(deadline),
                None,
            ),
            // Armed, and its answer lost: its place is held until its deadline.
            Some(Fault::Lose) => {
                let refusal = if t.full() {
                    Refusal::Full { limit: t.limit() }
                } else {
                    match t.key(deadline) {
                        Some(key) => {
                            t.ghosts.insert(key);
                            Refusal::Lost
                        }
                        None => Refusal::Full { limit: t.limit() },
                    }
                };
                (deadline, Some(refusal))
            }
            _ => (deadline, None),
        };
        Box::pin(SimSleep {
            clock: Arc::clone(&self.inner),
            wake,
            key: None,
            refused,
            after_end,
        })
    }
}

struct SimSleep {
    clock: Arc<Mutex<Ticking>>,
    wake: Instant,
    key: Option<(Instant, u64)>,
    refused: Option<Refusal>,
    /// It was made once the clock had ended.
    after_end: bool,
}

impl Future for SimSleep {
    type Output = Result<(), Refusal>;

    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        if let Some(refusal) = this.refused.take() {
            return Poll::Ready(Err(refusal));
        }
        let mut t = this.clock.lock();
        let now = t.now();
        // Made after the clock ended, or due after its wakes ended: refused Ended. Due before:
        // complete once its time has come.
        let done = match t.ended {
            Some(_) if this.after_end => Some(Err(Refusal::Ended)),
            Some(end) if this.wake > end && now >= end => Some(Err(Refusal::Ended)),
            _ if now >= this.wake => Some(Ok(())),
            _ => None,
        };
        if let Some(outcome) = done {
            if let Some(key) = this.key.take() {
                t.sleepers.remove(&key);
            }
            return Poll::Ready(outcome);
        }
        if this.key.is_some() {
            return Poll::Pending;
        }
        let full = Refusal::Full { limit: t.limit() };
        if t.full() {
            return Poll::Ready(Err(full));
        }
        let Some(key) = t.key(this.wake) else {
            return Poll::Ready(Err(full));
        };
        t.sleepers.insert(key);
        this.key = Some(key);
        Poll::Pending
    }
}

impl Drop for SimSleep {
    fn drop(&mut self) {
        if let Some(key) = self.key.take() {
            self.clock.lock().sleepers.remove(&key);
        }
    }
}

// -------------------------------------------------------------------------------- conformance

/// What a clock's conformance suite runs against.
pub trait ClockFixture: Fixture {
    fn clock(&self) -> &dyn Clock;
    /// Lets `d` pass by the clock: a simulated one is moved, a real one waited out.
    fn pass(&self, d: Duration) -> Result<(), Failure>;
    /// A span the clock lets pass and still tells from twice it: what the cases' times are
    /// counted in.
    fn quantum(&self) -> Duration;
    /// The most sleeps the clock holds waiting at once.
    fn sleeps_max(&self) -> usize;
    /// Arms `fault` for the clock's next sleep.
    fn arm(&self, fault: Fault<Refusal>) -> Result<(), Failure>;
    /// Ends the clock, as its runtime closing.
    fn end(&self) -> Result<(), Failure>;
    /// Cuts the clock's stream of wakes at `at`: once time reaches it, the clock has ended there.
    fn cut(&self, at: Instant) -> Result<(), Failure>;
}

/// The fixture for a [`SimClock`]: its quantum, and the steps [`SimClock::run`] has, as its maker
/// states them.
pub struct SimClockFixture {
    pub clock: SimClock,
    pub quantum: Duration,
    pub sleeps: usize,
    pub steps: u64,
}

impl Fixture for SimClockFixture {
    fn drive<'a, T>(&'a self, work: BoxFuture<'a, T>) -> Result<T, Failure> {
        self.clock
            .run(work, self.steps)
            .map_err(|stalled| stalled.to_string())
    }

    fn injected(&self) -> Counts {
        self.clock.injected()
    }
}

impl ClockFixture for SimClockFixture {
    fn clock(&self) -> &dyn Clock {
        &self.clock
    }

    fn pass(&self, d: Duration) -> Result<(), Failure> {
        self.clock.advance(d).map_err(|e| e.to_string())
    }

    fn quantum(&self) -> Duration {
        self.quantum
    }

    fn sleeps_max(&self) -> usize {
        self.sleeps
    }

    fn arm(&self, fault: Fault<Refusal>) -> Result<(), Failure> {
        self.clock.arm(fault).map_err(|e| e.to_string())
    }

    fn end(&self) -> Result<(), Failure> {
        self.clock.end();
        Ok(())
    }

    fn cut(&self, at: Instant) -> Result<(), Failure> {
        self.clock.cut_at(at).map_err(|e| e.to_string())
    }
}

/// What `sleep` says when polled once now: its outcome, or `None` while it waits.
fn poll_once(sleep: &mut Sleep) -> Option<Result<(), Refusal>> {
    match sleep.as_mut().poll(&mut Context::from_waker(Waker::noop())) {
        Poll::Ready(outcome) => Some(outcome),
        Poll::Pending => None,
    }
}

/// `q` after `at`: a quantum later.
fn after(at: Instant, q: Duration) -> Result<Instant, Failure> {
    at.checked_add(q)
        .ok_or_else(|| "a deadline past what the platform's time can say".to_owned())
}

fn c1<F: ClockFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        let c = f.clock();
        let first = c.now();
        let mut last = first;
        for _ in 0..f.sleeps_max() {
            let now = c.now();
            ensure(now >= last, || "now() went back".into())?;
            last = now;
        }
        f.pass(f.quantum())?;
        ensure(
            c.now().saturating_duration_since(first) >= f.quantum(),
            || "now() did not move by the time that passed".into(),
        )
    })
}

fn c2<F: ClockFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        let c = f.clock();
        let before = c.wall();
        f.pass(f.quantum())?;
        let moved = c
            .wall()
            .duration_since(before)
            .map_err(|e| format!("wall() went back by {:?} while time passed", e.duration()))?;
        ensure(moved >= f.quantum(), || {
            format!("wall() moved {moved:?} while {:?} passed", f.quantum())
        })
    })
}

fn c3<F: ClockFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        let c = f.clock();
        let then = c.now();
        same(
            "a sleep until now",
            poll_once(&mut c.sleep_until(then)),
            Some(Ok(())),
        )?;
        f.pass(f.quantum())?;
        same(
            "a sleep until a deadline already past",
            poll_once(&mut c.sleep_until(then)),
            Some(Ok(())),
        )
    })
}

fn c4<F: ClockFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        let c = f.clock();
        let q = f.quantum();
        let deadline = after(after(c.now(), q)?, q)?;
        let mut sleep = c.sleep_until(deadline);
        same("a sleep before its deadline", poll_once(&mut sleep), None)?;
        f.pass(q)?;
        let early = poll_once(&mut sleep).is_some() && c.now() < deadline;
        ensure(!early, || "a sleep completed before its deadline".into())?;
        f.pass(q)?;
        same(
            "a sleep once its deadline has come",
            poll_once(&mut sleep),
            Some(Ok(())),
        )
    })
}

fn c5<F: ClockFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        let c = f.clock();
        let q = f.quantum();
        let start = c.now();
        // Three deadlines a quantum apart, started out of order: the last first, then the first,
        // then the middle.
        let first = after(start, q)?;
        let middle = after(first, q)?;
        let last = after(middle, q)?;
        let mut sleeps: Vec<(Instant, Option<Sleep>)> = [last, first, middle]
            .into_iter()
            .map(|deadline| (deadline, Some(c.sleep_until(deadline))))
            .collect();
        sleeps.sort_by_key(|(deadline, _)| *deadline);
        // A quantum passes per turn, so by the last turn every deadline has passed.
        for _ in 0..sleeps.len() {
            f.pass(q)?;
            let now = c.now();
            for (deadline, slot) in &mut sleeps {
                // Each sleep is polled until it completes, and not after.
                let Some(sleep) = slot.as_mut() else { continue };
                let done = poll_once(sleep).is_some();
                ensure(done == (*deadline <= now), || {
                    format!("at {now:?}, the sleep until {deadline:?} done: {done}")
                })?;
                if done {
                    *slot = None;
                }
            }
        }
        ensure(sleeps.iter().all(|(_, slot)| slot.is_none()), || {
            "a sleep did not complete once every deadline had passed".into()
        })
    })
}

fn c6<F: ClockFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        let c = f.clock();
        let q = f.quantum();
        f.arm(Fault::Delay(q))?;
        let deadline = after(c.now(), q)?;
        let mut sleep = c.sleep_until(deadline);
        f.pass(q)?;
        same(
            "a sleep woken late, at its deadline",
            poll_once(&mut sleep),
            None,
        )?;
        f.pass(q)?;
        same(
            "a sleep woken late, once its lateness passed",
            poll_once(&mut sleep),
            Some(Ok(())),
        )?;
        ensure(c.now().saturating_duration_since(deadline) >= q, || {
            "now() does not say how late the sleep woke".into()
        })
    })
}

fn c7<F: ClockFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        let c = f.clock();
        let q = f.quantum();
        let limit = u64::try_from(f.sleeps_max()).unwrap_or(u64::MAX);
        f.arm(Fault::Refuse(Refusal::Full { limit }))?;
        same(
            "a sleep the clock was made to refuse",
            poll_once(&mut c.sleep_until(after(c.now(), q)?)),
            Some(Err(Refusal::Full { limit })),
        )?;
        let deadline = after(c.now(), q)?;
        let mut held: Vec<Sleep> = Vec::new();
        for _ in 0..f.sleeps_max() {
            let mut sleep = c.sleep_until(deadline);
            same("a sleep within the bound", poll_once(&mut sleep), None)?;
            held.push(sleep);
        }
        same(
            "a sleep past the bound",
            poll_once(&mut c.sleep_until(deadline)),
            Some(Err(Refusal::Full { limit })),
        )?;
        f.pass(q)?;
        for sleep in &mut held {
            same(
                "a sleep within the bound, at its deadline",
                poll_once(sleep),
                Some(Ok(())),
            )?;
        }
        Ok(())
    })
}

fn c8<F: ClockFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        let c = f.clock();
        let q = f.quantum();
        f.arm(Fault::Refuse(Refusal::Ended))?;
        same(
            "a sleep the clock was made to refuse",
            poll_once(&mut c.sleep_until(after(c.now(), q)?)),
            Some(Err(Refusal::Ended)),
        )?;
        let mut waiting = c.sleep_until(after(c.now(), q)?);
        same(
            "a sleep before the clock ends",
            poll_once(&mut waiting),
            None,
        )?;
        f.end()?;
        same(
            "a sleep waiting when the clock ended",
            poll_once(&mut waiting),
            Some(Err(Refusal::Ended)),
        )?;
        let refusal = refused(
            "a sleep after the clock ended",
            poll_once(&mut c.sleep_until(after(c.now(), q)?)).unwrap_or(Ok(())),
        )?;
        same("its refusal", refusal, Refusal::Ended)
    })
}

fn c9<F: ClockFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        let c = f.clock();
        let q = f.quantum();
        f.arm(Fault::Refuse(Refusal::Lost))?;
        same(
            "a sleep the clock was made to refuse",
            poll_once(&mut c.sleep_until(after(c.now(), q)?)),
            Some(Err(Refusal::Lost)),
        )?;
        let deadline = after(c.now(), q)?;
        f.arm(Fault::Lose)?;
        same(
            "a sleep whose answer is lost once it was armed",
            poll_once(&mut c.sleep_until(deadline)),
            Some(Err(Refusal::Lost)),
        )?;
        // Until its deadline, the lost sleep holds its place: one fewer is held.
        let limit = u64::try_from(f.sleeps_max()).unwrap_or(u64::MAX);
        let mut held: Vec<Sleep> = Vec::new();
        for _ in 1..f.sleeps_max() {
            let mut sleep = c.sleep_until(deadline);
            same("a sleep within the bound", poll_once(&mut sleep), None)?;
            held.push(sleep);
        }
        same(
            "a sleep past the bound, a lost one holding its place",
            poll_once(&mut c.sleep_until(deadline)),
            Some(Err(Refusal::Full { limit })),
        )?;
        f.pass(q)?;
        for sleep in &mut held {
            same("a sleep at its deadline", poll_once(sleep), Some(Ok(())))?;
        }
        // Past its deadline the lost sleep holds nothing: the bound's every place is free.
        let later = after(c.now(), q)?;
        let mut again: Vec<Sleep> = Vec::new();
        for _ in 0..f.sleeps_max() {
            let mut sleep = c.sleep_until(later);
            same(
                "a sleep within the bound, the lost one gone",
                poll_once(&mut sleep),
                None,
            )?;
            again.push(sleep);
        }
        Ok(())
    })
}

fn c10<F: ClockFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        let c = f.clock();
        let q = f.quantum();
        let first = after(c.now(), q)?;
        let middle = after(first, q)?;
        let last = after(middle, q)?;
        let mut sleeps: Vec<(Instant, Sleep)> = [first, middle, last]
            .into_iter()
            .map(|deadline| (deadline, c.sleep_until(deadline)))
            .collect();
        for (_, sleep) in &mut sleeps {
            same("a sleep before the cut", poll_once(sleep), None)?;
        }
        // Cut at the first deadline: what is due by then completes, the rest are refused Ended.
        f.cut(first)?;
        f.pass(q)?;
        for (deadline, sleep) in &mut sleeps {
            let want = if *deadline <= first {
                Ok(())
            } else {
                Err(Refusal::Ended)
            };
            same(
                &format!("the sleep until {deadline:?}, cut at {first:?}"),
                poll_once(sleep),
                Some(want),
            )?;
        }
        same(
            "a sleep after the cut",
            poll_once(&mut c.sleep_until(after(c.now(), q)?)),
            Some(Err(Refusal::Ended)),
        )
    })
}

/// The `Clock` contract, clause by clause (`docs/design/seams.md`, "Clock").
#[must_use]
pub fn cases<F: ClockFixture>() -> Vec<Case<F>> {
    vec![
        Case {
            id: "CLOCK-1",
            says: "now() never goes back, and moves by the time that passes",
            check: c1::<F>,
        },
        Case {
            id: "CLOCK-2",
            says: "wall() moves forward by the time that passes",
            check: c2::<F>,
        },
        Case {
            id: "CLOCK-3",
            says: "a sleep until now, or until a deadline already past, completes at once",
            check: c3::<F>,
        },
        Case {
            id: "CLOCK-4",
            says: "a sleep never completes before its deadline, and completes once it has passed",
            check: c4::<F>,
        },
        Case {
            id: "CLOCK-5",
            says: "sleeps complete in deadline order, whatever order they started in",
            check: c5::<F>,
        },
        Case {
            id: "CLOCK-6",
            says: "a sleep woken late completes after its deadline, and now() says how late",
            check: c6::<F>,
        },
        Case {
            id: "CLOCK-7",
            says: "past the most sleeps a clock holds, a sleep is refused Full at once, never completed early",
            check: c7::<F>,
        },
        Case {
            id: "CLOCK-8",
            says: "once the clock ends, every sleep waiting and every later one is refused Ended",
            check: c8::<F>,
        },
        Case {
            id: "CLOCK-9",
            says: "a sleep whose answer is lost once it was armed is refused Lost, and holds its place until its deadline and no longer: a lost answer leaks no sleep",
            check: c9::<F>,
        },
        Case {
            id: "CLOCK-10",
            says: "a clock whose stream of wakes is cut at an instant completes every sleep due by then, and refuses every later one Ended",
            check: c10::<F>,
        },
    ]
}
