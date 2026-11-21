# Phase 0: The ground

**Goal RQ0.** Every later change is made under gates that hold the rules, against seams that have
simulators, and judged by a measurement tool that states intervals.

**Delivers.** No registry yet. A workspace where a picked number, a recursion, a panic or an untested
target fails the gate; seams to Amphora, Mantle, Quiver and Veil with simulators that inject every
fault; a reference model and a simulated world to test the registry in; and a benchmark tool that
compares two builds honestly.

**Serves.** CP-T9.1 (the bar). **Needs.** Nothing.

**Order.** RQ0.1 first. RQ0.2, RQ0.3, RQ0.4, RQ0.5, RQ0.7 and RQ0.9 side by side after it. RQ0.6 after
RQ0.4 and RQ0.5; RQ0.8 after RQ0.6.

---

### RQ0.1 Vendor the shared crates at kiln's revisions

Needs: none · Serves: CP-T9.1

Take what Phases 0 and 1 link, each at the revision kiln's `vendor/MANIFEST.toml` holds: hyper-raft's
`hyper-rt`, `hyper-sim`, `hyper-measure`, `hyper-tls`, `hyper-quic`, `hyper-transport`,
`hyper-rt-transport` and `hyper-timing`; aws-lc-rs as kiln links it; Mantle's `mantle-client`,
`mantle-protocol`, `mantle-http` and `mantle-crc`; Amphora's `amphora-api` and its embedded store.
Name each in `vendor/MANIFEST.toml`, patch each name to its copy in `[patch.crates-io]`, and record in
the manifest, in Hex's words, the exception for crates that have no `-api` crate yet. Later phases add
what they first use (Shards' `http2`, `archive`, `sigstore`, `tuf`, `rego`; Quiver's `quiver-api`) by
the same rule. Nothing is vendored before something links it.

