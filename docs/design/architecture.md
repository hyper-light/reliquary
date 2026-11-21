# Reliquary's architecture

Status: design, 2026-11-20. Nothing here is built. Each decision says what it chose, what it rests
on (the notes in `docs/research/`, cited as "research/NN §x"), what would falsify it, and the plan
item that builds or re-decides it (`docs/design/plan/`). A decision marked "by data" names its
candidates; the item's A|B record picks one.

## 1. The shape

```
 Docker, containerd, Podman, ORAS,        Shards, Hex, kiln, reliquary CLI
 Helm, cosign, MCP clients, curl          (reliquary-client)
          │ HTTP/1.1, HTTP/2, TLS 1.3              │ hyper-transport lanes on hyper-quic,
          ▼                                        ▼ TLS 1.3, AES-256-GCM, mutual
   ┌──────────────┐                        ┌──────────────┐
   │  OCI door    │                        │ native door  │
   └──────┬───────┘                        └──────┬───────┘
          └────────────── commands ───────────────┘
                 ┌────────────────────────┐
                 │  core (no I/O)         │  names and digests · uploads · manifests, tags,
                 │                        │  referrers · artifact checks · leases and collection ·
                 │                        │  quotas, retention · facts and policy · catalog
                 │                        │  projection · cache and replication state machines
                 └───┬───────┬───────┬────┘
            Records  │ Bytes │Events │ Secrets, Upstream, Clock, Entropy     (seams, each with a simulator)
                     ▼       ▼       ▼
                 Amphora   Mantle   Quiver        Veil · other registries
```

One binary holds all of it. On a laptop Amphora's store and a Mantle cell of one run inside the
process. In a fleet the same binary runs on many machines against Amphora's and Mantle's clusters.
Across regions each region is a whole registry and repositories replicate between them.

A pull through the native door is one exchange and then bytes read straight from Mantle. A pull
through the OCI door is the distribution spec's requests, with each blob redirected to Mantle or
relayed, whichever is cheaper for that request.

## 2. Decisions

### D1. Two doors over one core

**Chosen.** The OCI door serves the distribution spec to every stock client. The native door serves
Reliquary's own protocol on hyper-transport lanes to Shards, Hex and kiln. Both are shells over one
sans-IO core; a request through either becomes the same command and gets the same refusal.

**Rests on.** Mantle has exactly this shape, an S3 door and a native QUIC door over one store
(research/01 §4). The person's direction: Shards uses "Mantle's hyper-quic protocol for artifact
upload and download as well as other operations" (research/01 [11]). An OCI pull costs a round trip
per manifest, config and layer, and most manifest requests are re-checks (research/04 F3, F4), so a
protocol that answers a whole pull in one exchange removes the dominant count.

**Falsified by.** A history that the two doors answer differently (RQ3.9), or a native pull that
does not beat the OCI door's on round trips and time at every image shape in the matrix (RQ3.3).

### D2. Records are rows, changed by conditional transactions, read through one change feed

**Chosen.** Everything Reliquary knows is rows in Amphora, in families: repository, blob, link,
upload, manifest, edge, tag, referrer, and later fact, decision, quota, candidate, holding. Each
operation is one conditional transaction (`Absent`, `At` a revision) or a counted series with a
stated state after each. "A manifest is taken only when what it references is there" is a condition
of the transaction that writes the manifest. Everything derived (caches, the catalog's index,
garbage candidates, replication, events on Quiver, the tag history) is built by following Amphora's
change feed.

**Rests on.** `amphora-api`'s `Records`: `commit(Tx)` with `conds` and `writes`, `Row.rev`,
`changes`, `wait` (research/01 §3). A value is at most Amphora's bound for one, and a transaction's
conditions are bounded, so a manifest's bytes past the bound live in Mantle and a manifest with more
references than one transaction holds commits in stages (RQ1.2).

