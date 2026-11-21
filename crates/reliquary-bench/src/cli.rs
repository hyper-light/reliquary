//! `reliquary bench`'s command line. The binary passes the arguments after `bench`, its streams and the
//! commit it was built from, and turns the [`Ending`] into its exit status.

use std::collections::BTreeMap;
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::ab::Better;
use crate::bench::{self, Bench};
use crate::changes::{self, Measured};
use crate::driver::{self, ArmSpec, Offered, Outcome, Processes, Request};
use crate::gate::{self, Counter, GateFile, GateRequest};
use crate::load::Standing;
use crate::protocol::Metric;
use crate::record::{self, Record, What};
use crate::{report, rounds, worker};

/// How a command ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ending {
    /// It did what was asked.
    Done,
    /// It ran and its answer is a failure: a gate failed, a change point alerted, a worker refused.
    Failed,
    /// The arguments could not be used; the usage was written.
    Misuse,
}

/// The usage.
pub const USAGE: &str = "usage: reliquary bench <command>
  run BENCH [--set K=V]... [--build PATH]... [--metric M] [--load L] [--tail Q] [--precision P]
            [--seed N] [--build-seconds S] [--machine NAME] [--record DIR] [--metrics FILE]
  ab BENCH [--a PATH]... [--a-set K=V]... [--b PATH]... [--b-set K=V]... [the options of run]
  gate GATED.toml --a PATH --b PATH [--counter os|cachegrind] [--valgrind PATH] [--scratch DIR]
            [--accept BENCH]... [--precision P] [--seed N] [--machine NAME] [--record DIR]
  changes SERIES.json [--acknowledged CHANGES.toml]
  list
M: throughput | instructions | latency:Q;  L: closed | open:RATE | power;  Q: pNN or pNN.N…
";

/// The commands a worker process is started with.
const WORKER: &str = "worker";

/// Parsed options: each `--name value` under its name, the rest in order.
struct Args {
    positional: Vec<String>,
    options: BTreeMap<String, Vec<String>>,
}

impl Args {
    fn parse(args: &[String], takes: &[&str]) -> Result<Self, String> {
        let mut positional = Vec::new();
        let mut options: BTreeMap<String, Vec<String>> = BTreeMap::new();
        let mut it = args.iter();
        while let Some(a) = it.next() {
            if let Some(name) = a.strip_prefix("--") {
                if !takes.contains(&name) {
                    return Err(format!("unknown option --{name}"));
                }
                let v = it.next().ok_or_else(|| format!("--{name} needs a value"))?;
                options.entry(name.to_owned()).or_default().push(v.clone());
            } else {
                positional.push(a.clone());
            }
        }
        Ok(Self {
            positional,
            options,
        })
    }

    fn all(&self, name: &str) -> Vec<String> {
        self.options.get(name).cloned().unwrap_or_default()
    }

    fn one(&self, name: &str) -> Result<Option<String>, String> {
        match self.options.get(name).map(Vec::as_slice) {
            None | Some([]) => Ok(None),
            Some([v]) => Ok(Some(v.clone())),
            Some(_) => Err(format!("--{name} given more than once")),
        }
    }

    fn only(&self, what: &str) -> Result<String, String> {
        match self.positional.as_slice() {
            [one] => Ok(one.clone()),
            [] => Err(format!("name {what}")),
            more => Err(format!("one {what}, not {}", more.join(" "))),
        }
    }
}

/// Format: the base of a percentile's decimal places.
const DECIMAL: u64 = 10;

