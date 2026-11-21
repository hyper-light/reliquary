# Phase 2: Artifacts, checked on push

**Goal RQ2.** Reliquary knows what it is handed. An agent, a harness, an MCP server, a microVM, a
Firecracker pair, a tar or a zip is checked by its own rules as it is pushed, and one that breaks
them is refused before anyone can pull it. Kiln's registry moves here.

**Delivers.** The first thing no other registry does: `shards push` of a malformed agent fails at the
registry, with the rule and the path. Archives become browsable: one file can be read out of a tar or
a zip without pulling the rest. Kiln serves everything it served before, from Reliquary.

**Serves.** CP-T9.1 (AC-T9.1.1, AC-T9.1.3, T-T9.1.1). **Needs.** RQ1.

**Order.** RQ2.1 and RQ2.2 first, side by side. Then RQ2.3 to RQ2.8 side by side. RQ2.9 needs RQ2.3
and RQ2.4 and closes the phase.

---

### RQ2.1 The admission pipeline

Needs: RQ1 · Serves: AC-T9.1.3

When a manifest is pushed, find its kind from its `artifactType`, or its config's media type where
it has none, and run that kind's checks before the manifest becomes visible. A check is a pure
function of the manifest, its config and its layers' bytes, read from Mantle as a stream. Its verdict
is kept under the content's digest and the checker's version, so the same bytes are never checked
twice. A check's work is charged before it runs: compressed bytes from the descriptors, unpacked
bytes and entries against the pusher's quota as they are produced. A refusal is the spec's
`MANIFEST_INVALID` whose `detail` names the rule, the layer and the path. A kind Reliquary does not
know is stored unchecked, as the image spec requires.

Done when:
- AC-RQ2.1.1 A manifest of a known kind is visible only after its checks pass; a failed one leaves no manifest, no tag and no referrer entry.
- AC-RQ2.1.2 A refusal names its rule by a stable ID, the layer's digest and the offending path or field, in the error's `detail`.
- AC-RQ2.1.3 A blob already checked for a kind at the current checker version is not read again.
- AC-RQ2.1.4 A check's memory is bounded by its admitted budget whatever the layer's size, and its unpacked bytes and entries are charged to the pusher's quota as they are produced.
- AC-RQ2.1.5 A check that exceeds its budget is refused typed, saying which budget, and is not retried by the server.
- AC-RQ2.1.6 An unknown `artifactType` or config media type is stored and served unchanged.
- AC-RQ2.1.7 Every rule of every kind has an ID in `docs/design/artifact-rules.md`, and a doc-truth test holds the document and the checkers to each other.

Tests:
- T-RQ2.1.1 [happy] Push a valid artifact of each known kind → `201`; pull returns it identical.
- T-RQ2.1.2 [refusal] Push an artifact whose second layer breaks a rule → `MANIFEST_INVALID` naming the rule, that layer and the path; `GET` of the manifest is `404`; its tag does not exist.
- T-RQ2.1.3 [edge] Push the same layers under a second manifest → the second push reads no layer bytes (the bytes-read counter does not move).
- T-RQ2.1.4 [edge] Raise a checker's version → the next push of old bytes checks them again under the new rules.
- T-RQ2.1.5 [race] Two pushes of one invalid manifest at once → both refused; no interleaving leaves it visible.
- T-RQ2.1.6 [race] A layer deleted while its manifest is being checked → the push is refused `MANIFEST_BLOB_UNKNOWN`; the check stops reading.
- T-RQ2.1.7 [exhaustion] A layer that unpacks past the pusher's quota → refused typed at the byte that crossed it; memory stayed within the check's budget.
- T-RQ2.1.8 [failure] The server is killed mid-check → after restart the manifest is absent, and pushing again checks and succeeds.
- T-RQ2.1.9 [happy] A manifest of an unknown artifact type with arbitrary layers → stored, served, never read by a checker.
- T-RQ2.1.10 [deadlock] As many checks as the admission limit allows, each waiting on bytes while the bytes budget is held by checks → each proceeds in turn; the queue drains.

---

