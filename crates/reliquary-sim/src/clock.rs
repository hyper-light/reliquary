//! A process's clocks in the world (docs/design/sim.md, W3): the [`Clock`] seam over the readings
//! hyper-sim's world gives the process's node. The world sets the readings before it hands the
//! process an event, so time moves only between events, as every input is an event the world
//! chose (hyper-sim's design, §3.3). A machine's monotonic clock drifts within its stated rate and
//! never goes back; its wall clock moves with it and can be stepped forward or back by the nemesis.
//!
//! Wall readings sit on a grid of [`WALL_GRID_NS`]: Windows keeps wall time in 100-nanosecond
//! intervals, so a `SystemTime` made from a finer reading would be cut there and on no other
//! platform, and a run would differ by target (plan AC-RQ0.6.1).
//!
//! Sleeps complete once the process's monotonic reading reaches them. The clock keeps no waker:
//! the world polls the process again after it moves the clock, and arms the process's timer at
//! [`ProcessClock::next_wake`] (the same rule as `SimClock`, docs/design/seams.md "Clock").

use std::collections::BTreeSet;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::{Duration, Instant, SystemTime};

use parking_lot::Mutex;
use reliquary_core::seams::clock::{Clock, Refusal, Sleep};

/// The grid wall readings sit on, in nanoseconds: Windows' `FILETIME`, which Rust's `SystemTime`
/// is on Windows, "contains a 64-bit value representing the number of 100-nanosecond intervals
/// since January 1, 1601" (Microsoft's documentation of `FILETIME`).
pub const WALL_GRID_NS: u64 = 100;

/// A wall reading in nanoseconds since the Unix epoch, put on the grid every platform can say.
#[must_use]
pub fn on_grid(wall_ns: u64) -> u64 {
    wall_ns
        .checked_div(WALL_GRID_NS)
        .and_then(|intervals| intervals.checked_mul(WALL_GRID_NS))
        .unwrap_or(wall_ns)
}

struct Readings {
    /// The instant the world's virtual time began: a monotonic reading `m` is `anchor + m`.
    anchor: Instant,
    monotonic_ns: u64,
    wall_ns: u64,
    /// Sleeps waiting, by the monotonic reading they complete at and the order they started.
    sleepers: BTreeSet<(u64, u64)>,
    next: u64,
    /// The most sleeps waiting at once, as the process's maker states.
    limit: usize,
}

/// One process's clocks, as the world sets them. Clones share them.
#[derive(Clone)]
pub struct ProcessClock {
    inner: Arc<Mutex<Readings>>,
}

impl ProcessClock {
    /// Clocks reading `monotonic_ns` and `wall_ns` from `anchor`, holding at most `sleeps` sleeps.
    #[must_use]
    pub fn new(anchor: Instant, monotonic_ns: u64, wall_ns: u64, sleeps: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Readings {
                anchor,
                monotonic_ns,
                wall_ns: on_grid(wall_ns),
                sleepers: BTreeSet::new(),
                next: 0,
                limit: sleeps,
            })),
        }
    }

    /// The world's readings for the process now. The monotonic reading never goes back: one
    /// earlier than the last is kept at the last.
    pub fn set(&self, monotonic_ns: u64, wall_ns: u64) {
        let mut r = self.inner.lock();
        r.monotonic_ns = r.monotonic_ns.max(monotonic_ns);
        r.wall_ns = on_grid(wall_ns);
    }

    /// The monotonic reading, in nanoseconds of the world's time line.
    #[must_use]
    pub fn monotonic_ns(&self) -> u64 {
        self.inner.lock().monotonic_ns
    }

    /// The wall reading, in nanoseconds since the Unix epoch, on the grid.
    #[must_use]
    pub fn wall_ns(&self) -> u64 {
        self.inner.lock().wall_ns
    }

    /// The monotonic reading `instant` stands for; an instant before the anchor reads zero.
    #[must_use]
    pub fn nanos_of(&self, instant: Instant) -> u64 {
        let anchor = self.inner.lock().anchor;
        u64::try_from(instant.saturating_duration_since(anchor).as_nanos()).unwrap_or(u64::MAX)
    }

    /// When the earliest sleep waiting completes, as a monotonic reading.
    #[must_use]
    pub fn next_wake(&self) -> Option<u64> {
        self.inner.lock().sleepers.first().map(|(at, _)| *at)
    }

    /// Every sleep waiting is let go: the process they were in is gone.
    pub fn forget_sleeps(&self) {
        self.inner.lock().sleepers.clear();
    }
}

impl Clock for ProcessClock {
    fn now(&self) -> Instant {
        let r = self.inner.lock();
        r.anchor
            .checked_add(Duration::from_nanos(r.monotonic_ns))
            .unwrap_or(r.anchor)
    }

    fn wall(&self) -> SystemTime {
        let wall_ns = self.inner.lock().wall_ns;
        SystemTime::UNIX_EPOCH
            .checked_add(Duration::from_nanos(wall_ns))
            .unwrap_or(SystemTime::UNIX_EPOCH)
    }

    fn sleep_until(&self, deadline: Instant) -> Sleep {
        let wake = self.nanos_of(deadline);
        Box::pin(ProcessSleep {
            clock: Arc::clone(&self.inner),
            wake,
            key: None,
        })
    }
}

struct ProcessSleep {
    clock: Arc<Mutex<Readings>>,
    wake: u64,
    key: Option<(u64, u64)>,
}

impl Future for ProcessSleep {
    type Output = Result<(), Refusal>;

    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        let mut r = this.clock.lock();
        if r.monotonic_ns >= this.wake {
            if let Some(key) = this.key.take() {
                r.sleepers.remove(&key);
            }
            return Poll::Ready(Ok(()));
        }
        if this.key.is_some() {
            return Poll::Pending;
        }
        let full = Refusal::Full {
            limit: u64::try_from(r.limit).unwrap_or(u64::MAX),
        };
        if r.sleepers.len() >= r.limit {
            return Poll::Ready(Err(full));
        }
        let id = r.next;
        let Some(next) = id.checked_add(1) else {
            return Poll::Ready(Err(full));
        };
        r.next = next;
        r.sleepers.insert((this.wake, id));
        this.key = Some((this.wake, id));
        Poll::Pending
    }
}

impl Drop for ProcessSleep {
    fn drop(&mut self) {
        if let Some(key) = self.key.take() {
            self.clock.lock().sleepers.remove(&key);
        }
    }
}
