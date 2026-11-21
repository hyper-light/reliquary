# The simulated world

Status: built 2026-11-20 by plan item RQ0.6 (`crates/reliquary-sim`); not committed. Each decision says
what it chose, what it rests on (research/12, cited "F"), and what would falsify it (CLAUDE.md §5).

## What it is

A world in which Reliquary runs as it runs on machines, every input of which the world chooses, so a
run is a function of its seed. It is built on hyper-sim, the house's simulator (architecture D21): its
virtual time, each machine's clocks, its event queue, its named streams of draws, its network and its
run-twice check. On it, `reliquary-sim` puts:

| Part | Where | What it does |
|---|---|---|
| Machines and the run | `src/world.rs` | Machines with clocks and the processes they host; the loop that hands each event to its process; crashes, power cuts and restarts; the outcome |
| Nodes | `src/node.rs` | The `Core` a node runs, the `Io` it is lent, and the shell that makes its calls and holds them to their deadlines |
| Seam calls as data | `src/call.rs` | A core's calls on `Records` and `Bytes` and their answers, in the seams' own types |
| Stores | `src/store.rs` | Amphora's and Mantle's sides: RQ0.4's simulators behind the network, served in order, slowed, cut |
| Clients | `src/client.rs` | Scripts of requests; patience, counted attempts and retries; grants followed |
| The nemesis | `src/nemesis.rs` | Every fault kind, written out or drawn from the run's stream; the calm |
| Verdicts | `src/verdict.rs`, `src/sweep.rs` | The history against the model; liveness; leases and waits; the crash sweep |
| The model node | `src/model_node.rs`, `src/entry.rs` | The reference model run as a node over the seams: what nodes run until RQ1's cores |
| The log and the ledger | `src/log.rs`, `src/ledger.rs`, `src/clock.rs` | The event log and its digest; what a run tallies; each process's clocks |

Later items run their nodes in it: RQ1.11 the slice's durability, RQ3.9 the two doors, RQ4.4 the
caches, RQ7.13 events, RQ9 three nodes and collection, RQ12 regions, RQ13 Hex's scale.

## Decisions

### W1. One world, every choice drawn from it

