//! Histories and the plans that make them (docs/design/model.md, H11). A history is what clients
//! asked, when, and what they were told: each operation's invocation and, unless its answer was
//! lost, its completion, on one logical clock. A plan is a history to be: each client's requests
//! and the order in which every request is invoked, reaches the registry, and is answered or lost.
//! Running a plan against a registry gives a history; a plan is data, so a failing one can be
//! shrunk and run again unchanged.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use hyper_sim::rng::Seeded;

use crate::answer::Answer;
use crate::harness::Registry;
use crate::request::{Op, Request, SessionTag};

/// A client: one sequence of requests, each sent when the last was answered or given up.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Client(pub u16);

/// How an operation ended.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Answered at `at`.
    Answered { at: u64, answer: Answer },
    /// The answer never came: the request may or may not have taken effect.
    Lost,
}

/// One operation of a history.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Operation {
    pub client: Client,
    pub request: Request,
    pub invoked: u64,
    pub outcome: Outcome,
}

impl Operation {
    /// When it completed: never, for a lost answer.
    pub fn completed(&self) -> Option<u64> {
        match self.outcome {
            Outcome::Answered { at, .. } => Some(at),
            Outcome::Lost => None,
        }
    }

    /// Whether its interval and `other`'s share an instant.
    fn overlaps(&self, other: &Self) -> bool {
        let before = |a: &Self, b: &Self| a.completed().is_some_and(|done| done < b.invoked);
        !before(self, other) && !before(other, self)
    }
}

/// What clients asked and were told, in the order the operations were invoked.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct History {
    pub operations: Vec<Operation>,
}

impl History {
    /// How many clients had an operation overlap another client's: the clients whose requests
    /// truly interleaved (plan AC-RQ0.5.2).
    pub fn interleaved_clients(&self) -> usize {
        let mut clients = BTreeSet::new();
        for (i, a) in self.operations.iter().enumerate() {
            let crossed = self
                .operations
                .iter()
                .enumerate()
                .any(|(j, b)| i != j && a.client != b.client && a.overlaps(b));
            if crossed {
                clients.insert(a.client);
            }
        }
        clients.len()
    }

    /// How many operations lost their answers.
    pub fn lost(&self) -> usize {
        self.operations
            .iter()
            .filter(|op| op.outcome == Outcome::Lost)
            .count()
    }
}

/// A request a plan sends, and which client sends it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Planned {
    pub client: Client,
    pub request: Request,
}

/// One step of a plan, naming an operation by its index in the plan.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    /// The client sends the request: its interval starts.
    Invoke(usize),
    /// The request reaches the registry, which answers it now.
    Call(usize),
    /// The answer reaches the client: its interval ends.
    Return(usize),
    /// The client gives up: the answer, or the request itself if it was not called, is lost.
    Lose(usize),
}

/// Why a plan cannot run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlanError {
    /// An event names an operation the plan does not have.
    NoSuchOperation { event: usize },
    /// An operation's events are not invoke, an optional call, then a return (after a call) or a
    /// loss.
    OutOfOrder { event: usize },
    /// A client invokes a request while its last one is still waiting.
    Overlap { event: usize },
    /// An operation never ends.
    Unfinished { operation: usize },
    /// Two requests open one session.
    SessionReused { session: SessionTag },
    /// More clients than a client number counts.
    TooManyClients,
    /// More events than the clock counts.
    Clock,
}

/// Where an operation stands as a plan's events are checked or run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stage {
    Planned,
    Invoked,
    Called,
    Ended,
}

/// One client's place in an interleaving: the operations it has yet to invoke, and the one it is
/// waiting on with how far it has got.
type Lane = (VecDeque<usize>, Option<(usize, Stage)>);

/// A history to be: operations and the order of their events.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Plan {
    operations: Vec<Planned>,
    events: Vec<Event>,
}

/// The session a request opens, if it opens one.
fn opens(request: &Request) -> Option<SessionTag> {
    match request.op {
        Op::StartUpload { session, .. } | Op::MountBlob { session, .. } => Some(session),
        _ => None,
    }
}

