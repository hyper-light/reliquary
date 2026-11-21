//! A client (docs/design/sim.md, W5): it sends its script's requests one at a time, each to a node,
//! waits for each answer as long as its patience, and records every attempt with when it was
//! invoked and how it ended, on the world's own time line, which is what a history is judged on.
//! A refusal, or an answer that never came, is tried again on the next node after a pause, up to a
//! counted number of attempts (plan T-RQ0.6.4). A grant for a blob is redeemed at the bytes store
//! it names, as a client follows a redirect (architecture D4).

use std::num::NonZeroU32;
use std::time::Duration;

use hyper_sim::{NodeId, SimError};
use reliquary_api::answer::{Answer, Body, Status};
use reliquary_api::history::Client;
use reliquary_api::request::{Principal, Request};

use crate::clock::ProcessClock;
use crate::ctx::Ctx;
use crate::wire::{Msg, Offer, Refused, Reply};

/// How a client waits and tries again. Every figure is its maker's.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Policy {
    /// How long an attempt waits for its answer before it is given up as lost.
    pub patience: Duration,
    /// How many attempts an operation has, the first included.
    pub attempts: u32,
    /// How long it waits after a refusal or a lost answer before trying again.
    pub pause: Duration,
}

/// How an attempt ended. Attempts count from one: an operation's first is the least nonzero number.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ended {
    /// The registry's answer, at `at`.
    Answered { at: u64, answer: Answer },
    /// A node's typed refusal, at `at`.
    Refused { at: u64, refused: Refused },
    /// No answer by the client's patience, given up at `at`: it may or may not have applied.
    Lost { at: u64 },
}

/// One attempt of one operation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Attempt {
    pub client: Client,
    /// The operation's place in its client's script.
    pub op: u64,
    /// Its attempt, from one.
    pub attempt: u32,
    pub request: Request,
    /// When it was sent, on the world's time line.
    pub invoked: u64,
    pub ended: Ended,
}

#[derive(Clone, Debug)]
enum State {
    /// Waiting for another client to finish before it starts.
    Waiting,
    /// The next attempt goes at this monotonic reading.
    Ready {
        at_ns: u64,
        attempt: u32,
    },
    /// Waiting for a node's reply.
    Asking {
        attempt: u32,
        invoked: u64,
        deadline_ns: u64,
    },
    /// Waiting for a bytes store to honor a grant.
    Redeeming {
        attempt: u32,
        invoked: u64,
        deadline_ns: u64,
        offer: Offer,
    },
    Done,
}

/// A client process.
pub(crate) struct ClientProc {
    id: Client,
    script: Vec<Request>,
    op: usize,
    nodes: Vec<NodeId>,
    node_at: usize,
    policy: Policy,
    pub(crate) clock: ProcessClock,
    state: State,
    pub(crate) attempts: Vec<Attempt>,
}

/// The holder a principal's grants name: who presents them at Mantle's side.
#[must_use]
pub fn holder(principal: Principal) -> String {
    match principal {
        Principal::Anonymous => "anonymous".to_owned(),
        Principal::Named(n) => format!("principal-{n}"),
    }
}

fn nanos(d: Duration) -> u64 {
    u64::try_from(d.as_nanos()).unwrap_or(u64::MAX)
}

impl ClientProc {
    pub(crate) fn new(
        id: Client,
        script: Vec<Request>,
        nodes: Vec<NodeId>,
        policy: Policy,
        clock: ProcessClock,
        start_ns: Option<u64>,
    ) -> Self {
        let state = match start_ns {
            _ if script.is_empty() => State::Done,
            Some(at_ns) => State::Ready {
                at_ns,
                attempt: NonZeroU32::MIN.get(),
            },
            None => State::Waiting,
        };
        Self {
            id,
            script,
            op: 0,
            nodes,
            node_at: 0,
            policy,
            clock,
            state,
            attempts: Vec::new(),
        }
    }

    /// Whether its script is done.
    pub(crate) fn done(&self) -> bool {
        matches!(self.state, State::Done)
    }

    /// The client it waited for has finished: it starts now.
    pub(crate) fn start(&mut self) {
        if matches!(self.state, State::Waiting) {
            self.state = State::Ready {
                at_ns: self.clock.monotonic_ns(),
                attempt: NonZeroU32::MIN.get(),
            };
        }
    }

    /// The operation it is on, and its attempt, when it is not done.
    pub(crate) fn on(&self) -> Option<(usize, u32)> {
        match &self.state {
            State::Ready { attempt, .. }
            | State::Asking { attempt, .. }
            | State::Redeeming { attempt, .. } => Some((self.op, *attempt)),
            State::Waiting => Some((self.op, 0)),
            State::Done => None,
        }
    }

    pub(crate) fn next_wake(&self) -> Option<u64> {
        match &self.state {
            State::Ready { at_ns, .. } => Some(*at_ns),
            State::Asking { deadline_ns, .. } | State::Redeeming { deadline_ns, .. } => {
                Some(*deadline_ns)
            }
            State::Waiting | State::Done => None,
        }
    }

    fn op_id(&self) -> u64 {
        u64::try_from(self.op).unwrap_or(u64::MAX)
    }

    fn request(&self) -> Option<&Request> {
        self.script.get(self.op)
    }