**Falsified by.** TLC finding a reachable state that breaks an invariant of the model (RQ1.2); a
crash between two transactions of an operation leaving the registry wrong (RQ1.11); a hot row that
makes commit throughput fall with concurrency (RQ9.5).

### D3. A blob's bytes live under a key of their own; the digest lives in the records

**Chosen.** An upload streams to Mantle under a key made for that upload. On completion the blob's
row maps its digest to that key. A second upload of the same content loses the race for the row and
deletes its own object. The object carries its digest as metadata, so blob rows can be rebuilt from
Mantle if records are lost.

**Rests on.** The distribution spec's own statement that concurrent identical uploads both proceed
and the first to complete is kept (research/02 §1). Collection needs a blob to be deleted and
uploaded again without two writers sharing a key (D6). Kiln's Mantle store already keeps "a pointer,
naming its content's SHA-256 ... to bytes kept once" (research/01 §4).

**Falsified by.** A measurement showing the row's indirection costs a blob read more than its
interval (RQ4.5), or a rebuild drill that cannot restore the rows (RQ9.9).

### D4. Bytes do not pass through Reliquary where they need not

**Chosen.** On the OCI door a blob `GET` is answered by a redirect to a presigned URL on Mantle's
S3 door, or relayed; on the native door a client is handed a grant Mantle's native door honors, or
relayed. The choice is made per request from measured quantities: the blob's size, the path's
learned rate, and what a new connection costs that client. No size threshold is written down.

**Rests on.** Kiln's plan for the track: "pulls redirect to presigned URLs, so bytes go straight
from storage nodes" (research/01 §1). In the one public production trace, 95% of registry time went
to fetching from the backend store, and about 65% of requested layers were under 1 MB (research/04
F5, F6), and small layers took 0.008 s from memory in front of the store against 0.11 s by redirect
(research/03 F360): large blobs should bypass the registry, small ones should not pay a second
connection.

**Falsified by.** The rule losing to always-redirect or always-relay at any size in the matrix
(RQ4.5); a stock client in the matrix that cannot follow the redirect and is not caught (RQ4.5).

### D5. No node of Reliquary is special

**Chosen.** Reliquary's nodes elect nothing and lock nothing among themselves. Any node serves any
request and continues any upload from its record. Coordination is a conditional transaction.
Hints between nodes (who is filling a blob, who holds what) travel on hyper-swim and are only hints.

**Rests on.** Amphora and Mantle already replicate and elect; a second consensus above them would
add a failure mode and no guarantee. The master plan's bar: one code path from a laptop to regions
(research/01 §1).

**Falsified by.** The differential suite failing on three nodes with every hint dropped (RQ9.1).

### D6. Garbage is collected online, by three conditional steps

**Chosen.** A blob is live, condemned or removing. Condemn (only if no link exists), check the links
again and restore if one appeared, then move to removing and only then delete the bytes and the row.
A push conditions its link on the blob's row being live at the revision it read; it restores a
condemned blob and uploads afresh past a removing one. Candidates come from the change feed into a
review queue; a candidate is reviewed after a horizon learned per repository from the gaps between
a blob's assertion and the manifest that used it. A read of a condemned blob raises an alarm and
restores it. No pause, no read-only mode, no collector node.

**Rests on.** Existing registries either stop the world or pick a window: Distribution's collector
needs read-only mode, Harbor reserves two hours, zot one hour, GitLab one day, containerd's lease 24
hours, and GitLab left the existence-check race open by decision (research/05 F1 to F12). Collectors
elsewhere have lost data: Distribution's untagged sweep removed multi-arch manifests, zot's deleted
blobs pushed a second earlier, and GitLab's can delete layers of an interrupted push (research/03
F019, F120, F244). One Rust registry's design states the same proof obligations this protocol
meets, a verdict stamped with a revision and a guard per digest (research/03 F354). AWS Lambda's
image store collects by generations and keeps retired data in an alarmed "expired" state before
deleting it (research/04 F49). Per-key conditions are what Amphora gives (research/01 §3).