### RQ2.2 Readers for tar, gzip, zstd and zip that cannot be turned against us

Needs: RQ1 · Serves: AC-T9.1.3

Checks read archives a stranger made. Take Shards' tar, gzip and zstd readers, vendored, so Reliquary
and Shards read an archive the same way. Write a zip reader, sans-IO over ranged reads: it finds the
central directory from the end and never needs the whole file. Each reader is streaming, flat (no
recursion), and refuses instead of guessing: a path that leaves its root, a link that points out, an
entry whose headers disagree, overlapping zip entries, a decoder window larger than the memory the
read was admitted with, more output than the budget.

Done when:
- AC-RQ2.2.1 The tar, gzip and zstd readers are Shards' own, at kiln's revision, and Shards' oracle tests for them run in Reliquary's gate.
- AC-RQ2.2.2 The zip reader agrees with a reference implementation on a generated corpus, and refuses every archive in the published parser-differential corpus that two mainstream readers read differently.
- AC-RQ2.2.3 No reader's memory exceeds its admitted budget on any input; a zstd frame whose window exceeds the budget is refused before it is allocated.
- AC-RQ2.2.4 No reader recurses, and each ends on its byte and entry budgets.
- AC-RQ2.2.5 Every refusal is typed and names the entry and what was wrong with it.
- AC-RQ2.2.6 A deflate stream cannot expand by more than 1032 times (derived from the format, cited): a zip entry or gzip member that declares more is refused before a byte is inflated, and inflation's budget is the least of that bound, the declared size and what was admitted.
- AC-RQ2.2.7 A zip entry is inflated only by a decoder the reader bounds (stored, deflate, zstd); one packed by any other method is refused typed, naming the method.

Tests:
- T-RQ2.2.1 [happy] Archives made by GNU tar, bsdtar, Go's archive/tar, Docker, Info-ZIP, 7-Zip and Python's zipfile → each lists and extracts identically to its own tool.
- T-RQ2.2.2 [hostile] Paths with `..`, absolute paths, a backslash, a drive letter or a NUL; a symlink then a file written through it; a hard link to a path outside → each refused, naming the entry.
- T-RQ2.2.3 [hostile] A zip whose entries overlap (a non-recursive zip bomb); a zip whose central directory and local headers disagree on name or size; two central directories; an entry named twice → each refused.
- T-RQ2.2.4 [hostile] A gzip that expands past the budget; a zstd frame declaring a window larger than the budget; a pax header declaring a path longer than the budget → refused at the budget, with memory within it.
- T-RQ2.2.5 [hostile] Fuzz each reader with mutated real archives → no panic, no hang, memory within budget.
- T-RQ2.2.6 [edge] An empty archive; a tar with only directories; a zip64 archive with an entry over 4 GiB; a multi-member gzip; a zstd file of several frames and a skippable frame → each read correctly.
- T-RQ2.2.7 [edge] Names that differ only by case, by Unicode normalization, or that are reserved on Windows (`CON`, `aux.txt`, a trailing dot) → listed as they are, and flagged for the checks that care.
- T-RQ2.2.8 [model] Generated trees packed by the reference and read by ours, and the reverse → identical trees.
- T-RQ2.2.9 [failure] The byte source ends mid-entry → a typed "truncated", naming the entry and offset.
- T-RQ2.2.10 [hostile] A tar entry whose pax size and ustar size disagree, so that its content would be read as headers (CVE-2025-62518) → refused, naming the entry; nothing inside its content is listed as an entry.
- T-RQ2.2.11 [hostile] A zip entry and a gzip member each declaring an unpacked size more than 1032 times its packed size; a zip entry packed with a method the reader does not bound → each refused before any inflation, by counter.

---

### RQ2.3 Agents, harnesses and MCP servers, by Shards' rules

Needs: RQ2.1, RQ2.2 · Serves: AC-T9.1.3, AC-T9.1.1

