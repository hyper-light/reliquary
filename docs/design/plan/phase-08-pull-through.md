# Phase 8: Pull-through caches and mirrors

**Goal RQ8.** Every runner pulls its images through Reliquary at LAN speed. CI never meets an
upstream's rate limit, and a pull of anything Reliquary holds succeeds while its upstream is down.

**Delivers.** Docker Hub, GHCR, Quay, registry.k8s.io, MCR and any other OCI registry reached
through Reliquary: fetched once, verified, kept, and served from Mantle. Tags as fresh as the
operator asks, and never a mix of an old and a new image. Chosen repositories mirrored ahead of need.

**Serves.** CP-T9.5 (AC-T9.5.1 to AC-T9.5.3, T-T9.5.1). **Needs.** RQ1. RQ8.6 takes `veil-api` when
it exists and the `Secrets` simulator's file-backed stand-in until then.

**Order.** RQ8.1 and RQ8.2 first, side by side. Then RQ8.3, RQ8.4, RQ8.5, RQ8.6, RQ8.7 and RQ8.8
side by side. RQ8.9 closes.

---

### RQ8.1 A client for other registries that never trips their limits

Needs: RQ1 · Serves: AC-T9.5.2

Implement the `Upstream` seam on Shards' registry client, vendored: it already speaks the token
flow, follows redirects without leaking credentials across hosts, learns each registry path's waits,
and learns how many transfers pay. Add what a cache needs: read each upstream's stated limits from
its response headers and its `429` answers, and keep a budget of requests per upstream that Reliquary
never overspends. Foreground misses draw on the budget first, background refreshes on what is left.
A request that would overspend waits its turn or is refused typed with when to come back.

Done when:
- AC-RQ8.1.1 The client pulls from Docker Hub, GHCR, Quay, registry.k8s.io and MCR, anonymously and with credentials, and passes Shards' own registry test suite as vendored.
- AC-RQ8.1.2 Against a simulated upstream with a stated limit, requests sent never exceed the limit in any window, at any offered load.
- AC-RQ8.1.3 The budget's figures come from the upstream's own headers and answers; no upstream's limit is written in the code, and an upstream that states none is paced by its `429` answers alone.
- AC-RQ8.1.4 A foreground miss is never delayed by background work while budget remains.
- AC-RQ8.1.5 Credentials for one host are never sent to another, redirect or not.

Tests:
- T-RQ8.1.1 [happy] Pull a manifest and its layers from each of the five upstreams' recorded behaviors → identical bytes; digests verify.
- T-RQ8.1.2 [edge] An upstream that lowers its limit mid-run; one that states a limit in one header form and another in a second form; one that states none and answers `429` → the budget follows each.
- T-RQ8.1.3 [failure] `429` with `Retry-After`; `503`; a reset mid-body; a TLS failure → waited out as told; retried by RQ0.8's rule; resumed by range; refused typed.
- T-RQ8.1.4 [hostile] An upstream that redirects a blob to a third host and expects the token there → the token is not sent.
- T-RQ8.1.5 [hostile] An upstream that sends a body unlike its digest, an endless body, or a manifest of another media type than stated → refused; nothing cached.
- T-RQ8.1.6 [exhaustion] More misses at once than the budget admits → the excess waits in a bounded queue or is refused with `Retry-After`; none is sent upstream early.
- T-RQ8.1.7 [livelock] Background refresh and foreground misses competing at the budget's edge → foreground always proceeds; background resumes when budget returns.

---

### RQ8.2 Names, and how clients are pointed here

Needs: RQ1 · Serves: AC-T9.5.1

Decide how an upstream's repository is named in Reliquary and how each client asks for it. A cached
repository lives under a namespace for its upstream, so `docker.io/library/ubuntu` has one place.
Serve the `ns` query parameter containerd sends to mirrors, answer `OCI-Namespace` as the spec's
next version says, and apply Docker Hub's rule that a bare name is under `library/`. Produce the
mirror configuration for Docker, containerd, Podman and Shards from one command. A cached repository
refuses pushes.

Done when:
- AC-RQ8.2.1 The same upstream image is one repository in Reliquary however a client names it (by mirror configuration, by `ns`, or by its cache path).
- AC-RQ8.2.2 Docker, containerd, Podman and Shards each pull `ubuntu`, `ghcr.io/x/y` and `registry.k8s.io/pause` through Reliquary with the configuration `reliquary mirror-config` prints for it.
- AC-RQ8.2.3 A push, a tag move or a delete by a client on a cached repository is refused `DENIED`, saying it is a cache.
- AC-RQ8.2.4 An upstream not on the operator's list is refused `DENIED`; Reliquary is never an open proxy.

