//! What a simulator can be made to do wrong (AC-RQ0.4.3), and the counts that show a suite made it
//! do each (T-RQ0.4.9).
//!
//! A test, or later a nemesis (plan RQ0.6), arms a [`Fault`] for one operation of a simulator. The
//! simulator's next call of that operation takes it, applies it and counts it by its kind: the
//! refusal's variant's name (`Full`), or a phrase for the others ([`DELAY`], [`VANISH`], [`LOSE`],
//! [`CUT`]), so no kind of fault is counted under a refusal's name. A cut waits until a
//! call's stream reaches its byte. One fault is armed per operation at a time: arming a second is
//! refused [`Unarmed::Armed`], so none is overwritten unseen, and a fault an operation cannot meet
//! (a cut of a call that carries no stream) is refused [`Unarmed::Unfit`], so none is armed and
//! then silently never applied.

use std::collections::BTreeMap;
use std::future::poll_fn;
use std::time::{Duration, Instant};

use crate::seams::clock::{self, Clock};

/// A refusal's kind: its variant's name, as fault counts and `docs/design/seams.md` say it.
pub trait Kind {
    fn kind(&self) -> &'static str;
}

/// The kind of an answer delayed ([`Fault::Delay`]).
pub const DELAY: &str = "answer delayed";
/// The kind of a request that never arrives ([`Fault::Vanish`]).
pub const VANISH: &str = "request vanished";
/// The kind of an answer lost after its effect applied ([`Fault::Lose`]).
pub const LOSE: &str = "answer lost";
/// The kind of a stream cut ([`Fault::Cut`]).
pub const CUT: &str = "stream cut";

/// One fault, injected into one call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fault<R> {
    /// The call is refused this, at once, and nothing applies.
    Refuse(R),
    /// The call applies at once and its answer comes this much later: past its caller's deadline,
    /// the caller is told at its deadline that it is late, and what the call did is the caller's
    /// to learn by reading.
    Delay(Duration),
    /// The request never arrives: nothing applies, and no answer comes before the caller's
    /// deadline.
    Vanish,
    /// The call applies, and its answer is lost on the way back: the caller is told the outcome is
    /// unknown.
    Lose,
    /// The call's stream is cut once it reaches this byte (of an object, or a row of a page, or a
    /// message of a topic): what came before the cut arrived, nothing after it.
    Cut(u64),
}

impl<R: Kind> Fault<R> {
    /// Its kind, as it is counted.
    #[must_use]
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Refuse(refusal) => refusal.kind(),
            Self::Delay(_) => DELAY,
            Self::Vanish => VANISH,
            Self::Lose => LOSE,
            Self::Cut(_) => CUT,
        }
    }
}

/// How many faults of each kind were injected. A count stays at `u64::MAX` once there, which no
/// run approaches.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Counts(BTreeMap<&'static str, u64>);

impl Counts {
    /// Counts one fault of `kind`.
    pub fn add(&mut self, kind: &'static str) {
        let n = self.0.entry(kind).or_insert(0);
        *n = n.saturating_add(1);
    }

    /// How many of `kind` were counted.
    #[must_use]
    pub fn get(&self, kind: &str) -> u64 {
        self.0.get(kind).copied().unwrap_or(0)
    }

    /// Adds every count of `other` to this one.
    pub fn merge(&mut self, other: &Self) {
        for (kind, n) in &other.0 {
            let mine = self.0.entry(kind).or_insert(0);
            *mine = mine.saturating_add(*n);
        }
    }

    /// The kinds of `all` this has counted none of.
    #[must_use]
    pub fn missing(&self, all: &[&'static str]) -> Vec<&'static str> {
        all.iter()
            .copied()
            .filter(|kind| self.get(kind) == 0)
            .collect()
    }

    /// Each kind counted, and how many.
    pub fn iter(&self) -> impl Iterator<Item = (&'static str, u64)> + '_ {
        self.0.iter().map(|(kind, n)| (*kind, *n))
    }
}

/// Why a fault was not armed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Unarmed {
    #[error(
        "a fault is armed for that operation already: one at a time, so none is overwritten unseen"
    )]
    Armed,
    #[error("that operation cannot meet a fault of kind {0}")]
    Unfit(&'static str),
}

/// The faults armed for a simulator's operations, one per operation at a time; what it injected,
/// by kind; and each operation's attempts, which show that a refusal was not asked again.
#[derive(Debug)]
pub struct Faults<Op, R> {
    armed: BTreeMap<Op, Fault<R>>,
    attempts: BTreeMap<Op, u64>,
    injected: Counts,
}

impl<Op, R> Default for Faults<Op, R> {
    fn default() -> Self {
        Self {
            armed: BTreeMap::new(),
            attempts: BTreeMap::new(),
            injected: Counts::default(),
        }
    }
}

impl<Op: Ord + Copy, R: Kind> Faults<Op, R> {
    /// Arms `fault` for the next call of `op` that it fits; `fits` says whether `op` can meet it.
    pub fn arm(
        &mut self,
        op: Op,
        fault: Fault<R>,
        fits: impl Fn(Op, &Fault<R>) -> bool,
    ) -> Result<(), Unarmed> {
        if !fits(op, &fault) {
            return Err(Unarmed::Unfit(fault.kind()));
        }
        if self.armed.contains_key(&op) {
            return Err(Unarmed::Armed);
        }
        self.armed.insert(op, fault);
        Ok(())
    }

