//! The process that plans a point, starts its workers, interleaves its arms and judges what they
//! measured.
//!
//! A point is measured in four steps, each recorded with why:
//! 1. **The round's size** (Mantle's design §5a): enough operations that the tail the report states has
//!    its interval after the fewest rounds, `⌈fewest(q)/10⌉`; that the operations still out at a round's
//!    end cost it at most the precision, `⌈in flight/(2 × precision)⌉`; and, from a sizing round not
//!    kept, enough that the operations' spread leaves the round within the precision,
//!    `⌈(1.96·cv/precision)²⌉` (Jain's sample size).
//! 2. **The levels** (AC-RQ0.7.7, `levels`): a dimensioning experiment of ten rounds in each of ten
//!    process starts (ten, the fewest at which each level's order in time is tested), every build given
//!    when there are several, from which each level's own variance and cost set its count.
//! 3. **The units**: the top level's units, in cycles whose arms' order is drawn (`ab`), until every
//!    arm's units are settled or the most a point runs; each process starts with one round discarded
//!    (Georges, Buytaert and Eeckhout, OOPSLA '07, §4.1) and with its environment's size drawn anew
//!    (Mytkowicz et al., ASPLOS '09).
//! 4. **The judgment**: each arm's units judged (`rounds`), each state's latency quantiles over its own
//!    operations (`tails`), the arms compared (`ab`), and the record held to the load rule (`load`).

use std::io::{BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, ChildStdout, Command as Process, Stdio};

use hyper_measure::quantile::{Level as Quantile, samples_for};
use serde::Serialize;

use crate::ab::{self, Arm, Comparison, cycle_order};
use crate::drive::Load;
use crate::levels::{self, Count, Dimensioning, Level, LevelPlan, Nested};
use crate::load::{self, Own, Reading, Standing};
use crate::protocol::{self, Command, Metric, Reply, RoundResult, RoundSpec};
use crate::rng::SplitMix64;
use crate::rounds::{self, FEWEST_TESTED, Judgment, Policy, as_f64, median, share};
use crate::tails::{self, Counts, Tail};

/// One arm: its builds, and the bench with its settings.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ArmSpec {
    pub builds: Vec<PathBuf>,
    pub bench: String,
    pub settings: Vec<String>,
}

/// How operations are offered, as asked.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Offered {
    /// Closed loop.
    Closed,
    /// Open loop at a stated rate a second.
    Open { per_second: f64 },
    /// Open loop at half the closed-loop rate measured first: the point of greatest power, throughput
    /// over delay, of an M/M/1 queue (Kleinrock, "Power and Deterministic Rules of Thumb for
    /// Probabilistic Problems in Computer Communications", ICC '79), the operating point Mantle's
    /// admission seeks (research/01 §5).
    Power,
}

/// The share of the closed-loop rate an M/M/1 queue's power, `λ/T = λ(μ − λ)`, is greatest at: one
/// half (Kleinrock, ICC '79).
pub(crate) const POWER_SHARE: f64 = 0.5;

/// Operations a worker has in flight: one, as it runs them one after another on its thread.
pub const SERIAL: u64 = 1;

/// The median as a quantile, `1/2`.
pub const MEDIAN: (u64, u64) = (1, 2);

/// What to measure.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Request {
    pub a: ArmSpec,
    /// The second arm of an A|B; none for one point.
    pub b: Option<ArmSpec>,
    pub metric: Metric,
    pub offered: Offered,
    /// The highest latency quantile the report must state, which sizes the round.
    pub tail: (u64, u64),
    /// Operations in flight at once: one, until a bench drives more.
    pub in_flight: u64,
    /// The widest interval a state may end with, as a share of its median.
    pub precision: f64,
    pub seed: u64,
    /// The time a build takes, stated, when several builds make a level.
    pub build_cost_ns: Option<f64>,
}

/// Why a point could not be measured.
#[derive(Debug)]
pub enum DriverError {
    /// A worker could not be started, or failed.
    Worker { arm: &'static str, why: String },
    /// A worker refused a round, with its reason.
    Refused { arm: &'static str, why: String },
    /// The levels could not be dimensioned.
    Levels(levels::LevelsError),
    /// The request cannot be measured as asked.
    Request(String),
}

impl std::fmt::Display for DriverError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Worker { arm, why } => write!(f, "arm {arm}'s worker: {why}"),
            Self::Refused { arm, why } => write!(f, "arm {arm}'s worker refused: {why}"),
            Self::Levels(e) => write!(f, "the levels: {e}"),
            Self::Request(why) => f.write_str(why),
        }
    }
}

impl std::error::Error for DriverError {}

/// What starts and runs workers: processes (`Processes`), or a simulation in the tests.
pub trait Runner {
    type Process;
    /// Starts a worker of `arm`'s build `build`, its draws from `seed`, and returns it with the commit
    /// its build names.
    fn start(
        &mut self,
        arm: Arm,
        build: usize,
        seed: u64,
    ) -> Result<(Self::Process, String), DriverError>;
    /// Runs one round on a worker.
    fn round(
        &mut self,
        arm: Arm,
        process: &mut Self::Process,
        spec: &RoundSpec,
    ) -> Result<RoundResult, DriverError>;
    /// Ends a worker.
    fn stop(&mut self, arm: Arm, process: Self::Process) -> Result<(), DriverError>;
    /// Nanoseconds on the driver's monotonic clock.
    fn now_ns(&mut self) -> u64;
    /// The machine's one-minute load average, where the platform keeps one.
    fn load(&mut self) -> Option<f64>;
    /// CPU the run's processes have used so far, the driver's own with its workers'.
    fn cpu_ns(&mut self) -> u64;
}

