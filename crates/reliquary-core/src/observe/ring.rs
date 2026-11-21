//! The event ring (`docs/design/observability.md`, O5; plan RQ0.9, T-RQ0.9.5): where Reliquary's
//! event lines wait for their sink when no host takes its events. It holds a stated number of
//! lines of at most a stated length. A line offered to a full ring evicts the oldest
//! (`concurrent_queue`'s `force_push`, lock-free), so an emitter never waits for the sink. What is
//! let go is counted by reason (`reliquary_events_dropped_total`) and said in the first line of
//! the sink's next batch (`events.dropped`), so the stream itself says where it lost what.
//!
//! Every count here goes up by one a line offered, at most, and offering a line takes more than a
//! nanosecond: a `u64` would take centuries to wrap, so the counters add without a check. The held
//! bytes are a line's length added before the line enters the queue and taken away after it
//! leaves, the queue's push and pop ordering the two, so they never fall below zero.

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::SystemTime;

use concurrent_queue::{ConcurrentQueue, ForcePushError};

use super::catalog::{
    Dropped, EVENTS_BUDGET_BYTES, EVENTS_DROPPED, EVENTS_EMITTED, EVENTS_HELD_BYTES,
};
use super::line;
use super::openmetrics::{Family, Series};

/// A ring's bounds, as its caller states them: how many lines it holds, and the longest line it
/// keeps, in bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bounds {
    pub lines: usize,
    pub line_bytes: usize,
}

/// Why a ring can't be made of the bounds given.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Refused {
    #[error("an event ring holds at least one line, and its bounds give it none")]
    NoLines,
    #[error("an event ring keeps lines of at least one byte, and its bounds keep none")]
    NoBytes,
    #[error(
        "an event ring of {lines} lines of {line_bytes} bytes holds more bytes than this machine counts"
    )]
    TooLarge { lines: usize, line_bytes: usize },
}

/// What became of a line offered to the ring.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Offered {
    Kept,
    /// Kept, and the oldest line let go to make room for it.
    KeptEvictingOldest,
    /// Let go: longer than the ring's line bound.
    Oversize,
    /// Let go: the ring is closed.
    Closed,
}

/// What a drain took.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Drained {
    /// The lines taken from the ring, after the drop marker when there is one.
    pub lines: usize,
    /// What the ring let go since the previous drain, by reason: said in the marker.
    pub lost: Lost,
}

/// Lines let go, by reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Lost {
    pub evicted: u64,
    pub oversize: u64,
    pub closed: u64,
}

impl Lost {
    /// Those let go for `reason`.
    #[must_use]
    pub fn of(&self, reason: Dropped) -> u64 {
        match reason {
            Dropped::Evicted => self.evicted,
            Dropped::Oversize => self.oversize,
            Dropped::Closed => self.closed,
        }
    }

    /// Whether any was let go.
    #[must_use]
    pub fn any(&self) -> bool {
        Dropped::ALL.iter().any(|&reason| self.of(reason) > 0)
    }
}

/// Counts of lines let go, by reason.
#[derive(Default)]
struct Counts {
    evicted: AtomicU64,
    oversize: AtomicU64,
    closed: AtomicU64,
}

impl Counts {
    fn of(&self, reason: Dropped) -> &AtomicU64 {
        match reason {
            Dropped::Evicted => &self.evicted,
            Dropped::Oversize => &self.oversize,
            Dropped::Closed => &self.closed,
        }
    }

    /// The counts, each set back to zero.
    fn take(&self) -> Lost {
        Lost {
            evicted: self.evicted.swap(0, Ordering::Relaxed),
            oversize: self.oversize.swap(0, Ordering::Relaxed),
            closed: self.closed.swap(0, Ordering::Relaxed),
        }
    }
}

/// The ring, shared by the layer that offers it lines and the sink that drains them.
pub struct EventRing {
    lines: ConcurrentQueue<Box<[u8]>>,
    bounds: Bounds,
    budget_bytes: usize,
    emitted: AtomicU64,
    held_bytes: AtomicUsize,
    /// Let go since the ring was made, and since its previous drain.
    dropped: Counts,
    unsaid: Counts,
}

impl EventRing {
    /// A ring of `bounds`: room for `bounds.lines` lines, made now, of at most `bounds.line_bytes`
    /// bytes each.
    ///
    /// # Errors
    ///
    /// [`Refused`] when the bounds give no line, no byte, or more bytes than a `usize` counts.
    pub fn new(bounds: Bounds) -> Result<Self, Refused> {
        if bounds.lines == 0 {
            return Err(Refused::NoLines);
        }
        if bounds.line_bytes == 0 {
            return Err(Refused::NoBytes);
        }
        let budget_bytes =
            bounds
                .lines
                .checked_mul(bounds.line_bytes)
                .ok_or(Refused::TooLarge {
                    lines: bounds.lines,
                    line_bytes: bounds.line_bytes,
                })?;
        Ok(Self {
            lines: ConcurrentQueue::bounded(bounds.lines),
            bounds,
            budget_bytes,
            emitted: AtomicU64::new(0),
            held_bytes: AtomicUsize::new(0),
            dropped: Counts::default(),
            unsaid: Counts::default(),
        })
    }