/// A quantile named as a percentile, `p99.9`, as `per/of`.
fn quantile(s: &str) -> Result<(u64, u64), String> {
    let bad = || format!("`{s}` is not a percentile such as p50 or p99.9");
    let digits = s.strip_prefix('p').ok_or_else(bad)?;
    let (whole, frac) = match digits.split_once('.') {
        Some((_, "")) => return Err(bad()),
        Some(parts) => parts,
        None => (digits, ""),
    };
    if whole.is_empty()
        || !whole
            .bytes()
            .chain(frac.bytes())
            .all(|b| b.is_ascii_digit())
    {
        return Err(bad());
    }
    let places = u32::try_from(frac.len()).map_err(|_| bad())?;
    let scale = DECIMAL.checked_pow(places).ok_or_else(bad)?;
    let per = format!("{whole}{frac}").parse::<u64>().map_err(|_| bad())?;
    let of = report::PERCENT.checked_mul(scale).ok_or_else(bad)?;
    if per == 0 || per >= of {
        return Err(bad());
    }
    Ok((per, of))
}

fn metric(s: &str) -> Result<Metric, String> {
    match s {
        "throughput" => Ok(Metric::Throughput),
        "instructions" => Ok(Metric::Instructions),
        other => {
            let q = other
                .strip_prefix("latency:")
                .ok_or_else(|| format!("no metric `{other}`"))?;
            let (per, of) = quantile(q)?;
            Ok(Metric::Latency { per, of })
        }
    }
}

fn offered(s: &str) -> Result<Offered, String> {
    match s {
        "closed" => Ok(Offered::Closed),
        "power" => Ok(Offered::Power),
        other => {
            let rate = other
                .strip_prefix("open:")
                .ok_or_else(|| format!("no load `{other}`"))?
                .parse::<f64>()
                .map_err(|e| format!("`{other}`: {e}"))?;
            if !(rate.is_finite() && rate > 0.0) {
                return Err(format!("`{other}`: a rate is a positive number a second"));
            }
            Ok(Offered::Open { per_second: rate })
        }
    }
}

fn precision(a: &Args) -> Result<f64, String> {
    match a.one("precision")? {
        None => Ok(rounds::PRECISION),
        Some(p) => {
            let v = p
                .parse::<f64>()
                .map_err(|e| format!("--precision {p}: {e}"))?;
            if v.is_finite() && v > 0.0 && v < 1.0 {
                Ok(v)
            } else {
                Err(format!("--precision {p}: a share between 0 and 1"))
            }
        }
    }
}

fn seed(a: &Args) -> Result<u64, String> {
    match a.one("seed")? {
        Some(s) => s.parse::<u64>().map_err(|e| format!("--seed {s}: {e}")),
        // A seed only orders arms and draws schedules; the clock's gives a fresh one, which the record
        // keeps so the run replays.
        None => Ok(std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| {
                u64::try_from(d.as_nanos() & u128::from(u64::MAX)).unwrap_or(0)
            })),
    }
}

fn builds(a: &Args, name: &str) -> Result<Vec<PathBuf>, String> {
    let given: Vec<PathBuf> = a.all(name).into_iter().map(PathBuf::from).collect();
    if given.is_empty() {
        std::env::current_exe()
            .map(|p| vec![p])
            .map_err(|e| format!("this binary's own path: {e}"))
    } else {
        Ok(given)
    }
}

/// Runs `reliquary bench` with `args`, the words after `bench`.
pub fn main<I: BufRead, O: Write, E: Write>(
    args: &[String],
    commit: &str,
    input: &mut I,
    out: &mut O,
    err: &mut E,
) -> Ending {
    let (command, rest) = match args.split_first() {
        Some((c, rest)) => (c.as_str(), rest),
        None => return misuse(err, "name a command"),
    };
    let result = match command {
        "run" => run(rest, commit, out, false),
        "ab" => run(rest, commit, out, true),
        "gate" => gate_command(rest, commit, out),
        "changes" => changes_command(rest, out),
        "list" => list(out),
        WORKER => worker_command(rest, commit, input, out),
        "once" => once(rest),
        "--help" | "-h" | "help" => write(out, USAGE),
        other => Err(Failure::Misuse(format!("no command `{other}`"))),
    };
    match result {
        Ok(ending) => ending,
        Err(Failure::Misuse(why)) => misuse(err, &why),
        Err(Failure::Failed(why)) => {
            let _ = writeln!(err, "reliquary bench: {why}");
            Ending::Failed
        }
    }
}

enum Failure {
    Misuse(String),
    Failed(String),
}

