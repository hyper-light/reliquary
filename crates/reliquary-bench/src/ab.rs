//! A|B: two builds, or two settings of one, measured in interleaved, randomized units on one machine in
//! one session, and a verdict only when their intervals separate (AC-RQ0.7.2).
//!
//! The order is Abedi and Brecht's randomized multiple interleaved trials (ICPE '17): each cycle runs
//! one unit of each arm, in an order drawn for that cycle, so drift on the machine falls on both arms
//! alike and no arm always runs first. Each arm's units are judged on their own as a point is
//! (`rounds::judge`); the verdict compares the arms' median intervals and nothing else. Means are never
//! computed, so no verdict can come from them.
//!
//! Requiring two 95% intervals not to overlap is a test of a difference at a level well below 5%
//! (Schenker and Gentleman, "On Judging the Significance of Differences by Examining the Overlap
//! Between Confidence Intervals", The American Statistician 55(3), 2001), so an A|A run reports a
//! difference less often than the stated 5% (AC-RQ0.7.3); `tests` hold it there by a binomial test.

use serde::Serialize;

use crate::rng::SplitMix64;
use crate::rounds::{Judgment, Order};

/// One of the two arms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Arm {
    A,
    B,
}

impl Arm {
    /// The arm's letter.
    #[must_use]
    pub fn letter(self) -> &'static str {
        match self {
            Self::A => "A",
            Self::B => "B",
        }
    }
}

/// The order of one cycle's two units: a fair coin, the low bit of a draw.
#[must_use]
pub fn cycle_order(rng: &mut SplitMix64) -> (Arm, Arm) {
    if rng.next_u64() & 1 == 0 {
        (Arm::A, Arm::B)
    } else {
        (Arm::B, Arm::A)
    }
}

/// Which way a metric is better.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Better {
    Higher,
    Lower,
}

/// The verdict of an A|B.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Verdict {
    #[serde(rename = "A better")]
    ABetter,
    #[serde(rename = "B better")]
    BBetter,
    #[serde(rename = "not separated")]
    NotSeparated,
}

impl Verdict {
    /// The verdict's words, as AC-RQ0.7.2 has them.
    #[must_use]
    pub fn words(self) -> &'static str {
        match self {
            Self::ABetter => "A better",
            Self::BBetter => "B better",
            Self::NotSeparated => "not separated",
        }
    }
}

/// One arm's side of a comparison: its single state's median and interval, where it has them.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Side {
    pub units: usize,
    pub order: Order,
    pub states: usize,
    pub median: Option<f64>,
    pub interval: Option<(f64, f64)>,
}

/// An A|B's verdict, both sides, and why.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Comparison {
    pub verdict: Verdict,
    pub better: Better,
    pub a: Side,
    pub b: Side,
    pub why: String,
}

fn side(j: &Judgment) -> Side {
    let one = (j.states.len() == 1).then(|| j.states.first()).flatten();
    Side {
        units: j.rounds,
        order: j.order,
        states: j.states.len(),
        median: one.map(|s| s.median),
        interval: one.and_then(|s| s.interval),
    }
}

/// Why an arm gives no interval to compare, if it gives none.
fn missing(arm: Arm, j: &Judgment) -> Option<String> {
    let name = arm.letter();
    if j.order != Order::Independent {
        return Some(format!(
            "{name}'s units are {} in time, so no interval is stated",
            j.order.word()
        ));
    }
    if j.states.len() != 1 {
        return Some(format!(
            "{name}'s units fall in {} states, each shown with its share; a verdict compares one \
             state with one",
            j.states.len()
        ));
    }
    if j.states.first().is_none_or(|s| s.interval.is_none()) {
        return Some(format!(
            "{name}'s state has too few units for a 95% interval of its median"
        ));
    }
    None
}