Check `application/vnd.osi.agent.v1`, `.harness.v1` and `.mcp.v1` as AGENTFILE_ARCH §12.17 defines
them. The config: a known `schemaVersion`, a stage-style `name`, a version, a platform or none, a
`run` command, `skills`, `mcp` and `asks`, every path relative and inside the artifact's directory.
The content: layers of the kind's content media type, plain or with gzip or zstd, holding files and
directories alone. An index lists one artifact per platform. Write one test for every refusal the
specification states, and hold Reliquary's verdicts to Shards' on a shared corpus, so a thing Shards
would refuse to build or pull is never accepted here.

Done when:
- AC-RQ2.3.1 §12.17's refusal cases accepted on push is exactly 0, measured by a test of every refusal case (AC-T9.1.3).
- AC-RQ2.3.2 Every artifact `shards build --agent`, `--harness` and `--mcp` makes is accepted, for each platform and as an index.
- AC-RQ2.3.3 On the shared corpus of valid and mutated artifacts, Reliquary's verdict equals Shards' for every one.
- AC-RQ2.3.4 `shards push`, `shards pull` and `AGENT … FROM`, `HARNESS … FROM` and `MCP … FROM` work against Reliquary unchanged.
- AC-RQ2.3.5 The config's `asks` are kept as records for the catalog: network, volumes, processes and memory, per platform.

Tests:
- T-RQ2.3.1 [happy] Push an agent, a harness and an MCP server built by Shards → accepted; `shards pull` returns each identical.
- T-RQ2.3.2 [refusal] A config with `schemaVersion` 2; with a name that is no stage-style name; with an absolute `run.workdir`; with a `skills` path of `../x`; with an `mcp` command outside the directory → each refused, naming the field.
- T-RQ2.3.3 [refusal] A content layer holding a whiteout, a device node, a FIFO, a socket, a set-user-ID file, a symlink out of the directory, a hard link out of it → each refused, naming the path.
- T-RQ2.3.4 [refusal] A layer of another media type under an agent manifest; an agent config under a harness artifact type → refused.
- T-RQ2.3.5 [edge] An agent with no `platform`; with no layers beyond one empty directory; with `asks` absent; with unknown extra config fields → the first three accepted; the fourth as Shards' reader decides, and the test pins which.
- T-RQ2.3.6 [edge] An index of two platforms where one member is invalid → the index is refused, naming the member.
- T-RQ2.3.7 [edge] Layers applied in order where a later layer replaces a file with a directory → accepted or refused as Shards' apply does, pinned.
- T-RQ2.3.8 [model] The shared corpus through Shards' checker and Reliquary's → equal verdicts, and the corpus's counters show every rule was hit.
- T-RQ2.3.9 [happy] `AGENT main FROM <reliquary>/agents/x:1` in an Agentfile → `shards build image` takes it and the agent runs.

---

### RQ2.4 Shards microVM images

Needs: RQ2.1, RQ2.2 · Serves: AC-T9.1.1, CP-T9.6

Check `application/vnd.shards.microvm.v1`: one manifest whose layers are an EROFS disk
(`application/vnd.shards.erofs.v1`) and the image config it was made from. Write an EROFS validator,
flat and bounded: the superblock and its checksum, the features Shards' writer sets and no others,
every inode, directory, extended attribute and block address inside the image, no cycle in the
directory tree. A guest's kernel will mount this disk, so a disk the validator passes must be one the
kernel reads without leaving the image. Record the disk's block size and length for lazy loading
later. MicroVMs made ad hoc from any image and those built from an Agentfile take the same path.

Done when:
- AC-RQ2.4.1 Every disk Shards' EROFS writer makes is accepted, and the corpus's counters show every structure was walked.
- AC-RQ2.4.2 A disk with any block address, inode number or name offset outside the image, a directory cycle, a wrong checksum or a feature flag Shards does not write is refused, naming the structure.
- AC-RQ2.4.3 Every disk the validator accepts mounts read-only in a Linux guest and reads back, file for file, what the validator listed.
- AC-RQ2.4.4 The validator's memory is bounded by its budget whatever the disk's size, and it walks without recursion.
- AC-RQ2.4.5 A microVM artifact whose config is not a valid image config, or whose layers are not exactly one disk and one config, is refused.

