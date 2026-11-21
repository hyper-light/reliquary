# Phase 9: A cluster: many nodes, garbage collection, quotas, retention

**Goal RQ9.** Reliquary runs as many nodes over Mantle's and Amphora's clusters. It reclaims what
nothing references while pushes and pulls go on, never collects what is referenced, holds each
namespace to its quota and its retention rules, and loses nothing when a disk or a machine is lost.

**Delivers.** The same binary as the laptop's, started more than once. Any node finishes what
another began. Garbage collection with no pause and no read-only mode. Quotas that are exact.
Kiln's registry survives the loss of a disk or a machine.

**Serves.** CP-T9.7 (AC-T9.7.1 to AC-T9.7.4, T-T9.7.1). **Needs.** RQ1; Mantle's cluster (kiln's
CP-T10.3) and Amphora's replicated groups for the real drills, their simulators until then.

**Order.** RQ9.1 and RQ9.2 first, side by side. Then RQ9.3, RQ9.5, RQ9.6, RQ9.8 and RQ9.9 side by
side; RQ9.4 after RQ9.3. RQ9.7 and RQ9.10 close.

---

### RQ9.1 Many nodes, one registry

Needs: RQ1 · Serves: AC-T9.7.1

Start the binary on several machines against the same records and bytes, and it is one registry. A
node holds nothing another needs: an upload begun on one continues on any, from the session's record;
a token made by one is checked by any; a cache on one is never stale because another wrote. Nodes find
each other through hyper-swim, only to share hints (who is filling a blob, who holds what); a wrong
or missing hint costs speed, never correctness. The differential suite that passes on one node passes
on three, with the nemesis between them.

Done when:
- AC-RQ9.1.1 The differential suite passes on three nodes with clients switching node on every request, under partitions, crashes and delays.
- AC-RQ9.1.2 An upload's chunks sent to different nodes in turn complete with the right digest.
- AC-RQ9.1.3 A write acknowledged by one node is read from every node from the acknowledgement on.
- AC-RQ9.1.4 A node joins or leaves with no request refused for that reason beyond those in flight on it, which their clients resume elsewhere.
- AC-RQ9.1.5 With every hint between nodes dropped, every answer is still correct.
- AC-RQ9.1.6 No node elects itself anything: there is no leader among Reliquary's nodes, and every coordination is a conditional transaction on the records.

Tests:
- T-RQ9.1.1 [model] Generated histories against three nodes with the nemesis → all match the model.
- T-RQ9.1.2 [happy] A chunked upload with each chunk to a different node → `201`; pulls identical from every node.
- T-RQ9.1.3 [race] A tag moved on one node and read on another in the same instant, repeated under injected delays → every read is one a linearization allows.
- T-RQ9.1.4 [failure] Kill a node mid-upload and mid-download → clients resume on another node from their durable offset and verified prefix.
- T-RQ9.1.5 [failure] A node partitioned from the records but reachable by clients → it refuses typed and says where to go; it serves nothing it cannot prove current.
- T-RQ9.1.6 [failure] All hints dropped, delayed and duplicated → correct answers; duplicate upstream fills are the only cost, and they are counted.
- T-RQ9.1.7 [livelock] Three nodes each retrying a conditional write the others keep winning → each completes within its counted budget or refuses typed.
- T-RQ9.1.8 [exhaustion] One node at its admission limit while others idle → its refusals carry `Retry-After`; clients that move to another node are served.

---

### RQ9.2 Records and bytes on their clusters

Needs: RQ0.4 · Serves: AC-T9.7.1, AC-T9.7.2

Run the `Records` seam on Amphora's replicated groups and the `Bytes` seam on a Mantle cell of many
nodes, through the same client code as the laptop's. A write is acknowledged when Amphora's quorum
and Mantle's erasure-coded chunks are durable. Handle what a cluster can answer that a single
process cannot: not the leader, fenced, an outcome unknown after a failover. How many failures to
survive is the operator's statement; Mantle and Amphora turn it into placement, or say the hardware
cannot meet it.

