# Phase 4: Speed on one node

**Goal RQ4.** On the same hardware, class for class, Reliquary's latency, throughput and cost per
operation are each ten times better than the best of CNCF Distribution, Zot, Harbor and the Rust
registries, or sit at the hardware's measured limit where a competitor already does; and it says so
with records anyone can run again.

**Delivers.** The slice made fast, one measured decision at a time: each item starts from a cost
table, tries its candidates as A|B runs, keeps the winner and records why. Nothing is tuned by hand.

**Serves.** AC-T9.1.4, AC-T9.10.2, and the person's bar: an order of magnitude on every key metric.
**Needs.** RQ1; RQ4.1 needs only RQ0.7 and starts as the phase does. Runs beside RQ2 and RQ3.

**Order.** RQ4.1 and RQ4.2 first, side by side. RQ4.3 to RQ4.9 side by side, each on its own stage of
the path. RQ4.10 closes.

**How "ten times" is judged.** For each metric and each point of the comparison's matrix, Reliquary's
interval lies wholly beyond a tenth (for latency and cost) or ten times (for throughput) of the best
competitor's median at that point. Where the best competitor's interval overlaps the hardware limit
RQ4.1 measured that day (the link, the device, one core's SHA-256), the bar at that point is to
overlap the limit too, and the record says which bar applied.

---

### RQ4.1 Measure the competitors on our hardware

Needs: RQ0.7 · Serves: AC-T9.10.2, CP-T9.10

Stand up CNCF Distribution, Zot and Harbor, and the maintained Rust registries the research found
(angos, nora and peryx), each pinned by digest and configured as its own documentation recommends
for production, on the same machine Reliquary will be measured on, one at a time. No registry
publishes figures that can be compared as they stand, so every baseline is measured here. Write the workloads once, for every registry alike: a matrix of object sizes by clients in
flight for manifest and blob pulls and pushes; tag, referrers and catalog listings at growing
repository sizes; real clients pulling real images; and a replay of a published production registry
trace. Record for each: latency quantiles, throughput, CPU seconds and peak memory per operation,
idle memory, start time and binary size. These records are the baseline every later comparison cites.

Done when:
- AC-RQ4.1.1 Each competitor's version, digest, configuration and storage backend are in the record, and its own conformance run is recorded beside it.
- AC-RQ4.1.2 Every workload runs unchanged against any registry that speaks the distribution spec; a registry is named by URL alone.
- AC-RQ4.1.3 Each point is measured by RQ0.7's rules and is of record.
- AC-RQ4.1.4 Each registry is measured at its best: its recommended cache and storage settings, stated, and a second run with its defaults.
- AC-RQ4.1.5 The hardware's own limits are measured the same day: link bandwidth, the device's durable write bandwidth, and SHA-256 throughput on one core, so a result at a limit can be told from a result below it.
- AC-RQ4.1.6 The trace replay's source, its request mix and its size distribution are cited in `docs/research/`.

Tests:
- T-RQ4.1.1 [happy] Run the pull matrix against Distribution twice on a quiet machine → each point's two intervals overlap.
- T-RQ4.1.2 [edge] Run the workload against a registry that refuses deletes → the delete points are reported as unsupported, not as zero.
- T-RQ4.1.3 [refusal] Point the workload at a registry that fails conformance → the record says so before any timing.
- T-RQ4.1.4 [happy] Verify every byte pulled in a timed run against its digest → no mismatch; a registry that serves a wrong byte fails the run.
- T-RQ4.1.5 [edge] A point where the client, not the registry, is the limit → the harness detects its own saturation (client CPU at its ceiling) and marks the point.

---

### RQ4.2 Find where a request's time and bytes go

Needs: RQ1, RQ0.7 · Serves: AC-T9.1.4

Before changing anything, measure the slice. For each kind of request through each door, record what
each stage costs: accepting, the handshake, parsing, the token, the records read or write, the bytes
path, and writing the answer. Count instructions, allocations, system calls, bytes copied per byte
served, and time on and off the CPU. Publish the table. Its largest rows set the order of the items
that follow, and each of them must move its own row.

Done when:
- AC-RQ4.2.1 `bench/records/` holds a cost table per request kind and door: every stage's instructions, allocations, system calls, copies and time, with intervals.
- AC-RQ4.2.2 The stages' times sum to the request's measured time within the table's interval; an unexplained remainder is its own row.
- AC-RQ4.2.3 The table is produced by one command and is of record under RQ0.7.
- AC-RQ4.2.4 Each later item of this phase names the rows it will move and by what test it will know.

Tests:
- T-RQ4.2.1 [happy] Produce the table twice on a quiet machine → each row's intervals overlap.
- T-RQ4.2.2 [edge] A request kind with a planted sleep in one stage → the table shows it in that stage's off-CPU time and nowhere else.
- T-RQ4.2.3 [edge] Tracing switched on and off → the cost of tracing itself is a stated row, and the untraced latency is the one reported.
- T-RQ4.2.4 [non-vacuity] List the request kinds of both doors against the tables → each has one; a new endpoint without one fails the check.

---

### RQ4.3 Checking a token costs almost nothing

Needs: RQ4.2 · Serves: AC-T9.1.4

A registry checks a credential on every request. Keep what a verified token granted, so a token seen
again is a lookup, not a signature check. The store of verified tokens is bounded by bytes drawn from
the node's memory budget and forgets by a stated rule. A revoked grant or a rotated key takes effect
exactly as before the cache existed. Bound how fast one address or principal may fail to
authenticate, from what a check costs, so guessing passwords cannot starve real requests.

Done when:
- AC-RQ4.3.1 A request with a token seen before runs no signature verification, by counter, and its token stage's cost is a row of the cost table.
- AC-RQ4.3.2 Every answer with the cache equals the answer without it, over generated histories that include expiry, revocation and key rotation.
- AC-RQ4.3.3 The cache's memory is within its budget, and an eviction never changes an answer, only its cost.
- AC-RQ4.3.4 Failed authentications by one source are admitted at a rate derived from the check's measured cost and the cores granted; past it they are refused `429` before the check runs.
- AC-RQ4.3.5 The choice of eviction rule is an A|B record over the trace's token reuse.

Tests:
- T-RQ4.3.1 [happy] Many requests with one token → one signature verification.
- T-RQ4.3.2 [edge] A token that expires between two requests → the second is `401`, cached or not.
- T-RQ4.3.3 [race] A signing key rotated while tokens under the old key are cached → they are honored exactly as long as the uncached rule honors them.
- T-RQ4.3.4 [hostile] A flood of distinct forged tokens → each costs at most one refusal; the cache does not fill with them; valid tokens' latency stays within its interval.
- T-RQ4.3.5 [hostile] Password guessing from one address and from many → the per-source bound holds for the one; the total stays within the share of CPU the stage is admitted.
- T-RQ4.3.6 [exhaustion] More live tokens than the cache holds → correct answers, verification cost for the evicted.
- T-RQ4.3.7 [model] The cache against the no-cache oracle on generated histories → identical answers.

---

### RQ4.4 Manifests, tags and referrers from memory, never stale

Needs: RQ4.2 · Serves: AC-T9.1.4

Most requests a registry gets ask for a manifest or whether one exists. Serve them from the node's
memory, and still give every reader an answer a linearizable read of the records would give. Try the
candidates as A|B runs: no cache; a cache invalidated by the records' change feed with a guard that
proves freshness; a cache validated by a revision read. Try the eviction rules the literature offers
on the replayed trace. Size the cache from the memory the node is allowed. Keep the candidate whose
latency and cost win with correctness intact, and record the others' numbers.

Done when:
- AC-RQ4.4.1 Every cached answer equals a linearizable read's, over generated concurrent histories with the nemesis, on one node and on three.
- AC-RQ4.4.2 A manifest or tag `GET`/`HEAD` served from memory performs no records read beyond what the freshness proof needs, by counter.
- AC-RQ4.4.3 The A|B records name the winner for the freshness design and for the eviction rule, with every candidate's intervals.
- AC-RQ4.4.4 The cache's size is derived from the node's memory budget; with the budget halved the node still serves, and the hit rate's change is recorded.
- AC-RQ4.4.5 A manifest too large for the cache's admission is served from Mantle, correct, at its own cost.
- AC-RQ4.4.6 A share inside the chosen eviction rule (a queue's part of the cache) stays fixed only where the record shows the result insensitive across the range tested on our traces; otherwise it is learned per node, by whichever of a downsized simulation on sampled requests and an online climber wins the A|B.

Tests:
- T-RQ4.4.1 [happy] `GET` one manifest by tag repeatedly → after the first, the path's records-read counter moves only by the freshness proof's cost.
- T-RQ4.4.2 [race] A tag moved on node A and read on node B at once, with delays and partitions injected → B never returns a value that was not current at some instant inside the read.
- T-RQ4.4.3 [race] A manifest deleted while cached on another node → the next read after the delete's acknowledgement is `404`.
- T-RQ4.4.4 [failure] The change feed stalls → reads fall back to validated reads or refuse typed; none serves from a cache it cannot prove fresh.
- T-RQ4.4.5 [failure] The feed answers that its history was compacted past the node's position → the cache is emptied and rebuilt; no stale answer in between.
- T-RQ4.4.6 [exhaustion] More distinct manifests requested than fit → eviction by the chosen rule; answers stay correct.
- T-RQ4.4.7 [hostile] A scan of distinct, non-existent tags → the cache of absences, if any, is bounded and validated; the scan cannot evict the working set past the rule's guarantee.
- T-RQ4.4.8 [model] The cached read path against the uncached oracle over generated histories → identical answers.
- T-RQ4.4.9 [deadlock] Readers waiting on a freshness proof while the feed's apply waits to update the entries they read → no reader holds an entry across a wait; the apply completes and every reader is answered or refused typed.
- T-RQ4.4.10 [livelock] A tag moved faster than a reader can prove its copy fresh → the reader falls back to a validated read within its counted budget; it does not retry the proof without end.
- T-RQ4.4.11 [edge] Replay a trace whose objects are mostly asked for exactly twice, the case queue-based rules lose on → the comparison record includes it; the chosen rule is the winner over all traces by the stated objective, and its loss on this one is stated, not hidden.

---

### RQ4.5 Blob bytes by the shortest path

Needs: RQ4.2 · Serves: AC-T9.1.4, AC-T9.10.2

Get blob bytes to the client with the fewest copies and round trips. Two paths exist: Reliquary
answers with a redirect to a presigned URL on Mantle's S3 door, and the client reads straight from
storage; or Reliquary relays the bytes itself. A redirect costs a new connection; a relay costs
Reliquary's CPU. Choose per request from measured quantities: the blob's size, the path's learned
rate, and what a new connection costs that client. Small blobs are relayed or answered inline; large
ones go straight to storage. On the relay path, bytes go from Mantle's buffers to the socket without
an extra copy, and TLS records are sealed from the pieces as they are. Ranges and `HEAD` take the
same care.

Done when:
- AC-RQ4.5.1 A 1 MiB blob `GET` on a LAN has p99 at most 10 ms, measured by bench `reliquary_pull` (AC-T9.1.4).
- AC-RQ4.5.2 The rule that picks redirect or relay has no threshold in the code: its inputs are measured, its form is in the design record, and an A|B record shows it at least matches the better of always-redirect and always-relay at every blob size in the matrix.
- AC-RQ4.5.3 On the relay path, bytes copied per byte served, and CPU seconds per GiB served, are each at most a tenth of the best competitor's, or the link is the limit.
- AC-RQ4.5.4 A redirect's URL lets its holder read exactly that blob, for a life derived as RQ3.5's grants are, and carries no credential of the client's across hosts.
- AC-RQ4.5.5 Every stock client in RQ1.10's matrix follows the redirect and verifies; one that cannot is served by relay, decided from what it says of itself or how it behaves, and the list is in the record.
- AC-RQ4.5.6 Blob `HEAD` and range `GET` meet the same bar as whole `GET` at their own points of the matrix.
- AC-RQ4.5.7 A nightly run measures bench `reliquary_pull`'s p99 on the bench host in its slot and feeds it to the change-point detector (`reliquary bench changes`, RQ0.7), whose alert names the commit range.

Tests:
- T-RQ4.5.1 [happy] `GET` blobs of every size in the matrix by each path → identical bytes; digests verify.
- T-RQ4.5.2 [edge] A blob at the size where the two paths cost the same → either path, and latency within the better path's interval.
- T-RQ4.5.3 [edge] A client on a path far slower than the last one seen → the rule follows that client's path, not the previous client's.
- T-RQ4.5.4 [refusal] A presigned URL used for another blob, after its life, or with another method → refused by Mantle.
- T-RQ4.5.5 [failure] Mantle's S3 door is unreachable from the client → the client's retry is served by relay; the rule learns that path.
- T-RQ4.5.6 [race] A blob deleted between the redirect and the client's read → Mantle answers `404`; no other blob's bytes.
- T-RQ4.5.7 [hostile] A client that follows redirects and sends its registry token to the storage host → the storage host ignores it, and it is not logged.
- T-RQ4.5.8 [exhaustion] Relay readers past the relay's admitted bytes in flight → refused with `Retry-After`; redirected readers are unaffected.
- T-RQ4.5.9 [perf] Bench `reliquary_pull` across the matrix, of record → the bar's verdict at every point.
- T-RQ4.5.10 [perf] A slowdown planted on main once three nights are fed (M8's fewest on each side) → by the sixth fed night the detector's alert names its commit range; the slowdown is reverted after.

---

### RQ4.6 HTTP/2 on the OCI door

Needs: RQ4.2 · Serves: AC-T9.10.2

Docker, containerd and most clients ask for HTTP/2 when a server offers it, and it lets one
connection carry many layer requests. Add it with Shards' sans-IO HTTP/2, vendored, negotiated by
ALPN beside HTTP/1.1. Flow-control windows come from the connection's memory charge and the path's
measured bandwidth and delay. Every way HTTP/2 has been used to exhaust a server is bounded and
refused: streams reset as fast as they open, endless header continuation, floods of settings, pings
and empty frames, and windows a client never opens. The bound is on work in flight, not on the
protocol's count of open streams: a stream that either side reset still counts until its handler has
stopped, which is what the reset attacks of 2023 and 2025 got around.

Done when:
- AC-RQ4.6.1 The conformance suite and RQ1.10's client matrix pass over HTTP/2, and over HTTP/1.1 as before.
- AC-RQ4.6.2 An h2spec run reports no failure, or each one is listed with the RFC text that decides it.
- AC-RQ4.6.3 A connection's memory is bounded by its charge whatever a peer sends; the bound is counted and pinned.
- AC-RQ4.6.4 Each named attack costs the server no more than a stated, counted amount of work per attacker byte, and ends in a typed connection close.
- AC-RQ4.6.5 An A|B record shows HTTP/2 against HTTP/1.1 for pulls of many small layers and of few large ones, and the door's default follows it.
- AC-RQ4.6.6 Window sizes have no fixed value: each is derived, and a slow path and a fast one get different windows in the same run.
- AC-RQ4.6.7 Handlers running for a connection never exceed its advertised stream limit, by counter, whoever reset the streams and however fast; a stream past the limit is refused.
- AC-RQ4.6.8 A header block is bounded in encoded bytes, in frames and by progress toward its end, and its decoded size is counted while it is decoded; each refusal is counted by kind in a metric and told as an event, since no request line exists for it.

Tests:
- T-RQ4.6.1 [happy] Pull an image of many layers over one HTTP/2 connection → identical; streams ran concurrently.
- T-RQ4.6.2 [hostile] Open and reset streams as fast as the peer can; send HEADERS with CONTINUATION frames that never end; send SETTINGS and PING floods; send zero-length DATA frames → each closed typed; CPU per attacker byte within the stated bound; other connections' latency within their interval.
- T-RQ4.6.3 [hostile] A client that opens every stream and never grants window → held to its charge and cut by progress.
- T-RQ4.6.4 [hostile] HPACK: a header block that expands past the bound; a dynamic table resize flood → refused at the bound.
- T-RQ4.6.5 [edge] A request body larger than its window sent in one flight; a response to a client that closes its stream midway → flow control holds; the stream ends cleanly.
- T-RQ4.6.6 [deadlock] Both ends waiting on window (the connection's is full of data for a stream the client stopped reading) → the stalled stream is cut by progress; the others continue.
- T-RQ4.6.7 [exhaustion] Streams up to the advertised limit, then one more → refused with `REFUSED_STREAM`; no state kept for it.
- T-RQ4.6.8 [happy] A client that speaks only HTTP/1.1 → served as before.
- T-RQ4.6.9 [hostile] Make the server reset streams (frames that break flow control, malformed frames) while their requests are being served, as fast as the peer can → handlers running never exceed the stream limit; cost per attacker byte within the stated bound; the connection is closed typed.
- T-RQ4.6.10 [hostile] A header block begun and never ended, sent a byte at a time just often enough to look alive → cut by progress; counted by kind; the metric shows the refusal though no request was ever logged.

---

### RQ4.7 Pushes as fast as the hardware hashes and writes

Needs: RQ4.2 · Serves: AC-T9.10.2

A push must hash every byte and make it durable; nothing can be faster than that, so reach it.
Hash with the CPU's SHA extensions on each architecture, once per byte, as the bytes pass to Mantle.
Let a client's layers upload side by side without one stalling another. Commit records in groups so
many pushes share one durable write. Measure the ceiling (one core's SHA-256, the device's durable
bandwidth) the same day and report how close a push comes.

Done when:
- AC-RQ4.7.1 A large blob's push throughput per core is what the cost table predicts from that core's measured SHA-256 throughput and the device's durable bandwidth, within its interval; the gap to each ceiling is itemized by stage.
- AC-RQ4.7.2 Each uploaded byte is hashed with SHA-256 once on the server, shown by the instruction count per byte.
- AC-RQ4.7.3 Push throughput, and CPU seconds per GiB pushed, meet the phase's bar against the best competitor at every point of the matrix.
- AC-RQ4.7.4 Under many concurrent small pushes, durable commits per second exceed one per flush: commits group, with no timer, and each push is answered only after its group is durable.
- AC-RQ4.7.5 One slow upload on a connection does not slow another on it, within the interval.
- AC-RQ4.7.6 The hash and its chunking are right on x86_64 with and without SHA extensions, and on arm64, against the published test vectors.

Tests:
- T-RQ4.7.1 [happy] Push blobs across the size matrix → the bar's verdict at every point; every digest verifies.
- T-RQ4.7.2 [edge] A CPU without SHA extensions (masked in a VM) → correct digests, the software path's throughput reported as its own class.
- T-RQ4.7.3 [race] Many pushes committing at once → all durable before answered; flushes are fewer than commits; none answered early.
- T-RQ4.7.4 [failure] A power cut between a group's write and its flush (simulated) → none of the group was acknowledged; none is visible after restart.
- T-RQ4.7.5 [exhaustion] More concurrent uploads than the staging budget → admission refuses the excess typed; admitted uploads keep their throughput.
- T-RQ4.7.6 [deadlock] A group commit waiting on a flush that waits on room that a commit in the group would free → cannot form: the order is fixed and the test drives the store to full to show it.
- T-RQ4.7.7 [perf] Run bench `reliquary_push` beside the day's SHA-256 and durable-write ceilings → a record of record, with the gap to each ceiling itemized.

---

### RQ4.8 Overload: refuse early, never collapse

Needs: RQ4.2, RQ0.8 · Serves: AC-T9.10.2, AC-T9.10.3

Past its capacity a server either refuses work or does all of it badly. Give each bottleneck one
admission authority: the request shards, the records, the bytes path, the checks. Each learns its
limit as the point of greatest power (RQ0.8) and holds what waits in a queue bounded by what its
callers can still use. What is refused is refused before it costs anything, as `429` or `503` with a
`Retry-After` computed from the queue's drain. Pulls are not starved by pushes or listings, and one
principal cannot take what others are waiting for.

Done when:
- AC-RQ4.8.1 As offered load rises past capacity, goodput stays within its interval of the peak, instead of falling, up to the highest load the harness can offer.
- AC-RQ4.8.2 A refused request costs at most a stated, counted number of instructions and no records or bytes work.
- AC-RQ4.8.3 `Retry-After` is derived from measured drain; clients that honor it are admitted on their return unless load rose meanwhile, and the record compares their admission with a jittered-backoff control's.
- AC-RQ4.8.4 No admission limit, queue length or share in the code is a picked number; each has a row of an accepted kind.
- AC-RQ4.8.5 With one principal offering many times its fair share, every other principal's latency stays within its interval.
- AC-RQ4.8.6 Health, metrics and the administrator's own calls are admitted from a reserve when everything else is refused.

Tests:
- T-RQ4.8.1 [happy] Offered load from well under capacity to well over it, open loop → goodput rises, flattens and stays flat; latency of admitted requests stays bounded.
- T-RQ4.8.2 [edge] Load that steps up and down faster than the limit's probe cycle → the limit does not oscillate to its floor; goodput stays within the interval.
- T-RQ4.8.3 [edge] A workload of only cheap requests, then only expensive ones → each finds its own limit; the cheap ones are not held to the expensive ones' limit.
- T-RQ4.8.4 [failure] The records slow by a factor (injected) → the records authority lowers its limit; the request shards keep serving cached reads.
- T-RQ4.8.5 [hostile] One principal opens connections and requests without bound → held to its share; others unaffected.
- T-RQ4.8.6 [livelock] Every client retrying at once after a stall (a thundering herd) → `Retry-After` spreads them; the herd drains in a bounded number of rounds.
- T-RQ4.8.7 [deadlock] Each authority full and waiting on the next → no cycle: admission is taken in one fixed order and released on every path, shown by driving all four to their limits together.
- T-RQ4.8.8 [exhaustion] Sustained overload for as long as the harness runs → memory stays within budget; no queue grows.

---

### RQ4.9 A small footprint, counted

Needs: RQ4.2 · Serves: AC-T9.10.2

What a registry costs when it does nothing, and per connection and request when it does something,
decides how far a laptop and a fleet's budget go. Count them with the counting allocator: idle
memory, memory per idle and active connection, per request by kind, per open upload; the binary's
size; the time from start to ready on an empty and on a full store. Pin each with a test so a
regression fails the gate. Compare each with the competitors' own.

Done when:
- AC-RQ4.9.1 Each footprint figure is in the constants register as counted, pinned by a test, on each of the five targets.
- AC-RQ4.9.2 Idle memory, memory per connection and time to ready are each at most a tenth of the best competitor's, measured the same way.
- AC-RQ4.9.3 Time to ready does not grow with the number of repositories, tags or blobs stored, shown at three store sizes.
- AC-RQ4.9.4 No per-request allocation remains on the manifest and blob `GET` paths beyond what the cost table lists and justifies.

Tests:
- T-RQ4.9.1 [happy] Start, idle, measure → the pinned figure.
- T-RQ4.9.2 [edge] Open the admitted number of idle connections → memory grows by the pinned amount each, and returns when they close.
- T-RQ4.9.3 [edge] Start against stores of the plan's three scale sizes → times to ready overlap.
- T-RQ4.9.4 [exhaustion] Hold open uploads to the staging limit → memory within the counted bound.
- T-RQ4.9.5 [failure] Start after an unclean stop with a long recovery → ready only after recovery; its time is reported apart from a clean start's.
- T-RQ4.9.6 [perf] The same measurements on each competitor → recorded beside Reliquary's.

---

### RQ4.10 The comparison, published

Needs: RQ4.1, RQ4.3, RQ4.4, RQ4.5, RQ4.6, RQ4.7, RQ4.8, RQ4.9 · Serves: AC-T9.10.2, AC-T9.1.4

Run RQ4.1's whole matrix against Reliquary and each competitor on the same machine in one session,
interleaved. Publish the result as a document anyone can reproduce: every point, every interval,
every configuration, the hardware's limits that day, and every point where Reliquary did not reach
the bar, with why. A fresh subagent given only the document and the repository repeats a sample of
it.

Done when:
- AC-RQ4.10.1 `bench/compare/` holds the records and one command reproduces them; the document cites each record.
- AC-RQ4.10.2 At every point of the matrix the verdict is one of: ten times better; at the hardware limit; or short of the bar, named in an open list with its cause and the item that would close it.
- AC-RQ4.10.3 Pull p99 against CNCF Distribution's on the same hardware is at most 100% (AC-T9.10.2), and the document says by how much.
- AC-RQ4.10.4 A fresh subagent reproduces a sample of points within their intervals from the document alone.
- AC-RQ4.10.5 Every byte served in the comparison was verified against its digest.

Tests:
- T-RQ4.10.1 [perf] Run the full matrix against every registry, interleaved, in one session → records of record for every point.
- T-RQ4.10.2 [happy] Re-run a sample of points a day later → intervals overlap the published ones.
- T-RQ4.10.3 [edge] A point where a competitor wins → it is in the open list, not left out.
- T-RQ4.10.4 [non-vacuity] Check the matrix against the phase's bar → every metric it names has points for every registry.
