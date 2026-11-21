//! CI's gate on instruction counts (AC-RQ0.7.6): each bench `bench/gated.toml` names is measured in a
//! build of the change's base (A) and of the change (B), interleaved, and the gate fails a bench whose B
//! is "A better" by the A|B rule, its instructions an operation separated above A's, unless the change
//! accepts it by name.
//!
//! Instructions are counted where the counting needs no privilege. On Linux, CI's platform, by
//! Valgrind's cachegrind, which counts every instruction a process executes (`Ir`) without the kernel's
//! performance counters, which a container may be refused (research/07). Cachegrind counts a whole
//! process, so an operation's count is the difference of two process starts: one that runs a discarded
//! first operation alone, and one that runs it and then `N` more, so what a process does to start and
//! end cancels. `N` follows from how much the start's count varies between processes, so the
//! difference holds the precision. Where the OS counts a process's instructions itself (macOS), the
//! worker reads them round by round and the gate is the driver's A|B.

use std::path::{Path, PathBuf};
use std::process::{Command as Process, Stdio};

use serde::{Deserialize, Serialize};

use crate::ab::{self, Better, Comparison, cycle_order};
use crate::driver::{self, ArmSpec, Processes, Request};
use crate::load::{self, Own, Reading, Standing};
use crate::protocol::Metric;
use crate::rng::SplitMix64;
use crate::rounds::{self, FEWEST_TESTED, Judgment, Policy, Z_95, as_f64, u64_as_f64};

/// A bench CI gates, as `bench/gated.toml` names it.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Gated {
    pub name: String,
    #[serde(default)]
    pub settings: Vec<String>,
    /// Why CI gates it.
    pub why: String,
}

/// `bench/gated.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct GateFile {
    pub bench: Vec<Gated>,
}

/// How instructions are counted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Counter {
    /// The OS's own account of the process, round by round in the worker.
    Os,
    /// Valgrind's cachegrind at `valgrind`, its files in `scratch`.
    Cachegrind { valgrind: PathBuf, scratch: PathBuf },
}

/// One bench's gate.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BenchGate {
    pub bench: Gated,
    /// Operations a measured process runs after its discarded first, and why.
    pub ops: u64,
    pub ops_why: String,
    /// Each unit's instructions an operation, in time order.
    pub a: Vec<f64>,
    pub b: Vec<f64>,
    pub a_judgment: Judgment,
    pub b_judgment: Judgment,
    pub comparison: Comparison,
    /// Whether the change accepts this bench's rise or undecided verdict.
    pub accepted: bool,
    pub decision: Decision,
    pub outcome: String,
}

/// The gate's outcome.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GateOutcome {
    pub counter: String,
    /// The seed the order and the environments were drawn from, and the precision asked: a gate run
    /// replays from these.
    pub seed: u64,
    pub precision: f64,
    /// The commits builds A and B name (`--version`).
    pub commits: Vec<String>,
    pub benches: Vec<BenchGate>,
    /// The benches that fail the gate: risen, or undecided, and not accepted.
    pub failed: Vec<String>,
    pub readings: Vec<Reading>,
    pub standing: Standing,
}

/// Why the gate could not run.
#[derive(Debug)]
pub enum GateError {
    /// A process under the counter failed, or its count could not be read.
    Count(String),
    /// The driver's A|B failed.
    Driver(driver::DriverError),
}

