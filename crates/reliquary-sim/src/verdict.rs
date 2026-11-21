//! What a run is judged by (docs/design/sim.md, W7, W9): its history against the reference model,
//! for linearizability (plan RQ0.5's checker); its liveness once the faults stopped (AC-RQ0.6.5);
//! starvation (T-RQ0.6.4); what was still waiting at its end (T-RQ0.6.5); grants honored again
//! after they were refused as expired, and deadlines set before their setter's now (T-RQ0.6.6).
//!
//! The history holds every attempt a client made, on the world's time line. An attempt answered by
//! the registry is an operation with its answer. One with no answer, or refused as `Unknown`, may
//! or may not have applied: an operation whose answer was lost. One refused `Busy` or
//! `Unavailable` applied nothing, and is no operation at all: the refusal is no answer of the
//! model's, and allowed in any state (docs/design/model.md, H7).

use std::fmt;

use reliquary_api::check::{self, Budget};
use reliquary_api::history::{History, Operation, Outcome as HistoryOutcome};
use reliquary_api::model::{Config, Model};

use crate::client::{Attempt, Ended};
use crate::ledger::{Breach, NegativeWait};
use crate::wire::RefusedKind;
use crate::world::{End, Outcome, Waiter};

/// Why a run is not live.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Stuck {
    /// An attempt invoked once the faults had stopped that neither completed nor was refused
    /// (plan AC-RQ0.6.5).
    Lost {
        client: u16,
        op: u64,
        attempt: u32,
        invoked: u64,
    },
    /// An operation that used every attempt, the last once the faults had stopped, and never
    /// completed (plan T-RQ0.6.4).
    Starved { client: u16, op: u64, attempts: u32 },
    /// Something still waiting when the run ended (plan T-RQ0.6.5).
    Waiting(Waiter),
    /// The run ended idle or out of steps with its clients unfinished, or never reached its calm.
    Unfinished(End),
}

/// A run's verdict.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Verdict {
    pub linearizable: check::Verdict,
    pub stuck: Vec<Stuck>,
    pub breaches: Vec<Breach>,
    pub waits: Vec<NegativeWait>,
}

impl Verdict {
    /// Whether the run passed every check.
    #[must_use]
    pub fn passed(&self) -> bool {
        matches!(self.linearizable, check::Verdict::Linearizable { .. })
            && self.stuck.is_empty()
            && self.breaches.is_empty()
            && self.waits.is_empty()
    }
}

/// The history a run's attempts make.
#[must_use]
pub fn history(attempts: &[Attempt]) -> History {
    let mut operations: Vec<Operation> = attempts
        .iter()
        .filter_map(|a| {
            let outcome = match &a.ended {
                Ended::Answered { at, answer } => HistoryOutcome::Answered {
                    at: *at,
                    answer: answer.clone(),
                },
                Ended::Lost { .. } => HistoryOutcome::Lost,
                Ended::Refused { refused, .. } => match refused.kind {
                    RefusedKind::Unknown => HistoryOutcome::Lost,
                    RefusedKind::Busy | RefusedKind::Unavailable => return None,
                },
            };
            Some(Operation {
                client: a.client,
                request: a.request.clone(),
                invoked: a.invoked,
                outcome,
            })
        })
        .collect();
    operations.sort_by_key(|op| op.invoked);
    History { operations }
}

/// Judges `outcome` against a model of `config`, its linearizability searched within `budget`.
#[must_use]
pub fn judge(outcome: &Outcome, config: &Config, budget: Budget) -> Verdict {
    let linearizable = check::check(
        &Model::new(config.clone()),
        &history(&outcome.attempts),
        budget,
    );
    let mut stuck = Vec::new();
    if outcome.end != End::Finished || outcome.quiet_from.is_none() {
        stuck.push(Stuck::Unfinished(outcome.end));
    }
    let quiet = outcome.quiet_from.unwrap_or(u64::MAX);
    for a in &outcome.attempts {
        if a.invoked >= quiet && matches!(a.ended, Ended::Lost { .. }) {
            stuck.push(Stuck::Lost {
                client: a.client.0,
                op: a.op,
                attempt: a.attempt,
                invoked: a.invoked,
            });
        }
    }
    // An operation's attempts are consecutive in its client's record: the last tells how it ended.
    let mut i = 0;
    while let Some(first) = outcome.attempts.get(i) {
        let mut last = first;
        let mut j = i;
        while let Some(next) = outcome.attempts.get(j) {
            if next.client != first.client || next.op != first.op {
                break;
            }
            last = next;
            j = j.saturating_add(1);
        }
        let completed = outcome
            .attempts
            .get(i..j)
            .unwrap_or(&[])
            .iter()
            .any(|a| matches!(a.ended, Ended::Answered { .. }));
        if !completed && last.invoked >= quiet {
            stuck.push(Stuck::Starved {
                client: first.client.0,
                op: first.op,
                attempts: last.attempt,
            });
        }
        i = j.max(i.saturating_add(1));
    }
    stuck.extend(outcome.waiters.iter().cloned().map(Stuck::Waiting));
    Verdict {
        linearizable,
        stuck,
        breaches: outcome.breaches.clone(),
        waits: outcome.waits.clone(),
    }
}

impl fmt::Display for Verdict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "linearizability: {:?}", self.linearizable)?;
        for s in &self.stuck {
            writeln!(f, "stuck: {s:?}")?;
        }
        for b in &self.breaches {
            writeln!(f, "lease honored twice: {b:?}")?;
        }
        for w in &self.waits {
            writeln!(f, "negative wait: {w:?}")?;
        }
        Ok(())
    }
}
