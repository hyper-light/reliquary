//! The stores (docs/design/sim.md, W4): Amphora's and Mantle's sides of a node's seams, each a
//! process of the world answering calls through its seam's own trait on RQ0.4's simulator. What
//! the simulator holds is the store's medium: it outlives a crash or a power cut of its machine,
//! which loses only what was in flight (calls queued, readings open, answers not sent).
//!
//! A store serves its calls in the order they arrive. A slow disk ([`Store::set_latency`]) serves
//! each one that much after the one before it, as a device's queue does. Before each write the
//! store asks the run's plan whether power is cut there: before the write (it never reaches the
//! medium) or after it (it applied, and its answer is never sent) (plan AC-RQ0.6.3), or, for a
//! piece of a put, under it, cut at a byte inside the piece, which keeps what was durable and
//! nothing after it (docs/design/seams.md BYTES-4).

use std::collections::{BTreeMap, VecDeque};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll, Waker};
use std::time::Duration;

use amphora_api::Records as _;
use hyper_sim::{NodeId, SimError, StreamId};
use reliquary_core::seams::bytes::sim::SimBytes;
use reliquary_core::seams::bytes::{
    Body, Bytes as _, Op as BytesOp, Redeemed, Refusal as BytesRefusal,
};
use reliquary_core::seams::clock::{Clock, SimClock};
use reliquary_core::seams::fault::Fault;
use reliquary_core::seams::records::SimRecords;

use crate::call::{
    Answered, BytesAnswer, BytesCall, Call, Opened, ReadingId, RecordsAnswer, RecordsCall,
    bytes_refused,
};
use crate::clock::ProcessClock;
use crate::ctx::{Action, Ctx};
use crate::ledger::Side;
use crate::wire::{CallId, Msg};

/// What a store holds at most, as its maker states: calls waiting their turn, and (a bytes store)
/// readings open. Past the first a call is refused at once; past the second the oldest reading is
/// let go, and a read of it is refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StoreLimits {
    pub queue: usize,
    pub readings: usize,
}

/// What a store was asked: a node's call, or a client's redemption of a grant.
enum Asked {
    Call {
        id: CallId,
        incarnation: u32,
        call: Call,
    },
    Redeem {
        op: u64,
        attempt: u32,
        grant: reliquary_core::seams::bytes::Grant,
        holder: String,
    },
}

struct Queued {
    from: NodeId,
    asked: Asked,
    remaining: Duration,
    /// When the store serves it, as its monotonic reading.
    ready_ns: u64,
}

/// What a bytes call's future ends with.
enum Done {
    Answer(BytesAnswer),
    /// A reading opened: its body, to keep.
    Opened(
        Result<
            (
                reliquary_core::seams::bytes::Meta,
                reliquary_core::seams::range::Span,
                Box<dyn Body>,
            ),
            BytesRefusal,
        >,
    ),
    /// A read: the bytes it gave, and the body, to keep for the next.
    Read {
        reading: ReadingId,
        body: Box<dyn Body>,
        result: Result<Arc<[u8]>, BytesRefusal>,
    },
    Redeemed(Result<Arc<[u8]>, BytesRefusal>),
}

type Work = Pin<Box<dyn Future<Output = Done> + Send>>;

/// A call whose answer its simulator holds on the store's clock (a delay armed in the simulator).
struct Parked {
    from: NodeId,
    asked: Asked,
    work: Work,
}

/// Which simulator a store keeps.
enum Medium {
    Records {
        sim: SimRecords,
        clock: SimClock,
    },
    Bytes {
        sim: SimBytes,
        readings: BTreeMap<ReadingId, Box<dyn Body>>,
        next_reading: u64,
        parked: Vec<Parked>,
    },
}

/// A store process.
pub(crate) struct Store {
    medium: Medium,
    pub(crate) clock: ProcessClock,
    up: bool,
    latency_ns: u64,
    queue: VecDeque<Queued>,
    limits: StoreLimits,
    /// The stream the store draws where under a piece power is cut.
    cuts: StreamId,
}

