# 02. The specifications Reliquary implements

Status: research note, 2026-11-20. Sources [1] to [13] are the specifications' own files, fetched on
2026-11-20 from the addresses given and read whole where this note quotes them. Quotes are verbatim.

## Sources

[1] OCI Distribution Specification v1.1.1 (released 2025-01-29),
https://raw.githubusercontent.com/opencontainers/distribution-spec/v1.1.1/spec.md

[2] OCI Distribution Specification, `main` at `97274622c1` (2026-08-31; the next version, unreleased),
https://raw.githubusercontent.com/opencontainers/distribution-spec/main/spec.md

[3] OCI distribution conformance test, `main`,
https://raw.githubusercontent.com/opencontainers/distribution-spec/main/conformance/README.md

[4] OCI Image Specification v1.1.1 (released 2025-03-03): `manifest.md`, `descriptor.md`,
`image-index.md`, `layer.md`, `image-layout.md`,
https://raw.githubusercontent.com/opencontainers/image-spec/v1.1.1/manifest.md (and siblings).

[5] Distribution's token authentication specification: `token.md`, `jwt.md`, `scope.md`, `oauth.md`,
https://raw.githubusercontent.com/distribution/distribution/main/docs/content/spec/auth/token.md
(and siblings).

[6] MCP Registry v1.8.10 (released 2026-11-20): `docs/reference/api/generic-registry-api.md`,
`openapi.yaml`, `CHANGELOG.md`, `docs/reference/server-json/generic-server-json.md`,
`draft/server.schema.json`, `docs/design/ecosystem-vision.md`,
https://raw.githubusercontent.com/modelcontextprotocol/registry/v1.8.10/docs/reference/api/generic-registry-api.md
(and siblings).

[7] Agent Skills specification, `main` at `83021e2be4` (2026-11-04),
https://raw.githubusercontent.com/agentskills/agentskills/main/docs/specification.mdx and its
reference validator `skills-ref/src/skills_ref/validator.py`.

[8] GitHub releases API for [1], [4] and [6], read 2026-11-20: distribution-spec `v1.1.1
2025-01-29`, image-spec `v1.1.1 2025-03-03`, MCP registry `v1.8.10 2026-11-20`.

[9] shards at `7158020b61`, `docs/architecture/AGENTFILE_ARCH.md` §12.17 (the OSI artifact types;
research/01 §7).

[10] The Update Framework specification, `master`,
https://raw.githubusercontent.com/theupdateframework/specification/master/tuf-spec.md

[11] C2SP `tlog-tiles`, "Tiled Transparency Logs",
https://raw.githubusercontent.com/C2SP/C2SP/main/tlog-tiles.md

[12] RFC 6298, "Computing TCP's Retransmission Timer", June 2011,
https://www.rfc-editor.org/rfc/rfc6298.txt

[13] RFC 9113, "HTTP/2", June 2022, https://www.rfc-editor.org/rfc/rfc9113.txt; RFC 9112, "HTTP/1.1",
June 2022, https://www.rfc-editor.org/rfc/rfc9112.txt; RFC 8878, "Zstandard Compression", February
2021, https://www.rfc-editor.org/rfc/rfc8878.txt

## 1. Distribution: what conformance is

- Four categories. "All registries conforming to this specification MUST support, at a minimum, all
  APIs in the **Pull** category. Registries SHOULD also support the **Push**, **Content
  Discovery**, and **Content Management** categories. A registry claiming conformance with one of
  these specification categories MUST implement all APIs in the claimed category." [1 §Requirements]
- Fourteen endpoints, `end-1` to `end-13` with `end-4a/4b`, `8a/8b`, `12a/12b` [1 §Endpoints], and
  fourteen error codes, `BLOB_UNKNOWN` to `TOOMANYREQUESTS` [1 §Error Codes].