Tests:
- T-RQ2.4.1 [happy] `shards` saves a microVM of a real image; push it, pull it and boot it → the push is accepted, and the pulled microVM runs as the image did.
- T-RQ2.4.2 [hostile] Flip each superblock field in turn; point an inode's data past the end; make two directories each other's parent; set an unknown feature bit → each refused, naming the structure.
- T-RQ2.4.3 [hostile] Mutation-fuzz valid disks and mount every one the validator accepts in a throwaway guest → no accepted disk makes the guest's kernel fault or read outside the image.
- T-RQ2.4.4 [edge] A disk of one empty directory; one with a file of exactly one block; one with the longest name and the deepest path the format allows; one larger than memory → accepted.
- T-RQ2.4.5 [refusal] A microVM manifest with two disks; with no config; with a disk of the wrong media type → refused.
- T-RQ2.4.6 [exhaustion] A disk whose directory count exceeds the check's entry budget → refused typed at the budget.
- T-RQ2.4.7 [happy] A microVM built from an Agentfile and one made ad hoc from `ubuntu` → both accepted by the same checks.

---

### RQ2.5 Firecracker kernel and root filesystem pairs

Needs: RQ2.1, RQ2.2 · Serves: AC-T9.1.1, CP-T9.6

Agree with Hex's owner an artifact for what Hex's `firecracker` class boots: a kernel, a root
filesystem, an optional initial ramdisk, and a config with the boot arguments, the architecture and
what the guest asks. Check it on push: the kernel is the image format its architecture's Firecracker
boots, the root filesystem is a filesystem image whose superblock validates, and the config's paths
and sizes agree with the layers. Record it so Hex can ask for "the pair for this digest".

Done when:
- AC-RQ2.5.1 The artifact's media types and config schema are in the design record, agreed with Hex's owner by message, with the date.
- AC-RQ2.5.2 A pair Hex's `firecracker` class boots is accepted, on x86_64 and arm64.
- AC-RQ2.5.3 A kernel of the wrong architecture or format, a root filesystem that is not a filesystem image, or a config that names a layer the manifest lacks is refused, naming which.
- AC-RQ2.5.4 Hex's simulator of the seam and Reliquary's implementation pass one conformance suite for "fetch the pair for a digest".

Tests:
- T-RQ2.5.1 [happy] Push the pair Hex's lab boots; Hex pulls it by digest and boots it → the guest runs.
- T-RQ2.5.2 [refusal] An arm64 kernel in an artifact whose config says x86_64; a gzip of a kernel where the uncompressed image is required; a text file as the root filesystem → each refused.
- T-RQ2.5.3 [edge] A pair with and without a ramdisk; a root filesystem that is EROFS, ext4 or squashfs → accepted where Hex's class takes it, refused typed where it does not.
- T-RQ2.5.4 [hostile] A kernel image with a truncated header; a filesystem whose superblock claims a size beyond the layer → refused.
- T-RQ2.5.5 [refusal] Boot arguments in the config that try to name a device outside the pair → refused by the rule Hex's owner states.

---

### RQ2.6 Images built from Agentfiles

Needs: RQ2.1 · Serves: AC-T9.1.1, CP-T9.3

An image Shards built from an Agentfile carries what the Agentfile declared: its agents, harnesses,
skills, MCP servers, networks, volumes and grants. Read it on push with Shards' own parser, vendored,
exactly as Shards' daemon reads it before it runs the image. A declaration the daemon would refuse is
refused here. What it declares is kept as records, so the catalog can say what an image runs and what
it asks for without pulling it.

Done when:
- AC-RQ2.6.1 Where an image carries its Agentfile, and how it is read, is recorded as Shards' owner states it.
- AC-RQ2.6.2 Every image `shards build image` makes from the Agentfiles in Shards' examples is accepted, and its declared members and asks are in the records.
- AC-RQ2.6.3 An image whose carried Agentfile fails Shards' own run-time checks (a domain reaching the world through another, a grant to a member that does not exist) is refused with Shards' words.
- AC-RQ2.6.4 A plain OCI image with no Agentfile is accepted untouched.

