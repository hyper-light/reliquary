//! Where a series of nightly measurements changed, and between which commits (AC-RQ0.7.6, AC-RQ0.7.8):
//! E-Divisive means (Matteson and James, "A Nonparametric Approach for Multiple Change Point Analysis
//! of Multivariate Data", JASA 109(505), 2014), the method MongoDB's CI moved to from fixed thresholds
//! (Daly et al., ICPE '20; research/06 F50).
//!
//! A segment's best split maximizes `Q̂ = (m·n/(m+n))·Ê`, where `Ê` is twice the mean distance between
//! the `m` values before the split and the `n` after, less the mean distance within each side. Splits
//! are found one at a time, each in whichever current segment holds the largest `Q̂`, and kept only while
//! a permutation test finds it significant: the values are shuffled within their segments, the search run
//! again, and the split kept when its `Q̂` is beaten by no more than the level's share of the shuffles.
//!
//! The test's shuffles are drawn from a generator seeded by the series itself, its commits and its
//! values' bits, so the same history gives the same change points every time, on every platform (the
//! arithmetic is IEEE 754 sums and absolute differences, in a fixed order). Hunter (Fleming et al., ICPE
//! '23; research/06 F51) met the permutation test's nondeterminism by replacing it with Student's
//! t-test, which assumes normal metrics; seeding from the history keeps the test distribution-free and
//! makes it deterministic.

use serde::{Deserialize, Serialize};

use crate::rng::SplitMix64;
use crate::rounds::{as_f64, median};

/// One measurement of the series: the commit measured and its value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub commit: String,
    pub value: f64,
}

/// A change point: the series changed between `before` (the last commit of the old segment) and `after`
/// (the first of the new), so `git log before..after` lists the commits in range.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Change {
    /// The first point of the new segment.
    pub index: usize,
    pub before: String,
    pub after: String,
    /// The split's divergence `Q̂`.
    pub divergence: f64,
    /// The permutation test's p-value for it.
    pub p_value: f64,
    /// The medians of the segments on either side, as the change was found.
    pub median_before: f64,
    pub median_after: f64,
}

impl Change {
    /// The commit range, as git writes it.
    #[must_use]
    pub fn range(&self) -> String {
        format!("{}..{}", self.before, self.after)
    }
}

/// Format: the shuffles a split is tested against, 199, and the level it must reach, 5%: `ecp`'s
/// defaults for `e.divisive` (Matteson and James's R package; research/07), whose p-value resolution,
/// 1/200, is a tenth of the level. The level is the one Mantle's measurement design uses.
pub const PERMUTATIONS: usize = 199;
/// See [`PERMUTATIONS`].
pub const LEVEL: f64 = 0.05;

/// Why a series could not be searched.
#[derive(Debug, Clone, PartialEq)]
pub enum ChangesError {
    /// More points than the search's room holds: its distance sums are `8·n²` bytes.
    TooLong { points: usize, most: usize },
    /// A value that is not a finite number.
    NotFinite { index: usize },
}

impl std::fmt::Display for ChangesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooLong { points, most } => {
                write!(f, "{points} points, past the {most} the search holds")
            }
            Self::NotFinite { index } => write!(f, "point {index} is not a finite number"),
        }
    }
}

impl std::error::Error for ChangesError {}

/// The most points a series may hold: one a day over kiln's widest window of a metric's history, 365
/// days (`days` is clamped to 1..=365, kiln `crates/kiln/src/ci/results.rs`), which spans at most 366
/// UTC days; [`nightly`] keeps one point a day. Its distance sums take `8·n²` bytes, about 1 MiB.
pub const MOST_POINTS: usize = 366;

/// One measurement as kiln's metrics API returns it: the commit, the value, and when, in Unix seconds.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Measured {
    pub sha: String,
    pub value: f64,
    pub ts: i64,
}

/// Format: seconds in a UTC day.
pub(crate) const DAY_SECONDS: i64 = 86_400;

/// One point a UTC day, the last measured that day, in time order: a night's scheduled run, or a run
/// made by hand after it, never both.
#[must_use]
pub fn nightly(measured: &[Measured]) -> Vec<Point> {
    let mut sorted: Vec<&Measured> = measured.iter().collect();
    sorted.sort_by_key(|m| m.ts);
    let mut out: Vec<(i64, Point)> = Vec::new();
    for m in sorted {
        let day = m.ts.div_euclid(DAY_SECONDS);
        let point = Point {
            commit: m.sha.clone(),
            value: m.value,
        };
        match out.last_mut() {
            Some((d, p)) if *d == day => *p = point,
            _ => out.push((day, point)),
        }
    }
    out.into_iter().map(|(_, p)| p).collect()
}