fn poll_once(work: &mut Work) -> Option<Done> {
    match work.as_mut().poll(&mut Context::from_waker(Waker::noop())) {
        Poll::Ready(done) => Some(done),
        Poll::Pending => None,
    }
}

impl Store {
    /// A records store over `sim`, whose clock `sim_clock` is kept at the store's monotonic reading.
    pub(crate) fn records(
        sim: SimRecords,
        sim_clock: SimClock,
        clock: ProcessClock,
        limits: StoreLimits,
        cuts: StreamId,
    ) -> Self {
        Self {
            medium: Medium::Records {
                sim,
                clock: sim_clock,
            },
            clock,
            up: true,
            latency_ns: 0,
            queue: VecDeque::new(),
            limits,
            cuts,
        }
    }

    /// A bytes store over `sim`, which was made on `clock`.
    pub(crate) fn bytes(
        sim: SimBytes,
        clock: ProcessClock,
        limits: StoreLimits,
        cuts: StreamId,
    ) -> Self {
        Self {
            medium: Medium::Bytes {
                sim,
                readings: BTreeMap::new(),
                next_reading: 0,
                parked: Vec::new(),
            },
            clock,
            up: true,
            latency_ns: 0,
            queue: VecDeque::new(),
            limits,
            cuts,
        }
    }

    pub(crate) fn is_up(&self) -> bool {
        self.up
    }

    /// Every call served this much after the one before it: a slow disk. Zero is the device's
    /// own pace.
    pub(crate) fn set_latency(&mut self, latency: Duration) {
        self.latency_ns = u64::try_from(latency.as_nanos()).unwrap_or(u64::MAX);
    }

    /// The machine lost power or the process crashed: what was in flight is gone; the medium stays.
    pub(crate) fn down(&mut self) {
        self.up = false;
        self.queue.clear();
        if let Medium::Bytes {
            readings, parked, ..
        } = &mut self.medium
        {
            readings.clear();
            parked.clear();
        }
        self.clock.forget_sleeps();
    }

    pub(crate) fn restart(&mut self) {
        self.up = true;
    }

    /// When its timer is to fire: the next call due, or a held answer's sleep.
    pub(crate) fn next_wake(&self) -> Option<u64> {
        let queued = self.queue.front().map(|q| q.ready_ns);
        let sleeping = match &self.medium {
            Medium::Bytes { parked, .. } if !parked.is_empty() => self.clock.next_wake(),
            _ => None,
        };
        queued.into_iter().chain(sleeping).min()
    }

    /// A message for the store.
    pub(crate) fn message(
        &mut self,
        ctx: &mut Ctx<'_>,
        from: NodeId,
        msg: Msg,
    ) -> Result<(), SimError> {
        if !self.up {
            ctx.ledger.let_go("to a process that is down");
            return Ok(());
        }
        let (asked, remaining) = match msg {
            Msg::Call {
                id,
                incarnation,
                call,
                remaining,
            } => (
                Asked::Call {
                    id,
                    incarnation,
                    call,
                },
                remaining,
            ),
            Msg::Redeem {
                op,
                attempt,
                grant,
                holder,
                remaining,
            } if matches!(self.medium, Medium::Bytes { .. }) => (
                Asked::Redeem {
                    op,
                    attempt,
                    grant,
                    holder,
                },
                remaining,
            ),
            _ => {
                ctx.ledger.let_go("a message a store does not take");
                return Ok(());
            }
        };
        if self.queue.len() >= self.limits.queue {
            ctx.log(format_args!("queue full: refused"));
            return self.refuse(ctx, from, asked);
        }
        let now = self.clock.monotonic_ns();
        let after = self.queue.back().map_or(now, |q| q.ready_ns.max(now));
        let ready_ns = after.saturating_add(self.latency_ns);
        self.queue.push_back(Queued {
            from,
            asked,
            remaining,
            ready_ns,
        });
        self.serve_due(ctx)
    }

