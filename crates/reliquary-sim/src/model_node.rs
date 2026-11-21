//! The model node (docs/design/sim.md, W10): what the world's nodes run until the registry's own
//! cores land (plan RQ1). It is the reference model of plan RQ0.5 run as a node over the seams:
//! the model decides every request, and the registry's state is the log of the model's changes,
//! kept as rows of Amphora's `Records`, with every content those changes carry kept as an object
//! of the `Bytes` seam.
//!
//! - A node holds the model's state as a cache of the log, read up to the log's head before every
//!   request (one read at a time, which every request waiting joins once it started after them),
//!   so a read reflects every change acknowledged before it was asked.
//! - A change is written bytes first, records second (plan T-RQ1.8.7): its content as an object
//!   under a key no other write has, made whole and durable at its finish, and only then the
//!   entry naming it, conditioned on the log's next position being absent. Of two nodes writing
//!   at one position one wins; the other reads the log again, decides again, and writes again, as
//!   many times as its budget allows, and is then refused `Busy`.
//! - An answer lost or late is settled by reading the position (docs/design/seams.md S7): the
//!   entry there is this attempt's, or another's, or none yet, and the same entry is asked again.
//!   An attempt that may have applied and cannot be settled is refused `Unknown`, never guessed.
//! - A node that restarts reads the log from its start: it knows only what its stores kept.
//! - Where its maker says, a whole blob is read by a grant on its object, which its client redeems
//!   at the bytes store (architecture D4), living as long as its maker states.

use std::collections::{BTreeMap, VecDeque};
use std::sync::Arc;
use std::time::Duration;

use amphora_api::{Cond, Consistency, Refusal as RecordsRefusal, Row, Scan, Tx, Write};
use reliquary_api::answer::{Answer, Body};
use reliquary_api::model::{Change, Config, Model};
use reliquary_api::request::{Op, Request};
use reliquary_core::seams::bytes::{
    Access, Method, Put, Refusal as BytesRefusal, Sha256, UploadId, Whole,
};

use crate::call::{Answered, BytesAnswer, BytesCall, Call, ReadingId, RecordsAnswer, RecordsCall};
use crate::client::holder;
use crate::clock::on_grid;
use crate::entry::{self, Entry, ObjectRef};
use crate::node::{Core, Io};
use crate::wire::{CallId, JobId, Offer, Refused, RefusedKind, Reply};

/// The family the log's entries are kept in, each under its position.
pub const LOG: &str = "log";

/// What a model node spends and waits, as its maker states each (docs/design/numbers.md: until
/// plan RQ0.8's estimators, a test states them).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Budgets {
    /// How long each call may take: its deadline is the node's now plus this.
    pub wait: Duration,
    /// Rows a read of the log asks for at once.
    pub page: usize,
    /// The most bytes a piece of a put or a read carries.
    pub piece: usize,
    /// Writes a request may lose to another, answers it may settle by asking again and pieces it
    /// may resume, together, before it is refused.
    pub tries: u32,
    /// Requests in flight at once; past them a request is refused `Busy`.
    pub jobs: usize,
    /// How long a grant for a blob lives; with none, a blob's bytes are relayed.
    pub grant_life: Option<Duration>,
}

/// A content's bytes being written as an object.
struct Upload {
    key: String,
    content: Arc<[u8]>,
    sha256: Sha256,
    id: Option<UploadId>,
    sent: u64,
    piece: usize,
}

/// A decision, and the log position it was made after.
#[derive(Clone)]
struct Decided {
    answer: Answer,
    change: Change,
    at: u64,
    identity: u128,
}

enum Stage {
    /// Waiting for a read of the log that started after `epoch` was asked.
    Syncing {
        epoch: u64,
    },
    Writing {
        decided: Decided,
        upload: Upload,
    },
    Committing {
        decided: Decided,
        object: Option<ObjectRef>,
    },
    Reconciling {
        decided: Decided,
        object: Option<ObjectRef>,
    },
    Granting {
        answer: Answer,
    },
}

struct Job {
    request: Request,
    stage: Stage,
    /// Every commit attempt's identity, and the answer it gives if it landed.
    tried: Vec<(u128, Answer)>,
    tries: u32,
    /// A commit of this request's may have applied, unsettled.
    unknown: bool,
}

