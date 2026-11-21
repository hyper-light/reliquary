//! Records: Amphora's `Records` (`amphora-api`), the trait Reliquary keeps its records through
//! (architecture D2): rows by family and key, each at a revision; conditional transactions; one
//! change feed, kept back to a horizon. Amphora gives the trait and its bounds but no simulator,
//! so [`SimRecords`] is Reliquary's, and [`cases`] are the clauses of the trait Reliquary relies on,
//! which Amphora's store passes when Reliquary links it (plan RQ1.8).
//!
//! Amphora's methods block until their deadline. A simulation runs nothing else while a call
//! blocks, so [`SimRecords`] answers at once from what it holds and moves its simulated clock by
//! the time a call takes: a delayed answer moves it by the delay, and a call that never answers, or
//! a wait for a change that never comes, moves it to the caller's deadline and is refused
//! [`Refusal::Timeout`] there.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::ops::Bound;
use std::time::{Duration, Instant};

use amphora_api::{
    self as api, Change, Changes, Cond, Consistency, Page, Records, Refusal, Rev, Row, Scan, Tx,
    Write,
};
use parking_lot::Mutex;

use crate::seams::BoxFuture;
use crate::seams::clock::{Clock, SimClock};
use crate::seams::conformance::{Case, Failure, Fixture, at_once, ensure, fine, refused, same};
use crate::seams::fault::{CUT, Counts, DELAY, Fault, Faults, Kind, LOSE, Unarmed, VANISH};

impl Kind for Refusal {
    fn kind(&self) -> &'static str {
        match self {
            Self::Conflict { .. } => "Conflict",
            Self::TooLarge { .. } => "TooLarge",
            Self::Invalid { .. } => "Invalid",
            Self::Full { .. } => "Full",
            Self::Compacted { .. } => "Compacted",
            Self::Ahead { .. } => "Ahead",
            Self::Timeout => "Timeout",
            Self::Unavailable(_) => "Unavailable",
            Self::UnknownOutcome => "UnknownOutcome",
            Self::NotLeader { .. } => "NotLeader",
            Self::Fenced(_) => "Fenced",
        }
    }
}

/// The faults the records simulator injects (`docs/design/seams.md`, "Records"): each refusal
/// Amphora's trait names, an answer delayed past its caller's deadline, a request that never
/// arrives, a commit's answer lost after it applied, and a page cut at any row.
pub const FAULTS: &[&str] = &[
    "Conflict",
    "TooLarge",
    "Invalid",
    "Full",
    "Compacted",
    "Ahead",
    "Timeout",
    "Unavailable",
    "UnknownOutcome",
    "NotLeader",
    "Fenced",
    DELAY,
    VANISH,
    LOSE,
    CUT,
];

/// The operations a fault can be armed for: each method of `Records`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Op {
    Get,
    Scan,
    Commit,
    Changes,
    Wait,
    Head,
    Compact,
}

fn fits(op: Op, fault: &Fault<Refusal>) -> bool {
    match fault {
        Fault::Cut(_) => matches!(op, Op::Scan | Op::Changes),
        Fault::Lose => matches!(op, Op::Commit | Op::Compact),
        Fault::Refuse(_) | Fault::Delay(_) | Fault::Vanish => true,
    }
}

/// What a records simulator holds at most, as its maker states.
#[derive(Debug, Clone, Copy)]
pub struct SimLimits {
    /// Rows.
    pub rows: usize,
    /// Changes its feed keeps: past them, the oldest go and the horizon rises past them.
    pub changes: usize,
}

/// One change, as the feed keeps it: what the row was before it, for reads at a revision before.
struct Logged {
    rev: Rev,
    family: String,
    key: Vec<u8>,
    value: Option<Vec<u8>>,
    prior: Option<(Vec<u8>, Rev)>,
}

struct Kept {
    rows: BTreeMap<(String, Vec<u8>), (Vec<u8>, Rev)>,
    /// Changes after the horizon, oldest first.
    log: VecDeque<Logged>,
    head: Rev,
    /// The newest revision whose changes are gone: reads at or after it are served.
    horizon: Rev,
    faults: Faults<Op, Refusal>,
}

/// Rows in memory. Its clock is the one its calls' deadlines are on.
pub struct SimRecords {
    clock: SimClock,
    limits: SimLimits,
    kept: Mutex<Kept>,
}

fn unavailable(why: impl std::fmt::Display) -> Refusal {
    Refusal::Unavailable(why.to_string())
}

impl Kept {
    /// What the row was at `rev`: the change after `rev` that came first says what it was before,
    /// and with none, the row is as it is now.
    fn at(&self, family: &str, key: &[u8], rev: Rev) -> Option<(Vec<u8>, Rev)> {
        match self
            .log
            .iter()
            .find(|c| c.rev > rev && c.family == family && c.key == key)
        {
            Some(change) => change.prior.clone(),
            None => self.rows.get(&(family.to_owned(), key.to_vec())).cloned(),
        }
    }

