//! `Secrets`: a named credential, leased and never stored (plan RQ0.4; architecture D16, D25):
//! an upstream registry's credential for a pull-through cache (RQ8.6), a signing lease (RQ7.10).
//! Veil is the real one, and has no `-api` crate yet (`docs/design/plan/README.md`, "What the plan
//! asks of other owners"), so the trait is Reliquary's own; [`SimSecrets`] is Veil's side in
//! memory.
//!
//! A lease gives the credential's value and when the lease ends, by the store's wall clock. The
//! value is a [`Credential`]: it has no `Clone` and no serialization, its `Debug` says only its
//! length, and its bytes are overwritten when it is dropped, so a holder keeps it only as long as
//! it holds the lease and never writes it anywhere. A lease whose answer never reached its holder
//! is the store's to end, by its end, so a lost answer leaks nothing.

use std::collections::BTreeMap;
use std::fmt;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};

use parking_lot::Mutex;
use zeroize::Zeroizing;

use crate::seams::BoxFuture;
use crate::seams::clock::{self, Clock, SimClock};
use crate::seams::conformance::{Case, Failure, Fixture, ensure, fine, refused, same};
use crate::seams::fault::{
    self, CUT, Counts, DELAY, Fault, Faults, Kind, LOSE, Unanswered, Unarmed, VANISH,
};

/// Named credentials, leased.
pub trait Secrets: Send + Sync + 'static {
    /// Leases the credential `name`: its value, and when the lease ends.
    fn lease<'a>(
        &'a self,
        name: &'a str,
        deadline: Instant,
    ) -> BoxFuture<'a, Result<Lease, Refusal>>;
    /// Gives lease `id` back before its end.
    fn release<'a>(
        &'a self,
        id: &'a LeaseId,
        deadline: Instant,
    ) -> BoxFuture<'a, Result<Released, Refusal>>;
}

/// A credential's bytes. It is never copied, said or kept: no `Clone`, no serialization, a `Debug`
/// that says only its length, and its bytes overwritten when it is dropped.
pub struct Credential(Zeroizing<Vec<u8>>);

impl Credential {
    #[must_use]
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(Zeroizing::new(bytes))
    }

    /// The bytes, for the one call that presents them.
    #[must_use]
    pub fn expose(&self) -> &[u8] {
        &self.0
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Debug for Credential {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Credential(<{} bytes, never shown>)", self.0.len())
    }
}

/// A lease's name, as the store gave it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LeaseId(pub String);

/// A credential leased: its value, until `ends` by the store's wall clock.
#[derive(Debug)]
pub struct Lease {
    pub id: LeaseId,
    pub name: String,
    pub value: Credential,
    pub ends: SystemTime,
}

/// What a release did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Released {
    Released,
    /// The lease was not held: given back already, or ended.
    NotHeld,
}

/// Why a call was refused, or ended without its answer.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Refusal {
    #[error("no credential {0:?}")]
    NotFound(String),
    /// The holder may not lease it.
    #[error("leasing {0:?} is not this holder's to do")]
    Denied(String),
    /// The store holds its most leases at once.
    #[error("the store holds its most leases ({limit})")]
    Full { limit: u64 },
    /// The store admits no more work now, with when to ask again if it says.
    #[error("the store is busy{}", retry_after.map(|d| format!("; ask again after {d:?}")).unwrap_or_default())]
    Busy { retry_after: Option<Duration> },
    /// The answer was cut at byte `at` of its value: no value is given, never a shorter one.
    #[error("the answer was cut at byte {at} of its value")]
    Cut { at: u64 },
    #[error("the deadline passed before the call was answered")]
    Late,
    #[error("the answer was lost: the call may have applied")]
    Lost,
    #[error("the store is unavailable: {0}")]
    Unavailable(String),
}

impl Kind for Refusal {
    fn kind(&self) -> &'static str {
        match self {
            Self::NotFound(_) => "NotFound",
            Self::Denied(_) => "Denied",
            Self::Full { .. } => "Full",
            Self::Busy { .. } => "Busy",
            Self::Cut { .. } => "Cut",
            Self::Late => "Late",
            Self::Lost => "Lost",
            Self::Unavailable(_) => "Unavailable",
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
        Self::Unavailable(format!("the store's clock: {refusal}"))
    }
}

