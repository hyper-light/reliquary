//! The differential harness (plan RQ0.5; docs/design/model.md, H11 to H14). A [`Registry`] is
//! anything that answers requests: the model, an implementation, or an implementation with a
//! planted fault. A sequential history is compared answer by answer with the model's; a concurrent
//! one is checked for linearizability against it. A hunt runs histories from one seed until one
//! fails, then shrinks the failure to a 1-minimal history and reports the seed it replays from. The
//! harness drives anything that implements [`Registry`], so the same hunts later drive one node,
//! three nodes and three regions (CLAUDE.md §8).

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use hyper_sim::rng::stream_seed;

use crate::answer::{ANSWERS, Answer, ErrorCode, Status};
use crate::check::{self, Budget, Verdict};
use crate::generate::{self, Mix};
use crate::history::{Client, History, Outcome, Plan, PlanError};
use crate::model::{Config, Model};
use crate::request::{Endpoint, OpKind, Request};
use crate::shrink::shrink;

/// What answers requests.
pub trait Registry {
    /// The answer to `request`, sent by `client`.
    fn call(&mut self, client: Client, request: &Request) -> Answer;
}

/// The model answers as it decides, and makes its change.
impl Registry for Model {
    fn call(&mut self, _client: Client, request: &Request) -> Answer {
        self.step(request)
    }
}

/// The seed of history `index` of a hunt seeded `seed`: a stream of its own, so a history replays
/// from its seed alone.
pub fn history_seed(seed: u64, index: u64) -> u64 {
    stream_seed(seed, "reliquary.history", &[index])
}

fn bump<K: Ord>(counts: &mut BTreeMap<K, u64>, key: K) {
    let count = counts.entry(key).or_insert(0);
    // A count of requests cannot pass u64; saturation is the stated meaning.
    *count = count.saturating_add(1);
}

/// What a run produced (plan AC-RQ0.5.2, T-RQ0.5.5): requests by kind, answers by endpoint,
/// status and code, codes, and concurrent histories by how many clients interleaved in them.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Coverage {
    pub kinds: BTreeMap<OpKind, u64>,
    pub answers: BTreeMap<(Endpoint, Status, Option<ErrorCode>), u64>,
    pub codes: BTreeMap<ErrorCode, u64>,
    pub interleaved: BTreeMap<usize, u64>,
    pub lost: u64,
    pub histories: u64,
}

/// Something a run did not produce, or produced and should not have.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Gap {
    /// No request of this kind.
    Kind(OpKind),
    /// No answer with this code.
    Code(ErrorCode),
    /// A row of [`ANSWERS`] never given.
    Answer {
        endpoint: Endpoint,
        status: Status,
        code: Option<ErrorCode>,
    },
    /// No history in which this many clients interleaved.
    Interleaved(usize),
    /// An answer [`ANSWERS`] does not list.
    Unlisted {
        endpoint: Endpoint,
        status: Status,
        code: Option<ErrorCode>,
    },
}

impl Coverage {
    /// Counts `answer` to `request`.
    pub fn count(&mut self, request: &Request, answer: &Answer) {
        bump(&mut self.kinds, request.op.kind());
        bump(
            &mut self.answers,
            (request.op.endpoint(), answer.status, answer.error),
        );
        if let Some(code) = answer.error {
            bump(&mut self.codes, code);
        }
    }

    /// Counts every answered operation of `history`, its lost answers, and how many of its
    /// clients interleaved.
    pub fn count_history(&mut self, history: &History) {
        for op in &history.operations {
            if let Outcome::Answered { answer, .. } = &op.outcome {
                self.count(&op.request, answer);
            }
        }
        let lost = u64::try_from(history.lost()).unwrap_or(u64::MAX);
        self.lost = self.lost.saturating_add(lost);
        let interleaved = history.interleaved_clients();
        if interleaved > 0 {
            bump(&mut self.interleaved, interleaved);
        }
        self.histories = self.histories.saturating_add(1);
    }

