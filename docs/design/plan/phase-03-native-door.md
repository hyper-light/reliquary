# Phase 3: The native door

**Goal RQ3.** Shards, Hex and kiln push, pull and manage artifacts over hyper-quic: a pull is one
exchange and a stream of bytes from Mantle, a push is two, and both resume where they stopped.

**Delivers.** A second door beside the OCI one, over the same core: Reliquary's own protocol on
hyper-transport lanes, mutually authenticated, AES-256-GCM only. A client library others link. Bytes
move by Mantle's native protocol, straight between the client and Mantle's nodes once Mantle honors a
grant. Hex's nodes get images into their stores through it and tell the scheduler what they hold.

**Serves.** CP-T9.1 (AC-T9.1.1), CP-T9.6, Hex's CP-T7.8. The person's direction of 2026-11-20:
"interface with the Hex runtime", and for Shards "utilize Mantle's hyper-quic protocol for artifact
upload and download as well as other operations". **Needs.** RQ1. RQ3.7 also needs RQ2.3.

**Order.** RQ3.1 first. RQ3.2, RQ3.3, RQ3.4 and RQ3.5 side by side. RQ3.6 joins them. RQ3.7, RQ3.8
and RQ3.9 close the phase side by side.

---

### RQ3.1 The protocol

Needs: RQ1 · Serves: AC-T9.1.1

Define Reliquary's native protocol as messages on hyper-transport: a control lane of request and
reply exchanges, stream lanes for bytes when they pass through Reliquary, and an event lane a client
can follow. Every request the OCI door serves has a message here, with the same typed refusals. The
wire is a crate of its own with a golden vector for every message; it changes append-only, and each
end states the version it speaks so a peer of another is refused at the hello with both versions
named. Message classes and their credit come from hyper-transport; nothing a peer sends chooses its
own class.

Done when:
- AC-RQ3.1.1 Every operation of the registry core has a request, a reply and its refusals in the wire crate, and a doc-truth test holds `docs/design/native-protocol.md` to it.
- AC-RQ3.1.2 Every message has a golden vector; a change to any encoded byte fails a test.
- AC-RQ3.1.3 A new message or field is appended; an existing one's encoding never changes, checked against the vectors of every released version.
- AC-RQ3.1.4 Two ends of different versions either interoperate on the messages both know or refuse at the hello, naming both versions; never a misparse.
- AC-RQ3.1.5 Every length in a message is bounded by the wire's own field widths or a budget its receiver derives; a longer one is refused before it is read.

Tests:
- T-RQ3.1.1 [happy] Encode and decode every message → round-trips; equals its golden vector.
- T-RQ3.1.2 [hostile] Fuzz the decoder with truncated, extended and bit-flipped messages → typed refusal or a clean decode; no panic, no allocation past the bound.
- T-RQ3.1.3 [edge] A message with every optional field absent, and with every one at its longest → decodes.
- T-RQ3.1.4 [refusal] An unknown message kind; a known kind on the wrong lane; a reply with no request → refused typed, and the connection survives where the transport allows.
- T-RQ3.1.5 [happy] A version N client against a version N+1 server and the reverse → the common subset works; a call only the newer knows is refused as unsupported.
- T-RQ3.1.6 [model] The OCI door and the native door given the same generated history → the same answers, operation for operation.

---

### RQ3.2 The listener, and who is calling

Needs: RQ3.1 · Serves: AC-T9.1.1

Listen for QUIC on hyper-rt through hyper-rt-transport. TLS is 1.3 with AES-256-GCM alone, both ends
presenting certificates, each end asserting the suite it negotiated. An identity is a certificate
issued by the deployment's authority on a signing request, so a private key never leaves the machine
that made it: kiln's authority when Reliquary is embedded, one `reliquary init` makes when it stands
alone. A certificate names a principal, and the principal's grants are the same ones the OCI door
checks. Every wait on the door is learned per kind of call and per path, by RQ0.8.

