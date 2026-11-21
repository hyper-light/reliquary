# The reference model and the differential harness

Status: built 2026-11-20 by plan item RQ0.5 (crates/reliquary-api); not committed. Each decision says
what it chose, what it rests on, and what would falsify it (CLAUDE.md §5). Sources: the distribution
spec v1.1.1 and its `main` (research/02 [1], [2]); the conformance suite of v1.1.1
(https://github.com/opencontainers/distribution-spec/tree/v1.1.1/conformance, `01_pull_test.go` to
`04_management_test.go`, `setup.go`, `README.md`); the image spec v1.1.1 (research/02 [4]); the
registries surveyed in research/03; and the papers and RFCs of research/10.

## What it is

The registry's meaning, written as a small program that answers requests, and the tools that hold
an implementation to it. It lives in `reliquary-api` because architecture D24 gives that crate "the
trait others reach Reliquary through, its types, a simulator and a conformance suite": the model is
the simulator, the harness is the suite, and the [`Registry`](../../crates/reliquary-api/src/harness.rs)
trait is what both doors, a node, three nodes and three regions implement to be run by it.

| Part | Where | What it does |
|---|---|---|
| Requests and answers | `src/request.rs`, `src/answer.rs` | Each endpoint of the spec as a request; statuses, the spec's 14 codes, what a client reads; `ANSWERS`, every answer the model gives |
| Grammars | `src/grammar.rs` | Names, tags, digests, references and manifest media types as the specs write them |
| The model | `src/model.rs` | A state; `decide(&self, request)` gives the answer and the change; `apply` makes the change; `check` holds the invariants |
| The generator | `src/generate.rs` | Seeded requests, valid or broken one way each, built on a shadow of the model; sequential histories, concurrent plans, the tag race |
| Histories and plans | `src/history.rs` | What clients asked and were told on one clock; plans that interleave clients and lose answers, run against any registry |
| The checker | `src/check.rs` | Linearizability against the model, on a stated budget, naming the first operation no order explains |
| The shrinker | `src/shrink.rs` | A failing input cut to a 1-minimal one |
| The harness | `src/harness.rs` | The `Registry` trait, coverage counters and their gaps, comparisons, hunts that shrink what they find and report the seed it replays from |

Later items run the same harness on more: RQ0.6's simulated world produces histories for `check`;
RQ1's OCI door and RQ3's native door implement `Registry` (RQ3.9 sends one history through both);
RQ9 runs it on three nodes and RQ12 on three regions (CLAUDE.md §8).

## Decisions

### H1. The model answers every endpoint, from text a client could send

**Chosen.** One request kind per endpoint of v1.1.1's table (end-1 to end-13, with end-4a/4b,
end-8a/8b and end-12a/12b), and the upload cancel `main` adds (end-14), which RQ1.3 builds. What a
client writes as text (names, digests, references, tags, `Content-Range`) stays text in the request,
so a request can be malformed the way a client's can, and the model parses it. Every answer is a
status, the spec's code when it refuses ("The `code` field MUST be one of the following", research/02
§1), and what the spec makes meaningful in the response (`Docker-Content-Digest`, `OCI-Subject`,
`Range`, `Link`, `OCI-Filters-Applied`, the body). `decide` takes `&self`; `apply` is the only writer;
the model's files hold no I/O, clock, thread, lock or cache, which a test reads (AC-RQ0.5.1).

**Falsified by.** An endpoint of the spec the model cannot express, or an answer RQ1's conformance run
(AC-T9.1.2) needs that the model does not give.

### H2. One order of checks for every request