**Falsified by.** A generated history in which a referenced blob is collected (AC-T9.7.4, RQ9.3); a
failed push or pull in the drill (AC-T9.7.3); a stock client's push refused because the horizon was
too short (RQ9.4).

### D7. Every number has a kind, and a gate refuses the rest

**Chosen.** `docs/design/numbers.md`. Mantle's inventory, kinds and checker, with an empty open list
(`docs/design/constants.md`). The checker reads more than Mantle's: every literal but 0 and 1, wherever
it stands. A `learned` row states what it measures, on what, where it is kept and how it follows a change.
A `derived` or `counted` row names the test that computes or pins its value. Vendored code is read only
where Reliquary answers for its numbers.

**Rests on.** The person's rule, as Mantle, Shards and Hex already hold it (research/01 §5). An
estimator's figures stand in its arithmetic, where Mantle's checker does not yet read (its own words:
"Comparisons and other call arguments are not read yet"), and RQ0.8 holds each of them to a row
(AC-RQ0.8.6). Other products are asked, not edited (plan README, principle 6), and vendored code
Reliquary changes follows Reliquary's rules (CLAUDE.md §1).

**Falsified by.** A production number without a row of an accepted kind passing the gate (RQ0.2).

### D8. A wait follows liveness and progress, then the slowest answer seen

**Chosen.** No timeout is configured or compiled in. At a cold start, and after a wait runs out, a
wait lasts while the other party shows liveness and progress. After that its bound is the slowest
answer of its kind that path has shown. Connection and handshake times use RFC 6298's estimator.

**Rests on.** The person's rule of 2026-11-13, as Shards' registry client implements it; Hex's
"waits are learned per (call kind, class)"; Mantle's client (research/01 §5).

**Falsified by.** A healthy request cut, or a dead peer waited on past its liveness, in RQ0.8's
simulations of stationary, stepped, drifting and two-state paths.

### D9. Each bottleneck admits what it can finish, and learns how much that is

**Chosen.** One admission authority per bottleneck (request shards, records, bytes, checks). Its
limit is the point of greatest power, found by probing as Mantle's node finds it. Memory is charged
per operation at its own size. What is refused is refused before it costs anything, typed, with a
`Retry-After` computed from measured drain.

**Rests on.** Mantle node design §2.8, "No fixed counts" (research/01 §5). The distribution spec's
next version asks that `429` carry `Retry-After` (research/02 §2).

**Falsified by.** Goodput falling as offered load rises past capacity (RQ4.8).

### D10. An artifact is checked when pushed, by a pure function of its bytes

**Chosen.** A manifest of a known kind becomes visible only after its kind's checks pass. A check
reads the config and layers as streams, charges what it unpacks to the pusher's quota, and records
its verdict under the content's digest and the checker's version. Tar, gzip, zstd, the Agentfile
parser and the skill validator are Shards' own code, vendored, so Reliquary and Shards cannot
disagree. A kind Reliquary does not know is stored unchecked.

**Rests on.** AGENTFILE_ARCH §12.17 and Shards' crates (research/01 §7). The image spec: an
implementation "MUST NOT error on encountering an `artifactType` that is unknown" (research/02 §3).

**Falsified by.** Any §12.17 refusal case accepted (AC-T9.1.3); a verdict that differs from Shards'
on the shared corpus (RQ2.3).

### D11. What is read whole is stored whole; what is read by range is stored by chunk

**Chosen (by data).** OCI layers stay single blobs. MicroVM disks, flattened images and binaries
across releases are stored as recipes of chunks, each chunk once. Chunk size and whether cuts are
fixed or content-defined are parameters of the recipe, chosen per kind of content from RQ10.1's
measurements.

**Rests on.** Deduplicating inside a registry and rebuilding layers on pull slowed pulls by up to
98 times in the literature; layer sharing alone deduplicates little (about 90% of Docker Hub layers
belong to one image); block-level stores report large savings (Lambda: about 80% of uploads add no
new chunk); and every deployed chunk size is a judgement no source derives (research/04 F10, F14,
F47, F68).