/// One read of the log, a page at a time, each entry's content fetched before it is applied.
struct Pass {
    epoch: u64,
    rows: VecDeque<Row>,
    more: bool,
    fetch: Option<Fetch>,
}

struct Fetch {
    position: u64,
    entry: Entry,
    object: ObjectRef,
    reading: Option<ReadingId>,
    got: Vec<u8>,
}

#[derive(Default)]
struct Syncer {
    asked: u64,
    done: u64,
    pass: Option<Pass>,
}

enum Owner {
    Sync,
    Job(JobId),
}

/// Work a handler leaves for the node to do once it returns: the node does it in a loop, so no
/// handler calls back into another that called it (CLAUDE.md §11).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Step {
    /// Decide job `JobId` again, on what the node holds now.
    Decide(JobId),
    /// Start a read of the log, if one was asked and none runs.
    Pump,
}

/// The reference model, run as a node.
pub struct ModelNode {
    budgets: Budgets,
    model: Model,
    applied: u64,
    objects: BTreeMap<(Sha256, u64), ObjectRef>,
    /// Entries applied, by the identity of the attempt that wrote each.
    landed: BTreeMap<u128, u64>,
    sync: Syncer,
    jobs: BTreeMap<JobId, Job>,
    calls: BTreeMap<CallId, Owner>,
    /// Work left by the handlers of one input: at most a decision for each request held and one
    /// read of the log.
    steps: VecDeque<Step>,
}

/// The key of the log's entry at `position`: big-endian, so keys sort as positions do.
#[must_use]
pub fn key(position: u64) -> Vec<u8> {
    position.to_be_bytes().to_vec()
}

fn position_of(key: &[u8]) -> Option<u64> {
    <[u8; size_of::<u64>()]>::try_from(key)
        .ok()
        .map(u64::from_be_bytes)
}

fn length_of(content: &[u8]) -> u64 {
    u64::try_from(content.len()).unwrap_or(u64::MAX)
}

impl ModelNode {
    /// A node of a registry `config` states, spending what `budgets` says.
    #[must_use]
    pub fn new(config: Config, budgets: Budgets) -> Self {
        Self {
            budgets,
            model: Model::new(config),
            applied: 0,
            objects: BTreeMap::new(),
            landed: BTreeMap::new(),
            sync: Syncer::default(),
            jobs: BTreeMap::new(),
            calls: BTreeMap::new(),
            steps: VecDeque::new(),
        }
    }

    /// Does the work the input's handlers left, in order, until none is left.
    fn drain(&mut self, io: &mut Io<'_>) {
        while let Some(step) = self.steps.pop_front() {
            match step {
                Step::Decide(id) => self.decide(io, id),
                Step::Pump => self.pump(io),
            }
        }
    }

    /// The model's state as this node holds it.
    #[must_use]
    pub fn model(&self) -> &Model {
        &self.model
    }

    /// Makes `call` for `owner`; false when no call could be made.
    fn call(&mut self, io: &mut Io<'_>, owner: Owner, call: Call) -> bool {
        let Some(deadline) = io.now().checked_add(self.budgets.wait) else {
            return false;
        };
        match io.call(call, deadline) {
            Some(id) => {
                self.calls.insert(id, owner);
                true
            }
            None => false,
        }
    }

    /// Ends job `id` with `reply`.
    fn reply(&mut self, io: &mut Io<'_>, id: JobId, reply: Reply) {
        self.jobs.remove(&id);
        io.reply(id, reply);
    }

    /// Refuses job `id`: `kind`, unless a commit of its may have applied, which makes it
    /// `Unknown`.
    fn refuse(&mut self, io: &mut Io<'_>, id: JobId, kind: RefusedKind, why: &str) {
        let unknown = self.jobs.get(&id).is_some_and(|job| job.unknown);
        let kind = if unknown { RefusedKind::Unknown } else { kind };
        self.reply(
            io,
            id,
            Reply::Refused(Refused {
                kind,
                why: why.to_owned(),
            }),
        );
    }

    // ------------------------------------------------------------------------------ the log

