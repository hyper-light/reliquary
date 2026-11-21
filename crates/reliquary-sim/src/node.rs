//! A node (docs/design/sim.md, W2): the shell that runs a [`Core`] in the world. A core is a state
//! machine: it is given a client's request or a call's answer and gives calls on its seams and
//! replies to clients, through an [`Io`] that holds no seam itself. The shell sends each call to the
//! store its seam is on, holds it to the deadline the core gave it (answering it late there, as a
//! seam does, docs/design/seams.md S7), hands each answer back, and sends each reply. A crash drops
//! the core with everything it held; a restart makes a new one, which knows only what its stores
//! kept. Answers to an earlier incarnation's calls are let go.

use std::collections::{BTreeMap, BTreeSet};
use std::time::{Duration, Instant, SystemTime};

use hyper_sim::{NodeId, SimError, StreamId, World};
use reliquary_api::request::Request;

use crate::call::{Answered, Call, Seam};
use crate::clock::ProcessClock;
use crate::ctx::Ctx;
use crate::ledger::Side;
use crate::wire::{CallId, Ev, JobId, Msg, Refused, RefusedKind, Reply};

/// What a node runs: the registry's core, as a state machine its shell drives. Its durable state is
/// in its stores; everything it holds itself is lost when its node crashes.
pub trait Core {
    /// A client's request arrives as job `job`. The core answers it with [`Io::reply`], now or later.
    fn request(&mut self, io: &mut Io<'_>, job: JobId, request: &Request);

    /// A call the core made is answered: by its store, or by its shell at its deadline.
    fn answered(&mut self, io: &mut Io<'_>, call: CallId, answer: Answered);
}

/// What a core is lent while it handles one input: its node's clocks, calls to make, replies to
/// send, and draws from its node's own stream of the world.
pub struct Io<'a> {
    now: Instant,
    wall: SystemTime,
    world: &'a mut World<Ev>,
    entropy: StreamId,
    calls: &'a mut Vec<(CallId, Call, Instant)>,
    replies: &'a mut Vec<(JobId, Reply)>,
    next_call: &'a mut u64,
    failed: &'a mut Option<SimError>,
}

impl Io<'_> {
    /// The node's monotonic now: what deadlines are derived from.
    #[must_use]
    pub fn now(&self) -> Instant {
        self.now
    }

    /// The node's wall clock.
    #[must_use]
    pub fn wall(&self) -> SystemTime {
        self.wall
    }

    /// Makes `call`, answered by `deadline` at the latest. `None` when the node has made every call
    /// number it can, which no run reaches.
    pub fn call(&mut self, call: Call, deadline: Instant) -> Option<CallId> {
        let id = CallId(*self.next_call);
        *self.next_call = self.next_call.checked_add(1)?;
        self.calls.push((id, call, deadline));
        Some(id)
    }

    /// Replies `reply` to the request of job `job`.
    pub fn reply(&mut self, job: JobId, reply: Reply) {
        self.replies.push((job, reply));
    }

    /// The calls made so far in this input, taken back before they are sent: what a core that
    /// wraps another sees of its calls, to hold, drop or answer itself (a defect planted to show a
    /// check finds it, docs/design/sim.md W11).
    pub fn take_calls(&mut self) -> Vec<(CallId, Call, Instant)> {
        std::mem::take(self.calls)
    }

    /// A call taken back with [`Io::take_calls`], made after all, under its own number.
    pub fn make_call(&mut self, id: CallId, call: Call, deadline: Instant) {
        self.calls.push((id, call, deadline));
    }

    /// The replies given so far in this input, taken back before they are sent.
    pub fn take_replies(&mut self) -> Vec<(JobId, Reply)> {
        std::mem::take(self.replies)
    }

    /// 128 bits no other draw of the run gives: an operation's identity, an object's key. They come
    /// from the node's stream of the world, so a run replays from its seed or its trace.
    pub fn identity(&mut self) -> u128 {
        let mut draw = || match self.world.below(self.entropy, u64::MAX) {
            Ok(word) => word,
            Err(error) => {
                self.failed.get_or_insert(error);
                0
            }
        };
        let high = u128::from(draw());
        let low = u128::from(draw());
        high.checked_shl(u64::BITS).unwrap_or(0) | low
    }
}

/// What a node's shell holds at most, as its maker states: requests in flight, and calls waiting
/// for answers. Past either, a request is refused `Busy` before its core sees it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NodeLimits {
    pub jobs: usize,
    pub calls: usize,
}

/// A call waiting for its answer: the store it went to, and when its deadline passes, as its
/// node's monotonic reading.
struct Waiting {
    to: NodeId,
    deadline_ns: u64,
    call: Call,
}

struct Up<C> {
    core: C,
    /// Requests in flight: who sent each, and the client's numbers for it.
    jobs: BTreeMap<JobId, (NodeId, u64, u32)>,
    /// Requests a client sent that the node took, so one delivered twice is taken once: the
    /// transport's, not the registry's, duty (docs/design/sim.md, W5).
    seen: BTreeSet<(NodeId, u64, u32)>,
    calls: BTreeMap<CallId, Waiting>,
    next_job: u64,
    next_call: u64,
}

