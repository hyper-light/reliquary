//! What the driver and a worker say to each other: one JSON object a line each way, each line bounded by
//! the longest its own format can make, so neither side's buffer grows with what the other sends
//! (CLAUDE.md §2).

use std::io::BufRead;
use std::sync::OnceLock;

use hyper_measure::quantile::Level;
use hyper_timing::Histogram;
use serde::{Deserialize, Serialize};

use crate::ab::Better;
use crate::drive::Load;
use crate::tails::Bucket;

/// What a round's value is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Metric {
    /// Operations a second.
    Throughput,
    /// The round's `per/of` quantile of latency from each operation's intended start, in nanoseconds.
    Latency { per: u64, of: u64 },
    /// Instructions an operation, where the OS counts a process's instructions.
    Instructions,
}

impl Metric {
    /// Which way the metric is better.
    #[must_use]
    pub fn better(self) -> Better {
        match self {
            Self::Throughput => Better::Higher,
            Self::Latency { .. } | Self::Instructions => Better::Lower,
        }
    }

    /// The metric's name in a record and its unit.
    #[must_use]
    pub fn describe(self) -> String {
        match self {
            Self::Throughput => "throughput, operations a second".to_owned(),
            Self::Latency { per, of } => {
                format!("latency from the intended start, the round's {per}/{of} quantile, ns")
            }
            Self::Instructions => "instructions an operation".to_owned(),
        }
    }

    /// The latency quantile, for a latency metric.
    #[must_use]
    pub fn level(self) -> Option<Level> {
        match self {
            Self::Latency { per, of } => Level::new(per, of).ok(),
            _ => None,
        }
    }
}

/// One round to run.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RoundSpec {
    pub index: u64,
    pub ops: u64,
    pub load: Load,
    pub metric: Metric,
}

/// A command to a worker.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "do", rename_all = "snake_case")]
pub enum Command {
    Round(RoundSpec),
    End,
}

/// A round's measurements.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoundResult {
    pub index: u64,
    pub ops: u64,
    /// From the round's first intended start to its last answer.
    pub wall_ns: u64,
    /// The round's value of its metric.
    pub value: f64,
    /// CPU the worker used in the round, and in all since it began.
    pub cpu_ns: u64,
    pub cpu_total_ns: u64,
    /// Instructions the worker retired in the round, where the OS counts them.
    pub instructions: Option<u64>,
    /// The driver's worst lateness, its start less the intended.
    pub late_max_ns: u64,
    /// Each operation's latency from its intended start, in hyper-timing's buckets.
    pub latencies: Vec<Bucket>,
    /// The operations' latencies' standard deviation over their average: what sizes a round
    /// (Mantle's design §5a), never a report's figure.
    pub op_spread: f64,
}

/// A worker's line.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "reply", rename_all = "snake_case")]
pub enum Reply {
    Hello {
        bench: String,
        commit: String,
        pid: u32,
    },
    Round(RoundResult),
    Refused {
        why: String,
    },
}

/// Why a line could not be read.
#[derive(Debug)]
pub enum LineError {
    /// A line longer than the format can make.
    TooLong {
        most: usize,
    },
    Io(std::io::Error),
    /// A line that is not one of the format's.
    Json(serde_json::Error),
}

impl std::fmt::Display for LineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooLong { most } => {
                write!(f, "a line longer than the {most} bytes its format can make")
            }
            Self::Io(e) => write!(f, "{e}"),
            Self::Json(e) => write!(f, "a line that is not the protocol's: {e}"),
        }
    }
}

impl std::error::Error for LineError {}

/// The longest line a command can make: a round with every number at its widest, and its newline.
pub fn most_command() -> usize {
    static MOST: OnceLock<usize> = OnceLock::new();
    *MOST.get_or_init(|| {
        let widest = Command::Round(RoundSpec {
            index: u64::MAX,
            ops: u64::MAX,
            load: Load::Open {
                per_second: -f64::MAX,
                seed: u64::MAX,
            },
            metric: Metric::Latency {
                per: u64::MAX,
                of: u64::MAX,
            },
        });
        serde_json::to_string(&widest).map_or(0, |s| s.len().saturating_add(1))
    })
}

