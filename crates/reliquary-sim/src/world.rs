//! A run of the world (docs/design/sim.md, W1): machines, each with its clocks and the processes it
//! hosts (a records store, a bytes store, nodes, clients), on hyper-sim's world and its network;
//! the nemesis's schedule; and the loop that takes the world's next event, hands it to the process
//! it is for, and arms that process's timer again. Every choice is the world's, drawn from the
//! seed's named streams or read from a trace, so a run is a function of its seed (plan
//! AC-RQ0.6.1); [`run`] gives everything a run did, and [`crate::verdict`] judges it.

use std::collections::VecDeque;
use std::rc::Rc;
use std::time::Duration;

use hyper_sim::net::{Net, NetLimits, NetStats, Path};
use hyper_sim::{
    Clock as WorldClock, Discipline, Limits, NodeId, Record, SimError, Source, Step, Strategy,
    World,
};
use reliquary_api::history::Client;
use reliquary_api::request::Request;
use reliquary_core::seams::bytes::sim::{SimBytes, SimLimits as BytesLimits};
use reliquary_core::seams::clock::SimClock;
use reliquary_core::seams::records::{SimLimits as RecordsLimits, SimRecords};

use crate::client::{Attempt, ClientProc, Policy};
use crate::clock::ProcessClock;
use crate::ctx::{Action, Ctx, name_of, summary};
use crate::ledger::{Breach, Grants, Ledger, NegativeWait, Write, WriteCut};
use crate::log::{Digest, Log};
use crate::nemesis::{self, Envelope, Fault, Schedule};
use crate::node::{Core, NodeLimits, NodeShell};
use crate::store::{Store, StoreLimits};
use crate::wire::{Ev, Msg, Refused, RefusedKind, Reply};

/// What a machine hosts.
#[derive(Clone, Debug)]
pub enum Role {
    /// The records store (Amphora's side).
    Records,
    /// The bytes store (Mantle's side).
    Bytes,
    /// A node, reaching its records and its bytes on the machines named.
    Node { records: usize, bytes: usize },
    /// A client.
    Client(ClientSpec),
}

/// A client's script, the machines whose nodes it sends to (in turn, the first first), how it
/// waits, and when it starts: at `start`, or, with `after`, once that client (by its place among
/// the spec's clients) has finished its script, as a plan's clients follow its setup.
#[derive(Clone, Debug)]
pub struct ClientSpec {
    pub script: Vec<Request>,
    pub nodes: Vec<usize>,
    pub policy: Policy,
    pub start: Duration,
    pub after: Option<usize>,
}

/// A machine: its clocks (offset, drift within the deployment's bound, wall time, timer lateness)
/// and what runs on it.
#[derive(Clone, Debug)]
pub struct Machine {
    pub clock: WorldClock,
    pub roles: Vec<Role>,
}

/// Everything a run is: the machines, the network between them, every bound, the core nodes run,
/// the nemesis's schedule, and the power cut a sweep makes. Every figure is its maker's.
pub struct Spec<C> {
    pub machines: Vec<Machine>,
    /// The path between any two machines.
    pub path: Path,
    /// Each delivery keeps its message for another with this chance, in parts per million.
    pub duplicate_ppm: u32,
    pub discipline: Discipline,
    pub limits: Limits,
    pub net: NetLimits,
    pub records: RecordsLimits,
    pub bytes: BytesLimits,
    /// The key the bytes store signs its grants with.
    pub grant_key: Vec<u8>,
    pub stores: StoreLimits,
    pub nodes: NodeLimits,
    /// The most sleeps each process's clock holds.
    pub sleeps: usize,
    /// Makes a node's core, at each start.
    pub core: Rc<dyn Fn() -> C>,
    pub nemesis: Schedule,
    /// Faults drawn from the run's own stream within this envelope, beside the schedule's: they
    /// replay from the seed or the trace like every other choice.
    pub drawn: Option<Envelope>,
    /// How long a machine stays down after a crash or a power cut.
    pub downtime: Duration,
    /// The log's last lines kept for a failure's report.
    pub keep: usize,
}

/// Why a run could not be made or run: the world refused (a stated bound reached, time past what
/// it can say), or the spec names what it does not hold.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("the world: {0}")]
    World(SimError),
    #[error("the spec: {0}")]
    Spec(String),
}

