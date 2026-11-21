//! The benches a worker runs, by name. Until the registry has operations of its own (RQ1 brings
//! `reliquary_pull`, the bench AC-T9.1.4 names), these are the tool's own: `spin`, a CPU operation of a
//! stated amount of work whose settings plant a slowdown or a variation between process starts, so the
//! tool's verdicts can be checked against a known truth; and `judge`, the tool's judgment of a point's
//! rounds, a real piece of Reliquary whose instruction count CI gates (`bench/gated.toml`).

use std::collections::BTreeMap;

use crate::rng::SplitMix64;
use crate::rounds::{Policy, judge};

/// A bench and its settings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Bench {
    /// `work` rounds of SplitMix64's mixing an operation, plus up to `jitter` more drawn once per
    /// process start: work that varies between process starts and not between rounds. `work` is the
    /// caller's to state; `jitter` is none unless stated.
    Spin { work: u64, jitter: u64 },
    /// The judgment of `Policy::standard()`'s most rounds, the same values in every process: a gated
    /// bench does the same work wherever it runs, so its instructions differ between builds and not
    /// between the draws of its input.
    Judge,
}

/// Why a bench could not be named or set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BenchError {
    /// No bench of that name.
    Unknown(String),
    /// A setting the bench does not take.
    Setting { bench: &'static str, key: String },
    /// A setting whose value is not a whole number.
    Value { key: String, value: String },
    /// A setting given without `=`.
    Form(String),
    /// A setting the bench needs and was not given.
    Missing { bench: &'static str, key: String },
}

impl std::fmt::Display for BenchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unknown(name) => write!(
                f,
                "no bench named `{name}` (the benches: {})",
                NAMES.join(", ")
            ),
            Self::Setting { bench, key } => write!(f, "`{bench}` takes no setting `{key}`"),
            Self::Value { key, value } => {
                write!(f, "`{key}={value}`: the value is not a whole number")
            }
            Self::Form(s) => write!(f, "`{s}` is not key=value"),
            Self::Missing { bench, key } => write!(f, "`{bench}` needs `--set {key}=…`"),
        }
    }
}

impl std::error::Error for BenchError {}

/// The benches there are.
pub const NAMES: &[&str] = &["spin", "judge"];

/// Parses `key=value` settings.
pub fn settings(raw: &[String]) -> Result<BTreeMap<String, String>, BenchError> {
    raw.iter()
        .map(|s| {
            s.split_once('=')
                .map(|(k, v)| (k.to_owned(), v.to_owned()))
                .ok_or_else(|| BenchError::Form(s.clone()))
        })
        .collect()
}

impl Bench {
    /// The bench `name` with `settings`.
    pub fn named(name: &str, settings: &BTreeMap<String, String>) -> Result<Self, BenchError> {
        let whole = |key: &str, default: Option<u64>| -> Result<u64, BenchError> {
            match (settings.get(key), default) {
                (Some(v), _) => v.parse::<u64>().map_err(|_| BenchError::Value {
                    key: key.to_owned(),
                    value: v.clone(),
                }),
                (None, Some(d)) => Ok(d),
                (None, None) => Err(BenchError::Missing {
                    bench: "spin",
                    key: key.to_owned(),
                }),
            }
        };
        match name {
            "spin" => {
                if let Some(k) = settings
                    .keys()
                    .find(|k| !matches!(k.as_str(), "work" | "jitter"))
                {
                    return Err(BenchError::Setting {
                        bench: "spin",
                        key: k.clone(),
                    });
                }
                Ok(Self::Spin {
                    work: whole("work", None)?,
                    jitter: whole("jitter", Some(0))?,
                })
            }
            "judge" => match settings.keys().next() {
                Some(k) => Err(BenchError::Setting {
                    bench: "judge",
                    key: k.clone(),
                }),
                None => Ok(Self::Judge),
            },
            other => Err(BenchError::Unknown(other.to_owned())),
        }
    }