**Chosen.** The text first (a malformed name, digest, reference or `Content-Range` is `400` with its
code), then who asks (`401 UNAUTHORIZED` without credentials, `403 DENIED` without the grant, whether
or not the repository exists), then what exists (`404` with the code of what is missing: the
repository, the blob, the manifest, the session), then what the request would do. Codes the spec's
table leaves open: a digest whose algorithm is not registered is `400 UNSUPPORTED` (plan T-RQ1.1.4); a
`<reference>` with no `:` that is not a tag is `400 MANIFEST_INVALID`; a manifest past the bound is
`413 MANIFEST_INVALID`; a chunk out of order, by `PATCH` or in the closing `PUT`, is
`416 BLOB_UPLOAD_INVALID` (v1.1.1: "If a chunk is uploaded out of order, the registry MUST respond with
a `416`"; `main` adds 416 to end-6 for the final chunk).

**Rests on.** Architecture D18: a principal without the grant learns nothing of what exists, so `403`
comes before any `404`. The conformance suite's "400 response body should contain OCI-conforming JSON
message" accepts `400` with any of the 14 codes for `sha256:totallywrong`.

**Falsified by.** A conformance case expecting another status where two checks fail at once.

### H3. A manifest is carried as structure and as bytes

**Chosen.** A pushed manifest carries the bytes a door sends and the structure those bytes encode
(image manifest, index, a body the model does not read, or bytes that are not a manifest); the model
reads the structure, the doors send the bytes, and the digest is of the bytes. The generator makes
both from one structure (`src/json.rs`), and a test reads every manifest it writes back with a JSON
reader and compares field by field.

**Rests on.** CLAUDE.md §11: a JSON parser written by recursion has no place in production code, and
RQ1.5 holds Reliquary's own flat parser to hostile input; the model's job is the meaning, not parsing.

**Falsified by.** A generated manifest whose bytes do not say what its structure says (the test).

### H4. What a manifest's media type makes the model check

**Chosen.** The `Content-Type`, without parameters, says the kind: OCI's image manifest and Docker's
schema 2 manifest are image manifests; OCI's index and Docker's manifest list are indexes; Docker's
schema 1 is refused `MANIFEST_INVALID` by name (T-RQ1.1.8); any other type is stored and served
unchecked (AC-RQ1.5.8). For a known kind: the body must be that kind, a `mediaType` field must agree
with the `Content-Type`, and every descriptor's digest must keep the grammar. A descriptor of an
algorithm the image spec does not register passes validation ("Implementations SHOULD allow digests
with unrecognized algorithms to pass validation if they comply with the above grammar", image spec
descriptor.md) and names nothing a registry can hold, so as a reference it is `MANIFEST_BLOB_UNKNOWN`.

**Falsified by.** A manifest the conformance suite pushes that the model refuses.

### H5. References are strong, the subject is weak, and what is referenced cannot be deleted

**Chosen.** An image manifest's config and layers must be blobs its repository links, with the size
its descriptor states (else `SIZE_INVALID`, "provided length did not match content length"); an
index's manifests must be manifests of its repository; the first reference in document order that
fails is named in the answer (AC-RQ1.5.2). A `subject` may name nothing ("A registry MUST initially
accept an otherwise valid manifest with a `subject` field that references a manifest that does not
exist"). A blob a manifest of its repository lists, or a manifest an index of its repository lists,
cannot be deleted: `405 UNSUPPORTED`. Deleting a manifest by digest removes it and every tag on it;
by tag, the tag alone.

**Rests on.** The spec makes `405` the answer when a deletion is disabled ("If blob deletion is
disabled, the registry MUST respond with either a `400 Bad Request` or a `405 Method Not Allowed`"),
and the conformance suite expects registries that refuse it: its README says "Some registries require
the opposite order, deleting manifests before blobs", its default deletes manifests first
(`OCI_DELETE_MANIFEST_BEFORE_BLOBS`), and every blob delete it makes accepts `405`. Quay refuses blob
deletes outright: "We do not support deleting arbitrary digests, as they break repo images"
(research/03 F175); trow answers its refusals `405 UNSUPPORTED` (F305). RQ1.2's invariants and
T-RQ1.2.4 ask that a push and a delete never leave a manifest whose blob is gone. With deletes refused,
"a tag names a manifest that exists; a manifest's blobs exist" holds in every state the model can
reach, which T-RQ0.5.1 checks after every request: the model's form of AC-T9.7.4, referenced blobs
collected exactly 0. The spec on manifests: "Once deleted, a `GET` to `/v2/<name>/manifests/<digest>`
and any tag pointing to that digest will return a 404."

**Falsified by.** A stock client's workflow that deletes a blob before the manifest that lists it and
fails on the `405`.

### H6. Tags are listed in one total order

**Chosen.** Case folded first, bytes where that ties. `last` is exclusive and need not exist; `n` cuts
the list and `Link` says more remain; `n=0` is an empty list with no `Link`; a repository with no tags
lists none; one that does not exist is `404 NAME_UNKNOWN`.

**Rests on.** v1.1.1: "the tags MUST be in lexical order (i.e. case-insensitive alphanumeric order)";
`main` also allows "ASCIIbetical" order (research/02 [2]). A total order lets `last` page without
skipping or repeating a tag; case folding alone would tie `latest` and `Latest`.

**Falsified by.** A client that pages a tag list and misses or repeats a tag.

### H7. Uploads are bounded by what they stage

**Chosen.** A principal's open uploads hold at most the staging budget the run states; a request that
would stage past it (a chunk, a closing chunk, a blob in one `POST`) is refused
`429 TOOMANYREQUESTS` and changes nothing. A closing `PUT` whose content is not its digest's is
refused `DIGEST_INVALID`, ends the session and frees what it staged (AC-RQ1.3.3); a malformed one
(no `digest`, a malformed digest, a chunk out of order) leaves the session as it was.

**Rests on.** RQ1.3: sessions are "bounded by the bytes they stage, charged to who opened them";
CLAUDE.md §2: a bound with a typed refusal; RFC 6585 §4: 429 "indicates that the user has sent too
many requests", and its representation "MAY include a Retry-After header" (research/10 F19).

**Open.** A `429` an implementation gives under load (architecture D9) is no function of the model's
state. When RQ1 puts an implementation in the harness, such a refusal is recorded as an operation that
changed nothing, allowed in any state and counted, so that a run refused mostly by load fails its
coverage (research/10 F19, derived).

**Falsified by.** RQ1.3's staging bound refusing with another code.

### H8. Sessions are named by the request that opened them, and a mount reveals nothing

**Chosen.** A history names a session by the tag of the `POST` (or fallen-back mount) that opened it,
and the door maps the implementation's `Location` to that tag; so the model's answers do not depend on
the order in which sessions opened, which a concurrent history does not fix. A mount links the blob
(`201`) only from a repository the caller may read that links it; anything else, including a mount
with no `from`, opens a session (`202`) and says nothing of where the blob is.

**Rests on.** Architecture D18 and plan AC-RQ1.3.7; the conformance suite accepts `201` or `202` for a
mount and runs its automatic cross-mount case only when configured to.

**Falsified by.** A client that needs a mount without `from` to find the blob.

### H9. Byte ranges follow RFC 9110

**Chosen.** For a `GET` of a blob of length L with one range: reversed bounds make the range invalid,
and it is ignored (`200`, the whole blob); a first byte at or past L, or a suffix of zero, is
`416` with `bytes */L`; a suffix longer than the blob is the whole blob as `206`; a last byte past the
end reads to the end; a suffix of an empty blob is satisfiable yet no `Content-Range` can state it, so
the range is ignored (`200`). `HEAD` takes no range.

**Rests on.** RFC 9110 §14.1.1, §14.1.2, §14.2, §15.3.7.1 and §15.5.17 (research/10 F14 to F18): an
invalid range "MAY" be ignored or rejected and the RFC names no status for rejecting it; `416` is for
a valid range that is unsatisfiable.

**Falsified by.** A stock client that resumes a pull by range and fails on these answers.

### H10. The referrers API never answers 404

**Chosen.** Every manifest of the repository whose subject is the digest, listed by digest, each with
its `artifactType` (the manifest's, else an image manifest's config media type, else none for an
index) and annotations; filtered by `artifactType` when asked, saying so. A repository that does not
exist has no referrers: `200` with an empty list.

**Rests on.** "If the registry supports the referrers API, the registry MUST NOT return a `404 Not
Found` to a referrers API requests"; "If a query results in no matching referrers, an empty manifest
list MUST be returned" (v1.1.1, "Listing Referrers").

**Falsified by.** A conformance case that expects a `404` here.

### H11. Histories, plans and the generator are functions of a seed

**Chosen.** Every draw comes from hyper-sim's SplitMix64, each source on a stream of its own
(`stream_seed`), so a history replays from its seed alone and drawing more in one place changes
nothing elsewhere. A plan is data: each operation is invoked, reaches the registry (where it is
answered at once) or is lost on the way, and is answered or loses its answer; running it gives a
history on one logical clock. The generator builds each request on a shadow of the model, so valid
requests meet existing repositories, blobs, manifests, tags and sessions; one in the mix's stated
number is broken one of 29 ways, so that between them every refusal of `ANSWERS` occurs. Every figure
of a generation is its caller's (`Mix`, `Shape`, `Race`): the library states none.

**Rests on.** Architecture D21 (hyper-sim for every deterministic simulation); docs/design/numbers.md.

**Falsified by.** One seed giving two histories (T-RQ0.5.6).

### H12. Concurrent histories are judged by linearizability, on a counted budget

**Chosen.** A concurrent history is right when one order of its operations keeps real time and the
model, given the requests in that order, gives every answered operation the answer its client got. The
search is Wing and Gong's with Lowe's cache of (operations placed, state) configurations, the algorithm
Porcupine and Knossos's `wgl` implement, kept on an explicit stack. An operation whose answer was lost
may have taken effect anywhere after its invocation, or not at all: its completion never arrives, as
Knossos completes an unfinished call. The search counts what it explores against a budget its caller
states and says when the budget ran out (`Exhausted`), never a guess. When no order exists, the first
operation no order explains is found by a binary search over cuts at each completion, since a cut no
order explains stays unexplained as it moves later.

**Rests on.** Herlihy and Wing's definition and its locality (research/10 F1, F2); Wing and Gong's
search, Lowe's cache, and their implementations (F3, F4); the problem's NP-completeness and the
practice of bounding a search (F6, F7); pending and `:info` operations (F9 to F11).

**Falsified by.** A history the checker passes that a hand check refutes, or the reverse (the tests
build both by hand); the model run against itself reported (T-RQ0.5.2).

### H13. A failure is shrunk to a 1-minimal history of the same failure

**Chosen.** Runs of operations are removed while the failure stays, the runs halving, ending with a
pass that removes each single operation in turn and keeps none: what is left fails, and removing any
one of its operations makes the failure go. A smaller history counts only if it fails the same way: a
divergence at a request of the same kind with the same two answers (status and code), or a tangle
whose first unexplained operation is of the same kind. The shrinker runs on a budget of tries its
caller states and says whether it reached a 1-minimal history.

**Rests on.** Zeller and Hildebrandt's ddmin and its 1-minimality (research/10 F12); a test counts as
failing only if it "has produced the failure it was intended to capture" (F13).

**Falsified by.** A shrunk history that passes, or one from which an operation can be removed without
the failure going (the tests remove each in turn).

### H14. Coverage is counted, and a run that missed something fails

**Chosen.** A run counts requests by kind, answers by endpoint, status and code, codes, histories by how
many clients interleaved in them, and lost answers. `Coverage::gaps` lists every kind, code and row of
`ANSWERS` at zero, every asked-for interleaving that never happened, and every answer given that
`ANSWERS` does not list; a test fails on any (T-RQ0.5.5), and a run that never breaks a request is
shown failing it.

**Falsified by.** A refusal the generator never produces passing the coverage test.

## Answers

Every answer the model gives, by endpoint: a status, and the spec's code for a refusal. The test
`the_design_record_lists_the_answers_the_model_gives` holds this table to `ANSWERS` row for row, and
`every_kind_and_code_occurs_while_the_invariants_hold` holds the model to giving every row and no other.

<!-- answers:begin -->
| Endpoint | Answers | Request |
|---|---|---|
| end-1 | 200, 401 UNAUTHORIZED | `GET /v2/` |
| end-2 | 200, 206, 416, 400 NAME_INVALID, 400 DIGEST_INVALID, 400 UNSUPPORTED, 401 UNAUTHORIZED, 403 DENIED, 404 NAME_UNKNOWN, 404 BLOB_UNKNOWN | `GET`, `HEAD /v2/<name>/blobs/<digest>` |
| end-3 | 200, 400 NAME_INVALID, 400 DIGEST_INVALID, 400 UNSUPPORTED, 400 MANIFEST_INVALID, 401 UNAUTHORIZED, 403 DENIED, 404 NAME_UNKNOWN, 404 MANIFEST_UNKNOWN | `GET`, `HEAD /v2/<name>/manifests/<reference>` |
| end-4a | 202, 400 NAME_INVALID, 401 UNAUTHORIZED, 403 DENIED | `POST /v2/<name>/blobs/uploads/` |
| end-4b | 201, 400 NAME_INVALID, 400 DIGEST_INVALID, 400 UNSUPPORTED, 401 UNAUTHORIZED, 403 DENIED, 429 TOOMANYREQUESTS | `POST /v2/<name>/blobs/uploads/?digest=<digest>` |
| end-5 | 202, 400 BLOB_UPLOAD_INVALID, 400 SIZE_INVALID, 401 UNAUTHORIZED, 403 DENIED, 404 BLOB_UPLOAD_UNKNOWN, 416 BLOB_UPLOAD_INVALID, 429 TOOMANYREQUESTS | `PATCH <location>` |
| end-6 | 201, 400 DIGEST_INVALID, 400 UNSUPPORTED, 400 BLOB_UPLOAD_INVALID, 400 SIZE_INVALID, 401 UNAUTHORIZED, 403 DENIED, 404 BLOB_UPLOAD_UNKNOWN, 416 BLOB_UPLOAD_INVALID, 429 TOOMANYREQUESTS | `PUT <location>?digest=<digest>` |
| end-7 | 201, 400 NAME_INVALID, 400 DIGEST_INVALID, 400 UNSUPPORTED, 400 MANIFEST_INVALID, 400 MANIFEST_BLOB_UNKNOWN, 400 SIZE_INVALID, 401 UNAUTHORIZED, 403 DENIED, 413 MANIFEST_INVALID | `PUT /v2/<name>/manifests/<reference>` |
| end-8a | 200, 400 NAME_INVALID, 401 UNAUTHORIZED, 403 DENIED, 404 NAME_UNKNOWN | `GET /v2/<name>/tags/list` |
| end-8b | 200, 400 NAME_INVALID, 401 UNAUTHORIZED, 403 DENIED, 404 NAME_UNKNOWN | `GET /v2/<name>/tags/list?n=<integer>&last=<tagname>` |
| end-9 | 202, 400 NAME_INVALID, 400 DIGEST_INVALID, 400 UNSUPPORTED, 400 MANIFEST_INVALID, 401 UNAUTHORIZED, 403 DENIED, 404 NAME_UNKNOWN, 404 MANIFEST_UNKNOWN, 405 UNSUPPORTED | `DELETE /v2/<name>/manifests/<reference>` |
| end-10 | 202, 400 NAME_INVALID, 400 DIGEST_INVALID, 400 UNSUPPORTED, 401 UNAUTHORIZED, 403 DENIED, 404 NAME_UNKNOWN, 404 BLOB_UNKNOWN, 405 UNSUPPORTED | `DELETE /v2/<name>/blobs/<digest>` |
| end-11 | 201, 202, 400 NAME_INVALID, 400 DIGEST_INVALID, 400 UNSUPPORTED, 401 UNAUTHORIZED, 403 DENIED | `POST /v2/<name>/blobs/uploads/?mount=<digest>&from=<other_name>` |
| end-12a | 200, 400 NAME_INVALID, 400 DIGEST_INVALID, 400 UNSUPPORTED, 401 UNAUTHORIZED, 403 DENIED | `GET /v2/<name>/referrers/<digest>` |
| end-12b | 200, 400 NAME_INVALID, 400 DIGEST_INVALID, 400 UNSUPPORTED, 401 UNAUTHORIZED, 403 DENIED | `GET /v2/<name>/referrers/<digest>?artifactType=<artifactType>` |
| end-13 | 204, 401 UNAUTHORIZED, 403 DENIED, 404 BLOB_UPLOAD_UNKNOWN | `GET <location>` |
| end-14 | 204, 401 UNAUTHORIZED, 403 DENIED, 404 BLOB_UPLOAD_UNKNOWN | `DELETE <location>` (the spec's `main`) |
<!-- answers:end -->

### What the spec lists that Reliquary never answers

The test `the_answers_cover_every_endpoint_of_the_spec` holds this list to the code:

- `end-1 404`: Reliquary implements the spec.
- `end-4a 404`, `end-4b 404`, `end-7 404` and `end-11 404`: a repository is made by the first write into
  it, so a push never meets an unknown repository.
- `end-4b 202`: Reliquary takes a blob in one `POST`.
- `end-12a 404` and `end-12b 404`: the referrers API never answers `404` (H10).

And what Reliquary answers that the spec's table does not list, each with its decision: `400` for a
malformed request anywhere, `401` and `403` anywhere (H2), `429` where bytes are staged (H7), `206` and
`416` on a blob read (H9), and `416` on end-6 (H2).

## What the harness found planted

`tests/faults.rs` runs four registries that are the model with one fault each. Every one is reported,
by the comparison of sequential histories and by the linearizability check of concurrent ones, and
the model run against itself is reported by neither (T-RQ0.5.2, AC-RQ0.5.3):

| Fault | Shrunk to (1-minimal), on 2026-11-20 |
|---|---|
| A lost tag update: a move acknowledged, not made | 5 requests: a blob uploaded, a tag put on one manifest, moved to another, then read |
| A manifest accepted with a missing blob | the push itself, answered `201` where the model refuses `MANIFEST_BLOB_UNKNOWN` |
| A blob readable through a repository it was never pushed to | 3 requests: a blob pushed to one repository, another made, the blob read there |
| A non-linearizable tag read (a lagging replica answers every other read) | 7 operations: two blobs pushed, two writers' moves of the tag, and three reads, the last two disagreeing on which move came last; sequentially, 7 requests ending in a read that misses the tag just put |

A hunt reports the failing history's seed; `replay` (or `replay_concurrent` with the index) gives the
same failure at the same operation, and hunting again gives the same report (AC-RQ0.5.4, T-RQ0.5.6).