Done when:
- AC-RQ0.1.1 `python3 scripts/vendor.py check` passes: each vendored tree is its upstream at its revision plus its recorded changes, byte for byte.
- AC-RQ0.1.2 Each vendored project's revision equals the one kiln's manifest names, checked by a test that reads both manifests.
- AC-RQ0.1.3 The workspace links one revision of each shared crate: a test fails the build when `cargo metadata` shows two.
- AC-RQ0.1.4 The workspace builds for `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, `aarch64-apple-darwin`, `x86_64-pc-windows-msvc` and `aarch64-pc-windows-msvc`.
- AC-RQ0.1.5 Each exception to the `-api` rule is one line of `vendor/MANIFEST.toml` saying which crate, why, and what ends it.

Tests:
- T-RQ0.1.1 [happy] Re-vendor each project at its current revision → the tree is unchanged, by diff.
- T-RQ0.1.2 [refusal] Change one byte in a vendored file without recording it → `vendor.py check` fails and names the file.
- T-RQ0.1.3 [refusal] Point one project at a revision other than kiln's → the pin test fails and names both revisions.
- T-RQ0.1.4 [refusal] Add a dependency that pulls a second copy of `hyper-quic` → the one-revision test fails and names both paths.
- T-RQ0.1.5 [edge] A recorded change that conflicts with a new upstream revision → the re-vendor stops and names the file and the change.
- T-RQ0.1.6 [edge] Build each of the five targets from a clean checkout with no network after `cargo fetch` → each builds.

---

### RQ0.2 Make the rule on numbers a gate

Needs: RQ0.1 · Serves: CP-T9.1

Adopt Mantle's inventory and its checker. `docs/design/constants.md` gets one row for every number
Reliquary's production code runs with: its file and key, its kind, its value and what establishes it.
The accepted kinds are format, external, derived, cited, learned and counted. The refused kinds are
picked, knee, once, copied, guessed, inherited and open. `scripts/check-contracts.py` reads every
numeric `const`, `static` and literal parameter and fails the gate on a number without a row, a row
without a number, a value that differs, or an accepted kind whose stated basis is a refused one.
`scripts/arbitrary-open.txt` starts empty and the gate runs without `--known`: Reliquary has no
legacy numbers to forgive. `docs/design/numbers.md` says, for each class of tunable (waits,
concurrency, sizes of chunks, batches, queues and caches, rates, lifetimes), which estimator or model
replaces a constant and where its inputs are measured.

Done when:
- AC-RQ0.2.1 The gate fails on each of: an unlisted number, a row naming no number, a row whose value is not the code's, and a `learned` or `derived` row whose basis names one machine's measurement or a point read off one curve.
- AC-RQ0.2.2 The open list is empty, and the gate refuses a non-empty one.
- AC-RQ0.2.3 Every `learned` row names what is measured, on what (path, device, peer, repository or workload class), where it is kept, and how it follows a change.
- AC-RQ0.2.4 Every `derived` row names its model and its inputs, and a unit test computes the value from them.
- AC-RQ0.2.5 `docs/design/numbers.md` lists what an operator states, what a node measures and what it derives, and a doc-truth test checks each list against the configuration type and the code.

Tests:
- T-RQ0.2.1 [refusal] Add `const RETRIES: u32 = 3;` with no row → the gate fails and names the file and key.
- T-RQ0.2.2 [refusal] Add a row whose kind is `picked` → the gate fails.
- T-RQ0.2.3 [refusal] Add a `derived` row whose basis reads "measured on fedora" → the gate fails with "one machine's measurement".
- T-RQ0.2.4 [refusal] Change a listed constant's value in code only → the gate fails and shows both values.
- T-RQ0.2.5 [edge] A literal passed as a duration or capacity argument (`Duration::from_millis(500)`, `with_capacity(64)`) → found and held to a row like a named constant.
- T-RQ0.2.6 [edge] Zero as an empty count, and one as a first value or a `max(1)` floor → accepted without rows, as Mantle's checker accepts them.
- T-RQ0.2.7 [happy] The checker's own tests run it on a fixture tree with one of each violation → each is reported once, by name.

---

### RQ0.3 Structural gates and the platform matrix

Needs: RQ0.1 · Serves: CP-T9.1

Add the gates the rules need beyond lints: no recursion (Mantle's `check-recursion.py`, on the built
code's call graph), `unsafe` only in files CLAUDE.md names, `cargo deny` for advisories, bans,
licenses and sources, and a citation check that every test names a requirement that exists in
`docs/requirements.toml`. Lint all five targets and run the test suite on all five. A test that
cannot run where the gate runs skips only through a check that records what it lacked, as Hex's skip
record does. New lines for CLAUDE.md's Gates list go to the person for approval before they land.

Done when:
- AC-RQ0.3.1 `bash scripts/gates.sh` runs, in order: contracts, fmt, clippy for five targets, the production check, recursion, deny, tests, and the vendor check, and stops at the first failure.
- AC-RQ0.3.2 The test suite runs on Linux x86_64 and arm64, macOS arm64, and Windows x86_64 and arm64 on every change; a target with no runner is named in STATUS.md with what would close it.
- AC-RQ0.3.3 A call-graph cycle anywhere in production code fails the gate, named.
- AC-RQ0.3.4 Every `#[test]` cites an `AC-…` or `T-…` ID that the registry holds; an unknown or missing ID fails the gate.
- AC-RQ0.3.5 Every skipped test appears in the skip record printed after the suite, pass or fail.
- AC-RQ0.3.6 The proposed Gates lines are recorded as approved by the person, with the date, before CLAUDE.md changes.
- AC-RQ0.3.7 On Linux the suite also runs in containers of each major distribution family (Debian, Red Hat, SUSE, Arch, and Alpine with musl), on both architectures; a family that fails is named, never skipped in silence.

