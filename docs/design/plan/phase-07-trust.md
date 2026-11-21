# Phase 7: Trust, identity and audit

**Goal RQ7.** What Reliquary hands out can be held to a rule: who signed it, that it is the newest
its name should give, that it has an SBOM and a provenance, that nothing known is wrong with it. An
artifact a policy forbids is not served, and kiln refuses to launch an Agentfile whose agent or
harness is not signed by an allowed key. Who pushed is known, with no stored secret wherever a
workload can prove itself; every change has its audit record; and whoever must act on a change is
told of it, with no event lost.

**Delivers.** Signatures, Sigstore's and the Notary Project's, checked as they are attached and kept
as facts; trust roots that cannot be rolled back; signed, expiring statements of what each tag points
at; Rego policies decided at pull; provenance on everything kiln builds; a history of every tag that
anyone can verify; signing on push by rule; identities for workloads, machines and people; an audit
trail; events that are never lost; and a plug for the scanners teams already run.

**Serves.** CP-T9.4 (AC-T9.4.1 to AC-T9.4.3, T-T9.4.1); CP-T9.10 (AC-T9.10.1) through RQ7.11 to
RQ7.15. **Needs.** RQ1. RQ7.7 also needs RQ2.3 and RQ3.6.

**Order.** RQ7.1, RQ7.2, RQ7.8, RQ7.11, RQ7.12 and RQ7.13 first, side by side: each needs only RQ1.
Then RQ7.3, RQ7.4, RQ7.5, RQ7.6 and RQ7.9 side by side; then RQ7.10, RQ7.14 and RQ7.15. RQ7.7
closes.

---

### RQ7.1 Signatures, verified when they arrive

Needs: RQ1 · Serves: AC-T9.4.2

A signature is a referrer of what it signs. When one is pushed, verify it then: a Sigstore bundle
made with a key, or keyless with a certificate, a transparency log entry and a timestamp, by Shards'
verifier, vendored, which is held to sigstore-go. Two generations of the public log are in use at
once, so an entry may be the older kind (a signed entry timestamp) or the newer (a checkpoint with an
inclusion proof, and a timestamp from a timestamp authority); every key the check uses comes from
the trust root (RQ7.2). Whether a log entry is required at all is the trust root's statement, since
a private Sigstore instance may keep no log. Record what was verified as a fact about the subject:
the key or the certificate's identity and issuer, the log entry, the time, and the trust root's
version used. A bundle that does not verify is stored as any referrer is, and its fact says it failed
and why; one of a format version the verifier does not know is marked unverifiable, never failed and
never verified. Signatures kept the older way, under a tag named for the digest, are recognized and
verified the same: cosign wrote that form by default until its third major version, and both are in
the field. Facts are what policies read; a pull never verifies a signature again.

Done when:
- AC-RQ7.1.1 Every bundle sigstore-go's own test vectors accept is recorded as verified, and every one they reject as failed, with the same reason class.
- AC-RQ7.1.2 A signature made by `cosign sign` and by `shards sign`, with a key and keyless, pushed with or without the referrers API, yields a verified fact naming the right signer.
- AC-RQ7.1.3 A fact names the trust root's version it was verified under; when a root or a key is revoked, the facts that rest on it are re-evaluated, and the count re-evaluated is reported.
- AC-RQ7.1.4 Verifying a signature makes no network request at push: the log's inclusion and the timestamp are checked from the bundle and the cached root.
- AC-RQ7.1.5 A signature's subject need not exist yet, and its fact attaches when the subject arrives.
- AC-RQ7.1.6 A bundle whose log entry is of the public log's older generation and one of its newer generation both verify, and every key used comes from the trust root: none is in the code.
- AC-RQ7.1.7 Whether a log entry is required is stated per trust root; where it is not, a signed timestamp from an authority the root names is required instead, as Sigstore's client specification says.
- AC-RQ7.1.8 A bundle of a format version the verifier does not know yields the fact "unverifiable", naming the version, and is re-evaluated when a verifier for it ships.

Tests:
- T-RQ7.1.1 [happy] Sign an agent with a key, push the signature → the agent's facts list that key as verified.
- T-RQ7.1.2 [happy] A keyless signature with a certificate, a log entry and a timestamp → verified, with its identity and issuer.
- T-RQ7.1.3 [refusal] A bundle whose signature is over another digest; whose certificate had expired at the timestamp; whose log entry's inclusion proof is wrong; whose key is not the stated one; that carries no trusted time; whose certificate lacks its certificate-transparency timestamp or carries a bad one → each recorded as failed, with its reason.
- T-RQ7.1.4 [edge] The signature pushed before its subject; two signatures by one key; a signature of an index and of one of its members → attached when the subject arrives; one fact; each on its own subject.
- T-RQ7.1.5 [edge] A signature under the `sha256-<hex>.sig` tag form, and one listed under the referrers fallback tag `sha256-<hex>` → each recognized, verified, and listed with the referrers.
- T-RQ7.1.6 [hostile] A bundle of nested JSON larger than its budget; a certificate chain longer than its budget; a malformed protobuf-JSON → refused in bounded time and memory.
- T-RQ7.1.7 [race] A signature pushed while its subject is being deleted → attached to nothing, or the delete is refused by policy; never a fact on a missing subject.
- T-RQ7.1.8 [model] Sigstore's conformance vectors through Shards' verifier as vendored → the published verdicts.
- T-RQ7.1.9 [edge] Sign one image with cosign's current release in its default form (a bundle as a referrer) and with its previous major release (a tag), in one repository → both verified; the subject shows both facts.
- T-RQ7.1.10 [edge] A bundle of an unknown format version; a keyless signature from an instance that keeps no log, under a root that requires a log entry and under one that does not → "unverifiable", naming the version; failed; verified by its timestamp.

