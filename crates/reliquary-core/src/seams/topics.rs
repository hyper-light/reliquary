//! Events: Quiver's `Topics` (`quiver-api`), the trait Reliquary publishes its events through
//! (architecture D26; plan RQ7.13). Quiver gives the trait, a simulator and a suite of its own;
//! [`FaultyTopics`] wraps any implementation of it (Quiver's simulator here, its node in a test of
//! the real thing) and injects what Quiver's simulator does not: each refusal the trait names, an
//! answer delayed or never given, an answer lost after its call applied, and a read or a
//! subscription cut at any message. [`cases`] are the clauses Reliquary relies on.
//!
//! Quiver's methods take no deadline: its futures are dropped by their caller. A caller holds each
//! call to the deadline it derived with [`within`](crate::seams::clock::within), which drops the
//! call there; what the call did by then, the caller learns by reading (TOPICS-5).

use std::collections::VecDeque;
use std::future::poll_fn;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

use parking_lot::Mutex;
use quiver_api::{
    Batch, Message, MessageRef, Next, Offset, Refusal, Start, Subscription, Topic, Topics,
};

use crate::seams::BoxFuture;
use crate::seams::clock::{self, Clock, Past, SimClock};
use crate::seams::conformance::{Case, Failure, Fixture, ensure, fine, refused, same};
use crate::seams::fault::{CUT, Counts, DELAY, Fault, Faults, Kind, LOSE, Unarmed, VANISH};

impl Kind for Refusal {
    fn kind(&self) -> &'static str {
        match self {
            Self::NoTopic(_) => "NoTopic",
            Self::Exists(_) => "Exists",
            Self::Invalid { .. } => "Invalid",
            Self::TooLarge { .. } => "TooLarge",
            Self::Full { .. } => "Full",
            Self::Behind { .. } => "Behind",
            Self::Unsupported(_) => "Unsupported",
            Self::Unavailable(_) => "Unavailable",
            Self::NoQuorum(_) => "NoQuorum",
        }
    }
}

/// The faults the topics simulator injects (`docs/design/seams.md`, "Topics"): each refusal
/// Quiver's trait names, an answer delayed past its caller's deadline, a request that never
/// arrives, an answer lost after its call applied, and a read or a subscription cut at any message.
pub const FAULTS: &[&str] = &[
    "NoTopic",
    "Exists",
    "Invalid",
    "TooLarge",
    "Full",
    "Behind",
    "Unsupported",
    "Unavailable",
    "NoQuorum",
    DELAY,
    VANISH,
    LOSE,
    CUT,
];

/// The operations a fault can be armed for: each method of `Topics`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Op {
    Create,
    Delete,
    Publish,
    Read,
    Wait,
    Bounds,
    Subscribe,
}

fn fits(op: Op, fault: &Fault<Refusal>) -> bool {
    match fault {
        Fault::Cut(_) => matches!(op, Op::Read | Op::Subscribe),
        Fault::Lose => matches!(op, Op::Create | Op::Delete | Op::Publish),
        Fault::Refuse(_) | Fault::Delay(_) | Fault::Vanish => true,
    }
}

fn unavailable(why: impl std::fmt::Display) -> Refusal {
    Refusal::Unavailable(format!("{why}"))
}

/// Any `Topics`, with faults injected between it and its caller. Its held answers wait on `clock`.
pub struct FaultyTopics<T> {
    inner: T,
    clock: Arc<dyn Clock>,
    faults: Mutex<Faults<Op, Refusal>>,
}

impl<T: Topics> FaultyTopics<T> {
    /// `inner`, with no fault armed.
    pub fn new(inner: T, clock: Arc<dyn Clock>) -> Self {
        Self {
            inner,
            clock,
            faults: Mutex::new(Faults::default()),
        }
    }

    /// Arms `fault` for the next call of `op` (a cut, for the next read or subscription that
    /// reaches its message).
    pub fn arm(&self, op: Op, fault: Fault<Refusal>) -> Result<(), Unarmed> {
        self.faults.lock().arm(op, fault, fits)
    }

    /// How many calls of `op` were made.
    #[must_use]
    pub fn attempts(&self, op: Op) -> u64 {
        self.faults.lock().attempts(op)
    }

