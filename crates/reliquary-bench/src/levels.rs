//! How many builds, process starts and rounds a point is measured over (AC-RQ0.7.7): Kalibera and
//! Jones's dimensioning ("Rigorous Benchmarking in Reasonable Time", ISMM '13, §9; Mantle's
//! research/21 §2.4–§2.5). A dimensioning experiment repeats every level a stated number of times;
//! each level's own variance `T_i²` is estimated from it (their Eq. 1 and 2), and each level but the
//! top is repeated `r_i = ⌈√((c_{i+1}/c_i)·(T_i²/T_{i+1}²))⌉` times (Eq. 3), `c_i` the time a
//! repetition at level `i` adds by itself. The top level is repeated until its interval is narrow
//! enough. A level whose `T_i²` is not above zero adds no variation and is not repeated.
//!
//! Repetition below the top is for cost alone: "Including other levels (n − 1 and below) is purely for
//! optimisation of the experimental time, as repetition there is never needed for correctness" (KJ13
//! §9.1). So a dimensioning estimate that is off changes what a point costs, never what its interval
//! means: the interval is stated from the top level's values only (KJ13 Eq. 4), each of them a median
//! over the units below it.

use serde::Serialize;

use crate::rounds::{MAX_ROUNDS, as_f64};

/// A level of the experiment, from the lowest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Level {
    /// A counted number of operations in one process.
    Rounds,
    /// A fresh process of the build, its environment's size drawn anew (Mytkowicz et al., ASPLOS '09).
    ProcessStarts,
    /// A separate build of the same source.
    Builds,
}

impl Level {
    /// The level's name in a record.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Rounds => "rounds",
            Self::ProcessStarts => "process starts",
            Self::Builds => "builds",
        }
    }
}

/// Why the levels could not be dimensioned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LevelsError {
    /// The values are not `∏ r_i` of them.
    Shape { values: usize, expected: usize },
    /// A level repeated fewer than twice has no variance to estimate.
    Unrepeated { level: usize },
    /// A level's cost is not a positive finite number of nanoseconds.
    Cost { level: usize },
    /// No level was given.
    Empty,
}

impl std::fmt::Display for LevelsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Shape { values, expected } => {
                write!(
                    f,
                    "{values} values where the levels' counts make {expected}"
                )
            }
            Self::Unrepeated { level } => write!(
                f,
                "level {level} was repeated fewer than twice, so its variance cannot be estimated"
            ),
            Self::Cost { level } => write!(f, "level {level}'s cost is not a positive time"),
            Self::Empty => f.write_str("no levels"),
        }
    }
}

impl std::error::Error for LevelsError {}

/// A balanced nested experiment: `counts[0]` values at the lowest level within each unit of the next,
/// and so on up; `values` in time order, the lowest level varying fastest.
#[derive(Debug, Clone, PartialEq)]
pub struct Nested {
    pub counts: Vec<usize>,
    pub values: Vec<f64>,
}

impl Nested {
    fn check(&self) -> Result<(), LevelsError> {
        if self.counts.is_empty() {
            return Err(LevelsError::Empty);
        }
        for (i, &r) in self.counts.iter().enumerate() {
            if r <= 1 {
                return Err(LevelsError::Unrepeated {
                    level: i.saturating_add(1),
                });
            }
        }
        let expected = self
            .counts
            .iter()
            .try_fold(1usize, |a, &r| a.checked_mul(r))
            .unwrap_or(usize::MAX);
        if self.values.len() != expected {
            return Err(LevelsError::Shape {
                values: self.values.len(),
                expected,
            });
        }
        Ok(())
    }
}

/// `S_i²` for every level, from the lowest (KJ13 Eq. 1): within each unit of level `i + 1`, the
/// variance (over `r_i − 1`) of the means of its `r_i` units of level `i`, averaged over the units of
/// level `i + 1`.
pub fn level_variances(nested: &Nested) -> Result<Vec<f64>, LevelsError> {
    nested.check()?;
    let mut means = nested.values.clone();
    let mut out = Vec::with_capacity(nested.counts.len());
    for &r in &nested.counts {
        let mut variance_sum = 0.0f64;
        let mut groups = 0usize;
        let mut next = Vec::with_capacity(means.len().checked_div(r).unwrap_or(0));
        for group in means.chunks(r) {
            let centre = group.iter().sum::<f64>() / as_f64(group.len());
            let squares: f64 = group.iter().map(|y| (y - centre) * (y - centre)).sum();
            variance_sum += squares / as_f64(r.saturating_sub(1));
            groups = groups.saturating_add(1);
            next.push(centre);
        }
        out.push(variance_sum / as_f64(groups.max(1)));
        means = next;
    }
    Ok(out)
}