---

### RQ7.2 Trust roots that cannot be rolled back

Needs: RQ1 · Serves: T-T9.4.1, AC-T9.4.2

The keys and logs a keyless signature is checked against come from a trust root, and the root itself
is fetched. Fetch it by The Update Framework, with Shards' client, vendored, which is held to go-tuf:
from a root carried in the binary, through every later root in order, with the timestamp, snapshot
and targets each checked for signatures, version and expiry. Keep the last trusted metadata in the
records, so no node accepts an older one. Support the public Sigstore root and an operator's own. A
root that has expired and cannot be refreshed makes facts stale, and a policy can say what stale
means for it.

Done when:
- AC-RQ7.2.1 A rollback attack with old TUF metadata is refused (T-T9.4.1): for each role, metadata of a lower version than the last trusted is rejected, on every node.
- AC-RQ7.2.2 The client's verdicts equal go-tuf's on the TUF conformance suite and on Shards' recorded oracle.
- AC-RQ7.2.3 Freeze (expired metadata replayed), mix-and-match (a snapshot that disagrees with its targets) and an endless-data attack are each refused, typed.
- AC-RQ7.2.4 When the root cannot be refreshed past its expiry, facts verified under it are marked stale from that instant, and the event says which root.
- AC-RQ7.2.5 How often the root is refreshed is derived from the metadata's own expiry and the path's learned availability, with no fixed period.

Tests:
- T-RQ7.2.1 [refusal] Serve an older timestamp, snapshot, targets and root in turn → each refused; the trusted version does not move back.
- T-RQ7.2.2 [refusal] Serve expired metadata; a snapshot naming a targets version the repository does not serve; metadata signed by a key the root does not name; too few signatures for the threshold → each refused.
- T-RQ7.2.3 [happy] A root rotated twice since the binary was built → the client walks both and trusts the newest.
- T-RQ7.2.4 [failure] The TUF repository is unreachable for longer than the metadata lives → facts go stale at expiry; pulls follow the policy's stale rule; nothing is trusted past it.
- T-RQ7.2.5 [hostile] A metadata file that never ends; one of deeply nested JSON; a targets file listing a path that escapes its directory → refused at the byte budget; refused; refused.
- T-RQ7.2.6 [race] Two nodes refresh at once and one sees a newer version → both end at the newer; neither accepts the older afterwards.
- T-RQ7.2.7 [crash] Power cut while the trusted metadata is being replaced → after restart the old or the new is trusted, whole, never neither.

---

### RQ7.3 A signed statement of what each tag points at

Needs: RQ7.2 · Serves: T-T9.4.1, AC-T9.4.2

A registry that is compromised or behind can serve an old digest for a tag. Let clients detect it.
Reliquary publishes TUF metadata of its own for each namespace: targets that say which digest each
tag names, a snapshot of all targets, and a short-lived timestamp, each signed. Keys for the roles
that sign often come through the `Secrets` seam; the root's keys are the operator's, offline.
`reliquary-client`, and so Shards, Hex and kiln, check a tag's digest against this metadata and
refuse an older answer. Publishing a tag makes the new statement available within the same bound as
the tag itself.

Done when:
- AC-RQ7.3.1 A client that has seen a tag at one digest refuses a later answer whose metadata is older, on any node or region (T-T9.4.1).
- AC-RQ7.3.2 A stock TUF client (go-tuf) verifies Reliquary's metadata from its root with no special case.
- AC-RQ7.3.3 After a tag moves, a verifying client sees the new digest with valid metadata no later than a plain client sees the new tag, measured as a difference of intervals.
- AC-RQ7.3.4 The timestamp's life is the operator's statement of how stale a tag may be; nothing in the code picks one.
- AC-RQ7.3.5 Signing keys never leave the `Secrets` seam's holder; Reliquary holds a lease to sign, not a key; a lost lease stops new statements and is told as an event.
- AC-RQ7.3.6 The metadata for a namespace of the largest scale size is produced, served and verified in bounded memory, by delegations whose size the design derives.

Tests:
- T-RQ7.3.1 [refusal] Replay to a client yesterday's timestamp and tag digest → refused as rolled back.
- T-RQ7.3.2 [refusal] Serve a current timestamp with an old snapshot; a targets file missing a tag the client saw → refused.
- T-RQ7.3.3 [happy] Move a tag; a verifying client pulls → the new digest, verified.
- T-RQ7.3.4 [failure] The signing lease is lost → existing statements serve until they expire; tag moves are refused or wait, by the stated rule; an event names the cause.
- T-RQ7.3.5 [race] Two tags of one namespace move at once on two nodes → one snapshot covers both; no client sees a snapshot that names one move and metadata for the other.
- T-RQ7.3.6 [crash] Power cut between committing a tag and publishing its statement → after restart the statement is published before the tag is served to verifying clients.
- T-RQ7.3.7 [exhaustion] A namespace at the largest size the scale phase tests → metadata operations stay within their memory budget.
- T-RQ7.3.8 [hostile] A client that asks for metadata versions in a loop → served from memory within its share; signing is never triggered by a read.
- T-RQ7.3.9 [deadlock] A tag move waiting for its statement while signing waits for the snapshot that includes the move → the order is fixed (commit, snapshot, sign) and no step waits on a later one; the move completes or is refused typed.
- T-RQ7.3.10 [livelock] Tags of one namespace moving faster than a snapshot can be signed → each snapshot covers every move committed before it; signing does not restart for each move, and the backlog drains when moves stop.

---

### RQ7.4 Policy at pull

Needs: RQ7.1 · Serves: AC-T9.4.2