Done when:
- AC-RQ9.2.1 Both seams' conformance suites pass against the clusters, on Linux, macOS and Windows clients.
- AC-RQ9.2.2 A leader change in Amphora or a node loss in Mantle during any operation ends in the operation's success, a typed refusal, or a reconciled outcome; never a duplicate or a lost acknowledged write.
- AC-RQ9.2.3 The laptop and the cluster differ only in what the configuration states about where records and bytes are; a diff of the code paths taken, by coverage, shows no cluster-only branch in the core.
- AC-RQ9.2.4 If the stated failures to survive cannot be met by the hardware present, startup refuses and names what is missing.

Tests:
- T-RQ9.2.1 [happy] Run both seam suites against a three-node Amphora and a three-node Mantle cell → every clause passes.
- T-RQ9.2.2 [failure] Kill Amphora's leader during a manifest commit, a tag move and a delete → each completes once or is refused typed; a retry never applies twice.
- T-RQ9.2.3 [failure] Kill a Mantle node during a blob upload and a download → both complete from the other nodes.
- T-RQ9.2.4 [failure] Partition the minority side of each cluster → the majority serves; the minority refuses typed.
- T-RQ9.2.5 [edge] An `UnknownOutcome` after a failover → reconciled by reading; the client is told the true outcome.
- T-RQ9.2.6 [exhaustion] A Mantle cell that answers `Full` on one node only → writes go elsewhere by Mantle's placement; Reliquary retries nothing in a loop.

---

### RQ9.3 Garbage collection that never stops the registry and never takes what is used

Needs: RQ1.2, RQ9.1 · Serves: AC-T9.7.3, AC-T9.7.4, T-T9.7.1

A blob's bytes are deleted only through three conditional steps on its record, so that every race
with a push ends safely. Condemn: the blob's state moves from live to condemned, only if no link
exists. Check: read the links again after the condemn; if one appeared, restore the blob to live.
Remove: move condemned to removing, conditionally; only then delete the bytes, then the record. A
push that would link a blob conditions its link on the blob's record being live at the revision it
read; meeting a condemned blob it restores it and links; meeting a removing one it uploads the bytes
again under a new key. Collection is work any node does a counted step at a time; no node is the
collector and nothing is locked. Before bytes are deleted, a condemned blob that anything tries to
read raises an alarm and is restored. The protocol is the TLA+ model of RQ1.2 with its collector,
and its safety is a property test over generated histories.

Done when:
- AC-RQ9.3.1 Referenced blobs collected is exactly 0, measured by a property test of generated histories (AC-T9.7.4), with the nemesis, on one node and three.
- AC-RQ9.3.2 Pushes or pulls failed during garbage collection under load is exactly 0, measured by a GC drill (AC-T9.7.3).
- AC-RQ9.3.3 A push racing a collection of its blobs completes and its blobs stay (T-T9.7.1).
- AC-RQ9.3.4 The registry is never read-only and never paused for collection; throughput and latency during collection stay within their intervals without it.
- AC-RQ9.3.5 Every unreferenced blob is eventually collected: after the system is quiet, bytes held equal bytes referenced plus what leases and the horizon protect, exactly.
- AC-RQ9.3.6 A collector that dies at any step leaves a state another node completes or undoes; no blob stays condemned or removing for good.
- AC-RQ9.3.7 A read of a condemned blob through any path is counted as an alarm, restores the blob, and is 0 in every test that does not plant it.