    /// The revision a read at `at` is served at, if it can be.
    fn served(&self, at: Consistency) -> Result<Rev, Refusal> {
        match at {
            Consistency::Linearizable => Ok(self.head),
            Consistency::Snapshot(asked) if asked > self.head => Err(Refusal::Ahead {
                asked,
                head: self.head,
            }),
            Consistency::Snapshot(asked) if asked < self.horizon => Err(Refusal::Compacted {
                asked,
                horizon: self.horizon,
            }),
            Consistency::Snapshot(asked) => Ok(asked),
        }
    }

    fn get(&self, family: &str, key: &[u8], at: Consistency) -> Result<Option<Row>, Refusal> {
        api::family(family)?;
        api::key(key)?;
        let rev = self.served(at)?;
        Ok(self.at(family, key, rev).map(|(value, rev)| Row {
            key: key.to_vec(),
            value,
            rev,
        }))
    }

    fn scan(&mut self, family: &str, scan: &Scan) -> Result<Page, Refusal> {
        api::family(family)?;
        let limit = api::limit(scan.limit)?;
        let rev = self.served(scan.at)?;
        let under = |key: &[u8]| {
            key.starts_with(&scan.prefix)
                && scan
                    .after
                    .as_ref()
                    .is_none_or(|after| key > after.as_slice())
        };
        // The family's keys under the prefix as they are now, from past `after`, read only as far
        // as the page needs.
        let from = match &scan.after {
            Some(after) if after.as_slice() >= scan.prefix.as_slice() => {
                Bound::Excluded((family.to_owned(), after.clone()))
            }
            _ => Bound::Included((family.to_owned(), scan.prefix.clone())),
        };
        let mut now = self
            .rows
            .range((from, Bound::Unbounded))
            .take_while(|((f, key), _)| f == family && key.starts_with(&scan.prefix))
            .map(|((_, key), row)| (key.as_slice(), row))
            .peekable();
        // And, for a read at a revision before head, the keys a change since then touched: at
        // most the changes the feed keeps, whatever the family holds.
        let at_head = rev == self.head;
        let changed: BTreeSet<&[u8]> = if at_head {
            BTreeSet::new()
        } else {
            self.log
                .iter()
                .filter(|c| c.rev > rev && c.family == family && under(&c.key))
                .map(|c| c.key.as_slice())
                .collect()
        };
        let mut changed = changed.into_iter().peekable();
        let mut rows = Vec::new();
        let mut more = false;
        // The two in key order, each key once, until a page is full and one more row is found: a
        // row as it is now comes with the key, and a row at an earlier revision is looked up.
        loop {
            let next = match (now.peek(), changed.peek()) {
                (None, None) => None,
                (Some((a, _)), Some(b)) if a == b => {
                    changed.next();
                    now.next().map(|(key, row)| (key, Some(row)))
                }
                (Some((a, _)), Some(b)) if a < b => now.next().map(|(key, row)| (key, Some(row))),
                (Some(_) | None, Some(_)) => changed.next().map(|key| (key, None)),
                (Some(_), None) => now.next().map(|(key, row)| (key, Some(row))),
            };
            let Some((key, current)) = next else {
                break;
            };
            if rows.len() >= limit {
                // The page is full: whether another row follows is all that is asked.
                let follows = match current {
                    Some(_) if at_head => true,
                    _ => self.at(family, key, rev).is_some(),
                };
                if follows {
                    more = true;
                    break;
                }
                continue;
            }
            let found = match current {
                Some(row) if at_head => Some(row.clone()),
                _ => self.at(family, key, rev),
            };
            let Some((value, rev)) = found else {
                continue;
            };
            rows.push(Row {
                key: key.to_vec(),
                value,
                rev,
            });
        }
        let length = u64::try_from(rows.len()).unwrap_or(u64::MAX);
        if let Some(cut) = self.faults.cut(Op::Scan, 0, length) {
            rows.truncate(usize::try_from(cut).unwrap_or(usize::MAX));
            more = true;
        }
        Ok(Page { rows, more })
    }