Let an operator say what may be pulled, in Rego, evaluated by Shards' engine, vendored, which is
held to OPA. A policy belongs to a namespace or a repository. Its input is the artifact's facts:
kind, signers, whether its signatures are stale, SBOM and provenance, known vulnerabilities and how
fresh that knowledge is, age, asks. A pull, by either door, of a manifest the policy denies is
refused `DENIED` with the rule that denied it; so is a pull of its blobs through that manifest's
repository. A decision is computed when a fact or the policy changes, not at each pull. A policy can
run in report-only mode first. Policies are versioned, and a change takes effect at its commit on
every node.

Done when:
- AC-RQ7.4.1 Unsigned, wrongly signed or stale artifacts accepted where policy forbids them is exactly 0, measured by a test of planted cases (AC-T9.4.2).
- AC-RQ7.4.2 Each decision equals OPA's on the same policy and input, over the policy test corpus.
- AC-RQ7.4.3 A pull's added cost with policies on is a lookup: no Rego runs in a pull's path, by counter.
- AC-RQ7.4.4 A policy change or a fact change is reflected in the next pull after its commit, on every node.
- AC-RQ7.4.5 A policy that errors, does not terminate within its counted budget, or returns no decision denies, and says so.
- AC-RQ7.4.6 A refusal names the policy, its version and the rule, and never reveals facts about artifacts the caller may not read.
- AC-RQ7.4.7 Report-only mode records what would have been denied and denies nothing.

Tests:
- T-RQ7.4.1 [refusal] Planted cases under "must be signed by key K": unsigned; signed by another key; signed by K over another digest; signed by K with stale facts where the policy forbids stale → each pull `DENIED`.
- T-RQ7.4.2 [happy] The same artifact signed by K → pulled.
- T-RQ7.4.3 [edge] An index allowed while one member is denied; a manifest denied but pulled by digest; a blob of a denied manifest shared with an allowed one → the member denied; denied; the blob served only through the allowed manifest's repository.
- T-RQ7.4.4 [race] A signature attached while a pull is in flight; a policy tightened mid-pull → the pull sees the decision of one instant; later pulls the new one.
- T-RQ7.4.5 [failure] The decision record is unavailable → pulls under a policy are refused typed, never allowed by default.
- T-RQ7.4.6 [hostile] A policy with unbounded recursion-like rules, a regex bomb, or a gigabyte comprehension → stopped at its budget, denied, and reported to the policy's owner.
- T-RQ7.4.7 [exhaustion] A policy change that re-decides every artifact in a large namespace → decided in the background within its budget; until an artifact is re-decided its pull is evaluated on demand or denied, by the stated rule; pulls elsewhere are unaffected.
- T-RQ7.4.8 [model] The policy corpus through OPA and through the vendored engine → identical decisions.
- T-RQ7.4.9 [livelock] Facts changing while a policy change re-decides the same artifacts → each decision is for one revision of facts and policy; the queue drains when changes stop; no pull waits on the backlog.
- T-RQ7.4.10 [deadlock] A re-decision that needs a fact whose verification waits on a trust-root refresh that the policy itself gates → the refresh never passes through policy; the decision completes or denies typed.

---

### RQ7.5 SBOMs, vulnerabilities and exceptions

Needs: RQ7.1 · Serves: AC-T9.4.2

A policy can ask that an artifact has an SBOM and that nothing critical is known against it. Accept
SBOMs as referrers, in SPDX and CycloneDX, and check each parses and names its subject. An SBOM is as
often attached as an attestation as under its own type, and attestations of every kind share one
artifact type, so find it by its predicate type: read from the referrer's annotation or, lacking
one, from the statement. Match an SBOM's packages against an advisory database kept in Reliquary and
refreshed from its publishers through the pull-through path. Record the result as facts, with the
database's date. Where an advisory's affected versions cannot be evaluated here (an ecosystem with no
comparator, a range that names git commits), the fact says unknown, never clean. A VEX statement,
signed and attached as a referrer, can say a vulnerability does not apply, and its signer decides
whether a policy believes it. Reliquary does not scan images itself (RQ7.15 lets a team's own scanner
do it): an artifact with no SBOM and no scan has no vulnerability facts, and a policy can refuse
that.

Done when:
- AC-RQ7.5.1 An SBOM referrer that is not valid SPDX or CycloneDX, or names another subject, is recorded as invalid and gives no "has SBOM" fact.
- AC-RQ7.5.2 On a corpus of SBOMs with known advisories, every advisory that applies is found and none that does not, compared with the advisory publishers' own matching rules.
- AC-RQ7.5.3 When the advisory database changes, the facts of every affected artifact are updated, and only those.
- AC-RQ7.5.4 A fact says how old its advisory data is, and a policy can deny on age.
- AC-RQ7.5.5 A VEX statement changes a decision only where the policy trusts its signer.
- AC-RQ7.5.6 An SBOM attached as an attestation is found by its predicate type, from the referrer's annotation or else from the statement; each referrer's fact records that type, so the SBOM is found without fetching the others.
- AC-RQ7.5.7 An advisory range that cannot be evaluated gives "unknown", saying why; unknown is never reported as not affected, and a policy states what unknown means.
- AC-RQ7.5.8 A VEX "not affected" with neither a justification nor an impact statement is invalid and changes no decision, as OpenVEX requires.