impl std::fmt::Display for GateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Count(why) => f.write_str(why),
            Self::Driver(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for GateError {}

/// The instruction count in a cachegrind output file: the `summary:` line's count in the place the
/// `events:` line gives `Ir` (Valgrind manual §5.9.2, the output file's grammar: an `events:` line
/// naming the events, and a `summary:` line of their counts in that order; research/07 F40).
pub fn summary(text: &str) -> Result<u64, GateError> {
    let field = |name: &str| {
        text.lines()
            .find_map(|l| l.strip_prefix(name))
            .map(str::split_whitespace)
    };
    let at = field("events:")
        .and_then(|mut events| events.position(|e| e == "Ir"))
        .ok_or_else(|| GateError::Count("cachegrind's output names no `Ir` event".to_owned()))?;
    field("summary:")
        .and_then(|mut counts| counts.nth(at))
        .ok_or_else(|| GateError::Count("cachegrind's `summary:` holds no `Ir` count".to_owned()))?
        .parse::<u64>()
        .map_err(|e| GateError::Count(format!("cachegrind's `Ir` count is not a count: {e}")))
}

/// Operations a measured process runs: enough that the difference of two process starts, whose start
/// counts vary with standard deviation `start_sd`, holds an operation of `per_op` instructions within
/// `precision` at 95%: the difference's deviation is `√2·start_sd/N`, so
/// `N = ⌈Z_95·√2·start_sd/(precision·per_op)⌉`, at least one.
#[must_use]
pub fn ops_for(start_sd: f64, per_op: f64, precision: f64) -> (u64, String) {
    let need = Z_95 * std::f64::consts::SQRT_2 * start_sd / (precision * per_op.max(1.0));
    let mut n = 1u64;
    while u64_as_f64(n) < need {
        let Some(next) = n.checked_shl(1).filter(|&x| x > n) else {
            break;
        };
        n = next;
    }
    // The least power of two at or above the need; halve back while still enough.
    let mut low = n >> 1;
    while n.saturating_sub(low) > 1 {
        let mid = low.saturating_add(n.saturating_sub(low) >> 1);
        if u64_as_f64(mid) >= need {
            n = mid;
        } else {
            low = mid;
        }
    }
    (
        n.max(1),
        format!(
            "⌈{Z_95}·√2·{start_sd:.1}/({precision}·{per_op:.1})⌉ = {n}: the start's count varies by \
             {start_sd:.1} instructions between processes, and an operation is {per_op:.1}"
        ),
    )
}

/// The standard deviation of `xs` (over `n − 1`), zero for one or none.
fn deviation(xs: &[f64]) -> f64 {
    if xs.len() <= 1 {
        return 0.0;
    }
    let n = as_f64(xs.len());
    let centre = xs.iter().sum::<f64>() / n;
    (xs.iter().map(|x| (x - centre) * (x - centre)).sum::<f64>() / (n - 1.0)).sqrt()
}

/// One process under cachegrind: its instruction count.
fn cachegrind(
    valgrind: &Path,
    scratch: &Path,
    build: &Path,
    bench: &Gated,
    seed: u64,
    ops: u64,
    pad: usize,
) -> Result<u64, GateError> {
    let file = scratch.join(format!("cachegrind.{seed}.{ops}"));
    let mut command = Process::new(valgrind);
    command
        .arg("--tool=cachegrind")
        .arg("--cache-sim=no")
        .arg(format!("--cachegrind-out-file={}", file.display()))
        .arg(build)
        .args(["bench", "once", "--bench", &bench.name])
        .args(["--seed", &seed.to_string(), "--ops", &ops.to_string()])
        .env("RELIQUARY_BENCH_PAD", "x".repeat(pad))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    for s in &bench.settings {
        command.args(["--set", s]);
    }
    let out = command
        .output()
        .map_err(|e| GateError::Count(format!("{}: {e}", valgrind.display())))?;
    if !out.status.success() {
        return Err(GateError::Count(format!(
            "cachegrind of {} ended {}: {}",
            build.display(),
            out.status,
            String::from_utf8_lossy(&out.stderr)
        )));
    }
    let text = std::fs::read_to_string(&file)
        .map_err(|e| GateError::Count(format!("{}: {e}", file.display())))?;
    std::fs::remove_file(&file)
        .map_err(|e| GateError::Count(format!("{}: {e}", file.display())))?;
    summary(&text)
}

/// Gates one bench under cachegrind: dimension `N`, then interleaved units until both arms settle.
fn gate_cachegrind(
    valgrind: &Path,
    scratch: &Path,
    a: &Path,
    b: &Path,
    bench: &Gated,
    precision: f64,
    rng: &mut SplitMix64,
) -> Result<(u64, String, Vec<f64>, Vec<f64>), GateError> {
    let count = |rng: &mut SplitMix64, build: &Path, ops: u64| {
        let seed = rng.next_u64();
        let pad = usize::try_from(rng.below(driver::PAD_MOST)).unwrap_or(0);
        cachegrind(valgrind, scratch, build, bench, seed, ops, pad)
    };
    // Dimensioning: the start's variation over the fewest starts its order can be tested in, and one
    // operation's count, from A.
    let mut starts = Vec::new();
    for _ in 0..FEWEST_TESTED {
        starts.push(u64_as_f64(count(rng, a, 0)?));
    }
    let one = u64_as_f64(count(rng, a, 1)?);
    let mut sorted = starts.clone();
    sorted.sort_by(f64::total_cmp);
    let per_op = (one - rounds::median(&sorted)).max(1.0);
    let (ops, why) = ops_for(deviation(&starts), per_op, precision);
    let policy = Policy {
        precision,
        ..Policy::standard()
    };
    let (mut av, mut bv) = (Vec::new(), Vec::new());
    for _ in 0..policy.limit() {
        let (first, second) = cycle_order(rng);
        for arm in [first, second] {
            let build = if arm == ab::Arm::A { a } else { b };
            let zero = count(rng, build, 0)?;
            let some = count(rng, build, ops)?;
            let value = u64_as_f64(some.saturating_sub(zero)) / u64_as_f64(ops);
            if arm == ab::Arm::A {
                av.push(value);
            } else {
                bv.push(value);
            }
        }
        if policy.enough(&rounds::judge(&av)) && policy.enough(&rounds::judge(&bv)) {
            break;
        }
    }
    Ok((ops, why, av, bv))
}

/// What a gate run is given.
#[derive(Debug, Clone)]
pub struct GateRequest {
    pub a: PathBuf,
    pub b: PathBuf,
    pub benches: Vec<Gated>,
    pub counter: Counter,
    /// Benches whose rise the change accepts, by name.
    pub accept: Vec<String>,
    pub precision: f64,
    pub seed: u64,
}

/// The commit a build names: the parenthesis of its `--version` line.
fn commit_of(build: &Path) -> Result<String, GateError> {
    let out = Process::new(build)
        .arg("--version")
        .stdin(Stdio::null())
        .output()
        .map_err(|e| GateError::Count(format!("{}: {e}", build.display())))?;
    let line = String::from_utf8_lossy(&out.stdout);
    Ok(line
        .split_once('(')
        .and_then(|(_, rest)| rest.split_once(')'))
        .map_or_else(|| "none".to_owned(), |(c, _)| c.to_owned()))
}

/// What the gate makes of one bench's comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Decision {
    /// Both arms judged, and B did not rise: it fell, or the intervals meet.
    Passed,
    /// B's interval lies wholly above A's.
    Rose,
    /// An arm could not be judged (its units ordered in time, in two states, or too few for an
    /// interval), but every unit of B lies above every unit of A: B rose whatever order or states the
    /// units fell in.
    RoseWholly,
    /// An arm could not be judged, and B's units do not all lie above A's: no rise is shown. Mantle's
    /// two checks of order mark about one independent arm in ten (its design §2), so failing here
    /// would fail about one A|A gate in five.
    Undecided,
    /// The base build does not run this bench: nothing to compare against.
    New,
}