    /// The store's timer fired: it serves what is due and polls held answers.
    pub(crate) fn wake(&mut self, ctx: &mut Ctx<'_>) -> Result<(), SimError> {
        if !self.up {
            return Ok(());
        }
        self.serve_due(ctx)?;
        self.poll_parked(ctx)
    }

    fn serve_due(&mut self, ctx: &mut Ctx<'_>) -> Result<(), SimError> {
        let now = self.clock.monotonic_ns();
        while self.up && self.queue.front().is_some_and(|q| q.ready_ns <= now) {
            let Some(q) = self.queue.pop_front() else {
                break;
            };
            self.serve(ctx, q)?;
        }
        Ok(())
    }

    /// Refuses `asked` at once, as a store whose queue is full: Amphora's `Unavailable`, Mantle's
    /// `Busy`. Nothing applies.
    fn refuse(&mut self, ctx: &mut Ctx<'_>, from: NodeId, asked: Asked) -> Result<(), SimError> {
        match asked {
            Asked::Call {
                id,
                incarnation,
                call,
            } => {
                let answer = match call {
                    Call::Records(records_call) => {
                        let refusal = amphora_api::Refusal::Unavailable(
                            "the store's queue is full".to_owned(),
                        );
                        Answered::Records(match records_call {
                            RecordsCall::Get { .. } => RecordsAnswer::Row(Err(refusal)),
                            RecordsCall::Scan { .. } => RecordsAnswer::Page(Err(refusal)),
                            RecordsCall::Commit { .. } | RecordsCall::Head => {
                                RecordsAnswer::Rev(Err(refusal))
                            }
                        })
                    }
                    Call::Bytes(bytes_call) => Answered::Bytes(bytes_refused(
                        &bytes_call,
                        BytesRefusal::Busy { retry_after: None },
                    )),
                };
                ctx.send(
                    from,
                    Msg::Answer {
                        id,
                        incarnation,
                        answer,
                    },
                )
            }
            Asked::Redeem { op, attempt, .. } => ctx.send(
                from,
                Msg::Redeemed {
                    op,
                    attempt,
                    content: Err(BytesRefusal::Busy { retry_after: None }),
                },
            ),
        }
    }

    /// The cut planned at the write about to apply on this machine: the sweep's, at this write's
    /// number, or the nemesis's, armed for the machine's next write.
    fn planned_cut(ctx: &mut Ctx<'_>, index: u64) -> Option<Side> {
        if let Some(cut) = ctx.ledger.cut
            && cut.write == index
        {
            return Some(cut.side);
        }
        ctx.ledger.armed_cuts.remove(&ctx.machine)
    }