    /// A call of `op` begins: counts its attempt, and takes the fault armed for it, but a cut,
    /// which stays armed until its call's stream reaches its byte ([`Faults::cut`]).
    pub fn call(&mut self, op: Op) -> Option<Fault<R>> {
        let n = self.attempts.entry(op).or_insert(0);
        *n = n.saturating_add(1);
        if matches!(self.armed.get(&op), None | Some(Fault::Cut(_))) {
            return None;
        }
        let fault = self.armed.remove(&op)?;
        self.injected.add(fault.kind());
        Some(fault)
    }

    /// Where a stream of `op` that goes from byte `from` to byte `to` (inclusive) is cut: the
    /// armed cut's byte, when it lies between them, taken and counted.
    pub fn cut(&mut self, op: Op, from: u64, to: u64) -> Option<u64> {
        let Some(Fault::Cut(at)) = self.armed.get(&op) else {
            return None;
        };
        let at = *at;
        if at < from || at > to {
            return None;
        }
        self.armed.remove(&op);
        self.injected.add(CUT);
        Some(at)
    }

    /// How many calls of `op` were made.
    #[must_use]
    pub fn attempts(&self, op: Op) -> u64 {
        self.attempts.get(&op).copied().unwrap_or(0)
    }

    /// What was injected, by kind.
    #[must_use]
    pub fn injected(&self) -> &Counts {
        &self.injected
    }
}

/// What a seam's refusals say when a call ends without its answer.
pub trait Unanswered: Kind + Sized {
    /// The caller's deadline passed with no answer.
    fn late() -> Self;
    /// The request went whole and its answer was lost.
    fn lost() -> Self;
    /// The clock the answer was held on refused to wait.
    fn clock(refusal: clock::Refusal) -> Self;
}

/// How an asynchronous simulator's call answers, given the fault armed for it: refused at once;
/// a request that never arrives, refused late at its caller's deadline; applied, and its answer
/// lost; applied, and its answer held (past its caller's deadline, refused late there); or applied
/// and answered at once. `apply` makes the call's effect and its answer, taking the simulator's
/// lock itself, so no lock is held while the answer is held.
pub async fn answer<T: Send, R: Unanswered + Send>(
    clock: &dyn Clock,
    fault: Option<Fault<R>>,
    deadline: Instant,
    apply: impl FnOnce() -> Result<T, R> + Send,
) -> Result<T, R> {
    match fault {
        Some(Fault::Refuse(refusal)) => Err(refusal),
        Some(Fault::Vanish) => {
            held(clock, None, deadline).await.map_err(R::clock)?;
            Err(R::late())
        }
        Some(Fault::Lose) => {
            let _lost = apply();
            Err(R::lost())
        }
        Some(Fault::Delay(by)) => {
            let answered = apply();
            let at = clock.now().checked_add(by);
            if held(clock, at, deadline).await.map_err(R::clock)? {
                answered
            } else {
                Err(R::late())
            }
        }
        Some(Fault::Cut(_)) | None => apply(),
    }
}

/// When an answer that comes at `at` reaches a caller whose deadline is `deadline`: waits on
/// `clock` until whichever comes first, and says whether the answer came in time. An `at` the
/// clock cannot say (`None`, past the farthest instant) never comes in time.
pub async fn held(
    clock: &dyn Clock,
    at: Option<Instant>,
    deadline: Instant,
) -> Result<bool, clock::Refusal> {
    let (until, in_time) = match at {
        Some(at) if at <= deadline => (at, true),
        _ => (deadline, false),
    };
    let mut sleep = clock.sleep_until(until);
    // Polled by name, so the recursion gate reads this call as a sleep's poll: an `.await` on the
    // boxed sleep is, at its call site, a call through a vtable's slot with no method's name.
    poll_fn(|cx| sleep.as_mut().poll(cx)).await?;
    Ok(in_time)
}