/// A node's shell, holding its core while it is up.
pub struct NodeShell<C> {
    make: Box<dyn Fn() -> C>,
    up: Option<Up<C>>,
    incarnation: u32,
    records: NodeId,
    bytes: NodeId,
    /// Whether its records store, and its bytes store, are on its own machine: a power cut there
    /// falls at the store, which sees the write apply; elsewhere it falls here, as the write leaves.
    local: (bool, bool),
    limits: NodeLimits,
    pub(crate) clock: ProcessClock,
    entropy: StreamId,
}

impl<C: Core> NodeShell<C> {
    /// A node whose core `make` makes, at each start, reaching its records at `records` and its
    /// bytes at `bytes`.
    pub(crate) fn new(
        make: Box<dyn Fn() -> C>,
        (records, bytes): (NodeId, NodeId),
        local: (bool, bool),
        limits: NodeLimits,
        clock: ProcessClock,
        entropy: StreamId,
    ) -> Self {
        let mut shell = Self {
            make,
            up: None,
            incarnation: 0,
            records,
            bytes,
            local,
            limits,
            clock,
            entropy,
        };
        shell.start();
        shell
    }

    fn start(&mut self) {
        self.up = Some(Up {
            core: (self.make)(),
            jobs: BTreeMap::new(),
            seen: BTreeSet::new(),
            calls: BTreeMap::new(),
            next_job: 0,
            next_call: 0,
        });
    }

    /// Whether it is up.
    pub(crate) fn is_up(&self) -> bool {
        self.up.is_some()
    }

    /// Requests it holds that it has not answered, and calls it waits on: what a stuck run names.
    pub(crate) fn waiting(&self) -> (usize, usize) {
        self.up
            .as_ref()
            .map_or((0, 0), |up| (up.jobs.len(), up.calls.len()))
    }

    /// The node stops: its core and all it held are gone.
    pub(crate) fn down(&mut self) {
        self.up = None;
        self.clock.forget_sleeps();
    }

    /// The node starts again, with a new core.
    pub(crate) fn restart(&mut self) {
        self.incarnation = self.incarnation.saturating_add(1);
        self.start();
    }

    /// When its timer is to fire: the earliest deadline of a call it waits on.
    pub(crate) fn next_wake(&self) -> Option<u64> {
        self.up
            .as_ref()
            .and_then(|up| up.calls.values().map(|w| w.deadline_ns).min())
    }

    /// A message for the node.
    pub(crate) fn message(
        &mut self,
        ctx: &mut Ctx<'_>,
        from: NodeId,
        msg: Msg,
    ) -> Result<(), SimError> {
        let Some(up) = self.up.as_mut() else {
            ctx.ledger.let_go("to a process that is down");
            return Ok(());
        };
        match msg {
            Msg::Request {
                op,
                attempt,
                request,
            } => {
                if !up.seen.insert((from, op, attempt)) {
                    ctx.ledger.let_go("a request delivered again");
                    ctx.log(format_args!("request {op}.{attempt} taken already"));
                    return Ok(());
                }
                if up.jobs.len() >= self.limits.jobs || up.calls.len() >= self.limits.calls {
                    let reply = Reply::Refused(Refused {
                        kind: RefusedKind::Busy,
                        why: "the node holds its most requests".to_owned(),
                    });
                    let store = self.bytes;
                    return ctx.send(
                        from,
                        Msg::Reply {
                            op,
                            attempt,
                            reply,
                            store,
                        },
                    );
                }
                let job = JobId(up.next_job);
                up.next_job = up.next_job.saturating_add(1);
                up.jobs.insert(job, (from, op, attempt));
                self.drive(ctx, |core, io| core.request(io, job, &request))
            }
            Msg::Answer {
                id,
                incarnation,
                answer,
            } => {
                if incarnation != self.incarnation || up.calls.remove(&id).is_none() {
                    ctx.ledger.let_go("an answer no call waits for");
                    return Ok(());
                }
                self.drive(ctx, |core, io| core.answered(io, id, answer))
            }
            _ => {
                ctx.ledger.let_go("a message a node does not take");
                Ok(())
            }
        }
    }

    /// The node's process died: every client it was serving is told its connection was reset, its
    /// request's outcome unknown.
    pub(crate) fn reset_clients(&mut self, ctx: &mut Ctx<'_>) -> Result<(), SimError> {
        let Some(up) = self.up.as_mut() else {
            return Ok(());
        };
        let store = self.bytes;
        for (_, (client, op, attempt)) in std::mem::take(&mut up.jobs) {
            let reply = Reply::Refused(Refused {
                kind: RefusedKind::Unknown,
                why: "the connection was reset".to_owned(),
            });
            ctx.send(
                client,
                Msg::Reply {
                    op,
                    attempt,
                    reply,
                    store,
                },
            )?;
        }
        Ok(())
    }