    fn serve(&mut self, ctx: &mut Ctx<'_>, q: Queued) -> Result<(), SimError> {
        let Queued {
            from,
            asked,
            remaining,
            ..
        } = q;
        let mut cut = None;
        if let Asked::Call { call, .. } = &asked
            && call.writes()
        {
            let index = ctx.ledger.next_write();
            cut = Self::planned_cut(ctx, index);
            let seam = call.seam();
            let method = call.method();
            let at = ctx.now();
            let machine = ctx.machine;
            ctx.ledger.write(at, seam, method, machine);
            if cut == Some(Side::Before) {
                {
                    let at = ctx.now();
                    ctx.ledger.fault(crate::nemesis::POWER_CUT, at);
                }
                ctx.log(format_args!("power cut before write {index} ({method})"));
                ctx.act(Action::PowerCut { machine });
                return Ok(());
            }
            if cut == Some(Side::Under)
                && let (Call::Bytes(BytesCall::Send { at, piece, .. }), Medium::Bytes { sim, .. }) =
                    (call, &self.medium)
            {
                let within = u64::try_from(piece.len()).unwrap_or(u64::MAX);
                let offset = ctx.world.below(self.cuts, within)?;
                let byte = at.saturating_add(offset);
                if sim.arm(BytesOp::Send, Fault::Cut(byte)).is_ok() {
                    ctx.log(format_args!("power cut under write {index} at byte {byte}"));
                }
            }
        }
        match &mut self.medium {
            Medium::Records { sim, clock } => {
                let Asked::Call {
                    id,
                    incarnation,
                    call: Call::Records(call),
                } = asked
                else {
                    ctx.ledger.let_go("a bytes call at a records store");
                    return Ok(());
                };
                // The simulator's clock is kept at the store's monotonic reading, and the call's
                // deadline is what the caller had left, from the store's own now.
                let target = self.clock.monotonic_ns();
                let have = u64::try_from(clock.elapsed().as_nanos()).unwrap_or(u64::MAX);
                if target > have {
                    let _ = clock.advance(Duration::from_nanos(target.saturating_sub(have)));
                }
                let until = clock
                    .now()
                    .checked_add(remaining)
                    .unwrap_or_else(|| clock.now());
                let answer = Answered::Records(match call {
                    RecordsCall::Get { family, key, at } => {
                        RecordsAnswer::Row(sim.get(&family, &key, at, until))
                    }
                    RecordsCall::Scan { family, scan } => {
                        RecordsAnswer::Page(sim.scan(&family, scan, until))
                    }
                    RecordsCall::Commit { tx } => RecordsAnswer::Rev(sim.commit(tx, until)),
                    RecordsCall::Head => RecordsAnswer::Rev(sim.head(until)),
                });
                self.answer(ctx, from, id, incarnation, answer, cut)
            }
            Medium::Bytes { .. } => {
                let until = self
                    .clock
                    .now()
                    .checked_add(remaining)
                    .unwrap_or_else(|| self.clock.now());
                let mut work = self.work(asked_ref(&asked), until);
                match poll_once(&mut work) {
                    Some(done) => self.finish(ctx, from, asked, done, cut),
                    None => {
                        if let Medium::Bytes { parked, .. } = &mut self.medium {
                            parked.push(Parked { from, asked, work });
                        }
                        Ok(())
                    }
                }
            }
        }
    }

    fn poll_parked(&mut self, ctx: &mut Ctx<'_>) -> Result<(), SimError> {
        let Medium::Bytes { parked, .. } = &mut self.medium else {
            return Ok(());
        };
        let held = std::mem::take(parked);
        for mut p in held {
            match poll_once(&mut p.work) {
                Some(done) => self.finish(ctx, p.from, p.asked, done, None)?,
                None => {
                    if let Medium::Bytes { parked, .. } = &mut self.medium {
                        parked.push(p);
                    }
                }
            }
        }
        Ok(())
    }