    fn commit(&mut self, tx: Tx, limits: SimLimits) -> Result<Rev, Refusal> {
        api::transaction(&tx)?;
        for cond in &tx.conds {
            let (family, key, wanted) = match cond {
                Cond::Absent { family, key } => (family, key, None),
                Cond::At { family, key, rev } => (family, key, Some(*rev)),
            };
            let at = self
                .rows
                .get(&(family.clone(), key.clone()))
                .map(|(_, rev)| *rev);
            if at != wanted {
                return Err(Refusal::Conflict {
                    family: family.clone(),
                    key: key.clone(),
                    at,
                });
            }
        }
        if tx.writes.is_empty() {
            return Ok(self.head);
        }
        // The rows the transaction leaves, counted before anything applies.
        let mut touched: BTreeMap<(&str, &[u8]), bool> = BTreeMap::new();
        for write in &tx.writes {
            match write {
                Write::Put { family, key, .. } => touched.insert((family, key), true),
                Write::Delete { family, key } => touched.insert((family, key), false),
            };
        }
        let mut rows = self.rows.len();
        for ((family, key), present) in &touched {
            let was = self
                .rows
                .contains_key(&((*family).to_owned(), key.to_vec()));
            match (was, *present) {
                (false, true) => rows = rows.saturating_add(1),
                (true, false) => rows = rows.saturating_sub(1),
                _ => {}
            }
        }
        if rows > limits.rows {
            return Err(Refusal::Full { limit: limits.rows });
        }
        let rev = self
            .head
            .checked_add(1)
            .ok_or_else(|| unavailable("the store has made every revision it can"))?;
        for write in tx.writes {
            let (family, key, value) = match write {
                Write::Put { family, key, value } => (family, key, Some(value)),
                Write::Delete { family, key } => (family, key, None),
            };
            let row = (family.clone(), key.clone());
            let prior = match &value {
                Some(value) => self.rows.insert(row, (value.clone(), rev)),
                None => self.rows.remove(&row),
            };
            if prior.is_none() && value.is_none() {
                continue;
            }
            self.log.push_back(Logged {
                rev,
                family,
                key,
                value,
                prior,
            });
        }
        self.head = rev;
        // Past the changes kept, the oldest revisions go whole, and the horizon rises past them.
        while self.log.len() > limits.changes {
            let Some(oldest) = self.log.front().map(|c| c.rev) else {
                break;
            };
            while self.log.front().is_some_and(|c| c.rev == oldest) {
                self.log.pop_front();
            }
            self.horizon = self.horizon.max(oldest);
        }
        Ok(rev)
    }

    fn changes(&mut self, family: &str, after: Rev, limit: usize) -> Result<Changes, Refusal> {
        api::family(family)?;
        let limit = api::limit(limit)?;
        if after > self.head {
            return Err(Refusal::Ahead {
                asked: after,
                head: self.head,
            });
        }
        if after < self.horizon {
            return Err(Refusal::Compacted {
                asked: after,
                horizon: self.horizon,
            });
        }
        let mut changes: Vec<Change> = self
            .log
            .iter()
            .filter(|c| c.rev > after && c.family == family)
            .take(limit)
            .map(|c| Change {
                rev: c.rev,
                key: c.key.clone(),
                value: c.value.clone(),
            })
            .collect();
        let length = u64::try_from(changes.len()).unwrap_or(u64::MAX);
        if let Some(cut) = self.faults.cut(Op::Changes, 0, length) {
            changes.truncate(usize::try_from(cut).unwrap_or(usize::MAX));
        }
        Ok(Changes {
            changes,
            head: self.head,
        })
    }

    /// The family's newest change after `after`, if one is kept.
    fn newer(&self, family: &str, after: Rev) -> Result<Option<Rev>, Refusal> {
        api::family(family)?;
        if after > self.head {
            return Err(Refusal::Ahead {
                asked: after,
                head: self.head,
            });
        }
        if after < self.horizon {
            return Err(Refusal::Compacted {
                asked: after,
                horizon: self.horizon,
            });
        }
        Ok(self
            .log
            .iter()
            .rev()
            .find(|c| c.rev > after && c.family == family)
            .map(|c| c.rev))
    }

    fn compact(&mut self, before: Rev) -> Result<(), Refusal> {
        if before > self.head {
            return Err(Refusal::Ahead {
                asked: before,
                head: self.head,
            });
        }
        while self.log.front().is_some_and(|c| c.rev <= before) {
            self.log.pop_front();
        }
        self.horizon = self.horizon.max(before);
        Ok(())
    }
}

impl SimRecords {
    /// Rows in memory on `clock`, holding what `limits` says.
    #[must_use]
    pub fn new(clock: SimClock, limits: SimLimits) -> Self {
        Self {
            clock,
            limits,
            kept: Mutex::new(Kept {
                rows: BTreeMap::new(),
                log: VecDeque::new(),
                head: 0,
                horizon: 0,
                faults: Faults::default(),
            }),
        }
    }

    /// Arms `fault` for the next call of `op` (a cut, for the next page that reaches its row).
    pub fn arm(&self, op: Op, fault: Fault<Refusal>) -> Result<(), Unarmed> {
        self.kept.lock().faults.arm(op, fault, fits)
    }