    /// What it injected, by kind.
    #[must_use]
    pub fn injected(&self) -> Counts {
        self.faults.lock().injected().clone()
    }

    /// How a call answers, with the fault armed for `op`: refused at once, or never, and then
    /// `call` is never made, as a store may apply a call when it is made; or `call` made, and its
    /// answer lost, held on the clock, or given as it comes.
    async fn answer<'a, V: Send>(
        &'a self,
        op: Op,
        call: impl FnOnce() -> BoxFuture<'a, Result<V, Refusal>> + Send,
    ) -> Result<V, Refusal> {
        let fault = self.faults.lock().call(op);
        match fault {
            Some(Fault::Refuse(refusal)) => Err(refusal),
            Some(Fault::Vanish) => {
                // No answer comes: its caller's deadline ends the call.
                poll_fn(|_| Poll::<()>::Pending).await;
                Err(unavailable("a call that never answers answered"))
            }
            Some(Fault::Lose) => {
                let _lost = call().await;
                Err(unavailable(
                    "the answer was lost: the call may have applied",
                ))
            }
            Some(Fault::Delay(by)) => {
                let answered = call().await;
                match self.clock.now().checked_add(by) {
                    Some(at) => {
                        let mut sleep = self.clock.sleep_until(at);
                        poll_fn(|cx| sleep.as_mut().poll(cx))
                            .await
                            .map_err(|e| unavailable(format!("the clock: {e}")))?;
                        answered
                    }
                    None => {
                        poll_fn(|_| Poll::<()>::Pending).await;
                        answered
                    }
                }
            }
            Some(Fault::Cut(_)) | None => call().await,
        }
    }

    /// A read of `name` cut at the message an armed cut names, if it reaches one.
    async fn read_cut(&self, name: &str, from: Offset, limit: usize) -> Result<Batch, Refusal> {
        let mut batch = self.inner.read(name, from, limit).await?;
        let length = u64::try_from(batch.messages.len()).unwrap_or(u64::MAX);
        if let Some(at) = self.faults.lock().cut(Op::Read, 0, length) {
            batch
                .messages
                .truncate(usize::try_from(at).unwrap_or(usize::MAX));
            batch.next = batch
                .messages
                .last()
                .and_then(|m| m.offset.checked_add(1))
                .unwrap_or(from);
        }
        Ok(batch)
    }

    /// A subscription from `start` that gives `at` messages, and then ends, as one whose stream
    /// was cut there: built from reads, the messages its topic keeps now.
    async fn cut_subscription(
        &self,
        name: &str,
        start: Start,
        at: u64,
    ) -> Result<Box<dyn Subscription>, Refusal> {
        let (first, end) = self.inner.bounds(name).await?;
        let mut from = start.at(first, end);
        let mut given = VecDeque::new();
        if from < first {
            given.push_back(Given::Lag { from, to: first });
            from = first;
        }
        let mut left = at;
        // Each read gives at least one message or ends the loop, so it runs at most `at` times.
        while left > 0 {
            let limit = usize::try_from(left).unwrap_or(usize::MAX);
            let batch = self
                .inner
                .read(name, from, quiver_api::check_limit(limit)?)
                .await?;
            if batch.messages.is_empty() {
                break;
            }
            left = left.saturating_sub(u64::try_from(batch.messages.len()).unwrap_or(u64::MAX));
            from = batch.next;
            given.extend(batch.messages.into_iter().map(Given::Message));
        }
        given.push_back(Given::End);
        Ok(Box::new(CutSubscription {
            given,
            held: None,
            position: start.at(first, end),
        }))
    }
}

/// What a cut subscription gives, in order.
enum Given {
    Message(Message),
    Lag { from: Offset, to: Offset },
    End,
}

/// A subscription whose stream was cut: it gives what came before the cut, and then ends.
struct CutSubscription {
    given: VecDeque<Given>,
    /// The message taken last, which the take borrows until the next.
    held: Option<Message>,
    position: Offset,
}

impl Subscription for CutSubscription {
    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<()> {
        Poll::Ready(())
    }