impl From<String> for Failure {
    fn from(why: String) -> Self {
        Self::Misuse(why)
    }
}

fn misuse(err: &mut impl Write, why: &str) -> Ending {
    let _ = write!(err, "reliquary bench: {why}\n{USAGE}");
    Ending::Misuse
}

fn write(out: &mut impl Write, text: &str) -> Result<Ending, Failure> {
    out.write_all(text.as_bytes())
        .map_err(|e| Failure::Failed(e.to_string()))?;
    Ok(Ending::Done)
}

fn list(out: &mut impl Write) -> Result<Ending, Failure> {
    write(
        out,
        "spin    --set work=N [--set jitter=N]: N rounds of mixing an operation, up to jitter more a process\n\
         judge   the tool's judgment of a point's rounds\n",
    )
}

/// The options `run` and `ab` share.
const SHARED: &[&str] = &[
    "metric",
    "load",
    "tail",
    "precision",
    "seed",
    "build-seconds",
    "machine",
    "record",
    "metrics",
];

/// `run`'s own options: one arm's builds and settings.
const POINT: &[&str] = &["set", "build"];

/// `ab`'s own options: each arm's builds and settings.
const ARMS: &[&str] = &["a", "a-set", "b", "b-set"];

/// `run` and `ab`.
fn run(args: &[String], commit: &str, out: &mut impl Write, ab: bool) -> Result<Ending, Failure> {
    let own = if ab { ARMS } else { POINT };
    let takes: Vec<&str> = SHARED.iter().chain(own).copied().collect();
    let a = Args::parse(args, &takes)?;
    let name = a.only("a bench")?;
    if !bench::NAMES.contains(&name.as_str()) {
        return Err(Failure::Misuse(format!("no bench `{name}`")));
    }
    let metric = metric(&a.one("metric")?.unwrap_or_else(|| "throughput".to_owned()))?;
    let offered = offered(&a.one("load")?.unwrap_or_else(|| "closed".to_owned()))?;
    let tail = match (a.one("tail")?, metric) {
        (Some(q), _) => quantile(&q)?,
        (None, Metric::Latency { per, of }) => (per, of),
        (None, _) => quantile("p50")?,
    };
    let arm = |paths: &str, sets: &str| -> Result<ArmSpec, Failure> {
        let spec = ArmSpec {
            builds: builds(&a, paths)?,
            bench: name.clone(),
            settings: a.all(sets),
        };
        let parsed = bench::settings(&spec.settings).map_err(|e| Failure::Misuse(e.to_string()))?;
        Bench::named(&name, &parsed).map_err(|e| Failure::Misuse(e.to_string()))?;
        Ok(spec)
    };
    let (first, second) = if ab {
        (arm("a", "a-set")?, Some(arm("b", "b-set")?))
    } else {
        (arm("build", "set")?, None)
    };
    if let Some(b) = &second
        && b.builds.len() != first.builds.len()
    {
        return Err(Failure::Misuse(format!(
            "A names {} builds and B {}: a level is repeated alike in both arms",
            first.builds.len(),
            b.builds.len()
        )));
    }
    let build_cost_ns = match a.one("build-seconds")? {
        None => None,
        Some(s) => Some(
            s.parse::<f64>()
                .map_err(|e| format!("--build-seconds {s}: {e}"))?
                * crate::worker::NS_PER_SECOND,
        ),
    };
    let request = Request {
        a: first.clone(),
        b: second.clone(),
        metric,
        offered,
        tail,
        in_flight: driver::SERIAL,
        precision: precision(&a)?,
        seed: seed(&a)?,
        build_cost_ns,
    };
    let arms: Vec<ArmSpec> = std::iter::once(first).chain(second).collect();
    let mut runner = Processes::new(arms, request.seed);
    let outcome =
        driver::measure(&mut runner, &request).map_err(|e| Failure::Failed(e.to_string()))?;
    let host = record::host(a.one("machine")?);
    let date = record::now();
    write(out, &report::render(&outcome, &host, &date))?;
    // A point fed to kiln, or the failure to feed it, is said after the record is written, so a run is
    // kept whatever becomes of feeding it.
    let fed = a
        .one("metrics")?
        .map(|file| metrics_line(&outcome, Path::new(&file)));
    if let Some(dir) = a.one("record")? {
        let kind = if ab { "ab" } else { "point" };
        let path = Path::new(&dir).join(file_name(&date, &name, kind));
        record::write(
            &Record {
                format: record::FORMAT,
                tool: format!("reliquary bench {} ({commit})", env!("CARGO_PKG_VERSION")),
                date,
                host,
                rule: record::rule(),
                what: What::Measured(Box::new(outcome)),
            },
            &path,
        )
        .map_err(|e| Failure::Failed(e.to_string()))?;
        write(out, &format!("record: {}\n", path.display()))?;
    }
    match fed {
        Some(Ok(said)) => write(out, &said).map(|_| Ending::Done),
        Some(Err(why)) => Err(Failure::Failed(format!("metrics: {why}"))),
        None => Ok(Ending::Done),
    }
}