    /// How many calls of `op` were made.
    #[must_use]
    pub fn attempts(&self, op: Op) -> u64 {
        self.kept.lock().faults.attempts(op)
    }

    /// What it injected, by kind.
    #[must_use]
    pub fn injected(&self) -> Counts {
        self.kept.lock().faults.injected().clone()
    }

    /// Moves the clock to `t`, as a call that blocked until then.
    fn block_until(&self, t: Instant) -> Result<(), Refusal> {
        self.clock.advance_to(t).map_err(unavailable)
    }

    /// How a call answers, with the fault armed for it: a refusal at once; a request that never
    /// arrives, refused at its deadline; an answer lost after its effect; an answer held, past its
    /// deadline refused there; or at once.
    fn answer<T>(
        &self,
        op: Op,
        until: Instant,
        apply: impl FnOnce(&mut Kept) -> Result<T, Refusal>,
    ) -> Result<T, Refusal> {
        let fault = self.kept.lock().faults.call(op);
        match fault {
            Some(Fault::Refuse(refusal)) => Err(refusal),
            Some(Fault::Vanish) => {
                self.block_until(until)?;
                Err(Refusal::Timeout)
            }
            Some(Fault::Lose) => {
                let _lost = apply(&mut self.kept.lock());
                Err(Refusal::UnknownOutcome)
            }
            Some(Fault::Delay(by)) => {
                let answered = apply(&mut self.kept.lock());
                match self.clock.now().checked_add(by) {
                    Some(at) if at <= until => {
                        self.block_until(at)?;
                        answered
                    }
                    _ => {
                        self.block_until(until)?;
                        Err(Refusal::Timeout)
                    }
                }
            }
            Some(Fault::Cut(_)) | None => apply(&mut self.kept.lock()),
        }
    }
}

impl Records for SimRecords {
    fn get(
        &self,
        family: &str,
        key: &[u8],
        at: Consistency,
        until: Instant,
    ) -> Result<Option<Row>, Refusal> {
        self.answer(Op::Get, until, |kept| kept.get(family, key, at))
    }

    fn scan(&self, family: &str, scan: Scan, until: Instant) -> Result<Page, Refusal> {
        self.answer(Op::Scan, until, |kept| kept.scan(family, &scan))
    }

    fn commit(&self, tx: Tx, until: Instant) -> Result<Rev, Refusal> {
        let limits = self.limits;
        self.answer(Op::Commit, until, |kept| kept.commit(tx, limits))
    }

    fn changes(
        &self,
        family: &str,
        after: Rev,
        limit: usize,
        until: Instant,
    ) -> Result<Changes, Refusal> {
        self.answer(Op::Changes, until, |kept| {
            kept.changes(family, after, limit)
        })
    }

    fn wait(&self, family: &str, after: Rev, until: Instant) -> Result<Rev, Refusal> {
        let newer = self.answer(Op::Wait, until, |kept| {
            kept.newer(family, after)
                .map(|newer| newer.map(|_| kept.head))
        })?;
        match newer {
            Some(head) => Ok(head),
            // Nothing else runs while a simulated call blocks: no change can come before the
            // deadline.
            None => {
                self.block_until(until)?;
                Err(Refusal::Timeout)
            }
        }
    }

    fn head(&self, until: Instant) -> Result<Rev, Refusal> {
        self.answer(Op::Head, until, |kept| Ok(kept.head))
    }

    fn compact(&self, before: Rev, until: Instant) -> Result<(), Refusal> {
        self.answer(Op::Compact, until, |kept| kept.compact(before))
    }
}

// -------------------------------------------------------------------------------- conformance

/// What a records store's conformance suite runs against.
pub trait RecordsFixture: Fixture {
    fn records(&self) -> &dyn Records;
    /// The clock the store's deadlines are on.
    fn clock(&self) -> &dyn Clock;
    /// Arms `fault` for the store's next call of `op`.
    fn arm(&self, op: Op, fault: Fault<Refusal>) -> Result<(), Failure>;
    /// How many calls of `op` the store has had.
    fn attempts(&self, op: Op) -> u64;
    /// How long a case's caller waits for an answer: each deadline is the clock's now plus this.
    fn patience(&self) -> Duration;
    /// How many rows a case writes to show paging: more than one page of [`RecordsFixture::page`].
    fn rows(&self) -> usize;
    /// The page a case's scans and change reads ask for.
    fn page(&self) -> usize;
}

/// The fixture for a [`SimRecords`]: the clock it was made with, and what the test that makes it
/// states.
pub struct SimRecordsFixture {
    pub records: SimRecords,
    pub clock: SimClock,
    pub patience: Duration,
    pub rows: usize,
    pub page: usize,
}