    fn take(&mut self) -> Option<Next<'_>> {
        match self.given.pop_front() {
            Some(Given::Message(message)) => {
                self.position = message.offset.saturating_add(1);
                let message = self.held.insert(message);
                Some(Next::Message(MessageRef {
                    offset: message.offset,
                    key: message.key.as_deref(),
                    payload: &message.payload,
                    at_us: message.at_us,
                }))
            }
            Some(Given::Lag { from, to }) => {
                self.position = to;
                Some(Next::Lag { from, to })
            }
            // Ended, it stays ended.
            Some(Given::End) | None => {
                self.given.clear();
                Some(Next::Ended)
            }
        }
    }

    fn position(&self) -> Offset {
        self.position
    }
}

impl<T: Topics> Topics for FaultyTopics<T> {
    fn create<'a>(&'a self, name: &'a str, topic: Topic) -> BoxFuture<'a, Result<(), Refusal>> {
        Box::pin(self.answer(Op::Create, move || self.inner.create(name, topic)))
    }

    fn delete<'a>(&'a self, name: &'a str) -> BoxFuture<'a, Result<(), Refusal>> {
        Box::pin(self.answer(Op::Delete, move || self.inner.delete(name)))
    }

    fn publish<'a>(
        &'a self,
        name: &'a str,
        key: Option<&'a [u8]>,
        payload: &'a [u8],
    ) -> BoxFuture<'a, Result<Offset, Refusal>> {
        Box::pin(self.answer(Op::Publish, move || self.inner.publish(name, key, payload)))
    }

    fn read<'a>(
        &'a self,
        name: &'a str,
        from: Offset,
        limit: usize,
    ) -> BoxFuture<'a, Result<Batch, Refusal>> {
        Box::pin(self.answer(Op::Read, move || Box::pin(self.read_cut(name, from, limit))))
    }

    fn wait<'a>(&'a self, name: &'a str, from: Offset) -> BoxFuture<'a, Result<(), Refusal>> {
        Box::pin(self.answer(Op::Wait, move || self.inner.wait(name, from)))
    }

    fn bounds<'a>(&'a self, name: &'a str) -> BoxFuture<'a, Result<(Offset, Offset), Refusal>> {
        Box::pin(self.answer(Op::Bounds, move || self.inner.bounds(name)))
    }

    fn subscribe<'a>(
        &'a self,
        name: &'a str,
        start: Start,
    ) -> BoxFuture<'a, Result<Box<dyn Subscription>, Refusal>> {
        Box::pin(self.answer(Op::Subscribe, move || {
            // A cut armed for a subscription is taken whatever message it names: the
            // subscription is made to stop there.
            let cut = self.faults.lock().cut(Op::Subscribe, 0, u64::MAX);
            match cut {
                Some(at) => Box::pin(self.cut_subscription(name, start, at)),
                None => self.inner.subscribe(name, start),
            }
        }))
    }
}

// -------------------------------------------------------------------------------- conformance

/// What a topics store's conformance suite runs against.
pub trait TopicsFixture: Fixture {
    fn topics(&self) -> &dyn Topics;
    /// The clock its callers' deadlines are on.
    fn clock(&self) -> &dyn Clock;
    /// Arms `fault` for the store's next call of `op`.
    fn arm(&self, op: Op, fault: Fault<Refusal>) -> Result<(), Failure>;
    /// How many calls of `op` the store has had.
    fn attempts(&self, op: Op) -> u64;
    /// How long a case's caller waits for an answer: each deadline is the clock's now plus this.
    fn patience(&self) -> Duration;
    /// The topic a case creates: of a class the store serves, keeping fewer messages than a case
    /// publishes ([`TopicsFixture::messages`]).
    fn topic(&self) -> Topic;
    /// How many messages a case publishes.
    fn messages(&self) -> u64;
}

/// The fixture for a [`FaultyTopics`] around Quiver's simulator: the clock it was made with, and
/// what the test that makes it states.
pub struct SimTopicsFixture {
    pub topics: FaultyTopics<quiver_api::sim::SimTopics>,
    pub clock: SimClock,
    pub patience: Duration,
    pub steps: u64,
    pub topic: Topic,
    pub messages: u64,
}