impl From<SimError> for Error {
    fn from(error: SimError) -> Self {
        Self::World(error)
    }
}

/// How a run ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum End {
    /// Every client finished its script, and the faults had stopped.
    Finished,
    /// Nothing was pending, with a client unfinished.
    Idle,
    /// The run's step budget was spent first.
    Spent,
}

/// Something still waiting when the run ended.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Waiter {
    /// A client on operation `op`, attempt `attempt`.
    Client {
        name: String,
        op: usize,
        attempt: u32,
    },
    /// A node holding `jobs` requests it never answered, and waiting on `calls` calls.
    Node {
        name: String,
        jobs: usize,
        calls: usize,
    },
}

/// Everything a run did.
#[derive(Debug)]
pub struct Outcome {
    /// The world's record: the digest of every decision, event and observation, and the trace.
    pub record: Record,
    pub end: End,
    pub log: Digest,
    /// The log's last lines.
    pub tail: Vec<String>,
    /// Every attempt of every client, client by client, in order.
    pub attempts: Vec<Attempt>,
    /// When the faults stopped, on the world's time line: the calm, the last fault's taking
    /// effect, or the last time a machine came back up, whichever is latest. A run that never
    /// reached its calm has none.
    pub quiet_from: Option<u64>,
    /// Faults armed that never took effect: a power cut or a lost answer whose machine wrote or
    /// answered nothing more. Each by its kind and machine.
    pub unlanded: Vec<(&'static str, usize)>,
    pub waiters: Vec<Waiter>,
    pub writes: Vec<Write>,
    pub faults: std::collections::BTreeMap<&'static str, u64>,
    pub breaches: Vec<Breach>,
    pub waits: Vec<NegativeWait>,
    pub let_go: std::collections::BTreeMap<&'static str, u64>,
    /// Every answer a store gave: a refusal by its kind, else its seam.
    pub answers: std::collections::BTreeMap<String, u64>,
    pub net: NetStats,
    /// What the bytes stores did with the grants presented to them.
    pub grants: Grants,
    /// Each process's clocks when the run ended: its name, monotonic and wall readings.
    pub clocks: Vec<(String, u64, u64)>,
}

/// Whether a machine is up; its processes crashed, its network still there to refuse a connection
/// and reset those that were open; or its power off, nothing answering.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Power {
    Up,
    Crashed,
    Off,
}

enum Proc<C> {
    Store(Store),
    Node(NodeShell<C>),
    Client(ClientProc),
}

impl<C: Core> Proc<C> {
    fn clock(&self) -> &ProcessClock {
        match self {
            Self::Store(s) => &s.clock,
            Self::Node(n) => &n.clock,
            Self::Client(c) => &c.clock,
        }
    }

    fn next_wake(&self) -> Option<u64> {
        match self {
            Self::Store(s) => s.next_wake(),
            Self::Node(n) => n.next_wake(),
            Self::Client(c) => c.next_wake(),
        }
    }

    fn down(&mut self) {
        match self {
            Self::Store(s) => s.down(),
            Self::Node(n) => n.down(),
            // A client's machine is never cut: the nemesis touches the registry's machines.
            Self::Client(_) => {}
        }
    }

    fn restart(&mut self) {
        match self {
            Self::Store(s) => s.restart(),
            Self::Node(n) => n.restart(),
            Self::Client(_) => {}
        }
    }

    fn is_up(&self) -> bool {
        match self {
            Self::Store(s) => s.is_up(),
            Self::Node(n) => n.is_up(),
            Self::Client(_) => true,
        }
    }
}

/// The run's state between events.
struct Run<C> {
    world: World<Ev>,
    net: Net<Msg>,
    procs: Vec<Proc<C>>,
    names: Vec<String>,
    /// Each machine's processes, and its power.
    machines: Vec<(Vec<NodeId>, Power)>,
    /// What processes asked of the run, carried out in order between events.
    actions: VecDeque<Action>,
    log: Log,
    ledger: Ledger,
    nemesis: NodeId,
    schedule: Schedule,
    downtime_ns: u64,
    path: Path,
    /// Links the nemesis slowed, to restore at the calm.
    slowed: Vec<(NodeId, NodeId)>,
    /// How many partitions cut each pair now.
    cuts: std::collections::BTreeMap<(NodeId, NodeId), u32>,
    calm_at: Option<u64>,
    /// When a machine last came back up.
    last_up: u64,
    /// Clients that start when another finishes: (follower, leader), as processes.
    after: Vec<(usize, usize)>,
}

