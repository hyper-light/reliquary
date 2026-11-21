# Phase 1: The slice

**Goal RQ1.** One `reliquary` binary on one machine takes `docker push` and gives `docker pull`,
passes the OCI conformance suite, and loses nothing it acknowledged when its power is cut.

**Delivers.** The smallest registry worth using: the distribution spec's four categories over
HTTP/1.1 and TLS, records in Amphora, bytes in Mantle, both inside the one process. Kiln can opt
into it behind `Artifacts`. Everything after this phase deepens one axis of this slice.

**Serves.** CP-T9.1 (AC-T9.1.1 in part, AC-T9.1.2). **Needs.** RQ0.

**Order.** RQ1.1 and RQ1.2 first, side by side. Then RQ1.3, RQ1.4, RQ1.5, RQ1.6, RQ1.7 and RQ1.8
side by side. RQ1.9 joins them. RQ1.10 and RQ1.11 close the phase.

---

### RQ1.1 Names, tags, digests and media types

Needs: RQ0 · Serves: AC-T9.1.2

Write the parsers every request passes through, as pure functions: repository names and tags by the
distribution spec's grammars, digests with `sha256` and `sha512`, references (`name:tag`,
`name@digest`), and the media types Reliquary knows (OCI image manifest and index, Docker's schema 2
manifest and list, the empty descriptor). Write the streaming digest: it takes bytes as they come,
and its state can be saved at a block boundary and resumed in another process, as Mantle's patched
aws-lc-rs exposes it. Docker's schema 1 is refused by name.

Done when:
- AC-RQ1.1.1 A name is accepted exactly when it matches the spec's regular expression and fits the length a registry host plus name may take; a tag exactly when it matches the spec's and is at most 128 characters.
- AC-RQ1.1.2 A digest is accepted exactly when its algorithm is registered and its encoded part has that algorithm's length in lowercase hex.
- AC-RQ1.1.3 A digest resumed from a saved state at any block boundary equals the one-shot digest.
- AC-RQ1.1.4 No parser allocates more than its input's length, reads past it, or takes time worse than linear in it.
- AC-RQ1.1.5 Each parser agrees with a reference on generated input: the spec's expression run by a regex engine for names and tags, and go-digest's vectors for digests.

Tests:
- T-RQ1.1.1 [happy] Parse every name, tag and digest in the spec's own examples → each accepted, each round-trips to the same text.
- T-RQ1.1.2 [refusal] Names with an upper-case letter, a leading or trailing separator, `..`, `//`, three underscores, or an empty component → each refused `NAME_INVALID`.
- T-RQ1.1.3 [edge] A tag of exactly 128 characters, and one of 129; a tag starting with `.` or `-` → accepted, refused, refused.
- T-RQ1.1.4 [edge] `sha256` with 63 and 65 hex digits, upper-case hex, an unknown algorithm, `sha512` with 128 digits → refused, refused, refused, refused `UNSUPPORTED`, accepted.
- T-RQ1.1.5 [hostile] Fuzz each parser with arbitrary bytes, invalid UTF-8 and inputs longer than any bound the parser has → no panic, no hang, memory within the bound.
- T-RQ1.1.6 [model] Generated strings through the parser and the reference → the same verdict on each.
- T-RQ1.1.7 [edge] Save a digest's state after each block of generated inputs, resume, finish → equals the one-shot digest every time; a state taken mid-block is refused.
- T-RQ1.1.8 [refusal] A manifest of media type `application/vnd.docker.distribution.manifest.v1+json` or `…v1+prettyjws` → refused `MANIFEST_INVALID`, saying schema 1 is not taken.

---

### RQ1.2 The records and their transactions

Needs: RQ0 · Serves: AC-T9.1.2, AC-T9.7.4

