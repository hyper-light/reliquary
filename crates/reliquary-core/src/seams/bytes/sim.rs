//! [`SimBytes`]: the bytes simulator, and Mantle's side of a grant ([`SimBytes::redeem`]).
//!
//! It keeps objects' and uploads' bytes in a [`Store`] its maker gives it ([`MemStore`] in memory;
//! a test of objects larger than memory gives one that keeps them elsewhere), and their facts in
//! memory, bounded by what its maker states ([`SimLimits`]): past a bound it refuses, typed. Its
//! time is its clock's, and its grants' key comes from its maker's entropy, so a run is a function
//! of its seed.

use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime};

use aws_lc_rs::hmac;
use parking_lot::Mutex;

use super::{
    Access, Body, Bytes, Deleted, Grant, Hasher, Meta, Method, Op, Plan, Progress, Put, Reading,
    Redeemed, Refusal, Upload, UploadId, Whole, signed,
};
use crate::seams::BoxFuture;
use crate::seams::clock::{Clock, SimClock};
use crate::seams::conformance::{Failure, Fixture};
use crate::seams::fault::{self, Counts, Fault, Faults, Unarmed};
use crate::seams::range::{self, Range, Span};

use super::suite::BytesFixture;

/// Where a bytes simulator keeps bodies' bytes: each body numbered by the simulator, appended to,
/// cut back, read from anywhere, and removed.
pub trait Store: Send + Sync + 'static {
    /// Appends `bytes` to body `body`, which a first append makes.
    fn append(&self, body: u64, bytes: &[u8]) -> Result<(), String>;
    /// Cuts body `body` back to its first `length` bytes.
    fn truncate(&self, body: u64, length: u64) -> Result<(), String>;
    /// Reads body `body` from byte `at` into `into`: how many bytes, zero at its end.
    fn read_at(&self, body: u64, at: u64, into: &mut [u8]) -> Result<usize, String>;
    /// Removes body `body`; one never made is removed already.
    fn remove(&self, body: u64) -> Result<(), String>;
}

/// Bodies in memory.
#[derive(Default)]
pub struct MemStore {
    bodies: Mutex<BTreeMap<u64, Vec<u8>>>,
}

impl Store for MemStore {
    fn append(&self, body: u64, bytes: &[u8]) -> Result<(), String> {
        let mut bodies = self.bodies.lock();
        let kept = bodies.entry(body).or_default();
        kept.try_reserve(bytes.len()).map_err(|e| e.to_string())?;
        kept.extend_from_slice(bytes);
        Ok(())
    }

    fn truncate(&self, body: u64, length: u64) -> Result<(), String> {
        if let Some(kept) = self.bodies.lock().get_mut(&body) {
            kept.truncate(usize::try_from(length).unwrap_or(usize::MAX));
        }
        Ok(())
    }

    fn read_at(&self, body: u64, at: u64, into: &mut [u8]) -> Result<usize, String> {
        let bodies = self.bodies.lock();
        let kept = bodies.get(&body).map_or(&[][..], Vec::as_slice);
        let rest = usize::try_from(at)
            .ok()
            .and_then(|at| kept.get(at..))
            .unwrap_or(&[]);
        let mut n = 0usize;
        for (to, from) in into.iter_mut().zip(rest) {
            *to = *from;
            n = n.saturating_add(1);
        }
        Ok(n)
    }

    fn remove(&self, body: u64) -> Result<(), String> {
        self.bodies.lock().remove(&body);
        Ok(())
    }
}

/// What a bytes simulator holds at most, and the plan it gives every upload, as its maker states.
#[derive(Debug, Clone, Copy)]
pub struct SimLimits {
    pub plan: Plan,
    /// Objects kept.
    pub objects: usize,
    /// Uploads open at once.
    pub uploads: usize,
    /// Bytes kept, objects' and open uploads' together.
    pub bytes: u64,
    /// The longest key, in bytes.
    pub key: usize,
}

/// A body in the store, removed from it once nothing holds it: neither its object nor its upload,
/// nor a reading of it.
struct Held {
    id: u64,
    store: Arc<dyn Store>,
    /// Removals the store refused, counted rather than dropped unseen.
    refused: Arc<AtomicU64>,
}