**Falsified by.** RQ10.1 finding no sharing worth its index on Reliquary's corpora; a whole read of
a recipe-stored artifact missing Phase 4's bar with no hot whole copy (RQ10.3).

### D12. Any range can be verified alone

**Chosen.** A hash tree over an artifact's blocks, computed in the pass that hashes the upload, kept
as a referrer and bound to the blob's digest in its record, its tag's signed statement and the tag
history. The fetch size is not part of the format. Linux verity's tree shape where measurements
allow, so a kernel can check the same image.

**Rests on.** Kraken and Dragonfly check pieces with CRC32 only; BitTorrent v2 and BLAKE3 show
per-block proofs against a root (research/04 F70, F75, F78, F81). containerd's EROFS snapshotter
uses dm-verity per layer (research/04 F45).

**Falsified by.** A flipped bit accepted anywhere (RQ10.2); a tree that costs a second read of the
upload (RQ10.2).

### D13. Lazy loading happens at the block device, inside the VMM

**Chosen (by data, with Shards' and Hex's owners).** The node's image store presents a disk to the
VMM as a block backend in the VMM's own process; missing blocks are fetched, verified, stored and
answered. NBD serves a VMM that cannot link it. On Linux hosts, containers use the kernel's own
mechanism where it has one.

**Rests on.** FUSE, fscache, fanotify, ublk, TCMU, userfaultfd and vhost-user are Linux host
mechanisms; Hypervisor.framework and the Windows Hypervisor Platform leave devices to the VMM
(research/04 F61). Lambda serves block chunks behind virtio-blk and is replacing FUSE for its jitter
(research/04 F46, F52). Lazy loading can lose to a full pull on slow links without a prefetched hot
set (research/04 F22, F37), which is why working sets (RQ10.5) are part of the design and not an
extra.

**Falsified by.** AC-T9.6.2 unmet on fedora; steady-state reads slower than a full pull's
(RQ10.4); a platform where the backend cannot be linked and NBD loses to a full pull.

### D14. Peers are verified per piece, and the origin is always there

**Chosen (by data).** Nodes fetch blocks from nodes that hold them, over hyper-quic, each piece
verified by D12 before use or forwarding. Reliquary tells a node who holds what from the holdings
reports. Tree or mesh is decided in the lab per kind of transfer.

**Rests on.** Trees for in-order block needs (DADI, FaaSNet) against swarms (Kraken, Dragonfly);
no system publishes origin bytes for N pullers; Kraken's own caveat that P2P does not help when the
registry is not the bottleneck (research/04 F41, F53, F74 to F79, F83).

**Falsified by.** AC-T9.6.3 unmet; a corrupted piece used (T-T9.6.1).

### D15. Trust is facts computed once, decided ahead, and checked again by the client

**Chosen.** A signature, SBOM or provenance is verified when it is attached, and the result is a
fact about its subject. Policies (Rego) are decided when a fact or the policy changes; a pull does a
lookup. Clients that launch things (`reliquary-client` in kiln, Shards, Hex) verify on their own
machine as well.

**Rests on.** Shards' Sigstore verifier, TUF client and Rego engine, each held to its Go reference
(research/01 §7). Kiln's criteria AC-T9.4.1 to AC-T9.4.3. Harbor's enforcement at pull checks that a
signature is attached, not that it verifies (research/03 F080).

**Falsified by.** A planted case accepted (AC-T9.4.2); Rego running in a pull's path (RQ7.4).

### D16. A tag carries a signed, expiring statement, and its history can be verified

**Chosen.** Per namespace, TUF metadata of Reliquary's own says which digest each tag names;
clients refuse an older answer. Every tag change is an entry in a tiled hash-tree log with signed
checkpoints. Signing is a lease through the `Secrets` seam; Reliquary holds no key.