impl Fixture for SimRecordsFixture {
    fn drive<'a, T>(&'a self, work: BoxFuture<'a, T>) -> Result<T, Failure> {
        at_once(work)
    }

    fn injected(&self) -> Counts {
        self.records.injected()
    }
}

impl RecordsFixture for SimRecordsFixture {
    fn records(&self) -> &dyn Records {
        &self.records
    }

    fn clock(&self) -> &dyn Clock {
        &self.clock
    }

    fn arm(&self, op: Op, fault: Fault<Refusal>) -> Result<(), Failure> {
        self.records.arm(op, fault).map_err(|e| e.to_string())
    }

    fn attempts(&self, op: Op) -> u64 {
        self.records.attempts(op)
    }

    fn patience(&self) -> Duration {
        self.patience
    }

    fn rows(&self) -> usize {
        self.rows
    }

    fn page(&self) -> usize {
        self.page
    }
}

fn until<F: RecordsFixture>(f: &F) -> Result<Instant, Failure> {
    f.clock()
        .now()
        .checked_add(f.patience())
        .ok_or_else(|| "a deadline past what the platform's time can say".to_owned())
}

fn put(family: &str, key: &[u8], value: &[u8]) -> Write {
    Write::Put {
        family: family.into(),
        key: key.to_vec(),
        value: value.to_vec(),
    }
}

fn absent(family: &str, key: &[u8]) -> Cond {
    Cond::Absent {
        family: family.into(),
        key: key.to_vec(),
    }
}

fn row_at<F: RecordsFixture>(f: &F, family: &str, key: &[u8]) -> Result<Option<Row>, Failure> {
    fine(
        "a get",
        f.records()
            .get(family, key, Consistency::Linearizable, until(f)?),
    )
}

/// The keys a case writes `n` of, in key order.
fn keys(n: usize) -> Vec<Vec<u8>> {
    (0..n).map(|i| format!("key-{i:08}").into_bytes()).collect()
}

/// Writes rows `keys` in family `family`, one transaction each; their revisions.
fn fill<F: RecordsFixture>(f: &F, family: &str, keys: &[Vec<u8>]) -> Result<Vec<Rev>, Failure> {
    keys.iter()
        .map(|key| {
            let tx = Tx {
                conds: vec![absent(family, key)],
                writes: vec![put(family, key, key)],
            };
            fine("a commit", f.records().commit(tx, until(f)?))
        })
        .collect()
}

fn r1<F: RecordsFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        let tx = Tx {
            conds: vec![absent("r1", b"a"), absent("r1", b"b")],
            writes: vec![put("r1", b"a", b"one"), put("r1", b"b", b"two")],
        };
        let rev = fine("a commit", f.records().commit(tx, until(f)?))?;
        for (key, value) in [(&b"a"[..], &b"one"[..]), (b"b", b"two")] {
            same(
                "a row the commit wrote",
                row_at(f, "r1", key)?,
                Some(Row {
                    key: key.to_vec(),
                    value: value.to_vec(),
                    rev,
                }),
            )?;
        }
        same("head", fine("a head", f.records().head(until(f)?))?, rev)
    })
}

fn r2<F: RecordsFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        let rev = fill(f, "r2", &[b"a".to_vec()])?
            .first()
            .copied()
            .ok_or_else(|| "no revision".to_owned())?;
        let stale = rev
            .checked_sub(1)
            .ok_or_else(|| "a first commit at revision zero".to_owned())?;
        let conflicting = [
            absent("r2", b"a"),
            Cond::At {
                family: "r2".into(),
                key: b"a".to_vec(),
                rev: stale,
            },
        ];
        for cond in conflicting {
            let tx = Tx {
                conds: vec![cond.clone()],
                writes: vec![put("r2", b"a", b"over"), put("r2", b"b", b"new")],
            };
            same(
                &format!("a commit under {cond:?}"),
                f.records().commit(tx, until(f)?),
                Err(Refusal::Conflict {
                    family: "r2".into(),
                    key: b"a".to_vec(),
                    at: Some(rev),
                }),
            )?;
        }
        same(
            "the row after them",
            row_at(f, "r2", b"a")?.map(|row| row.value),
            Some(b"a".to_vec()),
        )?;
        same("a row they would have made", row_at(f, "r2", b"b")?, None)
    })
}

fn r3<F: RecordsFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        let rev = fill(f, "r3", &[b"a".to_vec()])?
            .first()
            .copied()
            .ok_or_else(|| "no revision".to_owned())?;
        let moved = |value: &[u8]| Tx {
            conds: vec![Cond::At {
                family: "r3".into(),
                key: b"a".to_vec(),
                rev,
            }],
            writes: vec![put("r3", b"a", value)],
        };
        let first = f.records().commit(moved(b"first"), until(f)?);
        let second = f.records().commit(moved(b"second"), until(f)?);
        let won = match (&first, &second) {
            (Ok(_), Err(Refusal::Conflict { .. })) => &b"first"[..],
            (Err(Refusal::Conflict { .. }), Ok(_)) => &b"second"[..],
            other => return Err(format!("two commits under one condition ended {other:?}")),
        };
        same(
            "the row",
            row_at(f, "r3", b"a")?.map(|row| row.value),
            Some(won.to_vec()),
        )
    })
}