fn index(node: NodeId) -> usize {
    usize::try_from(node.0).unwrap_or(usize::MAX)
}

fn nanos(d: Duration) -> u64 {
    u64::try_from(d.as_nanos()).unwrap_or(u64::MAX)
}

/// Builds the world `spec` states from `source` and runs it to its end, `strategy` choosing among
/// what is enabled at each point.
pub fn run<C: Core + 'static, S: Strategy>(
    spec: &Spec<C>,
    source: Source,
    strategy: &mut S,
) -> Result<Outcome, Error> {
    run_cut(spec, None, source, strategy)
}

/// [`run`], with power cut at `cut`, if any: the store's machine loses power at that write
/// (plan AC-RQ0.6.3).
pub fn run_cut<C: Core + 'static, S: Strategy>(
    spec: &Spec<C>,
    cut: Option<WriteCut>,
    source: Source,
    strategy: &mut S,
) -> Result<Outcome, Error> {
    let mut run = build(spec, cut, source)?;
    let end = run.go(strategy)?;
    Ok(run.finish(end))
}

/// The world `spec` states, every process in place, the nemesis's faults scheduled.
fn build<C: Core + 'static>(
    spec: &Spec<C>,
    cut: Option<WriteCut>,
    source: Source,
) -> Result<Run<C>, Error> {
    let mut world: World<Ev> = World::new(source, spec.discipline, spec.limits)?;
    let mut net = Net::new(spec.net);
    net.set_path(spec.path);
    net.set_duplicate_ppm(spec.duplicate_ppm);
    let nemesis = world.node(WorldClock::default())?;
    let mut procs: Vec<Proc<C>> = Vec::new();
    let mut names = vec!["nemesis".to_owned()];
    let mut machines: Vec<(Vec<NodeId>, Power)> = Vec::new();
    // Where each machine's records and bytes stores are, once made, and where its node is.
    let mut records_at = vec![None; spec.machines.len()];
    let mut bytes_at = vec![None; spec.machines.len()];
    let mut node_at = vec![None; spec.machines.len()];
    // Stores first, so a node anywhere can name its stores.
    for (m, machine) in spec.machines.iter().enumerate() {
        machines.push((Vec::new(), Power::Up));
        for role in &machine.roles {
            if !matches!(role, Role::Records | Role::Bytes) {
                continue;
            }
            let node = world.node(machine.clock)?;
            let clock = process_clock(&world, node, spec.sleeps)?;
            let cuts = world.stream("store.cuts", &[u64::from(node.0)])?;
            let store = match role {
                Role::Records => {
                    let sim_clock = SimClock::new(clock_wall(&clock), spec.sleeps);
                    let sim = SimRecords::new(sim_clock.clone(), spec.records);
                    if let Some(slot) = records_at.get_mut(m) {
                        *slot = Some(node);
                    }
                    names.push(format!("records{m}"));
                    Store::records(sim, sim_clock, clock, spec.stores, cuts)
                }
                _ => {
                    let sim = SimBytes::in_memory(
                        std::sync::Arc::new(clock.clone()),
                        &spec.grant_key,
                        spec.bytes,
                    )
                    .map_err(|e| Error::Spec(format!("the bytes store's limits: {e}")))?;
                    if let Some(slot) = bytes_at.get_mut(m) {
                        *slot = Some(node);
                    }
                    names.push(format!("bytes{m}"));
                    Store::bytes(sim, clock, spec.stores, cuts)
                }
            };
            procs.push(Proc::Store(store));
            if let Some((list, _)) = machines.get_mut(m) {
                list.push(node);
            }
        }
    }
    for (m, machine) in spec.machines.iter().enumerate() {
        for role in &machine.roles {
            let Role::Node { records, bytes } = role else {
                continue;
            };
            let (Some(Some(r)), Some(Some(b))) = (records_at.get(*records), bytes_at.get(*bytes))
            else {
                return Err(Error::Spec(format!(
                    "node on machine {m} names records on {records} and bytes on {bytes}, which hold none"
                )));
            };
            let node = world.node(machine.clock)?;
            let clock = process_clock(&world, node, spec.sleeps)?;
            let entropy = world.stream("node.entropy", &[u64::from(node.0)])?;
            let make = Rc::clone(&spec.core);
            let local = (*records == m, *bytes == m);
            let shell = NodeShell::new(
                Box::new(move || make()),
                (*r, *b),
                local,
                spec.nodes,
                clock,
                entropy,
            );
            if let Some(slot) = node_at.get_mut(m) {
                *slot = Some(node);
            }
            names.push(format!("node{m}"));
            procs.push(Proc::Node(shell));
            if let Some((list, _)) = machines.get_mut(m) {
                list.push(node);
            }
        }
    }
    let mut client_count: u16 = 0;
    // Each client's process, and the client it waits for, by its place among the clients.
    let mut followers: Vec<(usize, Option<usize>)> = Vec::new();
    for (m, machine) in spec.machines.iter().enumerate() {
        for role in &machine.roles {
            let Role::Client(client) = role else {
                continue;
            };
            let node = world.node(machine.clock)?;
            let clock = process_clock(&world, node, spec.sleeps)?;
            let nodes: Vec<NodeId> = client
                .nodes
                .iter()
                .filter_map(|n| node_at.get(*n).copied().flatten())
                .collect();
            let start_ns = match client.after {
                Some(_) => None,
                None => Some(clock.monotonic_ns().saturating_add(nanos(client.start))),
            };
            let id = Client(client_count);
            client_count = client_count.saturating_add(1);
            names.push(format!("client{}", id.0));
            followers.push((procs.len(), client.after));
            procs.push(Proc::Client(ClientProc::new(
                id,
                client.script.clone(),
                nodes,
                client.policy,
                clock,
                start_ns,
            )));
            if let Some((list, _)) = machines.get_mut(m) {
                list.push(node);
            }
        }
    }
    let clients: Vec<usize> = followers.iter().map(|(p, _)| *p).collect();
    let after: Vec<(usize, usize)> = followers
        .iter()
        .filter_map(|(p, after)| {
            after
                .and_then(|a| clients.get(a))
                .map(|leader| (*p, *leader))
        })
        .collect();
    // Processes are hyper-sim's nodes 1 onward, in the order made (the nemesis's node is 0); the
    // vector holds them from 0, and `names` from the nemesis.
    // Processes on one machine reach one another at once, and no partition cuts them.
    for (list, _) in &machines {
        for a in list {
            for b in list {
                if a != b {
                    net.set_pair_path(*a, *b, Path::NONE)?;
                }
            }
        }
    }
    let mut run = Run {
        world,
        net,
        procs,
        names,
        machines,
        actions: VecDeque::new(),
        log: Log::new(spec.keep),
        ledger: Ledger::new(cut),
        nemesis,
        schedule: spec.nemesis.clone(),
        downtime_ns: nanos(spec.downtime),
        path: spec.path,
        slowed: Vec::new(),
        cuts: std::collections::BTreeMap::new(),
        calm_at: None,
        last_up: 0,
        after,
    };
    if let Some(envelope) = &spec.drawn {
        let drawn = Schedule::drawn(&mut run.world, envelope)?;
        run.schedule.faults.extend(drawn.faults);
        run.schedule.faults.sort_by_key(|(at, _)| *at);
        run.schedule.calm = run.schedule.calm.max(drawn.calm);
    }
    for (i, (at, _)) in run.schedule.faults.iter().enumerate() {
        run.world.schedule(nanos(*at), nemesis, Ev::Nemesis(i))?;
    }
    let calm = run.schedule.faults.len();
    run.world
        .schedule(nanos(run.schedule.calm), nemesis, Ev::Nemesis(calm))?;
    for node in 0..run.procs.len() {
        run.arm(proc_node(node))?;
    }
    Ok(run)
}