/// Compares two arms' judged units: "A better" or "B better" only when both arms have one independent
/// state with an interval and the intervals do not meet; otherwise "not separated", with why.
#[must_use]
pub fn compare(a: &Judgment, b: &Judgment, better: Better) -> Comparison {
    let (sa, sb) = (side(a), side(b));
    let why_missing: Vec<String> = [missing(Arm::A, a), missing(Arm::B, b)]
        .into_iter()
        .flatten()
        .collect();
    let not = |why: String, sa: Side, sb: Side| Comparison {
        verdict: Verdict::NotSeparated,
        better,
        a: sa,
        b: sb,
        why,
    };
    if !why_missing.is_empty() {
        return not(why_missing.join("; "), sa, sb);
    }
    let (Some((alo, ahi)), Some((blo, bhi))) = (sa.interval, sb.interval) else {
        return not("an arm has no interval".to_owned(), sa, sb);
    };
    // Which arm's values lie wholly above the other's, if either does.
    let higher = if ahi < blo {
        Some(Arm::B)
    } else if bhi < alo {
        Some(Arm::A)
    } else {
        None
    };
    let Some(higher) = higher else {
        return not(
            format!("the intervals meet: A [{alo:.6e}, {ahi:.6e}], B [{blo:.6e}, {bhi:.6e}]"),
            sa,
            sb,
        );
    };
    let verdict = match (higher, better) {
        (Arm::A, Better::Higher) | (Arm::B, Better::Lower) => Verdict::ABetter,
        (Arm::B, Better::Higher) | (Arm::A, Better::Lower) => Verdict::BBetter,
    };
    let why = format!(
        "{}'s interval lies wholly above {}'s: A [{alo:.6e}, {ahi:.6e}], B [{blo:.6e}, {bhi:.6e}]; {} is better",
        higher.letter(),
        if higher == Arm::A { "B" } else { "A" },
        match better {
            Better::Higher => "higher",
            Better::Lower => "lower",
        }
    );
    Comparison {
        verdict,
        better,
        a: sa,
        b: sb,
        why,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rounds::{binomial_cdf, judge};

    /// `n` independent draws, each the sum of four uniform draws scaled to `spread` around `centre`.
    fn bell(rng: &mut SplitMix64, n: usize, centre: f64, spread: f64) -> Vec<f64> {
        (0..n)
            .map(|_| centre + spread * ((0..4).map(|_| rng.unit()).sum::<f64>() - 2.0))
            .collect()
    }

    /// T-RQ0.7.6: a planted slowdown wider than the point's interval → "B worse", intervals separated.
    #[test]
    fn a_planted_slowdown_wider_than_the_interval_is_a_better() {
        let mut rng = SplitMix64::new(1);
        // Time an operation (lower is better): B's planted slowdown is 8%, its interval about ±1%.
        let a = judge(&bell(&mut rng, 28, 1_000.0, 20.0));
        let b = judge(&bell(&mut rng, 28, 1_080.0, 20.0));
        let c = compare(&a, &b, Better::Lower);
        assert_eq!(c.verdict, Verdict::ABetter, "{}", c.why);
        assert_eq!(c.verdict.words(), "A better");
        let ((_, ahi), (blo, _)) = (c.a.interval.unwrap(), c.b.interval.unwrap());
        assert!(ahi < blo);
        // As a throughput (higher is better), the same values make B better.
        assert_eq!(compare(&a, &b, Better::Higher).verdict, Verdict::BBetter);
    }

    /// AC-RQ0.7.2: never a verdict from means alone: means far apart, medians' intervals meeting.
    #[test]
    fn means_far_apart_with_meeting_intervals_are_not_separated() {
        let mut rng = SplitMix64::new(2);
        let mut a = bell(&mut rng, 28, 100.0, 2.0);
        // One round of A caught a stall a hundred times longer: A's mean moves by a factor of 4.5, its
        // median by nothing.
        a[13] = 10_000.0;
        let b = bell(&mut rng, 28, 100.0, 2.0);
        let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
        assert!(mean(&a) > 3.0 * mean(&b));
        let c = compare(&judge(&a), &judge(&b), Better::Lower);
        assert_eq!(c.verdict, Verdict::NotSeparated, "{}", c.why);
        assert!(c.why.contains("intervals meet"), "{}", c.why);
        assert!(c.a.interval.is_some() && c.b.interval.is_some());
    }

    /// AC-RQ0.7.2: an arm without an interval gives no verdict, and says why.
    #[test]
    fn an_arm_in_order_or_in_two_states_gives_no_verdict() {
        let mut rng = SplitMix64::new(4);
        let steady = judge(&bell(&mut rng, 28, 1_000.0, 5.0));
        let drift: Vec<f64> = (0..28).map(|i| 1_000.0 + 10.0 * i as f64).collect();
        let c = compare(&judge(&drift), &steady, Better::Lower);
        assert_eq!(c.verdict, Verdict::NotSeparated);
        assert!(c.why.contains("A's units are persistent"), "{}", c.why);
        // Ten of 28 units slowed fourfold, at places drawn at random so the units stay independent.
        let mut two = bell(&mut rng, 28, 1_000.0, 5.0);
        let mut places: Vec<usize> = (0..28).collect();
        rng.shuffle(&mut places);
        for &i in &places[..10] {
            two[i] *= 4.0;
        }
        let judged = judge(&two);
        assert_eq!(
            judged.order,
            Order::Independent,
            "the premise: independent units"
        );
        let c = compare(&steady, &judged, Better::Lower);
        assert_eq!(c.verdict, Verdict::NotSeparated);
        assert!(c.why.contains("B's units fall in 2 states"), "{}", c.why);
        let few = judge(&bell(&mut rng, 5, 1_000.0, 5.0));
        assert_eq!(
            compare(&few, &steady, Better::Lower).verdict,
            Verdict::NotSeparated
        );
    }

    /// T-RQ0.7.7 (AC-RQ0.7.3): A|A, repeated → verdicts of a difference within the stated 5% level, by a
    /// one-sided binomial test at 5%.
    #[test]
    fn an_a_a_run_reports_a_difference_within_five_percent() {
        let mut rng = SplitMix64::new(7);
        let runs = 2_000usize;
        let mut differences = 0usize;
        let mut judged = 0usize;
        let mut kinds = [0usize; 3];
        for k in 0..runs {
            // Three shapes of one distribution, both arms drawn alike: bell, skewed, two-state.
            let draw = |rng: &mut SplitMix64| -> Vec<f64> {
                match k % 3 {
                    0 => bell(rng, 28, 1_000.0, 10.0),
                    1 => (0..28)
                        .map(|_| 1_000.0 - 50.0 * (1.0 - rng.unit()).ln())
                        .collect(),
                    _ => (0..28)
                        .map(|_| {
                            if rng.below(5) == 0 {
                                1_500.0 + rng.unit()
                            } else {
                                1_000.0 + rng.unit()
                            }
                        })
                        .collect(),
                }
            };
            let (a, b) = (draw(&mut rng), draw(&mut rng));
            let c = compare(&judge(&a), &judge(&b), Better::Lower);
            if c.a.interval.is_some() && c.b.interval.is_some() {
                judged += 1;
            }
            if c.verdict != Verdict::NotSeparated {
                differences += 1;
                kinds[k % 3] += 1;
            }
        }
        // Reject "rate ≤ 5%" when P(X ≥ differences | 5%) < 5%: the test must not reject.
        let tail = if differences == 0 {
            1.0
        } else {
            1.0 - binomial_cdf(differences - 1, runs, 0.05)
        };
        assert!(
            tail >= 0.05,
            "{differences} of {runs} A|A runs reported a difference ({kinds:?})"
        );
        // Non-vacuity: the rule was put to work: the bell-shaped and skewed runs, two thirds of all,
        // mostly had an interval on both arms, so a "not separated" there is the intervals meeting.
        assert!(
            judged > runs / 2,
            "{judged} of {runs} runs had both intervals"
        );
    }

    /// (AC-RQ0.7.2)
    #[test]
    fn a_cycles_order_is_drawn_and_replays() {
        // AC-RQ0.7.2: interleaved, randomized order, replayed from its seed.
        let (mut r1, mut r2) = (SplitMix64::new(3), SplitMix64::new(3));
        let orders: Vec<(Arm, Arm)> = (0..64).map(|_| cycle_order(&mut r1)).collect();
        let again: Vec<(Arm, Arm)> = (0..64).map(|_| cycle_order(&mut r2)).collect();
        assert_eq!(orders, again);
        let a_first = orders.iter().filter(|o| o.0 == Arm::A).count();
        assert!((16..=48).contains(&a_first), "{a_first}");
        assert!(orders.iter().all(|o| o.0 != o.1));
    }
}
