//! `Entropy`: bytes no one can predict, for what Reliquary must not repeat or let be guessed: an
//! operation's identity (Mantle's are 128 bits from the operating system), an upload's key, a
//! grant's key. A node's come from the operating system, whose source can be unready for a while
//! after a machine starts (Linux's `getrandom` blocks until its pool is seeded): a fill waits for
//! it until its caller's deadline, and no longer. [`SimEntropy`] is a function of its seed, so a
//! simulated run is a function of its seed (plan RQ0.6).
//!
//! A fill fills every byte asked or is refused: a source that gives fewer bytes than asked is
//! refused [`Refusal::Cut`], never taken as whole.

use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::Mutex;

use aws_lc_rs::digest::{self, SHA256, SHA256_OUTPUT_LEN};

use crate::seams::BoxFuture;
use crate::seams::clock::{self, Clock, SimClock};
use crate::seams::conformance::{Case, Failure, Fixture, ensure, fine, same};
use crate::seams::fault::{
    self, CUT, Counts, DELAY, Fault, Faults, Kind, LOSE, Unanswered, Unarmed, VANISH,
};

/// Bytes no one can predict.
pub trait Entropy: Send + Sync + 'static {
    /// Fills every byte of `into` by `deadline`, or refuses.
    fn fill<'a>(
        &'a self,
        into: &'a mut [u8],
        deadline: Instant,
    ) -> BoxFuture<'a, Result<(), Refusal>>;
}

/// Why a fill was refused. What the buffer holds after one is not to be used.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Refusal {
    /// The source failed.
    #[error("no unpredictable bytes to be had: {0}")]
    Unavailable(String),
    /// The source was not ready by the caller's deadline.
    #[error("the source was not ready by the deadline")]
    Late,
    /// The bytes were drawn and the answer lost: a fill asked again draws others.
    #[error("the fill's answer was lost after its bytes were drawn")]
    Lost,
    /// The source gave the first `at` bytes asked, and no more.
    #[error("the source stopped after {at} bytes")]
    Cut { at: u64 },
}

impl Kind for Refusal {
    fn kind(&self) -> &'static str {
        match self {
            Self::Unavailable(_) => "Unavailable",
            Self::Late => "Late",
            Self::Lost => "Lost",
            Self::Cut { .. } => "Cut",
        }
    }
}

impl Unanswered for Refusal {
    fn late() -> Self {
        Self::Late
    }

    fn lost() -> Self {
        Self::Lost
    }

    fn clock(refusal: clock::Refusal) -> Self {
        Self::Unavailable(format!("the source's clock: {refusal}"))
    }
}

/// The faults an entropy simulator injects (`docs/design/seams.md`, "Entropy"): each refusal, a
/// fill whose answer is held past its caller's deadline, one that never comes, a fill whose answer
/// is lost after its bytes were drawn, and a source cut at any byte of a fill.
pub const FAULTS: &[&str] = &[
    "Unavailable",
    "Late",
    "Lost",
    "Cut",
    DELAY,
    VANISH,
    LOSE,
    CUT,
];

/// An entropy simulator's one operation that faults are armed for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Op {
    Fill,
}

fn fits(_: Op, _: &Fault<Refusal>) -> bool {
    true
}

/// Bytes that are a function of a seed: SHA-256 of the seed and a block's number, block after
/// block (SHA-256 in counter mode, which is a pseudorandom function of its input as far as SHA-256
/// is one: FIPS 180-4). Deterministic, so a simulation's run is a function of its seed; not for
/// anything a real adversary sees.
pub struct Stream {
    seed: u64,
    /// The next block's number.
    block: u64,
    /// The block being given, and how many of its bytes were given.
    pending: [u8; SHA256_OUTPUT_LEN],
    used: usize,
}

impl Stream {
    /// The stream of `seed`.
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            block: 0,
            pending: [0; SHA256_OUTPUT_LEN],
            used: SHA256_OUTPUT_LEN,
        }
    }

    /// Fills `into` with the stream's next bytes. Refused only past its every block, 2^64 of them.
    pub fn fill(&mut self, into: &mut [u8]) -> Result<(), Refusal> {
        for byte in into {
            *byte = self.next_byte()?;
        }
        Ok(())
    }

    fn next_byte(&mut self) -> Result<u8, Refusal> {
        if self.used >= self.pending.len() {
            let mut context = digest::Context::new(&SHA256);
            context.update(&self.seed.to_le_bytes());
            context.update(&self.block.to_le_bytes());
            let block = context.finish();
            for (into, from) in self.pending.iter_mut().zip(block.as_ref()) {
                *into = *from;
            }
            self.block = self.block.checked_add(1).ok_or_else(|| {
                Refusal::Unavailable("the simulated source has given every block it has".into())
            })?;
            self.used = 0;
        }
        let byte = self.pending.get(self.used).copied().ok_or_else(|| {
            Refusal::Unavailable("the simulated source's block is shorter than SHA-256's".into())
        })?;
        self.used = self.used.saturating_add(1);
        Ok(byte)
    }
}