- Names: "`<name>` MUST match the following regular expression:
  `[a-z0-9]+((\.|_|__|-+)[a-z0-9]+)*(\/[a-z0-9]+((\.|_|__|-+)[a-z0-9]+)*)*`"; a tag "MUST be at
  most 128 characters in length" [1 lines 155-164].
- Chunked uploads are ordered: "Chunks MUST be uploaded in order, with the first byte of a chunk
  being the last chunk's `<end-of-range>` plus one. If a chunk is uploaded out of order, the
  registry MUST respond with a `416 Requested Range Not Satisfiable` code." A registry may state a
  least chunk: "If the registry provides an `OCI-Chunk-Min-Length` header in the `POST` response,
  the size of each chunk, except for the final chunk, SHOULD be greater or equal to that value."
  [2 lines 377-392]
- Manifest size is the registry's to bound, with a floor: "A registry SHOULD enforce some limit on
  the maximum manifest size that it can accept. ... Client and registry implementations SHOULD
  expect to be able to support manifest pushes of at least 4 megabytes." [2 lines 542-544]
- Subjects and referrers: "A registry MUST initially accept an otherwise valid manifest with a
  `subject` field that references a manifest that does not exist in the repository"; a registry
  with the referrers API "MUST respond with the response header `OCI-Subject: <subject digest>`";
  "If the registry supports the referrers API, the registry MUST NOT return a `404 Not Found` to a
  referrers API requests"; "If a query results in no matching referrers, an empty manifest list
  MUST be returned"; filtering answers `OCI-Filters-Applied: artifactType` [2 lines 221, 548,
  626-681].
- Enabling referrers on a registry that held data: "Registries MUST include preexisting image
  manifests that are listed in an image index tagged with the referrers tag schema and have a valid
  `subject` field in the referrers API response." [2 line 821] This binds the migration of kiln's
  registry (T-T9.1.1).
- Concurrent identical uploads are a stated use case: "If process A and B upload the same layer at
  the same time, both operations will proceed and the first to complete will be stored in the
  registry." [1 §Layer Upload De-duplication]
- Warnings are bounded by the spec: "A registry MUST NOT send more than 4096 bytes of warning data
  from all headers combined." [2 line 914]

## 2. Distribution: what the next version adds (unreleased, on `main`)

Read from the difference between [1] and [2] (327 changed lines):

- A digest algorithm chosen at upload: "When pushing a blob with a digest algorithm other than
  `sha256`, the post request SHOULD include the `digest-algorithm` parameter" (`end-4c`).
- Tags pushed with a manifest by digest: "When pushing a manifest by digest, the registry MAY
  support the pushing of tags specified by addition of `tag` query parameters", "SHOULD support
  pushing at least 10 tags per request" and "MUST include an `OCI-Tag` response header ... for each
  accepted tag" (`end-7b`).
- Cancelling an upload: `DELETE <blob-push-location>` answered `204` (`end-14`).
- The upload location is opaque: "The `<blob-push-location>` MAY be provided by a server other
  than the registry itself"; "It MUST NOT be assembled manually by clients".
- Ranges: "A registry SHOULD support the `Range` request header".
- The digest header becomes required: "A successful response MUST contain the digest of the
  uploaded blob in the header `Docker-Content-Digest`."
- Proxying is specified: "A `ns` query parameter on pull operations is OPTIONAL, but when used
  specifies the host in a repository name used by a client"; "A registry that uses the `ns` query
  parameter to scope the request SHOULD return the `ns` query parameter value in the
  `OCI-Namespace` header."
- Throttling: "A `429 Too Many Requests` response ... SHOULD include a `Retry-After` header".
- Redirects: "Registries MAY respond to any request with a redirect ...; clients SHOULD follow such
  redirects, and MUST NOT forward `Authorization` headers across host boundaries unless explicitly
  configured to do so."