/// The longest line a worker's round can make: every bucket counted at its widest, and its newline. A
/// hello or a refusal longer than that is refused too.
pub fn most_reply() -> usize {
    static MOST: OnceLock<usize> = OnceLock::new();
    *MOST.get_or_init(|| {
        let widest = Reply::Round(RoundResult {
            index: u64::MAX,
            ops: u64::MAX,
            wall_ns: u64::MAX,
            value: -f64::MAX,
            cpu_ns: u64::MAX,
            cpu_total_ns: u64::MAX,
            instructions: Some(u64::MAX),
            late_max_ns: u64::MAX,
            latencies: (0..Histogram::BUCKETS)
                .filter_map(|b| {
                    Some(Bucket {
                        bucket: u16::try_from(b).ok()?,
                        count: u64::MAX,
                    })
                })
                .collect(),
            op_spread: -f64::MAX,
        });
        serde_json::to_string(&widest).map_or(0, |s| s.len().saturating_add(1))
    })
}

/// Reads one line of at most `most` bytes, its newline included; none at the end of the input.
pub fn read_line(input: &mut impl BufRead, most: usize) -> Result<Option<String>, LineError> {
    let mut line = Vec::new();
    loop {
        let buf = input.fill_buf().map_err(LineError::Io)?;
        if buf.is_empty() {
            return if line.is_empty() {
                Ok(None)
            } else {
                String::from_utf8(line)
                    .map(Some)
                    .map_err(|e| LineError::Io(std::io::Error::other(e)))
            };
        }
        let (take, done) = match buf.iter().position(|&b| b == b'\n') {
            Some(at) => (at.saturating_add(1), true),
            None => (buf.len(), false),
        };
        if line.len().saturating_add(take) > most {
            return Err(LineError::TooLong { most });
        }
        line.extend_from_slice(buf.get(..take).unwrap_or(&[]));
        input.consume(take);
        if done {
            return String::from_utf8(line)
                .map(Some)
                .map_err(|e| LineError::Io(std::io::Error::other(e)));
        }
    }
}

/// Reads and parses one line.
pub fn read<T: for<'de> Deserialize<'de>>(
    input: &mut impl BufRead,
    most: usize,
) -> Result<Option<T>, LineError> {
    match read_line(input, most)? {
        None => Ok(None),
        Some(line) => serde_json::from_str(&line)
            .map(Some)
            .map_err(LineError::Json),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// AC-RQ0.7.1: every command and reply fits its bound, and a longer line is refused, typed.
    #[test]
    fn lines_are_bounded_by_their_format() {
        let line = serde_json::to_string(&Command::End).unwrap() + "\n";
        assert!(line.len() <= most_command());
        let mut input = std::io::Cursor::new(line.into_bytes());
        assert_eq!(
            read::<Command>(&mut input, most_command()).unwrap(),
            Some(Command::End)
        );
        assert_eq!(read::<Command>(&mut input, most_command()).unwrap(), None);
        let long = vec![b'x'; most_command() + 1];
        let mut input = std::io::Cursor::new(long);
        assert!(matches!(
            read_line(&mut input, most_command()),
            Err(LineError::TooLong { .. })
        ));
        assert!(most_reply() > most_command());
    }

    /// Hostile input: lines that are not the protocol's are refused, typed, never a panic.
    /// (AC-RQ0.7.1)
    #[test]
    fn hostile_lines_are_refused() {
        // AC-RQ0.7.1: the worker's parser takes only its format.
        for bad in [
            "{}\n",
            "{\"do\":\"round\"}\n",
            "{\"do\":\"round\",\"index\":-1,\"ops\":1,\"load\":{\"kind\":\"closed\"},\"metric\":{\"kind\":\"cpu\"}}\n",
            "\u{0}\u{1}\n",
            "[1,2,3]\n",
            "{\"do\":\"explode\"}\n",
        ] {
            let mut input = std::io::Cursor::new(bad.as_bytes().to_vec());
            assert!(
                read::<Command>(&mut input, most_command()).is_err(),
                "{bad:?}"
            );
        }
        let mut input = std::io::Cursor::new(vec![0xff, 0xfe, b'\n']);
        assert!(read_line(&mut input, most_command()).is_err());
    }
}