Done when:
- AC-RQ3.2.1 A connection that does not negotiate TLS 1.3 with AES-256-GCM, or presents no certificate under the authority, is refused in the handshake.
- AC-RQ3.2.2 `reliquary enroll` obtains a certificate by signing request; the private key is created on the client and never sent.
- AC-RQ3.2.3 A principal has the same rights through both doors: a request the OCI door refuses `DENIED` the native door refuses `Denied`, and the reverse.
- AC-RQ3.2.4 A revoked certificate is refused from the revocation's commit on, on every node, with no wait for a list to expire.
- AC-RQ3.2.5 A handshake that answers but does not advance gives up its place by its progress, not at a fixed time.
- AC-RQ3.2.6 The listener works on Linux, macOS and Windows, and a client on each reaches a server on each.

Tests:
- T-RQ3.2.1 [happy] Enroll a client, connect, call → answered; the node's link report names the peer, suite and group.
- T-RQ3.2.2 [refusal] A client offering only ChaCha20-Poly1305 or AES-128-GCM; a self-signed certificate; a certificate from another deployment; an expired one → each refused in the handshake.
- T-RQ3.2.3 [refusal] A signing request for a name the enrolling principal may not take → refused, nothing issued.
- T-RQ3.2.4 [race] A certificate revoked while its connection is mid-push → the exchange in flight ends as the stated rule says, and the next is refused.
- T-RQ3.2.5 [hostile] A flood of initial packets from spoofed addresses → answered with retry tokens; memory per unverified address is zero.
- T-RQ3.2.6 [hostile] A peer that opens lanes up to its grant and sends nothing → holds its grant and no more; cut by progress.
- T-RQ3.2.7 [failure] The client's address changes mid-transfer (Wi-Fi to wired) → the connection migrates and the transfer continues.
- T-RQ3.2.8 [exhaustion] More connections than the memory budget admits → refused at the handshake with a typed close; existing ones unaffected.

---

### RQ3.3 A pull in one exchange

Needs: RQ3.1 · Serves: CP-T9.6, AC-T9.1.4

A client names a reference and its platform. The server answers once with everything a pull needs:
the manifest, or for an index the manifest of that platform, its config, every layer's descriptor,
which of them the client need not fetch if it says what it holds, and where and with what grant to
fetch the rest. The client then reads the bytes, as many at once as its path pays for, each verified
against its digest as it arrives and resumable from its verified prefix. The OCI door needs a round
trip per manifest, config and layer; this needs one.

Done when:
- AC-RQ3.3.1 Pulling an image of any number of layers takes one control exchange, measured by a counter on the lane.
- AC-RQ3.3.2 The bytes a client receives for each blob match its digest, checked before any of it is handed to the caller as complete.
- AC-RQ3.3.3 A pull interrupted at any byte resumes from its verified prefix and sends no verified byte again.
- AC-RQ3.3.4 A pull by tag is pinned: the plan names the digest the tag had, and every later fetch of that pull uses the digest.
- AC-RQ3.3.5 A reference the principal may not read, and one that does not exist, are answered alike in content and, within the measurement's interval, in time.
- AC-RQ3.3.6 The plan's size is bounded; a closure too large for one reply is paged, and the client still makes no per-layer control request.