Tests:
- T-RQ7.5.1 [happy] An image with an SBOM listing a package with a critical advisory, under "no critical vulnerability" → `DENIED`, naming the advisory.
- T-RQ7.5.2 [happy] The same image with a trusted VEX "not affected"; with an untrusted one → pulled; `DENIED`.
- T-RQ7.5.3 [edge] A version range at each boundary (first affected, last affected, fixed); a package named by different ecosystems' conventions; an SBOM of zero packages → matched as the publishers' rules state.
- T-RQ7.5.4 [failure] The advisory source is unreachable → facts age; the policy's age rule decides; nothing is treated as clean because data is missing.
- T-RQ7.5.5 [race] An advisory arrives while an artifact is being pulled → this pull or the next is denied; none after the fact's commit is allowed.
- T-RQ7.5.6 [hostile] An SBOM of more packages than its budget; one with cyclic relationships; one that is a decompression bomb → bounded by its budget, refused typed.
- T-RQ7.5.7 [edge] The same SBOM attached under its own media type, as a cosign attestation and as BuildKit writes it → "has SBOM" in each case, from the same packages.
- T-RQ7.5.8 [edge] An advisory with a semantic-version range; one in an ecosystem's own ordering; one that names git commits → matched; matched by that ecosystem's comparator, or "unknown" where there is none; "unknown".
- T-RQ7.5.9 [refusal] A signed, trusted VEX "not affected" that gives no justification and no impact statement → invalid; the pull stays `DENIED`.

---

### RQ7.6 Provenance on everything kiln builds

Needs: RQ7.1 · Serves: AC-T9.4.3

Kiln's CI already attests what it builds. Take each attestation as a referrer, verify it as a
signature is verified, and record who built the artifact, from which source at which commit, by
which pipeline. Provenance is a SLSA statement: match its predicate type exactly, ignore fields a
later minor version adds, and require the artifact's digest among the statement's subjects. A
builder's name is believed only from a signer the operator allows to speak for it. Read attestations
in every form in use: a bundle as a referrer, cosign's older `.att` tag, and BuildKit's attestation
manifests, which are referrers and also members of the image's index. Give kiln a check that walks
the registry and reports every artifact kiln built that lacks a verified provenance. A policy can
require provenance from a named builder.

Done when:
- AC-RQ7.6.1 Kiln-built artifacts without provenance is exactly 0, measured by a check over the registry (AC-T9.4.3).
- AC-RQ7.6.2 A provenance whose subject, builder identity or signature does not verify gives no provenance fact.
- AC-RQ7.6.3 The check's cost per run does not grow with the registry's size beyond what changed since its last run.
- AC-RQ7.6.4 The artifact page shows the source repository, commit and pipeline of a build, linked to kiln's own record of the run.
- AC-RQ7.6.5 A provenance fact needs the SLSA predicate type exactly, the artifact's digest among the statement's subjects, and a signer the operator allows to speak for the named builder; fields the parser does not know are ignored, not refused.
- AC-RQ7.6.6 An attestation is found in each form in use: a bundle as a referrer, cosign's `.att` tag, and BuildKit's attestation manifest as a referrer and as an index member of platform `unknown/unknown`; one reachable two ways is counted once, and it is never offered as a platform to run.

Tests:
- T-RQ7.6.1 [happy] Kiln CI builds and pushes an image → its provenance is attached and verified; the check reports nothing missing.
- T-RQ7.6.2 [refusal] An artifact pushed to a kiln-built namespace by hand with no attestation; with an attestation for another digest; with one signed by another builder → each reported by the check, and denied where a policy requires provenance.
- T-RQ7.6.3 [edge] A multi-platform build → each platform's manifest and the index have provenance.
- T-RQ7.6.4 [race] The check runs while a build's artifact has arrived and its attestation has not → reported only if the attestation is still missing after the push's commit completes.
- T-RQ7.6.5 [refusal] A provenance naming kiln's builder, validly signed by an identity not allowed to speak for it; one whose predicate type differs by one character; one whose subjects omit the artifact → no provenance fact, each with its reason.
- T-RQ7.6.6 [edge] A provenance carrying a field from a later minor version; an image built by BuildKit in its current and in its older attestation form; an attestation under an `.att` tag → accepted; found in both, and no listing offers an `unknown/unknown` entry to run; found.
- T-RQ7.6.7 [edge] An attestation manifest that is both an index member and a referrer, after its index is deleted → still held by its subject, counted once; collected only when RQ9's rule says nothing holds it.

---

### RQ7.7 Kiln refuses to launch what is not signed

Needs: RQ7.4, RQ2.3, RQ3.6 · Serves: AC-T9.4.1

Put it to use, and do not make the registry the only guard. When kiln launches an Agentfile, each
agent and harness it names is resolved through `reliquary-client`, which verifies the tag's statement
(RQ7.3) and the signature against the keys kiln allows, on the launching machine. An agent or harness
that fails is not launched, and the refusal says which and why. The registry's own policy refuses the
same pull, so either guard alone would have stopped it.

Done when:
- AC-RQ7.7.1 Kiln refuses to launch an Agentfile whose agent or harness isn't signed by an allowed key, in a change request that passes the gate (AC-T9.4.1).
- AC-RQ7.7.2 With the registry's policy turned off, the client's check alone refuses the launch; with the client's check turned off in a test build, the registry's policy alone refuses the pull.
- AC-RQ7.7.3 A launch of signed members adds no network round trip beyond the pull's own.
- AC-RQ7.7.4 The refusal reaches the person in kiln's words, naming the member, its digest and the keys that would have been accepted.

Tests:
- T-RQ7.7.1 [refusal] An Agentfile naming an unsigned agent; one signed by a key kiln does not allow; one whose signature is of an older digest of the same tag → not launched, each named.
- T-RQ7.7.2 [happy] The same Agentfile with every member signed by an allowed key → launched.
- T-RQ7.7.3 [refusal] A registry (simulated) that answers an old digest for the tag with old metadata → the client refuses it as rolled back.
- T-RQ7.7.4 [edge] An Agentfile pinning members by digest → the signature of that digest is checked; the tag's statement is not needed.
- T-RQ7.7.5 [failure] The trust root is stale beyond the policy's rule → the launch is refused, saying the root is stale.

