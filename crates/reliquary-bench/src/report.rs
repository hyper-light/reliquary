//! A point's report, as Mantle's design §6 has it (AC-RQ0.7.1): for each state a row with its median,
//! the wider side of the median's interval as a share of the median, its units out of the point's, the
//! share's interval when there are two states, and latency quantiles over the state's own operations;
//! on the first row, whether the units are ordered in time. A state without an interval, and a quantile
//! its operations cannot support, show a dash. No mean and no coefficient of variation is reported.

use std::fmt::Write as _;

use crate::ab::Comparison;
use crate::driver::{ArmOutcome, Outcome};
use crate::levels::Count;
use crate::load::Standing;
use crate::record::Host;
use crate::tails::Tail;

/// A dash: nothing stated.
pub const DASH: &str = "—";

/// Format: the SI units a duration is written in, each with its nanoseconds, largest first.
const UNITS: &[(f64, &str)] = &[(1e9, "s"), (1e6, "ms"), (1e3, "µs")];

/// Format: a percentage's whole.
pub(crate) const PERCENT: u64 = 100;

fn percent(share: f64) -> f64 {
    share * crate::rounds::u64_as_f64(PERCENT)
}

/// A count of nanoseconds in the largest unit it reaches one of.
#[must_use]
pub fn duration(ns: u64) -> String {
    let v = crate::rounds::u64_as_f64(ns);
    UNITS.iter().find(|(scale, _)| v >= *scale).map_or_else(
        || format!("{ns} ns"),
        |(scale, unit)| format!("{:.3} {unit}", v / scale),
    )
}

/// A tail: its estimate, with its interval only where the units were independent.
fn tail(t: Option<&Tail>, independent: bool) -> String {
    t.map_or_else(
        || DASH.to_owned(),
        |t| {
            if independent {
                format!(
                    "{} [{}–{}]",
                    duration(t.estimate),
                    duration(t.lower),
                    duration(t.upper)
                )
            } else {
                format!("{} [{DASH}]", duration(t.estimate))
            }
        },
    )
}

/// The report of a measured point or A|B.
#[must_use]
pub fn render(outcome: &Outcome, host: &Host, date: &str) -> String {
    let mut out = String::new();
    let r = &outcome.request;
    let _ = writeln!(
        out,
        "reliquary bench: {}{}; {}; {}",
        r.a.bench,
        if r.a.settings.is_empty() {
            String::new()
        } else {
            format!(" ({})", r.a.settings.join(", "))
        },
        r.metric.describe(),
        outcome.size.load_why
    );
    let _ = writeln!(
        out,
        "{date}; {} {}, {}, {} cores, {} memory{}",
        host.os,
        host.arch,
        host.cpu.as_deref().unwrap_or("CPU unnamed"),
        host.cores
            .map_or_else(|| DASH.to_owned(), |c| c.to_string()),
        host.memory_bytes
            .map_or_else(|| DASH.to_owned(), |b| format!("{b} bytes")),
        host.machine
            .as_deref()
            .map_or_else(String::new, |m| format!("; machine {m}"))
    );
    match &outcome.standing {
        Standing::OfRecord => {
            let _ = writeln!(out, "of record");
        }
        Standing::NotOfRecord { why } => {
            let _ = writeln!(out, "NOT OF RECORD: {}", why.join("; "));
        }
    }
    let _ = writeln!(
        out,
        "round: {} operations: {}",
        outcome.size.ops, outcome.size.why
    );
    let _ = writeln!(out, "levels:");
    for l in &outcome.levels {
        let count = match &l.count {
            Count::Fixed { times } => format!("{times}"),
            Count::UntilSettled { min, max } => format!("until settled, {min} to {max}"),
        };
        let _ = writeln!(out, "  {}: {count}: {}", l.level.name(), l.why);
    }
    for arm in &outcome.arms {
        arm_rows(&mut out, arm, outcome.request.b.is_some());
    }
    if let Some(c) = &outcome.comparison {
        verdict(&mut out, c);
    }
    out
}

fn arm_rows(out: &mut String, arm: &ArmOutcome, ab: bool) {
    let j = &arm.judgment;
    // The generator reports itself (Mantle's design §10): how late its worst start was against its
    // schedule, which every latency from an intended start includes.
    let late = arm.units.iter().map(|u| u.late_max_ns).max().unwrap_or(0);
    let _ = writeln!(
        out,
        "{}{} units, commit {}; the generator's latest start, {} after its schedule",
        if ab {
            format!("{}: {}: ", arm.arm.letter(), arm.spec.settings.join(", "))
        } else {
            String::new()
        },
        j.rounds,
        if arm.commits.is_empty() {
            DASH.to_owned()
        } else {
            arm.commits.join(", ")
        },
        duration(late)
    );
    let _ = writeln!(
        out,
        "  {:<5} {:>14} {:>9} {:>7} {:>22} {:>28} {:>28} {:>28}  order in time",
        "state", "median", "interval", "units", "share", "p50", "p99", "p99.9"
    );
    for (i, s) in arm.states.iter().enumerate() {
        let interval = s
            .spread
            .map_or_else(|| DASH.to_owned(), |w| format!("±{:.2}%", percent(w)));
        let share = match (s.share, s.share_interval) {
            (Some(p), Some((lo, hi))) => format!(
                "{:.1}% [{:.1}–{:.1}%]",
                percent(p),
                percent(lo),
                percent(hi)
            ),
            (Some(p), None) => format!("{:.1}% [{DASH}]", percent(p)),
            (None, _) => DASH.to_owned(),
        };
        let mut tails = s.tails.iter().map(|(_, t)| tail(t.as_ref(), s.independent));
        let mut next = || tails.next().unwrap_or_else(|| DASH.to_owned());
        let (p50, p99, p999) = (next(), next(), next());
        let order = if i == 0 {
            format!(
                "{} (lag-1 {:.3}; {} runs)",
                j.order.word(),
                j.lag1,
                j.runs.map_or_else(|| DASH.to_owned(), |r| r.to_string())
            )
        } else {
            String::new()
        };
        let _ = writeln!(
            out,
            "  {:<5} {:>14.6e} {:>9} {:>7} {:>22} {:>28} {:>28} {:>28}  {order}",
            i.saturating_add(1),
            s.median,
            interval,
            format!("{}/{}", s.units, j.rounds),
            share,
            p50,
            p99,
            p999,
        );
    }
}

fn verdict(out: &mut String, c: &Comparison) {
    let side = |s: &crate::ab::Side| {
        s.interval.map_or_else(
            || DASH.to_owned(),
            |(lo, hi)| format!("[{lo:.6e}, {hi:.6e}]"),
        )
    };
    let _ = writeln!(
        out,
        "verdict: {} (A {}, B {}): {}",
        c.verdict.words(),
        side(&c.a),
        side(&c.b),
        c.why
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A tail is a number with its interval, or a dash; never a mean (AC-RQ0.7.1, T-RQ0.7.4).
    #[test]
    fn tails_are_numbers_with_intervals_or_dashes() {
        assert_eq!(tail(None, true), DASH);
        let t = Tail {
            estimate: 1_500,
            lower: 1_000,
            upper: 2_000_000,
        };
        assert_eq!(tail(Some(&t), true), "1.500 µs [1.000 µs–2.000 ms]");
        assert_eq!(tail(Some(&t), false), "1.500 µs [—]");
        assert_eq!(duration(999), "999 ns");
        assert_eq!(duration(3_000_000_000), "3.000 s");
    }
}