Tests:
- T-RQ8.2.1 [happy] Pull one image by each naming → one repository, one set of blobs.
- T-RQ8.2.2 [edge] `ubuntu`, `library/ubuntu` and `docker.io/library/ubuntu`; an upstream on a port; an upstream name that contains the cache's own prefix → resolved to the right upstream each time.
- T-RQ8.2.3 [refusal] `ns=evil.example` where that host is not listed; `ns` with a path, credentials or an IP in a private range → `DENIED` or `400`.
- T-RQ8.2.4 [refusal] `docker push` to a cached repository → `DENIED`.
- T-RQ8.2.5 [hostile] A request whose `ns` differs from the namespace in its path → refused; neither is trusted over the other.

---

### RQ8.3 Tags as fresh as asked, and never a mix

Needs: RQ8.1, RQ8.2 · Serves: T-T9.5.1

A manifest fetched by digest is kept for good. A tag is different: its upstream can move it. The
operator states how stale a cached tag may be; unstated, a tag is checked against its upstream on
each pull by a request that does not spend the upstream's pull budget, with every pull that arrives
during one check sharing its answer. When the upstream's tag has moved, Reliquary first stores the
new manifest, and for an index its members, and only then moves its own tag in one transaction. A
client that resolved the old digest keeps pulling the old image's blobs; one that resolves after gets
the new. No pull is ever handed one image's manifest with another's parts.

Done when:
- AC-RQ8.3.1 When an upstream tag moves, the cache serves the new digest within its freshness window and never mixes the two (T-T9.5.1).
- AC-RQ8.3.2 The freshness window is the operator's statement per upstream or repository; where none is stated every pull is checked, and concurrent pulls share one check.
- AC-RQ8.3.3 A cached tag never points at a manifest Reliquary does not hold whole.
- AC-RQ8.3.4 A check that fails (the upstream is down or throttling) serves the held tag, marked with a `Warning` header and its age.
- AC-RQ8.3.5 Checks draw on the budget's cheapest request the upstream offers, and the record says which each upstream counts.

Tests:
- T-RQ8.3.1 [happy] Move an upstream tag; pull after the window → the new digest and its layers.
- T-RQ8.3.2 [race] Move the tag while a client is between its manifest and its layers → that client completes with the old image, whole.
- T-RQ8.3.3 [race] Many pulls arrive during one check → one upstream request; all share its answer.
- T-RQ8.3.4 [failure] The upstream dies after Reliquary learned the new digest and before it stored the manifest → the tag stays on the old digest; nothing half-moved.
- T-RQ8.3.5 [edge] The upstream deletes the tag; replaces an image with an index under the same tag; moves the tag back to an older digest → the cache follows each, by the stated rule for deletes.
- T-RQ8.3.6 [edge] A window of zero, and one longer than the test → every pull checks; none does.
- T-RQ8.3.7 [hostile] An upstream that flips a tag between two digests on every request → each pull gets one whole image; the cache does not thrash its storage past the retention rule.

---

### RQ8.4 Fetched once, by however many ask

Needs: RQ8.1 · Serves: AC-T9.5.1

When many clients miss on the same blob at once, fetch it from the upstream once. The first bytes
go to every waiting client as they arrive, and to Mantle. A fetch that is cut resumes by byte range.
Nothing is published in the cache until its digest verifies; a client streaming an unverified fill is
told at the end if it failed, as a client of the upstream itself would be. A client that leaves does
not stop the fill others wait on.

Done when:
- AC-RQ8.4.1 For any number of concurrent misses on one blob, upstream requests for it are one, plus resumes after cuts.
- AC-RQ8.4.2 A client's first byte arrives without waiting for the whole blob.
- AC-RQ8.4.3 A blob whose bytes do not match its digest is never stored and never served to a later client.
- AC-RQ8.4.4 Memory for a fill is bounded by one piece per reader, whatever the blob's size and the slowest reader's pace; a reader too slow to keep up is moved to read from what is already stored.
- AC-RQ8.4.5 After a fill, serving the blob is the same path and the same speed as a pushed blob.