    /// Every kind, code and row of [`ANSWERS`] at zero, every client count in `interleaved` with
    /// no history that interleaved that many, and every answer given that [`ANSWERS`] does not
    /// list. Empty when the run
    /// covered everything (plan T-RQ0.5.5).
    pub fn gaps(&self, interleaved: &[usize]) -> Vec<Gap> {
        let mut gaps: Vec<Gap> = OpKind::ALL
            .iter()
            .filter(|kind| !self.kinds.contains_key(kind))
            .map(|kind| Gap::Kind(*kind))
            .collect();
        gaps.extend(
            ErrorCode::ALL
                .iter()
                .filter(|code| !self.codes.contains_key(code))
                .map(|code| Gap::Code(*code)),
        );
        gaps.extend(
            ANSWERS
                .iter()
                .filter(|row| !self.answers.contains_key(row))
                .map(|(endpoint, status, code)| Gap::Answer {
                    endpoint: *endpoint,
                    status: *status,
                    code: *code,
                }),
        );
        gaps.extend(
            interleaved
                .iter()
                .filter(|k| !self.interleaved.contains_key(k))
                .map(|k| Gap::Interleaved(*k)),
        );
        gaps.extend(
            self.answers
                .keys()
                .filter(|key| !ANSWERS.contains(key))
                .map(|(endpoint, status, code)| Gap::Unlisted {
                    endpoint: *endpoint,
                    status: *status,
                    code: *code,
                }),
        );
        gaps
    }
}

/// The first request a registry answered otherwise than the model.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Divergence {
    pub index: usize,
    pub request: Request,
    pub expected: Answer,
    pub observed: Answer,
}

/// What makes two divergences one failure: the kind of request, and the status and code the model
/// and the registry each gave.
pub type Signature = (
    OpKind,
    (Status, Option<ErrorCode>),
    (Status, Option<ErrorCode>),
);

impl Divergence {
    /// Its signature. Shrinking keeps a smaller history only when it fails with the same one: "the
    /// failure it was intended to capture", not another (Zeller and Hildebrandt; research/10 F13).
    pub fn signature(&self) -> Signature {
        (
            self.request.op.kind(),
            (self.expected.status, self.expected.error),
            (self.observed.status, self.observed.error),
        )
    }
}

/// Sends `requests` in order to `registry` and to a model of `config`, counting the model's
/// answers, and stops at the first that differs.
pub fn compare<R: Registry + ?Sized>(
    config: &Config,
    registry: &mut R,
    requests: &[Request],
    coverage: &mut Coverage,
) -> Result<(), Box<Divergence>> {
    let mut model = Model::new(config.clone());
    for (index, request) in requests.iter().enumerate() {
        let expected = model.step(request);
        let observed = registry.call(Client(0), request);
        coverage.count(request, &expected);
        if expected != observed {
            return Err(Box::new(Divergence {
                index,
                request: request.clone(),
                expected,
                observed,
            }));
        }
    }
    Ok(())
}

/// A hunt over sequential histories: `histories` of them, each `length` requests long, from
/// `seed` under `mix`, a failure shrunk with at most `shrink_tries` runs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hunt {
    pub seed: u64,
    pub mix: Mix,
    pub histories: u64,
    pub length: usize,
    pub shrink_tries: u64,
}

/// A sequential history that diverged, and what shrinking left of it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Failure {
    /// The history's seed: [`replay`] with it gives the same divergence at the same request.
    pub seed: u64,
    pub divergence: Box<Divergence>,
    /// The shortest history found that still diverges.
    pub shrunk: Vec<Request>,
    /// Where the shrunk history diverges.
    pub shrunk_divergence: Option<Box<Divergence>>,
    /// Whether removing any one request of `shrunk` makes the divergence go.
    pub minimal: bool,
}