/// One unit of the top level, as measured.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Unit {
    /// Its place in the arm's units, in time order.
    pub index: usize,
    pub build: usize,
    pub value: f64,
    pub ops: u64,
    pub rounds: usize,
    pub late_max_ns: u64,
    #[serde(skip)]
    pub counts: Counts,
}

/// A state of an arm's units, as a report gives it.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StateReport {
    pub median: f64,
    pub interval: Option<(f64, f64)>,
    pub spread: Option<f64>,
    pub units: usize,
    /// The state's share of the units when there are two states, and its interval when the units are
    /// independent.
    pub share: Option<f64>,
    pub share_interval: Option<(f64, f64)>,
    /// Whether the units were independent: a tail's interval, like every other, is stated only then.
    pub independent: bool,
    pub operations: u64,
    /// p50, p99 and p99.9 of the state's operations' latencies from their intended starts; none where
    /// the state has too few operations to state one.
    pub tails: Vec<(String, Option<Tail>)>,
}

/// One arm's outcome.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ArmOutcome {
    pub arm: Arm,
    pub spec: ArmSpec,
    pub commits: Vec<String>,
    pub judgment: Judgment,
    pub states: Vec<StateReport>,
    pub units: Vec<Unit>,
}

/// The round's size and the load offered, each with why.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Size {
    pub ops: u64,
    pub why: String,
    pub load: Load,
    pub load_why: String,
}

/// A point or an A|B, measured.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Outcome {
    pub request: Request,
    pub size: Size,
    pub levels: Vec<LevelPlan>,
    pub arms: Vec<ArmOutcome>,
    pub comparison: Option<Comparison>,
    pub readings: Vec<Reading>,
    pub standing: Standing,
}

/// The tails a state's report states: its median, and the two tails Mantle's and hyper-raft's reports
/// state beside it.
const REPORTED: &[(&str, Quantile)] = &[
    ("p50", Quantile::P50),
    ("p99", Quantile::P99),
    ("p99.9", Quantile::P999),
];

/// The share of the operations in flight that a round's end leaves short: its last operations run as
/// the depth falls from `N` to none, half of it on average (Mantle's design §5a; research/07), so a round
/// of at least `N·DRAIN/precision` operations loses at most the precision to its end.
const DRAIN: f64 = 0.5;

/// Measures `request` with `runner`.
pub fn measure<R: Runner>(runner: &mut R, request: &Request) -> Result<Outcome, DriverError> {
    let arms: Vec<(Arm, &ArmSpec)> = std::iter::once((Arm::A, &request.a))
        .chain(request.b.as_ref().map(|b| (Arm::B, b)))
        .collect();
    if arms.iter().any(|(_, s)| s.builds.is_empty()) {
        return Err(DriverError::Request("an arm names no build".to_owned()));
    }
    let mut rng = SplitMix64::new(request.seed);
    let mut readings = Vec::new();
    let mut own = Own::default();
    let read_load = |runner: &mut R, own: &mut Own, when: String| {
        let (at, cpu) = (runner.now_ns(), runner.cpu_ns());
        Reading {
            when,
            one_minute: runner.load(),
            own: own.at(at, cpu),
        }
    };
    readings.push(read_load(
        runner,
        &mut own,
        "before the first unit".to_owned(),
    ));
    let mut commits: Vec<Vec<String>> = vec![Vec::new(); arms.len()];

    let size = size(runner, request, &arms, &mut rng, &mut commits)?;
    let levels = dimension(runner, request, &arms, &size, &mut rng, &mut commits)?;
    let shape = Shape::of(&levels, &arms)?;

    let policy = Policy {
        min: FEWEST_TESTED,
        max: Policy::standard().limit(),
        precision: request.precision,
    };
    let mut units: Vec<Vec<Unit>> = vec![Vec::new(); arms.len()];
    // The top level of rounds: one process an arm for the whole point.
    let mut live: Vec<Option<R::Process>> = Vec::new();
    if shape.top == Level::Rounds {
        for &(arm, _) in &arms {
            let (mut p, commit) = runner.start(arm, 0, rng.next_u64())?;
            discard(runner, arm, &mut p, request, &size)?;
            note(&mut commits, arm, commit);
            live.push(Some(p));
        }
    }
    // Builds are the top level only as many times as builds were given: a build measured again is not a
    // new sample of its layout.
    let most = if shape.top == Level::Builds {
        policy.limit().min(shape.builds)
    } else {
        policy.limit()
    };
    for cycle in 0..most {
        let order: Vec<Arm> = if request.b.is_some() {
            let (first, second) = cycle_order(&mut rng);
            vec![first, second]
        } else {
            vec![Arm::A]
        };
        for arm in order {
            let i = arm_index(arm);
            let made = match live.get_mut(i).and_then(Option::as_mut) {
                Some(p) => {
                    let r = runner.round(arm, p, &spec(request, &size, rng.next_u64()))?;
                    unit_of(arm, cycle, 0, std::slice::from_ref(&r))?
                }
                None => {
                    let build = cycle.checked_rem(shape.builds).unwrap_or(0);
                    let (results, commit) =
                        process_unit(runner, arm, build, request, &size, &shape, &mut rng)?;
                    note(&mut commits, arm, commit);
                    unit_of(arm, cycle, build, &results)?
                }
            };
            if let Some(list) = units.get_mut(i) {
                list.push(made);
            }
        }
        readings.push(read_load(
            runner,
            &mut own,
            format!("after cycle {}", cycle.saturating_add(1)),
        ));
        let enough = units.iter().all(|u| {
            let values: Vec<f64> = u.iter().map(|x| x.value).collect();
            policy.enough(&rounds::judge(&values))
        });
        if enough {
            break;
        }
    }
    for (i, p) in live.into_iter().enumerate() {
        if let Some(p) = p {
            runner.stop(if i == 0 { Arm::A } else { Arm::B }, p)?;
        }
    }
    readings.push(read_load(runner, &mut own, "at the end".to_owned()));

    let mut outcomes = Vec::new();
    for (i, (arm, spec)) in arms.iter().enumerate() {
        let list = units.get(i).cloned().unwrap_or_default();
        let values: Vec<f64> = list.iter().map(|u| u.value).collect();
        let judgment = rounds::judge(&values);
        let states = judgment
            .states
            .iter()
            .map(|s| {
                state_report(
                    s,
                    &list,
                    judgment.states.len(),
                    judgment.order == rounds::Order::Independent,
                )
            })
            .collect();
        outcomes.push(ArmOutcome {
            arm: *arm,
            spec: (*spec).clone(),
            commits: commits.get(i).cloned().unwrap_or_default(),
            judgment,
            states,
            units: list,
        });
    }
    let comparison = match outcomes.as_slice() {
        [a, b] => Some(ab::compare(
            &a.judgment,
            &b.judgment,
            request.metric.better(),
        )),
        _ => None,
    };
    let named: Vec<Option<String>> = commits
        .iter()
        .map(|c| {
            let mut distinct = c.clone();
            distinct.sort();
            distinct.dedup();
            match distinct.as_slice() {
                [one] => Some(one.clone()),
                _ => None,
            }
        })
        .collect();
    let standing = load::standing(&readings, &named, std::env::consts::OS);
    Ok(Outcome {
        request: request.clone(),
        size,
        levels,
        arms: outcomes,
        comparison,
        readings,
        standing,
    })
}

