# Phase 5: Skills and MCP servers

**Goal RQ5.** A skill and an MCP server are first-class: published from where they are written,
checked against the formats that define them, versioned, and found by the clients that use them. An
MCP client discovers Reliquary's servers through the MCP Registry API, and kiln's agents take their
skills and servers from Reliquary.

**Delivers.** Skills as artifacts, held to the Agent Skills format by the same checker Shards uses.
MCP servers with their `server.json`, served by the MCP Registry's own API, where the metadata and
the bytes it points at are checked against each other because both live here.

**Serves.** CP-T9.2 (AC-T9.2.1 to AC-T9.2.3, T-T9.2.1). **Needs.** RQ1; RQ2.1 to RQ2.3 for the
artifact checks; RQ2.8 for bundle URLs.

**Order.** RQ5.1 and RQ5.4 first, side by side. Then RQ5.2, RQ5.3, RQ5.5 and RQ5.6 side by side.
RQ5.7 closes.

---

### RQ5.1 A skill as an artifact

Needs: RQ2.1 · Serves: AC-T9.2.3, T-T9.2.1

Shards names artifact types for agents, harnesses and MCP servers, and none for a skill. Propose one
to Shards' owner in the same shape: an artifact type, a JSON config (the skill's name, description,
license, compatibility, metadata and allowed tools as its `SKILL.md` states them, a version, and
where it came from: a git URL and commit, a URL, or a path), and content layers that are the skill's
directory as a tar. The layers hold files and directories alone, as an agent's do. Record the
agreement in both projects. A skill packed and unpacked is the same directory, byte for byte.

Done when:
- AC-RQ5.1.1 The skill artifact's media types and config schema are in the design record, agreed with Shards' owner, with the date.
- AC-RQ5.1.2 Packing a skill directory twice gives the same digest; unpacking gives the directory back byte for byte, with file modes kept where the platform has them.
- AC-RQ5.1.3 The config's fields equal the `SKILL.md` frontmatter's, checked on push; a config that disagrees with its content is refused.
- AC-RQ5.1.4 `SKILL … FROM` a Reliquary reference in an Agentfile takes the skill, once Shards' change lands, and until then the artifact unpacks to a directory `SKILL` takes by path.

Tests:
- T-RQ5.1.1 [happy] Pack each skill of the public Agent Skills examples, push, pull, unpack → identical trees.
- T-RQ5.1.2 [edge] A skill of one `SKILL.md` alone; one with `scripts/`, `references/` and `assets/`; one with an executable script; one with an empty directory → each round-trips, modes and empty directories kept.
- T-RQ5.1.3 [edge] The same skill packed on Linux, macOS and Windows → the same digest.
- T-RQ5.1.4 [refusal] A config naming the skill `pdf-tools` over a `SKILL.md` naming it `pdf` → refused, naming both.
- T-RQ5.1.5 [refusal] A skill layer with a symlink out of the directory or a device node → refused by RQ2's content rule.

---

### RQ5.2 Skills checked on publish

Needs: RQ5.1, RQ2.2 · Serves: AC-T9.2.3

Refuse a skill that is not one. The rules are the Agent Skills specification's, as its reference
validator applies them: a `SKILL.md` with YAML frontmatter; a `name` of 1 to 64 lowercase letters,
digits and hyphens that neither starts nor ends with a hyphen, has no two in a row, and equals its
directory's name; a `description` of 1 to 1,024 characters; `compatibility` of at most 500; `metadata`
a map of strings; no field the format does not allow. Use Shards' implementation of that validator,
vendored, so a skill Shards' build would refuse is never accepted here. Build the corpus the
criterion asks for: real skills, and mutations of them that break one rule at a time.

Done when:
- AC-RQ5.2.1 Invalid skills accepted on publish, over a corpus of real and mutated skills, is exactly 0 (AC-T9.2.3).
- AC-RQ5.2.2 Valid skills refused, over the corpus's real skills, is exactly 0.
- AC-RQ5.2.3 On every skill of the corpus, Reliquary's verdict and message equal the reference validator's, recorded from the validator at a pinned commit.
- AC-RQ5.2.4 The corpus's mutations cover every rule: its counters show each rule refused at least one skill.
- AC-RQ5.2.5 Fields that Claude Code reads beyond the open format are kept and reported, and accepted or refused by a stated, per-namespace choice between the strict format and Claude Code's.