Tests:
- T-RQ3.3.1 [happy] Pull a multi-platform image by tag → one exchange; the right platform's layers; every digest verifies.
- T-RQ3.3.2 [edge] An image with no layers; with one layer listed twice; with more layers than one reply holds; an index nested in an index → pulled correctly.
- T-RQ3.3.3 [edge] The client says it holds every layer → the plan grants nothing and the pull moves no blob bytes.
- T-RQ3.3.4 [refusal] A platform the index lacks → refused typed, listing the platforms it has.
- T-RQ3.3.5 [race] The tag moves between the plan and the last fetch → the pull completes with the planned digest's content.
- T-RQ3.3.6 [race] A layer is deleted between the plan and its fetch → that fetch is refused typed; the client's pull fails naming the layer; nothing partial is kept as complete.
- T-RQ3.3.7 [failure] The connection is cut at a random byte, many times → each resume continues from the verified prefix; total bytes received equal the image's size plus at most one unverified piece per cut.
- T-RQ3.3.8 [hostile] A server (simulated) that sends a wrong byte in a layer → the client refuses the layer at the piece that fails, and keeps nothing of it as verified.
- T-RQ3.3.9 [exhaustion] A client pulling more at once than its memory budget allows → its own admission holds transfers back; none is dropped, and peak memory stays within the budget.

---

### RQ3.4 A push in two exchanges

Needs: RQ3.1 · Serves: AC-T9.1.1

A client offers what it wants to push: the manifests, and each blob's digest, size and media type.
The server answers which blobs it lacks and how to send them. The client sends those, as many at
once as pays, each a resumable upload whose digest is checked at its end. A second exchange commits:
the blobs are linked, the manifests checked (RQ2) and stored, the tags moved and the referrers listed,
all or nothing. Because the offer states each layer's type, the server can check content as it
streams in instead of reading it again.

Done when:
- AC-RQ3.4.1 Pushing an image of any number of layers takes two control exchanges and the uploads, measured by a counter.
- AC-RQ3.4.2 A blob the repository already links is never sent; one another repository holds is linked only where the principal may read it there.
- AC-RQ3.4.3 The commit is atomic: after it, every manifest, tag and referrer of the push is visible; before it, or if it is refused, none is.
- AC-RQ3.4.4 An upload interrupted at any byte resumes from the server's durable offset, on any node.
- AC-RQ3.4.5 A commit asked again after its answer was lost returns the first answer and changes nothing.
- AC-RQ3.4.6 Content checks that RQ2 defines run during the upload where the offer names the type, and a failing layer ends the push before its commit.
- AC-RQ3.4.7 What an offer reserves (sessions, staged bytes) is charged to the principal and freed if no commit comes, by RQ1.3's rule.

Tests:
- T-RQ3.4.1 [happy] Push an image, then push it again → the first sends its blobs; the second sends none; both commit.
- T-RQ3.4.2 [happy] Push an index, its two images and a signature for each in one commit → all visible together.
- T-RQ3.4.3 [refusal] A commit naming a blob never uploaded; a manifest RQ2 refuses; a tag the repository holds immutable → the whole commit refused, naming the cause; nothing visible.
- T-RQ3.4.4 [failure] Kill the client after the uploads and before the commit, then run it again → no blob is sent again; the commit succeeds.
- T-RQ3.4.5 [failure] Lose the commit's answer → the retry returns the same result; the tag moved once.
- T-RQ3.4.6 [race] Two clients push different manifests to one tag at once → one tag value results; each client's answer says which won.
- T-RQ3.4.7 [race] Two clients push the same new blob at once → both succeed; one copy is kept.
- T-RQ3.4.8 [hostile] A blob whose bytes do not match the offered digest; one longer or shorter than offered → refused at its end; the push's commit refuses.
- T-RQ3.4.9 [exhaustion] An offer for more staged bytes than the principal's quota → refused at the offer, before any byte is sent, saying what remains.
- T-RQ3.4.10 [livelock] Two clients each waiting to commit on a blob the other is uploading → each commit proceeds when its own blobs are present; neither waits on the other.

---

### RQ3.5 Bytes straight between the client and Mantle

Needs: RQ3.1 · Serves: CP-T9.6, AC-T9.1.4