fn r4<F: RecordsFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        let revs = fill(f, "r4", &keys(f.rows()))?;
        ensure(
            revs.iter().zip(revs.iter().skip(1)).all(|(a, b)| a < b),
            || format!("revisions did not grow: {revs:?}"),
        )?;
        same(
            "head",
            fine("a head", f.records().head(until(f)?))?,
            revs.last().copied().unwrap_or_default(),
        )
    })
}

fn r5<F: RecordsFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        let written = keys(f.rows());
        fill(f, "r5", &written)?;
        fill(f, "r5-other", &written)?;
        let mut seen = Vec::new();
        let mut after = None;
        for _ in 0..=written.len() {
            let scan = Scan {
                prefix: b"key-".to_vec(),
                after: after.clone(),
                limit: f.page(),
                at: Consistency::Linearizable,
            };
            let page = fine("a scan", f.records().scan("r5", scan, until(f)?))?;
            ensure(page.rows.len() <= f.page(), || {
                format!("a page of {} rows, asked {}", page.rows.len(), f.page())
            })?;
            after = page.rows.last().map(|row| row.key.clone());
            seen.extend(page.rows.into_iter().map(|row| row.key));
            if !page.more {
                break;
            }
        }
        same("the keys scanned, in order", seen, written)
    })
}

fn r6<F: RecordsFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        let start = fine("a head", f.records().head(until(f)?))?;
        let put_rev = fill(f, "r6", &[b"a".to_vec()])?
            .first()
            .copied()
            .ok_or_else(|| "no revision".to_owned())?;
        let delete = Tx {
            conds: vec![],
            writes: vec![Write::Delete {
                family: "r6".into(),
                key: b"a".to_vec(),
            }],
        };
        let delete_rev = fine("a delete", f.records().commit(delete, until(f)?))?;
        let read = fine(
            "a change read",
            f.records().changes("r6", start, f.page(), until(f)?),
        )?;
        same(
            "the changes",
            read.changes,
            vec![
                Change {
                    rev: put_rev,
                    key: b"a".to_vec(),
                    value: Some(b"a".to_vec()),
                },
                Change {
                    rev: delete_rev,
                    key: b"a".to_vec(),
                    value: None,
                },
            ],
        )?;
        same("the feed's head", read.head, delete_rev)?;
        let ahead = delete_rev.saturating_add(1);
        same(
            "a change read ahead of head",
            f.records().changes("r6", ahead, f.page(), until(f)?),
            Err(Refusal::Ahead {
                asked: ahead,
                head: delete_rev,
            }),
        )
    })
}

fn r7<F: RecordsFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        let first = fill(f, "r7", &[b"a".to_vec()])?
            .first()
            .copied()
            .ok_or_else(|| "no revision".to_owned())?;
        let over = Tx {
            conds: vec![],
            writes: vec![put("r7", b"a", b"later")],
        };
        let later = fine("a commit", f.records().commit(over, until(f)?))?;
        same(
            "the row at the first revision",
            fine(
                "a snapshot read",
                f.records()
                    .get("r7", b"a", Consistency::Snapshot(first), until(f)?),
            )?
            .map(|row| row.value),
            Some(b"a".to_vec()),
        )?;
        fine("a compaction", f.records().compact(later, until(f)?))?;
        let refusal = refused(
            "a snapshot read before the horizon",
            f.records()
                .get("r7", b"a", Consistency::Snapshot(first), until(f)?),
        )?;
        ensure(
            matches!(refusal, Refusal::Compacted { asked, .. } if asked == first),
            || format!("refused {refusal:?}, not Compacted naming {first}"),
        )?;
        let refusal = refused(
            "a change read from before the horizon",
            f.records().changes("r7", first, f.page(), until(f)?),
        )?;
        ensure(matches!(refusal, Refusal::Compacted { .. }), || {
            format!("refused {refusal:?}, not Compacted")
        })
    })
}

fn r8<F: RecordsFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        let start = fine("a head", f.records().head(until(f)?))?;
        let rev = fill(f, "r8", &[b"a".to_vec()])?
            .first()
            .copied()
            .ok_or_else(|| "no revision".to_owned())?;
        let waited = fine("a wait", f.records().wait("r8", start, until(f)?))?;
        ensure(waited >= rev, || {
            format!("a wait answered {waited}, before the change at {rev}")
        })?;
        let deadline = until(f)?;
        same(
            "a wait for a change that never comes",
            f.records().wait("r8", rev, deadline),
            Err(Refusal::Timeout),
        )?;
        ensure(f.clock().now() >= deadline, || {
            "a wait was refused before its deadline".into()
        })
    })
}