Tests:
- T-RQ2.6.1 [happy] Push an Agentfile image with two agents, a harness, a skill and an MCP server → accepted; the records list each with its grants.
- T-RQ2.6.2 [refusal] An image whose Agentfile was edited after the build to join an internal-only agent to the world → refused, naming the path, as Shards' daemon refuses it.
- T-RQ2.6.3 [edge] An image that references an agent by digest that Reliquary does not hold → accepted, and the catalog marks the member as unresolved.
- T-RQ2.6.4 [hostile] An Agentfile with a line longer than the parser's bound, deep continuation and invalid UTF-8 → refused in bounded time by the vendored parser.
- T-RQ2.6.5 [happy] `nginx` pushed as is → no Agentfile found, accepted, no member records.

---

### RQ2.7 Tar and zip artifacts, and one file out of them

Needs: RQ2.1, RQ2.2 · Serves: CP-T9.1

Make a plain archive a first-class artifact: a manifest with an archive artifact type and one layer
that is the tar (plain, gzip or zstd) or the zip. On push, check it reads cleanly by RQ2.2's rules,
and write its file index beside it as a referrer: every entry's path, size, mode, digest and where its
bytes lie. With the index, the registry answers "list this archive" and "give me this one file"
without the client pulling the archive: from a zip or an uncompressed tar by a byte range; from a
compressed tar by decompressing up to it, and the index says which it will be.

Done when:
- AC-RQ2.7.1 A tar, tar.gz, tar.zst or zip pushed as an archive artifact is stored byte for byte and pulls back identical.
- AC-RQ2.7.2 Its index lists exactly the entries its own tool lists, each with a digest that matches the extracted file.
- AC-RQ2.7.3 One file read through the registry equals the file its own tool extracts, for every entry of every archive in the corpus.
- AC-RQ2.7.4 Reading one file from a zip or plain tar reads only that file's bytes from Mantle, measured by the bytes-read counter.
- AC-RQ2.7.5 An archive RQ2.2 refuses is refused on push with the entry named; an encrypted zip entry is stored and indexed as unreadable, not refused.
- AC-RQ2.7.6 The index is itself an OCI artifact that any client can list through the referrers API.

Tests:
- T-RQ2.7.1 [happy] Push each archive form with ORAS, list it, and read one file from the middle → accepted; listed; the file identical to the tool's extraction.
- T-RQ2.7.2 [edge] An archive with one empty file; with a file larger than memory; with more entries than one index page holds; with a name of every script Unicode has → listed and read correctly.
- T-RQ2.7.3 [edge] A zip with stored, deflated and unsupported-method entries → the first two readable; the third listed with its method and refused typed on read.
- T-RQ2.7.4 [refusal] Ask for a path the archive lacks; a path with `..`; a directory → `404`, `400`, the directory's listing.
- T-RQ2.7.5 [race] The archive deleted while one of its files streams → the stream completes or ends in an error; never another archive's bytes.
- T-RQ2.7.6 [hostile] A zip whose index would be larger than the archive (millions of empty entries) → refused at the entry budget.
- T-RQ2.7.7 [failure] The index referrer is deleted → listing rebuilds it on demand within the check's budget, or answers typed that it is being rebuilt.
- T-RQ2.7.8 [exhaustion] Many clients each reading a late file of a large tar.gz → each read is charged its decompression; past the budget they are refused with `Retry-After`, and the registry stays responsive.

---

### RQ2.8 Files by URL

Needs: RQ2.7 · Serves: CP-T9.1, CP-T9.2

Give people without an OCI client a way in: `PUT` a file to a URL under a repository and version, and
`GET` it back, with ranges, its digest in a header and its length. Behind the URL is an ordinary
artifact, visible to every OCI client. The same authentication and the same checks apply. This is what
an MCP bundle's download URL and a release asset point at.

