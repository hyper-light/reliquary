# 01. Internal ground: what Reliquary is built on, and what it must serve

Status: research note, 2026-11-20. Sources [1] to [12] are hyper-light repositories and kiln's plans,
read on 2026-11-20 at the revisions named. Every claim carries its source; a conclusion that is
derived says so and from what. Quotes are verbatim.

## Sources

[1] kiln plan `01a157fe104d7444a38c808b89f2fae0`, "Critical path T9: Reliquary", version 1 (read
through `plans_get`).

[2] kiln plan `01a157fddb8276a2996246ea931c4808`, "Critical path, master", version 4: §3 the bar, §5
tracks and vendoring, §6 landing, §8 standing rules.

[3] kiln at `ed8991d29` (origin/main, 2026-11-20): `crates/kiln/src/ci/registry/{mod,auth,mirror,proxy}.rs`
(2,597 lines), `crates/kiln-seams/src/{artifacts,blobs}.rs`, `docs/mantle.md`, `docs/goals.md`,
`docs/plans.md`, `vendor/MANIFEST.toml`.

[4] mantle at `f41d8344` (origin/perf-10k, 2026-11-20; main is `87ab09e1`): `CLAUDE.md`,
`docs/STATUS.md`, `docs/design/{constants,measurement,doors,gateway,node}.md`,
`docs/research/87-no-fixed-counts.md`, `vendor/hyper-*`.

[5] shards at `7158020b61` (origin/main, 2026-11-19): `docs/architecture/AGENTFILE_ARCH.md`,
`docs/design/architecture.md` (D116, D180, D222, D226, D270), `crates/{registry,remote,image,build,erofs,sigstore,tuf,rego}`.

[6] hex at `cfe430167` (origin/main, 2026-11-18): `docs/design/{README,hex-quic,shards-image-class,scheduler-architecture}.md`,
`docs/requirements.toml`, `STATUS.md`.

[7] amphora at `db991a05` (origin/main, 2026-11-07): `README.md`, `STATUS.md`, `crates/api/src/lib.rs`.

[8] quiver at `966ea139` (origin/main, 2026-11-12): `crates/quiver-api/src/lib.rs`.

[9] veil at `c474fc9` (2026-10-20): `README.md`.

[10] hyper-raft (crates as mantle vendors them at [4]): crate-level docs of `hyper-rt`,
`hyper-rt-transport`, `hyper-transport`, `hyper-quic`, `hyper-tls`, `hyper-sim`, `hyper-measure`,
`hyper-timing`, `hyper-liveness`, `hyper-seal`, `hyper-durable`, `hyper-log`, `hyper-swim`.

[11] The person's directions in the conversation that opened this plan (2026-11-20): Reliquary
"must satisfy the single binary, laptop to globally distributed agentic Meta scale requirement";
"We DO NOT TOLERATE ARBITRARY CONSTANTS"; a data-derived value "learned from a specific machine that
[is] effectively a constant" does not count; "We do NOT assume hardware or underlying machine";
Windows, macOS and Linux on amd64 and arm64 "excepting Intel MacOS"; "you are to interface with the
Hex runtime"; "For shards, you should utilize Mantle's hyper-quic protocol for artifact upload and
download as well as other operations."

[12] reliquary at `fc383a2` (main): `CLAUDE.md`, `docs/requirements.toml`, `docs/design/README.md`.

## 1. What the track asks

- Reliquary is "the registry for: OCI images; Shards and Firecracker microVM images; every artifact
  the Agentfile spec names: agents, harnesses, skills and MCP servers" [1 §0].
- "Its records (manifests, tags, repositories, referrers, the catalog, signatures and policies) in
  Amphora. Its bytes (layers, artifacts and images) on Mantle, through the `Blobs` seam" [1 §0].
  Events go on Quiver and upstream-registry credentials come from Veil [1 §0].
- Mantle's fit, as the plan states it: "content-addressed blobs are created once, with
  `If-None-Match: *`; resumable pushes map to multipart uploads; pulls redirect to presigned URLs,
  so bytes go straight from storage nodes; every read is verified, and repaired from another copy"
  [1 §0].