impl Fixture for SimTopicsFixture {
    fn drive<'a, V>(&'a self, work: BoxFuture<'a, V>) -> Result<V, Failure> {
        self.clock
            .run(work, self.steps)
            .map_err(|stalled| stalled.to_string())
    }

    fn injected(&self) -> Counts {
        self.topics.injected()
    }
}

impl TopicsFixture for SimTopicsFixture {
    fn topics(&self) -> &dyn Topics {
        &self.topics
    }

    fn clock(&self) -> &dyn Clock {
        &self.clock
    }

    fn arm(&self, op: Op, fault: Fault<Refusal>) -> Result<(), Failure> {
        self.topics.arm(op, fault).map_err(|e| e.to_string())
    }

    fn attempts(&self, op: Op) -> u64 {
        self.topics.attempts(op)
    }

    fn patience(&self) -> Duration {
        self.patience
    }

    fn topic(&self) -> Topic {
        self.topic
    }

    fn messages(&self) -> u64 {
        self.messages
    }
}

fn deadline<F: TopicsFixture>(f: &F) -> Result<Instant, Failure> {
    f.clock()
        .now()
        .checked_add(f.patience())
        .ok_or_else(|| "a deadline past what the platform's time can say".to_owned())
}

/// Message `n`'s key and payload.
fn message(n: u64) -> (Vec<u8>, Vec<u8>) {
    (
        format!("key-{n}").into_bytes(),
        format!("payload-{n}").into_bytes(),
    )
}

/// Creates topic `name` and publishes `count` messages to it: their offsets.
async fn filled<F: TopicsFixture>(f: &F, name: &str, count: u64) -> Result<Vec<Offset>, Failure> {
    fine("a create", f.topics().create(name, f.topic()).await)?;
    let mut offsets = Vec::new();
    for n in 0..count {
        let (key, payload) = message(n);
        offsets.push(fine(
            "a publish",
            f.topics().publish(name, Some(&key), &payload).await,
        )?);
    }
    Ok(offsets)
}

/// Every message of `name` from `from`, read until the end: each read once.
async fn read_all<F: TopicsFixture>(
    f: &F,
    name: &str,
    from: Offset,
) -> Result<Vec<Message>, Failure> {
    let mut out = Vec::new();
    let mut at = from;
    for _ in 0..=f.messages() {
        let batch = fine(
            "a read",
            f.topics().read(name, at, quiver_api::READ_MAX).await,
        )?;
        let (_, end) = fine("its bounds", f.topics().bounds(name).await)?;
        at = batch.next;
        out.extend(batch.messages);
        if at >= end {
            break;
        }
    }
    Ok(out)
}

fn t1<F: TopicsFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        let kept = f.topic().retain_messages;
        let offsets = filled(f, "t1", kept).await?;
        let expected: Vec<Offset> = (0..kept).collect();
        same("the offsets given", offsets, expected)?;
        let messages = read_all(f, "t1", 0).await?;
        same(
            "messages read",
            u64::try_from(messages.len()).unwrap_or(u64::MAX),
            kept,
        )?;
        for m in &messages {
            let (key, payload) = message(m.offset);
            ensure(
                m.key.as_deref() == Some(key.as_slice()) && m.payload == payload,
                || format!("message {} is not the one published there", m.offset),
            )?;
        }
        Ok(())
    })
}

fn t2<F: TopicsFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        let topic = f.topic();
        filled(f, "t2", 1).await?;
        fine(
            "a create again with the same settings",
            f.topics().create("t2", topic).await,
        )?;
        same(
            "its bounds after it",
            fine("bounds", f.topics().bounds("t2").await)?,
            (0, 1),
        )?;
        let other = Topic {
            retain_messages: topic.retain_messages.saturating_add(1),
            ..topic
        };
        let refusal = refused(
            "a create with other settings",
            f.topics().create("t2", other).await,
        )?;
        same("its refusal", refusal, Refusal::Exists("t2".into()))
    })
}

fn t3<F: TopicsFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        filled(f, "t3", f.messages()).await?;
        let (first, end) = fine("bounds", f.topics().bounds("t3").await)?;
        ensure(first > 0 && end == f.messages(), || {
            format!(
                "a topic keeping {} of {} messages keeps {first}..{end}",
                f.topic().retain_messages,
                f.messages()
            )
        })?;
        same(
            "a read from offset 0",
            f.topics()
                .read("t3", 0, quiver_api::READ_MAX)
                .await
                .map(|b| b.next),
            Err(Refusal::Behind { asked: 0, first }),
        )
    })
}