Ask Mantle's owner for a grant its native door honors: a short-lived, signed statement that one
principal may read or write one object of a stated length. With it, a native client's transfers are
`mantle-client` transfers to Mantle's nodes, resumable in runs and held to their checksums, and
Reliquary carries only the control exchanges. Until Mantle honors grants, and wherever a client cannot
reach Mantle's nodes, the same bytes flow through Reliquary's stream lanes in bounded memory. Which
path a transfer takes is decided from what is reachable, and where both are, by measured cost.

Done when:
- AC-RQ3.5.1 The request to Mantle's owner, the grant's shape and Mantle's answer are in the design record.
- AC-RQ3.5.2 Through either path, the bytes a client gets are the same, verified against the blob's digest.
- AC-RQ3.5.3 A grant lets its holder read or write exactly the named object, by the named method, of the named length, until its expiry; anything else is refused by Mantle.
- AC-RQ3.5.4 A grant's life is derived from the transfer it is for (its bytes over the path's learned rate, renewed while the transfer shows progress), never a fixed duration.
- AC-RQ3.5.5 With the relay path, Reliquary's memory per transfer is one piece and the counted overhead.
- AC-RQ3.5.6 An A|B record compares the relay and the direct path on the same hardware, and the default follows the record.

Tests:
- T-RQ3.5.1 [happy] Pull and push the same blob by the relay and by the direct path → identical bytes and digests.
- T-RQ3.5.2 [refusal] Use a read grant to write; use it on another key; use it after expiry; change its length → each refused by Mantle's door.
- T-RQ3.5.3 [edge] A transfer slower than its grant's first life → the grant is renewed while bytes move, and the transfer completes.
- T-RQ3.5.4 [failure] The client cannot reach Mantle's nodes (a firewall) → it falls back to the relay with no error to the caller, and the event says which path it took.
- T-RQ3.5.5 [failure] A Mantle node dies mid-transfer → the client resumes at another node from the durable offset.
- T-RQ3.5.6 [hostile] A client that takes a write grant and uploads other bytes of the same length → the commit's digest check refuses the blob; the object is deleted.
- T-RQ3.5.7 [exhaustion] More relayed transfers than the node's stream credit → held by credit, none dropped, memory within budget.

---

### RQ3.6 The client library and the command line

Needs: RQ3.2, RQ3.3, RQ3.4, RQ3.5 · Serves: AC-T9.1.1

Write `reliquary-client`, the crate Shards, Hex and kiln link: a sans-IO core and a hyper-rt shell.
It resolves, pulls, pushes, tags, lists, deletes and follows events. It keeps a journal of open
transfers, flushed by the platform's rules, so a killed process resumes them. It learns each path's
waits and how many transfers pay, and keeps what it learned by path. It runs on all five targets.
The `reliquary` command gains `push`, `pull`, `cp`, `ls`, `tag`, `rm`, `inspect`, `referrers`,
`login` and `enroll`, through either door.

Done when:
- AC-RQ3.6.1 The library's memory is its stated budget and does not grow with an artifact's size or layer count, shown by a counting-allocator test.
- AC-RQ3.6.2 A process killed at any journal write or protocol step resumes every open transfer, sends no committed byte again, and never commits a wrong artifact.
- AC-RQ3.6.3 A source file changed between attempts is a new push, never a resume into a wrong blob.
- AC-RQ3.6.4 The library carries no fixed timeout, retry count, part size or concurrency: each has a row in the constants register of an accepted kind.
- AC-RQ3.6.5 Every command prints the same lines through the OCI door and the native door.
- AC-RQ3.6.6 `reliquary-api`'s simulator passes the same client suite as a real server, so a consumer can test against it.