/// The fewest points a side of a split holds: the least `k` at which a two-sample permutation test of
/// `k` against `k` can reach [`LEVEL`], `C(2k, k)·LEVEL ≥ 1` (an exact two-sample test's smallest
/// p-value is `1/C(m+n, m)`). Three at 5%.
#[must_use]
pub fn fewest_per_side() -> usize {
    (1..MOST_POINTS)
        .find(|&k| {
            let mut c = 1.0f64;
            // C(2k, k) = Π (k + i) / i over i in 1..=k.
            for i in 1..=k {
                c = c * as_f64(k.saturating_add(i)) / as_f64(i);
            }
            c * LEVEL >= 1.0
        })
        .unwrap_or(MOST_POINTS)
}

/// The change points of `points`, in time order, each kept at [`LEVEL`] against [`PERMUTATIONS`]
/// shuffles.
pub fn changes(points: &[Point]) -> Result<Vec<Change>, ChangesError> {
    if points.len() > MOST_POINTS {
        return Err(ChangesError::TooLong {
            points: points.len(),
            most: MOST_POINTS,
        });
    }
    if let Some(index) = points.iter().position(|p| !p.value.is_finite()) {
        return Err(ChangesError::NotFinite { index });
    }
    let values: Vec<f64> = points.iter().map(|p| p.value).collect();
    let min = fewest_per_side();
    let mut rng = SplitMix64::new(seed(points));
    // Segment bounds, `[start, end)`, in time order.
    let mut cuts: Vec<usize> = vec![0, values.len()];
    let mut found = Vec::new();
    // Each pass keeps one split or stops; a split leaves both sides at least `min` long, so there are
    // at most `len / min` passes.
    for _ in 0..values.len().checked_div(min).unwrap_or(0) {
        let Some(best) = search(&values, &cuts, min) else {
            break;
        };
        let mut beaten = 0usize;
        let mut shuffled = values.clone();
        for _ in 0..PERMUTATIONS {
            for (&s, &e) in cuts.iter().zip(cuts.iter().skip(1)) {
                if let Some(segment) = shuffled.get_mut(s..e) {
                    rng.shuffle(segment);
                }
            }
            if search(&shuffled, &cuts, min).is_some_and(|b| b.q >= best.q) {
                beaten = beaten.saturating_add(1);
            }
            // The next shuffle starts from the series as measured.
            shuffled.clone_from(&values);
        }
        let p_value = as_f64(beaten.saturating_add(1)) / as_f64(PERMUTATIONS.saturating_add(1));
        if p_value > LEVEL {
            break;
        }
        let side = |a: usize, b: usize| {
            let mut v: Vec<f64> = values.get(a..b).unwrap_or(&[]).to_vec();
            v.sort_by(f64::total_cmp);
            median(&v)
        };
        found.push(Change {
            index: best.tau,
            before: points
                .get(best.tau.saturating_sub(1))
                .map(|p| p.commit.clone())
                .unwrap_or_default(),
            after: points
                .get(best.tau)
                .map(|p| p.commit.clone())
                .unwrap_or_default(),
            divergence: best.q,
            p_value,
            median_before: side(best.start, best.tau),
            median_after: side(best.tau, best.kappa),
        });
        let at = cuts.partition_point(|&c| c < best.tau);
        cuts.insert(at, best.tau);
    }
    found.sort_by_key(|c| c.index);
    Ok(found)
}

/// The best split found in a pass.
#[derive(Debug, Clone, Copy)]
struct Best {
    start: usize,
    tau: usize,
    kappa: usize,
    q: f64,
}

/// The split of largest `Q̂` over every current segment, each side at least `min` long.
fn search(values: &[f64], cuts: &[usize], min: usize) -> Option<Best> {
    let mut best: Option<Best> = None;
    for (&s, &e) in cuts.iter().zip(cuts.iter().skip(1)) {
        if let Some(b) = values.get(s..e).and_then(|seg| best_split(seg, min)) {
            let b = Best {
                start: s,
                tau: s.saturating_add(b.tau),
                kappa: s.saturating_add(b.kappa),
                q: b.q,
            };
            if best.is_none_or(|x| b.q > x.q) {
                best = Some(b);
            }
        }
    }
    best
}