impl Drop for Held {
    fn drop(&mut self) {
        if self.store.remove(self.id).is_err() {
            self.refused.fetch_add(1, Ordering::Relaxed);
        }
    }
}

struct Object {
    meta: Meta,
    body: Arc<Held>,
}

struct Open {
    key: String,
    put: Put,
    /// A granted put's exact length.
    exact: Option<u64>,
    body: Arc<Held>,
    progress: Progress,
    /// The SHA-256 of what was received, and of what is durable, to go back to after a cut.
    hash: Hasher,
    durable_hash: Hasher,
}

struct State {
    objects: BTreeMap<String, Object>,
    uploads: BTreeMap<UploadId, Open>,
    /// Open uploads by the identity of the operation that began them.
    begun: BTreeMap<u128, UploadId>,
    next: u64,
    /// Bytes kept: objects' lengths and open uploads' received bytes.
    kept: u64,
    /// The latest wall time the store has read. A grant's expiry is judged against it, so a wall
    /// clock stepped back never brings back a grant the store has refused as expired (plan
    /// RQ0.6, T-RQ0.6.6; docs/design/seams.md S6).
    latest_wall: SystemTime,
    faults: Faults<Op, Refusal>,
}

struct Shared {
    clock: Arc<dyn Clock>,
    store: Arc<dyn Store>,
    key: hmac::Key,
    limits: SimLimits,
    refused: Arc<AtomicU64>,
    state: Mutex<State>,
}

/// Objects in a store its maker gives it, their facts in memory. Clones share them.
#[derive(Clone)]
pub struct SimBytes {
    shared: Arc<Shared>,
}

fn count(n: usize) -> u64 {
    u64::try_from(n).unwrap_or(u64::MAX)
}

fn unavailable(what: &str, why: impl std::fmt::Display) -> Refusal {
    Refusal::Unavailable(format!("{what}: {why}"))
}

fn fits(op: Op, fault: &Fault<Refusal>) -> bool {
    match fault {
        Fault::Cut(_) => matches!(op, Op::Send | Op::Read),
        Fault::Lose => !matches!(op, Op::Read),
        Fault::Refuse(_) | Fault::Delay(_) | Fault::Vanish => true,
    }
}

fn check_key(key: &str, longest: usize) -> Result<(), Refusal> {
    if key.is_empty() || key.len() > longest {
        return Err(Refusal::Invalid {
            what: "a key",
            why: format!("it is {} bytes: a key is 1 to {longest}", key.len()),
        });
    }
    Ok(())
}

impl State {
    /// The store's wall clock as grants are judged by it: the latest it has read, never earlier.
    fn wall(&mut self, clock: &dyn Clock) -> SystemTime {
        self.latest_wall = self.latest_wall.max(clock.wall());
        self.latest_wall
    }

    fn take_next(&mut self) -> Result<u64, Refusal> {
        let n = self.next;
        self.next = n.checked_add(1).ok_or(Refusal::Full {
            what: "names",
            limit: u64::MAX,
        })?;
        Ok(n)
    }

    fn open(
        &mut self,
        shared: &Shared,
        key: &str,
        put: Put,
        exact: Option<u64>,
    ) -> Result<Upload, Refusal> {
        check_key(key, shared.limits.key)?;
        if let Some(id) = self.begun.get(&put.identity)
            && let Some(open) = self.uploads.get(id)
        {
            // Asked again with its identity: answered with the upload it made.
            return Ok(Upload {
                id: id.clone(),
                plan: shared.limits.plan,
                progress: open.progress,
            });
        }
        if put.create_once && self.objects.contains_key(key) {
            return Err(Refusal::Exists(key.to_owned()));
        }
        if self.uploads.len() >= shared.limits.uploads {
            return Err(Refusal::Full {
                what: "uploads",
                limit: count(shared.limits.uploads),
            });
        }
        let n = self.take_next()?;
        // Its name is as long as every other's, so opening uploads grows nothing.
        let id = UploadId(format!("upload-{n:016x}"));
        let body = Arc::new(Held {
            id: n,
            store: Arc::clone(&shared.store),
            refused: Arc::clone(&shared.refused),
        });
        self.uploads.insert(
            id.clone(),
            Open {
                key: key.to_owned(),
                put,
                exact,
                body,
                progress: Progress::default(),
                hash: Hasher::default(),
                durable_hash: Hasher::default(),
            },
        );
        self.begun.insert(put.identity, id.clone());
        Ok(Upload {
            id,
            plan: shared.limits.plan,
            progress: Progress::default(),
        })
    }