/// What a crashed machine's network answers `msg` with, from `node`: a request or a call refused,
/// nothing having applied; anything else is lost.
fn refusal(node: NodeId, msg: Msg) -> Option<Msg> {
    let why = "the connection was refused";
    match msg {
        Msg::Request { op, attempt, .. } => Some(Msg::Reply {
            op,
            attempt,
            reply: Reply::Refused(Refused {
                kind: RefusedKind::Unavailable,
                why: why.to_owned(),
            }),
            store: node,
        }),
        Msg::Call {
            id,
            incarnation,
            call,
            ..
        } => Some(Msg::Answer {
            id,
            incarnation,
            answer: call.refused(),
        }),
        Msg::Redeem { op, attempt, .. } => Some(Msg::Redeemed {
            op,
            attempt,
            content: Err(reliquary_core::seams::bytes::Refusal::Unavailable(
                why.to_owned(),
            )),
        }),
        Msg::Reply { .. } | Msg::Answer { .. } | Msg::Redeemed { .. } => None,
    }
}

/// hyper-sim's node for the process at `index`: the nemesis's node is 0, processes follow.
fn proc_node(index: usize) -> NodeId {
    NodeId(u32::try_from(index).unwrap_or(u32::MAX).saturating_add(1))
}

/// The process at hyper-sim's node `node`.
fn proc_index(node: NodeId) -> Option<usize> {
    index(node).checked_sub(1)
}