/// The faults a secrets simulator injects (`docs/design/seams.md`, "Secrets"): each refusal, an
/// answer delayed past its caller's deadline, a request that never arrives, an answer lost after
/// its lease was made, and a lease's value cut at any byte.
pub const FAULTS: &[&str] = &[
    "NotFound",
    "Denied",
    "Full",
    "Busy",
    "Cut",
    "Late",
    "Lost",
    "Unavailable",
    DELAY,
    VANISH,
    LOSE,
    CUT,
];

/// The operations a fault can be armed for: each method of [`Secrets`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Op {
    Lease,
    Release,
}

fn fits(op: Op, fault: &Fault<Refusal>) -> bool {
    !matches!(fault, Fault::Cut(_)) || op == Op::Lease
}

/// What a secrets simulator holds at most, and how long its leases last, as its maker states.
#[derive(Debug, Clone, Copy)]
pub struct SimLimits {
    /// Leases held at once.
    pub leases: usize,
    /// A lease's life.
    pub life: Duration,
}

struct Vault {
    /// Each credential's bytes, and whether its holder may lease it.
    credentials: BTreeMap<String, (Zeroizing<Vec<u8>>, bool)>,
    /// Leases held, by when each ends.
    leases: BTreeMap<LeaseId, (String, SystemTime)>,
    next: u64,
    faults: Faults<Op, Refusal>,
}

impl Vault {
    /// Lets go every lease whose end has come.
    fn end_leases(&mut self, now: SystemTime) {
        self.leases.retain(|_, (_, ends)| *ends > now);
    }

    fn lease(&mut self, name: &str, now: SystemTime, limits: SimLimits) -> Result<Lease, Refusal> {
        self.end_leases(now);
        let (value, allowed) = self
            .credentials
            .get(name)
            .ok_or_else(|| Refusal::NotFound(name.to_owned()))?;
        if !*allowed {
            return Err(Refusal::Denied(name.to_owned()));
        }
        let value = Credential::new(value.to_vec());
        if self.leases.len() >= limits.leases {
            return Err(Refusal::Full {
                limit: u64::try_from(limits.leases).unwrap_or(u64::MAX),
            });
        }
        let ends = now.checked_add(limits.life).ok_or_else(|| {
            Refusal::Unavailable("a lease's end past what the platform can say".into())
        })?;
        let n = self.next;
        self.next = n.checked_add(1).ok_or(Refusal::Full { limit: u64::MAX })?;
        // Its name is as long as every other's, so taking leases grows nothing.
        let id = LeaseId(format!("lease-{n:016x}"));
        self.leases.insert(id.clone(), (name.to_owned(), ends));
        // An armed cut within the value: the lease is made, and its answer never whole.
        let length = u64::try_from(value.len()).unwrap_or(u64::MAX);
        if let Some(at) = self.faults.cut(Op::Lease, 0, length) {
            return Err(Refusal::Cut { at });
        }
        Ok(Lease {
            id,
            name: name.to_owned(),
            value,
            ends,
        })
    }
}

/// Veil's side, in memory: credentials kept by name, and the leases held on them.
pub struct SimSecrets {
    clock: Arc<dyn Clock>,
    limits: SimLimits,
    vault: Mutex<Vault>,
}

impl SimSecrets {
    /// Credentials on `clock`, holding what `limits` says.
    #[must_use]
    pub fn new(clock: Arc<dyn Clock>, limits: SimLimits) -> Self {
        Self {
            clock,
            limits,
            vault: Mutex::new(Vault {
                credentials: BTreeMap::new(),
                leases: BTreeMap::new(),
                next: 0,
                faults: Faults::default(),
            }),
        }
    }

    /// Keeps the credential `name`, which its holder may lease or not.
    pub fn keep(&self, name: &str, value: &[u8], allowed: bool) {
        self.vault
            .lock()
            .credentials
            .insert(name.to_owned(), (Zeroizing::new(value.to_vec()), allowed));
    }

    /// Arms `fault` for the next call of `op` (a cut, for the next lease whose value reaches its
    /// byte).
    pub fn arm(&self, op: Op, fault: Fault<Refusal>) -> Result<(), Unarmed> {
        self.vault.lock().faults.arm(op, fault, fits)
    }