Tests:
- T-RQ8.4.1 [happy] Many clients pull one uncached image at once → one upstream fetch per blob; every client verifies.
- T-RQ8.4.2 [failure] The upstream cuts the body at a random byte → resumed by range; clients see no error; one extra upstream request.
- T-RQ8.4.3 [failure] The upstream sends a wrong byte mid-blob → every streaming client's pull fails verification; nothing is cached; the next pull fetches again.
- T-RQ8.4.4 [race] The first client disconnects mid-fill → the fill continues for the others and completes.
- T-RQ8.4.5 [race] A client asks for a range of a blob that is half filled → served from what is stored and what arrives, or made to wait; never wrong bytes.
- T-RQ8.4.6 [exhaustion] One reader far slower than the rest → the fast ones are not slowed; memory stays bounded.
- T-RQ8.4.7 [deadlock] A fill waiting for storage room that retention can free only by dropping what a reader of the fill holds → the fill is refused typed; no cycle.
- T-RQ8.4.8 [crash] Kill the node mid-fill → after restart the partial fill resumes or is discarded whole; no partial blob is served as complete.

---

### RQ8.5 The upstream is down, and nobody notices

Needs: RQ8.3, RQ8.4 · Serves: AC-T9.5.3

Whatever Reliquary holds, it serves, whether or not its upstream answers. A held tag is served with
its age. A blob or manifest Reliquary does not hold is refused typed, saying the upstream is
unreachable. Whether an upstream is up is learned from its answers, and asking again is paced by
RQ0.8, so a dead upstream costs each pull nothing.

Done when:
- AC-RQ8.5.1 Cached pulls failed while an upstream is down is exactly 0, measured by an outage drill (AC-T9.5.3).
- AC-RQ8.5.2 During an outage, a pull of held content adds no wait on the upstream, within the measurement's interval of a pull with the upstream up.
- AC-RQ8.5.3 A pull of content not held fails fast and typed during an outage, and succeeds without operator action once the upstream returns.
- AC-RQ8.5.4 The outage and the recovery are each one event, with the upstream named, and a gauge of held-but-unverifiable tags.

Tests:
- T-RQ8.5.1 [happy] The outage drill: cut the upstream; pull every held image from many runners → all succeed.
- T-RQ8.5.2 [failure] The upstream accepts connections and never answers; answers `500` to everything; resolves to nothing in DNS; presents an expired certificate → held content is served in each case; unheld is refused typed.
- T-RQ8.5.3 [edge] The upstream comes back with a tag moved during the outage → the next pull after the window gets the new image.
- T-RQ8.5.4 [livelock] The upstream flaps up and down faster than a fill takes → fills resume across flaps and complete; pulls of held content never fail.
- T-RQ8.5.5 [exhaustion] Thousands of pulls of unheld content during an outage → each refused cheaply; no connection pile-up toward the dead upstream.

---

### RQ8.6 Upstream credentials, and private images

Needs: RQ8.1 · Serves: AC-T9.5.2

Reliquary authenticates to upstreams with credentials it leases through the `Secrets` seam, from Veil
when Veil's API exists. A credential is used for its upstream and repository scope alone and is
never logged, stored in a record or shown in an error. What a credential fetched is not public by
being cached: a cached repository is readable by the principals its grant names, and private
upstream content fetched with one team's credential is not served to another.

Done when:
- AC-RQ8.6.1 An authenticated upstream pull works for each of the five upstreams with a leased credential, and the credential appears in no log, event, record or core dump path the tests can read.
- AC-RQ8.6.2 A principal without a grant on a cached private repository is refused, with the same answer as for a repository that does not exist.
- AC-RQ8.6.3 A lease that expires or is revoked stops upstream fetches under it at once; held content is served by its grant as before.
- AC-RQ8.6.4 A credential rotated in Veil is used on the next upstream request, with no restart.

Tests:
- T-RQ8.6.1 [happy] Pull a private upstream image through Reliquary as a granted principal → served.
- T-RQ8.6.2 [refusal] The same pull as an ungranted principal, before and after the image is cached → refused alike.
- T-RQ8.6.3 [failure] The `Secrets` seam is unavailable → fetches needing a credential are refused typed; anonymous upstreams and held content are unaffected.
- T-RQ8.6.4 [hostile] An upstream that echoes the `Authorization` header into its error body → the body is not logged or relayed.
- T-RQ8.6.5 [race] A credential revoked mid-fill → the fill stops at its next request; what was fetched and verified so far is kept by the stated rule or discarded whole.

---

### RQ8.7 Mirrors of chosen repositories

Needs: RQ8.3, RQ8.4 · Serves: AC-T9.5.1

Some images should be here before anyone asks. The operator lists repositories and tag patterns to
mirror. Reliquary copies them and keeps them current: it watches each tag with the upstream's
cheapest request, within the budget left after foreground work, and copies a new image when a tag
moves. What kiln's registry mirrored before the move is mirrored here.