    fn send(
        &mut self,
        shared: &Shared,
        upload: &UploadId,
        at: u64,
        piece: &[u8],
    ) -> Result<Progress, Refusal> {
        let plan = shared.limits.plan;
        let open = self
            .uploads
            .get_mut(upload)
            .ok_or_else(|| Refusal::NoUpload(upload.0.clone()))?;
        let length = count(piece.len());
        if piece.len() > plan.piece {
            return Err(Refusal::TooLarge {
                what: "a piece",
                size: length,
                limit: count(plan.piece),
            });
        }
        if at != open.progress.received {
            return Err(Refusal::Offset {
                expected: open.progress.received,
            });
        }
        let end = at.checked_add(length).ok_or(Refusal::TooLarge {
            what: "an upload",
            size: u64::MAX,
            limit: u64::MAX,
        })?;
        if let Some(exact) = open.exact
            && end > exact
        {
            return Err(Refusal::TooLarge {
                what: "a granted put's body",
                size: end,
                limit: exact,
            });
        }
        // A sum past u64::MAX is past any limit: saturating says so.
        let kept = self.kept.saturating_add(length);
        if kept > shared.limits.bytes {
            return Err(Refusal::Full {
                what: "bytes",
                limit: shared.limits.bytes,
            });
        }
        // An armed cut within this piece: what came before it arrives, nothing after.
        let cut = self.faults.cut(Op::Send, at, end);
        let taken_end = cut.unwrap_or(end);
        let taken = taken_end.saturating_sub(at);
        let (bytes, _) = piece
            .split_at_checked(usize::try_from(taken).unwrap_or(usize::MAX))
            .ok_or_else(|| unavailable("a piece", "shorter than it says"))?;
        shared
            .store
            .append(open.body.id, bytes)
            .map_err(|e| unavailable("the store", e))?;
        // Runs end at multiples of the plan's run: the bytes up to the last one ended are durable.
        let durable = taken_end
            .checked_div(plan.run)
            .and_then(|runs| runs.checked_mul(plan.run))
            .unwrap_or(open.progress.durable);
        if durable > open.progress.durable {
            // `durable` is past `at`: the durable offset was the last run's end at or before `at`.
            let (before, after) = bytes
                .split_at_checked(usize::try_from(durable.saturating_sub(at)).unwrap_or(usize::MAX))
                .ok_or_else(|| unavailable("a run", "past the piece it ends in"))?;
            open.hash.update(before);
            open.durable_hash = open.hash.clone();
            open.hash.update(after);
            open.progress.durable = durable;
        } else {
            open.hash.update(bytes);
        }
        open.progress.received = taken_end;
        self.kept = self.kept.saturating_add(taken);
        if cut.is_none() {
            return Ok(open.progress);
        }
        // The stream was cut: what had not become durable is gone.
        shared
            .store
            .truncate(open.body.id, open.progress.durable)
            .map_err(|e| unavailable("the store", e))?;
        let gone = open.progress.received.saturating_sub(open.progress.durable);
        self.kept = self.kept.saturating_sub(gone);
        open.hash = open.durable_hash.clone();
        open.progress.received = open.progress.durable;
        Err(Refusal::Cut {
            at: open.progress.durable,
        })
    }

