# Phase 12: Regions

**Goal RQ12.** Reliquary runs in several regions as one registry. A pull is served in the client's
own region. A tag pushed in one region is visible in the others within a second. Losing a region
loses no artifact and fails no pull of what was replicated.

**Delivers.** Repositories replicated between regions by a rule the operator states. One stated rule
for what happens when the same tag moves in two regions at once. The same test suite passing on one
node, three nodes and three regions.

**Serves.** CP-T9.9 (AC-T9.9.1 to AC-T9.9.3, T-T9.9.1). **Needs.** RQ9; RQ8.3 and RQ8.4 (a region
fetches from another as a cache fetches from an upstream); RQ7.3 where tags carry signed statements.

**Order.** RQ12.1 first. RQ12.2, RQ12.3 and RQ12.4 side by side. RQ12.5 and RQ12.6 after them.
RQ12.7 closes.

---

### RQ12.1 What a region is, and what crosses between them

Needs: RQ9 · Serves: CP-T9.9

Decide the model with Amphora's and Mantle's owners, from what each gives across regions. A region is
a whole registry: its nodes, its records and its bytes. A repository has a replication rule: which
regions hold it, and whether a push waits for other regions before it is acknowledged. A repository
is either homed, with one region ordering its writes and the others forwarding to it, or
multi-writer, with every region accepting writes that converge by a stated rule. State what each
choice promises a client and what it costs when a region is unreachable. The laptop is a deployment
of one region and runs the same code.

Done when:
- AC-RQ12.1.1 The design record states the model, what Amphora and Mantle each provide across regions as their owners confirmed, what Reliquary adds, and what would falsify the choice.
- AC-RQ12.1.2 For each of homed and multi-writer, the record states: what a read may return, when a write is acknowledged, and what happens to reads and writes when the home or a peer region is unreachable.
- AC-RQ12.1.3 The replication rule is the operator's statement per namespace or repository; Reliquary's only default is one region.
- AC-RQ12.1.4 A model of the tag register across regions is checked for convergence and for the stated guarantees, in the gate.

Tests:
- T-RQ12.1.1 [model] The model with three regions, partitions and concurrent moves → every execution converges; a homed repository's history is linearizable; a planted missing rule → a counterexample.
- T-RQ12.1.2 [happy] A single-region deployment runs the differential suite → identical to Phase 9's.
- T-RQ12.1.3 [refusal] A replication rule naming a region that does not exist, or asking for more acknowledging regions than hold the repository → refused when stated.

---

### RQ12.2 Records between regions, within a second

Needs: RQ12.1 · Serves: AC-T9.9.2

Carry each repository's changes from the region that committed them to the regions that hold it, in
order, over hyper-quic links between regions. A change is applied once, however many times it is
delivered. A region that was cut off catches up from where it stopped; one that fell behind the
others' retained history rebuilds the repository from a snapshot. How far behind each region is, is
measured and shown.

Done when:
- AC-RQ12.2.1 Tag replication between regions, p99, is at most 1 s, measured by a simulated 3-region lab at 60 ms apart (AC-T9.9.2).
- AC-RQ12.2.2 After the system is quiet, every region holding a repository has identical tags, manifests and referrers for it, compared row for row.
- AC-RQ12.2.3 A change is applied exactly once in each region under duplicated, reordered and replayed delivery.
- AC-RQ12.2.4 A region's lag is a metric, and a reader can ask a region to answer only once it has caught up to a point the reader names.
- AC-RQ12.2.5 Batching of changes on a link has no timer and no fixed size: what waited goes, as the link's credit allows.

Tests:
- T-RQ12.2.1 [perf] The 3-region lab at 60 ms: push tags in one region under load, measure visibility in the others → p99 at most 1 s, of record.
- T-RQ12.2.2 [failure] Cut one link for a long time, then heal → the region catches up; states equal.
- T-RQ12.2.3 [failure] A region is behind past the others' retained history → it rebuilds from a snapshot and says so; states equal.
- T-RQ12.2.4 [hostile] Deliver every change twice, out of order, and replay an old stream → states equal; nothing applied twice.
- T-RQ12.2.5 [race] A repository deleted in one region while pushed in another → the stated rule decides; all regions end the same.
- T-RQ12.2.6 [exhaustion] A burst of changes larger than a link can carry in a second → lag grows and is reported; nothing is dropped; lag returns to its usual level after.
- T-RQ12.2.7 [crash] Kill the replicator at every write → after restart it resumes from its durable position; states equal.
- T-RQ12.2.8 [livelock] Two regions each sending the other the change it just received from it → a change applied from a region is never sent back to it; the planted echo stops after one hop.