fn r9<F: RecordsFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        let over = api::TX_MAX.saturating_add(1);
        let wide = Tx {
            conds: vec![],
            writes: (0..over).map(|_| put("r9", b"a", b"v")).collect(),
        };
        let refusal = refused(
            "a transaction past its most writes",
            f.records().commit(wide, until(f)?),
        )?;
        ensure(matches!(refusal, Refusal::TooLarge { .. }), || {
            format!("refused {refusal:?}, not TooLarge")
        })?;
        for (family, key) in [("r9", &b""[..]), ("R9 Not A Family", b"a")] {
            let tx = Tx {
                conds: vec![],
                writes: vec![put(family, key, b"v")],
            };
            let refusal = refused(
                &format!("a write to {family:?} at {key:?}"),
                f.records().commit(tx, until(f)?),
            )?;
            ensure(matches!(refusal, Refusal::Invalid { .. }), || {
                format!("refused {refusal:?}, not Invalid")
            })?;
        }
        same("a row they would have made", row_at(f, "r9", b"a")?, None)
    })
}

fn r10<F: RecordsFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        let tx = Tx {
            conds: vec![absent("r10", b"a")],
            writes: vec![put("r10", b"a", b"once")],
        };
        f.arm(Op::Commit, Fault::Lose)?;
        same(
            "a commit whose answer is lost",
            f.records().commit(tx.clone(), until(f)?),
            Err(Refusal::UnknownOutcome),
        )?;
        // The caller reconciles by reading: the commit applied.
        let row = row_at(f, "r10", b"a")?.ok_or_else(|| "the commit did not apply".to_owned())?;
        same("the row", row.value.as_slice(), &b"once"[..])?;
        same(
            "the commit asked again under its condition",
            f.records().commit(tx, until(f)?),
            Err(Refusal::Conflict {
                family: "r10".into(),
                key: b"a".to_vec(),
                at: Some(row.rev),
            }),
        )?;
        same(
            "the row after it",
            row_at(f, "r10", b"a")?.map(|r| r.rev),
            Some(row.rev),
        )
    })
}

fn r11<F: RecordsFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        let past = f
            .patience()
            .checked_add(f.patience())
            .ok_or_else(|| "a patience as long as time".to_owned())?;
        let applied = Tx {
            conds: vec![absent("r11", b"held")],
            writes: vec![put("r11", b"held", b"v")],
        };
        f.arm(Op::Commit, Fault::Delay(past))?;
        let deadline = until(f)?;
        same(
            "a commit whose answer comes after the deadline",
            f.records().commit(applied, deadline),
            Err(Refusal::Timeout),
        )?;
        ensure(f.clock().now() >= deadline, || {
            "told it timed out before its deadline".into()
        })?;
        ensure(row_at(f, "r11", b"held")?.is_some(), || {
            "the held commit did not apply".into()
        })?;
        let lost = Tx {
            conds: vec![absent("r11", b"vanished")],
            writes: vec![put("r11", b"vanished", b"v")],
        };
        f.arm(Op::Commit, Fault::Vanish)?;
        same(
            "a commit that never arrives",
            f.records().commit(lost, until(f)?),
            Err(Refusal::Timeout),
        )?;
        same(
            "the row it would have made",
            row_at(f, "r11", b"vanished")?,
            None,
        )?;
        f.arm(Op::Head, Fault::Delay(f.patience()))?;
        fine(
            "a head answered at its deadline",
            f.records().head(until(f)?),
        )
        .map(drop)
    })
}

fn r12<F: RecordsFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        let written = keys(f.rows());
        let start = fine("a head", f.records().head(until(f)?))?;
        fill(f, "r12", &written)?;
        for cut in 0..=f.page() {
            let at = u64::try_from(cut).unwrap_or(u64::MAX);
            f.arm(Op::Scan, Fault::Cut(at))?;
            let mut seen = Vec::new();
            let mut after = None;
            for _ in 0..=written.len() {
                let scan = Scan {
                    prefix: Vec::new(),
                    after: after.clone(),
                    limit: f.page(),
                    at: Consistency::Linearizable,
                };
                let page = fine("a scan", f.records().scan("r12", scan, until(f)?))?;
                if let Some(last) = page.rows.last() {
                    after = Some(last.key.clone());
                }
                seen.extend(page.rows.into_iter().map(|row| row.key));
                if !page.more {
                    break;
                }
            }
            same(
                &format!("the keys scanned, a page cut at row {cut}"),
                &seen,
                &written,
            )?;
            f.arm(Op::Changes, Fault::Cut(at))?;
            let mut changed = Vec::new();
            let mut from = start;
            for _ in 0..=written.len() {
                let read = fine(
                    "a change read",
                    f.records().changes("r12", from, f.page(), until(f)?),
                )?;
                let Some(last) = read.changes.last().map(|c| c.rev) else {
                    if from >= read.head {
                        break;
                    }
                    continue;
                };
                from = last;
                changed.extend(read.changes.into_iter().map(|c| c.key));
            }
            same(
                &format!("the changes read, a page cut at row {cut}"),
                &changed,
                &written,
            )?;
        }
        Ok(())
    })
}