    /// How many calls of `op` were made.
    #[must_use]
    pub fn attempts(&self, op: Op) -> u64 {
        self.vault.lock().faults.attempts(op)
    }

    /// What it injected, by kind.
    #[must_use]
    pub fn injected(&self) -> Counts {
        self.vault.lock().faults.injected().clone()
    }

    /// The life its leases have.
    #[must_use]
    pub fn limits(&self) -> SimLimits {
        self.limits
    }
}

impl Secrets for SimSecrets {
    fn lease<'a>(
        &'a self,
        name: &'a str,
        deadline: Instant,
    ) -> BoxFuture<'a, Result<Lease, Refusal>> {
        Box::pin(async move {
            let fault = self.vault.lock().faults.call(Op::Lease);
            fault::answer(self.clock.as_ref(), fault, deadline, || {
                self.vault
                    .lock()
                    .lease(name, self.clock.wall(), self.limits)
            })
            .await
        })
    }

    fn release<'a>(
        &'a self,
        id: &'a LeaseId,
        deadline: Instant,
    ) -> BoxFuture<'a, Result<Released, Refusal>> {
        Box::pin(async move {
            let fault = self.vault.lock().faults.call(Op::Release);
            fault::answer(self.clock.as_ref(), fault, deadline, || {
                let mut vault = self.vault.lock();
                vault.end_leases(self.clock.wall());
                Ok(match vault.leases.remove(id) {
                    Some(_) => Released::Released,
                    None => Released::NotHeld,
                })
            })
            .await
        })
    }
}

// -------------------------------------------------------------------------------- conformance

/// What a secrets store's conformance suite runs against.
pub trait SecretsFixture: Fixture {
    fn secrets(&self) -> &dyn Secrets;
    /// The clock the store's deadlines and leases' ends are kept by.
    fn clock(&self) -> &dyn Clock;
    /// Lets `d` pass by the store's clock.
    fn pass(&self, d: Duration) -> Result<(), Failure>;
    /// Arms `fault` for the store's next call of `op`.
    fn arm(&self, op: Op, fault: Fault<Refusal>) -> Result<(), Failure>;
    /// How many calls of `op` the store has had.
    fn attempts(&self, op: Op) -> u64;
    /// How long a case's caller waits for an answer: each deadline is the clock's now plus this.
    fn patience(&self) -> Duration;
    /// A credential its holder may lease, and its value.
    fn credential(&self) -> (&str, &[u8]);
    /// A credential its holder may not lease.
    fn denied(&self) -> &str;
    /// The most leases the store holds at once.
    fn leases_max(&self) -> usize;
    /// How long the store's leases last.
    fn life(&self) -> Duration;
}

/// The fixture for a [`SimSecrets`]: the clock it was made with, a credential it keeps and one it
/// denies, and what the test that makes it states.
pub struct SimSecretsFixture {
    pub secrets: SimSecrets,
    pub clock: SimClock,
    pub patience: Duration,
    pub steps: u64,
    pub name: String,
    pub value: Vec<u8>,
    pub denied: String,
}

impl Fixture for SimSecretsFixture {
    fn drive<'a, T>(&'a self, work: BoxFuture<'a, T>) -> Result<T, Failure> {
        self.clock
            .run(work, self.steps)
            .map_err(|stalled| stalled.to_string())
    }

    fn injected(&self) -> Counts {
        self.secrets.injected()
    }
}

impl SecretsFixture for SimSecretsFixture {
    fn secrets(&self) -> &dyn Secrets {
        &self.secrets
    }

    fn clock(&self) -> &dyn Clock {
        &self.clock
    }

    fn pass(&self, d: Duration) -> Result<(), Failure> {
        self.clock.advance(d).map_err(|e| e.to_string())
    }

    fn arm(&self, op: Op, fault: Fault<Refusal>) -> Result<(), Failure> {
        self.secrets.arm(op, fault).map_err(|e| e.to_string())
    }

    fn attempts(&self, op: Op) -> u64 {
        self.secrets.attempts(op)
    }

    fn patience(&self) -> Duration {
        self.patience
    }

    fn credential(&self) -> (&str, &[u8]) {
        (&self.name, &self.value)
    }