- "Reliquary starts from kiln's own registry (`ci/registry`), which moves into Reliquary's
  repository. Kiln vendors it back at `vendor/reliquary` and reaches it only through kiln-seams'
  `Artifacts`" [1 §0].
- Ten goals, CP-T9.1 to CP-T9.10, each with numbered criteria and tests [1 §3]; `docs/requirements.toml`
  carries them [12].
- The bar every goal meets [2 §3]: usable at once; no panics; bounded work; typed refusals; tests
  that exercise behavior; recorded benchmarks; "Laptop equals fleet equals multi-region: one code
  path"; seams with simulators; observability; adversarial verification by a fresh subagent.
- A track agent "runs at most 2 [subagents] at once, each with a tight, written brief" [2 §5].

## 2. Kiln's registry, the starting point

- It serves "the OCI Distribution Specification v1.1 at `/v2/`", blobs in kiln's content-addressed
  store, monolithic and chunked uploads, cross-repository mounts, manifests "in SQLite, by digest
  and tag", the referrers API, deletes and a collector, auth, mirrors and a proxy [3 mod.rs:1-12].
- It is 2,597 lines in four files, written against kiln's `Hub`, `rusqlite`, `anyhow` and kiln's own
  HTTP server [3]. It is a behavior baseline and a migration source (T-T9.1.1), not code that meets
  Reliquary's bar as it stands (derived: `anyhow` and `rusqlite` are not Reliquary's seams).