Tests:
- T-RQ9.3.1 [model] TLC on the model with pushes, mounts, deletes and two collectors → no state where a linked blob's bytes are gone; with the condemn's condition removed → a counterexample.
- T-RQ9.3.2 [race] A push that found its blob present races the blob's condemn, check and remove at every interleaving the simulator can produce → the push completes; the blob is readable after.
- T-RQ9.3.3 [race] Two collectors on one blob → one proceeds; the other's condition fails; the bytes are deleted once.
- T-RQ9.3.4 [race] A mount of a blob in the instant it is removed → `202` and a fresh upload, or `201` on a restored blob; never `201` on missing bytes.
- T-RQ9.3.5 [crash] Kill the collector between each pair of its steps → another node finishes; the property holds.
- T-RQ9.3.6 [crash] Power cut after the bytes are deleted and before the record is → after restart the record is removed, and a push of the same content uploads again and succeeds.
- T-RQ9.3.7 [failure] Mantle refuses the delete → the blob stays removing and is retried by RQ0.8's rule; it is never linked meanwhile.
- T-RQ9.3.8 [hostile] Plant a read of a condemned blob → the alarm counts one; the blob is live again; nothing is deleted.
- T-RQ9.3.9 [non-vacuity] Read the property test's counters → blobs collected, pushes that raced a condemn, and blobs restored are each above zero.
- T-RQ9.3.10 [perf] The GC drill: sustained pushes and pulls at the admission limit while a backlog is collected → no failed request; latency within its interval.
- T-RQ9.3.11 [livelock] A pusher restoring a blob and a collector condemning it, in turn, again and again → the push completes within its counted budget; the collector leaves the blob until its horizon passes again; neither spins.
- T-RQ9.3.12 [deadlock] Collectors on several nodes each holding a condemned blob while a push needs all of them → no collector waits on a push and no push waits on a collector: each step is one conditional write that succeeds or fails at once.

---

### RQ9.4 Finding garbage without looking at everything

Needs: RQ9.3 · Serves: AC-T9.7.3

Collection must not scan the registry. Everything that can make garbage is a change in the records:
a manifest deleted, a tag moved off its last manifest where the retention rule says untagged
manifests go, a link removed, an upload abandoned. Each puts a candidate in a review queue, itself
rows in the records. A candidate is reviewed after a horizon that protects pushes in flight: a client
that was told a blob exists and has not yet pushed the manifest that names it. The horizon is learned
per repository as the longest gap seen between a blob's last assertion and the manifest that used it,
and while a pushing principal still shows progress in that repository its assertions stay protected.
A full reconciliation that compares everything exists too, as a paced background check that should
find nothing.

Done when:
- AC-RQ9.4.1 The work to collect one unreferenced blob does not grow with the number of blobs, manifests or repositories stored, at the plan's three scale sizes.
- AC-RQ9.4.2 Every change that can orphan a blob enqueues a candidate in the same transaction as the change; a comparison of the queue with a full scan after generated histories finds no orphan the queue missed.
- AC-RQ9.4.3 The horizon has no fixed length: it is learned per repository, starts from liveness and progress, and is in the constants register as learned.
- AC-RQ9.4.4 With stock clients pushing large images over slow links, pushes refused `MANIFEST_BLOB_UNKNOWN` because a blob was collected under them is exactly 0 in the drill.
- AC-RQ9.4.5 The background reconciliation's rate is derived from idle capacity; in every test without planted corruption it finds nothing, and what it finds is an alarm.
- AC-RQ9.4.6 The review queue is bounded by what the records hold of it; past its bound new candidates wait in the change feed, and none is dropped.