    /// Asks for a read of the log that starts after now: its number, which a request waits for.
    fn ask(&mut self) -> u64 {
        self.sync.asked = self.sync.asked.saturating_add(1);
        self.sync.asked
    }

    /// Starts a read of the log if one was asked after the last and none is running.
    fn pump(&mut self, io: &mut Io<'_>) {
        if self.sync.pass.is_some() || self.sync.asked <= self.sync.done {
            return;
        }
        self.sync.pass = Some(Pass {
            epoch: self.sync.asked,
            rows: VecDeque::new(),
            more: false,
            fetch: None,
        });
        self.scan(io);
    }

    fn scan(&mut self, io: &mut Io<'_>) {
        let after = (self.applied > 0).then(|| key(self.applied));
        let call = Call::Records(RecordsCall::Scan {
            family: LOG.to_owned(),
            scan: Scan {
                prefix: Vec::new(),
                after,
                limit: self.budgets.page,
                at: Consistency::Linearizable,
            },
        });
        if !self.call(io, Owner::Sync, call) {
            self.sync_failed(io, "no call could be made");
        }
    }

    /// The read of the log failed: every request waiting on it is refused, having applied nothing.
    fn sync_failed(&mut self, io: &mut Io<'_>, why: &str) {
        let epoch = self
            .sync
            .pass
            .take()
            .map_or(self.sync.asked, |pass| pass.epoch);
        self.sync.done = self.sync.done.max(epoch);
        let waiting: Vec<JobId> = self
            .jobs
            .iter()
            .filter(|(_, job)| matches!(job.stage, Stage::Syncing { epoch: e } if e <= epoch))
            .map(|(id, _)| *id)
            .collect();
        for id in waiting {
            self.refuse(io, id, RefusedKind::Unavailable, why);
        }
        self.steps.push_back(Step::Pump);
    }

    /// The read of the log is answered.
    fn synced(&mut self, io: &mut Io<'_>, answer: Answered) {
        let Some(pass) = self.sync.pass.as_mut() else {
            return;
        };
        match answer {
            Answered::Records(RecordsAnswer::Page(Ok(page))) => {
                pass.rows = page.rows.into();
                pass.more = page.more;
                self.next_row(io);
            }
            Answered::Bytes(BytesAnswer::Opened(Ok(opened))) => {
                let Some(fetch) = pass.fetch.as_mut() else {
                    return self.sync_failed(io, "a reading no fetch asked for");
                };
                if opened.meta.length != fetch.object.length
                    || opened.meta.sha256 != fetch.object.sha256
                {
                    return self
                        .sync_failed(io, "the log names an object that is not what it says");
                }
                fetch.reading = Some(opened.reading);
                self.read_on(io);
            }
            Answered::Bytes(BytesAnswer::Read(Ok(bytes))) => {
                let Some(fetch) = pass.fetch.as_mut() else {
                    return self.sync_failed(io, "a read no fetch asked for");
                };
                if !bytes.is_empty() {
                    fetch.got.extend_from_slice(&bytes);
                    if length_of(&fetch.got) > fetch.object.length {
                        return self.sync_failed(io, "an object longer than the log says");
                    }
                    return self.read_on(io);
                }
                let Some(fetch) = pass.fetch.take() else {
                    return;
                };
                if length_of(&fetch.got) != fetch.object.length
                    || Sha256::of(&fetch.got) != fetch.object.sha256
                {
                    return self.sync_failed(io, "an object that is not what the log says");
                }
                self.apply(fetch.position, fetch.entry, Arc::from(fetch.got));
                self.next_row(io);
            }
            Answered::Records(RecordsAnswer::Page(Err(refusal))) => {
                self.sync_failed(io, &format!("the log could not be read: {refusal}"));
            }
            Answered::Bytes(
                BytesAnswer::Opened(Err(refusal)) | BytesAnswer::Read(Err(refusal)),
            ) => {
                self.sync_failed(
                    io,
                    &format!("an object the log names could not be read: {refusal}"),
                );
            }
            _ => self.sync_failed(io, "an answer no read of the log asked for"),
        }
    }