/// The split `(τ, κ)` of one segment that maximizes `Q̂(Z[..τ], Z[τ..κ])` (Matteson and James §2.2, with
/// α = 1), from sums of distances kept so each `(τ, κ)` costs a constant: `before[j][t]` is the summed
/// distance from value `j` to the values before position `t`.
fn best_split(seg: &[f64], min: usize) -> Option<Best> {
    let len = seg.len();
    if len < min.checked_add(min)? {
        return None;
    }
    // before[j·len + t] = Σ_{i<t} |seg[i] − seg[j]|, for t ≤ j.
    let mut before = vec![0.0f64; len.checked_mul(len)?];
    for j in 0..len {
        let zj = *seg.get(j)?;
        let row = j.checked_mul(len)?;
        let mut acc = 0.0f64;
        for t in 0..j {
            acc += (seg.get(t)? - zj).abs();
            *before.get_mut(row.checked_add(t)?.checked_add(1)?)? = acc;
        }
    }
    let at = |j: usize, t: usize| {
        j.checked_mul(len)
            .and_then(|r| r.checked_add(t))
            .and_then(|i| before.get(i))
            .copied()
            .unwrap_or(0.0)
    };
    let mut best: Option<Best> = None;
    // Within the first side, Σ_{i<k<τ} |Z_i − Z_k|, grown as τ grows.
    let mut within_x = 0.0f64;
    for k in 0..min {
        within_x += at(k, k);
    }
    for tau in min..=len.checked_sub(min)? {
        let m = as_f64(tau);
        let (mut between, mut within_y) = (0.0f64, 0.0f64);
        for kappa in tau.checked_add(1)?..=len {
            // Value kappa − 1 joins the second side.
            let j = kappa.checked_sub(1)?;
            between += at(j, tau);
            within_y += at(j, j) - at(j, tau);
            let n_count = kappa.checked_sub(tau)?;
            if n_count < min {
                continue;
            }
            let n = as_f64(n_count);
            // Ê: twice the mean distance between the sides less each side's mean distance within, a
            // side's pairs being C(k, 2) = k(k − 1)/2 (Matteson and James §2.1, α = 1).
            let e = 2.0 * between / (m * n)
                - within_x / (m * (m - 1.0) / 2.0)
                - within_y / (n * (n - 1.0) / 2.0);
            let q = m * n / (m + n) * e;
            if best.is_none_or(|b| q > b.q) {
                best = Some(Best {
                    start: 0,
                    tau,
                    kappa,
                    q,
                });
            }
        }
        within_x += at(tau, tau);
    }
    best
}