/// A record's file name: its date, bench and kind, the date's colons made dashes for every file
/// system.
fn file_name(date: &str, bench: &str, kind: &str) -> String {
    format!("{}-{bench}-{kind}.json", date.replace(':', "-"))
}

/// Appends the point's median to kiln's metrics file (`$KILN_METRICS`, kiln's docs/ci.md W16-R07) when
/// the record is of record and its units form one state with an interval; says why otherwise.
fn metrics_line(outcome: &Outcome, file: &Path) -> Result<String, String> {
    let Some(arm) = outcome.arms.first() else {
        return Ok("metrics: none fed: no arm\n".to_owned());
    };
    if let Standing::NotOfRecord { .. } = outcome.standing {
        return Ok("metrics: none fed: the record is not of record\n".to_owned());
    }
    let one = match arm.states.as_slice() {
        [one] if one.interval.is_some() => one,
        _ => {
            return Ok(
                "metrics: none fed: the units form no single state with an interval\n".to_owned(),
            );
        }
    };
    let (key, unit) = match outcome.request.metric {
        Metric::Throughput => ("throughput".to_owned(), "ops/s"),
        Metric::Latency { per, of } => (format!("latency_{per}_{of}"), "ns"),
        Metric::Instructions => ("instructions".to_owned(), "instructions"),
    };
    let name = format!("reliquary.bench.{}.{key}", outcome.request.a.bench);
    let line = serde_json::json!({
        "name": name,
        "value": one.median,
        "unit": unit,
        "better": match outcome.request.metric.better() { Better::Higher => "higher", Better::Lower => "lower" },
        "tags": { "interval": one.interval, "units": one.units },
    });
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(file)
        .map_err(|e| format!("{}: {e}", file.display()))?;
    writeln!(f, "{line}").map_err(|e| format!("{}: {e}", file.display()))?;
    Ok(format!(
        "metrics: fed {name} = {} to {}\n",
        one.median,
        file.display()
    ))
}