    /// Reads the next piece of the object being fetched.
    fn read_on(&mut self, io: &mut Io<'_>) {
        let reading = self
            .sync
            .pass
            .as_ref()
            .and_then(|pass| pass.fetch.as_ref())
            .and_then(|fetch| fetch.reading);
        let Some(reading) = reading else {
            return self.sync_failed(io, "no reading to read");
        };
        let call = Call::Bytes(BytesCall::Read {
            reading,
            length: self.budgets.piece,
        });
        if !self.call(io, Owner::Sync, call) {
            self.sync_failed(io, "no call could be made");
        }
    }

    /// Applies the page's entries in order, fetching each one's content first; then reads the
    /// next page, or ends the read and lets every request waiting on it go on.
    fn next_row(&mut self, io: &mut Io<'_>) {
        loop {
            let Some(pass) = self.sync.pass.as_mut() else {
                return;
            };
            let Some(row) = pass.rows.pop_front() else {
                if pass.more {
                    return self.scan(io);
                }
                let epoch = pass.epoch;
                self.sync.pass = None;
                self.sync.done = self.sync.done.max(epoch);
                self.release(io, epoch);
                return self.pump(io);
            };
            let Some(position) = position_of(&row.key) else {
                return self.sync_failed(io, "a log key that is no position");
            };
            if position <= self.applied {
                continue;
            }
            if Some(position) != self.applied.checked_add(1) {
                return self.sync_failed(io, "a gap in the log");
            }
            let entry = match entry::decode(&row.value) {
                Ok(entry) => entry,
                Err(e) => return self.sync_failed(io, &e.to_string()),
            };
            let Some(object) = entry.change.object().cloned() else {
                self.apply(position, entry, Arc::from(Vec::new()));
                continue;
            };
            let key = object.key.clone();
            pass.fetch = Some(Fetch {
                position,
                entry,
                object,
                reading: None,
                got: Vec::new(),
            });
            let call = Call::Bytes(BytesCall::Get { key, range: None });
            if !self.call(io, Owner::Sync, call) {
                return self.sync_failed(io, "no call could be made");
            }
            return;
        }
    }

    /// Applies the entry at `position`, its content `content`, if it is the next the node holds:
    /// one its own commit applied already is not applied again.
    fn apply(&mut self, position: u64, entry: Entry, content: Arc<[u8]>) {
        if self.applied.checked_add(1) != Some(position) {
            return;
        }
        if let Some(object) = entry.change.object() {
            self.objects
                .insert((object.sha256, object.length), object.clone());
        }
        self.landed.insert(entry.identity, position);
        self.model.apply(entry.change.change(content));
        self.applied = position;
    }

    /// Every request waiting on a read of the log that has ended goes on.
    fn release(&mut self, io: &mut Io<'_>, epoch: u64) {
        let ready: Vec<JobId> = self
            .jobs
            .iter()
            .filter(|(_, job)| matches!(job.stage, Stage::Syncing { epoch: e } if e <= epoch))
            .map(|(id, _)| *id)
            .collect();
        for id in ready {
            self.decide(io, id);
        }
    }

    // ------------------------------------------------------------------------------ requests

    /// Decides job `id` on the state the node holds.
    fn decide(&mut self, io: &mut Io<'_>, id: JobId) {
        let Some(job) = self.jobs.get_mut(&id) else {
            return;
        };
        // An earlier attempt of this request that landed is its answer.
        let landed = job
            .tried
            .iter()
            .find(|(identity, _)| self.landed.contains_key(identity))
            .map(|(_, answer)| answer.clone());
        if let Some(answer) = landed {
            return self.reply(io, id, Reply::Answer(answer));
        }
        // Every earlier attempt is settled: each one's position is taken by another's entry.
        job.unknown = false;
        let decision = self.model.decide(&job.request);
        if decision.change == Change::None {
            let whole_read = matches!(job.request.op, Op::GetBlob { range: None, .. });
            return if whole_read && self.budgets.grant_life.is_some() {
                self.grant(io, id, decision.answer)
            } else {
                self.reply(io, id, Reply::Answer(decision.answer))
            };
        }
        let identity = io.identity();
        job.tried.push((identity, decision.answer.clone()));
        let decided = Decided {
            answer: decision.answer,
            change: decision.change,
            at: self.applied,
            identity,
        };
        self.write(io, id, decided);
    }

