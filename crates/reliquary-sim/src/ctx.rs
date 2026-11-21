//! What a process is lent while it handles an event: the world, the network, the log and the
//! ledger, and where it is. A process sends through it, and asks for what it cannot do to the
//! world itself (a power cut takes every process of a machine, the one asking among them) as an
//! [`Action`] the run carries out once the process is done.

use std::fmt;

use hyper_sim::net::{Dropped, Fate, Net};
use hyper_sim::{NodeId, SimError, World};

use crate::ledger::Ledger;
use crate::log::Log;
use crate::wire::{Ev, Msg, Reply};

/// What a process asks the run to do once it is done with its event.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// The machine loses power: every process on it stops, and comes back when the run restarts
    /// the machine.
    PowerCut { machine: usize },
}

/// A process's view of the world while it handles one event.
pub struct Ctx<'a> {
    pub(crate) world: &'a mut World<Ev>,
    pub(crate) net: &'a mut Net<Msg>,
    pub(crate) log: &'a mut Log,
    pub(crate) ledger: &'a mut Ledger,
    pub(crate) actions: &'a mut Vec<Action>,
    pub(crate) names: &'a [String],
    pub(crate) me: NodeId,
    pub(crate) machine: usize,
}

/// The name the log gives `node`.
pub(crate) fn name_of(names: &[String], node: NodeId) -> &str {
    usize::try_from(node.0)
        .ok()
        .and_then(|i| names.get(i))
        .map_or("?", String::as_str)
}

/// Why a message was dropped, in the log's words.
const fn dropped(why: Dropped) -> &'static str {
    match why {
        Dropped::Mtu => "mtu",
        Dropped::Queue => "queue",
        Dropped::Loss => "loss",
        Dropped::Partition => "partition",
        Dropped::Capacity => "capacity",
        Dropped::Nat => "nat",
    }
}

impl Ctx<'_> {
    /// The world's present.
    pub(crate) fn now(&self) -> u64 {
        self.world.now()
    }

    /// Writes a line of the log, as this process.
    pub(crate) fn log(&mut self, what: fmt::Arguments<'_>) {
        let at = self.world.now();
        let who = name_of(self.names, self.me);
        self.log.line(at, who, what);
    }

    /// Sends `msg` to `to` over the network.
    pub(crate) fn send(&mut self, to: NodeId, msg: Msg) -> Result<(), SimError> {
        let carried = msg.carried();
        let line = summary(&msg);
        let fate = self
            .net
            .send(self.world, (self.me, to), msg, carried, Ev::Arrive)?;
        let to_name = name_of(self.names, to).to_owned();
        match fate {
            Fate::Arrives { at } => self.log(format_args!("send {to_name} {line} arrives {at}")),
            Fate::Dropped(why) => {
                self.log(format_args!(
                    "send {to_name} {line} dropped {}",
                    dropped(why)
                ));
            }
        }
        Ok(())
    }

    /// Whether the nemesis armed this machine's next answer to be lost: if so, it is, counted, and
    /// the caller sends nothing.
    pub(crate) fn lose_answer(&mut self) -> bool {
        let machine = self.machine;
        let Some(left) = self.ledger.lose_answers.get_mut(&machine) else {
            return false;
        };
        *left = left.saturating_sub(1);
        if *left == 0 {
            self.ledger.lose_answers.remove(&machine);
        }
        {
            let at = self.world.now();
            self.ledger.fault(crate::nemesis::LOSE_ANSWER, at);
        }
        self.log(format_args!("answer lost after its effect"));
        true
    }

    /// Asks the run to do `action` once this process is done.
    pub(crate) fn act(&mut self, action: Action) {
        self.actions.push(action);
    }
}

/// A message in the log's words.
pub(crate) fn summary(msg: &Msg) -> String {
    match msg {
        Msg::Request {
            op,
            attempt,
            request,
        } => format!("request {op}.{attempt} {request}"),
        Msg::Reply {
            op, attempt, reply, ..
        } => match reply {
            Reply::Answer(answer) => format!("reply {op}.{attempt} {answer}"),
            Reply::Grant(offer) => format!(
                "reply {op}.{attempt} grant {} for {}",
                offer.grant.key, offer.digest
            ),
            Reply::Refused(refused) => {
                format!("reply {op}.{attempt} refused {}", refused.kind.name())
            }
        },
        Msg::Call {
            id,
            incarnation,
            call,
            ..
        } => format!("call {}.{} {}", incarnation, id.0, call.method()),
        Msg::Answer {
            id,
            incarnation,
            answer,
        } => format!("answer {}.{} {}", incarnation, id.0, answer.summary()),
        Msg::Redeem {
            op, attempt, grant, ..
        } => {
            format!("redeem {op}.{attempt} {}", grant.key)
        }
        Msg::Redeemed {
            op,
            attempt,
            content,
        } => match content {
            Ok(bytes) => format!("redeemed {op}.{attempt} {} bytes", bytes.len()),
            Err(refusal) => {
                use reliquary_core::seams::fault::Kind;
                format!("redeemed {op}.{attempt} refused {}", refusal.kind())
            }
        },
    }
}