    /// The bench made ready in one process, its per-process draws from `seed`.
    #[must_use]
    pub fn prepare(&self, seed: u64) -> Prepared {
        let mut rng = SplitMix64::new(seed);
        match *self {
            Self::Spin { work, jitter } => Prepared::Spin {
                work: work.saturating_add(rng.below(jitter.saturating_add(1))),
                state: rng.next_u64(),
            },
            Self::Judge => {
                // One input for every process: the stream from seed zero, whatever this process drew.
                let mut fixed = SplitMix64::new(0);
                Prepared::Judge {
                    rounds: (0..Policy::standard().limit())
                        .map(|_| fixed.unit())
                        .collect(),
                }
            }
        }
    }
}

/// A bench ready to run operations.
#[derive(Debug, Clone)]
pub enum Prepared {
    Spin { work: u64, state: u64 },
    Judge { rounds: Vec<f64> },
}

impl Prepared {
    /// One operation; its result is passed through `black_box` so it is not optimized away.
    pub fn op(&mut self) {
        match self {
            Self::Spin { work, state } => {
                let mut r = SplitMix64::new(*state);
                let mut x = 0u64;
                for _ in 0..*work {
                    x ^= r.next_u64();
                }
                *state = std::hint::black_box(x);
            }
            Self::Judge { rounds } => {
                std::hint::black_box(judge(std::hint::black_box(rounds)));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(pairs: &[&str]) -> BTreeMap<String, String> {
        settings(&pairs.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>()).unwrap()
    }

    /// AC-RQ0.7.2: two settings of one build are two arms: spin's work is a setting.
    #[test]
    fn benches_are_named_and_set_or_refused_typed() {
        assert_eq!(
            Bench::named("spin", &set(&["work=1080"])).unwrap(),
            Bench::Spin {
                work: 1080,
                jitter: 0
            }
        );
        assert!(matches!(
            Bench::named("spin", &BTreeMap::new()),
            Err(BenchError::Missing { .. })
        ));
        assert_eq!(
            Bench::named("judge", &BTreeMap::new()).unwrap(),
            Bench::Judge
        );
        assert!(matches!(
            Bench::named("pull", &BTreeMap::new()),
            Err(BenchError::Unknown(_))
        ));
        assert!(matches!(
            Bench::named("spin", &set(&["speed=2"])),
            Err(BenchError::Setting { .. })
        ));
        assert!(matches!(
            Bench::named("spin", &set(&["work=fast"])),
            Err(BenchError::Value { .. })
        ));
        assert!(matches!(
            settings(&["work".to_owned()]),
            Err(BenchError::Form(_))
        ));
    }

    /// T-RQ0.7.9: spin's jitter varies the work between process starts and never within one.
    #[test]
    fn jitter_varies_work_between_processes_not_within() {
        let b = Bench::Spin {
            work: 100,
            jitter: 50,
        };
        let works: Vec<u64> = (0..20)
            .map(|seed| match b.prepare(seed) {
                Prepared::Spin { work, .. } => work,
                Prepared::Judge { .. } => panic!("spin prepared as judge"),
            })
            .collect();
        assert!(works.iter().all(|w| (100..=150).contains(w)));
        assert!(works.iter().any(|&w| w != works[0]));
        // Within a process the work stays as drawn.
        let mut p = b.prepare(3);
        let before = match &p {
            Prepared::Spin { work, .. } => *work,
            Prepared::Judge { .. } => panic!("spin prepared as judge"),
        };
        p.op();
        p.op();
        assert!(matches!(p, Prepared::Spin { work, .. } if work == before));
        let mut j = Bench::Judge.prepare(1);
        j.op();
        // Judge's input is one, whatever the process's seed: its work does not vary between processes.
        let input = |seed| match Bench::Judge.prepare(seed) {
            Prepared::Judge { rounds } => rounds,
            Prepared::Spin { .. } => panic!("judge prepared as spin"),
        };
        assert_eq!(input(1), input(2));
    }
}
