# Phase 13: Scale, drills and the release

**Goal RQ13.** Reliquary is shown to hold at the scale Hex runs and beyond, loses nothing in any
fire drill, installs on its own from its README, runs production kiln, and publishes its comparison
with every competitor for anyone to repeat.

**Delivers.** The evidence. A simulated fleet at Hex's stated scale. A capacity model checked in the
lab. Fire drills with counts. A signed release for every target. Dashboards, alerts and runbooks. A
security review. And a fresh pair of eyes on every criterion of every phase.

**Serves.** CP-T9.10 (AC-T9.10.1 to AC-T9.10.3, T-T9.10.1). **Needs.** Every phase before it; RQ13.1,
RQ13.4 and RQ13.6 can start once RQ9 is met.

**Order.** RQ13.1 to RQ13.6 side by side. RQ13.7 after RQ13.2. RQ13.8 after RQ13.3 and RQ13.6.
RQ13.9 last.

---

### RQ13.1 The fleet Hex runs, simulated

Needs: RQ9, RQ10 · Serves: CP-T9.10

Hex's stated target is a million workloads on ten thousand nodes in one cluster. Reliquary must feed
that fleet. Run `reliquary-sim` at that size: ten thousand nodes with their image stores, a million
starts drawn from a recorded mix of images, registries of a size to match, with the nemesis. Then
grow the registry's contents through the plan's scale sizes and show which costs stay flat. Nothing in
Reliquary may cost work in proportion to everything it holds while serving one request.

Done when:
- AC-RQ13.1.1 The simulation at Hex's stated scale completes with every start served, every answer matching the model, and no bound exceeded.
- AC-RQ13.1.2 For each request kind, work per request at the plan's three scale sizes is flat or grows no faster than the logarithm the design states, by counter.
- AC-RQ13.1.3 Memory per node in the simulation stays within the budget derived for it at every size.
- AC-RQ13.1.4 A burst in which every node asks for one new image at once is served with origin bytes within RQ10.7's bound and no request waiting past what its admission stated.
- AC-RQ13.1.5 The scale figures come from Hex's design record and published sources cited in `docs/research/`; none is picked here.

Tests:
- T-RQ13.1.1 [model] The simulation at Hex's scale with the nemesis → matches the model; liveness holds.
- T-RQ13.1.2 [exhaustion] The burst: every node starts the same unheld image in the same instant → served; origin within bound; no queue unbounded.
- T-RQ13.1.3 [edge] One repository with as many tags as the largest test size; one namespace with as many repositories; one blob linked by as many repositories → listings page; pulls and collection keep their cost.
- T-RQ13.1.4 [perf] Measure work per request at the three registry sizes → records of record; flat or within the stated logarithm.
- T-RQ13.1.5 [deadlock] The whole fleet at its admission limits at once, across records, bytes and peers → the simulation ends with all work done or refused, none waiting.

---

### RQ13.2 A capacity model, checked against real machines

Needs: RQ4.10, RQ9 · Serves: AC-T9.10.2

Simulation shows shape; hardware shows size. Write a model that predicts, from per-operation costs
measured on a machine and the limits measured on that machine, how many of each request a
deployment serves. Check it in the lab on one node and on three, at loads up to saturation. Then
state what the model predicts for the public scale figures in `docs/research/`, with its
assumptions, as a prediction and not a claim.

Done when:
- AC-RQ13.2.1 The model's predicted saturation throughput for each request kind is within the measured interval on one node and on three, on at least two different machines.
- AC-RQ13.2.2 Every input of the model is measured by `reliquary doctor` or `reliquary bench` on the machine in question; none is a number carried from another machine.
- AC-RQ13.2.3 The document separates what was measured, what was simulated and what is predicted, and says which hardware each measurement is from.
- AC-RQ13.2.4 Where a prediction depends on Mantle's or Amphora's own scaling, it cites their records and does not restate them.

Tests:
- T-RQ13.2.1 [perf] Saturation runs for each request kind on one node and three, of record → within the model's interval.
- T-RQ13.2.2 [edge] The same runs on a second, different machine → the model, fed that machine's measurements, holds there too.
- T-RQ13.2.3 [refusal] A model input missing on a machine → the tool says which measurement to run; it does not substitute a default.

---

### RQ13.3 Fire drills

Needs: RQ9.7, RQ12.6 · Serves: AC-T9.10.3

Run the drills the criterion names on the lab's fleet, under load, each more than once: a node
powered off, a disk lost, a network partition, a region lost. After each, verify every artifact that
was acknowledged. Keep each drill's record with its counts.

Done when:
- AC-RQ13.3.1 Artifacts lost in the fire drills (node off, disk lost, partition, region lost) is exactly 0, measured by the drills' counts (AC-T9.10.3).
- AC-RQ13.3.2 Each drill's record states what was running, what failed, what requests failed and why, how long recovery took and what bounded it.
- AC-RQ13.3.3 After each drill the fleet returns to its stated redundancy with no operator action, and says when it has.
- AC-RQ13.3.4 Each drill is one command and is repeated until its record shows the same outcome on consecutive runs.