struct Drawing {
    stream: Stream,
    faults: Faults<Op, Refusal>,
}

/// A source whose bytes are a [`Stream`]'s, on a clock its held answers wait on.
pub struct SimEntropy {
    clock: Arc<dyn Clock>,
    inner: Mutex<Drawing>,
}

impl SimEntropy {
    /// A source whose bytes are a function of `seed`.
    #[must_use]
    pub fn new(seed: u64, clock: Arc<dyn Clock>) -> Self {
        Self {
            clock,
            inner: Mutex::new(Drawing {
                stream: Stream::new(seed),
                faults: Faults::default(),
            }),
        }
    }

    /// Arms `fault` for the next fill (a cut, for the next fill that reaches its byte).
    pub fn arm(&self, fault: Fault<Refusal>) -> Result<(), Unarmed> {
        self.inner.lock().faults.arm(Op::Fill, fault, fits)
    }

    /// How many fills were asked.
    #[must_use]
    pub fn attempts(&self) -> u64 {
        self.inner.lock().faults.attempts(Op::Fill)
    }

    /// What it injected, by kind.
    #[must_use]
    pub fn injected(&self) -> Counts {
        self.inner.lock().faults.injected().clone()
    }
}

impl Entropy for SimEntropy {
    fn fill<'a>(
        &'a self,
        into: &'a mut [u8],
        deadline: Instant,
    ) -> BoxFuture<'a, Result<(), Refusal>> {
        Box::pin(async move {
            let fault = self.inner.lock().faults.call(Op::Fill);
            fault::answer(self.clock.as_ref(), fault, deadline, || {
                let mut drawing = self.inner.lock();
                let asked = u64::try_from(into.len()).unwrap_or(u64::MAX);
                let cut = drawing.faults.cut(Op::Fill, 0, asked);
                let filled = cut.map_or(into.len(), |at| {
                    usize::try_from(at).map_or(into.len(), |at| at.min(into.len()))
                });
                let given = into
                    .get_mut(..filled)
                    .ok_or_else(|| Refusal::Unavailable("a fill past its buffer".into()))?;
                drawing.stream.fill(given)?;
                match cut {
                    Some(at) => Err(Refusal::Cut { at }),
                    None => Ok(()),
                }
            })
            .await
        })
    }
}

// -------------------------------------------------------------------------------- conformance

/// What an entropy source's conformance suite runs against.
pub trait EntropyFixture: Fixture {
    fn entropy(&self) -> &dyn Entropy;
    /// The clock its callers' deadlines are on.
    fn clock(&self) -> &dyn Clock;
    /// Arms `fault` for the source's next fill.
    fn arm(&self, fault: Fault<Refusal>) -> Result<(), Failure>;
    /// How many fills the source has had.
    fn attempts(&self) -> u64;
    /// How long a case's caller waits for a fill: each deadline is the clock's now plus this.
    fn patience(&self) -> Duration;
    /// How many draws a case makes to show none repeats, as the fixture's maker states.
    fn draws(&self) -> usize;
}

/// The fixture for a [`SimEntropy`]: the clock it was made with, and what the test that makes it
/// states.
pub struct SimEntropyFixture {
    pub entropy: SimEntropy,
    pub clock: SimClock,
    pub patience: Duration,
    pub draws: usize,
    pub steps: u64,
}

impl Fixture for SimEntropyFixture {
    fn drive<'a, T>(&'a self, work: BoxFuture<'a, T>) -> Result<T, Failure> {
        self.clock
            .run(work, self.steps)
            .map_err(|stalled| stalled.to_string())
    }

    fn injected(&self) -> Counts {
        self.entropy.injected()
    }
}

impl EntropyFixture for SimEntropyFixture {
    fn entropy(&self) -> &dyn Entropy {
        &self.entropy
    }

    fn clock(&self) -> &dyn Clock {
        &self.clock
    }

    fn arm(&self, fault: Fault<Refusal>) -> Result<(), Failure> {
        self.entropy.arm(fault).map_err(|e| e.to_string())
    }

    fn attempts(&self) -> u64 {
        self.entropy.attempts()
    }

    fn patience(&self) -> Duration {
        self.patience
    }

    fn draws(&self) -> usize {
        self.draws
    }
}

/// An operation identity's width, in bytes: Mantle's 128 bits.
fn identity() -> usize {
    size_of::<u128>()
}

fn deadline<F: EntropyFixture>(f: &F) -> Result<Instant, Failure> {
    f.clock()
        .now()
        .checked_add(f.patience())
        .ok_or_else(|| "a deadline past what the platform's time can say".to_owned())
}