/// One refusal of every kind the trait names.
fn every_refusal() -> Vec<Refusal> {
    vec![
        Refusal::Conflict {
            family: "f".into(),
            key: b"k".to_vec(),
            at: None,
        },
        Refusal::TooLarge {
            what: "w".into(),
            size: 0,
            limit: 0,
        },
        Refusal::Invalid {
            what: "w".into(),
            why: "y".into(),
        },
        Refusal::Full { limit: 0 },
        Refusal::Compacted {
            asked: 0,
            horizon: 0,
        },
        Refusal::Ahead { asked: 0, head: 0 },
        Refusal::Timeout,
        Refusal::Unavailable("u".into()),
        Refusal::UnknownOutcome,
        Refusal::NotLeader { leader: None },
        Refusal::Fenced("f".into()),
    ]
}

fn r13<F: RecordsFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        for refusal in every_refusal() {
            let before = f.attempts(Op::Head);
            f.arm(Op::Head, Fault::Refuse(refusal.clone()))?;
            same(
                "a head the store refuses",
                f.records().head(until(f)?),
                Err(refusal.clone()),
            )?;
            same(
                &format!("attempts of a head refused {refusal:?}"),
                f.attempts(Op::Head),
                before.saturating_add(1),
            )?;
        }
        Ok(())
    })
}

/// The clauses of Amphora's `Records` that Reliquary relies on (`docs/design/seams.md`,
/// "Records").
#[must_use]
pub fn cases<F: RecordsFixture>() -> Vec<Case<F>> {
    vec![
        Case {
            id: "RECORDS-1",
            says: "a commit whose conditions hold applies every write at one new revision: a get gives each row at it, and head is it",
            check: r1::<F>,
        },
        Case {
            id: "RECORDS-2",
            says: "a commit with an Absent condition on a row that exists, or an At condition at another revision, is refused Conflict naming the row's revision, and applies none of its writes",
            check: r2::<F>,
        },
        Case {
            id: "RECORDS-3",
            says: "of two commits conditioned on one row at one revision, exactly one applies; the other is refused Conflict",
            check: r3::<F>,
        },
        Case {
            id: "RECORDS-4",
            says: "revisions only grow: each commit's is above every earlier one's",
            check: r4::<F>,
        },
        Case {
            id: "RECORDS-5",
            says: "a scan gives a family's rows under a prefix in key order, a page of at most its limit at a time; going on after each page's last key gives every row once",
            check: r5::<F>,
        },
        Case {
            id: "RECORDS-6",
            says: "the change feed gives every change after a revision, oldest first, a delete as no value; a revision ahead of head is refused Ahead",
            check: r6::<F>,
        },
        Case {
            id: "RECORDS-7",
            says: "a read at a revision gives the rows as they were then; once compacted past it, it is refused Compacted, naming it",
            check: r7::<F>,
        },
        Case {
            id: "RECORDS-8",
            says: "a wait completes once a change of its family after its revision exists, and is refused Timeout at its deadline when none comes",
            check: r8::<F>,
        },
        Case {
            id: "RECORDS-9",
            says: "a transaction past amphora-api's bounds, or with an empty key or a family outside its alphabet, is refused before anything applies",
            check: r9::<F>,
        },
        Case {
            id: "RECORDS-10",
            says: "a commit whose answer is lost after it applied is refused UnknownOutcome; a read shows it, and asked again under its condition it is refused Conflict: nothing applies twice",
            check: r10::<F>,
        },
        Case {
            id: "RECORDS-11",
            says: "an answer delayed past its caller's deadline is refused Timeout at the deadline, and what the call did is learned by reading",
            check: r11::<F>,
        },
        Case {
            id: "RECORDS-12",
            says: "a scan or a change read cut at any row gives the rows before the cut; going on from the last gives the rest, none twice",
            check: r12::<F>,
        },
        Case {
            id: "RECORDS-13",
            says: "every refusal the store gives reaches its caller as it is, after one attempt: the seam never asks again",
            check: r13::<F>,
        },
    ]
}