Tests:
- T-RQ13.3.1 [failure] Node off, under load → no acknowledged artifact lost; in-flight requests resume.
- T-RQ13.3.2 [failure] Disk lost → repaired; every blob verifies.
- T-RQ13.3.3 [failure] Partition, each way of cutting three machines → the majority serves; the rest refuses typed; no divergence.
- T-RQ13.3.4 [failure] Region lost → RQ12.6's outcome.
- T-RQ13.3.5 [failure] Two of the above at once, within and beyond the stated failures to survive → within: as above; beyond: typed refusal of what is no longer safe, and nothing acknowledged is lost.

---

### RQ13.4 A security review that tries to break it

Needs: RQ9 · Serves: CP-T9.10

Write the threat model: who can reach which door with what, and what each must not be able to do.
Then attack it. Every parser has a fuzz target that has run to a stated coverage plateau. A fresh
subagent, given the threat model and a running instance, tries to read what it may not, write what it
may not, exhaust the node, poison a cache, a peer or a working set, and confuse the two doors. Every
finding becomes a failing test before its fix.

Done when:
- AC-RQ13.4.1 `docs/design/threat-model.md` lists each asset, each actor, each boundary and each mitigation, with the test that shows the mitigation.
- AC-RQ13.4.2 Every parser of external input (HTTP, HPACK, JSON, YAML, tar, gzip, zstd, zip, EROFS, JWT, DER, protobuf-JSON, Rego, the native wire) has a fuzz target, and each has run until new coverage stopped for the campaign's stated length.
- AC-RQ13.4.3 The adversarial review's findings are each recorded in `docs/bugs/` with a failing test, a fix and a re-check by a second fresh subagent.
- AC-RQ13.4.4 No secret appears in any log, event, error, metric or dump the review could obtain.
- AC-RQ13.4.5 Dependencies have no open advisory, and every vendored change is recorded.

Tests:
- T-RQ13.4.1 [hostile] The review's attempts: cross-tenant read by mount, by cache timing, by search counts, by peer holdings; privilege gain through token confusion between doors; path escape through every archive kind → each refused, each a test.
- T-RQ13.4.2 [hostile] Resource exhaustion through each door's every bound → typed refusals; service to others within its interval.
- T-RQ13.4.3 [hostile] A malicious upstream, peer, publisher and client, one at a time → none makes Reliquary serve wrong bytes or store unverified ones.
- T-RQ13.4.4 [non-vacuity] Measure each fuzz target's coverage → its corpus reaches the code it is for; a target that reaches nothing fails.

---

### RQ13.5 Operating it

Needs: RQ9 · Serves: AC-T9.10.1

Give an operator what they need in the middle of an incident. Dashboards for each component, from the rows
each shipped. Service objectives for pulls and pushes, with alerts on how fast the error budget
burns. A runbook for every alert. Commands to inspect a repository, a blob's state, a queue, a
region's lag, and to drain a node. Backups of the records to Mantle, restored in a drill.

Done when:
- AC-RQ13.5.1 Every alert has a runbook, and a fresh subagent following each runbook on a lab fault resolves it or reaches the stated escalation.
- AC-RQ13.5.2 Every metric and event in code is on a dashboard or is listed as deliberately not; the doc-truth test of RQ0.9 enforces it.
- AC-RQ13.5.3 A restore of the records from a backup, in the lab, yields a registry in which every artifact acknowledged before the backup verifies.
- AC-RQ13.5.4 Draining a node fails no request: in-flight ones finish or move, by counter.
- AC-RQ13.5.5 Objectives and alert thresholds are the operator's statements with documented starting points derived from the comparison's measured latencies; none is fixed in the binary.

Tests:
- T-RQ13.5.1 [happy] Plant each fault a runbook covers → its alert fires; the runbook resolves it.
- T-RQ13.5.2 [failure] The backup and restore drill → verified.
- T-RQ13.5.3 [happy] Drain a node under load → no failed request.
- T-RQ13.5.4 [edge] An alert's condition flapping at its threshold → one page, not many.

---

### RQ13.6 The release, and a stranger's first hour

Needs: RQ9 · Serves: AC-T9.10.1, T-T9.10.1

Ship Reliquary as its own product. One archive per supported target, each built and tested on its
own runner, signed, with its SBOM and provenance, published in Reliquary itself and verifiable with
stock tools. A README that takes someone from nothing to a working registry they pushed to and pulled
from. Documentation for configuration, both doors, every artifact kind, policies, operations and
upgrades, each example in it run by a test.

Done when:
- AC-RQ13.6.1 A fresh machine reaches a working Reliquary from its README alone, by a fresh subagent (T-T9.10.1), on Linux, macOS and Windows.
- AC-RQ13.6.2 Each target's archive passes the client matrix and the conformance suite on a runner of that target before it is published.
- AC-RQ13.6.3 Each archive's signature, SBOM and provenance verify with cosign and with `reliquary`, from Reliquary's own registry.
- AC-RQ13.6.4 Every command and configuration example in the documentation is executed by a doc-truth test.
- AC-RQ13.6.5 An upgrade from the previous release, and a rollback to it, are documented and drilled.
- AC-RQ13.6.6 The Linux archives run unchanged on the current releases of Debian, Ubuntu, Fedora, a RHEL-compatible distribution, openSUSE, Arch and Alpine, on x86_64 and arm64; the Windows archives on x86_64 and arm64; the macOS archive on arm64 alone, and an Intel Mac is refused by name.

