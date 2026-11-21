# Phase 11: Kiln's own artifacts

**Goal RQ11.** Kiln hands out everything it builds and keeps from Reliquary: runner binaries, CI
caches and artifacts, its own images and Agentfile images. A runner's upgrade downloads only what
changed.

**Delivers.** One store behind kiln instead of several. Upgrades that fetch a fraction of a binary.
GitHub's cache and artifact protocols, which kiln already speaks, with Reliquary beneath them.

**Serves.** CP-T9.8 (AC-T9.8.1 to AC-T9.8.3, T-T9.8.1). **Needs.** RQ10.2 and RQ10.3 (ranges and
chunks) for deltas; RQ7.1 and RQ7.6 for signing and provenance; RQ9.5 and RQ9.6 for quotas and
retention.

**Order.** RQ11.1 first. RQ11.2, RQ11.3, RQ11.4 and RQ11.5 side by side. RQ11.6 closes.

---

### RQ11.1 An upgrade that downloads only what changed

Needs: RQ10.1, RQ10.3 · Serves: AC-T9.8.2

A new build of a binary shares much with the last, but where and how much depends on how it was
built. Measure first, on kiln's own last releases for every target: what a runner would download
with each method. Content-defined chunks need no work per pair of versions; a patch made against the
previous build is smaller and costs the publisher a computation per pair. Keep the method, or the
combination, that the measurements favor, and publish each release with what that method needs
attached as referrers. A runner states which build it has and receives the cheapest way from there
to the new one, whole download included when that is cheapest. Whatever path it took, the result is
checked against the new binary's digest and signature before it is used.

Done when:
- AC-RQ11.1.1 Bytes a runner upgrade downloads, against the binary's size, is at most 30%, measured by the upgrade lab over 10 consecutive releases (AC-T9.8.2), for each target kiln ships.
- AC-RQ11.1.2 The A|B record lists each method's bytes downloaded, publisher cost and client cost over those releases, and names the method kept.
- AC-RQ11.1.3 The binary a runner ends with is byte-identical to the published one, and its signature verifies, whatever build it started from.
- AC-RQ11.1.4 A runner on a build with no cheap path (too old, or unknown) gets the whole binary, with no error.
- AC-RQ11.1.5 The client's memory and time to apply an upgrade are recorded per target and bounded by a budget derived from the runner's allowance.
- AC-RQ11.1.6 No chunk size, patch window or "how many versions back" is fixed in code; each comes from the measurement or the release's own history.

Tests:
- T-RQ11.1.1 [happy] Upgrade through ten consecutive releases on each target → each download at most 30% of its binary; each result verifies.
- T-RQ11.1.2 [edge] Upgrade from two, five and twenty releases back; from a build of another branch; from a corrupted local binary → cheapest valid path; whole download; whole download.
- T-RQ11.1.3 [failure] The download is cut at points across it → resumed; the result verifies; no verified byte fetched twice.
- T-RQ11.1.4 [hostile] A delta that applies to a binary other than the stated one; a delta whose result has the wrong digest → refused before use; the whole binary is fetched.
- T-RQ11.1.5 [crash] Kill the runner while it applies an upgrade → the old binary is intact; the retry completes.
- T-RQ11.1.6 [exhaustion] A runner with little memory or disk → the method within its budget is used, or the whole download; never a failed upgrade for lack of room that a smaller method would fit.
- T-RQ11.1.7 [race] A release is published while a runner is mid-upgrade to the previous one → it finishes that upgrade; the next takes it to the newest.

---

### RQ11.2 The blue/green upgrade, on Reliquary

Needs: RQ11.1 · Serves: T-T9.8.1, AC-T9.8.1

Kiln's runners upgrade blue/green: download the new build, check it, start it beside the old, hand
over. Move the download and the check onto Reliquary and change nothing else. The lab that checks
the upgrade keeps every one of its checks exactly as it is.

