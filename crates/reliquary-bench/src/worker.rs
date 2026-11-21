//! The process that measures: it runs the rounds its driver sends, one at a time, and answers each with
//! its measurements. Its own account (CPU time, and instructions where the OS counts them) is read from
//! hyper-measure's `usage` around each round; each operation's latency runs from its intended start
//! (`drive`). A worker runs nothing between rounds, so a round measures its operations alone.

use std::io::{BufRead, Write};

use hyper_measure::usage;

use crate::bench::{Bench, Prepared};
use crate::drive::{self, Monotonic, Op};
use crate::protocol::{self, Command, Metric, Reply, RoundResult, RoundSpec};
use crate::rounds::u64_as_f64;
use crate::tails::{Counts, exact};

/// Why a worker stopped.
#[derive(Debug)]
pub enum WorkerError {
    /// Its input or output failed.
    Line(protocol::LineError),
    Io(std::io::Error),
    /// A round asked for more operations than this machine would hold the latencies of.
    Memory {
        ops: u64,
    },
    /// The OS would not give the process's account.
    Usage(usage::UsageError),
    /// The metric asked for is one this platform's account does not count.
    Uncounted {
        metric: &'static str,
    },
    /// The open loop's schedule could not continue.
    Schedule(String),
}

impl std::fmt::Display for WorkerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Line(e) => write!(f, "{e}"),
            Self::Io(e) => write!(f, "{e}"),
            Self::Memory { ops } => {
                write!(
                    f,
                    "a round of {ops} operations needs room the allocator refused"
                )
            }
            Self::Usage(e) => write!(f, "the process's account: {e}"),
            Self::Uncounted { metric } => write!(
                f,
                "{} counts no {metric} for a process (hyper-measure's usage); gate instructions \
                 under cachegrind (`reliquary bench gate --counter cachegrind`)",
                std::env::consts::OS
            ),
            Self::Schedule(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for WorkerError {}

/// Serves rounds from `input` until told to end or the input ends, answering on `output`.
pub fn serve(
    input: &mut impl BufRead,
    output: &mut impl Write,
    bench: &Bench,
    name: &str,
    seed: u64,
    commit: &str,
) -> Result<(), WorkerError> {
    let mut prepared = bench.prepare(seed);
    let mut clock = Monotonic::default();
    send(
        output,
        &Reply::Hello {
            bench: name.to_owned(),
            commit: commit.to_owned(),
            pid: std::process::id(),
        },
    )?;
    loop {
        let command = match protocol::read::<Command>(input, protocol::most_command()) {
            Ok(c) => c,
            Err(e) => {
                refuse(output, &e.to_string())?;
                return Err(WorkerError::Line(e));
            }
        };
        match command {
            None | Some(Command::End) => return Ok(()),
            Some(Command::Round(spec)) => match round(&mut prepared, &mut clock, &spec) {
                Ok(r) => send(output, &Reply::Round(r))?,
                Err(e) => {
                    refuse(output, &e.to_string())?;
                    return Err(e);
                }
            },
        }
    }
}

fn send(output: &mut impl Write, reply: &Reply) -> Result<(), WorkerError> {
    let mut line =
        serde_json::to_vec(reply).map_err(|e| WorkerError::Io(std::io::Error::other(e)))?;
    line.push(b'\n');
    output.write_all(&line).map_err(WorkerError::Io)?;
    output.flush().map_err(WorkerError::Io)
}

fn refuse(output: &mut impl Write, why: &str) -> Result<(), WorkerError> {
    send(
        output,
        &Reply::Refused {
            why: why.to_owned(),
        },
    )
}

/// Runs one round and measures it.
pub fn round(
    prepared: &mut Prepared,
    clock: &mut Monotonic,
    spec: &RoundSpec,
) -> Result<RoundResult, WorkerError> {
    let count = usize::try_from(spec.ops).map_err(|_| WorkerError::Memory { ops: spec.ops })?;
    let mut ops: Vec<Op> = Vec::new();
    ops.try_reserve_exact(count)
        .map_err(|_| WorkerError::Memory { ops: spec.ops })?;
    let mut latencies: Vec<u64> = Vec::new();
    latencies
        .try_reserve_exact(count)
        .map_err(|_| WorkerError::Memory { ops: spec.ops })?;
    let before = usage::this().map_err(WorkerError::Usage)?;
    drive::round(clock, spec.load, spec.index, count, &mut ops, |_| {
        prepared.op();
        Ok::<(), std::convert::Infallible>(())
    })
    .map_err(|e| WorkerError::Schedule(format!("{e:?}")))?;
    let after = usage::this().map_err(WorkerError::Usage)?;
    let spent = after.since(&before);
    let first = ops.first().map_or(0, |o| o.intended);
    let last = ops.last().map_or(0, |o| o.end);
    let wall_ns = last.saturating_sub(first);
    let mut counts = Counts::default();
    let mut late_max_ns = 0u64;
    for o in &ops {
        latencies.push(o.latency());
        counts.record(o.latency());
        late_max_ns = late_max_ns.max(o.late());
    }
    latencies.sort_unstable();
    let per_op = |total: u64| u64_as_f64(total) / u64_as_f64(spec.ops.max(1));
    let value = match spec.metric {
        Metric::Throughput => u64_as_f64(spec.ops) / (u64_as_f64(wall_ns.max(1)) / NS_PER_SECOND),
        Metric::Latency { .. } => spec
            .metric
            .level()
            .and_then(|l| exact(&latencies, l))
            .map_or(0.0, u64_as_f64),
        Metric::Instructions => per_op(spent.instructions.ok_or(WorkerError::Uncounted {
            metric: "instructions",
        })?),
    };
    Ok(RoundResult {
        index: spec.index,
        ops: spec.ops,
        wall_ns,
        value,
        cpu_ns: spent.cpu_ns(),
        cpu_total_ns: after.cpu_ns(),
        instructions: spent.instructions,
        late_max_ns,
        latencies: counts.sparse(),
        op_spread: spread(&latencies),
    })
}

/// Format: nanoseconds a second.
pub(crate) const NS_PER_SECOND: f64 = 1e9;

/// The standard deviation of `xs` over their average, zero for one or none or an average of zero:
/// Jain's coefficient of variation, which Mantle's design §5a sizes a round from.
fn spread(xs: &[u64]) -> f64 {
    let n = xs.len();
    if n <= 1 {
        return 0.0;
    }
    let count = crate::rounds::as_f64(n);
    let centre = xs.iter().map(|&x| u64_as_f64(x)).sum::<f64>() / count;
    if centre <= 0.0 {
        return 0.0;
    }
    let squares: f64 = xs
        .iter()
        .map(|&x| (u64_as_f64(x) - centre) * (u64_as_f64(x) - centre))
        .sum();
    (squares / (count - 1.0)).sqrt() / centre
}

/// Runs one operation, discarded as a process's first is (Georges et al. §4.1), then `ops` more, and
/// returns: a whole process's work for a counter outside it (cachegrind), whose difference between
/// `ops` of zero and of `N` is `N` operations alone.
pub fn once(bench: &Bench, seed: u64, ops: u64) {
    let mut prepared = bench.prepare(seed);
    prepared.op();
    for _ in 0..ops {
        prepared.op();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drive::Load;

    fn spec(index: u64, ops: u64, metric: Metric) -> RoundSpec {
        RoundSpec {
            index,
            ops,
            load: Load::Closed,
            metric,
        }
    }

    /// AC-RQ0.7.5: a worker answers each round with its operations' latencies from their intended starts.
    #[test]
    fn a_worker_serves_rounds_and_ends() {
        let mut commands = Vec::new();
        for (i, m) in [Metric::Throughput, Metric::Latency { per: 1, of: 2 }]
            .into_iter()
            .enumerate()
        {
            commands.extend(serde_json::to_vec(&Command::Round(spec(i as u64, 50, m))).unwrap());
            commands.push(b'\n');
        }
        commands.extend(serde_json::to_vec(&Command::End).unwrap());
        commands.push(b'\n');
        let mut out = Vec::new();
        serve(
            &mut std::io::Cursor::new(commands),
            &mut out,
            &Bench::Spin {
                work: 100,
                jitter: 0,
            },
            "spin",
            1,
            "abc",
        )
        .unwrap();
        let mut lines = std::io::Cursor::new(out);
        let hello: Reply = protocol::read(&mut lines, protocol::most_reply())
            .unwrap()
            .unwrap();
        assert!(matches!(hello, Reply::Hello { ref commit, .. } if commit == "abc"));
        for i in 0..2 {
            let r: Reply = protocol::read(&mut lines, protocol::most_reply())
                .unwrap()
                .unwrap();
            let Reply::Round(r) = r else { panic!("{r:?}") };
            assert_eq!((r.index, r.ops), (i, 50));
            assert!(r.value > 0.0 && r.wall_ns > 0);
            let counted: u64 = r.latencies.iter().map(|b| b.count).sum();
            assert_eq!(counted, 50);
        }
        assert_eq!(
            protocol::read::<Reply>(&mut lines, protocol::most_reply()).unwrap(),
            None
        );
    }

    /// AC-RQ0.7.1: a command the worker cannot read is refused with its reason, never a panic.
    #[test]
    fn a_worker_refuses_a_line_it_cannot_read() {
        let mut out = Vec::new();
        let e = serve(
            &mut std::io::Cursor::new(b"{\"do\":\"dance\"}\n".to_vec()),
            &mut out,
            &Bench::Judge,
            "judge",
            1,
            "abc",
        )
        .unwrap_err();
        assert!(matches!(e, WorkerError::Line(_)));
        let text = String::from_utf8(out).unwrap();
        assert!(text.lines().nth(1).unwrap().contains("refused"), "{text}");
    }

    /// SERIAL: a worker runs one operation at a time, each starting after the last answered, so at most
    /// one is ever in flight (AC-RQ0.7.5).
    #[test]
    fn operations_run_one_at_a_time() {
        let mut clock = Monotonic::default();
        let mut prepared = Bench::Spin {
            work: 50,
            jitter: 0,
        }
        .prepare(1);
        let mut ops = Vec::new();
        drive::round(&mut clock, Load::Closed, 0, 200, &mut ops, |_| {
            prepared.op();
            Ok::<(), ()>(())
        })
        .unwrap();
        let most = ops
            .iter()
            .map(|a| {
                ops.iter()
                    .filter(|b| b.start < a.end && b.end > a.start)
                    .count()
            })
            .max()
            .unwrap();
        assert_eq!(most as u64, crate::driver::SERIAL);
    }

    /// AC-RQ0.7.7: the spread that sizes a round is Jain's coefficient of variation.
    #[test]
    fn the_spread_is_the_coefficient_of_variation() {
        assert_eq!(spread(&[5]), 0.0);
        assert_eq!(spread(&[4, 4, 4]), 0.0);
        // 2, 4, 6: sd 2 over 4.
        assert!((spread(&[2, 4, 6]) - 0.5).abs() < 1e-12);
    }
}