    fn finish(
        &mut self,
        shared: &Shared,
        upload: &UploadId,
        whole: Whole,
    ) -> Result<Meta, Refusal> {
        let open = self
            .uploads
            .get(upload)
            .ok_or_else(|| Refusal::NoUpload(upload.0.clone()))?;
        if open.progress.received != whole.length {
            return Err(Refusal::Mismatch(format!(
                "the upload received {} bytes; the finish says {}",
                open.progress.received, whole.length
            )));
        }
        if let Some(exact) = open.exact
            && exact != whole.length
        {
            return Err(Refusal::Mismatch(format!(
                "a granted put of exactly {exact} bytes"
            )));
        }
        let sha256 = open.hash.clone().finish();
        if sha256 != whole.sha256 {
            return Err(Refusal::Mismatch(format!(
                "the upload's SHA-256 is {sha256:?}; the finish says {:?}",
                whole.sha256
            )));
        }
        let there = self.objects.contains_key(&open.key);
        if open.put.create_once && there {
            return Err(Refusal::Exists(open.key.clone()));
        }
        if !there && self.objects.len() >= shared.limits.objects {
            return Err(Refusal::Full {
                what: "objects",
                limit: count(shared.limits.objects),
            });
        }
        let open = self
            .uploads
            .remove(upload)
            .ok_or_else(|| Refusal::NoUpload(upload.0.clone()))?;
        self.begun.remove(&open.put.identity);
        let meta = Meta {
            key: open.key.clone(),
            length: whole.length,
            sha256,
        };
        let object = Object {
            meta: meta.clone(),
            body: open.body,
        };
        if let Some(old) = self.objects.insert(open.key, object) {
            self.kept = self.kept.saturating_sub(old.meta.length);
        }
        Ok(meta)
    }

    fn abort(&mut self, upload: &UploadId) -> Result<(), Refusal> {
        let open = self
            .uploads
            .remove(upload)
            .ok_or_else(|| Refusal::NoUpload(upload.0.clone()))?;
        self.begun.remove(&open.put.identity);
        self.kept = self.kept.saturating_sub(open.progress.received);
        Ok(())
    }

    fn reading(
        &self,
        shared: &Arc<Shared>,
        key: &str,
        range: Option<Range>,
    ) -> Result<Reading, Refusal> {
        check_key(key, shared.limits.key)?;
        let object = self
            .objects
            .get(key)
            .ok_or_else(|| Refusal::NotFound(key.to_owned()))?;
        let span = match range {
            Some(range) => range::resolve(range, object.meta.length)?,
            None => Span::whole(object.meta.length),
        };
        Ok(Reading {
            meta: object.meta.clone(),
            span,
            body: Box::new(SimBody {
                shared: Arc::clone(shared),
                held: Arc::clone(&object.body),
                at: span.first,
                end: span.end(),
                ended: None,
            }),
        })
    }
}

/// How a call answers, with the fault armed for `op` ([`fault::answer`]): `apply` takes the
/// simulator's lock, makes the call's effect, and gives its answer.
async fn answer<T: Send>(
    shared: &Shared,
    op: Op,
    deadline: Instant,
    apply: impl FnOnce(&mut State) -> Result<T, Refusal> + Send,
) -> Result<T, Refusal> {
    let fault = shared.state.lock().faults.call(op);
    fault::answer(shared.clock.as_ref(), fault, deadline, || {
        apply(&mut shared.state.lock())
    })
    .await
}

impl SimBytes {
    /// A store of objects in `store`, its time `clock`'s, its grants signed with `grant_key` (which
    /// its maker draws from its entropy), holding what `limits` says.
    pub fn new(
        clock: Arc<dyn Clock>,
        grant_key: &[u8],
        store: Arc<dyn Store>,
        limits: SimLimits,
    ) -> Result<Self, Refusal> {
        if limits.plan.piece == 0 || limits.plan.run == 0 {
            return Err(Refusal::Invalid {
                what: "a plan",
                why: "a piece and a run each take at least a byte".into(),
            });
        }
        let state = State {
            objects: BTreeMap::new(),
            uploads: BTreeMap::new(),
            begun: BTreeMap::new(),
            next: 0,
            kept: 0,
            latest_wall: SystemTime::UNIX_EPOCH,
            faults: Faults::default(),
        };
        Ok(Self {
            shared: Arc::new(Shared {
                clock,
                store,
                key: hmac::Key::new(hmac::HMAC_SHA256, grant_key),
                limits,
                refused: Arc::new(AtomicU64::new(0)),
                state: Mutex::new(state),
            }),
        })
    }