---

### RQ7.8 A history of every tag that anyone can verify

Needs: RQ1 · Serves: AC-T9.4.2

Keep an append-only log of every tag change and every delete, in the order the records committed
them, in the tiled, hash-tree form transparency logs use, with a signed checkpoint. Anyone can check
that an entry is in the log and that the log only grew. A monitor that watches a namespace sees every
change to it. The log is built from the change feed; it is evidence, not the source of truth.

Done when:
- AC-RQ7.8.1 Every committed tag change and delete has exactly one entry, in commit order; a comparison of the log with the records' feed finds no gap and no extra.
- AC-RQ7.8.2 Inclusion and consistency proofs verify with an independent implementation of the published log format.
- AC-RQ7.8.3 A checkpoint is signed through the `Secrets` seam and never covers an entry that is not durable.
- AC-RQ7.8.4 The log's serving cost per read does not grow with its length, and a tile, once complete, never changes.

Tests:
- T-RQ7.8.1 [happy] Move a tag three times → three entries; each proves included; each pair of checkpoints proves consistent.
- T-RQ7.8.2 [refusal] A checkpoint for a smaller tree after a larger one was seen; an entry altered in a served tile → a monitor's check fails on each.
- T-RQ7.8.3 [crash] Power cut between a tag's commit and its log entry → after restart the entry is appended before a new checkpoint is signed.
- T-RQ7.8.4 [race] Tags of many namespaces moving at once on several nodes → one total order, the feed's.
- T-RQ7.8.5 [exhaustion] A log of the scale phase's length → proofs and tiles served within their bound.
---

### RQ7.9 Notary Project signatures

Needs: RQ7.1 · Serves: AC-T9.4.2

ECR and ACR sign with the Notary Project's format, and zot verifies it. Verify it too. A Notary
signature is a referrer whose artifact type names it (older signers name it only in the config's
media type), holding one JWS or COSE envelope. Check the envelope, its certificate chain against a
trust store the operator states, the signer against the identities the operator trusts, and the
signing time against the envelope's countersigned timestamp where it has one. Record the result as a
fact beside Sigstore's, so one policy can ask for either. Whether a certificate has been revoked is
fetched outside the push, kept with its date, and a policy says how old that knowledge may be.

Done when:
- AC-RQ7.9.1 On a corpus of signatures made by notation, valid and invalid in each way its specification names, in both envelope formats, our verdicts equal notation's own, with the same reason class.
- AC-RQ7.9.2 A signature made by `notation sign`, in the current form and in the older form that names its type only in the config, is listed by the referrers API under the Notary artifact type and yields a verified fact naming the signer's certificate subject.
- AC-RQ7.9.3 A signature verifies only under a trust store and a trusted identity the operator stated for its repository; no certificate is trusted because the machine's own store trusts it.
- AC-RQ7.9.4 Verifying at push makes no network request; revocation data is fetched outside the push, its date is part of the fact, and a fact is re-evaluated when that data changes.
- AC-RQ7.9.5 A policy can require a signature of either kind, or of both, by one rule over facts.

Tests:
- T-RQ7.9.1 [happy] Sign an image with `notation sign` under a test authority and push → the fact names the certificate's subject as verified.
- T-RQ7.9.2 [refusal] A signature whose chain ends at an authority not in the store; whose signer is not a trusted identity; whose certificate had expired at the signing time; whose envelope was altered; whose payload names another digest → each recorded as failed, with its reason.
- T-RQ7.9.3 [edge] A JWS envelope and a COSE envelope of one subject; a signature in the older form; a signature whose certificate has since expired but whose countersigned timestamp predates the expiry → both verified; listed under the Notary type; verified.
- T-RQ7.9.4 [failure] The revocation source cannot be reached → the fact says how old its revocation data is; the policy's age rule decides; nothing is treated as unrevoked because data is missing.
- T-RQ7.9.5 [hostile] An envelope larger than its budget; a certificate chain longer than its budget; a COSE structure nested past its budget; a certificate with a malformed extension → refused in bounded time and memory.
- T-RQ7.9.6 [race] A certificate revoked while its signature is being verified → the fact reflects one instant; after the revocation's commit the fact is failed on every node.
- T-RQ7.9.7 [model] The notation-made corpus through our verifier and through notation → identical verdicts.

---

### RQ7.10 Signing on push, by the operator's rule

Needs: RQ7.1, RQ7.3 · Serves: AC-T9.4.2

ECR signs what is pushed, so a team need not run a signer in every pipeline. Offer the same, and say
what it means. A namespace's rule names which pushes are signed and with which key; the key stays
behind the `Secrets` seam, as RQ7.3's do. The signature is a Sigstore bundle attached as a referrer,
which cosign verifies with the namespace's public key. The artifact and its signature become visible
in one commit, and a push that cannot be signed is refused. The signature says only that Reliquary
accepted this artifact from this principal under this rule: its fact is marked as the registry's own,
and a policy can tell it from a signature made where the artifact was built.

Done when:
- AC-RQ7.10.1 Under a signing rule an artifact and its signature become visible in one commit: no reader sees the artifact unsigned, and when the signing lease is lost the push is refused typed.
- AC-RQ7.10.2 `cosign verify --key` with the namespace's public key accepts the signature, with no step that is Reliquary's own.
- AC-RQ7.10.3 The signed statement and the fact name the rule and its version, the principal that pushed, and the time; the fact is marked as made by the registry, and a policy can require a signature that is not.
- AC-RQ7.10.4 The key never leaves the `Secrets` seam's holder; Reliquary holds a lease to sign; no signature is made for an artifact the admission checks refused.
- AC-RQ7.10.5 A push under a signing rule costs one signing operation more than one without, by counter, and an A|B record states what that adds to its latency.