    /// The future that answers a bytes call or a redemption, owning what it needs.
    fn work(&mut self, asked: AskedRef<'_>, until: std::time::Instant) -> Work {
        let Medium::Bytes { sim, readings, .. } = &mut self.medium else {
            return Box::pin(async {
                Done::Answer(BytesAnswer::Done(Err(BytesRefusal::Unavailable(
                    "not a bytes store".to_owned(),
                ))))
            });
        };
        let sim = sim.clone();
        match asked {
            AskedRef::Redeem { grant, holder } => {
                let (grant, holder) = (grant.clone(), holder.to_owned());
                Box::pin(async move { Done::Redeemed(redeem(&sim, &grant, &holder, until).await) })
            }
            AskedRef::Call(call) => match call.clone() {
                BytesCall::Begin { key, put } => Box::pin(async move {
                    Done::Answer(BytesAnswer::Upload(
                        named(sim.begin(&key, put, until)).await,
                    ))
                }),
                BytesCall::Send { upload, at, piece } => Box::pin(async move {
                    Done::Answer(BytesAnswer::Progress(
                        named(sim.send(&upload, at, &piece, until)).await,
                    ))
                }),
                BytesCall::Status { upload } => Box::pin(async move {
                    Done::Answer(BytesAnswer::Progress(
                        named(sim.status(&upload, until)).await,
                    ))
                }),
                BytesCall::Finish { upload, whole } => Box::pin(async move {
                    Done::Answer(BytesAnswer::Meta(
                        named(sim.finish(&upload, whole, until)).await,
                    ))
                }),
                BytesCall::Abort { upload } => Box::pin(async move {
                    Done::Answer(BytesAnswer::Done(named(sim.abort(&upload, until)).await))
                }),
                BytesCall::Get { key, range } => Box::pin(async move {
                    Done::Opened(
                        named(sim.get(&key, range, until))
                            .await
                            .map(|reading| (reading.meta, reading.span, reading.body)),
                    )
                }),
                BytesCall::Read { reading, length } => match readings.remove(&reading) {
                    Some(mut body) => Box::pin(async move {
                        let mut buffer = vec![0; length];
                        let read = named(body.read(&mut buffer, until)).await;
                        let result = read.map(|n| {
                            buffer.truncate(n);
                            Arc::from(buffer)
                        });
                        Done::Read {
                            reading,
                            body,
                            result,
                        }
                    }),
                    None => Box::pin(async move {
                        Done::Answer(BytesAnswer::Read(Err(BytesRefusal::NotFound(format!(
                            "reading {}",
                            reading.0
                        )))))
                    }),
                },
                BytesCall::Close { reading } => {
                    let closed = readings.remove(&reading).is_some();
                    Box::pin(async move {
                        Done::Answer(BytesAnswer::Done(if closed {
                            Ok(())
                        } else {
                            Err(BytesRefusal::NotFound(format!("reading {}", reading.0)))
                        }))
                    })
                }
                BytesCall::Head { key } => Box::pin(async move {
                    Done::Answer(BytesAnswer::Meta(named(sim.head(&key, until)).await))
                }),
                BytesCall::Delete { key } => Box::pin(async move {
                    Done::Answer(BytesAnswer::Deleted(named(sim.delete(&key, until)).await))
                }),
                BytesCall::Grant {
                    key,
                    access,
                    holder,
                    expires,
                } => Box::pin(async move {
                    Done::Answer(BytesAnswer::Grant(
                        named(sim.grant(&key, access, &holder, expires, until)).await,
                    ))
                }),
            },
            AskedRef::Records => Box::pin(async {
                Done::Answer(BytesAnswer::Done(Err(BytesRefusal::Unavailable(
                    "a records call at a bytes store".to_owned(),
                ))))
            }),
        }
    }

    /// A bytes call's future ended: keep what it opened, then answer.
    fn finish(
        &mut self,
        ctx: &mut Ctx<'_>,
        from: NodeId,
        asked: Asked,
        done: Done,
        cut: Option<Side>,
    ) -> Result<(), SimError> {
        let answer = match done {
            Done::Answer(answer) => answer,
            Done::Opened(opened) => BytesAnswer::Opened(match opened {
                Ok((meta, span, body)) => match self.keep_reading(ctx, body) {
                    Some(reading) => Ok(Opened {
                        meta,
                        span,
                        reading,
                    }),
                    None => Err(BytesRefusal::Full {
                        what: "readings",
                        limit: u64::try_from(self.limits.readings).unwrap_or(u64::MAX),
                    }),
                },
                Err(refusal) => Err(refusal),
            }),
            Done::Read {
                reading,
                body,
                result,
            } => {
                // A reading read to its end, or refused, is let go; one with more is kept.
                if let Ok(bytes) = &result
                    && !bytes.is_empty()
                    && let Medium::Bytes { readings, .. } = &mut self.medium
                {
                    readings.insert(reading, body);
                }
                BytesAnswer::Read(result)
            }
            Done::Redeemed(content) => {
                if let Asked::Redeem {
                    op, attempt, grant, ..
                } = asked
                {
                    let at = ctx.now();
                    let honored = content.is_ok();
                    let expired = matches!(content, Err(BytesRefusal::Expired));
                    ctx.ledger
                        .grant(&grant.tag, &grant.key, at, honored, expired);
                    if ctx.lose_answer() {
                        return Ok(());
                    }
                    return ctx.send(
                        from,
                        Msg::Redeemed {
                            op,
                            attempt,
                            content,
                        },
                    );
                }
                ctx.ledger.let_go("a redemption with no redeemer");
                return Ok(());
            }
        };
        let Asked::Call {
            id, incarnation, ..
        } = asked
        else {
            ctx.ledger.let_go("an answer to no call");
            return Ok(());
        };
        self.answer(ctx, from, id, incarnation, Answered::Bytes(answer), cut)
    }