    /// A store of objects in memory ([`MemStore`]): [`SimBytes::new`] with one.
    pub fn in_memory(
        clock: Arc<dyn Clock>,
        grant_key: &[u8],
        limits: SimLimits,
    ) -> Result<Self, Refusal> {
        Self::new(clock, grant_key, Arc::new(MemStore::default()), limits)
    }

    /// Arms `fault` for the next call of `op` (a cut, for the next that reaches its byte).
    pub fn arm(&self, op: Op, fault: Fault<Refusal>) -> Result<(), Unarmed> {
        self.shared.state.lock().faults.arm(op, fault, fits)
    }

    /// How many calls of `op` were made.
    #[must_use]
    pub fn attempts(&self, op: Op) -> u64 {
        self.shared.state.lock().faults.attempts(op)
    }

    /// What it injected, by kind.
    #[must_use]
    pub fn injected(&self) -> Counts {
        self.shared.state.lock().faults.injected().clone()
    }

    /// Bodies whose removal the store refused.
    #[must_use]
    pub fn removals_refused(&self) -> u64 {
        self.shared.refused.load(Ordering::Relaxed)
    }

    /// What it holds at most, and the plan it gives.
    #[must_use]
    pub fn limits(&self) -> SimLimits {
        self.shared.limits
    }

    /// Uploads open now.
    #[must_use]
    pub fn uploads_open(&self) -> usize {
        self.shared.state.lock().uploads.len()
    }

    /// Mantle's side of a grant: `holder` presents `grant` to do what it names. A grant whose tag
    /// does not hold for its fields, or presented by another than its holder, is refused
    /// [`Refusal::Forbidden`]; one past its expiry by this store's wall clock,
    /// [`Refusal::Expired`]. A get's is a reading of the object, which must still be the grant's
    /// length; a put's, an upload of exactly the grant's length.
    pub fn redeem<'a>(
        &'a self,
        grant: &'a Grant,
        holder: &'a str,
        deadline: Instant,
    ) -> BoxFuture<'a, Result<Redeemed, Refusal>> {
        let shared = Arc::clone(&self.shared);
        Box::pin(answer(&self.shared, Op::Redeem, deadline, move |st| {
            let message = signed(&grant.key, grant.access, &grant.holder, grant.expires)?;
            hmac::verify(&shared.key, &message, &grant.tag)
                .map_err(|_| Refusal::Forbidden("its tag does not hold for what it says".into()))?;
            if holder != grant.holder {
                return Err(Refusal::Forbidden(format!(
                    "it is {:?}'s, presented by {holder:?}",
                    grant.holder
                )));
            }
            if st.wall(shared.clock.as_ref()) >= grant.expires {
                return Err(Refusal::Expired);
            }
            match grant.access.method {
                Method::Get => {
                    let reading = st.reading(&shared, &grant.key, None)?;
                    if reading.meta.length != grant.access.length {
                        return Err(Refusal::Mismatch(format!(
                            "the object is {} bytes; the grant is for {}",
                            reading.meta.length, grant.access.length
                        )));
                    }
                    Ok(Redeemed::Reading(reading))
                }
                Method::Put => {
                    // The operation's identity, made from the grant's tag: presenting a grant
                    // again is answered with the upload it made.
                    let mut identity = [0u8; size_of::<u128>()];
                    for (into, from) in identity.iter_mut().zip(&grant.tag) {
                        *into = *from;
                    }
                    let put = Put {
                        create_once: false,
                        identity: u128::from_le_bytes(identity),
                    };
                    st.open(&shared, &grant.key, put, Some(grant.access.length))
                        .map(Redeemed::Upload)
                }
            }
        }))
    }
}