Tests:
- T-RQ7.10.1 [happy] Push an image to a namespace with a signing rule → its referrers list a signature; `cosign verify --key` accepts it.
- T-RQ7.10.2 [refusal] Push while the signing lease is lost → refused typed, naming the rule; nothing of the push is visible; after the lease returns the same push succeeds.
- T-RQ7.10.3 [edge] A push that matches no rule; an index and its members under one rule; the same manifest pushed twice; a client's own signature pushed under a signing rule → unsigned; each signed as the rule says; one signature, not two; stored as the client's own and not countersigned.
- T-RQ7.10.4 [refusal] A policy requiring a signature made outside the registry, on an artifact that has only the registry's → `DENIED`.
- T-RQ7.10.5 [race] The rule removed while a push it covered is in flight → the push is signed under the rule's version it began with, or unsigned under the new one, and its fact says which.
- T-RQ7.10.6 [crash] Power cut after signing and before the commit → after restart neither the artifact nor a stray signature is visible; a retry signs again.
- T-RQ7.10.7 [exhaustion] Pushes arriving faster than the signer signs → admitted at the signer's measured rate, the rest refused `429` with `Retry-After`; pushes to namespaces with no rule are unaffected.

---

### RQ7.11 Machines that push with no stored secret

Needs: RQ1 · Serves: AC-T9.10.1

A pipeline that holds a registry password can leak it. Let a workload prove who it is instead, as
zot, Quay and the hosted registries now do. An operator states a trust rule: an OpenID Connect
issuer, the audience a token must name, and which of its claims earn which grants. A CI job presents
its own token at the token endpoint and receives a registry token for exactly those grants, living
no longer than the job's token. A client certificate under an authority the operator states does the
same on the OCI door, as certificates already do on the native one. An issuer's keys are fetched
outside any request and kept for the lifetime the issuer gives them.

Done when:
- AC-RQ7.11.1 A token from a stated issuer whose audience and claims match a rule is exchanged for a registry token carrying exactly the rule's grants; one that fails on issuer, audience, signature, expiry or claims is refused `401`, saying which check failed and nothing about the rules.
- AC-RQ7.11.2 The audience is always checked: a rule without one cannot be stated.
- AC-RQ7.11.3 A registry token made by exchange expires no later than the token it was exchanged for.
- AC-RQ7.11.4 GitHub Actions, GitLab CI and a Kubernetes service account each push and pull with their platform's own token and no stored secret, through `docker login` or a credential helper, with no change to the client.
- AC-RQ7.11.5 A rule can match on any claim of the token, not the subject alone, so a repository can be named by the immutable ID its issuer gives; a rule that names an ID is not matched by another repository re-created under the same name.
- AC-RQ7.11.6 An issuer's keys are refreshed outside any request, by the lifetime the issuer states; a key rotation is followed without a failed exchange, and an unreachable issuer never makes a token valid.
- AC-RQ7.11.7 A client certificate under a stated authority names a principal on the OCI door by a stated field (its subject, a DNS name or a URI name), and a revoked one is refused from the revocation's commit on.

Tests:
- T-RQ7.11.1 [happy] A CI job with its platform's token and no secret configured → logs in, pushes and pulls.
- T-RQ7.11.2 [refusal] A token for another audience; from an issuer not stated; signed by a key the issuer does not publish; expired; with `alg: none`; whose claims match no rule → each `401`, naming the failed check.
- T-RQ7.11.3 [refusal] A rule naming a repository by its ID, and a token from a repository deleted and re-created under the same name → refused.
- T-RQ7.11.4 [edge] A job's token about to expire; a rule granting two repositories and a request for one; two rules that both match → the registry token expires with it; only the asked scope is granted; the grants are the union, and the answer names both rules.
- T-RQ7.11.5 [failure] The issuer's key endpoint is unreachable across a rotation → exchanges signed by keys still held succeed until those keys' lifetime ends; ones signed by the new key are refused typed until it is fetched; none is accepted unverified.
- T-RQ7.11.6 [hostile] An issuer document that redirects to an internal address; a key set larger than its budget; a token whose header names a key URL of its own; a flood of tokens each naming a different unknown key → the fetch is refused; refused at the budget; the URL is ignored; no fetch is made per token beyond what the refresh rule allows.
- T-RQ7.11.7 [race] A rule removed while an exchange is in flight → the exchange is decided by one version of the rules; tokens already made live out their short lives, and no new one is made.
- T-RQ7.11.8 [edge] A client with a certificate under the stated authority; one with a certificate from another authority → pushes over the OCI door with no token; refused in the handshake or `401`, never served.

---

### RQ7.12 Credentials for machines, and sign-in for people

Needs: RQ1 · Serves: AC-T9.10.1

Not every machine can prove who it is, and people do not carry certificates. Give an operator
machine identities like Harbor's and Quay's robot accounts: each belongs to a namespace, holds grants
on named repositories, and has credentials that expire, can be rotated with both valid for a stated
overlap, and can be revoked. Let people sign in through the operator's OpenID Connect issuer:
`reliquary login`, and the credential helper Docker calls, run the issuer's device flow, and group
claims earn grants. A directory reached only by LDAP is put behind such an issuer; Reliquary does not
speak LDAP itself, so that it has one sign-in protocol and one parser to defend.