Tests:
- T-RQ9.4.1 [happy] Delete a manifest whose blobs nothing else references → after the horizon the bytes are gone and the quota is credited.
- T-RQ9.4.2 [edge] Delete one of two manifests sharing a layer → the shared layer stays; the unshared goes.
- T-RQ9.4.3 [edge] A blob pushed and never referenced → collected after the horizon; one referenced a moment before the horizon ends → kept.
- T-RQ9.4.4 [race] A slow client: `HEAD` finds a layer, then it uploads other layers for longer than any gap seen before, then pushes the manifest → the manifest is accepted; the layer was protected by the client's progress.
- T-RQ9.4.5 [failure] The collector is down for a long time, then starts → it works through the backlog a counted step at a time; requests are unaffected.
- T-RQ9.4.6 [exhaustion] A mass delete of a huge repository → the queue grows to its bound and no further; the delete's acknowledgement does not wait for collection.
- T-RQ9.4.7 [model] After generated histories, compare what the queue collected with a full mark from the tags → the same set, less what the horizon protects.
- T-RQ9.4.8 [hostile] A client that asserts millions of blobs it never references, to pin them → its assertions protect only what its quota is charged for, and lapse with its progress.

---

### RQ9.5 Quotas that are exact

Needs: RQ9.1 · Serves: CP-T9.7

Hold each namespace to the bytes, repositories and tags its operator grants. A namespace is charged
the logical size of everything it links, whether or not another namespace holds the same bytes, so
no one learns what others store and no one stores for free. Bytes staged by open uploads count. The
charge changes in the same transaction as the link, so it is never wrong; a namespace's counter is
kept so that many pushes at once do not contend on one row. A push that would pass the quota is
refused before its bytes are sent where the size is known, and at the byte that passes it where it is
not.

Done when:
- AC-RQ9.5.1 After any generated history, each namespace's charged bytes equal the sum of the sizes of what it links plus its open uploads, exactly.
- AC-RQ9.5.2 Bytes written past a quota is exactly 0, in a test that fills a namespace from many nodes at once.
- AC-RQ9.5.3 A refusal for quota is typed, names the quota and what remains, and costs no upload.
- AC-RQ9.5.4 Under many concurrent pushes to one namespace, commit throughput is within the interval of the same pushes spread over many namespaces.
- AC-RQ9.5.5 A blob shared by two namespaces is charged to both, and one namespace's deletion of it changes only its own charge.

Tests:
- T-RQ9.5.1 [happy] Push until the quota is reached → the push that would pass it is refused with the remainder; a smaller one fits.
- T-RQ9.5.2 [race] Many nodes push to one namespace at once with a quota that fits half → exactly the bytes that fit are accepted; the counter is exact.
- T-RQ9.5.3 [edge] A push of exactly the remainder; a zero-byte blob at a full quota; a mount of a blob larger than the remainder → accepted; accepted; refused.
- T-RQ9.5.4 [edge] A quota lowered below current use → nothing is deleted; new pushes are refused; deletes are allowed.
- T-RQ9.5.5 [crash] Kill a node between staging bytes and charging them, at every write → after restart the charge is exact.
- T-RQ9.5.6 [hostile] A chunked upload with no stated length that runs past the quota → cut at the byte that passes it; staged bytes freed.
- T-RQ9.5.7 [model] Generated histories → charges equal a recomputation from the records.
- T-RQ9.5.8 [deadlock] Two pushes each holding staged bytes and each needing the quota the other holds to finish → one is refused typed and frees its bytes; neither waits for the other.

---

### RQ9.6 Retention and immutability, as the operator states them

Needs: RQ9.3 · Serves: CP-T9.7

Let an operator say what a repository keeps. Tags can be immutable: once set, a tag cannot move or be
deleted, with exceptions the rule names. A retention rule says which versions to keep (by tag
pattern, by count of the newest, by age, by whether anything pulled them lately) and whether untagged
manifests and their referrers go. Rules are the operator's statements; Reliquary ships none of its
own beyond keeping everything. A rule is applied by following the change feed, and can be run dry
first, listing what it would remove. What a rule removes goes through RQ9.3 like any delete.