impl Plan {
    /// One client's requests, each invoked, called and answered before the next.
    pub fn sequential(client: Client, requests: Vec<Request>) -> Self {
        let mut plan = Self {
            operations: Vec::new(),
            events: Vec::new(),
        };
        for request in requests {
            let i = plan.operations.len();
            plan.operations.push(Planned { client, request });
            plan.events
                .extend([Event::Invoke(i), Event::Call(i), Event::Return(i)]);
        }
        plan
    }

    /// `setup` sent sequentially by client 0, then each script by a client of its own (1, 2, …),
    /// their events interleaved by `rng`: at each step one client with something to do, chosen
    /// uniformly, invokes its next request, has it called, or receives its answer. With
    /// `lose_one_in` above zero, a request, or its answer, is lost one time in that many.
    pub fn interleave(
        setup: Vec<Request>,
        scripts: Vec<Vec<Request>>,
        rng: &mut Seeded,
        lose_one_in: u32,
    ) -> Result<Self, PlanError> {
        let mut plan = Self::sequential(Client(0), setup);
        let mut lanes: Vec<Lane> = Vec::new();
        for (k, script) in scripts.into_iter().enumerate() {
            let client = u16::try_from(k)
                .ok()
                .and_then(|k| k.checked_add(1))
                .ok_or(PlanError::TooManyClients)?;
            let mut queue = VecDeque::new();
            for request in script {
                queue.push_back(plan.operations.len());
                plan.operations.push(Planned {
                    client: Client(client),
                    request,
                });
            }
            lanes.push((queue, None));
        }
        let lose = |rng: &mut Seeded| lose_one_in > 0 && rng.below(u64::from(lose_one_in)) == 0;
        loop {
            let ready: Vec<usize> = lanes
                .iter()
                .enumerate()
                .filter(|(_, (queue, at))| at.is_some() || !queue.is_empty())
                .map(|(i, _)| i)
                .collect();
            let Some(&lane) = usize::try_from(rng.below(u64::try_from(ready.len()).unwrap_or(0)))
                .ok()
                .and_then(|pick| ready.get(pick))
            else {
                break;
            };
            let Some((queue, at)) = lanes.get_mut(lane) else {
                break;
            };
            match *at {
                None => {
                    if let Some(op) = queue.pop_front() {
                        plan.events.push(Event::Invoke(op));
                        *at = Some((op, Stage::Invoked));
                    }
                }
                Some((op, Stage::Invoked)) => {
                    if lose(rng) {
                        plan.events.push(Event::Lose(op));
                        *at = None;
                    } else {
                        plan.events.push(Event::Call(op));
                        *at = Some((op, Stage::Called));
                    }
                }
                Some((op, _)) => {
                    plan.events.push(if lose(rng) {
                        Event::Lose(op)
                    } else {
                        Event::Return(op)
                    });
                    *at = None;
                }
            }
        }
        Ok(plan)
    }

    /// Its operations, by index.
    pub fn operations(&self) -> &[Planned] {
        &self.operations
    }

    /// The plan with only the operations `keep` keeps, renumbered, their events in the same order.
    pub fn keeping(&self, keep: &BTreeSet<usize>) -> Self {
        let renumbered: BTreeMap<usize, usize> = keep
            .iter()
            .filter(|i| **i < self.operations.len())
            .enumerate()
            .map(|(new, old)| (*old, new))
            .collect();
        let operations = renumbered
            .keys()
            .filter_map(|old| self.operations.get(*old).cloned())
            .collect();
        let events =
            self.events
                .iter()
                .filter_map(|event| {
                    let (Event::Invoke(old)
                    | Event::Call(old)
                    | Event::Return(old)
                    | Event::Lose(old)) = *event;
                    renumbered.get(&old).map(|new| match event {
                        Event::Invoke(_) => Event::Invoke(*new),
                        Event::Call(_) => Event::Call(*new),
                        Event::Return(_) => Event::Return(*new),
                        Event::Lose(_) => Event::Lose(*new),
                    })
                })
                .collect();
        Self { operations, events }
    }