impl Decision {
    /// Whether it fails the gate, given whether the change accepts this bench.
    #[must_use]
    pub fn fails(self, accepted: bool) -> bool {
        matches!(self, Self::Rose | Self::RoseWholly) && !accepted
    }

    /// Its words in the gate's line.
    #[must_use]
    pub fn words(self, accepted: bool) -> &'static str {
        match (self, accepted) {
            (Self::Passed, _) => "passed",
            (Self::New, _) => "new: the base build does not run it, so it passes",
            (Self::Undecided, _) => {
                "passed, undecided: an arm's units could not be judged, and B's did not all lie above \
                 A's"
            }
            (Self::Rose | Self::RoseWholly, true) => "rose, and the change accepts it",
            (Self::Rose, false) => "rose: fails the gate",
            (Self::RoseWholly, false) => {
                "rose, every unit of B above every unit of A: fails the gate"
            }
        }
    }
}

/// The gate's decision on a comparison of instructions an operation, from the arms' units: their
/// intervals where both arms are judged; where an arm is not, whether every unit of B lies above every
/// unit of A, which for exchangeable units of one distribution happens once in `C(m + n, m)` (one in
/// 184,756 for ten a side), and so asks neither independence nor one state.
#[must_use]
pub fn decide(comparison: &Comparison, a: &[f64], b: &[f64]) -> Decision {
    let judged = |s: &ab::Side| s.interval.is_some();
    match comparison.verdict {
        ab::Verdict::ABetter => Decision::Rose,
        _ if judged(&comparison.a) && judged(&comparison.b) => Decision::Passed,
        _ => {
            let a_most = a.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            let b_least = b.iter().copied().fold(f64::INFINITY, f64::min);
            if !a.is_empty() && !b.is_empty() && b_least > a_most {
                Decision::RoseWholly
            } else {
                Decision::Undecided
            }
        }
    }
}

