//! A state's latency quantiles over its own operations, with their order-statistic intervals, read
//! from counts in hyper-timing's log-linear buckets so a point's memory does not grow with its
//! operations (CLAUDE.md §2).
//!
//! The ranks are hyper-measure's (`quantile::ranks`, David and Nagaraja's distribution-free interval,
//! 95%); each rank is read as the bucket holding it, the estimate and the interval's upper end at their
//! buckets' largest values and its lower end at its bucket's least, so the interval stated is never
//! narrower than the samples' own and a quantile is high by at most a bucket's width, an eighth of its
//! value (hyper-raft's tails design, §3.2). A quantile whose interval has no upper end, because the
//! state has fewer operations than it needs (3,688 for a p99.9), is not stated: the report shows a dash
//! (Mantle's design §5a).

use hyper_measure::quantile::{Level, ranks};
use hyper_timing::Histogram;
use serde::{Deserialize, Serialize};

/// Format: the interval's coverage, 95%, the level Mantle's measurement design states every interval
/// at (its §4).
pub const COVERAGE: Level = match Level::new(95, 100) {
    Ok(level) => level,
    Err(_) => Level::P50,
};

/// Counts of latencies, a bucket each, in hyper-timing's buckets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Counts {
    buckets: Vec<u64>,
    count: u64,
}

impl Default for Counts {
    fn default() -> Self {
        Self {
            buckets: vec![0; Histogram::BUCKETS],
            count: 0,
        }
    }
}

/// A count in one bucket, as a worker sends it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bucket {
    pub bucket: u16,
    pub count: u64,
}

/// Why counts could not be read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CountsError {
    /// A bucket past the last.
    NoSuchBucket(u16),
}

impl std::fmt::Display for CountsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoSuchBucket(b) => write!(f, "bucket {b} is past the last"),
        }
    }
}

impl Counts {
    /// One latency counted.
    pub fn record(&mut self, ns: u64) {
        if let Some(slot) = self.buckets.get_mut(Histogram::bucket(ns)) {
            *slot = slot.saturating_add(1);
            self.count = self.count.saturating_add(1);
        }
    }

    /// Everything `other` counted, counted here too.
    pub fn add(&mut self, other: &Self) {
        for (slot, theirs) in self.buckets.iter_mut().zip(other.buckets.iter()) {
            *slot = slot.saturating_add(*theirs);
        }
        self.count = self.count.saturating_add(other.count);
    }

    /// How many latencies are counted.
    #[must_use]
    pub fn count(&self) -> u64 {
        self.count
    }

    /// The buckets that hold anything, for a worker's reply.
    #[must_use]
    pub fn sparse(&self) -> Vec<Bucket> {
        self.buckets
            .iter()
            .enumerate()
            .filter(|(_, c)| **c > 0)
            .filter_map(|(b, &count)| {
                Some(Bucket {
                    bucket: u16::try_from(b).ok()?,
                    count,
                })
            })
            .collect()
    }

    /// Counts from a worker's reply; a bucket past the last is refused.
    pub fn from_sparse(sparse: &[Bucket]) -> Result<Self, CountsError> {
        let mut c = Self::default();
        for b in sparse {
            let slot = c
                .buckets
                .get_mut(usize::from(b.bucket))
                .ok_or(CountsError::NoSuchBucket(b.bucket))?;
            *slot = slot.saturating_add(b.count);
            c.count = c.count.saturating_add(b.count);
        }
        Ok(c)
    }

    /// The bucket holding the latency of `rank` (from 1).
    fn bucket_of(&self, rank: usize) -> Option<usize> {
        let rank = u64::try_from(rank).ok()?;
        let mut seen = 0u64;
        for (bucket, &held) in self.buckets.iter().enumerate() {
            seen = seen.saturating_add(held);
            if seen >= rank {
                return Some(bucket);
            }
        }
        None
    }
}

/// A stated quantile: its estimate and 95% interval, in nanoseconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Tail {
    pub estimate: u64,
    pub lower: u64,
    pub upper: u64,
}

/// The largest value bucket `b` holds.
fn top(b: usize) -> u64 {
    let (_, past) = Histogram::bounds(b);
    if b == Histogram::BUCKETS.saturating_sub(1) {
        past
    } else {
        past.saturating_sub(1)
    }
}

/// The `q`-quantile of the counted latencies with its interval; none when the counts cannot state it
/// (no upper end, or none counted): a dash.
#[must_use]
pub fn tail(counts: &Counts, q: Level) -> Option<Tail> {
    let n = usize::try_from(counts.count).ok()?;
    let r = ranks(n, q, COVERAGE).ok()?;
    let upper = r.upper?;
    // A lower end below the first sample is the least latency there could be: zero.
    let lower = r
        .lower
        .and_then(|l| counts.bucket_of(l))
        .map_or(0, |b| Histogram::bounds(b).0);
    Some(Tail {
        estimate: top(counts.bucket_of(r.estimate)?),
        lower,
        upper: top(counts.bucket_of(upper)?),
    })
}

/// The exact `q`-quantile of `sorted` latencies by the nearest rank, `⌈n·q⌉`: a round's own value
/// when its metric is a latency.
#[must_use]
pub fn exact(sorted: &[u64], q: Level) -> Option<u64> {
    let r = ranks(sorted.len(), q, COVERAGE).ok()?;
    sorted.get(r.estimate.checked_sub(1)?).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// T-RQ0.7.4: a p99.9 asked of too few samples → a dash, not a number.
    #[test]
    fn a_p999_of_too_few_operations_is_a_dash() {
        let mut c = Counts::default();
        for i in 0..3_687u64 {
            c.record(1_000 + i);
        }
        assert_eq!(tail(&c, Level::P999), None);
        assert!(tail(&c, Level::P99).is_some());
        c.record(5_000);
        // 3,688, the fewest whose p99.9 has an upper end.
        assert!(tail(&c, Level::P999).is_some());
        assert_eq!(tail(&Counts::default(), Level::P50), None);
    }

    /// AC-RQ0.7.1: a stated quantile is the bucket of its rank, its interval never narrower than the
    /// samples'.
    #[test]
    fn a_tail_reads_its_ranks_buckets() {
        let mut c = Counts::default();
        let samples: Vec<u64> = (1..=1_000u64).map(|i| i * 1_000).collect();
        for &s in &samples {
            c.record(s);
        }
        let t = tail(&c, Level::P50).unwrap();
        let r = ranks(1_000, Level::P50, COVERAGE).unwrap();
        let (lo, est, hi) = (
            samples[r.lower.unwrap() - 1],
            samples[r.estimate - 1],
            samples[r.upper.unwrap() - 1],
        );
        assert!(t.lower <= lo && hi <= t.upper, "{t:?} against {lo}–{hi}");
        assert!(est <= t.estimate && t.estimate <= est + est / 8);
        assert_eq!(exact(&samples, Level::P50), Some(est));
    }

    /// AC-RQ0.7.1: counts move through a worker's reply unchanged; a bucket past the last is refused.
    #[test]
    fn counts_round_trip_and_refuse_a_bucket_past_the_last() {
        let mut c = Counts::default();
        for v in [0u64, 7, 8, 9_999, u64::MAX] {
            c.record(v);
        }
        let back = Counts::from_sparse(&c.sparse()).unwrap();
        assert_eq!(back, c);
        let mut sum = Counts::default();
        sum.add(&c);
        sum.add(&back);
        assert_eq!(sum.count(), 10);
        assert_eq!(
            Counts::from_sparse(&[Bucket {
                bucket: 496,
                count: 1
            }]),
            Err(CountsError::NoSuchBucket(496))
        );
    }
}