Done when:
- AC-RQ7.12.1 A machine identity can do only what its grants name, on the repositories they name; it cannot sign in as a person, create identities or change grants.
- AC-RQ7.12.2 A credential's expiry must be stated when it is made; it is generated by Reliquary with as much entropy as its hash holds, shown once, stored only as that hash, and never logged.
- AC-RQ7.12.3 A revoked or expired credential makes no new token from that commit on, on every node; tokens it already made end within the token lifetime of RQ1.7.
- AC-RQ7.12.4 Rotation gives a second credential before the first ends, and the overlap is the operator's statement.
- AC-RQ7.12.5 A person signs in with `reliquary login` and with Docker's credential helper through the issuer's device flow; the registry never sees the person's password, and group claims earn grants by the operator's rule.
- AC-RQ7.12.6 Removing a person from a group at the issuer takes effect no later than the end of their session's stated life, and a session cannot be extended without the issuer.
- AC-RQ7.12.7 Each identity's last use is recorded with no write on the request's path, so an unused credential can be found.

Tests:
- T-RQ7.12.1 [happy] Make a machine identity with pull on one repository; pull with it; push with it → pulled; `403` `DENIED`.
- T-RQ7.12.2 [refusal] Use a credential after its expiry; after its revocation; a credential of one namespace on another; a machine credential at the sign-in for people → each refused.
- T-RQ7.12.3 [edge] Rotate a credential; make one with no expiry stated → both work during the overlap and only the new one after; refused, naming the missing expiry.
- T-RQ7.12.4 [happy] `reliquary login` against a test issuer, approved on another device, then `docker pull` → pulled with the person's grants; the registry's records hold no password.
- T-RQ7.12.5 [failure] The issuer is unreachable → sessions in being last their stated life; new sign-ins are refused typed; machine credentials and workload exchanges are unaffected.
- T-RQ7.12.6 [hostile] Guess credentials for one identity as fast as a core allows → each guess costs one hash and is counted; the limiter of RQ4.3 holds the source to its share; no answer shows whether the identity exists.
- T-RQ7.12.7 [race] A credential revoked on one node while it is used on another → no token is made on any node after the revocation's commit.
- T-RQ7.12.8 [edge] A person in more groups than the issuer's token carries, where the issuer says the list was cut → the groups are fetched from the issuer or the sign-in is refused typed; grants are never computed from a list known to be cut.

---

### RQ7.13 Events others can act on, with none lost

Needs: RQ1 · Serves: AC-T9.10.1

Distribution's and GitLab's registries tell subscribers of pushes from a queue in memory, and say
themselves that events are lost when a node stops or the queue fills. Deliver events from the
records' change feed instead. Every push, tag move, delete, denial of a pull by policy, and new fact
(a signature verified, an advisory matched) is an event; each subscriber has a durable cursor;
delivery is at least once, in commit order within a repository. Speak three ways: Distribution's own
notification envelope by webhook, so existing receivers work unchanged; `Topics` for kiln; and the
native door's event lane. A subscriber sees only what its principal may read. A webhook is retried
by RQ0.8's waits. A subscriber that falls behind the feed's kept history is told so, with the point
to list again from, and is never skipped ahead in silence.

Done when:
- AC-RQ7.13.1 Every committed change within a subscription's filter is delivered at least once across node loss, restart and upgrade: with the nemesis running, a comparison of what a subscriber received with the feed finds no gap.
- AC-RQ7.13.2 Events of one repository arrive in commit order, and each carries an ID that is the same on redelivery.
- AC-RQ7.13.3 A receiver written for Distribution's notifications accepts Reliquary's webhook envelope unchanged.
- AC-RQ7.13.4 No event is held only in memory: a subscriber's backlog is its cursor, so its memory cost on a node is bounded whatever its lag.
- AC-RQ7.13.5 A subscriber whose cursor is older than the feed's history is answered `GONE` with the revision to list again from; no event is dropped without that answer.
- AC-RQ7.13.6 A webhook request is signed with a key from the `Secrets` seam, carries no credential of the registry, and goes only to addresses the operator's rule allows; the address is resolved once and held for the request.
- AC-RQ7.13.7 No delivery wait, retry count or batch size is a picked number: waits are RQ0.8's, and a subscriber that never answers costs a bounded, counted amount.
- AC-RQ7.13.8 An event never shows a repository, a tag or a digest to a subscriber whose principal may not read it.

Tests:
- T-RQ7.13.1 [happy] Subscribe by webhook, by topic and on the event lane; push, move a tag, delete → each receives the three events, in order, in its own form.
- T-RQ7.13.2 [failure] The receiver is down while changes commit, then returns → every event since its cursor arrives, in order; none is repeated except the one in flight when it went down.
- T-RQ7.13.3 [crash] Kill the delivering node between a delivery and the cursor's advance → another node redelivers that event with the same ID and carries on.
- T-RQ7.13.4 [edge] A subscriber lags past the feed's kept history → `GONE` with a revision; after listing again from it the subscriber's state equals the registry's.
- T-RQ7.13.5 [hostile] A webhook address that resolves to loopback, to a link-local or a metadata address, or that changes its answer between check and connect; a receiver that answers just often enough to look alive; one that redirects elsewhere → refused by the address rule; cut by progress and retried by the learned wait; the redirect is not followed.
- T-RQ7.13.6 [exhaustion] More subscriptions than the node's delivery budget → admitted to the budget, the rest refused typed when they subscribe; existing deliveries keep their order.
- T-RQ7.13.7 [livelock] A receiver that fails every delivery while changes keep committing → its retries do not grow in rate; its cursor stays; other subscribers are unaffected; an event names the stuck subscription.
- T-RQ7.13.8 [race] A repository's grant removed from a subscriber's principal while events for it wait → none committed after the removal is delivered.
- T-RQ7.13.9 [deadlock] Delivery waiting on the feed while the feed's compaction waits on the slowest cursor → compaction never waits on a subscriber; the laggard is answered `GONE`.

---

### RQ7.14 An audit trail

Needs: RQ7.8, RQ7.13 · Serves: AC-T9.10.1