---

### RQ12.3 Bytes between regions

Needs: RQ12.1, RQ8.4 · Serves: AC-T9.9.1, AC-T9.9.3

A tag is no use in a region that cannot produce its bytes. For a repository the rule replicates
eagerly, blobs are copied to its other regions as they are pushed, and the tag becomes visible in a
region only when that region can serve its whole closure. For one replicated lazily, a region that
lacks a blob fetches it through from a region that has it, as RQ8.4 fetches from an upstream:
once for all who ask, verified, kept. A region never serves a manifest whose blobs it can neither
hold nor fetch.

Done when:
- AC-RQ12.3.1 Under eager replication, a tag visible in a region is always pullable there in full with every other region unreachable.
- AC-RQ12.3.2 Under lazy replication, a first pull in a region completes with bytes fetched through, and the next is served locally.
- AC-RQ12.3.3 Bytes copied between regions for one blob wanted by many clients at once is the blob's size, plus resumes.
- AC-RQ12.3.4 A push that the rule says must be durable in more than one region is acknowledged only when it is.
- AC-RQ12.3.5 Inter-region transfer takes a share of the link the operator states or the node derives from what foreground traffic leaves; it has no fixed rate.

Tests:
- T-RQ12.3.1 [happy] Push in one region under eager replication → pullable in the others; cut every link → still pullable in each.
- T-RQ12.3.2 [happy] Push under lazy replication; pull from another region → served through; pull again → local.
- T-RQ12.3.3 [failure] The source region dies mid-copy → the tag is not yet visible in the destination; it appears when the copy completes from another region or after the source returns.
- T-RQ12.3.4 [race] Many clients in one region pull a blob it lacks → one fetch between regions.
- T-RQ12.3.5 [refusal] A push requiring two regions while only one is reachable → refused typed, or held, as the rule says; never acknowledged early.
- T-RQ12.3.6 [hostile] A region (simulated as compromised) sends bytes that do not match the digest → refused; nothing stored.

---

### RQ12.4 The same tag moved in two regions at once

Needs: RQ12.1 · Serves: T-T9.9.1

State the rule and hold every region to it. For a homed repository there is no such thing: the home
orders the moves, and a region that cannot reach the home refuses the move, typed. For a multi-writer
repository the later move wins, where "later" is a hybrid logical clock's order and a tie goes to a
fixed order of regions, so every region picks the same winner without talking. The loser's push is
not lost: its manifest stays, by digest, and an event tells its pusher the tag went elsewhere. An
immutable tag can only be homed.

Done when:
- AC-RQ12.4.1 A tag moved in two regions at once ends the same in both, by the stated rule (T-T9.9.1), in every interleaving the simulator produces.
- AC-RQ12.4.2 The winner under multi-writer is a function of the two moves alone: any region computes it from them, in any delivery order.
- AC-RQ12.4.3 The losing move's manifest remains pullable by digest, and an event names both digests and the rule that decided.
- AC-RQ12.4.4 Immutability is never broken: an immutable tag in a multi-writer repository is refused when the rule is stated.
- AC-RQ12.4.5 A clock that is wrong in one region cannot make that region win forever: the logical part of the clock bounds its lead, and the test shows it.

Tests:
- T-RQ12.4.1 [race] Move one tag in two regions in the same instant, repeated under every delivery order and delay → both regions end at the same digest.
- T-RQ12.4.2 [race] Three regions move the tag at once, with one partitioned until later → all three end the same.
- T-RQ12.4.3 [edge] Two moves with equal clocks → the fixed order of regions decides, the same everywhere.
- T-RQ12.4.4 [edge] A move and a delete of the tag at once → the stated rule; the same everywhere.
- T-RQ12.4.5 [hostile] One region's wall clock set a day ahead → its moves win only until the others' clocks pass what it stamped; it does not win a move made after its own.
- T-RQ12.4.6 [refusal] A homed repository's tag moved in a region cut off from the home → refused typed, saying where the home is.
- T-RQ12.4.7 [model] The register's model with generated concurrent moves → convergence in every run.