**Rests on.** Kiln's T-T9.4.1 (a rollback with old TUF metadata is refused); the plan's "TUF for
the freshness of metadata" (research/01 §1). Research/02 §8 for what TUF defends against and the tiled
log format.

**Falsified by.** A rolled-back answer accepted by a verifying client (RQ7.3); a tag visible to a
verifying client later than to a plain one beyond the interval (RQ7.3).

### D17. A cache checks a tag on each pull unless told how stale it may be, and never moves a tag ahead of its manifest

**Chosen.** Digests are kept for good. A tag is revalidated against its upstream by the upstream's
cheapest request, concurrent pulls sharing one check, unless the operator states a freshness window.
The cached tag moves only after the new manifest, and an index's members, are stored. Request
budgets come from each upstream's own headers and `429` answers.

**Rests on.** Kiln's T-T9.5.1; the spec's `ns` parameter and `OCI-Namespace` (research/02 §2).
Docker Hub states its limit in response headers, counts a manifest `GET` as a pull and does not count
a `HEAD`; ECR's cache revalidates on a fixed 24-hour window (research/05 F31 to F36). Distribution's
cache sends every concurrent miss upstream (research/03 F027).

**Falsified by.** A pull handed one image's manifest with another's parts (RQ8.3); a rate-limit
failure in CI's week (AC-T9.5.2).

### D18. A namespace is charged for what it links, and learns nothing about others

**Chosen.** Quota is the logical size of everything a namespace links, shared or not, charged in the
link's own transaction. A mount succeeds only where the caller may read the source; a mount without
a source never reveals that another tenant holds the blob. Upload time and answers do not depend on
whether content already exists.

**Rests on.** Cross-user deduplication is a side channel, and a mount's `201` against `202` is an
existence oracle unless read access is checked (research/04 F71 to F73).

**Falsified by.** A timing or answer difference outside the interval in RQ1.3, RQ1.4, RQ10.3 or the
security review (RQ13.4).

### D19. A repository is homed unless its operator says otherwise

**Chosen.** Homed: one region orders a repository's writes and the others forward to it. Multi-
writer, when stated: every region accepts writes and the later wins by hybrid logical clock, ties by
a fixed order of regions. Immutable tags require a home. A region serves a tag only when it can
produce the tag's whole closure, held or fetched through.

**Rests on.** Kiln's T-T9.9.1 asks for a stated rule. Hybrid logical clocks stay bounded from
physical time; a last-writer-wins register converges when timestamps are unique and totally ordered;
hosted registries document eventual consistency and no winner rule (research/05 F17 to F24).

**Falsified by.** Two regions ending at different digests in any interleaving (RQ12.4); a region
serving a manifest whose blobs it can neither hold nor fetch (RQ12.3).

### D20. The catalog's index is derived, per node, and can always be rebuilt

**Chosen (by data).** Each node follows the change feed and keeps its own index; a query can ask to
see everything before a point it names. The index's structure is picked by RQ6.2's benchmark.

**Rests on.** Kiln's AC-T9.3.2 and AC-T9.3.3. D2's single feed.

**Falsified by.** An artifact missing one second after publish; a rebuilt index answering
differently from the incremental one (RQ6.2).

### D21. The house's runtime, codecs and simulator

**Chosen.** hyper-rt's thread-per-core shards; Mantle's HTTP/1.1 codec and Shards' HTTP/2;
hyper-tls and hyper-quic; hyper-sim for every deterministic simulation; Mantle's node on a shard of
its own.

**Rests on.** Research/01 §6: each is sans-IO, bounded and already fuzzed or held to an oracle, and
kiln holds one revision of each.

**Falsified by.** A door that cannot meet Phase 4's bar on these with the cost table showing the
runtime or a codec as the cause (RQ4.2).

### D22. Skills and MCP servers: the metadata and the bytes in one place