Tests:
- T-RQ3.6.1 [happy] Push and pull each artifact kind with the command line through each door → identical output and bytes.
- T-RQ3.6.2 [crash] Kill the client at every journal write and every protocol step of a push and a pull → run again; each completes; committed runs are not re-sent.
- T-RQ3.6.3 [edge] Pull to a file that exists; to a directory that is read-only; with the disk full → replaced atomically; refused typed; refused typed with the partial file removed or kept as the journal says.
- T-RQ3.6.4 [edge] The machine sleeps mid-transfer and wakes → the client reconciles with the server and continues; it trusts no local timer across the sleep.
- T-RQ3.6.5 [failure] The path's rate halves mid-transfer → concurrency follows; no transfer is cut while it progresses.
- T-RQ3.6.6 [race] Two processes of one user push to one tag → both journals end clean; one tag value.
- T-RQ3.6.7 [exhaustion] The journal reaches its bound of open transfers → the next is refused typed, naming the oldest open one.
- T-RQ3.6.8 [hostile] A journal file corrupted on disk → detected by its checksum, that transfer starts over, others resume.
- T-RQ3.6.9 [happy] The client suite against the simulator and against a real node → the same clauses pass.

---

### RQ3.7 Shards over the native door

Needs: RQ3.6, RQ2.3 · Serves: AC-T9.1.1

With Shards' owner, make `shards push`, `shards pull`, `shards ls`, `inspect`, `rm`, and `AGENT`,
`HARNESS`, `MCP` and `SKILL … FROM` a Reliquary reference go over the native door, by linking
`reliquary-client`. A registry says in its `/v2/` answer that it has a native door and where; a
client that hears it and is enrolled uses it, and otherwise speaks the OCI protocol as today.
Reliquary does not edit Shards: this item is the request, the client it needs, the conformance tests
both sides run, and Shards' change vendored when it lands.

Done when:
- AC-RQ3.7.1 The request and Shards' owner's answer are recorded; the discovery rule is in both projects' design records in the same words.
- AC-RQ3.7.2 Every Shards registry command gives the same result through the native door as through the OCI door against the same Reliquary, artifact for artifact and line for line.
- AC-RQ3.7.3 Against a registry with no native door (Docker Hub, GHCR), Shards behaves exactly as before.
- AC-RQ3.7.4 A build that takes an agent, a harness, an MCP server and a skill from Reliquary records each by digest in its provenance.
- AC-RQ3.7.5 Shards' learned waits and transfer shape for the path are kept as Shards keeps them; nothing fixed is added by this change.

Tests:
- T-RQ3.7.1 [happy] `shards push` then `shards pull` of an agent over the native door → identical; the server's counters show no OCI request.
- T-RQ3.7.2 [happy] An Agentfile with `AGENT`, `HARNESS`, `MCP` and `SKILL … FROM` Reliquary → built; the image runs.
- T-RQ3.7.3 [failure] UDP blocked between Shards and Reliquary → Shards uses the OCI door with no error, and says so once.
- T-RQ3.7.4 [failure] The native door goes away mid-pull → the pull resumes over the OCI door from its verified prefix.
- T-RQ3.7.5 [refusal] An unenrolled Shards against a private repository → the OCI door's `401` flow, as before.
- T-RQ3.7.6 [edge] A reference with a tag, with a digest, and with both → resolved as Shards' reference parser states.
- T-RQ3.7.7 [model] Shards' registry test suite run against Reliquary through each door → the same verdicts.

---

### RQ3.8 Hex's nodes get their images from Reliquary

Needs: RQ3.6 · Serves: Hex's CP-T7.8, CP-T9.6

With Hex's owner, give Hex's node agent an image store fed by Reliquary. The node asks for an
artifact by digest in the form it runs: unpacked layers for the `oci` class, an EROFS disk for the
Shards classes, a kernel and root filesystem for Firecracker, a component for a swap. The store
fetches what is missing through `reliquary-client`, verifies it, shares identical content between
artifacts, and holds each for as long as a box uses it. When the disk is short it drops what is
unused, by what each would cost to fetch again. Each node tells Reliquary what it holds, and Hex's
scheduler asks Reliquary what moving an artifact to a node would cost in bytes. A node hears on the
event lane when something it holds is revoked. The seam is a trait in Hex's `-api` with a simulator;
Reliquary provides the real one.