Tests:
- T-RQ0.3.1 [refusal] Add a function that calls itself → the recursion gate fails and names it.
- T-RQ0.3.2 [refusal] Add two functions that call each other through a trait object → the gate fails and names the cycle.
- T-RQ0.3.3 [refusal] Add `unsafe` in a file CLAUDE.md does not name → the gate fails.
- T-RQ0.3.4 [refusal] Add a dependency with an advisory, and one with a license outside the allow list → `cargo deny` fails each.
- T-RQ0.3.5 [refusal] Add a test citing `T-RQ9.99.9` → the citation gate fails.
- T-RQ0.3.6 [edge] A test that needs a privilege the runner lacks → it skips, and its line is in the skip record with what it lacked.
- T-RQ0.3.7 [edge] A test that returns early without recording a skip → the gate's audit of early returns flags it.
- T-RQ0.3.8 [happy] Run the gate on one commit on each of the five targets → it passes on each.
- T-RQ0.3.9 [edge] Run the suite in a container of each Linux distribution family, glibc and musl, on x86_64 and arm64 → it passes in each, or the record names the family and what it lacked.

---

### RQ0.4 Seams to other products, each with a simulator and a suite

Needs: RQ0.1 · Serves: CP-T9.1

Reliquary reaches records through `amphora-api`'s `Records`, events through `quiver-api`'s `Topics`,
and defines the narrow traits no `-api` crate gives yet: `Bytes` (a streamed put that resumes, a
ranged get, head, delete, create-once, and a grant another party can redeem at Mantle), `Secrets`
(a named credential leased, never stored), `Upstream` (another registry, for pull-through), `Clock`
and `Entropy`. Each trait is sans-IO, has a simulator that can inject every refusal the trait names,
a delay, a lost answer and a cut stream, and has a conformance suite that the simulator passes now and
the real implementation must pass when it lands.

Done when:
- AC-RQ0.4.1 Each trait's contract is a numbered list of clauses in `docs/design/seams.md`, and a doc-truth test checks each clause has a suite case.
- AC-RQ0.4.2 Each simulator passes its suite.
- AC-RQ0.4.3 Each simulator can inject: every typed refusal of its trait, an answer delayed past the caller's patience, an answer lost after the effect applied, and a stream cut at any byte.
- AC-RQ0.4.4 No trait carries a whole object in memory: bodies move as bounded pieces, and a counting-allocator test shows peak memory does not grow with object size.
- AC-RQ0.4.5 No trait method takes a timeout as a number: each takes a deadline its caller derives.

Tests:
- T-RQ0.4.1 [happy] Put an object many times larger than the memory budget through `Bytes` in pieces and get it back → identical bytes; peak heap within one piece plus the counted overhead.
- T-RQ0.4.2 [refusal] Create-once on a key that exists → refused as existing, and the first object is unchanged.
- T-RQ0.4.3 [edge] Resume a put from each possible durable offset after a cut → the object is whole, and no byte was sent twice past the last run.
- T-RQ0.4.4 [edge] Ranges at the first byte, the last byte, past the end, and of zero length → answered as RFC 9110 has them.
- T-RQ0.4.5 [race] Two create-once puts of one key at once → exactly one wins; the other is told it exists.
- T-RQ0.4.6 [failure] An answer lost after a commit applied → the caller reconciles by reading and never applies twice.
- T-RQ0.4.7 [exhaustion] The simulator refuses `Full` and `Busy` → the caller gets a typed refusal, and nothing is retried in a loop.
- T-RQ0.4.8 [hostile] A grant with a changed key, method, size or expiry → refused by the simulator's Mantle side.
- T-RQ0.4.9 [non-vacuity] The suite counts each injected fault kind per trait → every count is above zero.

---

### RQ0.5 A reference model and a differential harness

Needs: RQ0.1 · Serves: CP-T9.1, AC-T9.7.4

Write the registry's meaning as a small in-memory model: repositories, blobs, upload sessions,
manifests, tags, referrers and deletes, with the distribution spec's answers and error codes. Write a
generator of operation histories, concurrent and sequential, valid and invalid. The harness runs a
history against the model and against an implementation and compares every answer; for concurrent
histories it checks linearizability against the model. The same harness later drives one node, three
nodes and three regions.