fn gate_command(args: &[String], commit: &str, out: &mut impl Write) -> Result<Ending, Failure> {
    let a = Args::parse(
        args,
        &[
            "a",
            "b",
            "counter",
            "valgrind",
            "scratch",
            "accept",
            "precision",
            "seed",
            "machine",
            "record",
        ],
    )?;
    let file = a.only("a gated list")?;
    let text =
        std::fs::read_to_string(&file).map_err(|e| Failure::Failed(format!("{file}: {e}")))?;
    let gated: GateFile =
        toml::from_str(&text).map_err(|e| Failure::Failed(format!("{file}: {e}")))?;
    let path = |name: &str| -> Result<PathBuf, Failure> {
        a.one(name)?
            .map(PathBuf::from)
            .ok_or_else(|| Failure::Misuse(format!("--{name} names a build")))
    };
    let counter = match a.one("counter")?.as_deref() {
        Some("os") => Counter::Os,
        Some("cachegrind") => Counter::Cachegrind {
            valgrind: PathBuf::from(a.one("valgrind")?.unwrap_or_else(|| "valgrind".to_owned())),
            scratch: PathBuf::from(a.one("scratch")?.ok_or_else(|| {
                Failure::Misuse("--scratch names cachegrind's directory".to_owned())
            })?),
        },
        Some(other) => return Err(Failure::Misuse(format!("no counter `{other}`"))),
        None => return Err(Failure::Misuse("--counter os or cachegrind".to_owned())),
    };
    let request = GateRequest {
        a: path("a")?,
        b: path("b")?,
        benches: gated.bench,
        counter,
        accept: a.all("accept"),
        precision: precision(&a)?,
        seed: seed(&a)?,
    };
    let outcome = gate::run(&request).map_err(|e| Failure::Failed(e.to_string()))?;
    let mut text = format!("instruction gate, counted by {}\n", outcome.counter);
    for g in &outcome.benches {
        let side = |s: &crate::ab::Side| {
            s.interval.map_or_else(
                || report::DASH.to_owned(),
                |(lo, hi)| format!("[{lo:.1}, {hi:.1}]"),
            )
        };
        text.push_str(&format!(
            "{}: {} instructions an operation: A {}, B {}: {}; {} operations a process ({})\n",
            g.bench.name,
            g.comparison.verdict.words(),
            side(&g.comparison.a),
            side(&g.comparison.b),
            g.outcome,
            g.ops,
            g.ops_why
        ));
    }
    let failed = !outcome.failed.is_empty();
    if failed {
        text.push_str(&format!(
            "FAILED: {}: risen, or undecided; a change accepts it with the trailer \
             `Bench-accepted: <bench>: <why>`\n",
            outcome.failed.join(", ")
        ));
    }
    write(out, &text)?;
    if let Some(dir) = a.one("record")? {
        let date = record::now();
        let path = Path::new(&dir).join(file_name(&date, "gate", "gate"));
        record::write(
            &Record {
                format: record::FORMAT,
                tool: format!("reliquary bench {} ({commit})", env!("CARGO_PKG_VERSION")),
                date,
                host: record::host(a.one("machine")?),
                rule: record::rule(),
                what: What::Gate(Box::new(outcome)),
            },
            &path,
        )
        .map_err(|e| Failure::Failed(e.to_string()))?;
    }
    Ok(if failed { Ending::Failed } else { Ending::Done })
}

/// kiln's metrics API's answer for one metric (kiln `crates/kiln/src/ci/results.rs`, `repo_metrics`).
#[derive(Debug, Deserialize)]
struct KilnAnswer {
    data: KilnMetric,
}

#[derive(Debug, Deserialize)]
struct KilnMetric {
    name: String,
    better: Option<String>,
    series: Vec<KilnSeries>,
}

#[derive(Debug, Deserialize)]
struct KilnSeries {
    fingerprint: String,
    points: Vec<Measured>,
}

/// `bench/changes.toml`: the change points a person has looked at, by commit range, with why.
#[derive(Debug, Default, Deserialize)]
struct Acknowledged {
    #[serde(default)]
    change: Vec<AcknowledgedChange>,
}

#[derive(Debug, Deserialize)]
struct AcknowledgedChange {
    range: String,
    why: String,
}