fn arm_index(arm: Arm) -> usize {
    match arm {
        Arm::A => 0,
        Arm::B => 1,
    }
}

fn note(commits: &mut [Vec<String>], arm: Arm, commit: String) {
    if let Some(c) = commits.get_mut(arm_index(arm))
        && !c.contains(&commit)
    {
        c.push(commit);
    }
}

fn spec(request: &Request, size: &Size, index: u64) -> RoundSpec {
    RoundSpec {
        index,
        ops: size.ops,
        load: size.load,
        metric: request.metric,
    }
}

/// One round run and not kept: a process's first (Georges et al. §4.1).
fn discard<R: Runner>(
    runner: &mut R,
    arm: Arm,
    p: &mut R::Process,
    request: &Request,
    size: &Size,
) -> Result<(), DriverError> {
    runner.round(arm, p, &spec(request, size, u64::MAX))?;
    Ok(())
}

/// The top level's shape: which level is repeated until settled, and the fixed counts below it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Shape {
    top: Level,
    rounds_per_process: usize,
    processes_per_build: usize,
    builds: usize,
}

impl Shape {
    fn of(levels: &[LevelPlan], arms: &[(Arm, &ArmSpec)]) -> Result<Self, DriverError> {
        let fixed = |level: Level| {
            levels
                .iter()
                .find(|l| l.level == level)
                .map_or(1, |l| match l.count {
                    Count::Fixed { times } => times,
                    Count::UntilSettled { .. } => 1,
                })
        };
        let top = levels
            .iter()
            .find(|l| matches!(l.count, Count::UntilSettled { .. }))
            .map_or(Level::Rounds, |l| l.level);
        let builds = arms.iter().map(|(_, s)| s.builds.len()).min().unwrap_or(1);
        if top == Level::Builds && builds < FEWEST_TESTED {
            return Err(DriverError::Request(format!(
                "the builds vary by themselves, so they are the level repeated until settled, which \
                 takes at least {FEWEST_TESTED} builds an arm; {builds} were given"
            )));
        }
        Ok(Self {
            top,
            rounds_per_process: fixed(Level::Rounds),
            processes_per_build: fixed(Level::ProcessStarts),
            builds: if top == Level::Builds { builds } else { 1 },
        })
    }
}

/// One unit that starts processes: a process start's rounds, or a build's process starts.
fn process_unit<R: Runner>(
    runner: &mut R,
    arm: Arm,
    build: usize,
    request: &Request,
    size: &Size,
    shape: &Shape,
    rng: &mut SplitMix64,
) -> Result<(Vec<RoundResult>, String), DriverError> {
    let starts = if shape.top == Level::Builds {
        shape.processes_per_build
    } else {
        1
    };
    let mut results = Vec::new();
    let mut commit = String::new();
    for _ in 0..starts {
        let (mut p, c) = runner.start(arm, build, rng.next_u64())?;
        commit = c;
        discard(runner, arm, &mut p, request, size)?;
        for _ in 0..shape.rounds_per_process {
            // Each round's open-loop schedule is its own draw, so no two units replay one schedule.
            results.push(runner.round(arm, &mut p, &spec(request, size, rng.next_u64()))?);
        }
        runner.stop(arm, p)?;
    }
    Ok((results, commit))
}