Done when:
- AC-RQ9.6.1 A move or delete of an immutable tag is refused `DENIED` by both doors, naming the rule.
- AC-RQ9.6.2 After a rule settles, what remains is exactly what the rule keeps, compared with a direct evaluation of the rule over the records.
- AC-RQ9.6.3 A dry run lists exactly what the live run then removes, where nothing changed between them.
- AC-RQ9.6.4 A manifest is never removed while a tag a rule keeps, an index a rule keeps, or a policy's hold names it.
- AC-RQ9.6.5 Whether a signature, SBOM or other referrer outlives its subject is the rule's to say, and the default keeps it.
- AC-RQ9.6.6 Applying a rule to a repository costs work in proportion to what changed, not to what it holds.

Tests:
- T-RQ9.6.1 [refusal] Move, delete and re-push with other content an immutable tag → each `DENIED`; re-push with the same content succeeds.
- T-RQ9.6.2 [happy] "Keep the newest five `v*` tags and anything pulled since its upload" → the others are removed; their unshared blobs are collected.
- T-RQ9.6.3 [edge] A rule matching nothing; everything; a manifest kept by one rule and removed by another → nothing removed; everything but holds; kept.
- T-RQ9.6.4 [edge] A multi-platform index kept while a rule would remove one member → the member stays.
- T-RQ9.6.5 [race] A tag is pushed as a rule removes its manifest's previous version → the new version is untouched; the old goes.
- T-RQ9.6.6 [race] A rule changed while it is being applied → each manifest is judged by one version of the rule; the next pass uses the new one.
- T-RQ9.6.7 [hostile] A tag pattern built to be expensive → bounded by a counted budget, refused if it cannot be evaluated within it.

---

### RQ9.7 A disk is wiped, a machine is lost, and nothing is missing

Needs: RQ9.2, RQ9.3 · Serves: AC-T9.7.2, AC-T9.7.1

Drill the failures on the lab's cluster, with load running. Wipe a storage node's disk. Power off a
machine that runs a Reliquary node, a Mantle node and an Amphora replica together. Stall a disk.
Partition the cluster. After each, every artifact acknowledged before the failure verifies by digest,
and service continued for what the stated failures to survive allow.

Done when:
- AC-RQ9.7.1 Blobs lost when a storage node's disk is wiped is exactly 0, measured by the Mantle lab (AC-T9.7.2).
- AC-RQ9.7.2 Acknowledged artifacts lost across every drill is exactly 0, by a full verification after each.
- AC-RQ9.7.3 During a drill within the stated failures to survive, pushes and pulls that fail are only those in flight on the lost machine, and each resumes elsewhere.
- AC-RQ9.7.4 After each drill the cluster returns to full redundancy without an operator, and the time it took is recorded with what bounded it.
- AC-RQ9.7.5 Each drill's record names its hardware, commit, load and counts, and is reproducible by one command.

Tests:
- T-RQ9.7.1 [failure] Wipe one storage node's disk under load → no blob lost; repair completes; every blob verifies.
- T-RQ9.7.2 [failure] Power off one machine of three under load → service continues; in-flight requests resume; nothing acknowledged is lost.
- T-RQ9.7.3 [failure] A disk that stalls for longer than any wait learned so far → requests move off it by RQ0.8's rule; none hangs.
- T-RQ9.7.4 [failure] A partition that isolates one machine, then heals → the isolated node refused typed while cut off; no divergent state after.
- T-RQ9.7.5 [failure] One more failure than stated → the registry refuses what it can no longer make durable, typed, and serves what it can verify; nothing is acknowledged that is not safe.
- T-RQ9.7.6 [crash] Garbage collection running through each drill → the property of RQ9.3 holds after.

---

### RQ9.8 Upgrades with two versions running

Needs: RQ9.1 · Serves: CP-T9.7, CP-T9.10

A cluster is upgraded a node at a time, so two versions serve at once. Records change append-only: a
new field or family is added, an old one keeps its meaning. A new behavior is behind a capability
that turns on only when every node has it. The native wire is append-only already. A rollback one
release back is drilled before each release.