Done when:
- AC-RQ0.5.1 The model answers every endpoint of distribution spec v1.1.1, with its status and error code, with no I/O, no concurrency and no cache: each answer is one function of the model's state.
- AC-RQ0.5.2 The generator produces every operation, every refusal the spec names, and interleavings of two to many clients; its counters show each was produced.
- AC-RQ0.5.3 The checker finds a planted bug in each of: a lost tag update, a manifest accepted with a missing blob, a blob readable through a repository it was never pushed to, and a non-linearizable tag read.
- AC-RQ0.5.4 A failing history is shrunk to a minimal one and replays from its seed.

Tests:
- T-RQ0.5.1 [model] Run generated histories against the model alone until every operation kind and error code has occurred → its own invariants hold throughout (a tag always names a manifest that exists; a manifest's blobs exist).
- T-RQ0.5.2 [model] Run the model against itself with one planted fault per run → each fault is reported; none is missed.
- T-RQ0.5.3 [edge] Histories with zero operations, one operation, and only refused operations → pass.
- T-RQ0.5.4 [race] Two clients moving one tag while a third reads it → every read is a value some linearization allows.
- T-RQ0.5.5 [non-vacuity] Read the counters after a run → each operation kind and each error code is above zero; a run where one is zero fails.
- T-RQ0.5.6 [happy] Run again with a failing seed that one run printed → the same failure, at the same operation.

---

### RQ0.6 A simulated world with a nemesis

Needs: RQ0.4, RQ0.5 · Serves: CP-T9.1, CP-T9.7, CP-T9.9

Build `reliquary-sim` on `hyper-sim`: nodes that run the real cores against the simulated seams,
clients that run histories, a network that delays, drops, duplicates, reorders and partitions, disks
that lose power between any two writes, and clocks that drift and jump. A nemesis schedules faults.
Every run is a function of its seed.

Done when:
- AC-RQ0.6.1 The same seed gives the same history and the same verdict on every supported target.
- AC-RQ0.6.2 The nemesis can crash a node, cut power under a write, partition any set of nodes, slow a link or a disk, jump a clock, and lose an answer.
- AC-RQ0.6.3 Crash injection covers every write: a sweep cuts power before and after each write a history makes to records and to bytes.
- AC-RQ0.6.4 No simulated wait is a wall-clock sleep: a grep for `thread::sleep` and runtime sleeps in tests finds none.
- AC-RQ0.6.5 A liveness check fails a run in which a client's operation neither completes nor is refused once faults stop.

Tests:
- T-RQ0.6.1 [happy] A history with no faults → every operation completes and matches the model.
- T-RQ0.6.2 [crash] Power cut at each write of a push, one run per write → after restart no acknowledged operation is lost and no partial object is visible.
- T-RQ0.6.3 [failure] A partition between a node and its records → operations are refused typed, and complete after the partition heals.
- T-RQ0.6.4 [livelock] Two clients retrying conflicting tag moves forever → both finish within a counted number of attempts; the run fails if either starves.
- T-RQ0.6.5 [deadlock] A history that takes every pair of resources in both orders → no run ends with waiters and no progress.
- T-RQ0.6.6 [edge] A clock that jumps backward by more than any lease in the run → no lease is honored twice and no wait becomes negative.
- T-RQ0.6.7 [non-vacuity] Read the fault counters after a nemesis run → each fault kind is above zero.
- T-RQ0.6.8 [happy] One seed run on Linux, macOS and Windows → the three event logs are byte-identical.

---

### RQ0.7 A measurement tool that states intervals and compares two builds

Needs: RQ0.1 · Serves: CP-T9.10, AC-T9.1.4

Build `reliquary bench` on Mantle's measurement design: a point is measured in rounds of a counted
number of operations; rounds are tested for order in time before any interval is stated; a point's
rounds are tested for one state or two; each state reports its median with its order-statistic
interval and latency quantiles only where enough samples support them. Add an A|B mode: two builds or
two settings run in interleaved, randomized rounds on one machine in one session, and a verdict is
given only when their intervals separate. Every record names its hardware, commit, date and load,
and is "of record" only when the load rule held. Latency is measured against intended send times, so
a stalled server cannot hide its stalls.

Done when:
- AC-RQ0.7.1 A point's report follows Mantle's `docs/design/measurement.md` §2 to §6: order in time, states, medians with intervals, and no mean.
- AC-RQ0.7.2 A|B mode reports "A better", "B better" or "not separated", with both intervals, and never a verdict from non-overlapping means alone.
- AC-RQ0.7.3 An A|A run (the same build twice) reports a difference no more often than the tests' stated level, 5%, the level Mantle's measurement design uses.
- AC-RQ0.7.4 A record made above the load rule is marked "not of record" and cannot be cited by an acceptance criterion.
- AC-RQ0.7.5 Open-loop load generation records each operation's intended start, and latency is measured from it.
- AC-RQ0.7.6 CI gates instruction counts for the benches it names, and a nightly run feeds latency to a change-point detector whose alert names the commit range.
- AC-RQ0.7.7 How many builds, process starts and rounds a point is measured over follows from the variation measured at each level and the interval asked for; the record states each count and why, and no "best of N" has a picked N.
- AC-RQ0.7.8 The change-point detector gives the same verdict on the same history every time it is run.

Tests:
- T-RQ0.7.1 [happy] Measure a synthetic operation with a known latency distribution → the reported quantiles' intervals contain the true values.
- T-RQ0.7.2 [edge] A synthetic operation with two states, the minority above the share the dip test can see → two states are reported, each with its share.
- T-RQ0.7.3 [edge] Rounds that drift upward in time → reported as ordered, with no interval stated.
- T-RQ0.7.4 [refusal] Ask for a p99.9 from too few samples → a dash, not a number.
- T-RQ0.7.5 [failure] A server that stalls for a second mid-run under open-loop load → the stall appears in the tail; under a closed-loop control run it would not, and the test shows the difference.
- T-RQ0.7.6 [happy] A|B of a build against one with a planted slowdown wider than the point's interval → "B worse", intervals separated.
- T-RQ0.7.7 [refusal] A|B of two identical builds, repeated → verdicts of a difference stay within the stated 5% level, by a binomial test.
- T-RQ0.7.8 [edge] A benchmark started while another process loads the machine → the record says "not of record" and names the load.
- T-RQ0.7.9 [edge] A synthetic benchmark whose variation lies between process starts and not between rounds → the tool repeats process starts, not rounds, and the record says so.
- T-RQ0.7.10 [edge] Run the detector twice over one recorded series with a planted shift → the same change point both times, at the planted commit range.

---

### RQ0.8 Estimators for everything that would have been a constant

Needs: RQ0.1, RQ0.6 · Serves: CP-T9.1

Write the small, sans-IO estimators the registry's waits, limits and sizes come from, and prove each
in simulation. Waits follow the person's rule: at a cold start, and after a wait runs out, a wait
lasts while the other party shows liveness and progress; after that its bound is the slowest answer
of its kind that path has shown. Connection and handshake times use RFC 6298's estimator. An
admission limit is the point of greatest power, found by probing as Mantle's node finds it. How many
transfers run at once, and how large a chunk is, are what pays on the path, as Shards' D226 learns
them. Each estimator keeps what it learned under the thing it is a property of (a path, a peer, a
device, a repository, a class of request), merges across processes, and ages so that a change in the
world moves it. Use hyper-timing's and Shards' and Mantle's implementations where they exist rather
than write a second one.

Done when:
- AC-RQ0.8.1 Each estimator is a pure state machine with no clock, socket or file of its own.
- AC-RQ0.8.2 No estimator starts from a guessed value: its cold-start behavior is stated and is a function of liveness, progress or a protocol's own figure.
- AC-RQ0.8.3 In simulation each estimator converges on a stationary path, and follows a step change, a slow drift and a two-state path, with the lag stated and bounded by samples, not time.
- AC-RQ0.8.4 What is learned is keyed by what it describes and never by the machine alone; a record learned on one path is not applied to another.
- AC-RQ0.8.5 Records written by two processes at once merge without losing either's samples.
- AC-RQ0.8.6 Every figure inside an estimator has a row in `docs/design/constants.md` of an accepted kind.
- AC-RQ0.8.7 Each estimator states its objective, the range its output cannot leave (derived, never picked) and what it does when its feedback stops or contradicts itself.
- AC-RQ0.8.8 Learning is not assumed to win: where a cited source measured a fixed rule (a share, a percentile), the estimator is adopted only if an A|B on recorded traces shows it no worse; otherwise the cited rule stands, as a `cited` row with the range over which its source found it insensitive.

Tests:
- T-RQ0.8.1 [happy] A path with a fixed latency → the wait settles above it and no healthy request is cut.
- T-RQ0.8.2 [edge] Latency doubles mid-run → no more than the stated number of requests are cut before the wait follows.
- T-RQ0.8.3 [edge] Latency halves mid-run → the wait comes down, by the aging rule, within the stated number of samples.
- T-RQ0.8.4 [failure] A peer that accepts a request and then goes silent → the wait ends when liveness ends, not at a fixed time.
- T-RQ0.8.5 [edge] A cold start against a path slower than any seen before → the first request is not cut while the peer shows progress.
- T-RQ0.8.6 [hostile] A peer that sends one byte just often enough to look alive → progress, not liveness alone, bounds the wait, and the transfer is cut and told as stalled.
- T-RQ0.8.7 [livelock] An admission limit probing around its base while load oscillates at the probe period → the limit does not chase the oscillation to its floor.
- T-RQ0.8.8 [edge] The limit when no request was ever turned away → unchanged: a phase that measured its clients moves nothing.
- T-RQ0.8.9 [race] Two processes ending transfers at once write one path's record → both samples are in the merged record.
- T-RQ0.8.10 [exhaustion] More distinct paths than the record store's bound → the least recently used are evicted by the stated rule, and a typed count says how many.
- T-RQ0.8.11 [failure] Feed an estimator feedback that is wrong (successes reported as failures), then none at all → its output stays inside its stated range; it falls back as stated; the anomaly is told as an event.
- T-RQ0.8.12 [refusal] An estimator that loses its A|B to the cited fixed rule on the recorded traces → it is not adopted: the record names the fixed rule as the winner and the constants table carries it as `cited`.

---

### RQ0.9 Observability from the first line

Needs: RQ0.1 · Serves: CP-T9.1

Set the conventions before there is anything to observe. Metrics are `reliquary_*` in base units with
no label whose values a client chooses. Events and spans go on dotted targets with an `event` field.
`docs/observability.md` lists every metric, event and dashboard row, and a doc-truth test holds the
list and the code to each other. Agree the names with kiln's Observe owner by message, and keep the
answer in the design record.

Done when:
- AC-RQ0.9.1 A metric or event in code that the document lacks fails a test, and so does a documented one the code lacks.
- AC-RQ0.9.2 A label whose values come from a request (a repository, a tag, a digest, a principal) fails a lint.
- AC-RQ0.9.3 The naming scheme is recorded as agreed with Observe's owner, with the message's date.
- AC-RQ0.9.4 Every component added later ships its dashboard rows in the same change, checked by the same test.

Tests:
- T-RQ0.9.1 [refusal] Add a counter with a `repository` label → the lint fails.
- T-RQ0.9.2 [refusal] Add a metric without documenting it → the doc-truth test fails and names it.
- T-RQ0.9.3 [refusal] Name a metric `reliquary_pull_ms` → the unit lint fails and asks for `_seconds`.
- T-RQ0.9.4 [happy] Scrape a running stub → every sample parses as OpenMetrics and has a HELP and a TYPE.
- T-RQ0.9.5 [exhaustion] Emit events faster than the sink takes them → the oldest are dropped by the stated rule and a counter says how many; the emitter never blocks.
