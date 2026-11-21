//! What the world tallies as a run goes (docs/design/sim.md, W6, W7, W9): every write a store
//! applies, in order, which a crash sweep names its cuts by (plan AC-RQ0.6.3); the faults the
//! nemesis injected, by kind (T-RQ0.6.7); every grant a bytes store honored or refused, so a grant
//! honored again after it was refused as expired is found (T-RQ0.6.6); every deadline a process set
//! before its own now; and what the world let go, each by why. Everything here is bounded by the
//! run's step budget: each entry is one step's.

use std::collections::BTreeMap;

use crate::call::Seam;

/// Where a power cut falls relative to a write.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Side {
    /// The write never reaches the store's medium.
    Before,
    /// The write applied, and its answer is never sent.
    After,
    /// A piece of a put cut at a byte inside it: what was durable stays and nothing after it
    /// (docs/design/seams.md BYTES-4); any other write applies and its answer is never sent.
    Under,
}

/// A power cut at one write of a run: the store's machine loses power at it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WriteCut {
    /// The write's number in the run, from zero, in the order the stores applied them.
    pub write: u64,
    pub side: Side,
}

/// A write a store applied.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Write {
    pub index: u64,
    pub at: u64,
    pub seam: Seam,
    pub method: &'static str,
    pub machine: usize,
}

/// A grant honored at a bytes store after the store had refused it as expired: a lease honored
/// twice (plan T-RQ0.6.6).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Breach {
    pub key: String,
    pub refused_at: u64,
    pub honored_at: u64,
}

/// A deadline a process set before its own clock's now: a wait that would be negative.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NegativeWait {
    pub process: String,
    pub at: u64,
    pub short_by_ns: u64,
}

/// What a bytes store did with the grants presented to it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Grants {
    /// Presentations honored.
    pub honored: u64,
    /// Presentations refused as expired.
    pub expired: u64,
    /// Presentations of a grant already refused as expired: the ones a store could honor twice.
    pub again: u64,
}

/// A grant's history at its store.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct GrantLife {
    expired_at: Option<u64>,
}

/// What a run tallied.
#[derive(Debug, Default)]
pub struct Ledger {
    pub(crate) writes: Vec<Write>,
    pub(crate) cut: Option<WriteCut>,
    /// The nemesis's power cut armed for a machine's next write, by machine.
    pub(crate) armed_cuts: BTreeMap<usize, Side>,
    /// Answers to lose, by machine: the next answer a store on it sends is lost after its effect.
    pub(crate) lose_answers: BTreeMap<usize, u32>,
    pub(crate) faults: BTreeMap<&'static str, u64>,
    /// When the last fault took effect.
    pub(crate) last_fault: u64,
    lives: BTreeMap<Vec<u8>, GrantLife>,
    pub(crate) breaches: Vec<Breach>,
    pub(crate) waits: Vec<NegativeWait>,
    pub(crate) let_go: BTreeMap<&'static str, u64>,
    pub(crate) grants: Grants,
    /// Every answer a store gave, by what it was: a refusal by its kind, else its seam.
    pub(crate) answers: BTreeMap<String, u64>,
}

fn bump<K: Ord>(counts: &mut BTreeMap<K, u64>, key: K) {
    let n = counts.entry(key).or_insert(0);
    // A count of steps cannot pass u64; saturation is the stated meaning.
    *n = n.saturating_add(1);
}

impl Ledger {
    /// A ledger for a run that cuts power at `cut`, if any.
    #[must_use]
    pub fn new(cut: Option<WriteCut>) -> Self {
        Self {
            cut,
            ..Self::default()
        }
    }

    /// The next write's number, recorded as applied by a store on `machine`.
    pub(crate) fn write(
        &mut self,
        at: u64,
        seam: Seam,
        method: &'static str,
        machine: usize,
    ) -> u64 {
        let index = u64::try_from(self.writes.len()).unwrap_or(u64::MAX);
        self.writes.push(Write {
            index,
            at,
            seam,
            method,
            machine,
        });
        index
    }

    /// The number the next write will have.
    pub(crate) fn next_write(&self) -> u64 {
        u64::try_from(self.writes.len()).unwrap_or(u64::MAX)
    }

    /// Counts one fault of `kind`, which took effect at `at`.
    pub(crate) fn fault(&mut self, kind: &'static str, at: u64) {
        bump(&mut self.faults, kind);
        self.last_fault = self.last_fault.max(at);
    }

    /// Counts one thing let go, by why.
    pub(crate) fn let_go(&mut self, why: &'static str) {
        bump(&mut self.let_go, why);
    }

    /// A store on `machine` honored (or refused, `expired` saying whether as expired) the grant
    /// whose tag is `tag`, for `key`, at `at`.
    pub(crate) fn grant(&mut self, tag: &[u8], key: &str, at: u64, honored: bool, expired: bool) {
        let life = self.lives.entry(tag.to_vec()).or_default();
        if life.expired_at.is_some() {
            self.grants.again = self.grants.again.saturating_add(1);
        }
        if honored {
            self.grants.honored = self.grants.honored.saturating_add(1);
            if let Some(refused_at) = life.expired_at {
                self.breaches.push(Breach {
                    key: key.to_owned(),
                    refused_at,
                    honored_at: at,
                });
            }
        } else if expired {
            self.grants.expired = self.grants.expired.saturating_add(1);
            life.expired_at.get_or_insert(at);
        }
    }

    /// Counts a store's answer: a refusal by its kind (`refused Conflict`), else what it was.
    pub(crate) fn answered(&mut self, answer: &crate::call::Answered) {
        let summary = answer.summary();
        let key = if summary.starts_with("refused") {
            summary
        } else {
            match answer {
                crate::call::Answered::Records(_) => "records answered".to_owned(),
                crate::call::Answered::Bytes(_) => "bytes answered".to_owned(),
            }
        };
        bump(&mut self.answers, key);
    }

    /// `process` set a deadline `short_by_ns` before its now.
    pub(crate) fn negative_wait(&mut self, process: &str, at: u64, short_by_ns: u64) {
        self.waits.push(NegativeWait {
            process: process.to_owned(),
            at,
            short_by_ns,
        });
    }
}