    /// Writes the content `decided` carries, unless an object already holds it, then commits.
    fn write(&mut self, io: &mut Io<'_>, id: JobId, decided: Decided) {
        let Some(content) = entry::content(&decided.change).cloned() else {
            return self.commit(io, id, decided, None);
        };
        let sha256 = Sha256::of(&content);
        let length = length_of(&content);
        if let Some(object) = self.objects.get(&(sha256, length)).cloned() {
            return self.commit(io, id, decided, Some(object));
        }
        let key = format!("object-{:032x}", io.identity());
        let put = Put {
            create_once: true,
            identity: io.identity(),
        };
        let call = Call::Bytes(BytesCall::Begin {
            key: key.clone(),
            put,
        });
        if let Some(job) = self.jobs.get_mut(&id) {
            job.stage = Stage::Writing {
                decided,
                upload: Upload {
                    key,
                    content,
                    sha256,
                    id: None,
                    sent: 0,
                    piece: self.budgets.piece,
                },
            };
        }
        if !self.call(io, Owner::Job(id), call) {
            self.refuse(io, id, RefusedKind::Unavailable, "no call could be made");
        }
    }

    /// Sends the upload's next piece, or finishes it.
    fn send_on(&mut self, io: &mut Io<'_>, id: JobId) {
        let Some(Job {
            stage: Stage::Writing { upload, .. },
            ..
        }) = self.jobs.get(&id)
        else {
            return;
        };
        let Some(upload_id) = upload.id.clone() else {
            return self.refuse(io, id, RefusedKind::Unavailable, "an upload with no name");
        };
        let length = length_of(&upload.content);
        let call = if upload.sent < length {
            let start = usize::try_from(upload.sent).unwrap_or(usize::MAX);
            let end = start.saturating_add(upload.piece).min(upload.content.len());
            let Some(piece) = upload.content.get(start..end) else {
                return self.refuse(io, id, RefusedKind::Unavailable, "a piece past the content");
            };
            Call::Bytes(BytesCall::Send {
                upload: upload_id,
                at: upload.sent,
                piece: Arc::from(piece),
            })
        } else {
            Call::Bytes(BytesCall::Finish {
                upload: upload_id,
                whole: Whole {
                    length,
                    sha256: upload.sha256,
                },
            })
        };
        if !self.call(io, Owner::Job(id), call) {
            self.refuse(io, id, RefusedKind::Unavailable, "no call could be made");
        }
    }

    /// Commits `decided`'s entry at the position after the one it was decided at.
    fn commit(&mut self, io: &mut Io<'_>, id: JobId, decided: Decided, object: Option<ObjectRef>) {
        if decided.at != self.applied {
            // The node applied more since this was decided: decide again on what it holds now.
            self.steps.push_back(Step::Decide(id));
            return;
        }
        let Some(position) = decided.at.checked_add(1) else {
            return self.refuse(
                io,
                id,
                RefusedKind::Unavailable,
                "the log is at its last position",
            );
        };
        let Some(logged) = entry::logged(&decided.change, object.clone()) else {
            return self.refuse(
                io,
                id,
                RefusedKind::Unavailable,
                "a change the log cannot hold",
            );
        };
        let value = entry::encode(&Entry {
            identity: decided.identity,
            change: logged,
        });
        let tx = Tx {
            conds: vec![Cond::Absent {
                family: LOG.to_owned(),
                key: key(position),
            }],
            writes: vec![Write::Put {
                family: LOG.to_owned(),
                key: key(position),
                value,
            }],
        };
        if let Some(job) = self.jobs.get_mut(&id) {
            job.stage = Stage::Committing { decided, object };
        }
        if !self.call(
            io,
            Owner::Job(id),
            Call::Records(RecordsCall::Commit { tx }),
        ) {
            self.refuse(io, id, RefusedKind::Unavailable, "no call could be made");
        }
    }