**Chosen.** A run is one hyper-sim `World`: one event queue, under the ordered discipline by default
(the free one is a spec's choice), and one strategy choosing among ties. Every draw the run makes,
the network's fates, the nemesis's schedule, each node's identities, where under a piece a power cut
falls, is from a named stream of the world, so it is recorded in the run's trace and the run replays
from its seed or from its trace alone. The run folds its event log's digest into the world's, and a
test runs one seed twice and its trace once (`hyper_sim::twice`), every digest equal (AC-RQ0.6.1).

**Rests on.** F2 (every input the world's), F7 (determinism checked).

**Falsified by.** One seed giving two digests on one target, or two targets' logs differing for one
seed (`tests/determinism.rs`).

### W2. A node runs a core as a state machine; its seam calls are data

**Chosen.** What a node runs is a [`Core`](../../crates/reliquary-sim/src/node.rs): it is given a
client's request or a call's answer, and gives calls on its seams and replies, through an `Io` that
holds no seam. The node's shell makes each call: a message to the store its seam is on, answered by the
store through the seam's own trait, the answer an event. So every call is a point where anything else
in the world may run, and a crash can fall between any two.

**Rests on.** CLAUDE.md §9 ("New subsystems are pure cores (sans-IO) driven by shells"); F1. Amphora's
`Records` blocks until its deadline (seams.md S11), so a core that called it directly could not be
suspended between its calls, and two nodes' read and conditional commit could never interleave: the
races a registry must survive would never be run.

**Falsified by.** A core of RQ1 that cannot be written as a state machine of its calls, or a shell of
RQ1.8 that cannot make the same calls on the real `Records` and `Bytes`.

### W3. Each process's clocks are its machine's, on a portable grid

**Chosen.** A process's `Clock` is the hyper-sim clocks of its node: a monotonic clock with an offset
and a drift in parts per million, which never goes back, and a wall clock that moves with it and that
the nemesis steps forward or back. The world sets the readings before it hands the process an event.
Wall readings are put on a 100 ns grid, as a `SystemTime` made from a finer reading would be cut on
Windows alone (`clock.rs` `WALL_GRID_NS`). A store's simulator that keeps its own `SimClock` has it
moved to the store's monotonic reading before each call, and a deadline crosses processes as what is
left of it, not as an instant of another machine's clock.

**Rests on.** F3; F11.

**Falsified by.** A run differing between targets in a grant's or a lease's expiry.

### W4. The stores are RQ0.4's simulators behind the network

**Chosen.** The records store is a `SimRecords` and the bytes store a `SimBytes`, each a process of the
world, each answering calls through the seam's own trait. What the simulator holds is the store's
medium: a crash or a power cut of its machine loses what was in flight (calls queued, readings open,
answers not sent) and never what it committed or finished. A store serves calls in the order they
arrive; a slow disk serves each that much after the one before. Before each write the store asks the
run's plan whether power is cut before it (it never reaches the medium), after it (it applied, its
answer is never sent) or, for a piece of a put, under it (the stream cut at a byte inside the piece,
which keeps what was durable, seams.md BYTES-4). A node whose store is on another machine has its
power cut as a write leaves it.

**Rests on.** F1, F5; seams.md S2, S11.

**Falsified by.** A crash run passing in the world that the same core fails on real stores (RQ1.11's
kill runs on real file systems).

### W5. Clients run scripts and record every attempt on the world's time line

**Chosen.** A client sends its script's requests one at a time, each to a node, waits for each as long
as its patience, and tries again after a pause, on the next node, up to its counted attempts, a
refusal and an answer that never came alike. It follows a grant to the bytes store it names. Every
attempt is recorded with when it was invoked and how it ended, in the world's virtual time, which is
the history's one clock. A request delivered twice by the network is taken once by its node, a
transport's duty; a call delivered twice reaches its store twice, which the seams' conditions and
identities must make harmless (seams.md S7).

**Rests on.** F4; model.md H11, H12.

**Falsified by.** A history the checker passes for an implementation that a client of the real world
would see violate the model.

### W6. The nemesis: every fault kind, written out or drawn, then a calm

**Chosen.** The network is hyper-sim's: each path delays a message by its one-way time and jitter,
in order or letting later messages overtake, and loses what its loss process draws; every link
delivers a message twice with a stated chance. On it the nemesis has seven kinds (`nemesis::KINDS`):
crash a machine; cut power under a write; partition any set
of machines from any other, for a stated time; slow a link; slow a disk; jump a clock; lose an answer
after its effect. A schedule is a test's list, or drawn from the run's `nemesis` stream within an
envelope the test states, power cuts and lost answers among the machines that keep writing. A power
cut and a lost answer are armed and land at their machine's next write or answer. A crash refuses new
connections to its machine and resets the open ones, its clients and callers told at once that the
outcome is unknown; a machine with no power answers nothing. At the calm partitions heal, links and
disks take their own pace, and every machine down starts again; an armed fault lands even after it.
Each fault is counted by its kind when it takes effect (T-RQ0.6.7).

**Rests on.** F2, F4, F12, F15.

**Falsified by.** A fault kind a later item needs that the nemesis cannot give at a seam's boundary.

### W7. A run is judged by its history, its liveness and what still waits

**Chosen.** The history is every attempt: one answered is an operation with its answer; one with no
answer, or refused `Unknown`, may have applied, an operation whose answer was lost; one refused `Busy`
or `Unavailable` applied nothing and is no operation (model.md H7). It is checked for linearizability
against the model (RQ0.5's checker, on a budget the test states). The run is quiet from the calm, the
last fault's landing or the last restart, whichever is latest. Then:

- an attempt invoked once quiet that waited out its patience unanswered fails the run (AC-RQ0.6.5);
- an operation whose last attempt was invoked once quiet and that never completed is starved
  (T-RQ0.6.4);
- a node holding requests with no call pending has nothing that will move it (T-RQ0.6.5);
- a run that ends idle or out of steps with a client unfinished fails.

**Rests on.** F6; model.md H12.

**Falsified by.** A run passing these checks whose operation hangs once the faults stop.

### W8. The event log is the run's words, the same bytes on every target

**Chosen.** One line for everything the world did, in order: its virtual time, the process, and what
happened, in words with no host address, host time or unordered iteration. The log keeps its last lines
for a failure's report and its SHA-256 over every line, so it is bounded however long the run. One
seed's digest is recorded in `tests/determinism.rs` and checked on every target the suite runs on
(T-RQ0.6.8).

**Rests on.** F7, F11.

**Falsified by.** Two targets giving two digests for the recorded seed.

### W9. Leases are honored once, and no wait is negative

**Chosen.** The leases of a run are the grants its nodes issue and its bytes store honors (architecture
D4): the ledger records each grant's presentations, and one honored after it was refused as expired is
a lease honored twice. Every deadline a node's core sets before its own monotonic now is a negative
wait. Both fail a run (T-RQ0.6.6). The world found RQ0.4's bytes simulator honoring a grant again after
its wall clock was stepped back (F13); the fix, at its cause, judges expiry against the latest wall time
the store has read (seams.md S6), and Mantle's grant is asked to keep the same rule
(`plan/README.md`, "What the plan asks of other owners").

**Rests on.** F3, F10, F13.

**Falsified by.** A grant honored after its refusal in a run with clocks stepped back.

### W10. Until RQ1's cores, a node runs the reference model over the seams

**Chosen.** `ModelNode` decides every request with the model of RQ0.5 and keeps the registry's state as
the log of the model's changes: an entry a row of `Records` at its position, each content an object of
`Bytes` the entry names by key, length and SHA-256. Before a request, the node reads the log to its
head (one read at a time, joined only by requests that came before it started); it writes a change's
content first and its entry second, conditioned on the next position being absent; it answers only
once both are durable. Losing a position to another writer, it reads and decides again, within a
counted budget, then refuses `Busy`. An answer lost or late is settled by reading the position, the
same entry asked again if none is there; one that cannot be settled is refused `Unknown`. A restarted
node reads the log from its start. A handler leaves further work as steps a loop drains, so nothing
calls back into what called it (F14).

It is not the registry's core: RQ1.2 decides the records and transactions, and its cores take this
node's place. It is what the world's tests need now, a node whose writes, reads and failures run
through both seams.

**Rests on.** F9; seams.md S2, S3, S4, S7; model.md H1.

**Falsified by.** A history of the model node's that the checker refutes, or a crash run that loses
what it acknowledged.

### W11. Each check is shown to catch a defect planted for it

**Chosen.** Each check of the world runs once against a node with one thing wrong, a wrapper around the
model node that takes back its calls and replies (`Io::take_calls`, `Io::take_replies`) and holds,
drops or answers them itself (`tests/mutants`):

| Defect | Caught by |
|---|---|
| bytes said to be made before they are (`EagerFinish`) | the crash sweep: ten cuts between a commit and the object it names fail (T-RQ0.6.2) |
| a request never answered (`Silent`) | the liveness check names each attempt and the node's held requests (AC-RQ0.6.5) |
| the second contender always turned away (`OneAtATime`) | starvation (T-RQ0.6.4) |
| locks taken in request order, a lock at a time (`Locking`) | waiters with nothing pending (T-RQ0.6.5) |
| waits measured by the wall clock (`WallWaits`) | negative waits once a clock steps back (T-RQ0.6.6) |

**Rests on.** F8.

**Falsified by.** A planted defect a check passes.

### W12. The crash sweep cuts power before and after every write

**Chosen.** A run as it is gives every write its stores apply, in order. Then for each, one run with
power cut before it and one after it, each of the same seed, so each is the first run up to its cut,
each judged as any run is (AC-RQ0.6.3, T-RQ0.6.2). A history's reads after its writes make what was
acknowledged, and anything partial, part of what is judged.

**Rests on.** F5.

**Falsified by.** A write the sweep did not cut (the sweep counts every one, by seam and side).

## What the runs show (2026-11-20)

The suite on macOS arm64, and in containers of Linux arm64 and Linux x86_64 (emulated) on the same
Mac: every test passes on all three.

| Test | What it ran | What it found |
|---|---|---|
| T-RQ0.6.1 | 16 sequential histories of 60 requests and 12 concurrent plans, on a laptop and on three nodes | every operation answered first time; each sequential answer the model's; every history linearizable |
| T-RQ0.6.2, AC-RQ0.6.3 | a push of a layer in two chunks, a config and a tagged manifest, then its reads: 22 writes, 44 cut runs, on a laptop and a fleet | none lost or partial; cuts before and after records (6 each) and bytes (16 each) writes; the planted defect failed 10 cuts |
| T-RQ0.6.3 | three pushes, the node cut from its records 5 to 300 ms | every reply inside the partition a typed refusal; every operation completed after |
| T-RQ0.6.4 | two writers moving one tag 40 times each, 4 seeds, one node and two | the stores refused commits `Conflict`; every move completed within 8 attempts; the planted node starved a writer |
| T-RQ0.6.5 | two clients naming two repositories and two blobs in opposite orders, 6 seeds, three shapes | nothing waiting at the end; the planted locking node deadlocked and was named |
| T-RQ0.6.6 | 40 grant reads over a slow, duplicating link, clocks stepped back 40 ms three times, 8 seeds | grants honored, refused expired and presented again; none honored twice after the fix (F13) |
| T-RQ0.6.7, AC-RQ0.6.2 | every fault kind written out, and drawn runs of a three-node fleet | every kind counted above zero in every run; every run linearizable and live |
| AC-RQ0.6.2, T-RQ0.6.7 | every link between machines delaying, reordering and losing one message in twenty for 200 ms, one in twenty delivered twice | messages dropped, duplicated and overtaken (read from the event log); the history linearizable and live |
| T-RQ0.6.8, AC-RQ0.6.1 | one seed of a three-node fleet under drawn faults, grants and duplicated deliveries, 2,133 log lines | the same log SHA-256 and decision digest on macOS arm64, Linux arm64 and Linux x86_64; run twice and from its trace, one digest. Windows x86_64 waits for jupiter, whose link was down; Windows arm64 has no runner |

## Observability

The world is a test's tool and never runs in a deployment, so it has no metrics and emits no events of
the catalog's (docs/observability.md). What it shows is its outcome: its event log, its ledger's counts
and its verdict.

## Open

- **An upload's bytes past its durable offset outlive a power cut.** `SimBytes` keeps what an open
  upload received across a cut of its machine, where Mantle loses what was not yet durable. A core that
  relies on bytes received and not durable would pass here and fail on Mantle. It closes with a hook on
  `SimBytes` that drops every open upload's bytes past its durable offset at a power cut, RQ0.4's
  simulator's to grow (its session has ended; the person routes it), before RQ1.3's resumable uploads
  run in the world.
- **A clause for expiry under a stepped clock.** The bytes simulator now judges expiry against the latest
  wall time it read (W9), but `Bytes`' conformance suite has no clause for it: its fixture's `SimClock`
  cannot step a wall clock back. The clause and a fixture that can come with RQ1.8, where Mantle's
  real grant is held to the suite.
- **Seams not yet in the world.** `Secrets`, `Upstream` and `Topics` have simulators (RQ0.4) but no
  store process here; they join with the items that first use them (RQ7.13, RQ8).
- **Crash enumeration by fork** (hyper-sim's design §3.7) would cut the sweep's runs from twice the
  writes to one run and its branches; the world is not `Clone` while its stores' simulators sit behind
  `Arc`s.