/// Runs the hunt's histories against registries `make` builds for each history's configuration.
pub fn hunt<R, F>(hunt: &Hunt, mut make: F) -> Result<Coverage, Box<Failure>>
where
    R: Registry,
    F: FnMut(&Config) -> R,
{
    let mut coverage = Coverage::default();
    for index in 0..hunt.histories {
        let seed = history_seed(hunt.seed, index);
        let (config, requests) = generate::sequential(seed, &hunt.mix, hunt.length);
        let mut registry = make(&config);
        coverage.histories = coverage.histories.saturating_add(1);
        if let Err(divergence) = compare(&config, &mut registry, &requests, &mut coverage) {
            let signature = divergence.signature();
            let diverges = |candidate: &[Request]| {
                let mut registry = make(&config);
                compare(&config, &mut registry, candidate, &mut Coverage::default())
                    .err()
                    .is_some_and(|d| d.signature() == signature)
            };
            let shrunk = shrink(&requests, diverges, hunt.shrink_tries);
            let shrunk_divergence = compare(
                &config,
                &mut make(&config),
                &shrunk.items,
                &mut Coverage::default(),
            )
            .err();
            return Err(Box::new(Failure {
                seed,
                divergence,
                shrunk: shrunk.items,
                shrunk_divergence,
                minimal: shrunk.minimal,
            }));
        }
    }
    Ok(coverage)
}

/// The history of `seed` under `mix`, `length` requests long, run again: the same divergence a
/// hunt reported for that seed, or none.
pub fn replay<R, F>(seed: u64, mix: &Mix, length: usize, mut make: F) -> Result<(), Box<Divergence>>
where
    R: Registry,
    F: FnMut(&Config) -> R,
{
    let (config, requests) = generate::sequential(seed, mix, length);
    compare(
        &config,
        &mut make(&config),
        &requests,
        &mut Coverage::default(),
    )
}

/// A hunt over concurrent histories: `histories` plans, from `seed`, each checked with at most
/// `budget` configurations explored, a failure shrunk with at most `shrink_tries` runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Interleaving {
    pub seed: u64,
    pub histories: u64,
    pub budget: Budget,
    pub shrink_tries: u64,
}

/// A concurrent history no order explains, and what shrinking left of its plan.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tangle {
    /// The history's seed and index: the plan source given them makes the same plan.
    pub seed: u64,
    pub index: u64,
    pub history: History,
    /// The first operation (by completion) no order explains, as an index into `history`.
    pub operation: usize,
    /// The smallest plan found whose history is still not linearizable, and that history.
    pub shrunk: Plan,
    pub shrunk_history: Option<History>,
    pub minimal: bool,
}

/// How a concurrent hunt ended other than clean.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Unexplained {
    /// A history no order explains.
    Tangle(Box<Tangle>),
    /// A plan that could not run.
    Plan { seed: u64, error: PlanError },
    /// A check that ran out of budget.
    Exhausted { seed: u64, explored: u64 },
}