fn t4<F: TopicsFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        filled(f, "t4", 0).await?;
        let (key, payload) = message(0);
        f.arm(Op::Publish, Fault::Lose)?;
        let refusal = refused(
            "a publish whose answer is lost",
            f.topics().publish("t4", Some(&key), &payload).await,
        )?;
        ensure(matches!(refusal, Refusal::Unavailable(_)), || {
            format!("refused {refusal:?}, not Unavailable")
        })?;
        same(
            "the bounds after it",
            fine("bounds", f.topics().bounds("t4").await)?,
            (0, 1),
        )?;
        let kept = read_all(f, "t4", 0).await?;
        ensure(
            kept.first().map(|m| m.payload.as_slice()) == Some(payload.as_slice()),
            || "the publish whose answer was lost is not in the topic".into(),
        )
    })
}

fn t5<F: TopicsFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        filled(f, "t5", 0).await?;
        let past = f
            .patience()
            .checked_add(f.patience())
            .ok_or_else(|| "a patience as long as time".to_owned())?;
        let (key, payload) = message(0);
        f.arm(Op::Publish, Fault::Delay(past))?;
        let due = deadline(f)?;
        same(
            "a publish held past its caller's deadline",
            clock::within(
                f.clock(),
                due,
                f.topics().publish("t5", Some(&key), &payload),
            )
            .await,
            Err(Past::Deadline),
        )?;
        ensure(f.clock().now() >= due, || {
            "dropped before its deadline".into()
        })?;
        same(
            "the bounds after it",
            fine("bounds", f.topics().bounds("t5").await)?,
            (0, 1),
        )?;
        f.arm(Op::Publish, Fault::Vanish)?;
        same(
            "a publish that never arrives",
            clock::within(
                f.clock(),
                deadline(f)?,
                f.topics().publish("t5", Some(&key), &payload),
            )
            .await,
            Err(Past::Deadline),
        )?;
        same(
            "the bounds after it",
            fine("bounds", f.topics().bounds("t5").await)?,
            (0, 1),
        )?;
        f.arm(Op::Bounds, Fault::Delay(f.patience()))?;
        same(
            "bounds answered at the deadline",
            clock::within(f.clock(), deadline(f)?, f.topics().bounds("t5")).await,
            Ok(Ok((0, 1))),
        )
    })
}

fn t6<F: TopicsFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        let kept = f.topic().retain_messages;
        filled(f, "t6", kept).await?;
        for cut in 0..=kept {
            f.arm(Op::Read, Fault::Cut(cut))?;
            let batch = fine(
                "a cut read",
                f.topics().read("t6", 0, quiver_api::READ_MAX).await,
            )?;
            same(
                &format!("messages before a cut at {cut}"),
                u64::try_from(batch.messages.len()).unwrap_or(u64::MAX),
                cut,
            )?;
            same(
                &format!("where a read cut at {cut} goes on"),
                batch.next,
                cut,
            )?;
            let mut offsets: Vec<Offset> = batch.messages.iter().map(|m| m.offset).collect();
            offsets.extend(
                read_all(f, "t6", batch.next)
                    .await?
                    .iter()
                    .map(|m| m.offset),
            );
            let expected: Vec<Offset> = (0..kept).collect();
            same(
                &format!("offsets read around a cut at {cut}"),
                offsets,
                expected,
            )?;
        }
        Ok(())
    })
}