    fn denied(&self) -> &str {
        &self.denied
    }

    fn leases_max(&self) -> usize {
        self.secrets.limits().leases
    }

    fn life(&self) -> Duration {
        self.secrets.limits().life
    }
}

fn deadline<F: SecretsFixture>(f: &F) -> Result<Instant, Failure> {
    f.clock()
        .now()
        .checked_add(f.patience())
        .ok_or_else(|| "a deadline past what the platform's time can say".to_owned())
}

/// Leases up to the store's most, then shows the next is refused `Full`: whether the store holds
/// `already` leases its caller does not know of. The leases taken are given back.
async fn at_most<F: SecretsFixture>(f: &F, already: usize) -> Result<(), Failure> {
    let (name, _) = f.credential();
    let mut held = Vec::new();
    for _ in already..f.leases_max() {
        held.push(fine(
            "a lease within the bound",
            f.secrets().lease(name, deadline(f)?).await,
        )?);
    }
    let refusal = refused(
        "a lease past the most held",
        f.secrets().lease(name, deadline(f)?).await,
    )?;
    ensure(matches!(refusal, Refusal::Full { .. }), || {
        format!("refused {refusal:?}, not Full")
    })?;
    for lease in held {
        same(
            "a lease given back",
            fine(
                "a release",
                f.secrets().release(&lease.id, deadline(f)?).await,
            )?,
            Released::Released,
        )?;
    }
    Ok(())
}

fn s1<F: SecretsFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        let (name, value) = f.credential();
        let lease = fine("a lease", f.secrets().lease(name, deadline(f)?).await)?;
        same("its name", lease.name.as_str(), name)?;
        same("its value", lease.value.expose(), value)?;
        ensure(lease.ends > f.clock().wall(), || {
            "a lease that has ended already".into()
        })
    })
}

fn s2<F: SecretsFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        let unknown = "no-such-credential";
        same(
            "a lease of a credential the store does not keep",
            f.secrets().lease(unknown, deadline(f)?).await.map(|l| l.id),
            Err(Refusal::NotFound(unknown.into())),
        )?;
        same(
            "a lease its holder may not take",
            f.secrets()
                .lease(f.denied(), deadline(f)?)
                .await
                .map(|l| l.id),
            Err(Refusal::Denied(f.denied().into())),
        )
    })
}

fn s3<F: SecretsFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        let (name, value) = f.credential();
        let lease = fine("a lease", f.secrets().lease(name, deadline(f)?).await)?;
        let said = format!("{lease:?}");
        let text = String::from_utf8_lossy(value);
        ensure(!value.is_empty() && !said.contains(text.as_ref()), || {
            "a lease's Debug shows its value".into()
        })?;
        ensure(said.contains(&lease.value.len().to_string()), || {
            "a credential's Debug does not say its length".into()
        })
    })
}

fn s4<F: SecretsFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        let (name, _) = f.credential();
        let lease = fine("a lease", f.secrets().lease(name, deadline(f)?).await)?;
        same(
            "a lease given back",
            fine(
                "a release",
                f.secrets().release(&lease.id, deadline(f)?).await,
            )?,
            Released::Released,
        )?;
        same(
            "a lease given back again",
            fine(
                "a release",
                f.secrets().release(&lease.id, deadline(f)?).await,
            )?,
            Released::NotHeld,
        )
    })
}

fn s5<F: SecretsFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        at_most(f, 0).await?;
        // Every lease the bound held was given back: one can be taken again.
        let (name, _) = f.credential();
        fine(
            "a lease once others were given back",
            f.secrets().lease(name, deadline(f)?).await,
        )
        .map(drop)
    })
}

fn s6<F: SecretsFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        let (name, _) = f.credential();
        f.arm(Op::Lease, Fault::Lose)?;
        same(
            "a lease whose answer is lost",
            f.secrets().lease(name, deadline(f)?).await.map(|l| l.id),
            Err(Refusal::Lost),
        )?;
        // The lease its caller never learned of is held until its end, and no longer.
        at_most(f, 1).await?;
        f.pass(f.life())?;
        at_most(f, 0).await
    })
}