Say who did what. Every change (a push, a tag move, a delete, a grant, a policy, a rule, a credential
made or revoked) carries its audit record in the same commit as the change, so there is never one
without the other, and a pull writes nothing. Harbor writes to its database on every pull and offers
a switch to stop; here the question does not arise. Denials and sign-ins are recorded in batches off
the request's path, each kind counted exactly even when their detail is shed under load. A namespace
may ask that pulls are recorded too, and then a pull that cannot be recorded is refused. A record
names the principal, how it proved itself, the action, the object and the outcome, never a secret.
Records are read by cursor, streamed to subscribers (RQ7.13), kept as long as the operator states,
and entered in a log of RQ7.8's form, so a removed or altered record is detectable.

Done when:
- AC-RQ7.14.1 Every committed change has exactly one audit record, written in the change's own transaction; a comparison of the trail with the records' feed finds no gap and no extra.
- AC-RQ7.14.2 A manifest or blob `GET` in a namespace that does not audit pulls performs no write on its path, by counter.
- AC-RQ7.14.3 In a namespace that audits pulls, every served pull has a record, and a pull whose record cannot be queued within its budget is refused typed; an A|B record states the cost this adds to a pull.
- AC-RQ7.14.4 Denied requests and sign-ins are counted exactly by kind; when their detail is shed under load, the trail says how many records were shed, by kind.
- AC-RQ7.14.5 No record holds a secret, a token or a credential: a lint over the record's type and a scan of a soak's trail both show it.
- AC-RQ7.14.6 A record altered or removed after its checkpoint is detected by the log's proofs.
- AC-RQ7.14.7 The trail is kept as long as the operator states; what is removed is removed by that rule alone, and its count is recorded.

Tests:
- T-RQ7.14.1 [happy] Push, move a tag, delete, change a grant → four records, each naming the principal, the action, the object and the outcome.
- T-RQ7.14.2 [crash] Power cut at every write of a push → after restart the change and its record are both present or both absent.
- T-RQ7.14.3 [exhaustion] A flood of denied requests → each counted by kind; detail shed at the budget and the shed count recorded; memory within budget; served requests unaffected.
- T-RQ7.14.4 [refusal] In a namespace that audits pulls, a pull while the trail's queue is full → refused typed; none is served unrecorded.
- T-RQ7.14.5 [hostile] A repository name, a user agent and a tag holding control characters, terminal escapes and a record separator → stored and shown escaped; no record is forged or split.
- T-RQ7.14.6 [refusal] Alter one stored record; remove one → the monitor's check fails on each.
- T-RQ7.14.7 [race] Two administrators change one grant at once → two records in commit order, each with the value it replaced.
- T-RQ7.14.8 [edge] A change made by the registry itself (a collection, a retention rule, a signing rule) → recorded with the rule as its principal, not a person.

---

### RQ7.15 Scanners that are not ours

Needs: RQ7.5, RQ7.13 · Serves: AC-T9.10.1

Reliquary does not scan images, but teams have scanners they trust, and Harbor lets them plug one
in. Take the same plug. A scanner that speaks Harbor's scanner adapter API is attached to a namespace
by its address. When an artifact it can read arrives, Reliquary asks it for a scan, handing over a
credential that reads that artifact alone and soon ends; waits as the scanner says to; stores the
report as a referrer; and records its findings as facts beside those from SBOMs, each naming its
scanner and the date of its data. Where the scanner offers an SBOM, that is kept too. A scan never
holds up a push, and a policy says what an artifact not yet scanned may do.

Done when:
- AC-RQ7.15.1 Harbor's adapter for Trivy, unchanged, scans artifacts pushed to Reliquary, and its report's findings appear as facts.
- AC-RQ7.15.2 The credential given to a scanner reads only the artifact to be scanned, and expires; with it nothing else can be read and nothing written.
- AC-RQ7.15.3 A push's latency is the same with and without a scanner attached, within the interval.
- AC-RQ7.15.4 Facts from a scanner name the scanner, its version and its database's date; two scanners' facts on one artifact are kept apart, and a policy chooses which it believes.
- AC-RQ7.15.5 A scan that fails, never finishes or returns a report that does not parse leaves the artifact "not scanned", with the reason; it is never recorded as clean.
- AC-RQ7.15.6 Waiting for a report follows the scanner's own answers and RQ0.8's waits; no timeout is a picked number, and scans in flight are bounded by the scanner's measured rate.

Tests:
- T-RQ7.15.1 [happy] Attach the Trivy adapter; push an image with a known vulnerable package → a report referrer and vulnerability facts; a policy of "no critical vulnerability" denies the pull.
- T-RQ7.15.2 [refusal] Use the scanner's credential to read another repository; to push; after its expiry → each refused.
- T-RQ7.15.3 [failure] The scanner is down; answers "not ready" without end; returns a report for another digest; returns malformed JSON → "not scanned" with the reason each time; retried by the learned wait; never clean.
- T-RQ7.15.4 [edge] An artifact type the scanner's metadata does not list; an index; an artifact pushed again unchanged; the scanner's database updated → not sent; each member scanned once; not scanned again; scanned again as the operator's rule states.
- T-RQ7.15.5 [exhaustion] Pushes faster than the scanner scans → the backlog is RQ7.13's durable cursor, bounded in memory; pushes are not slowed; the count waiting is a metric.
- T-RQ7.15.6 [hostile] A report larger than its budget; one naming more findings than its budget; a scanner address that resolves to an internal address the rule forbids → refused typed; refused typed; never called.
- T-RQ7.15.7 [race] An artifact deleted while its scan is in flight → the report attaches to nothing and is discarded by the stated rule, counted; no fact is left on a missing subject.