/// `T_i²` from `S_i²` (KJ13 Eq. 2): `T_1² = S_1²`, `T_i² = S_i² − S_{i−1}²/r_{i−1}`.
#[must_use]
pub fn own_variances(s: &[f64], counts: &[usize]) -> Vec<f64> {
    s.iter()
        .enumerate()
        .map(|(i, &si)| match i.checked_sub(1) {
            None => si,
            Some(below) => {
                let prev = s.get(below).copied().unwrap_or(0.0);
                let r = counts.get(below).copied().unwrap_or(1).max(1);
                si - prev / as_f64(r)
            }
        })
        .collect()
}

/// How many times level `i` (from the lowest) is repeated within each unit of level `i + 1` (KJ13
/// Eq. 3), for each level but the top: `None` where level `i + 1` adds no variation (its `T²` is not
/// above zero), so it is not repeated and the repetition stays at level `i` and below; at least one.
pub fn repetitions(t: &[f64], costs: &[f64]) -> Result<Vec<Option<usize>>, LevelsError> {
    for (i, &c) in costs.iter().enumerate() {
        if !(c.is_finite() && c > 0.0) {
            return Err(LevelsError::Cost {
                level: i.saturating_add(1),
            });
        }
    }
    let mut out = Vec::new();
    for i in 0..t.len().saturating_sub(1) {
        let (ti, tn) = (
            t.get(i).copied().unwrap_or(0.0),
            t.get(i.saturating_add(1)).copied().unwrap_or(0.0),
        );
        let (ci, cn) = (
            costs.get(i).copied().unwrap_or(1.0),
            costs.get(i.saturating_add(1)).copied().unwrap_or(1.0),
        );
        if tn <= 0.0 {
            out.push(None);
            continue;
        }
        let root = ((cn / ci) * (ti.max(0.0) / tn)).sqrt();
        // ⌈√ratio⌉, at least one, and at most MAX_ROUNDS: a count past that only moves cost from the
        // level above to this one, never what the interval means (KJ13 §9.1).
        out.push(Some(
            (1..=MAX_ROUNDS)
                .find(|&k| as_f64(k) >= root)
                .unwrap_or(MAX_ROUNDS),
        ));
    }
    Ok(out)
}

/// How a level is repeated in the real experiment.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Count {
    /// A fixed number of times within each unit above.
    Fixed { times: usize },
    /// The top level: repeated until the point's rounds are settled, between these counts.
    UntilSettled { min: usize, max: usize },
}

/// One level of a point's plan, with its count and why.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LevelPlan {
    pub level: Level,
    pub count: Count,
    /// `T_i²`, the variance this level adds by itself, from the dimensioning experiment; none where
    /// the level was not repeated there.
    pub own_variance: Option<f64>,
    /// `c_i`, nanoseconds a repetition at this level adds by itself.
    pub cost_ns: Option<f64>,
    pub why: String,
}

/// What a dimensioning experiment measured, for [`plan`]: the levels repeated (from the lowest), their
/// values and each level's cost.
#[derive(Debug, Clone, PartialEq)]
pub struct Dimensioning {
    pub levels: Vec<Level>,
    pub nested: Nested,
    pub costs_ns: Vec<f64>,
}