    /// `decided`'s entry is in the log: the node applies it, if it has not read it already, and
    /// answers.
    fn committed(&mut self, io: &mut Io<'_>, id: JobId, decided: Decided) {
        let position = decided.at.saturating_add(1);
        if self.applied.checked_add(1) == Some(position) {
            self.landed.insert(decided.identity, position);
            self.model.apply(decided.change.clone());
            self.applied = position;
        }
        self.reply(io, id, Reply::Answer(decided.answer));
    }

    /// Counts a try; false, and the request refused, when it has used its budget.
    fn tried(&mut self, io: &mut Io<'_>, id: JobId) -> bool {
        let Some(job) = self.jobs.get_mut(&id) else {
            return false;
        };
        job.tries = job.tries.saturating_add(1);
        if job.tries > self.budgets.tries {
            self.refuse(io, id, RefusedKind::Busy, "its tries are spent");
            return false;
        }
        true
    }

    /// Another entry took the position: read the log and decide again.
    fn conflicted(&mut self, io: &mut Io<'_>, id: JobId) {
        if !self.tried(io, id) {
            return;
        }
        let epoch = self.ask();
        if let Some(job) = self.jobs.get_mut(&id) {
            job.stage = Stage::Syncing { epoch };
        }
        self.pump(io);
    }

    fn grant(&mut self, io: &mut Io<'_>, id: JobId, answer: Answer) {
        let (
            Some(life),
            Body::Blob {
                content: Some(content),
                ..
            },
        ) = (self.budgets.grant_life, &answer.body)
        else {
            return self.reply(io, id, Reply::Answer(answer));
        };
        let length = length_of(content);
        let Some(object) = self.objects.get(&(Sha256::of(content), length)).cloned() else {
            return self.reply(io, id, Reply::Answer(answer));
        };
        let principal = self
            .jobs
            .get(&id)
            .map_or(reliquary_api::request::Principal::Anonymous, |job| {
                job.request.principal
            });
        let life =
            Duration::from_nanos(on_grid(u64::try_from(life.as_nanos()).unwrap_or(u64::MAX)));
        let Some(expires) = io.wall().checked_add(life) else {
            return self.reply(io, id, Reply::Answer(answer));
        };
        let call = Call::Bytes(BytesCall::Grant {
            key: object.key,
            access: Access {
                method: Method::Get,
                length,
            },
            holder: holder(principal),
            expires,
        });
        if let Some(job) = self.jobs.get_mut(&id) {
            job.stage = Stage::Granting {
                answer: answer.clone(),
            };
        }
        if !self.call(io, Owner::Job(id), call) {
            self.reply(io, id, Reply::Answer(answer));
        }
    }