fn process_clock(world: &World<Ev>, node: NodeId, sleeps: usize) -> Result<ProcessClock, SimError> {
    let monotonic = world.monotonic(node)?;
    let instant = world.instant(node)?;
    let anchor = instant
        .checked_sub(Duration::from_nanos(monotonic))
        .ok_or(SimError::TimeOverflow)?;
    Ok(ProcessClock::new(
        anchor,
        monotonic,
        world.wall(node)?,
        sleeps,
    ))
}

fn clock_wall(clock: &ProcessClock) -> std::time::SystemTime {
    use reliquary_core::seams::clock::Clock;
    clock.wall()
}

impl<C: Core> Run<C> {
    /// Arms the timer of the process on `node` at its next wake.
    fn arm(&mut self, node: NodeId) -> Result<(), SimError> {
        let wake = proc_index(node)
            .and_then(|i| self.procs.get(i))
            .and_then(|p| if p.is_up() { p.next_wake() } else { None });
        self.world.wake(node, wake)
    }

    fn clients_done(&self) -> bool {
        self.procs.iter().all(|p| match p {
            Proc::Client(c) => c.done(),
            _ => true,
        })
    }

    fn go<S: Strategy>(&mut self, strategy: &mut S) -> Result<End, SimError> {
        loop {
            if self.calm_at.is_some() && self.clients_done() {
                return Ok(End::Finished);
            }
            match self.world.next(strategy)? {
                Step::Event { event, .. } => match event {
                    Ev::Arrive(ticket) => {
                        if let Some(delivery) =
                            self.net.deliver(&mut self.world, ticket, Ev::Arrive)?
                        {
                            self.dispatch(delivery.to, Some((delivery.from, delivery.payload)))?;
                        }
                    }
                    Ev::Nemesis(i) => self.nemesis(i)?,
                    Ev::Restart(machine) => self.restart(machine)?,
                    Ev::Heal(i) => self.heal(i),
                },
                Step::Wake { node } => {
                    if node != self.nemesis {
                        self.dispatch(node, None)?;
                    }
                }
                Step::Idle => return Ok(End::Idle),
                Step::Spent => return Ok(End::Spent),
            }
            while let Some(action) = self.actions.pop_front() {
                match action {
                    Action::PowerCut { machine } => self.down(machine, Power::Off)?,
                }
            }
            self.start_followers()?;
        }
    }

    /// The machine `node` is on, and its power.
    fn machine_of(&self, node: NodeId) -> (usize, Power) {
        self.machines
            .iter()
            .position(|(list, _)| list.contains(&node))
            .and_then(|m| self.machines.get(m).map(|(_, power)| (m, *power)))
            .unwrap_or((usize::MAX, Power::Off))
    }