impl Bytes for SimBytes {
    fn begin<'a>(
        &'a self,
        key: &'a str,
        put: Put,
        deadline: Instant,
    ) -> BoxFuture<'a, Result<Upload, Refusal>> {
        let shared = &self.shared;
        Box::pin(answer(shared, Op::Begin, deadline, move |st| {
            st.open(shared, key, put, None)
        }))
    }

    fn send<'a>(
        &'a self,
        upload: &'a UploadId,
        at: u64,
        piece: &'a [u8],
        deadline: Instant,
    ) -> BoxFuture<'a, Result<Progress, Refusal>> {
        let shared = &self.shared;
        Box::pin(answer(shared, Op::Send, deadline, move |st| {
            st.send(shared, upload, at, piece)
        }))
    }

    fn status<'a>(
        &'a self,
        upload: &'a UploadId,
        deadline: Instant,
    ) -> BoxFuture<'a, Result<Progress, Refusal>> {
        Box::pin(answer(&self.shared, Op::Status, deadline, move |st| {
            st.uploads
                .get(upload)
                .map(|open| open.progress)
                .ok_or_else(|| Refusal::NoUpload(upload.0.clone()))
        }))
    }

    fn finish<'a>(
        &'a self,
        upload: &'a UploadId,
        whole: Whole,
        deadline: Instant,
    ) -> BoxFuture<'a, Result<Meta, Refusal>> {
        let shared = &self.shared;
        Box::pin(answer(shared, Op::Finish, deadline, move |st| {
            st.finish(shared, upload, whole)
        }))
    }

    fn abort<'a>(
        &'a self,
        upload: &'a UploadId,
        deadline: Instant,
    ) -> BoxFuture<'a, Result<(), Refusal>> {
        Box::pin(answer(&self.shared, Op::Abort, deadline, move |st| {
            st.abort(upload)
        }))
    }

    fn get<'a>(
        &'a self,
        key: &'a str,
        range: Option<Range>,
        deadline: Instant,
    ) -> BoxFuture<'a, Result<Reading, Refusal>> {
        let shared = Arc::clone(&self.shared);
        Box::pin(answer(&self.shared, Op::Get, deadline, move |st| {
            st.reading(&shared, key, range)
        }))
    }

    fn head<'a>(&'a self, key: &'a str, deadline: Instant) -> BoxFuture<'a, Result<Meta, Refusal>> {
        let shared = &self.shared;
        Box::pin(answer(shared, Op::Head, deadline, move |st| {
            check_key(key, shared.limits.key)?;
            st.objects
                .get(key)
                .map(|object| object.meta.clone())
                .ok_or_else(|| Refusal::NotFound(key.to_owned()))
        }))
    }

    fn delete<'a>(
        &'a self,
        key: &'a str,
        deadline: Instant,
    ) -> BoxFuture<'a, Result<Deleted, Refusal>> {
        let shared = &self.shared;
        Box::pin(answer(shared, Op::Delete, deadline, move |st| {
            check_key(key, shared.limits.key)?;
            Ok(match st.objects.remove(key) {
                Some(object) => {
                    st.kept = st.kept.saturating_sub(object.meta.length);
                    Deleted::Removed
                }
                None => Deleted::Absent,
            })
        }))
    }

    fn grant<'a>(
        &'a self,
        key: &'a str,
        access: Access,
        holder: &'a str,
        expires: SystemTime,
        deadline: Instant,
    ) -> BoxFuture<'a, Result<Grant, Refusal>> {
        let shared = &self.shared;
        Box::pin(answer(shared, Op::Grant, deadline, move |st| {
            check_key(key, shared.limits.key)?;
            if holder.is_empty() {
                return Err(Refusal::Invalid {
                    what: "a grant's holder",
                    why: "it is empty".into(),
                });
            }
            if expires <= st.wall(shared.clock.as_ref()) {
                return Err(Refusal::Invalid {
                    what: "a grant's expiry",
                    why: "it has passed".into(),
                });
            }
            if access.method == Method::Get {
                let object = st
                    .objects
                    .get(key)
                    .ok_or_else(|| Refusal::NotFound(key.to_owned()))?;
                if object.meta.length != access.length {
                    return Err(Refusal::Mismatch(format!(
                        "the object is {} bytes; the grant asks {}",
                        object.meta.length, access.length
                    )));
                }
            }
            let message = signed(key, access, holder, expires)?;
            Ok(Grant {
                key: key.to_owned(),
                access,
                holder: holder.to_owned(),
                expires,
                tag: hmac::sign(&shared.key, &message).as_ref().to_vec(),
            })
        }))
    }
}

/// A reading's body: the bytes from `at` to `end` of a body it holds, so a delete or an overwrite
/// of its object does not take them from under it.
struct SimBody {
    shared: Arc<Shared>,
    held: Arc<Held>,
    at: u64,
    end: u64,
    /// How the reading ended, if it was cut or a read went unanswered: every later read is refused
    /// the same, as its stream is gone.
    ended: Option<Refusal>,
}