fn t7<F: TopicsFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        let kept = f.topic().retain_messages;
        filled(f, "t7", kept).await?;
        for cut in 0..=kept {
            f.arm(Op::Subscribe, Fault::Cut(cut))?;
            let mut offsets = Vec::new();
            let mut subscription = fine(
                "a cut subscription",
                f.topics().subscribe("t7", Start::Oldest).await,
            )?;
            for _ in 0..=kept {
                match subscription.take() {
                    Some(Next::Message(m)) => offsets.push(m.offset),
                    Some(Next::Ended) => break,
                    other => return Err(format!("a cut subscription gave {other:?}")),
                }
            }
            same(
                &format!("messages before a cut at {cut}"),
                u64::try_from(offsets.len()).unwrap_or(u64::MAX),
                cut,
            )?;
            let position = subscription.position();
            drop(subscription);
            let mut rest = fine(
                "a subscription from there",
                f.topics().subscribe("t7", Start::At(position)).await,
            )?;
            for _ in 0..kept.saturating_sub(cut) {
                match rest.take() {
                    Some(Next::Message(m)) => offsets.push(m.offset),
                    other => return Err(format!("the rest gave {other:?}")),
                }
            }
            let expected: Vec<Offset> = (0..kept).collect();
            same(&format!("offsets around a cut at {cut}"), offsets, expected)?;
        }
        Ok(())
    })
}

/// One refusal of every kind the trait names.
fn every_refusal() -> Vec<Refusal> {
    vec![
        Refusal::NoTopic("t".into()),
        Refusal::Exists("t".into()),
        Refusal::Invalid {
            what: "a topic name",
            why: "w".into(),
        },
        Refusal::TooLarge {
            what: "a payload",
            size: 0,
            limit: 0,
        },
        Refusal::Full {
            what: "topics",
            limit: 0,
        },
        Refusal::Behind { asked: 0, first: 0 },
        Refusal::Unsupported(quiver_api::Class::Persisted),
        Refusal::Unavailable("u".into()),
        Refusal::NoQuorum("q".into()),
    ]
}

fn t8<F: TopicsFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        filled(f, "t8", 0).await?;
        for refusal in every_refusal() {
            let before = f.attempts(Op::Bounds);
            f.arm(Op::Bounds, Fault::Refuse(refusal.clone()))?;
            same(
                "bounds the store refuses",
                f.topics().bounds("t8").await,
                Err(refusal.clone()),
            )?;
            same(
                &format!("attempts of bounds refused {refusal:?}"),
                f.attempts(Op::Bounds),
                before.saturating_add(1),
            )?;
        }
        let (key, payload) = message(0);
        f.arm(Op::Publish, Fault::Refuse(Refusal::NoQuorum("q".into())))?;
        refused(
            "a publish the store refuses",
            f.topics().publish("t8", Some(&key), &payload).await,
        )?;
        same(
            "the bounds after a refused publish",
            fine("bounds", f.topics().bounds("t8").await)?,
            (0, 0),
        )
    })
}

/// The clauses of Quiver's `Topics` that Reliquary relies on (`docs/design/seams.md`, "Topics").
#[must_use]
pub fn cases<F: TopicsFixture>() -> Vec<Case<F>> {
    vec![
        Case {
            id: "TOPICS-1",
            says: "a publish is given the topic's next offset, and a read from an offset gives the messages from it in publish order, with their keys and payloads",
            check: t1::<F>,
        },
        Case {
            id: "TOPICS-2",
            says: "a topic created again with the same settings is unchanged; with other settings it is refused Exists",
            check: t2::<F>,
        },
        Case {
            id: "TOPICS-3",
            says: "a read from an offset retention let go is refused Behind, naming the first kept",
            check: t3::<F>,
        },
        Case {
            id: "TOPICS-4",
            says: "a publish whose answer is lost after it applied is refused Unavailable, and the topic's bounds and a read show it was kept",
            check: t4::<F>,
        },
        Case {
            id: "TOPICS-5",
            says: "a call held to its caller's deadline is dropped there when its answer is delayed past it or never comes, and what it did is learned by reading",
            check: t5::<F>,
        },
        Case {
            id: "TOPICS-6",
            says: "a read cut at any message gives the messages before the cut and where to go on from; reading on gives the rest, none twice",
            check: t6::<F>,
        },
        Case {
            id: "TOPICS-7",
            says: "a subscription cut at any message gives every message before the cut, then ends; one from its position gives the rest, none skipped and none twice",
            check: t7::<F>,
        },
        Case {
            id: "TOPICS-8",
            says: "every refusal the store gives reaches its caller as it is, after one attempt, and a refused publish applies nothing: the seam never asks again",
            check: t8::<F>,
        },
    ]
}