    /// Lends the process on `node` its view of the world for `work`, its clock set to now; then
    /// arms its timer again, and queues what it asked of the run.
    fn with_ctx<R>(
        &mut self,
        node: NodeId,
        work: impl FnOnce(&mut Proc<C>, &mut Ctx<'_>) -> Result<R, SimError>,
    ) -> Result<Option<R>, SimError> {
        let Some(i) = proc_index(node) else {
            return Ok(None);
        };
        let monotonic = self.world.monotonic(node)?;
        let wall = self.world.wall(node)?;
        let (machine, _) = self.machine_of(node);
        let mut actions = Vec::new();
        let Some(proc) = self.procs.get_mut(i) else {
            return Ok(None);
        };
        proc.clock().set(monotonic, wall);
        let mut ctx = Ctx {
            world: &mut self.world,
            net: &mut self.net,
            log: &mut self.log,
            ledger: &mut self.ledger,
            actions: &mut actions,
            names: &self.names,
            me: node,
            machine,
        };
        let out = work(proc, &mut ctx)?;
        self.actions.extend(actions);
        self.arm(node)?;
        Ok(Some(out))
    }

    /// Hands the process on `node` a message, or its timer's firing. A message for a crashed
    /// machine is refused there, as its network refuses a connection; one for a machine with no
    /// power is lost.
    fn dispatch(&mut self, node: NodeId, message: Option<(NodeId, Msg)>) -> Result<(), SimError> {
        let (_, power) = self.machine_of(node);
        match (power, message) {
            (Power::Up, Some((from, msg))) => {
                self.with_ctx(node, |proc, ctx| {
                    let line = summary(&msg);
                    let from_name = name_of(ctx.names, from).to_owned();
                    ctx.log(format_args!("recv {from_name} {line}"));
                    match proc {
                        Proc::Store(store) => store.message(ctx, from, msg),
                        Proc::Node(shell) => shell.message(ctx, from, msg),
                        Proc::Client(client) => client.message(ctx, msg),
                    }
                })?;
            }
            (Power::Up, None) => {
                self.with_ctx(node, |proc, ctx| {
                    ctx.log(format_args!("wake"));
                    match proc {
                        Proc::Store(store) => store.wake(ctx),
                        Proc::Node(shell) => shell.wake(ctx),
                        Proc::Client(client) => client.wake(ctx),
                    }
                })?;
            }
            (Power::Crashed, Some((from, msg))) => {
                self.with_ctx(node, |_, ctx| {
                    let line = summary(&msg);
                    ctx.log(format_args!("refused {line}: crashed"));
                    match refusal(node, msg) {
                        Some(refusal) => ctx.send(from, refusal),
                        None => {
                            ctx.ledger.let_go("to a process that is down");
                            Ok(())
                        }
                    }
                })?;
            }
            (Power::Off, Some(_)) => self.ledger.let_go("to a machine with no power"),
            (Power::Crashed | Power::Off, None) => {}
        }
        Ok(())
    }

    /// Starts every client whose leader has finished.
    fn start_followers(&mut self) -> Result<(), SimError> {
        let ready: Vec<usize> = self
            .after
            .iter()
            .filter(
                |(_, leader)| matches!(self.procs.get(*leader), Some(Proc::Client(c)) if c.done()),
            )
            .map(|(follower, _)| *follower)
            .collect();
        if ready.is_empty() {
            return Ok(());
        }
        self.after.retain(|(follower, _)| !ready.contains(follower));
        for follower in ready {
            let node = proc_node(follower);
            let monotonic = self.world.monotonic(node)?;
            let wall = self.world.wall(node)?;
            if let Some(Proc::Client(client)) = self.procs.get_mut(follower) {
                client.clock.set(monotonic, wall);
                client.start();
            }
            self.arm(node)?;
        }
        Ok(())
    }

    /// Every process of `machine` stops, and the machine starts again after the downtime. A crash
    /// resets the connections its processes had open: each client a node there was serving is told
    /// its request's outcome is unknown, and each call a node elsewhere made to a store there is
    /// answered so. A machine whose power is cut tells no one.
    fn down(&mut self, machine: usize, power: Power) -> Result<(), SimError> {
        let Some((list, now)) = self.machines.get(machine) else {
            return Ok(());
        };
        if *now != Power::Up {
            return Ok(());
        }
        let list = list.clone();
        if power == Power::Crashed {
            for node in &list {
                self.with_ctx(*node, |proc, ctx| match proc {
                    Proc::Node(shell) => shell.reset_clients(ctx),
                    _ => Ok(()),
                })?;
            }
            let stores: Vec<NodeId> = list
                .iter()
                .copied()
                .filter(|n| {
                    matches!(
                        proc_index(*n).and_then(|i| self.procs.get(i)),
                        Some(Proc::Store(_))
                    )
                })
                .collect();
            let nodes: Vec<NodeId> = (0..self.procs.len())
                .map(proc_node)
                .filter(|n| !list.contains(n))
                .filter(|n| {
                    matches!(
                        proc_index(*n).and_then(|i| self.procs.get(i)),
                        Some(Proc::Node(_))
                    )
                })
                .collect();
            for node in nodes {
                if self.machine_of(node).1 != Power::Up {
                    continue;
                }
                for store in &stores {
                    self.with_ctx(node, |proc, ctx| match proc {
                        Proc::Node(shell) => shell.reset_calls(ctx, *store),
                        _ => Ok(()),
                    })?;
                }
            }
        }
        if let Some((_, now)) = self.machines.get_mut(machine) {
            *now = power;
        }
        for node in &list {
            if let Some(p) = proc_index(*node).and_then(|i| self.procs.get_mut(i)) {
                p.down();
            }
            self.world.wake(*node, None)?;
        }
        let at = self.world.now();
        let how = if power == Power::Crashed {
            "crashed"
        } else {
            "lost power"
        };
        self.log
            .line(at, "nemesis", format_args!("machine {machine} {how}"));
        self.world
            .after(self.downtime_ns, self.nemesis, Ev::Restart(machine))?;
        Ok(())
    }

    fn restart(&mut self, machine: usize) -> Result<(), SimError> {
        let Some((list, power)) = self.machines.get_mut(machine) else {
            return Ok(());
        };
        if *power == Power::Up {
            return Ok(());
        }
        *power = Power::Up;
        let list = list.clone();
        let at = self.world.now();
        self.last_up = self.last_up.max(at);
        self.log
            .line(at, "nemesis", format_args!("machine {machine} up"));
        for node in list {
            let monotonic = self.world.monotonic(node)?;
            let wall = self.world.wall(node)?;
            if let Some(p) = proc_index(node).and_then(|i| self.procs.get_mut(i)) {
                p.clock().set(monotonic, wall);
                p.restart();
            }
            self.arm(node)?;
        }
        Ok(())
    }

    /// Every directed pair of processes across machines `a` and machines `b`, both ways.
    fn across(&self, a: &[usize], b: &[usize]) -> Vec<(NodeId, NodeId)> {
        let mut pairs = Vec::new();
        for x in a {
            for y in b {
                for p in self.on(*x) {
                    for q in self.on(*y) {
                        pairs.push((p, q));
                        pairs.push((q, p));
                    }
                }
            }
        }
        pairs
    }

    /// The partition action `i` made heals: each pair it cut is joined again, unless another
    /// partition still cuts it.
    fn heal(&mut self, i: usize) {
        let Some((_, Fault::Partition { a, b, .. })) = self.schedule.faults.get(i).cloned() else {
            return;
        };
        let at = self.world.now();
        self.log.line(at, "nemesis", format_args!("heal {i}"));
        for pair in self.across(&a, &b) {
            let left = self.cuts.get(&pair).copied().unwrap_or(0).saturating_sub(1);
            if left == 0 {
                self.cuts.remove(&pair);
                self.net.partition(pair.0, pair.1, false);
            } else {
                self.cuts.insert(pair, left);
            }
        }
    }

    /// The processes of a machine.
    fn on(&self, machine: usize) -> Vec<NodeId> {
        self.machines
            .get(machine)
            .map(|(list, _)| list.clone())
            .unwrap_or_default()
    }

    fn nemesis(&mut self, i: usize) -> Result<(), SimError> {
        let at = self.world.now();
        let Some((_, fault)) = self.schedule.faults.get(i).cloned() else {
            return self.calm();
        };
        self.log.line(at, "nemesis", format_args!("{fault:?}"));
        match fault {
            Fault::Crash { machine } => {
                if self
                    .machines
                    .get(machine)
                    .is_some_and(|(_, power)| *power == Power::Up)
                {
                    self.ledger.fault(nemesis::CRASH, at);
                    self.down(machine, Power::Crashed)?;
                }
            }
            Fault::PowerCut { machine, side } => {
                self.ledger.armed_cuts.insert(machine, side);
            }
            Fault::Partition { a, b, lasts } => {
                self.ledger.fault(nemesis::PARTITION, at);
                for (p, q) in self.across(&a, &b) {
                    let n = self.cuts.entry((p, q)).or_insert(0);
                    *n = n.saturating_add(1);
                    self.net.partition(p, q, true);
                }
                if let Some(lasts) = lasts {
                    self.world.after(nanos(lasts), self.nemesis, Ev::Heal(i))?;
                }
            }
            Fault::SlowLink { from, to, path } => {
                self.ledger.fault(nemesis::SLOW_LINK, at);
                for p in self.on(from) {
                    for q in self.on(to) {
                        if p != q {
                            self.net.set_pair_path(p, q, path)?;
                            self.slowed.push((p, q));
                        }
                    }
                }
            }
            Fault::SlowDisk { machine, latency } => {
                self.ledger.fault(nemesis::SLOW_DISK, at);
                for node in self.on(machine) {
                    if let Some(Proc::Store(store)) =
                        proc_index(node).and_then(|i| self.procs.get_mut(i))
                    {
                        store.set_latency(latency);
                    }
                }
            }
            Fault::JumpClock { machine, by_ns } => {
                self.ledger.fault(nemesis::JUMP_CLOCK, at);
                for node in self.on(machine) {
                    self.world.step_wall(node, by_ns)?;
                }
            }
            Fault::LoseAnswer { machine } => {
                let left = self.ledger.lose_answers.entry(machine).or_insert(0);
                *left = left.saturating_add(1);
            }
        }
        Ok(())
    }

    /// The faults stop: partitions heal, links and disks take their own pace again, and every
    /// machine that is down starts again. A power cut or a lost answer armed before it still lands
    /// at its machine's next write or answer, and the run is quiet only once it has.
    fn calm(&mut self) -> Result<(), SimError> {
        let at = self.world.now();
        self.log.line(at, "nemesis", format_args!("calm"));
        self.calm_at = Some(at);
        self.net.heal();
        self.cuts.clear();
        for (p, q) in std::mem::take(&mut self.slowed) {
            self.net.set_pair_path(p, q, self.path)?;
        }
        for proc in &mut self.procs {
            if let Proc::Store(store) = proc {
                store.set_latency(Duration::ZERO);
            }
        }
        for machine in 0..self.machines.len() {
            self.restart(machine)?;
        }
        Ok(())
    }

    fn finish(mut self, end: End) -> Outcome {
        let mut waiters = Vec::new();
        let mut attempts = Vec::new();
        for (i, proc) in self.procs.iter().enumerate() {
            let name = self
                .names
                .get(i.saturating_add(1))
                .cloned()
                .unwrap_or_default();
            match proc {
                Proc::Client(client) => {
                    attempts.extend(client.attempts.iter().cloned());
                    if let Some((op, attempt)) = client.on() {
                        waiters.push(Waiter::Client { name, op, attempt });
                    }
                }
                // A node that holds requests and waits on no call has nothing that will move it: a
                // node waiting on a call is answered by its store or at the call's deadline.
                Proc::Node(shell) => {
                    let (jobs, calls) = shell.waiting();
                    if jobs > 0 && calls == 0 {
                        waiters.push(Waiter::Node { name, jobs, calls });
                    }
                }
                Proc::Store(_) => {}
            }
        }
        let log = self.log.digest();
        // The log's digest joins the world's: a run that logs differently digests differently.
        let mut word = [0u8; size_of::<u64>()];
        for (into, from) in word.iter_mut().zip(log.sha256.bytes()) {
            *into = from;
        }
        self.world.observe(u64::from_le_bytes(word));
        let tail = self.log.tail();
        let net = self.net.stats();
        let clocks = (0..self.procs.len())
            .map(|i| {
                let name = self
                    .names
                    .get(i.saturating_add(1))
                    .cloned()
                    .unwrap_or_default();
                let node = proc_node(i);
                let monotonic = self.world.monotonic(node).unwrap_or(0);
                let wall = self.world.wall(node).unwrap_or(0);
                (name, monotonic, wall)
            })
            .collect();
        let ledger = self.ledger;
        Outcome {
            record: self.world.finish(),
            end,
            log,
            tail,
            attempts,
            quiet_from: self
                .calm_at
                .map(|calm| calm.max(self.last_up).max(ledger.last_fault)),
            unlanded: ledger
                .armed_cuts
                .keys()
                .map(|m| (nemesis::POWER_CUT, *m))
                .chain(
                    ledger
                        .lose_answers
                        .keys()
                        .map(|m| (nemesis::LOSE_ANSWER, *m)),
                )
                .collect(),
            waiters,
            writes: ledger.writes,
            faults: ledger.faults,
            breaches: ledger.breaches,
            waits: ledger.waits,
            let_go: ledger.let_go,
            answers: ledger.answers,
            net,
            grants: ledger.grants,
            clocks,
        }
    }
}