/// Whether `build` runs `bench`: a process of it running the bench once, with no counter. A build
/// that does not know the bench, or has no `bench` command, refuses as misuse (status 2).
fn runs_bench(build: &Path, bench: &Gated) -> Result<bool, GateError> {
    let mut command = Process::new(build);
    command
        .args([
            "bench",
            "once",
            "--bench",
            &bench.name,
            "--seed",
            "0",
            "--ops",
            "0",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    for s in &bench.settings {
        command.args(["--set", s]);
    }
    let out = command
        .output()
        .map_err(|e| GateError::Count(format!("{}: {e}", build.display())))?;
    let misuse = 2;
    match out.status.code() {
        Some(0) => Ok(true),
        Some(code) if code == misuse => Ok(false),
        _ => Err(GateError::Count(format!(
            "{} running {} ended {}: {}",
            build.display(),
            bench.name,
            out.status,
            String::from_utf8_lossy(&out.stderr)
        ))),
    }
}

/// Runs the gate.
pub fn run(request: &GateRequest) -> Result<GateOutcome, GateError> {
    let mut rng = SplitMix64::new(request.seed);
    let clock = std::time::Instant::now();
    let elapsed = || u64::try_from(clock.elapsed().as_nanos()).unwrap_or(u64::MAX);
    let mut own = Own::default();
    let mut readings = vec![Reading {
        when: "before the first unit".to_owned(),
        one_minute: hyper_measure::usage::load(),
        own: own.at(elapsed(), 0),
    }];
    let commits = vec![commit_of(&request.a)?, commit_of(&request.b)?];
    let mut benches = Vec::new();
    for bench in &request.benches {
        let accepted = request.accept.iter().any(|n| n == &bench.name);
        if !runs_bench(&request.a, bench)? {
            let (aj, bj) = (rounds::judge(&[]), rounds::judge(&[]));
            let comparison = ab::compare(&aj, &bj, Better::Lower);
            benches.push(BenchGate {
                bench: bench.clone(),
                ops: 0,
                ops_why: "not measured: the base build does not run this bench".to_owned(),
                a: Vec::new(),
                b: Vec::new(),
                a_judgment: aj,
                b_judgment: bj,
                comparison,
                accepted,
                decision: Decision::New,
                outcome: Decision::New.words(accepted).to_owned(),
            });
            continue;
        }
        let (ops, ops_why, a, b) = match &request.counter {
            Counter::Cachegrind { valgrind, scratch } => gate_cachegrind(
                valgrind,
                scratch,
                &request.a,
                &request.b,
                bench,
                request.precision,
                &mut rng,
            )?,
            Counter::Os => {
                let arm = |build: &PathBuf| ArmSpec {
                    builds: vec![build.clone()],
                    bench: bench.name.clone(),
                    settings: bench.settings.clone(),
                };
                let (a, b) = (arm(&request.a), arm(&request.b));
                let mut runner = Processes::new(vec![a.clone(), b.clone()], rng.next_u64());
                let out = driver::measure(
                    &mut runner,
                    &Request {
                        a,
                        b: Some(b),
                        metric: Metric::Instructions,
                        offered: driver::Offered::Closed,
                        tail: driver::MEDIAN,
                        in_flight: driver::SERIAL,
                        precision: request.precision,
                        seed: rng.next_u64(),
                        build_cost_ns: None,
                    },
                )
                .map_err(GateError::Driver)?;
                let values = |i: usize| {
                    out.arms
                        .get(i)
                        .map(|a| a.units.iter().map(|u| u.value).collect())
                        .unwrap_or_default()
                };
                (out.size.ops, out.size.why.clone(), values(0), values(1))
            }
        };
        let (aj, bj) = (rounds::judge(&a), rounds::judge(&b));
        let comparison = ab::compare(&aj, &bj, Better::Lower);
        let decision = decide(&comparison, &a, &b);
        let outcome = decision.words(accepted);
        benches.push(BenchGate {
            bench: bench.clone(),
            ops,
            ops_why,
            a,
            b,
            a_judgment: aj,
            b_judgment: bj,
            comparison,
            accepted,
            decision,
            outcome: outcome.to_owned(),
        });
    }
    let failed = benches
        .iter()
        .filter(|g| g.decision.fails(g.accepted))
        .map(|g| g.bench.name.clone())
        .collect();
    // The gate runs one counted process at a time, so its own share is at most one core: the wall
    // time stands for its CPU.
    let end = elapsed();
    readings.push(Reading {
        when: "at the end".to_owned(),
        one_minute: hyper_measure::usage::load(),
        own: own.at(end, end),
    });
    let named: Vec<Option<String>> = commits.iter().cloned().map(Some).collect();
    let standing = load::standing(&readings, &named, std::env::consts::OS);
    Ok(GateOutcome {
        counter: match request.counter {
            Counter::Os => "the OS's account of the process".to_owned(),
            Counter::Cachegrind { .. } => "cachegrind, a process's Ir".to_owned(),
        },
        seed: request.seed,
        precision: request.precision,
        commits,
        benches,
        failed,
        readings,
        standing,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Cachegrind's summary line is read, and anything else is refused, typed (AC-RQ0.7.6; hostile
    /// input to the parser).
    #[test]
    fn cachegrind_summaries_are_read_and_bad_files_refused() {
        let file = "desc: I1 cache: none\ncmd: reliquary bench once\nevents: Ir\nfl=a.rs\nfn=main\n1 20\nsummary: 1234567\n";
        assert_eq!(summary(file).unwrap(), 1_234_567);
        // With the cache simulation on, Ir is read from its own place, wherever the events put it.
        let simulated = "events: I1mr Ir ILmr\nsummary: 7 1234567 3\n";
        assert_eq!(summary(simulated).unwrap(), 1_234_567);
        for bad in [
            "",
            "summary:\n",
            "events: Ir\nsummary: many\n",
            "events: Ir\n",
            "events: Ir\nsummary: -4\n",
            "events: Dr Dw\nsummary: 1 2\n",
            "summary: 1234567\n",
        ] {
            assert!(summary(bad).is_err(), "{bad:?}");
        }
    }

    /// N grows with the start's variation and shrinks with the operation's size; a start that does not
    /// vary needs one operation (AC-RQ0.7.6, AC-RQ0.7.7).
    #[test]
    fn ops_follow_the_starts_variation() {
        assert_eq!(ops_for(0.0, 5_000.0, 0.05).0, 1);
        // 1.96·√2·1000/(0.05·5000) = 11.09 → 12.
        assert_eq!(ops_for(1_000.0, 5_000.0, 0.05).0, 12);
        let (n, why) = ops_for(1_000.0, 50.0, 0.05);
        assert_eq!(n, 1_109);
        assert!(why.contains("varies by 1000.0"), "{why}");
    }

    /// A rise fails unless accepted, whether shown by intervals or, where an arm cannot be judged, by
    /// every unit of B above every unit of A; a pass, a fall, an undecided gate and a new bench pass
    /// (AC-RQ0.7.6).
    #[test]
    fn decisions_fail_a_rise_and_only_a_rise() {
        let steady = vec![5_000.0; 10];
        let higher = vec![5_013.0; 10];
        let ordered: Vec<f64> = (0..10).map(|i| 5_000.0 + 10.0 * f64::from(i)).collect();
        let lifted: Vec<f64> = ordered.iter().map(|v| v + 1_000.0).collect();
        let d = |a: &[f64], b: &[f64]| {
            decide(
                &ab::compare(&rounds::judge(a), &rounds::judge(b), Better::Lower),
                a,
                b,
            )
        };
        assert_eq!(d(&steady, &higher), Decision::Rose);
        assert_eq!(d(&higher, &steady), Decision::Passed);
        assert_eq!(d(&steady, &steady), Decision::Passed);
        assert_eq!(d(&steady, &ordered), Decision::Undecided);
        assert_eq!(d(&ordered, &lifted), Decision::RoseWholly);
        for rose in [Decision::Rose, Decision::RoseWholly] {
            assert!(rose.fails(false) && !rose.fails(true));
        }
        for passes in [Decision::Passed, Decision::Undecided, Decision::New] {
            assert!(!passes.fails(false));
        }
    }

    /// T-RQ0.7.7 for the gate (AC-RQ0.7.3, AC-RQ0.7.6): A|A, two arms of one build whose counts vary
    /// a little between processes, repeated → the gate fails within the 5% level, by a one-sided
    /// binomial test, and the decisions made were mostly judged ones.
    #[test]
    fn an_a_a_gate_fails_within_five_percent() {
        let mut rng = SplitMix64::new(19);
        let runs = 2_000usize;
        let (mut failed, mut judged) = (0usize, 0usize);
        for _ in 0..runs {
            // Counts of ~63,000 instructions an operation, a few tens apart as layout moves them.
            let mut draw =
                || -> Vec<f64> { (0..10).map(|_| 63_000.0 + (rng.below(60) as f64)).collect() };
            let (a, b) = (draw(), draw());
            let c = ab::compare(&rounds::judge(&a), &rounds::judge(&b), Better::Lower);
            if c.a.interval.is_some() && c.b.interval.is_some() {
                judged += 1;
            }
            if decide(&c, &a, &b).fails(false) {
                failed += 1;
            }
        }
        let tail = if failed == 0 {
            1.0
        } else {
            1.0 - rounds::binomial_cdf(failed - 1, runs, 0.05)
        };
        assert!(tail >= 0.05, "{failed} of {runs} A|A gates failed");
        assert!(judged > runs / 2, "{judged} of {runs} had both intervals");
    }

    /// A rise in B separated from A fails the gate unless accepted; equal counts pass (AC-RQ0.7.6,
    /// AC-RQ0.7.2).
    #[test]
    fn a_separated_rise_fails_unless_accepted() {
        let a: Vec<f64> = vec![5_000.0; 10];
        let b: Vec<f64> = vec![5_013.0; 10];
        let c = ab::compare(&rounds::judge(&a), &rounds::judge(&b), Better::Lower);
        assert_eq!(c.verdict, ab::Verdict::ABetter, "{}", c.why);
        let same = ab::compare(&rounds::judge(&a), &rounds::judge(&a), Better::Lower);
        assert_eq!(same.verdict, ab::Verdict::NotSeparated);
        assert_eq!(deviation(&[1.0]), 0.0);
        assert!((deviation(&[2.0, 4.0, 6.0]) - 2.0).abs() < 1e-12);
    }
}