fn changes_command(args: &[String], out: &mut impl Write) -> Result<Ending, Failure> {
    let a = Args::parse(args, &["acknowledged"])?;
    let file = a.only("a series")?;
    let text =
        std::fs::read_to_string(&file).map_err(|e| Failure::Failed(format!("{file}: {e}")))?;
    let answer: KilnAnswer =
        serde_json::from_str(&text).map_err(|e| Failure::Failed(format!("{file}: {e}")))?;
    let known: Acknowledged = match a.one("acknowledged")? {
        None => Acknowledged::default(),
        Some(f) => {
            let t =
                std::fs::read_to_string(&f).map_err(|e| Failure::Failed(format!("{f}: {e}")))?;
            toml::from_str(&t).map_err(|e| Failure::Failed(format!("{f}: {e}")))?
        }
    };
    let lower = match answer.data.better.as_deref() {
        Some("lower") => true,
        Some("higher") => false,
        other => {
            return Err(Failure::Failed(format!(
                "{}: which way is better is `{}`, not lower or higher",
                answer.data.name,
                other.unwrap_or("unsaid")
            )));
        }
    };
    let mut text = String::new();
    let mut alerts = 0usize;
    for s in &answer.data.series {
        let points = changes::nightly(&s.points);
        let found = changes::changes(&points).map_err(|e| Failure::Failed(e.to_string()))?;
        // A change point can move as nights are added (research/07 F7), so each verdict names the
        // history it was read from.
        text.push_str(&format!(
            "{} on {}: {} nights, the last at {}; {} change points\n",
            answer.data.name,
            s.fingerprint,
            points.len(),
            points.last().map_or("none", |p| p.commit.as_str()),
            found.len()
        ));
        for c in found {
            let worse = if lower {
                c.median_after > c.median_before
            } else {
                c.median_after < c.median_before
            };
            let seen = known.change.iter().find(|k| k.range == c.range());
            let note = match (worse, seen) {
                (_, Some(k)) => format!("acknowledged: {}", k.why),
                (true, None) => {
                    alerts = alerts.saturating_add(1);
                    "ALERT: worse, not acknowledged".to_owned()
                }
                (false, None) => "better".to_owned(),
            };
            text.push_str(&format!(
                "  between {} (git log {}): median {:.6e} → {:.6e}, p = {:.3}: {note}\n",
                c.range(),
                c.range(),
                c.median_before,
                c.median_after,
                c.p_value
            ));
        }
    }
    write(out, &text)?;
    Ok(if alerts > 0 {
        Ending::Failed
    } else {
        Ending::Done
    })
}

fn worker_command<I: BufRead>(
    args: &[String],
    commit: &str,
    input: &mut I,
    out: &mut impl Write,
) -> Result<Ending, Failure> {
    let a = Args::parse(args, &["bench", "seed", "set"])?;
    let name = a.one("bench")?.ok_or_else(|| "--bench".to_owned())?;
    let settings = bench::settings(&a.all("set")).map_err(|e| Failure::Misuse(e.to_string()))?;
    let b = Bench::named(&name, &settings).map_err(|e| Failure::Misuse(e.to_string()))?;
    let seed = seed(&a)?;
    worker::serve(input, out, &b, &name, seed, commit)
        .map_err(|e| Failure::Failed(e.to_string()))?;
    Ok(Ending::Done)
}