The conformance suite on `main` tests these behind settings [3]: `OCI_VERSION` ("1.1", "stable" or
"dev"), `OCI_API_BLOBS_UPLOAD_CANCEL`, `OCI_API_MANIFESTS_TAG_PARAM`,
`OCI_API_BLOBS_MOUNT_ANONYMOUS`, `OCI_API_*_ATOMIC` ("whether blob delete operations should be
immediate"), `OCI_DATA_SHA512`, `OCI_DATA_SPARSE`, `OCI_DATA_SUBJECT_MISSING`,
`OCI_DATA_NONDISTRIBUTABLE`, `OCI_DATA_EMPTY_BLOB` and the read-only `OCI_RO_DATA_*` lists. It
writes `results.yaml`, `report.html` and `junit.xml`.

**Derived.** Passing the suite at `OCI_VERSION=1.1` with every `OCI_API_*` and `OCI_DATA_*` setting
at its default is AC-T9.1.2. Passing it at `dev` with every optional setting turned on is the
stronger bar, and it is checkable by the same tool.

## 3. Images, indexes and artifacts

- Digests: "compliant implementations SHOULD use SHA-256"; registered algorithms are `sha256` and
  `sha512`; "When the _algorithm identifier_ is `sha256`, the _encoded_ portion MUST match
  `/[a-f0-9]{64}/`" [4 descriptor.md §Registered algorithms].
- Artifacts are image manifests with an `artifactType`: "Implementations storing or copying image
  manifests MUST NOT error on encountering an `artifactType` that is unknown to the
  implementation"; "If the `config.mediaType` is set to the empty value, the `artifactType` MUST be
  defined"; the empty descriptor is `application/vnd.oci.empty.v1+json` [4 manifest.md].
- `subject` "defines a weak association to a separate Merkle Directed Acyclic Graph (DAG)
  structure, and is used by the `referrers` API" [4 manifest.md]. Weak: a referrer does not keep
  its subject alive, and a subject's deletion does not by itself delete its referrers (derived).
- History: "artifacts have historically been created without an `artifactType` field, and tooling
  to work with artifacts should fallback to the `config.mediaType` value" [4 manifest.md].

## 4. Authentication

- The challenge: a `401` with `Www-Authenticate: Bearer realm="...",service="...",scope="repository:samalba/my-app:pull,push"` [5 token.md].
- Token life has a floor from the specification: "`expires_in` (Optional) The duration in seconds
  since the token was issued that it will remain valid. When omitted, this defaults to 60 seconds.
  For compatibility with older clients, a token should never be returned with less than 60 seconds
  to live." [5 token.md]
- OAuth2 form with refresh tokens: `grant_type=refresh_token`, `access_token`, `expires_in`,
  `issued_at` [5 oauth.md].

## 5. The MCP Registry API

- The registry is a specification others implement: "**The MCP registry spec**: An API
  specification that allows anyone to implement a registry" [6 ecosystem-vision.md].
- It is a metaregistry: "MCP registries are _metaregistries_. They host metadata about packages,
  but not the package code or binaries. Instead, they reference other package registries (like
  NPM, PyPi or Docker) for this." Subregistries are expected: "we expect clients pull from
  _subregistries_" [6 ecosystem-vision.md].
- Endpoints [6 generic-registry-api.md]: "`GET /v0.1/servers` - List all servers with pagination",
  "`GET /v0.1/servers/{serverName}/versions`", "`GET /v0.1/servers/{serverName}/versions/{version}`
  - Get specific version of server. Use the special version `latest` to get the latest version.",
  "`POST /v0.1/publish`", and optional `PUT`, `DELETE` and `PATCH .../status`.
- Paging is by opaque cursor: "Always treat cursors as opaque strings. Never manually construct or
  modify cursor values." [6 generic-registry-api.md]
- Status values: `active`, `deprecated`, `deleted`; `updated_since` listings include deleted
  servers [6 api CHANGELOG.md].
- A server's packages name where its bytes are: `"registryType": "oci"` with an image reference as
  `identifier`, `npm`, `pypi`, `nuget`, `cargo`, and `mcpb` with a URL and `fileSha256`
  [6 generic-server-json.md].

**Derived.** Reliquary can be the one place where an MCP server's `server.json` and its bytes live
together: an `oci` package whose identifier is a Reliquary reference pinned by digest, or an `mcpb`
bundle (a zip) held as an artifact, each checked against the other at publish. No registry read for
this note does both.

## 6. Agent Skills

- "A skill is a directory containing, at minimum, a `SKILL.md` file"; optional `scripts/`,
  `references/`, `assets/` [7 specification.mdx].
- Frontmatter [7]: `name` ("Max 64 characters. Lowercase letters, numbers, and hyphens only. Must
  not start or end with a hyphen."; "Must not contain consecutive hyphens"; "Must match the parent
  directory name"), `description` ("Max 1024 characters. Non-empty."), `license`, `compatibility`
  ("Max 500 characters"), `metadata` ("a map from string keys to string values"), `allowed-tools`
  ("Experimental").
- Shards already checks skills "as the Agent Skills reference validator checks them (131 skills
  held to skills-ref)" [9 §11], so Reliquary's publish check and Shards' build check must be one
  implementation or held to one corpus (derived).

## 7. Shards' artifact types

In research/01 §7, from [9]: `application/vnd.osi.agent.v1`, `.harness.v1`, `.mcp.v1`, their
configs and content layers, their refusals, and `application/vnd.shards.microvm.v1` with
`application/vnd.shards.erofs.v1`.

## 8. Freshness, history and waits

- What TUF is specified to defeat [10]: rollback, "An attacker cannot trick clients into installing
  software that is older than that which the client previously knew to be available"; freeze, "An
  attacker cannot respond to client requests with the same, outdated metadata without the client
  being aware of the problem"; and mix-and-match, "An attacker cannot trick clients into using a
  combination of metadata that never existed together on the repository at the same time". Kiln's
  T-T9.4.1 is the first of these.
- A tiled transparency log [11] is "an efficient HTTP API to fetch the signed checkpoint, Merkle
  Tree hashes, and entries of a transparency log containing arbitrary entries"; a complete tile
  never changes: "This endpoint is immutable, so its caching headers SHOULD be long-lived." This is
  the form the plan's tag history takes (RQ7.8).
- RFC 6298 [12], the estimator the plan uses for connection and handshake waits (RQ0.8): "Until a
  round-trip time (RTT) measurement has been made for a segment sent between the sender and
  receiver, the sender SHOULD set RTO <- 1 second"; then `RTO <- SRTT + max (G, K*RTTVAR)`; and on a
  timeout "The host MUST set RTO <- RTO * 2 ("back off the timer")".

## 9. Hazards the specifications themselves name

- HTTP/2 [13, RFC 9113 §10.5]: "An endpoint that doesn't monitor use of these features exposes
  itself to a risk of denial of service. Implementations SHOULD track the use of these features and
  set limits on their use. An endpoint MAY treat activity that is suspicious as a connection error
  (Section 5.4.1) of type ENHANCE_YOUR_CALM." The plan's HTTP/2 item bounds each such use and tests
  each known attack (RQ4.6).
- HTTP/1.1 [13, RFC 9112 §6.3]: a message with both `Transfer-Encoding` and `Content-Length` "might
  indicate an attempt to perform request smuggling (Section 11.2) or response splitting (Section
  11.1) and ought to be handled as an error." The OCI door refuses it (RQ1.6).
- Zstandard [13, RFC 8878 §3.1.1.1.2]: "a decoder is allowed to reject a compressed frame that
  requests a memory size beyond the decoder's authorized range", and "decoders are free to support
  higher or lower limits, depending on local limitations". A reader's window bound is therefore its
  admitted memory, not a fixed figure (RQ2.2).