/// A unit from its rounds: the median of their values, their operations' latencies together; a
/// round whose latencies name a bucket past the last is refused, never dropped.
fn unit_of(
    arm: Arm,
    index: usize,
    build: usize,
    results: &[RoundResult],
) -> Result<Unit, DriverError> {
    let mut values: Vec<f64> = results.iter().map(|r| r.value).collect();
    values.sort_by(f64::total_cmp);
    let mut counts = Counts::default();
    for r in results {
        let c = Counts::from_sparse(&r.latencies).map_err(|e| worker_error(arm, e))?;
        counts.add(&c);
    }
    Ok(Unit {
        index,
        build,
        value: median(&values),
        ops: results
            .iter()
            .map(|r| r.ops)
            .fold(0u64, u64::saturating_add),
        rounds: results.len(),
        late_max_ns: results.iter().map(|r| r.late_max_ns).max().unwrap_or(0),
        counts,
    })
}

fn state_report(
    s: &rounds::State,
    units: &[Unit],
    states: usize,
    independent: bool,
) -> StateReport {
    let mut counts = Counts::default();
    for &i in &s.rounds {
        if let Some(u) = units.get(i) {
            counts.add(&u.counts);
        }
    }
    // A share's interval, like a median's, needs independent units (Mantle's design §2).
    let share_of = (states > 1).then(|| as_f64(s.rounds.len()) / as_f64(units.len().max(1)));
    let share_interval = (states > 1 && independent).then(|| share(s.rounds.len(), units.len()));
    StateReport {
        median: s.median,
        interval: s.interval,
        spread: s.spread(),
        units: s.rounds.len(),
        share: share_of,
        share_interval,
        independent,
        operations: counts.count(),
        tails: REPORTED
            .iter()
            .map(|(name, q)| ((*name).to_owned(), tails::tail(&counts, *q)))
            .collect(),
    }
}

/// The round's size (Mantle's design §5a), from a sizing round of each arm, not kept.
fn size<R: Runner>(
    runner: &mut R,
    request: &Request,
    arms: &[(Arm, &ArmSpec)],
    rng: &mut SplitMix64,
    commits: &mut [Vec<String>],
) -> Result<Size, DriverError> {
    let (per, of) = request.tail;
    let tail = Quantile::new(per, of)
        .map_err(|e| DriverError::Request(format!("the tail {per}/{of}: {e}")))?;
    if !(request.precision.is_finite() && request.precision > 0.0) {
        return Err(DriverError::Request(format!(
            "a precision of {} is not a positive share",
            request.precision
        )));
    }
    let fewest = samples_for(tail, tails::COVERAGE)
        .map_err(|e| DriverError::Request(format!("the tail {per}/{of}: {e}")))?;
    let by_tail = fewest.div_ceil(u64::try_from(FEWEST_TESTED).unwrap_or(u64::MAX));
    // A latency metric's round is valued at its own quantile, which its operations must resolve: the
    // fewest whose interval of that quantile has an upper end (hyper-measure's `samples_for`).
    let by_metric = match request.metric.level() {
        Some(q) => samples_for(q, tails::COVERAGE)
            .map_err(|e| DriverError::Request(format!("the metric's quantile: {e}")))?,
        None => 1,
    };
    let by_flight = ceil_count(as_u64_f64(request.in_flight) * DRAIN / request.precision);
    let first = by_tail.max(by_flight).max(by_metric);
    let closed = |ops: u64| RoundSpec {
        index: u64::MAX,
        ops,
        load: Load::Closed,
        metric: request.metric,
    };
    // The load: as stated, or at the power point of the slower arm's closed-loop rate, so no arm is
    // offered past its own.
    let (load, load_why) = match request.offered {
        Offered::Closed => (
            Load::Closed,
            "closed loop: each operation when the last answers, a control".to_owned(),
        ),
        Offered::Open { per_second } => (
            Load::Open {
                per_second,
                seed: rng.next_u64(),
            },
            format!("open loop, Poisson at the stated {per_second} a second"),
        ),
        Offered::Power => {
            let mut slowest = f64::INFINITY;
            for &(arm, _) in arms {
                let (mut p, commit) = runner.start(arm, 0, rng.next_u64())?;
                note(commits, arm, commit);
                runner.round(arm, &mut p, &closed(first))?;
                let r = runner.round(arm, &mut p, &closed(first))?;
                runner.stop(arm, p)?;
                let rate = as_u64_f64(r.ops)
                    / (as_u64_f64(r.wall_ns.max(1)) / crate::worker::NS_PER_SECOND);
                slowest = slowest.min(rate);
            }
            let per_second = slowest * POWER_SHARE;
            (
                Load::Open {
                    per_second,
                    seed: rng.next_u64(),
                },
                format!(
                    "open loop, Poisson at {per_second:.1} a second: {POWER_SHARE} of {slowest:.1}, the \
                     slower arm's closed-loop rate in a round of {first} after one discarded, M/M/1's \
                     point of greatest power (Kleinrock, ICC '79)"
                ),
            )
        }
    };
    let mut cv = 0.0f64;
    for &(arm, _) in arms {
        let (mut p, commit) = runner.start(arm, 0, rng.next_u64())?;
        note(commits, arm, commit);
        runner.round(arm, &mut p, &closed(first))?;
        let r = runner.round(
            arm,
            &mut p,
            &RoundSpec {
                index: u64::MAX,
                ops: first,
                load,
                metric: request.metric,
            },
        )?;
        runner.stop(arm, p)?;
        cv = cv.max(r.op_spread);
    }
    let root = rounds::Z_95 * cv / request.precision;
    let by_spread = ceil_count(root * root);
    let ops = first.max(by_spread);
    Ok(Size {
        ops,
        why: format!(
            "the most of ⌈{fewest}/{FEWEST_TESTED}⌉ = {by_tail} (the {per}/{of} quantile's interval needs \
             {fewest} operations, after the fewest rounds), ⌈{} × {DRAIN}/{}⌉ = {by_flight} (operations in \
             flight at a round's end cost it at most the precision), {by_metric} (the operations a round's \
             own quantile needs, for a latency metric), and ⌈({} × {cv:.3}/{})²⌉ = {by_spread} (Jain's \
             sample size, from the operations' spread in a sizing round of {first}, not kept); Mantle's \
             design §5a",
            request.in_flight,
            request.precision,
            rounds::Z_95,
            request.precision
        ),
        load,
        load_why,
    })
}

