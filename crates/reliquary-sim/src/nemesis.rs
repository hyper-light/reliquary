//! The nemesis (docs/design/sim.md, W6): the faults a run suffers, at the times its schedule says,
//! then a calm in which every fault stops. It can crash a machine, cut power under a write,
//! partition any set of machines from any other, slow a link or a disk, jump a clock, and lose an
//! answer (plan AC-RQ0.6.2). A schedule is written out by a test, or drawn from the run's own
//! stream within an envelope the test states, so it replays from the seed or the trace like every
//! other choice. Each fault is counted by its kind when it takes effect (T-RQ0.6.7).

use std::time::Duration;

use hyper_sim::net::Path;
use hyper_sim::{SimError, StreamId, World};

use crate::ledger::Side;
use crate::wire::Ev;

/// The kinds of fault, as the ledger counts them.
pub const CRASH: &str = "crash a node";
pub const POWER_CUT: &str = "power cut";
pub const PARTITION: &str = "partition";
pub const SLOW_LINK: &str = "slow a link";
pub const SLOW_DISK: &str = "slow a disk";
pub const JUMP_CLOCK: &str = "jump a clock";
pub const LOSE_ANSWER: &str = "lose an answer";

/// Every kind (plan AC-RQ0.6.2).
pub const KINDS: &[&str] = &[
    CRASH,
    POWER_CUT,
    PARTITION,
    SLOW_LINK,
    SLOW_DISK,
    JUMP_CLOCK,
    LOSE_ANSWER,
];

/// One fault, on machines by their place in the run's spec.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Fault {
    /// Every process of the machine stops; it starts again after the run's downtime.
    Crash { machine: usize },
    /// The machine loses power at its next write, `side` of it (and starts again after the run's
    /// downtime).
    PowerCut { machine: usize, side: Side },
    /// Every pair of machines across the two sides is cut, both ways, for `lasts` (until the
    /// calm, with none); the rest keep their links.
    Partition {
        a: Vec<usize>,
        b: Vec<usize>,
        lasts: Option<Duration>,
    },
    /// Messages from one machine to another take `path`.
    SlowLink { from: usize, to: usize, path: Path },
    /// Every store on the machine serves each call `latency` after the one before.
    SlowDisk { machine: usize, latency: Duration },
    /// The machine's wall clock is stepped by this many nanoseconds, forward or back.
    JumpClock { machine: usize, by_ns: i64 },
    /// The next answer a process on the machine sends is lost after its effect.
    LoseAnswer { machine: usize },
}

impl Fault {
    /// Its kind.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::Crash { .. } => CRASH,
            Self::PowerCut { .. } => POWER_CUT,
            Self::Partition { .. } => PARTITION,
            Self::SlowLink { .. } => SLOW_LINK,
            Self::SlowDisk { .. } => SLOW_DISK,
            Self::JumpClock { .. } => JUMP_CLOCK,
            Self::LoseAnswer { .. } => LOSE_ANSWER,
        }
    }
}

/// When each fault happens, and when they all stop: at the calm, partitions heal, links and disks
/// take their own pace again, armed cuts and losses are disarmed, and every machine that is down
/// starts again. Clocks stepped stay stepped.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Schedule {
    pub faults: Vec<(Duration, Fault)>,
    pub calm: Duration,
}

/// What a drawn schedule may hold: every figure its maker's.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Envelope {
    /// The machines faults may touch.
    pub machines: Vec<usize>,
    /// The machines power cuts and lost answers fall on: each lands at its machine's next write or
    /// answer, so these are machines that keep writing and answering while any client works.
    pub stores: Vec<usize>,
    /// How many faults of each kind.
    pub each: u32,
    /// The span faults fall in: from the run's start, up to `calm`.
    pub calm: Duration,
    /// A slowed link's path.
    pub slow_path: Path,
    /// A slowed disk's latency.
    pub latency: Duration,
    /// How far a clock jumps, either way.
    pub jump_ns: i64,
    /// How long a partition lasts.
    pub lasts: Duration,
}

/// The sides a drawn power cut falls on.
const SIDES: &[Side] = &[Side::Before, Side::After, Side::Under];

fn below<E>(world: &mut World<E>, stream: StreamId, bound: usize) -> Result<usize, SimError> {
    let bound = u64::try_from(bound).unwrap_or(u64::MAX);
    let drawn = world.below(stream, bound)?;
    Ok(usize::try_from(drawn).unwrap_or(0))
}

impl Schedule {
    /// A schedule drawn from the world's `nemesis` stream within `envelope`: `each` faults of every
    /// kind, each at a time drawn in the span, on machines drawn among those it may touch.
    pub fn drawn(world: &mut World<Ev>, envelope: &Envelope) -> Result<Self, SimError> {
        let stream = world.stream("nemesis", &[])?;
        let span = u64::try_from(envelope.calm.as_nanos()).unwrap_or(u64::MAX);
        let count = envelope.machines.len();
        let mut faults = Vec::new();
        for kind in KINDS {
            for _ in 0..envelope.each {
                let at = Duration::from_nanos(world.below(stream, span)?);
                let pick = below(world, stream, count)?;
                let machine = envelope.machines.get(pick).copied().unwrap_or(0);
                let store = below(world, stream, envelope.stores.len())?;
                let store = envelope.stores.get(store).copied().unwrap_or(machine);
                let fault = match *kind {
                    CRASH => Fault::Crash { machine },
                    POWER_CUT => Fault::PowerCut {
                        machine: store,
                        side: SIDES
                            .get(below(world, stream, SIDES.len())?)
                            .copied()
                            .unwrap_or(Side::Before),
                    },
                    PARTITION => {
                        // A side of one to all but one of the machines, drawn by a partial shuffle.
                        let mut order = envelope.machines.clone();
                        let size = below(world, stream, count.saturating_sub(1))?.saturating_add(1);
                        for i in 0..size.min(count) {
                            let j =
                                i.saturating_add(below(world, stream, count.saturating_sub(i))?);
                            if j < count {
                                order.swap(i, j);
                            }
                        }
                        let b = order.split_off(size.min(order.len()));
                        Fault::Partition {
                            a: order,
                            b,
                            lasts: Some(envelope.lasts),
                        }
                    }
                    SLOW_LINK => {
                        let other = below(world, stream, count)?;
                        Fault::SlowLink {
                            from: machine,
                            to: envelope.machines.get(other).copied().unwrap_or(0),
                            path: envelope.slow_path,
                        }
                    }
                    SLOW_DISK => Fault::SlowDisk {
                        machine,
                        latency: envelope.latency,
                    },
                    JUMP_CLOCK => {
                        let ways = [true, false];
                        let back = ways
                            .get(below(world, stream, ways.len())?)
                            .copied()
                            .unwrap_or(true);
                        Fault::JumpClock {
                            machine,
                            by_ns: if back {
                                envelope.jump_ns.saturating_neg()
                            } else {
                                envelope.jump_ns
                            },
                        }
                    }
                    _ => Fault::LoseAnswer { machine: store },
                };
                faults.push((at, fault));
            }
        }
        faults.sort_by_key(|(at, _)| *at);
        Ok(Self {
            faults,
            calm: envelope.calm,
        })
    }
}
