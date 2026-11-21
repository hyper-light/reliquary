//! The load rule (AC-RQ0.7.4): a record is "of record" only when the machine was otherwise quiet
//! throughout, by the person's rule in CLAUDE.md §7, "load average under 2, nothing else building", and
//! when its build names the commit it was built from.
//!
//! The one-minute load average counts the run's own busy threads with everyone else's, so the rule is
//! held to what is left after the run's own share: the CPU the run's processes used, in cores, decayed as
//! the kernel decays the average itself, by `e^(−Δt/60 s)` (the one-minute average's definition, Linux
//! `kernel/sched/loadavg.c`, research/07). The first reading is taken before any of the run's own work,
//! so a machine already loaded when the run begins is caught whatever the run does after. A platform
//! that keeps no load average (Windows) makes no record of record, and says so.

use serde::Serialize;

/// The most load, less the run's own, a record of record may meet: CLAUDE.md §7, the person's rule.
pub const LIMIT: f64 = 2.0;

/// Format: the one-minute load average's window, in nanoseconds: the average decays by `e^(−5/60)`
/// every five seconds (Linux `kernel/sched/loadavg.c`, `EXP_1`; research/07).
const WINDOW_NS: f64 = 60e9;

/// One reading of the machine's load.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Reading {
    /// When it was taken: "before the first unit", "after unit 7", "at the end".
    pub when: String,
    /// The one-minute load average; none where the platform keeps none.
    pub one_minute: Option<f64>,
    /// The run's own share of it, in cores.
    pub own: f64,
}

impl Reading {
    /// The load not the run's own.
    #[must_use]
    pub fn others(&self) -> Option<f64> {
        self.one_minute.map(|l| (l - self.own).max(0.0))
    }
}

/// The run's own share of the load average: its processes' CPU over wall time, decayed as the average
/// is decayed.
#[derive(Debug, Clone, Default)]
pub struct Own {
    share: f64,
    last: Option<(u64, u64)>,
}

impl Own {
    /// Folds in that by `at_ns` on the run's clock its processes had used `cpu_ns` of CPU in all,
    /// returning the share in cores.
    pub fn at(&mut self, at_ns: u64, cpu_ns: u64) -> f64 {
        if let Some((then, then_cpu)) = self.last {
            let dt = at_ns.saturating_sub(then);
            if dt > 0 {
                let busy = crate::rounds::u64_as_f64(cpu_ns.saturating_sub(then_cpu))
                    / crate::rounds::u64_as_f64(dt);
                let keep = (-crate::rounds::u64_as_f64(dt) / WINDOW_NS).exp();
                self.share = self.share * keep + busy * (1.0 - keep);
            }
        }
        self.last = Some((at_ns, cpu_ns));
        self.share
    }
}

/// Whether a record may be cited.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "standing", rename_all = "snake_case")]
pub enum Standing {
    /// The load rule held at every reading, and the build names its commit.
    #[serde(rename = "of record")]
    OfRecord,
    /// What broke the rule: each reading over it, a reading the platform could not give, or a build
    /// that names no commit. Such a record cannot be cited by an acceptance criterion.
    #[serde(rename = "not of record")]
    NotOfRecord { why: Vec<String> },
}

impl Standing {
    /// The words a record and its report carry.
    #[must_use]
    pub fn words(&self) -> &'static str {
        match self {
            Self::OfRecord => "of record",
            Self::NotOfRecord { .. } => "not of record",
        }
    }
}

