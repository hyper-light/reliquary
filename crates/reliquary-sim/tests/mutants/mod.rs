//! Defects planted in the model node, each a core that wraps it and does one thing wrong, so a
//! check of the world is shown to find what it is for (docs/design/sim.md W11; hyper-raft's
//! docs/sim.md §4.6, "Testing the tests"). Each takes back the calls and replies the node gave
//! ([`Io::take_calls`], [`Io::take_replies`]) and holds, drops, or answers them itself.

#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::time::{Duration, Instant};

use reliquary_api::request::{Op, Request};
use reliquary_core::seams::bytes::Meta;
use reliquary_sim::call::{Answered, BytesAnswer, BytesCall, Call, RecordsCall};
use reliquary_sim::model_node::ModelNode;
use reliquary_sim::node::{Core, Io};
use reliquary_sim::wire::{CallId, JobId, Refused, RefusedKind, Reply};

/// A node whose bytes say an object is made before it is: it answers each finish itself, at once,
/// and sends the real finish only once the commit that names the object is answered. A power cut
/// between the two leaves the log naming an object that was never made.
pub struct EagerFinish {
    pub inner: ModelNode,
    held: Vec<(CallId, Call, Instant)>,
    faked: BTreeSet<CallId>,
    commits: BTreeSet<CallId>,
}

impl EagerFinish {
    pub fn new(inner: ModelNode) -> Self {
        Self {
            inner,
            held: Vec::new(),
            faked: BTreeSet::new(),
            commits: BTreeSet::new(),
        }
    }

    fn intercept(&mut self, io: &mut Io<'_>) {
        let mut out = Vec::new();
        loop {
            let calls = io.take_calls();
            if calls.is_empty() {
                break;
            }
            for (id, call, deadline) in calls {
                match &call {
                    Call::Bytes(BytesCall::Finish { whole, .. }) => {
                        let meta = Meta {
                            key: String::new(),
                            length: whole.length,
                            sha256: whole.sha256,
                        };
                        self.held.push((id, call.clone(), deadline));
                        self.faked.insert(id);
                        self.inner
                            .answered(io, id, Answered::Bytes(BytesAnswer::Meta(Ok(meta))));
                    }
                    Call::Records(RecordsCall::Commit { .. }) => {
                        self.commits.insert(id);
                        out.push((id, call, deadline));
                    }
                    _ => out.push((id, call, deadline)),
                }
            }
        }
        for (id, call, deadline) in out {
            io.make_call(id, call, deadline);
        }
    }
}

impl Core for EagerFinish {
    fn request(&mut self, io: &mut Io<'_>, job: JobId, request: &Request) {
        self.inner.request(io, job, request);
        self.intercept(io);
    }

    fn answered(&mut self, io: &mut Io<'_>, call: CallId, answer: Answered) {
        if self.faked.remove(&call) {
            return;
        }
        let release = self.commits.remove(&call);
        self.inner.answered(io, call, answer);
        self.intercept(io);
        if release {
            for (id, call, deadline) in std::mem::take(&mut self.held) {
                io.make_call(id, call, deadline);
            }
        }
    }
}

/// A node that never answers requests of one kind: it takes them, and drops what it would reply.
pub struct Silent {
    pub inner: ModelNode,
    pub silent: fn(&Request) -> bool,
    jobs: BTreeSet<JobId>,
}

impl Silent {
    pub fn new(inner: ModelNode, silent: fn(&Request) -> bool) -> Self {
        Self {
            inner,
            silent,
            jobs: BTreeSet::new(),
        }
    }

    fn drop_replies(&mut self, io: &mut Io<'_>) {
        for (job, reply) in io.take_replies() {
            if !self.jobs.contains(&job) {
                io.reply(job, reply);
            }
        }
    }
}

impl Core for Silent {
    fn request(&mut self, io: &mut Io<'_>, job: JobId, request: &Request) {
        if (self.silent)(request) {
            self.jobs.insert(job);
        }
        self.inner.request(io, job, request);
        self.drop_replies(io);
    }

    fn answered(&mut self, io: &mut Io<'_>, call: CallId, answer: Answered) {
        self.inner.answered(io, call, answer);
        self.drop_replies(io);
    }
}

/// A node that refuses `Busy` every request that arrives while another is in flight, and holds
/// each answer back until it has read its records `pad` more times: whoever comes second meets a
/// node that is nearly always busy with the other.
pub struct OneAtATime {
    pub inner: ModelNode,
    pad: u32,
    wait: Duration,
    in_flight: BTreeSet<JobId>,
    /// Answers held back: each job's reply, and the reads it still makes before it is sent.
    held: BTreeMap<JobId, (Reply, u32)>,
    padding: BTreeMap<CallId, JobId>,
}

impl OneAtATime {
    pub fn new(inner: ModelNode, pad: u32, wait: Duration) -> Self {
        Self {
            inner,
            pad,
            wait,
            in_flight: BTreeSet::new(),
            held: BTreeMap::new(),
            padding: BTreeMap::new(),
        }
    }

    fn settle(&mut self, io: &mut Io<'_>) {
        for (job, reply) in io.take_replies() {
            self.held.insert(job, (reply, self.pad));
            self.pad_on(io, job);
        }
    }

    /// One more read before `job`'s answer goes, or the answer.
    fn pad_on(&mut self, io: &mut Io<'_>, job: JobId) {
        let Some((_, left)) = self.held.get_mut(&job) else {
            return;
        };
        if *left == 0 {
            if let Some((reply, _)) = self.held.remove(&job) {
                self.in_flight.remove(&job);
                io.reply(job, reply);
            }
            return;
        }
        *left -= 1;
        let deadline = io.now() + self.wait;
        if let Some(id) = io.call(Call::Records(RecordsCall::Head), deadline) {
            self.padding.insert(id, job);
        }
    }
}