/// An identity drawn from `f`'s source.
async fn draw<F: EntropyFixture>(f: &F) -> Result<Result<Vec<u8>, Refusal>, Failure> {
    let mut id = vec![0u8; identity()];
    Ok(f.entropy().fill(&mut id, deadline(f)?).await.map(|()| id))
}

fn e1<F: EntropyFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        let mut seen = std::collections::BTreeSet::new();
        for _ in 0..f.draws() {
            let id = fine("a fill", draw(f).await?)?;
            ensure(seen.insert(id), || {
                format!("a draw repeated within {} draws", seen.len())
            })?;
        }
        // Across many of SHA-256's blocks at once, and none at all.
        let mut wide = vec![0u8; identity().saturating_mul(f.draws())];
        fine(
            "a wide fill",
            f.entropy().fill(&mut wide, deadline(f)?).await,
        )?;
        let narrow: std::collections::BTreeSet<&[u8]> = wide.chunks(identity()).collect();
        same(
            "distinct identities in a wide fill",
            narrow.len(),
            f.draws(),
        )?;
        fine(
            "a fill of nothing",
            f.entropy().fill(&mut [], deadline(f)?).await,
        )
    })
}

fn e2<F: EntropyFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        let past = f
            .patience()
            .checked_add(f.patience())
            .ok_or_else(|| "a patience as long as time".to_owned())?;
        for fault in [Fault::Delay(past), Fault::Vanish] {
            f.arm(fault.clone())?;
            let mut id = vec![0u8; identity()];
            let due = deadline(f)?;
            same(
                &format!("a fill {fault:?}"),
                f.entropy().fill(&mut id, due).await,
                Err(Refusal::Late),
            )?;
            ensure(f.clock().now() >= due, || {
                "told late before its deadline".into()
            })?;
        }
        f.arm(Fault::Delay(f.patience()))?;
        fine("a fill ready at its deadline", draw(f).await?).map(drop)
    })
}

fn e3<F: EntropyFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        for at in 0..=identity() {
            let at = u64::try_from(at).unwrap_or(u64::MAX);
            f.arm(Fault::Cut(at))?;
            same(
                &format!("a fill cut after {at} bytes"),
                draw(f).await?,
                Err(Refusal::Cut { at }),
            )?;
        }
        Ok(())
    })
}

fn e4<F: EntropyFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        f.arm(Fault::Lose)?;
        let mut lost = vec![0u8; identity()];
        same(
            "a fill whose answer is lost",
            f.entropy().fill(&mut lost, deadline(f)?).await,
            Err(Refusal::Lost),
        )?;
        // The bytes were drawn: asked again, the source gives others.
        let again = fine("the fill asked again", draw(f).await?)?;
        ensure(again != lost, || {
            "a fill asked again after its answer was lost gave the same bytes".into()
        })
    })
}

/// One refusal of every kind a source gives.
fn every_refusal() -> Vec<Refusal> {
    vec![
        Refusal::Unavailable("the source failed".into()),
        Refusal::Late,
        Refusal::Lost,
        Refusal::Cut { at: 0 },
    ]
}

fn e5<F: EntropyFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        for refusal in every_refusal() {
            let before = f.attempts();
            f.arm(Fault::Refuse(refusal.clone()))?;
            same(
                "a fill the source refuses",
                draw(f).await?,
                Err(refusal.clone()),
            )?;
            same(
                &format!("attempts of a fill refused {refusal:?}"),
                f.attempts(),
                before.saturating_add(1),
            )?;
        }
        fine("the next fill", draw(f).await?).map(drop)
    })
}

/// The `Entropy` contract, clause by clause (`docs/design/seams.md`, "Entropy").
#[must_use]
pub fn cases<F: EntropyFixture>() -> Vec<Case<F>> {
    vec![
        Case {
            id: "ENTROPY-1",
            says: "a fill fills every byte asked, and no two operation identities drawn repeat",
            check: e1::<F>,
        },
        Case {
            id: "ENTROPY-2",
            says: "a fill whose source is not ready by its caller's deadline, or whose answer never comes, is refused Late at the deadline",
            check: e2::<F>,
        },
        Case {
            id: "ENTROPY-3",
            says: "a fill cut at any byte is refused Cut, naming the bytes given, never given as whole",
            check: e3::<F>,
        },
        Case {
            id: "ENTROPY-4",
            says: "a fill whose answer is lost after its bytes were drawn is refused Lost, and asked again it draws other bytes",
            check: e4::<F>,
        },
        Case {
            id: "ENTROPY-5",
            says: "every refusal the source gives reaches its caller as it is, after one attempt: the seam never asks again",
            check: e5::<F>,
        },
    ]
}