/// A record's standing from its load readings and the commits its builds name.
#[must_use]
pub fn standing(readings: &[Reading], commits: &[Option<String>], platform: &str) -> Standing {
    let mut why = Vec::new();
    if readings.is_empty() {
        why.push("no load was read".to_owned());
    }
    // Every reading is kept in the record; the reason names the first over the rule, how many were,
    // and the worst.
    let over: Vec<(&Reading, f64, f64)> = readings
        .iter()
        .filter_map(|r| Some((r, r.one_minute?, r.others()?)))
        .filter(|&(_, _, others)| others >= LIMIT)
        .collect();
    if let Some(&(first, load, others)) = over.first() {
        let mut said = format!(
            "the one-minute load average was {load:.2} {}, {others:.2} of it not this run's, at or \
             over the rule's {LIMIT} (CLAUDE.md §7)",
            first.when
        );
        if let Some(&(worst, wload, wothers)) = over.iter().max_by(|a, b| a.2.total_cmp(&b.2))
            && over.len() > 1
        {
            said.push_str(&format!(
                "; {} of {} readings were over it, the most {wload:.2} {}, {wothers:.2} not this \
                 run's",
                over.len(),
                readings.len(),
                worst.when
            ));
        }
        why.push(said);
    }
    if readings.iter().any(|r| r.one_minute.is_none()) {
        why.push(format!(
            "{platform} keeps no load average this tool can read, so the rule cannot be held"
        ));
    }
    for (i, c) in commits.iter().enumerate() {
        if c.as_deref()
            .is_none_or(|c| c.is_empty() || c.contains("unknown") || c.ends_with("-dirty"))
        {
            why.push(format!(
                "build {} names no clean commit ({}); a record of record is of a build made from one \
                 (scripts/bench-build.sh)",
                i.saturating_add(1),
                c.as_deref().unwrap_or("none")
            ));
        }
    }
    if why.is_empty() {
        Standing::OfRecord
    } else {
        Standing::NotOfRecord { why }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reading(when: &str, load: Option<f64>, own: f64) -> Reading {
        Reading {
            when: when.to_owned(),
            one_minute: load,
            own,
        }
    }

    /// T-RQ0.7.8: a benchmark started while another process loads the machine → "not of record", and the
    /// record names the load.
    #[test]
    fn a_run_begun_on_a_loaded_machine_is_not_of_record_and_names_the_load() {
        let s = standing(
            &[
                reading("before the first unit", Some(5.31), 0.0),
                reading("at the end", Some(1.2), 1.0),
            ],
            &[Some("abc123".to_owned())],
            "linux",
        );
        let Standing::NotOfRecord { why } = &s else {
            panic!("{s:?}")
        };
        assert_eq!(s.words(), "not of record");
        assert_eq!(why.len(), 1, "{why:?}");
        assert!(why[0].contains("5.31 before the first unit"), "{why:?}");
    }

    /// AC-RQ0.7.4: the run's own threads do not break the rule; others' do.
    #[test]
    fn the_runs_own_share_is_not_held_against_it() {
        let quiet = [
            reading("before the first unit", Some(0.4), 0.0),
            reading("after unit 10", Some(3.9), 3.5),
            reading("at the end", Some(4.1), 3.9),
        ];
        let commit = [Some("abc123".to_owned())];
        assert_eq!(standing(&quiet, &commit, "linux"), Standing::OfRecord);
        let busy = [
            reading("before the first unit", Some(0.4), 0.0),
            reading("after unit 10", Some(6.2), 3.5),
        ];
        assert!(matches!(
            standing(&busy, &commit, "linux"),
            Standing::NotOfRecord { .. }
        ));
    }

    /// AC-RQ0.7.4: no load average, or a build with no clean commit, is not of record, and says which.
    #[test]
    fn an_unreadable_load_or_an_unnamed_commit_is_not_of_record() {
        let s = standing(
            &[reading("before the first unit", None, 0.0)],
            &[Some("abc".to_owned())],
            "windows",
        );
        let Standing::NotOfRecord { why } = s else {
            panic!()
        };
        assert!(why[0].contains("windows keeps no load average"), "{why:?}");
        let quiet = [reading("before the first unit", Some(0.1), 0.0)];
        for c in [
            None,
            Some("unknown commit".to_owned()),
            Some("abc-dirty".to_owned()),
        ] {
            let s = standing(&quiet, std::slice::from_ref(&c), "linux");
            let Standing::NotOfRecord { why } = s else {
                panic!("{c:?}")
            };
            assert!(why[0].contains("names no clean commit"), "{why:?}");
        }
        assert!(matches!(
            standing(&[], &[], "linux"),
            Standing::NotOfRecord { .. }
        ));
    }

    /// AC-RQ0.7.4: the run's share follows its CPU and decays as the load average does.
    #[test]
    fn the_runs_share_follows_its_cpu_with_the_averages_decay() {
        let mut own = Own::default();
        assert_eq!(own.at(0, 0), 0.0);
        // Two cores busy for a minute: 1 − e^−1 of the way to two.
        let share = own.at(60_000_000_000, 120_000_000_000);
        assert!(
            (share - 2.0 * (1.0 - (-1.0f64).exp())).abs() < 1e-9,
            "{share}"
        );
        // Idle for a minute more: it decays by e^−1.
        let later = own.at(120_000_000_000, 120_000_000_000);
        assert!((later - share * (-1.0f64).exp()).abs() < 1e-9, "{later}");
    }
}