Done when:
- AC-RQ9.8.1 The differential suite passes on a cluster of mixed versions, N and N-1, with clients on either.
- AC-RQ9.8.2 A record written by N is read correctly by N-1, and the reverse, for every family, checked by golden records of every released version.
- AC-RQ9.8.3 A capability is used only after every node reports it; a node that joins without it holds the capability off.
- AC-RQ9.8.4 Rolling back one release restores service with no record lost, in a drill recorded before the release.

Tests:
- T-RQ9.8.1 [model] Generated histories against a mixed cluster → all match the model.
- T-RQ9.8.2 [happy] Upgrade a three-node cluster a node at a time under load → no request fails beyond those in flight on the restarting node.
- T-RQ9.8.3 [edge] A new node writes a record with a field the old does not know → the old node preserves the field when it rewrites the record.
- T-RQ9.8.4 [failure] An upgrade stopped halfway for a long time → the mixed cluster keeps serving.
- T-RQ9.8.5 [failure] The rollback drill → service restored; every artifact verifies.
- T-RQ9.8.6 [refusal] A node two releases behind tries to join → refused at its hello, naming both versions.

---

### RQ9.9 Checking everything, end to end, in the background

Needs: RQ9.2 · Serves: AC-T9.10.3

Mantle checks every chunk it reads and scrubs its disks. Add the check only Reliquary can make: that
the bytes under a blob's record hash to its digest, that every manifest's references exist, that
every row agrees with the others. It runs at a pace taken from idle capacity and yields to requests.
Damage is repaired from Mantle's redundancy where it can be, and otherwise the artifact is marked
damaged, refused on pull, and reported. Keep the last resort too: rebuild the blob records from
Mantle's objects and their stored digests if the records were lost.

Done when:
- AC-RQ9.9.1 A planted wrong byte, missing object, dangling edge or orphan row is each found within one pass and reported with what it affects.
- AC-RQ9.9.2 A pass over the store completes in a time the pace states, and request latency during it stays within its interval.
- AC-RQ9.9.3 A damaged blob is never served: its pull is refused typed, naming the damage, until it is repaired or re-pushed.
- AC-RQ9.9.4 With the records of blobs deleted in a drill, the rebuild restores every blob record from Mantle, and every manifest then verifies.
- AC-RQ9.9.5 In every run with nothing planted, the check finds nothing.

Tests:
- T-RQ9.9.1 [failure] Flip a byte beneath Mantle's own checks (in a test build) → found; repaired from redundancy; verified.
- T-RQ9.9.2 [failure] Remove an object a blob record names → found; the blob marked damaged; pulls refused typed; a re-push repairs it.
- T-RQ9.9.3 [failure] The rebuild drill → records restored; the differential check of tags and manifests passes.
- T-RQ9.9.4 [exhaustion] The check running at full load → it yields; requests keep their latency; the pass takes longer and says so.
- T-RQ9.9.5 [race] A blob collected while the check reads it → no false damage report.

---

### RQ9.10 Kiln's registry survives the loss of a disk or a machine

Needs: RQ9.7 · Serves: AC-T9.7.1

Put it to use: kiln runs Reliquary on Mantle's cluster across its server and runners. The change is
kiln's; the drills of RQ9.7 are repeated on kiln's lab before the person's go-ahead for production.

Done when:
- AC-RQ9.10.1 Kiln's registry survives the loss of a disk or a machine, in a change request that passes the gate (AC-T9.7.1).
- AC-RQ9.10.2 The migration from the single-machine store to the cluster is behind a capability with read fallback for one release, and its rollback is drilled.
- AC-RQ9.10.3 After the move every artifact kiln held verifies by digest on the cluster.

Tests:
- T-RQ9.10.1 [failure] On kiln's lab: power off one machine during CI legs that pull and push → the legs complete.
- T-RQ9.10.2 [happy] The migration, then a full verification → every digest matches.
- T-RQ9.10.3 [failure] The rollback drill → kiln serves from the old store with nothing lost.