Done when:
- AC-RQ2.8.1 `curl -T file URL` then `curl -O URL` round-trips any file byte for byte, and `oras pull` of the same name and tag gets the same bytes.
- AC-RQ2.8.2 A `PUT` is atomic: a reader sees the old file or the new, never a part; a failed `PUT` leaves the old.
- AC-RQ2.8.3 The `GET` carries the file's SHA-256, honors ranges and conditional requests, and an immutable version answers with caching headers that say so.
- AC-RQ2.8.4 A `PUT` with a stated digest that is not the body's is refused and nothing changes.
- AC-RQ2.8.5 The URL's grammar maps onto a valid repository name and tag, and a path that does not is refused `400` saying why.

Tests:
- T-RQ2.8.1 [happy] Put and get a zip, a tar and a single binary → identical; each appears in the repository's tag list.
- T-RQ2.8.2 [edge] Put a zero-byte file; a file larger than memory; the same file twice; a second file to the same URL → stored; stored; unchanged; replaced atomically, or refused where the version is immutable.
- T-RQ2.8.3 [refusal] A path with an upper-case repository, an empty segment, or a tag over 128 characters → `400` with the rule.
- T-RQ2.8.4 [failure] The upload's connection drops at points across the file → the old file is still served whole.
- T-RQ2.8.5 [race] Two `PUT`s to one URL at once → one wins whole; the other is told it lost or wins after; a reader never sees a mix.
- T-RQ2.8.6 [hostile] A `PUT` with a `Content-Length` it does not honor; a filename with control characters or a NUL → refused.
- T-RQ2.8.7 [happy] An `.mcpb` bundle put by URL → its `fileSha256` as an MCP `server.json` would state it equals the header's digest.

---

### RQ2.9 Kiln's registry moves here

Needs: RQ2.3, RQ2.4 · Serves: AC-T9.1.1, T-T9.1.1

Move what kiln's registry holds into Reliquary and make Reliquary what kiln serves. An importer reads
kiln's manifests from its SQLite and its blobs from its content store, pushes each through Reliquary's
own checks, and verifies every digest. Referrers kept under the tag schema join the referrers API, as
the spec requires. Kiln's repository policies, mirrors and grants map onto Reliquary's. The switch is
a capability in kiln: reads fall back to the old registry for one release, and the rollback is drilled
before the switch. Anything the new checks refuse is listed for the person, not dropped.

Done when:
- AC-RQ2.9.1 Everything kiln's registry served before the move is served identically after it, by digest: every manifest, blob, tag and referrer (T-T9.1.1).
- AC-RQ2.9.2 `shards push`, `shards pull` and `AGENT … FROM` work against kiln's registry with every artifact checked, in a kiln change request that passes kiln's gate (AC-T9.1.1).
- AC-RQ2.9.3 An artifact the importer's checks refuse is neither imported nor deleted: it stays in the old store and is in a report with its rule.
- AC-RQ2.9.4 The import can stop and resume at any point and never imports a thing twice or half.
- AC-RQ2.9.5 With the capability off, kiln behaves exactly as before; a rollback drill in the lab returns service with no artifact lost.
- AC-RQ2.9.6 Kiln reaches Reliquary only through kiln-seams' `Artifacts`, checked by kiln's own dependency test.

Tests:
- T-RQ2.9.1 [happy] Import a copy of production's registry; pull every tag from both and compare digests → all equal.
- T-RQ2.9.2 [edge] A manifest in SQLite whose blob is missing from the store; a blob no manifest names; a tag on a deleted manifest → each reported, none imported as valid, none stops the import.
- T-RQ2.9.3 [edge] A signature kept under `sha256-<hex>` tag schema → listed by the referrers API after import, and still pullable by its tag.
- T-RQ2.9.4 [failure] Kill the importer at random points, many times → the result equals an uninterrupted import.
- T-RQ2.9.5 [race] Pushes to the old registry during the import → each is either imported or reported as arriving after the cut; none is silently lost.
- T-RQ2.9.6 [refusal] An agent in the old registry that breaks §12.17 → left in the old store, in the report, and refused on a fresh push.
- T-RQ2.9.7 [happy] The rollback drill: switch on, push, switch off → the old registry serves what it had, and what was pushed in between is listed for re-push.
- T-RQ2.9.8 [happy] A mixed fleet, one runner a release behind → pulls and pushes work through either registry during the fallback release.