Tests:
- T-RQ5.2.1 [happy] Publish every real skill in the corpus → accepted.
- T-RQ5.2.2 [refusal] One mutation per rule: no `SKILL.md`; no frontmatter; a name with an upper-case letter, a leading hyphen, a double hyphen, 65 characters, or unlike its directory; an empty description; one of 1,025 characters; `compatibility` of 501; `metadata` with a non-string value; an unknown field → each refused with the validator's message.
- T-RQ5.2.3 [edge] A name of exactly 64 characters; a description of exactly 1,024; a name in a non-Latin lowercase script; a name that is equal only after Unicode normalization → as the validator decides, pinned.
- T-RQ5.2.4 [hostile] Frontmatter with YAML anchors, aliases, tags, merge keys, flow style, duplicate keys, or a megabyte scalar → refused, in bounded time.
- T-RQ5.2.5 [hostile] A `SKILL.md` of invalid UTF-8; with a byte-order mark; with CRLF line ends; with only a first `---` → as the validator reads a file, pinned.
- T-RQ5.2.6 [model] The whole corpus through the reference validator and through Reliquary → equal verdicts.
- T-RQ5.2.7 [non-vacuity] Each rule's counter after the corpus run → above zero.

---

### RQ5.3 Skills from git, a URL, a path or a registry

Needs: RQ5.1, RQ5.2 · Serves: T-T9.2.1, AC-T9.2.1

Publish a skill from where it lives. `reliquary skill publish` takes a git URL at a ref, an http(s)
URL, a local path or another registry's reference, finds the skill or skills there, checks each, and
pushes each as a version. The work is the client's: it fetches with the credentials its user has, so
the server never fetches from an address a stranger chose. A version is a tag; where the skill's
metadata names a version, the tag is that; a git ref's commit is recorded in the config either way.
`latest` follows the highest version by semantic-version order where every version parses, and the
last published otherwise, and the rule is stated in the artifact's page.

Done when:
- AC-RQ5.3.1 A skill published from git at a ref pulls back byte for byte, with its version and digest (T-T9.2.1).
- AC-RQ5.3.2 The published config names the source URL and the exact commit, and the same commit publishes to the same digest from any machine.
- AC-RQ5.3.3 A repository holding several skills publishes each under its own name; one invalid skill fails its own publish and is reported, without publishing a part of it.
- AC-RQ5.3.4 Publishing a version that exists with other content is refused unless the repository allows tags to move; with the same content it succeeds and changes nothing.
- AC-RQ5.3.5 The server makes no outbound request on behalf of a publish.

Tests:
- T-RQ5.3.1 [happy] Publish a skill from a git tag, a branch and a commit; from a tarball URL; from a path; from another registry → each pulls back identical to its source tree.
- T-RQ5.3.2 [edge] A ref that moves between two publishes → two versions, each recording its own commit.
- T-RQ5.3.3 [edge] A git repository whose skill sits in a subdirectory; one with a submodule; one with large files by pointer → the first published; the others handled by a stated rule and refused typed where they cannot be read whole.
- T-RQ5.3.4 [refusal] A URL that answers `404`, redirects to another host, or serves a different body on each fetch → refused, naming what happened; the digest is of what was fetched once.
- T-RQ5.3.5 [failure] The network drops mid-fetch or mid-push → nothing is published; running again publishes once.
- T-RQ5.3.6 [race] Two publishers push version `1.2.0` of one skill with different content at once → one wins; the other is refused, naming the existing digest.
- T-RQ5.3.7 [hostile] A git repository with a path that escapes the checkout, a file named like a device, or a symlink to `/etc` → refused by the content rules before any push.

---

### RQ5.4 MCP servers and their `server.json`

Needs: RQ2.3 · Serves: AC-T9.2.2

An MCP server in Reliquary is a `server.json` record and, where the server's bytes are here, the
artifact it points at. Validate every `server.json` against the official schema at a pinned version,
and against what Reliquary can see: an `oci` package whose image is in Reliquary must exist, is
pinned by its digest, and carries the ownership mark the MCP registry's package rules ask for; an
`mcpb` package's URL, when it is a Reliquary file URL, must serve bytes whose SHA-256 is the stated
`fileSha256`; a stdio server packaged as a Shards `mcp` artifact must pass RQ2.3. A server's name
is a reverse-DNS namespace and a name; the publisher must hold that namespace. Nothing in a
publisher's `_meta` is dropped in silence: it is kept or refused.