/// The levels of the real experiment from a dimensioning experiment (KJ13 §9.3): each level but the top
/// at its Eq. 3 count, the top repeated until settled between `min` and `max`, and levels above the
/// first that adds no variation dropped, each with why.
pub fn plan(dim: &Dimensioning, min: usize, max: usize) -> Result<Vec<LevelPlan>, LevelsError> {
    let s = level_variances(&dim.nested)?;
    let t = own_variances(&s, &dim.nested.counts);
    let r = repetitions(&t, &dim.costs_ns)?;
    // The top: the highest level whose own variance is above zero; above it nothing varies.
    let top = (0..t.len())
        .rev()
        .find(|&i| t.get(i).is_some_and(|&v| v > 0.0))
        .unwrap_or(0);
    let mut out = Vec::new();
    for (i, &level) in dim.levels.iter().enumerate() {
        let own = t.get(i).copied();
        let cost = dim.costs_ns.get(i).copied();
        if i > top {
            let once = 1;
            out.push(LevelPlan {
                level,
                count: Count::Fixed { times: once },
                own_variance: own,
                cost_ns: cost,
                why: format!(
                    "not repeated: its own variance, {:.3e}, is not above zero, so it adds no \
                     variation to the levels below (KJ13 §9.2)",
                    own.unwrap_or(0.0)
                ),
            });
            continue;
        }
        if i == top {
            out.push(LevelPlan {
                level,
                count: Count::UntilSettled { min, max },
                own_variance: own,
                cost_ns: cost,
                why: format!(
                    "the top level with variation of its own ({:.3e}): repeated until its values are \
                     independent and every state's median interval is within the precision, from {min} \
                     (the fewest at which order in time is tested) to {max} (the fewest at which a \
                     second state holds its own interval); the interval is stated from its values only \
                     (KJ13 Eq. 4)",
                    own.unwrap_or(0.0)
                ),
            });
            continue;
        }
        let times = r.get(i).copied().flatten().unwrap_or(1);
        let above = dim.levels.get(i.saturating_add(1)).copied();
        out.push(LevelPlan {
            level,
            count: Count::Fixed { times },
            own_variance: own,
            cost_ns: cost,
            why: format!(
                "⌈√((c₊/c)·(T²/T₊²))⌉ = ⌈√(({:.3e}/{:.3e})·({:.3e}/{:.3e}))⌉ = {times} a unit of {} \
                 (KJ13 Eq. 3): {}",
                dim.costs_ns
                    .get(i.saturating_add(1))
                    .copied()
                    .unwrap_or(0.0),
                cost.unwrap_or(0.0),
                own.unwrap_or(0.0).max(0.0),
                t.get(i.saturating_add(1)).copied().unwrap_or(0.0),
                above.map_or("the level above", Level::name),
                if times == 1 {
                    "its variation is small against the level above's at their costs, so the level \
                     above is repeated instead"
                } else {
                    "repeating it is cheaper than repeating the level above for the same precision"
                }
            ),
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::SplitMix64;

    /// AC-RQ0.7.7: Eq. 3 reproduces Kalibera and Jones's own Table 5 (Mantle's research/21 §2.4 check).
    #[test]
    fn repetitions_reproduce_kalibera_and_jones_table_5() {
        // bloat6: c1 = 35.5 s, c2 = 110.0 s, √T1² = 14.0, √T2² = 2.7 → √(3.10·26.9) = 9.13 → 10.
        let r = repetitions(&[14.0 * 14.0, 2.7 * 2.7], &[35.5, 110.0]).unwrap();
        assert_eq!(r, vec![Some(10)]);
        // A level above that adds nothing: the level below carries the repetition.
        assert_eq!(repetitions(&[4.0, 0.0], &[1.0, 9.0]).unwrap(), vec![None]);
        // A level that adds nothing itself is run once.
        assert_eq!(
            repetitions(&[0.0, 4.0], &[1.0, 9.0]).unwrap(),
            vec![Some(1)]
        );
        assert_eq!(
            repetitions(&[1.0, 1.0], &[0.0, 1.0]),
            Err(LevelsError::Cost { level: 1 })
        );
    }

    /// AC-RQ0.7.7: Eq. 1 and 2 recover the variance each level adds, from a seeded nested experiment.
    #[test]
    fn level_variances_recover_each_levels_own_variance() {
        // 40 processes of 40 rounds: a process effect of sd 3 and a round effect of sd 1, both uniform
        // draws scaled to those deviations.
        let mut rng = SplitMix64::new(17);
        let uniform_sd = |rng: &mut SplitMix64, sd: f64| (rng.unit() - 0.5) * sd * 12f64.sqrt();
        let (rounds, processes) = (40usize, 40usize);
        let mut values = Vec::new();
        for _ in 0..processes {
            let p = uniform_sd(&mut rng, 3.0);
            for _ in 0..rounds {
                values.push(100.0 + p + uniform_sd(&mut rng, 1.0));
            }
        }
        let nested = Nested {
            counts: vec![rounds, processes],
            values,
        };
        let s = level_variances(&nested).unwrap();
        let t = own_variances(&s, &nested.counts);
        assert!((t[0] - 1.0).abs() < 0.15, "rounds: {}", t[0]);
        assert!((t[1] - 9.0).abs() < 4.0, "processes: {}", t[1]);
    }

    /// T-RQ0.7.9: variation between process starts and not between rounds → process starts are
    /// repeated, not rounds, and the plan says so.
    #[test]
    fn variation_between_process_starts_repeats_process_starts_not_rounds() {
        let mut rng = SplitMix64::new(23);
        let mut values = Vec::new();
        for _ in 0..10 {
            // Each process runs 20% faster or slower than another; its rounds agree to 0.01%.
            let process = 1_000.0 * (1.0 + 0.2 * rng.unit());
            for _ in 0..10 {
                values.push(process * (1.0 + 1e-4 * rng.unit()));
            }
        }
        let dim = Dimensioning {
            levels: vec![Level::Rounds, Level::ProcessStarts],
            nested: Nested {
                counts: vec![10, 10],
                values,
            },
            // A round costs 2 ms; a process start, 20 ms.
            costs_ns: vec![2e6, 2e7],
        };
        let p = plan(&dim, 10, 28).unwrap();
        assert_eq!(p[0].level, Level::Rounds);
        assert_eq!(p[0].count, Count::Fixed { times: 1 });
        assert!(
            p[0].why.contains("level above is repeated instead"),
            "{}",
            p[0].why
        );
        assert_eq!(p[1].level, Level::ProcessStarts);
        assert_eq!(p[1].count, Count::UntilSettled { min: 10, max: 28 });
    }

    /// AC-RQ0.7.7: variation only between rounds → one process, its rounds repeated until settled.
    #[test]
    fn variation_only_between_rounds_keeps_one_process() {
        let mut rng = SplitMix64::new(29);
        // Every process alike; each round varies by 5%.
        let values: Vec<f64> = (0..100)
            .map(|_| 1_000.0 * (1.0 + 0.05 * rng.unit()))
            .collect();
        let dim = Dimensioning {
            levels: vec![Level::Rounds, Level::ProcessStarts],
            nested: Nested {
                counts: vec![10, 10],
                values,
            },
            costs_ns: vec![2e6, 2e7],
        };
        let p = plan(&dim, 10, 28).unwrap();
        // The processes' own variance estimates at or below zero: they are not repeated.
        if p[1].own_variance.unwrap() <= 0.0 {
            assert_eq!(p[0].count, Count::UntilSettled { min: 10, max: 28 });
            assert_eq!(p[1].count, Count::Fixed { times: 1 });
            assert!(p[1].why.contains("not repeated"));
        } else {
            // Sampling noise left the processes a little variance: rounds still carry most of it.
            assert!(
                matches!(p[0].count, Count::Fixed { times } if times > 1),
                "{p:?}"
            );
        }
    }

    /// AC-RQ0.7.7: a malformed experiment is refused, typed.
    #[test]
    fn malformed_experiments_are_refused() {
        let bad = Nested {
            counts: vec![2, 2],
            values: vec![1.0; 3],
        };
        assert_eq!(
            level_variances(&bad),
            Err(LevelsError::Shape {
                values: 3,
                expected: 4
            })
        );
        let once = Nested {
            counts: vec![1, 2],
            values: vec![1.0; 2],
        };
        assert_eq!(
            level_variances(&once),
            Err(LevelsError::Unrepeated { level: 1 })
        );
        assert_eq!(
            level_variances(&Nested {
                counts: vec![],
                values: vec![]
            }),
            Err(LevelsError::Empty)
        );
    }

    /// (AC-RQ0.7.7)
    #[test]
    fn counts_are_ceilings_held_to_the_judged_most() {
        // AC-RQ0.7.7: Eq. 3's count is the ceiling, at least one and at most MAX_ROUNDS.
        assert_eq!(
            repetitions(&[4.0, 1.0], &[1.0, 1.0]).unwrap(),
            vec![Some(2)]
        );
        assert_eq!(
            repetitions(&[1e12, 1.0], &[1.0, 1.0]).unwrap(),
            vec![Some(MAX_ROUNDS)]
        );
        assert_eq!(
            repetitions(&[1e-9, 1.0], &[1.0, 1.0]).unwrap(),
            vec![Some(1)]
        );
    }
}