    /// Offers `line`: kept, evicting the oldest when the ring is full; let go and counted when it
    /// is longer than the line bound or the ring is closed. Takes no lock and never waits.
    pub fn offer(&self, line: Box<[u8]>) -> Offered {
        self.emitted.fetch_add(1, Ordering::Relaxed);
        let bytes = line.len();
        if bytes > self.bounds.line_bytes {
            self.lose(Dropped::Oversize);
            return Offered::Oversize;
        }
        self.held_bytes.fetch_add(bytes, Ordering::Relaxed);
        match self.lines.force_push(line) {
            Ok(None) => Offered::Kept,
            Ok(Some(oldest)) => {
                self.held_bytes.fetch_sub(oldest.len(), Ordering::Relaxed);
                self.lose(Dropped::Evicted);
                Offered::KeptEvictingOldest
            }
            Err(ForcePushError(_)) => {
                self.held_bytes.fetch_sub(bytes, Ordering::Relaxed);
                self.lose(Dropped::Closed);
                Offered::Closed
            }
        }
    }

    /// Moves what the ring holds to the end of `out`, each line whole: first, when any was let go
    /// since the previous drain, the `events.dropped` line at `at` saying how many for each reason;
    /// then the lines, oldest first, at most one ring's worth, so a drain ends while emitters
    /// outpace it.
    pub fn drain(&self, at: SystemTime, out: &mut Vec<u8>) -> Drained {
        let lost = self.unsaid.take();
        if lost.any() {
            out.extend_from_slice(&line::dropped(at, |reason| lost.of(reason)));
        }
        let lines = self
            .lines
            .try_iter()
            .take(self.bounds.lines)
            .map(|line| {
                self.held_bytes.fetch_sub(line.len(), Ordering::Relaxed);
                out.extend_from_slice(&line);
            })
            .count();
        Drained { lines, lost }
    }

    /// Closes the ring: a line offered from now on is let go and counted as closed, and drains
    /// still take what it holds. Whether this call closed it.
    pub fn close(&self) -> bool {
        self.lines.close()
    }

    /// The ring's metrics, every series of each, for a scrape.
    #[must_use]
    pub fn families(&self) -> Vec<Family> {
        // A usize is 64 bits on every target Reliquary builds for, so the fallback is never taken.
        let whole = |n: usize| u64::try_from(n).unwrap_or(u64::MAX);
        vec![
            Family {
                metric: &EVENTS_EMITTED,
                series: vec![Series::of(self.emitted.load(Ordering::Relaxed))],
            },
            Family {
                metric: &EVENTS_DROPPED,
                series: Dropped::ALL
                    .iter()
                    .map(|&reason| Series {
                        labels: vec![reason.word()],
                        value: self.dropped.of(reason).load(Ordering::Relaxed),
                    })
                    .collect(),
            },
            Family {
                metric: &EVENTS_HELD_BYTES,
                series: vec![Series::of(whole(self.held_bytes.load(Ordering::Relaxed)))],
            },
            Family {
                metric: &EVENTS_BUDGET_BYTES,
                series: vec![Series::of(whole(self.budget_bytes))],
            },
        ]
    }