Done when:
- AC-RQ5.4.1 Every stored `server.json` validates against the official schema version it names; one that does not is refused with the schema's path and reason.
- AC-RQ5.4.2 A package reference into Reliquary that does not resolve, or whose digest or hash disagrees with the bytes, is refused; references elsewhere are recorded as unverified and say so.
- AC-RQ5.4.3 A publisher can publish only under a namespace it holds, proven by a DNS record for a domain namespace or by the identity provider for an account namespace; the proof is recorded.
- AC-RQ5.4.4 A version, once published, never changes content; its status can move between active, deprecated and deleted with a message.
- AC-RQ5.4.5 Publisher-provided `_meta` is returned exactly as published, or the publish is refused naming the key; nothing is silently dropped.

Tests:
- T-RQ5.4.1 [happy] Publish a server with an `oci` package held here, one with an `mcpb` bundle held here, one with a remote URL, and one with an npm package → each stored; the first two verified, the others marked unverified.
- T-RQ5.4.2 [refusal] A `server.json` missing `name`; with a version of `latest`; with a package of an unknown registry type; with a remote URL that is not https → refused with the schema's reason.
- T-RQ5.4.3 [refusal] An `oci` identifier naming a tag here whose digest differs from the stated one; an `mcpb` `fileSha256` that is not the file's → refused, naming both values.
- T-RQ5.4.4 [refusal] Publishing `com.example/x` without holding `example.com` → refused; nothing stored.
- T-RQ5.4.5 [edge] The same version published twice with identical content; with different content → unchanged and accepted; refused.
- T-RQ5.4.6 [edge] A name at the schema's length limit; a description at its limit; every optional field absent → accepted.
- T-RQ5.4.7 [race] Two publishers publish the first version of one name at once → one wins the name; the other is refused.
- T-RQ5.4.8 [race] The image an `oci` package points at is deleted after publish → the server's page marks the package unresolved; a policy can forbid the delete while a server references it.
- T-RQ5.4.9 [hostile] A `server.json` with deep nesting, megabyte strings or duplicate keys → refused in bounded time.

---

### RQ5.5 The MCP Registry API

Needs: RQ5.4 · Serves: AC-T9.2.2

Serve the generic MCP Registry API as its OpenAPI document states it: list servers with a cursor and
`updated_since`, list a server's versions, get one version or `latest`, publish, and set a status.
Cursors are opaque and stable while the data changes. Authorization follows the registry
authorization specification. Reliquary's own facts about a server (its digest, its signatures, what
it asks for) ride in `_meta` under Reliquary's own namespace. Every response is validated against
the OpenAPI schema in the gate.

Done when:
- AC-RQ5.5.1 An MCP client discovers Reliquary's servers through the MCP Registry API, checked against the official `server.json` schema (AC-T9.2.2): every response validates against the pinned OpenAPI document.
- AC-RQ5.5.2 The official registry's own publisher tool and at least two independent MCP clients list, fetch and, where they publish, publish against Reliquary unchanged.
- AC-RQ5.5.3 Walking a listing by cursor while servers are published and deleted returns every server that existed throughout exactly once.
- AC-RQ5.5.4 `updated_since` returns every server whose record changed after that instant, deletions included.
- AC-RQ5.5.5 A reader sees only the servers its grants allow; a server it may not see is absent from lists and `404` by name, alike in time within the interval.
- AC-RQ5.5.6 A listing's cost does not grow with the number of servers stored, at the plan's three scale sizes.