fn once(args: &[String]) -> Result<Ending, Failure> {
    let a = Args::parse(args, &["bench", "seed", "set", "ops"])?;
    let name = a.one("bench")?.ok_or_else(|| "--bench".to_owned())?;
    let settings = bench::settings(&a.all("set")).map_err(|e| Failure::Misuse(e.to_string()))?;
    let b = Bench::named(&name, &settings).map_err(|e| Failure::Misuse(e.to_string()))?;
    let ops = a
        .one("ops")?
        .ok_or_else(|| "--ops".to_owned())?
        .parse::<u64>()
        .map_err(|e| format!("--ops: {e}"))?;
    worker::once(&b, seed(&a)?, ops);
    Ok(Ending::Done)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strings(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| (*s).to_owned()).collect()
    }

    /// Percentiles name their quantiles exactly; anything else is refused (AC-RQ0.7.1, T-RQ0.7.4).
    #[test]
    fn percentiles_parse_exactly_and_bad_ones_are_refused() {
        assert_eq!(quantile("p50").unwrap(), (50, 100));
        assert_eq!(quantile("p99").unwrap(), (99, 100));
        assert_eq!(quantile("p99.9").unwrap(), (999, 1000));
        assert_eq!(quantile("p99.99").unwrap(), (9999, 10_000));
        for bad in [
            "99", "p", "p100", "p0", "p-1", "p9x", "p.5", "p50.", "p1.2.3",
        ] {
            assert!(quantile(bad).is_err(), "{bad}");
        }
        assert_eq!(
            metric("latency:p99").unwrap(),
            Metric::Latency { per: 99, of: 100 }
        );
        assert!(metric("speed").is_err());
        assert_eq!(offered("power").unwrap(), Offered::Power);
        assert!(offered("open:0").is_err() && offered("open:fast").is_err());
    }

    /// Arguments that cannot be used end in the usage, never a panic (AC-RQ0.7.1; hostile input).
    #[test]
    fn misuse_ends_in_the_usage() {
        let mut input = std::io::Cursor::new(Vec::new());
        for args in [
            vec![],
            vec!["dance"],
            vec!["run"],
            vec!["run", "pull"],
            vec!["run", "spin"],
            vec!["run", "spin", "--set", "work=1", "--metric", "cpu"],
            vec!["run", "spin", "--set", "work=1", "--precision", "2"],
            vec!["run", "spin", "--frobnicate", "1"],
            vec!["run", "spin", "--seed"],
            vec!["gate", "bench/gated.toml"],
            vec!["run", "spin", "--set", "work=1", "--b", "elsewhere"],
            vec!["ab", "spin", "--build", "elsewhere"],
            vec![
                "ab", "spin", "--a", "x", "--a", "y", "--b", "z", "--a-set", "work=1", "--b-set",
                "work=1",
            ],
            vec!["changes"],
        ] {
            let (mut out, mut err) = (Vec::new(), Vec::new());
            let e = main(&strings(&args), "abc", &mut input, &mut out, &mut err);
            let said = String::from_utf8(err).unwrap();
            assert!(
                e == Ending::Misuse || e == Ending::Failed,
                "{args:?} ended {e:?}"
            );
            assert!(said.contains("reliquary bench:"), "{args:?}: {said}");
        }
    }

    /// The change-point command reads kiln's answer, alerts on an unacknowledged worsening with its
    /// commit range, and stays quiet once it is acknowledged (AC-RQ0.7.6, T-RQ0.7.10).
    #[test]
    fn a_worsening_alerts_with_its_range_until_acknowledged() {
        let dir = tempfile::tempdir().unwrap();
        let mut points = Vec::new();
        for night in 0..40i64 {
            let value = if night < 25 { 1_000.0 } else { 1_300.0 } + (night % 3) as f64;
            points.push(serde_json::json!({
                "sha": format!("c{night:02}"), "run": "r", "value": value, "unit": "ns",
                "ts": 1_790_000_000 + night * 86_400
            }));
        }
        let answer = serde_json::json!({"repo": "reliquary", "branch": "main", "days": 365,
            "data": {"name": "reliquary.bench.judge.latency_99_100", "better": "lower",
                "series": [{"fingerprint": "x86_64 test", "points": points}]}});
        let series = dir.path().join("series.json");
        std::fs::write(&series, answer.to_string()).unwrap();
        let run = |extra: &[&str]| {
            let mut args = strings(&["changes", series.to_str().unwrap()]);
            args.extend(strings(extra));
            let (mut out, mut err) = (Vec::new(), Vec::new());
            let e = main(
                &args,
                "abc",
                &mut std::io::Cursor::new(Vec::new()),
                &mut out,
                &mut err,
            );
            (e, String::from_utf8(out).unwrap())
        };
        let (e, text) = run(&[]);
        assert_eq!(e, Ending::Failed, "{text}");
        assert!(text.contains("between c24..c25"), "{text}");
        assert!(text.contains("the last at c39"), "{text}");
        assert!(text.contains("ALERT"), "{text}");
        let known = dir.path().join("changes.toml");
        std::fs::write(
            &known,
            "[[change]]\nrange = \"c24..c25\"\nwhy = \"the test's planted change\"\n",
        )
        .unwrap();
        let (e, text) = run(&["--acknowledged", known.to_str().unwrap()]);
        assert_eq!(e, Ending::Done, "{text}");
        assert!(
            text.contains("acknowledged: the test's planted change"),
            "{text}"
        );
    }
}