    /// Whether the plan can run: each operation invoked once, called at most once after that, and
    /// ended once by a return after its call or by a loss; each client's operations one at a time;
    /// each session opened by one request.
    pub fn check(&self) -> Result<(), PlanError> {
        let mut stage = vec![Stage::Planned; self.operations.len()];
        let mut waiting: BTreeSet<Client> = BTreeSet::new();
        for (event_index, event) in self.events.iter().enumerate() {
            let (Event::Invoke(op) | Event::Call(op) | Event::Return(op) | Event::Lose(op)) =
                *event;
            let client = self
                .operations
                .get(op)
                .ok_or(PlanError::NoSuchOperation { event: event_index })?
                .client;
            let at = stage
                .get_mut(op)
                .ok_or(PlanError::NoSuchOperation { event: event_index })?;
            let out_of_order = PlanError::OutOfOrder { event: event_index };
            *at = match (*event, *at) {
                (Event::Invoke(_), Stage::Planned) => {
                    if !waiting.insert(client) {
                        return Err(PlanError::Overlap { event: event_index });
                    }
                    Stage::Invoked
                }
                (Event::Call(_), Stage::Invoked) => Stage::Called,
                (Event::Return(_), Stage::Called)
                | (Event::Lose(_), Stage::Invoked | Stage::Called) => {
                    waiting.remove(&client);
                    Stage::Ended
                }
                _ => return Err(out_of_order),
            };
        }
        if let Some(operation) = stage.iter().position(|s| *s != Stage::Ended) {
            return Err(PlanError::Unfinished { operation });
        }
        let mut opened = BTreeSet::new();
        for planned in &self.operations {
            if let Some(session) = opens(&planned.request)
                && !opened.insert(session)
            {
                return Err(PlanError::SessionReused { session });
            }
        }
        Ok(())
    }

    /// Runs the plan against `registry`: each `Call` is the registry answering then. The history
    /// lists the operations in the order they were invoked.
    pub fn run<R: Registry + ?Sized>(&self, registry: &mut R) -> Result<History, PlanError> {
        self.check()?;
        let mut clock: u64 = 0;
        let mut invoked: Vec<Option<u64>> = vec![None; self.operations.len()];
        let mut answers: Vec<Option<Answer>> = vec![None; self.operations.len()];
        let mut outcomes: Vec<Option<Outcome>> = vec![None; self.operations.len()];
        for (event_index, event) in self.events.iter().enumerate() {
            clock = clock.checked_add(1).ok_or(PlanError::Clock)?;
            let missing = PlanError::NoSuchOperation { event: event_index };
            match *event {
                Event::Invoke(op) => {
                    *invoked.get_mut(op).ok_or(missing)? = Some(clock);
                }
                Event::Call(op) => {
                    let planned = self.operations.get(op).ok_or(missing.clone())?;
                    *answers.get_mut(op).ok_or(missing)? =
                        Some(registry.call(planned.client, &planned.request));
                }
                Event::Return(op) => {
                    let answer = answers
                        .get_mut(op)
                        .and_then(Option::take)
                        .ok_or(PlanError::OutOfOrder { event: event_index })?;
                    *outcomes.get_mut(op).ok_or(missing)? =
                        Some(Outcome::Answered { at: clock, answer });
                }
                Event::Lose(op) => {
                    *outcomes.get_mut(op).ok_or(missing)? = Some(Outcome::Lost);
                }
            }
        }
        let mut operations = Vec::with_capacity(self.operations.len());
        for (operation, ((planned, invoked), outcome)) in self
            .operations
            .iter()
            .zip(invoked)
            .zip(outcomes)
            .enumerate()
        {
            let unfinished = PlanError::Unfinished { operation };
            operations.push(Operation {
                client: planned.client,
                request: planned.request.clone(),
                invoked: invoked.ok_or(unfinished.clone())?,
                outcome: outcome.ok_or(unfinished)?,
            });
        }
        operations.sort_by_key(|op| op.invoked);
        Ok(History { operations })
    }
}