/// The test's seed: FNV-1a over each point's commit and value bits (Fowler, Noll and Vo; the 64-bit
/// offset basis and prime of its specification, draft-eastlake-fnv), so the shuffles are a function of
/// the history alone.
fn seed(points: &[Point]) -> u64 {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;
    let mut h = OFFSET;
    for p in points {
        for b in p
            .commit
            .bytes()
            .chain(p.value.to_bits().to_le_bytes())
            .chain(std::iter::once(0xff))
        {
            h = (h ^ u64::from(b)).wrapping_mul(PRIME);
        }
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    fn series(rng: &mut SplitMix64, values: impl Iterator<Item = f64>) -> Vec<Point> {
        values
            .enumerate()
            .map(|(i, v)| Point {
                commit: format!("c{i:03}"),
                value: v * (1.0 + 0.01 * (rng.unit() - 0.5)),
            })
            .collect()
    }

    /// T-RQ0.7.10 (AC-RQ0.7.8): the detector run twice over one series with a planted shift → the same
    /// change point both times, at the planted commit range.
    #[test]
    fn a_planted_shift_is_found_at_its_commits_the_same_every_time() {
        let mut rng = SplitMix64::new(31);
        // Sixty nights at 10 ms, then the change at night 40: 12 ms.
        let points = series(&mut rng, (0..60).map(|i| if i < 40 { 10e6 } else { 12e6 }));
        let first = changes(&points).unwrap();
        let second = changes(&points).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.len(), 1, "{first:?}");
        assert_eq!(first[0].index, 40);
        assert_eq!(first[0].range(), "c039..c040");
        assert!(first[0].p_value <= LEVEL);
        assert!(first[0].median_after > first[0].median_before);
    }

    /// AC-RQ0.7.6: a steady series has no change point; two shifts are both found.
    #[test]
    fn a_steady_series_has_none_and_two_shifts_give_two() {
        let mut rng = SplitMix64::new(37);
        let steady = series(&mut rng, (0..80).map(|_| 5e6));
        assert_eq!(changes(&steady).unwrap(), vec![]);
        let two = series(
            &mut rng,
            (0..90).map(|i| match i {
                0..30 => 5e6,
                30..60 => 7e6,
                _ => 4e6,
            }),
        );
        let found = changes(&two).unwrap();
        let at: Vec<usize> = found.iter().map(|c| c.index).collect();
        assert_eq!(at, vec![30, 60], "{found:?}");
    }

    /// AC-RQ0.7.8: the search agrees with Q̂ computed from its definition, for every split.
    #[test]
    fn the_best_split_is_the_definitions_maximum() {
        let mut rng = SplitMix64::new(41);
        let seg: Vec<f64> = (0..24).map(|i| f64::from(i / 9) + rng.unit()).collect();
        let min = fewest_per_side();
        let q = |x: &[f64], y: &[f64]| {
            let (m, n) = (x.len() as f64, y.len() as f64);
            let mut between = 0.0;
            for a in x {
                for b in y {
                    between += (a - b).abs();
                }
            }
            let within = |v: &[f64]| {
                let mut s = 0.0;
                for i in 0..v.len() {
                    for k in i + 1..v.len() {
                        s += (v[i] - v[k]).abs();
                    }
                }
                s / (v.len() as f64 * (v.len() as f64 - 1.0) / 2.0)
            };
            m * n / (m + n) * (2.0 * between / (m * n) - within(x) - within(y))
        };
        let mut best = (f64::MIN, 0, 0);
        for tau in min..=seg.len() - min {
            for kappa in tau + min..=seg.len() {
                let v = q(&seg[..tau], &seg[tau..kappa]);
                if v > best.0 {
                    best = (v, tau, kappa);
                }
            }
        }
        let got = best_split(&seg, min).unwrap();
        assert_eq!((got.tau, got.kappa), (best.1, best.2));
        assert!((got.q - best.0).abs() < 1e-9 * best.0.abs());
    }

    /// AC-RQ0.7.6: three a side is the fewest at which a split can reach 5%; a series too long or not a
    /// number is refused, typed.
    #[test]
    fn the_sides_bound_and_refusals() {
        assert_eq!(fewest_per_side(), 3);
        let long: Vec<Point> = (0..=MOST_POINTS)
            .map(|i| Point {
                commit: i.to_string(),
                value: 1.0,
            })
            .collect();
        assert!(matches!(changes(&long), Err(ChangesError::TooLong { .. })));
        let nan = vec![Point {
            commit: "a".into(),
            value: f64::NAN,
        }];
        assert_eq!(changes(&nan), Err(ChangesError::NotFinite { index: 0 }));
        assert_eq!(changes(&[]).unwrap(), vec![]);
    }

    /// AC-RQ0.7.6: a nightly series keeps one point a day, the day's last, in time order.
    #[test]
    fn a_nightly_series_keeps_each_days_last_point() {
        let m = |sha: &str, value: f64, ts: i64| Measured {
            sha: sha.into(),
            value,
            ts,
        };
        let got = nightly(&[
            m("b", 2.0, 86_400 + 10),
            m("a", 1.0, 3_600),
            m("c", 3.0, 86_400 + 7_200),
            m("d", 4.0, 3 * 86_400),
        ]);
        let commits: Vec<&str> = got.iter().map(|p| p.commit.as_str()).collect();
        assert_eq!(commits, vec!["a", "c", "d"]);
    }

    /// AC-RQ0.7.8: the seed is the history's: a different history draws different shuffles.
    #[test]
    fn the_seed_is_a_function_of_the_history() {
        let a = vec![Point {
            commit: "a".into(),
            value: 1.0,
        }];
        let mut b = a.clone();
        assert_eq!(seed(&a), seed(&b));
        b[0].value = 1.0000001;
        assert_ne!(seed(&a), seed(&b));
    }
}