impl SimBody {
    fn take(&mut self, into: &mut [u8]) -> Result<usize, Refusal> {
        let left = self.end.saturating_sub(self.at);
        let mut n = count(into.len()).min(left);
        let to = self.at.saturating_add(n);
        if let Some(at) = self.shared.state.lock().faults.cut(Op::Read, self.at, to) {
            self.ended = Some(Refusal::Cut { at });
            if at == self.at {
                return Err(Refusal::Cut { at });
            }
            n = at.saturating_sub(self.at);
        }
        let want = usize::try_from(n).unwrap_or(usize::MAX);
        let into = into
            .get_mut(..want)
            .ok_or_else(|| unavailable("a read", "past its buffer"))?;
        let got = self
            .shared
            .store
            .read_at(self.held.id, self.at, into)
            .map_err(|e| unavailable("the store", e))?;
        if got != want {
            return Err(unavailable(
                "the store",
                format!("it gave {got} of the {want} bytes asked"),
            ));
        }
        self.at = self.at.saturating_add(n);
        Ok(want)
    }
}

impl Body for SimBody {
    fn read<'a>(
        &'a mut self,
        into: &'a mut [u8],
        deadline: Instant,
    ) -> BoxFuture<'a, Result<usize, Refusal>> {
        Box::pin(async move {
            if let Some(ended) = &self.ended {
                return Err(ended.clone());
            }
            let fault = self.shared.state.lock().faults.call(Op::Read);
            // A read's answer is its bytes: held, they arrive late, and past the deadline they do
            // not arrive at all, and the stream is gone.
            let held = match fault {
                Some(Fault::Refuse(refusal)) => return Err(refusal),
                Some(Fault::Vanish) => Some(None),
                Some(Fault::Delay(by)) => Some(self.shared.clock.now().checked_add(by)),
                Some(Fault::Lose | Fault::Cut(_)) | None => None,
            };
            if let Some(at) = held
                && !fault::held(self.shared.clock.as_ref(), at, deadline)
                    .await
                    .map_err(|e| unavailable("the store's clock", e))?
            {
                self.ended = Some(Refusal::Late);
                return Err(Refusal::Late);
            }
            self.take(into)
        })
    }
}

/// The fixture for a [`SimBytes`]: the simulated clock it was made with, and what the test that
/// makes it states.
pub struct SimBytesFixture {
    pub bytes: SimBytes,
    /// The clock `bytes` was made with.
    pub clock: SimClock,
    pub patience: Duration,
    pub length: u64,
    /// The steps [`SimClock::run`] has for a case.
    pub steps: u64,
}

impl Fixture for SimBytesFixture {
    fn drive<'a, T>(&'a self, work: BoxFuture<'a, T>) -> Result<T, Failure> {
        self.clock
            .run(work, self.steps)
            .map_err(|stalled| stalled.to_string())
    }

    fn injected(&self) -> Counts {
        self.bytes.injected()
    }
}

impl BytesFixture for SimBytesFixture {
    fn bytes(&self) -> &dyn Bytes {
        &self.bytes
    }

    fn clock(&self) -> &dyn Clock {
        &self.clock
    }

    fn pass(&self, d: Duration) -> Result<(), Failure> {
        self.clock.advance(d).map_err(|e| e.to_string())
    }

    fn arm(&self, op: Op, fault: Fault<Refusal>) -> Result<(), Failure> {
        self.bytes.arm(op, fault).map_err(|e| e.to_string())
    }

    fn attempts(&self, op: Op) -> u64 {
        self.bytes.attempts(op)
    }

    fn redeem<'a>(
        &'a self,
        grant: &'a Grant,
        holder: &'a str,
        deadline: Instant,
    ) -> BoxFuture<'a, Result<Redeemed, Refusal>> {
        self.bytes.redeem(grant, holder, deadline)
    }

    fn patience(&self) -> Duration {
        self.patience
    }

    fn length(&self) -> u64 {
        self.length
    }

    fn uploads_max(&self) -> usize {
        self.bytes.limits().uploads
    }
}