---

### RQ12.5 Pulls in the client's own region

Needs: RQ12.2, RQ12.3 · Serves: AC-T9.9.1

A client reaches the region the operator's naming sends it to, and that region answers from its own
nodes and bytes. Where it lacks something, it fetches through and says so. Kiln's runners are
pointed at their own region.

Done when:
- AC-RQ12.5.1 Kiln's runners pull from their own region, in a change request that passes the gate (AC-T9.9.1).
- AC-RQ12.5.2 A pull of replicated content sends no request across regions, by counter.
- AC-RQ12.5.3 A pull's latency in a region is within the interval of the same pull in a one-region deployment.
- AC-RQ12.5.4 A response says which region served it, in a header, for anyone debugging.

Tests:
- T-RQ12.5.1 [happy] A runner in each of three regions pulls the same image → each served by its region; the cross-region counters do not move.
- T-RQ12.5.2 [edge] A runner pulls something only another region has → fetched through once; the header says so.
- T-RQ12.5.3 [failure] A runner's own region is down → it is sent to another by the operator's naming and served there.

---

### RQ12.6 A region is lost

Needs: RQ12.2, RQ12.3, RQ12.4 · Serves: AC-T9.9.3

Drill it in the lab: take a whole region away, under load. Pulls of replicated content keep working
in the others. Repositories homed in the lost region are read-only until the operator, or a rule the
operator stated, moves their home. When the region returns, it does not serve until it has
reconciled: the regions compare each repository by a summary of its state and exchange only what
differs.

Done when:
- AC-RQ12.6.1 Pulls failed while one region is lost is exactly 0, measured by the region-loss drill (AC-T9.9.3), for content the rule replicated.
- AC-RQ12.6.2 Artifacts lost in the drill is exactly 0: everything acknowledged under a rule that named a second region is present there.
- AC-RQ12.6.3 A home moves only by the operator's word or stated rule, and after the move the old home, when it returns, accepts that it is no longer the home before it serves a write.
- AC-RQ12.6.4 A returning region serves no stale tag: it is not ready until it has reconciled every repository it holds.
- AC-RQ12.6.5 Reconciling two regions that differ in a few repositories costs work in proportion to the difference, not to what they hold.

Tests:
- T-RQ12.6.1 [failure] The region-loss drill under pull and push load → no failed pull of replicated content; pushes to repositories homed elsewhere continue.
- T-RQ12.6.2 [failure] The lost region held the only copy of a lazily replicated blob → pulls of it are refused typed elsewhere, naming the region; nothing is served wrong; the drill's report lists it as the rule's consequence.
- T-RQ12.6.3 [race] The home is moved while the old home is alive but partitioned, and both take a write → the old home's write is refused or, when it rejoins, superseded by the stated rule; never two histories.
- T-RQ12.6.4 [failure] The region returns after a long absence with a different state → reconciled; states equal; it was not ready before.
- T-RQ12.6.5 [exhaustion] Reconciliation of regions holding the scale phase's repository count with a small difference → work proportional to the difference.

---

### RQ12.7 One suite, three regions

Needs: RQ12.5, RQ12.6 · Serves: CP-T9.9

Run the differential suite, unchanged, against three regions with the nemesis cutting links, killing
nodes and skewing clocks, and against one node and three nodes on the same day. One code path means
one suite.

Done when:
- AC-RQ12.7.1 The same differential suite passes on one node, three nodes and three regions, with the guarantees each repository's rule states.
- AC-RQ12.7.2 Coverage shows no branch of the core that only a multi-region deployment takes, other than the replication rule's own.
- AC-RQ12.7.3 The suite's non-vacuity counters show cross-region conflicts, forwards, fetch-throughs and reconciliations each occurred.

Tests:
- T-RQ12.7.1 [model] Generated histories on three regions with the nemesis → all match the model under each rule.
- T-RQ12.7.2 [livelock] Two regions forwarding a write to each other because each believes the other is home → cannot happen: the home is a record with one value; the test forces the belief and shows the refusal.
- T-RQ12.7.3 [deadlock] Each region waiting for another's acknowledgement under a rule that needs two → a region that cannot get it refuses typed; none waits for good.
- T-RQ12.7.4 [non-vacuity] The counters → all above zero.