    /// A call of job `id` is answered.
    fn job_answered(&mut self, io: &mut Io<'_>, id: JobId, answer: Answered) {
        let Some(job) = self.jobs.get_mut(&id) else {
            return;
        };
        let stage = std::mem::replace(&mut job.stage, Stage::Syncing { epoch: 0 });
        match (stage, answer) {
            (
                Stage::Writing {
                    decided,
                    mut upload,
                },
                Answered::Bytes(answer),
            ) => match answer {
                BytesAnswer::Upload(Ok(begun)) => {
                    upload.piece = upload.piece.min(begun.plan.piece).max(1);
                    upload.sent = begun.progress.received;
                    upload.id = Some(begun.id);
                    job.stage = Stage::Writing { decided, upload };
                    self.send_on(io, id);
                }
                BytesAnswer::Progress(Ok(progress)) => {
                    upload.sent = progress.received;
                    job.stage = Stage::Writing { decided, upload };
                    self.send_on(io, id);
                }
                BytesAnswer::Progress(Err(
                    BytesRefusal::Cut { at } | BytesRefusal::Offset { expected: at },
                )) => {
                    // The put goes on from where the store says it stands (docs/design/seams.md S2).
                    upload.sent = at;
                    job.stage = Stage::Writing { decided, upload };
                    if self.tried(io, id) {
                        self.send_on(io, id);
                    }
                }
                BytesAnswer::Meta(Ok(meta)) => {
                    let object = ObjectRef {
                        key: upload.key,
                        length: meta.length,
                        sha256: meta.sha256,
                    };
                    self.objects
                        .insert((object.sha256, object.length), object.clone());
                    self.commit(io, id, decided, Some(object));
                }
                other => {
                    let why = Answered::Bytes(other).summary();
                    self.refuse(
                        io,
                        id,
                        RefusedKind::Unavailable,
                        &format!("the content could not be written: {why}"),
                    );
                }
            },
            (
                Stage::Committing { decided, object },
                Answered::Records(RecordsAnswer::Rev(result)),
            ) => {
                match result {
                    Ok(_) => self.committed(io, id, decided),
                    Err(RecordsRefusal::Conflict { .. }) => self.conflicted(io, id),
                    Err(
                        RecordsRefusal::Timeout
                        | RecordsRefusal::Unavailable(_)
                        | RecordsRefusal::UnknownOutcome
                        | RecordsRefusal::NotLeader { .. }
                        | RecordsRefusal::Fenced(_),
                    ) => {
                        // It may have applied: read the position to settle it.
                        job.unknown = true;
                        let position = decided.at.saturating_add(1);
                        job.stage = Stage::Reconciling { decided, object };
                        let call = Call::Records(RecordsCall::Get {
                            family: LOG.to_owned(),
                            key: key(position),
                            at: Consistency::Linearizable,
                        });
                        if !self.call(io, Owner::Job(id), call) {
                            self.refuse(io, id, RefusedKind::Unknown, "no call could be made");
                        }
                    }
                    Err(refusal) => {
                        self.refuse(
                            io,
                            id,
                            RefusedKind::Unavailable,
                            &format!("the commit was refused: {refusal}"),
                        );
                    }
                }
            }
            (
                Stage::Reconciling { decided, object },
                Answered::Records(RecordsAnswer::Row(result)),
            ) => {
                match result {
                    Ok(Some(row)) => match entry::decode(&row.value) {
                        Ok(found) if found.identity == decided.identity => {
                            self.committed(io, id, decided)
                        }
                        // Another's entry holds the position: this attempt can never land there.
                        Ok(_) => {
                            job.unknown = false;
                            self.conflicted(io, id);
                        }
                        Err(e) => self.refuse(io, id, RefusedKind::Unknown, &e.to_string()),
                    },
                    // Not there yet: the same entry, asked again, lands once at most.
                    Ok(None) => {
                        if self.tried(io, id) {
                            self.commit(io, id, decided, object);
                        }
                    }
                    Err(refusal) => {
                        self.refuse(
                            io,
                            id,
                            RefusedKind::Unknown,
                            &format!("the commit could not be settled: {refusal}"),
                        );
                    }
                }
            }
            (Stage::Granting { answer }, Answered::Bytes(BytesAnswer::Grant(result))) => {
                match (result, &answer.body) {
                    (Ok(grant), Body::Blob { digest, size, .. }) => {
                        let offer = Offer {
                            grant,
                            digest: digest.clone(),
                            size: *size,
                        };
                        self.reply(io, id, Reply::Grant(offer));
                    }
                    // No grant: the bytes are relayed.
                    _ => self.reply(io, id, Reply::Answer(answer)),
                }
            }
            (_, answer) => {
                let why = answer.summary();
                self.refuse(
                    io,
                    id,
                    RefusedKind::Unavailable,
                    &format!("an answer its request did not ask for: {why}"),
                );
            }
        }
    }
}

impl Core for ModelNode {
    fn request(&mut self, io: &mut Io<'_>, job: JobId, request: &Request) {
        if self.jobs.len() >= self.budgets.jobs {
            io.reply(
                job,
                Reply::Refused(Refused {
                    kind: RefusedKind::Busy,
                    why: "the node holds its most requests".to_owned(),
                }),
            );
            return;
        }
        let epoch = self.ask();
        self.jobs.insert(
            job,
            Job {
                request: request.clone(),
                stage: Stage::Syncing { epoch },
                tried: Vec::new(),
                tries: 0,
                unknown: false,
            },
        );
        self.steps.push_back(Step::Pump);
        self.drain(io);
    }

    fn answered(&mut self, io: &mut Io<'_>, call: CallId, answer: Answered) {
        match self.calls.remove(&call) {
            Some(Owner::Sync) => self.synced(io, answer),
            Some(Owner::Job(id)) => self.job_answered(io, id, answer),
            None => {}
        }
        self.drain(io);
    }
}