Done when:
- AC-RQ11.2.1 The blue/green upgrade lab (33 checks), unchanged in what it checks, passes with builds served by Reliquary (T-T9.8.1).
- AC-RQ11.2.2 A runner verifies the build's signature and the tag's signed statement before it starts the new process.
- AC-RQ11.2.3 With Reliquary unreachable, a runner keeps running its current build and reports why it did not upgrade.
- AC-RQ11.2.4 A Mac, a Linux and a Windows runner each upgrade from Reliquary.

Tests:
- T-RQ11.2.1 [happy] The lab's 33 checks → all pass.
- T-RQ11.2.2 [refusal] A build whose signature does not verify; one that is older than the running build under a moved tag → not started; the runner stays on its build and says why.
- T-RQ11.2.3 [failure] Reliquary dies after the download and before the handover → the handover completes from the local copy.
- T-RQ11.2.4 [happy] A fleet with one runner a release behind → both builds are served; each upgrades when told.

---

### RQ11.3 CI caches

Needs: RQ9.5, RQ9.6 · Serves: AC-T9.8.3

Kiln serves GitHub's cache protocol to `actions/cache` and the tools built on it. Keep the protocol
and its semantics exactly, and keep what it stores in Reliquary: a cache entry is an artifact named
by its scope, key and version, found by exact key or by the longest matching restore prefix as
GitHub defines it. Entries are created once and never changed. A repository's caches are held to its
quota, and what goes when the quota is reached is decided as RQ8.8 decides for cached images: by the
chance of another hit against the bytes freed. Large entries are stored by chunk, so a cache that
changes a little costs a little.