    fn lose(&self, reason: Dropped) {
        self.dropped.of(reason).fetch_add(1, Ordering::Relaxed);
        self.unsaid.of(reason).fetch_add(1, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use std::time::UNIX_EPOCH;

    use super::*;
    use crate::observe::catalog::{EVENTS_DROPPED_EVENT, METRICS, Metric};

    fn ring(lines: usize, line_bytes: usize) -> EventRing {
        EventRing::new(Bounds { lines, line_bytes }).unwrap()
    }

    fn line(text: &str) -> Box<[u8]> {
        format!("{text}\n").into_bytes().into_boxed_slice()
    }

    fn value(ring: &EventRing, metric: &Metric, label: Option<&str>) -> u64 {
        let families = ring.families();
        let family = families
            .iter()
            .find(|f| f.metric.name == metric.name)
            .unwrap();
        family
            .series
            .iter()
            .find(|s| s.labels.first().copied() == label)
            .unwrap()
            .value
    }

    /// Make rings of no line, of lines of no byte, and of more bytes than a usize counts → each
    /// refused, saying which; never a panic (T-RQ0.9.5; CLAUDE.md §1, §2).
    #[test]
    fn t_rq0_9_5_a_ring_without_room_is_refused_typed() {
        assert_eq!(
            EventRing::new(Bounds {
                lines: 0,
                line_bytes: 8
            })
            .err(),
            Some(Refused::NoLines)
        );
        assert_eq!(
            EventRing::new(Bounds {
                lines: 8,
                line_bytes: 0
            })
            .err(),
            Some(Refused::NoBytes)
        );
        assert_eq!(
            EventRing::new(Bounds {
                lines: usize::MAX,
                line_bytes: 2
            })
            .err(),
            Some(Refused::TooLarge {
                lines: usize::MAX,
                line_bytes: 2
            })
        );
    }

    /// Offer five lines to a ring of three, stalled → the first two evicted; the drain gives the
    /// marker saying two were evicted, then the newest three in order; the counters agree
    /// (T-RQ0.9.5).
    #[test]
    fn t_rq0_9_5_a_full_ring_evicts_its_oldest_and_says_how_many() {
        let ring = ring(3, 16);
        let offered: Vec<Offered> = (0..5).map(|i| ring.offer(line(&format!("e{i}")))).collect();
        assert_eq!(
            offered,
            [
                Offered::Kept,
                Offered::Kept,
                Offered::Kept,
                Offered::KeptEvictingOldest,
                Offered::KeptEvictingOldest
            ]
        );
        let mut out = Vec::new();
        let drained = ring.drain(UNIX_EPOCH, &mut out);
        assert_eq!(drained.lines, 3);
        assert_eq!(
            drained.lost,
            Lost {
                evicted: 2,
                oversize: 0,
                closed: 0
            }
        );
        let text = String::from_utf8(out).unwrap();
        let lines: Vec<&str> = text.lines().collect();
        assert!(
            lines[0].contains(&format!("\"event\":\"{}\"", EVENTS_DROPPED_EVENT.name)),
            "{text}"
        );
        assert!(lines[0].contains("\"evicted\":2"), "{text}");
        assert_eq!(&lines[1..], ["e2", "e3", "e4"]);
        assert_eq!(value(&ring, &EVENTS_EMITTED, None), 5);
        assert_eq!(value(&ring, &EVENTS_DROPPED, Some("evicted")), 2);
        assert_eq!(value(&ring, &EVENTS_HELD_BYTES, None), 0);
        // What was said is not said again: the next drain has no marker.
        let mut again = Vec::new();
        assert_eq!(ring.drain(UNIX_EPOCH, &mut again).lost, Lost::default());
        assert!(again.is_empty());
    }

    /// Offer a line one byte past the line bound, and one at it → the first let go as oversize and
    /// counted, the second kept: a line is never cut (T-RQ0.9.5; CLAUDE.md §3).
    #[test]
    fn t_rq0_9_5_a_line_past_the_bound_is_counted_never_cut() {
        let ring = ring(2, 4);
        assert_eq!(ring.offer(line("abcd")), Offered::Oversize);
        assert_eq!(ring.offer(line("abc")), Offered::Kept);
        assert_eq!(value(&ring, &EVENTS_DROPPED, Some("oversize")), 1);
        assert_eq!(value(&ring, &EVENTS_HELD_BYTES, None), 4);
        let mut out = Vec::new();
        let drained = ring.drain(UNIX_EPOCH, &mut out);
        assert_eq!(drained.lost.oversize, 1);
        assert!(String::from_utf8(out).unwrap().ends_with("\nabc\n"));
    }

    /// Close a ring holding a line, then offer another → the drain still gives the held line,
    /// and the late one is counted as closed and said in the marker (T-RQ0.9.5).
    #[test]
    fn t_rq0_9_5_a_closed_ring_drains_what_it_held_and_counts_what_came_late() {
        let ring = ring(2, 16);
        assert_eq!(ring.offer(line("held")), Offered::Kept);
        assert!(ring.close());
        assert!(!ring.close());
        assert_eq!(ring.offer(line("late")), Offered::Closed);
        let mut out = Vec::new();
        let drained = ring.drain(UNIX_EPOCH, &mut out);
        assert_eq!(drained.lines, 1);
        assert_eq!(drained.lost.closed, 1);
        let text = String::from_utf8(out).unwrap();
        assert!(text.contains("\"closed\":1"), "{text}");
        assert!(text.ends_with("\nheld\n"), "{text}");
        assert_eq!(value(&ring, &EVENTS_DROPPED, Some("closed")), 1);
        assert_eq!(value(&ring, &EVENTS_HELD_BYTES, None), 0);
    }

    /// Read the families of a new ring → every series of every ring metric, at zero but the
    /// budget, which is its lines times its line bound (T-RQ0.9.4; Prometheus, "Avoid missing
    /// metrics").
    #[test]
    fn ac_rq0_9_1_a_new_ring_reports_every_series_from_the_start() {
        let ring = ring(4, 256);
        let families = ring.families();
        let names: Vec<&str> = families.iter().map(|f| f.metric.name).collect();
        let catalog: Vec<&str> = METRICS.iter().map(|m| m.name).collect();
        assert_eq!(names, catalog);
        for family in &families {
            assert_eq!(Some(family.series.len()), family.metric.series());
        }
        assert_eq!(value(&ring, &EVENTS_BUDGET_BYTES, None), 1024);
        assert_eq!(value(&ring, &EVENTS_DROPPED, Some("evicted")), 0);
    }
}