/// Runs concurrent histories: for each index, `plans` makes a plan from the history's seed and the
/// index, `make` a registry for its configuration; the history the plan gives is checked for
/// linearizability against a model of the configuration.
pub fn hunt_concurrent<R, P, F>(
    hunt: &Interleaving,
    mut plans: P,
    mut make: F,
) -> Result<Coverage, Unexplained>
where
    R: Registry,
    P: FnMut(u64, u64) -> Result<(Config, Plan), PlanError>,
    F: FnMut(&Config) -> R,
{
    let mut coverage = Coverage::default();
    for index in 0..hunt.histories {
        let seed = history_seed(hunt.seed, index);
        let (config, plan) =
            plans(seed, index).map_err(|error| Unexplained::Plan { seed, error })?;
        let history = plan
            .run(&mut make(&config))
            .map_err(|error| Unexplained::Plan { seed, error })?;
        coverage.count_history(&history);
        let initial = Model::new(config.clone());
        match check::check(&initial, &history, hunt.budget) {
            Verdict::Linearizable { .. } => {}
            Verdict::Exhausted { explored } => {
                return Err(Unexplained::Exhausted { seed, explored });
            }
            Verdict::NotLinearizable { operation } => {
                // The same failure: no order explains an operation of the same kind.
                let kind = history
                    .operations
                    .get(operation)
                    .map(|op| op.request.op.kind());
                let tangled = |kept: &[usize]| {
                    let keep: BTreeSet<usize> = kept.iter().copied().collect();
                    plan.keeping(&keep)
                        .run(&mut make(&config))
                        .is_ok_and(
                            |history| match check::check(&initial, &history, hunt.budget) {
                                Verdict::NotLinearizable { operation } => {
                                    history
                                        .operations
                                        .get(operation)
                                        .map(|op| op.request.op.kind())
                                        == kind
                                }
                                _ => false,
                            },
                        )
                };
                let all: Vec<usize> = (0..plan.operations().len()).collect();
                let shrunk = shrink(&all, tangled, hunt.shrink_tries);
                let keep: BTreeSet<usize> = shrunk.items.iter().copied().collect();
                let smaller = plan.keeping(&keep);
                let shrunk_history = smaller.run(&mut make(&config)).ok();
                return Err(Unexplained::Tangle(Box::new(Tangle {
                    seed,
                    index,
                    history,
                    operation,
                    shrunk: smaller,
                    shrunk_history,
                    minimal: shrunk.minimal,
                })));
            }
        }
    }
    Ok(coverage)
}

/// The plan `plans` makes for `seed` and `index`, run again and checked: the same verdict a hunt
/// reported for them.
pub fn replay_concurrent<R, P, F>(
    seed: u64,
    index: u64,
    budget: Budget,
    mut plans: P,
    mut make: F,
) -> Result<(History, Verdict), PlanError>
where
    R: Registry,
    P: FnMut(u64, u64) -> Result<(Config, Plan), PlanError>,
    F: FnMut(&Config) -> R,
{
    let (config, plan) = plans(seed, index)?;
    let history = plan.run(&mut make(&config))?;
    let verdict = check::check(&Model::new(config), &history, budget);
    Ok((history, verdict))
}

impl fmt::Display for Divergence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "request {} diverged: {}", self.index, self.request)?;
        writeln!(f, "  the model answered: {}", self.expected)?;
        write!(f, "  the registry answered: {}", self.observed)
    }
}

/// A failure as a report a person can replay: the seed, the divergence, and the shrunk history.
impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "history seed {:#018x}: {}", self.seed, self.divergence)?;
        writeln!(
            f,
            "shrunk to {} requests{}:",
            self.shrunk.len(),
            if self.minimal { " (1-minimal)" } else { "" }
        )?;
        for (i, request) in self.shrunk.iter().enumerate() {
            writeln!(f, "  {i}: {request}")?;
        }
        match &self.shrunk_divergence {
            Some(divergence) => writeln!(f, "where {divergence}"),
            None => writeln!(f, "which no longer diverges when run alone"),
        }
    }
}

/// A tangle as a report: the seed and index, the operation no order explains, and the shrunk
/// history with each operation's interval.
impl fmt::Display for Tangle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "history seed {:#018x} (index {}): no order explains operation {}",
            self.seed, self.index, self.operation
        )?;
        if let Some(op) = self.history.operations.get(self.operation) {
            writeln!(f, "  {}", op.request)?;
        }
        writeln!(
            f,
            "shrunk to {} operations{}:",
            self.shrunk.operations().len(),
            if self.minimal { " (1-minimal)" } else { "" }
        )?;
        if let Some(history) = &self.shrunk_history {
            for op in &history.operations {
                match &op.outcome {
                    Outcome::Answered { at, answer } => writeln!(
                        f,
                        "  client {} [{}..{}] {} -> {}",
                        op.client.0, op.invoked, at, op.request, answer
                    )?,
                    Outcome::Lost => writeln!(
                        f,
                        "  client {} [{}..] {} -> lost",
                        op.client.0, op.invoked, op.request
                    )?,
                }
            }
        }
        Ok(())
    }
}