Tests:
- T-RQ13.6.1 [happy] The fresh-subagent test on each platform → a push and a pull succeed within the README's steps, with no step the README lacks.
- T-RQ13.6.2 [refusal] An archive whose signature is stripped or altered → `reliquary` refuses to self-upgrade to it.
- T-RQ13.6.3 [edge] Install with no network after the download; with a data directory on a small disk; as an unprivileged user → works or refuses typed, saying what is needed.
- T-RQ13.6.4 [happy] The documentation's examples → all pass.
- T-RQ13.6.5 [edge] Install and run the client matrix's smoke test on each Linux distribution of AC-RQ13.6.6, on Windows on both architectures, and on macOS arm64 → each works from the same archive for its target.
- T-RQ13.6.6 [refusal] Run the macOS archive's installer on an Intel Mac (or under the x86_64 slice) → one typed line naming the platform as unsupported.

---

### RQ13.7 The comparison, final

Needs: RQ13.2 · Serves: AC-T9.10.2

Repeat Phase 4's comparison on the finished system, on one node and on a cluster, against CNCF
Distribution, Harbor, Zot and the Rust registries of RQ4.1 on the same hardware, and against ECR as the
hosted reference, measured from the same region. Add what only a cluster shows: behavior during garbage collection, a node's
loss, and overload. Publish every point.

Done when:
- AC-RQ13.7.1 Pull p99 against CNCF Distribution's on the same hardware is at most 100%, measured by bench `reliquary_compare` (AC-T9.10.2), and the document states the ratio at every point.
- AC-RQ13.7.2 At every point, the verdict is ten times better, at the hardware limit, or in the open list with its cause, as Phase 4 defines.
- AC-RQ13.7.3 During garbage collection, during a node's loss and past capacity, Reliquary's latency and goodput are compared with each competitor's in the same condition.
- AC-RQ13.7.4 The comparison with ECR states its conditions (region, instance, network) and is labeled as a different class where it is one.
- AC-RQ13.7.5 The document is published with the release, and a fresh subagent reproduces a sample from it.

Tests:
- T-RQ13.7.1 [perf] Run bench `reliquary_compare` over the full matrix → records of record for every registry and point.
- T-RQ13.7.2 [perf] Measure each registry during garbage collection, a node's loss and overload → records of record for each condition.
- T-RQ13.7.3 [happy] The reproduction of a sample by a fresh subagent → within intervals.

---

### RQ13.8 Production kiln runs on Reliquary

Needs: RQ13.3, RQ13.6 · Serves: AC-T9.10.1

The person says go. Then kiln's production registry, caches, artifacts and upgrades are Reliquary's,
through the merge train kiln's master plan describes: canaried, health-gated, each migration behind
its capability with read fallback for a release, each rollback drilled first. Built, merged and live
are said apart.

Done when:
- AC-RQ13.8.1 Production kiln runs on Reliquary with its comparison published, and Reliquary installs on its own (AC-T9.10.1).
- AC-RQ13.8.2 The person's go-ahead is recorded before anything lands or deploys.
- AC-RQ13.8.3 The pods' start times are later than the merge, and the commit each component serves is the one merged.
- AC-RQ13.8.4 Service objectives hold for 24 hours after the landing, by the monitors kiln names.

Tests:
- T-RQ13.8.1 [happy] The canary serves a share of pulls → objectives hold; then the fleet.
- T-RQ13.8.2 [failure] The rollback drill of this train in the lab → one release back restores service with nothing lost.
- T-RQ13.8.3 [happy] A mixed fleet, one runner a release behind → pulls, pushes, caches and upgrades work.

---

### RQ13.9 Every criterion, checked by someone who did not build it

Needs: RQ13.1 to RQ13.8 · Serves: CP-T9.10

Before the track is called met, a fresh subagent for each phase gets that phase's criteria and their
evidence, and nothing else, and tries to show one unmet. Anything it finds is fixed and checked
again by another. The registry of requirements links every criterion and test to its evidence.

Done when:
- AC-RQ13.9.1 Acceptance criteria and tests without linked evidence is exactly 0, measured by the citation gate over `docs/requirements.toml`.
- AC-RQ13.9.2 Each phase's adversarial check found no unmet criterion, or what it found was fixed and checked again first.
- AC-RQ13.9.3 STATUS.md's open list names every gap that is left, why, and what would close it.

Tests:
- T-RQ13.9.1 [happy] The citation gate on the final tree → every `AC-` and `T-` has evidence.
- T-RQ13.9.2 [hostile] The adversarial checks, one per phase → no criterion shown unmet on the final tree.