fn as_u64_f64(v: u64) -> f64 {
    rounds::u64_as_f64(v)
}

/// `⌈x⌉` as a count, at least one; a count past what a round's latencies could be held for is the
/// worker's to refuse (it reserves them before it starts).
fn ceil_count(x: f64) -> u64 {
    if !(x.is_finite() && x > 1.0) {
        return 1;
    }
    // Doubling then halving, by shifts: each step compares floats, so no float is cast.
    let mut high = 1u64;
    while as_u64_f64(high) < x {
        let Some(next) = high.checked_shl(1).filter(|&n| n > high) else {
            return u64::MAX;
        };
        high = next;
    }
    let mut low = high >> 1;
    while high.saturating_sub(low) > 1 {
        let mid = low.saturating_add(high.saturating_sub(low) >> 1);
        if as_u64_f64(mid) >= x {
            high = mid;
        } else {
            low = mid;
        }
    }
    high
}

/// The levels (KJ13): a dimensioning experiment, then each level's count.
fn dimension<R: Runner>(
    runner: &mut R,
    request: &Request,
    arms: &[(Arm, &ArmSpec)],
    size: &Size,
    rng: &mut SplitMix64,
    commits: &mut [Vec<String>],
) -> Result<Vec<LevelPlan>, DriverError> {
    let builds = arms.iter().map(|(_, s)| s.builds.len()).min().unwrap_or(1);
    let with_builds = builds > 1;
    let build_cost = match (with_builds, request.build_cost_ns) {
        (true, Some(c)) => Some(c),
        (true, None) => {
            return Err(DriverError::Request(
                "several builds make a level, whose cost must be stated (--build-seconds)"
                    .to_owned(),
            ));
        }
        (false, _) => None,
    };
    let (r1, r2) = (FEWEST_TESTED, FEWEST_TESTED);
    let r3 = if with_builds { builds } else { 1 };
    let mut values: Vec<Vec<f64>> = vec![Vec::new(); arms.len()];
    let mut round_costs: Vec<f64> = Vec::new();
    let mut start_costs: Vec<f64> = Vec::new();
    for build in 0..r3 {
        for _ in 0..r2 {
            let order: Vec<Arm> = if request.b.is_some() {
                let (first, second) = cycle_order(rng);
                vec![first, second]
            } else {
                vec![Arm::A]
            };
            for arm in order {
                let began = runner.now_ns();
                let (mut p, commit) = runner.start(arm, build, rng.next_u64())?;
                note(commits, arm, commit);
                discard(runner, arm, &mut p, request, size)?;
                start_costs.push(as_u64_f64(runner.now_ns().saturating_sub(began)));
                for _ in 0..r1 {
                    let result = runner.round(arm, &mut p, &spec(request, size, rng.next_u64()))?;
                    round_costs.push(as_u64_f64(result.wall_ns.max(1)));
                    if let Some(v) = values.get_mut(arm_index(arm)) {
                        v.push(result.value);
                    }
                }
                runner.stop(arm, p)?;
            }
        }
    }
    let typical = |mut v: Vec<f64>| {
        v.sort_by(f64::total_cmp);
        median(&v)
    };
    let mut level_names = vec![Level::Rounds, Level::ProcessStarts];
    let mut counts = vec![r1, r2];
    let mut costs = vec![typical(round_costs).max(1.0), typical(start_costs).max(1.0)];
    if let Some(c) = build_cost {
        level_names.push(Level::Builds);
        counts.push(r3);
        costs.push(c);
    }
    // Each arm's plan; the plan run is each level's larger count, so neither arm is under-repeated.
    let mut plans = Vec::new();
    for v in values {
        let dim = Dimensioning {
            levels: level_names.clone(),
            nested: Nested {
                counts: counts.clone(),
                values: v,
            },
            costs_ns: costs.clone(),
        };
        plans.push(
            levels::plan(&dim, FEWEST_TESTED, Policy::standard().limit())
                .map_err(DriverError::Levels)?,
        );
    }
    let mut levels = combine(plans);
    if !with_builds {
        // One build is one sample of code layout however often it runs (research/07 F32): say so, so
        // the record states the builds' count and why with the others'.
        let once = 1;
        levels.push(LevelPlan {
            level: Level::Builds,
            count: Count::Fixed { times: once },
            own_variance: None,
            cost_ns: None,
            why: "one build an arm was given: its variation, code layout's, is not measured, so the \
                  interval covers process starts and rounds only (KJ13 §3; STABILIZER, research/07 F32)"
                .to_owned(),
        });
    }
    Ok(levels)
}