Tests:
- T-RQ5.5.1 [happy] List, page to the end, fetch a version, fetch `latest` → each validates against the schema.
- T-RQ5.5.2 [edge] `limit` of 1, of the maximum, and above it; a cursor from another listing; an empty registry → the specified answers; a foreign cursor is refused `400`.
- T-RQ5.5.3 [edge] A server name with a slash and characters that need URL encoding → found by its encoded path.
- T-RQ5.5.4 [race] Page through a listing while another client publishes and deletes → nothing that existed throughout is skipped or repeated.
- T-RQ5.5.5 [refusal] Publish without a token; with a read-only token; to another's namespace → `401`, `403`, `403`.
- T-RQ5.5.6 [hostile] A cursor edited by the client; a `limit` beyond any integer the schema allows; an `updated_since` far in the past or the future → refused or clamped as the schema states; never a full scan.
- T-RQ5.5.7 [exhaustion] Many clients polling `updated_since` as fast as they can → served from the change feed's position, each poll's cost bounded by what changed.
- T-RQ5.5.8 [model] Generated publish, status and list histories against a model of the API → every answer matches.

---

### RQ5.6 Following the official registry

Needs: RQ5.5 · Serves: AC-T9.2.1

The MCP ecosystem expects registries like Reliquary to be subregistries: they follow the official
registry and add to it. Follow it: read what changed upstream with `updated_since`, at a pace the
upstream's own limits and answers set, and keep each server under a namespace that marks it as
followed, with where and when it was read. What Reliquary followed and what a publisher published
here are never confused. When the upstream is down, what was followed keeps being served.

Done when:
- AC-RQ5.6.1 After a follow completes, every active server the upstream lists is in Reliquary with identical `server.json`, compared by a full listing of both.
- AC-RQ5.6.2 A followed server says so in `_meta`, with its source and the time read; a local publish can never take a followed server's name.
- AC-RQ5.6.3 The follow's request rate is derived from the upstream's stated limits and its `429` answers, and a follow never draws a `429` twice in a row.
- AC-RQ5.6.4 With the upstream unreachable, followed servers are served, marked with the age of the last successful read.
- AC-RQ5.6.5 Following is off unless the operator turns it on, and each namespace can be excluded.

Tests:
- T-RQ5.6.1 [happy] Follow a recorded copy of the official registry → the two listings are equal.
- T-RQ5.6.2 [edge] Upstream deprecates, deletes and restores a server between follows → Reliquary's copy follows each change.
- T-RQ5.6.3 [failure] Upstream answers `429` with `Retry-After`, `500`, or a truncated body → waited out as told, retried by RQ0.8's rule, refused whole; never a half-updated server.
- T-RQ5.6.4 [failure] Upstream's cursor is lost mid-follow (a restart) → the follow resumes from the last committed position and ends equal.
- T-RQ5.6.5 [hostile] Upstream serves a `server.json` that fails the schema, or claims a name in a locally held namespace → that record is refused and reported; the rest follow.
- T-RQ5.6.6 [race] A local publish of a name while the follow brings the same name → the local namespace rule decides; the result is the same whichever came first.

---

### RQ5.7 Kiln's agents take their skills and MCP servers from Reliquary

Needs: RQ5.3, RQ5.5 · Serves: AC-T9.2.1

Put it to use. Kiln's default Agentfiles name their skills and MCP servers as Reliquary references.
Kiln's agents, when they start, get those skills and servers from Reliquary, by digest. Kiln's own
skills are published to Reliquary by its CI. The change is kiln's change request; Reliquary's part is
whatever the consumer finds missing.

Done when:
- AC-RQ5.7.1 Kiln's agents get their skills and MCP servers from Reliquary, in a kiln change request that passes kiln's gate (AC-T9.2.1).
- AC-RQ5.7.2 An agent started twice from one Agentfile gets byte-identical skills, by digest.
- AC-RQ5.7.3 With Reliquary unreachable, an agent whose skills its runner already holds starts; one whose skills are missing is refused, naming them.
- AC-RQ5.7.4 A skill updated in Reliquary reaches a new agent only when its Agentfile's reference says so (a moved tag or a new digest), never by surprise.

Tests:
- T-RQ5.7.1 [happy] Launch an agent from a default Agentfile → its skills directory and MCP configuration equal what Reliquary holds at those digests.
- T-RQ5.7.2 [edge] An Agentfile pinning a skill by digest while its tag has moved on → the pinned content.
- T-RQ5.7.3 [failure] Reliquary down at launch → the cached case starts; the uncached is refused typed.
- T-RQ5.7.4 [refusal] An Agentfile naming a skill version that was deleted, or an MCP server whose status is deleted → the launch is refused, naming it.
- T-RQ5.7.5 [happy] Kiln's CI publishes a changed skill → a new version appears, with its commit in its config.