**Chosen.** A skill is an artifact in the shape Shards gives agents, proposed to Shards' owner. An
MCP server is a `server.json` validated against the official schema and against the artifact it
points at when that artifact is here. Reliquary serves the MCP Registry API and can follow the
official registry as a subregistry.

**Rests on.** Research/02 §5 and §6: MCP registries are metaregistries that point elsewhere for
bytes, and expect subregistries; the Agent Skills format's rules; Shards names no skill type
(research/01 §7).

**Falsified by.** An MCP client that cannot use Reliquary through the API unchanged (AC-T9.2.2); an
invalid skill accepted (AC-T9.2.3).

### D23. Measurement follows Mantle's design, and every choice with candidates is an A|B record

**Chosen.** `reliquary bench`: rounds of counted operations, order in time tested before any
interval, one state or two, medians with order-statistic intervals, open-loop load with intended
start times, records that are "of record" only under the load rule. A|B runs interleave two builds
in one session and give a verdict only when intervals separate.

**Rests on.** Mantle's `docs/design/measurement.md` (research/01 §5); the master plan's bar on
recorded benchmarks.

**Falsified by.** An A|A run reporting a difference more often than its stated level (RQ0.7).

### D24. Crates

**Chosen.** `reliquary-api` (the trait others reach Reliquary through, its types, a simulator and a
conformance suite), `reliquary-core` (the sans-IO cores and the seams Reliquary reaches others
through), `reliquary-wire` (the native protocol's messages and golden vectors), `reliquary-client`
(what Shards, Hex and kiln link), `reliquary-node` (the doors and the shells of the seams),
`reliquary-sim`, `reliquary-bench`, and the `reliquary` binary. A crate appears with its first
code, not before.

**Rests on.** CLAUDE.md §9 and §13; Hex's layout (research/01 §8), which lets items of one phase be
built side by side without sharing files.

**Falsified by.** A crate with one consumer and no seam of its own after its phase.

### D25. Identity is proved, not stored, wherever it can be

**Chosen.** A workload presents its own OpenID Connect token and receives a registry token for the
grants a stated rule gives it, living no longer than the token it came from; the audience is always
checked, and a rule may match any claim, so a repository is named by an ID that cannot be re-used. A
client certificate under a stated authority does the same on the OCI door. Where a machine cannot
prove itself it gets a credential that must expire. People sign in through the operator's OpenID
Connect issuer by its device flow. Reliquary does not speak LDAP: a directory is put behind an
issuer, so there is one sign-in protocol and one parser to defend.

**Rests on.** zot's workload identity, Quay's exchange of a token for a robot token that lasts an
hour, angos' and nora's keyless CI (research/03 F136, F198, F320, F336); Quay's own statement that
tokens which do not expire are a hazard (F198); GitHub's change of its subject to immutable IDs for
repositories created after 2026-07-15 (research/06 F19); the OCI's auth working group having
published no specification (research/06 F20), which keeps the token flow the compatibility target.

**Falsified by.** A token accepted without its audience checked, or outliving the token it was
exchanged for (RQ7.11); a credential made with no expiry (RQ7.12); an operator who can show a
directory that cannot be put behind an issuer.

### D26. An audit record rides in the change's own commit; events come from the change feed

**Chosen.** Every change carries its audit record in the same transaction, so neither exists
without the other and a pull writes nothing. Events for webhooks, kiln's topics and the native
door's event lane are read from the records' change feed through a durable cursor per subscriber:
at least once, in commit order within a repository, with nothing held only in memory. A subscriber
that falls behind the feed's history is told so and where to list again from.

**Rests on.** Distribution's notifications, which its documents call best effort and lost with the
instance, and GitLab's, which drop events once a queue of 3000 is full (research/03 F037, F252);
Harbor's write to its database on every pull, with switches to stop it labelled temporary, and
Quay's audit logging that multiplies log volume about 2.5 to 3 times (research/03 F096, F214);
CLAUDE.md §3: nothing is dropped silently.

**Falsified by.** A committed change with no audit record or no event delivered (RQ7.13, RQ7.14);
a write on a pull's path in a namespace that does not audit pulls (RQ7.14); an event lost with a
node (RQ7.13).

### D27. Both families of signature are facts; the registry signs only what it knows; scanning stays outside

**Chosen.** Notary Project signatures are verified beside Sigstore's and recorded as the same kind
of fact, so one policy can ask for either. Where an operator's rule says so, Reliquary signs what it
accepts, in the same commit, with a key it never holds; that signature says only that the registry
accepted the artifact from a principal under a rule, and its fact is marked as the registry's own.
Reliquary does not scan images: a scanner that speaks Harbor's adapter API is attached by its
address, given a credential for one artifact, and its report becomes facts. A verifier accepts only
the formats it fully checks; an unknown bundle version is "unverifiable", never failed or verified.

**Rests on.** zot verifies cosign and notation signatures but does not enforce them, and Harbor
enforces presence only (research/03 F144, F080); ECR's signing at push, which it recommends
(research/06 F15); Harbor's pluggable scanners, and a user's report of a scan that took more than 50
hours (research/03 F079); cosign's two storage forms, Rekor's two generations and the announced
post-quantum bundle revision (research/06 F3 to F7); the public log's own removal of entry types it
cannot justify parsing (research/06 F8).

**Falsified by.** A Notary signature notation accepts recorded as failed, or the reverse (RQ7.9); an
artifact under a signing rule visible unsigned (RQ7.10); a push slowed by a scanner, or an artifact
recorded clean after a failed scan (RQ7.15).

### D28. Learning is not assumed to win

**Chosen.** A learned value replaces a fixed one only where an A|B on recorded traces shows it no
worse than the best fixed rule a cited source measured. Where it is not, the cited rule stands, as
a `cited` row that names the range over which its source found the result insensitive, and that
range is checked again on Reliquary's own traces. Every estimator states its objective, the range
its output cannot leave, and what it does when its feedback stops or contradicts itself.

**Rests on.** S3-FIFO's authors found their adaptive variant worse than a fixed share on most of
6594 traces, and the result insensitive between 5% and 20% (research/06 F43); ARC's adaptation step
is an implicit parameter (F46); Meta's Spiral warns that wrong feedback teaches wrong decisions
(F53); CoDel's pattern, one measured quantity with its target a cited fraction of it (F40). Our
reading of the person's rule, which is theirs to confirm: it refuses arbitrary constants and
constants learned on one machine, not a value a primary source measured across workloads and that
our own records confirm.

**Falsified by.** A fixed share kept where our traces show the result sensitive to it (RQ4.4); an
estimator adopted without its A|B against the cited rule (RQ0.8); an estimator whose output left its
stated range (RQ0.8).

## 3. What a deployment states, what a node measures, what it derives

| Stated by the operator | Measured by the node | Derived by the node |
|---|---|---|
| where to listen; where data lives | cores granted; memory the OS allows | every queue bound, window, budget and cache size |
| who may do what (grants, policies); which issuers, audiences and claims earn which grants; trust stores; signing rules | its devices, through Mantle's probe | shard counts; admission floors; the manifest bound |
| failures to survive; regions and replication rules | each peer's and path's round trips, rates and waits | every wait; transfer concurrency; fetch sizes |
| upstreams to cache or mirror; freshness, if any | each upstream's stated limits and answers | request budgets; revalidation pacing |
| quotas and retention rules; who is told of which changes; how long the audit trail is kept and whether pulls are in it | what each start reads; each repository's push gaps | working sets; the collection horizon |
| token, credential and statement lifetimes, if not the specifications' defaults | per-operation costs (`reliquary bench`) | grant lifetimes; `Retry-After`; capacity predictions |
| optionally, a memory ceiling | | |

Nothing else is configurable, and nothing in the right-hand columns is a constant.