/// The plans of two arms as one: the higher of the arms' top levels is repeated until settled; each
/// level below it runs the larger of the arms' fixed counts, an arm whose own top lay lower giving none
/// there; each level above it runs once. Neither arm is repeated less than its own plan asks.
fn combine(plans: Vec<Vec<LevelPlan>>) -> Vec<LevelPlan> {
    let top = |p: &[LevelPlan]| {
        p.iter()
            .position(|l| matches!(l.count, Count::UntilSettled { .. }))
            .unwrap_or(0)
    };
    let mut both = plans.into_iter();
    let Some(mut first) = both.next() else {
        return Vec::new();
    };
    let Some(second) = both.next() else {
        return first;
    };
    let (top_a, top_b) = (top(&first), top(&second));
    let at = top_a.max(top_b);
    let once = 1;
    for (i, (mine, theirs)) in first.iter_mut().zip(second).enumerate() {
        let take = if i == at {
            top_b > top_a
        } else if i > at {
            mine.count = Count::Fixed { times: once };
            false
        } else {
            match (&mine.count, &theirs.count) {
                (Count::Fixed { times: a }, Count::Fixed { times: b }) => b > a,
                (Count::UntilSettled { .. }, Count::Fixed { .. }) => true,
                _ => false,
            }
        };
        if take {
            mine.count = theirs.count.clone();
            mine.own_variance = theirs.own_variance;
            mine.why = format!("arm B's: {}", theirs.why);
        }
    }
    first
}

/// Workers as processes of each arm's builds.
pub struct Processes {
    arms: Vec<ArmSpec>,
    clock: std::time::Instant,
    rng: SplitMix64,
    /// CPU of workers already ended.
    ended_cpu_ns: u64,
    /// CPU of each arm's live worker, as its last round said: an arm runs one worker at a time, so
    /// this holds one figure an arm.
    live_cpu_ns: Vec<u64>,
}

/// A worker process.
pub struct Worker {
    child: Child,
    input: ChildStdin,
    output: BufReader<ChildStdout>,
}

impl Processes {
    /// Workers for these arms, A then B; `seed` draws each start's environment size.
    #[must_use]
    pub fn new(arms: Vec<ArmSpec>, seed: u64) -> Self {
        let live_cpu_ns = vec![0; arms.len()];
        Self {
            arms,
            clock: std::time::Instant::now(),
            rng: SplitMix64::new(seed),
            ended_cpu_ns: 0,
            live_cpu_ns,
        }
    }
}

/// Format: the largest environment padding a start draws, a 4 KiB page, the range of environment sizes
/// over which Mytkowicz et al. (ASPLOS '09) found a process's stack alignment, and its speed, to vary.
pub(crate) const PAD_MOST: u64 = 4_096;

fn worker_error(arm: Arm, why: impl std::fmt::Display) -> DriverError {
    DriverError::Worker {
        arm: arm.letter(),
        why: why.to_string(),
    }
}

impl Runner for Processes {
    type Process = Worker;