fn s7<F: SecretsFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        let (name, _) = f.credential();
        let past = f
            .patience()
            .checked_add(f.patience())
            .ok_or_else(|| "a patience as long as time".to_owned())?;
        f.arm(Op::Lease, Fault::Delay(past))?;
        let due = deadline(f)?;
        same(
            "a lease whose answer comes after the deadline",
            f.secrets().lease(name, due).await.map(|l| l.id),
            Err(Refusal::Late),
        )?;
        ensure(f.clock().now() >= due, || {
            "told late before its deadline".into()
        })?;
        let lease = fine("a lease", f.secrets().lease(name, deadline(f)?).await)?;
        f.arm(Op::Release, Fault::Vanish)?;
        same(
            "a release that never arrives",
            f.secrets().release(&lease.id, deadline(f)?).await,
            Err(Refusal::Late),
        )?;
        same(
            "the release asked again",
            fine(
                "a release",
                f.secrets().release(&lease.id, deadline(f)?).await,
            )?,
            Released::Released,
        )
    })
}

fn s8<F: SecretsFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        let (name, value) = f.credential();
        let length = u64::try_from(value.len()).unwrap_or(u64::MAX);
        for at in 0..=length {
            f.arm(Op::Lease, Fault::Cut(at))?;
            same(
                &format!("a lease cut at byte {at} of its value"),
                f.secrets().lease(name, deadline(f)?).await.map(|l| l.id),
                Err(Refusal::Cut { at }),
            )?;
            // The cut lease is held until its end, like any whose answer was lost.
            f.pass(f.life())?;
        }
        Ok(())
    })
}

/// One refusal of every kind the store gives.
fn every_refusal<F: SecretsFixture>(f: &F) -> Vec<Refusal> {
    vec![
        Refusal::NotFound("n".into()),
        Refusal::Denied("n".into()),
        Refusal::Full { limit: 0 },
        Refusal::Busy {
            retry_after: Some(f.patience()),
        },
        Refusal::Cut { at: 0 },
        Refusal::Late,
        Refusal::Lost,
        Refusal::Unavailable("u".into()),
    ]
}

fn s9<F: SecretsFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        let (name, _) = f.credential();
        for refusal in every_refusal(f) {
            let before = f.attempts(Op::Lease);
            f.arm(Op::Lease, Fault::Refuse(refusal.clone()))?;
            same(
                "a lease the store refuses",
                f.secrets().lease(name, deadline(f)?).await.map(|l| l.id),
                Err(refusal.clone()),
            )?;
            same(
                &format!("attempts of a lease refused {refusal:?}"),
                f.attempts(Op::Lease),
                before.saturating_add(1),
            )?;
        }
        Ok(())
    })
}

/// The `Secrets` contract, clause by clause (`docs/design/seams.md`, "Secrets").
#[must_use]
pub fn cases<F: SecretsFixture>() -> Vec<Case<F>> {
    vec![
        Case {
            id: "SECRETS-1",
            says: "a lease of a named credential gives its value, and an end after now",
            check: s1::<F>,
        },
        Case {
            id: "SECRETS-2",
            says: "a credential the store does not keep is refused NotFound; one its holder may not lease, Denied",
            check: s2::<F>,
        },
        Case {
            id: "SECRETS-3",
            says: "a credential's value never shows: its Debug says only its length",
            check: s3::<F>,
        },
        Case {
            id: "SECRETS-4",
            says: "a lease given back is no longer held: given back again, it is answered NotHeld",
            check: s4::<F>,
        },
        Case {
            id: "SECRETS-5",
            says: "the leases a store holds are bounded: past the most, a lease is refused Full, and none held is dropped",
            check: s5::<F>,
        },
        Case {
            id: "SECRETS-6",
            says: "a lease whose answer is lost is held until its end, and no longer: a lost answer leaks nothing",
            check: s6::<F>,
        },
        Case {
            id: "SECRETS-7",
            says: "an answer delayed past its caller's deadline is refused Late at the deadline, and a call that never arrived can be asked again",
            check: s7::<F>,
        },
        Case {
            id: "SECRETS-8",
            says: "a lease whose value is cut at any byte is refused Cut, never given as a shorter value",
            check: s8::<F>,
        },
        Case {
            id: "SECRETS-9",
            says: "every refusal the store gives reaches its caller as it is, after one attempt: the seam never asks again",
            check: s9::<F>,
        },
    ]
}