- Two numbers in it are set by hand: `MAX_MANIFEST: usize = 4 << 20` and `UPLOAD_TTL: i64 = 24 * 3600`
  ("Uploads untouched this long are dropped") [3 mod.rs:34-36]. The first has a specification
  behind it (the distribution spec's 4 megabytes); the second is picked.
- It "knows nothing of agents, harnesses, skills or MCP servers", has "no catalog or search beyond
  `_catalog`", "no policy at pull", and keeps "one copy of every byte, on the server's disk" [2 §1.7-8].

## 3. The seams Reliquary is reached through, and reaches others through

- **kiln-seams `Artifacts`** [3 artifacts.rs]: `push_blob`, `blob`, `push_manifest`, `manifest`,
  `tags`, `delete_manifest`, `referrers`, with `Refusal` variants named for the distribution spec's
  error codes, a simulator `SimArtifacts` and a conformance suite. "Reliquary is the real one
  (CP-T9.1, which passes the OCI conformance suite)". It carries whole bodies as `Vec<u8>`; a
  registry's own seam must stream (derived).
- **kiln-seams `Blobs`** [3 blobs.rs:1-13]: "bytes, with S3's semantics (CP-T10.1): put; get, with
  byte ranges; head, list and delete; conditional writes (`If-None-Match: *`, `If-Match`);
  multipart; checksums; presigned URLs". "An object's etag is its content's SHA-256, however it was
  uploaded."
- **amphora-api `Records`** [7 lib.rs:177-199]: `get`, `scan`, `commit(Tx)`, `changes`, `wait`,
  `head`, `compact`; a `Tx` is `conds: Vec<Cond>` and `writes: Vec<Write>`; every row has a `Rev`;
  families of keys; a change feed. "Methods block until their deadline." "The embedded
  implementation currently promises local disk durability, not a replicated quorum." A `sql`
  module carries statements and migrations.
- **quiver-api `Topics`** [8]: `create`, `delete`, `publish`, `read`, `wait`, `bounds`,
  `subscribe`; classes ephemeral, replicated, persisted and windowed; a simulator and a conformance
  suite.
- **Veil** has no `-api` crate yet: "the `veil` binary says what it is, and nothing more yet" [9].
- Products "reach each other only through their `-api` crates", each trait with "a simulator
  implementation" [2 §3].

## 4. Mantle: where the bytes go, and how

- Two doors, one store [4 STATUS]: "*The S3 HTTP/1.1 door*: S3's object, bucket and multipart
  operations served by each node's listener ... over mantle's own I/O-free, bounded, fuzzed HTTP/1.1
  codec"; "*The native QUIC door*: the `mantle-client` crate and `mantle`'s object commands, for
  other hyper-light projects, streaming any object in bounded memory, uploads of any size in runs
  the node cuts, resumable".
- The native client [4 gateway.md §6]: "One connection per gateway carries all of a client's
  transfers"; "It takes the server's plan. Upload part size, run interval and window come from the
  gateway"; a journal of open operations; every mutation "carries its operation's identity, 128
  bits from the OS"; a put "states its CRC-64/NVME in a trailer"; a get "is held to a full-object
  checksum as its bytes arrive".
- kiln already reaches a Mantle cell this way: "`kiln::blobs::mantle::MantleBlobs`, kiln-seams'
  `Blobs` on the vendored `mantle-client`, on hyper-rt"; "a key's object is a pointer, naming its
  content's SHA-256, its size and its bytes' native ETag, to bytes kept once by their SHA-256"
  [3 mantle.md, "What it means for kiln"].
- A node inside another process is asked for and not yet a library: "The library entry point isn't
  written down"; "A node and the client in one process: nothing in the design forbids it; the client
  would reach its own node through the loopback peer, not QUIC"; "A node needs a shard of its own"
  [3 mantle.md R1, R3]. The laptop path is designed: "a loopback peer that skips the socket, TLS and
  serialization, and keeps the rest" [4 node.md §5.5].
- Its gateway "cuts 8 MiB blocks" [3 mantle.md, "Agreed next"]; a block is erasure-coded into chunks
  on distinct disks or nodes, the scheme "chosen from the failure domains available and a durability
  target, S3's eleven nines by default" [4 STATUS, "Erasure coding"].
- Supported targets: "Linux and Windows on x86_64 and arm64, macOS on arm64" [4 STATUS:3-5], which
  is the set [11] asks of Reliquary.

## 5. The rule on numbers, as the house already holds it

- Mantle's inventory: "Every number mantle's production code runs with, and every number of the
  vendored crates on its write path, has a row here: its kind, the value the code has, and what
  establishes it" [4 constants.md:3-5]. The owner's rule: "We have a no ARBITRARY VALUES rule.
  Learned and derived approaches or exceedingly well researched ONLY." [4 constants.md:5-6].
- Accepted kinds [4 constants.md:11-22]: **format**, **external**, **derived**, **cited**,
  **learned**, **counted**. Refused kinds [4 constants.md:26-32]: **picked**, **knee** ("read off
  one measured curve by a chosen criterion and frozen"), **once** ("measured once, on one
  machine"), **copied** ("another system's tuning"), **guessed**, **inherited**, **open**.
- It is a gate: "`scripts/check-contracts.py` reads every `const` and `static` holding a number ...
  It fails when a number has no row, when a row names no number, when a row's value is not the
  code's, and when a row of an accepted kind states a basis the rule refuses";
  "`scripts/arbitrary-open.txt` lists the open items and only shrinks" [4 constants.md:34-45].
- Counts in admission and batching are learned: "The operations a node admits: learned. The limit
  is the point of greatest power, Kleinrock's throughput over delay ... The node finds it as BBR
  probes its pipe, from above" [4 node.md §2.8].
- Waits are learned. Shards: "How long shards waits on a registry, learned from that registry's own
  path as shards runs (D222), never a figure measured on one network and frozen"; the person's rule
  (2026-11-13): "The cold start, and every wait after a run-out, follows the party's liveness and
  progress. After that the bound is the slowest answer seen." [5 crates/registry/src/patience.rs:1-9].
  Hex: "Waits are learned per (call kind, class), never set by constants" [6 hex-quic.md:6-7].
- Transfer shape is learned. Shards D226: "Pushes and pulls learn their network path: as many
  transfers at once as pay, chunks of the size that costs least a byte"; its first version "still
  chose three figures ... the user refused them as encoding our own network: every value is now a
  protocol's, measured on the path as shards runs, or an estimator's of what was measured"
  [5 architecture.md D226].
- Measurement has a design [4 measurement.md]: a point is measured in rounds; rounds are checked for
  order in time (lag-1 autocorrelation and a runs test) before any interval is stated; one state or
  two by Hartigan's dip test; each state's median with its order-statistic interval; "a round is a
  count of transfers, not a length of time"; "No mean or coefficient of variation is reported".
- What a deployment states, what a node measures, and what it derives are separate lists
  [4 node.md §1.4]: stated are roles, devices, addresses, failures to survive and a durability
  target; measured are devices, memory allowed, granted cores, peers' round trips; derived is
  "every queue bound, window and budget".

## 6. The shared runtime and transport

- `hyper-rt`: "a thread-per-core executor with arena task slots ... a hierarchical timing wheel,
  per-shard inbound rings with cross-shard wake kicks, the OS drivers behind one completion seam,
  and a deterministic simulation driver"; "A task belongs to one shard for its whole life; nothing
  is work-stolen" [10].
- `hyper-transport`: "The application layer over hyper-quic, sans-io ... one connection per peer,
  bidirectional exchanges of a request and a reply, classes with a credit reserve for the classes
  above, absolute credits, typed refusals, streaming bodies through reservations" [10].
- `hyper-quic` is quinn-proto's protocol logic and `hyper-tls` is rustls, both conformed and
  vendored; cryptography is AWS-LC through aws-lc-rs [10; 4 vendor/UPSTREAM.md].
- `hyper-sim`: "The world every deterministic simulation here runs in": named random streams,
  virtual time, a network with partitions and duplication, and a block device that loses power [10].
- `hyper-measure`: "every allocation, reallocation and free the process makes ... and the page
  faults the operating system charged it"; "no shipped crate depends on it" [10].
- `hyper-timing` and `hyper-liveness`: failure detection and election timing "derived from what its
  links and paths measure", "none is a picked multiple of another" [10].
- HTTP building blocks already in the house: mantle's HTTP/1.1 codec, "I/O-free, bounded, fuzzed"
  [4 STATUS]; shards' `http2`, "HTTP/2 (RFC 9113) without I/O ... with HPACK", whose every bound
  "is the protocol's ... or a value its caller derives from what it holds; none is chosen here"
  [5 crates/http2/src/lib.rs].
- One revision of each: kiln, Hex, Mantle and Shards hold one hyper-raft, by `[patch]`
  [3 vendor/MANIFEST.toml; 6 hex-quic.md Q1].

## 7. Shards: the artifacts and the client

- Artifact types [5 AGENTFILE_ARCH.md §12.17]: `application/vnd.osi.agent.v1`,
  `application/vnd.osi.harness.v1` and `application/vnd.osi.mcp.v1`, each with a JSON config
  (`application/vnd.osi.<kind>.config.v1+json`) and content layers
  (`application/vnd.osi.<kind>.content.v1.tar` "with `+zstd` or `+gzip` (or none), applied in order
  onto an empty directory").
- Content rule: layers "hold files and directories alone: no whiteouts, device nodes, FIFOs,
  sockets, set-ID bits or links leaving the directory" [5 §12.17].
- Config: `schemaVersion`, `name`, `version`, `description`, `platform`, `run`, `skills`, `mcp`,
  `asks` (`network`, `volumes`, `processes`, `memory`); "every path relative to the domain's
  directory, none may leave it" [5 §12.17].
- Refusals: "Readers refuse a `schemaVersion` they do not know, an unknown media type, a config
  whose `name` is no stage-style name, and any path in it that is absolute or leaves the directory"
  [5 §12.17].
- "Signatures and SBOMs are its OCI 1.1 referrers (D116)": a Sigstore bundle
  (`application/vnd.dev.sigstore.bundle.v0.3+json`) and `application/spdx+json` [5 §12.17].
- A microVM "as an OCI artifact": `application/vnd.shards.microvm.v1`, whose layers are its EROFS
  disk (`application/vnd.shards.erofs.v1`) and the image config it was made from
  (`application/vnd.shards.microvm.config.v1+json`) [5 crates/image/src/save.rs:297-301]. Shards
  writes EROFS "uncompressed, 4 KiB blocks, inline xattrs, and no feature flags", regular files as
  whole blocks "so guests map it straight from pmem with DAX" [5 crates/erofs/src/lib.rs].
- Skills are "checked as the format's reference validator checks it (agentskills skills-ref,
  `validate`...)" and sourced from "git repositories; OCI tarballs, decompressed to the output path;
  http(s) endpoints; local paths" [5 crates/build/src/skill.rs:1-7; AGENTFILE_ARCH.md §4.3]. Shards
  defines no `vnd.osi.skill` type: a skill's OCI form is Reliquary's to propose to Shards' owner
  (derived: `git grep vnd.osi` at [5] lists agent, harness and mcp alone).
- Components are attached "by digest" to running image VMs (D270), and Hex's node keeps them in
  "each node's Shards home" [5 D270; 6 shards-image-class.md §8].
- Shards already has: an OCI client with learned waits and learned transfers (`crates/registry`),
  QUIC to another home through hyper-quic (`crates/remote`, D180), Sigstore verification "as
  sigstore-go v1.2.2 verifies" (`crates/sigstore`), a TUF client "as go-tuf v2.4.2 runs it"
  (`crates/tuf`), and Rego "as OPA v1.14.1 evaluates it" (`crates/rego`) [5].

## 8. Hex: the runtime Reliquary feeds

- Hex "keeps its state in Amphora, its events on Quiver, its images in Reliquary, its secrets in
  Veil" [6 README.md:18]. Its open decision: "Storage: a CSI-like interface; images from Reliquary,
  lazily | CP-T7.8 | open" [6 README.md:75].
- An image box is "Pinned by digest (`NAME@sha256:…`...)", "because create records the box and
  start boots it: a tag that moved between them would boot another image than the one admitted";
  the image "is in the node's Shards home" [6 shards-image-class.md §1].
- Its scheduler prices transfers: "an image's layers from Reliquary, a Firecracker snapshot
  template or a volume from Slates moved to a node cost B4 their bytes over the pair's learned
  rate" [6 scheduler-architecture.md B10]. So Hex needs, of Reliquary, an artifact's bytes by
  digest and which of them a node already holds (derived).
- Its wire is the convention to follow: "TLS is AES-256-GCM only, and identity comes from kiln's CA
  through CSR enrollment, so keys never travel" on "hyper-transport lanes" [6 hex-quic.md:3-7]; the
  person's directive: "all such communications should be AESGCM256 bidirectional encrypted".

## 9. What follows for Reliquary (derived)

1. **Two doors over one core**, as Mantle has: the OCI door (HTTP/1.1 and HTTP/2, the distribution
   spec, for Docker, containerd, Podman, ORAS and every stock client) and the native door (a
   Reliquary protocol on hyper-transport lanes over hyper-quic, for Shards, Hex and kiln). From §4,
   §6, §8 and [11].
2. **Bytes never cross Reliquary where they need not**: the OCI door redirects a blob GET to
   Mantle's S3 door by a presigned URL; the native door hands the client a grant for Mantle's
   native door, so pushes and pulls are `mantle-client` transfers, resumable and checksummed. From
   §1 and §4.
3. **Records are Amphora rows committed by conditional transactions**, so "a manifest is taken only
   when what it references is there" is a `Cond` of the same `Tx` that writes the manifest, and
   garbage collection, replication, the catalog's index and Quiver's events all read one change
   feed. From §3.
4. **Every number has a row and a kind from the first commit**, with Mantle's gate; waits,
   concurrency, chunk and batch sizes, cache sizes and admission limits are learned or derived, on
   the machine and path that run, and re-learned as they change. From §5 and [11].
5. **Sans-IO cores on hyper-rt, proven in hyper-sim**, with the HTTP codecs the house already
   fuzzed. From §6.
6. **One revision of each shared crate**: Reliquary vendors hyper-raft, Mantle, Amphora, Quiver and
   Shards' crates at the revisions kiln holds. From §6 and [2 §5].
7. **What Reliquary must ask of others** (requests, not edits; [2 §5]): Mantle's in-process node
   (R1) and a grant its native door honors for one object; Shards' client taking
   `reliquary-client` for its native transport, and a skill artifact type; Hex's CP-T7.8 consuming
   Reliquary's node-side cache and "what a node holds" report; Veil's `-api`.