Done when:
- AC-RQ8.7.1 After a mirror settles, every listed tag's digest equals its upstream's, and every blob of its closure is held, for each platform the operator listed.
- AC-RQ8.7.2 A mirrored tag that moves upstream is current here within the watch's cycle, whose length is what the budget's remainder allows and is reported.
- AC-RQ8.7.3 Mirroring never delays a foreground pull's upstream request.
- AC-RQ8.7.4 The mirror list kiln's registry had maps to Reliquary's with no entry lost, shown by a comparison.

Tests:
- T-RQ8.7.1 [happy] Mirror a repository's `v1.*` tags → each present and current.
- T-RQ8.7.2 [edge] A pattern matching no tag; matching more tags than one cycle's budget; a tag added and one deleted upstream → nothing; all, within budget over several cycles; followed.
- T-RQ8.7.3 [failure] The upstream is down for many cycles → the mirror keeps what it has and catches up after.
- T-RQ8.7.4 [exhaustion] A mirror list larger than the budget can keep current → the report says which tags are behind and by how long; foreground is unaffected.
- T-RQ8.7.5 [race] A mirrored tag moves twice within one cycle → the mirror ends at the latest; no intermediate image is half-copied.

---

### RQ8.8 What to keep when the disk is not endless

Needs: RQ8.4 · Serves: AC-T9.5.3

A cache fills. When a namespace's cached bytes reach its quota, drop what costs least to lose: for
each cached image, the chance it is pulled again, learned from its own history of pulls, against
the bytes it frees and what fetching it again would cost. Never drop what a pull in flight is
reading. Prefer to keep what cannot be fetched again while its upstream is down. No image is dropped
because a fixed time passed.

Done when:
- AC-RQ8.8.1 Cached bytes per namespace never exceed its quota beyond one fill in flight.
- AC-RQ8.8.2 On the replayed trace, the rule's miss bytes are no worse than least-recently-used's, shown by an A|B record, and the record names the rule kept.
- AC-RQ8.8.3 Nothing a pull is reading is dropped; a fill that cannot make room is refused typed.
- AC-RQ8.8.4 During an upstream outage nothing from that upstream is dropped unless nothing else can make the room, and the drop is an event.
- AC-RQ8.8.5 The rule has no age, period or count picked by hand; each input is measured and listed in the constants register.

Tests:
- T-RQ8.8.1 [happy] Fill past the quota with a workload of some hot and many cold images → cold ones go; hot ones stay; pulls of hot ones never miss.
- T-RQ8.8.2 [edge] An image pulled once long ago, and one never pulled since it was mirrored → dropped before the hot set; the mirror's listed tags are kept by the mirror's own rule.
- T-RQ8.8.3 [race] An image chosen to drop is pulled at that instant → the pull completes; the drop yields or happens after.
- T-RQ8.8.4 [exhaustion] Every cached image is in use and the quota is full → new fills are refused typed, naming the quota.
- T-RQ8.8.5 [failure] The upstream is down and the quota is full → images from live upstreams are dropped first.

---

### RQ8.9 Every runner pulls through Reliquary

Needs: RQ8.5, RQ8.6, RQ8.7 · Serves: AC-T9.5.1, AC-T9.5.2

Put it to use. Kiln configures every runner's container tools and Shards to pull through Reliquary.
Kiln's CI records every upstream rate-limit failure; the count over a week is the criterion. Run the
outage drill in the lab before production and keep its record.

Done when:
- AC-RQ8.9.1 Every runner pulls through Reliquary at LAN speed and CI never hits an upstream rate limit, in a change request that passes the gate (AC-T9.5.1).
- AC-RQ8.9.2 Upstream rate-limit failures in CI over a week is exactly 0, measured by CI's records (AC-T9.5.2).
- AC-RQ8.9.3 A cached pull on a runner's LAN is link-bound: its throughput's interval overlaps the measured link's, for images above the size where the link, not the round trips, is the limit.
- AC-RQ8.9.4 A runner whose Reliquary is unreachable falls back as kiln's change states, and the fallback is an event, never silent.

Tests:
- T-RQ8.9.1 [happy] A CI leg on each runner platform pulls its image → served by Reliquary; the upstream's request counter moved only on the first.
- T-RQ8.9.2 [happy] A week of CI's records → no rate-limit failure.
- T-RQ8.9.3 [failure] The outage drill on the lab fleet → no cached pull fails.
- T-RQ8.9.4 [edge] A leg that needs an image no runner has pulled before, during an upstream throttle → it waits within the budget and succeeds, or is refused typed with when to retry; it is not a rate-limit failure of CI.