Done when:
- AC-RQ3.8.1 The seam's trait, its simulator and its conformance suite are agreed with Hex's owner and recorded in both projects.
- AC-RQ3.8.2 A box Hex starts from an image only Reliquary holds runs, on Linux, macOS and Windows, for the `oci`, `shards` and `shards-image` classes.
- AC-RQ3.8.3 Content shared by two artifacts is on the node's disk once.
- AC-RQ3.8.4 Nothing a running box uses is ever dropped; under pressure the store drops unused content by re-fetch cost, and refuses typed when nothing can be dropped.
- AC-RQ3.8.5 "Bytes to bring this artifact to this node" answers from the holdings report, and equals what a pull then transfers, within one piece.
- AC-RQ3.8.6 An artifact revoked by policy reaches every node that holds it as an event; a node that missed events reconciles on reconnect.
- AC-RQ3.8.7 The store survives the node agent's crash: after restart it holds exactly what it verified, and nothing half-fetched is usable.

Tests:
- T-RQ3.8.1 [happy] Hex starts an `oci` box, a Shards image VM and a Firecracker VM from Reliquary by digest → each runs.
- T-RQ3.8.2 [edge] Two images sharing a base → the base's layers are fetched once and stored once.
- T-RQ3.8.3 [race] Two boxes ask for one image at once → one fetch; both start.
- T-RQ3.8.4 [race] The store is dropping an image as a box asks for it → the box gets it (the drop yields) or it is fetched again; never a box on a half-removed image.
- T-RQ3.8.5 [exhaustion] The disk fills during a fetch → the fetch is refused typed; unused content is dropped by cost; a retry succeeds if room was made; running boxes are untouched.
- T-RQ3.8.6 [failure] Kill the node agent mid-fetch, restart → the fetch resumes from its verified prefix; boxes started before still run.
- T-RQ3.8.7 [failure] Reliquary is unreachable → boxes whose images the node holds still start; others are refused typed, naming the image.
- T-RQ3.8.8 [hostile] A file in the store altered on disk → detected on its next verification, refetched, and never handed to a box.
- T-RQ3.8.9 [happy] Ask the cost of bringing an image to a node that holds some of its layers, then pull it there → the bytes moved equal the cost answered.
- T-RQ3.8.10 [model] Hex's seam suite against Hex's simulator and against Reliquary → the same clauses pass.

---

### RQ3.9 Two doors, one truth

Needs: RQ3.3, RQ3.4 · Serves: AC-T9.1.2

Prove the doors are two views of one registry. Run the reference model's generated histories with
each operation sent through a door chosen at random, and compare every answer with the model. What
one door writes the other reads at once. A refusal has the same meaning through both.

Done when:
- AC-RQ3.9.1 A history split at random across both doors matches the model on every operation.
- AC-RQ3.9.2 An artifact pushed through one door pulls identically through the other, for every kind.
- AC-RQ3.9.3 Every refusal of the core maps to exactly one OCI error code and one native refusal, in a table a doc-truth test holds to the code.
- AC-RQ3.9.4 The OCI conformance suite still passes at 100% with the native door serving other clients at the same time.

Tests:
- T-RQ3.9.1 [model] Generated histories across both doors, with the nemesis → all match the model.
- T-RQ3.9.2 [happy] Push each artifact kind by Docker or ORAS, pull by `reliquary-client`, and the reverse → identical.
- T-RQ3.9.3 [race] A tag moved through one door while read through the other → every read is a value some linearization allows.
- T-RQ3.9.4 [race] An OCI chunked upload and a native upload of the same blob at once → both succeed; one copy.
- T-RQ3.9.5 [refusal] Each refusal provoked through each door → the table's pair of answers.
- T-RQ3.9.6 [exhaustion] One door driven to its admission limit → the other door's latency stays within its interval while its own classes have credit.