    fn start(
        &mut self,
        arm: Arm,
        build: usize,
        seed: u64,
    ) -> Result<(Worker, String), DriverError> {
        let spec = self
            .arms
            .get(arm_index(arm))
            .ok_or_else(|| worker_error(arm, "no such arm"))?;
        let path = spec
            .builds
            .get(build)
            .ok_or_else(|| worker_error(arm, format!("no build {build}")))?;
        let pad_len = usize::try_from(self.rng.below(PAD_MOST)).unwrap_or(0);
        let mut command = Process::new(path);
        command
            .args([
                "bench",
                "worker",
                "--bench",
                &spec.bench,
                "--seed",
                &seed.to_string(),
            ])
            .env("RELIQUARY_BENCH_PAD", "x".repeat(pad_len))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());
        for s in &spec.settings {
            command.args(["--set", s]);
        }
        let mut child = command
            .spawn()
            .map_err(|e| worker_error(arm, format!("{}: {e}", path.display())))?;
        let (Some(input), Some(output)) = (child.stdin.take(), child.stdout.take()) else {
            return Err(worker_error(arm, "its pipes were not made"));
        };
        let mut worker = Worker {
            child,
            input,
            output: BufReader::new(output),
        };
        match protocol::read::<Reply>(&mut worker.output, protocol::most_reply()) {
            Ok(Some(Reply::Hello { commit, .. })) => Ok((worker, commit)),
            Ok(Some(Reply::Refused { why })) => Err(DriverError::Refused {
                arm: arm.letter(),
                why,
            }),
            Ok(other) => Err(worker_error(
                arm,
                format!("said {other:?} where a hello was due"),
            )),
            Err(e) => Err(worker_error(arm, e)),
        }
    }

    fn round(
        &mut self,
        arm: Arm,
        w: &mut Worker,
        spec: &RoundSpec,
    ) -> Result<RoundResult, DriverError> {
        let mut line =
            serde_json::to_vec(&Command::Round(*spec)).map_err(|e| worker_error(arm, e))?;
        line.push(b'\n');
        w.input.write_all(&line).map_err(|e| worker_error(arm, e))?;
        w.input.flush().map_err(|e| worker_error(arm, e))?;
        // A round answers once its operations are done; a worker that ends without answering is
        // read as the end of its output, so the read ends on the worker's own liveness.
        match protocol::read::<Reply>(&mut w.output, protocol::most_reply()) {
            Ok(Some(Reply::Round(r))) => {
                if let Some(slot) = self.live_cpu_ns.get_mut(arm_index(arm)) {
                    *slot = r.cpu_total_ns;
                }
                Ok(r)
            }
            Ok(Some(Reply::Refused { why })) => Err(DriverError::Refused {
                arm: arm.letter(),
                why,
            }),
            Ok(other) => Err(worker_error(
                arm,
                format!("said {other:?} where a round was due"),
            )),
            Err(e) => Err(worker_error(arm, e)),
        }
    }

    fn stop(&mut self, arm: Arm, mut w: Worker) -> Result<(), DriverError> {
        let mut line = serde_json::to_vec(&Command::End).map_err(|e| worker_error(arm, e))?;
        line.push(b'\n');
        w.input.write_all(&line).map_err(|e| worker_error(arm, e))?;
        w.input.flush().map_err(|e| worker_error(arm, e))?;
        drop(w.input);
        let status = w.child.wait().map_err(|e| worker_error(arm, e))?;
        let last = self
            .live_cpu_ns
            .get_mut(arm_index(arm))
            .map_or(0, std::mem::take);
        self.ended_cpu_ns = self.ended_cpu_ns.saturating_add(last);
        if status.success() {
            Ok(())
        } else {
            Err(worker_error(arm, format!("ended {status}")))
        }
    }

    fn now_ns(&mut self) -> u64 {
        u64::try_from(self.clock.elapsed().as_nanos()).unwrap_or(u64::MAX)
    }

    fn load(&mut self) -> Option<f64> {
        hyper_measure::usage::load()
    }

    fn cpu_ns(&mut self) -> u64 {
        let own = hyper_measure::usage::this().map_or(0, |u| u.cpu_ns());
        own.saturating_add(self.ended_cpu_ns).saturating_add(
            self.live_cpu_ns
                .iter()
                .fold(0u64, |a, &b| a.saturating_add(b)),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Workers simulated: arm B's operations take `slow` times A's; each process draws a factor of
    /// `process_jitter` and each round one of `round_jitter`; time moves only as they run.
    struct Sim {
        now: u64,
        rng: SplitMix64,
        slow: f64,
        process_jitter: f64,
        round_jitter: f64,
        load: f64,
        started: usize,
    }

    struct SimProcess {
        factor: f64,
    }

    impl Runner for Sim {
        type Process = SimProcess;
        fn start(
            &mut self,
            arm: Arm,
            _: usize,
            _: u64,
        ) -> Result<(SimProcess, String), DriverError> {
            self.now += 5_000_000;
            self.started += 1;
            let arm_factor = if arm == Arm::B { self.slow } else { 1.0 };
            Ok((
                SimProcess {
                    factor: arm_factor * (1.0 + self.process_jitter * self.rng.unit()),
                },
                "abc123".to_owned(),
            ))
        }
        fn round(
            &mut self,
            _: Arm,
            p: &mut SimProcess,
            spec: &RoundSpec,
        ) -> Result<RoundResult, DriverError> {
            let per_op = 1_000.0 * p.factor * (1.0 + self.round_jitter * self.rng.unit());
            let wall = (per_op * spec.ops as f64) as u64;
            self.now += wall;
            let mut counts = Counts::default();
            for _ in 0..spec.ops.min(4_000) {
                counts.record(per_op as u64);
            }
            Ok(RoundResult {
                index: spec.index,
                ops: spec.ops,
                wall_ns: wall,
                value: 1e9 / per_op,
                cpu_ns: wall,
                cpu_total_ns: wall,
                instructions: None,
                late_max_ns: 0,
                latencies: counts.sparse(),
                op_spread: 0.1,
            })
        }
        fn stop(&mut self, _: Arm, _: SimProcess) -> Result<(), DriverError> {
            self.now += 1_000_000;
            Ok(())
        }
        fn now_ns(&mut self) -> u64 {
            self.now
        }
        fn load(&mut self) -> Option<f64> {
            Some(self.load)
        }
        fn cpu_ns(&mut self) -> u64 {
            self.now
        }
    }

    fn sim(slow: f64, process_jitter: f64, round_jitter: f64, load: f64) -> Sim {
        Sim {
            now: 0,
            rng: SplitMix64::new(5),
            slow,
            process_jitter,
            round_jitter,
            load,
            started: 0,
        }
    }

    fn request(ab: bool) -> Request {
        let arm = ArmSpec {
            builds: vec![PathBuf::from("reliquary")],
            bench: "spin".to_owned(),
            settings: vec![],
        };
        Request {
            a: arm.clone(),
            b: if ab { Some(arm) } else { None },
            metric: Metric::Throughput,
            offered: Offered::Closed,
            tail: MEDIAN,
            in_flight: SERIAL,
            precision: 0.05,
            seed: 9,
            build_cost_ns: None,
        }
    }

    /// T-RQ0.7.9: variation between process starts and not between rounds → the tool repeats process
    /// starts, not rounds, and the record says so.
    #[test]
    fn process_variation_makes_process_starts_the_repeated_level() {
        let mut s = sim(1.0, 0.3, 1e-4, 0.1);
        let out = measure(&mut s, &request(false)).unwrap();
        let rounds = &out.levels[0];
        assert_eq!(rounds.level, Level::Rounds);
        assert_eq!(rounds.count, Count::Fixed { times: 1 });
        assert!(
            rounds.why.contains("level above is repeated instead"),
            "{}",
            rounds.why
        );
        assert_eq!(
            out.levels[1].count,
            Count::UntilSettled { min: 10, max: 28 }
        );
        // The builds' count is stated too: one, as one was given.
        assert_eq!(out.levels[2].level, Level::Builds);
        assert!(
            out.levels[2].why.contains("one build an arm was given"),
            "{:?}",
            out.levels[2]
        );
        // Each unit is a process start of one round.
        assert!(out.arms[0].units.iter().all(|u| u.rounds == 1));
        assert!(out.arms[0].units.len() >= 10);
    }

    /// AC-RQ0.7.7: variation between rounds and not between process starts → one process, its rounds
    /// repeated.
    #[test]
    fn round_variation_keeps_one_process() {
        let mut s = sim(1.0, 0.0, 0.05, 0.1);
        let out = measure(&mut s, &request(false)).unwrap();
        let top = out
            .levels
            .iter()
            .find(|l| matches!(l.count, Count::UntilSettled { .. }))
            .unwrap();
        assert_eq!(top.level, Level::Rounds, "{:?}", out.levels);
        assert!(out.size.why.contains("§5a"));
    }

    /// T-RQ0.7.6: an A|B of a build against one with a planted slowdown wider than the point's interval
    /// → "A better" (B worse), the intervals separated.
    #[test]
    fn a_planted_slowdown_is_found_through_the_driver() {
        let mut s = sim(1.10, 0.0, 0.02, 0.1);
        let out = measure(&mut s, &request(true)).unwrap();
        let c = out.comparison.unwrap();
        assert_eq!(c.verdict, ab::Verdict::ABetter, "{}", c.why);
        // Throughput, higher better: B's interval lies wholly below A's.
        let (alo, _) = c.a.interval.unwrap();
        let (_, bhi) = c.b.interval.unwrap();
        assert!(bhi < alo);
    }

    /// T-RQ0.7.8: a run on a loaded machine is not of record and names the load.
    #[test]
    fn a_loaded_machine_makes_the_record_not_of_record() {
        let mut s = sim(1.0, 0.0, 0.02, 7.5);
        let out = measure(&mut s, &request(false)).unwrap();
        let Standing::NotOfRecord { why } = &out.standing else {
            panic!("{:?}", out.standing)
        };
        assert!(why[0].contains("7.50 before the first unit"), "{why:?}");
        let mut quiet = sim(1.0, 0.0, 0.02, 0.2);
        assert_eq!(
            measure(&mut quiet, &request(false)).unwrap().standing,
            Standing::OfRecord
        );
    }

    /// AC-RQ0.7.1: each state reports its median, interval, units and tails; a dash where too few.
    #[test]
    fn states_report_medians_intervals_and_tails() {
        let mut s = sim(1.0, 0.0, 0.02, 0.1);
        let out = measure(&mut s, &request(false)).unwrap();
        let st = &out.arms[0].states[0];
        assert!(st.interval.is_some());
        assert_eq!(st.share, None);
        assert_eq!(st.tails.len(), 3);
        assert!(st.tails[0].1.is_some(), "p50 is stated");
    }

    /// (AC-RQ0.7.7)
    /// POWER_SHARE is where an M/M/1 queue's power, throughput over delay, λ(μ − λ), is greatest: half
    /// the service rate (AC-RQ0.7.5).
    #[test]
    fn the_power_share_is_where_an_mm1_queues_power_peaks() {
        let mu = 1.0f64;
        let steps = 100_000;
        let best = (1..steps)
            .map(|i| f64::from(i) / f64::from(steps))
            .max_by(|a, b| (a * (mu - a)).total_cmp(&(b * (mu - b))))
            .unwrap();
        assert!((best - POWER_SHARE * mu).abs() < 1e-4, "{best}");
    }

    /// DRAIN is the share of the operations in flight that a round's end leaves short: as the last N
    /// finish one by one the depth falls N, N − 1, …, 1, short of N by 0, 1, …, N − 1, which averages
    /// (N − 1)/2, at most N·DRAIN and approaching it (AC-RQ0.7.7).
    #[test]
    fn the_drain_share_is_a_falling_depths_average_shortfall() {
        for n in [1u64, 2, 8, 64, 4_096, 1_000_000] {
            let shortfall = (0..n).map(|k| k as f64).sum::<f64>() / n as f64;
            assert!(shortfall <= DRAIN * n as f64, "{n}");
            assert!((shortfall - DRAIN * n as f64).abs() <= DRAIN, "{n}");
        }
    }

    /// Two arms whose plans put the top at different levels → the higher top, the larger fixed count
    /// below it, and nothing repeated above, whichever arm is A (AC-RQ0.7.7).
    #[test]
    fn combined_plans_take_the_higher_top() {
        let plan = |rounds: Count, processes: Count| {
            vec![
                LevelPlan {
                    level: Level::Rounds,
                    count: rounds,
                    own_variance: None,
                    cost_ns: None,
                    why: String::new(),
                },
                LevelPlan {
                    level: Level::ProcessStarts,
                    count: processes,
                    own_variance: None,
                    cost_ns: None,
                    why: String::new(),
                },
            ]
        };
        let settled = Count::UntilSettled { min: 10, max: 28 };
        let rounds_top = plan(settled.clone(), Count::Fixed { times: 1 });
        let processes_top = plan(Count::Fixed { times: 3 }, settled.clone());
        for plans in [
            vec![rounds_top.clone(), processes_top.clone()],
            vec![processes_top.clone(), rounds_top.clone()],
        ] {
            let c = combine(plans);
            assert_eq!(c[0].count, Count::Fixed { times: 3 }, "{c:?}");
            assert_eq!(c[1].count, settled, "{c:?}");
        }
        let both = combine(vec![
            plan(Count::Fixed { times: 2 }, settled.clone()),
            plan(Count::Fixed { times: 5 }, settled.clone()),
        ]);
        assert_eq!(both[0].count, Count::Fixed { times: 5 });
    }

    /// A round's size is its formula's ceiling, at least one (AC-RQ0.7.7).
    #[test]
    fn ceilings_are_exact() {
        // AC-RQ0.7.7: the round's size is the formula's ceiling.
        assert_eq!(ceil_count(0.3), 1);
        assert_eq!(ceil_count(10.0), 10);
        assert_eq!(ceil_count(10.2), 11);
        assert_eq!(ceil_count(1_536.9), 1_537);
        assert_eq!(ceil_count(f64::INFINITY), 1);
    }
}