Decide what Reliquary keeps in Amphora and how each operation changes it. Rows: a repository; a blob
(its bytes' key in Mantle, its size, its state); a link that lets a repository read a blob; an upload
session; a manifest; an edge from a manifest to each thing it references; a tag; a referrer under its
subject. Each operation is one conditional transaction, or a counted series of them with a stated
state after each, so that a crash between any two leaves the registry correct. State the invariants,
write them as a TLA+ model beside Mantle's, and check the model in the gate. A blob's bytes live under
a key of their own making, never under their digest, so a blob can be collected and uploaded again
without two writers sharing a key. An answer lost after a commit is reconciled by reading, never by
doing the operation again blind.

Done when:
- AC-RQ1.2.1 `docs/design/records.md` lists each family, its key, its value and the transactions that write it, and a doc-truth test checks the list against the code.
- AC-RQ1.2.2 The invariants hold after every transaction and after a crash between any two: a tag names a manifest that exists; a manifest's every reference has a row; a link names a live blob; a live blob's bytes exist; a readable blob has a link in that repository.
- AC-RQ1.2.3 The TLA+ model has the operations push blob, mount, push manifest, move tag, delete tag, delete manifest, delete blob and collect, and TLC finds no reachable state that breaks an invariant.
- AC-RQ1.2.4 A manifest with more references than one transaction's conditions hold is committed in stages, and a reader never sees it before its last stage.
- AC-RQ1.2.5 Every write that can meet `Conflict`, `Timeout` or `UnknownOutcome` has a stated answer for each: retry on a fresh read within a counted budget, refuse, or reconcile.
- AC-RQ1.2.6 A row's value fits Amphora's bound for a value by construction; what cannot (a large manifest's bytes) is kept in Mantle and named by the row.

Tests:
- T-RQ1.2.1 [model] TLC on the model with two repositories, two clients and a collector → no invariant broken; with a planted missing condition → a trace that breaks one.
- T-RQ1.2.2 [crash] In simulation, cut power between every pair of transactions of every operation → invariants hold after restart, and an unfinished operation is either absent or finishable.
- T-RQ1.2.3 [race] Two pushes of one manifest digest at once → one row, both answered `201`.
- T-RQ1.2.4 [race] A manifest pushed while one of its blobs is deleted → the push is refused `MANIFEST_BLOB_UNKNOWN`, or the delete is refused; never a manifest whose blob is gone.
- T-RQ1.2.5 [race] Two clients moving one tag to different manifests → the tag ends at one of them, and each client's answer agrees with a linearization.
- T-RQ1.2.6 [failure] A commit whose answer is lost → the operation is not applied twice, and the client's retry gets the first outcome.
- T-RQ1.2.7 [edge] A manifest referencing one more blob than a transaction holds → committed in stages; a concurrent GET sees `404` until it is whole, then `200`.
- T-RQ1.2.8 [livelock] Many clients in a conflict loop on one tag → each finishes within its counted budget or is refused typed; none retries without end.
- T-RQ1.2.9 [exhaustion] Amphora answers `Full` → the push is refused typed and leaves no partial rows.
- T-RQ1.2.10 [edge] An index that references itself by digest (impossible by hashing) and an index nested in an index → the first cannot be made; the second is accepted and its edges recorded.

---

### RQ1.3 Blob uploads

Needs: RQ1.1, RQ1.2 · Serves: AC-T9.1.2

Take a blob by every way the spec allows: one `POST` with the digest, `POST` then `PUT`, `POST` then
`PATCH` chunks then `PUT`, with or without `Content-Range`, with a fixed length or chunked transfer
encoding. Bytes stream straight to Mantle as they arrive; nothing is buffered whole. A session's
durable offset and its digest state are in its record, so any node, or the same node after a restart,
continues it. At the closing `PUT` the digest and length are checked, and the blob becomes visible in
one transaction. Mount a blob from another repository the caller may read. Cancel on `DELETE`. Sessions
are bounded by the bytes they stage, charged to who opened them; one is reclaimed when its room is
needed, least recently progressed first, never at a fixed age.

Done when:
- AC-RQ1.3.1 Each upload form in distribution spec v1.1.1, and the upload cancel and `digest-algorithm` forms on its `main`, answers with the status and headers the spec states.
- AC-RQ1.3.2 A chunk that does not start where the last ended is refused `416`, and the session's `GET` then reports the true offset.
- AC-RQ1.3.3 A closing digest that is not the content's is refused `DIGEST_INVALID`, the blob never becomes visible, and its staged bytes are freed.
- AC-RQ1.3.4 A session continues from its durable offset after the process restarts, with no byte before that offset sent again.
- AC-RQ1.3.5 The server's memory for one upload is bounded by one piece and the counted overhead, whatever the blob's size.
- AC-RQ1.3.6 Two uploads of the same content at once both succeed, and one copy is kept.
- AC-RQ1.3.7 A mount succeeds only where the caller may read the source repository; a mount without a source follows the stated policy and never reveals whether another tenant holds the blob.
- AC-RQ1.3.8 `OCI-Chunk-Min-Length`, when sent, is the least piece Mantle's upload plan takes, not a figure of Reliquary's.

Tests:
- T-RQ1.3.1 [happy] Push one blob by each form → each `201` with `Location` and `Docker-Content-Digest`; each blob then pulls back identical.
- T-RQ1.3.2 [edge] A zero-byte blob; a one-byte blob; a blob one byte under and over a Mantle block; a blob larger than memory → each stored and read back identical.
- T-RQ1.3.3 [refusal] A chunk out of order; a chunk overlapping the last; a final `PUT` whose `Content-Range` skips bytes → each `416`, session intact.
- T-RQ1.3.4 [refusal] A closing digest of the wrong content, of the wrong length, of an unknown algorithm, and malformed → `DIGEST_INVALID`, `DIGEST_INVALID`, `UNSUPPORTED`/`400`, `DIGEST_INVALID`.
- T-RQ1.3.5 [refusal] `PATCH`, `PUT`, `GET` and `DELETE` on an unknown, finished or cancelled session → `404` `BLOB_UPLOAD_UNKNOWN`.
- T-RQ1.3.6 [failure] Kill the server mid-chunk, restart, ask the session's status, resume → the blob completes; bytes sent equal the blob's length plus at most the unacknowledged run.
- T-RQ1.3.7 [failure] The client's connection drops mid-body → the session's offset is what was made durable, and a later `PATCH` from there succeeds.
- T-RQ1.3.8 [race] Two clients writing the same session at once → one chunk wins at each offset; the other is refused `416`; the blob's digest is right.
- T-RQ1.3.9 [race] Two sessions upload the same content and close at the same instant → both `201`; one copy of the bytes remains; both repositories can read it.
- T-RQ1.3.10 [race] A session cancelled while its last chunk is in flight → either cancelled with nothing visible, or completed; never a visible blob with a freed body.
- T-RQ1.3.11 [exhaustion] Open sessions until the staging budget is reached → the next is refused typed with what is held and by whom; after the least recently progressed is reclaimed, a new one opens.
- T-RQ1.3.12 [exhaustion] Mantle answers `Full` mid-upload → the chunk is refused typed, the session keeps its durable offset, and the upload completes once room returns.
- T-RQ1.3.13 [hostile] A body longer than its `Content-Length`; a shorter one; a `Content-Range` beyond 2⁶⁴; chunked encoding with a malformed chunk size → each refused, none corrupts the session.
- T-RQ1.3.14 [hostile] A client that sends one byte at long intervals → it holds one session's room and no thread; it is cut when its progress no longer keeps it alive by RQ0.8's rule.
- T-RQ1.3.15 [edge] A mount of a digest the source repository has; one it lacks; one from a repository the caller cannot read → `201`; `202` with a new session; `202` with a new session and no hint that it exists.
- T-RQ1.3.16 [deadlock] Many sessions closing while the staging budget is full and reclaim is running → every close completes or is refused; none waits on reclaim that waits on it.

---

### RQ1.4 Blob reads

Needs: RQ1.1, RQ1.2 · Serves: AC-T9.1.2, AC-T9.1.4

Serve a blob by digest to a caller whose repository links it: `HEAD` with its length and digest, `GET`
whole or by byte range, streamed from Mantle in bounded pieces with backpressure from the client's
socket. A blob is found only through a repository that holds it. Conditional requests answer from the
digest, which is the blob's entity tag. In this phase bytes pass through Reliquary; sending the client
straight to Mantle is RQ5's.

Done when:
- AC-RQ1.4.1 `GET` and `HEAD` answer `200` with `Content-Length`, `Docker-Content-Digest` and `Content-Type: application/octet-stream`, and `404` `BLOB_UNKNOWN` for a blob the repository does not link.
- AC-RQ1.4.2 A single byte range is answered `206` with the right `Content-Range`; an unsatisfiable one `416`; several ranges are answered as RFC 9110 allows and the answer is stated.
- AC-RQ1.4.3 `If-None-Match` with the blob's digest answers `304` with no body.
- AC-RQ1.4.4 The server's memory for one read is bounded by one piece, whatever the blob's size and however slowly the client reads.
- AC-RQ1.4.5 Every byte served was verified by Mantle on read; a blob Mantle cannot verify is refused, never served wrong.

Tests:
- T-RQ1.4.1 [happy] `GET` a blob whole and in every pair of adjacent ranges → the pieces joined equal the blob, and its digest matches.
- T-RQ1.4.2 [edge] Ranges `0-0`, the last byte, `-1` (suffix), past the end, and reversed bounds → `206`, `206`, `206`, `416`, `416` or whole per RFC 9110.
- T-RQ1.4.3 [refusal] A blob that exists in another repository only → `404`, in the same time as a blob that exists nowhere, within the measurement's interval.
- T-RQ1.4.4 [failure] The client closes mid-body → the read from Mantle stops, and nothing is left open.
- T-RQ1.4.5 [failure] Mantle refuses a chunk as damaged and cannot repair it → the response ends with an error, not with wrong or padded bytes, and the event names the blob.
- T-RQ1.4.6 [race] A blob deleted while a `GET` of it streams → the stream completes with the right bytes or ends in an error; a later `GET` is `404`.
- T-RQ1.4.7 [exhaustion] More slow readers than the read budget admits → the rest are refused `429` or `503` with `Retry-After`; admitted readers are not slowed past their interval.
- T-RQ1.4.8 [hostile] A client that opens a `GET` and never reads → its memory stays one piece, and it is cut when it shows no progress by RQ0.8's rule.

---

### RQ1.5 Manifests, tags and referrers

Needs: RQ1.1, RQ1.2 · Serves: AC-T9.1.2

Take a manifest by tag or digest and keep its bytes exactly. Check it: valid JSON of its media type,
`mediaType` agreeing with `Content-Type`, every descriptor well-formed, and every blob or manifest it
references present in the repository, its `subject` excepted. Bound its size from the memory one
request is admitted with, never below the spec's 4 megabytes. Tags move atomically. A manifest with a
`subject` answers `OCI-Subject` and appears in the referrers API, filtered by `artifactType` on
request. List tags in order with `n` and `last`. Delete a tag, or a manifest with every tag on it.
Manifests are served as stored, whatever `Accept` asks.

Done when:
- AC-RQ1.5.1 A manifest pushed by digest or tag is returned byte for byte, with the `Content-Type` it was pushed with and its digest in `Docker-Content-Digest`.
- AC-RQ1.5.2 A manifest that references something absent is refused `MANIFEST_BLOB_UNKNOWN` and names it; one with a missing `subject` alone is accepted.
- AC-RQ1.5.3 A manifest over the bound is refused `413`; the bound is derived, is at least 4 megabytes, and is in the constants register as derived.
- AC-RQ1.5.4 The referrers API never answers `404` for a valid digest, returns an empty index where there are none, sets `artifactType` by the spec's fallback rule, and pages with `Link` where the index would pass the manifest bound.
- AC-RQ1.5.5 Tag listings are in the spec's order, honor `n` and `last`, set `Link` when more remain, and cost time independent of the repository's total tags.
- AC-RQ1.5.6 Deleting a manifest by digest removes every tag on it and its referrer entries; deleting by tag removes that tag alone; each is `202`, then `404`.
- AC-RQ1.5.7 The `tag` query parameters on a push by digest create each tag and answer `OCI-Tag`, as the spec's `main` states.
- AC-RQ1.5.8 A media type Reliquary does not know is stored and served, never refused for being unknown; an unknown `artifactType` likewise.

Tests:
- T-RQ1.5.1 [happy] Push an image manifest, an index of two images, an index of indexes, and an artifact with the empty config → each `201`; each pulls back identical by tag and by digest.
- T-RQ1.5.2 [refusal] A body that is not JSON; JSON whose `mediaType` differs from `Content-Type`; a descriptor with no digest; a digest in the URL that is not the body's → `MANIFEST_INVALID` ×3, `DIGEST_INVALID`.
- T-RQ1.5.3 [refusal] A manifest whose config, one layer, or one index entry is absent → `MANIFEST_BLOB_UNKNOWN` naming the digest.
- T-RQ1.5.4 [edge] A manifest of exactly the bound, and one byte over → `201`, `413`.
- T-RQ1.5.5 [edge] A manifest with no layers; with one layer listed many times; with unknown extra fields; with a `data` field; with a nondistributable layer not pushed → accepted as the conformance suite's data settings expect.
- T-RQ1.5.6 [edge] A manifest pushed with a `subject` that does not exist, then the subject pushed → the referrers list holds it both before and after.
- T-RQ1.5.7 [edge] Referrers of a digest with none; with one; with more than fit one index; filtered by a type none has → empty index; one; pages by `Link`; empty with `OCI-Filters-Applied`.
- T-RQ1.5.8 [edge] Tag lists with `n=0`, `n` larger than the tags, `last` naming a tag that does not exist, and a repository with no tags → the spec's answers.
- T-RQ1.5.9 [race] A tag read while it moves → the old digest or the new, with that digest's manifest; never one's digest with the other's bytes.
- T-RQ1.5.10 [race] A manifest deleted while a tag is being put on it → the tag ends on a manifest that exists, or the put is refused; never a dangling tag.
- T-RQ1.5.11 [race] Two referrers pushed for one subject at once → both listed, neither lost.
- T-RQ1.5.12 [hostile] JSON nested deeper than the parser's flat budget; a megabyte string of escapes; duplicate keys; a manifest listing the same digest with two sizes → refused typed, in linear time, with no recursion.
- T-RQ1.5.13 [exhaustion] A listing of a repository with more tags than memory would hold at once → served page by page in bounded memory.
- T-RQ1.5.14 [happy] `GET` a manifest with `Accept` naming only another type → the stored manifest with its own `Content-Type`, and the decision is in the design record.

---

### RQ1.6 The OCI door over HTTP/1.1 and TLS

Needs: RQ0 · Serves: AC-T9.1.2

Listen for HTTP/1.1 on hyper-rt's shards, plain on loopback and under TLS 1.3 elsewhere, using
Mantle's HTTP codec. Route `/v2/` requests to the core. Stream request and response bodies with
backpressure both ways. Answer errors in the spec's JSON. Honor `Expect: 100-continue`, chunked
request bodies, keep-alive and `HEAD`. Every bound on a head, a line or a header count is the codec's
own or derived from the memory a connection is charged. A connection that shows no progress is cut by
RQ0.8's rule. Stopping drains: requests in flight finish, new ones are refused.

Done when:
- AC-RQ1.6.1 Every endpoint of the spec is routed, and any other path under `/v2/` answers the spec's `404` or `405` with a JSON error.
- AC-RQ1.6.2 A response's errors are `{"errors":[{"code","message","detail"}]}` with a code from the spec's table.
- AC-RQ1.6.3 A connection's memory is counted and bounded, idle or active; the count is pinned by a test.
- AC-RQ1.6.4 No request can hold a shard: a handler that would run past its turn yields, and a slow client's socket never blocks another connection.
- AC-RQ1.6.5 TLS is 1.3 with the certificate the operator states or one `reliquary init` makes; plain HTTP is served only on loopback unless stated.
- AC-RQ1.6.6 On a stop signal, requests in flight complete or are cut at the drain's end with a typed close, and the listener takes no new ones.
- AC-RQ1.6.7 `Warning` headers, when sent, follow the spec's form and its 4,096-byte bound.

Tests:
- T-RQ1.6.1 [happy] Each endpoint through a raw socket and through curl → the spec's status, headers and body.
- T-RQ1.6.2 [edge] `/v2`, `/v2/`, a name with the most components the grammar allows, a path that only looks like a route (`/v2/a/blobs/uploads/` as a repository named `a/blobs`) → routed as the grammar says.
- T-RQ1.6.3 [edge] `Expect: 100-continue` on an upload that will be refused → the refusal comes before any body is read.
- T-RQ1.6.4 [edge] Two requests pipelined on one connection; a request after a response with `Connection: close` → answered in order; the second refused by closing.
- T-RQ1.6.5 [hostile] The HTTP request-smuggling corpus: `Content-Length` with `Transfer-Encoding`, duplicate lengths, obsolete line folding, bare LF, a chunk extension flood → each refused `400` and the connection closed.
- T-RQ1.6.6 [hostile] A head sent one byte at a time; a head that never ends; more headers than the bound → cut by progress, refused by the bound, refused by the bound.
- T-RQ1.6.7 [hostile] TLS: a handshake that stalls after ClientHello; a downgrade to TLS 1.2; a client that renegotiates → cut by progress; refused; refused.
- T-RQ1.6.8 [exhaustion] Open connections until the memory budget is reached → further ones are refused at accept with nothing allocated per refusal; closing some admits new ones.
- T-RQ1.6.9 [deadlock] Every admitted connection uploading while every one's response is blocked on a full send buffer → each makes progress or is cut; the shard's loop never stops turning.
- T-RQ1.6.10 [failure] Stop the server during a large upload and a large download → the download finishes or is cut cleanly at the drain's end; the upload's session keeps its durable offset.
- T-RQ1.6.11 [happy] The same requests against the door on Linux, macOS and Windows → the same answers.

---

### RQ1.7 Authentication and authorization

Needs: RQ1.1 · Serves: AC-T9.1.2

Answer `/v2/` with the bearer challenge and serve the token endpoint as Distribution's token
specification describes, with HTTP Basic beside it. A principal comes from an `Identity` seam: a file
of users and grants when Reliquary stands alone, kiln's own when it is embedded. Scopes are the
specification's: pull, push and delete on a repository, and the catalog. Tokens are signed, name their
scopes, and live as long as the operator states; unstated, they live the specification's default.
Anonymous pull is off unless a repository's grant says otherwise. Checking a token costs no records
read.

Done when:
- AC-RQ1.7.1 An unauthenticated request to a protected endpoint is answered `401` with `WWW-Authenticate: Bearer realm=…,service=…,scope=…` naming exactly the scope the request needs.
- AC-RQ1.7.2 The token endpoint grants only the intersection of the scopes asked and the principal's grants, and says which it granted.
- AC-RQ1.7.3 A request beyond its token's scopes is `403` `DENIED`; with an expired, malformed or wrongly signed token `401` `UNAUTHORIZED`.
- AC-RQ1.7.4 A token's lifetime is the operator's statement, or the token specification's 60-second default, and is never below that specification's 60-second floor.
- AC-RQ1.7.5 `docker login`, containerd, Podman, ORAS and crane each authenticate, push and pull with no client-side special case.
- AC-RQ1.7.6 Secrets (signing keys, passwords) are never logged, never in an error body, and held in memory that is wiped on drop.
- AC-RQ1.7.7 A password check costs the same time for a known and an unknown user, within the measurement's interval.

Tests:
- T-RQ1.7.1 [happy] `docker login`, then push and pull a private repository → succeeds; the same without login → `401` then the client's prompt.
- T-RQ1.7.2 [refusal] A pull token used to push; a token for repository A used on B; a token after its expiry; a token signed by another key; a token with `alg: none` → `403`, `403`, `401`, `401`, `401`.
- T-RQ1.7.3 [edge] A request needing two scopes (a mount: pull on the source, push on the target) → the challenge names both, and a token with one is refused.
- T-RQ1.7.4 [edge] A token that expires during a long upload → the chunk in flight completes; the next is `401`; after a new token the session continues from its offset.
- T-RQ1.7.5 [hostile] A token longer than the head's bound; a JWT with a nested JWT; a `kid` that is a file path; scopes repeated past the bound → refused in bounded time and memory.
- T-RQ1.7.6 [hostile] A run of wrong passwords for one user → each costs the full check; the limiter RQ4.3 adds later is not assumed here, and the test records the rate one core allows.
- T-RQ1.7.7 [race] A grant revoked while a token made under it is in use → the token works until it expires and no longer; a new token lacks the scope.
- T-RQ1.7.8 [exhaustion] The token endpoint asked faster than it signs → refused `429` with `Retry-After`; registry requests with valid tokens are unaffected.
- T-RQ1.7.9 [happy] Ask the catalog endpoint as each of two principals with different grants → each sees only the repositories it may pull.

---

### RQ1.8 Records on Amphora and bytes on Mantle, inside the process

Needs: RQ0.4 · Serves: AC-T9.1.1

Implement `Records` on Amphora's embedded store and `Bytes` on `mantle-client` talking to a Mantle
cell of one that the same process runs on a shard of its own. Ask Mantle's owner for the in-process
node entry point their design names (a node opened by a call, reached through the loopback peer);
until it lands, start the node as `mantle node` does, from the vendored crates, and reach it over
loopback QUIC. Both pass RQ0.4's suites. An answer goes to a client only after Amphora and Mantle have
each made their part durable with the platform's full flush.

Done when:
- AC-RQ1.8.1 The real `Records` and `Bytes` pass the same conformance suites as their simulators, on each of the five targets.
- AC-RQ1.8.2 No push is acknowledged before its bytes and its rows are durable; a power cut after an acknowledgement loses nothing.
- AC-RQ1.8.3 Mantle's node has a shard to itself, and the registry's request shards are never blocked by its compaction or flushes.
- AC-RQ1.8.4 The request to Mantle's owner for the in-process entry point is recorded with its answer; the interim start is one module, named in the manifest's exception, and is deleted when the entry point lands.
- AC-RQ1.8.5 `Busy` and `Full` from Mantle, and `Full`, `Timeout` and `UnknownOutcome` from Amphora, each reach the client as a typed refusal with a derived `Retry-After` or are reconciled; none is retried in a loop.

Tests:
- T-RQ1.8.1 [happy] Run both seam suites against the real implementations → every clause passes.
- T-RQ1.8.2 [crash] `kill -9` the process at random points during pushes, many times, on APFS, ext4 and NTFS → after each restart every acknowledged blob and manifest is present and verifies; none is partial.
- T-RQ1.8.3 [failure] Fill the device until Mantle answers `Full` → pushes are refused typed; pulls keep working; after space is freed pushes resume without a restart.
- T-RQ1.8.4 [failure] Amphora's flush fails once → the store refuses what waits, the registry refuses writes typed and says why, and a restart recovers every acknowledged row.
- T-RQ1.8.5 [edge] A machine that sleeps mid-push and wakes → the push resumes or is refused typed; no wait is trusted across the sleep.
- T-RQ1.8.6 [exhaustion] Mantle's write queue answers `Busy` under a burst → the door answers `503` with a `Retry-After` derived from the queue's drain, and the burst completes without a retry storm.
- T-RQ1.8.7 [deadlock] Records and bytes each waiting on the other's flush under full load → the commit order is fixed (bytes, then rows), and no cycle of waits forms.

---

### RQ1.9 The binary and what it is told

Needs: RQ1.6, RQ1.8 · Serves: AC-T9.1.1

`reliquary serve` starts the registry from a data directory and a short configuration. The operator
states only what cannot be measured: where to listen, where data lives, who may do what, how many
failures to survive, and optionally a ceiling on memory. The node measures the rest (cores granted,
memory allowed, its devices through Mantle's probe) and derives every budget and bound from them.
`reliquary doctor` prints what was stated, measured and derived, as `mantle disk probe` does. The
directory is locked against a second process. Startup recovers before it listens, and a failure to
start is one typed line saying what and why.

Done when:
- AC-RQ1.9.1 From an empty directory, `reliquary init` then `reliquary serve` gives a registry that takes a push, with no other file written by hand.
- AC-RQ1.9.2 The configuration has no field for a timeout, a retry count, a queue length, a cache size, a chunk size or a concurrency.
- AC-RQ1.9.3 `reliquary doctor` lists each stated, measured and derived value with its source, and each derived value matches the one the running node uses.
- AC-RQ1.9.4 A second `reliquary serve` on the same directory is refused, naming the holder.
- AC-RQ1.9.5 The node refuses to start, naming the class of work, when its memory allowance is below the floor that admits one unit of each class.
- AC-RQ1.9.6 The node is ready (its health endpoint says so) only after recovery completes, and says "not ready" with the reason before.
- AC-RQ1.9.7 The same binary, configuration and data directory work on each of the five targets.

Tests:
- T-RQ1.9.1 [happy] `init`, `serve`, push, stop, `serve`, pull → the image is identical.
- T-RQ1.9.2 [refusal] A configuration with an unknown key, a port in use, an unreadable certificate, or a data directory on a read-only device → one typed line each, exit status non-zero, nothing half-started.
- T-RQ1.9.3 [edge] Start under a cgroup memory limit, a Windows job limit and no limit → the derived budgets follow each.
- T-RQ1.9.4 [edge] Start with one core granted; with all cores → both serve; the derived shard count follows.
- T-RQ1.9.5 [exhaustion] A memory ceiling below the floor → refused at start with the class and the floor.
- T-RQ1.9.6 [failure] `kill -9` during startup recovery, then start again → recovery completes; nothing is recovered twice.
- T-RQ1.9.7 [race] Two `serve` processes started at the same instant on one directory → exactly one runs.
- T-RQ1.9.8 [happy] Run `reliquary --version` and `reliquary doctor` on a fresh directory → the name, version and commit; every value with its source, and none marked unknown without saying what would measure it.

---

### RQ1.10 Conformance and the clients people use

Needs: RQ1.3, RQ1.4, RQ1.5, RQ1.6, RQ1.7, RQ1.9 · Serves: AC-T9.1.2, AC-T9.1.1

Run the OCI conformance suite, pinned at a commit, against the binary in the gate, with every
category on. Then run the clients: Docker with both image stores, containerd, Podman and Skopeo, crane,
ORAS, regctl, Helm, cosign, BuildKit's registry cache and Shards. Each pushes, pulls, lists and
deletes, and every artifact comes back byte for byte. Give kiln the opt-in: `Artifacts` implemented on
Reliquary inside kiln's process, behind a setting, passing kiln-seams' own suite.

Done when:
- AC-RQ1.10.1 The conformance suite at `OCI_VERSION=1.1` passes 100% of pull, push, content discovery and content management with every `OCI_API_*` and `OCI_DATA_*` setting at its default (AC-T9.1.2).
- AC-RQ1.10.2 The suite at `OCI_VERSION=dev` with upload cancel, tag parameters, anonymous mount, SHA-512 and sparse data turned on passes, or each failing case is listed with the spec text that decides it.
- AC-RQ1.10.3 Each named client completes push, pull, tag list, referrers where it has them, and delete, on Linux; Docker, containerd and ORAS also on macOS and Windows.
- AC-RQ1.10.4 An image pushed by one client pulls identically with every other.
- AC-RQ1.10.5 kiln-seams' `Artifacts` suite passes against Reliquary embedded in kiln's process, in a kiln change request that passes kiln's gate with the setting off by default.
- AC-RQ1.10.6 The suite's `report.html` and `junit.xml` are kept with the commit as evidence.

Tests:
- T-RQ1.10.1 [happy] The conformance suite, all categories → 100%.
- T-RQ1.10.2 [happy] A multi-platform image pushed with Docker and pulled with containerd, Podman and crane on another architecture → each gets its platform's layers and runs.
- T-RQ1.10.3 [happy] Docker's schema 2 image pushed and pulled back → identical manifest bytes and media type.
- T-RQ1.10.4 [happy] A Helm chart, a cosign signature and a BuildKit cache pushed and read back by their own tools → each verifies.
- T-RQ1.10.5 [edge] An image whose layers are zstd, gzip and uncompressed; one with a layer larger than memory; one with as many layers as Docker itself allows → each round-trips.
- T-RQ1.10.6 [failure] Each client's push interrupted at a random point and run again → it completes without re-sending finished layers where the client supports resuming.
- T-RQ1.10.7 [race] Clients of every kind in the matrix pushing different tags of one repository at once → every tag ends where its client put it; shared layers are stored once.
- T-RQ1.10.8 [refusal] Each client against a private repository without credentials → its own "unauthorized" message, not a parse error.
- T-RQ1.10.9 [happy] kiln-seams' `Artifacts` suite, simulator and Reliquary side by side → the same clauses pass on both.

---

### RQ1.11 Nothing acknowledged is lost

Needs: RQ1.10 · Serves: AC-T9.1.1, AC-T9.10.3

Prove the slice's durability and safety as a whole. In simulation, run generated histories with the
nemesis and a power cut at every write, and compare each run with the model. On real file systems,
kill the process at random points under load. Count what the tests exercised, so a quiet pass cannot
hide a path never taken.

Done when:
- AC-RQ1.11.1 Acknowledged blobs, manifests or tags lost across every simulated and real crash run is exactly 0.
- AC-RQ1.11.2 Bytes served that do not match their digest across every run is exactly 0.
- AC-RQ1.11.3 Every run that ends without a verdict (a hang) fails, with the waiters named.
- AC-RQ1.11.4 Each crash point, each fault kind and each operation kind has a non-zero count in the gate's nightly sweep.
- AC-RQ1.11.5 A failing seed reproduces its failure, and its shrunk history is kept as a regression test with the bug's record in `docs/bugs/`.

Tests:
- T-RQ1.11.1 [crash] Power cut before and after every write of every operation in generated histories → the registry matches the model on every acknowledged operation.
- T-RQ1.11.2 [crash] `kill -9` under mixed push and pull load on each of APFS, ext4, tmpfs and NTFS → restart, verify every acknowledged artifact by digest.
- T-RQ1.11.3 [failure] A torn write to a records page; a bit flip in a stored chunk; a flush that fails → detected, the affected operation refused typed, nothing wrong served.
- T-RQ1.11.4 [livelock] A client retrying a refused push forever while another deletes what it pushes → both make progress or are refused; the run ends.
- T-RQ1.11.5 [deadlock] Every operation kind at once at the admission limit → no run ends with work waiting and nothing running.
- T-RQ1.11.6 [non-vacuity] The sweep's counters for crash points, faults and operations → all above zero; a new write site without a crash point fails the sweep.
- T-RQ1.11.7 [exhaustion] Memory, staging room, records room and connections each driven to its bound in turn → each ends in its typed refusal, and service resumes when the pressure lifts, without a restart.