    /// Records how the current attempt ended.
    fn record(&mut self, attempt: u32, invoked: u64, ended: Ended) {
        if let Some(request) = self.request().cloned() {
            self.attempts.push(Attempt {
                client: self.id,
                op: self.op_id(),
                attempt,
                request,
                invoked,
                ended,
            });
        }
    }

    /// The operation is over: on to the next, at once.
    fn next_op(&mut self) {
        self.op = self.op.saturating_add(1);
        self.state = if self.op < self.script.len() {
            State::Ready {
                at_ns: self.clock.monotonic_ns(),
                attempt: NonZeroU32::MIN.get(),
            }
        } else {
            State::Done
        };
    }

    /// The attempt failed: another after the pause, on the next node, while attempts are left.
    fn again(&mut self, attempt: u32) {
        if attempt >= self.policy.attempts {
            self.next_op();
            return;
        }
        if !self.nodes.is_empty() {
            self.node_at = self
                .node_at
                .saturating_add(1)
                .checked_rem(self.nodes.len())
                .unwrap_or(0);
        }
        self.state = State::Ready {
            at_ns: self
                .clock
                .monotonic_ns()
                .saturating_add(nanos(self.policy.pause)),
            attempt: attempt.saturating_add(1),
        };
    }

    pub(crate) fn wake(&mut self, ctx: &mut Ctx<'_>) -> Result<(), SimError> {
        let now = self.clock.monotonic_ns();
        match self.state.clone() {
            State::Ready { at_ns, attempt } if at_ns <= now => self.send(ctx, attempt),
            State::Asking {
                attempt,
                invoked,
                deadline_ns,
            }
            | State::Redeeming {
                attempt,
                invoked,
                deadline_ns,
                ..
            } if deadline_ns <= now => {
                let at = ctx.now();
                ctx.log(format_args!("op {}.{attempt} lost: no answer", self.op));
                self.record(attempt, invoked, Ended::Lost { at });
                self.again(attempt);
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn send(&mut self, ctx: &mut Ctx<'_>, attempt: u32) -> Result<(), SimError> {
        let (Some(request), Some(node)) = (
            self.request().cloned(),
            self.nodes.get(self.node_at).copied(),
        ) else {
            self.state = State::Done;
            return Ok(());
        };
        let invoked = ctx.now();
        let deadline_ns = self
            .clock
            .monotonic_ns()
            .saturating_add(nanos(self.policy.patience));
        self.state = State::Asking {
            attempt,
            invoked,
            deadline_ns,
        };
        let op = self.op_id();
        ctx.send(
            node,
            Msg::Request {
                op,
                attempt,
                request,
            },
        )
    }

    pub(crate) fn message(&mut self, ctx: &mut Ctx<'_>, msg: Msg) -> Result<(), SimError> {
        let current = self.op_id();
        match (msg, self.state.clone()) {
            (
                Msg::Reply {
                    op,
                    attempt,
                    reply,
                    store,
                },
                State::Asking {
                    attempt: asking,
                    invoked,
                    ..
                },
            ) if op == current && attempt == asking => {
                let at = ctx.now();
                match reply {
                    Reply::Answer(answer) => {
                        self.record(attempt, invoked, Ended::Answered { at, answer });
                        self.next_op();
                        Ok(())
                    }
                    Reply::Refused(refused) => {
                        self.record(attempt, invoked, Ended::Refused { at, refused });
                        self.again(attempt);
                        Ok(())
                    }
                    Reply::Grant(offer) => {
                        let principal =
                            self.request().map_or(Principal::Anonymous, |r| r.principal);
                        let deadline_ns = self
                            .clock
                            .monotonic_ns()
                            .saturating_add(nanos(self.policy.patience));
                        let grant = offer.grant.clone();
                        self.state = State::Redeeming {
                            attempt,
                            invoked,
                            deadline_ns,
                            offer,
                        };
                        ctx.send(
                            store,
                            Msg::Redeem {
                                op,
                                attempt,
                                grant,
                                holder: holder(principal),
                                remaining: self.policy.patience,
                            },
                        )
                    }
                }
            }
            (
                Msg::Redeemed {
                    op,
                    attempt,
                    content,
                },
                State::Redeeming {
                    attempt: asking,
                    invoked,
                    offer,
                    ..
                },
            ) if op == current && attempt == asking => {
                let at = ctx.now();
                match content {
                    Ok(bytes) => {
                        let answer = Answer::success(
                            Status::Ok,
                            Body::Blob {
                                digest: offer.digest,
                                size: offer.size,
                                content: Some(bytes),
                            },
                        );
                        self.record(attempt, invoked, Ended::Answered { at, answer });
                        self.next_op();
                    }
                    Err(refusal) => {
                        // A read changes nothing, so a grant refused is a refusal of nothing applied.
                        let refused = Refused {
                            kind: crate::wire::RefusedKind::Unavailable,
                            why: refusal.to_string(),
                        };
                        self.record(attempt, invoked, Ended::Refused { at, refused });
                        self.again(attempt);
                    }
                }
                Ok(())
            }
            _ => {
                ctx.ledger.let_go("an answer to an attempt given up");
                Ok(())
            }
        }
    }
}