    fn keep_reading(&mut self, ctx: &mut Ctx<'_>, body: Box<dyn Body>) -> Option<ReadingId> {
        let Medium::Bytes {
            readings,
            next_reading,
            ..
        } = &mut self.medium
        else {
            return None;
        };
        if self.limits.readings == 0 {
            return None;
        }
        // Past its most readings, the store lets the oldest go: a later read of it is refused.
        while readings.len() >= self.limits.readings {
            if readings.pop_first().is_none() {
                break;
            }
            ctx.ledger.let_go("the oldest reading, for room");
        }
        let id = ReadingId(*next_reading);
        *next_reading = next_reading.checked_add(1)?;
        readings.insert(id, body);
        Some(id)
    }

    fn answer(
        &mut self,
        ctx: &mut Ctx<'_>,
        from: NodeId,
        id: CallId,
        incarnation: u32,
        answer: Answered,
        cut: Option<Side>,
    ) -> Result<(), SimError> {
        if matches!(cut, Some(Side::After | Side::Under)) {
            let machine = ctx.machine;
            {
                let at = ctx.now();
                ctx.ledger.fault(crate::nemesis::POWER_CUT, at);
            }
            ctx.log(format_args!(
                "power cut after a write: {}",
                answer.summary()
            ));
            ctx.act(Action::PowerCut { machine });
            return Ok(());
        }
        ctx.ledger.answered(&answer);
        if ctx.lose_answer() {
            return Ok(());
        }
        ctx.send(
            from,
            Msg::Answer {
                id,
                incarnation,
                answer,
            },
        )
    }
}

/// What a bytes call's future is made from, borrowed.
enum AskedRef<'a> {
    Call(&'a BytesCall),
    Redeem {
        grant: &'a reliquary_core::seams::bytes::Grant,
        holder: &'a str,
    },
    Records,
}

fn asked_ref(asked: &Asked) -> AskedRef<'_> {
    match asked {
        Asked::Call {
            call: Call::Bytes(call),
            ..
        } => AskedRef::Call(call),
        Asked::Call { .. } => AskedRef::Records,
        Asked::Redeem { grant, holder, .. } => AskedRef::Redeem { grant, holder },
    }
}

/// Mantle's side of a grant: the reading it gives, read whole into one answer.
async fn redeem(
    sim: &SimBytes,
    grant: &reliquary_core::seams::bytes::Grant,
    holder: &str,
    until: std::time::Instant,
) -> Result<Arc<[u8]>, BytesRefusal> {
    let redeemed = named(sim.redeem(grant, holder, until)).await?;
    let Redeemed::Reading(mut reading) = redeemed else {
        return Err(BytesRefusal::Invalid {
            what: "a grant redeemed for reading",
            why: "it grants a put".to_owned(),
        });
    };
    let length = usize::try_from(reading.span.length).map_err(|_| BytesRefusal::TooLarge {
        what: "an object read whole",
        size: reading.span.length,
        limit: u64::try_from(usize::MAX).unwrap_or(u64::MAX),
    })?;
    let mut content = Vec::new();
    content
        .try_reserve_exact(length)
        .map_err(|e| BytesRefusal::Unavailable(e.to_string()))?;
    let mut buffer = vec![0; length];
    while content.len() < length {
        let n = named(reading.body.read(&mut buffer, until)).await?;
        if n == 0 {
            break;
        }
        content.extend(buffer.iter().take(n));
    }
    Ok(Arc::from(content))
}

/// A seam's boxed future, polled by name: the recursion gate reads the call as a poll, where an
/// `.await` on the box is a call through a vtable's slot with no method's name at its site
/// (scripts/check-recursion.py; reliquary-core's `fault::held` does the same).
async fn named<T>(mut work: reliquary_core::seams::BoxFuture<'_, T>) -> T {
    std::future::poll_fn(|cx| work.as_mut().poll(cx)).await
}