Done when:
- AC-RQ11.3.1 The CI cache service (GitHub's cache API) works unchanged on Reliquary (AC-T9.8.3): `actions/cache`, the setup actions' caches and BuildKit's cache backend pass kiln's existing suite with no client change.
- AC-RQ11.3.2 Lookup returns what GitHub's rules return for every case of kiln's cache conformance: exact key, restore prefixes in order, scope fallbacks, version mismatch.
- AC-RQ11.3.3 A cache entry, once finalized, is immutable; a second save of the same key and version is refused as GitHub refuses it.
- AC-RQ11.3.4 Stored bytes for a sequence of slightly changed caches grow by about the changes, per RQ10.1's measured sharing for that content.
- AC-RQ11.3.5 A restore's download is a signed URL to bytes served at the speed RQ4 measured for blobs of its size.

Tests:
- T-RQ11.3.1 [happy] A workflow saves a cache and a later run restores it → identical bytes.
- T-RQ11.3.2 [edge] Keys that share a prefix; a restore key that matches several entries; an entry from the default branch read by a pull request and the reverse → GitHub's answers.
- T-RQ11.3.3 [race] Two legs save the same key at once → one wins; the other gets GitHub's "already exists".
- T-RQ11.3.4 [race] An entry evicted while a leg downloads it → the download completes or fails cleanly; a miss on the next lookup.
- T-RQ11.3.5 [failure] A save cut midway → no entry; no partial restore possible.
- T-RQ11.3.6 [exhaustion] Saves past the repository's quota → the least valuable entries go first; the save succeeds or is refused typed if nothing can go.
- T-RQ11.3.7 [hostile] A token for one repository's legs asking for another's cache → refused.

---

### RQ11.4 CI artifacts

Needs: RQ9.5, RQ9.6 · Serves: AC-T9.8.3

Kiln serves GitHub's artifact protocol to `upload-artifact` and `download-artifact`. Keep it, with
its semantics: names unique within a run, immutable once finalized, listed and downloaded by name or
ID. An uploaded artifact is an artifact in Reliquary, under its run. How long artifacts are kept is
the repository's stated retention. A run's artifacts can be browsed file by file through RQ2.7's
index without downloading the zip.

Done when:
- AC-RQ11.4.1 The CI artifact service (GitHub's artifact API) works unchanged on Reliquary (AC-T9.8.3): the stock actions pass kiln's existing suite with no client change.
- AC-RQ11.4.2 An artifact finalized is immutable and has the digest the client stated; a mismatch refuses the finalize.
- AC-RQ11.4.3 Artifacts are removed when the stated retention says, through RQ9.6, and never before.
- AC-RQ11.4.4 One file of an uploaded zip can be listed and read through the registry.

Tests:
- T-RQ11.4.1 [happy] Upload in one job, download in another → identical.
- T-RQ11.4.2 [edge] An empty artifact; one of many small files; one larger than memory; two artifacts with names differing by case → handled as GitHub handles them.
- T-RQ11.4.3 [refusal] A second upload under a finalized name; a finalize with a wrong hash → refused with the protocol's own errors.
- T-RQ11.4.4 [failure] The upload's connection drops and the action retries → one artifact, whole.
- T-RQ11.4.5 [race] A run deleted while its artifact uploads → the upload fails cleanly; nothing orphaned past the collector.
- T-RQ11.4.6 [hostile] A zip built to mislead the file index (RQ2.2's cases) uploaded as an artifact → stored as bytes, indexed as unreadable, never refused by a protocol that promised to take it.

---

### RQ11.5 Kiln's images and default Agentfiles, published as they should be

Needs: RQ7.1, RQ7.6, RQ2.6 · Serves: AC-T9.8.1

Kiln's CI publishes kiln's own images, its runner images, and the default Agentfile image for every
harness to Reliquary: each with its SBOM and provenance attached and signed by kiln's key, each
microVM image with its boot-ready forms (RQ10.8). Reliquary's own release is published the same way,
from the same pipeline.

Done when:
- AC-RQ11.5.1 Every image kiln's pipelines build is in Reliquary with a verified signature, an SBOM and a provenance; RQ7.6's check finds none missing.
- AC-RQ11.5.2 Each default Agentfile image passes RQ2.6's checks and launches under RQ7.7's rule.
- AC-RQ11.5.3 A build is reproducible to its digest from its recorded source and pipeline, for the images kiln declares reproducible.

Tests:
- T-RQ11.5.1 [happy] The pipeline runs on a change → new versions appear, signed, attested, with forms attached.
- T-RQ11.5.2 [refusal] A pipeline step that pushes without signing → the namespace's policy refuses the push or the pull, as stated.
- T-RQ11.5.3 [happy] Launch each harness from its default Agentfile on a fresh runner → it starts, lazily.

---

### RQ11.6 One store

Needs: RQ11.2, RQ11.3, RQ11.4, RQ11.5 · Serves: AC-T9.8.1

Finish the move. What kiln still keeps in its own content-addressed store goes to Reliquary or, for
bytes that are not artifacts, straight to Mantle through `Blobs`. The old store is read-only for one
release, then gone. Kiln's change request makes the move; a capability and a drilled rollback guard
it.

Done when:
- AC-RQ11.6.1 Kiln hands out every artifact from one store, and runner upgrades download only what changed, in a change request that passes the gate (AC-T9.8.1).
- AC-RQ11.6.2 After the move, every object the old store held is reachable through the new path with the same digest, or is listed as deliberately not moved, with why.
- AC-RQ11.6.3 With the capability off, kiln reads and writes the old store as before; the rollback drill loses nothing.
- AC-RQ11.6.4 Kiln's code has one path to artifacts, checked by kiln's dependency test: no direct use of the old store remains.

Tests:
- T-RQ11.6.1 [happy] Move a copy of production's store → every digest verifies through the new path.
- T-RQ11.6.2 [failure] Kill the move at random points → it resumes; nothing is moved twice or half.
- T-RQ11.6.3 [race] Legs save caches and artifacts during the move → each lands in exactly one store and is found.
- T-RQ11.6.4 [happy] The rollback drill → the old store serves; what was written meanwhile is listed.