    /// The store on `store` crashed: every call the node waits on there is answered now, as a reset
    /// connection answers, the call perhaps applied.
    pub(crate) fn reset_calls(&mut self, ctx: &mut Ctx<'_>, store: NodeId) -> Result<(), SimError> {
        let Some(up) = self.up.as_ref() else {
            return Ok(());
        };
        let reset: Vec<CallId> = up
            .calls
            .iter()
            .filter(|(_, w)| w.to == store)
            .map(|(id, _)| *id)
            .collect();
        for id in reset {
            let Some(waiting) = self.up.as_mut().and_then(|up| up.calls.remove(&id)) else {
                continue;
            };
            let answer = waiting.call.reset();
            self.drive(ctx, |core, io| core.answered(io, id, answer))?;
        }
        Ok(())
    }

    /// The node's timer fired: every call past its deadline is answered late.
    pub(crate) fn wake(&mut self, ctx: &mut Ctx<'_>) -> Result<(), SimError> {
        let now = self.clock.monotonic_ns();
        let Some(up) = self.up.as_mut() else {
            return Ok(());
        };
        let due: Vec<CallId> = up
            .calls
            .iter()
            .filter(|(_, w)| w.deadline_ns <= now)
            .map(|(id, _)| *id)
            .collect();
        for id in due {
            let Some(waiting) = self.up.as_mut().and_then(|up| up.calls.remove(&id)) else {
                continue;
            };
            ctx.log(format_args!("call {} late at its deadline", id.0));
            let answer = waiting.call.late();
            self.drive(ctx, |core, io| core.answered(io, id, answer))?;
        }
        Ok(())
    }

    /// The node's machine loses power at a write it was making.
    fn cut(ctx: &mut Ctx<'_>, method: &str, when: &str) -> Result<(), SimError> {
        let machine = ctx.machine;
        {
            let at = ctx.now();
            ctx.ledger.fault(crate::nemesis::POWER_CUT, at);
        }
        ctx.log(format_args!("power cut {when}: {method}"));
        ctx.act(crate::ctx::Action::PowerCut { machine });
        Ok(())
    }

    /// Runs `input` on the core, then makes the calls and sends the replies it gave.
    fn drive(
        &mut self,
        ctx: &mut Ctx<'_>,
        input: impl FnOnce(&mut C, &mut Io<'_>),
    ) -> Result<(), SimError> {
        use reliquary_core::seams::clock::Clock;
        let Some(up) = self.up.as_mut() else {
            return Ok(());
        };
        let mut calls = Vec::new();
        let mut replies = Vec::new();
        let mut failed = None;
        let now = self.clock.now();
        {
            let mut io = Io {
                now,
                wall: self.clock.wall(),
                world: ctx.world,
                entropy: self.entropy,
                calls: &mut calls,
                replies: &mut replies,
                next_call: &mut up.next_call,
                failed: &mut failed,
            };
            input(&mut up.core, &mut io);
        }
        if let Some(error) = failed {
            return Err(error);
        }
        let now_ns = self.clock.monotonic_ns();
        for (id, call, deadline) in calls {
            let deadline_ns = self.clock.nanos_of(deadline);
            if deadline < now {
                let short = u64::try_from(now.saturating_duration_since(deadline).as_nanos())
                    .unwrap_or(u64::MAX);
                let at = ctx.now();
                let name = crate::ctx::name_of(ctx.names, ctx.me).to_owned();
                ctx.ledger.negative_wait(&name, at, short);
            }
            let remaining = Duration::from_nanos(deadline_ns.saturating_sub(now_ns));
            let (to, local) = match call.seam() {
                Seam::Records => (self.records, self.local.0),
                Seam::Bytes => (self.bytes, self.local.1),
            };
            // A power cut armed for this machine's next write falls here when the store is
            // elsewhere: before the write leaves, or as it leaves.
            let cut = if call.writes() && !local {
                ctx.ledger.armed_cuts.remove(&ctx.machine)
            } else {
                None
            };
            if cut == Some(Side::Before) {
                return Self::cut(ctx, call.method(), "before its write left");
            }
            let msg = Msg::Call {
                id,
                incarnation: self.incarnation,
                call: call.clone(),
                remaining,
            };
            let method = call.method();
            up.calls.insert(
                id,
                Waiting {
                    to,
                    deadline_ns,
                    call,
                },
            );
            ctx.send(to, msg)?;
            if cut.is_some() {
                return Self::cut(ctx, method, "as its write left");
            }
        }
        for (job, reply) in replies {
            let Some((client, op, attempt)) = up.jobs.remove(&job) else {
                ctx.ledger.let_go("a reply to no request");
                continue;
            };
            if ctx.lose_answer() {
                continue;
            }
            let store = self.bytes;
            ctx.send(
                client,
                Msg::Reply {
                    op,
                    attempt,
                    reply,
                    store,
                },
            )?;
        }
        Ok(())
    }
}