impl Core for OneAtATime {
    fn request(&mut self, io: &mut Io<'_>, job: JobId, request: &Request) {
        if !self.in_flight.is_empty() {
            io.reply(
                job,
                Reply::Refused(Refused {
                    kind: RefusedKind::Busy,
                    why: "one at a time".to_owned(),
                }),
            );
            return;
        }
        self.in_flight.insert(job);
        self.inner.request(io, job, request);
        self.settle(io);
    }

    fn answered(&mut self, io: &mut Io<'_>, call: CallId, answer: Answered) {
        if let Some(job) = self.padding.remove(&call) {
            return self.pad_on(io, job);
        }
        self.inner.answered(io, call, answer);
        self.settle(io);
    }
}

/// A node that locks what a request names, in the order it names them, a lock at a time, with a
/// read of the records between taking one and the next, and waits for a lock another request
/// holds. Two requests naming two things in opposite orders each hold one and wait on the other.
pub struct Locking {
    pub inner: ModelNode,
    /// Each lock's holder.
    held: BTreeMap<String, JobId>,
    /// Each request's locks still to take, its request, and the call it waits on between locks.
    pending: BTreeMap<JobId, (Vec<String>, Request)>,
    pacing: BTreeMap<CallId, JobId>,
    holding: BTreeMap<JobId, Vec<String>>,
    pub wait: Duration,
}

/// What a request names, in the order it names them: its repository, then the repository it
/// mounts from, then the blobs a manifest lists.
pub fn resources(request: &Request) -> Vec<String> {
    match &request.op {
        Op::MountBlob { name, from, .. } => {
            let mut names = vec![name.clone()];
            names.extend(from.clone());
            names
        }
        Op::PutManifest { name, manifest, .. } => {
            let mut names = vec![name.clone()];
            if let reliquary_api::request::ManifestBody::Image(image) = &manifest.body {
                names.extend(image.layers.iter().map(|l| l.digest.clone()));
            }
            names
        }
        _ => Vec::new(),
    }
}

impl Locking {
    pub fn new(inner: ModelNode, wait: Duration) -> Self {
        Self {
            inner,
            held: BTreeMap::new(),
            pending: BTreeMap::new(),
            pacing: BTreeMap::new(),
            holding: BTreeMap::new(),
            wait,
        }
    }

    /// Takes `job`'s next lock if it is free, then reads the records before the one after; with
    /// every lock held, hands the request to the node.
    fn advance(&mut self, io: &mut Io<'_>, job: JobId) {
        let Some((locks, _)) = self.pending.get_mut(&job) else {
            return;
        };
        match locks.first().cloned() {
            Some(lock) => {
                if self.held.get(&lock).is_some_and(|holder| *holder != job) {
                    return;
                }
                locks.remove(0);
                self.held.insert(lock.clone(), job);
                self.holding.entry(job).or_default().push(lock);
                let deadline = io.now() + self.wait;
                if let Some(id) = io.call(Call::Records(RecordsCall::Head), deadline) {
                    self.pacing.insert(id, job);
                }
            }
            None => {
                let Some((_, request)) = self.pending.remove(&job) else {
                    return;
                };
                self.inner.request(io, job, &request);
                self.release(io);
            }
        }
    }

    /// Lets go the locks of every request the node answered, and moves the requests waiting.
    fn release(&mut self, io: &mut Io<'_>) {
        let replies = io.take_replies();
        for (job, reply) in replies {
            for lock in self.holding.remove(&job).unwrap_or_default() {
                self.held.remove(&lock);
            }
            io.reply(job, reply);
        }
        let waiting: Vec<JobId> = self.pending.keys().copied().collect();
        for job in waiting {
            self.advance(io, job);
        }
    }
}

impl Core for Locking {
    fn request(&mut self, io: &mut Io<'_>, job: JobId, request: &Request) {
        self.pending
            .insert(job, (resources(request), request.clone()));
        self.advance(io, job);
    }

    fn answered(&mut self, io: &mut Io<'_>, call: CallId, answer: Answered) {
        if let Some(job) = self.pacing.remove(&call) {
            return self.advance(io, job);
        }
        self.inner.answered(io, call, answer);
        self.release(io);
    }
}

/// A node that measures how long its calls may take by its wall clock: each call's deadline is the
/// instant it first made a call, plus the wall time since then, plus its wait. A wall clock stepped
/// back past where it started gives deadlines before the node's own now.
pub struct WallWaits {
    pub inner: ModelNode,
    wait: Duration,
    start: Option<(Instant, std::time::SystemTime)>,
}

impl WallWaits {
    pub fn new(inner: ModelNode, wait: Duration) -> Self {
        Self {
            inner,
            wait,
            start: None,
        }
    }

    fn rewrite(&mut self, io: &mut Io<'_>) {
        let (began, wall) = *self.start.get_or_insert((io.now(), io.wall()));
        let elapsed = io.wall().duration_since(wall).unwrap_or(Duration::ZERO);
        for (id, call, _) in io.take_calls() {
            io.make_call(id, call, began + elapsed + self.wait);
        }
    }
}

impl Core for WallWaits {
    fn request(&mut self, io: &mut Io<'_>, job: JobId, request: &Request) {
        self.inner.request(io, job, request);
        self.rewrite(io);
    }

    fn answered(&mut self, io: &mut Io<'_>, call: CallId, answer: Answered) {
        self.inner.answered(io, call, answer);
        self.rewrite(io);
    }
}
