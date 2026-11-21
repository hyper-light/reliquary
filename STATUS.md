# Status

Updated 2026-11-20. A piece is done when its tests pass in the gate (`scripts/gates.sh`: the whole gate on
Linux x86_64 with the lints of all five targets, and the gates a platform decides on macOS arm64, Windows
x86_64 and Linux arm64, `.kiln/platforms.yaml`) and the evidence its requirement names is recorded.

## Implemented

- **The bar** (kiln's CP-M1): a Cargo workspace whose lints deny panics, this repository's rules
  (`CLAUDE.md`), the requirement registry (`docs/requirements.toml`), the design's starting point
  (`docs/design/README.md`), the gate (`scripts/gates.sh`, run by kiln CI from `.kiln/ci.yaml`), and the
  vendoring tool (`scripts/vendor.py`, kiln's).
- **The `reliquary` binary** says what it is (`reliquary --version`, `reliquary --help`) and measures
  (`reliquary bench`, below); the registry is not in it yet.
- **The shared crates, vendored at kiln's revisions** (plan RQ0.1; `vendor/MANIFEST.toml`, built, not
  committed): hyper-raft's runtime, simulator, measurement, transport and timing crates at `e0613f52`;
  aws-lc-rs 1.18.2 with Mantle's changes; Mantle's client, protocol, HTTP codec and CRCs at `87ab09e1`;
  each with kiln's recorded changes byte for byte, but those of kiln's layout alone; and Amphora's
  `amphora-api` and embedded store at Amphora's main, `db991a05`, its pins held to this hyper-raft by a
  recorded change and the store's two calls of Amphora's own hyper-durable change made on kiln's by a
  second (Amphora's two tests of them pass on it). The workspace names them by name and version, patched
  to the copies (`Cargo.toml`), and builds aws-lc-sys as kiln does (`.cargo/config.toml`).
  `crates/reliquary/tests/vendored.rs` holds them to it: the pins equal kiln's manifest at `[kiln]`'s
  commit (AC-RQ0.1.2), one revision of each linked by `cargo metadata` (AC-RQ0.1.3), each exception to
  the -api rule a line of the manifest (AC-RQ0.1.5), each crate run with an answer its specification
  fixes (the store commits a row only while its key is absent), and kiln's tool refusing an unrecorded
  edit and stopping at a conflict (T-RQ0.1.1, .2, .5). The gate compiles aws-lc-sys's C for targets
  other than the host's with zig 0.16.0 (`scripts/zig-cc`, Hex's; `scripts/install-zig`, Shards' pin),
  which CI installs first.
  The whole workspace (every `crates/*` member, as the tree stood at 15:25) built for AC-RQ0.1.4's
  five targets on 2026-11-20, by hand until the gate's matrix runs them (RQ0.3): macOS arm64 and
  Windows x86_64 natively, RQ0.1's tests passing on both; Linux x86_64 and arm64 linked with
  cargo-zigbuild, the CLI's and the vendored crates' tests passing in a container of each with no
  network; Windows arm64 linked with cargo-xwin, not run, for want of a machine. RQ0.1's tree also built
  for each from a clean checkout after `cargo fetch`, from an empty target folder, without the network
  (macOS's sandbox denying it; on Windows, cargo offline) (T-RQ0.1.6). The tests that run kiln's tool
  or fetch kiln's manifest skip through the skip record where git, Python 3.11 or the network is
  missing; on Windows kiln's `vendor.py --abort` can't give up a stopped re-vendor (git's read-only
  objects, WinError 5), so its test records a skip there, and the fix is asked of kiln
  (`docs/design/plan/README.md`).
- **The rule on numbers, a gate** (plan RQ0.2; built, not committed). `docs/design/constants.md` is
  Mantle's inventory kept for Reliquary's code: a row for each number production code runs with, its
  kind, its value and what establishes it. `scripts/check-contracts.py`, Mantle's checker, reads every
  `const`, `static` and literal parameter in crates/, and beyond Mantle's, every other literal but 0 and
  1, where an estimator's figures stand. It fails on:
  - a number without a row, a row without a number, or a changed value;
  - a refused kind;
  - a basis that is one machine's measurement, a point off one curve, a picked value or another
    system's tuning;
  - a `learned` row that doesn't say what it measures, on what, where it is kept and how it follows
    a change;
  - a `derived` or `counted` row without the test that computes or pins it;
  - any item in `scripts/arbitrary-open.txt`, which stays empty; there is no `--known`.
  It runs first in `scripts/gates.sh`. `crates/reliquary/tests/contracts.rs` runs it on fixture trees
  (T-RQ0.2.1 to T-RQ0.2.7, AC-RQ0.2.1 to AC-RQ0.2.4) and over this tree. `docs/design/numbers.md` says,
  for each class of tunable, what replaces its constant and where its inputs are measured, and lists
  what an operator states, what a node measures and what it derives; `crates/reliquary/tests/numbers.rs`
  holds those to the plan and the code (AC-RQ0.2.5).
- **Observability's conventions** (plan RQ0.9; built, not committed). `reliquary_core::observe` holds
  the catalog of every metric and event, the OpenMetrics 1.0 encoder, and the event ring with its
  tracing layer: a bounded lock-free queue that lets its oldest line go rather than make an emitter
  wait, and says what it lost. `docs/observability.md` lists every metric, event and dashboard row.
  `crates/reliquary-core/tests/observe.rs` holds that list and the code to each other (AC-RQ0.9.1,
  AC-RQ0.9.4), lints every name and label (AC-RQ0.9.2, T-RQ0.9.1, T-RQ0.9.3), scrapes a running stub
  (T-RQ0.9.4) and floods the ring with its sink stalled (T-RQ0.9.5). The decisions are
  `docs/design/observability.md` O1 to O6, and the names were agreed with kiln's Observe owner on
  2026-11-20 (AC-RQ0.9.3; who that is, by the person's word that day, is recorded there).
- **The structural gates and the platform matrix** (plan RQ0.3; `docs/design/gates.md`; built, not
  committed). `scripts/gates.sh` runs contracts, fmt, clippy for each of the five targets, the production
  check (the no-panic lints on each target, then `unsafe` only in files CLAUDE.md §12 names,
  `scripts/check-unsafe.py`), recursion (`scripts/check-recursion.py`, Mantle's on the built code's call
  graph, following calls through trait objects and function pointers), cargo-deny 0.22.0 by its digest
  (`deny.toml`), the tests (`scripts/test.sh`: every test cites a registry ID and skips only through the
  skip record, `scripts/check-tests.py`; the suite; the record after it, pass or fail,
  `crates/reliquary-testkit`), and the vendor check, stopping at the first failure. `gates.sh here` runs
  the gates a platform decides, which kiln CI runs on macOS arm64, Windows x86_64 and Linux arm64
  (`.kiln/platforms.yaml`), and the suite runs in Debian, Red Hat, SUSE, Arch and Alpine containers on
  both architectures (`.kiln/distros.yaml`). `crates/reliquary/tests/gates.rs` plants what each gate must
  refuse (T-RQ0.3.1 to T-RQ0.3.7). The new CLAUDE.md Gates lines wait for the person's word.
- **The measurement tool, `reliquary bench`** (plan RQ0.7; `docs/design/measurement.md`, research/07;
  built, on branch rq0-ground, not merged; its two workflows ran in kiln CI on 2026-11-21). `crates/reliquary-bench` judges a point's units as
  Mantle's design does (order in time, one state or two, medians with order-statistic intervals, tails
  where enough operations support them, dashes where not, no mean), sizes a round by a count of
  operations, sets how many builds, process starts and rounds a point needs from the variation measured
  at each level (Kalibera and Jones), measures latency open loop from each intended start, compares two
  builds or settings in randomized interleaved cycles with a verdict only when intervals separate, holds
  every record to the load rule and a clean commit (a run of record first waiting, `bench settle`, for
  the average its build left to fall under the rule), and finds change points in a nightly series
  (E-Divisive, seeded from the history). Its tests cover T-RQ0.7.1 to .10; simulated A|A runs report a
  difference 0.03% to 0.29% of the time against the 5% AC-RQ0.7.3 allows. `.kiln/instructions.yaml`
  gates `bench/gated.toml`'s instruction counts under cachegrind on every change, and
  `.kiln/nightly.yaml` feeds the nightly p99 to kiln's metrics and the detector. Both ran by hand in
  kiln CI on branch rq0-ground on 2026-11-21: the gate passed twice in a Linux container, `judge` not
  separated between a commit and its parent (runs 01a1fc07f950, 01a1fc188d9a); the nightly built,
  measured open loop at the power point on an arm64 Mac runner, held its record not of record by the
  load rule (a load average of 7.90), fed nothing, and its detector read kiln's series through the
  metrics API (run 01a1fc1f55aa). No night is fed yet, so no alert has been raised live: scheduled
  nights run from main on the bench host, in a slot the person books on kiln's bench calendar, and are
  of record only if that slot lets its load average fall under 2 (it read 84 on 2026-11-21); the
  detector's first alert needs at least six fed nights and a change for the worse. Checked on macOS arm64: lints, the production check, every test of `reliquary` and
  `reliquary-bench` (the end-to-end `tests/bench.rs` among them), and the gate live (ten A|A gates all
  passed; three gates of a build planted with 10% more work all failed). On Windows x86_64 (jupiter),
  at rq0-ground's b95b7e3a on 2026-11-21: the crate's 79 tests and `tests/bench.rs`'s 6, the release
  build, and real runs: an open-loop point, not of record since Windows keeps no load average; an A|A
  comparison, not separated; and the gate with the OS's counter, refused, naming the platform.
  On Linux, in a Debian 13 arm64 container on the Mac (kiln's one Linux runner, adacachy, runs
  production, and `fedora` is stale in kiln): the crate's tests, the release build, and the cachegrind
  gate live (three A|A gates passed; two gates of a separate build with 10% more work failed, and passed
  when accepted). The detector read kiln's own metrics answer for `hyper-light/kiln`. The first record of record
  (AC-RQ0.7.4) is `bench/records/2026-11-21T02-30-24Z-judge-point.json`: `judge`'s p99 from intended
  starts, built from b0f11d6c on an Apple M4 Max Mac Studio and of record at all 17 readings, 15 process
  starts of 23 rounds of 368 operations, a median p99 of 10.09 µs within 3.84%; `bench/records/` also
  keeps one run not of record, as evidence of the format.
- **The reference model and the differential harness** (plan RQ0.5; `docs/design/model.md` H1 to H14,
  research/10; built, not committed). `crates/reliquary-api` holds the registry's meaning as a pure
  model: every endpoint of distribution spec v1.1.1 and the upload cancel of its `main`, with the spec's
  statuses and codes, each answer one function of the model's state (`answer::ANSWERS` lists all 116, and
  model.md's table is held to it). It also holds a seeded generator of sequential and concurrent
  histories, valid and broken in 29 ways; plans that interleave clients and lose answers; a
  linearizability checker on a counted budget; a shrinker to 1-minimal histories of the same failure;
  and hunts that report the seed a failure replays from. Its 17 tests (T-RQ0.5.1 to .6, AC-RQ0.5.1 to .4)
  run generated histories until every request kind, error code and answer occurs, with the invariants
  held after every request. They also catch four planted bugs, by comparison and by linearizability: a
  lost tag update, a manifest taken with a missing blob, a blob read through a repository it was never
  pushed to, and a non-linearizable tag read. RQ0.6, RQ1, RQ9 and RQ12 run their nodes through its
  `Registry` trait.
- **The seams to other products** (plan RQ0.4; `docs/design/seams.md` S1 to S11, research/08; built, not
  committed). `reliquary_core::seams` reaches Amphora's `Records` and Quiver's `Topics` as their owners
  wrote them. Where no product's `-api` crate gives a seam, it holds Reliquary's own: `Bytes` (Mantle),
  `Secrets` (Veil), `Upstream` (another registry), `Clock` and `Entropy`. Each seam has:
  - a simulator, in memory and bounded by what its maker states, that injects each refusal its trait
    names, an answer delayed past its caller's deadline, an answer lost after its effect, and a stream
    cut at any byte, row, message or instant, and (all but the clock's) a request that never arrives;
    it counts each it injected;
  - a conformance suite of one case per clause, 75 clauses in all, which the simulator passes now and
    the real implementation must pass when it lands. Quiver's own suite also passes against the topics
    simulator with nothing armed.
  Every method that waits takes a deadline its caller derives; Quiver's take none and are held to their
  callers' with `clock::within`. In `crates/reliquary-core/tests/`:
  - `doc_truth.rs` holds seams.md's clauses, methods and faults to the code (AC-RQ0.4.1, AC-RQ0.4.3,
    AC-RQ0.4.5);
  - `seams.rs` runs every suite and the plan's tests (T-RQ0.4.2 to .9), and shows the bytes suite fails
    a store that forgets create-once;
  - `memory.rs` moves 1, 64 and 1024 times as much through every seam under a counting allocator:
    objects of up to 16 MiB through `Bytes` and `Upstream`, tables and topics of up to 1024 pages,
    and as many leases, fills and sleeps. Each seam's heap peak is the same at every size: 21,584
    bytes for `Bytes`, one 16 KiB piece and its overhead (T-RQ0.4.1, AC-RQ0.4.4).
- **The simulated world** (plan RQ0.6; `docs/design/sim.md` W1 to W12, research/12; built, not
  committed). `crates/reliquary-sim` runs Reliquary on hyper-sim's world: machines whose clocks drift
  and jump; a network that delays, drops, duplicates, reorders and partitions; the records and bytes
  stores (RQ0.4's simulators) losing power before, after or under any write; clients running histories;
  and a nemesis that crashes a machine, cuts power under a write, partitions any set of machines, slows a
  link or a disk, jumps a clock and loses an answer, written out or drawn from the run's own stream.
  Every run is a function of its seed: run twice and from its trace, one digest (AC-RQ0.6.1). A node runs
  a `Core`, a state machine whose seam calls are data its shell makes; until RQ1's cores, the reference
  model run over the seams (`ModelNode`). A run is judged by its history against the model, its liveness
  once the faults stop, starvation, what still waits, leases honored twice and negative waits; each check
  is shown failing a defect planted for it. In `crates/reliquary-sim/tests/`, T-RQ0.6.1 to .8 and
  AC-RQ0.6.1 to .5: the crash sweep cut each of a push's 22 writes, to records and to bytes, before and
  after; a partition from the records store was refused typed and healed; contending tag moves all
  finished within their attempts; crossing resources left nothing waiting; every fault kind was counted.
  The world found RQ0.4's bytes simulator honoring a grant again after its clock stepped back; it now
  judges expiry against the latest wall time it read (seams.md S6, research/12 F13).
  Run on macOS arm64 and in Linux arm64 and x86_64 containers on the same Mac: every test passes, and
  the recorded seed's event log is the same bytes on all three (T-RQ0.6.8). Windows x86_64 waits for
  jupiter (its run is queued in kiln; the machine's link was down); Windows arm64 has no runner.

## Designed, not built

- **The architecture** (`docs/design/architecture.md`, decisions D1 to D28), each decision with its
  sources (`docs/research/`) and what would falsify it.
- **The plan** (`docs/design/plan/`): fourteen phases, RQ0 to RQ13, 126 items, each with its acceptance
  criteria and tests; every ID is in `docs/requirements.toml` beside kiln's. In kiln each phase is a goal
  on the Reliquary agent's stack and each item a step of that goal's plan. RQ0's plan was approved, and
  its steps are under way (2026-11-20).

## Next

- **RQ0, the ground:** the estimators that replace constants (RQ0.8), proved in the simulated
  world.
- **RQ1, the slice:** one binary that takes `docker push` and gives `docker pull`, passes the OCI
  conformance suite, and loses nothing it acknowledged (serves CP-T9.1).
- Then, side by side: RQ2 artifacts checked on push and kiln's registry moved here; RQ3 the native door
  on hyper-quic for Shards, Hex and kiln; RQ4 speed; RQ5 skills and MCP servers (CP-T9.2); RQ6 the catalog
  (CP-T9.3); RQ7 trust, identity and audit (CP-T9.4); RQ8 pull-through caches (CP-T9.5); RQ9 a cluster with online garbage
  collection (CP-T9.7).
- After those: RQ10 fast distribution (CP-T9.6); RQ11 kiln's own artifacts (CP-T9.8); RQ12 regions
  (CP-T9.9); RQ13 scale, drills and the release (CP-T9.10).

## Open

Everything the plan names. What Reliquary must ask of other owners before it can finish (Mantle's
in-process node and its grant, Shards' skill artifact type and client transport, Hex's image-store seam,
Veil's API, kiln's identity behind the `Identity` seam, Quiver's topics, Amphora across regions) is
listed in `docs/design/plan/README.md`.

- **Amphora on kiln's one hyper-raft.** Reliquary's copy of Amphora carries two changes of its own
  (`vendor/amphora/UPSTREAM.md`): its pins held to kiln's hyper-raft, and the store's held verdicts told
  through hyper-durable's `suspect` and `trust`, since kiln's hyper-raft lacks Amphora's
  `Replica::suspect_held` and `trust_held`. Both go once Amphora builds on kiln's hyper-raft, its change
  upstream in hyper-raft or the store's taken in Amphora: Amphora's owner's to do, asked through the
  coordinator, with one more: Amphora's `scripts/vendor.py` made kiln's, which `vendor.py check` asks
  of it. Until kiln vendors Amphora, Reliquary's Amphora pin is Amphora's own (`kiln_holds_none`).
- **What the seams wait on from other owners** (plan RQ0.4, `docs/design/seams.md`). Mantle's native
  door has no grant yet (RQ3.5): the bytes simulator's grant, a tag over one key, method, length, holder
  and expiry, stands in for it and follows Mantle's when its owner defines it. Veil has no `-api` crate,
  so `Secrets` is what Reliquary needs of it, written ahead. Quiver's `Topics` takes no deadline:
  Reliquary holds each call to its caller's with `clock::within`, and a deadline on each method, as
  Amphora's have, is a request to Quiver's owner.
- **Numbers in vendored code** (plan RQ0.2, RQ0.8). A vendored crate is its owner's to inventory. The
  gate reads a vendored path only once `docs/design/constants.md`'s "Vendored crates" section names it,
  as it will when RQ0.8 takes an estimator from hyper-timing (AC-RQ0.8.6). Nothing is forgiven, so a
  refused number there is replaced first, by hyper-raft's owner or by a change Reliquary records. Mantle
  names 3 of hyper-timing's numbers open at its revision of them.
- **Targets and families with no runner** (plan RQ0.3, AC-RQ0.3.2, AC-RQ0.3.7). Windows on arm64
  (`aarch64-pc-windows-msvc`) has no runner: the gate lints it, and its suite runs once a Windows on Arm
  machine is a kiln runner (Windows 11 on Arm in a VM on one of the Mac Studios, enrolled as a runner,
  would close it): the person's to provide. Arch Linux on arm64 has no container: Arch's official image is
  x86_64 alone; it closes when Arch publishes an arm64 image, or the person accepts Arch Linux ARM's own
  as the family's.
- **Advisories in what vendored Amphora brings** (plan RQ0.3, `deny.toml`). cargo-deny finds chrono
  unmaintained (RUSTSEC-2026-0543, through amphora-engine), nine unsound advisories in heapless 0.7.17
  (RUSTSEC-2026-0574 to 0582, through postcard 1.1, whose `^0.7` keeps out the fixes), and socket2 taken
  from git at a fixed commit. Reliquary may not change Amphora's dependencies, so each is an exception
  allowed only for Amphora's crate that brings it, expiring 2026-12-20; they close when Amphora moves off
  them, Amphora's owner's to do, asked through the coordinator.
- **The simulated world's stores** (plan RQ0.6, `docs/design/sim.md` "Open"). `SimBytes` keeps an
  open upload's bytes past its durable offset across a power cut, where Mantle loses them; a hook that
  drops them at a cut is RQ0.4's simulator's to grow (its session has ended: the person routes it) before
  RQ1.3's resumable uploads run in the world. `Bytes`' suite has no clause yet for expiry under a clock
  stepped back (its fixture's clock cannot step back): it comes with RQ1.8, which holds Mantle's grant to
  the suite, and Mantle's own grant is asked to keep the rule (`docs/design/plan/README.md`).
