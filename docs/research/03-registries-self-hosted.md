# Self-hosted registries: source notes
Researched 2026-11-20. Question: for each self-hosted or open-source OCI registry (CNCF Distribution, Harbor, zot, Project Quay, GitLab Container Registry, olareg, the Rust registries, and the registries embedded in other products), what do primary sources say about its architecture, spec conformance, garbage collection, pull-through and replication, auth, search, trust, storage layout, limits and defaults, performance, operational pain, platforms and releases? Method: each project's source tree at its newest release tag, its documentation, release notes, design documents and issue tracker were downloaded; every quoted passage was matched by script against the downloaded copy and then against a fresh fetch of the cited URL.

How to read these notes:

- Where to start: the comparison table, the five digests under it and the catalogue of constants are the entry points (search for `## Comparison table` and `## Catalogue of constants`). Their cells cite finding ids; search for `### F123.` to reach a finding and for `| S123 |` to reach a source. The file is long because every finding carries its quotes.
- Finding ranges: CNCF Distribution v3.1.2, with the OCI distribution-spec as yardstick: F001 to F048; Harbor v2.16.0: F049 to F106; zot v2.1.23: F107 to F168; Project Quay v3.18.3: F169 to F221; GitLab Container Registry v4.43.0-gitlab: F222 to F280; olareg v0.2.2; the Rust registries trow, angos, nora and smaller ones; go-containerregistry pkg/registry; cloudflare/serverless-registry: F281 to F350; peryx (Rust), examined briefly; two peer-reviewed papers on Distribution-based registries: F351 to F361; Gitea and Forgejo, Pulp container, Sonatype Nexus Repository, Dragonfly, Uber Kraken, OpenShift image registry: F362 to F401.
- Each claim starts with the registry and the questions of the brief it answers: Q1 architecture, Q2 OCI spec and conformance, Q3 garbage collection, Q4 pull-through cache, mirroring and replication, Q5 authentication and authorization, Q6 search, catalog and UI, Q7 trust, Q8 storage layout and deduplication, Q9 limits and defaults, Q10 performance, Q11 operational pain points, Q12 platforms, Q13 releases.
- Quotes are verbatim from the cited source at the cited version. Where the source is markdown or code, its line breaks are not reproduced and its formatting marks (backticks, emphasis, link targets, comment and block-quote prefixes) are often left out. Some quotes are shorter than 8 words because the source is a constant, a heading or a table cell.
- After the source id comes the heading the quote sits under or, for code, the function or constant with its line numbers (L10-L14) in the file at the cited tag.
- Source type `forum` covers issue trackers, pull requests and discussions; `vendor` is a company's own documentation or blog about its product; `paper` is peer-reviewed; "(secondary)" marks a third-party source.
- A statement that is an inference from reading code, and not a statement of the project, says so in its Conditions/notes line.
- Totals: 656 sources, 401 findings, 1289 quoted passages, 407 constants. No benchmark was run and nothing was built for these notes: every number is as published by its source.

## Sources
| id | type | title | URL | version/date |
|---|---|---|---|---|
| S001 | docs | distribution: README.md | https://github.com/distribution/distribution/blob/v3.1.2/README.md | v3.1.2 (2026-09-24) |
| S002 | docs | distribution docs: Distribution Registry (index) | https://github.com/distribution/distribution/blob/v3.1.2/docs/content/_index.md | v3.1.2 (2026-09-24) |
| S003 | docs | distribution docs: Configuring a registry | https://github.com/distribution/distribution/blob/v3.1.2/docs/content/about/configuration.md | v3.1.2 (2026-09-24) |
| S004 | code | distribution: configuration/configuration.go | https://github.com/distribution/distribution/blob/v3.1.2/configuration/configuration.go | v3.1.2 (2026-09-24) |
| S005 | docs | distribution docs: Registry storage driver | https://github.com/distribution/distribution/blob/v3.1.2/docs/content/storage-drivers/_index.md | v3.1.2 (2026-09-24) |
| S006 | code | distribution: registry/storage/paths.go | https://github.com/distribution/distribution/blob/v3.1.2/registry/storage/paths.go | v3.1.2 (2026-09-24) |
| S007 | docs | distribution docs: Deploy a registry server | https://github.com/distribution/distribution/blob/v3.1.2/docs/content/about/deploying.md | v3.1.2 (2026-09-24) |
| S008 | docs | distribution docs: Architecture (marked draft) | https://github.com/distribution/distribution/blob/v3.1.2/docs/content/about/architecture.md | v3.1.2 (2026-09-24) |
| S009 | code | distribution: registry/storage/vacuum.go | https://github.com/distribution/distribution/blob/v3.1.2/registry/storage/vacuum.go | v3.1.2 (2026-09-24) |
| S010 | code | distribution: .github/workflows/conformance.yml | https://github.com/distribution/distribution/blob/v3.1.2/.github/workflows/conformance.yml | v3.1.2 (2026-09-24) |
| S011 | docs | distribution: ROADMAP.md | https://github.com/distribution/distribution/blob/v3.1.2/ROADMAP.md | v3.1.2 (2026-09-24) |
| S012 | code | distribution: manifest/ocischema/manifest.go | https://github.com/distribution/distribution/blob/v3.1.2/manifest/ocischema/manifest.go | v3.1.2 (2026-09-24) |
| S013 | code | distribution: registry/storage/blobwriter.go | https://github.com/distribution/distribution/blob/v3.1.2/registry/storage/blobwriter.go | v3.1.2 (2026-09-24) |
| S014 | docs | distribution docs: Garbage collection | https://github.com/distribution/distribution/blob/v3.1.2/docs/content/about/garbage-collection.md | v3.1.2 (2026-09-24) |
| S015 | code | distribution: registry/storage/garbagecollect.go | https://github.com/distribution/distribution/blob/v3.1.2/registry/storage/garbagecollect.go | v3.1.2 (2026-09-24) |
| S016 | code | distribution: registry/root.go | https://github.com/distribution/distribution/blob/v3.1.2/registry/root.go | v3.1.2 (2026-09-24) |
| S017 | code | distribution: registry/handlers/app.go | https://github.com/distribution/distribution/blob/v3.1.2/registry/handlers/app.go | v3.1.2 (2026-09-24) |
| S018 | code | distribution: registry/storage/purgeuploads.go | https://github.com/distribution/distribution/blob/v3.1.2/registry/storage/purgeuploads.go | v3.1.2 (2026-09-24) |
| S019 | docs | distribution docs: Registry as a pull through cache | https://github.com/distribution/distribution/blob/v3.1.2/docs/content/recipes/mirror.md | v3.1.2 (2026-09-24) |
| S020 | code | distribution: registry/proxy/proxyblobstore.go | https://github.com/distribution/distribution/blob/v3.1.2/registry/proxy/proxyblobstore.go | v3.1.2 (2026-09-24) |
| S021 | code | distribution: registry/proxy/proxyregistry.go | https://github.com/distribution/distribution/blob/v3.1.2/registry/proxy/proxyregistry.go | v3.1.2 (2026-09-24) |
| S022 | code | distribution: registry/proxy/proxytagservice.go | https://github.com/distribution/distribution/blob/v3.1.2/registry/proxy/proxytagservice.go | v3.1.2 (2026-09-24) |
| S023 | code | distribution: registry/proxy/scheduler/scheduler.go | https://github.com/distribution/distribution/blob/v3.1.2/registry/proxy/scheduler/scheduler.go | v3.1.2 (2026-09-24) |
| S024 | code | distribution: registry/handlers/catalog.go | https://github.com/distribution/distribution/blob/v3.1.2/registry/handlers/catalog.go | v3.1.2 (2026-09-24) |
| S025 | code | distribution: registry/storage/catalog.go | https://github.com/distribution/distribution/blob/v3.1.2/registry/storage/catalog.go | v3.1.2 (2026-09-24) |
| S026 | code | distribution: registry/handlers/tags.go | https://github.com/distribution/distribution/blob/v3.1.2/registry/handlers/tags.go | v3.1.2 (2026-09-24) |
| S027 | code | distribution: registry/handlers/manifests.go | https://github.com/distribution/distribution/blob/v3.1.2/registry/handlers/manifests.go | v3.1.2 (2026-09-24) |
| S028 | code | distribution: registry/handlers/helpers.go | https://github.com/distribution/distribution/blob/v3.1.2/registry/handlers/helpers.go | v3.1.2 (2026-09-24) |
| S029 | code | distribution: registry/storage/linkedblobstore.go | https://github.com/distribution/distribution/blob/v3.1.2/registry/storage/linkedblobstore.go | v3.1.2 (2026-09-24) |
| S030 | code | distribution: registry/handlers/blobupload.go | https://github.com/distribution/distribution/blob/v3.1.2/registry/handlers/blobupload.go | v3.1.2 (2026-09-24) |
| S031 | code | distribution: registry/storage/driver/s3-aws/s3.go | https://github.com/distribution/distribution/blob/v3.1.2/registry/storage/driver/s3-aws/s3.go | v3.1.2 (2026-09-24) |
| S032 | code | distribution: registry/storage/driver/s3-aws/s3_64bit.go | https://github.com/distribution/distribution/blob/v3.1.2/registry/storage/driver/s3-aws/s3_64bit.go | v3.1.2 (2026-09-24) |
| S033 | code | distribution: registry/storage/driver/gcs/gcs.go | https://github.com/distribution/distribution/blob/v3.1.2/registry/storage/driver/gcs/gcs.go | v3.1.2 (2026-09-24) |
| S034 | code | distribution: registry/storage/driver/azure/azure.go | https://github.com/distribution/distribution/blob/v3.1.2/registry/storage/driver/azure/azure.go | v3.1.2 (2026-09-24) |
| S035 | docs | distribution docs: Azure storage driver | https://github.com/distribution/distribution/blob/v3.1.2/docs/content/storage-drivers/azure.md | v3.1.2 (2026-09-24) |
| S036 | code | distribution: registry/storage/driver/filesystem/driver.go | https://github.com/distribution/distribution/blob/v3.1.2/registry/storage/driver/filesystem/driver.go | v3.1.2 (2026-09-24) |
| S037 | docs | distribution docs: Filesystem storage driver | https://github.com/distribution/distribution/blob/v3.1.2/docs/content/storage-drivers/filesystem.md | v3.1.2 (2026-09-24) |
| S038 | code | distribution: registry/storage/cache/memory/memory.go | https://github.com/distribution/distribution/blob/v3.1.2/registry/storage/cache/memory/memory.go | v3.1.2 (2026-09-24) |
| S039 | code | distribution: registry/storage/filereader.go | https://github.com/distribution/distribution/blob/v3.1.2/registry/storage/filereader.go | v3.1.2 (2026-09-24) |
| S040 | code | distribution: registry/registry.go | https://github.com/distribution/distribution/blob/v3.1.2/registry/registry.go | v3.1.2 (2026-09-24) |
| S041 | code | distribution: notifications/sinks.go | https://github.com/distribution/distribution/blob/v3.1.2/notifications/sinks.go | v3.1.2 (2026-09-24) |
| S042 | code | distribution: notifications/endpoint.go | https://github.com/distribution/distribution/blob/v3.1.2/notifications/endpoint.go | v3.1.2 (2026-09-24) |
| S043 | docs | distribution docs: Work with notifications | https://github.com/distribution/distribution/blob/v3.1.2/docs/content/about/notifications.md | v3.1.2 (2026-09-24) |
| S044 | code | distribution: registry/auth/token/token.go | https://github.com/distribution/distribution/blob/v3.1.2/registry/auth/token/token.go | v3.1.2 (2026-09-24) |
| S045 | docs | distribution docs: Token Authentication Specification | https://github.com/distribution/distribution/blob/v3.1.2/docs/content/spec/auth/token.md | v3.1.2 (2026-09-24) |
| S046 | docs | distribution docs: Token Scope Documentation | https://github.com/distribution/distribution/blob/v3.1.2/docs/content/spec/auth/scope.md | v3.1.2 (2026-09-24) |
| S047 | code | distribution: registry/storage/tagstore.go | https://github.com/distribution/distribution/blob/v3.1.2/registry/storage/tagstore.go | v3.1.2 (2026-09-24) |
| S048 | code | distribution: registry/storage/ocimanifesthandler.go | https://github.com/distribution/distribution/blob/v3.1.2/registry/storage/ocimanifesthandler.go | v3.1.2 (2026-09-24) |
| S049 | code | distribution: registry/storage/cache/redis/redis.go | https://github.com/distribution/distribution/blob/v3.1.2/registry/storage/cache/redis/redis.go | v3.1.2 (2026-09-24) |
| S050 | code | distribution: docker-bake.hcl | https://github.com/distribution/distribution/blob/v3.1.2/docker-bake.hcl | v3.1.2 (2026-09-24) |
| S051 | docs | distribution docs: macOS setup guide | https://github.com/distribution/distribution/blob/v3.1.2/docs/content/recipes/osx-setup-guide.md | v3.1.2 (2026-09-24) |
| S052 | code | distribution (vendored distribution/reference v0.6.0): reference.go | https://github.com/distribution/distribution/blob/v3.1.2/vendor/github.com/distribution/reference/reference.go | v3.1.2 (2026-09-24) |
| S053 | code | distribution: go.mod | https://github.com/distribution/distribution/blob/v3.1.2/go.mod | v3.1.2 (2026-09-24) |
| S054 | docs | distribution docs: Google Cloud Storage driver | https://github.com/distribution/distribution/blob/v3.1.2/docs/content/storage-drivers/gcs.md | v3.1.2 (2026-09-24) |
| S055 | docs | distribution docs: S3 storage driver | https://github.com/distribution/distribution/blob/v3.1.2/docs/content/storage-drivers/s3.md | v3.1.2 (2026-09-24) |
| S056 | code | distribution: registry/storage/registry.go | https://github.com/distribution/distribution/blob/v3.1.2/registry/storage/registry.go | v3.1.2 (2026-09-24) |
| S057 | code | distribution: registry/storage/driver/azure/parser.go | https://github.com/distribution/distribution/blob/v3.1.2/registry/storage/driver/azure/parser.go | v3.1.2 (2026-09-24) |
| S058 | code | distribution: .github/workflows/conformance.yml on main | https://github.com/distribution/distribution/blob/1ceccfa2d2048b3c6fe5936ca925e3b513b86e3e/.github/workflows/conformance.yml | main at 1ceccfa (2026-11-19) |
| S059 | code | distribution v3.0.0: registry/handlers/tags.go | https://github.com/distribution/distribution/blob/v3.0.0/registry/handlers/tags.go | v3.0.0 (2025-04-03) |
| S060 | code | distribution v3.0.0: configuration/configuration.go | https://github.com/distribution/distribution/blob/v3.0.0/configuration/configuration.go | v3.0.0 (2025-04-03) |
| S061 | docs | distribution release notes v3.0.0-alpha.1 | https://github.com/distribution/distribution/releases/tag/v3.0.0-alpha.1 | 2023-12-19 |
| S062 | docs | distribution release notes v3.0.0 | https://github.com/distribution/distribution/releases/tag/v3.0.0 | 2025-04-03 |
| S063 | docs | distribution release notes v3.1.0 | https://github.com/distribution/distribution/releases/tag/v3.1.0 | 2026-04-06 |
| S064 | docs | distribution release notes v3.1.1 | https://github.com/distribution/distribution/releases/tag/v3.1.1 | 2026-05-01 |
| S065 | docs | distribution release notes v3.1.2 | https://github.com/distribution/distribution/releases/tag/v3.1.2 | 2026-09-24 |
| S066 | docs | distribution v3.1.2 release assets | https://github.com/distribution/distribution/releases/expanded_assets/v3.1.2 | 2026-09-24 |
| S067 | docs | distribution releases feed | https://github.com/distribution/distribution/releases.atom | fetched 2026-11-20 |
| S068 | vendor | distribution security advisories (list) | https://github.com/distribution/distribution/security/advisories | fetched 2026-11-20 |
| S069 | vendor | GHSA-hqxw-f8mx-cpmw: Catalog API endpoint can lead to OOM via malicious user input | https://github.com/distribution/distribution/security/advisories/GHSA-hqxw-f8mx-cpmw | 2023-05-09 |
| S070 | vendor | GHSA-f2g3-hh2r-cwgc: Stale blob access resurrection via repo-scoped redis descriptor cache invalidation | https://github.com/distribution/distribution/security/advisories/GHSA-f2g3-hh2r-cwgc | 2026-04-06 |
| S071 | vendor | GHSA-3p65-76g6-3w7r: Pull-through cache credential exfiltration via www-authenticate bearer realm | https://github.com/distribution/distribution/security/advisories/GHSA-3p65-76g6-3w7r | 2026-04-06 |
| S072 | vendor | GHSA-6pjf-3r9x-m592: Tag deletion bypasses storage.delete.enabled configuration | https://github.com/distribution/distribution/security/advisories/GHSA-6pjf-3r9x-m592 | 2026-05-01 |
| S073 | vendor | GHSA-685w-q87j-wqw3: Client manifests.Get() does not verify returned content against requested digest | https://github.com/distribution/distribution/security/advisories/GHSA-685w-q87j-wqw3 | 2026-09-23 |
| S074 | forum | distribution PR #4828: feat: implement OCI Distribution Spec v1.1 referrers API (open) | https://github.com/distribution/distribution/pull/4828 | opened 2026-03-26; open on 2026-11-20 |
| S075 | forum | distribution issue #3716: Proposal: References support (open) | https://github.com/distribution/distribution/issues/3716 | opened 2022-08-18; open on 2026-11-20 |
| S076 | forum | distribution issue #4420: Pushing a manifest with a sha512 digest fails (open) | https://github.com/distribution/distribution/issues/4420 | opened 2024-07-25; open on 2026-11-20 |
| S077 | forum | distribution PR #4946: Add sha512 support (open) | https://github.com/distribution/distribution/pull/4946 | opened 2026-08-31; open on 2026-11-20 |
| S078 | forum | distribution issue #1609: What's the plan for online (background) garbage collection (open) | https://github.com/distribution/distribution/issues/1609 | opened 2016-04-07; open on 2026-11-20 |
| S079 | forum | distribution issue #3178: Registry garbage-collect --delete-untagged removes multi-arch manifests (closed) | https://github.com/distribution/distribution/issues/3178 | opened 2020-06-08; closed 2024-04-21 |
| S080 | forum | distribution issue #4461: Garbage collector removed all blobs and corrupted the registry (closed) | https://github.com/distribution/distribution/issues/4461 | opened 2024-09-11 |
| S081 | forum | distribution issue #5010: Garbage collection and blobdescriptor cache do not coordinate blob cleanup | https://github.com/distribution/distribution/issues/5010 | opened 2026-10-13 |
| S082 | forum | distribution issue #3851: garbage-collect does not purge unused _layers (open) | https://github.com/distribution/distribution/issues/3851 | opened 2023-02-23; open on 2026-11-20 |
| S083 | forum | distribution issue #4382: Incomplete GC on S3 backend (open) | https://github.com/distribution/distribution/issues/4382 | opened 2024-06-13; open on 2026-11-20 |
| S084 | forum | distribution issue #4358: Registry keeps crashing after starting purgeuploads.go (closed) | https://github.com/distribution/distribution/issues/4358 | opened 2024-05-20 |
| S085 | forum | distribution issue #2367: pull through cache (proxy): docker pull error unexpected EOF due to common blob expiration (open) | https://github.com/distribution/distribution/issues/2367 | opened 2017-08-15; open on 2026-11-20 |
| S086 | forum | distribution issue #4191: proxy: TTLExpirationScheduler removes blobs that are still in use by other images (closed as duplicate of #2367) | https://github.com/distribution/distribution/issues/4191 | opened 2023-12-15; closed 2024-01-16 |
| S087 | forum | distribution issue #4478: proxy: panic: unexpected end of JSON input (closed) | https://github.com/distribution/distribution/issues/4478 | opened 2024-10-04 |
| S088 | forum | distribution issue #4038: Proxy registry crashes at start when there is no Internet connection (open) | https://github.com/distribution/distribution/issues/4038 | opened 2023-09-04; open on 2026-11-20 |
| S089 | forum | distribution issue #3270: Proxy handles 429 poorly (open) | https://github.com/distribution/distribution/issues/3270 | opened 2020-10-13; open on 2026-11-20 |
| S090 | forum | distribution issue #3725: Support for multiple proxies (open) | https://github.com/distribution/distribution/issues/3725 | opened 2022-09-04; open on 2026-11-20 |
| S091 | forum | distribution issue #5038: /tags/list performance regression with filesystem storage in v3.1.2 (open) | https://github.com/distribution/distribution/issues/5038 | opened 2026-10-22; open on 2026-11-20 |
| S092 | forum | distribution issue #3678: Paginated _catalog endpoint walks entire S3 repositories directory (closed) | https://github.com/distribution/distribution/issues/3678 | opened 2022-06-30 |
| S093 | forum | distribution issue #3525: Deleting an image from a repo with a large number of tags can be very slow (open) | https://github.com/distribution/distribution/issues/3525 | opened 2021-11-11; open on 2026-11-20 |
| S094 | forum | distribution PR #3480: Optimize storagedriver/s3 Walk (up to ~500x) + small bugfix (merged) | https://github.com/distribution/distribution/pull/3480 | opened 2021-08-12 |
| S095 | forum | distribution PR #5081: storage: serve filesystem blobs directly from the file (merged to main, not in v3.1.2) | https://github.com/distribution/distribution/pull/5081 | opened 2026-11-12; merged by 2026-11-20 |
| S096 | forum | distribution PR #4906: storage: reduce manifest verification latency (merged to main, not in v3.1.2) | https://github.com/distribution/distribution/pull/4906 | opened 2026-07-09; merged |
| S097 | forum | distribution PR #4919: fix(s3): bound pooled part buffer growth to chunk size (merged to main, not in v3.1.2) | https://github.com/distribution/distribution/pull/4919 | opened 2026-07-21; merged |
| S098 | forum | distribution PR #4329: add bounded concurrency for tag lookup and untag (merged) | https://github.com/distribution/distribution/pull/4329 | opened 2024-04-18 |
| S099 | forum | distribution issue #2212: Registry garbage-collect does not clean up blobs from overwritten tags (closed) | https://github.com/distribution/distribution/issues/2212 | opened 2017-03-15 |
| S100 | standard | OCI Distribution Specification v1.1.1: spec.md | https://github.com/opencontainers/distribution-spec/blob/v1.1.1/spec.md | v1.1.1 (2025-01-29) |
| S101 | standard | OCI Distribution Specification v1.1.1: conformance/README.md | https://github.com/opencontainers/distribution-spec/blob/v1.1.1/conformance/README.md | v1.1.1 (2025-01-29) |
| S102 | standard | OCI Distribution Specification, unreleased main: spec.md | https://github.com/opencontainers/distribution-spec/blob/97274622c11112caa21efb8c52acca3c6b8fa7f1/spec.md | main at 9727462 (2026-08-31) |
| S103 | standard | OCI Distribution Specification, unreleased main: conformance/README.md | https://github.com/opencontainers/distribution-spec/blob/97274622c11112caa21efb8c52acca3c6b8fa7f1/conformance/README.md | main at 9727462 (2026-08-31) |
| S104 | standard | OCI-Conformant Products (OCI conformance results site) | https://conformance.opencontainers.org/ | generated Aug 5 from oci-conformance commit 1609761 (commit dated 2026-08-05) |
| S105 | code | opencontainers/oci-conformance commit 1609761 (site source) | https://github.com/opencontainers/oci-conformance/commit/16097618fca61dc5d7aed970d61514aa8efad2c9 | 2026-08-05 |
| S106 | code | harbor: Makefile | https://github.com/goharbor/harbor/blob/v2.16.0/Makefile | v2.16.0 (2026-11-10) |
| S107 | docs | Harbor docs 2.16.0: Garbage Collection | https://goharbor.io/docs/2.16.0/administration/garbage-collection/ | docs 2.16.0 (fetched 2026-11-20) |
| S108 | code | harbor: src/jobservice/job/impl/gc/garbage_collection.go | https://github.com/goharbor/harbor/blob/v2.16.0/src/jobservice/job/impl/gc/garbage_collection.go | v2.16.0 (2026-11-10) |
| S109 | code | harbor: src/pkg/blob/models/blob.go | https://github.com/goharbor/harbor/blob/v2.16.0/src/pkg/blob/models/blob.go | v2.16.0 (2026-11-10) |
| S110 | code | harbor: src/pkg/blob/dao/dao.go | https://github.com/goharbor/harbor/blob/v2.16.0/src/pkg/blob/dao/dao.go | v2.16.0 (2026-11-10) |
| S111 | code | harbor: src/server/middleware/blob/head_blob.go | https://github.com/goharbor/harbor/blob/v2.16.0/src/server/middleware/blob/head_blob.go | v2.16.0 (2026-11-10) |
| S112 | code | harbor: src/server/middleware/blob/util.go | https://github.com/goharbor/harbor/blob/v2.16.0/src/server/middleware/blob/util.go | v2.16.0 (2026-11-10) |
| S113 | code | harbor: src/registryctl/api/registry/manifest/manifest.go | https://github.com/goharbor/harbor/blob/v2.16.0/src/registryctl/api/registry/manifest/manifest.go | v2.16.0 (2026-11-10) |
| S114 | code | harbor: src/server/v2.0/handler/gc.go | https://github.com/goharbor/harbor/blob/v2.16.0/src/server/v2.0/handler/gc.go | v2.16.0 (2026-11-10) |
| S115 | code | harbor: src/common/const.go | https://github.com/goharbor/harbor/blob/v2.16.0/src/common/const.go | v2.16.0 (2026-11-10) |
| S116 | docs | Harbor design proposal: Non-Blocking GC | https://github.com/goharbor/community/blob/58c9d106428b98f55ae2a891e1c64f0715280144/proposals/new/Non-Blocking-GC.md | proposal for v2.1 (2020); repository commit 58c9d10 |
| S117 | docs | Harbor design proposal: Garbage Collection Performance Enhancement | https://github.com/goharbor/community/blob/58c9d106428b98f55ae2a891e1c64f0715280144/proposals/new/gc_perf_enhancement.md | 2025 (community PR 265 opened 2025-07-21); commit 58c9d10 |
| S118 | forum | goharbor/harbor issue 12948: [GC performance] The performance of v2 manifest deletion is not good in S3 environment | https://github.com/goharbor/harbor/issues/12948 | opened 2020-09-02, closed 2025-11-20 |
| S119 | forum | goharbor/harbor issue 15445: GC deletes used blobs | https://github.com/goharbor/harbor/issues/15445 | opened 2021-08-18, closed as not planned 2023-06-09 |
| S120 | forum | goharbor/harbor issue 19991: Large Garbage Collections Slow and Can Easily Fail Indefinitely | https://github.com/goharbor/harbor/issues/19991 | opened 2024-02-16, closed |
| S121 | forum | goharbor/harbor issue 22739: Long running GC with S3 storage expires before finishing | https://github.com/goharbor/harbor/issues/22739 | opened 2026-01-17, closed |
| S122 | forum | goharbor/harbor issue 16010: GC enhancement engineering story | https://github.com/goharbor/harbor/issues/16010 | opened 2021-11-16, open |
| S123 | code | harbor: src/server/registry/manifest.go | https://github.com/goharbor/harbor/blob/v2.16.0/src/server/registry/manifest.go | v2.16.0 (2026-11-10) |
| S124 | code | harbor: make/harbor.yml.tmpl | https://github.com/goharbor/harbor/blob/v2.16.0/make/harbor.yml.tmpl | v2.16.0 (2026-11-10) |
| S125 | code | harbor: make/photon/prepare/templates/registry/config.yml.jinja | https://github.com/goharbor/harbor/blob/v2.16.0/make/photon/prepare/templates/registry/config.yml.jinja | v2.16.0 (2026-11-10) |
| S126 | code | harbor: src/lib/config/metadata/metadatalist.go | https://github.com/goharbor/harbor/blob/v2.16.0/src/lib/config/metadata/metadatalist.go | v2.16.0 (2026-11-10) |
| S127 | docs | Harbor docs 2.16.0: Harbor Installation Prerequisites | https://goharbor.io/docs/2.16.0/install-config/installation-prereqs/ | docs 2.16.0 (fetched 2026-11-20) |
| S128 | docs | Harbor docs 2.16.0: Harbor Installation and Configuration | https://goharbor.io/docs/2.16.0/install-config/ | docs 2.16.0 (fetched 2026-11-20) |
| S129 | code | harbor: make/photon/prepare/templates/docker_compose/docker-compose.yml.jinja | https://github.com/goharbor/harbor/blob/v2.16.0/make/photon/prepare/templates/docker_compose/docker-compose.yml.jinja | v2.16.0 (2026-11-10) |
| S130 | docs | Harbor release notes v2.16.0 | https://github.com/goharbor/harbor/releases/tag/v2.16.0 | 2026-11-10 |
| S131 | docs | Harbor release notes v2.15.0 | https://github.com/goharbor/harbor/releases/tag/v2.15.0 | 2026-03-20 |
| S132 | docs | Harbor release notes v2.14.0 | https://github.com/goharbor/harbor/releases/tag/v2.14.0 | 2025-09-17 |
| S133 | docs | Harbor release notes v2.13.0 | https://github.com/goharbor/harbor/releases/tag/v2.13.0 | 2025-04-10 |
| S134 | code | harbor: .github/workflows/conformance_test.yml | https://github.com/goharbor/harbor/blob/v2.16.0/.github/workflows/conformance_test.yml | v2.16.0 (2026-11-10) |
| S135 | code | harbor: tests/ci/conformance_test.sh | https://github.com/goharbor/harbor/blob/v2.16.0/tests/ci/conformance_test.sh | v2.16.0 (2026-11-10) |
| S136 | docs | harbor: README.md | https://github.com/goharbor/harbor/blob/v2.16.0/README.md | v2.16.0 (2026-11-10) |
| S137 | code | harbor: src/server/registry/referrers.go | https://github.com/goharbor/harbor/blob/v2.16.0/src/server/registry/referrers.go | v2.16.0 (2026-11-10) |
| S138 | code | harbor: src/controller/proxy/manifestcache.go | https://github.com/goharbor/harbor/blob/v2.16.0/src/controller/proxy/manifestcache.go | v2.16.0 (2026-11-10) |
| S139 | docs | Harbor docs 2.16.0: Configure Proxy Cache | https://goharbor.io/docs/2.16.0/administration/configure-proxy-cache/ | docs 2.16.0 (fetched 2026-11-20) |
| S140 | code | harbor: src/server/middleware/repoproxy/proxy.go | https://github.com/goharbor/harbor/blob/v2.16.0/src/server/middleware/repoproxy/proxy.go | v2.16.0 (2026-11-10) |
| S141 | code | harbor: make/photon/prepare/templates/core/env.jinja | https://github.com/goharbor/harbor/blob/v2.16.0/make/photon/prepare/templates/core/env.jinja | v2.16.0 (2026-11-10) |
| S142 | code | harbor: src/controller/proxy/controller.go | https://github.com/goharbor/harbor/blob/v2.16.0/src/controller/proxy/controller.go | v2.16.0 (2026-11-10) |
| S143 | code | harbor: api/v2.0/swagger.yaml | https://github.com/goharbor/harbor/blob/v2.16.0/api/v2.0/swagger.yaml | v2.16.0 (2026-11-10) |
| S144 | code | harbor: src/controller/proxy/local.go | https://github.com/goharbor/harbor/blob/v2.16.0/src/controller/proxy/local.go | v2.16.0 (2026-11-10) |
| S145 | code | harbor: src/pkg/proxy/connection/limit.go | https://github.com/goharbor/harbor/blob/v2.16.0/src/pkg/proxy/connection/limit.go | v2.16.0 (2026-11-10) |
| S146 | forum | goharbor/harbor issue 23025: Proxy Cache project not working on v2.15.0 | https://github.com/goharbor/harbor/issues/23025 | opened 2026-03-23, closed |
| S147 | forum | goharbor/harbor issue 18335: ProxyCache project doesn't tag saved artifacts | https://github.com/goharbor/harbor/issues/18335 | opened 2023-03-10, open |
| S148 | docs | Harbor docs 2.16.0: Configuring Replication | https://goharbor.io/docs/2.16.0/administration/configuring-replication/ | docs 2.16.0 (fetched 2026-11-20) |
| S149 | docs | Harbor docs 2.16.0: Creating a Replication Rule | https://goharbor.io/docs/2.16.0/administration/configuring-replication/create-replication-rules/ | docs 2.16.0 (fetched 2026-11-20) |
| S150 | code | harbor: src/controller/replication/transfer/image/transfer.go | https://github.com/goharbor/harbor/blob/v2.16.0/src/controller/replication/transfer/image/transfer.go | v2.16.0 (2026-11-10) |
| S151 | docs | Harbor docs 2.16.0: Configuring Authentication | https://goharbor.io/docs/2.16.0/administration/configure-authentication/ | docs 2.16.0 (fetched 2026-11-20) |
| S152 | docs | Harbor docs 2.16.0: Configure OIDC Provider Authentication | https://goharbor.io/docs/2.16.0/administration/configure-authentication/oidc-auth/ | docs 2.16.0 (fetched 2026-11-20) |
| S153 | docs | Harbor docs 2.16.0: Managing Users | https://goharbor.io/docs/2.16.0/administration/managing-users/ | docs 2.16.0 (fetched 2026-11-20) |
| S154 | docs | Harbor docs 2.16.0: Create System Robot Accounts | https://goharbor.io/docs/2.16.0/administration/robot-accounts/ | docs 2.16.0 (fetched 2026-11-20) |
| S155 | code | harbor: src/server/registry/catalog.go | https://github.com/goharbor/harbor/blob/v2.16.0/src/server/registry/catalog.go | v2.16.0 (2026-11-10) |
| S156 | docs | Harbor docs 2.16.0: Vulnerability Scanning | https://goharbor.io/docs/2.16.0/administration/vulnerability-scanning/ | docs 2.16.0 (fetched 2026-11-20) |
| S157 | docs | Harbor docs 2.16.0: SBOM Generation Capabilities | https://goharbor.io/docs/2.16.0/administration/sbom-integration/ | docs 2.16.0 (fetched 2026-11-20) |
| S158 | docs | Harbor docs 2.16.0: Implementing Content Trust | https://goharbor.io/docs/2.16.0/working-with-projects/project-configuration/implementing-content-trust/ | docs 2.16.0 (fetched 2026-11-20) |
| S159 | code | harbor: src/server/middleware/contenttrust/contentrust.go | https://github.com/goharbor/harbor/blob/v2.16.0/src/server/middleware/contenttrust/contentrust.go | v2.16.0 (2026-11-10) |
| S160 | docs | Harbor docs 2.16.0: Project Configuration | https://goharbor.io/docs/2.16.0/working-with-projects/project-configuration/ | docs 2.16.0 (fetched 2026-11-20) |
| S161 | docs | Harbor docs 2.16.0: Configure Project Quotas | https://goharbor.io/docs/2.16.0/administration/configure-project-quotas/ | docs 2.16.0 (fetched 2026-11-20) |
| S162 | code | harbor: src/server/middleware/blob/post_initiate_blob_upload.go | https://github.com/goharbor/harbor/blob/v2.16.0/src/server/middleware/blob/post_initiate_blob_upload.go | v2.16.0 (2026-11-10) |
| S163 | forum | goharbor/harbor issue 16915: Harbor exausts postgresql connections | https://github.com/goharbor/harbor/issues/16915 | opened 2022-05-26, closed |
| S164 | code | harbor: make/photon/prepare/templates/jobservice/config.yml.jinja | https://github.com/goharbor/harbor/blob/v2.16.0/make/photon/prepare/templates/jobservice/config.yml.jinja | v2.16.0 (2026-11-10) |
| S165 | code | harbor: src/lib/config/systemconfig.go | https://github.com/goharbor/harbor/blob/v2.16.0/src/lib/config/systemconfig.go | v2.16.0 (2026-11-10) |
| S166 | docs | goharbor/perf wiki: Harbor 2.15.4 performance report | https://github.com/goharbor/perf/wiki/Harbor-2.15.4 | Harbor v2.15.4; page edited 2026-11-06 |
| S167 | code | goharbor/perf: scripts/config.js | https://github.com/goharbor/perf/blob/4c8c19d400ab3075c6a9da18307b4bcf6e30c494/scripts/config.js | commit 4c8c19d (main, fetched 2026-11-20) |
| S168 | code | goharbor/perf: scripts/test/pull-artifacts-from-same-project.js | https://github.com/goharbor/perf/blob/4c8c19d400ab3075c6a9da18307b4bcf6e30c494/scripts/test/pull-artifacts-from-same-project.js | commit 4c8c19d (main, fetched 2026-11-20) |
| S169 | docs | goharbor/perf wiki: Harbor 2.14.4 performance report | https://github.com/goharbor/perf/wiki/Harbor-2.14.4 | Harbor v2.14.4; page edited 2026-11-06 |
| S170 | code | harbor v2.15.4: src/common/utils/encrypt.go | https://github.com/goharbor/harbor/blob/v2.15.4/src/common/utils/encrypt.go | v2.15.4 |
| S171 | code | harbor v2.14.4: src/common/utils/encrypt.go | https://github.com/goharbor/harbor/blob/v2.14.4/src/common/utils/encrypt.go | v2.14.4 |
| S172 | docs | goharbor/perf wiki: Harbor 2.13.0 performance report | https://github.com/goharbor/perf/wiki/Harbor-2.13.0 | Harbor v2.13.0; page edited 2025-04-16 |
| S173 | docs | goharbor/perf wiki: Cache layer | https://github.com/goharbor/perf/wiki/Cache-layer | page edited 2022-09-28 (Harbor v2.6) |
| S174 | docs | goharbor/perf wiki: Quota Update | https://github.com/goharbor/perf/wiki/Quota-Update | page edited 2023-07-10 (Harbor v2.9) |
| S175 | docs | goharbor/perf wiki: Harbor 2.10.0 performance report | https://github.com/goharbor/perf/wiki/Harbor-2.10.0 | Harbor v2.10.0; page edited 2025-04-16 |
| S176 | forum | goharbor/harbor issue 20505: Performance Improvement Proposal for queries related with the Task Table | https://github.com/goharbor/harbor/issues/20505 | opened 2024-05-28, closed |
| S177 | docs | Harbor v2.16.0 release assets | https://github.com/goharbor/harbor/releases/expanded_assets/v2.16.0 | 2026-11-10 |
| S178 | forum | goharbor/harbor issue 15048: harbor-core sporadically throws HTTP.5xx - Database performance related | https://github.com/goharbor/harbor/issues/15048 | opened 2021-06-02, closed |
| S179 | docs | Harbor docs 2.16.0: Upgrade Harbor and Migrate Data | https://goharbor.io/docs/2.16.0/administration/upgrade/ | docs 2.16.0 (fetched 2026-11-20) |
| S180 | docs | Harbor release notes v2.9.0 | https://github.com/goharbor/harbor/releases/tag/v2.9.0 | 2023-09-01 |
| S181 | docs | Harbor design proposal: Support OCI distribution spec v1.1.0 in Harbor | https://github.com/goharbor/community/blob/58c9d106428b98f55ae2a891e1c64f0715280144/proposals/new/distribution-1.1-adoption.md | proposal for v2.9 (2023); commit 58c9d10 |
| S182 | forum | goharbor/harbor issue 22111: GC issue when using S3 as backend storage | https://github.com/goharbor/harbor/issues/22111 | opened 2025-06-23, open |
| S183 | docs | Harbor docs 2.16.0: Deploying Harbor with High Availability via Helm | https://goharbor.io/docs/2.16.0/install-config/harbor-ha-helm/ | docs 2.16.0 (fetched 2026-11-20) |
| S184 | docs | Harbor docs 2.16.0: Configure the Harbor YML File | https://goharbor.io/docs/2.16.0/install-config/configure-yml-file/ | docs 2.16.0 (fetched 2026-11-20) |
| S185 | docs | Harbor docs 2.16.0: Create Tag Retention Rules | https://goharbor.io/docs/2.16.0/working-with-projects/working-with-images/create-tag-retention-rules/ | docs 2.16.0 (fetched 2026-11-20) |
| S186 | docs | Harbor docs 2.16.0: Create Tag Immutability Rules | https://goharbor.io/docs/2.16.0/working-with-projects/working-with-images/create-tag-immutability-rules/ | docs 2.16.0 (fetched 2026-11-20) |
| S187 | code | harbor: src/server/v2.0/handler/retention.go | https://github.com/goharbor/harbor/blob/v2.16.0/src/server/v2.0/handler/retention.go | v2.16.0 (2026-11-10) |
| S188 | forum | goharbor/harbor issue 14358: High CPU usage (core and database) after 2.2.0 upgrade | https://github.com/goharbor/harbor/issues/14358 | opened 2021-03-04, closed |
| S189 | docs | Harbor design proposal: Pull through proxy cache | https://github.com/goharbor/community/blob/58c9d106428b98f55ae2a891e1c64f0715280144/proposals/new/pull_through_proxy_cache.md | proposal for v2.1 (2020); commit 58c9d10 |
| S190 | code | harbor: src/common/utils/encrypt.go | https://github.com/goharbor/harbor/blob/v2.16.0/src/common/utils/encrypt.go | v2.16.0 (2026-11-10) |
| S191 | code | harbor: make/photon/prepare/templates/nginx/nginx.https.conf.jinja | https://github.com/goharbor/harbor/blob/v2.16.0/make/photon/prepare/templates/nginx/nginx.https.conf.jinja | v2.16.0 (2026-11-10) |
| S192 | code | harbor: make/photon/prepare/templates/exporter/env.jinja | https://github.com/goharbor/harbor/blob/v2.16.0/make/photon/prepare/templates/exporter/env.jinja | v2.16.0 (2026-11-10) |
| S193 | code | harbor: src/pkg/scan/job.go | https://github.com/goharbor/harbor/blob/v2.16.0/src/pkg/scan/job.go | v2.16.0 (2026-11-10) |
| S194 | code | harbor: src/common/api/base.go | https://github.com/goharbor/harbor/blob/v2.16.0/src/common/api/base.go | v2.16.0 (2026-11-10) |
| S195 | forum | goharbor/harbor issue 14774: Using buildkit exporting cache reporting status: 404 Not Found | https://github.com/goharbor/harbor/issues/14774 | opened 2021-04-28, open |
| S196 | docs | OCI Conformance: OCI-Conformant Products | https://conformance.opencontainers.org/ | fetched 2026-11-20 |
| S197 | code | goharbor/harbor-helm: values.yaml | https://github.com/goharbor/harbor-helm/blob/v1.19.4/values.yaml | chart v1.19.4, appVersion 2.15.4 (2026-10-16) |
| S198 | code | harbor: src/server/registry/proxy.go | https://github.com/goharbor/harbor/blob/v2.16.0/src/server/registry/proxy.go | v2.16.0 (2026-11-10) |
| S199 | code | harbor: src/controller/blob/controller.go | https://github.com/goharbor/harbor/blob/v2.16.0/src/controller/blob/controller.go | v2.16.0 (2026-11-10) |
| S200 | code | zot: README.md | https://github.com/project-zot/zot/blob/v2.1.23/README.md | v2.1.23 (2026-11-02) |
| S201 | docs | zot: examples/README.md (in-repo configuration reference) | https://github.com/project-zot/zot/blob/v2.1.23/examples/README.md | v2.1.23 (2026-11-02) |
| S202 | code | zot: pkg/api/config/config.go | https://github.com/project-zot/zot/blob/v2.1.23/pkg/api/config/config.go | v2.1.23 (2026-11-02) |
| S203 | code | zot: pkg/api/constants/consts.go | https://github.com/project-zot/zot/blob/v2.1.23/pkg/api/constants/consts.go | v2.1.23 (2026-11-02) |
| S204 | code | zot: pkg/storage/constants/constants.go | https://github.com/project-zot/zot/blob/v2.1.23/pkg/storage/constants/constants.go | v2.1.23 (2026-11-02) |
| S205 | code | zot: pkg/storage/gc/gc.go | https://github.com/project-zot/zot/blob/v2.1.23/pkg/storage/gc/gc.go | v2.1.23 (2026-11-02) |
| S206 | code | zot: pkg/storage/types/types.go | https://github.com/project-zot/zot/blob/v2.1.23/pkg/storage/types/types.go | v2.1.23 (2026-11-02) |
| S207 | code | zot: pkg/storage/imagestore/imagestore.go | https://github.com/project-zot/zot/blob/v2.1.23/pkg/storage/imagestore/imagestore.go | v2.1.23 (2026-11-02) |
| S208 | code | zot: pkg/storage/imagestore/blob_lifecycle.go | https://github.com/project-zot/zot/blob/v2.1.23/pkg/storage/imagestore/blob_lifecycle.go | v2.1.23 (2026-11-02) |
| S209 | code | zot: pkg/api/controller.go | https://github.com/project-zot/zot/blob/v2.1.23/pkg/api/controller.go | v2.1.23 (2026-11-02) |
| S210 | code | zot: pkg/cli/server/root.go | https://github.com/project-zot/zot/blob/v2.1.23/pkg/cli/server/root.go | v2.1.23 (2026-11-02) |
| S211 | code | zot: pkg/api/routes.go | https://github.com/project-zot/zot/blob/v2.1.23/pkg/api/routes.go | v2.1.23 (2026-11-02) |
| S212 | code | zot: pkg/scheduler/scheduler.go | https://github.com/project-zot/zot/blob/v2.1.23/pkg/scheduler/scheduler.go | v2.1.23 (2026-11-02) |
| S213 | code | zot: pkg/cluster/cluster.go | https://github.com/project-zot/zot/blob/v2.1.23/pkg/cluster/cluster.go | v2.1.23 (2026-11-02) |
| S214 | code | zot: pkg/extensions/sync/on_demand.go | https://github.com/project-zot/zot/blob/v2.1.23/pkg/extensions/sync/on_demand.go | v2.1.23 (2026-11-02) |
| S215 | code | zot: pkg/extensions/sync/constants/constants.go | https://github.com/project-zot/zot/blob/v2.1.23/pkg/extensions/sync/constants/constants.go | v2.1.23 (2026-11-02) |
| S216 | code | zot: pkg/extensions/config/sync/config.go | https://github.com/project-zot/zot/blob/v2.1.23/pkg/extensions/config/sync/config.go | v2.1.23 (2026-11-02) |
| S217 | code | zot: pkg/extensions/extension_search.go | https://github.com/project-zot/zot/blob/v2.1.23/pkg/extensions/extension_search.go | v2.1.23 (2026-11-02) |
| S218 | code | zot: pkg/extensions/events/events.go | https://github.com/project-zot/zot/blob/v2.1.23/pkg/extensions/events/events.go | v2.1.23 (2026-11-02) |
| S219 | code | zot: pkg/api/cookiestore.go | https://github.com/project-zot/zot/blob/v2.1.23/pkg/api/cookiestore.go | v2.1.23 (2026-11-02) |
| S220 | code | zot: .github/workflows/benchmark.yaml | https://github.com/project-zot/zot/blob/v2.1.23/.github/workflows/benchmark.yaml | v2.1.23 (2026-11-02) |
| S221 | code | zot: Makefile | https://github.com/project-zot/zot/blob/v2.1.23/Makefile | v2.1.23 (2026-11-02) |
| S222 | docs | zot: cmd/zb/README.md | https://github.com/project-zot/zot/blob/v2.1.23/cmd/zb/README.md | v2.1.23 (2026-11-02) |
| S223 | code | zot: .github/workflows/oci-conformance-action.yaml | https://github.com/project-zot/zot/blob/v2.1.23/.github/workflows/oci-conformance-action.yaml | v2.1.23 (2026-11-02) |
| S224 | code | zot: .github/workflows/build.yaml | https://github.com/project-zot/zot/blob/v2.1.23/.github/workflows/build.yaml | v2.1.23 (2026-11-02) |
| S225 | code | zot: .github/workflows/compare-binary-size.yaml | https://github.com/project-zot/zot/blob/v2.1.23/.github/workflows/compare-binary-size.yaml | v2.1.23 (2026-11-02) |
| S226 | code | zot: pkg/extensions/config/config.go | https://github.com/project-zot/zot/blob/v2.1.23/pkg/extensions/config/config.go | v2.1.23 (2026-11-02) |
| S227 | code | zot: go.mod | https://github.com/project-zot/zot/blob/v2.1.23/go.mod | v2.1.23 (2026-11-02) |
| S228 | code | zot: pkg/storage/common/common.go | https://github.com/project-zot/zot/blob/v2.1.23/pkg/storage/common/common.go | v2.1.23 (2026-11-02) |
| S229 | docs | zot docs: Storage Planning with zot | https://zotregistry.dev/v2.1.23/articles/storage/ | docs v2.1.23; page dated 2026-11-02 |
| S230 | docs | zot docs: OCI Registry Mirroring With zot | https://zotregistry.dev/v2.1.23/articles/mirroring/ | docs v2.1.23; page dated 2026-11-02 |
| S231 | docs | zot docs: Configuring zot Tag Retention Policies | https://zotregistry.dev/v2.1.23/articles/retention/ | docs v2.1.23; page dated 2026-11-02 |
| S232 | docs | zot docs: Scaling zot (scale-out clustering) | https://zotregistry.dev/v2.1.23/articles/scaleout/ | docs v2.1.23; page dated 2025-11-14 |
| S233 | docs | zot docs: User Authentication and Authorization with zot | https://zotregistry.dev/v2.1.23/articles/authn-authz/ | docs v2.1.23; page dated 2026-11-02 |
| S234 | docs | zot docs: Benchmarking zot with zb | https://zotregistry.dev/v2.1.23/articles/benchmarking-with-zb/ | docs v2.1.23; page dated 2025-12-04 |
| S235 | docs | zot docs: Verifying image signatures | https://zotregistry.dev/v2.1.23/articles/verifying-signatures/ | docs v2.1.23; page dated 2026-10-06 |
| S236 | docs | zot docs: Event Support for zot | https://zotregistry.dev/v2.1.23/articles/events/ | docs v2.1.23; page dated 2026-09-06 |
| S237 | docs | zot docs: CVE Scanning with zot | https://zotregistry.dev/v2.1.23/articles/cve-scanning/ | docs v2.1.23; page dated 2026-09-06 |
| S238 | docs | zot docs: Using GraphQL for Enhanced Searches | https://zotregistry.dev/v2.1.23/articles/graphql/ | docs v2.1.23; page dated 2026-08-04 |
| S239 | docs | zot docs: What's New (per-release notes v1.4.3 to v2.1.23) | https://zotregistry.dev/v2.1.23/general/whats-new/ | docs v2.1.23; page dated 2026-11-02 |
| S240 | docs | zot docs: zot Security Posture | https://zotregistry.dev/v2.1.23/articles/security-posture/ | docs v2.1.23; page dated 2026-11-02 |
| S241 | docs | zot docs: Released Images for zot | https://zotregistry.dev/v2.1.23/general/releases/ | docs v2.1.23; page dated 2025-12-04 |
| S242 | docs | zot docs: Concepts | https://zotregistry.dev/v2.1.23/general/concepts/ | docs v2.1.23; page dated 2023-05-18 |
| S243 | docs | zot docs: Architecture | https://zotregistry.dev/v2.1.23/general/architecture/ | docs v2.1.23; page dated 2023-09-13 |
| S244 | docs | zot docs: Deploying a Highly Available zot Registry | https://zotregistry.dev/v2.1.23/articles/high-availability/ | docs v2.1.23; page dated 2024-06-24 |
| S245 | docs | zot docs: Immutable Image Tags | https://zotregistry.dev/v2.1.23/articles/immutable-tags/ | docs v2.1.23; page dated 2026-11-19 |
| S246 | docs | zot docs: Configuring zot (admin guide) | https://zotregistry.dev/v2.1.23/admin-guide/admin-configuration/ | docs v2.1.23; page dated 2026-11-02 |
| S247 | docs | zot docs: API Reference | https://zotregistry.dev/v2.1.23/developer-guide/api-reference/ | docs v2.1.23; page dated 2026-11-02 |
| S248 | docs | zot docs: Installing zot on Bare Metal Linux | https://zotregistry.dev/v2.1.23/install-guides/install-guide-linux/ | docs v2.1.23; page dated 2026-10-06 |
| S249 | docs | zot v2.1.23 release: asset list (names, sizes, timestamps) | https://github.com/project-zot/zot/releases/expanded_assets/v2.1.23 | v2.1.23 (2026-11-02) |
| S250 | docs | zot release page v2.1.23 | https://github.com/project-zot/zot/releases/tag/v2.1.23 | released 2026-11-02 |
| S251 | docs | zot release page v2.1.0 | https://github.com/project-zot/zot/releases/tag/v2.1.0 | released 2024-07-09 |
| S252 | docs | zot release page v2.1.3 | https://github.com/project-zot/zot/releases/tag/v2.1.3 | released 2025-05-22 |
| S253 | docs | zot release page v2.0.0 | https://github.com/project-zot/zot/releases/tag/v2.0.0 | released 2023-12-18 |
| S254 | docs | zot GitHub milestones | https://github.com/project-zot/zot/milestones | fetched 2026-11-20 |
| S255 | docs | OCI conformance submission for zot, distribution-spec v1.1 (README.md with results) | https://github.com/opencontainers/oci-conformance/blob/16097618fca61dc5d7aed970d61514aa8efad2c9/distribution-spec/v1.1/zot/README.md | zot v2.1.19; repository commit 1609761, fetched 2026-11-20 |
| S256 | docs | OCI conformance submission for zot, distribution-spec v1.1 (PRODUCT.yaml) | https://github.com/opencontainers/oci-conformance/blob/16097618fca61dc5d7aed970d61514aa8efad2c9/distribution-spec/v1.1/zot/PRODUCT.yaml | zot v2.1.19; repository commit 1609761, fetched 2026-11-20 |
| S257 | forum | zot issue #4550: GC deletes freshly pushed deduped blobs before the manifest arrives | https://github.com/project-zot/zot/issues/4550 | opened 2026-10-16; closed |
| S258 | forum | zot issue #2964: Significant performance hit due to excessive image store lock | https://github.com/project-zot/zot/issues/2964 | opened 2025-02-14; open on 2026-11-20 |
| S259 | forum | zot issue #2999: Push stuck because of restoring deduped blobs | https://github.com/project-zot/zot/issues/2999 | opened 2025-03-02; closed |
| S260 | forum | zot issue #3396: Garbage collect does not free disk space without retention policies | https://github.com/project-zot/zot/issues/3396 | opened 2025-09-18; closed |
| S261 | forum | zot issue #2137: Excessive Memory Usage | https://github.com/project-zot/zot/issues/2137 | opened 2023-12-13; closed |
| S262 | forum | zot issue #3129: Pull-through cache re-triggers upstream pulls, exhausting rate limits and downloading all archs | https://github.com/project-zot/zot/issues/3129 | opened 2025-05-02; closed |
| S263 | forum | zot issue #4098: Is possible to enforce signatures on images? | https://github.com/project-zot/zot/issues/4098 | opened 2026-05-25; closed |
| S264 | forum | zot issue #4079: Pushing an image with a layer that takes more than 60 seconds get aborted | https://github.com/project-zot/zot/issues/4079 | opened 2026-05-19; closed |
| S265 | forum | zot issue #4552: Zot default timeout causes failure when uploading large OCI artifacts | https://github.com/project-zot/zot/issues/4552 | opened 2026-10-16; closed |
| S266 | forum | zot issue #4271: GC aborts the whole repo (and blocks remaining repo tasks) when index.json references a missing blob | https://github.com/project-zot/zot/issues/4271 | opened 2026-07-28; closed |
| S267 | forum | zot issue #4471: storage: classify driver errors (missing vs transient) for safe GC / scrub / enumeration (opened by a maintainer) | https://github.com/project-zot/zot/issues/4471 | opened 2026-09-27; closed |
| S268 | forum | zot issue #4695: A repository x/blobs/sha256 is stored inside x's blob directory and breaks GC of x | https://github.com/project-zot/zot/issues/4695 | opened 2026-11-11; open on 2026-11-20 |
| S269 | forum | zot issue #3723: Scheduler Generator Stuck in Waiting State - Persistent Deadlock | https://github.com/project-zot/zot/issues/3723 | opened 2026-01-21; closed |
| S270 | forum | zot issue #4062: HA (active/active) with shared bucket | https://github.com/project-zot/zot/issues/4062 | opened 2026-05-12; closed |
| S271 | forum | zot issue #2627: Listing images in UI fails with DynamoDB and many repositories | https://github.com/project-zot/zot/issues/2627 | opened 2024-09-02; closed |
| S272 | forum | zot issue #4666: GC of a repository on S3 fails on every pass when a stale blob upload cannot be deleted | https://github.com/project-zot/zot/issues/4666 | opened 2026-11-05; closed |
| S273 | forum | zot issue #4444: Overwriting the last tag referencing a manifest makes it inaccessible by digest | https://github.com/project-zot/zot/issues/4444 | opened 2026-09-19; closed |
| S274 | forum | zot issue #2625: can't download image (error for several blobs) from s3 even right after image was pushed | https://github.com/project-zot/zot/issues/2625 | opened 2024-08-22; closed |
| S275 | docs | zot security advisories (list) | https://github.com/project-zot/zot/security/advisories | fetched 2026-11-20 |
| S276 | docs | zot advisory GHSA-c45h-jrgv-35rc: Unauthenticated on-demand sync requests create unbounded detached work | https://github.com/project-zot/zot/security/advisories/GHSA-c45h-jrgv-35rc | published 2026-10-23 |
| S277 | docs | zot advisory GHSA-82rc-37x2-7w36: CVE-2024-39897 incomplete fix (cross-repository blob reuse) | https://github.com/project-zot/zot/security/advisories/GHSA-82rc-37x2-7w36 | published 2026-09-11 |
| S278 | docs | zot advisory GHSA-55r9-5mx9-qq7r (CVE-2024-39897): Cache driver GetBlob() allows read access to any blob without access control check | https://github.com/project-zot/zot/security/advisories/GHSA-55r9-5mx9-qq7r | published 2024-07-09 |
| S279 | code | zot: pkg/extensions/sync/service.go | https://github.com/project-zot/zot/blob/v2.1.23/pkg/extensions/sync/service.go | v2.1.23 (2026-11-02) |
| S280 | code | tollbooth (rate limiter library used by zot): tollbooth.go | https://github.com/didip/tollbooth/blob/v7.0.2/tollbooth.go | v7.0.2 (version pinned in zot v2.1.23 go.mod) |
| S281 | vendor | peryx docs: OCI performance (secondary; third-party comparison of pull-through caches, written by the authors of a competing registry) | https://peryx.readthedocs.io/en/latest/ecosystems/oci/performance/ | unversioned "latest" docs; fetched 2026-11-20 |
| S282 | code | quay: config.py (DefaultConfig) | https://github.com/quay/quay/blob/v3.18.3/config.py | v3.18.3 (GitHub release 2026-11-13) |
| S283 | code | quay: data/model/gc.py | https://github.com/quay/quay/blob/v3.18.3/data/model/gc.py | v3.18.3 |
| S284 | code | quay: data/model/storage.py | https://github.com/quay/quay/blob/v3.18.3/data/model/storage.py | v3.18.3 |
| S285 | code | quay: workers/gc/gcworker.py | https://github.com/quay/quay/blob/v3.18.3/workers/gc/gcworker.py | v3.18.3 |
| S286 | code | quay: data/model/oci/tag.py | https://github.com/quay/quay/blob/v3.18.3/data/model/oci/tag.py | v3.18.3 |
| S287 | code | quay: data/model/repository.py | https://github.com/quay/quay/blob/v3.18.3/data/model/repository.py | v3.18.3 |
| S288 | code | quay: workers/blobuploadcleanupworker/blobuploadcleanupworker.py | https://github.com/quay/quay/blob/v3.18.3/workers/blobuploadcleanupworker/blobuploadcleanupworker.py | v3.18.3 |
| S289 | code | quay: endpoints/v2/__init__.py | https://github.com/quay/quay/blob/v3.18.3/endpoints/v2/__init__.py | v3.18.3 |
| S290 | code | quay: endpoints/v2/blob.py | https://github.com/quay/quay/blob/v3.18.3/endpoints/v2/blob.py | v3.18.3 |
| S291 | code | quay: endpoints/v2/referrers.py | https://github.com/quay/quay/blob/v3.18.3/endpoints/v2/referrers.py | v3.18.3 |
| S292 | code | quay: digest/digest_tools.py | https://github.com/quay/quay/blob/v3.18.3/digest/digest_tools.py | v3.18.3 |
| S293 | code | quay: data/registry_model/blobuploader.py | https://github.com/quay/quay/blob/v3.18.3/data/registry_model/blobuploader.py | v3.18.3 |
| S294 | code | quay: storage/cloud.py | https://github.com/quay/quay/blob/v3.18.3/storage/cloud.py | v3.18.3 |
| S295 | code | quay: conf/nginx/server-base.conf.jnj | https://github.com/quay/quay/blob/v3.18.3/conf/nginx/server-base.conf.jnj | v3.18.3 |
| S296 | code | quay: conf/nginx/rate-limiting.conf.jnj | https://github.com/quay/quay/blob/v3.18.3/conf/nginx/rate-limiting.conf.jnj | v3.18.3 |
| S297 | code | quay: conf/gunicorn_registry.py | https://github.com/quay/quay/blob/v3.18.3/conf/gunicorn_registry.py | v3.18.3 |
| S298 | code | quay: conf/supervisord.conf.jnj | https://github.com/quay/quay/blob/v3.18.3/conf/supervisord.conf.jnj | v3.18.3 |
| S299 | code | quay: data/database.py | https://github.com/quay/quay/blob/v3.18.3/data/database.py | v3.18.3 |
| S300 | code | quay: proxy/__init__.py | https://github.com/quay/quay/blob/v3.18.3/proxy/__init__.py | v3.18.3 |
| S301 | code | quay: data/registry_model/registry_proxy_model.py | https://github.com/quay/quay/blob/v3.18.3/data/registry_model/registry_proxy_model.py | v3.18.3 |
| S302 | code | quay: util/repomirror/skopeomirror.py | https://github.com/quay/quay/blob/v3.18.3/util/repomirror/skopeomirror.py | v3.18.3 |
| S303 | code | quay: data/model/repo_mirror.py | https://github.com/quay/quay/blob/v3.18.3/data/model/repo_mirror.py | v3.18.3 |
| S304 | code | quay: .github/workflows/ci-python.yaml (job "OCI Distribution Spec") | https://github.com/quay/quay/blob/v3.18.3/.github/workflows/ci-python.yaml | v3.18.3 |
| S305 | code | quay: .github/workflows/ci-go.yaml (job "OCI Conformance") | https://github.com/quay/quay/blob/v3.18.3/.github/workflows/ci-go.yaml | v3.18.3 |
| S306 | code | quay: .github/workflows/build-and-publish.yaml | https://github.com/quay/quay/blob/v3.18.3/.github/workflows/build-and-publish.yaml | v3.18.3 |
| S307 | docs | quay: CHANGELOG.md | https://github.com/quay/quay/blob/v3.18.3/CHANGELOG.md | v3.18.3 |
| S308 | docs | quay: README.md | https://github.com/quay/quay/blob/v3.18.3/README.md | v3.18.3 |
| S309 | docs | quay: ARCHITECTURE.md | https://github.com/quay/quay/blob/v3.18.3/ARCHITECTURE.md | v3.18.3 |
| S310 | code | quay: storage/__init__.py | https://github.com/quay/quay/blob/v3.18.3/storage/__init__.py | v3.18.3 |
| S311 | code | quay: endpoints/api/search.py | https://github.com/quay/quay/blob/v3.18.3/endpoints/api/search.py | v3.18.3 |
| S312 | code | quay: util/config/schema.py | https://github.com/quay/quay/blob/v3.18.3/util/config/schema.py | v3.18.3 |
| S313 | code | quay (Go registry): internal/registry/distribution/middleware/subject.go | https://github.com/quay/quay/blob/v3.18.3/internal/registry/distribution/middleware/subject.go | v3.18.3 |
| S314 | code | quay: endpoints/v2/v2auth.py | https://github.com/quay/quay/blob/v3.18.3/endpoints/v2/v2auth.py | v3.18.3 |
| S315 | code | quay: initdb.py | https://github.com/quay/quay/blob/v3.18.3/initdb.py | v3.18.3 |
| S316 | code | quay: endpoints/v2/manifest.py | https://github.com/quay/quay/blob/v3.18.3/endpoints/v2/manifest.py | v3.18.3 |
| S317 | code | quay: data/model/oci/manifest.py | https://github.com/quay/quay/blob/v3.18.3/data/model/oci/manifest.py | v3.18.3 |
| S318 | code | quay: workers/repositorygcworker.py | https://github.com/quay/quay/blob/v3.18.3/workers/repositorygcworker.py | v3.18.3 |
| S319 | code | quay (Go mirror registry): internal/gc/sqlite_collector.go | https://github.com/quay/quay/blob/mirror-registry-v3.0.0/internal/gc/sqlite_collector.go | tag mirror-registry-v3.0.0 (2026-11-09) |
| S320 | code | quay (Go mirror registry): internal/gc/gc.go | https://github.com/quay/quay/blob/mirror-registry-v3.0.0/internal/gc/gc.go | tag mirror-registry-v3.0.0 (2026-11-09) |
| S321 | code | quay (Go mirror registry): Dockerfile.mirror | https://github.com/quay/quay/blob/mirror-registry-v3.0.0/Dockerfile.mirror | tag mirror-registry-v3.0.0 (2026-11-09) |
| S322 | code | quay (Go mirror registry): go.mod | https://github.com/quay/quay/blob/mirror-registry-v3.0.0/go.mod | tag mirror-registry-v3.0.0 (2026-11-09) |
| S323 | docs | quay-docs 3.18: modules/garbage-collection-practice.adoc | https://github.com/quay/quay-docs/blob/a4420c1359090a2735003589fe74240db605737e/modules/garbage-collection-practice.adoc | redhat-3.18 @ a4420c1 (2026-11-13) |
| S324 | docs | quay-docs 3.18: modules/core-prereqs-db.adoc | https://github.com/quay/quay-docs/blob/a4420c1359090a2735003589fe74240db605737e/modules/core-prereqs-db.adoc | redhat-3.18 @ a4420c1 |
| S325 | docs | quay-docs 3.18: modules/core-prereqs-redis.adoc | https://github.com/quay/quay-docs/blob/a4420c1359090a2735003589fe74240db605737e/modules/core-prereqs-redis.adoc | redhat-3.18 @ a4420c1 |
| S326 | docs | quay-docs 3.18: modules/core-prereqs-storage.adoc | https://github.com/quay/quay-docs/blob/a4420c1359090a2735003589fe74240db605737e/modules/core-prereqs-storage.adoc | redhat-3.18 @ a4420c1 |
| S327 | docs | quay-docs 3.18: modules/quay-as-cache-proxy.adoc | https://github.com/quay/quay-docs/blob/a4420c1359090a2735003589fe74240db605737e/modules/quay-as-cache-proxy.adoc | redhat-3.18 @ a4420c1 |
| S328 | docs | quay-docs 3.18: modules/proxy-cache-arch.adoc | https://github.com/quay/quay-docs/blob/a4420c1359090a2735003589fe74240db605737e/modules/proxy-cache-arch.adoc | redhat-3.18 @ a4420c1 |
| S329 | docs | quay-docs 3.18: modules/proxy-cache-limitations.adoc | https://github.com/quay/quay-docs/blob/a4420c1359090a2735003589fe74240db605737e/modules/proxy-cache-limitations.adoc | redhat-3.18 @ a4420c1 |
| S330 | docs | quay-docs 3.18: modules/proxy-cache-procedure.adoc | https://github.com/quay/quay-docs/blob/a4420c1359090a2735003589fe74240db605737e/modules/proxy-cache-procedure.adoc | redhat-3.18 @ a4420c1 |
| S331 | docs | quay-docs 3.18: modules/proxy-cache-leveraging-storage-quota-limits.adoc | https://github.com/quay/quay-docs/blob/a4420c1359090a2735003589fe74240db605737e/modules/proxy-cache-leveraging-storage-quota-limits.adoc | redhat-3.18 @ a4420c1 |
| S332 | docs | quay-docs 3.18: modules/mirroring-using.adoc | https://github.com/quay/quay-docs/blob/a4420c1359090a2735003589fe74240db605737e/modules/mirroring-using.adoc | redhat-3.18 @ a4420c1 |
| S333 | docs | quay-docs 3.18: modules/mirroring-creating-repository.adoc | https://github.com/quay/quay-docs/blob/a4420c1359090a2735003589fe74240db605737e/modules/mirroring-creating-repository.adoc | redhat-3.18 @ a4420c1 |
| S334 | docs | quay-docs 3.18: modules/mirroring-recommend.adoc | https://github.com/quay/quay-docs/blob/a4420c1359090a2735003589fe74240db605737e/modules/mirroring-recommend.adoc | redhat-3.18 @ a4420c1 |
| S335 | docs | quay-docs 3.18: modules/georepl-prereqs.adoc | https://github.com/quay/quay-docs/blob/a4420c1359090a2735003589fe74240db605737e/modules/georepl-prereqs.adoc | redhat-3.18 @ a4420c1 |
| S336 | docs | quay-docs 3.18: modules/sizing-intro.adoc | https://github.com/quay/quay-docs/blob/a4420c1359090a2735003589fe74240db605737e/modules/sizing-intro.adoc | redhat-3.18 @ a4420c1 |
| S337 | docs | quay-docs 3.18: modules/sizing-sample.adoc | https://github.com/quay/quay-docs/blob/a4420c1359090a2735003589fe74240db605737e/modules/sizing-sample.adoc | redhat-3.18 @ a4420c1 |
| S338 | docs | quay-docs 3.18: modules/preparing-registry-large-artifacts.adoc | https://github.com/quay/quay-docs/blob/a4420c1359090a2735003589fe74240db605737e/modules/preparing-registry-large-artifacts.adoc | redhat-3.18 @ a4420c1 |
| S339 | docs | quay-docs 3.18: modules/oci-intro.adoc | https://github.com/quay/quay-docs/blob/a4420c1359090a2735003589fe74240db605737e/modules/oci-intro.adoc | redhat-3.18 @ a4420c1 |
| S340 | docs | quay-docs 3.18: modules/set-team-role.adoc | https://github.com/quay/quay-docs/blob/a4420c1359090a2735003589fe74240db605737e/modules/set-team-role.adoc | redhat-3.18 @ a4420c1 |
| S341 | docs | quay-docs 3.18: modules/robot-account-overview.adoc | https://github.com/quay/quay-docs/blob/a4420c1359090a2735003589fe74240db605737e/modules/robot-account-overview.adoc | redhat-3.18 @ a4420c1 |
| S342 | docs | quay-docs 3.18: modules/keyless-authentication-robot-accounts.adoc | https://github.com/quay/quay-docs/blob/a4420c1359090a2735003589fe74240db605737e/modules/keyless-authentication-robot-accounts.adoc | redhat-3.18 @ a4420c1 |
| S343 | docs | quay-docs 3.18: modules/cosign-oci-intro.adoc | https://github.com/quay/quay-docs/blob/a4420c1359090a2735003589fe74240db605737e/modules/cosign-oci-intro.adoc | redhat-3.18 @ a4420c1 |
| S344 | docs | quay-docs 3.18: modules/arch-intro-security.adoc | https://github.com/quay/quay-docs/blob/a4420c1359090a2735003589fe74240db605737e/modules/arch-intro-security.adoc | redhat-3.18 @ a4420c1 |
| S345 | docs | quay-docs 3.18: modules/config-fields-rate-limit-performance.adoc | https://github.com/quay/quay-docs/blob/a4420c1359090a2735003589fe74240db605737e/modules/config-fields-rate-limit-performance.adoc | redhat-3.18 @ a4420c1 |
| S346 | docs | quay-docs 3.18: modules/new-features-and-enhancements-318.adoc | https://github.com/quay/quay-docs/blob/a4420c1359090a2735003589fe74240db605737e/modules/new-features-and-enhancements-318.adoc | redhat-3.18 @ a4420c1 |
| S347 | docs | quay-docs master (5.0 draft): modules/new-features-and-enhancements-500.adoc | https://github.com/quay/quay-docs/blob/1350e7d1257ae9a124ceeef3a1f44720a2396c9e/modules/new-features-and-enhancements-500.adoc | master @ 1350e7d (2026-11-18) |
| S348 | docs | quay-docs master (5.0 draft): _attributes/attributes.adoc | https://github.com/quay/quay-docs/blob/1350e7d1257ae9a124ceeef3a1f44720a2396c9e/_attributes/attributes.adoc | master @ 1350e7d (2026-11-18) |
| S349 | docs | quay-docs master (5.0 draft): modules/config-fields-proxy-cache.adoc | https://github.com/quay/quay-docs/blob/1350e7d1257ae9a124ceeef3a1f44720a2396c9e/modules/config-fields-proxy-cache.adoc | master @ 1350e7d (2026-11-18) |
| S350 | docs | quay-docs 3.17: modules/new-features-and-enhancements-317.adoc | https://github.com/quay/quay-docs/blob/14005dd9a34b3315948a7399c7d468841555d687/modules/new-features-and-enhancements-317.adoc | redhat-3.17 @ 14005dd |
| S351 | docs | quay-docs 3.16: modules/new-features-and-enhancements-316.adoc | https://github.com/quay/quay-docs/blob/1076fa5a52a5742f4748c8ef360141fa7c33f7e9/modules/new-features-and-enhancements-316.adoc | redhat-3.16 @ 1076fa5 |
| S352 | docs | quay-docs 3.15: modules/new-features-and-enhancements-315.adoc | https://github.com/quay/quay-docs/blob/778459415eaf4b70ce8022108d89d0add4121121/modules/new-features-and-enhancements-315.adoc | redhat-3.15 @ 7784594 |
| S353 | docs | quay-docs 3.14: modules/new-features-and-enhancements-314.adoc | https://github.com/quay/quay-docs/blob/81ada6d15a4df113033602cd6ff79ad8f7011278/modules/new-features-and-enhancements-314.adoc | redhat-3.14 @ 81ada6d |
| S354 | forum | Tracker PROJQUAY-14526: Blob upload returns 500 when garbage collection deletes the deduplicated storage row mid-upload | https://redhat.atlassian.net/browse/PROJQUAY-14526 | opened 2026-11-17, status New |
| S355 | forum | Tracker PROJQUAY-12886: OMR 3.0: digest-only / multi-arch child manifests collected within ~30s | https://redhat.atlassian.net/browse/PROJQUAY-12886 | opened 2026-08-31, closed 2026-10-01 |
| S356 | forum | Tracker PROJQUAY-2451: Quay is not garbage collecting blobs correctly in repositorygcworker | https://redhat.atlassian.net/browse/PROJQUAY-2451 | opened 2021-08-24, closed 2022-01-31 |
| S357 | forum | Tracker PROJQUAY-9516: Quay should not allow to push images to the Organization set as Proxy Cache | https://redhat.atlassian.net/browse/PROJQUAY-9516 | opened 2025-10-08, updated 2026-11-19, status POST |
| S358 | forum | Tracker PROJQUAY-7962: Proxy cache responds with HTTP 500 on simultaneous pulls | https://redhat.atlassian.net/browse/PROJQUAY-7962 | opened 2024-09-17, status New |
| S359 | forum | Tracker PROJQUAY-12311: Proxy cache fails for large blobs due to asynchronous download | https://redhat.atlassian.net/browse/PROJQUAY-12311 | opened 2026-07-15, resolved 2026-10-19 |
| S360 | forum | Tracker PROJQUAY-8440: Pulling of cached image from Quay requires upstream registry availability | https://redhat.atlassian.net/browse/PROJQUAY-8440 | opened 2025-01-14, resolved 2026-07-14 |
| S361 | forum | Tracker PROJQUAY-10515: (Phase 1) Next-gen OpenShift Mirror Registry (OMR) | https://redhat.atlassian.net/browse/PROJQUAY-10515 | opened 2026-02-06, In Progress |
| S362 | forum | Tracker PROJQUAY-11433: perf(db): slow repository UPDATE queries during namespace deletion (14s+) | https://redhat.atlassian.net/browse/PROJQUAY-11433 | opened 2026-04-30, closed 2026-07-29 |
| S363 | forum | Tracker PROJQUAY-14084: Quay 3.18 upgrade fails in b1a79fa8e630 | https://redhat.atlassian.net/browse/PROJQUAY-14084 | opened 2026-10-28, status ASSIGNED |
| S364 | forum | Tracker PROJQUAY-7182: Upgrade from 3.8 to 3.9, postgresql data lost | https://redhat.atlassian.net/browse/PROJQUAY-7182 | opened 2024-05-13, closed 2024-07-15 |
| S365 | forum | Tracker PROJQUAY-6862: Quay on GCP fails to push layers > 4GiB on GCS with error 413 | https://redhat.atlassian.net/browse/PROJQUAY-6862 | opened 2024-03-11, closed 2025-06-19 |
| S366 | forum | Tracker PROJQUAY-8758: Quay SQLite database locked | https://redhat.atlassian.net/browse/PROJQUAY-8758 | opened 2025-03-31, closed 2026-08-12 |
| S367 | forum | Tracker PROJQUAY-4297: Enabling quota management in Quay significantly slows down opening of the super user panel | https://redhat.atlassian.net/browse/PROJQUAY-4297 | opened 2022-08-17, closed 2023-06-20 |
| S368 | forum | Tracker PROJQUAY-7249: Skopeo timeout for the mirror worker currently hardcoded to 300 seconds | https://redhat.atlassian.net/browse/PROJQUAY-7249 | opened 2024-05-22, closed 2025-06-19 |
| S369 | forum | Tracker PROJQUAY-4126: Clair database growing | https://redhat.atlassian.net/browse/PROJQUAY-4126 | opened 2022-07-08, closed 2022-12-16 |
| S370 | forum | Tracker PROJQUAY-3872: Tag list caching violates OCI spec | https://redhat.atlassian.net/browse/PROJQUAY-3872 | opened 2022-05-30, closed without resolution |
| S371 | forum | Tracker PROJQUAY-11365: source container upload fails with 413 Request Entity Too Large | https://redhat.atlassian.net/browse/PROJQUAY-11365 | opened 2026-04-21, closed 2026-04-21 |
| S372 | forum | quay/quay PR #5390: gc: Fix race conditions in proxy cache and GC code | https://github.com/quay/quay/pull/5390 | merged 2026-03-19 |
| S373 | docs | quay/enhancements: enhancements/proxy-cache.md (design proposal) | https://github.com/quay/enhancements/blob/4e971eed549443fa4faadc12ab15664a9427dd39/enhancements/proxy-cache.md | created 2021-12-10, history to 2022-03-28 |
| S374 | docs | quay/quay-performance-scripts: README.md | https://github.com/quay/quay-performance-scripts/blob/f736a04603c659bd61e3d4826a7008a2748fadc0/README.md | commit f736a04 (2026-06-24) |
| S375 | blog | Vendor developer blog: Sizing Red Hat Quay: Profiling components to find real limits (archived copy of developers.redhat.com/articles/2026/10/29/sizing-red-hat-quay-profiling-components-find-real-limits) | https://web.archive.org/web/20261120165807/https://developers.redhat.com/articles/2026/10/29/sizing-red-hat-quay-profiling-components-find-real-limits | 2026-10-29 |
| S376 | blog | Vendor developer blog: Optimizing Quay/Clair: Profiling, performance, and efficiency (archived copy) | https://web.archive.org/web/20260713145751/https://developers.redhat.com/articles/2024/03/19/optimizing-quayclair-profiling-performance-and-efficiency | 2024-03-19 |
| S377 | blog | Vendor blog: About the Quay.io Outage: Post Mortem | https://www.redhat.com/en/blog/about-the-quay.io-outage-post-mortem | 2020-08-07 |
| S378 | blog | Vendor blog: How we moved Quay.io's database from MySQL to Postgres and lived to tell about it | https://www.redhat.com/en/blog/moved-quay-io-database-from-mysql-to-aurora-postgres | 2024-02-29 |
| S379 | blog | Vendor blog: Quay.io Serves 1 Billion Container Images in August | https://www.redhat.com/en/blog/quay.io-serves-1-billion-container-images-in-august | 2020-10-02 |
| S380 | vendor | Vendor status page: quay.io intermittent API failures (postmortem) | https://status.redhat.com/incidents/ygxq09yvyx6w | incident 2026-02-18, postmortem 2026-02-26 |
| S381 | vendor | Vendor status page: Quay.io HTTP 502 on Pull/Push (postmortem) | https://status.redhat.com/incidents/3vlkclps2214 | incident 2026-05-07, postmortem 2026-05-15 |
| S382 | vendor | Vendor status page: quay.io API failures (postmortem) | https://status.redhat.com/incidents/04vm68m00t61 | incident 2026-03-30, postmortem 2026-04-20 |
| S383 | standard | OCI Distribution Specification, spec.md | https://github.com/opencontainers/distribution-spec/blob/v1.1.1/spec.md | v1.1.1 (2025-01-29) |
| S384 | docs | nginx: Module ngx_http_core_module | https://nginx.org/en/docs/http/ngx_http_core_module.html | fetched 2026-11-20 |
| S385 | docs | quay/quay GitHub release page v3.18.3 | https://github.com/quay/quay/releases/tag/v3.18.3 | published 2026-11-13 |
| S386 | docs | quay-docs 3.16: modules/deprecations-316.adoc | https://github.com/quay/quay-docs/blob/1076fa5a52a5742f4748c8ef360141fa7c33f7e9/modules/deprecations-316.adoc | redhat-3.16 @ 1076fa5 |
| S387 | docs | GitLab container registry: README.md | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/README.md | v4.43.0-gitlab (2026-10-23) |
| S388 | docs | GitLab container registry: Online Garbage Collection specification | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/docs/spec/gitlab/online-garbage-collection.md | v4.43.0-gitlab (2026-10-23) |
| S389 | docs | GitLab container registry: configuration reference | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/docs/configuration.md | v4.43.0-gitlab (2026-10-23) |
| S390 | code | GitLab container registry: configuration/configuration.go | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/configuration/configuration.go | v4.43.0-gitlab (2026-10-23) |
| S391 | docs | GitLab container registry: CHANGELOG.md | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/CHANGELOG.md | v4.43.0-gitlab (2026-10-23) |
| S392 | docs | GitLab container registry: Differences From Upstream | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/docs/upstream-differences.md | v4.43.0-gitlab (2026-10-23) |
| S393 | docs | GitLab container registry: HTTP API V1 specification (gitlab/v1) | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/docs/spec/gitlab/api.md | v4.43.0-gitlab (2026-10-23) |
| S394 | code | GitLab container registry: registry/api/v2/routes.go | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/api/v2/routes.go | v4.43.0-gitlab (2026-10-23) |
| S395 | code | GitLab container registry: .gitlab/ci/test.yml (OCI conformance job) | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/.gitlab/ci/test.yml | v4.43.0-gitlab (2026-10-23) |
| S396 | code | GitLab container registry: registry/datastore/digest.go | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/datastore/digest.go | v4.43.0-gitlab (2026-10-23) |
| S397 | docs | GitLab container registry: Supported Media Types | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/docs/supported-media-types.md | v4.43.0-gitlab (2026-10-23) |
| S398 | code | GitLab container registry: registry/datastore/migrations/structure.sql | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/datastore/migrations/structure.sql | v4.43.0-gitlab (2026-10-23) |
| S399 | code | GitLab container registry: registry/handlers/manifests.go | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/handlers/manifests.go | v4.43.0-gitlab (2026-10-23) |
| S400 | code | GitLab container registry: registry/handlers/app.go | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/handlers/app.go | v4.43.0-gitlab (2026-10-23) |
| S401 | code | GitLab container registry: registry/gc/agent.go | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/gc/agent.go | v4.43.0-gitlab (2026-10-23) |
| S402 | code | GitLab container registry: registry/gc/worker/worker.go | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/gc/worker/worker.go | v4.43.0-gitlab (2026-10-23) |
| S403 | code | GitLab container registry: registry/gc/worker/blobworker.go | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/gc/worker/blobworker.go | v4.43.0-gitlab (2026-10-23) |
| S404 | code | GitLab container registry: registry/handlers/catalog.go | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/handlers/catalog.go | v4.43.0-gitlab (2026-10-23) |
| S405 | code | GitLab container registry: registry/handlers/repositories.go | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/handlers/repositories.go | v4.43.0-gitlab (2026-10-23) |
| S406 | code | GitLab container registry: registry/datastore/datastore.go | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/datastore/datastore.go | v4.43.0-gitlab (2026-10-23) |
| S407 | code | GitLab container registry: registry/storage/driver/s3-aws/common/common.go | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/storage/driver/s3-aws/common/common.go | v4.43.0-gitlab (2026-10-23) |
| S408 | code | GitLab container registry: registry/storage/driver/s3-aws/v2/s3.go | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/storage/driver/s3-aws/v2/s3.go | v4.43.0-gitlab (2026-10-23) |
| S409 | code | GitLab container registry: notifications/endpoint.go | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/notifications/endpoint.go | v4.43.0-gitlab (2026-10-23) |
| S410 | code | GitLab container registry: registry/datastore/gcmanifesttask.go | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/datastore/gcmanifesttask.go | v4.43.0-gitlab (2026-10-23) |
| S411 | docs | GitLab container registry: Importing Existing Data to the Database | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/docs/database-import-tool.md | v4.43.0-gitlab (2026-10-23) |
| S412 | docs | GitLab container registry: Database Load Balancing specification | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/docs/spec/gitlab/database-load-balancing.md | v4.43.0-gitlab (2026-10-23) |
| S413 | docs | GitLab container registry: Storage Usage | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/docs/storage-usage.md | v4.43.0-gitlab (2026-10-23) |
| S414 | docs | GitLab container registry: Cross-Repository Blob Mount | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/docs/cross-repository-blob-mount.md | v4.43.0-gitlab (2026-10-23) |
| S415 | code | GitLab container registry: registry/auth/token/token.go | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/auth/token/token.go | v4.43.0-gitlab (2026-10-23) |
| S416 | code | GitLab container registry: registry/registry.go | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/registry.go | v4.43.0-gitlab (2026-10-23) |
| S417 | code | GitLab container registry: registry/storage/paths.go | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/storage/paths.go | v4.43.0-gitlab (2026-10-23) |
| S418 | code | GitLab container registry: .gitlab/ci/validate.yml (build matrix) | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/.gitlab/ci/validate.yml | v4.43.0-gitlab (2026-10-23) |
| S419 | docs | GitLab container registry: Rate limiting specification | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/docs/spec/gitlab/rate-limiting.md | v4.43.0-gitlab (2026-10-23) |
| S420 | code | GitLab container registry: internal/feature/feature.go | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/internal/feature/feature.go | v4.43.0-gitlab (2026-10-23) |
| S421 | code | GitLab container registry: registry/datastore/repository.go | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/datastore/repository.go | v4.43.0-gitlab (2026-10-23) |
| S422 | code | GitLab container registry: registry/root.go (CLI flags) | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/root.go | v4.43.0-gitlab (2026-10-23) |
| S423 | code | GitLab container registry: registry/storage/driver/filesystem/driver.go | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/storage/driver/filesystem/driver.go | v4.43.0-gitlab (2026-10-23) |
| S424 | code | GitLab container registry: registry/storage/driver/gcs/common.go | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/storage/driver/gcs/common.go | v4.43.0-gitlab (2026-10-23) |
| S425 | code | GitLab container registry: registry/datastore/loadbalancing.go | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/datastore/loadbalancing.go | v4.43.0-gitlab (2026-10-23) |
| S426 | docs | GitLab docs: Container registry metadata database (administration) | https://gitlab.com/gitlab-org/gitlab/-/blob/v19.6.0-ee/doc/administration/packages/container_registry_metadata_database.md | GitLab 19.6.0 (tag 2026-11-18) |
| S427 | docs | GitLab docs: GitLab container registry administration | https://gitlab.com/gitlab-org/gitlab/-/blob/v19.6.0-ee/doc/administration/packages/container_registry.md | GitLab 19.6.0 (tag 2026-11-18) |
| S428 | docs | GitLab docs: Three-step import | https://gitlab.com/gitlab-org/gitlab/-/blob/v19.6.0-ee/doc/administration/packages/container_registry_metadata_database_three_step_import.md | GitLab 19.6.0 (tag 2026-11-18) |
| S429 | docs | GitLab docs: Troubleshooting the container registry metadata database | https://gitlab.com/gitlab-org/gitlab/-/blob/v19.6.0-ee/doc/administration/packages/container_registry_metadata_database_troubleshooting.md | GitLab 19.6.0 (tag 2026-11-18) |
| S430 | docs | GitLab docs: Troubleshooting the container registry | https://gitlab.com/gitlab-org/gitlab/-/blob/v19.6.0-ee/doc/administration/packages/container_registry_troubleshooting.md | GitLab 19.6.0 (tag 2026-11-18) |
| S431 | docs | GitLab docs: Container registry for a secondary site (Geo) | https://gitlab.com/gitlab-org/gitlab/-/blob/v19.6.0-ee/doc/administration/geo/replication/container_registry.md | GitLab 19.6.0 (tag 2026-11-18) |
| S432 | docs | GitLab docs: Deprecations and removals by version | https://gitlab.com/gitlab-org/gitlab/-/blob/v19.6.0-ee/doc/update/deprecations.md | GitLab 19.6.0 (tag 2026-11-18) |
| S433 | docs | GitLab docs: GitLab container registry (user) | https://gitlab.com/gitlab-org/gitlab/-/blob/v19.6.0-ee/doc/user/packages/container_registry/_index.md | GitLab 19.6.0 (tag 2026-11-18) |
| S434 | docs | GitLab docs: Immutable container tags | https://gitlab.com/gitlab-org/gitlab/-/blob/v19.6.0-ee/doc/user/packages/container_registry/immutable_container_tags.md | GitLab 19.6.0 (tag 2026-11-18) |
| S435 | docs | GitLab docs: Dependency proxy for container images | https://gitlab.com/gitlab-org/gitlab/-/blob/v19.6.0-ee/doc/user/packages/dependency_proxy/_index.md | GitLab 19.6.0 (tag 2026-11-18) |
| S436 | docs | GitLab docs: Install GitLab with the Linux package (supported platforms) | https://gitlab.com/gitlab-org/gitlab/-/blob/v19.6.0-ee/doc/install/package/_index.md | GitLab 19.6.0 (tag 2026-11-18) |
| S437 | docs | GitLab docs: GitLab.com settings | https://gitlab.com/gitlab-org/gitlab/-/blob/v19.6.0-ee/doc/user/gitlab_com/_index.md | GitLab 19.6.0 (tag 2026-11-18) |
| S438 | docs | GitLab docs: Instance limits | https://gitlab.com/gitlab-org/gitlab/-/blob/v19.6.0-ee/doc/administration/instance_limits.md | GitLab 19.6.0 (tag 2026-11-18) |
| S439 | docs | GitLab handbook design document: Container registry metadata database (rendered: handbook.gitlab.com/handbook/engineering/architecture/design-documents/container_registry_metadata_database/) | https://gitlab.com/gitlab-com/content-sites/handbook/-/blob/4b92af5ec51035d2b65c0e22456dc08bf1f9b3bb/content/handbook/engineering/architecture/design-documents/container_registry_metadata_database/_index.md | created 2020-09-29, status implemented; handbook commit 4b92af5e (2026-11-20) |
| S440 | docs | GitLab handbook design document: Container registry self-managed database rollout | https://gitlab.com/gitlab-com/content-sites/handbook/-/blob/4b92af5ec51035d2b65c0e22456dc08bf1f9b3bb/content/handbook/engineering/architecture/design-documents/container_registry_metadata_database_self_managed_rollout/_index.md | created 2023-06-09, status proposed; handbook commit 4b92af5e (2026-11-20) |
| S441 | blog | GitLab blog: GitLab's next-generation container registry is now available | https://about.gitlab.com/blog/2023/12/04/gitlabs-next-generation-container-registry-is-now-available/ | 2023-12-04 |
| S442 | blog | GitLab blog: Next-generation GitLab container registry goes GA | https://about.gitlab.com/blog/2024/07/23/next-generation-gitlab-container-registry-goes-ga/ | 2024-07-23 |
| S443 | blog | GitLab blog: Introducing the next generation of the GitLab.com Container Registry | https://about.gitlab.com/blog/2022/04/12/next-generation-container-registry/ | 2022-04-12 |
| S444 | vendor | container-registry issue 199: Proposal for continuous, on-demand online garbage collection | https://gitlab.com/gitlab-org/container-registry/-/work_items/199 | opened 2020-08-21, closed 2021-03-18 |
| S445 | vendor | container-registry issue 94: Estimate database rate requirements | https://gitlab.com/gitlab-org/container-registry/-/work_items/94 | opened 2020-04-28, closed 2020-06-25 |
| S446 | vendor | container-registry issue 374: Gradual migration plan for GitLab.com Container Registry | https://gitlab.com/gitlab-org/container-registry/-/work_items/374 | opened 2021-05-11, closed 2022-10-28 |
| S447 | vendor | gitlab-org epic 5523: Deploy and migrate the new container registry for GitLab.com | https://gitlab.com/groups/gitlab-org/-/epics/5523 | opened 2021-03-01, closed 2022-12-15 |
| S448 | vendor | gitlab issue 423459: Feedback issue: Next generation container registry rollout to self-managed | https://gitlab.com/gitlab-org/gitlab/-/work_items/423459 | opened 2023-08-28, closed 2026-05-18, description updated 2026-08-17 |
| S449 | forum | container-registry issue 2381: Manifest PUT skips missing layer associations of an existing manifest; online GC then deletes the layers | https://gitlab.com/gitlab-org/container-registry/-/work_items/2381 | opened 2026-09-02, open on 2026-11-20 |
| S450 | vendor | container-registry issue 1205: Support for referrers API | https://gitlab.com/gitlab-org/container-registry/-/work_items/1205 | opened 2024-02-26, open on 2026-11-20 |
| S451 | vendor | container-registry issue 1206: Run both 1.0.x and 1.1.x OCI conformance tests | https://gitlab.com/gitlab-org/container-registry/-/work_items/1206 | opened 2024-02-26, open on 2026-11-20 |
| S452 | forum | container-registry issue 2137: Tag listing is slower than expected (user report) | https://gitlab.com/gitlab-org/container-registry/-/work_items/2137 | opened 2026-02-13, closed 2026-02-19 |
| S453 | vendor | gitlab issue 201960: Optimize the mark stage for GCS for Container Registry garbage collection | https://gitlab.com/gitlab-org/gitlab/-/work_items/201960 | opened 2020-02-04, closed 2020-03-18 |
| S454 | vendor | gitlab-org epic 10208: Use new container registry API for listing repository tags | https://gitlab.com/groups/gitlab-org/-/epics/10208 | opened 2023-03-30, closed 2024-05-27 |
| S455 | docs | GitLab runbooks: Builds failing with MANIFEST_INVALID request body too large | https://gitlab.com/gitlab-com/runbooks/-/blob/6ec746844dbd1375929ad305dcd118902c517211/docs/registry/manifest-invalid-request-body-too-large.md | runbooks commit 6ec74684 (2026-07-04) |
| S456 | docs | GitLab runbooks: gitlab-registry.md | https://gitlab.com/gitlab-com/runbooks/-/blob/6ec746844dbd1375929ad305dcd118902c517211/docs/registry/gitlab-registry.md | runbooks commit 6ec74684 (2026-07-04) |
| S457 | docs | OCI conformance results site (registry listings) | https://conformance.opencontainers.org/ | fetched 2026-11-20 |
| S458 | forum | GitLab forum: Upgrade 18.* to 19.0 results in metadata database empty for container registry | https://forum.gitlab.com/t/upgrade-18-to-19-0-results-in-metadata-database-empty-for-container-registry/134020 | 2026-05-28 |
| S459 | vendor | container-registry issue 732: Online GC: Lock contention on manifest deletes | https://gitlab.com/gitlab-org/container-registry/-/work_items/732 | opened 2022-07-18, closed 2022-08-30 |
| S460 | vendor | container-registry issue 1489: Manifest delete FK violation attempt during online GC | https://gitlab.com/gitlab-org/container-registry/-/work_items/1489 | opened 2025-01-22, closed 2025-04-28 |
| S461 | forum | container-registry issue 2375: Registry rejects attestation manifest whose subject isn't present yet | https://gitlab.com/gitlab-org/container-registry/-/work_items/2375 | opened 2026-08-26, open on 2026-11-20 |
| S462 | forum | container-registry issue 2391: garbage-collect refuses to run when the registry has fallen back to filesystem metadata in prefer mode | https://gitlab.com/gitlab-org/container-registry/-/work_items/2391 | opened 2026-09-17, open on 2026-11-20 |
| S463 | vendor | gitlab-org epic 5521: Release container registry with metadata database and online GC for self-managed installs | https://gitlab.com/groups/gitlab-org/-/epics/5521 | opened 2021-03-01, closed 2026-05-18 |
| S464 | docs | GitLab container registry: Database migrations guide | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/docs/database-migrations.md | v4.43.0-gitlab (2026-10-23) |
| S465 | docs | GitLab docs: Installation requirements (PostgreSQL versions) | https://gitlab.com/gitlab-org/gitlab/-/blob/v19.6.0-ee/doc/install/requirements.md | GitLab 19.6.0 (tag 2026-11-18) |
| S466 | vendor | container-registry issue 687: Failed Manifest Pushes Due to Layer Linking Errors May Succeed On Retry and Leave Layers Unlinked | https://gitlab.com/gitlab-org/container-registry/-/work_items/687 | opened 2022-05-19, closed 2025-08-21 |
| S467 | docs | GitLab docs: Delete container images from the container registry | https://gitlab.com/gitlab-org/gitlab/-/blob/v19.6.0-ee/doc/user/packages/container_registry/delete_container_registry_images.md | GitLab 19.6.0 (tag 2026-11-18) |
| S468 | docs | GitLab runbooks: High Number of Overdue Online GC Tasks | https://gitlab.com/gitlab-com/runbooks/-/blob/6ec746844dbd1375929ad305dcd118902c517211/docs/registry/online-gc-high-overdue-tasks.md | runbooks commit 6ec74684 (2026-07-04) |
| S469 | vendor | gitlab issue 207368: Listing tags for a container registry image is taking ~10 seconds | https://gitlab.com/gitlab-org/gitlab/-/work_items/207368 | opened 2020-02-20, closed 2023-08-17 |
| S470 | docs | olareg: README.md | https://github.com/olareg/olareg/blob/v0.2.2/README.md | v0.2.2 (2026-06-24) |
| S471 | code | olareg: config/config.go | https://github.com/olareg/olareg/blob/v0.2.2/config/config.go | v0.2.2 (2026-06-24) |
| S472 | code | olareg: olareg.go | https://github.com/olareg/olareg/blob/v0.2.2/olareg.go | v0.2.2 (2026-06-24) |
| S473 | code | olareg: cmd/olareg/serve.go | https://github.com/olareg/olareg/blob/v0.2.2/cmd/olareg/serve.go | v0.2.2 (2026-06-24) |
| S474 | code | olareg: internal/store/store.go | https://github.com/olareg/olareg/blob/v0.2.2/internal/store/store.go | v0.2.2 (2026-06-24) |
| S475 | code | olareg: internal/store/dir.go | https://github.com/olareg/olareg/blob/v0.2.2/internal/store/dir.go | v0.2.2 (2026-06-24) |
| S476 | code | olareg: blob.go | https://github.com/olareg/olareg/blob/v0.2.2/blob.go | v0.2.2 (2026-06-24) |
| S477 | code | olareg: manifest.go | https://github.com/olareg/olareg/blob/v0.2.2/manifest.go | v0.2.2 (2026-06-24) |
| S478 | code | olareg: referrer.go | https://github.com/olareg/olareg/blob/v0.2.2/referrer.go | v0.2.2 (2026-06-24) |
| S479 | code | olareg: tag.go | https://github.com/olareg/olareg/blob/v0.2.2/tag.go | v0.2.2 (2026-06-24) |
| S480 | code | olareg: config/auth.go | https://github.com/olareg/olareg/blob/v0.2.2/config/auth.go | v0.2.2 (2026-06-24) |
| S481 | code | olareg: internal/cache/cache.go | https://github.com/olareg/olareg/blob/v0.2.2/internal/cache/cache.go | v0.2.2 (2026-06-24) |
| S482 | docs | olareg: RELEASE.md (release notes) | https://github.com/olareg/olareg/blob/v0.2.2/RELEASE.md | v0.2.2 (2026-06-24) |
| S483 | code | olareg: Makefile | https://github.com/olareg/olareg/blob/v0.2.2/Makefile | v0.2.2 (2026-06-24) |
| S484 | docs | olareg.org: Auth | https://olareg.org/auth/ | page updated 2026-05-26; fetched 2026-11-20 |
| S485 | docs | olareg.org: Installing | https://olareg.org/install/ | page updated 2026-05-26; fetched 2026-11-20 |
| S486 | docs | OCI Conformance: OCI-Conformant Products | https://conformance.opencontainers.org/ | fetched 2026-11-20 |
| S487 | forum | olareg issue 156: help does not mention the shared store option | https://github.com/olareg/olareg/issues/156 | opened 2024-12-31, closed |
| S488 | docs | olareg: release v0.2.2 asset list | https://github.com/olareg/olareg/releases/expanded_assets/v0.2.2 | v0.2.2 (2026-06-24) |
| S489 | docs | trow: README.md | https://github.com/Trow-Registry/trow/blob/v0.10.0/README.md | v0.10.0 (2026-04-13) |
| S490 | docs | trow: docs/ARCHITECTURE.md | https://github.com/Trow-Registry/trow/blob/v0.10.0/docs/ARCHITECTURE.md | v0.10.0 (2026-04-13) |
| S491 | docs | trow: docs/USER_GUIDE.md | https://github.com/Trow-Registry/trow/blob/v0.10.0/docs/USER_GUIDE.md | v0.10.0 (2026-04-13) |
| S492 | docs | trow: CHANGELOG.md | https://github.com/Trow-Registry/trow/blob/v0.10.0/CHANGELOG.md | v0.10.0 (2026-04-13) |
| S493 | code | trow: src/registry/garbage_collect.rs | https://github.com/Trow-Registry/trow/blob/v0.10.0/src/registry/garbage_collect.rs | v0.10.0 (2026-04-13) |
| S494 | code | trow: src/main.rs | https://github.com/Trow-Registry/trow/blob/v0.10.0/src/main.rs | v0.10.0 (2026-04-13) |
| S495 | code | trow: src/init_db.rs | https://github.com/Trow-Registry/trow/blob/v0.10.0/src/init_db.rs | v0.10.0 (2026-04-13) |
| S496 | code | trow: src/routes/manifest.rs | https://github.com/Trow-Registry/trow/blob/v0.10.0/src/routes/manifest.rs | v0.10.0 (2026-04-13) |
| S497 | code | trow: src/registry/storage.rs | https://github.com/Trow-Registry/trow/blob/v0.10.0/src/registry/storage.rs | v0.10.0 (2026-04-13) |
| S498 | code | trow: src/registry/proxy/download.rs | https://github.com/Trow-Registry/trow/blob/v0.10.0/src/registry/proxy/download.rs | v0.10.0 (2026-04-13) |
| S499 | code | trow: src/routes/manifest_referrers.rs | https://github.com/Trow-Registry/trow/blob/v0.10.0/src/routes/manifest_referrers.rs | v0.10.0 (2026-04-13) |
| S500 | code | trow: .github/workflows/pr-tests.yaml | https://github.com/Trow-Registry/trow/blob/v0.10.0/.github/workflows/pr-tests.yaml | v0.10.0 (2026-04-13) |
| S501 | code | trow: src/routes/response/trow_token.rs | https://github.com/Trow-Registry/trow/blob/v0.10.0/src/routes/response/trow_token.rs | v0.10.0 (2026-04-13) |
| S502 | code | trow: migrations/03_manifest_blob_map.sql | https://github.com/Trow-Registry/trow/blob/v0.10.0/migrations/03_manifest_blob_map.sql | v0.10.0 (2026-04-13) |
| S503 | code | trow: charts/trow/values.yaml | https://github.com/Trow-Registry/trow/blob/v0.10.0/charts/trow/values.yaml | v0.10.0 (2026-04-13) |
| S504 | code | trow: src/routes/catalog.rs | https://github.com/Trow-Registry/trow/blob/v0.10.0/src/routes/catalog.rs | v0.10.0 (2026-04-13) |
| S505 | docs | trow: docs/COMPARISON.md | https://github.com/Trow-Registry/trow/blob/v0.10.0/docs/COMPARISON.md | v0.10.0 (2026-04-13) |
| S506 | code | trow: src/utils/digest.rs | https://github.com/Trow-Registry/trow/blob/v0.10.0/src/utils/digest.rs | v0.10.0 (2026-04-13) |
| S507 | code | trow: src/routes/response/errors.rs | https://github.com/Trow-Registry/trow/blob/v0.10.0/src/routes/response/errors.rs | v0.10.0 (2026-04-13) |
| S508 | docs | angos: README.md | https://github.com/project-angos/angos/blob/v1.13.0/README.md | v1.13.0 (2026-11-17) |
| S509 | docs | angos: doc/explanation/architecture.md | https://github.com/project-angos/angos/blob/v1.13.0/doc/explanation/architecture.md | v1.13.0 (2026-11-17) |
| S510 | docs | angos: doc/explanation/storage-backends.md | https://github.com/project-angos/angos/blob/v1.13.0/doc/explanation/storage-backends.md | v1.13.0 (2026-11-17) |
| S511 | docs | angos: doc/how-to/run-storage-maintenance.md | https://github.com/project-angos/angos/blob/v1.13.0/doc/how-to/run-storage-maintenance.md | v1.13.0 (2026-11-17) |
| S512 | docs | angos: doc/reference/configuration.md | https://github.com/project-angos/angos/blob/v1.13.0/doc/reference/configuration.md | v1.13.0 (2026-11-17) |
| S513 | docs | angos: doc/explanation/pull-through-caching.md | https://github.com/project-angos/angos/blob/v1.13.0/doc/explanation/pull-through-caching.md | v1.13.0 (2026-11-17) |
| S514 | docs | angos: CHANGELOG.md | https://github.com/project-angos/angos/blob/v1.13.0/CHANGELOG.md | v1.13.0 (2026-11-17) |
| S515 | code | angos: .github/workflows/build.yaml | https://github.com/project-angos/angos/blob/v1.13.0/.github/workflows/build.yaml | v1.13.0 (2026-11-17) |
| S516 | docs | angos: conformance/e2e/README.md | https://github.com/project-angos/angos/blob/v1.13.0/conformance/e2e/README.md | v1.13.0 (2026-11-17) |
| S517 | docs | angos: release v1.13.0 page | https://github.com/project-angos/angos/releases/tag/v1.13.0 | v1.13.0 (2026-11-17) |
| S518 | code | angos: src/registry/metadata_store/gc.rs | https://github.com/project-angos/angos/blob/v1.13.0/src/registry/metadata_store/gc.rs | v1.13.0 (2026-11-17) |
| S519 | docs | angos: doc/how-to/durable-cache-jobs.md | https://github.com/project-angos/angos/blob/v1.13.0/doc/how-to/durable-cache-jobs.md | v1.13.0 (2026-11-17) |
| S520 | docs | nora: README.md | https://github.com/getnora-io/nora/blob/v1.4.7/README.md | v1.4.7 (2026-11-18) |
| S521 | docs | nora: BENCHMARKS.md | https://github.com/getnora-io/nora/blob/v1.4.7/BENCHMARKS.md | v1.4.7 (2026-11-18) |
| S522 | docs | nora: ARCHITECTURE.md | https://github.com/getnora-io/nora/blob/v1.4.7/ARCHITECTURE.md | v1.4.7 (2026-11-18) |
| S523 | docs | nora: COMPAT.md | https://github.com/getnora-io/nora/blob/v1.4.7/COMPAT.md | v1.4.7 (2026-11-18) |
| S524 | code | nora: nora-registry/src/config/gc.rs | https://github.com/getnora-io/nora/blob/v1.4.7/nora-registry/src/config/gc.rs | v1.4.7 (2026-11-18) |
| S525 | code | nora: nora-registry/src/registry/docker.rs | https://github.com/getnora-io/nora/blob/v1.4.7/nora-registry/src/registry/docker.rs | v1.4.7 (2026-11-18) |
| S526 | code | nora: nora-registry/src/config/server.rs | https://github.com/getnora-io/nora/blob/v1.4.7/nora-registry/src/config/server.rs | v1.4.7 (2026-11-18) |
| S527 | code | nora: nora-registry/src/proxy_coalesce.rs | https://github.com/getnora-io/nora/blob/v1.4.7/nora-registry/src/proxy_coalesce.rs | v1.4.7 (2026-11-18) |
| S528 | code | nora: nora-registry/src/config/rate_limit.rs | https://github.com/getnora-io/nora/blob/v1.4.7/nora-registry/src/config/rate_limit.rs | v1.4.7 (2026-11-18) |
| S529 | forum | nora issue 584: GC can delete a live blob pushed after the orphan snapshot | https://github.com/getnora-io/nora/issues/584 | opened 2026-06-01, closed |
| S530 | forum | nora issue 130: Performance benchmarks, local vs S3 storage backend (maintainer's results comment) | https://github.com/getnora-io/nora/issues/130 | opened 2026-04-12; results comment 2026-04-18; closed |
| S531 | code | nora: nora-registry/src/config/auth.rs | https://github.com/getnora-io/nora/blob/v1.4.7/nora-registry/src/config/auth.rs | v1.4.7 (2026-11-18) |
| S532 | docs | nora: release v1.4.7 asset list | https://github.com/getnora-io/nora/releases/expanded_assets/v1.4.7 | v1.4.7 (2026-11-18) |
| S533 | code | nora: nora-registry/src/config/registry/docker.rs | https://github.com/getnora-io/nora/blob/v1.4.7/nora-registry/src/config/registry/docker.rs | v1.4.7 (2026-11-18) |
| S534 | docs | mcronce/oci-registry: README.md | https://github.com/mcronce/oci-registry/blob/v0.4.5/README.md | v0.4.5 (2024-04-23) |
| S535 | code | mcronce/oci-registry: src/upstream.rs | https://github.com/mcronce/oci-registry/blob/v0.4.5/src/upstream.rs | v0.4.5 (2024-04-23) |
| S536 | docs | container_registry-rs: README.md | https://github.com/mbr/container_registry-rs/blob/0.3.1/README.md | 0.3.1 (2024-08-14) |
| S537 | docs | rk8s: project/distribution/README.md | https://github.com/rk8s-dev/rk8s/blob/v0.1.3/project/distribution/README.md | v0.1.3 (2026-04-26) |
| S538 | docs | rk8s: README.md | https://github.com/rk8s-dev/rk8s/blob/v0.1.3/README.md | v0.1.3 (2026-04-26) |
| S539 | docs | rust-oci-client (crate oci-client): README.md | https://github.com/oras-project/rust-oci-client/blob/v0.18.0/README.md | v0.18.0 (2026-09-28) |
| S540 | code | rust-oci-client: src/lib.rs | https://github.com/oras-project/rust-oci-client/blob/v0.18.0/src/lib.rs | v0.18.0 (2026-09-28) |
| S541 | docs | go-containerregistry: pkg/registry/README.md | https://github.com/google/go-containerregistry/blob/v0.22.1/pkg/registry/README.md | v0.22.1 (2026-09-04) |
| S542 | code | go-containerregistry: pkg/registry/manifest.go | https://github.com/google/go-containerregistry/blob/v0.22.1/pkg/registry/manifest.go | v0.22.1 (2026-09-04) |
| S543 | code | go-containerregistry: pkg/registry/registry.go | https://github.com/google/go-containerregistry/blob/v0.22.1/pkg/registry/registry.go | v0.22.1 (2026-09-04) |
| S544 | code | go-containerregistry: pkg/registry/blobs.go | https://github.com/google/go-containerregistry/blob/v0.22.1/pkg/registry/blobs.go | v0.22.1 (2026-09-04) |
| S545 | code | go-containerregistry: cmd/crane/cmd/serve.go | https://github.com/google/go-containerregistry/blob/v0.22.1/cmd/crane/cmd/serve.go | v0.22.1 (2026-09-04) |
| S546 | docs | cloudflare/serverless-registry: README.md | https://github.com/cloudflare/serverless-registry/blob/a73605d6d5587286eb9cfcb910148f7d1423f218/README.md | commit a73605d (2026-08-17); no releases |
| S547 | docs | cloudflare/serverless-registry: docs/garbage-collection.md | https://github.com/cloudflare/serverless-registry/blob/a73605d6d5587286eb9cfcb910148f7d1423f218/docs/garbage-collection.md | commit a73605d (2026-08-17) |
| S548 | code | cloudflare/serverless-registry: src/chunk.ts | https://github.com/cloudflare/serverless-registry/blob/a73605d6d5587286eb9cfcb910148f7d1423f218/src/chunk.ts | commit a73605d (2026-08-17) |
| S549 | code | cloudflare/serverless-registry: src/registry/r2.ts | https://github.com/cloudflare/serverless-registry/blob/a73605d6d5587286eb9cfcb910148f7d1423f218/src/registry/r2.ts | commit a73605d (2026-08-17) |
| S550 | code | cloudflare/serverless-registry: src/registry/garbage-collector.ts | https://github.com/cloudflare/serverless-registry/blob/a73605d6d5587286eb9cfcb910148f7d1423f218/src/registry/garbage-collector.ts | commit a73605d (2026-08-17) |
| S551 | docs | angos: release v1.13.0 asset list | https://github.com/project-angos/angos/releases/expanded_assets/v1.13.0 | v1.13.0 (2026-11-17) |
| S552 | code | olareg: .github/workflows/ci-conformance.yml | https://github.com/olareg/olareg/blob/v0.2.2/.github/workflows/ci-conformance.yml | v0.2.2 (2026-06-24) |
| S553 | code | trow: Cargo.toml | https://github.com/Trow-Registry/trow/blob/v0.10.0/Cargo.toml | v0.10.0 (2026-04-13) |
| S554 | code | trow: src/lib.rs | https://github.com/Trow-Registry/trow/blob/v0.10.0/src/lib.rs | v0.10.0 (2026-04-13) |
| S555 | docs | peryx: README.md | https://github.com/tox-dev/peryx/blob/v2026.1118.1/README.md | v2026.1118.1 (2026-11-18) |
| S556 | docs | peryx docs: Configuration | https://github.com/tox-dev/peryx/blob/v2026.1118.1/site/content/core/operations/configuration.md | v2026.1118.1 (2026-11-18) |
| S557 | docs | peryx docs: OCI reference, Standards | https://github.com/tox-dev/peryx/blob/v2026.1118.1/site/content/ecosystems/oci/reference/standards.md | v2026.1118.1 (2026-11-18) |
| S558 | docs | peryx docs: OCI reference, Registry behavior | https://github.com/tox-dev/peryx/blob/v2026.1118.1/site/content/ecosystems/oci/reference/registry-behavior.md | v2026.1118.1 (2026-11-18) |
| S559 | vendor | peryx docs: OCI, Performance (the project's own comparison against Distribution and zot) | https://github.com/tox-dev/peryx/blob/v2026.1118.1/site/content/ecosystems/oci/performance.md | v2026.1118.1 (2026-11-18) |
| S560 | code | peryx: site/data/benchmark-machine.toml (host of the published benchmarks) | https://github.com/tox-dev/peryx/blob/v2026.1118.1/site/data/benchmark-machine.toml | v2026.1118.1 (2026-11-18) |
| S561 | docs | peryx docs: Availability (release status table) | https://github.com/tox-dev/peryx/blob/v2026.1118.1/site/content/core/availability/_index.md | v2026.1118.1 (2026-11-18) |
| S562 | docs | peryx docs: Availability, Blob reclamation | https://github.com/tox-dev/peryx/blob/v2026.1118.1/site/content/core/availability/blob-reclamation.md | v2026.1118.1 (2026-11-18) |
| S563 | docs | peryx docs: CLI | https://github.com/tox-dev/peryx/blob/v2026.1118.1/site/content/core/operations/cli.md | v2026.1118.1 (2026-11-18) |
| S564 | docs | peryx docs: OCI, Manifest serving | https://github.com/tox-dev/peryx/blob/v2026.1118.1/site/content/ecosystems/oci/manifest-serving.md | v2026.1118.1 (2026-11-18) |
| S565 | code | peryx: justfile (conformance recipe) | https://github.com/tox-dev/peryx/blob/v2026.1118.1/justfile | v2026.1118.1 (2026-11-18) |
| S566 | code | peryx: dist-workspace.toml (release targets) | https://github.com/tox-dev/peryx/blob/v2026.1118.1/dist-workspace.toml | v2026.1118.1 (2026-11-18) |
| S567 | code | peryx: Cargo.toml (workspace) | https://github.com/tox-dev/peryx/blob/v2026.1118.1/Cargo.toml | v2026.1118.1 (2026-11-18) |
| S568 | docs | peryx: CHANGELOG.md | https://github.com/tox-dev/peryx/blob/v2026.1118.1/CHANGELOG.md | v2026.1118.1 (2026-11-18) |
| S569 | docs | peryx docs: OCI reference, HTTP endpoints | https://github.com/tox-dev/peryx/blob/v2026.1118.1/site/content/ecosystems/oci/reference/endpoints.md | v2026.1118.1 (2026-11-18) |
| S570 | docs | peryx docs: Contributing, Benchmarking | https://github.com/tox-dev/peryx/blob/v2026.1118.1/site/content/contributing/benchmarking.md | v2026.1118.1 (2026-11-18) |
| S571 | paper | Anwar et al., Improving Docker Registry Design based on Production Workload Analysis, 16th USENIX Conference on File and Storage Technologies (FAST 18) | https://www.usenix.org/system/files/conference/fast18/fast18-anwar.pdf | 2018-02 |
| S572 | paper | Zhao et al., DupHunter: Flexible High-Performance Deduplication for Docker Registries, 2020 USENIX Annual Technical Conference (ATC 20) | https://www.usenix.org/system/files/atc20-zhao.pdf | 2020-07 |
| S573 | code | gitea: routers/api/packages/container/container.go | https://github.com/go-gitea/gitea/blob/v28.4.0/routers/api/packages/container/container.go | v28.4.0 (2026-11-15) |
| S574 | code | gitea: routers/api/packages/api.go | https://github.com/go-gitea/gitea/blob/v28.4.0/routers/api/packages/api.go | v28.4.0 (2026-11-15) |
| S575 | code | gitea: services/packages/cleanup/cleanup.go | https://github.com/go-gitea/gitea/blob/v28.4.0/services/packages/cleanup/cleanup.go | v28.4.0 (2026-11-15) |
| S576 | code | gitea: custom/conf/app.example.ini | https://github.com/go-gitea/gitea/blob/v28.4.0/custom/conf/app.example.ini | v28.4.0 (2026-11-15) |
| S577 | code | gitea: modules/packages/content_store.go | https://github.com/go-gitea/gitea/blob/v28.4.0/modules/packages/content_store.go | v28.4.0 (2026-11-15) |
| S578 | code | gitea: services/packages/container/blob_uploader.go | https://github.com/go-gitea/gitea/blob/v28.4.0/services/packages/container/blob_uploader.go | v28.4.0 (2026-11-15) |
| S579 | code | gitea: routers/api/packages/container/manifest.go | https://github.com/go-gitea/gitea/blob/v28.4.0/routers/api/packages/container/manifest.go | v28.4.0 (2026-11-15) |
| S580 | code | gitea: modules/packages/container/metadata.go | https://github.com/go-gitea/gitea/blob/v28.4.0/modules/packages/container/metadata.go | v28.4.0 (2026-11-15) |
| S581 | code | gitea: services/packages/auth.go | https://github.com/go-gitea/gitea/blob/v28.4.0/services/packages/auth.go | v28.4.0 (2026-11-15) |
| S582 | code | gitea: modules/packages/multi_hasher.go | https://github.com/go-gitea/gitea/blob/v28.4.0/modules/packages/multi_hasher.go | v28.4.0 (2026-11-15) |
| S583 | code | gitea: routers/api/packages/container/blob.go | https://github.com/go-gitea/gitea/blob/v28.4.0/routers/api/packages/container/blob.go | v28.4.0 (2026-11-15) |
| S584 | code | gitea: modules/packages/hashed_buffer.go | https://github.com/go-gitea/gitea/blob/v28.4.0/modules/packages/hashed_buffer.go | v28.4.0 (2026-11-15) |
| S585 | docs | Gitea docs: Container Registry | https://docs.gitea.com/usage/packages/container | fetched 2026-11-20 |
| S586 | docs | Gitea docs: Package Registry storage | https://docs.gitea.com/usage/packages/storage | fetched 2026-11-20 |
| S587 | docs | Gitea docs: Package Registry overview | https://docs.gitea.com/usage/packages/overview | fetched 2026-11-20 |
| S588 | forum | go-gitea/gitea issue 40359: container registry lower-cases tags | https://github.com/go-gitea/gitea/issues/40359 | opened 2026-11-13, open |
| S589 | forum | go-gitea/gitea issue 32053: images from manifests never cleaned up | https://github.com/go-gitea/gitea/issues/32053 | opened 2024-09-16, open |
| S590 | forum | go-gitea/gitea issue 24174: container registry cannot be set to private | https://github.com/go-gitea/gitea/issues/24174 | opened 2023-04-17, open |
| S591 | docs | go-gitea/gitea release v28.4.0 asset list | https://github.com/go-gitea/gitea/releases/expanded_assets/v28.4.0 | v28.4.0 (2026-11-15) |
| S592 | code | forgejo: services/packages/container/cleanup_sha256.go | https://codeberg.org/forgejo/forgejo/src/tag/v17.0.2/services/packages/container/cleanup_sha256.go | v17.0.2 (2026-11-18) |
| S593 | code | forgejo: routers/api/packages/api.go | https://codeberg.org/forgejo/forgejo/src/tag/v17.0.2/routers/api/packages/api.go | v17.0.2 (2026-11-18) |
| S594 | forum | forgejo issue 15324: compliance of OCI container registry with specification | https://codeberg.org/forgejo/forgejo/issues/15324 | opened 2026-11-18, open |
| S595 | forum | forgejo issue 15258: upload session GET reports Range one byte too far | https://codeberg.org/forgejo/forgejo/issues/15258 | opened 2026-11-13, open |
| S596 | docs | Forgejo FAQ | https://forgejo.org/faq/ | fetched 2026-11-20 |
| S597 | docs | Forgejo release v17.0.2 (Codeberg API record of the release) | https://codeberg.org/api/v1/repos/forgejo/forgejo/releases/tags/v17.0.2 | v17.0.2 (2026-11-18) |
| S598 | docs | pulp_container: docs/index.md | https://github.com/pulp/pulp_container/blob/2.30.1/docs/index.md | 2.30.1 (2026-10-28) |
| S599 | docs | pulpcore: docs/admin/learn/architecture.md | https://github.com/pulp/pulpcore/blob/3.124.0/docs/admin/learn/architecture.md | 3.124.0 (2026-11-19) |
| S600 | code | pulp_container: pulp_container/app/urls.py | https://github.com/pulp/pulp_container/blob/2.30.1/pulp_container/app/urls.py | 2.30.1 (2026-10-28) |
| S601 | code | pulp_container: pulp_container/app/registry_api.py | https://github.com/pulp/pulp_container/blob/2.30.1/pulp_container/app/registry_api.py | 2.30.1 (2026-10-28) |
| S602 | docs | pulpcore: docs/admin/reference/settings.md | https://github.com/pulp/pulpcore/blob/3.124.0/docs/admin/reference/settings.md | 3.124.0 (2026-11-19) |
| S603 | code | pulpcore: pulpcore/app/settings.py | https://github.com/pulp/pulpcore/blob/3.124.0/pulpcore/app/settings.py | 3.124.0 (2026-11-19) |
| S604 | code | pulp_container: pulp_container/app/models.py | https://github.com/pulp/pulp_container/blob/2.30.1/pulp_container/app/models.py | 2.30.1 (2026-10-28) |
| S605 | docs | pulp_container: docs/admin/guides/pull-through-caching.md | https://github.com/pulp/pulp_container/blob/2.30.1/docs/admin/guides/pull-through-caching.md | 2.30.1 (2026-10-28) |
| S606 | docs | pulp_container: CHANGES.md | https://github.com/pulp/pulp_container/blob/2.30.1/CHANGES.md | 2.30.1 (2026-10-28) |
| S607 | docs | pulp_container: docs/user/guides/push-image.md | https://github.com/pulp/pulp_container/blob/2.30.1/docs/user/guides/push-image.md | 2.30.1 (2026-10-28) |
| S608 | forum | pulp/pulp_container issue 1396: concurrent pushes throttled | https://github.com/pulp/pulp_container/issues/1396 | opened 2023-09-14, closed |
| S609 | forum | pulp/pulp_container issue 1730: 404 Not Found on layer push | https://github.com/pulp/pulp_container/issues/1730 | opened 2024-08-07, open |
| S610 | code | pulp_container: pulp_container/app/authorization.py | https://github.com/pulp/pulp_container/blob/2.30.1/pulp_container/app/authorization.py | 2.30.1 (2026-10-28) |
| S611 | docs | pulp_container: docs/admin/learn/authentication.md | https://github.com/pulp/pulp_container/blob/2.30.1/docs/admin/learn/authentication.md | 2.30.1 (2026-10-28) |
| S612 | docs | pulp_container: docs/user/guides/manage-cosign-signature.md | https://github.com/pulp/pulp_container/blob/2.30.1/docs/user/guides/manage-cosign-signature.md | 2.30.1 (2026-10-28) |
| S613 | code | pulp_container: pulp_container/constants.py | https://github.com/pulp/pulp_container/blob/2.30.1/pulp_container/constants.py | 2.30.1 (2026-10-28) |
| S614 | forum | pulp/pulp_container issue 1573: timing of the handle-image data migration | https://github.com/pulp/pulp_container/issues/1573 | opened 2024-03-22, closed |
| S615 | code | pulp_container: pyproject.toml | https://github.com/pulp/pulp_container/blob/2.30.1/pyproject.toml | 2.30.1 (2026-10-28) |
| S616 | code | sonatype/nexus-public: README.md | https://github.com/sonatype/nexus-public/blob/release-3.98.1-01/README.md | release-3.98.1-01 (2026-11-05) |
| S617 | vendor | Sonatype help: Community Edition onboarding | https://help.sonatype.com/en/ce-onboarding.html | last modified 2026-10-16 |
| S618 | vendor | Sonatype help: Nexus Repository system requirements | https://help.sonatype.com/en/sonatype-nexus-repository-system-requirements.html | last modified 2026-07-14 |
| S619 | vendor | Sonatype help: Docker Registry | https://help.sonatype.com/en/docker-registry.html | last modified 2026-08-26 |
| S620 | vendor | Sonatype help: Tasks | https://help.sonatype.com/en/tasks.html | last modified 2026-08-05 |
| S621 | vendor | Sonatype help: Cleanup Policies | https://help.sonatype.com/en/cleanup-policies.html | last modified 2026-10-25 |
| S622 | vendor | Sonatype help: Critical Cleanup Policy Bug Advisory | https://help.sonatype.com/en/critical-cleanup-policy-bug-advisory.html | last modified 2025-01-16 |
| S623 | vendor | Sonatype help: Cleanup Performance Data | https://help.sonatype.com/en/cleanup-performance-data.html | last modified 2025-08-07 |
| S624 | vendor | Sonatype help: Repository Types | https://help.sonatype.com/en/repository-types.html | last modified 2026-05-13 |
| S625 | vendor | Sonatype help: Pushing Images to a Group Repository | https://help.sonatype.com/en/pushing-images-to-a-group-repository.html | last modified 2026-10-25 |
| S626 | vendor | Sonatype help: Docker Authentication | https://help.sonatype.com/en/docker-authentication.html | last modified 2026-10-29 |
| S627 | vendor | Sonatype help: Docker Content Trust | https://help.sonatype.com/en/docker-content-trust.html | last modified 2024-12-19 |
| S628 | vendor | Sonatype help: Nexus Repository 3.98.0 - 3.98.3 release notes | https://help.sonatype.com/en/sonatype-nexus-repository-3-98-0-release-notes.html | last modified 2026-11-19 |
| S629 | docs | Dragonfly docs: Introduction | https://d7y.io/docs/ | docs v2.5.0, fetched 2026-11-20 |
| S630 | blog | Dragonfly blog: Lightweight Dragonfly deployment on Kubernetes | https://d7y.io/blog/2026/07/29/dragonfly-lightweight-deployment | 2026-07-29 |
| S631 | docs | Dragonfly docs: containerd integration | https://d7y.io/docs/operations/integrations/container-runtime/containerd/ | docs v2.5.0, fetched 2026-11-20 |
| S632 | code | dragonflyoss/client: dragonfly-client-config/src/dfdaemon.rs | https://github.com/dragonflyoss/client/blob/v1.5.14/dragonfly-client-config/src/dfdaemon.rs | v1.5.14 (2026-11-13) |
| S633 | blog | Dragonfly blog: Using dragonfly to distribute images and files for multi-cluster kubernetes | https://d7y.io/blog/2023/09/01/using-dragonfly-to-distribute-images-and-files-for-multi-cluster-kubernetes | 2023-09-01 |
| S634 | blog | Dragonfly blog: CNCF announces Dragonfly graduation | https://d7y.io/blog/2026/01/15/cncf-announces-dragonfly-graduation | 2026-01-15 |
| S635 | blog | Dragonfly blog: project paper accepted by IEEE Transactions on Networking | https://d7y.io/blog/2025/09/05/dragonfly-project-paper-accepted-by-IEEE-Transactions-on-Networking | 2025-09-05 |
| S636 | docs | dragonflyoss/client release v1.5.14 asset list | https://github.com/dragonflyoss/client/releases/expanded_assets/v1.5.14 | v1.5.14 (2026-11-13) |
| S637 | docs | Dragonfly docs: Harbor integration | https://d7y.io/docs/operations/integrations/harbor/ | docs v2.5.0, fetched 2026-11-20 |
| S638 | blog | Dragonfly blog: Dragonfly v2.5.0 has been released | https://d7y.io/blog/2026/06/25/dragonfly-v2-5-0-has-been-released | 2026-06-25 |
| S639 | docs | uber/kraken: README.md | https://github.com/uber/kraken/blob/v0.1.37/README.md | v0.1.37 (2026-11-16) |
| S640 | docs | uber/kraken: docs/CONFIGURATION.md | https://github.com/uber/kraken/blob/v0.1.37/docs/CONFIGURATION.md | v0.1.37 (2026-11-16) |
| S641 | code | uber/kraken: go.mod | https://github.com/uber/kraken/blob/v0.1.37/go.mod | v0.1.37 (2026-11-16) |
| S642 | code | uber/kraken: lib/dockerregistry/storage_driver.go | https://github.com/uber/kraken/blob/v0.1.37/lib/dockerregistry/storage_driver.go | v0.1.37 (2026-11-16) |
| S643 | docs | uber/kraken release v0.1.37 | https://github.com/uber/kraken/releases/tag/v0.1.37 | v0.1.37 (2026-11-16) |
| S644 | code | uber/kraken: Makefile | https://github.com/uber/kraken/blob/v0.1.37/Makefile | v0.1.37 (2026-11-16) |
| S645 | docs | openshift-docs: modules/pruning-hard-pruning-registry.adoc | https://github.com/openshift/openshift-docs/blob/0d60203614d4f683b27ce00852668dce37160993/modules/pruning-hard-pruning-registry.adoc | branch enterprise-4.21 at commit 0d60203 (2026-11-20) |
| S646 | docs | openshift-docs: modules/pruning-images.adoc | https://github.com/openshift/openshift-docs/blob/0d60203614d4f683b27ce00852668dce37160993/modules/pruning-images.adoc | branch enterprise-4.21 at commit 0d60203 (2026-11-20) |
| S647 | code | forgejo: services/packages/container/manifest.go | https://codeberg.org/forgejo/forgejo/src/tag/v17.0.2/services/packages/container/manifest.go | v17.0.2 (2026-11-18) |
| S648 | code | gitea: services/cron/tasks_basic.go | https://github.com/go-gitea/gitea/blob/v28.4.0/services/cron/tasks_basic.go | v28.4.0 (2026-11-15) |
| S649 | blog | Dragonfly blog: Dragonfly v2.2.0 has been released | https://d7y.io/blog/2025/01/07/dragonfly-v2-2-0-has-been-released | 2025-01-07 |
| S650 | docs | go-gitea/gitea release notes v28.0.0 | https://github.com/go-gitea/gitea/releases/tag/v28.0.0 | v28.0.0 (2026-09-30) |
| S651 | docs | go-gitea/gitea release notes v28.4.0 | https://github.com/go-gitea/gitea/releases/tag/v28.4.0 | v28.4.0 (2026-11-15) |
| S652 | docs | forgejo release notes 17.0.2 | https://codeberg.org/forgejo/forgejo/src/branch/forgejo/release-notes-published/17.0.2.md | v17.0.2 (2026-11-18) |
| S653 | code | dragonflyoss/dragonfly: go.mod | https://github.com/dragonflyoss/dragonfly/blob/v2.5.4/go.mod | v2.5.4 (2026-11-13) |
| S654 | docs | pulp_container: docs/admin/learn/rbac.md | https://github.com/pulp/pulp_container/blob/2.30.1/docs/admin/learn/rbac.md | 2.30.1 (2026-10-28) |
| S655 | docs | dragonflyoss/client: README.md | https://github.com/dragonflyoss/client/blob/v1.5.14/README.md | v1.5.14 (2026-11-13) |
| S656 | code | gitea: models/packages/package_blob.go | https://github.com/go-gitea/gitea/blob/v28.4.0/models/packages/package_blob.go | v28.4.0 (2026-11-15) |

## Findings

**CNCF Distribution v3.1.2, with the OCI distribution-spec as yardstick (F001 to F048)**

### F001. Distribution (Q1): it is a Go registry shipped as one `registry` binary, described by its docs as stateless, and it is the core library of Docker Hub, GHCR, GitLab, DigitalOcean and Harbor.
- Quote: "It is a core library for many registry operators including Docker Hub, GitHub Container Registry, GitLab Container Registry and DigitalOcean Container Registry, as well as the CNCF Harbor Project, and VMware Harbor Registry." — S001, README intro L20-L22
- Quote: "The Registry is a stateless, highly scalable server side application that stores and lets you distribute container images and other content." — S002, What it is L9-L10
- Conditions/notes: v3.1.2. The binary has two subcommands, `serve` and `garbage-collect` (S016). "Stateless" means all state is in the storage driver (F003).
- Relevance: it is the baseline every other registry is compared with or built on; a new registry must beat it on metadata operations, GC and conformance, not on blob serving alone.

### F002. Distribution (Q1): exactly one storage backend per registry out of filesystem, azure, gcs and s3 (inmemory for tests); oss and swift were dropped in v3.0.0 and no new drivers are accepted.
- Quote: "You must configure exactly one backend. If you configure more, the registry returns an error." — S003, storage L484-L485
- Quote: "New storage drivers are not currently being accepted." — S005, Driver contribution L41
- Quote: "oss and swift storage drivers are no longer supported" — S062, Deprecations
- Conditions/notes: v3.1.2 still links AWS SDK for Go v1 (`github.com/aws/aws-sdk-go v1.55.5`, go.mod L12, S053); issue #4564 "Upgrade AWS SDK to /v2" was open with 15 comments on 2026-11-20 (GitHub search API).
- Relevance: the driver interface is a filesystem-like key/value model; a registry with its own object store avoids the lowest-common-denominator API that causes the costs in F040..F044.

### F003. Distribution (Q1, Q8): all metadata (tags, manifest revisions, layer membership, upload state) is link files written through the storage driver; there is no database, and a load-balanced cluster must share the storage driver, the HTTP secret and the Redis cache.
- Quote: "The content-addressable blob store holds most data throughout the backend, keyed by algorithm and digests of the underlying content. Access to the blob store is controlled through links from the repository to blobstore." — S006, pathFor comment L55-L58
- Quote: "The most important aspect is that a load balanced cluster of registries must share the same resources. For the current version of the registry, this means the following must be the same:" — S007, Load balancing considerations L350-L352
- Conditions/notes: the list that follows the second quote is "Storage Driver", "HTTP Secret", "Redis Cache (if configured)". The only optional external service is Redis, used as blob-descriptor cache (F044).
- Relevance: no metadata database is why tag listing, tag lookup, catalog and GC cost O(n) storage listings; a record store for metadata removes that whole class.

### F004. Distribution (Q1, Q3): the design accepts last-writer-wins on link files and its cleanup code states it is only reliable on strongly consistent storage.
- Quote: "All other non-content files (links, hashes, etc.) are written as an atomic unit. Anything that requires additions and deletions is broken out into separate "files". Last writer still wins." — S008, Eventual Consistency L45-L47
- Quote: "vacuum contains functions for cleaning up repositories and blobs These functions will only reliably work on strongly consistent storage systems." — S009, package comment L12-L14
- Conditions/notes: architecture.md carries `draft: true` front matter (not published on the docs site) but ships in the v3.1.2 tree.
- Relevance: tag updates, manifest deletes and GC have no transaction; a new registry should make tag move, referrer index update and delete atomic records.

### F005. Distribution (Q2): the README claims the OCI Distribution Specification without a version, and CI runs the spec's conformance action pinned at v1.0.1 with all four categories enabled, at v3.1.2 and still on main on 2026-11-19.
- Quote: "This repository's main product is the Open Source Registry implementation for storing and distributing container images and other content using the OCI Distribution Specification." — S001, README intro L15-L17
- Quote: "name: Run OCI Distribution Spec conformance tests uses: opencontainers/distribution-spec@10efb871ee08eb13678fe275e2cf89b3df86542b # v1.0.1" — S010, job run-conformance-test L32-L33
- Quote: "OCI_TEST_PULL: 1 OCI_TEST_PUSH: 1 OCI_TEST_CONTENT_DISCOVERY: 1 OCI_TEST_CONTENT_MANAGEMENT: 1" — S010, env L37-L40
- Quote: "name: Run OCI Distribution Spec conformance tests uses: opencontainers/distribution-spec@10efb871ee08eb13678fe275e2cf89b3df86542b # v1.0.1" — S058, job run-conformance-test L32-L33
- Quote: "docker run --rm -p 5000:5000 -e REGISTRY_STORAGE_DELETE_ENABLED=true -idt "registry:local"" — S010, step Start distribution server L30
- Counter: the project calls itself a reference implementation of the OCI standards — "A reference implementation of the OCI registry standards, and an easy way to experiment with new propsals in the registry space as these standards change." — S011, Roadmap item 2 L9-L10
- Conditions/notes: the v1.0.1 suite predates the referrers API, so CI does not exercise distribution-spec 1.1 features. Content Management passes only because CI enables deletion; out of the box `storage.delete.enabled` "defaults to false" (S003 L545-L546). Distribution is not listed on the OCI conformance results site (F009).
- Relevance: "passes OCI conformance" for Distribution means the 1.0 suite; a new registry should run the current suite (F015) in CI and publish results.

### F006. Distribution (Q2): v3.1.2 has no referrers API and does not parse `subject` or `artifactType`; the implementing PR #4828 has been open since 2026-03-26 and the design issue #3716 since 2022-08-18.
- Quote: "Implements the OCI Distribution Spec v1.1 referrers API (GET /v2/{name}/referrers/{digest}), completing the work started in #3834 and #4483." — S074, PR description, Summary
- Quote: "Parse subject and artifactType fields on OCI manifest and index PUT" — S074, PR description, What's included
- Quote: "Is there an update on when this could get merged in and when 3.2.0 will be made available?" — S074, comment thread
- Quote: "Based on the distribution-spec which now contains details on references between digests within a repository from the reference types working group, I thought I'd write down some thoughts on how we could implement that in this project." — S075, issue body
- Quote: "References returns the descriptors of this manifests references. func (m Manifest) References() []v1.Descriptor { references := make([]v1.Descriptor, 0, 1+len(m.Layers)) references = append(references, m.Config) references = append(references, m.Layers...)" — S012, type Manifest and References L69-L73
- Conditions/notes: `rg -i referrers` and `rg OCI-Subject` over the v3.1.2 tree outside `vendor/` return nothing, so no `OCI-Subject` header is sent and clients must use the referrers tag schema (F012). The manifest bytes are stored unchanged, so a `subject` field survives a round trip but is not indexed. GC marks only config and layers (the `References` quote), so with `--delete-untagged` an untagged referrer survives only through the client-maintained `sha256-<hex>` index tag (inference from S012 and S015, not tested).
- Relevance: the most-used open registry still lacks the 1.1 referrers API in 2026-11; native referrers with artifactType filtering is a differentiator, and the tag-schema fallback must still be served to clients migrating from it.

### F007. Distribution (Q2): blobs pushed with a non-sha256 digest are accepted by re-reading and re-hashing the whole upload and are stored under the sha256 path, but a manifest pushed by sha512 digest is rejected with DIGEST_INVALID (issue #4420 open; PR #4946 open).
- Quote: "The client wants to use a different digest algorithm. They'll just have to be patient and wait for us to download and re-hash the uploaded content using that digest algorithm." — S013, validateBlob L221-L223
- Quote: "Pushing a manifest with a sha512 digest returns an HTTP 400 / "DIGEST_INVALID" error." — S076, Description
- Quote: "Verified against the OCI conformance suite (779 -> 950 passing tests, all core sha512 categories now green)." — S077, PR description
- Conditions/notes: issue #4420 was reported against 3.0.0-beta.1; in v3.1.2 `PutManifest` still compares the request digest with `digest.FromBytes` of the body (sha256) and returns `ErrorCodeDigestInvalid` on mismatch (S027 L269-L273, S012 L45). The 779 and 950 counts are the PR author's run of the new conformance suite against main; the total number of tests is not stated.
- Relevance: sha512 end to end (blob, manifest, tag schema, `digest-algorithm` parameter) is a concrete conformance gap of the incumbent; hash while streaming instead of re-reading the upload.

### F008. OCI distribution-spec v1.1.1 (Q2): conformance has four workflow categories; Pull is mandatory, Push, Content Discovery and Content Management are SHOULD, and a claimed category must be implemented completely.
- Quote: "All registries conforming to this specification MUST support, at a minimum, all APIs in the **Pull** category." — S100, Conformance > Requirements L132
- Quote: "A registry claiming conformance with one of these specification categories MUST implement all APIs in the claimed category." — S100, Conformance > Requirements L135
- Quote: "Content Management - Lowest Priority - Includes tag, blob, and repo deletion. (Note: Many registries may have other ways to accomplish this than the OCI API.)" — S101, Testing registry workflows L65-L66
- Conditions/notes: v1.1.1 is the newest released spec (2025-01-29). The v1.1.1 conformance tests contain no sha512 case (`rg -i sha512 conformance/*.go` is empty). `/v2/_catalog` is not an endpoint of the OCI spec (`grep _catalog spec.md` is empty); it is a Docker Registry API legacy.
- Relevance: the target is all four categories; "Content Management" is deletion of tags, manifests and blobs.

### F009. OCI conformance site (Q2): it lists Django-OCI, Gitea, Keppel, Keygen, olareg, OpenRegistry and Zot as open source and Artifactory, Azure Container Registry, Cloudsmith, GitLab and MetaHub as hosted; CNCF Distribution, Harbor and Quay are not listed, and only olareg and Zot show results of the new per-workflow suite.
- Quote: "Each of the products found below conform to the OCI Distribution Specification, in varying degrees." — S104, Distribution Spec
- Quote: "Latest software version tested: v2.1.19 Latest spec version supported: v1.1" — S104, Open Source > Zot
- Quote: "✖️ API: Blob anonymous mount ✖️ API: Blob upload cancel ✖️ API: Manifest put with tag params ✖️ Data: Sparse Manifests ✖️ Data: Tag Param ✖️ Data: Tag Param sha512 ✅✅...✅46 of 52 workflows passed" — S104, Open Source > Zot, Version v1.1 Workflows
- Quote: "Latest software version tested: v0.2.2 Latest spec version supported: v1.1" — S104, Open Source > olareg
- Quote: "✖️ API: Blob anonymous mount ✖️ API: Manifest put with tag params ✖️ Data: Tag Param ✖️ Data: Tag Param sha512 ✅✅...✅48 of 52 workflows passed" — S104, Open Source > olareg, Version v1.1 Workflows
- Quote: "Vendor: GitLab Latest software version tested: 16.5 Latest spec version supported: v1.0" — S104, Hosted > GitLab Container Registry
- Quote: "Vendor: Gitea Latest software version tested: 1.22 Latest spec version supported: v1.0" — S104, Open Source > Gitea Container Registry
- Conditions/notes: page footer "Last generated: Aug 5 21:56:41 UTC", "Git commit: 16097618fca61dc5d7aed970d61514aa8efad2c9"; the commit page dates it 2026-08-05 (S105). Failing workflows shown for Zot v2.1.19 under v1.1: API: Blob anonymous mount; API: Blob upload cancel; API: Manifest put with tag params; Data: Sparse Manifests; Data: Tag Param; Data: Tag Param sha512. For olareg v0.2.2 under v1.1: API: Blob anonymous mount; API: Manifest put with tag params; Data: Tag Param; Data: Tag Param sha512 (and 47 of 52 under v1.0, where API: Referrers also fails). Other entries: Django-OCI v0.0.16, Keppel 1.0.0, OpenRegistry V0.1.0, MetaHub v0.4.8 (spec v1.0, four categories passed); Keygen v1.8.0 (v1.1, Pull only); Artifactory 7.93.0 and Azure Container Registry 240626 (v1.1, four categories); Cloudsmith 1.630.6 (v1.1, no Content Management). Results are self-submitted.
- Relevance: nobody shows 52 of 52 on the new suite; the failing rows (tag params, anonymous mount, upload cancel, sparse manifests) are the features to implement first to lead the table.

### F010. OCI distribution-spec v1.1.1 (Q2, Q9): registries SHOULD limit manifest size, SHOULD answer 413 above the limit and SHOULD support at least 4 megabytes; a manifest must be stored byte for byte.
- Quote: "A registry that enforces this limit SHOULD respond to a request to push a manifest over this limit with a response code `413 Payload Too Large`. Client and registry implementations SHOULD expect to be able to support manifest pushes of at least 4 megabytes." — S100, Pushing Manifests L494-L495
- Quote: "The registry MUST store the manifest in the exact byte representation provided by the client." — S100, Pushing Manifests L480
- Conditions/notes: the spec gives no reason for 4 megabytes. Other limits in the spec: tags at most 128 characters; an implementers note that many clients limit hostname plus `/` plus name to 255 characters (S100 L160-L166).
- Relevance: 4 MiB is the de facto interoperability floor for manifests, and 413 is the expected refusal.

### F011. Distribution (Q9): the manifest body limit is a hard-coded 4 MiB constant with no comment or config key, and an oversized body is reported as MANIFEST_INVALID rather than 413.
- Quote: "maxManifestBodySize = 4 * 1024 * 1024" — S027, const block L31
- Quote: "if err := copyFullPayload(imh, w, r, &jsonBuf, maxManifestBodySize, "image manifest PUT"); err != nil {" — S027, PutManifest L256
- Quote: "imh.Errors = append(imh.Errors, errcode.ErrorCodeManifestInvalid.WithDetail(err.Error()))" — S027, PutManifest L258
- Conditions/notes: by code reading, not by running it. Blob PATCH and PUT bodies pass `-1` as the limit, and "The copy will be limited to `limit` bytes, if limit is greater than zero." (S028 L31), so blob size is unbounded.
- Relevance: copy the 4 MiB floor but make it a stated, configurable budget with a typed 413.

### F012. OCI distribution-spec v1.1.1 (Q2): the referrers API returns an image index, never 404 once supported, SHOULD filter by artifactType with an `OCI-Filters-Applied` header, and the client-side tag-schema fallback is documented as racy.
- Quote: "If the registry supports the referrers API, the registry MUST NOT return a `404 Not Found` to a referrers API requests." — S100, Listing Referrers L577
- Quote: "If filtering is requested and applied, the response MUST include a header `OCI-Filters-Applied: artifactType` denoting that an `artifactType` filter was applied." — S100, Listing Referrers L632
- Quote: "*Note*: multiple clients could attempt to update the tag simultaneously resulting in race conditions and data loss." — S100, Referrers Tag Schema L736
- Quote: "A registry MUST initially accept an otherwise valid manifest with a `subject` field that references a manifest that does not exist in the repository, allowing clients to push a manifest and referrers to that manifest in either order." — S100, Push L223
- Conditions/notes: a registry that processes `subject` MUST answer with `OCI-Subject: <subject digest>` (L499); a `Link` header MUST be sent when the list does not fit one index (L626); when enabling the API a registry MUST include manifests already listed in a `<alg>-<ref>` tag-schema index (L749).
- Relevance: referrers need a server-side index keyed by subject digest that tolerates a missing subject, pagination, the artifactType filter, and import of existing tag-schema indexes.

### F013. OCI distribution-spec v1.1.1 (Q2, Q9): a registry with a minimum chunk size SHOULD advertise it with `OCI-Chunk-Min-Length`, and out-of-order chunks MUST get 416; Distribution v3.1.2 does not send the header.
- Quote: "If the registry has a minimum chunk size, the `POST` response SHOULD include the following header, where `<size>` is the size in bytes (see the blob `PATCH` definition for usage):" — S100, Pushing a blob in chunks L336
- Quote: "If a chunk is uploaded out of order, the registry MUST respond with a `416 Requested Range Not Satisfiable` code." — S100, Pushing a blob in chunks L383
- Conditions/notes: `rg -i 'chunk-min'` over the v3.1.2 tree outside `vendor/` is empty. The S3 driver buffers to its own part size (F038), so the 5 MiB S3 part minimum is hidden from clients.
- Relevance: if the object store has a minimum part size, either buffer server-side (memory budget) or advertise the minimum.

### F014. OCI distribution-spec v1.1.1 (Q2, Q6): tag listing must be in lexical order, `n` and `last` paginate, a `Link` header lets the server return fewer than `n`, and mounting may ignore `from`.
- Quote: "The response to such a request MAY return fewer than `<int>` results, but only when the total number of tags attached to the repository is less than `<int>` or a `Link` header is provided." — S100, Listing Tags L543
- Quote: "The registry MAY treat the `from` parameter as optional, and it MAY cross-mount the blob if it can be found." — S100, Mounting a blob from another repository L449
- Quote: "The Docker-Content-Digest header returns the canonical digest of the uploaded blob, and MUST be equal to the client provided digest." — S100, Pushing Manifests L488
- Conditions/notes: "When `n` is zero, this endpoint MUST return an empty list, and MUST NOT include a `Link` header." (L547). Content Management: "Registries MAY implement deletion or they MAY disable it." (L670).
- Relevance: a server-side page cap with a `Link` header is conformant; ordered tag listing should come from an ordered index, not a directory walk.

### F015. OCI distribution-spec main, unreleased (Q2, Q4, Q13): after v1.1.1 the spec gained a `digest-algorithm` upload parameter, tag query parameters with an `OCI-Tag` response header, an upload-cancel DELETE, a Registry Proxying section with an `ns` parameter, and `Docker-Content-Digest` became MUST.
- Quote: "When pushing a blob with a digest algorithm other than `sha256`, the post request SHOULD include the `digest-algorithm` parameter:" — S102, Pushing a blob in chunks L339
- Quote: "When pushing a manifest by digest, the registry MAY support the pushing of tags specified by addition of `tag` query parameters." — S102, Pushing Manifests L519
- Quote: "A proxy registry MAY use the `ns` query parameter to resolve an upstream registry host." — S102, Registry Proxying L771
- Quote: "If this request fails or is not called, the server SHOULD eventually timeout unfinished uploads." — S102, Cancel a blob upload L455
- Quote: "A successful response MUST contain the digest of the uploaded blob in the header `Docker-Content-Digest`." — S102, Pulling manifests L171
- Quote: "export OCI_VERSION="1.1" # distribution-spec version to test against, this adjusts default values for the API tests, also accepts "stable" and "dev"" — S103, Environment Variables L22
- Conditions/notes: main at commit 9727462 (2026-08-31); no release after v1.1.1 exists on 2026-11-20. A registry supporting tag parameters "SHOULD support pushing at least 10 tags per request" (L522). The rewritten conformance suite on main has per-feature switches such as `OCI_API_BLOBS_UPLOAD_CANCEL`, `OCI_API_MANIFESTS_TAG_PARAM`, `OCI_API_BLOBS_MOUNT_ANONYMOUS`, `OCI_DATA_SPARSE`, `OCI_DATA_SHA512`; these are the rows shown on the results site (F009).
- Relevance: design for the next spec now: multi-tag atomic manifest push, digest-algorithm at upload start, upload cancel with server-side expiry, and `ns`-scoped pull-through with an `OCI-Namespace` response header.

### F016. Distribution (Q3): garbage collection is an offline, stop-the-world mark and sweep run as a separate CLI command, and the registry must be read-only or stopped while it runs.
- Quote: "You should ensure that the registry is in read-only mode or not running at all. If you were to upload an image while garbage collection is running, there is the risk that the image's layers are mistakenly deleted leading to a corrupted image." — S014, More details about garbage collection L83-L85
- Quote: "This type of garbage collection is known as stop-the-world garbage collection." — S014, More details about garbage collection L87
- Quote: "Before running garbage collection, the registry should be restarted with readonly's `enabled` set to true." — S003, storage > readonly L538-L539
- Conditions/notes: command `bin/registry garbage-collect [--dry-run] [--delete-untagged] [--quiet] /path/to/config.yml` (S014 L93); flags defined in S016 L19-L21. Nothing in the code checks that the running registry is read-only: the race is prevented only by the operator. Entering and leaving read-only mode takes two restarts.
- Relevance: avoid. GC must run online, concurrently with pushes, without a read-only window.

### F017. Distribution (Q3): the collector walks every repository and reads every manifest into one in-memory mark set, then walks every blob; with `--delete-untagged` it runs a full tag lookup for each manifest, so the mark phase costs manifests times tags per repository.
- Quote: "Garbage collection runs in two phases. First, in the 'mark' phase, the process scans all the manifests in the registry. From these manifests, it constructs a set of content address digests." — S014, More details about garbage collection L75-L77
- Quote: "fetch all tags where this manifest is the latest one tags, err := repository.Tags(ctx).Lookup(ctx, v1.Descriptor{Digest: dgst})" — S015, MarkAndSweep L71-L72
- Quote: "Lookup recovers a list of tags which refer to this digest. When a manifest is deleted by digest, tag entries which point to it need to be recovered to avoid dangling tags." — S047, tagStore.Lookup L146-L147
- Quote: "I should also let you know that the GC as it is in this code is in a dire state - it was a cute OG attempt but it doesn't scale and has a boat load of bugs in it. Many orgs have their own in-house registry GC solutions for that very reason." — S086, maintainer comment of 2024-01-14
- Conditions/notes: `Lookup` lists all tags of the repository and reads each tag's `current/link` (S047 L148-L199, concurrency `GOMAXPROCS` by default, S056 L15). `markManifestReferences` calls itself for nested indexes (S015 L233). Mark and sweep loops are sequential; the mark set is a Go map of every live digest. No duration for a whole GC run at a stated scale was found in docs or issues; the closest published numbers are the blob enumeration timings in F040.
- Relevance: cost grows with total manifests, total blobs and, with untagged deletion, manifests times tags; an online collector should work from reference counts or review queues over indexed records.

### F018. Distribution (Q3, Q11): online garbage collection was asked for in April 2016 (issue #1609) and is still open; the sentence promising a background collector is no longer in the v3.1.2 docs.
- Quote: ""Stop the world" garbage collection will not work for us. The SLA we have with our customers precludes the amount of downtime required." — S078, issue body
- Quote: "In future registry versions the intention is that garbage collection will be an automated background action and this manual process will no longer apply." — S078, issue body quoting the 2016 docs
- Quote: "We attempted to use the GC feature as-is, but we have enough developers and overnight processes that there is no time of day when the downtime to restart in readonly mode is acceptable for us." — S078, comment of 2016-10-11
- Conditions/notes: `grep -i 'background action'` over docs/content at v3.1.2 is empty.
- Relevance: ten years without online GC upstream is why GitLab, Harbor, Quay and zot each built their own; it is the largest single gap to beat.

### F019. Distribution (Q3, Q11): `garbage-collect --delete-untagged` deleted platform manifests referenced only by a tagged index; reported 2020-06-08 (#3178), fixed 2024-04-21 by PR #4285 in v3 only, and a 2.8.3 user reports a registry going from 445G to 4G.
- Quote: "We found that the garbage-collect command with the -m parameter on the registry removes the manifests that enable us to use multi-arch, thus rendering us unable to run garbage collection and keep our registry somewhat clean and space-efficient." — S079, issue body
- Quote: "This issue is closed because #4285 merged into main. The last v2 release was in October 2023, and the fix for this is in the v3 branch (currently at 3.0.0-rc3)." — S079, maintainer comment
- Quote: "The second command removed almost all blobs in the registry. The manifests are still in place. The storage decreased from 445G used to 4G, I don't even know if something is still useable." — S080, Description
- Conditions/notes: #4461 is registry 2.8.3 with multi-arch images built by buildx. In v3.1.2 the fix is `unmarkReferencedManifest`, which filters deletion candidates that the mark phase reached through an index (S015 L144, L202-L215).
- Relevance: a data-loss GC bug lived four years in the reference registry; untagged-manifest collection needs model-based tests with indexes, nested indexes and referrers before it ships.

### F020. Distribution (Q3, Q11): blob removal and the blob-descriptor cache are not coordinated, so a registry can answer 200 with a Content-Length and no body after a blob is removed (issue #5010 on v3.1.2; older issue #2966).
- Quote: "Periodically, the garbage collection will cleanup up an image without communicating with the blobdescriptor cache, resulting in the image being "pulled" but returning 0 bytes to the requestor." — S081, Description
- Quote: "A temporary workaround is to disable the blobdescriptor cache by setting the field to an empty string." — S081, Additional Info
- Conditions/notes: #5010 is v3.1.2 used as pull-through cache with `blobdescriptor: inmemory`; what the reporter calls garbage collection is presumably the proxy's TTL expiry (F024), which the report does not spell out. PR #5081 (S095, opened 2026-11-12, merged to main, not in v3.1.2) describes the defect for the filesystem driver as status 200, a Content-Length header and no body, and names #5010 and #2966 (zero-byte layer after GC with the in-memory cache).
- Relevance: a cache that is not invalidated by the deleter serves a silent corruption; deletes must go through the same record path as reads.

### F021. Distribution (Q3, Q11): GC leaves metadata behind: v3.1.2 now deletes unmarked layer links, but reports of tens of thousands of stale `_layers` entries (#3851) and of tags still listed after GC on S3 (#4382) are open.
- Quote: "emit("%s: layer link eligible for deletion: %s", repo, dgst)" — S015, MarkAndSweep sweep L187
- Quote: "Given 35 revisions, and manifests with 10 layers each, I expect, at most 350 entries in _layers/sha256 for this repository. Instead, there are 35k entries in _layers/sha256." — S082, What I expect
- Quote: "I have a registry with S3 backend for storage but when I run GC, some directories will remain and List API will return that removed tag/manifest." — S083, Description
- Quote: "the result was that the catalog listing only accounted for only 103G out of 686G used on the physical filesystem." — S099, issue body
- Conditions/notes: #3851 was filed against `registry:2` on MinIO; #4382 against 2.8.3 on MinIO (the maintainer's reply was to use v3). #2212 (2.5.1, filesystem, 2017) is the overwritten-tag case that `--delete-untagged` was added for. The mark set is global, so a layer link in repository A survives while any repository's manifest references the blob (S015 L41, L129-L133).
- Relevance: orphaned storage is the most reported symptom; track membership per repository in records so that a repository's usage is exact.

### F022. Distribution (Q3, Q9): abandoned uploads are purged by a goroutine in every registry instance, enabled by default with age 168h and interval 24h, after a random start delay of 0 to 59 minutes, by walking the repositories tree (descending only into `_uploads`).
- Quote: "Upload purging is a background process that periodically removes orphaned files from the upload directories of the registry. Upload purging is enabled by default." — S003, storage > uploadpurging L517-L519
- Quote: "config["enabled"] = true config["age"] = "168h" config["interval"] = "24h" config["dryrun"] = false" — S017, uploadPurgeDefaultConfig L1071-L1074
- Quote: "jitter := time.Duration(randInt.Int64()%60) * time.Minute log.Infof("Starting upload purge in %s", jitter)" — S017, startUploadPurger L1143-L1144
- Quote: "getOutstandingUploads walks the upload directory, collecting files which could be eligible for deletion." — S018, getOutstandingUploads L57-L58
- Conditions/notes: no reason is given for 168h, 24h or the 60-minute jitter window. The walk starts at the repositories root (S018 L66-L71), so each instance lists every repository every interval. Issue #4358 (S084, 33 comments) is a crash loop right after "PurgeUploads starting" on S3; v3.1.1 notes "Bounds-check the file basename in PurgeUploads Walk callback" and CVE-2026-41888 (S064); v3.1.2 notes "Stop the upload purger during registry shutdown (#4956)" (S065).
- Relevance: keep upload sessions as records with an expiry so that purging is an indexed scan, not a tree walk per instance.

### F023. Distribution (Q4): a pull-through cache mirrors exactly one upstream, cannot accept pushes, and its URL must be a domain root.
- Quote: "It's currently possible to mirror only one upstream registry at a time." — S019, Gotcha L25
- Quote: "Pushing to a registry configured as a pull-through cache is unsupported." — S003, proxy L1282-L1283
- Quote: "The current distribution only supports proxy requests for one domain name. Can we redesign the proxy to support multiple domain names?" — S090, issue body
- Conditions/notes: issue #3725 is open since 2022-09-04 with 34 comments (GitHub search API, 2026-11-20). In proxy mode Put, Create, Mount and Delete on the blob store return `ErrUnsupported` (S020 L209-L232). Distribution has no replication or push-mirroring feature; webhook notifications (F037) are the only integration point for one.
- Relevance: multi-upstream pull-through inside one registry that also accepts pushes is the commonly requested shape; the spec's `ns` parameter (F015) is the standard way to select the upstream.

### F024. Distribution (Q4, Q9): cached content expires after a TTL that defaults to 168h (0 disables expiry); expiry state is one JSON file written through the storage driver every 5 seconds when dirty, with one in-process timer per cached object, and cleanup only happens when deletes are enabled.
- Quote: "TTL is the expiry time of the content and will be cleaned up when it expires if not set, defaults to 7 * 24 hours If set to zero, will never expire cache" — S004, type Proxy L711-L713
- Quote: "if ttl != nil { s = scheduler.New(ctx, driver, "/scheduler-state.json")" — S021, NewRegistryPullThroughCache L66-L67
- Quote: "entryTypeBlob = iota entryTypeManifest indexSaveFrequency = 5 * time.Second" — S023, const block L19-L21
- Quote: "For the scheduler to clean up old entries, `delete` must be enabled in the registry configuration." — S019, Configure the cache L110-L111
- Conditions/notes: no reason is given for 7 days or 5 seconds; per the discussion in #4191 (S086) the 7 days were hard-coded in 2.8.3 and became configurable in v3. The TTL is fixed at first fetch and is not refreshed by later pulls (S020 L160-L165 adds the entry only on the fetch path). A truncated state file made the proxy panic at start and lose track of everything it had cached (issue #4478: "the proxy cannot recover automatically from it leaking (losing track of) all the cached images in its storage", S087); v3.1.0 changed non-append writes to use a temporary file (#4624, S063).
- Relevance: avoid a single rewritten state file and per-object timers; keep expiry as indexed records and prefer last-access eviction under a byte budget.

### F025. Distribution (Q4, Q11): expiring a cached blob deletes it globally although other cached images still reference it, which breaks pulls until those images expire too; reported 2017-08-15 (#2367) and still open.
- Quote: "This happens consistently when trying to pull an image that shares a layer with another image that has been expired. Because the TTL is 7 days we have images which we are unable to pull for multiple days until that image hits the TTL." — S085, issue body
- Quote: "Clear the repository reference and descriptor caches err = blobs.Delete(ctx, r.Digest()) if err != nil { return err } err = v.RemoveBlob(r.Digest().String())" — S021, OnBlobExpire L82-L88
- Quote: "To ensure best performance and guarantee correctness the Registry cache should be configured to use the `filesystem` driver for storage." — S019, What about my disk? L68-L69
- Conditions/notes: #2367 has 46 comments (GitHub search API, 2026-11-20). #4191 (S086, kube-apiserver image pulled through a proxy, containerd reported "unexpected EOF") was closed on 2024-01-16 as a duplicate of #2367 after its reporter mitigated it by disabling the blobdescriptor cache.
- Relevance: cache eviction must respect sharing between images; evict by reference state, never by per-object timers over a deduplicated store.

### F026. Distribution (Q4): freshness is checked upstream on every pull by tag; when the upstream cannot be reached the locally cached tag is served, but the proxy cannot start without the upstream.
- Quote: "When a pull is attempted with a tag, the Registry checks the remote to ensure if it has the latest version of the requested content. Otherwise, it fetches and caches the latest content." — S019, What if the content changes on the Hub? L57-L59
- Quote: "Get attempts to get the most recent digest for the tag by checking the remote tag service first and then caching it locally. If the remote is unavailable the local association is returned" — S022, proxyTagService.Get L20-L22
- Quote: "The server will not start and host cached images when internet is not available on boot." — S088, issue body
- Conditions/notes: in v3.1.2 `NewApp` still panics when `NewRegistryPullThroughCache` returns an error (S017 L361-L364), and its setup does an HTTP GET of `<remoteurl>/v2/` to discover token realms. Every tag pull therefore costs an upstream request and counts against upstream rate limits ("Mirrors of Docker Hub are still subject to Docker's fair usage policy", S019 L41). Issue #3270 (S089, open): an upstream 429 was returned to clients as 500; v3.1.2 notes "preserve upstream error codes (#4917)" (S065).
- Relevance: a freshness TTL for tags, stale-while-unavailable serving and start-up without the upstream are all missing here.

### F027. Distribution (Q4): there is no cache-stampede protection; concurrent pulls of an uncached blob each fetch it from the upstream and only the first one is written to the cache.
- Quote: "Note that concurrent inflight pulls will make multiple requests." — S019, Run a Registry as a pull-through cache L82
- Quote: "If the blob has been serving in other requests. Will return the blob from the remote store directly. TODO Maybe we could reuse the these blobs are serving remotely and caching locally." — S020, ServeBlob L101-L103
- Quote: "Multiple registry caches can be deployed over the same back-end, but each instance in a registry cache cluster is stateless, so it maintains its own cache." — S019, Run a Registry as a pull-through cache L78-L80
- Conditions/notes: the in-flight set is a process-local map keyed by digest (S020 L32-L36). Cache writes use a detached context with `cachewritetimeout`, default 5 minutes, which is in the v3.1.2 config struct (S004 L716-L720) but not in configuration.md and not in the v3.0.0 struct (S060).
- Relevance: coalesce concurrent misses for one digest into one upstream fetch that fans out to all waiters, across nodes.

### F028. Distribution (Q4, Q5): upstream credentials are one static username and password or an exec credential helper, cached private content is served to every client of the mirror, and two 2026 advisories hit this path (credential exfiltration through the bearer realm; unverified manifest content poisoning the cache).
- Quote: "If you specify a username and password, it's very important to understand that private resources that this user has access to Docker Hub is made available on your mirror." — S019, Configure the cache L105-L107
- Quote: "as a result, an attacker-controlled upstream (or an attacker with MitM position to the upstream) can cause distribution to send the configured upstream credentials via basic auth to an attacker-controlled realm URL." — S071, summary
- Quote: "so a poisoned remote can permanently inject a bad manifest into the mirror cache, affecting all subsequent consumers." — S073, Details
- Conditions/notes: GHSA-3p65-76g6-3w7r affected <= 3.0.0, patched in 3.1.0; GHSA-685w-q87j-wqw3 patched in 3.1.2 (versions as listed on the advisory pages). The exec helper re-runs a command per configured `lifetime` (S004 L728-L738).
- Relevance: a pull-through cache must verify every fetched object against its digest before caching, pin token realms to the upstream, and apply per-repository authorization to cached private content.

### F029. Distribution (Q5): authentication is one of `silly`, `token` or `htpasswd` (or none); authorization exists only as the access claims of a JWT issued by an external token server, so there is no built-in RBAC, user store, robot account or per-repository policy.
- Quote: "You can configure only one authentication provider." — S003, auth L638
- Quote: "An authorization server capable of managing access controls to their resources hosted by any given service (such as repositories in a Distribution Registry)." — S045, Requirements L30-L32
- Quote: "After authenticating the client (which may simply be an anonymous client if no attempt was made to authenticate), the token server must next query its access control list to determine whether the client has the requested scope." — S045, Requesting a Token L203-L205
- Quote: "Actions which may be performed on a `repository` are `pull` for accessing the collection and `push` for adding to it." — S046, Example Resource Types L58-L60
- Conditions/notes: htpasswd accepts only bcrypt entries and is loaded once at startup (S003 L715-L717); `silly` only checks that an Authorization header exists (S003 L642-L644). Token options: realm, service, issuer, rootcertbundle, jwks, signingalgorithms, autoredirect (S003 L661-L670). Anonymous pull is a token-server decision. Clock-skew leeway on token validity is 60 s (S044 L21-L23). Advisories on this surface: GHSA-phw4-mc57-4hwc "Token authentication allows to inject an untrusted signing key in a JWT" (2025-02-11) and GHSA-6pjf-3r9x-m592, tag deletion ignoring `storage.delete.enabled: false` (patched 3.1.1) (S068, S072).
- Relevance: the token protocol (scope `repository:<name>:pull,push`) is what every client speaks and must be kept; identity, RBAC and robot accounts have to be supplied by the registry itself.

### F030. Distribution (Q6, Q9): the catalog endpoint returns 100 entries by default and at most `maxentries` (default 1000), rejects a larger `n` with 400, and is documented as expensive; the cap exists because an unbounded `n` allowed an out-of-memory denial of service (advisory of 2023-05-09).
- Quote: "When a client requests more than `maxentries` via the `n` query parameter, the server rejects the request with a `400 Bad Request` response and a `PAGINATION_NUMBER_INVALID` error." — S003, catalog L812-L814
- Quote: "When a client does not specify the `n` query parameter, at most 100 entries are returned." — S003, catalog L811-L812
- Quote: "The server trusts that n has an acceptable value, however when using a maliciously large value, it allocates an array/slice of n of strings before filling the slice with data." — S069, References
- Quote: "Because it's a quite expensive operation, it should only be used when building up an initial set of repositories." — S025, registry.Repositories L15-L16
- Quote: "It is strongly recommended to keep this endpoint behind heightened privilege and avoid exposing it to the internet." — S003, catalog L818-L819
- Conditions/notes: constants `defaultReturnedEntries = 100` (S024 L16) and `defaultMaxEntries = 1000` (S004 L15). There is no search API, UI or vulnerability scanning in Distribution; the catalog is described as the feed for external search systems (S003 L816-L818).
- Relevance: every list endpoint needs a server-side page cap; the 100 and 1000 values themselves carry no stated reason beyond bounding memory.

### F031. Distribution (Q6, Q9, Q10, Q13): v3.0.0 read all tags and paginated in memory; v3.1.0 moved tag pagination into the storage walk and added `maxtags` (default 1000, larger `n` is clamped, no `n` returns every tag); v3.1.2 fixed dropped prefix tags on S3, and a user with about 100,000 tags per repository reports the unpaginated listing went from 30-50 seconds for a whole sync job on 2.8.3 to up to 30 minutes on 3.1.2.
- Quote: "Per the OCI distribution-spec, a server MAY return fewer than n results when a Link header is provided for continuation. Clamp to MaxTags instead of rejecting oversized requests." — S026, GetTags L51-L53
- Quote: "if limit is -1, we want to list all the tags, and receive a io.EOF error" — S026, GetTags L65
- Quote: "tagService := th.Repository.Tags(th) tags, err := tagService.All(th)" — S059, GetTags in v3.0.0 L37-L38
- Quote: "With 2.8.3, a synchronization job which lists repositories and then lists tags for each repository normally completed in approximately 30–50 seconds. After upgrading to 3.1.2, the same job intermittently takes many minutes (up to 30), and sometimes fails because requests to /tags/list time out." — S091, Description
- Quote: "Fix S3 tag pagination and catalog listing when one name is a prefix of another (#4898, #4904), and return an empty tag array after the final pagination marker (#4931)." — S065, Notable Changes
- Conditions/notes: #5038 conditions as reported: filesystem driver, "close to 100,000 tags" in some repositories, slowdown reproduced while a concurrent sequential read of all blobs saturates the disk; hardware not stated; the reporter attributes it to the change from `tagService.All()` (one directory read) to `List()`/`Walk()` in #4353/#4360. A fix commit referenced on the issue (2026-10-28) is not released. The v3.0.0 config struct has no `MaxTags` field (S060); the v3.1.0 notes describe the change as "Adds support for tag pagination (#4360, #4353)" (S063).
- Relevance: tag listing straight from a storage walk is fragile in both correctness and latency; serve it from an ordered index with a bounded page.

### F032. Distribution (Q7): the registry verifies no signatures and enforces no trust policy; validation covers only manifest structure, existence of referenced blobs, URL allow/deny lists for foreign layers and (optionally) presence of index platforms.
- Quote: "verifyManifest ensures that the manifest content is valid from the perspective of the registry. As a policy, the registry only tries to store valid content, leaving trust policies of that content up to consumers." — S048, verifyManifest L62-L64
- Quote: "By default the registry will validate that all platform images exist when an image index is uploaded to the registry. Disabling this validatation is experimental because other tooling that uses the registry may expect the image index to be complete." — S003, validation > indexes L1363-L1365
- Conditions/notes: "If `allow` is unset, pushing a manifest containing URLs fails." (S003 L1352). Signatures and SBOMs are stored like any other artifact; without referrers they are found through the tag schema (F006). The default index validation rejects sparse indexes (the "Data: Sparse Manifests" conformance row, F009) unless `platforms: none`.
- Relevance: signature verification and admission at pull are out of scope upstream; a registry that wants enforcement must own key and policy storage.

### F033. Distribution (Q8): the on-disk layout is `<root>/v2/blobs/<algorithm>/<first two hex bytes>/<hex digest>/data` plus per-repository `_layers`, `_manifests/revisions`, `_manifests/tags/<tag>/{current,index}` and `_uploads/<id>` link and state files; blobs are deduplicated registry-wide and a cross-repository mount writes one link file.
- Quote: "blobDataPathSpec: <root>/v2/blobs/<algorithm>/<first two hex bytes of digest>/<hex digest>/data" — S006, pathFor comment L115
- Quote: "manifestTagIndexEntryLinkPathSpec: <root>/v2/repositories/<name>/_manifests/tags/<tag>/index/<algorithm>/<hex digest>/link" — S006, pathFor comment L98
- Quote: "A layer can only be accessed through a qualified repository name if it is linked in the repository." — S006, pathFor comment L61-L63
- Quote: "if mountDigest != "" && fromRepo != "" {" — S030, StartBlobUpload L73
- Conditions/notes: the root prefix is `/docker/registry/` with layout version `v2` (S006 L12-L13). A mount needs both `mount` and `from`; without `from` a normal upload session starts (the optional behaviour that the new suite's "API: Blob anonymous mount" row tests, F009). A mount stats the blob in the source repository and writes a layer link in the target (S029 L274-L300). Upload sessions keep `data`, `startedat` and `hashstates/<algorithm>/<offset>` files (S006 L107-L109). Repository names are limited to 255 characters (`RepositoryNameTotalLengthMax = 255`, S052 L38-L39).
- Relevance: the link-file-per-fact layout is what makes every metadata query a listing; keep content addressing and per-repository membership, move the links into records.

### F034. Distribution (Q8, Q10): on S3 a finished upload is committed by a server-side copy to the content-addressed key followed by a delete, with multipart copy above 32 MB at up to 100 concurrent part copies; the code calls this terrible and justifies 32 MB as empirical.
- Quote: "This is terrible, but aws doesn't have an actual move." — S031, driver.Move L876
- Quote: "Empirically, multipart copy is fastest with 32 MB parts and is faster than PUT Object - Copy for objects larger than 32 MB." — S031, driver.copy L888-L889
- Quote: "defaultMultipartCopyMaxConcurrency defines the default maximum number of concurrent Upload Part - Copy operations for a multipart copy. defaultMultipartCopyMaxConcurrency = 100" — S031, const block L60-L62
- Conditions/notes: the measurement behind "empirically" (hardware, region, date) is not recorded. The commit path is `blobWriter.moveBlob` calling `driver.Move` (S013 L345).
- Relevance: uploading straight into content-addressed storage (digest known at commit, staged parts adopted without a copy) removes a full extra write per blob.

### F035. Distribution (Q8, Q9): blob GETs are redirected to the storage backend by default with pre-signed URLs that expire after a hard-coded 20 minutes on S3 and Azure.
- Quote: "For backends that support it, redirecting is enabled by default." — S003, storage > redirect L595-L596
- Quote: "func (d *driver) RedirectURL(r *http.Request, path string) (string, error) { expiresIn := 20 * time.Minute" — S031, driver.RedirectURL L1056-L1057
- Quote: "expiresTime := time.Now().UTC().Add(20 * time.Minute) // default expiration" — S034, signBlobURL L467
- Conditions/notes: no reason is given for 20 minutes and there is no config key.
- Relevance: redirect-to-store keeps blob bytes off the registry process; the URL lifetime bounds how long a revoked credential still works and should be a stated setting.

### F036. Distribution (Q9): the HTTP server is created without read, write, idle or header timeouts and connection draining on SIGTERM is off unless `draintimeout` is set; there is no request rate limit or concurrency limit option.
- Quote: "server := &http.Server{ Handler: handler, Protocols: serverProtocols(config), }" — S040, NewRegistry L169-L172
- Quote: "if config.HTTP.DrainTimeout == 0 { return registry.server.Serve(ln) }" — S040, ListenAndServe L325-L327
- Conditions/notes: the `http` section of the config struct has addr, net, host, prefix, secret, relativeurls, draintimeout, tls, headers, debug, http2, h2c and nothing for timeouts or limits (S004 L137-L187). A `TOOMANYREQUESTS` error code is defined and used when relaying upstream errors, but no handler emits it as a limiter (`rg -i TooManyRequests` hits only the error registry, the API descriptors and the internal client). Storage-side concurrency caps exist only for the filesystem driver (`maxthreads` default 100, minimum 25; S036, S037) and GCS (`maxconcurrency` default 50, minimum 25; S033).
- Relevance: every wait and queue needs a bound; the reference registry delegates all of this to a fronting proxy.

### F037. Distribution (Q9, Q11): webhook notifications go through an unbounded in-memory queue per endpoint and are lost when the instance stops; endpoint defaults are timeout 1 s, threshold 10 failures, backoff 1 s.
- Quote: "eventQueue accepts all messages into a queue for asynchronous consumption by a sink. It is unbounded and thread safe but the sink must be reliable or events will be dropped." — S041, type eventQueue L12-L14
- Quote: "Currently, the queues are inmemory, so endpoints should be _reasonably reliable_. They are designed to make a best-effort to send the messages but if an instance is lost, messages may be dropped." — S043, Considerations L332-L334
- Conditions/notes: defaults in `EndpointConfig.defaults` (S042 L26-L36); the docs mark timeout, threshold and backoff as required.
- Relevance: avoid both the unbounded queue and the silent drop; events should come from the durable record log.

### F038. Distribution (Q9): S3 multipart part size defaults to 10 MiB with a 5 MiB minimum taken from the S3 API and a 5 GiB maximum; GCS uses 16 MiB chunks in multiples of 256 KiB although its docs say 5242880; Azure writes at most 4 MiB per block.
- Quote: "minChunkSize defines the minimum multipart upload chunk size S3 API requires multipart upload chunks to be at least 5MB const minChunkSize = 5 * 1024 * 1024 const defaultChunkSize = 2 * minChunkSize" — S031, const block L48-L52
- Quote: "minChunkSize = 256 * 1024 defaultChunkSize = 16 * 1024 * 1024 defaultMaxConcurrency = 50 minConcurrency = 25" — S033, const block L50-L53
- Quote: "| `chunksize` | no (default 5242880) | This is the chunk size used for uploading large blobs, must be a multiple of 256*1024. |" — S054, Parameters L16
- Quote: "Depending on the speed of your connection to S3, a larger chunk size may result in better performance; faster connections benefit from larger chunk sizes." — S055, chunksize L66
- Conditions/notes: the GCS default in code (16 MiB) contradicts the GCS doc (5242880 bytes): a doc-truth gap. Azure `maxChunkSize = 4 * 1024 * 1024` (S034 L40); Azure copy-status polling retries 5 times with a 100ms step (S035, S057). S3 `maxChunkSize` is `5 * 1024 * 1024 * 1024` on 64-bit builds (S032). PR #4919 (S097, main only) measured the S3 writer's memory: "With 32 concurrent 100 MiB uploads at the default 10 MiB chunksize, the driver peaks at 740 MiB of heap".
- Relevance: provider minimums are the only justified numbers here; the defaults are untested trade-offs and per-upload buffers multiply by concurrency.

### F039. Distribution (Q9, Q10): blobs served through the registry are read through a 4 MiB buffer per GET whose size was never measured; a maintainer-merged PR on main (not in v3.1.2) removes it and reports 13% to 42% less CPU time per whole-blob GET.
- Quote: "TODO(stevvooe): Set an optimal buffer size here. We'll have to understand the latency characteristics of the underlying network to set this correctly, so we may want to leave it to the driver." — S039, fileReaderBufferSize L13-L15
- Quote: "The 4 MiB buffer dates from February 2015 (0270bec). That commit message records no measurement or specific reasoning." — S095, Performance impact
- Quote: "With a warm page cache, this patch reduces registry total CPU time by 13% to 42% per whole-blob GET (100 KiB to 32 MiB) and by 9% to 58% per range GET." — S095, Performance impact
- Quote: "plain HTTP 4 MiB 1.994 +/- 0.026 1.160 +/- 0.009 -41.8% plain HTTP 32 MiB 9.423 +/- 0.061 6.851 +/- 0.021 -27.3% TLS 4 MiB 3.401 +/- 0.027 2.404 +/- 0.018 -29.3% TLS 32 MiB 20.159 +/- 0.063 17.528 +/- 0.113 -13.0%" — S095, Performance impact table
- Conditions/notes: table unit and conditions as stated in the PR: "Registry total CPU time per GET in ms (mean +/- sample SD), warm page cache, linux/arm64", columns Before (n=15) and Now (n=15), filesystem driver; machine model, core count and Go version are not in the quoted part. Cold cache: "registry total CPU time was 2% to 7% higher for 1 GiB blobs, local NVMe drive". PR opened 2026-11-12 and merged to main; v3.1.2 still has `fileReaderBufferSize = 4 * 1024 * 1024` (S039 L18).
- Relevance: a concrete per-GET CPU baseline for the incumbent's filesystem path (about 1.2 to 2.0 ms CPU per 4 MiB blob over plain HTTP on arm64) and a documented case of a magic constant with no measurement behind it.

### F040. Distribution (Q10): the only before/after storage numbers published by the project are in PR descriptions; the S3 Walk rewrite (PR #3480, merged) reports enumerating about 50k blobs in 9.85 seconds and 49 API calls instead of an extrapolated 89.74 minutes and 48100 calls.
- Quote: "Test 1 (medium) ~300 blobs Before Took 53.9 seconds 479 API calls After Took 0.128 seconds 1 API call" — S094, Performance
- Quote: "Before - extrapolated (x18) Took 89.74 minutes 48100 API calls After Took 9.85 seconds 49 API call" — S094, Performance, Test 2 (large) ~50k blobs
- Quote: "On a few test registries, I performed a rough benchmark using BlobEnumerator::enumerate twice: Once before making these chances & again with the changes." — S094, Performance
- Conditions/notes: author's own rough benchmark (2021-08), S3-compatible object storage reached over the network (the PR's test instructions mention a DigitalOcean Spaces endpoint), hardware and network not stated; the "before" figure for 50k blobs is extrapolated from the first 5 minutes. The repository itself ships only `Benchmark*` harnesses for the storage drivers and path comparison, with no recorded results, and its docs publish no throughput or latency figures (`rg -i 'benchmark|req/s'` over docs/ and README is empty).
- Relevance: blob enumeration (the GC sweep) on object storage is bounded by LIST calls of 1000 keys; 50k blobs took about 10 s after the fix; scaled linearly (my extrapolation, not a measurement) millions of blobs take tens of minutes per sweep in the best case.

### F041. Distribution (Q10): manifest PUT verifies each referenced blob sequentially against storage; a PR merged to main (not in v3.1.2) makes the checks concurrent and reports 128,746,104 ns/op down to 21,460,250.5 ns/op with 10 ms injected per storage operation.
- Quote: "verifyManifest_latency (Primary Win): baseline 128,746,104 ns/op -> candidate 21,460,250.5 ns/op, -83.3313% reduction (n=10, p-value = 1.08251e-05, significant), validated/won." — S096, Headline Win
- Quote: "These figures are measured against a synthetic storage driver injecting 10ms of latency per operation to model a remote object-storage backend" — S096, Headline Win
- Conditions/notes: environment stated in the PR: OS linux, arch amd64, go1.26.4, baseline commit 472c9d3, candidate f22306a; benchmark `BenchmarkOCIManifestVerify` written for the PR; number of references in the benchmark manifest not quoted here. In v3.1.2 the loop is sequential (S048 L83-L136).
- Relevance: with link files, manifest push latency is references times storage round trip; with membership records it is one indexed lookup batch.

### F042. Distribution (Q10, Q11): deleting a manifest, and each manifest examined by `--delete-untagged`, reads every tag link in the repository; the docs admit it, a user reports more than 30 s to delete an image with about 5000 tags, and the lookup-then-untag sequence has a documented race with a concurrent push.
- Quote: "To find the tag references, the registry will iterate every tag in the repository and read it's link file to check if it matches the deleted manifest (i.e. to see if uses the same sha256 digest)." — S003, storage > tag L576-L578
- Quote: "Had a recent real-world example where it takes >30s to delete an image with ~5000 tags." — S093, issue body
- Quote: "Whilst we are still working on looking up all the other tags, a push is done over the test1 tag that changes the digests. test1 is deleted." — S093, issue body
- Quote: "The harbor GC will call into the registry to delete the manifest, which in turn then does a lookup for all tags that reference the deleted manifest." — S098, PR description
- Conditions/notes: #3525 does not state the backend or version (opened 2021-11). v3 added bounded concurrency (`storage.tag.concurrencylimit`, default `GOMAXPROCS`; PR #4329 motivated by Harbor issue 12948) but the work is still one read per tag and the delete is still not atomic with the lookup (S027 L494-L519).
- Relevance: a reverse index from manifest digest to tags makes delete O(result) and lets the tag check and the delete commit together.

### F043. Distribution (Q10, Q11): on S3 the catalog walked the whole repositories prefix for one page: a user with 1300+ repositories measured 57.79 s for `_catalog?n=50` on 2.8.1; v3 passes a start-after hint to the driver.
- Quote: "When we attempt to call the _catalog endpoint, we're seeing that some paginated requests can take over a minute to return, and do not correctly return the link header to retrieve the next page of results (at 150/1300 results, not at completion)." — S092, issue body
- Quote: "http.request.uri="/v2/_catalog?n=50&last=data-infra/airflow-export/export" http.request.useragent="curl/7.68.0" http.response.contenttype="application/json; charset=utf-8" http.response.duration=57.790265179s http.response.status=200 http.response.written=2251" — S092, registry log in issue body
- Conditions/notes: registry 2.8.1, S3, "1300+ repositories", hardware not stated. Fixed by PR #3685 ("Pass the last paging flag to storage drivers", merged 2022, in v3). v3.1.2 fixed a remaining catalog bug that dropped repositories whose name is a prefix of another (#4904, S065).
- Relevance: a registry-wide repository list must be an indexed scan; with link files even the fixed version costs a LIST per page plus skipped prefixes.

### F044. Distribution (Q10): there is no manifest or tag cache; the only cache holds blob descriptors (in memory, default 10000 entries, or Redis), the Redis keys never expire so Redis must run allkeys-lru, and stale repository membership in that cache was a High-severity advisory in 2026.
- Quote: "Currently, the only available cache provides fast access to layer metadata, which uses the `blobdescriptor` field if configured." — S003, storage > cache L557-L558
- Quote: "You **must** configure Redis with the **allkeys-lru** eviction policy, because the registry does not set an expiration value on keys." — S003, redis L1121-L1122
- Quote: "the delete path clears the shared digest descriptor but leaves stale repo-scoped membership behind, so a later Stat or Get from repo b repopulates the shared descriptor and makes the deleted blob readable from repo a again." — S070, summary
- Quote: "For AWS errors, we fail over to ListObjects:" — S031, driver.Stat L789
- Conditions/notes: a manifest GET by tag reads `tags/<tag>/current/link`, then the revision link, then the blob from the driver on every request (S027 L125-L148, S047 L89-L108). On S3 a `Stat` is a HEAD that falls back to a LIST when HEAD fails, so a miss costs two requests (S031 L786-L806). In-memory cache default: `DefaultSize = 10000` descriptors (S038 L16-L18), no reason given. The Redis layout is a set of member digests per repository plus a hash per digest (S049 L17-L27). v3.1.2 also notes "Check the storage backend directly when writing blobs to avoid stale descriptor-cache entries (#4868)" (S065). Advisory GHSA-f2g3-hh2r-cwgc: affected <= 3.0.x and <= 2.8.x with redis cache and delete enabled, patched 3.1.0.
- Relevance: an authorization-relevant fact (blob is a member of repository) lived in a best-effort cache; membership must be authoritative, and manifests are small enough to serve from the record store.

### F045. Distribution (Q10): resource footprint is published only as release artifact sizes: the v3.1.2 linux binaries are 13.6 MB to 15.3 MB as tar.gz; no idle memory or CPU figures are published.
- Quote: "registry_3.1.2_linux_amd64.tar.gz sha256:40df2224d410f72ae425c3371873b078bbdbda3b8b612be9571f0e6751f3acc8 15.3 MB 2026-09-24T12:10:31Z" — S066, Assets
- Quote: "registry_3.1.2_linux_arm64.tar.gz sha256:09d26f88d2c0f161bd1b8bfc6c123571cc3d291dcca8af5477b3cacc4ec95e73 13.6 MB 2026-09-24T12:10:31Z" — S066, Assets
- Conditions/notes: compressed archive sizes as shown by GitHub, not the unpacked binary size. Idle RSS and CPU: not found in docs, README or release notes.
- Relevance: a single static binary of this order (tens of MB) is the footprint to match.

### F046. Distribution (Q11): the most-commented open problems are in the pull-through cache, GC leftovers and storage-walk costs: #2367 (46 comments, 2017), #3725 multiple proxies (34), #4358 purge crash loop (33), #3716 references (25), #3178 GC data loss (21), #4478 proxy state panic (18).
- Quote: "registry container keeps crashing after log shows "PurgeUploads starting"" — S084, Description
- Quote: "the proxy cannot recover automatically from it leaking (losing track of) all the cached images in its storage." — S087, Description
- Quote: "When running in proxy/mirror mode, when upstream sends a 429 Too Many Requests, a 500 is returned by the proxy." — S089, issue body
- Quote: "Why was this issue closed as completed a year ago when it's still broken in registry 2? Just corrupted my company's docker registry." — S079, comment
- Conditions/notes: comment counts from the GitHub issue search API sorted by comments on 2026-11-20 (queries: `garbage`, `proxy in:title`, `s3 in:title`, `created:>2023-01-01`); #4358 is ECS Fargate with S3. Upgrade friction from 2.8 to 3: default config path moved to `/etc/distribution/config.yml`, oss and swift drivers removed, traces export had to be disabled by default (#4270, 27 comments), token auth key handling changed (#4299, 24 comments). The v2 line stopped at 2.8.3 while fixes landed only in v3 (F019).
- Relevance: the operational pain is metadata-on-object-storage (walks, leftovers, non-atomic deletes) and an under-designed proxy; both are design-level, not tuning-level.

### F047. Distribution (Q12): official binaries and images are Linux only, for amd64, arm64, armv6, armv7, ppc64le, s390x and riscv64; macOS is build-from-source with production use out of scope, and there are no Windows artifacts.
- Quote: ""linux/amd64", "linux/arm/v6", "linux/arm/v7", "linux/arm64", "linux/ppc64le", "linux/s390x", "linux/riscv64"" — S050, target platforms L59-L65
- Quote: "Production services operation on macOS is out of scope of this document. Be sure you understand well these aspects before considering going to production with this." — S051, Gotchas L21
- Quote: "If you are deploying a registry on Windows, a Windows volume mounted from the host is not recommended. Instead, you can use a S3 or Azure backing data-store." — S003, storage L500-L502
- Conditions/notes: the v3.1.2 asset list has only `registry_3.1.2_linux_<arch>.tar.gz` files plus checksums and provenance (S066).
- Relevance: first-class macOS and Windows server builds are not offered by the incumbent.

### F048. Distribution (Q13): v3.0.0 (2025-04-03) was the first stable v3 after v2.8.3; v3.1.0 (2026-04-06) added tag pagination and fixed two CVEs; v3.1.1 (2026-05-01) and v3.1.2 (2026-09-24, newest on 2026-11-20) are fix releases.
- Quote: "This is the first v3 stable release since v2.8.3 which is a culmination of years of hard work of the container community and registry maintainers!" — S062, release body
- Quote: "<id>tag:github.com,2008:Repository/28366058/v3.0.0</id> <updated>2025-04-03T06:26:12Z</updated>" — S067, entry v3.0.0
- Quote: "Deprecations Image Manifest v2 Schema v1 oss and swift storage drivers docker/libtrust has been replaced with go-jose/go-jose" — S061, release body
- Quote: "Fixes CVE-2026-35172 Fixes CVE-2026-33540 Adds support for tag pagination (#4360, #4353) Fixes default credentials in Azure storage provider (#4619) Drops support for go1.23 and go1.24 and updates to go1.25" — S063, Notable Changes
- Quote: "<id>tag:github.com,2008:Repository/28366058/v3.1.2</id> <updated>2026-09-24T12:21:27Z</updated>" — S067, entry v3.1.2
- Quote: "Verify manifest and blob content against the requested digest in the registry client, protecting digest-pinned requests and pull-through caches (CVE-2026-85747 / GHSA-685w-q87j-wqw3)." — S065, Notable Changes
- Conditions/notes: v3 pre-releases per the releases feed (S067): alpha.1 2023-12-19, beta.1 2024-07-10, rc.1 2024-11-08; the alpha.1 notes also list "Go module has changed from docker/distribution to distribution/distribution/v3" and "Major performance improvements across all supported storage drivers" without numbers. v3.0.0 deprecations listed in its notes: oss and swift drivers removed, docker/libtrust replaced by go-jose, client package no longer standalone, default config path `/etc/distribution/config.yml`. v3.1.0 also lists "Switch UUIDs to UUIDv7", "fix: sync parent dir to ensure data is reliably stored" (#4774) and "docs: pull through cache will pull from remote multiple times" (#4664). v3.1.1: CVE-2026-41888, S3 Express One Zone support, proxy tag-list fixes. v3.1.2: Azure append-write race fix, stale descriptor-cache fix, Go 1.26 required. On 2026-11-20 the referrers API (F006), sha512 manifests (F007), concurrent manifest verification (F041) and the file-serving change (F039) are on main or in open PRs, not in a release; the v3.1.2 page shows 101 commits to main since the release.
- Relevance: the reference implementation moves slowly (2.8.3 in 2023, 3.0.0 in 2025, 3.1.0 in 2026) and spec 1.1 features still trail the spec by more than two years.

**Harbor v2.16.0 (F049 to F106)**

### F049. Harbor (Q1): Harbor is a multi-service system written in Go: its installer deploys about ten containers (core, jobservice, registry, registryctl, portal, nginx proxy, PostgreSQL, a Redis-protocol cache, optional trivy-adapter and exporter) with Docker Compose or Helm; it is not a single binary.
- Quote: "Harbor can be deployed to a Docker host using Docker Compose, or to a Kubernetes cluster using Helm." — S127, Harbor Installation Prerequisites (intro)
- Quote: "BUILDBASETARGET=trivy-adapter core db jobservice nginx portal valkey registry registryctl exporter" — S106, Makefile L119
- Conditions/notes: v2.16.0. The compose template (S129) defines services log, registry, registryctl, postgresql, core, portal, jobservice, redis, proxy, trivy-adapter, exporter. Language: Go (backend, `src/go.mod`), Angular portal.
- Relevance: the baseline to beat on operability: one static binary versus ~10 containers plus two external stateful services.

### F050. Harbor (Q1): Harbor v2.16.0 does not embed Distribution v3; its registry container is built from Harbor's fork of Distribution v2.8.3, so blob bytes go through the v2.8.3 storage drivers (filesystem, azure, gcs, s3, swift, oss).
- Quote: "#versions REGISTRYVERSION=v2.8.3-patch-redis TRIVYVERSION=v0.76.1 TRIVYADAPTERVERSION=v0.40.0" — S106, Makefile L132-L135 (versions block)
- Quote: "# version of registry for pulling the source code REGISTRY_SRC_TAG=v2.8.3-harbor.3 # source of upstream distribution code DISTRIBUTION_SRC=https://github.com/goharbor/distribution.git" — S106, Makefile L141-L144
- Quote: "storage backend, default is filesystem, options include filesystem, azure, gcs, s3, swift and oss" — S124, storage_service block L75
- Counter: the GC-performance proposal names the Distribution v3 upgrade as out of scope while admitting v3 fixes the tag-lookup cost — "This proposal does not address the distribution v3 upgrade. We are aware that upstream distribution v3 introduces several improvements, particularly around tag lookup and deletion." — S117, Non Goals
- Conditions/notes: v2.16.0 (2026-11-10). The docs component table (S128) also lists "Distribution/Distribution 2.8.3".
- Relevance: Harbor inherits every Distribution v2 storage-layout cost (link files, tag walks) and none of the v3 fixes; a new registry competes with a 2.8.3-era data path.

### F051. Harbor (Q1): Metadata lives in PostgreSQL and a Redis-protocol store is mandatory (job queue, sessions, registry blob-descriptor cache, optional cache layer); v2.16.0 ships PostgreSQL 18 and replaces Redis with Valkey.
- Quote: "feat: replace redis with valkey as cache backend by @chlins in #23157" — S130, What's Changed > Exciting New Features
- Quote: "bump pg to v18 by @wy65701436 in #23184" — S130, What's Changed > Component updates
- Quote: "storage: cache: layerinfo: redis" — S125, storage.cache L6-L8
- Counter: the 2.16.0 docs component table still names the older stack — "Postgresql 15.10 Redis 7.2.6 Beego 2.3.4 Distribution/Distribution 2.8.3" — S128, Harbor Components
- Conditions/notes: v2.16.0. `DatabaseType` default is "postgresql" (S126 L79); the compose service is still named `redis` but its image is `goharbor/valkey-photon` (S129 L268-L269).
- Relevance: two external stateful services are the price of Harbor's features; a registry whose record store is built in removes both.

### F052. Harbor (Q1, Q10): The documented footprint for the whole stack is 2 CPU / 4 GB RAM / 40 GB disk minimum and 4 CPU / 8 GB / 160 GB recommended; no per-process idle RSS or binary size is published.
- Quote: "CPU 2 CPU 4 CPU Mem 4 GB 8 GB Disk 40 GB 160 GB" — S127, Resource Requirement (table: Resource, Minimum, Recommended)
- Conditions/notes: docs 2.16.0; the table gives no workload. Idle RSS, binary sizes: not found.
- Relevance: a single-binary registry can publish a measured idle RSS and binary size and compare against this 4 GB floor.

### F053. Harbor (Q13): Harbor's 2025-2026 minor releases were v2.13.0 (2025-04-10), v2.14.0 (2025-09-17), v2.15.0 (2026-03-20) and v2.16.0 (2026-11-10); the newest patch releases are v2.15.5 (2026-11-12) and v2.14.6 (2026-10-23).
- Quote: "Full Multi-Architecture Enablement for Harbor (amd64 + arm64) by @ranimandepudi in #22311" — S130, v2.16.0 > Exciting New Features
- Quote: "disable land tag in the backend storage by @wy65701436 in #22507" — S131, v2.15.0 > Exciting New Features
- Quote: "Syncs state with the upstream registry by deleting local cache when artifacts are removed; Serves the local manifest if its digest matches the upstream registry" — S132, v2.14.0 > Enhanced Proxy-cache
- Quote: "Enhances Harbor's audit logging with more granular tracking of user actions and system events, improving traceability and security." — S133, v2.13.0 > Audit log extension
- Conditions/notes: dates are the `datetime` of each GitHub release page (2025-04-10T11:24:18Z, 2025-09-17T06:02:05Z, 2026-03-20T04:36:02Z, 2026-11-10T13:28:07Z) and the releases feed for patches. Headlines: 2.13 audit-log extension, OIDC logout + PKCE, CNAI model artifacts, Redis TLS; 2.14 proxy-cache state sync, single active replication, GC progress display, replication adapter whitelist (breaking); 2.15 tags no longer written to backend storage, GC tag-deletion option, per-project `max_upstream_conn`, Cosign v3 bundles, pprof; 2.16 Valkey, amd64+arm64 images, referrers proxied through proxy-cache, proxy-cache repository filters, custom project roles, PostgreSQL 18.
- Relevance: the feature set a competitor is measured against as of 2026-11; note how many releases went to proxy-cache and GC repair.

### F054. Harbor (Q2): Harbor runs the OCI distribution-spec conformance suite nightly with all four categories enabled against the suite's current main branch, but the public report its README links to returned HTTP 404 on 2026-11-20, so per-category pass status could not be verified.
- Quote: "export OCI_TEST_PUSH=1 export OCI_TEST_PULL=1 export OCI_TEST_CONTENT_DISCOVERY=1 export OCI_TEST_CONTENT_MANAGEMENT=1" — S135, conformance_test.sh L26-L29
- Quote: "Check the OCI distribution conformance tests [report](https://storage.googleapis.com/harbor-conformance-test/report.html) of Harbor." — S136, OCI Distribution Conformance Tests
- Counter: Harbor is not among the registries listed on the OCI conformance site, whose distribution-spec list on 2026-11-20 reads — "Open Source Django-OCI Gitea Container Registry Keppel Keygen olareg OpenRegistry Zot Hosted Artifactory Azure Container Registry Cloudsmith GitLab Container Registry MetaHub" — S196, Table of Contents > Distribution Spec
- Conditions/notes: workflow `CONFORMANCE_TEST` runs on cron `0 6 * * *` and clones `opencontainers/distribution-spec` at HEAD (S134, S135 L5); it uploads `report.html` to an S3 bucket while the README points at a GCS URL. `curl` of the README URL and of the S3 bucket both gave 404 on 2026-11-20. No spec version number is claimed in README or docs (not found).
- Relevance: run conformance in CI at a pinned spec version and publish the per-category result where it stays reachable.

### F055. Harbor (Q2): Harbor implements the OCI referrers API itself (from its `artifact_accessory` table, not in Distribution) and filters by `artifactType`, comparing against the referrer's own artifact type.
- Quote: "at := req.URL.Query().Get("artifactType") var filter string if at != "" { filter = "artifactType" }" — S137, func (r *referrersHandler) ServeHTTP L63-L67
- Quote: "// Query accessories with matching subject artifact digest query := q.New(q.KeyWords{"SubjectArtifactDigest": reference, "SubjectArtifactRepo": repository})" — S137, ServeHTTP L75-L76
- Conditions/notes: v2.16.0. For each accessory the handler pulls the referrer manifest from the backend registry (or the Redis manifest cache when `cache.enabled`) to compute its size, so one referrers call costs one manifest fetch per referrer (L89-L133).
- Relevance: copy the idea (referrers answered from the metadata store), avoid the per-referrer manifest fetch by storing the descriptor size with the record.

### F056. Harbor (Q2): Digest algorithms beyond sha256 are not handled by Harbor's own layer: a manifest pushed by tag is re-addressed with the canonical (sha256) digest, and the proxy-cache code recognises a digest reference only by the `sha256:` prefix; no sha512 statement exists in the docs.
- Quote: "dgst := digest.FromBytes(data) req = req.Clone(req.Context()) req.URL.Path = strings.TrimSuffix(req.URL.Path, reference) + dgst.String()" — S123, func putManifest L201-L203
- Quote: "// when pushing with digest, should push to its actual digest if strings.HasPrefix(reference, "sha256:") { reference = string(newDig) }" — S138, func (m *ManifestListCache) push L165-L168
- Conditions/notes: v2.16.0. `rg -i sha512` over `src/**/*.go` (non-test) and over the 2.16.0 docs finds no digest-related hit. Whether a sha512 push passes conformance: not verified.
- Relevance: beat it: treat the digest algorithm as data end to end (sha256 and sha512 at least), never as a string prefix.

### F057. Harbor (Q3): Since v2.1 Harbor's garbage collection is online ("non-blocking"): it runs as a jobservice job while pushes, pulls and deletes continue, and protects in-flight uploads with a fixed 2-hour time window.
- Quote: "To avoid damaging the uploading artifact, the garbage collection introduces a time windows(2 hours) to reserve the recent uploaded layers. Garbage collection does not sweep the manifest & blob files that have a timestamp in the time window." — S107, Run Garbage Collection
- Quote: "Harbor runs garbage collection without interrupting your ability to continue use Harbor, for example you are able to push, pull, or delete artifacts while garbage collection is running." — S107, Run Garbage Collection
- Quote: "DefaultGCTimeWindowHours is the reserve blob time window used by GC, default is 2 hours" — S115, const block L166-L167
- Counter: the design proposal concedes the window is a second guard because optimistic concurrency alone does not close every race — "Even we introduce a update time to resolve the read/write, it still cannot resolve all of problems" — S116, Delete Blob & Manifest > DOUBLE GUARANTEE
- Conditions/notes: docs 2.16.0, code v2.16.0. The window is not a config option: only the env `GC_TIME_WINDOW_HOURS` overrides it and the code comment says "the env is for testing/debugging. For production, Do NOT set it." (src/lib/config/systemconfig.go). No measurement is cited for the value 2.
- Relevance: an online GC is table stakes; a wall-clock grace window is the weak part to replace with a proof (upload sessions and manifest commits recorded in the same store the collector reads).

### F058. Harbor (Q3): The GC algorithm is mark-and-sweep over database rows, not a storage walk: mark selects blobs with no `project_blob` row and `update_time` older than the window and sets status `delete`; sweep moves each to `deleting` and removes it through registryctl, which calls Distribution's vacuum code on the storage driver.
- Quote: "SELECT b.id, b.digest, b.content_type, b.status, b.version, b.size FROM blob AS b LEFT JOIN project_blob pb ON b.id = pb.blob_id WHERE pb.id IS NULL AND b.update_time <= now() - interval '%d hours';" — S110, func (d *dao) GetBlobsNotRefedByProjectBlob L406-L420
- Quote: "set the status firstly, if the blob is updated by any HEAD/PUT request, it should be fail and skip." — S108, func (gc *GarbageCollector) sweep L328
- Quote: "let the tags as empty here, as it non-blocking GC. The tags deletion will be handled via DELETE /v2/manifest" — S113, func (h *handler) delete L84
- Conditions/notes: v2.16.0. Candidates are held in memory for the whole run (`deleteSet []*blobModels.Blob`, S108 L75-L76); per-project orphan cleanup pages blobs 1000 at a time over projects listed 50 at a time (S108 L637, L647). Blobs the database does not know (written straight to storage, or lost rows) are never candidates.
- Relevance: copy "GC reads the record store, never lists the object store"; avoid the all-in-memory candidate set and the separate control service.

### F059. Harbor (Q3): Races between GC and pushes are handled with a four-state blob status machine guarded by an optimistic version column: a HEAD or PUT that touches a `delete` candidate rescues it, and a blob in `deleting` is answered 404 so the client uploads it again.
- Quote: "StatusDelete -> StatusNone : Client asks the existence of blob, remove it from the candidate." — S109, status comment L37-L53
- Quote: "UPDATE blob SET version = version + 1, update_time = ?, status = ? where id = ? AND version = ? AND status IN (%s) RETURNING version as new_version" — S110, func (d *dao) UpdateBlobStatus L185-L215
- Quote: "if the deleting exceed 2 hours, marks the blob as StatusDeleteFailed and gives a 404, so client can push it again" — S111, func handleHead L63-L72
- Counter: a `deleting` blob makes a concurrent push of the same content fail or re-upload rather than wait; the proposal's intent for manifests was different from what the middleware does — "if the manifest is in deleting status, pause the request and wait for deletion. Fail it when timeout." — S116, Sweep > Question 2 (uploading manifest at the phase of sweeping)
- Conditions/notes: v2.16.0. `probeBlob` (S112 L29-L64) applies the same rule to PUT blob and PUT manifest and returns a not-found error while a blob is `deleting` inside the window.
- Relevance: the state machine is sound and worth copying; do it in one transaction with the delete instead of across three services and a storage call.

### F060. Harbor (Q3, Q9): GC parallelism is a per-run `workers` setting, default 1 and at most 10; a GC job never retries and only one runs at a time; the UI limits manual runs to one per minute.
- Quote: "gc workers: default is 1. The business unit of removing blobs." — S108, func parseParams L160-L161
- Quote: "Error: Invalid number of workers:%s. Workers must be greater than 0 and less than or equal to 10." — S114, func (g *gcAPI) kick L112
- Quote: "To avoid triggering the garbage collection process too frequently, the availability of the GC Now button is restricted. Garbage collection can be only run once per minute." — S107, Run Garbage Collection
- Conditions/notes: v2.16.0. `MaxFails` 1, `MaxCurrency` 1, `ShouldRetry` false (S108 L81-L94). No reason is given for the bound 10.
- Relevance: constants without a stated reason; a new collector should bound concurrency by a measured storage-delete budget.

### F061. Harbor (Q3, Q10): GC cost at scale was dominated by Distribution's tag lookup: deleting one manifest through the v2 API walked every tag of the repository, measured by maintainers at about 39 s per manifest on S3 and by a user at over an hour for one manifest in a repository with more than 60,000 tags on GCS.
- Quote: "In S3 backend environment, we found that it took about 39 seconds to delete a manifest via v2 API." — S118, issue body (opened 2020-09-02 by a maintainer)
- Quote: "repositories with more than a few thousand tags is taking ~2 minutes to delete a single manifest during GC, GC is now taking 10~14 hours and the problem is getting worse every day" — S118, comment of 2021-10-21 (GCS backend)
- Quote: "Harbor relies on Distribution’s native tag deletion logic, which invokes the underlying storage driver to traverse all tags — a process that becomes slow, especially on object storage systems where directory traversal is costly." — S117, OCI Object Background
- Conditions/notes: 39 s: Harbor version not stated (issue opened 2020-09-02 by a maintainer), S3 backend, registry log `http.response.duration=38.598453034s`, repository size not stated. >1 h: Harbor v2.1.1, GCS, repository with >60,000 tags, 55525 `gcs.GetContent` log lines for that repository, registry log `http.response.duration=1h11m53.181188629s`. Issue open 2020-09-02 to 2025-11-20.
- Relevance: tag-to-manifest lookup must be an indexed record read; anything that lists per-tag objects makes GC O(tags) per manifest.

### F062. Harbor (Q3, Q11): Users report GC runs that never finish at tens of terabytes: an 18-day run on v2.8.4 that deleted nothing (about 10 TB live, about 75 TB in the bucket), and a v2.14.1 instance with 50 TB in S3 where 10 workers take minutes for a few objects.
- Quote: "we actually let a garbage collection task run for 18 days, only to find the following error happened about an hour ago, leaving us with nothing deleted from the bucket. We've got ~10TB of data in our actual registry, and ~75TB in the bucket, due to the GC failures just compounding over time." — S120, comment of 2024-03-12
- Quote: "garbage collection cycles are all-or-nothing, requiring a full pass of marking before any deletion occurs." — S120, issue body (2024-02-16)
- Quote: "I have a problem with GC essentially never finishing, even with 10 concurrent workers it seems to take several minutes for just a few objects." — S121, issue body (2026-01-17)
- Conditions/notes: user reports (forum-grade evidence). #19991: Harbor 2.8.4, about 150k artifacts in 648 repositories, AWS S3 + Aurora PostgreSQL + ElastiCache. #22739: Harbor v2.14.1, 50 TB in S3, RDS Aurora PostgreSQL 15.12 on db.r8g.2xlarge, 10 replicas each of jobservice/core/registry; the run had "21726 blobs and 12189 manifests eligible for deletion".
- Relevance: GC must be incremental and resumable (delete as candidates are proven, checkpoint progress); an all-or-nothing mark that outlives the job timeout frees nothing.

### F063. Harbor (Q3, Q13): The fix shipped in v2.15.0 (2026-03-20) is architectural: Harbor stopped writing tag link files to the backend registry (a tag push is proxied as a digest push; tags live only in PostgreSQL) and GC gained a `delete_tag` option to skip backend tag deletion.
- Quote: "If reference is a tag (not a digest), replace it with the computed digest before proxying to the backend. This prevents tags from being stored in the backend registry storage, while Harbor maintains the tag-to-digest mapping in the database." — S123, func putManifest L188-L191
- Quote: "Since harbor v2.0, harbor no longer leverages tags in the back-end storage. Tags are stored both in the storage and database, and harbor core interacts with the storage using digests." — S117, Background
- Counter: tag files written before the upgrade stay behind when the option is off, and the proposal's benchmark was still future work — "Benchmark GC performance in a real S3 environment before and after the change." — S117, Future Work
- Conditions/notes: v2.15.0/v2.16.0. `gc.deleteTag` defaults to true ("delete tag: default is to delete the tag in the backend storage", S108 L136-L140). No before/after GC benchmark was found in the proposal, the PRs (#22382, #22507) or the release notes.
- Relevance: confirms the design choice for a new registry: tags are records, never objects; and shows that an unmeasured fix leaves the performance claim open.

### F064. Harbor (Q3): A user-reported data-loss mode: on a PostgreSQL HA setup a GC reference query was cancelled on a standby ("conflict with recovery") and GC then deleted blobs that live manifests still referenced; the issue was closed as not planned without a confirmed root cause.
- Quote: "It appears that the GC deleted blobs despite being referred. Also, we verified the policies and there were repos with and without retention policies which were impacted by that." — S119, issue body (2021-08-18, Harbor 2.2.0, S3)
- Quote: "However, in the harbor side, it must handle the failure query in the GC to avoid any wrong deletion." — S119, comment of 2022-10-28 (Harbor 2.4.1, bitnami postgresql-ha)
- Counter: in v2.16.0 an error from the association query is returned to the caller, which stops cleaning that project instead of treating the blobs as unreferenced — "failed to clean untagged blobs of project: %d, %v" — S108, func markOrSweepUntaggedBlobs L686-L689
- Conditions/notes: user reports on Harbor 2.2.0, 2.3.5 and 2.4.1; 43 comments; maintainers never confirmed the cause; closed by the stale bot 2023-06-09 ("not planned"). The reporter's workaround was PostgreSQL `max_standby_streaming_delay` = 90s.
- Relevance: a collector must fail closed: any error or stale read while proving "unreferenced" aborts the delete; reads that justify deletion need linearizable (not replica) consistency.

### F065. Harbor (Q3, Q11): The maintainers' own list of online-GC defects (2021) names failure tolerance, the tag-deletion cost, upstream driver errors and a job that reports the wrong status once it runs longer than 24 hours.
- Quote: "Since Harbor introduces the online GC in v2.1, there are several issues that reported from community about online GC." — S122, issue body (2021-11-16, maintainer)
- Quote: "GC job reported the incorrect status whenever the execution exceeds 24 hours." — S122, issue body
- Conditions/notes: the 24 h figure matches `max_job_duration_hours: 24` (S124 L143-L144), made configurable in v2.13.0. Issue still open (31 comments).
- Relevance: a background collector needs no global job deadline if it works in bounded, checkpointed steps.

### F066. Harbor (Q4): Pull-through caching is a per-project feature: a "proxy cache project" is bound to exactly one registry endpoint, accepts no pushes, and supports nine upstream kinds.
- Quote: "A proxy cache project works similarly to a normal Harbor project, except that you are not able to push images to a proxy cache project." — S139, Configure Proxy Cache (intro)
- Quote: "Cannot push artifact to a proxy project: %v" — S140, func DisableBlobAndManifestUploadMiddleware L473-L491
- Quote: "PERMITTED_REGISTRY_TYPES_FOR_PROXY_CACHE=docker-hub,harbor,azure-acr,ali-acr,aws-ecr,google-gcr,docker-registry,github-ghcr,jfrog-artifactory" — S141, core env template L41
- Conditions/notes: docs 2.16.0, code v2.16.0. Multi-upstream fallback inside one project: not supported (one `registry_id` per project). Credentials are those of the registry endpoint: "Harbor users with access to the proxy cache project are able to pull any image available to the access account in the target repository" (S139).
- Relevance: a new registry can offer what Harbor does not: ordered multi-upstream mirrors per namespace and pushes alongside cached content, with an explicit rule for which wins.

### F067. Harbor (Q4): Freshness is checked on every pull by tag with an upstream HEAD (no TTL): the cached manifest is served only if the upstream digest still matches; if the upstream is unreachable the cached copy is served; if the tag is gone upstream nothing is served unless a per-project option says otherwise.
- Quote: "The next time a user requests that image, Harbor checks the image’s latest manifest in the target registry and serves the image based on the following scenarios:" — S139, How Harbor proxy cache works
- Quote: "If the target registry is not reachable, the proxy cache project serves the cached image." — S139, How Harbor proxy cache works
- Quote: "if rate limit, use local if it exists, otherwise return error" — S142, func (c *controller) UseLocalManifest L183-L189
- Counter: the documented rule is that nothing is served once the image is gone upstream; v2.16.0 added an opt-in project setting that serves the local copy instead — "Whether to serve images from local cache when they are removed from the upstream registry. The valid values are "true", "false"." — S143, definitions > ProjectMetadata > proxy_cache_local_on_not_found
- Conditions/notes: v2.16.0. Pull by digest of an artifact already cached skips the upstream call (S142 L177-L180). Any other upstream error falls back to the local copy in the middleware ("failed to proxy manifest, fallback to local", S140 L217). Each tag pull therefore costs one upstream round trip and counts against upstream HEAD rate limits.
- Relevance: copy "serve stale when upstream is down"; add a bounded freshness TTL so a tag pull does not need an upstream round trip every time, and state the stale-serving rule.

### F068. Harbor (Q4): A cache miss downloads each blob from the upstream twice (one stream to the client, a second in a goroutine to store it) and caches the manifest by polling: it sleeps 20 s up to 10 times for the blobs to arrive, and up to 20 times for the children of a manifest list.
- Quote: "wait more time than manifest (maxManifestWait) because manifest list depends on manifest ready" — S142, const block L41-L48 (maxManifestListWait = 20, maxManifestWait = 10, sleepIntervalSec = 20)
- Quote: "size, bReader, err := rHelper.BlobReader(remoteRepo, art.Digest)" — S142, func (c *controller) ProxyBlob L285-L306 (then `go func() { err := c.putBlobToLocal(...) }`, which opens a second BlobReader at L310)
- Quote: "it will wait and check for 30 mins, if all depend manifests are ready then push it" — S138, func (m *ManifestListCache) push L128-L131
- Counter: the comment's 30 minutes disagrees with the constants beside it: 20 iterations of a 20 s sleep is 400 s — "maxManifestListWait = 20" — S142, const block L43
- Conditions/notes: v2.16.0. No reason is given for 10, 20 or 20 s. The first pull of a tag is tagged locally by a retry loop: `ensureTagInterval = 10 * time.Second`, `ensureTagMaxRetry = 60` (S140 L65-L66).
- Relevance: avoid: tee the upstream stream into the store in one pass and commit the manifest when its blobs are durable (an event), not on a sleep loop.

### F069. Harbor (Q4): When a manifest list's children are not all cached in time, Harbor stores a trimmed manifest list containing only the children it has, so the cached index can differ in content and digest from the upstream one.
- Quote: "if time exceed, then push a updated manifest list which contains existing manifest" — S138, func (m *ManifestListCache) push L128-L131
- Quote: "cacheTrimmedDigest - cache the change Trimmed Digest for controller.EnsureTag when digest is changed" — S138, func (m *ManifestListCache) cacheTrimmedDigest L94-L107
- Conditions/notes: v2.16.0. The original index body is also kept in Redis for one week (`manifestListCacheInterval = 7 * 24 * 60 * 60 * time.Second`, S142 L46-L47, "keep manifest list in cache for one week") and served from there while the key lives. Open issue #20920 ("Harbor Proxy Cache not creating additional Multi-Architecture Images under OCI Image Index", 27 comments) is in this area.
- Relevance: avoid: a mirror must never rewrite content; store the index as pulled and fetch missing children on demand.

### F070. Harbor (Q4): Cache-stampede control is partial: an in-process map stops duplicate local stores of the same artifact, and an optional per-project `max_upstream_conn` counter in Redis rejects excess upstream fetches with HTTP 429 instead of coalescing them.
- Quote: "use inflight checker to avoid multiple requests to push blob to local in same time" — S144, func (l *localHelper) PushBlob L103-L114
- Quote: "The max connection per artifact to the upstream registry in current proxy cache project, if it is -1, no limit to upstream registry connections" — S143, definitions > ProjectMetadata > max_upstream_conn
- Quote: "redis.call('EXPIRE', KEYS[1], 3600) -- set expire to avoid always lock" — S145, increaseWithLimitText L37-L48
- Conditions/notes: v2.16.0. The in-flight map is per core process (`reqMap: make(map[string]any)`, src/controller/proxy/inflight.go), so replicas do not share it. `max_upstream_conn` arrived in v2.15.0 (#22348). The limiter key is per repository+digest by default, per project when env `UPSTREAM_REGISTRY_LIMIT_ON_PROJECT` is true (S140 L289-L296). Bandwidth to the upstream can be capped with `proxy_speed_kb`.
- Relevance: beat it with single-flight per digest across the fleet: concurrent requests for one missing blob share one upstream fetch and all receive the bytes.

### F071. Harbor (Q4): Proxy-cache content expires by tag retention, not by a cache TTL: each new proxy cache project gets a 7-day retention policy, and space returns only after garbage collection.
- Quote: "By default, Harbor creates a 7 day retention policy for each new proxy cache project." — S139, Create Proxy Cache Project
- Conditions/notes: docs 2.16.0. v2.14.0 added deletion of the local copy when the upstream artifact is gone (F053). No reason is given for 7 days.
- Relevance: eviction of cached content should be a first-class, budget-driven policy (size and age), not a side effect of tag retention plus GC.

### F072. Harbor (Q4, Q11): Proxy cache is the area with the most open, heavily commented defects, and v2.15.0 broke Docker Hub proxying outright until v2.15.1.
- Quote: "Because recent PR: #22535 doesn't set the User-Agent header in the request and dockerhub's Cloudflare consider it is a mallicious request and need to validation in browser. then it failed the Ping request to the registry." — S146, maintainer comment of 2026-03-24
- Quote: "Dockerhub return 429 frequently, we switched to registry.goharbor.io years before. that is why the e2e pipeline doesn't catch the error." — S146, maintainer comment of 2026-03-24
- Quote: "If I pull a new image (for example: harbor.hrp.extbrain.net/quay_proxy/argoproj/argocd:v2.6.3) through ProxyCache Project I expect the image is tagged in Harbor after it is pulled and cached. Currently the image is saved without any tag." — S147, issue body (2023-03-10, Harbor 2.7.1, containerd)
- Conditions/notes: #23025 (opened 2026-03-23, 28 comments, fixed in v2.15.1); #18335 (open, 58 comments); #21454 "Docker.io proxy cache isn't tagging official images" (open, 40 comments); #19609 "Images exists in a proxy cache project but it do not show in project repositories list" (open, 39 comments). Counts from GitHub issue search on 2026-11-20.
- Relevance: test the mirror path against the real public upstreams in CI, and make "cached and listed" one atomic record write.

### F073. Harbor (Q4): Replication is policy-driven copying between registries in push or pull mode with name/tag/label filters and manual, scheduled or event triggers; deletions replicate only for event-based rules, and Harbor-to-Harbor replication across different versions is unsupported.
- Quote: "Replication allows users to replicate resources, namely images and charts, between Harbor and non-Harbor registries, in both pull or push mode." — S148, Configuring Replication (intro)
- Quote: "Due to API changes, replication between different versions of Harbor is not supported." — S148, Configuring Replication (note)
- Quote: "Manual: Replicate the resources manually when needed. Note: Deletion operations are not replicated." — S149, Creating a Replication Rule > Trigger Mode
- Conditions/notes: docs 2.16.0. Failed tasks are "re-scheduled for a few minutes later and retried several times" (S148). Member information is not replicated. v2.14.0 added "single active replication" to stop overlapping runs of one policy.
- Relevance: replication built on per-artifact jobs is eventually consistent with no ordering or deletion guarantee; a registry with a replicated record store can make replication a property of the store.

### F074. Harbor (Q4, Q9): Replication constants: 10 concurrent executions per jobservice pod, chunked blob copy with a 10 MiB default chunk (officially verified only Harbor-to-Harbor), and 5 retries per blob or chunk with a 2 s doubling back-off.
- Quote: "The default chunk size is 10MB, you can override it by setting env REPLICATION_CHUNK_SIZE in the jobservice, the value should be united as bytes, for example 10MB=1024*1024*10 then you should set REPLICATION_CHUNK_SIZE=10485760." — S149, Creating a Replication Rule (Copy by chunk note)
- Quote: "please pay attention to the number of concurrent executions, the default value is 10 for each job-service pod." — S149, Creating a Replication Rule (bandwidth step)
- Quote: "for i, backoff := 1, 2*time.Second; i <= blobRetryCnt; i, backoff = i+1, backoff*2 {" — S150, func (t *transfer) copyBlobWithRetry L287-L302
- Conditions/notes: v2.16.0. `defaultChunkSize = 10 * 1024 * 1024`; `blobRetryCnt` and `chunkRetryCnt` default 5 (env `COPY_BLOB_RETRY_COUNT`, `COPY_CHUNK_RETRY_COUNT`), S150 L40-L61. No reason is given for any of them.
- Relevance: catalogue entries; chunk size and retry budget should be derived from measured link behaviour.

### F075. Harbor (Q5): Authentication is one of database, LDAP/AD or OIDC (UAA exists but is discouraged), chosen once: creating a local user locks the instance into database mode; with OIDC the Docker CLI logs in with a per-user "CLI secret" tied to the ID token.
- Quote: "If you create user accounts in the Harbor database, Harbor is locked in database mode. You cannot change to a different authentication mode after creating local users." — S151, Configuring Authentication (important note)
- Quote: "The Docker and Helm CLIs cannot handle redirection for OIDC, so Harbor provides a CLI secret for use when logging in from Docker or Helm." — S152, Using OIDC from the Docker or Helm CLI
- Counter: the CLI secret stops working when the provider gives no refresh token — "However, if the OIDC Provider does not provide a refresh token or the refresh fails, the CLI secret becomes invalid." — S152, Using OIDC from the Docker or Helm CLI
- Conditions/notes: docs 2.16.0. Registry clients get a bearer token from Harbor core's token service (`TokenServiceURL` default "http://core:8080/service/token"); the embedded registry itself only knows an htpasswd service user (S125 L69-L72). Token lifetime default 30 minutes (S126 L122). v2.13.0 added OIDC logout and PKCE.
- Relevance: support several identity sources at once and workload identity (OIDC token exchange) for CI, instead of a long-lived per-user secret.

### F076. Harbor (Q5): Authorization is project-scoped RBAC with five built-in roles (Limited Guest, Guest, Developer, Maintainer, ProjectAdmin) plus system administrator and anonymous; custom project roles arrived in v2.16.0; anonymous users can pull from public projects only.
- Quote: "Starting with Harbor v2.16.0, system administrators can also define custom project roles by selecting the permissions to grant. Project administrators can assign these roles to users or groups in each project. Built-in roles cannot be edited or deleted." — S153, Managing Users
- Quote: "An anonymous user has no access to private projects and has read-only access to public projects." — S153, Managing Users (system-level roles)
- Quote: "You can define up to 15 rules per project." — S185, Combining Rules on a Repository
- Conditions/notes: docs 2.16.0. Role constants `RoleProjectAdmin = 1 … RoleLimitedGuest = 5` in S115 L37-L41. Policies attach to the project (rules inside may select repositories and tags by pattern): public/private, quota, retention (max 15 rules, enforced in S187 L163-L164), immutability (max 15 rules, S186), CVE allowlist, deployment security, auto-scan, auto-SBOM. Access itself is granted per project, never per repository.
- Relevance: the unit of policy in Harbor is the project; per-repository and per-tag policy is the gap to fill.

### F077. Harbor (Q5): Robot accounts are the service-account mechanism: system-wide robots span projects with fine-grained permissions, project robots are confined to one project, tokens default to 30 days and may be set never to expire, and robots cannot use the UI.
- Quote: "An administrator can create a system-wide robot account covering multiple projects." — S154, Create System Robot Accounts (intro)
- Quote: "Robot Accounts cannot be used to log into the user interface." — S154, Create System Robot Accounts (intro)
- Quote: "{Name: common.RobotTokenDuration, Scope: UserScope, Group: BasicGroup, EnvKey: "ROBOT_TOKEN_DURATION", DefaultValue: "30", ItemType: &IntType{}, Editable: true, Description: `The robot account token duration in days`}," — S126, ConfigList L153-L154
- Conditions/notes: v2.16.0. The most-commented auth issue is #15253 ("Enabling OIDC causes all robot accounts to generate error: failed to verify secret", 73 comments, closed).
- Relevance: copy scoped, expiring machine credentials; prefer short-lived federated tokens to stored secrets.

### F078. Harbor (Q6): Search covers project and repository names only (`GET /api/v2.0/search?q=`); there is no search over tags, labels, annotations or CVEs in that endpoint, and the OCI `_catalog` and tag list are answered from PostgreSQL by loading every row and paginating in memory.
- Quote: "The Search endpoint returns information about the projects and repositories offered at public status or related to the current logged in user. The response includes the project and repository list in a proper display order." — S143, paths > /search
- Quote: "repoRecords, err := r.repoMgr.NonEmptyRepos(req.Context())" — S155, func (r *repositoryHandler) ServeHTTP L56-L74 (get all the non repositories; sort.Strings; without n everything is returned)
- Conditions/notes: v2.16.0. `/v2/<name>/tags/list` likewise lists all tags of the repository with one query and slices the result (src/server/registry/tag.go L71-L85). The REST API pages with `page_size` default 10, maximum 100 (S143, parameters > pageSize); the legacy base API uses 500 (S194 L35-L36). UI: Angular portal.
- Relevance: beat it with keyset pagination pushed into the record store and a stated bound on unpaginated `_catalog`.

### F079. Harbor (Q6): Vulnerability scanning is delegated to pluggable scanners over an adapter API; Trivy is the bundled default (Clair was dropped as default in v2.2), a Security Hub aggregates CVEs across the registry, and since v2.11 Trivy can generate an SBOM on push that is stored as an accessory.
- Quote: "Harbor provides static analysis of vulnerabilities in images through the open source project Trivy." — S156, Vulnerability Scanning (intro)
- Quote: "Since version 2.11 Harbor supports now automatic generation of SBOMs in combination with its default scanner - Trivy." — S157, SBOM Generation Capabilities (intro)
- Conditions/notes: docs 2.16.0. v2.16.0 bundles Trivy v0.76.1 and adapter v0.40.0 (S106 L134-L135). Scan timeout default `5m0s` with a hard cap: "There is upper bound of 30 minutes defined in scan job" (S124 L124-L126; `checkTimeout = 30 * time.Minute`, S193 L54). User report #22563: "Trivy Image Scanning took more than 50 hours" (2025-11-12).
- Relevance: keep scanning out of the registry binary behind a seam; store SBOMs and scan results as referrers.

### F080. Harbor (Q7): Signatures (Cosign, Notation) and SBOMs are stored as "accessories" linked to their subject, and a project can refuse pulls of unsigned images; the enforcement checks only that a signature accessory of the required type exists, it does not verify the signature against any key or identity.
- Quote: "When checked, Harbor will only allow verified images to be pulled from the project. Verified images are determined by Cosign or Notation, depending on the policy you have checked." — S158, Enforce deployment security
- Counter: the middleware's rule is presence, not verification — "If signature policy enabled, it has to at least have one signature." — S159, func ContentTrust L46-L54
- Quote: "for _, acc := range art.Accessories { if acc.GetData().Type == signatureType { hasSignature = true break } }" — S159, func signatureChecking L101-L107 (no cryptographic check follows)
- Conditions/notes: v2.16.0. `src/go.mod` has no sigstore or notation-go dependency. A Cosign signature is recognised by its tag shape `sha256-<hex>.sig` and layer media type `application/vnd.dev.cosign.simplesigning.v1+json` (src/server/middleware/cosign/cosign.go L43-L45), or by an OCI 1.1 `subject`. A User-Agent based exemption lets signing clients pull unsigned subjects unless `CONTENT_TRUST_LEGACY_SIGNER_PULL_ENABLED=false` (S159 L86-L95). Notary v1 was removed in v2.9.0 (F102).
- Relevance: beat it: verify signatures against configured trust roots at admission or pull and record the verified identity; "has a signature" is not a trust decision.

### F081. Harbor (Q7): A second pull-time gate blocks images whose scan shows vulnerabilities at or above a chosen severity; both gates are per-project switches.
- Quote: "Images cannot be pulled if their level is equal to or higher than the selected level of severity. Harbor does not prevent images with a vulnerability severity of negligible from running." — S160, Configure Vulnerability Settings in Projects
- Conditions/notes: docs 2.16.0; project metadata key `prevent_vul` ("prevent vulnerable images from being pulled", src/pkg/project/models/pro_meta.go L22). Open issue #16218 ("missing vulnerability visibiltity and broken Prevent vulnerable images from running", 30 comments) questions its reliability.
- Relevance: pull-time policy hooks are worth having; make them explicit typed refusals with the failing rule in the error.

### F082. Harbor (Q8): Bytes sit in Distribution v2.8.3's content-addressed layout (global blob store plus per-repository link files), deduplicated across the whole registry, while PostgreSQL tables `blob`, `artifact_blob` and `project_blob` mirror the references for quota and GC.
- Quote: "In Docker Distribution, blob storage is globally deduplicated. In Harbor, quota usage is tracked at the project level, so a blob referenced by multiple projects may be counted once in each project’s usage, even though the underlying blob storage may be shared." — S161, How Harbor Calculates Resource Usage
- Quote: "ProjectBlob holds the relationship between manifest and blob." — S109, type ProjectBlob L83-L89 (table project_blob: project_id, blob_id)
- Conditions/notes: v2.16.0. Layout example from a user's S3 listing: `docker/registry/v2/blobs/sha256/85/85f3…/data` and `docker/registry/v2/repositories/<repo>/_layers/sha256/<digest>/link` (issue #22111). Since v2.15.0 no `_manifests/tags/*` files are written (F063). The two stores can diverge: blobs present in storage but unknown to the DB are never collected (F058).
- Relevance: one source of truth: keep references only in the record store and make the object store hold nothing but content-addressed bytes.

### F083. Harbor (Q8): A cross-repository blob mount is recorded by Harbor core as a new project-to-blob association after the backend accepts the mount, and v2.16.0 added validation of the source project.
- Quote: "if err := blobController.AssociateWithProjectByDigest(ctx, mount, project.ProjectID); err != nil {" — S162, func PostInitiateBlobUploadMiddleware L32-L52
- Quote: "fix(security): validate blob-mount source project and reject tokens missing iat by @Vad1mo in #23270" — S130, v2.16.0 > Component updates
- Conditions/notes: v2.16.0. Harbor's conformance script sets `OCI_AUTOMATIC_CROSSMOUNT="false"` (S135 L31), which switches off the suite's automatic cross-mount test (mount without `from`).
- Relevance: a mount is a metadata-only operation; authorise it against the source repository and charge the destination's quota in the same transaction.

### F084. Harbor (Q9): Harbor caps a manifest PUT body at 4 MiB in its own layer and, unusually, states a reason; the audit-log payload shares the bound with none stated.
- Quote: "MaxManifestBodySize bounds the manifest PUT body buffered in memory. OCI manifests are sub-MiB in practice, so 4MiB is a generous ceiling." — S115, const block L253-L255 (MaxManifestBodySize = 4 << 20)
- Quote: "reference is a tag, not a digest; bound the buffered body so an over-limit manifest is rejected with 413 instead of buffered whole" — S123, func putManifest L192-L199
- Conditions/notes: v2.16.0. Used in three places (registry manifest PUT, blob middleware, quota middleware). Blob size has no Harbor-level cap: nginx is configured with `client_max_body_size 0` and the Beego limits default to 128 GB (`DefaultBeegoMaxUploadSizeBytes = 1 << 37`, S115 L246-L251).
- Relevance: the 4 MiB figure matches the OCI recommendation; the justification style ("observed sizes, so ceiling X") is the form every constant should take, with the measurement cited.

### F085. Harbor (Q9): Abandoned upload sessions are purged by the embedded Distribution (enabled by default, age one week, checked every 24 h), while Harbor's own record of a session, the accepted byte count used for quota, lives in Redis for only 24 hours.
- Quote: "remove files in _upload directories which exist for a period of time, default is one week." — S124, upload_purging block L309-L316 (age: 168h, interval: 24h, dryrun: false)
- Quote: "blobSizeExpiration: time.Hour * 24, // keep the size of blob in redis with 24 hours" — S199, func NewController L100-L103
- Conditions/notes: v2.16.0. The two lifetimes (168 h in storage, 24 h in Redis) are set independently and neither cites a reason. The values are passed to the embedded registry's `storage.maintenance.uploadpurging` through `make/photon/prepare/templates/registry/config.yml.jinja` L10-L23. No reason for one week is given.
- Relevance: a new registry should tie upload-session expiry to a stated budget (bytes held by open sessions) as well as an age.

### F086. Harbor (Q9): Database and cache pool defaults are sized for a single shared PostgreSQL: the installer sets 100 idle and 900 open connections per component against a bundled server limit of 1024, while the code's own defaults are 2 idle and unlimited open.
- Quote: "The maximum number of open connections to the database. If it <= 0, then there is no limit on the number of open connections. # Note: the default number of connections is 1024 for postgres of harbor. max_open_conns: 900" — S124, database block L55-L57
- Quote: "{Name: common.PostGreSQLMaxOpenConns, Scope: SystemScope, Group: DatabaseGroup, EnvKey: "POSTGRESQL_MAX_OPEN_CONNS", DefaultValue: "0", ItemType: &IntType{}, Editable: false}," — S126, ConfigList L111
- Counter: users hit the ceiling in production: connections "grew until the 3000" on an RDS instance, and a maintainer confirmed read replicas cannot be used — "Currently harbor does not support configure postgres read replica, the sql shown above is trying to update quota when client push image" — S163, maintainer comment of 2022-05-27
- Conditions/notes: v2.16.0 defaults; `conn_max_lifetime: 5m`, `conn_max_idle_time: 0`. Registry-to-Redis pool: `maxidle: 100`, `maxactive: 500`, `idletimeout: 60s`, read/write/dial timeouts 10 s (S125 L38-L48). The 900-of-1024 choice is explained only by the adjacent note. #16915: Harbor v2.5.0, 3 replicas per service, RDS m5.2xlarge (3000 connections).
- Relevance: every write path in Harbor funnels through one PostgreSQL primary; a registry on a replicated record store must show the same workload without a single-writer connection ceiling.

### F087. Harbor (Q9): Credential lifetimes are fixed defaults without stated reasons: registry bearer tokens 30 minutes, robot tokens 30 days, UI sessions 60 minutes, JWT leeway 60 seconds.
- Quote: "{Name: common.TokenExpiration, Scope: UserScope, Group: BasicGroup, EnvKey: "TOKEN_EXPIRATION", DefaultValue: "30", ItemType: &IntType{}, Editable: false, Description: `The expiration time of the token for internal Registry, in minutes.`}," — S126, ConfigList L122
- Quote: "{Name: common.SessionTimeout, Scope: UserScope, Group: BasicGroup, EnvKey: "SESSION_TIMEOUT", DefaultValue: "60", ItemType: &Int64Type{}, Editable: true, Description: `The session timeout in minutes`}," — S126, ConfigList L197
- Conditions/notes: v2.16.0. `JwtLeeway = 60 * time.Second` ("Global Leeway used for token validation", S115 L260-L261). A multi-request push or pull that lasts longer than 30 minutes needs a fresh token part-way through.
- Relevance: catalogue entries; token lifetime interacts with large-blob transfer time and should be derived from it.

### F088. Harbor (Q9): Background-work limits: 10 job workers per jobservice instance, a 24-hour maximum job duration after which a still-running task is marked as an error, and a 30-minute timeout on every HTTP client that talks to a registry.
- Quote: "the max time to wait for a task to finish, if unfinished after max_update_hours, the task will be mark as error, but the task will continue to run, default value is 24," — S164, reaper block L68-L72
- Quote: "defaultRegistryHTTPClientTimeoutMinutes is the default timeout (in minutes) for the registry related http clients (backend registry client, registry auth challenge/token client and reg adapters)." — S165, const block L33-L38 (= 30)
- Quote: "However, since each worker consumes a certain amount of network/CPU/IO resources, set the value of this attribute based on the hardware resource of the host. The default is 10." — S184, jobservice > max_job_workers
- Conditions/notes: v2.16.0. `max_job_workers: 10`, `max_job_duration_hours: 24` (S124 L140-L144); `max_dangling_hours: 168`; job log retrieval capped at 10 MB (`max_retrieve_size_mb: 10`). nginx gives `/v2/` a 900 s send/read timeout and `worker_connections 3096` (S191 L5, L191-L192). The 30-minute client timeout is what GC hit in issue #12948 ("context deadline exceeded" after exactly 30 minutes on one manifest delete).
- Relevance: a fixed 24 h job deadline and a 30 min client timeout turned slow GC into failed GC; long work should be chunked so no step needs a long timeout.

### F089. Harbor (Q9): Harbor has no request rate limiting of its own for registry or API traffic; the only limiters found are the optional per-project upstream-connection cap for proxy cache and bandwidth caps for proxy cache and replication.
- Quote: "The bandwidth limit of proxy cache, in Kbps (kilobits per second). It limits the communication between Harbor and the upstream registry, not the client and the Harbor." — S143, definitions > ProjectMetadata > proxy_speed_kb
- Conditions/notes: v2.16.0. `rg -i 'limit_req|limit_conn'` over the nginx templates finds nothing; `rg -i ratelimit` over `src/` finds only upstream-adapter and error-code uses. The Prometheus exporter limits itself (`HARBOR_EXPORTER_MAX_REQUESTS=30`, `HARBOR_EXPORTER_CACHE_TIME=23`, S192 L5-L6).
- Relevance: admission control (per tenant, per repository) is absent in the leading self-hosted registry; a typed 429 with a stated budget is a differentiator.

### F090. Harbor (Q10): The project publishes per-release load-test reports (k6, 500 virtual users). For v2.15.4 on AWS m8a.4xlarge workers (small data set), the authenticated API calls average 2.2 s to 3.62 s at 125.36/s to 186.23/s, while the unauthenticated `/v2/` ping (the script counts 200 or 401 as success) reaches 5105.29/s.
- Quote: "get-artifact-by-tag 2.58s 626.56ms 2.54s 4.92s 3.55s 4.23s 100% 177.75/s" — S166, Report > Small Size > Concurrency 500 (columns: Avg, Min, Med, Max, P(90), P(95), Success Rate, Iterations Rate)
- Quote: "get-v2 79.38ms 12.24ms 74.15ms 193.15ms 139.34ms 145.16ms 100% 5105.29/s" — S166, Report > Small Size > Concurrency 500
- Quote: "Hosts m8a.4xlarge (worker nodes in AWS) Kubernetes 1 * master , 3 * worker (v1.37) Harbor https://github.com/goharbor/harbor-helm (standard deploy) Storage AWS EBS" — S166, Environment
- Quote: "automountServiceAccountToken: false replicas: 1 podDisruptionBudget: enabled: false" — S197, core block L615-L618 (chart default for harbor-core)
- Conditions/notes: Harbor v2.15.4, wiki page edited 2026-11-06. "Small" data set: 100 projects x 100 repositories x 10 artifacts = 100000 artifacts, 5 tags each, 100 users, 500000+ audit logs. The report says only "standard deploy" of the Helm chart; the chart's defaults (v1.19.4, appVersion 2.15.4) are one replica of core, registry, jobservice, portal and nginx, no resource limits, the internal PostgreSQL and Redis, and the cache layer off. Whether the run changed them, best-of-N and load average are not stated.
- Relevance: the only maintained first-party numbers; a competitor should publish the same k6 scenarios on named hardware and beat both the v2.15.4 figure (177.75/s) and the v2.14.4 figure (3105.41/s, F092) for authenticated metadata reads.

### F091. Harbor (Q10): In the same reports an image pull or push is slow even for tiny content: at 500 virtual users v2.15.4 completes 13.07 pulls/s of one artifact (average 29.94 s, p95 67.91 s) and 7.91 pushes/s to one project (average 12.44 s); the scripts' default artifact is a single 1 KiB blob.
- Quote: "pull-artifacts-from-same-project 29.94s 644.25ms 26.69s 74.71s 61.6s 67.91s 100% 13.07/s" — S166, Report > Small Size > Concurrency 500
- Quote: "push-artifacts-to-same-projects 12.44s 2.71s 11.48s 19.54s 17.66s 18.06s 100% 7.91/s" — S166, Report > Small Size > Concurrency 500
- Quote: "BlobSize: getEnv('BLOB_SIZE', '1 KiB'), BlobsCountPerArtifact: getEnv('BLOBS_COUNT_PER_ARTIFACT', 1)" — S167, function getUserDataSettings > small L71-L82
- Counter: the wiki does not say whether `BLOB_SIZE` was overridden for the published run, so the artifact size behind these rows is the script default only by assumption — "vus: 500, iterations: 1000," — S168, export let options L15-L21
- Conditions/notes: Harbor v2.15.4, same environment as F090. The pull-same-project figure is flat across the four reports read (two test beds): 13.03/s (v2.10.0), 12.73/s (v2.13.0), 13.71/s (v2.14.4), 13.07/s (v2.15.4), with averages of 27.67 s to 30.1 s.
- Relevance: with a 1 KiB payload these numbers measure the metadata and auth path, not bytes; a registry that does one indexed read per manifest GET should be orders of magnitude faster on the identical scenario.

### F092. Harbor (Q10): Between the v2.14.4 and v2.15.4 reports, on the same stated environment, authenticated API throughput fell by more than an order of magnitude (get-artifact-by-tag 3105.41/s at 122.68 ms to 177.75/s at 2.58 s); in the same interval password hashing went from 4096 to 600000 PBKDF2 iterations.
- Quote: "get-artifact-by-tag 122.68ms 35.19ms 103.33ms 251.93ms 219.96ms 228.48ms 100% 3105.41/s" — S169, Report > Small Size > Concurrency 500
- Quote: "pbkdf2Iterations is the OWASP-recommended minimum iteration count for PBKDF2-HMAC-SHA256, used for all newly created/updated user passwords (PasswordVersion == PBKDF2SHA256). pbkdf2Iterations = 600000" — S170, const block L45-L54
- Quote: "return fmt.Sprintf("%x", pbkdf2.Key([]byte(content), []byte(salt), 4096, 16, HashAlg[encryptAlg]))" — S171, func Encrypt L50
- Conditions/notes: both reports: m8a.4xlarge workers, Kubernetes v1.37, harbor-helm standard deploy, AWS EBS, edited 2026-11-06. v2.16.0 keeps 600000 (S190 L45-L54). The causal link is this note's inference, not a project statement: the k6 harness authenticates with a username and password (`scripts/config.js` getHarborSettings), and the reports give no explanation for the change. `/v2/` ping, pull and push rows did not move comparably (get-v2 927.23/s then 5105.29/s; pull-same-project 13.71/s then 13.07/s).
- Relevance: per-request credential verification cost can dominate registry latency; verify a password once and issue a short-lived token, and benchmark the auth path separately.

### F093. Harbor (Q10): `/v2/_catalog` is the slowest API call in the v2.13.0, v2.14.4 and v2.15.4 reports and degrades with repository count: 125.36/s at 3.62 s average with 10000 repositories and 75.97/s at 5.87 s with 40000 (v2.15.4); on the older test bed v2.13.0 took 22.46 s on average at 40000.
- Quote: "get-catalog 5.87s 996.6ms 5.63s 11.99s 8.66s 9.03s 100% 75.97/s" — S166, Report > Medium Size > Concurrency 500
- Quote: "get-catalog 22.46s 5.85s 21.78s 26.81s 26.5s 26.68s 100% 20.78/s" — S172, Report > Medium Size > Concurrency 500
- Conditions/notes: repository counts derive from the data-size table (100 x 100 and 200 x 200). v2.13.0 report: 4 x AMD EPYC 7402P 24-core / 64 GB hosts, Kubernetes v1.23.6, OpenEBS dynamic NFS storage, page edited 2025-04-16. Cause in code: the handler loads all non-empty repositories, sorts them in memory, then slices (F078). In the v2.10.0 report only `list-audit-logs` was slower (medium data set, 5000000+ audit rows: 252.64 s average, S175); it is 2.48 s in the v2.15.4 report.
- Relevance: listing must be a bounded, keyset-paginated range read; cost per page should not depend on registry size.

### F094. Harbor (Q10): The optional Redis "cache layer" (v2.6+, off by default, 24 h expiry) was measured by the project at about 600 manifest GETs/s without it and over 4000/s with it; blobs are never cached.
- Quote: "if not enable cache, harbor can only handle about six hundreds requests every second, but if enable cache, harbor can process over 4 thousands requests, the improvement is about 7 times." — S173, Comparison > TPS
- Quote: "Example Script: hey -c 500 -z 10s -H "Authorization: YWRtaW46SGFyYm9yMTIzNDU=" https://harbor.domain/v2/library/alpine/manifests/sha256:4ff3ca91275773af45cb4b0834e12b7eb47d1c18f770a0b151381cd227f4c253" — S173, Comparison > Test Scenario
- Counter: enabling it trades consistency in HA deployments — "If you are deploying Harbor in HA mode, make sure that all the harbor instances have the same behaviour, all with caching enabled or disabled, otherwise it can lead to potential data inconsistency." — S124, cache block L318-L331
- Conditions/notes: wiki page edited 2022-09-28; hosts 4 x AMD EPYC 7402P 24-core / 64 GB, Kubernetes 1 master + 3 workers (v1.23.6), harbor-helm standard deploy, OpenEBS NFS; tool `hey`, one manifest (library/alpine by digest), concurrency 1000 to 20000. Without cache, "when the concurrent is 1000, not enable cache needs about 9 seconds"; at 20000 concurrency the success rate without cache was 38 percent. Exact figures are read off charts and given only in words.
- Relevance: a manifest GET that needs a cache to reach 4000/s on 96 cores is the bar; serving manifests from the record store's own memory-mapped pages should clear it without a second system.

### F095. Harbor (Q10): Concurrent pushes to one project contend on a single quota row: the project measured a more-than-4x throughput gain by moving quota updates to Redis (v2.9), and the push-to-same-project row improved from 4.06/s (v2.10.0) to 17.63/s (v2.14.4) across reports.
- Quote: "in the scenario of high concurrent push to the same project, because the update is the same quota_usage record in the db, so it will cause a lot of optimistic locking, which will cause the db connection number spike and a lot of occupation." — S174, Background
- Quote: "The throughput that can be processed per second has increased by more than 4 times after switching to Redis." — S174, Comparison > TPS
- Quote: "push-artifacts-to-same-projects 86.56s 2.8s 76.59s 226.39s 169.75s 190.25s 100% 4.06/s" — S175, Report > Small Size > Concurrency 500
- Counter: the Redis path makes quota accounting asynchronous and approximate — "However, the downside is that the displayed project quota usage in the UI may experience some delay and won't be updated in real-time. Nevertheless, the delay will be kept within a few minutes." — S174, Solution
- Conditions/notes: quota benchmark: "pushing a total of 5000 artifacts to the same project at 500 concurrence. (Harbor deployed by docker-compose in 16 CPU cores and 16G MEM VM)", page edited 2023-07-10; DB connections averaged "less than 25" with Redis. `quota_update_provider` still defaults to `db` in v2.16.0 (S126 L202). v2.14.4 small: "push-artifacts-to-same-projects 11.82s … 17.63/s"; hardware differs between the v2.10.0 and v2.14.4 reports, and the v2.15.4 report on the v2.14.4 hardware shows 7.91/s again (F091).
- Relevance: per-tenant counters updated on every push are a hot row; account usage with mergeable deltas (or per-shard counters) instead of a row lock.

### F096. Harbor (Q10): Every pull writes to PostgreSQL (pull count, pull time, audit log); the switches to turn these writes off are labelled a temporary solution for high-concurrency pulls.
- Quote: "When user disable the pull count/time/audit log, it will decrease the database access, especially in large concurrency pull scenarios." — S115, const block L194-L202 (PullCountUpdateDisable, PullTimeUpdateDisable, PullAuditLogDisable)
- Conditions/notes: v2.16.0; the comment cites issue #16039 and carries "TODO: Once we have a complete solution, delete these variables." All three default to false, i.e. the writes happen.
- Relevance: keep the pull path read-only; record pull statistics through a bounded, lossy-by-declared-rule side channel.

### F097. Harbor (Q10, Q11): Artifact listing has repeatedly been reported slow at scale; one user traced it to an unindexable JSON column in the task table and measured 45 s falling to 2 s after changing it to JSONB with a GIN index.
- Quote: "This is because the extra_attrs field is of type JSON and cannot have any kind of index on it." — S176, issue body (2024-05-28)
- Quote: "Reduced artifact listing time from 45 seconds to 2 seconds." — S176, issue body > Improvements
- Conditions/notes: user measurement, instance size not stated; 29 comments; closed. Related reports: #18598 "ListArtifacts API is slow, UI is timing out when accessing repositories with many artifacts", #21281 "List of artifacts with with_sbom_overview=true is slow" (open), #17511 "ListRepositories API slowed down considerably after upgrade to v2.5.3 from v2.3.3".
- Relevance: join-heavy list endpoints over a general SQL schema are a recurring cost; design list views as precomputed range scans.

### F098. Harbor (Q10, Q12): Published footprint data are coarse: the v2.16.0 offline installer is 734 MB (amd64) or 646 MB (arm64), and the stack's stated minimum is 2 CPU / 4 GB; no idle RSS per component is published.
- Quote: "harbor-offline-installer-v2.16.0-amd64.tgz sha256:643c137d8e829c857ef0ba45c46b5d6d8553077340bbd4929ded7e92808e77ff 734 MB 2026-11-10T13:37:21Z" — S177, release assets list
- Quote: "harbor-offline-installer-v2.16.0-arm64.tgz sha256:6525af9c26cc24e48ecf37d3f7d9d139895b39a037a3de697f995171ce65524a 646 MB 2026-11-10T13:37:21Z" — S177, release assets list
- Conditions/notes: the offline installer is a tarball of all container images. One incidental user observation (S188, issue #14358, v2.2.0, 2021) shows `harbor_core` at RES 94656 KiB in `top`; it is not a controlled measurement.
- Relevance: publish binary size and idle RSS for the single binary; the comparison point is a 0.6-0.7 GB image bundle.

### F099. Harbor (Q12): The server runs only as Linux containers; v2.16.0 (2026-11-10) is the first release whose notes and assets include arm64 beside amd64; there is no Windows or macOS server build.
- Quote: "Full Multi-Architecture Enablement for Harbor (amd64 + arm64) by @ranimandepudi in #22311" — S130, v2.16.0 > Exciting New Features
- Quote: "VALKEYVERSION and VALKEYSHA256 are only used for the arm64 source build. # On amd64 the Photon tdnf package version is used directly." — S106, Makefile L137-L139
- Conditions/notes: v2.15.5 (2026-11-12) still ships a single un-suffixed installer (`harbor-offline-installer-v2.15.5.tgz`). The Makefile maps only `aarch64` and `x86_64` (L75-L81). Deployment requires Docker Engine > 20.10 with Compose, or Kubernetes with Helm (S127).
- Relevance: Linux/macOS/Windows on x86_64 and aarch64 from one static binary is outside what Harbor offers.

### F100. Harbor (Q11): The most-commented operational problems cluster in six areas: GC duration and failure on object storage, proxy-cache correctness, PostgreSQL contention and connection exhaustion, slow list endpoints, replication failures, and breakage on upgrade.
- Quote: "The following queries are reported with duration over 10seconds:" — S178, issue body (2021-06-02, Harbor v2.2.1, AWS RDS, 300 repositories x about 800 artifacts)
- Quote: "When putting more images to be hosted and pulled from Harbor, basically the number of connections to the database grew until the 3000 (limit of an m5.2xlarge RDS with 32GB of RAM)." — S163, issue body (2022-05-26, Harbor v2.5.0)
- Quote: "The GC jobs claims it can clean up 2.7 Terabyte of space. Of the total Quota used: 5.56TiB that is 47.84% we can remove with GC." — S118, comment of 2025-06-25
- Conditions/notes: comment counts from GitHub issue search on 2026-11-20: GC #12948 (52), #15445 (43), #15807 (37, open), #16010 (31, open), #19991 (20); proxy cache #18335 (58, open), #21454 (40, open), #19609 (39, open), #23025 (28); database #14358 "High CPU usage (core and database) after 2.2.0 upgrade" (38), #15048 (27), #16915 (21), #21062 "Harbor DB deadlocks" (21, open); replication #8037 "504 Gateway Time-out when replication use pull mode" (40), #18813 "Harbor Replication Issue: Invalid Image Manifest Causes Transfer Failure and Vulnerability Scan Error" (40), #15283 (36, open); upgrades #13589 "Harbor Upgrade from v1.10.3 to v2.1.1 / Multiple Issues Seen" (50), #20800 "Harbor PG14 to PG15 migration failed" (19), #23025 (v2.15.0 broke Docker Hub proxy cache); BuildKit cache export #14774 (64, open); robot accounts under OIDC #15253 (73).
- Relevance: each cluster maps to a design rule for a new registry: GC from records, mirror writes atomic with listing, no hot rows, range-scan listings, schema migrations that are online and reversible.

### F101. Harbor (Q11): Upgrades are offline and one-directional: the documented procedure stops and removes the running instance, migrates the PostgreSQL schema, and insists on a prior backup; the guide for 2.16.0 covers only upgrades from v2.14.0 and later.
- Quote: "This guide only covers migration from v2.14.0 and later to the current version. If you are upgrading from an earlier version, refer to the migration guide for an earlier Harbor version." — S179, Upgrade Harbor and Migrate Data (intro)
- Quote: "Since the migration might alter the database schema and the settings of harbor.yml, you should always back up your data before any migration." — S179, Upgrade Harbor and Migrate Data (intro)
- Conditions/notes: docs 2.16.0. v2.16.0 also moves the bundled database from PostgreSQL 15 to 18 (the db image carries pg15 binaries "only used by pg_upgrade", make/photon/db/Dockerfile.base). High availability is delegated: "Highly available PostgreSQL 9.6+ (Harbor does not handle the deployment of HA of database)" and Redis Cluster is unsupported (install-config/harbor-ha-helm).
- Relevance: rolling, mixed-version upgrades with append-only formats are a concrete advantage over stop-migrate-start.

### F102. Harbor (Q2): The only spec-version statement found is in the v2.9.0 notes (2023-09-01): support for OCI Distribution Spec v1.1.0-rc2; the design names the referrers tag schema a non-goal, and manifest deletion by tag is refused because tags are deleted through Harbor's own API.
- Quote: "Harbor now supports OCI Distribution Spec v1.1.0-rc2 and added support for Notation signature and Nydus conversion as referrers." — S180, v2.9.0 > Supporting OCI Distribution Spec v1.1.0-rc2
- Quote: "3. Supports [Referrers Tag Schema](https://github.com/opencontainers/distribution-spec/blob/v1.1.0-rc1/spec.md#referrers-tag-schema)" — S181, Non Goal
- Quote: "If tag deletion is disabled, the registry MUST respond with either a 400 Bad Request or a 405 Method Not Allowed" — S123, func deleteManifest L143-L155 (v2 doesn't support delete by tag)
- Counter: a long-standing interoperability gap is still open: BuildKit registry cache export fails against Harbor — "ERROR: error writing manifest blob: failed commit on ref "sha256:878798161048504bee835f947ff94fdd915232bd8b277aacd6a511e5e77bd475": unexpected status: 404 Not Found" — S195, issue body (2021-04-28; open, 64 comments)
- Conditions/notes: no statement for the final v1.1.0 or v1.1.1 spec was found in the 2.13-2.16 release notes, README or docs. The same v2.9.0 notes record "Removal of Notary" ("Starting with version v2.9.0, Harbor no longer includes Notary in either the user interface or the backend."). Whether #14774 still reproduces on v2.16.0 was not tested.
- Relevance: state the spec version implemented and keep a public list of deliberate deviations (here: no tag deletion over `/v2/`, no referrers tag-schema fallback).

### F103. Harbor (Q3, Q11): Orphaned storage is a reported outcome of GC on S3-compatible stores: the blob delete returns "Path not found", GC treats not-found as done and removes the database row, and the bytes stay in the bucket where no later GC can see them.
- Quote: "Harbor GC only cleans up manifest objects from S3 (jobservice log shows that other objects like blobs are deleted though)" — S182, reporter's comment of 2025-06-24 (Harbor 2.13.1; Hitachi Content Platform, NetApp StorageGRID; another user: Ceph)
- Quote: "Other than on regular filesystems, S3 doesn't have "directory"-objects. These objects doesn't exist in S3 (that's why we see a "Path not found"), instead you find /docker/registry/v2/.../<sha>/data objects. These objects must be removed instead." — S182, reporter's comment of 2025-10-01
- Quote: "a not found error is ignored to keep the GC going, but such a blob is not removed by this run, so its size must not be counted as freed space." — S108, func (gc *GarbageCollector) sweep L432-L433
- Conditions/notes: user report, open issue (opened 2025-06-23, 18 comments), not confirmed by maintainers on AWS S3 itself. In v2.16.0 the not-found case still proceeds to "delete blob record from database" (S108 L468-L480); only the freed-space counter was corrected. The reporter recovered the space by running the embedded registry's offline `garbage-collect`.
- Relevance: deleting the record before the bytes are provably gone creates invisible orphans; delete bytes first, verify, then drop the record, and run a reconciling scrub with a counted budget.

### F104. Harbor (Q1, Q8): Scale-out means running more replicas of stateless services against one PostgreSQL, one Redis-compatible store and one shared volume or object bucket; Harbor does not provide HA for either database, and Redis Cluster is unsupported.
- Quote: "Highly available PostgreSQL 9.6+ (Harbor does not handle the deployment of HA of database)" — S183, Prerequisites
- Quote: "Please note that Harbor presently doesnt support Redis Clusters or TLS based connections." — S183, Prerequisites
- Quote: "Most of Harbor’s components are stateless now. So we can simply increase the replica of the pods to make sure the components are distributed to multiple worker nodes" — S183, Architecture
- Conditions/notes: docs 2.16.0 (the page still says PostgreSQL 9.6+ and no Redis TLS although v2.13.0 added Redis TLS and v2.9 requires PostgreSQL >= 12 for external databases, per the upgrade guide). Multi-region operation is by replication policies between independent Harbor instances (F073), not one logical registry.
- Relevance: "one code path from laptop to multi-region" is exactly what this architecture cannot offer; each region is a separate registry with eventually consistent copy jobs.

### F105. Harbor (Q4, Q10): The proxy-cache design set one performance target, a bound on overhead relative to a normal project, and no published measurement against it was found.
- Quote: "Because the proxy project needs to pull images from a remote registry, and have some concurrent limitation on the request, the overall image pull performance degradation must less than 50%." — S189, Goal
- Conditions/notes: design proposal for v2.1 (2020). The perf wiki reports contain no proxy-cache scenario; v2.16.0 added proxy cache hit/miss metrics (#23578).
- Relevance: publish cached-pull and miss-path numbers for the mirror path next to the direct path.

### F106. Harbor (Q1, Q10): Every registry request crosses three processes: nginx proxies `/v2/` to harbor-core, whose Go reverse proxy applies Harbor's middleware (auth, quota, GC status, proxy cache, policy) and forwards to the Distribution container with a service credential; v2.16.0 had to enlarge that hop's idle-connection pool to stop port exhaustion under load.
- Quote: "The reverse proxy forwards all /v2/ traffic to a single host (the registry), so it deserves a dedicated transport with a large per-host idle connection pool." — S198, func newProxy L36-L38
- Quote: "Relying on http.DefaultTransport (MaxIdleConnsPerHost=2) closes almost every connection after use under high concurrency, piling up TIME_WAIT sockets and eventually exhausting ephemeral ports (EADDRNOTAVAIL)." — S198, func newProxy L38-L40 (tr.MaxIdleConns = 1024; tr.MaxIdleConnsPerHost = 1024)
- Conditions/notes: v2.16.0 (release note "fix: set MaxIdleConnsPerHost on internal HTTP transports", #23591). nginx: `proxy_pass http://core/v2/;` with request and response buffering off (S191 L183-L192). Blob bytes follow the same path unless the storage driver redirects the client to the object store (`redirect: disable: false` is the default, S124 L80-L82).
- Relevance: one process that authenticates, authorises and serves from its own store removes two network hops and two connection pools from every request; a rare example here of a constant whose comment explains the failure it prevents.

**zot v2.1.23 (F107 to F168)**

### F107. Zot (Q1): zot is one Go binary driven by one configuration file, shipped as a "minimal" build (distribution-spec core only) and a "full" build (core plus compiled-in extensions).
- Quote: "zot is a single binary image with all features included so that deployment is extremely simple in various environments, including bare-metal, cloud, and embedded devices. Behavior is controlled by a single configuration file." — S243, Design Goals > Single binary model
- Quote: "The minimal flavor is for the security-minded and minimizes the number of dependencies and libraries." — S240, Build-time hardening > Conditional builds
- Quote: "zot: a production-ready vendor-neutral OCI image registry - images stored in OCI image format, distribution specification on-the-wire, that's it!" — S200, README opening line L4
- Conditions/notes: Go module `zotregistry.dev/zot/v2`, `go 1.27.0` in go.mod at v2.1.23. Release assets per platform: zot, zot-minimal, zot-debug, zli (CLI), zb (benchmark), zxp (metrics exporter for minimal builds) (S249). Extensions are Go build tags (sync, search, scrub, metrics, lint, ui, mgmt, profile, userprefs, imagetrust, events; Makefile L86-L87).
- Relevance: copy the single binary + single config + feature-gated build; note that zot stops being self-contained as soon as more than one instance runs (F110, F134).

### F108. Zot (Q1, Q8): blob bytes live on a local filesystem or, through the storage drivers of CNCF Distribution v3, on S3, GCS (since v2.1.15) or Azure Blob Storage (since v2.1.18).
- Quote: "Currently, zot supports AWS S3, Google Cloud Storage (GCS), and Azure Blob Storage. S3 storage is not supported on Windows; see Configuring remote storage with s3 for details." — S229, Storage backends > Remote filesystem
- Quote: "Azure Blob Storage support is based on the distribution v3 storage driver and follows the same remote-storage behavior model as S3 and GCS." — S229, Configuring remote storage with Azure Blob Storage
- Quote: "github.com/distribution/distribution/v3 v3.1.2" — S227, require block L32
- Conditions/notes: `storage.subPaths` routes repository prefixes to separate stores, each with its own driver, dedupe and GC settings (S201, Storage). NFS/FUSE mounts are treated as local filesystems (S229).
- Relevance: zot inherits Distribution's path-oriented driver (stat/list/move on keys), so every limitation of that abstraction (LIST-heavy walks, no transactional multi-key update, no transient-error class before v2.1.22) carries over; a content-addressed object store API avoids that class.

### F109. Zot (Q1, Q8): the registry's source of truth is the per-repository OCI image layout (index.json plus blobs); the metadata database (BoltDB by default, DynamoDB or Redis when remote) is created only for search, authn user data, image trust, retention or quotas, and is rebuilt from storage by a startup walk.
- Quote: "Furthermore, the actual storage is simply an OCI Image Layout." — S229, Storage model
- Quote: "// init metaDB if search is enabled or we need to store user profiles, api keys or signatures" — S209, func InitMetaDB L451
- Quote: "Previous releases supported a local BoltDB or a remote DynamoDB database in order to store image and blob metadata. This release now includes support for Redis." — S239, v2.1.3 > Redis driver Support
- Counter: the metadata database also holds state that cannot be rebuilt from storage — "Removing meta.db also deletes data that storage cannot rebuild, including API keys, user stars and bookmarks, and download statistics." — S235, Signatures, attestations, and metadata storage > Option B
- Conditions/notes: InitMetaDB condition at v2.1.23 (controller.go L456-L457): search enabled, or basic authn enabled, or image trust enabled, or retention enabled, or quota enabled. The same page says the registry is unavailable during the rebuild walk and that on a large registry the walk can exceed a systemd startup timeout.
- Relevance: copy "storage is self-describing and the index is derived"; avoid mixing derived and non-derivable state in one store, and avoid an O(registry) walk on the restart path (F162).

### F110. Zot (Q1, Q8): the dedupe "cache" database is BoltDB for one local instance and DynamoDB or Redis when instances share storage; with remote dedupe it is required state, not a cache.
- Quote: "Because BoltDB does not provide concurrent access for writes, multiple instances/replicas of zot are not supported with a BoltDB configuration." — S229, Cache drivers > BoltDB
- Quote: "Redis is an alternative to BoltDB (which cannot be shared by multiple zot instances) and DynamoDB (which requires access to AWS)." — S229, Cache drivers > Redis
- Counter: losing the cache database makes stored blobs unreadable in a shared-bucket deployment — "In this deployment, yes, zot does need the data in Redis to be able to recognize the blobs it has access to." — S270, comment by a zot contributor, 2026-05-12
- Counter: operators read the option names as a cache — "The config knobs cacheDriver and remoteCache are slightly misleading, though, since they affect more than just caching (as traditionally understood) then." — S270, reporter's reply, 2026-05-19
- Conditions/notes: reported on v2.1.16 with two instances, one shared bucket, `dedupe` and `remoteCache` true, Redis without persistence; after Redis was redeployed empty, pulls failed with unknown blob. Required external services for zot: none on one node with local disk; object storage plus DynamoDB or Redis for any multi-instance or object-store deployment that keeps dedupe. Config validation allows Redis with local or S3 storage and DynamoDB only with remote storage (root.go L462-L471).
- Relevance: avoid; dedupe bookkeeping must have the durability of the blobs it describes (same replicated record store), and its loss must degrade to "no dedupe", never to "blob unknown".

### F111. Zot (Q2): the project states full conformance with the OCI Distribution Specification and runs the upstream conformance action (distribution-spec main, OCI_VERSION 1.1) on every push and pull request to main.
- Quote: "zot fully conforms to the OCI Distribution Specification." — S242, Why zot?
- Quote: "# pinned to opencontainers/distribution-spec main@fcfba1ec55526073f48b2f6d4e3d7eef410ddcbc" — S223, job run > step run conformance, L38
- Conditions/notes: the workflow sets `OCI_VERSION: "1.1"` (L45) and uploads junit.xml, report.html and result YAML as artifacts; example configs declare `"distSpecVersion": "1.1.1"`; zot warns and overrides when the configured value differs from the compiled-in spec version (root.go L1314-L1323).
- Relevance: copy; run the conformance suite pinned to a spec commit in the gate, and publish the result file per release.

### F112. Zot (Q2): zot's published OCI conformance result for distribution-spec v1.1 (zot v2.1.19) is Pass with 848 passed, 0 failed, 4 skipped and 2 disabled of 854 checks; upload cancel, tag-parameter push and sparse manifests were disabled in that run.
- Quote: "OCI Conformance Result: Pass Disabled......................: 2 Skip..........................: 4 Pass..........................: 848 FAIL..........................: 0 Error.........................: 0 Total.........................: 854" — S255, Test results (summary block)
- Quote: "Manifest put with tag params..: Disabled Manifest put with subject.....: Pass" — S255, Test results > API conformance
- Quote: "name: Zot version: v2.1.19 website_url: https://zotregistry.dev/" — S256, PRODUCT.yaml L2-L4
- Conditions/notes: run with conformance commit fcfba1ec, TLS disabled, config `"gc": true, "dedupe": false`. API results: Blob upload cancel Disabled; Blob anonymous mount Skip; Manifest put with tag params Disabled; all other API rows Pass (tag listing, tag delete, blob push/post-only/post-put/chunked/streaming/mount/get/range/head/delete, manifest put/get/head/delete, referrers, ping). Data results: Sparse Manifests, Tag Param, Tag Param sha512 Disabled; others Pass. The current suite reports per API and per data type; it no longer reports the four legacy workflow categories (pull, push, content discovery, content management), so a per-category status in those terms was not found.
- Relevance: beat; a new registry should publish the same result file with zero disabled rows and with dedupe enabled.

### F113. Zot (Q2): the referrers API is implemented with artifactType filtering and the OCI-Filters-Applied response header, and passes the conformance check.
- Quote: "// filter by artifact type (more than one can be specified)" — S211, func GetReferrers L717
- Quote: "Manifest delete atomic........: Pass Referrers.....................: Pass Ping..........................: Pass" — S255, Test results > API conformance
- Conditions/notes: route `GET /v2/{name}/referrers/{digest}`; header set at routes.go L754-L757; a missing repository or a subject without referrers returns 200 with an empty index (L681-L682). The response is one index; no pagination of referrers was found in the handler.
- Relevance: copy the behaviour; bound the response (paginate) for subjects with very many referrers.

### F114. Zot (Q2): digest algorithms other than sha256 are accepted (blobs are stored under blobs/<algorithm>/<hex>), sha512 passes conformance, and since v2.1.23 an upload can declare its algorithm up front.
- Quote: "Beginning with v2.1.23, the optional digest-algorithm query parameter provides an algorithm hint for a multi-request upload, for example POST /v2/example/blobs/uploads/?digest-algorithm=sha512." — S247, POST /v2/{name}/blobs/uploads
- Quote: "return path.Join(is.rootDir, repo, ispec.ImageBlobsDir, digest.Algorithm().String(), digest.Encoded())" — S207, func BlobPath L2047
- Quote: "Blobs sha256..................: Pass Blobs sha512..................: Pass" — S255, Test results > Data conformance
- Conditions/notes: the accepted set is whatever go-digest v1.0.0 reports as available (routes.go func parseDigestAlgorithmQuery L2322-L2338); the set is not enumerated in zot's tree. The release in which sha512 was first accepted was not found.
- Relevance: copy the per-algorithm path and the up-front algorithm hint (it lets the server hash once while streaming).

### F115. Zot (Q2): zot is OCI-only by default: Docker schema 2 manifests are rejected unless a compatibility switch is set, and since v2.1.22 non-conformant tags are refused on push.
- Quote: "Without this compatibility setting, zot rejects Docker schema 2 content instead of converting it to OCI." — S230, introduction (warning)
- Quote: "zot does not implement, support, or require any vendor-specific protocols, including that of Docker." — S229, Storage model
- Quote: "Manifest pushes now reject invalid Distribution Specification tags with 400 MANIFEST_INVALID. Existing non-conformant tags remain readable and deletable." — S239, v2.1.22 > Runtime, API, and Compatibility Improvements
- Conditions/notes: the switch is `http.compat: ["docker2s2"]` (v2.1.2+); the official container image ships with it enabled in its default config (S241). The most-commented feature request in the tracker is #724 "FR: loose mode allowing Docker mediaTypes" (51 comments per GitHub issue search on 2026-11-20; not quoted here).
- Relevance: accept Docker schema 2 media types unchanged from day one (digest-preserving); "OCI-only" was the top adoption complaint.

### F116. Zot (Q3): garbage collection is online and on by default: no read-only mode, one background task per repository, default delay 1 hour and default interval 1 hour.
- Quote: "Garbage collection in zot is an inline feature meaning that it is not necessary to take the registry offline." — S229, Storage features > Garbage collection
- Quote: "DefaultGCDelay = 1 * time.Hour DefaultGCInterval = 1 * time.Hour" — S204, const block L24-L25
- Quote: "Dedupe: true, GC: true, GCDelay: storageConstants.DefaultGCDelay, GCInterval: storageConstants.DefaultGCInterval," — S202, func New L883-L886
- Counter: the same docs page describes the interval default differently in its attribute table — "If no value is specified, there is no periodic collection." — S229, Configuring zot storage > Configurable attributes > gcInterval
- Conditions/notes: code is authoritative: `config.New()` sets `Dedupe: true, GC: true, GCDelay: DefaultGCDelay, GCInterval: DefaultGCInterval` (config.go L883-L886); the page's own table further down says "GC enabled with 1 hour interval (default)" when `gc` is omitted. `gcInterval: 0` means one run only.
- Relevance: copy online GC; avoid documentation that disagrees with defaults (a doc-truth test would catch this).

### F117. Zot (Q3): the collector is a per-repository mark and sweep over the repository's own index.json: apply retention to tags, drop untagged manifests and orphan referrers, rewrite index.json, then delete every blob in the repository that the rewritten index no longer reaches.
- Quote: "CleanRepo executes a garbage collection of any blob found in storage which is not referenced in any manifests referenced in repo's index.json" — S205, func CleanRepo doc comment L114-L115
- Quote: "Instead of removing manifests one by one with storage APIs we just remove manifests descriptors from index.Manifests[] list and update repo's index.json afterwards." — S205, func cleanRepoLocked L184-L185
- Conditions/notes: order in cleanRepoLocked (L170-L303): tag retention; referrers with missing subject and untagged manifests; unknown-media-type entries; stale entries whose blobs are gone; stale uploads; write index.json; delete unreferenced blobs older than the delay; remove the repository when nothing is left and drop its metadata record. Each pass lists all blobs of the repository and reads every manifest, every interval, whether or not anything changed; cost is O(manifests + blobs) per repository per interval. Task starts are spread by a random delay of up to 30 s per repository (gc.go L97-L100).
- Relevance: avoid periodic full rescans; drive collection from recorded reference changes (delete and overwrite events) so cost follows churn, not registry size.

### F118. Zot (Q3): the only protection for an in-flight push is age: an unreferenced blob or an upload is kept while its modification time is younger than gcDelay.
- Quote: "Orphan blobs are removed if they are older than gcDelay." — S201, Storage L160
- Quote: "The configuration model also allows the configuration of a tunable delay (gcDelay), which can be set depending on client network speeds and the size of blobs." — S229, Configuring garbage collection
- Quote: "// Do not delete blob uploads which have been updated recently" — S205, func deleteBlobUploads L1242
- Conditions/notes: the age test is `modtime.Add(delay).After(time.Now())` (gc.go L1241 and L1352), on the storage object's mtime. There is no upload-intent or lease record: a push whose manifest arrives more than gcDelay after one of its blobs was written can lose that blob. The stated reason for the knob is client network speed and blob size; no reason is given for the 1 h default.
- Relevance: avoid a wall-clock grace period as the correctness mechanism; tie blob liveness to an explicit, renewable upload or push session recorded in the metadata store.

### F119. Zot (Q3, Q10): each image store has a single read-write lock that every repository maps to; a GC pass over one repository holds the write lock, so GC blocks pushes and pulls in the whole store while it runs.
- Quote: "That is safe only because the default store has a single lock, which every repo maps to." — S206, interface ImageStore, comment on WithRepoLock L34-L35
- Quote: "WithRepoLock runs wrappedFunc while holding the store's single write lock." — S207, func WithRepoLock comment L130
- Quote: "On high-load registries, garbage collection can add lock contention on storage while it runs." — S201, Storage L166-L167
- Counter: user-measured effect of the lock — "Approx 400 repositories. Each repositories have 1-5 tags. 90% of those are ephemeral Around 30 concurrent clients with 1-2 rps Latency currently is up to 10s" — S258, reporter's comment, 2025-02-17
- Conditions/notes: the report is for v2.1.2, one node, S3 storage with DynamoDB cache, GC and retention on; the issue is still open on 2026-11-20. `cleanRepo` takes `WithRepoCleanupLock`, which is the store's write lock (imagestore.go L183-L187); `PutImageManifest` takes the same write lock (L828), `GetImageManifest` the read lock (L773). v2.1.19 added `gcTimeWindow` to confine sweep starts to off-peak hours instead of removing the lock.
- Relevance: beat; metadata operations must be per-key transactions in the record store, with no process-wide or store-wide lock on the request path.

### F120. Zot (Q3): a documented data-loss race: with dedupe on S3, GC deleted blobs pushed one second earlier, because the age of a dedupe placeholder was read from the original object (fixed in v2.1.23).
- Quote: "With dedupe: true on S3 storage, garbage collection can delete blobs that were pushed a second ago, before the client has pushed the manifest that references them." — S257, Describe the bug
- Quote: "So in practice gcDelay only protects blobs whose content is new to the registry." — S257, Describe the bug
- Quote: "// Use the per-repo object age, not StatBlob: for remote dedupe placeholders" — S205, func isBlobOlderThan L1336
- Conditions/notes: reported on v2.1.22 with S3, Redis cache, three replicas, default gcDelay 1 h; the client's manifest PUT failed with MANIFEST_BLOB_UNKNOWN because four of six freshly pushed layers were collected. The v2.1.23 notes list "prevent premature garbage collection" (S239).
- Relevance: avoid; a grace period computed from a different object's timestamp is a silent correctness hole. Liveness must be a property of the reference, not of the bytes.

### F121. Zot (Q3, Q8): remote dedupe has lost or emptied blob content more than once (2024 to 2026): the single content copy was deleted or overwritten while zero-byte placeholders in other repositories still pointed at it.
- Quote: "after some pushes and retention intervals a real ones blobs are deleted, all other ones have zero size and Zot can't find the real blobs to return to client." — S274, Describe the bug
- Quote: "Additional fixes prevent remote dedupe self-link and concurrent-upload races from emptying shared content or returning spurious 500 responses." — S239, v2.1.22 > Storage Outage and Data-Integrity Handling
- Quote: "Corrupt S3 dedupe stubs no longer return a successful empty blob response." — S239, v2.1.22 > Storage Outage and Data-Integrity Handling
- Conditions/notes: #2625 is against v2.1.1 on S3 with retention; the v2.1.22 fixes are dated 2026-10-06. Mechanism in F147.
- Relevance: avoid placeholder-plus-pointer dedupe on an object store; store each blob once under its digest and let repositories hold references in the metadata store.

### F122. Zot (Q3): until v2.1.22 storage errors were collapsed into "not found", so GC, scrub and enumeration could act on an incomplete view; since v2.1.22 they fail closed.
- Quote: "Zot’s ImageStore.StatBlob / originalBlobInfo currently collapses almost every Stat failure to bare zerr.ErrBlobNotFound, so callers (especially GC) cannot tell a true miss from a storage blip." — S267, Summary (issue opened by a maintainer)
- Quote: "risk of pruning live tags if List/Stat is incomplete during a blip, or silent soft-degraded GC with no health signal." — S267, Motivation
- Quote: "Garbage collection, deduplication, scrub, and storage enumeration fail closed when a backend cannot provide a complete view." — S239, v2.1.22 > Storage Outage and Data-Integrity Handling
- Conditions/notes: v2.1.22 (2026-10-06) introduced three error classes across local, S3, GCS and Azure drivers: missing (404), transient (503), permanent (500) (S229, Storage failure handling).
- Relevance: copy the three-way classification and fail-closed rule from the start; typed errors that distinguish "absent" from "could not tell" are a precondition for any destructive background job.

### F123. Zot (Q3, Q11): one bad entry can stop collection for a repository or for the whole pass with no health signal: a dangling manifest reference (v2.1.5), an undeletable stale upload on S3 (v2.1.22 and v2.1.23) and a nested repository named <x>/blobs/sha256 (v2.1.23, open) each made GC fail on every pass.
- Quote: "In our estate GC reached only 3 of 16 repos before dying, on every pass, indefinitely. Storage reclamation was silently disabled estate-wide while the registry kept growing (~90 GiB of collectable data accumulated)." — S266, What happened
- Quote: "If a repository on S3 has a stale blob upload that GC cannot delete, GC of the repository fails on every pass." — S272, Describe the bug
- Quote: "With <x>, <x>/blobs and <x>/blobs/sha256 pushed, every GC run of <x> fails, and nothing in <x> is collected any more:" — S268, Summary
- Conditions/notes: #4271: S3 driver, retention and Redis cache, data migrated at bucket level from a Distribution layout; fixed later. #4695 exists because a nested repository is a subdirectory of its parent's layout on local storage, so repository names can collide with the layout's own `blobs/` directory.
- Relevance: avoid; isolate failures per item with a counted budget, surface a GC health metric, and never let repository names share a namespace with internal storage paths.

### F124. Zot (Q3): no GC duration or throughput figure is published; the documented mitigations are a memo that reads each manifest once per pass, a fix for repeated exponential index walks (v2.1.21), GC metrics (v2.1.17) and a daily time window (v2.1.19).
- Quote: "// every read of a manifest or index during that pass hits storage once." — S205, type GarbageCollect, field memos L69
- Quote: "Garbage collection now correctly handles multi-tag manifests, shared cosign referrers, nested indexes, and Docker-compatible media types without repeated exponential index walks." — S239, v2.1.21 > Storage and Distribution Behavior
- Quote: "Periodic garbage collection can be restricted to a daily UTC gcTimeWindow, including windows that cross midnight." — S239, v2.1.19 > Retention and Garbage Collection Controls
- Conditions/notes: GC durations at a stated registry size: not found in docs, release notes or the issues read. CI runs a GC stress job (zb -c 10 -n 100 against filesystem and MinIO with a short GC interval; .github/workflows/gc-stress-test.yaml) but publishes no numbers.
- Relevance: beat; publish GC cost per deleted reference and per stored object on named hardware.

### F125. Zot (Q3): retention is part of GC: per-repository glob policies keep tags by regex, recency of push or pull, or top-N counts; with any keepTags rule present, tags that match no rule are deleted; untagged manifests are deleted by default since v2.1.9.
- Quote: "If at least one keepTags policy is defined for a repository, all tags not matching those policies are removed." — S231, Configuring retention policies (warning)
- Quote: "Untagged manifests are now removed by default in garbage collection unless they are referenced by indexes or artifacts." — S239, v2.1.9 > Garbage Collection and Retention
- Counter: before v2.1.9 GC alone freed nothing after a tag was overwritten — "After some testing the disk space will only be freed by adding retention policies." — S260, Describe the bug (v2.1.8)
- Conditions/notes: rules: `patterns` (regex), `pulledWithin`, `pushedWithin`, `mostRecentlyPushedCount`, `mostRecentlyPulledCount`; `keepUntagged` (v2.1.19) protects digest-only pull-through entries; `deleteReferrers` default false, `deleteUntagged` default true, retention `delay` defaults to gcDelay (24 h before v2.1.9); `dryRun` and `zot verify-feature retention` preview a policy (v2.1.11). A repository uses the first policy whose glob matches. Pull-time based rules need the metadata database. Retention policies shipped in v2.0.0 (2023-12-18), not in 2.1.x.
- Relevance: copy the rule set and the dry-run; avoid "any rule present flips the default to delete" without an explicit confirmation.

### F126. Zot (Q4): mirroring is the built-in "sync" extension with two modes that can be combined: periodic polling of selected content, and on-demand (pull-through) fetch on a local miss; it uses the regclient library.
- Quote: "For a pull through cache mirrored registry, configure zot for on-demand synchronization. When an image is first requested from the local zot registry, the image is downloaded from the upstream registry and cached in local storage." — S230, Mirroring modes
- Quote: "Because Docker Hub rate-limits pulls and does not support catalog listing, do not use polled mirroring with Docker Hub. Use only on-demand mirroring with Docker Hub." — S230, Mirroring modes (note)
- Quote: "it uses regclient to first copy images into this dir (as oci layout) and then move them into storage." — S216, type Config, field DownloadDir L50-L52
- Conditions/notes: periodic mode needs `pollInterval` and at least one `content` entry (prefix glob, tag regex, semver filter, destination, stripPrefix); since v2.1.22 periodic sync can copy only selected platforms of an index while keeping the index digest (sparse index). With object storage, images are first staged on local disk (`extensions.sync.downloadDir`).
- Relevance: copy both modes and sparse multi-arch mirroring; avoid staging whole images on local disk before commit.

### F127. Zot (Q4): several upstream registries can be configured, each with an ordered list of fallback URLs and its own filters; a pull-through instance remains an ordinary registry that also accepts pushes.
- Quote: "A list of one or more URLs to an upstream image registry. If the main URL fails, the sync process will try the next URLs in the listed order." — S230, Basic configuration for mirroring with sync > urls
- Quote: "Registry requests are load-balanced between two zot instances, each of which mirrors the other." — S244, Active/active
- Conditions/notes: upstream selection for an on-demand request is by `content` prefix match per registry entry; there is no per-repository "proxy project" type that refuses pushes, so pushed and mirrored content share one namespace (an explicit statement about pushes to a mirrored repository was not found; the active/active recipe relies on it). Scale-out: the instance that owns the repository hash performs the sync (S232).
- Relevance: copy multi-upstream with ordered fallbacks; make the push-versus-mirror policy per repository prefix explicit and typed.

### F128. Zot (Q4): freshness model: by default every on-demand request for a tag revalidates the manifest against upstream; requests by digest are served locally; an opt-in interval (v2.1.21) throttles revalidation and is kept in memory only.
- Quote: "With onDemand enabled, every request for a tag validates the manifest against the upstream registry, because a tag is mutable and may have been repushed." — S201, Sync's manifestCheckInterval option L1569-L1570
- Quote: "the last check time is kept in memory, so the first request after a restart checks upstream again" — S201, Sync's manifestCheckInterval option L1589
- Counter: before these controls the revalidation exhausted upstream rate limits — "The main problem is, as soon an docker.io Image is being cached and gets refetched by a client, ZOT starts to reevaluate all existing images within its registry and starts downloading the images again." — S262, Describe the bug (v2.1.3-rc5)
- Conditions/notes: `manifestCheckInterval` default is 0 (check upstream on every tag request); there is no TTL-based expiry of cached content: cached images live until retention/GC removes them, and `keepUntagged` (v2.1.19) exists because digest-only cache entries were otherwise collected as untagged. The reporter of #3129 had a Docker Hub limit of 200 pulls that was used up "within a pull of 3-8 already cached images".
- Relevance: copy tag revalidation by HEAD with a persistent, per-reference freshness record; avoid GET-based revalidation and in-memory-only freshness state.

### F129. Zot (Q4): when the upstream is unreachable a locally cached image is still served; if nothing is cached, hard upstream failures return 503 since v2.1.22 (earlier they surfaced as 404).
- Quote: "// SyncImage returns opaque ErrSyncInternal for hard failures. Soft misses" — S211, func getImageManifest L3363
- Quote: "Hard on-demand upstream failures now return 503 rather than falling through to 404, while genuine upstream misses remain 404." — S239, v2.1.22 > Sync and Multi-Architecture Images
- Conditions/notes: in getImageManifest the sync error is logged ("failed to sync image", L3347-L3350) and the local store is then read; a local hit is returned regardless of the sync error (L3353-L3356). Retries are off unless `maxRetries` and `retryDelay` are set; `maxRetryDelay` (v2.1.19) enables exponential backoff.
- Relevance: copy serve-stale-on-upstream-failure and the 404 versus 503 distinction.

### F130. Zot (Q4): concurrent on-demand requests for the same image are coalesced with singleflight (since v2.1.21); before that fix the same code path admitted unbounded detached work per unique request, a published unauthenticated denial of service.
- Quote: "Concurrent SyncImage/SyncReferrers calls for the same key are deduplicated with singleflight (one upstream sync, shared result)." — S214, type BaseOnDemand doc comment L37-L38
- Quote: "Concurrent on-demand requests for the same image are now coalesced into one upstream sync operation." — S239, v2.1.21 > Sync Throughput and Upstream Check Controls
- Counter: admission was unbounded — "every unique missing repository and reference supplied to an ordinary OCI manifest GET is admitted into an unbounded request map and starts detached work that can survive client disconnection for the three-hour default sync timeout." — S276, Summary (affected 2.1.20, patched v2.1.21)
- Conditions/notes: the flight key is kind + repository + reference; per-upstream-host concurrency defaults to 3 requests (`reqConcurrent`, regclient default) and the request rate is unlimited unless `reqPerSec` is set. Whether v2.1.23 bounds the number of distinct queued on-demand syncs was not verified.
- Relevance: copy single-flight per reference; add an explicit bound on distinct in-flight upstream fetches with a typed refusal, and cancel or budget work whose client has gone.

### F131. Zot (Q4): until v2.1.23 a first pull through the cache waited until the whole image was stored (or, since v2.1.22, got an immediate 404 in background mode); v2.1.23 adds opt-in streaming with a default cap of 32 concurrently streamed blobs and local staging space equal to the blobs in flight.
- Quote: "a client pulling an image that is not yet local gets the upstream manifest right away and its blobs as they arrive from upstream, instead of waiting for the whole image to be synced." — S201, Streaming on-demand sync L1623-L1625
- Quote: "A single 20 GiB layer needs about 20 GiB of staging space." — S230, Streaming on-demand sync > Staging space
- Quote: "Bytes reach clients before zot finishes digest verification, so use a trusted upstream and keep TLS verification enabled." — S230, Streaming on-demand sync
- Quote: "DefaultSyncTimeout = 3 * time.Hour // default timeout for all sync operations (on-demand and periodic)" — S215, const block L20
- Conditions/notes: `maxConcurrentStreams` default 32 (sync constants L26), shared by all streaming registries; past the cap requests fall back to blocking sync; streaming is incompatible with retries, with `onDemandInBackground` and with `tlsVerify: false`; the overall sync timeout defaults to 3 h and continues after the client disconnects. Feature requests for streaming date from 2023 (#1395) and 2026 (#4323, "avoid first-pull timeouts (ImagePullBackOff)").
- Relevance: copy tee-while-fetching; beat it by verifying each chunk against a known digest tree or by withholding the final bytes until the digest matches, and by bounding staging bytes rather than blob count.

### F132. Zot (Q4): upstream credentials come from a JSON credentials file keyed by registry address, from Docker client credentials, or from credential helpers (AWS ECR, GCP, OAuth2 assertion and RFC 8693 token exchange).
- Quote: "Besides sync-auth.json file, zot also reads and uses docker credentials by default: https://docs.docker.com/reference/cli/docker/login/#description" — S201, Sync's credentials L1663
- Quote: "Sync adds OAuth2 JWT assertion and RFC 8693 token-exchange credential helpers for short-lived upstream credentials." — S239, v2.1.19 > Workload Identity and Sync Authentication
- Conditions/notes: the credentials file stores username and password in plain JSON per registry host (S230, credentialsFile); ECR helper since v2.1.3, GCP helper since v2.1.19. Credentials are per upstream, not per requesting user: every client of the mirror pulls with the mirror's upstream identity.
- Relevance: copy short-lived federated upstream credentials; keep static secrets out of the main config.

### F133. Zot (Q4, Q7): mirroring can be restricted to signed images, but an on-demand pull by digest skips the signature check.
- Quote: "true: Synchronize only signed images (either notary or cosign)." — S230, Basic configuration for mirroring with sync > onlySigned
- Quote: "A client that already knows an upstream digest can therefore sync that manifest without a per-manifest signature." — S201, Sync, registries example, comment on onlySigned L1509
- Quote: "// enforceOnlySigned returns zerr.ErrSyncImageNotSigned when OnlySigned is set and tag has no" — S279, func enforceOnlySigned doc comment L1292
- Conditions/notes: the gate tests for the presence of a signature referrer or of a legacy cosign `.sig` tag upstream (service.go L1307-L1340); it does not verify the signature against a trust root at sync time.
- Relevance: if a mirror gates on signatures, verify against configured trust roots and apply the gate to digest pulls as well.

### F134. Zot (Q4, Q13): scale-out clustering (v2.1.0, 2024-07-09) is static sharding: each repository is owned by one member chosen by SipHash of the repository name modulo the member list, and other members proxy to the owner; it is not self-healing and resizing needs a restart.
- Quote: "Each repo is served by one zot instance, and that instance is solely responsible for serving all images of that repo. A repo in storage can be written to only by the zot instance responsible for that repo." — S232, How it works
- Quote: "targetIdx := sum64 % uint64(len(members))" — S213, func ComputeTargetMember L14
- Quote: "The scale-out clustering scheme described in this article is not self-healing when an instance fails. In case of an instance failure, only those repositories that are mapped to the failed instance are affected. If the error is not transient, the cluster must be resized and restarted to exclude that instance." — S232, When an instance fails
- Counter: features that need local state are off in this mode — "CVE scanning is not supported for cloud deployments. In the scale-out clustering scheme described in this article, CVE scanning is disabled." — S232, CVE repository in a zot cluster environment
- Conditions/notes: members are a static list that must be identical and identically ordered on every instance; the hash is plain modulo (not consistent hashing), so changing the member count remaps most repositories; the trust extension is also unsupported; shared S3-compatible storage plus a shared cache (DynamoDB or Redis) is required for compute-only scale-out. The proxy hop header was the subject of two 2026 advisories, one titled "Forged X-Zot-Cluster-Hop-Count header panics the request handler in scale-out mode" (S275).
- Relevance: avoid single-writer-per-repository sharding as the scaling model; any node should serve any repository because coordination lives in the replicated record store.

### F135. Zot (Q4): the documented high-availability recipe is two standalone instances that mirror each other by periodic sync behind a load balancer, with an acknowledged loss window between syncs.
- Quote: "With periodic synchronization, a window of failure exists between synchronization actions. For example, if an image is posted to the active instance soon after the standby has synchronized with the active, and then the active fails, the standby will not have the new image." — S244, Configuring an active/standby registry
- Conditions/notes: replication is pull-based polling only; there is no push-on-write replication, no replication of deletes was found in the docs, and events (F156) are best effort.
- Relevance: beat; synchronous or log-shipped replication with a stated recovery point.

### F136. Zot (Q5): authentication methods are built in: htpasswd, LDAP, bearer tokens from an external token service, OIDC workload identity tokens, OpenID/OAuth2 browser login (Google, GitHub, GitLab, generic OIDC), API keys and mutual TLS; several can be enabled together since v2.1.22.
- Quote: "API keys, OpenID sessions, workload OIDC tokens, and traditional bearer authentication can coexist. zot's token endpoint exchanges locally issued API keys and can forward only unknown credentials to a configured upstream token service." — S239, v2.1.22 > Authentication and Security
- Quote: "zot can now authenticate workloads using OIDC ID tokens, enabling secret-less authentication for automated workflows." — S239, v2.1.14 > Workload Identity Federation
- Quote: "NOTE: When both htpasswd and LDAP configuration are specified, LDAP authentication is given preference." — S201, Passphrase Authentication L456
- Conditions/notes: provider lists in code: OpenID `google, gitlab, oidc`; OAuth2 `github` (config.go L26-L27). Bearer verification keys can come from a certificate file or AWS Secrets Manager (v2.1.15). mTLS identity can be taken from CN, subject, e-mail, URI (SPIFFE pattern) or DNS SAN (v2.1.12). A failed-login delay is configurable (`failDelay`, default 0).
- Relevance: copy the breadth (especially OIDC workload identity and mTLS/SPIFFE identity mapping) behind one typed identity model.

### F137. Zot (Q5): authorization is a static policy in the config file: per repository glob, lists of users or groups with actions read, create, update, delete; the longest matching pattern wins; default, anonymous and admin policies; optional CEL conditions (v2.1.17) and per-user namespaces (v2.1.23).
- Quote: "The access control policy is a function of repository, user, and the action being performed on that repository." — S233, Authorization
- Quote: "When a repository path matches more than one path description, authorization is granted based on the policy of the longest (most specific) path matched." — S233, Authorization > Configuring access control
- Quote: "An anonymous policy specifies what an unauthenticated user is allowed to do. This is an appropriate policy when you want to grant open read-only access to one or more repositories." — S233, Authorization > Access control policies
- Quote: "Authorization policies can now use conditional expressions through CEL (Common Expression Language), enabling more context-aware access control decisions." — S239, v2.1.17 > Conditional Authorization via CEL
- Conditions/notes: there are no stored roles or projects; groups are defined in the config or come from LDAP/OIDC claims/GitHub teams; `create`, `update` and `delete` each require `read`; a behaviour action `detectManifestCollision` refuses delete-by-digest when several tags share the digest. Bearer (token-service) requests are authorized by token scopes, not by this policy (S233, Admin-only routes). The policy is hot-reloaded with the config file.
- Relevance: copy longest-match repository policies, anonymous policy and CEL conditions; store policy in the record store with an API rather than only in a file.

### F138. Zot (Q5): there are no robot or service accounts; the substitutes are API keys, which carry the full permissions of the user who created them, and OIDC workload identity.
- Quote: "An API key has the same permissions as the user who generated it." — S201, API keys L776
- Conditions/notes: API keys have the prefix `zak_`, an optional `expirationDate` and a `scopes` field in the creation request (S201, How to create an API Key); no default expiry. A search of the v2.1.23 docs tree and examples for "robot" found nothing.
- Relevance: beat; first-class machine identities scoped per repository and action, with mandatory expiry.

### F139. Zot (Q5): tag immutability is not a setting; it is obtained by withholding the update action in the access policy.
- Quote: "While zot does not have an explicit configuration flag to make image tags immutable, the same effect can be achieved with authorization as follows." — S245, introduction
- Conditions/notes: page dated 2026-11-19. A feature request for protected tags (#4623) was closed on 2026-10-28 (per GitHub issue search; not read), so this may change after v2.1.23.
- Relevance: provide immutability as a per-repository, per-tag-pattern rule enforced in the manifest write path.

### F140. Zot (Q5, Q8): store-wide dedupe has twice been an authorization bypass: a blob known by digest could be linked into a repository the caller may read from a repository the caller may not (CVE-2024-39897, fixed in v2.1.0; incomplete fix patched in v2.1.21).
- Quote: "an attacker who knows the name of an image and the digest of a blob (that they do not have read access to), they may maliciously read it via a second repository they do have read access to." — S278, Details
- Quote: "CVE-2024-39897 added canMount() to require read access to a source repository before reusing a deduplicated blob. That requirement is still bypassed for traditional bearer requests." — S277, Summary
- Conditions/notes: the first advisory's mitigation was to set `"dedupe": false`. Since v2.1.21 blob HEAD and GET no longer consult the dedupe cache unless `hydrateBlobOnRead` is set (F148).
- Relevance: avoid; global content addressing must never be an access path. A blob becomes readable in a repository only through an authorized reference recorded for that repository.

### F141. Zot (Q6): search is a GraphQL API at /v2/_zot/ext/search over the metadata database (images by digest, by CVE, base and derived images, referrers, global text search), with request-size, token, complexity and cache limits that are each justified in code comments.
- Quote: "GraphQL queries are sent to the zot search extension API: /v2/_zot/ext/search" — S238, How to use GraphQL for search queries > Using the search API directly
- Quote: "Searches all repositories in the registry and returns list of images that matches given digest (manifest, config or layers)" — S238, What GraphQL queries are supported
- Quote: "The richest known real query costs on the order of 30; 500 leaves ample headroom." — S217, const searchComplexityLimit comment L41
- Conditions/notes: limits at v2.1.23: body 1 MiB (consts.go L37), parser tokens 2000, complexity 500, query cache 64 entries (extension_search.go L31-L50). They follow advisory GHSA-prmc-5wf3-9593, "Unbounded GraphQL document fanout permits unauthenticated CPU and allocation exhaustion" (S275, published 2026-09-11). Without the search extension only the distribution-spec catalog and tag list exist.
- Relevance: copy the idea of stating the measured real maximum next to each limit; decide on a bounded query surface before exposing a general query language.

### F142. Zot (Q6): CVE scanning embeds Trivy as a library: scans run in the background every 15 minutes (hardcoded) and on demand, the vulnerability database is refreshed at an interval of at least 2 hours, and the feature is unavailable on object storage and in scale-out clusters.
- Quote: "Images are automatically scanned in the background by a scheduler task that runs every 15 minutes (hardcoded, not configurable)." — S237, How It Works > Architecture
- Quote: "Scanning does not occur on image push; it happens asynchronously via the scheduler." — S237, How It Works > Architecture
- Quote: "The minimum database updateInterval is two hours." — S201, Search and CVE scanning (Trivy) L1680
- Quote: "UpdateInterval time.Duration // should be 2 hours or more, if not specified default be kept as 2 hours" — S226, type CVEConfig L57
- Quote: "CVE scanning is not supported for cloud deployments or scale-out clustering scenarios." — S237, limitations list, Scale-Out Clustering
- Conditions/notes: scan results are cached by manifest digest and purged when the database is refreshed; layers are unpacked under /tmp; SBOM generation during scans was added in v2.1.18 (spdx-json default, cyclonedx). Config validation rejects CVE scanning with a remote storage driver ("failed to enable cve scanning due to incompatibility with remote storage", root.go L698-L699). Scale-out limitation also quoted in F134. A scan-completed event (`image.scanned`) exists since v2.1.19.
- Relevance: keep scanning out of the registry process (it brought a 221 MB binary and memory spikes, F160, F161); expose scan results as referrers and events instead.

### F143. Zot (Q6): a web UI (zui), per-user stars and bookmarks, and a CLI (zli) are part of the full build since v2.0.0.
- Quote: "Using the new zot GUI, you can browse a zot registry for container images and artifacts." — S239, v2.0.0 > Built-in UI support
- Conditions/notes: UI needs the search, mgmt and userprefs extensions (Makefile L88); the UI is not supported in the "compute and storage scale-out" layout (S232). Browser sessions expire after 2 hours (F157).
- Relevance: a UI can be a separate client of the same public API; do not make core features depend on UI-only state.

### F144. Zot (Q7): the trust extension verifies cosign and notation signatures against uploaded keys and certificates and reports the result through search; it does not block a pull or a push of an unsigned or untrusted image.
- Quote: "The verification result for each signed image is stored in the database and is visible from GraphQL." — S235, How signature verification works
- Counter: users ask for enforcement, which the extension does not provide — "@rchincha but that one talks about signature verification, and I am wondering about enforcement." — S263, reporter's comment, 2026-05-26
- Conditions/notes: cosign public keys and notation certificates are uploaded to `/v2/_zot/ext/cosign` and `/v2/_zot/ext/notation` (admin-only since v2.1.22) and stored under the storage root or in AWS Secrets Manager; cosign v3 bundle format is recognized since v2.1.21; the extension is unsupported in scale-out clusters (S232). At v2.1.23 the lint extension can deny a push only for missing annotations (`mandatoryAnnotations`; extensions config.go L44-L48); a pull-time or push-time signature gate was not found in the v2.1.23 tree.
- Relevance: beat; offer policy enforcement (reject unsigned at push, refuse at pull) evaluated against trust roots held in the record store.

### F145. Zot (Q7): signatures, attestations and SBOMs are stored as ordinary OCI artifacts found through the referrers API; zot can generate SBOMs itself during Trivy scans (v2.1.18).
- Quote: "Cosign attestations are OCI referrers, not image signatures. An attestation by itself does not cause zot to report an image as signed." — S235, Signatures, attestations, and metadata storage
- Quote: "zot now supports generating SBOM artifacts during Trivy CVE scans and publishing them as OCI referrers for scanned images." — S239, v2.1.18 > Trivy-Based SBOM Artifact Generation
- Conditions/notes: legacy cosign tags (`sha256-<hex>.sig`, `.sbom`) are still recognized by GC and sync; until v2.1.22 signature payloads were copied into the metadata database, which inflated BoltDB files (S239, v2.1.22 note).
- Relevance: copy referrers-first handling; keep artifact payloads out of the metadata records.

### F146. Zot (Q8): on disk each repository is a directory holding an OCI image layout: index.json (tags are ref.name annotations on its descriptors), blobs/<algorithm>/<hex>, and a .uploads directory for sessions; nested repository names are nested directories.
- Quote: "return path.Join(is.rootDir, repo, ispec.ImageBlobsDir, digest.Algorithm().String(), digest.Encoded())" — S207, func BlobPath L2047
- Quote: "v, ok := manifest.Annotations[ispec.AnnotationRefName]" — S228, func GetTagsByIndex L41
- Quote: "A nested repository whose name starts with <x>/blobs/ is stored inside the blob directory of repository <x>." — S268, Summary
- Conditions/notes: `BlobUploadDir = ".uploads"` (storage constants L9); path limits 255 per component, 4095 for a local path, 1024 for an object key (imagestore.go L44-L46). Any directory that is a valid OCI layout can be served as a repository (S229, Hosting an OCI image layout). Every manifest read loads and parses the whole index.json and scans it linearly (F162).
- Relevance: copy the idea that exported data is a standard OCI layout; avoid using one growing JSON file as the tag index and avoid mapping repository names onto directory trees.

### F147. Zot (Q8): deduplication is on by default: on a local filesystem every copy of a blob is a hard link to one inode; on object storage one copy holds the bytes and every other repository gets a zero-byte placeholder object that is resolved through the cache database.
- Quote: "real directories, and dedupe makes each copy of a blob a hardlink to the same inode, so" — S208, type blobLifecycle doc comment L26
- Quote: "Remote storage keeps no empty directories, and dedupe leaves a zero-byte placeholder at each copy except the one holding the content." — S208, type blobLifecycle doc comment L27-L28
- Quote: "Upon startup, zot enforces the dedupe status on the existing storage. If the dedupe status upon startup is true, zot deduplicates all blobs found in storage, both local and remote." — S229, Storage features > Deduplication
- Counter: toggling dedupe triggers a store-wide rewrite under the store lock — "It takes up to 30 minites to push new tag with all image layers existing in registry" — S259, Describe the bug (v2.1.3-rc2, S3-compatible storage, dedupe false after true)
- Conditions/notes: deleting the content-holding copy requires moving the bytes to another placeholder ("HandOffContent"); a cache miss means a placeholder cannot be resolved (F110); races in this scheme are listed in F120 and F121. #2999 is the most-commented issue in the tracker (53 comments per GitHub issue search on 2026-11-20).
- Relevance: avoid per-repository copies and placeholders; a content-addressed blob store with reference records gives dedupe by construction, with no startup rewrite and no hand-off.

### F148. Zot (Q8): a cross-repository mount is answered from the store-wide dedupe cache by digest and falls back to a normal upload session; since v2.1.21 it needs read access to a source repository, and plain blob reads no longer pull content across repositories.
- Quote: "// CheckBlob looks for the digest under dest first, then the dedupe cache; on a" — S211, func CreateBlobUpload L2153
- Quote: "Cross-repository dedupe mounts now require read access to a source repository that contains the blob, as well as create access to the destination repository." — S239, v2.1.21 > Storage and Distribution Behavior
- Quote: "Beginning with zot v2.1.21, blob reads are repository-local by default. Even when the same digest exists in another repository and is present in the dedupe cache, a HEAD or GET request returns 404 until the blob is mounted or uploaded into the requested repository." — S229, Hydrating deduplicated blobs on read
- Conditions/notes: a successful mount returns 201, otherwise 202 with a new session (routes.go L2156-L2192); a `from` parameter without `mount` returns 405 (L2195-L2199). The mount works only within one store (not across subPaths) and only when dedupe is enabled; in the published conformance run (dedupe off) anonymous mount was skipped (F112).
- Relevance: copy mount-by-digest with a source-read check; make it independent of a dedupe option.

### F149. Zot (Q8): durability on local disk is opt-in: by default writes are left to the page cache; the commit option forces them to disk, and only since v2.1.23 does it also sync the directory after a rename.
- Quote: "The zot storage configuration exposes an option called commit which, when enabled, causes data writes to be committed to disk immediately. This option is disabled by default." — S229, Storage features > Commit
- Quote: "Beginning with v2.1.23, committed local-file renames also sync the containing directory so the renamed entry, not just its contents, is durable across a crash." — S229, Storage features > Commit
- Conditions/notes: the stated reason for the option is RAM pressure on embedded devices, not crash safety. v2.1.16 fixed index.json corruption when the disk fills during a repository update (S239, v2.1.16 > Storage and Authentication Fixes).
- Relevance: beat; acknowledge a push only after the bytes and the reference are durable, always.

### F150. Zot (Q8): a scrub extension re-hashes stored blobs in the background (default every 24 hours, minimum 2 hours) to report bit rot.
- Quote: "The scrub function, available as an extension, makes it possible to ascertain data validity by computing hashes on blobs periodically and continuously so that any bit rot is caught and reported early." — S229, Storage features > Scrub
- Quote: "The time interval between periodic scrub operations. This value must be at least two hours (2h)." — S246, Scrubbing the image registry > interval
- Conditions/notes: default 24 h set in root.go L1146-L1148; scrub reports, it does not repair. A user traced a daily memory spike and 100-200% CPU at restart to scrub (F161).
- Relevance: copy background integrity verification; budget it (bytes per second) and repair from redundancy when the store is erasure coded.

### F151. Zot (Q9): a manifest PUT body is limited to 4 MiB (since v2.1.16, 2026-04-19) and the constant carries its reason; the number of tag= parameters on a digest push is limited to 46, derived in a comment from an 8 KiB request-target budget.
- Quote: "// OCI manifest JSON is always small metadata; 4 MiB is well above any realistic manifest." — S203, const MaxManifestBodySize comment L24
- Quote: "MaxManifestBodySize = 4 * 1024 * 1024" — S203, const block L25
- Quote: "MaxManifestDigestQueryTags = (8192 - 2048) / (len("tag=") + 128 + 1)" — S203, const block L22
- Conditions/notes: enforced with `http.MaxBytesReader`; over-size returns 413 (routes.go L858-L866); too many tag parameters returns 414 (L829-L833). Before v2.1.16 the manifest body was unbounded ("Limited manifest PUT request bodies to 4 MiB" is listed under Security Hardening for v2.1.16, S239). Other body caps in the same block: API-key request 8 KiB, image-trust upload 8 MiB, token request 64 KiB, GraphQL request 1 MiB.
- Relevance: copy; 4 MiB matches the distribution-spec's "at least 4 megabytes" guidance, and writing the derivation next to the constant is the practice to adopt.

### F152. Zot (Q9, Q11): since v2.1.17 the HTTP server has 60-second read and write timeouts by default, and they are absolute deadlines for the whole request body and the whole response, so any single blob upload or download slower than 60 seconds is cut off unless the operator raises them.
- Quote: "defaultReadTimeout = 60 * time.Second defaultWriteTimeout = 60 * time.Second defaultShutdownTimeout = 30 * time.Second" — S210, const block L41-L43
- Quote: "a streamed blob is sent within one response, so http.writeTimeout (default 60s, an absolute deadline for writing the whole response) must cover the slowest blob pull expected, or the client is cut off mid-blob" — S201, Streaming on-demand sync, notes L1647-L1648
- Counter: the default broke large pushes for users — "When trying to upload an image with a large layer, I was trying to upload a layer that was 1,5GB after 60 seconds the upload is aborted and retried." — S264, Describe the bug (v2.1.17)
- Counter: and it was reported again five months later — "When uploading a large OCI artifact that takes longer than Zot's default 60-second timeout, Zot appears to cancel the upload once that limit is reached rather than treating it as an error-related timeout." — S265, Describe the bug (v2.1.22)
- Conditions/notes: fixed constants that are not configurable: idle timeout 120 s and read-header timeout 5 s (controller.go L39-L40). The stated reason for the timeouts is to stop slow or stalled clients from holding resources (S240, HTTP read/write timeout configuration); no reason is given for 60 s. No per-request deadline extension was found for blob routes in the v2.1.23 tree. The bundled benchmark config raises both to 2 m (examples/config-bench.json).
- Relevance: avoid whole-transfer deadlines; use idle (no-progress) timeouts and minimum-throughput floors for blob bodies, with typed refusals.

### F153. Zot (Q9): request rate limiting is off by default; when configured it is a requests-per-second cap with optional per-HTTP-method caps, which the docs call global but the library applies per client IP and URL path; repository-count and per-repository byte quotas exist since v2.1.16 and v2.1.23, answer 413, and are logical, not physical.
- Quote: "In this example, the rate attribute directly under ratelimit sets a global rate limit of ten API calls per second." — S246, Security and hardening > Rate limiting
- Quote: "A push that would increase usage beyond the limit returns 413 with a DENIED error whose detail contains current, projected, and limit as byte-count strings." — S229, Repository quotas
- Quote: "Independent writers are not atomically coordinated by this quota implementation." — S229, Repository quotas
- Counter: the limiter is not global: zot passes the configured rate to tollbooth's default limiter, whose key starts with the client IP (taken first from a client-supplied header) and the URL path — "SetIPLookups([]string{"X-Forwarded-For", "X-Real-IP", "RemoteAddr"})" — S280, func NewLimiter L41
- Counter: the release notes for v2.1.16 state a different status code for the repository-count limit than the current docs and code (413) — "When the limit is reached, pushes that create a new repository are rejected (HTTP 429), while pushes to existing repositories continue to work." — S239, v2.1.16 > Repository Quota Enforcement
- Conditions/notes: zot calls `tollbooth.NewLimiter(float64(rate), nil)` and `tollbooth.LimitHandler` as middleware (pkg/api/session.go L44-L70; go.mod pins tollbooth v7.0.2); tollbooth builds the bucket key from remote IP, then path, then methods (tollbooth.go func BuildKeys L269-L286). Because X-Forwarded-For is consulted first, a client that reaches zot directly can vary that header to get a fresh bucket. The limit is not per user or per repository. Excess requests get 429. `maxRepoBytes` counts unique digests reachable from tagged references using descriptor sizes; untagged content and referrers are not counted; the check runs at manifest push, so blobs are already stored when the push is refused. Quota code answers `http.StatusRequestEntityTooLarge` (pkg/api/quota.go L161).
- Relevance: copy quotas enforced before bytes are accepted where possible; make limits per principal and per repository, transactional across writers, and use 429 for rate and a distinct typed error for quota.

### F154. Zot (Q9): the catalog and tag-list endpoints have no default or maximum page size: without n the whole list is returned, and the tag list is read from index.json and sorted on every request.
- Quote: "// Tags need to be sorted regardless of pagination parameters" — S211, func ListTags L401
- Quote: "Catalog and tag pagination continue correctly if the last cursor was deleted, and large tag-list page sizes no longer overflow." — S239, v2.1.22 > Runtime, API, and Compatibility Improvements
- Conditions/notes: `numTags := -1` when `n` is absent (routes.go L335) and `maxEntries = -1` for the catalog (L2841-L2844). The only hard page cap found is for CVE lists: `maxCvePageLimit = 4 * 1024` (pkg/extensions/search/cve/pagination.go L100), without a stated reason. Tag length is capped at 128 characters as the spec requires (pkg/regexp/regexp.go L11).
- Relevance: beat; serve tag pages from an ordered index in the record store with a bounded default and maximum page size.

### F155. Zot (Q9): background work (GC, sync, scrub, scans, dedupe) runs on one scheduler with NumCPU x 4 workers by default, three priority queues of 400 tasks and a 50 ms dispatch tick; none of the three numbers has a stated reason.
- Quote: "The scheduler is by default allowed to simultaneously run a maximum number of tasks equal to four times the number of CPUs available to the zot process." — S246, Scheduling background tasks
- Quote: "rateLimiterScheduler = 400 rateLimit = 50 * time.Millisecond NumWorkersMultiplier = 4" — S212, const block L63-L65
- Counter: the scheduler has stalled in production — "Zot's internal scheduler consistently enters a deadlock state where a medium-priority generator gets stuck in "waiting" state indefinitely, while all 64 workers remain idle." — S269, Description (v2.1.13)
- Conditions/notes: `scheduler.numWorkers` overrides the worker count; tasks are in-memory only (nothing is persisted across restarts); #3723 was reported on Kubernetes with 1 CPU limit and 1Gi memory limit and is closed.
- Relevance: copy priority classes; make every queue bound and its overflow behaviour explicit, and export per-generator liveness so a stuck generator is visible.

### F156. Zot (Q4, Q13): event sinks (v2.1.3, 2025-05-22) publish CloudEvents to HTTP or NATS; delivery is fire-and-forget from a goroutine per event, with a failed delivery logged and dropped.
- Quote: "Registry-significant events can be generated and pushed to either HTTP or NATS endpoints." — S236, introduction
- Quote: "Sink delivery timeout duration. HTTP sinks default to 30s when this value is omitted or non-positive." — S236, Sink attributes > timeout
- Quote: "// publish dispatches asynchronously, so wait before tearing sinks down" — S218, type eventRecorder L22
- Conditions/notes: `publish` starts one goroutine per event and logs "failed to publish event" on a NACK or undelivered result, then continues (events.go L49-L81); no retry, queue, ordering guarantee or persistence was found. Event types include repository created, image updated, image deleted, image lint failed and image scanned (v2.1.19). Sinks can be changed by config reload since v2.1.21.
- Relevance: beat; events that drive replication or audit need an at-least-once outbox in the record store with a bounded backlog and a typed overflow.

### F157. Zot (Q9): there is no separate upload-session lifetime: an abandoned upload is removed by GC once its modification time is older than gcDelay (1 hour by default), and only if GC is enabled; no minimum chunk size is advertised; browser sessions last 2 hours.
- Quote: "Uploads younger than the GC delay are kept, so RemoveIdleRepository below still sees in-progress sessions." — S205, func cleanRepoLocked comment L248-L249
- Quote: "const cookiesMaxAge = 7200 // 2h" — S219, L26
- Quote: "The S3 API requires multipart upload chunks to be at least 5MB. This value should be a number that is larger than 5 * 1024 * 1024." — S229, Configuring remote storage with s3 > chunksize
- Conditions/notes: the string `OCI-Chunk-Min-Length` does not occur in the v2.1.23 tree (searched pkg/). Upload session ids are limited to 255 characters (pkg/storage/imagestore/upload_session.go L12). A stale upload that cannot be deleted on S3 blocks GC of its repository (F123). API keys have no default expiry (F138).
- Relevance: give upload sessions their own stated expiry and bound, independent of GC, and advertise the chunk minimum the blob store needs.

### F158. Zot (Q10): the only throughput figures zot publishes are one sample zb run in its docs and README: 2202.7788 requests/s for Get Catalog and 105.00491 requests/s for 1 MB monolithic pushes, at concurrency 2 over 100 requests against localhost, with hardware and zot version not stated.
- Quote: "Test name: Get Catalog Time taken for tests: 45.397205ms Complete requests: 100 Failed requests: 0 Requests per second: 2202.7788" — S234, Running zb > Example > Command output
- Quote: "Test name: Push Monolith 1MB Time taken for tests: 952.336383ms Complete requests: 100 Failed requests: 0 Requests per second: 105.00491" — S234, Running zb > Example > Command output
- Quote: "min: 402.259µs max: 3.295887ms p50: 855.045µs p75: 971.709µs p90: 1.127389ms p99: 3.295887ms" — S234, Running zb > Example > Command output (Get Catalog latencies)
- Conditions/notes: header of the same output: "Registry URL: http://localhost:8080", "Concurrency Level: 2", "Total requests: 100". Push Monolith 1MB latencies in the sample: min 11.125673ms, p50 18.917253ms, p99 26.375356ms. The command shown above the output uses -c 10 -n 1000, which does not match the output's concurrency 2 and 100 requests. The same output is in cmd/zb/README.md at v2.1.23 (S222). Page dated 2025-12-04. It is an illustration of the tool's output, not a recorded benchmark. No comparison against other registries is published by the project.
- Relevance: beat with recorded runs (hardware, commit, load, best of N); zb itself is a reusable workload generator for class-for-class comparisons because it targets any distribution-spec registry.

### F159. Zot (Q10): zb's workload set and zot's CI benchmark are defined in the repository: 18 tests (catalog, monolithic and chunked pushes of 1 MB, 10 MB and 100 MB, pulls, mixed, on-demand sync of 100 MB and 1 GB), run in CI as zb -c 10 -n 100 on a runner labelled cncf-ubuntu-8-32-x86 with a regression alert; the results are not published.
- Quote: "bin/zb-$(OS)-$(ARCH) -c 10 -n 100 -o $(BENCH_OUTPUT) --max-timeout-failures 1 http://localhost:8080" — S221, target run-bench L289
- Quote: "# Cache restore unpacks ~2GB onto disk; settle before the short bench so" — S220, job benchmark, step comment L24
- Quote: "Pull Mixed 20% 1MB, 70% 10MB, 10% 100MB Push Monolith Mixed 20% 1MB, 70% 10MB, 10% 100MB Push Chunk Mixed 33% 1MB, 33% 10MB, 33% 100MB" — S222, List tests
- Conditions/notes: runner label `cncf-ubuntu-8-32-x86` (benchmark.yaml L15; the label suggests 8 vCPU and 32 GB, the workflow does not state the hardware); server config examples/config-bench.json (local filesystem under /tmp/zot, read and write timeouts 2m, log output /dev/null); results go through github-action-benchmark with `fail-on-alert: true` into an Actions cache file, not to a public page (https://project-zot.github.io/zot/dev/bench/ answered 404 on 2026-11-20). zb options: concurrency default 1, requests default 1, source-IP spreading, JSON and ci-cd output.
- Relevance: copy a benchmark in the gate with a regression alert; record and publish its numbers.

### F160. Zot (Q10): binary size at v2.1.23: the full linux/amd64 server is 221 MB and the minimal build 86.6 MB; a CI check fails a pull request that grows the minimal binary by 1% or more.
- Quote: "zot-linux-amd64 sha256:1f6bab0b9eee426d0785de701ff510df5c02bdb8d7287494a582634abfc5d638 221 MB 2026-11-02T01:54:12Z" — S249, asset list
- Quote: "zot-linux-amd64-minimal sha256:8a6e77686bc0e7751766c49828cff6c66b115c854d350c9d936b855463f6c973 86.6 MB 2026-11-02T01:53:53Z" — S249, asset list
- Quote: "- name: Check if zot-minimal binary increased with more than 1%" — S225, job compare-binary-sizes L25
- Conditions/notes: sizes as listed on the GitHub release (stripped, PIE builds): linux/arm64 206 MB full and 81 MB minimal; darwin/arm64 192 MB and 75.1 MB; windows/amd64 205 MB and 79.9 MB; freebsd/amd64 200 MB and 78.1 MB; debug build linux/amd64 326 MB; zli 13.9 MB, zb 8.93 MB, zxp 16.8 MB (linux/amd64). The full build links the Trivy, cosign and notation libraries and the cloud SDKs listed in go.mod.
- Relevance: beat; a static registry binary one order of magnitude smaller is a measurable differentiator.

### F161. Zot (Q10): the project publishes no idle memory or CPU figures; the data points found are a user report (under 70 MB and about 0.05% CPU idle with scrub disabled, about 15.5 GB at start with scrub enabled, v2.0.0-rc7) and the docs' example systemd unit, which caps memory at 32G.
- Quote: "Not only causes it 100-200% CPU usage when restarting Zot, after disabling it the mem usage goes down to <70MB ... and the CPU usage is at ~0.05%, if anything." — S261, reporter's comment, 2023-12-13
- Quote: "The container starts up with ~15.5 GB mem usage right away, even though it's completely idle." — S261, reporter's comment, 2023-12-13
- Quote: "LimitNOFILE=500000 MemoryHigh=30G MemoryMax=32G" — S248, Step 4: Define the zot service (example unit)
- Quote: "Under some configurations, zot consumes significant CPU and memory resources. This has been fixed in this release." — S239, v2.0.1 > Bugfixes and performance improvements
- Conditions/notes: the report's machine: AMD Ryzen Threadripper 2950X, 128 GB RAM, container-level metrics, near-idle registry with search, CVE scanning, scrub and UI enabled; number of images not stated. In the same thread a maintainer suggested `GOMEMLIMIT=4000MiB` and `GOGC=50`. The unit-file numbers are an example without a stated basis. Idle RSS and CPU published by the project: not found.
- Relevance: publish idle and loaded RSS per feature; keep scrub and scanning memory bounded by budget rather than by Go GC tuning.

### F162. Zot (Q10): known performance problems documented by the project and its users: every manifest read loads and scans the repository's whole index.json under the store lock; a restart walks all of storage to rebuild metadata (fastRestart added in v2.1.18 to skip it); the DynamoDB backend failed beyond 100 repositories in the UI (2024); blob bytes are proxied through zot unless redirectBlobURL (v2.1.18) is enabled.
- Quote: "for _, manifest := range index.Manifests { if reference == manifest.Digest.String() { return manifest, true" — S228, func GetManifestDescByReference L53-L55
- Quote: "On large registries (for example a 1TB+ S3 backend with many repos), the startup walk that reconciles metaDB with the current storage can dominate restart time." — S201, Fast restart L247-L249
- Quote: "If there are more than 100 repositories the UI fails to list them on the start/explore page when using DynamoDB as database." — S271, Describe the bug (v2.1.0)
- Quote: "blob pull requests can be redirected (HTTP 307) to signed backend URLs from the configured storage driver, reducing proxy traffic through zot." — S229, Configurable attributes > redirectBlobURL
- Conditions/notes: GetImageManifest (imagestore.go L764-L809) takes the store read lock, reads index.json from the driver (an object GET on S3), unmarshals it, scans linearly, then reads the manifest blob; PutImageManifest rewrites index.json under the write lock, so push cost grows with the number of manifests in the repository. The default log level is "debug" (config.go L895). fastRestart trades the walk for not detecting out-of-band storage changes. Startup durations at a stated size: not found.
- Relevance: beat; O(1) or O(log n) manifest and tag lookups from the record store, no per-repository index file, no restart walk, and redirect or direct blob serving by default.

### F163. Zot (Q11): the most-commented operational issues are about remote dedupe (stuck pushes, lost content), the store-wide lock, GC not freeing or silently stopping, pull-through cache behaviour (re-pulls, rate limits, first-pull timeouts), the 60-second timeouts, and overwritten tags becoming unreachable by digest.
- Quote: "Push to registry is blocked because of deduplication process runing with storage locking" — S259, Describe the bug (v2.1.3-rc2)
- Quote: "We've identified a significant performance bottleneck in the image store layer due to a global lock." — S258, Describe the bug (v2.1.2)
- Quote: "Overwriting the last tag referencing a manifest makes that manifest immediately inaccessible by digest in the same repository." — S273, Describe the bug (v2.1.21)
- Conditions/notes: comment counts from the GitHub issue search API on 2026-11-20 (981 issues in total): #2999 push stuck restoring deduped blobs 53; #724 Docker media types 51; #2928 anonymous policy and docker pull 41; #2836 on-demand sync of OCI index 34; #2910 S3 eventual consistency 31; #3869 only anonymous images synced 29 (open); #2584 cache with tag@digest 27; #2964 store lock 20 (open); #3396 GC frees nothing without retention 16; #4323 stream on-demand pulls 16; #3129 pull-through re-pulls 14; #2137 memory 14; #4184 on-demand pull fails after long wait 14; #2627 DynamoDB listing 12; #4695 GC broken by nested repository name 12 (open); #4079 60-second upload abort 10. #4444 was fixed in v2.1.22 (an overwritten tag's previous digest stays pullable until retention and GC remove it, S239). Upgrade breakage noted in release notes: LDAP credentials moved to a separate file (v2.0.0), access-control syntax change (v2.0.0), blob reads became repository-local (v2.1.21), admin routes need an admin policy (v2.1.22), avoid downgrading after v2.1.22 rewrote metadata. A database migration step was not found; the metadata database is patched or rebuilt at startup.
- Relevance: the recurring root causes are (1) dedupe by placeholder, (2) one lock, (3) time-based GC safety, (4) blocking pull-through; a design that removes all four removes most of this tracker.

### F164. Zot (Q12): release binaries are built for linux, darwin, freebsd and windows on amd64 and arm64; container images are published for linux and freebsd; the docs' support statement still names only Linux and macOS (its table adds FreeBSD), and the S3 driver must not be used on Windows.
- Quote: "os: [linux, darwin, freebsd, windows] arch: [amd64, arm64]" — S224, job build, strategy matrix L24-L25
- Quote: "Native Windows binaries (both amd64 and arm64) are now distributed with the release." — S239, v2.1.3 > Windows Binaries
- Quote: "zot is supported on Linux and Apple MacOS platforms with Intel or ARM processors." — S241, Supported platforms
- Quote: "All images support multiple architectures (linux/amd64, linux/arm64, freebsd/amd64, freebsd/arm64) and Docker/Podman will automatically pull the correct architecture for your platform." — S241, Getting container images
- Counter: Windows support is partial — "Do not use the S3 storage driver when running zot on Windows." — S229, Configuring remote storage with s3 (warning)
- Conditions/notes: the v2.1.23 asset list contains zot, zot-minimal, zot-debug, zli, zb and zxp for all eight OS/arch pairs (S249). FIPS 140-3 mode via `GODEBUG="fips140=only"` is documented for Linux (v2.1.9). Other architectures (ppc64le, s390x, riscv64): no release assets.
- Relevance: match linux, macOS and Windows on x86_64 and aarch64 with every backend working on every platform, or a typed refusal naming the platform.

### F165. Zot (Q13): the newest release is v2.1.23, published 2026-11-02; releases were roughly monthly through 2026 (v2.1.14 on 2026-01-25 to v2.1.23), all on the v2.1 line, which began with v2.1.0 on 2024-07-09.
- Quote: "released this <relative-time class="no-wrap" prefix="" datetime="2026-11-02T01:51:26Z"> 02 Nov 01:51 </relative-time>" — S250, release header (page HTML)
- Quote: "released this <relative-time class="no-wrap" prefix="" datetime="2024-07-09T17:39:59Z"> 09 Jul 17:39 </relative-time>" — S251, release header (page HTML)
- Quote: "This release primarily restores the downloadable binaries that were not published with v2.1.19." — S239, v2.1.20
- Conditions/notes: dates from the GitHub release feed fetched 2026-11-20: v2.1.14 2026-01-25, v2.1.15 2026-03-08, v2.1.16 2026-04-19, v2.1.17 2026-05-18, v2.1.18 2026-06-24, v2.1.19 and v2.1.20 2026-08-04, v2.1.21 2026-09-06, v2.1.22 2026-10-06, v2.1.23 2026-11-02. v2.0.0 was released 2023-12-18 and v2.1.3 on 2025-05-22 (S253, S252). The tag v2.1.23 is commit d7915e82e6693255338ef7fa9bb15300c82db020. Patch releases carry features and behaviour changes ("Upgrade impact" notes appear in v2.1.21, v2.1.22 and v2.1.23).
- Relevance: track the monthly line; patch-level releases change defaults, so pin and read the notes.

### F166. Zot (Q13): what changed in 2025 and 2026: events, Redis driver and Windows binaries (v2.1.3); GCS (v2.1.15); repository quotas and multi-tag push (v2.1.16); CEL conditions and HTTP timeouts (v2.1.17); Azure, fast restart, blob redirects and SBOM generation (v2.1.18); GC time window and keepUntagged (v2.1.19); coalesced on-demand sync and repository-local blob reads (v2.1.21); storage error classes and digest-preserving sync (v2.1.22); streaming pull-through and byte quotas (v2.1.23).
- Quote: "zot can now generate registry-significant events that can be published to http or nats endpoints." — S239, v2.1.3 > Event generation
- Quote: "zot now supports Google Cloud Storage (GCS) as a storage backend, expanding the available storage options alongside existing S3-compatible storage solutions." — S239, v2.1.15 > GCS Storage Support
- Quote: "zot now supports pushing multiple tags for a single manifest, improving tag management workflows and reducing duplicate push operations." — S239, v2.1.16 > Multi-tag Push Support
- Quote: "Added optional storage.fastRestart to reduce startup time by skipping storage walk when safe" — S239, v2.1.18 > Storage: Azure Backend, Fast Restart, and Blob Redirects
- Quote: "Storage drivers now distinguish missing content from transient failures and permanent errors across local, S3, GCS, and Azure storage." — S239, v2.1.22 > Storage Outage and Data-Integrity Handling
- Quote: "Streaming sync: opt-in stream serves blobs while downloading from a verified HTTPS upstream." — S239, v2.1.23 > Highlights
- Conditions/notes: also in this period: zot usable as a Go library (v2.1.10), FIPS 140-3 mode and default removal of untagged manifests (v2.1.9), liveness/readiness endpoints (v2.1.6), OIDC workload identity (v2.1.14), TLS certificate reload (v2.1.15), `zot schema` JSON schema export (v2.1.16), cosign v3 bundles and systemd socket activation (v2.1.21), `zot healthcheck` and HSTS (v2.1.22). Scale-out clustering is older (v2.1.0, 2024-07-09) and retention policies older still (v2.0.0, 2023-12-18). Security advisories published in 2026 include unbounded on-demand sync work, unbounded GraphQL fanout, two cluster hop-header denial-of-service reports, cross-repository mount bypasses and fail-open admin guards (S275).
- Relevance: the 2026 work is mostly hardening of storage error handling, dedupe, GC and bounds; those are the areas a new design must get right at the start.

### F167. Zot (Q13): a v2.2.0 milestone is open with no due date (8 of 11 issues closed on 2026-11-20); no v3 plan was found; zot is a CNCF Sandbox project.
- Quote: "v2.2.0 No due date •8 / 11 issues closed 72 % complete3 open8 closed" — S254, Open milestones
- Quote: "CNCF project accepted as a Sandbox Project" — S242, Why zot? (table)
- Conditions/notes: the other open milestone is v2.1.24 (6 of 10 issues closed). An issue asking to move from CNCF sandbox to incubating (#2117) has been open since 2023-12-05 (GitHub issue search, 2026-11-20). Contents of the v2.2.0 milestone were not read.
- Relevance: none for design; watch v2.2.0 for per-repository locking and protected tags.

### F168. Zot (Q10, secondary): a third-party pull-through comparison on one Apple M4 host reports zot slower than Distribution's proxy mode on cold and warm image pulls but far faster at streaming a cached layer, and shows zot's tag requests paying an upstream round trip each time.
- Quote: "distribution pays 13.5 s and zot's on-demand sync far more, 51.2 s." — S281, Pulling images (cold pull of six official images against Docker Hub)
- Quote: "peryx serves a single stream at 6,838 MB/s against zot's 2,772, and eight readers take it to 8,192." — S281, Layer throughput
- Quote: "manifest by tag 58 µs ±21% (0.00x) 231 ms ±2% (1.00x) 245 ms ±4% (1.06x) 967 ms ±1% (4.18x)" — S281, Endpoint coverage (columns: peryx, direct, distribution, zot)
- Quote: "model Mac16,10 CPU Apple M4 (arm64) cores 10 (4 performance + 6 efficiency) memory 16 GB OS macOS 27.0 (kernel 27.0.0)" — S281, the host every table on this page was measured on
- Conditions/notes: (secondary) published by the authors of a competing registry (peryx); versions of zot and Distribution are not stated on the page; unversioned "latest" docs URL fetched 2026-11-20. Workloads: pull six official images through each registry with crane, cold then warm; stream one 30 MB cached layer over loopback, single and 8 parallel; ten clients pulling node:22-alpine at once. Other reported figures (median over rounds, against Docker Hub): warm pull of six images 2.7 s Distribution, 6.6 s zot, 7.4 s direct; hot-cache single stream 99 MB/s Distribution, 2,772 MB/s zot; 8 parallel streams 253 MB/s Distribution, 7,002 MB/s zot; ten parallel cold pulls 12.0 s Distribution, 8.2 s zot; manifest by digest 1.6 ms Distribution, 149 µs zot. The page attributes zot's layer speed to sendfile from the page cache. The 967 ms tag request is consistent with zot's default revalidation of tags against upstream on every request (F128).
- Relevance: the comparison shape to reproduce (cold fill, warm serve, hot-layer throughput, fleet single-flight, per-endpoint latency) and two targets to beat: whole-image sync before first byte, and upstream round trips on every tag read.

**Project Quay v3.18.3 (F169 to F221)**

### F169. Quay (Q1): Quay is a Python monolith: one container image runs nginx, three gunicorn applications, memcached and about thirty background workers under supervisord.
- Quote: "Quay is written in Python and each pod runs as a single monolithic container." — S377, Root cause analysis
- Quote: "Inside a Quay pod is a vast ecosystem of worker processes that handle the web UI, the registry itself and asynchronous tasks such as garbage collection and coordination with Clair for security scans." — S378, State of Quay.io pre-migration
- Quote: "command=memcached -u memcached -m 64 -l 127.0.0.1 -p 18080" — S298, [program:memcache] L282
- Conditions/notes: conf/supervisord.conf.jnj at v3.18.3 defines 35 `[program:...]` entries (gunicorn-web, gunicorn-registry, gunicorn-secscan, nginx, memcache, dnsmasq, pushgateway, and the gc, mirror, replication, quota, security and cleanup workers); counted with grep, not quoted.
- Relevance: avoid; the per-request path crosses nginx, gunicorn/gevent, a local memcached and the database, and every pod carries every worker, which is what a single static binary with in-process tasks removes.

### F170. Quay (Q1): State is split three ways: metadata in a relational database (PostgreSQL preferred, MySQL/MariaDB deprecated since 3.6), blobs in object storage, and Redis for build logs, user events and the locks garbage collection depends on.
- Quote: "Quay state is split between database metadata, Redis coordination/cache data, and object storage blobs." — S309, Contextification Addendum
- Quote: "Support for MySQL and MariaDB as the database backend is deprecated since the {productname} 3.6 release and is planned for removal." — S324, Database backend
- Quote: "If Redis fails, you will lose access to build logs, builders, and the garbage collector service." — S325, Redis backend
- Conditions/notes: required external services for a production deployment are therefore PostgreSQL, Redis and an object store; Clair adds a second PostgreSQL (F201). The code still registers `mysql`, `mysql+pymysql`, `sqlite`, `postgresql` and `postgresql+psycopg2` schemes (data/database.py L77-L82).
- Relevance: copy the metadata/bytes split; avoid making GC correctness depend on a third, non-durable service.

### F171. Quay (Q1, Q8): Thirteen storage driver classes are registered (local, S3 and S3-compatible variants, GCS, RadosGW, Swift, Azure, CDN-fronted variants), and the documentation restricts local storage to proof-of-concept use.
- Quote: ""S3Storage": S3Storage, "GoogleCloudStorage": GoogleCloudStorage, "RadosGWStorage": RadosGWStorage, "SwiftStorage": SwiftStorage," — S310, STORAGE_DRIVER_CLASSES L24-L38
- Quote: "{productname} can work with local storage, however this should only be used for proof of concept or test setups, as the durability of the binary blobs cannot be guaranteed. Mounted NFS volumes are not supported." — S326, Image storage backend
- Conditions/notes: full list at v3.18.3: LocalStorage, S3Storage, GoogleCloudStorage, RadosGWStorage, SwiftStorage, CloudFrontedS3Storage, AzureStorage, RHOCSStorage, CloudFlareStorage, MultiCDNStorage, IBMCloudStorage, STSS3Storage, AkamaiS3Storage.
- Relevance: beat; a registry whose laptop mode is explicitly not durable is the opposite of one code path from laptop to fleet.

### F172. Quay (Q1, Q13): In 2026 the project started replacing parts of the Python stack with Go: the tag mirror-registry-v3.0.0 (2026-11-09) is a statically linked Go binary that embeds CNCF Distribution v3 and keeps metadata in SQLite, motivated in part by the Python mirror registry needing 8 GB+ of RAM.
- Quote: "Reduce Resource Footprint: Target ~100-200MB memory vs current 8GB+ requirement, enabling OMR on resource-constrained environments." — S361, Goals
- Quote: "RUN CGO_ENABLED=0 go build -o /mirror-registry ./cmd/mirror-registry" — S321, L8
- Quote: "github.com/distribution/distribution/v3 v3.1.1" — S322, require block L6
- Quote: "The current OpenShift Mirror Registry is based on full Quay (Python), deployed via Ansible with the following known issues:" — S361, Background > Current OMR Issues
- Conditions/notes: the 100-200 MB figure is a stated target in the tracker epic, not a measurement; go.mod at that tag also requires `modernc.org/sqlite v1.48.1` and `github.com/jackc/pgx/v5`. The Go code (cmd/, internal/) also ships inside the v3.18.3 tree. mirror-registry-v3.0.0 is a tag with source archives only; the separate quay/mirror-registry repository's newest release is v2.0.12 (2026-09-03).
- Relevance: the incumbent itself concluded that the Python multi-process design is too heavy for the single-node case; its answer is a second code path, which a one-code-path design rules out.

### F173. Quay (Q2): Quay claims OCI v1.1 support and says it strives to be compatible; the referrers API arrived in v3.12.0 (2024-07-18) and was enabled by default in v3.13.0 (2024-10-24).
- Quote: "Docker Manifest Schema [v2.1], [v2.2] - OCI spec v1.1 support [oci]" — S308, High-level features
- Quote: "strives to be as compatible as possible with the OCI 1.1 Image and Distribution specifications" — S339, Open Container Initiative support
- Quote: "implements the OCI 1.1 referrers API (PROJQUAY-7280)" — S307, [v3.12.0] - 2024-07-18
- Quote: "enable the referrers api by default (PROJQUAY-7499)" — S307, [v3.13.0] - 2024-10-24
- Conditions/notes: the README's `[oci]` link points at distribution-spec v1.1.0-rc1.
- Relevance: "strives to be compatible" is a weaker claim than passing all four conformance categories; a new registry should state the categories it passes.

### F174. Quay (Q2): The project's own CI runs the OCI conformance suite against the Python registry with pull, push and content discovery enabled and content management disabled, at distribution-spec v1.1.0-rc1, and has to turn tag caching off for it to pass.
- Quote: "export OCI_TEST_CONTENT_DISCOVERY=1 export OCI_TEST_CONTENT_MANAGEMENT=0" — S304, job oci > Run conformance tests L323-L324
- Quote: "repository: opencontainers/distribution-spec ref: v1.1.0-rc1 path: dist-spec" — S304, job oci L225-L227
- Quote: "OCI Conformance tests don't expect tags to be cached. If we implement cache invalidation, we can enable it back." — S304, job oci > config DATA_MODEL_CACHE_CONFIG L273-L274
- Counter: the Go mirror registry in the same repository is tested with all four categories at spec v1.1.1 — "export OCI_TEST_CONTENT_DISCOVERY=1 export OCI_TEST_CONTENT_MANAGEMENT=1" — S305, job oci-conformance L183-L184
- Counter: with the default 120 s tag cache a deleted tag is still listed — "Quay caches information about tags for about a minute. OCI conformance tests check that deleted tags are not present in the tag list, and it fails because of the caching." — S370, Description
- Conditions/notes: conformance status per category for the Python registry: pull run, push run, content discovery run (with `active_repo_tags_cache_ttl: 0s`), content management not run. No published pass/fail report was found.
- Relevance: beat; pass all four categories with production defaults, not with a cache switched off for the test.

### F175. Quay (Q2): Blob deletion through the registry API is refused, and digests are computed with SHA-256 only.
- Quote: "We do not support deleting arbitrary digests, as they break repo images." — S290, delete_digest L507-L509
- Quote: "computed_digest = digest_tools.sha256_digest_from_hashlib(self.blob_upload.sha_state)" — S293, _validate_digest L324
- Conditions/notes: the digest route regex accepts any `algorithm:hex`, but the uploader only keeps a resumable SHA-256 state (`sha_state = ResumableSHA256Field`, data/database.py L1720), so a non-sha256 digest can never match (code reading; not tested). `delete_digest` raises `Unsupported`.
- Relevance: beat on sha512 (the distribution-spec conformance suite exercises it) and offer a typed refusal for blob DELETE if it is not supported.

### F176. Quay (Q2): The referrers endpoint supports the artifactType filter, but answers a query for a subject that is not in the repository with MANIFEST_INVALID, and only the Go registry code sets the OCI-Subject header.
- Quote: "headers["OCI-Filters-Applied"] = "artifactType"" — S291, list_manifest_referrers L58-L59
- Quote: "except ManifestDoesNotExist as e: raise ManifestInvalid(str(e))" — S291, list_manifest_referrers L48-L49
- Quote: "// SubjectHeaderMiddleware wraps an http.Handler and sets the OCI-Subject" — S313, L42
- Counter: the specification requires an empty list, not an error, when nothing refers to the digest — "If a query results in no matching referrers, an empty manifest list MUST be returned." — S383, Listing Referrers
- Conditions/notes: inferred from code at v3.18.3 and not tested against a running instance: `lookup_manifest_by_digest(..., raise_on_error=True)` raises when the subject manifest row is absent and `ManifestInvalid` defaults to HTTP 400; a case-insensitive search of the whole tree finds `OCI-Subject` only in the Go middleware. The changelog records the earlier decision "return 400 instead of 404 for manifest errors on referrers api (PROJQUAY-7523)".
- Relevance: copy the artifactType filter with `OCI-Filters-Applied`; avoid the missing-subject error and always send `OCI-Subject`.

### F177. Quay (Q2, Q9): The page maximum for tag and catalog listings is max(V2_PAGINATION_SIZE, 100), so the configured default of 50 is ignored and a page holds up to 100 entries; tag listings are served from a cache with a 120 s TTL.
- Quote: "_MAX_RESULTS_PER_PAGE = max( app.config.get("V2_PAGINATION_SIZE", 100), 100 ) # minimally server 100 tags per page" — S289, L91-L93
- Quote: ""active_repo_tags_cache_ttl": "120s"," — S282, DATA_MODEL_CACHE_CONFIG L743-L750
- Conditions/notes: `limit = max(min(requested_limit, _MAX_RESULTS_PER_PAGE), 0)` for tags; the catalog uses encrypted page tokens rather than the last repository name.
- Relevance: the only stated reason is "minimally server 100 tags per page"; a new registry should give its page bound a measured reason and never serve a stale listing after a delete.

### F178. Quay (Q3): Garbage collection is online and continuous, there is no command to run it on demand, and repository and namespace deletion run one worker at a time under a global lock.
- Quote: "Currently, all garbage collection happens discreetly, and {productname} does not provide commands to manually run garbage collection." — S323, Garbage collection in practice
- Quote: "Namespace and repository garbage collection workers require a global lock to work. As a result, and for performance reasons, only one worker runs at a time." — S323, Garbage collection in practice
- Quote: ""LARGE_GARBAGE_COLLECTION", lock_ttl=REPOSITORY_GC_TIMEOUT + LOCK_TIMEOUT_PADDING" — S318, RepositoryGCWorker.process_queue_item L27-L29
- Conditions/notes: the registry never goes read-only for GC. The same lock name `LARGE_GARBAGE_COLLECTION` is used by namespacegcworker, so repository purges and namespace purges serialize across the whole deployment.
- Relevance: copy online GC; avoid a deployment-wide single lock for purges.

### F179. Quay (Q3): Tag garbage collection handles one randomly chosen repository per worker iteration (every 30 s by default), drawn from at most 500 candidates, under a per-repository Redis lock with a time-to-live of 3 hours plus 60 s.
- Quote: "How often the Garbage Collection worker runs. GARBAGE_COLLECTION_FREQUENCY = 30  # seconds" — S282, L536-L537
- Quote: "GC_CANDIDATE_COUNT = 500  # repositories" — S286, L45
- Quote: "REPOSITORY_GC_TIMEOUT = 3 * 60 * 60  # 3h" — S285, L20
- Quote: ".limit(100)  # This sucks but it's the only way to limit memory" — S287, _get_gc_expiration_policies L207-L215
- Conditions/notes: `_garbage_collection_repos` picks a random time-machine policy (distinct `removed_tag_expiration_s` values, capped at 100 and cached 600 s), then one random repository with tags expired past that window. Throughput is therefore bounded by one repository per gcworker per interval; no duration or backlog measurements are published.
- Relevance: avoid random sampling with a fixed tick as the scheduling policy; GC backlog should be observable and bounded by work, not by a poll interval.

### F180. Quay (Q3): The collector is reference-driven inside the database and deletes storage objects afterwards, outside the transaction, accepting leaked objects as the price.
- Quote: "Runs garbage collection in two phases: first discovers all manifests, blobs and labels in the manifest tree via BFS, then deletes them iteratively. Labels are deleted in bulk." — S283, _run_garbage_collection L302-L306
- Quote: "We are going to make the conscious decision to not delete image storage blobs inside transactions. This may end up producing garbage in s3, trading off for higher availability in the database." — S284, garbage_collect_storage L240-L242
- Quote: "DEFAULT_TAG_EXPIRATION = "2w"" — S282, L688-L689
- Conditions/notes: order of work per repository: tags whose `lifetime_end_ms` is older than the namespace's time-machine window are deleted; a manifest is deleted when no tag, no parent index and no referrer points at it; a blob row (ImageStorage) is deleted when no ManifestBlob and no UploadedBlob row references it; then the object is removed if no other row has the same checksum. Deleted tags stay restorable for the time-machine window (default two weeks).
- Relevance: copy the time-machine window as a stated retention bound; avoid a delete order that can leave unreferenced bytes with no record of them.

### F181. Quay (Q3): In-flight pushes are protected by time, not by reachability: hidden temporary tags and UploadedBlob rows expire after PUSH_TEMP_TAG_EXPIRATION_SEC, which was raised from one hour to six in v3.17.5 and v3.18.1 because large multi-arch pushes outlived it.
- Quote: "Increased to 6 hours by default, to protect big multiarch images from not being GCed while pushes are still in progress" — S282, L500-L503
- Quote: "UploadedBlob tracks a recently uploaded blob and prevents it from being GCed while within the expiration window." — S299, class UploadedBlob L1701-L1705
- Quote: "Set temp tag and blob expiry to 6 hours by default" — S307, [v3.18.1] - 2026-10-05, PROJQUAY-13365
- Quote: "Python Quay PUT /v2/.../manifests/sha256:... (write_manifest_by_digest) creates a hidden $temp- tag with PUSH_TEMP_TAG_EXPIRATION_SEC = 3600." — S355, Root cause
- Quote: "This delay avoids prematurely deleting blobs that are part of an ongoing push." — S323, Garbage collection in practice (NOTE)
- Conditions/notes: the fix for a push that outlives the window was a larger constant. A push slower than six hours is still unprotected (inference from the mechanism).
- Relevance: avoid; this is a "longer timeout" fix, not a root-cause fix. Protect an upload session by its existence, not by a wall-clock guess.

### F182. Quay (Q3): Two GC races that made pulls fail (a blob deleted from storage while the database still referenced it) were documented and patched in March 2026 with a per-digest Redis lock, and the code states that the lock does not close the race when Redis is unavailable.
- Quote: "There are two critical race conditions that can happen during garbage collection of blobs which can cause complete pull failures." — S372, PR description
- Quote: "If a pull is now tried for the same manifest, then pull will fail because the blob is now missing from backend storage, but is referenced by the database. So this is a time-of-check-time-of-use (TOCTOU) race condition." — S372, PR description
- Quote: "Note: GlobalLock ensures that deletion is atomic with the database operation, but does not avoid *all* race conditions." — S284, garbage_collect_storage L250-L255
- Quote: "If Redis is unavailable, func may also create a missing blob without the lock. The race condition can *still* happen then, but the window is extremely narrow, and database uniqueness constraints provide the ultimate safety guarantee." — S284, with_blob_lock_or_fallback docstring L356-L359
- Conditions/notes: lock name `BLOB_DELETE_<digest>`, TTL 120 s on the GC side and 30 s on the push side; a push waits at most 3 s (`BLOB_DELETE_LOCK_ACQUIRE_TIMEOUT = 3`); if a GC worker still holds the lock and the blob row is missing, the push is answered with HTTP 503 for the client to retry. The tracker issue behind the PR (PROJQUAY-10909) is not publicly readable.
- Relevance: the central failure mode to design out: delete-object and create-reference must be ordered by the metadata store itself (one transaction or a fencing epoch), not by a best-effort lock in a cache.

### F183. Quay (Q3): The race fix introduced a new failure that is open as of 2026-11-17: a blob upload returns HTTP 500 when GC deletes the deduplicated storage row in the middle of the upload, in every release from v3.17.3 to v3.18.3.
- Quote: "A blob upload PUT returns HTTP 500 when garbage collection deletes the deduplicated ImageStorage row between the upload's storage lookup and its ImageStoragePlacement insert." — S354, Description
- Quote: "shipped in v3.17.3-v3.17.7 and v3.18.0-v3.18.3" — S354, Seen on version or branch
- Conditions/notes: reported from a CI run on 2026-11-17; stated workaround is to retry the push. The report is marked AI-assisted by its author.
- Relevance: evidence that lock-based mitigation of a GC/push race keeps producing new interleavings; a model-checked or simulated design is the alternative.

### F184. Quay (Q3, Q11): GC can leave bytes behind with no record: maintainers acknowledge that the row tracking a blob can be deleted without the object, and the product cannot report how much space GC freed.
- Quote: "We do GC in an iterative fashion, as opposed to trying to delete everything in a single transaction for performance reason. So in theory, there's always a chance that the database row tracking a blob could get deleted but not the actual storage" — S356, comment 2021-12-02
- Quote: "and yet my registry is still consuming 7.1Gb even though it has nothing in it." — S356, Description
- Quote: "{productname} does not have a way to track how much space is freed up by garbage collection." — S323, Garbage collection in practice
- Conditions/notes: PROJQUAY-2451 was a Blocker fixed in v3.6.4 (repository purge left storage behind); the structural admission in the comment still matches the v3.18.3 code (F180).
- Relevance: beat; charge every stored byte to a record so that orphaned storage is detectable by an audit, and export bytes reclaimed.

### F185. Quay (Q3, Q13): The Go mirror registry's first collector ran database-wide every 30 s and deleted the untagged per-architecture manifests of a multi-arch push that was still in progress, so pulls failed with BLOB_UNKNOWN about an hour after a successful push; it was a release blocker fixed by copying Python Quay's hidden temporary tags.
- Quote: "Go GC Collect() runs database-wide every 30s. Phase 2 (FindOrphanedManifests) deletes every manifest with no tag, no manifestchild parent, and no subject." — S355, Root cause
- Quote: "If that cycle ran after skopeo's child digest PUTs and before the tagged index PUT (slow copy / large image), children and manifestblob rows were deleted." — S355, Root cause
- Quote: "// Interval between GC cycles. Default: 30s." — S320, type Config L31-L35
- Quote: "const staleUploadThreshold = 48 * time.Hour" — S319, L14
- Conditions/notes: priority Blocker, fix version omr-v3.0.0, closed 2026-10-01. The collector at the tag has phases 0-4 (expired upload markers, expired tags, orphaned manifests with at most 100 iterations, orphaned blobs under a per-digest in-process lock, stale uploads older than 48 h).
- Relevance: the push-by-digest-then-tag-the-index sequence is the canonical test case for any online GC; it belongs in a new registry's simulation suite.

### F186. Quay (Q3, Q9): Abandoned upload sessions are purged after two days by an hourly worker, and stale S3 multipart uploads after one day by a six-hourly task whose threshold is deliberately not configurable.
- Quote: "threshold = app.config.get("BLOBUPLOAD_DELETION_DATE_THRESHOLD", 60 * 48)  # 2 days" — S288, L16
- Quote: "The TTL is explicitly chosen not to be configurable by the client: if the threshold is set too low, then MPUs still in progress might be deleted causing push failures." — S288, L21-L24
- Counter: the same file's own comment disagrees with the default — "Find all blob uploads older than the threshold (typically a week) and delete them." — S288, _cleanup_uploads L110
- Conditions/notes: `BLOBUPLOAD_CLEANUP_FREQUENCY` default 3600 s; `MPU_CLEANUP_TTL = 60 * 60 * 24`; `MPU_CLEANUP_FREQUENCY = 6 * 60 * 60`; lock TTL 20 minutes.
- Relevance: the one constant with a stated reason is the one that is fixed; copy the practice of writing the reason next to the bound.

### F187. Quay (Q4): The pull-through cache is an organization bound to exactly one upstream registry (or one namespace of it) with optional credentials and an expiration; it is off by default.
- Quote: "Caching of entire upstream registries or just a single namespace, for example, all of `docker.io` or just `docker.io/library`." — S327, Quay as a proxy cache for upstream registries
- Quote: "Feature Flag: Enables Quay to act as a pull through cache for upstream registries FEATURE_PROXY_CACHE = False" — S282, L940-L941
- Quote: "Anonymous users cannot pull images through a proxy cache if the image has not been previously cached." — S329, Proxy cache limitations
- Conditions/notes: one upstream per organization, so multi-upstream means several organizations; upstream credentials are stored encrypted on the organization's ProxyCacheConfig row (`EncryptedCharField`).
- Relevance: copy per-namespace upstream binding and encrypted credentials; consider multi-upstream fallback, which Quay lacks.

### F188. Quay (Q4): Freshness is checked on every tag pull with an upstream HEAD for the manifest digest, and each confirmed pull re-arms the cached tag's expiration (default 86400 s).
- Quote: "We use HEAD requests to check for new versions of an image, and HEAD requests do not count against rate limit." — S373, Open Questions
- Quote: "DEFAULT_PROXY_CACHE_EXPIRATION = 86400  # 24 hours, in seconds" — S299, L68
- Quote: "always bump tag expiration when retrieving tags that both are cached and exist upstream, as a means to auto-renew the cache." — S301, ProxyModel.get_repo_tag L389-L390
- Quote: "A manifest is considered stale when the manifest's digest changed in the upstream registry." — S301, _update_manifest_for_tag docstring L452-L453
- Quote: "In the proxy organization, the tag expiration is refreshed to the value set in the UI's *Expiration* field every time the tag is pulled." — S330, Using Quay to proxy a remote registry (NOTE)
- Quote: "if "timeout" not in kwargs: kwargs["timeout"] = (5, 30)" — S300, Proxy._safe_request L174-L175
- Conditions/notes: there is no TTL during which the cache answers without contacting upstream; the "expiration" is a staleness bound used only when upstream is unreachable, plus the input to garbage collection. Upstream request timeouts are (connect 5 s, read 30 s) by default and (5 s, 120 s) for blobs.
- Relevance: a revalidate-on-every-pull model makes the upstream's latency and availability part of every cached pull; a bounded-staleness TTL with background revalidation avoids that.

### F189. Quay (Q4): The design says a cached image is served while the upstream is down, but in at least 3.12 through 3.15 such pulls timed out because the upstream check ran first; the fix landed in v3.18.0.
- Quote: "If this happens with the configured staleness period, the image stored in cache is delivered. If the pull happens after the configured staleness period, the error is propagated to the user." — S328, Proxy cache architecture
- Counter: observed behaviour contradicted the design for more than a year — "Currently, it seems that regardless of the state of the image in Quay, proxy functionality calls the upstream registry to test whether it's available or not and only then provides access to the image." — S360, Description
- Quote: "serve cached images when upstream registry is unavailable" — S307, [v3.18.0] - 2026-07-23, PROJQUAY-8440
- Conditions/notes: the report shows a 504 after 60.008 s for a manifest already cached, with the upstream blocked; affected versions listed are 3.12.0 and 3.15.1; fix versions 3.16.z and 3.17.z in the tracker and v3.18.0 in the changelog.
- Relevance: "upstream down" must be a tested scenario (a nemesis in simulation), not a design-doc promise.

### F190. Quay (Q4): A proxy-cache organization still accepts pushes; making it read-only has been an open request since 2022 and was still undecided on 2026-11-19.
- Quote: "This is not part of the tech preview - proxy orgs can still receive pushes." — S373, Non-Goals
- Quote: "Quay user with write/admin role can still push images to this read-only organization successfully" — S357, Description
- Quote: "make proxy read only, simple, straightforward, but changes behaviour, i.e. customer pushes will start to fail" — S357, Update (19 Nov 2026)
- Conditions/notes: this refutes the lead in the brief that a Quay proxy organization cannot be pushed to. A pushed tag in a proxy organization is subject to the same expiration and upstream-digest retargeting as cached tags (inference from registry_proxy_model.py).
- Relevance: decide the push semantics of a proxy namespace on day one and refuse with a typed error; changing it later breaks users, which is why this issue is stuck.

### F191. Quay (Q4): Cache eviction exists only as a side effect of an organization storage quota: when a new image would exceed it, least-recently-used tags are expired for later garbage collection, and the quota is not enforced in the meantime.
- Quote: "When a proxied namespace has quota limits configured, {productname} can auto-prune the least recently used image tags so that a new push that exceeds available storage is stored instead of being skipped." — S331, Leveraging storage quota limits in proxy organizations
- Quote: "As a result, the quota size restriction is not fully enforced during this period." — S331, IMPORTANT
- Quote: "if your proxy cache organization has a maximum size of 500 MB, and the image a user wants to pull is 700 MB, the image will be cached and will overflow beyond the configured limit" — S329, Proxy cache limitations
- Counter: the code refuses instead of overflowing — "if image_size > ns_quota_limit: raise QuotaExceededException" — S301, _check_image_upload_possible_or_prune L184-L185
- Conditions/notes: the documentation and the v3.18.3 code disagree about an image larger than the quota; manifest lists skip the quota check entirely (`if upstream_manifest.is_manifest_list: return`).
- Relevance: give the cache its own stated budget and eviction rule, enforced at admission.

### F192. Quay (Q4): The proxy cache has no single-flight for concurrent first pulls, and until fixes in 2026 a blob over roughly 500 MB failed every pull and left a placeholder row that blocked later pulls.
- Quote: "duplicate key value violates unique constraint "manifest_repository_id_digest"" — S358, Description
- Quote: "Pulling an image through a proxy cache organization *always fails with HTTP 500* when any blob exceeds ~500 MB." — S359, Bug Summary
- Quote: "When the download fails (worker killed by timeout), the database record exists but has *no corresponding storage placement*: no data was written to storage." — S359, Bug Summary
- Quote: "Stream non-cached blobs from upstream directly to clients without downloading them first" — S307, [v3.18.2] - 2026-10-16, PROJQUAY-13517
- Conditions/notes: PROJQUAY-7962 (HTTP 500 on simultaneous pulls) has been open since 2024-09-17; the large-blob report cites a customer on 3.16.4 with a 4.3 GiB blob failing after about 31 s (the gunicorn worker timeout). In v3.18.3 a missing blob is redirected to an nginx upstream-proxy location and queued for a background download worker.
- Relevance: copy the final shape (stream to the client while filling the cache, in the background, with a bounded queue) and add single-flight per digest from the start.

### F193. Quay (Q4, Q7): Repository mirroring shells out to skopeo on a schedule (default every 24 h) with tag-glob and architecture filters, makes the target read-only except for one robot, strips signatures and does not mirror referrers.
- Quote: ""copy", "--all", "--remove-signatures"," — S302, SkopeoMirror.copy L130-L133
- Quote: "Only the designated robot can push images to a mirrored entity, superseding any role-based access control permissions set on the entity." — S332, features and limitations
- Quote: "Defaults to syncing every 24 hours. You can change that based on hours or days." — S333, Sync Interval
- Quote: "OCI Referrers API artifacts, such as Cosign signatures and SBOMs, are not currently mirrored." — S332, features and limitations
- Quote: "Multiple tags in the same repository cannot be mirrored in parallel." — S334, Repository mirroring capacity considerations
- Conditions/notes: organization-level mirroring (3.17) does not replicate deletions: "The downstream mirror acts as a cumulative archive." Mirror workers poll every 30 s (`REPO_MIRROR_INTERVAL = 30`).
- Relevance: avoid a subprocess-per-tag design and avoid dropping signatures; replication should carry referrers by default.

### F194. Quay (Q4, Q9): Mirror limits were raised after field failures: the skopeo timeout was hard-coded to 300 s until 3.15 and the maximum sync duration went from 2 to 12 hours.
- Quote: "The value of the {{SKOPEO_TIMEOUT_SECONDS}} is currently hard coded to 300 seconds which can cause problems when mirroring big images with very large layers." — S368, Description
- Quote: "Defaults to `300` seconds (5 minutes). The maximum timeout length is `43200` seconds (12 hours)." — S333, Skopeo timeout interval
- Quote: "We need to increase the MAX_SYNC_DURATION because for certain very big images, 2 hours might not be enough" — S303, L31-L33
- Conditions/notes: `MAX_SYNC_RETRIES = 3`; `MAX_SYNC_DURATION = 60 * 60 * 12`; manifest lists parsed during architecture-filtered mirroring are capped at 10 MB and 1000 entries, with the reason "Prevents DoS via oversized manifests from malicious registries."
- Relevance: both constants are whole-operation timeouts that had to grow with artifact size; bound progress (bytes per interval) instead of total duration.

### F195. Quay (Q4): Geo-replication is one shared database and one shared Redis with per-region object storage and asynchronous blob copying; the database is not replicated by Quay, and losing a site can lose blobs.
- Quote: "Geo-replication is asynchronous. Permanent loss of a site can lose blobs that were stored there but not yet replicated." — S335, Geo-replication requirements and constraints
- Quote: "A single database, and therefore all metadata and {productname} configuration, is shared across all regions. Geo-replication does not replicate the database and does not fail over to another database." — S335, Geo-replication requirements and constraints
- Quote: "{productname} has no internal awareness of object storage health." — S335, Geo-replication requirements and constraints
- Conditions/notes: every region must be able to read and write every other region's object store; the whole storage backend is replicated, not selected repositories; the replication worker polls its queue every 10 s with a 20-minute reservation per item.
- Relevance: beat; with replicated metadata and erasure-coded bytes an acknowledged push should survive the loss of a site.

### F196. Quay (Q5): Quay is its own token service: credentials are checked by one of six configured backends and registry access uses bearer JWTs valid for one hour.
- Quote: ""enum": ["Database", "LDAP", "JWT", "Keystone", "OIDC", "AppToken"]," — S312, AUTHENTICATION_TYPE L425-L430
- Quote: "TOKEN_VALIDITY_LIFETIME_S = 60 * 60  # 1 hour" — S314, L59
- Quote: "REGISTRY_JWT_AUTH_MAX_FRESH_S = ( 60 * 60 + 60 ) # At most signed one hour, accounting for clock skew" — S282, L508-L511
- Conditions/notes: tokens are RS256 (`ALGORITHM = "RS256"`, util/security/registry_jwt.py L21) signed with a per-instance service key that expires after 120 minutes and is refreshed every 55. External login providers (GitHub, Google, generic OIDC) are layered on top; 3.16 added PKCE and 3.18 multi-issuer OIDC.
- Relevance: copy the built-in token endpoint so that no separate auth service is required; the 60 s skew allowance is the only reasoned constant here.

### F197. Quay (Q5): Authorization is role-based over organizations, teams, users and robots: three team roles and three repository permission levels.
- Quote: "All member permissions, plus the ability to create new repositories." — S340, Setting a team role by using the UI (Creator)
- Quote: "Role.create(name="admin") Role.create(name="write") Role.create(name="read")" — S315, L422-L424
- Quote: "{productname} provides full isolation of registry content by organization and team with fine-grained entitlements for read, write, and administrative access by users and automated tools." — S344, Role-based access controls
- Conditions/notes: team roles are admin, creator and member (initdb.py L425-L427). Per-repository policy beyond permissions: visibility, tag expiration, immutability policies (3.17), auto-prune policies, mirroring state, notifications. Superusers and "global read-only superusers" are configured by name in config.yaml.
- Relevance: the read/write/admin repository triple with team indirection is the de facto model to be compatible with.

### F198. Quay (Q5): Robot accounts are namespace-scoped service identities with static tokens; keyless federation exchanges an external OIDC token for a robot token that lasts one hour, and the unreleased 5.0 documentation adds expiring, scoped robot API tokens as a technology preview.
- Quote: "Each Robot Account is limited to a single user namespace or Organization." — S341, Robot Account overview
- Quote: "With keyless authentication in {productname}, you can exchange an OIDC token for a short-lived robot account token that expires after one hour." — S342, Keyless authentication with robot accounts
- Quote: "Unlike the robot account's static push and pull token, a robot API token expires after at most 90 days and carries a set of OAuth scopes that limit which management API operations it can perform." — S347, Robot API tokens (Technology Preview)
- Conditions/notes: 3.18 added an optional `audiences` list per federation entry; without it audience validation is skipped with a deprecation warning.
- Relevance: copy keyless exchange for workloads (it is what CI systems want); make audience validation mandatory from the start.

### F199. Quay (Q5): Anonymous pull of public repositories is on by default, and a repository created by a push is private by default.
- Quote: "Feature Flag: Whether to allow anonymous users to browse and pull public repositories. FEATURE_ANONYMOUS_ACCESS = True" — S282, L335-L336
- Quote: "Repos created by push default to private visibility CREATE_PRIVATE_REPO_ON_PUSH = True" — S282, L883-L884
- Conditions/notes: `CREATE_NAMESPACE_ON_PUSH = False` by default.
- Relevance: sensible defaults to copy.

### F200. Quay (Q6): Search is a REST API for repositories (plus prefix lookup of users, teams and robots), capped at 10 results per page and 10 pages; the catalog endpoint lists public repositories only if a feature flag is set; there is no query language.
- Quote: "MAX_PER_PAGE = app.config.get("SEARCH_RESULTS_PER_PAGE", 10) MAX_RESULT_PAGE_COUNT = app.config.get("SEARCH_MAX_RESULT_PAGE_COUNT", 10)" — S311, L393-L394
- Quote: "Feature Flag: If set to true, the _catalog endpoint returns public repositories. Otherwise, only private repositories can be returned. FEATURE_PUBLIC_CATALOG = False" — S282, L430-L432
- Conditions/notes: endpoints `/api/v1/find/repositories`, `/api/v1/find/all`, `/api/v1/entities/<prefix>`; the UI is a React application (default since 3.16; the Angular UI is deprecated). At most 100 search results are reachable.
- Relevance: beat; search over artifact type, annotations and referrers is absent here.

### F201. Quay (Q6): Vulnerability scanning is delegated to Clair, a separate service that needs its own PostgreSQL database; a worker in Quay submits manifests for indexing every 30 s.
- Quote: "When you use Clair vulnerability scanning, plan a separate PostgreSQL database for Clair. Do not use the same PostgreSQL database for both {productname} and Clair." — S324, Database backend
- Quote: "The number of seconds between indexing intervals in the security scanner SECURITY_SCANNER_INDEXING_INTERVAL = 30" — S282, L552-L553
- Quote: "A client is reporting that Clair's database is growing uncontrollably. They are already hitting 75 GB for their database size." — S369, Description
- Conditions/notes: Clair's newest release is v4.9.0 (2025-12-10), with v4.10.0-rc.0 on 2026-11-13. The Clair database growth issue (2022) had 14 watchers.
- Relevance: integrate scanners through the referrers/artifact model rather than a second stateful service with its own unbounded store.

### F202. Quay (Q7): Quay stores cosign signatures, attestations and SBOMs as OCI artifacts and recognises them for display, but signing and verification are client-side and no pull-time enforcement was found.
- Quote: "COSIGN_TAG_SCHEMA_PATTERN = r"^sha256-[a-f0-9]{64}\.(sig|att|sbom)$"" — S286, L47
- Quote: "# Cosign / Sigstore signature artifact types stored as OCI referrers (cosign v3+)." — S317, L66
- Quote: "To sign and verify container images with Cosign in {productname}, you can install Cosign and authenticate to the registry." — S343, Cosign OCI support
- Conditions/notes: not found: any server-side signature verification, admission policy or notation support in the v3.18.3 tree (searched for sigstore, cosign, notation, rekor, fulcio). The legacy `FEATURE_SIGNING` flag (Docker Content Trust/TUF) defaults to False. Allowed artifact media types are an explicit allowlist (`ALLOWED_OCI_ARTIFACT_TYPES`). Mirroring removes signatures (F193).
- Relevance: beat; verification at pull with a typed refusal is an open differentiator.

### F203. Quay (Q8): Blobs are content-addressed at `<algorithm>/<first two hex characters>/<hex>` under the storage root, stored once per registry, and a re-upload of existing content is discarded after serving as proof of possession.
- Quote: "Generate a prefix which is always two characters, and which will be filled with leading zeros if the input does not contain at least two characters. e.g. ABC -> AB, A -> 0A" — S292, content_path L57-L58
- Quote: "It already existed, clean up our upload which served as proof that the uploader had the blob." — S293, _finalize_blob_storage L354-L355
- Quote: "{productname} shares blobs between namespaces and repositories in order to conserve disk space. For example, if the same image is pushed 10 times, only one copy of that image is stored." — S323, Garbage collection in practice (NOTE)
- Conditions/notes: example path from a tracker log: `sha256/f5/f5eed114...`; uploads are written as separate chunk objects under `uploads/` and assembled server-side (multipart copy) or client-side. Deduplication is global, with per-repository link rows (ManifestBlob, UploadedBlob) providing isolation.
- Relevance: copy global content addressing with per-repository reference rows; it is what makes cross-repository mounts cheap.

### F204. Quay (Q8, Q11): Manifests are rows in the database, one per repository and digest, so manifest reads never touch object storage; the hosted service's 2023 database migration silently truncated manifests above 32K.
- Quote: "Within a repository, there can only be one manifest with the same digest." — S299, class Manifest L1848-L1853
- Quote: "media_type = EnumField(MediaType) manifest_bytes = TextField()" — S299, class Manifest L1857-L1858
- Quote: "This meant that we had been running DMS in “Limited LOB” mode and this was truncating any ‘nclob’ fields to 32K." — S378, Migrating data is a dangerous business
- Conditions/notes: the same manifest pushed to two repositories is stored twice (unique index on repository and digest). The truncation was found from customer pull failures the Monday after the 2023-11-12 migration and repaired from a snapshot.
- Relevance: copy manifests-in-the-record-store; verify every stored manifest against its digest on read and in migration checks, which would have caught the truncation.

### F205. Quay (Q8): Cross-repository blob mount needs an explicit `from` repository that the caller can read (or that is public); there is no automatic discovery of an origin.
- Quote: "If we cannot mount the blob, fall back to the standard upload behavior, since we don't support automatic mount origin discovery across all repos." — S290, _try_to_mount_blob L185-L186
- Quote: "# If no direct permission, check if the repostory is public." — S290, _try_to_mount_blob L208
- Conditions/notes: a successful mount creates an UploadedBlob link with the push expiration (F181) and returns 201.
- Relevance: copy the permission check; the fallback to a normal upload session is the behaviour the specification asks for.

### F206. Quay (Q9): The layer limit is 20G by default, enforced in the application and in nginx, but with the default 5 MB minimum chunk an S3-backed registry cannot store a blob above about 50 GB because of S3's 10,000-part limit.
- Quote: "Maximum size allowed for layers in the registry. MAXIMUM_LAYER_SIZE = "20G"" — S282, L677-L678
- Quote: "client_max_body_size {{ maximum_layer_size }};" — S295, location blobs L308
- Quote: "Some backend storage providers, for example, Amazon Web Services (AWS) S3, are unable to store artifacts larger than 50 GB because of a strict limitation of 10,000 parts per upload" — S338, Preparing your registry to accept large artifacts
- Quote: "S3 supports up to 10000 parts in a multipart upload, with 5 MiB chunk we can only upload up to 50 GiB files." — S294, RadosGWStorage.__init__ L1106-L1110
- Conditions/notes: an oversized blob is refused with code `BLOB_TOO_LARGE` and HTTP 400 (not 413). No reason is given for 20G. Changing `minimum_chunk_size_mb` is documented as requiring a support case because it "can also slow down uploads".
- Relevance: derive the part size from the declared or observed blob size so that the configured maximum is reachable on every backend.

### F207. Quay (Q9): No manifest size limit is defined in the registry code; manifest uploads are bounded only by nginx's body limit, which the template leaves at the nginx default for the manifests location.
- Quote: "parsed = _parse_manifest(request.content_type, request.data)" — S316, write_manifest_by_tagname L308-L309
- Quote: "This block handles manifests endpoint requests, for which we want to restrict traffic heavier than the generic V2 operations, as it handles pushes and pulls." — S295, location manifests L357-L359
- Quote: "Default: client_max_body_size 1m;" — S384, client_max_body_size
- Quote: "Hummingbird source container upload fails with 413 Request Entity Too Large" — S371, Summary
- Quote: "writing manifest: uploading manifest" — S371, Description (client log ending in StatusCode: 413)
- Counter: the specification expects four times the nginx default — "Client and registry implementations SHOULD expect to be able to support manifest pushes of at least 4 megabytes." — S383, Pushing Manifests
- Conditions/notes: inference, not tested: `client_max_body_size` appears in the templates only for the blob, `/v1/` and `/push` locations and the build-manager server, so the 1m nginx default would apply to manifest PUTs. The 2026-04-21 tracker report shows a manifest upload to the hosted service failing with "413 Request Entity Too Large" (a source container with very many layers), which is consistent with this; the hosted service's own nginx settings are not public.
- Relevance: state the manifest limit explicitly (and at or above the specification's 4 MiB expectation) and refuse with a typed 413.

### F208. Quay (Q9): Rate limits are nginx limit_req zones that are off by default; the rates are per client address for HTTP/1 and per connection for HTTP/2, and the documentation describes different numbers from the template.
- Quote: "limit_req_zone $http1_bucket zone=dynamicauth_heavy_http1:10m rate=5r/s;" — S296, L84
- Quote: "This zone should always be used with burst=<number> (nodelay|delay) as the limit is very low on purpose but should allow for the burst of traffic required for a registry operation." — S296, L80-L83
- Quote: "This block handles tags endpoint requests, for which we want to restrict traffic due to how heavy an operation it can be" — S295, location tags L330-L331
- Counter: the product documentation gives other figures — "causes `nginx` to limit certain API calls to 30 per second. If that feature is not set, API calls are limited to 300 per second (effectively unlimited)." — S345, FEATURE_RATE_LIMITS
- Conditions/notes: zones at v3.18.3: very light 60 r/s (HTTP/1) and 600 r/s (HTTP/2); light 50 and 500; heavy 5 and 50; auth 120 r/s per namespace when enabled. Bursts: blobs 50/100, manifests 10/50, tags 2/2, `_catalog` 1/5 with a 10 s read timeout, other `/v2` 20/80, `/api/` 25/100. Exceeding returns 429. Limits are per nginx instance, not global.
- Relevance: the tags endpoint is singled out as the heavy operation; a new registry should make tag listing cheap enough not to need the tightest limit.

### F209. Quay (Q9, Q10): The registry application runs gunicorn with gevent workers, four per CPU with a floor of 8 and a ceiling of 64, and until the unreleased 5.0 documentation the worker timeout was gunicorn's 30 s default.
- Quote: "workers = get_worker_count("registry", 4, minimum=8, maximum=64)" — S297, L18
- Quote: "allow administrators to increase the gunicorn worker timeout beyond the previous hardcoded 30-second default." — S347, Configurable gunicorn worker timeout
- Quote: "Proxy cache pulls of large images (500 MB or greater) can exceed the default gunicorn worker timeout." — S349, NOTE
- Conditions/notes: 5.0 defaults are 300 s for the registry process and 60 s for the web process, each accepted in the range 30-300 s. nginx sets `proxy_read_timeout 2000` and `proxy_send_timeout 2000` for blob requests and `worker_connections 10240`.
- Relevance: a request-duration timeout on a worker that also streams multi-gigabyte bodies is the root of several incidents here (F192); streaming must not hold a bounded worker slot.

### F210. Quay (Q10): The only published benchmark with stated conditions (vendor, 2026-10-29) measured pull throughput of 0.22 to 0.56 images per second (37.62 to 95.56 MB/s of content) for 171 MB, 100-layer images on two Quay replicas of 2 to 16 vCPU, and says no official capacity guidance exists.
- Quote: "2 25 2 0.22 37.62 4 75 2 0.37 63.27 8 750 6 0.49 83.79 16 300* 8 0.56 95.56" — S375, Sizing recommendations (table body: vCPU per pod, recommended maximum concurrency, NooBaa endpoints, img/s, content MB/s)
- Quote: "After systematically eliminating bottlenecks at each layer, we ran a final iteration with proper NooBaa endpoint scaling and produced these recommendations for our specific configuration (2 Quay replicas, NooBaa storage, 171 MB/100-layer images):" — S375, Sizing recommendations
- Quote: "Red Hat doesn't currently publish official capacity planning guidance for Quay." — S375, What we learned
- Counter: the numbers are bounded by the storage backend and the test client, not by Quay — "Scaling NooBaa from 4 to 6 endpoints more than doubled throughput and recovered from 49% failure to 100% success." — S375, Bottleneck 1: NooBaa storage saturation
- Counter: a single Python test client collapsed before the registry did — "Throughput recovered to approximately 1.5 img/s—a 21x improvement over the single-pod result, with the same total concurrency hitting the same Quay deployment." — S375, Bottleneck 2: The test client itself
- Conditions/notes: system under test: self-managed OpenShift on AWS; 3 control-plane and 3 worker nodes, m5.4xlarge (m5.8xlarge for the 16 vCPU tests); network up to 10 Gbps; 2 Quay replicas with gunicorn workers = CPU x 4; external PostgreSQL on gp3 50Gi, 3,000 IOPS, 500m CPU / 2Gi; object storage NooBaa on OpenShift Data Foundation; Clair and autoscaling disabled; HAProxy router; each image pull fetches 6 layers in parallel; failed pulls timed out at 300 s. Recommended maximum concurrency per configuration: 25, 75, 750 and 300 (the last limited by the single test client). First-iteration baseline was 0.17 to 0.23 img/s. The Quay version is not stated. Pull only; no push, tag-list or GC measurements.
- Relevance: the bar to beat, class for class: about 0.5 image pulls per second (about 84 MB/s) at 750 concurrent pulls on two 8-vCPU replicas; a new registry's benchmark should publish the same shape of table with its hardware and commit.

### F211. Quay (Q10): The project publishes its load-test tool but no results with it; the tool's own 2024 write-up reports application pods saturating a 2-core limit and returning 502s.
- Quote: "These tests are intended to necessarily push Quay to its limits and collect metrics on various operations." — S374, Description
- Quote: "Because of a 2-core CPU hard limit on quay app pods, we were seeing those HTTP 502 errors indicating application unavailability." — S376, Initial test results and observations
- Conditions/notes: quay-performance-scripts drives the REST and registry APIs with Vegeta in LOAD, RUN, PUSH_PULL and DELETE phases on OpenShift and writes results to Elasticsearch or JSON files. The 2024 article raised the pod limit from 2 to 6 cores and reported "some improvement" without numbers.
- Relevance: a public, reproducible benchmark harness with recorded runs is absent in this project; a new registry should record every run.

### F212. Quay (Q10): Production data points exist for the hosted instance of the same code: about 5,000 requests per second with about as many database connections in 2020, 7K to 9K connections on a normal day in 2023, more than 60 million images in 2024, and the first month above one billion pulls in August 2020.
- Quote: "We knew from our monitoring that on average, Quay sustains about 5,000 requests per second and sees roughly that number of database connections on a constant basis." — S377, Root cause analysis
- Quote: "On a normal day, Quay.io would run with anywhere between 7K to 9K connections and even sustain bursts in excess of 10K based on traffic volumes." — S378, State of Quay.io pre-migration
- Quote: "This was not a quick job as Quay.io is currently managing over 60 million images." — S378, Migrating data is a dangerous business
- Quote: "August also saw the first month of over a billion container pulls from Quay.io to users around the world." — S379, Quay hits 1 billion containers pulls per month
- Conditions/notes: operator-reported production figures, not controlled measurements; the number of pods behind them is given only as a sizing sample (F213).
- Relevance: one database connection per in-flight request is the scaling limit this design reached; a billion pulls in a 31-day month is about 370 per second on average, which is the order of magnitude a single node of a new registry should be measured against.

### F213. Quay (Q10): The documented footprint is at least 6 GB of RAM (8 GB recommended) and 2 or more vCPUs per Quay container, and the reference sizing for the hosted service lists 15 to 30 Quay containers with a 32-core, 244 GB database.
- Quote: "minimum 6 GB; recommended 8 GB, 2 or more vCPUs" — S336, Scalability and minimum requirements
- Quote: "Sizing depends on many factors, such as the number of users, images, concurrent pulls and pushes, so standard sizing recommendations are not available." — S336, Scalability and minimum requirements
- Quote: "32 cores; 244 GB; 1+ TB disk" — S337, Sample deployment sizes (Database sizing, High End)
- Quote: "This change reduces image size by approximately 30%, shortens download and deployment times, and narrows the attack surface." — S350, Quay container image size optimization
- Conditions/notes: sample table, high end column: 15 Quay containers by default and 30 at scale-out, 10 to 15 Clair containers, Redis 4 cores and 28 GB, object storage 50+ TB up to PB, nodes for Quay 13 cores and 56 GB. Binary size, idle RSS and idle CPU: not found. The absolute image size before and after the 3.17 reduction: not found.
- Relevance: beat by an order of magnitude on the idle footprint; a 6 GB minimum is what rules this design out of the laptop case.

### F214. Quay (Q10, Q11): Known performance problems are database-shaped: a quota feature turned a 0.5 s listing into 55 s for 4,000 users, namespace deletion issued 14 s UPDATEs, and extended audit logging is documented to multiply log volume by about 2.5 to 3.
- Quote: "Loading times without quota management are usually 0.4-0.6 seconds. When quota management is turned on, load times increase to 55+ seconds, sometimes going over 60 seconds and causing a load timeout." — S367, Description
- Quote: "Two slow queries in a 20k trace sample:" — S362, Observed in Jaeger
- Quote: "This increases log volume by ~2.5-3x. Defaults to False (opt-in)." — S282, FEATURE_EXTENDED_ACTION_LOGGING L943-L946
- Conditions/notes: the two slow queries were `UPDATE repository ...` at 14,776 ms and 5,715 ms against a 500 ms threshold (hosted service, 2026); the quota slowdown was reported on 3.7 with "over 4 thousand users".
- Relevance: per-row queries in loops and per-request audit rows in the primary database are the recurring cause; keep audit events off the request's write path.

### F215. Quay (Q11): The database is the service's single point of failure and its dominant operational pain: connection storms caused outages in 2020 and 2023, and two 2026 incidents on the hosted service came from schema changes and lock conflicts in deployments.
- Quote: "The primary symptom of the outage was a storm of tens of thousands of database connections, effectively locking our MySQL instance." — S377, Root cause analysis
- Quote: "The worker processes within the Quay pods would have difficulty getting connections. As a result they would restart, requesting yet more connections, eventually causing a thundering herd of requests that would crash our RDS instance." — S378, State of Quay.io pre-migration
- Quote: "This incident was caused by a database change which was improperly rolled out." — S380, Postmortem
- Quote: "Image pulls were intermittently degraded for approximately 30 minutes and image pushes were unavailable for approximately 3 hours." — S381, Postmortem
- Quote: "caused by an unexpected burst of resource-intensive operations that temporarily overloaded our primary database" — S382, Postmortem
- Conditions/notes: three incidents on the hosted service in 2026 (February 18, March 30, May 7); in the March incident pulls were restored within 45 minutes by failing read traffic over.
- Relevance: bound connections and admission per node, and make pulls independent of the write path so that a metadata-store problem degrades pushes before pulls.

### F216. Quay (Q11): Upgrades are a reported source of breakage: a 3.18 schema migration fails on existing duplicate e-mail addresses (open since 2026-10-28), and a 3.8 to 3.9 upgrade lost a PostgreSQL volume when two pods ran migrations concurrently.
- Quote: "Running upgrade c3d4e5f6a7b8 -> b1a79fa8e630, replace user email unique constraint with partial index" — S363, Description (log)
- Quote: "Customer ugprade their quay from 3.8 to 3.9, and found that after upgrade success, the postgresql 10 PV was removed and all data lost" — S364, Description
- Quote: "running multiple Quay pods will mean starting up multiple migrations as well, which will overlap each other and the migration will crash and you'll be left in an undefined state" — S364, Description
- Conditions/notes: schema migrations are Alembic scripts run at start-up; the hosted service's February 2026 postmortem commits to "application update first and database migrations second".
- Relevance: append-only wire and record formats remove this class; an upgrade must never be a stop-the-world schema rewrite.

### F217. Quay (Q11): Storage-path and embedded-database limits surface as push failures: the GCS driver buffered whole layers in memory until 3.15, and the SQLite-backed mirror registry locks its database when workers and requests overlap.
- Quote: "The actual implementation fails when Quay is processing layers bigger than 4GiB because the upload on GCS is not multi-part. Therefore, Quay collects the whole layer in the memory and uploads it entirely on GCS." — S365, Description
- Quote: "When Quay is trying to do incompatible operations on the SQLite database on the same tables at the same time (for instance, {{SELECT/DELETE}}), the database can end up being locked." — S366, Description
- Conditions/notes: the GCS issue was a Blocker with 6 votes, fixed in v3.15.0; the SQLite locking issue (mirror registry v2.0.5, 13 watchers) is named in the next-generation mirror registry epic as a root reason for the Go rewrite.
- Relevance: every driver must stream with a stated memory bound; the small-scale mode must not swap in a different, weaker metadata store.

### F218. Quay (Q12): The server is distributed as Linux container images for amd64, ppc64le, s390x and, since 3.17 (March 2026), arm64; no Windows or macOS server build was found.
- Quote: "platforms: linux/amd64,linux/ppc64le,linux/s390x" — S306, job build-amd64-ppc64le-s390x L123
- Quote: "platforms: linux/arm64" — S306, job build-arm64 L184
- Quote: "{productname} 3.17 now supports the ARM architecture for both standalone deployments and Operator-based installations." — S350, Support for ARM Architecture
- Conditions/notes: base image is UBI 9 with Python 3.12 (Dockerfile L1). Not found: any native Windows or macOS packaging; running on those hosts means a Linux VM.
- Relevance: beat; Linux, macOS and Windows on both architectures from one binary is not offered by this incumbent.

### F219. Quay (Q13): The newest release is v3.18.3 (GitHub release published 2026-11-13; changelog heading dated 2026-11-03); minor releases came about every four months, and the next version is numbered 5, which exists only as draft documentation.
- Quote: "## [v3.18.3] - 2026-11-03" — S307, heading L7
- Quote: "released this 13 Nov 14:52" — S385, release header
- Quote: "## [v3.18.0] - 2026-07-23" — S307, heading L217
- Quote: ":producty: 5 :productminv: v5.0.0" — S348, L20-L21
- Conditions/notes: changelog dates of the x.y.0 releases: v3.14.0 2025-03-18, v3.15.0 2025-07-16, v3.16.0 2025-12-04, v3.17.0 2026-03-24, v3.18.0 2026-07-23. GitHub publication dates differ by days to weeks (v3.18.0 2026-07-31, v3.16.0 2026-01-05, v3.14.0 2025-04-07). On 2026-11-13 patch releases were published for seven maintained lines at once (3.18.3, 3.17.7, 3.16.7, 3.14.10, 3.12.24, 3.10.28, 3.9.28). The draft 5.0 release note says "Issued TBD"; tracker fix versions already name `quay-v5.0.z`.
- Relevance: context for comparisons; cite v3.18.3 as the class-for-class competitor.

### F220. Quay (Q13): Headline changes in 2025-2026 were operational rather than architectural: full-layer caching in the proxy (3.15), the React UI as default and pull statistics (3.16), ARM, immutable tags, sparse manifests and image hardening (3.17), and OIDC and token features (3.18).
- Quote: "With this release, {productname} now caches all layers when an image is pulled from a proxied repository." — S352, Proxy cache enhancements
- Quote: "With this release, the v2, _React_-based UI is now the default UI." — S351, default UI
- Quote: "{productname} now supports immutable tags for organizations and organization-owned repositories." — S350, Support for immutable tags
- Quote: "{productname} now accepts and serves sparse manifest lists where only a subset of referenced architectures such as `x86_64` or `arm64` are stored locally while the original manifest list digest is preserved." — S350, Sparse manifest support for multi-architecture filtering
- Quote: "{productname} 3.18 adds Prometheus metrics and a health API endpoint for repository mirroring." — S346, Repository mirror metrics and health monitoring
- Quote: "With the release of {productname} 3.14, the v2 UI now renders model card information for machine learning models that include a model card." — S353, Model card rendering on the v2 UI
- Quote: "With this release, the {productname} v1 UI (_legacy_ UI or _angular_ UI) is deprecated." — S386, v1 UI deprecation notice
- Conditions/notes: 3.14 also moved Clair's database to PostgreSQL 15; 3.16 added PKCE for OIDC and `FEATURE_IMAGE_PULL_STATS`; 3.17 added organization-level mirroring and architecture-filtered mirroring ("reducing storage and transfer by 50% or more for multi-arch images", a vendor claim without a measurement); 3.18 added multi-issuer OIDC, programmatic OAuth token provisioning (technology preview) and four mirror metrics; the 5.0 draft adds configurable gunicorn timeouts, OIDC superuser group sync, robot API tokens and a token-exchange endpoint.
- Relevance: sparse manifest lists and immutable-tag policies are features users now expect; the Go rewrite (F172) is the significant architectural change.

### F221. Quay (Q8, Q9, Q10): Each uploaded chunk is stored as its own object and the blob is assembled at commit: by server-side multipart copy only if every non-final chunk reaches the minimum chunk size, otherwise the registry process reads all chunks back and re-uploads them as one object.
- Quote: "# We are going to upload each chunk to a separate key" — S294, stream_upload_chunk L544
- Quote: "# If the chunk is both too small, and not the last chunk, we rule out server side assembly" — S294, complete_chunked_upload L658
- Quote: "We are going to turn all of the server side objects into a single file-like stream, and pass that to stream_write to chunk and upload the final object." — S294, complete_chunked_upload L721-L723
- Conditions/notes: the minimum is 5 MiB by default (F206); the registry does not advertise it to clients (no `OCI-Chunk-Min-Length` header anywhere in the Python tree, by search). The Google Cloud Storage driver always takes the read-back path ("Boto does not support GCS's multipart upload API"), which is the mechanism behind the 4 GiB failure in F217. The final commit runs inside the client's PUT request, so its duration grows with blob size.
- Relevance: avoid; a content-addressed store that takes chunks straight into their final erasure-coded form needs no assembly pass, and the minimum chunk length should be advertised with `OCI-Chunk-Min-Length`.

**GitLab Container Registry v4.43.0-gitlab (F222 to F280)**

### F222. GitLab registry (Q1): it is a Go fork of Docker Distribution (now CNCF Distribution) that has detached from upstream, and its architecture is now centred on a relational metadata database built to enable online garbage collection.
- Quote: "we have diverged enough to the point where we decided to detach from upstream and proceed on our own path." — S387, Historical Background
- Quote: "These changes culminated on a new architecture based on a relational metadata database and the original goal, enabling" — S387, Historical Background
- Conditions/notes: Go module path is still `github.com/docker/distribution` (go.mod, go 1.25.8). One `registry` binary plus PostgreSQL; Redis optional.
- Relevance: the closest production precedent for "records in a database, bytes in an object store"; copy the split, avoid inheriting a second (legacy) metadata path.

### F223. GitLab registry (Q1, Q8): with the database enabled, all metadata (repositories, manifests including their payloads, tags, layer links) lives in PostgreSQL and the storage backend is used only for blob bytes.
- Quote: "Instead, metadata will be stored and manipulated on the PostgreSQL database. The storage backend will then be used only for uploading and downloading blobs." — S439, New Architecture
- Quote: "digest bytea NOT NULL, payload bytea NOT NULL," — S398, CREATE TABLE public.manifests L3330-L3352
- Counter: upload sessions are not in the database — "Does not require database interactions. Uploads are [currently](https://gitlab.com/gitlab-org/container-registry/-/issues/427) tracked using the filesystem metadata only." — S412, OCI Distribution API table
- Conditions/notes: manifest GET/HEAD, tag list and blob HEAD need only the database; blob GET needs database plus storage (design document, HTTP API table).
- Relevance: manifest and tag reads never touch object storage; a new registry should keep that property and also move upload-session state out of the byte store.

### F224. GitLab registry (Q1, Q13): the metadata database is still optional in November 2026; since registry v4.40.0 (2026-05-13) and GitLab 19.0 an unset `database.enabled` means "prefer", and the planned removal of legacy filesystem metadata was pulled from its milestone.
- Quote: "default database.enabled to "prefer" instead of "false"" — S391, 4.40.0 (2026-05-13) Features
- Quote: "DatabaseEnabledPrefer the database remains enabled if already enabled OR there is no data detected in the container registry (fresh install)." — S390, type DatabaseEnabled const block L418-L419
- Quote: "In GitLab 19.0 and later, the metadata database is enabled by default in prefer mode for new installations:" — S426, Default configuration
- Quote: "The legacy metadata storage method will remain supported in maintenance mode, but will not receive new features or improvements." — S432, Container Registry legacy metadata storage (announced in GitLab 18.5)
- Quote: "This change has been removed from its original milestone and is being reassessed." — S432, Container Registry legacy metadata storage (announced in GitLab 18.5)
- Counter: in 2024 GitLab announced the opposite schedule — "We will target GitLab 19.0 for the legacy registry to stop being supported after over a year of co-existence." — S442, article body
- Counter: the registry's own configuration reference still states the old default — "When set to `"prefer"`, the metadata database is used on fresh installs and registries already using the database. Defaults to `false`." — S389, database
- Quote: "Require the Database :obd, 2027-05-21, 30d" — S463, Rollout Strategy (roadmap chart, Policy Changes)
- Conditions/notes: the epic's roadmap chart is a plan, not a commitment; it also lists "Enabled by Default" from 2026-05-21, which did happen with GitLab 19.0. Existing installations keep their setting on upgrade; no automatic import.
- Relevance: six years after the design (2020) the vendor still carries two metadata code paths; a new registry should have exactly one.

### F225. GitLab registry (Q1): the database must be PostgreSQL 16.0 or newer with native partitioning, triggers and functions, and the registry refuses to start on an unsupported version.
- Quote: "MinPostgresqlVersion defines minimal Postgresql version supported by the container registry/that is guaranteed to work." — S406, const MinPostgresqlVersion L6-L8
- Quote: "You must pick a provider that supports native Postgres partitioning, triggers, and functions, as the registry makes heavy use of these." — S427, Scaling by component > Database
- Quote: "The registry checks the version at startup and does not start on an unsupported version." — S426, Database requirements
- Conditions/notes: `MinPostgresqlVersion = "16.0"` at v4.43.0-gitlab; GitLab 19.x itself requires PostgreSQL 17.x minimum (S465). Prepared statements default off "for compatibility with PgBouncer" (S389).
- Relevance: the GC design is welded to one database engine's triggers and row locks; a registry on a replicated record store must express the same invariants without them.

### F226. GitLab registry (Q1): Redis is optional and used for caches (repository objects, pre-signed URLs, blob descriptors), the rate limiter and database load balancing; cached keys get a fixed 6 hour TTL.
- Quote: "The registry is currently applying a non-configurable TTL of 6 hours to all cached keys. We intend to fine-tune this value and make it configurable once the feature is considered stable." — S389, redis > cache
- Quote: "redisCacheTTL is the global expiry duration for objects cached in Redis." — S400, const redisCacheTTL L90-L91
- Conditions/notes: Redis is required for repository rename, database load balancing and rate limiting; GitLab's admin docs say enabling the Redis cache "also enables features such as renaming repositories" (S427).
- Relevance: an optional cache tier that silently gates features is an operational trap; state which features need which service.

### F227. GitLab registry (Q1, Q13): storage drivers are filesystem, GCS, S3 (`s3_v2`, AWS SDK v2) and Azure (`azure_v2`); `s3` and `azure` are now aliases, and OSS and Swift were removed in v4.0.0 (2024-04-29).
- Quote: "The `storage` option is **required** and defines which storage backend is in use. You must configure exactly one backend." — S389, storage
- Quote: "In GitLab 19.0, the deprecated s3 driver will become an alias for the `s3_v2` driver, and no manual action will be needed." — S432, S3 storage driver (AWS SDK v1) for the container registry
- Quote: "**storage:** remove OSS and Swift storage drivers" — S391, 4.0.0 (2024-4-29) BREAKING CHANGES
- Conditions/notes: `inmemory` exists "For testing only". The S3 driver was rewritten on aws-sdk-go-v2 in v4.20.0 (2025-04-16); `azure_v2` arrived in v4.15.0 (2025-01-14).
- Relevance: GitLab cut drivers to reduce maintenance burden; a new registry with one native byte store avoids the driver matrix entirely.

### F228. GitLab registry (Q2): CI runs the OCI distribution conformance suite only at spec v1.0.1 (all four workflows enabled); the v1.1 suite is excluded because the referrers API is not implemented.
- Quote: "latest/1.1 spec does not pass tests as we have yet to implement the referrers API" — S395, .oci-conformance-version-matrix L25-L31
- Quote: "OCI_TEST_PULL: 1 OCI_TEST_PUSH: 1 OCI_TEST_CONTENT_DISCOVERY: 1 OCI_TEST_CONTENT_MANAGEMENT: 1" — S395, job oci-conformance L65-L68
- Quote: "1.1.x is allowed to fail given as we have yet to implement the referrers API but 1.0.x must pass." — S451, issue description
- Quote: "Vendor: GitLab Latest software version tested: 16.5 Latest spec version supported: v1.0" — S457, GitLab Container Registry entry
- Conditions/notes: the conformance job runs against a registry with the database enabled and filesystem storage (`config/database-filesystem.yml`). The OCI site lists all four workflows as passing for GitLab at spec v1.0.
- Relevance: a new registry can beat this by passing the v1.1 suite including referrers on every commit.

### F229. GitLab registry (Q2): there is no `/v2/<name>/referrers/<digest>` endpoint; the `subject` field is stored, and referrers (with an artifact-type filter) are exposed only through GitLab's own tag-list API.
- Quote: "RoutePathBlobUploadChunk = "/v2/{name}/blobs/uploads/{uuid}" RoutePathCatalog = "/v2/_catalog"" — S394, const block L16-L22 (complete list of v2 routes)
- Quote: "While the GitLab container registry supports the OCI 1.1 manifest subject field, it does not fully implement the OCI 1.1 Referrers API" — S433, Sign container images with OCI referrer data
- Quote: "Referrer artifact type filter. Comma separated list of media types. Applies only when `referrers` is set to `true`." — S393, List Repository Tags > Request (referrer_type)
- Quote: "These tests are failing because the new OCI spec includes support for the referrers API." — S450, Support for refers API (5/8 failures) > Summary
- Conditions/notes: issue 1205 has been open since 2024-02-26. No `OCI-Subject` response header appears in the Go sources at v4.43.0 (grep, non-test files).
- Relevance: clients fall back to the referrers tag schema against GitLab; implement the referrers endpoint and `artifactType` filtering natively.

### F230. GitLab registry (Q2): until v4.43.0 (2026-10-23) a manifest whose `subject` was not yet present was rejected, which broke `buildx --attest` pushes intermittently; tolerance is now on by default.
- Quote: "the registry rejects an **attestation manifest whose `subject` is not yet present** in the repository." — S461, Summary
- Quote: "**registry:** enable REGISTRY_FF_TOLERATE_MISSING_SUBJECT by default" — S391, 4.43.0 (2026-10-23) Features
- Quote: "This fixes intermittent push failures from clients (e.g. buildkit `--attest`) that push a referrer/attestation manifest concurrently with, or before, its subject." — S420, var TolerateMissingSubject L48-L62
- Conditions/notes: implemented with a `subject_digest` column and a backfill trigger that links the referrer when the subject arrives (v4.42.0, 2026-09-24).
- Relevance: accept referrers before subjects from day one, as the OCI spec requires, and index by subject digest rather than by a foreign key.

### F231. GitLab registry (Q2): manifests are validated against an allowlist of media types and anything else is refused with 400 unless a feature flag is set; media type ids are SMALLINT.
- Quote: "Any manifest or any of its references with a media type outside this list will lead to a `400 Bad Request` response with detail `unknown media type` when trying to push it to the registry." — S397, Supported Media Types
- Quote: "By default, if an unknown media type encountered, the registry, and associated tools, will emit an error." — S420, var DynamicMediaTypes L33-L40
- Conditions/notes: `REGISTRY_FF_DYNAMIC_MEDIA_TYPES=true` adds types on push; the doc advises public registries not to enable it because ids could be exhausted. The changelog shows new artifact types being added by hand (WASM, OpenTofu, cosign, Notation, Fleet) through 2025-2026.
- Relevance: an allowlist makes every new artifact kind a server release; store media types as data with a bounded but large id space.

### F232. GitLab registry (Q2): the database can represent only sha256 and sha512 digests, through a two-digit algorithm prefix; any other algorithm is an error.
- Quote: "Algorithm prefixes are sequences of two digits. These should never change, only additions are allowed." — S396, const block L13-L17
- Quote: "sha256DigestAlgorithmPrefix = "01" sha512DigestAlgorithmPrefix = "02"" — S396, const block L15-L16
- Conditions/notes: no statement or test result was found showing sha512 pushes passing end to end (conformance runs at spec v1.0.1 only).
- Relevance: an append-only algorithm tag in the stored key is a cheap, proven way to stay digest-agile.

### F233. GitLab registry (Q3): the legacy garbage collector is offline mark-and-sweep that needs the registry down or read-only for the whole run; GitLab estimated 64 days for GitLab.com even after a 90% speed-up.
- Quote: "Doing this for a huge registry may require multiple hours/days to complete, during which the registry must remain in read-only mode." — S439, Challenges > Garbage Collection
- Quote: "Even with the performance improvements, we estimated a staggering 64 days to run garbage collection for GitLab.com." — S441, Optimizing the container registry code
- Quote: "If the command stops during the mark phase, it starts again from the beginning." — S427, Garbage collection for large registries
- Quote: "The time required to perform garbage collection is proportional to the container registry data size." — S427, Container registry garbage collection
- Conditions/notes: the 64-day figure is an estimate for a registry of petabytes on GCS, published 2023-12-04 without hardware detail. `gitlab-ctl registry-garbage-collect` stops the registry for the run. GitLab's fork adds a `parallelwalk` option for S3 and GCS, tolerance of invalid link files and a freed-storage estimate to the upstream collector (S392); none removes the read-only requirement.
- Relevance: the reference number for "why stop-the-world GC cannot work at scale".

### F234. GitLab registry (Q3): online GC replaces whole-registry mark-and-sweep with one small review task per event that might have created garbage, so cost follows the write rate and not the registry size.
- Quote: "Instead of analyzing all artifacts, we can focus on a single artifact at a time, dividing to conquer and ensuring that GC scales efficiently as the registry size increases;" — S444, Rationale > Benefits
- Quote: "As the *mark* phase run time increases, so do the chances of false positives and race conditions if we need to keep the registry in read/write mode." — S444, Rationale > Mark and sweep drawbacks
- Counter: the price is work that finds nothing — "A GC task is needed for every request that might generate garbage, so there will be a fair number of tasks that will not result in any deletion." — S444, Drawbacks and Mitigation Strategies
- Conditions/notes: proposal dated 2020-08-21; implemented and running on GitLab.com since the 2021-2022 migration.
- Relevance: copy the event-driven shape; budget for the review workload being proportional to pushes.

### F235. GitLab registry (Q3): tracking uploads and tracking potentially dangling objects are done by PostgreSQL triggers, and only the final review-and-delete stage runs in the application.
- Quote: "As described in this document, stages 1 and 2 are operated by database triggers and not by the application (API)." — S388, introduction
- Quote: "We are also not prone to application crashes or bugs that could lead to untracked artifacts." — S388, introduction
- Counter: the authors name the cost of trigger-based tracking — "As a downside, observability is not as good as it could be if done at the application level, so special attention must be paid to new triggers and how they perform after being created." — S388, introduction
- Conditions/notes: triggers fire on insert into blobs, manifests, layers and on delete from manifests, layers, tags, manifest_references, plus tag update (switch).
- Relevance: the invariant to copy is "the same atomic write that changes a reference also enqueues the review"; a record store gives that with a transaction, no triggers needed.

### F236. GitLab registry (Q3): candidates wait in two review-queue tables with a `review_after` time that defaults to one day for every event, plus jitter; a re-queued object has its review postponed, not duplicated.
- Quote: "The default for this column is 1 day for both queues and all kinds of events, but it is possible to set a different default value through application settings." — S388, Review Delay
- Quote: "To avoid scheduling multiple tasks to the exact same time, the `gc_review_after` functions adds a jitter of up to 60 seconds to every returned duration." — S388, Review Delay
- Quote: "(random() * (60 - 5 + 1) + 5) * INTERVAL '1 second' INTO jitter_s;" — S398, FUNCTION gc_review_after L29-L51
- Quote: "When an event adds the same blob or manifest to the queue, the `review_after` of the existing task is updated, which postpones the review. No duplicate task is created." — S426, Some tasks are old
- Conditions/notes: the SQL gives a jitter between 5 and 61 seconds, slightly different from the prose. `gc.reviewafter` (default 24h, `-1` for none) rewrites the per-event defaults at boot.
- Relevance: one bounded queue entry per object with a postponable deadline is a good shape; the 24h value is a policy choice (see F247).

### F237. GitLab registry (Q3): to make "is this blob referenced anywhere" a keyed lookup, references are mirrored into two global tables that are not partitioned by repository.
- Quote: "doing so would require scanning multiple (perhaps entire) tables on the database every time the garbage collector runs." — S388, Tracking blob and manifest uploads
- Quote: "The garbage collector is the only one with visibility across repositories (using the `gc_blobs_configurations` and `gc_blobs_layers` tables) and determines if any repository no longer references a blob." — S388, Tracking dangling blobs and manifests
- Conditions/notes: `gc_blobs_configurations` and `gc_blobs_layers` are hash-partitioned by digest (64 partitions each), like `blobs`.
- Relevance: a digest-keyed reverse index (blob to referencing manifests) is the core data structure of online GC; design it into the record schema.

### F238. GitLab registry (Q3): workers pop one task at a time with `SELECT ... FOR UPDATE SKIP LOCKED LIMIT 1` inside a transaction (10 s timeout), delete from storage first (5 s timeout), then from the database; failures back off exponentially.
- Quote: "A `SELECT FOR UPDATE SKIP LOCKED` statement should be used to synchronize multiple concurrent GC processes consuming the review queues, locking and popping records for review within a transaction, one at a time:" — S388, Consuming the review queues
- Quote: "It consumes tasks from the blob review queue, identifies if the corresponding blob is eligible for deletion, and if so, deletes it from storage and database backends, in this order." — S403, type BlobWorker L28-L30
- Quote: "The worker run is canceled if this timeout is exceeded to avoid stalled or long-running transactions. Defaults to `10s`." — S389, gc (transactiontimeout)
- Counter: the specification and the code disagree on the storage timeout default — "This timeout will be configurable and default to 2 seconds." — S388, Blobs > Race conditions L1025
- Conditions/notes: code has `defaultStorageTimeout = 5 * time.Second` (S403 L18) and `defaultTxTimeout = 10 * time.Second` (S402 L27); a failed task is postponed by `5 * time.Minute` doubling up to `24 * time.Hour` (S402 L150-L151).
- Relevance: the worker holds a database transaction and row lock across a remote object delete; a new design should not hold a record lock across byte-store I/O.

### F239. GitLab registry (Q3, Q10): each registry process runs one blob worker and one manifest worker that sleep 5 s between runs by default, so default throughput is about one task per 5 s per worker per instance; GitLab tells operators to drop to 1 s or 500 ms to drain backlogs.
- Quote: "Online GC is composed by a pair of background processes (workers) in every registry instance, one for each artifact type." — S468, Background
- Quote: "The Agent starts with a randomized jitter of up to 60 seconds to ease concurrency in clustered environments." — S401, func (a *Agent) Start L121-L127
- Quote: "The default is `5s`, and these can be configured with milliseconds as well, for example `500ms`." — S427, Scaling by component > Online garbage collection
- Quote: "If you are scaling the registry application horizontally with multi-node deployments, online garbage collection automatically scales without the need for configuration changes." — S427, Scaling by component > Online garbage collection
- Conditions/notes: `defaultInitialInterval = 5 * time.Second`, `defaultMaxBackoff = 24 * time.Hour`, `backoffJitterFactor = 0.33` (S401 L25-L31). When idle the sleep grows exponentially to the maximum unless `noidlebackoff` is set. The per-instance rate is my arithmetic from these defaults, not a published figure.
- Relevance: one-at-a-time review with a fixed sleep is simple but slow by default; batch the review and make the rate a budget, not a sleep.

### F240. GitLab registry (Q3, Q11): review queues are never pruned, so every push enqueues reviews that mostly find nothing, and after an import "millions" of reviews load the database.
- Quote: "However, pruning is prone to complex race conditions." — S388, Review queue pruning
- Quote: "After completing a large import, hundreds of thousands or even millions of blobs can be queued for garbage collection review. This is normal." — S426, Post import
- Quote: "This high load is caused by a high number of individual database calls from the online garbage collector processing the queued tasks." — S426, Monitor database performance
- Quote: "These queues constantly have new entries added to them; therefore, these queues never fully clear for an active registry." — S426, GC queue sizes
- Conditions/notes: GitLab added Prometheus gauges for overdue tasks and queue size in v4.29.0 (2025-10-13) and a `gc-stats` command in v4.37.0 (2026-02-17) because raw queue size is not a health signal.
- Relevance: expose "oldest overdue review" as the health metric; size the queue with a stated bound.

### F241. GitLab registry (Q3): the race between a blob existence check and GC deleting that blob is knowingly left unsynchronised; the documented worst case is a failed push that must be retried.
- Quote: "For all these reasons, we decided not to implement this explicit locking from the start." — S388, Blobs > Race conditions
- Quote: "In the worst-case scenario, they will lead to an image push retry." — S388, Blobs > Race conditions
- Quote: "as a read-write connection is required to perform a `FOR UPDATE` query, by implementing this strategy, we would lose the ability to prioritize/fallback to read-only database replicas for existence checks." — S388, Blobs > Race conditions
- Conditions/notes: at v4.43.0 the API takes locks only on the manifest review queue (calls to `FindAndLockBefore` / `FindAndLockNBefore` in registry/handlers/manifests.go); no API path locks `gc_blob_review_queue`.
- Relevance: a HEAD-then-reference window is the classic online-GC race; close it by making "reference this blob" a conditional write that fails or revives, not by trusting timing.

### F242. GitLab registry (Q3): manifest races are closed with explicit row locks: before deleting or creating a tag, deleting a manifest or creating a manifest list, the API locks any review record due within one hour and aborts after 5 s (a 503 for tag creation).
- Quote: "Checking for records to be reviewed one hour ahead ensures that we do not lock a record unless it will be reviewed (and possibly deleted) soon." — S388, Manifests > Race conditions > Deleting the last referencing tag
- Quote: "If this sensible timeout is exceeded, abort the tag creation and let the client retry. This will bubble up and lead to a 503 Service Unavailable response." — S399, func dbTagManifest L863-L866
- Quote: "manifestTagGCReviewWindow = 1 * time.Hour manifestTagGCLockTimeout = 5 * time.Second" — S399, const block L823-L826
- Quote: "Not using the same transaction for both operations in this situation would result in a deadlock." — S388, Deleting the last referencing tag
- Conditions/notes: five named races are specified (last tag deleted, last manifest list deleted, tag created for untagged manifest, list created over unreferenced manifest, referrer created for unreferenced subject), each with a sequence diagram.
- Relevance: a ready-made checklist of the manifest-level races any online GC must prove closed.

### F243. GitLab registry (Q3): a manifest counts as referenced when it has a tag, is a child of a manifest list, or is the `subject` of another manifest; deleting a subject cascades to its referrers.
- Quote: "The association is stored as `manifests.subject_id` with a foreign key (`fk_manifests_subject_id_manifests`) that has `ON DELETE CASCADE`: if `S` is deleted, every referrer `R` is deleted with it." — S388, Creating a referrer referencing an unreferenced subject
- Quote: "the referenced manifest should also be pushed to the GC manifest review queue, as the manifest being deleted may be the last reference preventing the referenced manifest from becoming dangling." — S388, Deleting a manifest > Atomic manifests
- Quote: "INSERT INTO gc_manifest_review_queue (top_level_namespace_id, repository_id, manifest_id, review_after, event) VALUES (NEW.top_level_namespace_id, NEW.repository_id, NEW.id, gc_review_after ('manifest_upload'), 'manifest_upload');" — S398, FUNCTION gc_track_manifest_uploads L186-L195
- Conditions/notes: the `IsDangling` query unions `tags`, `manifest_references` (child) and `manifests` where `subject_id` equals the candidate (S410 L364-L402). Every uploaded manifest is queued for review and the check has no exemption for a manifest that declares a subject, so an untagged referrer that no index lists looks dangling after the delay; that is my reading of the trigger and the query, not a documented statement. (A client that falls back to the OCI referrers tag schema creates a tagged index over its referrers, which would keep them referenced.)
- Relevance: decide explicitly which direction keeps what alive (subject keeps referrers, or referrers keep subject) and write it as a requirement; GitLab's choice is easy to get wrong.

### F244. GitLab registry (Q3, Q11): open data-loss bug (reported 2026-09-02): manifest and layer rows are inserted in separate auto-commits, so an interrupted push leaves layers with no GC anchor and online GC deletes them a day later; re-pushing does not repair it.
- Quote: "The unassociated blobs have no `gc_blobs_layers` anchor, and the online GC deletes them once their review task is due (1 day by default)." — S449, Summary
- Quote: "A retried `PUT` of the same manifest finds it by digest and returns `201` without touching `layers`, so the hole is permanent." — S449, Summary
- Quote: "Same root cause as #687 (closed as stale in 2025-08 without a fix)." — S449, Summary
- Quote: "Pulls will error out since the layers will not be found, while subsequent pushes will apparently succeed, but only due to the fact that if the manifest is found on the database, it is assumed to be in a healthy state." — S466, Context
- Conditions/notes: reproduced by the reporter on master at v4.41.0-gitlab with PostgreSQL 17. At v4.43.0 `dbPutManifestV2` still calls `CreateOrFind` and then `AssociateLayerBlob` per layer on the primary connection outside a transaction (S399 L1108-L1140). First reported in 2022 (issue 687).
- Relevance: the manifest record and all of its blob references must be one atomic write, and a re-push must verify and repair references; test it with crash injection between every write.

### F245. GitLab registry (Q3): a referrer race was fixed only in v4.43.0 (2026-10-23): GC could delete a subject between the API's lookup and the referrer insert, and the cascade then silently removed a referrer the client had been told was stored.
- Quote: "The `DELETE` goes through and the cascade deletes `R`. The referrer the client just pushed is silently gone." — S388, Creating a referrer referencing an unreferenced subject
- Quote: "lock subject GC review row when pushing a referrer" — S391, 4.43.0 (2026-10-23) Bug Fixes
- Quote: "take manifest subject ID references into account during online GC" — S391, 4.20.0 (2025-04-16) Bug Fixes
- Conditions/notes: referrers were added to the schema in 2023-2024; GC did not consider `subject_id` until v4.20.0 (2025-04-16).
- Relevance: every new reference kind (subject, index child, tag) needs its GC race analysed when it is added, not years later.

### F246. GitLab registry (Q3, Q11): online GC has caused production trouble through database lock contention and unexpected foreign-key violations.
- Quote: "we detected many manifest deletions timing out on the DB server side due to a ShareLock contention." — S459, Problem
- Quote: "The database is doing its job to prevent the deletion but it shouldn't even be attempted." — S460, Task
- Conditions/notes: issue 732 names itself the cause of a severity 3 production incident (2022-07); issue 1489 (2025-01) links a Sentry report and a possible earlier incident, and was followed by issue 1591 for blob deletes.
- Relevance: cascading deletes over partitioned tables contend; prefer explicit, bounded deletion steps.

### F247. GitLab registry (Q3, Q9): the one-day review delay exists to protect pushes in flight and to let users re-tag; in practice storage starts to shrink about 48 hours after the last tag is deleted.
- Quote: "By default, the online garbage collector will only start deleting unreferenced layers 48 hours from the time that all tags they were associated with were deleted." — S429, Storage usage not decreasing after deleting tags
- Quote: "This delay ensures that the garbage collector does not interfere with long-running or interrupted image pushes, as layers are pushed to the registry before they are associated with an image and tag." — S429, Storage usage not decreasing after deleting tags
- Quote: "to postpone their deletion as we do for blobs, allowing users to re-tag/re-reference them for a short period before deleting them." — S388, Review queues > Manifests
- Quote: "With the automatic garbage collector, the following are automatically scheduled for deletion in 24 hours if left unreferenced:" — S467, Garbage collection
- Conditions/notes: 48 h is two chained 24 h reviews (manifest, then its blobs). No measurement backs the 24 h value.
- Relevance: a grace period is a correctness crutch for uploads that are not tied to a session; tie blob liveness to the upload session and the delay becomes a pure undo window.

### F248. GitLab registry (Q3, Q11): the two GC modes must never be mixed: an upstream offline GC run against a database-managed store deletes tagged data, so lockfiles in storage record which metadata backend owns it.
- Quote: "commands, such as the one provided by the upstream registry, will delete data associated with tagged images." — S426, Before you start
- Quote: "The lockfiles directory is used to implement a durable communication method for all users of the file system layout." — S417, pathFor comment L69-L72
- Counter: the lockfile logic has its own failure mode — "The result is a registry with no working garbage collection at all, and an error message that actively directs the operator away from the cause." — S462, Summary
- Conditions/notes: lockfiles are `database-in-use` and `filesystem-in-use` under `/docker/registry/lockfiles/`. Enforcement was toggled on and off several times between v4.16.0 and v4.38.0 (CHANGELOG).
- Relevance: bytes whose liveness is known only to an external database are fragile against any other tool; make the byte store itself refuse deletes that the record store did not authorise.

### F249. GitLab registry (Q3, Q10): published results for online GC on GitLab.com are qualitative: "terabytes" deleted per day, and about 35% of new images being untagged and collected; the 40% storage-reduction figure was a goal.
- Quote: "Garbage collection deletes terabytes of data from GitLab.com each day." — S441, Solving the online garbage collection problem
- Quote: "during Phase 1 we're observing that ~35% of newly created images are untagged and then garbage collected" — S446, Migration (edit note)
- Counter: the 40% is a target, not a result — "Reduce total storage consumption for the Container Registry on GitLab.com by 40%." — S447, Success Metrics
- Conditions/notes: no per-task latency, queue depth or reclaimed-bytes series for GitLab.com was found in public sources.
- Relevance: roughly a third of pushed images becoming garbage within a day is a useful planning number for GC capacity.

### F250. GitLab registry (Q4): the registry has no pull-through cache; the upstream proxy mode was removed in v3.72.0-gitlab (2023-05-02) in favour of GitLab's Dependency Proxy, which lives in the Rails application.
- Quote: "However, we are removing the pull-through cache in favor of the GitLab Dependency Proxy, which allows you to proxy and cache container images from Docker Hub." — S432, Container registry pull-through cache
- Quote: "**proxy:** remove pull-through proxy cache feature" — S391, 3.72.0 (2023-05-02) Features
- Quote: "The dependency proxy starts all requests with a HEAD request. If the manifest has become stale, only then is a new image pulled." — S435, How does GitLab know if a cached tagged image is stale?
- Conditions/notes: the Dependency Proxy's support table lists only Docker (Hub) images; its freshness model is a HEAD to upstream on every request. Stampede handling and behaviour when the upstream is down: not found in the page read.
- Relevance: GitLab chose to keep proxying out of the registry; if a new registry does proxying natively, the upstream-HEAD-per-pull freshness model is the baseline to beat.

### F251. GitLab registry (Q4): cross-site replication (Geo) is one-way and event-driven from registry notifications, for disaster recovery only; the metadata database is not replicated and each site needs its own.
- Quote: "Do not push to the container registry on the secondary Geo site, because the data is not propagated to the primary site." — S431, introduction
- Quote: "Geo replication to the container registry uses events generated from registry notifications, rather than by database replication." — S426, Database architecture with Geo
- Quote: "We do not recommend pulling container registry data from the secondary site because it may be stale." — S431, introduction
- Quote: "The container registry metadata database cannot use the default GitLab-managed PostgreSQL database. Each site's metadata database operates independently without replication between them." — S431, introduction (warning)
- Conditions/notes: secondary sites re-pull images when notified; each site has separate object storage and a separate registry database.
- Relevance: the gap a multi-region record store closes: GitLab has no multi-writer or consistent-read story across regions.

### F252. GitLab registry (Q4, Q9): webhook notifications, which drive Geo and usage statistics, are best-effort: a bounded in-memory queue of 3000 events per endpoint that drops when full.
- Quote: "There is no delivery guarantee for notifications ATM, this is best effort." — S409, const DefaultQueuePurgeTimeout comment L32-L33
- Quote: "The maximum size of the notifications queue with events pending for sending. Once the queue gets full, the events are dropped. The default is 3000." — S389, notifications > endpoints (queuesizelimit)
- Quote: "The average event size in memory is around 3000 bytes, and in production, the queue does not normally go higher that 150 on a single pod so this should give enough headroom for most of the cases at the expense of around 8.5 MiB of RAM." — S409, const DefaultQueueSizeLimit L14-L20
- Conditions/notes: queue limits were added in v4.26.0 (2025-08-12); retries default to 10 with exponential backoff; unsent events get 5 s at shutdown ("Value chosen arbitrary").
- Relevance: a rare example of a measured, explained constant; but replication built on droppable events is exactly the silent drop a new registry must refuse.

### F253. GitLab registry (Q5): the registry has no users or RBAC of its own; it verifies short-lived JWTs issued by GitLab, and GitLab's tag protection and immutability rules reach it as regular-expression lists inside the token.
- Quote: "The container registry cannot authenticate users internally, so it validates credentials through GitLab." — S427, Communication between GitLab and the container registry
- Quote: "In GitLab, tokens for the container registry expire every five minutes." — S427, Increase token duration
- Quote: "TagDenyAccessPatterns contains the patterns used to deny access to specific tags." — S415, type Meta L68-L82
- Quote: "tagPatternMaxCount defines the maximum allowed number of inbound tag protection and immutability regexp patterns." — S399, const block L1769-L1773
- Conditions/notes: `auth` providers are `token`, `silly` (development) and none. GitLab.com uses a 15 minute token (S437). Token `nbf`/`exp` checks allow 60 s of clock skew. At most 5 patterns of at most 100 characters each; immutable tags are GitLab Ultimate, generally available in 18.10 (S434). Roles, deploy tokens, job tokens and anonymous pull of public projects are all decided in Rails when it mints the token.
- Relevance: policy carried in the token keeps the registry stateless, but bounds policy size by token size (5 patterns); a native policy store avoids that.

### F254. GitLab registry (Q6): search, listing and UI are GitLab's, fed by a registry-specific `/gitlab/v1/` API that needs the database: repository details with deduplicated size, tag details with filter and sort, sub-repository listing and rename.
- Quote: "This API requires the new metadata database and will not be implemented for filesystem metadata." — S393, introduction
- Quote: "The default page size is 100, and it can be optionally increased to a maximum of 1000." — S393, List Repository Tags > Pagination
- Quote: "Tag name filter. If set, tags are filtered using a partial match against its value. Does not support regular expressions." — S393, List Repository Tags > Request (name)
- Quote: "Container repository tags are in the container registry, so each tag deletion triggers network requests to the container registry. Because of this, we limit the number of tags that a single API call can delete to 20." — S438, Container Repository tag deletion limit
- Conditions/notes: tags can be sorted by name or published date, either direction, with `last`/`before` markers; `/v2/_catalog` pages default to 100 and are capped at 1000 (S404 L26-L27); rename is limited to 3000 sub-repositories (S405 L95, raised from 1000 to 2000 to 3000 during 2026). CVE scanning is a separate GitLab feature; nothing in the registry.
- Relevance: the useful listing API is non-standard and vendor-private; a new registry should offer filter, sort and size through a documented API of its own.

### F255. GitLab registry (Q6, Q10): with only the OCI API, showing one page of tags cost `1+3N` registry requests; the database-backed tag-list endpoint returns the same in one.
- Quote: "it requires doing `1+3N` network requests against the registry, where `N` is the number of tags on each GitLab UI/API tags list page:" — S454, Problems > Performance
- Quote: "We can replace the `1+3N` requests against the registry with a single one, and get an accurate publish timestamp AND major performance improvement for free." — S454, Solution
- Quote: "The video below shows the wait time to show 15 tags. If I show tags for image which has less tags, the wait time seems to be proportionately less." — S469, issue description (title: taking ~10 seconds)
- Conditions/notes: no before/after latency numbers are given. The 2020 GitLab.com issue (about 10 seconds for a 15-tag page on the legacy path) carries a later banner saying it "has been solved in the new version of the GitLab Container Registry", without a new measurement.
- Relevance: record the publish time and manifest digest with the tag so a tag page is one range read.

### F256. GitLab registry (Q6, Q10): repository size calculation is the heaviest database operation; for some namespaces the query exceeds the 15 s timeout and the registry falls back to a coarser estimate.
- Quote: "there are a few namespaces for which the query still takes way more than 15 seconds (timeout) to execute, making it unfeasible to operate online." — S413, storage usage caveats L154
- Quote: "This endpoint includes functionality for storage usage calculations - the most CPU-intensive DB operation." — S412, API Request Split > GitLab API table
- Quote: "This will ensure that the default statement timeout (e.g. 15 seconds for GitLab.com) does not apply and cause the statement to fail." — S464, statement timeout guidance for migrations L104-L106
- Conditions/notes: results are cached 5 minutes and a timeout is remembered for 24 hours (S421 L46, L449).
- Relevance: deduplicated size computed at read time does not scale; maintain it incrementally at write time.

### F257. GitLab registry (Q7): trust support is display only: GitLab shows cosign signatures attached through the OCI `subject` field (since GitLab 17.1); no verification or pull-time enforcement was found.
- Quote: "When viewing a container image's tags, you see an icon displayed next to each tag that has an associated signature." — S433, Container image signatures
- Quote: "Add support for Notation signature media type" — S391, 4.37.0 (2026-02-17) Features
- Conditions/notes: requires Cosign v2.0 or later and, for self-managed, the metadata database. SBOM media types (SPDX) are in the allowlist; SBOM handling beyond storage: not found.
- Relevance: enforcement at pull is an open field among forks of Distribution.

### F258. GitLab registry (Q8): eight tables are hash-partitioned into 64 partitions each, blobs by digest and repository-scoped tables by top-level namespace, so that lookups always carry the partition key.
- Quote: "we decided to partition manifests and tags by repository and blobs by digest, ensuring that lookups are always performed by partition key for optimal performance." — S439, Database > Partitioning
- Quote: "FOR VALUES WITH (MODULUS 64, REMAINDER 63);" — S398, partition definitions L6300-L7824
- Counter: the shipped schema partitions repository-scoped tables by top-level namespace, not by repository — "PARTITION BY HASH (top_level_namespace_id);" — S398, CREATE TABLE public.manifests L3352
- Conditions/notes: partitioned by digest: `blobs`, `gc_blobs_configurations`, `gc_blobs_layers`; by `top_level_namespace_id`: `layers`, `manifest_references`, `manifests`, `repository_blobs`, `tags`. No reason is given for the number 64. Repository name and path are limited to 255 characters by CHECK constraints.
- Relevance: partition keys chosen from the request path (namespace, digest) are the precedent for key design in a record store.

### F259. GitLab registry (Q8): bytes keep Distribution's content-addressed layout under `/docker/registry/v2`, deduplicated registry-wide and linked per repository; a cross-repository mount is a metadata link that needs pull scope on the source.
- Quote: "In the registry storage backend, image configurations and layers are stored as blobs, deduplicated, and shared across repositories." — S388, introduction
- Quote: "Authorization is checked *before* the mount is attempted, so a token lacking `<source>:pull` yields `401 insufficient_scope`, never a `202`." — S414, The three outcomes of a mount attempt
- Quote: "Mounted. The layer now belongs to the target repository too; zero bytes transferred." — S414, The three outcomes of a mount attempt
- Conditions/notes: with the database, per-repository links are rows in `repository_blobs`; deleting a blob row cascades to those links. A stale mount source yields 202 and a normal upload.
- Relevance: the mount contract (201, 202 fallback, 401 before any lookup) is worth copying verbatim.

### F260. GitLab registry (Q9): manifest size is capped twice: a fixed 4 MiB read limit in the handler and an optional `payloadsizelimit` that is off by default but set to 256KB on GitLab.com, where repeated cosign re-signing has pushed a manifest over it.
- Quote: "Limit the size in bytes of a manifest payload. `0` (default) disables limiting the manifest payload size." — S389, validation > manifests > payloadsizelimit
- Quote: "In production, this is set to **256KB**" — S455, Background
- Quote: "Over many pipeline runs, the accumulated signatures push the manifest past the registry's configured payload size limit." — S455, Causes
- Conditions/notes: `maxManifestBodySize = 4 << 20` (S399 L43), no reason given. `referencelimit` (layers, configs, child manifests) also defaults to 0, disabled. The runbook cites incident INC-8389 on 2026-03-12. Config blobs are cached in the database only up to `ConfigSizeLimit = 256000 // 256KB` (S406 L4).
- Relevance: the OCI spec's 4 MiB floor is the practical ceiling here too; any lower operational limit collides with signatures stored as layers.

### F261. GitLab registry (Q9): abandoned uploads are purged by a background loop that is on by default: age 168h, every 24h, first run after a random 0-59 minute delay, and off while the registry is read-only.
- Quote: "Upload directories which are older than this age will be deleted. Defaults to `168h` (1 week)." — S389, storage > maintenance > uploadpurging
- Quote: "jitter := time.Duration(rand.Int()%60) * time.Minute" — S400, func startUploadPurger L2146
- Quote: "Upload purging is enabled by default. Upload purging will be disabled if `readonly` is enabled." — S389, storage > maintenance > uploadpurging
- Conditions/notes: values inherited from upstream Distribution; no reason is given for one week or one day.
- Relevance: an unexplained week-long session lifetime; a new registry should set session expiry from a stated budget and refuse on expiry.

### F262. GitLab registry (Q9): the S3 driver's limits mostly restate S3's own (5 MiB minimum part, 5 GiB maximum part, 1000 keys per list or delete) and add a client-side rate limit of 350 requests per second, "10%" of S3's documented ceiling.
- Quote: "350 is 10% of the requestsPerSecondUpperLimit based on the figures listed in:" — S407, const DefaultMaxRequestsPerSecond L40-L45
- Quote: "DefaultMultipartCopyChunkSize defines the default chunk size for all but the last Upload Part - Copy operation of a multipart copy. Empirically, 32 MB is optimal." — S407, const block L26-L29
- Quote: "MinChunkSize defines the minimum multipart upload chunk size S3 API requires multipart upload chunks to be at least 5MB" — S407, const block L16-L18
- Counter: the default chunk size is too small for large pushes on S3 — "Start with a value between `25000000` (25 MB) and `50000000` (50 MB)." — S430, AWS S3 with the GitLab registry error when pushing large images
- Quote: "It is set to 10000 part pages, which signifies 10000 * 10485760 bytes (i.e. 100GB) of data. This is defined in order to prevent infinite loops (i.e. an unbounded amount of parts-pages)." — S408, const maxListRespLoop L56-L59
- Conditions/notes: `DefaultChunkSize = 2 * MinChunkSize` (10 MiB); `maxrequestspersecond` may be raised to 3500; `DefaultMaxRetries = 5`. The GCS driver's equivalents carry no comments at all (S424 L46-L52: 256 KiB minimum chunk, 5 MiB default, concurrency 50, 150 for deletes, 100 for walks, 5 tries).
- Relevance: "empirically optimal" without a recorded run is the pattern to avoid; the per-driver rate limiter is worth copying.

### F263. GitLab registry (Q9): application-side rate limiting exists (GCRA in a Redis Lua script, keyed by client IP) but is off by default and documented as work in progress.
- Quote: "This feature is currently a work-in-progress and not readily available." — S419, note at top
- Quote: "GCRA is a timestamp-based rate limiting algorithm that tracks the "theoretical arrival time" (TAT) for the next compliant request rather than physically counting tokens." — S419, Algorithm
- Quote: "One of [`IP`] (only IP is available right now)." — S389, Rate limiter > Limiters (match.type)
- Conditions/notes: rate-limiter middleware landed in v4.22.0 (2025-05-19) and the custom GCRA in v4.25.0 (2025-07-17); the configuration example uses 5000 per minute with burst 8000 and a `log_only` mode; a denied request gets 429 with a `TOOMANYREQUESTS` body.
- Relevance: GCRA with one timestamp per key is a compact limiter state; per-repository or per-identity keys are still missing here.

### F264. GitLab registry (Q9): connection-pool and load-balancing defaults: the pool is unbounded unless configured, replica reads are experimental, and read-your-writes is kept by recording the primary's LSN per repository in Redis.
- Quote: "The maximum number of open connections to the database. If `maxopen` is less than `maxidle`, then `maxidle` is reduced to match the `maxopen` limit. Defaults to 0 (unlimited)." — S389, database > pool
- Quote: "If the record exists, and the comparison returns a positive number, the candidate replica is lagging, and the request should be served by the primary." — S412, Primary Sticking
- Quote: "The current 100ms value is a starting point/educated guess that matches the one used in GitLab Rails" — S425, const upToDateReplicaTimeout L32-L37
- Conditions/notes: replica quarantine thresholds `MaxReplicaLagTime = 30 * time.Second` and `MaxReplicaLagBytes = 8 * 1024 * 1024` ("This matches the Rails default"); configuration.go separately declares `defaultDLBMaxReplicaLagTime = 1 * time.Minute`. The LSN key lives 1 hour. Load balancing reached all read endpoints during 2025 (CHANGELOG 4.24.0 to 4.29.0) but the configuration reference still labels it experimental.
- Relevance: per-repository "last write position" as the staleness token is a good fit for a replicated record store that exposes commit positions.

### F265. GitLab registry (Q9): several defaults are admitted in the code to be arbitrary, and one guards a real attack.
- Quote: "defaultDBCheckTimeout is the default timeout for DB connection checks. Chosen arbitrary" — S400, const defaultDBCheckTimeout L86-L88
- Quote: "NOTE(prozlach): prevent slow-loris attack, but be conservative at it" — S416, http.Server literal L168-L172
- Quote: "Leeway is the Duration that will be added to NBF and EXP claim checks to account for clock skew as per https://tools.ietf.org/html/rfc7519#section-4.1.5" — S415, const block L26-L28
- Quote: "minThreads is the minimum value for the maxthreads configuration parameter. If the driver's parameters are less than this we set the parameters to minThreads" — S423, const block L20-L30
- Quote: "minChunkSize int64 = 256 * 1024 defaultChunkSize = 20 * minChunkSize defaultMaxConcurrency = 50 minConcurrency = 25 maxDeleteConcurrency = 150 maxWalkConcurrency = 100 maxTries = 5" — S424, const block L46-L52 (no comments)
- Conditions/notes: `ReadHeaderTimeout: 60 * time.Second` is the only HTTP server timeout set; pre-signed URL expiry defaults to 20 minutes with no reason (S389, redirect). The filesystem driver's concurrency (default 100, floor 25) and the GCS driver's limits are inherited or set without any stated reason.
- Relevance: catalogue entry for "magic constants"; only the notification queue size (F252) and the S3 rate (F262) cite a measurement or a source.

### F266. GitLab registry (Q10): GitLab.com workload in June 2020, used to size the database: 303 database-relevant API requests per second in total, dominated by manifest GET, giving an estimated 740 SELECT and 22 INSERT per second.
- Quote: "| **Total** | | | **303** | **18,169** | **1,090,131** | **26,163,146** |" — S445, Current API Request Rate > Results
- Quote: "| Manifest | `GET` | Download manifest. | 141 | 8,469 | 508,112 | 12,194,681 |" — S445, Current API Request Rate > Results
- Quote: "| **Total** | | **740** | **22** | **0** | **0** |" — S445, Estimated Database Rate > Per-second (SELECT, INSERT, UPDATE, DELETE)
- Quote: "Per-second rate metrics collected twice (averaged) at 2020/06/19 18:00 UTC and 2020/06/25 09:00 UTC from Prometheus, looking at the previous 48 hours (business days only, for a **worst-case** estimate);" — S445, Methodology
- Conditions/notes: columns are per-second, per-minute, hourly, daily. Other per-second rows: blob GET 74, tags GET 52, blob HEAD 21, manifest HEAD 8, blob upload PUT 5, manifest PUT 2, manifest DELETE 0.03, tag DELETE 0.03. Requests ending in 401 (about 16%) and 500 were excluded. Worst-case query counts per request were assumed (for example 3 SELECT per manifest GET).
- Relevance: a real request mix for a large public registry: manifests are fetched about twice as often as blobs, and writes are about 2% of requests.

### F267. GitLab registry (Q10): GitLab.com workload in May 2024: about 95% of registry API requests are reads, about 1500 read requests and about 60 successful write requests per second.
- Quote: "Using GitLab.com metrics as source, **~95%** of the registry API usage are **read requests**" — S412, Background > Motivation
- Quote: "One GET command for each HEAD and GET API request on eligible endpoints. Estimate of ~1500 per second on GitLab.com" — S412, Primary Sticking > Redis
- Quote: "One SET command for each successful POST, PUT, PATCH, and DELETE API request across all endpoints. Estimate of ~60 per second on GitLab.com" — S412, Primary Sticking > Redis
- Conditions/notes: the source queries are dated 2024-05-22 (one-day rate). Until load balancing shipped in 2025, all queries went to one primary with one online replica.
- Relevance: order-of-magnitude target: a single PostgreSQL primary served roughly 1.5k metadata reads per second for a registry of petabytes.

### F268. GitLab registry (Q10): GitLab.com scale markers: 150k to 200k image pushes and about 700k API events per day (2020), more than 5 petabytes of registry data (2021), more than one million repositories and about 20 write requests per second (2021).
- Quote: "Each day on GitLab.com, between 150k and 200k images are pushed to the registry, generating about 700k API events." — S439, Usage of the GitLab container registry
- Quote: "At 5+ petabytes, migrating projects to the new registry will be a long process." — S447, Problem to solve
- Quote: "given the low write rate of the registry (~20 req/s at the time of writing) and the large number of known repositories (+1M)" — S446, Migration
- Conditions/notes: the design document dates from 2020-09-29; the epic from 2021-03-01. The GitLab.com runbook mentions a baseline of "roughly 20" registry pods (S456). No binary size, idle memory or CPU figures are published.
- Relevance: context for every other GitLab number in these notes.

### F269. GitLab registry (Q10): the only published offline-GC benchmarks are a 90% reduction after optimising the S3 and GCS drivers, and "2 hours to 93 seconds" for 15k blobs on S3.
- Quote: "We optimized the code for Google Cloud Storage (GCS) and Amazon S3, and saw a 90% reduction in the time it takes to run garbage collection." — S441, Optimizing the container registry code
- Quote: "We see similar performance gains to the optimizations we made for S3, in which we saw GC (for 15k blobs) go from 2 hours to 93 seconds." — S453, Success feels like
- Conditions/notes: no hardware, bucket size or version is given for either number; the second appears in a planning issue (2020-02-04) as the result of the earlier S3 work. The optimisation is the `parallelwalk` option, which "will increase the memory and CPU usage" (S392).
- Relevance: even a 77-fold improvement on a small bucket left GitLab.com at an estimated 64 days (F233).

### F270. GitLab registry (Q10, Q11): moving an existing registry to the database takes from minutes to weeks: published runs range from 8 minutes for 100 GiB on S3 to 970 hours for 609 TiB on S3, of which 12.5 hours were read-only.
- Quote: "| User Registry | 609 TiB | S3 | 344 hours | 12.5 hours | 615 hours | 970 hours |" — S448, Completed Tests and User Reports
- Quote: "| Test Registry| 100 GiB | S3 | 6 min | 11 sec | 1.5 min | 8 min |" — S448, Completed Tests and User Reports
- Quote: "In user reports, step one ran at rates from under 1 TB per hour to over 4 TB per hour." — S428, Pre-import repositories (step one)
- Quote: "For example, one user reported that after deleting unused tags, step one for a 535 TiB registry went from 278 hours to 145 hours." — S426, Reduce the number of tags before you import
- Conditions/notes: columns are step one (online), step two (read-only), step three (online), total. Other rows: 100 GiB GCS 23 min + 2 min + 27 min; 500 GiB GCS 2 hours + 9 min + 2.5 hours; 500 GiB S3 20 min + 90 sec + 6 min; 14 TiB S3 14.32 hours + 19 min + 51.5 hours. Duration tracks the number of tagged images more than bytes. Hardware is not stated; GitLab says to treat the figures as lower bounds.
- Relevance: the cost of having put metadata in the byte store first; also a ready table of walk speeds over S3 and GCS for sizing any importer from Distribution-layout registries.

### F271. GitLab registry (Q10): GitLab.com was migrated repository by repository with a short per-repository read-only window: about 5 seconds per 1k tags, under ten seconds for most repositories, with a 30 minute hard limit.
- Quote: "We can expect the second pass to take around 5 seconds per 1k tags based on our experiments. This is the time a repository would have to remain in read-only mode." — S446, Migration
- Quote: "This period is expected to be less than ten seconds for the vast majority of repositories." — S443, What to expect
- Quote: "We settled on a maximum duration of 30 minutes, time after which Rails will trigger an import cancelation request to force it to stop." — S446, Migration (edit note)
- Conditions/notes: phase 1 (new repositories) ran November 2021 to January 2022; phase 2 (existing repositories) from April 2022; the migration epic closed 2022-12-15. Only tagged images were migrated.
- Relevance: a per-repository two-pass import with a bounded write freeze is the proven pattern for adopting an existing registry's data.

### F272. GitLab registry (Q10, Q11): a user running v4.34.0 with about 500,000 tags in one repository measured 72 seconds for a tag listing, traced to an unbounded "are there more tags" query run on every page.
- Quote: "Tag listing for this largest repository takes much longer than expected (currently 72 seconds)." — S452, Overview
- Quote: "However, the first query has no `LIMIT`, which it probably should. As a result, we see a significantly higher `mean_exec_time`." — S452, Problem
- Conditions/notes: user report from 2026-02-13 (standalone registry, hardware not stated); `pg_stat_statements` showed the existence query at a mean of about 98 ms over 33.8 million calls, against 1.8 ms for the page query. The issue was closed 2026-02-19; I did not identify the fix in the CHANGELOG, and the query at v4.43.0 still has no LIMIT in the lines read (S421 L1464-L1480).
- Relevance: a database does not make listing O(page) by itself; every query on the list path needs a bound, and half a million tags in one repository is a real workload.

### F273. GitLab registry (Q11): the import is the main operational hazard: it needs a read-only window, carries only tagged images, resets tag timestamps to the import date, and cannot be redone from object storage once the database is live.
- Quote: "Metadata import for existing registries requires a period of read-only time." — S426, Known limitations
- Quote: "The metadata import only targets tagged images. Untagged and unreferenced manifests, and the layers exclusively referenced by them, are left behind and become inaccessible." — S426, Enable the database for existing registries (note)
- Quote: "After a successful import, the metadata database is the only copy of your registry metadata." — S429, Error: cannot import all repositories while the tags table has entries
- Quote: "Custom tools or scripts that clean up, copy, or modify registry storage directly are not supported and can cause data loss." — S426, Supported import methods
- Quote: "all objects imported during the pre import step are subject to online garbage collection, and therefore it is important to ensure that the subsequent import step is completed within the configured garbage collector workers" — S411, Options > Pre Import
- Quote: "Running this command ensures that any unreferenced blobs are visible to the online garbage collector, allowing them to be removed from object storage." — S411, Options > Common Blobs
- Quote: "limit the number of tags to retrieve concurrently, only applicable on gcs backed storage" — S422, ImportCmd flags L77
- Conditions/notes: an interrupted import can be resumed since GitLab 18.5 by skipping repositories pre-imported in the last 72 hours; backups must now cover storage and database "as close together in time as possible". A pre-import that is not followed by the final import within the review delay (24h by default) can have its not-yet-tagged objects collected, so GC should stay off until the import finishes; tag lookups run one at a time unless `--tag-concurrency` (1 to 5) is set.
- Relevance: once records and bytes are separate stores, backup, restore and disaster recovery must be defined across both; write that requirement early.

### F274. GitLab registry (Q11, Q13): upgrades in 2026 broke registries in at least two ways: the prefer-mode default started some upgraded registries against an empty database, and the S3 driver alias changed pre-signed URL hostnames so egress filters returned 403.
- Quote: "After upgrading from GitLab 18.x to 19.0, the Container Registry UI shows “Something went wrong while fetching the repository list” for all projects." — S458, Problem to solve (user post, 2026-05-28)
- Quote: "**handlers:** defer router init until metadata DB resolves prefer-fallback" — S391, 4.40.1 (2026-05-22) Bug Fixes
- Quote: "If a proxy, firewall, or secure web gateway allows only `s3.amazonaws.com`, the redirect to the regional hostname is blocked and image pulls fail with `403 Forbidden`." — S430, Error: 403 Forbidden when pulling images after upgrading to GitLab 19.0
- Quote: "No guarantee for registry zero downtime during upgrades on multi-node Linux package environments." — S426, Known limitations
- Conditions/notes: the forum post is a user report (secondary); the link between it and the 4.40.1/4.40.2 fixes is my inference from timing and wording. Before GitLab 18.3 registry schema migrations had to be run by hand on upgrade.
- Relevance: changing a default that selects the source of truth is the riskiest kind of release; a single code path removes the class.

### F275. GitLab registry (Q11): storage-backend problems that GitLab documents: pushes failing with 502 and CPU spikes over HTTP/2 to GCS, and space never returning when the bucket keeps deleted object versions.
- Quote: "This issue occurs when the registry communicates with GCS using the HTTP/2 protocol." — S430, Push failures and high CPU usage with Google Cloud Storage (GCS)
- Quote: "If object versioning or a retention policy keeps deleted objects indefinitely, the provider never frees the space." — S429, Storage usage not decreasing after deleting tags (note)
- Conditions/notes: the documented GCS workaround is to disable HTTP/2 with `GODEBUG=http2client=0`. The scaling guide states "there are no scaling guides which target number of seats or requests per second" (S427).
- Relevance: a registry that owns its byte store can account for reclaimed space itself instead of trusting a bucket policy.

### F276. GitLab registry (Q12): the server is built and shipped for Linux only: GitLab's packages cover amd64 and arm64, and the registry's CI cross-compiles for linux on 386, amd64, arm and arm64; no Windows or macOS server build was found.
- Quote: "GitLab provides Linux packages for the operating systems listed below. We build and distribute packages for these platforms." — S436, Supported platforms
- Quote: "NOTE(prozlach): Distribution builds container registry on some exotic archs like 32bit Arm (older RaspberryPis). Lets make sure that the compilation does not break in an embarrasing way like e.g. integer overflows:" — S418, job build_smoketest L151-L153
- Quote: "GOARCH: ["386", "amd64", "arm", "arm64"]" — S418, job build_smoketest L165
- Conditions/notes: both CI matrices set `GOOS: linux`. The repository publishes no standalone binaries; the registry ships inside GitLab's Linux packages and container images.
- Relevance: Linux, macOS and Windows on two architectures from one binary would be a differentiator.

### F277. GitLab registry (Q13): newest release is v4.43.0-gitlab (2026-10-23); the 2025-2026 line of work is the self-managed database rollout (prefer mode, lockfiles, background migrations, load balancing, rate limiting) plus rewritten S3, Azure and GCS drivers and partial referrer support.
- Quote: "## [4.43.0](https://gitlab.com/gitlab-org/container-registry/compare/v4.42.0-gitlab...v4.43.0) (2026-10-23)" — S391, top entry
- Quote: "**registry:** add gc-stats command to show online gc statistics" — S391, 4.37.0 (2026-02-17) Features
- Quote: "**datastore:** add last_pulled_at column to tags table" — S391, 4.41.0 (2026-07-27) Features
- Quote: "rewrite s3_v2 driver from deprecated aws-sdk-go to aws-sdk-go-v2" — S391, 4.20.0 (2025-04-16) Features
- Conditions/notes: other dated entries: tag immutability 4.16.0 (2025-02-13); rate-limiter middleware 4.22.0 (2025-05-19); minimum PostgreSQL version check 4.24.0 (2025-07-04); custom GCRA and URL cache 4.25.0 (2025-07-17); notification queue limits 4.26.0 (2025-08-12); GC queue metrics 4.29.0 (2025-10-13); prefer mode 4.31.0 (2025-11-05); "tolerate unknown DB migrations during rolling upgrade" 4.34.0 (2026-01-14); Go 1.25 in 4.40.2 (2026-06-23). GitLab itself is at 19.6 (tag v19.6.0-ee, 2026-11-18).
- Relevance: shows where a mature fork of Distribution still spends its effort: migration safety and GC correctness.

### F278. GitLab registry (Q13): rollout history of the database for self-managed users: generally available but off in GitLab 17.3 (August 2024), database provisioned by default from 18.3, prefer mode introduced in 18.7 and made the default for new installations in 19.0 (2026-05-21).
- Quote: "In GitLab 17.3, the new registry will be included, but disabled by default to allow time for planning migrations." — S442, article body
- Quote: "By default, GitLab 18.3 and later preprovisions a logical database for registry metadata in the main GitLab database." — S426, Database requirements
- Quote: "Prefer mode for new Linux package and self-compiled installations introduced in GitLab 19.0. Enabled by default." — S426, history note
- Quote: "the proven core migration functionality, which was used to migrate millions of container images on GitLab.com" — S440, Summary
- Conditions/notes: tag dates: v17.3.0-ee 2024-08-14, v19.0.0-ee 2026-05-21. The design document for the database is dated 2020-09-29; GitLab.com finished migrating in 2022.
- Relevance: four years from "done on the vendor's own service" to "default for new installs" is the cost of a dual-path migration.

### F279. GitLab registry (Q8, Q10): blob downloads are redirected to pre-signed object-store or CDN URLs by default, and since 2025 those URLs can be cached in Redis to cut signing calls.
- Quote: "This is supported and enabled by default for Azure, GCS, and S3 backends." — S389, storage > redirect
- Quote: "This reduces the number of calls to the storage backend for URL generation, improving performance for frequently accessed objects and reduces the load/costs of the underlying IAM service." — S389, middleware > storage > urlcache
- Quote: "storage middleware for caching URLs" — S391, 4.25.0 (2025-07-17) Features
- Conditions/notes: pre-signed URLs expire after 20 minutes by default; the URL cache serves a URL only while at least `min_url_validity` (default 10m) remains. CloudFront and Google CDN middlewares exist; GitLab.com serves blobs from `cdn.registry.gitlab-static.net` (S437). No measurement of the saving is published.
- Relevance: the registry process stays out of the byte path for pulls; a registry that owns its byte store needs an equivalent direct-read path to be comparable.

### F280. GitLab registry (Q11, Q13): the database brought schema-migration operations with it: pre- and post-deployment migrations that had to be run by hand before GitLab 18.3, and a batched background migration framework that the configuration reference still marks experimental.
- Quote: "Prior to 18.3, the registry's regular schema and post-deployment database migrations must be run manually when upgrading versions." — S426, Known limitations
- Quote: "Batched Background Migrations (BBM) are an experimental feature, please do not enable it in production." — S389, database > backgroundmigrations
- Quote: "tolerate unknown DB migrations during rolling upgrade" — S391, 4.34.0 (2026-01-14) Features
- Conditions/notes: the changelog nevertheless ships batched migrations through 2025-2026 (bigint conversions of media type ids over the 64-partition tables, sub-batching in v4.38.0, WAL-archive throttling in v4.26.0). To downgrade a registry after import "you must restore to a backup" (S426, Downgrade a registry).
- Relevance: schema evolution of a partitioned metadata store is a permanent cost; append-only record formats and rolling-upgrade tolerance need to be designed in from the first version.

**olareg v0.2.2; the Rust registries trow, angos, nora and smaller ones; go-containerregistry pkg/registry; cloudflare/serverless-registry (F281 to F350)**

### F281. olareg (Q1): olareg is a minimal Go registry with no external services: each repository is an OCI Layout directory (or lives in memory), and it ships as a container image, a standalone binary or an embeddable Go library.
- Quote: "It is designed around the OCI Layout structure for storing images in a directory. The minimal nature includes avoiding external dependencies, making the project easy to embed in unit tests or deployed in an edge environment as a cache." — S470, README intro L15-L16
- Quote: "Runs as a container, a standalone binary, or embedded within a Go application." — S470, Key Features L21
- Quote: "One of the goals of olareg was to serve as a simple registry implementation to test tooling that works with container registries." — S485, Using in Go Unit Tests
- Conditions/notes: v0.2.2. go.mod lists four direct dependencies (go-yaml, cobra, oci-digest, x/crypto); no database, no cache service. About 6,700 lines of non-test Go.
- Relevance: the smallest complete reference for the OCI API surface; its role (test fixture, edge cache, proving ground for spec changes) is not a production-scale design.

### F282. olareg (Q1, Q8): only the `mem` and `dir` stores are implemented; a `shared` store (one blob store for all repositories) is declared in the config type but not implemented, so every repository keeps its own copy of each blob.
- Quote: "StoreDir // StoreDir tracks each repository with a separate blob store StoreShared // StoreShared tracks the blobs in a single store for less disk usage" — S471, const block Store L26-L27
- Quote: "Correct, not implemented yet. Hard links are a consideration for how to implement it, but that may limit the underlying filesystem." — S487, maintainer reply in the issue thread
- Conditions/notes: `olareg.New` switches on `StoreMem` and `StoreDir` only (olareg.go L50-L55); the `--store-type` flag help reads "storage type (dir, mem)" (serve.go L79). The issue is the most-commented of the four issues the tracker has ever had (GitHub issue search, 2026-11-20).
- Relevance: avoid: per-repository blob stores make cross-repository deduplication impossible without a second mechanism; a content-addressed global blob store gives it for free.

### F283. olareg (Q8): a cross-repository blob mount is served by copying the bytes from the source repository directory into the target one, falling back to a normal upload on any error.
- Quote: "blobUploadMount is a handler for blob mount attempts. Any errors return the error without writing to the response. This allows the registry to fall back to a standard blob push." — S476, func blobUploadMount doc comment L312-L314
- Quote: "// copy content from source repo _, err = io.Copy(bc, rdr)" — S476, func blobUploadMount L351-L352
- Conditions/notes: no authorization check on the source repository was found: the auth handler is invoked once per request, for the target repository with write access (olareg.go L237-L245, L299-L301), so a client allowed to push to one repository can copy a blob it names by digest out of another. The code comment on the caller says "consider allowing anonymous blob mounts" (blob.go L186).
- Relevance: beat: a mount should be a metadata-only reference to an existing content-addressed blob, with an authorization check on the source.

### F284. olareg (Q2): olareg runs the OCI distribution-spec conformance action on every push for the "stable" and "dev" suite versions, and the OCI conformance site lists v0.2.2 against spec v1.1 with 48 of 52 workflows passing.
- Quote: "OCI Conformant, frequently leveraged as an example registry to verify feasibility of future OCI specification changes." — S470, Key Features L20
- Quote: "oci-version: [ "stable", "dev" ]" — S552, job oci-conformance matrix L21
- Quote: "Latest software version tested: v0.2.2 Latest spec version supported: v1.1" — S486, Distribution Spec > Open Source > olareg
- Quote: "48 of 52 workflows passed" — S486, Distribution Spec > Open Source > olareg, Version v1.1
- Counter: the four workflows the site marks as failing for olareg at v1.1 are "API: Blob anonymous mount", "API: Manifest put with tag params", "Data: Tag Param" and "Data: Tag Param sha512" — "API: Manifest put with tag params" — S486, Distribution Spec > Open Source > olareg, Version v1.1
- Conditions/notes: the site now reports per-workflow results (52 for olareg) instead of the four categories pull/push/content discovery/content management, which it still shows for older entries. The tag-parameter items test a feature newer than spec v1.1.1; olareg's release notes say v0.2.0 added "support for pushing digest with tags" (PR 233), so the site's failing mark may reflect a different suite revision: not resolved here. Whether anonymous blob mount is required by spec v1.1 was not checked in this section.
- Relevance: copy: run the conformance suite in CI against both the released and the development suite; track the per-workflow list, since the four-category summary is being replaced.

### F285. olareg (Q2): the referrers API is on by default, filters by artifactType and reports it in `OCI-Filters-Applied`; the response for a subject is a stored index blob that is split into pages above 4 MiB, with split pages held in an in-memory cache.
- Quote: "referrerFilterATParam = "artifactType" referrerFilterATHeaderKey = "OCI-Filters-Applied" referrerFilterATHeaderValue = "artifactType"" — S478, const block L18-L20
- Quote: "Limit int64 // max size of a referrers response (OCI recommends 4MiB)" — S471, type ConfigAPIReferrer L86
- Conditions/notes: each manifest push with a `subject` rewrites the whole referrers index blob for that subject and re-points index.json at it (referrer.go, func referrerAdd L269-L333), so the cost of adding one referrer grows with the number of referrers of that subject. Page cache defaults: 5 minutes, 1000 entries (config.go L14-L15). Fallback-tag referrers found in a preloaded layout are converted on load (store.go, func indexIngest).
- Relevance: copy the API behaviour (filter header, paging by response size); avoid the rewrite-whole-list-per-referrer storage shape.

### F286. olareg (Q2): sha512 digests are accepted for blobs and manifests, and an experimental `digest-algorithm` query parameter selects the algorithm of an upload.
- Quote: "// TODO(bmitch): the digest-algorithm field is EXPERIMENTAL and needs to be adopted by OCI: <https://github.com/opencontainers/distribution-spec/pull/543>" — S476, func blobUploadPost L195
- Quote: "filepath.Join(dr.path, blobsDir, "sha256"), filepath.Join(dr.path, blobsDir, "sha512")," — S475, func (dr *dirRepo) gc L682-L683
- Conditions/notes: the conformance site marks "Data: Blobs sha512" and "Data: Digest Algorithm sha512" as passing for v0.2.2 (S486). A manifest pushed by tag is hashed with the canonical algorithm unless a digest is supplied (manifest.go L275-L279).
- Relevance: copy: the blob path carries the algorithm (`blobs/<alg>/<hex>`), so a second algorithm needs no layout change.

### F287. olareg (Q3): garbage collection is online and in-process: a ticker (default every 15 minutes) collects one repository at a time, and while a repository is being collected new requests to it block and in-flight ones are waited for.
- Quote: "Frequency time.Duration // frequency to run garbage collection, disable gc with a negative value GracePeriod time.Duration // time to preserve recently pushed manifests and blobs, disable with a negative value" — S471, type ConfigGC L53-L54
- Quote: "newCmd.Flags().DurationVar(&opts.gcFreq, "gc-frequency", time.Minute*15, "garbage collection frequency") newCmd.Flags().DurationVar(&opts.gcGracePeriod, "gc-grace-period", time.Hour, "garbage collection grace period")" — S473, func newServeCmd L95-L96
- Quote: "// wgBlock prevents adding to the WG while a wg.Wait is running, GC blocks new requests" — S475, func (d *dir) RepoGet L122
- Quote: "// skip repos that were have not been recently updated" — S475, func (d *dir) gc L251
- Conditions/notes: a per-repository stop-the-world, not a registry-wide one. Each run visits only repositories still in the in-memory repository cache whose last modification falls after the previous run minus the grace period; a repository is also collected when it ages out of that cache (dir.go L71-L82). No recursion: the mark phase uses an explicit work list.
- Relevance: copy the idea of scoping collection to what changed; avoid blocking a repository's readers and writers for the duration of a sweep.

### F288. olareg (Q3): the algorithm is a per-repository mark-and-sweep rooted at index.json: tagged entries (and untagged ones unless `--gc-untagged` is set) and anything younger than the grace period are kept, then every blob file not reached and older than the cutoff is deleted, with deletion errors ignored.
- Quote: "// keep tagged entries or every entry if untagged entries are not GCed" — S474, func repoGarbageCollect L399
- Quote: "// keep recently uploaded blobs (manifests handled above)" — S474, func repoGarbageCollect L502
- Quote: "// attempt to prune from blob store, ignoring errors" — S474, func repoGarbageCollect L510
- Conditions/notes: age is the blob file's modification time (dir.go, func blobMeta L343). Cost per collected repository: read every reachable manifest, list `blobs/<alg>/`, stat each unreached blob. Referrers are kept or dropped with their subject according to `--gc-referrer-subject` (default true) and `--gc-referrer-dangling` (default false). An emptied repository directory is removed (default on).
- Relevance: the grace period by file mtime is the whole protection for a blob uploaded before its manifest; a new registry should state that window as a bound and test the race instead of relying on a default of one hour.

### F289. olareg (Q3): races between GC and other paths have been fixed twice in the release notes (v0.1.2, v0.2.0); no data-loss report was found in the tracker.
- Quote: "Fix: Prevent a race between closing a directory and GC." — S482, Release v0.2.0 > Fixes
- Quote: "Avoid race with GC in tests." — S482, Release v0.1.2 > Fixes
- Conditions/notes: upload sessions are guarded separately: a repository with uploads in progress is not pruned from the repository cache (dir.go L72-L74, error text "uploads in progress"). The issue tracker holds four issues in total as of 2026-11-20, none about lost data.
- Relevance: even a single-process GC with a per-repository lock needed race fixes; plan deterministic-simulation tests for GC against concurrent push.

### F290. olareg (Q4): there is no pull-through cache, mirroring or replication; the only related feature is a read-only mode for serving a pre-populated directory.
- Quote: "// TODO: proxy settings, pull only, or push+pull cache" — S471, type Config L36
- Quote: "# run a read-only server on port 5050 serving content from the mirror directory olareg serve --port 5050 --store-ro --dir mirror/" — S473, serve command examples L60-L61
- Conditions/notes: because storage is a plain OCI Layout, a mirror is populated out of band (for example by copying layouts into the directory); index.json is re-read at most once per second (store.go L22).
- Relevance: an on-disk format that other tools can write (OCI Layout) makes preloading and air-gapped mirrors trivial; worth supporting as an import/export format.

### F291. olareg (Q5): authentication arrived in v0.2.0: HTTP basic auth against a YAML file of bcrypt hashes, or opaque bearer tokens (5-minute lifetime) from a built-in token endpoint; authorization is a list of ACLs (repository path with optional trailing wildcard, access read/write/delete, members, anonymous flag) with longest-match selection.
- Quote: "This token expires after 5 minutes requiring clients to periodically reauthenticate." — S484, Enabling Opaque Token Authentication
- Quote: "When searching for an ACL entry, the longest matching path containing the user or one of their groups is used." — S484, Auth Config File Syntax
- Quote: "pollFreq = time.Second * 5 tokenExpire = time.Minute * 5" — S480, const block L36-L37
- Conditions/notes: no OIDC, no robot accounts, no JWT: opaque tokens live in a process-local map (auth.go, type authController L90), so several replicas would not accept each other's tokens. The docs give the reason for the token mode: password hashing happens once per token lifetime instead of once per request.
- Relevance: the ACL shape (path prefix, verbs, members, anonymous) is a workable minimum for per-repository policy; opaque in-memory tokens do not fit a multi-node design.

### F292. olareg (Q6): no catalog endpoint, search API, UI or scanner integration exists; the tag list handler loads the repository index, filters and sorts all tags on every request, and applies `n`/`last` with no default page size.
- Quote: "sort.Strings(tl.Tags) n := r.URL.Query().Get("n") if n != "" { if nInt, err := strconv.Atoi(n); err == nil && len(tl.Tags) > nInt {" — S479, func tagList L47-L50
- Conditions/notes: `grep -rn _catalog` over the non-test Go files of v0.2.2 returns nothing; the router in olareg.go has no catalog route. Tag listing cost is O(tags log tags) per request and unbounded in response size when `n` is absent.
- Relevance: beat: serve tag pages from an ordered index with a default and a maximum page size.

### F293. olareg (Q7): the server performs no signature verification and enforces nothing at pull; signatures and SBOMs are stored as ordinary referrers. The project signs its own binaries and images with cosign.
- Quote: "Binaries and images have been signed with cosign." — S485, Verifying Signatures
- Conditions/notes: no cosign/notation verification code in the tree (search for "cosign" finds only release tooling and the v0.2.0 note "Add support for cosign v3 bundles", RELEASE.md). Release assets include `.sigstore.json` bundles and SPDX/CycloneDX SBOMs per binary (S488).
- Relevance: n/a for enforcement; copy the release hygiene (signed binaries, SBOM per artifact).

### F294. olareg (Q8): the on-disk layout per repository is `oci-layout`, `index.json`, `blobs/<alg>/<hex>` and `_uploads/`; index.json is the only metadata and is rewritten through a temporary file and a rename on every change, with no fsync anywhere in the tree.
- Quote: "freqCheck = time.Second indexFile = "index.json" layoutFile = "oci-layout" blobsDir = "blobs" uploadDir = "_uploads"" — S474, const block L22-L26
- Quote: "err = os.Rename(fh.Name(), filepath.Join(dr.path, indexFile))" — S475, func (dr *dirRepo) indexSave L629
- Conditions/notes: `grep -rn '\.Sync()'` over v0.2.2 returns no call, so durability after a crash depends on the filesystem. Every tag push rewrites the whole index.json, so write cost grows with the number of manifests in the repository. Tags are `org.opencontainers.image.ref.name` annotations on index entries.
- Relevance: avoid a single rewritten index file per repository at scale; copy the layout as an interchange format only.

### F295. olareg (Q9): defaults: manifest body at most 8 MiB (413 above it); referrers response 4 MiB; at most 1000 concurrent upload sessions per repository, with the least-recently-used sessions evicted on overflow; an upload session expires when unused for the GC grace period (1 hour); there is no blob size limit.
- Quote: "manifestLimitDefault = 1024 * 1024 * 8 referrerCacheExpireDefault = time.Minute * 5 referrerCacheLimitDefault = 1000 referrersLimitDefault = 1024 * 1024 * 4 repoUploadMaxDefault = 1000" — S471, const block L13-L17
- Quote: "RepoUploadMax int // limit on number of concurrent uploads to a repository, unlimited with a negative value" — S471, type ConfigGC L55
- Quote: "if r.ContentLength > s.conf.API.Manifest.Limit { w.WriteHeader(http.StatusRequestEntityTooLarge)" — S477, func manifestPut L221-L222
- Quote: "Package cache is used to store values with limits. Items are automatically pruned when too many entries are stored, or values become stale." — S481, package doc L1-L2
- Conditions/notes: overflow behaviour for uploads is eviction, not refusal: above the count the cache prunes least-recently-used sessions down to 90% of the limit (cache.go L51, L229-L270), deleting their temporary files; the evicted client's next request is answered 400 BLOB_UPLOAD_UNKNOWN ("upload session not found", blob.go L384-L390). Blob bodies are copied with `io.Copy` without a cap (blob.go L259, L425, L520). No `OCI-Chunk-Min-Length` header is sent.
- Relevance: copy the explicit per-repository session bound; prefer a typed refusal (429) over silently evicting another client's session.

### F296. olareg (Q9): rate limiting is off by default and, when set, counts requests per source IP per second and answers 429 with `Retry-After: 1`; the only HTTP timeout configured is a 5-second header read timeout.
- Quote: "newCmd.Flags().IntVar(&opts.apiRateLimit, "rate-limit", 0, "limit requests per second per source IP")" — S473, func newServeCmd L87
- Quote: "// block, retry after 1 second resp.Header().Add("Retry-After", "1") resp.WriteHeader(http.StatusTooManyRequests)" — S472, func ServeHTTP L183-L185
- Quote: "ReadHeaderTimeout: 5 * time.Second," — S472, func Run L95
- Conditions/notes: the source IP is the first element of `X-Forwarded-For` when present, with no trusted-proxy list (olareg.go L150-L158), so a client can choose its own bucket. No write, idle or request timeouts are set.
- Relevance: avoid trusting forwarding headers without a configured proxy set; state every timeout.

### F297. olareg (Q10): no benchmark or resource-footprint figures are published; the only size data are the release binaries, about 10 MB each.
- Quote: "olareg-linux-amd64 sha256:27de8650edb8d56843fd262ae39836ccd964b32ebaee46451a3be3a73330db46 10.1 MB" — S488, asset list
- Conditions/notes: v0.2.2 assets as listed by GitHub: olareg-linux-amd64 10.1 MB, olareg-darwin-arm64 9.6 MB, olareg-windows-amd64.exe 10.2 MB. Searched README, RELEASE.md, olareg.org and the issue tracker for benchmarks: not found.
- Relevance: a static registry binary of about 10 MB is the footprint floor to compare a single-binary Rust registry against.

### F298. olareg (Q12): release binaries are built for Linux (amd64, arm64, ppc64le, s390x, riscv64), macOS (amd64, arm64) and Windows (amd64), and container images for eight Linux platforms.
- Quote: "ARTIFACT_PLATFORMS?=linux-amd64 linux-arm64 linux-ppc64le linux-s390x linux-riscv64 darwin-amd64 darwin-arm64 windows-amd64.exe" — S483, L5
- Quote: "TEST_PLATFORMS?=linux/386,linux/amd64,linux/arm/v6,linux/arm/v7,linux/arm64,linux/ppc64le,linux/s390x,linux/riscv64" — S483, L7
- Conditions/notes: no Windows arm64 binary. An open issue (276, opened 2026-11-18) reports the linux/arm/v6 image crashing on a Raspberry Pi Zero W (title from the GitHub issue search; body not read).
- Relevance: the only registry in this section that ships a Windows server binary; a portable Rust registry would match it on Windows amd64 and add Windows arm64.

### F299. olareg (Q13): 2026 releases: v0.2.0 (2026-04-13) added authentication, sparse manifests, foreign layers and pushing a digest with tags; v0.2.1 (2026-04-19) changed the YAML library; v0.2.2 (2026-06-24) added riscv64 artifacts and moved the documentation to olareg.org. Delete APIs have been disabled by default since v0.1.1.
- Quote: "Add static basic auth. (PR 174) - Adding config for auth. (PR 181) - Add opaque token auth. (PR 183)" — S482, Release v0.2.0 > Features
- Quote: "Disable delete APIs by default. (PR 106)" — S482, Release v0.1.1 > Breaking
- Conditions/notes: release dates are from the GitHub release feed (v0.1.2 was 2025-02-11, so there was a 14-month gap before v0.2.0). Every release lists a single contributor.
- Relevance: maturity signal: a one-maintainer reference implementation that tracks spec proposals closely.

### F300. trow (Q1): Trow is a Rust (axum) registry meant to run inside each Kubernetes cluster as an image cache; blobs are files on a local volume and, since v0.7.0, registry state is in SQLite. It has no object-storage backend, describes itself as beta, and lists a distributed/HA architecture as not yet done.
- Quote: "Trow saves container image data to file. Currently, we don't have any options to use different storage like S3." — S490, Trow Architecture note 2, L14-L15
- Quote: "Trow now uses a database to store the registry state. Currently no migration is provided, the `data/` directory needs to be wiped." — S492, v0.7.0 (2025-01-28) > BREAKING
- Quote: "Note that Trow is currently beta and you can expect to find rough edges." — S489, Install L33
- Counter: high availability is an unchecked item in the feature list — "- [ ] distributed architecture for HA and scalability _(coming soon)_" — S489, Features L22
- Conditions/notes: v0.10.0 (2026-04-13). Deployed by Helm chart as a StatefulSet with `replicaCount: 1` (charts/trow/values.yaml L5). The v0.7.0 move from files to SQLite shipped without a migration: existing data had to be wiped.
- Relevance: a Rust precedent for the split "metadata in a database, bytes in files"; avoid shipping a metadata-format change without a migration path.

### F301. trow (Q1, Q10): the server runs on a single-threaded Tokio runtime and writes SQLite through a one-connection pool (WAL journal, synchronous NORMAL), with a separate read-only pool.
- Quote: "#[tokio::main(flavor = "current_thread")]" — S494, fn main L97
- Quote: ".synchronous(sqlx::sqlite::SqliteSynchronous::Normal) .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal) .foreign_keys(true);" — S495, fn init_db L7-L9
- Quote: "let writer_conn = SqlitePoolOptions::new() .max_connections(1)" — S495, fn init_db L11-L12
- Conditions/notes: the changelog claims "Separate rw and ro sqlite connection pools (faster response times)" for v0.7.2 and "perf: use SQL triggers to fill a manifest_blob_assoc table" for v0.8.0, both without numbers. SQLite's own documentation says WAL with synchronous=NORMAL may lose the most recent commits on power loss while staying consistent; trow's docs do not discuss it (not quoted here).
- Relevance: beat: one core and one writer cap throughput; no published measurement exists to compare against.

### F302. trow (Q2): CI runs the distribution-spec v1.1.1 conformance action with all four categories enabled, and the README claims conformance; the referrers endpoint exists since v0.7.0 but artifactType filtering is still a TODO. Trow is not listed on the OCI conformance site.
- Quote: "uses: opencontainers/distribution-spec@a139cc423184af6078077b9b7ee336eddbd03f8f # v1.1.1" — S500, job oci-conformance L106
- Quote: "OCI_TEST_PULL: 1 OCI_TEST_PUSH: 1 OCI_TEST_CONTENT_DISCOVERY: 1 OCI_TEST_CONTENT_MANAGEMENT: 1" — S500, job oci-conformance env L110-L113
- Quote: "conforms to the OCI Distribution Specification for registries" — S489, Features L19
- Counter: referrers cannot be filtered by artifact type — "(TODO) artifactType: The type of artifact to list referrers for." — S499, doc comment above get_referrers L30
- Conditions/notes: the conformance site (S486, fetched 2026-11-20) has no trow entry, so per-category results are only what the project's CI job produces; the CI log was not read. Referrers, manifest PUT/DELETE and uploads are refused for proxied repositories (F305).
- Relevance: the artifactType filter and its `OCI-Filters-Applied` header are the part of the referrers API small registries skip.

### F303. trow (Q2): the digest parser accepts sha256 and sha512 names, but content is hashed with SHA-256 only.
- Quote: "if !["sha256", "sha512"].contains(&algo) {" — S506, fn try_from_raw L65
- Counter: the architecture guide states SHA-256 is the only algorithm in use — "All digests at the moment are SHA256 hashes of the content, but note that other algorithms could be used in the future - hence the existence of the `sha256` parent directory." — S490, Upload/Download Tracking and File Layout L44-L46
- Conditions/notes: digest.rs imports only `Sha256` from the sha2 crate (L7). The architecture guide's file-layout section predates the SQLite schema (it still describes a `manifests/` directory with tag files) and is partly out of date.
- Relevance: accepting a digest algorithm in the parser without hashing with it is a conformance trap; test sha512 push end to end.

### F304. trow (Q3): garbage collection is an online background task that wakes every 10 minutes: it deletes upload sessions idle for a day, blobs referenced by no manifest and not accessed for a day, and, when `registry_proxies.max_size` is set and usage passes 80% of it, the least recently accessed proxied blobs.
- Quote: "let mut interval = time::interval(Duration::from_secs(600)); // 10 minutes" — S493, fn watchdog L21
- Quote: "WHERE b.last_accessed < strftime('%s', 'now', '-1 day') AND NOT EXISTS ( SELECT 1 FROM manifest_blob_assoc mba WHERE mba.blob_digest = b.digest );" — S493, fn delete_orphan_blobs L117-L122
- Quote: "let space_available = (limit.bytes() as f64 * 0.8) as usize;" — S493, fn watchdog L38
- Quote: "This feature deletes older proxied blobs if the disk usage goes above the specified limit (LRU for proxied blobs)." — S492, v0.7.2 (2025-02-21)
- Quote: "The current plan is to avoid doing this in a "pause-the-world" GC pass and instead continually ensure the file system is synchronised." — S490, Upload/Download Tracking and File Layout L65-L66
- Conditions/notes: the mark set is not recomputed by walking: a SQLite trigger maintains `manifest_blob_assoc` on every manifest insert (F309), so finding orphans is one indexed query (the code comments tag the passes "(fast)" and "(slow)" without figures). The proxied-blob eviction query takes at most 500 candidates per run (L161). Failure modes visible in the code: the database row is deleted before the file and not atomically with it (L128-L131), and the one-day `last_accessed` threshold is the only guard for a blob whose manifest has not arrived yet. No pass over untagged manifests was found in this file. No GC bug reports were found among the 25 most-commented issues.
- Relevance: copy: keep the blob-to-manifest reference set incrementally in the metadata store so GC never has to walk manifests; state the grace window as a bound and order "delete record" and "delete bytes" so a crash between them is recoverable.

### F305. trow (Q4): any upstream registry is proxied without prior configuration, under `f/<registry>/<repository>` or through containerd's `ns` query parameter; every pull of a tag re-checks the upstream digest with a HEAD request, a cached copy is returned when the upstream cannot be reached, and proxied repositories refuse pushes and deletes.
- Quote: "Trow will keep a cached copy and check for new versions on each pull. The check is done via a HEAD request which does not count towards the dockerhub rate limits. If the image cannot be pulled a cached version will be returned, if available." — S491, Proxying other registries L64-L66
- Quote: "Can proxy any registry (no prior configuration needed)" — S492, v0.9.0 (2025-08-06)
- Quote: "The operation is unsupported for proxied repos." — S507, impl Display for Error L44
- Conditions/notes: the refusal is HTTP 405 with code UNSUPPORTED (errors.rs L140). There is no TTL: freshness is the per-pull HEAD. Credentials are per upstream host with longest `path_prefix` match (v0.10.0); private ECR hosts get credentials from the AWS SDK automatically (v0.9.1). Inference from the code, not a documented statement: because any host is proxied by default, a Trow without `--user` relays pulls to every registry it can reach.
- Relevance: copy the `ns` mirror protocol and serve-stale-on-upstream-failure; add an allow-list of upstreams and a bound on revalidation rate.

### F306. trow (Q4): on a cache miss Trow downloads the manifest and all of its blobs, all in parallel, before recording the manifest; a second request for a blob that is already being fetched waits by polling for the temporary file every 200 ms with no deadline.
- Quote: "Fix: when downloading proxied image, wait for all blobs to be successfully downloaded before saving the manifest in the DB." — S492, v0.7.1 (2025-01-29)
- Quote: "let futures = blobs .iter() .map(|l| download_blob(cl, state, ref_, l, local_repo_name)); try_join_all(futures).await?;" — S498, fn download_manifest_and_layers L216-L219
- Quote: "tracing::info!("Waiting for concurrently fetched blob"); while tmp_location.exists() { // wait for download to be done (temp file to be moved) tokio::time::sleep(Duration::from_millis(200)).await; }" — S497, fn write_blob_stream L106-L111
- Conditions/notes: consequences read from the code: the first manifest request for a large image blocks until every layer is on disk; the number of concurrent upstream blob downloads per image is the number of layers (no cap); the stampede guard is a temp-file existence check, which leaves a waiter looping if the temp file is never removed.
- Relevance: beat: coalesce concurrent misses in memory with a bounded wait, cap upstream concurrency, and stream to the client while filling the cache.

### F307. trow (Q5): authentication is one optional username/password pair (Argon2-hashed) exchanged for a bearer token valid for 3600 seconds; there is no per-repository authorization, and access auditing is listed as planned.
- Quote: "Set the username that can be used to access Trow (e.g. via docker login)." — S494, struct Args L76
- Quote: "const TOKEN_DURATION: u64 = 3600;" — S501, L22
- Quote: "token_secret: Uuid::new_v4().as_bytes().to_vec()," — S554, impl TrowConfig fn new L103
- Counter: finer-grained access control is not implemented — "- [ ] full auditing and authentication of image access _(planned)_" — S489, Features L23
- Conditions/notes: the token signing secret is a random UUID generated at start (lib.rs L103), so tokens do not survive a restart and could not be shared between replicas. Without `--user` the registry is open.
- Relevance: n/a as a model; shows the minimum a cluster-local cache gets away with.

### F308. trow (Q7): trust is an admission-control feature, not a signature feature: a Kubernetes validating webhook allows or denies images by reference prefix.
- Quote: "It also integrates with the cluster itself, for example by controlling which images can run in the cluster through a validating webhook" — S505, To other Registries L25-L26
- Conditions/notes: configuration is `image_validation: default / allow / deny` prefix lists (docs/USER_GUIDE.md L146-L157). No cosign or notation verification exists in the tree; no SBOM handling.
- Relevance: enforcement at admission by reference prefix is orthogonal to signature verification at pull; a new registry should not conflate them.

### F309. trow (Q8): the SQLite schema is five tables (blob, manifest, tag, repo_blob_assoc, manifest_blob_assoc); manifest bytes live in the database, layer and config blobs are files named by digest and shared by all repositories, and a trigger extracts each new manifest's layer and config digests into manifest_blob_assoc.
- Quote: "Note: a manifest can reference a non existing blob (eg foreign blobs)" — S502, trigger after_manifest_insert_blob_map L46
- Quote: "perf: use SQL triggers to fill a manifest_blob_assoc table" — S492, v0.8.0 (2025-06-18)
- Conditions/notes: blobs are deduplicated across repositories by construction (primary key `digest`), with repository visibility recorded as rows in `repo_blob_assoc`; deleting a manifest cascades to its tags and association rows (foreign keys with ON DELETE CASCADE). The trigger reads only `$.layers` and `$.config` of the manifest JSON, so an index's child manifests are tracked through `repo_blob_assoc`, not through this table.
- Relevance: copy: global content-addressed blobs plus per-repository grant rows is the layout that makes cross-repository mount a metadata write and GC a query.

### F310. trow (Q9): hard-coded limits: manifest body 4 MiB (answered as MANIFEST_INVALID, with a TODO to return 413); catalog and tag listing default to an unlimited page; graceful shutdown waits 30 seconds; the chart's default memory limit is 256Mi.
- Quote: "const MANIFEST_BODY_SIZE_LIMIT_MB: usize = 4; let manifest_bytes = axum::body::to_bytes(body, MANIFEST_BODY_SIZE_LIMIT_MB * 1024 * 1024)" — S496, fn put_image_manifest L153-L154
- Quote: "TODO: return 413 payload too large" — S496, comment above put_image_manifest L140
- Quote: "let limit = query.n.unwrap_or(i64::MAX as u64) as i64;" — S504, fn get_catalog L31
- Quote: "handle.graceful_shutdown(Some(Duration::from_secs(30)));" — S494, fn shutdown_signal L203
- Quote: "resources: limits: memory: 256Mi" — S503, L126-L128
- Conditions/notes: none of these constants carries a stated reason. No blob size limit, no request timeout and no rate limit were found. The user guide tells operators to lift the ingress body limit for large images (`proxy-body-size: "0"`).
- Relevance: catalogue entry: 4 MiB again for manifests, without justification.

### F311. trow (Q10, Q12, Q13): no benchmark or footprint numbers are published; container images exist for Linux amd64 and arm64 and no standalone binaries are attached to releases; the newest release is v0.10.0 (2026-04-13), after v0.9.0 (2025-08-06) made every registry proxyable by default.
- Quote: "The Trow Registry runs inside the cluster with very little resources, and is simple to set-up so it caches every image." — S489, README intro L8-L9
- Quote: "Trow has builds for amd64 and arm64. Images tagged `latest` or `default` are currently amd64 only." — S491, Multiplatform Builds L211
- Quote: "feat: support `path_prefix` for scoped proxy credentials (#478)" — S492, v0.10.0 (2026-04-13)
- Conditions/notes: "very little resources" is unquantified; the only number is the chart's 256Mi memory limit (F310). The v0.10.0 release page lists source archives only. The build script has an armv7 target (docker/platform.sh); nothing targets macOS or Windows. Most-commented issues (GitHub issue search, 2026-11-20) are from 2019-2021 and concern installation, TLS and "manifest invalid" push errors, plus a 2023 thread about the project's ownership transfer to the Trow-Registry organisation; titles only, bodies not read.
- Relevance: maturity signal: a Rust registry whose issue tracker goes back to 2019 and which is still beta and single-node.

### F312. angos (Q1): Angos is a Rust registry (hyper, no unsafe code) that keeps everything it knows in the storage that holds the images, a directory or an S3 bucket, with no database and no lock service; additional replicas are started against the same bucket.
- Quote: "There is no database or lock service to run beside Angos. Everything it knows lives in the storage that holds your images, a directory or an S3 bucket." — S508, Little to look after L26-L27
- Quote: "Scale by adding replicas: replicas share the same bucket, with nothing to coordinate them." — S508, Little to look after L30
- Quote: "No lock backend and no extra infrastructure is required: reads and writes are lock-free everywhere." — S510, Multi-Replica Deployments L187-L188
- Counter: the lock-free design is recent: until 1.6.0 Angos used a transaction engine with Redis or S3 locks — "The internal transaction engine is removed: `lock_strategy` and `conditional_operations` are accepted and ignored, and `angos scrub` reclaims leftover `.tx-*` keys and repairs any torn legacy write from content." — S514, 1.6.0 > Changed L390
- Conditions/notes: v1.13.0 (2026-11-17); v1.0.0 was 2026-01-01. One binary with subcommands `server`, `worker`, `scrub`, `prune`, `reconcile`, `scanner`. Optional Redis only as a token/key cache. The repository was formerly named simple-registry: https://github.com/simple-registry/simple-registry redirects to project-angos/angos (HTTP redirect observed 2026-11-20; no rename statement found in the changelog).
- Relevance: the closest existing design to "stateless replicas over an object store" in Rust; its coordination-free metadata layout (F313 to F316) is worth studying before choosing between a replicated record store and object-store-only metadata.

### F313. angos (Q8): metadata is a tree of write-once objects: blobs at `v2/blobs/{algorithm}/{hash_prefix}/{hash}/data`, one empty reference key per (blob, namespace, link) under `v2/ref/`, and each tag as an append-only set of entries whose key embeds an inverted millisecond timestamp, so the newest entry wins without any lock, at the price of requiring synchronized clocks.
- Quote: "Writers only append, so concurrent pushes and replicas never contend; last-writer-wins is a property of the key names." — S509, Data Model > Repository Structure L198
- Quote: "Each reference is its own empty, write-once key, so no index object is ever read, merged, or rewritten:" — S510, Blob Index Reference Keys L367
- Counter: correctness of tag order depends on wall clocks — "Tag order is timestamp order, so replicas sharing a backend need synchronized clocks (NTP or equivalent)." — S509, Concurrency Model > Write Coordination L246-L247
- Conditions/notes: blobs are global and content-addressed (deduplicated across repositories); a namespace's right to read a blob is a `{namespace}!own` key, so a cross-repository mount is one key write. Superseded tag entries are moved to a `!hist/` prefix by scrub. The catalog is a listing of `v2/cat/` keys. Pull times are also append-only entries (one extra storage write per recorded pull, off by default).
- Relevance: copy: make every metadata mutation an idempotent write-once key so retries and replicas cannot conflict; avoid: ordering by wall-clock timestamps (a record store with real ordering removes the clock dependency).

### F314. angos (Q8, Q3): a push is a fixed order of unconditional single-object writes (reference keys, then the revision record, then the tag entry and referrer record), so a reader that resolves a tag always finds a complete manifest and a crash between steps leaves only states the collector tolerates.
- Quote: "Registry metadata writes need no transactions and no locks: every record is write-once and idempotent, so a push or delete is ordered waves of unconditional single-object writes." — S510, Write Coordination L480-L482
- Quote: "A crash between waves leaves only legal states: over-approximated references the collector ages out, or a revision with no tag, which is the push-by-digest state." — S510, Write Coordination L485-L487
- Conditions/notes: the only compare-and-set in the system is the job queue's claim key (`If-None-Match: *` on S3, `link(2)` on a filesystem), probed at startup.
- Relevance: copy the write ordering (references before the record that makes them reachable) as the crash-injection invariant: every prefix of a push must be a legal state.

### F315. angos (Q3): garbage collection is online and split in two commands that run beside live servers: a DELETE only removes records and answers 202; `angos scrub` later reclaims unreferenced blob bytes, protected by a grace period (`gc_grace_secs`, default 300 s) and by a run-marker protocol instead of a lock; `angos prune` applies retention policies and reaps stale uploads.
- Quote: "A delete removes records immediately and answers `202 Accepted`; the freed blob bytes wait for the next `angos scrub` sweep to be reclaimed. Schedule scrub periodically or storage only grows." — S511, Deletes and Scheduled Maintenance L18
- Quote: "A collector about to delete blob data publishes a run marker under `v2/gc/` naming the digest range it is working on, re-reads its own marker before the irreversible delete, and expires it a few seconds afterwards rather than removing it" — S510, Write Coordination L490-L493
- Quote: "writer that has just written its reference keys lists `v2/gc/` once: an unexpired run covering one of its digests means back off briefly" — S510, Write Coordination L495-L496
- Quote: "Reclamation grace period, used by the serving process and scrub alike: young keys read as live, the push path re-checks the collector after this long, and gc run markers derive their TTL from it." — S512, Global Options > gc_grace_secs L139
- Quote: "Attempts and jittered backoff for a writer waiting out a collector run; a run only covers one batch, so the wait is short." — S518, const WRITER_BACKOFF_ATTEMPTS doc comment L36-L37
- Conditions/notes: the writer waits at most 5 attempts with 50 ms to 800 ms jittered backoff (gc.rs L38-L40); if still covered it must treat the blobs as being reclaimed. After writing its keys a push re-checks that the referenced blobs still have bytes. Scrub also repairs derivable records, quarantines unknown keys under `_lost_and_found/`, and "takes no age thresholds" other than the grace period; prune's upload window defaults to 1 hour.
- Relevance: the most detailed published design for GC without stop-the-world and without a lock on an object store: marker fencing plus a grace period plus a post-write liveness re-check. A registry with a transactional record store can replace all three with a transaction; without one, this is the protocol to beat.

### F316. angos (Q3): documented failure modes of this GC: safety depends on the grace period exceeding clock skew plus the longest write stall; scrub must run from the same version as the servers because it quarantines or deletes keys it does not recognize; and the 2026 changelog records scrub bugs that deleted live data or leaked storage.
- Quote: "the serving processes must keep a value that exceeds clock skew plus the longest write stall." — S512, Global Options > gc_grace_secs L139
- Quote: "Because scrub quarantines (or with --delete-unknown, deletes) what it does not recognize, run it from the same angos version as the server fleet." — S511, The lost-and-found prefix L57
- Quote: "Scrub orphan manifest deletion no longer removes blobs still referenced by other manifests." — S514, 1.2.0 - 2026-06-03 > Fixed L810
- Quote: "Scrub orphan-blob deletion acquires the blob-data lock and re-checks ownership before deleting, so a concurrent upload cannot lose its bytes." — S514, 1.2.0 - 2026-06-03 > Fixed L811
- Quote: "The transaction engine's body janitor never actually reclaimed orphaned `.tx-bodies/` staging: it mishandled the listing's relative names and always concluded there was nothing to delete." — S514, 1.4.0 > Fixed L638
- Quote: "a provider that does not enforce them degrades to advisory claims, where a claim race may run an idempotent job more than once, and two racing first pushes of an immutable tag may both land" — S510, S3 Backend > Trade-offs L171
- Conditions/notes: the two 1.2.0 fixes concern the earlier lock-based scrub (before the 1.6.0 redesign); no data-loss report against the marker protocol was found in the issue tracker (20 issues in total, GitHub issue search 2026-11-20).
- Relevance: evidence that GC races are the main source of data-loss bugs even in a small, recent, test-heavy Rust registry; a time-based grace period is a liveness assumption that should be replaced by a fact the collector can check.

### F317. angos (Q3, Q10): the cost of a scrub is a walk over every key of both stores in three passes at a default concurrency of 25; no durations are published, and the project's CI gate for GC safety is a scrub run while zot's `zb` benchmark pushes to the same registry.
- Quote: "Scrub streams every key in both stores (blob and metadata), categorizes it by shape, and validates it concurrently, in three ordered passes: tag entries, records and job records, then the blob-index reference keys, then blob data." — S511, What Scrub Does L27
- Quote: "Keys validated concurrently per pass (default 25)" — S511, What Scrub Does options table L48
- Quote: "Only run one scrub instance at a time; scrub is safe alongside live traffic, but concurrent scrubs duplicate work and log noise" — S511, Troubleshooting > Concurrent Runs L325
- Quote: "Scrub races the live benchmark traffic: the grace-gated classify-then-recheck discipline must never reclaim or repair in-flight state, so even under load the walk must report all zeros." — S515, step Run zot benchmark with scrub under live load L306-L309
- Conditions/notes: the CI load is `zb -c 10 -n 100` against one instance on a GitHub-hosted runner (build.yaml L302); it is a correctness gate, not a published benchmark. Scrub cost is O(all keys) per run regardless of how little changed; on S3 that is LIST plus one or more reads per key.
- Relevance: beat: collection work proportional to what changed, not to the size of the store; copy: a "GC under live load must report zero repairs" gate.

### F318. angos (Q4): a repository with upstreams is a read-only pull-through cache: clients cannot push to it; mutable tags are revalidated against the upstream on each pull and immutable tags never; when no upstream answers, cached content is served and uncached content gets 503 rather than 404; a blob miss is fetched from the upstream twice.
- Quote: "When pull-through cache is enabled: - Push operations are disabled - Clients receive 401 Unauthorized - The namespace is read-only" — S513, Write Behavior L255-L258
- Quote: "Mutable tags (e.g., nginx:latest): - Registry checks upstream for updates - If same digest, serves from cache - If different, refreshes cache - If no upstream can answer, serves from cache" — S513, Cache Hit L72-L76
- Quote: "Uncached content answers 503 UPSTREAM_UNAVAILABLE, not 404, so clients retry instead of reporting a missing image." — S513, Upstream Unavailable L90-L91
- Counter: the streaming design doubles upstream traffic on a miss — "On a blob cache miss, Angos fetches the blob twice from the upstream registry: once to stream directly to the client, and once to store in the cache." — S513, Request Flow > Blob Request note L164
- Quote: "cross-replica deduplication: only one worker pulls each distinct blob from upstream, regardless of how many replicas saw the miss." — S519, When should I use this? L33-L34
- Conditions/notes: there is no TTL: freshness is "HEAD on every pull of a mutable tag" versus "never for tags declared immutable" (regex exclusions allowed). Several upstreams per repository form an ordered fallback list; an upstream that answers 404/401/403 is believed, and the cached tag is then not served. Credentials per upstream: basic or mTLS; upstream bearer tokens are cached in memory or Redis. Defaults: connect timeout 30 s, read-inactivity timeout 300 s, at most 5 redirects, 4 concurrent cache jobs. The `ns` query parameter of the distribution spec's registry-proxying section is supported for pulls only.
- Relevance: copy: 503 for "upstream unavailable and not cached", serve-stale, immutable-tag short-circuit, queue-level dedupe of cache fills; avoid: fetching a missed blob twice (tee the stream instead).

### F319. angos (Q4): outbound replication is per repository and event-driven on manifest push and delete, carried by a durable job queue with retry and coalescing; conflicts between peers resolve last-writer-wins.
- Quote: "- Per-repository downstream lists, event-driven on manifest push/delete - Rides the durable job queue for retry, coalescing, and restart survival - Loop prevention via receiver-side no-op suppression; last-writer-wins tag conflict resolution" — S509, Core Components > Replication L88-L90
- Conditions/notes: modes `event+reconcile`, `event-only`, `reconcile-only`; default 4 concurrent pushes per downstream; failing jobs retry 5 times with 100 ms to 10 s backoff and then dead-letter; a worker's claim lease is 60 s (configuration reference L183-L186, L527-L529). Reconciliation with `prune = true` deletes downstream-only tags and is documented as unsafe for active-active peers.
- Relevance: copy: replication as queued, coalesced, idempotent jobs keyed by (operation, target, repository, tag) that re-resolve the tag at execution time.

### F320. angos (Q5): identities come from mTLS client certificates, registry-issued tokens, OIDC (GitHub Actions, Kubernetes or any issuer) and basic auth with Argon2 hashes; authorization is CEL expressions, global and per repository, with an optional webhook, and evaluation errors deny.
- Quote: "Access policies: CEL expressions decide who may push, pull or delete, per repository, with optional webhook authorization" — S508, Features L39
- Quote: "Passwordless CI: OIDC from GitHub Actions, Kubernetes or any issuer, a token service, and mTLS client certificates" — S508, Features L40
- Quote: "- Webhooks fail-closed on timeout/error - CEL policy evaluation errors and non-boolean results deny the request" — S509, Security Design > Fail-Closed Authorization L267-L268
- Conditions/notes: the global access policy defaults to `deny` (configuration reference L197). Users are static entries in the config file; there is no user database, team model or robot-account object: CI identities are OIDC claims matched by policy. Token service lifetime defaults to 3600 s, at most 86400 s. Retention and immutability are also per-repository policies (CEL rules; regex exclusions).
- Relevance: copy: policy as expressions over (identity claims, repository, action) with fail-closed evaluation, instead of a fixed role table.

### F321. angos (Q6, Q7): a built-in web UI browses repositories, manifests, referrers, pull history and image filesystems; scanning is delegated to Trivy or Grype with the report stored as an OCI referrer; no signature verification or pull-time enforcement was found.
- Quote: "Vulnerability scanning: Trivy or Grype on push, the report kept next to the image as an OCI referrer" — S508, Features L43
- Quote: "Web UI: browse repositories, manifests, referrers, pull history and image filesystems" — S508, Features L45
- Conditions/notes: no search endpoint was found in the README or the architecture document; the extension API under `/v2/_angos/` offers listings (doc/reference/api-endpoints.md was not read in full). A search of doc/ for cosign and notation finds only the project's own release-verification commands and an example artifactType; signatures are stored as referrers and never checked. Event webhooks (required, optional or async delivery, HMAC-signed) notify external systems of pushes and deletes.
- Relevance: storing scan reports as referrers of the image keeps them subject to the same retention and replication as the image; worth copying.

### F322. angos (Q2): the project claims OCI 1.1 compliance and runs the distribution-spec conformance binary in CI with every API and data toggle enabled (including referrers, sha512, anonymous blob mount and tag parameters) against four backend variants; it is not listed on the OCI conformance site.
- Quote: "OCI 1.1 compliant: Docker, Podman, containerd and any OCI tool, with referrers for signatures and SBOMs" — S508, Features L36
- Quote: "runs every scenario against the same backend matrix as the conformance suite (`fs-inherit`, `fs-split`, `s3-variable-parts`, `s3-uniform-parts`)" — S516, End-to-end docker workflow scenarios L8-L9
- Quote: "- OCI Distribution Specification v1.1 endpoints, plus two the specification added after it: the `digest-algorithm` upload parameter (end-4c) and the `ns` proxy parameter" — S509, Core Components > Router L46-L47
- Conditions/notes: the toggles are in conformance/conformance-variables (for example `OCI_API_REFERRER=true`, `OCI_DATA_SHA512=true`, `OCI_API_BLOBS_MOUNT_ANONYMOUS=true`, `OCI_API_MANIFESTS_TAG_PARAM=true`); CI results themselves were not read. The S3 variants run against rustfs.
- Relevance: copy: run conformance against every storage backend variant, not once.

### F323. angos (Q9): documented defaults: manifest body 5 MiB; blob upload 100 GiB (413 above); request timeout 3600 s; 64 worker threads; S3 multipart part 50 MiB; S3 operation timeout 900 s with 300 s per attempt and 3 attempts; presigned URL lifetime 1800 s; upload reclaim window 1 hour. Reasons are given for the thread count and the upload window, not for the size limits.
- Quote: "Maximum manifest body size accepted from clients or upstream registries" — S512, Global Options > max_manifest_size L122
- Quote: "Maximum total size of a single blob upload; a larger upload is rejected with `BLOB_UPLOAD_INVALID` (HTTP 413)" — S512, Global Options > max_blob_size L123
- Quote: "Registry operations are likely I/O-bound (network transfers, storage I/O), so more threads than CPU cores typically improves throughput." — S512, Performance Tuning > max_concurrent_requests L659
- Quote: "The `-u` window must exceed the longest push you expect, since a deleting retention policy revokes grant-only blobs older than it." — S511, What Prune Does L75
- Conditions/notes: `max_concurrent_requests` is documented as "Tokio worker threads", with the rule of thumb "Start with 8-16x your CPU core count"; it is not a request admission limit. Full list in the Constants table.
- Relevance: catalogue entry: 5 MiB manifests and 100 GiB blobs, both unexplained.

### F324. angos (Q10): no benchmark numbers are published. Two documented performance choices: on S3, blob GETs and by-digest manifest GETs are answered with a redirect to a presigned URL by default; and completing an upload copies the bytes to the content-addressed key (a second full write on ext4, a server-side copy on S3).
- Quote: "With S3 storage, blob GETs and manifest GETs by digest redirect to pre-signed URLs by default, avoiding proxying data through the registry; a manifest GET by tag is served inline." — S509, Request Flow L133
- Quote: "Completing a blob upload copies its bytes into place, a reflink on XFS or btrfs but a full second write on ext4" — S510, Filesystem Backend > Trade-offs L112
- Quote: "angos-linux-amd64 sha256:ffd6d2de234360171631d333cdf6f3b432cac0e8a1d0e8e724e529841f93c00a 20.5 MB" — S551, asset list
- Conditions/notes: release binary sizes for v1.13.0 as listed by GitHub: linux-amd64 20.5 MB, linux-arm64 18.3 MB, darwin-amd64 18.7 MB, darwin-arm64 16.8 MB. Searched README, doc/, website/ and the changelog for throughput, latency or memory figures: not found.
- Relevance: beat: publish measured numbers; note the upload-completion copy as a cost a content-addressed store with native ingest avoids.

### F325. angos (Q12, Q13): release binaries exist for Linux and macOS on amd64 and arm64 and there is no Windows build; releases are frequent (v1.0.0 on 2026-01-01, v1.13.0 on 2026-11-17).
- Quote: "macOS amd64 (Intel) angos-darwin-amd64 macOS arm64 (Apple) angos-darwin-arm64 Linux amd64 angos-linux-amd64 Linux arm64 angos-linux-arm64" — S517, Standalone Binaries
- Quote: "`prune_orphan_namespaces = false` in `[global]` keeps `angos prune` from clearing the namespaces no configured repository owns." — S514, 1.13.0 > Added
- Conditions/notes: 2026 milestones from the changelog: 1.2.0 (2026-06-03) scrub fixes; 1.6.0 removal of the transaction engine and locks; 1.7 retirement of link-file layouts (pre-upgrade conversion required on 1.6.x). The upgrade guide is 1,061 lines, a sign of how often the storage layout changed in one year.
- Relevance: maturity signal: fast-moving, storage layout changed several times in 2026; treat its design as informative and its stability as unproven.

### F326. angos (Q9, Q8): on S3, upload chunks smaller than the 5 MiB multipart minimum are parked in a per-session staging object and streamed back before the next PATCH, so no minimum chunk size is imposed on clients; an upload session is an object plus a `session.json` record with the committed offset and a hasher checkpoint, kept in shared storage rather than in one server's memory.
- Quote: "bytes below the 5 MiB S3 minimum are parked at a per-session staging key and streamed back ahead of the next `PATCH`, so a multipart session is created only once there are enough bytes to flush a part of at least 5 MiB." — S510, Blob Upload Modes (S3) > Non-Uniform Upload Mode L250
- Quote: "on the known-length path bytes stream frame-by-frame, so memory is essentially constant regardless of blob size." — S510, Blob Upload Modes (S3) > Non-Uniform Upload Mode L252
- Quote: "A blob upload session persists as its assembled `data` object plus one `session.json` record (last activity, committed offset, hasher checkpoint) under `v2/repositories/<namespace>/_uploads/<uuid>/`" — S510, Write Coordination L513-L515
- Conditions/notes: no `OCI-Chunk-Min-Length` header appears in the sources of angos v1.13.0, trow v0.10.0 or nora v1.4.7 (text search); of the registries in this section only serverless-registry advertises chunk bounds (F348). A chunked PATCH without Content-Length buffers up to one `multipart_part_size` (50 MiB) per in-flight request. Completion copies the assembled object to the content-addressed key: one CopyObject up to 5 GB, multipart copy above.
- Relevance: copy: persist the upload offset and the hash state with the session so an upload survives a change of replica; decide explicitly between advertising a chunk minimum and staging small chunks.

### F327. nora (Q1): NORA is a single Rust binary that serves many package formats including Docker/OCI over one port, with no database: files on local disk, or objects in S3 or GCS through the `object_store` crate, are the only source of truth, and it runs as a single instance by design.
- Quote: "There is no database, no JVM, no plugin runtime. The filesystem (or S3) is the only source of truth." — S522, Architecture intro L10-L11
- Quote: "NORA supports exactly two storage backends: local filesystem and object storage. Object storage covers S3-compatible stores and, since v1.0.1, Google Cloud Storage natively; both run through one `ObjectStorage` implementation over the `object_store` crate." — S522, ADR-3: Two Storage Backends L278-L281
- Quote: "No high availability. NORA runs as a single instance with a single RWO volume. This is a design decision, not a missing feature." — S522, Known Trade-offs L467-L468
- Counter: the same paragraph asserts a capacity without a measurement, and the maintainers' own test of the S3 backend contradicts it for small objects (F329) — "a single instance with S3 storage handles thousands of pulls per minute" — S522, Known Trade-offs L469-L470
- Conditions/notes: v1.4.7 (2026-11-18); 1.0.0 was 2026-07-13 (CHANGELOG headings). MIT licence. The README counts eighteen formats, the architecture document "up to 16". In-memory indexes are rebuilt from storage at startup. The repository carries fuzz targets and a Kani workflow.
- Relevance: the one multi-format artifact registry written in Rust; its "filesystem is the database" stance is the opposite pole from a replicated record store, and it gives up HA to keep it.

### F328. nora (Q10): published footprint for v1.4.6: 30.8 MB binary, 62–69 ms from process start to a healthy response, about 18 MB idle RSS; the project states that throughput, latency and memory under load are not yet measured.
- Quote: "| Binary size (`nora-linux-amd64`) | 30.8 MB | 23.4 MB | | Cold start (process start to `/health` 200) | 62–69 ms | 329–554 ms | | RAM (idle, 10 s after start) | about 18 MB (RSS) | about 15 MB (RSS) |" — S521, Measured (v1.4.6) table L15-L17
- Quote: "The release binaries, three runs each, empty local storage, NORA_CURATION_MODE=off, on a 20-core Linux 5.15 host. v1.4.6 measured 17.8–18.1 MB RSS; v1.4.0, re-run the same way for comparison, 14.8–15.5 MB." — S521, Measured (v1.4.6) note L19-L21
- Counter: load figures do not exist yet — "Load-test figures — throughput, request latency, and memory under concurrent load — are not yet measured: the k6 load harness is not implemented (tracked in #693)." — S521, Scope L7-L9
- Conditions/notes: columns are v1.4.6 and v1.4.0. Hardware: a 20-core Linux 5.15 host, release binary, three runs, empty local storage. The file also lists targets that are goals, not measurements (under 100 MB RAM at 100 concurrent pulls; cached Docker pull p95 under 50 ms). GitHub lists the v1.4.7 `nora-linux-amd64` asset at 29.4 MB and `nora-linux-arm64` at 24.9 MB (S532).
- Relevance: the only Rust registry here with recorded footprint numbers and their conditions: about 18 MB idle RSS and sub-100 ms cold start are the figures to match.

### F329. nora (Q10): the maintainer published one load test (2026-04-18, k6, 50 virtual users, 100 KB raw artifacts, 3-node dev Kubernetes cluster): 1,825 requests/s from local disk (p95 16 ms, p99 26 ms) against 131 requests/s with 40% errors from an S3-compatible store, and 2,046 requests/s with an nginx cache in front of S3.
- Quote: "Environment: Dev K8s cluster (3 nodes, K8s 1.31.4), Samsung SSD 870 EVO (ZFS mirror), 50 VU k6 load test, 100KB artifacts." — S530, comment Benchmark Results (18.04.2026)
- Quote: "Local FS 1,825 16ms 26ms 0% 79 MB/s S3 (RustFS) 131 264ms 580ms 40% 5.5 MB/s S3 + nginx cache 2,046 22ms 38ms 0% 201 MB/s" — S530, comment Benchmark Results, table Raw Download Performance (columns: scenario, RPS, p95, p99, error rate, throughput)
- Quote: "S3 without cache is not viable for high-throughput — 14x slower, 40% errors under load (connection pool exhaustion on RustFS)." — S530, comment Benchmark Results > Key Findings
- Counter: the author qualifies the numbers in the next comment — "Treat these numbers as directional, not absolute." — S530, comment Benchmark Caveats and Recommendations
- Conditions/notes: source type is an issue comment by the maintainer, not a recorded benchmark file. Workload: the raw-file download endpoint, not the OCI path; 100 pre-seeded artifacts of 100 KB; k6 ramping to 50 virtual users, 2 minutes sustained; S3 backend RustFS on the same cluster; NORA version not stated (the issue was titled for v0.6.x at the time). CPU and memory of the pods are not given.
- Relevance: a data point for "registry in front of S3 without a cache": small-object reads through an object store at about 131 requests/s with errors is the failure a content-addressed store with a local read path has to avoid.

### F330. nora (Q2): the Docker/OCI endpoint is labelled Distribution Spec 1.1 but documents gaps: no referrers API, no cross-repository blob mount, at most two path levels in a repository name, and large monolithic blob PUTs may fail. No conformance-suite run was found.
- Quote: "| Cross-repo blob mount | POST | — | Not implemented | | Referrers API | GET | — | OCI 1.1 referrers |" — S523, Docker (OCI Distribution Spec 1.1) endpoint table
- Quote: "- Max 2-level image path: `org/image:tag` works, `org/sub/path/image:tag` returns 404 - Large monolithic blob PUT (>~500MB) may fail even with high body limit - No cross-repository blob mounting" — S523, Docker > Known Limitations
- Conditions/notes: a search of the repository's Markdown and workflow files for "conformance" finds no OCI conformance job. sha512 support was not examined.
- Relevance: a multi-format registry tends to implement the pull/push subset of OCI and skip referrers and mounts; full conformance is a differentiator.

### F331. nora (Q3): GC is a background mark-and-sweep that is off by default, runs every 24 hours when enabled, and skips orphans younger than a 7-day grace period; the grace period was added after issue 584 showed that GC could delete a blob pushed after its orphan snapshot, leaving a manifest with a missing layer.
- Quote: "- `NORA_GC_ENABLED` — enable/disable background GC (default: false) - `NORA_GC_INTERVAL` — interval in seconds between GC runs (default: 86400)" — S524, struct GcConfig doc comment L12-L13
- Quote: "7 days — matches docker-distribution's reference default. Must exceed the slowest expected blob→manifest gap (large/resumed pushes of multi-GB images over slow links). Lowering it risks the #584 race for heavy pushes; the only cost of a high value is deferred orphan reclaim." — S524, fn default_gc_grace L41-L44
- Quote: "A blob pushed after the orphan snapshot can be deleted by GC, leaving a manifest that references a missing layer (broken image)." — S529, issue body > Summary
- Quote: "But the write side does not take publish_lock at all, so publish_lock serializes GC-delete vs GC-delete (and vs retention) — never GC-delete vs push." — S529, issue body > Summary
- Conditions/notes: issue opened 2026-06-01, labelled P1 bug, closed. The fix is time-based (file mtime older than the grace period), the same shape as olareg's and angos's. The code comment says 7 days "matches docker-distribution's reference default"; which Distribution setting is meant is not stated (from memory and not verified in this section: Distribution's 168h default is its upload purge age; check against the Distribution section). GC also evicts proxy-cached rpm/deb and Hugging Face files oldest-first when `proxy_cache_max_bytes` is set (default 0, disabled); that cap does not cover Docker blobs.
- Relevance: another independent instance of the same race (mark from a snapshot, sweep later, writer not fenced; compare angos, F316) and of the same patch (a long grace period); the constant here carries its reason.

### F332. nora (Q4): proxied tags are revalidated against the upstream on every pull by default, with an optional staleness window; if every upstream fails the last cached manifest is served and marked stale; missing tags are remembered for 30 seconds; concurrent misses for one key are collapsed into a single upstream fetch; a locally pushed tag is never revalidated.
- Quote: "Tag references on proxied images are revalidated against the upstream on every pull by default (docker.metadata_ttl = -1)." — S523, Docker > Proxy cache and tag freshness
- Quote: "- If every configured upstream fails and `docker.serve_stale = true` (default), the last cached manifest is served with `x-nora-stale: true`." — S523, Docker > Proxy cache and tag freshness
- Quote: "In-memory single-flight coalescing for the proxy cache-miss path (#595)." — S527, module doc L4
- Quote: "Locally pushed (hosted) images are authoritative: a pushed tag is served as-is and never revalidated against an upstream." — S523, Docker > Proxy cache and tag freshness
- Quote: "Default `miss_ttl`: 30 seconds (#1192)." — S533, fn default_docker_miss_ttl doc comment L212
- Conditions/notes: unlike the other registries here, one repository name can be both pushed to and proxied (a pushed tag pins that name's tag to the local copy). Several upstreams form a fallback chain or own namespaces. Docker upstream defaults: timeout 300 s, read timeout 60 s (config/registry/docker.rs L200-L206). Coalescing is in memory, per process. Interrupted upstream blob downloads resume up to 5 times with n x 500 ms backoff (docker.rs L4623-L4625).
- Relevance: copy: single-flight on the miss path, a negative cache with a short TTL, an explicit stale marker header; decide deliberately whether a name may be both hosted and proxied.

### F333. nora (Q9): NORA's limits are unusual in that most carry a written reason: at most 100 concurrent upload sessions (to prevent memory exhaustion; overflow answered 429 TOOMANYREQUESTS with a jittered Retry-After of 3 to 10 s), 2 GB per session, 30-minute session TTL, 4 MiB manifests, a 2048 MB request body limit, a 120 s keep-alive chosen to exceed reverse proxies' idle timeouts, and a 15 s shutdown drain sized to Kubernetes' 30 s grace period.
- Quote: "Max concurrent upload sessions (prevent memory exhaustion) const DEFAULT_MAX_UPLOAD_SESSIONS: usize = 100; Max data per session (default 2 GB, configurable via NORA_MAX_UPLOAD_SESSION_SIZE_MB) const DEFAULT_MAX_SESSION_SIZE_MB: usize = 2048; Session TTL (30 minutes) const SESSION_TTL: Duration = Duration::from_secs(30 * 60);" — S525, consts L494-L499
- Quote: "4 MiB matches common registry limits. const MAX_MANIFEST_BYTES: usize = 4 * 1024 * 1024;" — S525, const MAX_MANIFEST_BYTES L548-L549
- Quote: "Jittered, not fixed: a constant retry window re-synchronizes every refused client onto the same cadence, so the herd returns together and the same subset wins each round." — S525, fn too_many_uploads L519-L521
- Quote: "Seconds an idle keep-alive connection waits for the next request before it is closed. Default 120: above the idle timeout of the reverse proxies in front (traefik keeps idle backend connections 90 s, nginx 60 s), so the proxy, not NORA, closes an idle connection and never sends a request into one NORA is closing." — S526, struct ServerConfig field keep_alive_timeout L50-L53
- Quote: "Default 15: with the scheduler wait (up to 10 s) it fits Kubernetes' default 30 s grace period." — S526, struct ServerConfig field shutdown_timeout L41-L42
- Quote: "2048 // 2GB - enough for any Docker image" — S526, fn default_body_limit_mb L86
- Quote: "4 hours — long enough for slow multi-GB downloads over constrained links, short enough to reclaim disk from orphaned files (crash, cancel, OOM)." — S525, const PROXY_TEMP_MAX_AGE doc comment L695-L696
- Quote: "The most pages of an upstream's tag list one listing reads. A longer list is refused as an upstream failure: a listing that stopped early would look complete (#1218)." — S525, const MAX_UPSTREAM_TAG_PAGES L4978-L4979
- Conditions/notes: the reasons are engineering arguments, not measurements; "enough for any Docker image" for a 2 GB body limit sits oddly with the project's own limitation "Large monolithic blob PUT (>~500MB) may fail" (F330). Other defaults: header read timeout 30 s, 4096 connections, maintenance tick every 30 s with a 300 s stall threshold for liveness.
- Relevance: copy the practice (every constant states its reason and its overflow behaviour is a typed refusal); the values themselves are small-deployment choices.

### F334. nora (Q9): rate limiting is on by default and per client: 1 request/s (burst 5) on auth routes, 200 requests/s (burst 500) on registry writes and 1000 requests/s (burst 2000) on registry reads.
- Quote: "Per client: auth routes by `auth_*`; registry writes (PUT, POST, PATCH, DELETE) by `upload_*`; registry reads (GET, HEAD, OPTIONS, npm's audit POST) by `general_*`." — S528, struct RateLimitConfig doc comment L11-L12
- Quote: "[rate_limit] auth_rps = 1 auth_burst = 5 upload_rps = 200 upload_burst = 500 general_rps = 1000 general_burst = 2000" — S528, struct RateLimitConfig doc example L17-L23
- Quote: "Enable rate limiting (default: true). Set `NORA_RATE_LIMIT_ENABLED=false` to disable." — S528, field enabled L35
- Conditions/notes: the example values equal the defaults returned by the `default_*` functions (L55-L72). No reason is given for the numbers.
- Relevance: catalogue entry; the only registry in this section with a default-on rate limit split by route class.

### F335. nora (Q8): Docker blobs are stored under the image name (`docker/{name}/blobs/{digest}`), so by key layout the same layer pushed to two repositories is two objects, and cross-repository mount is absent, although the benchmark document describes storage as deduplicated.
- Quote: "Build a namespaced storage key for Docker blobs. With namespace: `docker/{ns}/{name}/blobs/{digest}` Without (local push): `docker/{name}/blobs/{digest}`" — S525, fn blob_key doc comment L40-L43
- Counter: the project's own description claims deduplication — "NORA stores raw, content-addressable files, so identical blobs are stored once." — S521, Storage overhead L67-L68
- Conditions/notes: the deduplication claim holds only within one repository name. Manifests are JSON files per reference (`docker/{name}/manifests/{reference}.json`) with `.meta.json` sidecars (docker.rs L52-L66).
- Relevance: avoid: a per-repository blob namespace forfeits the largest storage saving a registry has (shared base layers).

### F336. nora (Q5, Q6, Q7): users come from an htpasswd file, API tokens carry read, write or admin roles, CI can publish keylessly with OIDC, and anonymous Docker pull is a separate switch that is off by default; there is a web UI and in-memory search, no vulnerability scanner, and no verification of image signatures was found.
- Quote: "A web UI in English, Russian and Chinese, API tokens with read, write and admin roles, keyless publishing from GitHub Actions and GitLab CI with OIDC, rate limits, an audit log and Prometheus metrics." — S520, Why NORA L30
- Quote: "Allow anonymous Docker/OCI pull (`docker pull` without `docker login`)." — S531, struct AuthConfig field docker_anon_pull L239
- Quote: "Not a vulnerability scanner. Curation blocks known-bad packages. For CVE scanning of your own artifacts, use Trivy, Grype, or similar." — S522, What NORA Is Not L489-L490
- Conditions/notes: the README names three token roles (read, write, admin); whether a token can be scoped to a repository was not examined; namespace isolation and curation (allow list, block list, minimum release age, quarantine) are the policy tools. Search is an in-memory map rebuilt at startup ("~5ms for 10k packages", ARCHITECTURE.md L272, unverified claim). A search of README, COMPAT and ARCHITECTURE for cosign/notation finds only the signing of NORA's own releases.
- Relevance: curation of proxied content (minimum release age, quarantine of new versions) is a supply-chain control the container-only registries lack; worth considering for the pull-through path.

### F337. nora (Q12, Q13): binaries, packages and images exist for Linux amd64 and arm64 only; releases are very frequent (1.0.0 on 2026-07-13, 1.4.7 on 2026-11-18).
- Quote: "`getnora/nora` on Docker Hub and `ghcr.io/getnora-io/nora` (amd64, arm64), with RED OS and Astra Linux SE variants" — S520, Install L60
- Quote: "nora-linux-amd64 sha256:65499f8febc3bd30bad38a55e2cf86fecb961244765a9936b75ede9bf0d0ddb8 29.4 MB" — S532, asset list
- Conditions/notes: the v1.4.7 asset list has nora-linux-amd64, nora-linux-arm64, .deb and .rpm for both architectures, SBOMs and signature bundles; nothing for macOS or Windows. Eight releases (1.4.0 to 1.4.7) between 2026-10-19 and 2026-11-18 (CHANGELOG headings).
- Relevance: none of the Rust registries surveyed ships a Windows server binary; of those with release binaries only angos ships macOS ones.

### F338. mcronce/oci-registry (Q1, Q4, Q13): a Rust pull-through cache for any upstream registry with filesystem and S3 back-ends that cannot accept pushes, has no authentication and supports only SHA-256; its last release is v0.4.5 from 2024-04-23.
- Quote: "Pushing is not currently implemented; `oci-registry` only supports being a pull-through cache (a mirror) at this time." — S534, Limitations L15
- Quote: "Only SHA256 content hashes are supported, but supporting other schemes is planned" — S534, Limitations L17
- Quote: "Has not yet had the [OCI distribution spec conformance test suite][oci-test-suite] run against it; only manual compatibility testing with `docker` and `containerd` has been performed." — S534, Limitations L22
- Conditions/notes: about 1,700 lines of Rust. The GitHub repository is a mirror of a self-hosted GitLab project; last push to the mirror 2024-06-24 (GitHub repository search, 2026-11-20), so it does not meet "maintained" in 2026.
- Relevance: shows the minimum viable mirror: namespace-routed upstreams, object store behind it, nothing else.

### F339. mcronce/oci-registry (Q10): the README gives one resource comparison from the author's own system: about 130 MiB for the reference `registry` mirroring docker.io against about 60 MiB for five replicas of oci-registry combined.
- Quote: "in my test system, the official `registry` uses approximately 130 MiB of memory to mirror docker.io; five replicas of `oci-registry` combined use approximately 60 MiB to mirror everything in [example.yaml](example.yaml), plus one private registry." — S534, Features L11
- Conditions/notes: no hardware, versions, load or measurement method are given; "CPU is negligible for both". Anecdotal, 2024 or earlier.
- Relevance: an order-of-magnitude hint only (about 60 MiB over five Rust mirror replicas against about 130 MiB for one Distribution instance); not usable as a benchmark claim.

### F340. mcronce/oci-registry (Q4): it has no stampede protection (two simultaneous requests for a blob download it twice) and its freshness model is a per-upstream invalidation time for manifests and blobs, both defaulting to 14 days.
- Quote: "If two clients request the same blob simultaneously, it will be downloaded from upstream twice in parallel instead of having the later request wait for the download to finish, then serve it from cache." — S534, Limitations L21
- Quote: "fn default_manifest_invalidation_time() -> Duration { core::time::Duration::from_secs(14 * 86400).into()" — S535, L74-L75
- Conditions/notes: per-upstream settings `manifest_invalidation_time` and `blob_invalidation_time` (README example sets 0s and 30d); a cleanup loop runs every 300 s (main.rs L99). Upstream credentials come from a config file or an environment variable; the README warns that this makes it easy to expose a private registry as an unauthenticated mirror.
- Relevance: a TTL-based cache (serve without revalidation until the invalidation time) is the alternative to revalidate-on-every-pull; 14 days for mutable tags is a staleness bound few users would expect.

### F341. container_registry-rs (Q1, Q13): the `container-registry` crate is a minimal "best effort" registry for embedding in an axum application (basic auth, push and pull with podman or docker, local filesystem storage); its README advises against mission-critical use and the last release is 0.3.1 from 2024-08-14.
- Quote: "authentication via HTTP basic auth, image uploading via podman or docker, image downloading via podman or docker, and storing container images on the local filesystem." — S536, Feature set and standard conformity
- Quote: "The crate has not been thoroughly battle tested in contested production environments, or seen a deep review, so relying on it for mission critical deployments is probably a bad idea." — S536, Production readiness
- Conditions/notes: targets distribution-spec v1.0.1 by its own links; extracted from the small PaaS rockslide. Not maintained in 2025-2026 by the release feed.
- Relevance: none beyond confirming that embeddable Rust registries exist only as minimal crates.

### F342. rk8s distribution (Q1): the rk8s project contains a "lite" OCI Distribution implementation in Rust with filesystem blob storage, PostgreSQL for users and repositories, JWT and GitHub OAuth login; rk8s as a whole is declared work in progress and not for production.
- Quote: "A lite implementation of the OCI Distribution Spec in Rust, with added support for user management and authentication." — S537, Distribution L3
- Quote: "Prerequisites: Ensure you have a PostgreSQL server running and accessible." — S537, Quick Start > With Cargo
- Quote: "The project is currently in WIP stage and should not be used in production environments." — S538, README intro L7
- Conditions/notes: rk8s v0.1.3 (2026-04-26). Only the README of project/distribution was read (configuration via `OCI_REGISTRY_STORAGE=FILESYSTEM`, `DATABASE_URL`, `JWT_LIFETIME_SECONDS=3600`); source not examined.
- Relevance: a second Rust example of "database for metadata, files for blobs", at prototype maturity.

### F343. oci-distribution (correction): the Rust crate `oci-distribution` is a client library, since renamed `oci-client`; it is not a registry server.
- Quote: "Rust OCI Client Formerly known as `oci-distribution`" — S539, README title and first line
- Quote: "An OCI Distribution client for fetching oci images from an OCI compliant remote store" — S540, crate doc L1
- Quote: "oci-client = { version = "0.16.0", default-features = false, features = [" — S553, dependencies
- Conditions/notes: crates.io lists oci-distribution 0.11.0 as last updated 2024-03-27 and oci-client 0.18.0 updated 2026-09-18 (crates.io API, 2026-11-20). Trow uses the crate as the client for its pull-through proxy.
- Relevance: useful as the upstream client of a pull-through cache, not as a design reference for a server.

### F344. go-containerregistry pkg/registry (Q1): the registry package in google/go-containerregistry is an in-process registry for tests: manifests and upload sessions are Go maps, blobs are in memory or in a flat directory, and the package is kept to dependencies on the standard library and the rest of go-containerregistry.
- Quote: "It is designed to be used anywhere a low dependency container registry is needed, with an initial focus on tests." — S541, README L5
- Quote: "This is currently a low flightmiles system. It's likely quite safe to use in tests; If you're using it in production, please let us know how and send us PRs for integration tests." — S541, README L9
- Quote: "// maps repo -> manifest tag/digest -> manifest manifests map[string]map[string]manifest" — S542, type manifests L48-L49
- Quote: "Contents are can be stored in memory (when the process exits, pushed data is lost.), and disk (--disk)." — S545, crane registry serve long help L49
- Conditions/notes: go-containerregistry v0.22.1 (2026-09-04). Exposed as `crane registry serve`. With `--disk` only blobs go to disk (`<dir>/<alg>/<hex>`); manifests and tags stay in memory and are lost on exit.
- Relevance: what it is for: a dependency-free fixture. A new registry should offer the same embedding story (a simulator implementation of its own API) for client tests.

### F345. go-containerregistry pkg/registry (Q2): it does not implement the full specification: the referrers endpoint is opt-in and ignores the artifactType filter, the catalog has no pagination, and a pushed manifest's blobs are not checked for existence.
- Quote: "// WithReferrersSupport enables the referrers API endpoint (OCI 1.1+)" — S543, func WithReferrersSupport L124
- Quote: "// TODO: implement handling of artifactType querystring" — S542, above func handleReferrers L366
- Quote: "// TODO: Probably want to do an existence check for blobs." — S542, func handle (manifest PUT) L197
- Quote: "// TODO: implement pagination" — S542, func handleCatalog L338
- Conditions/notes: not listed on the OCI conformance site (S486).
- Relevance: tests written against this fixture can pass against behaviour a conformant registry must reject (manifests with missing blobs); do not use it as a behavioural oracle.

### F346. go-containerregistry pkg/registry (Q3, Q9): there is no garbage collection and no limit of any kind: each upload session is a byte slice in a map that grows with the upload.
- Quote: "// Each upload gets a unique id that writes occur to until finalized. uploads map[string][]byte" — S544, type blobs L167-L168
- Conditions/notes: no session expiry, no manifest or blob size limit, no authentication were found in the package.
- Relevance: avoid for anything but tests: an unauthenticated client can exhaust memory with one unfinished upload.

### F347. cloudflare/serverless-registry (Q1, Q5): the registry is a Cloudflare Worker (TypeScript) whose only state is an R2 bucket; it authenticates with one username/password or with JWTs verified by a public key, and has no per-repository authorization.
- Quote: "This repository contains a container registry implementation in Workers that uses R2. It supports all pushing and pulling workflows. It also supports Username/Password and public key JWT based authentication." — S546, README L3-L6
- Quote: "This registry has no per-account or per-repository scoping: any token that verifies grants the full extent of its capabilities over the whole registry." — S546, Adding JWT authentication with public key L76-L78
- Conditions/notes: default-branch commit a73605d (2026-08-17); the repository publishes no releases (package.json version 1.1.0). Manifests, blobs, upload state, referrers (`<name>/_referrers/<digest>/`) and GC marks are all R2 objects.
- Relevance: the reference point for "object store is the whole database" on an edge runtime: stateless compute, every coordination problem solved with object writes.

### F348. cloudflare/serverless-registry (Q9): docker pushes are limited to layers of at most 500 MB by the Workers request-body limit; the code bounds chunks to between 5 MiB and 5 GiB with at most 100 MB per uploaded chunk, and advertises the bounds to clients in OCI-Chunk-Min-Length and OCI-Chunk-Max-Length headers.
- Quote: "Pushing with docker is limited to images that have layers of maximum size 500MB. Refer to maximum request body sizes in your Workers plan." — S546, Known limitations L170
- Quote: "// 5MiB export const MINIMUM_CHUNK = 1024 * 1024 * 5; // 5GiB export const MAXIMUM_CHUNK = MINIMUM_CHUNK * 1024; // 100MB export const MAXIMUM_CHUNK_UPLOAD_SIZE = 1000 * 1000 * 100;" — S548, consts L5-L12
- Quote: "// Objects will typically be removed from a bucket within 24 hours of the x-amz-expiration value." — S549, func uploadChunk L894
- Conditions/notes: the chunk headers are set in src/router.ts L468-L471 and L498-L501. Upload state is a signed token with a 2-hour expiry (r2.ts L188); in full push-compatibility mode, undersized chunks are staged as separate objects given a one-hour expiration (r2.ts L886-L895). The 500 MB figure is the platform's request-body limit, which varies by plan; the repository's `push/` tool works around it by chunking.
- Relevance: copy: advertise chunk bounds in OCI-Chunk-Min-Length/Max-Length so clients adapt, instead of failing oversized PATCHes; the minimum part size of the backing store leaks into the registry API unless small chunks are staged.

### F349. cloudflare/serverless-registry (Q3): garbage collection is triggered per repository by an authenticated POST and is lock-free: the collector and each manifest insertion write marks into R2 and re-check the other side's mark before committing, so a conflict produces an error rather than a deletion. The authors chose this because a stop-the-world lock is impossible with R2 alone.
- Quote: "Some registries take a lock stop the world approach, however serverless-registry can't really do that due to its objective of only using R2. However, we need to fail whenever a race condition happens, a data race that causes data-loss would be completely unacceptable." — S547, How does it work L65-L67
- Quote: "It's a pattern where you build the state you need a lock in, get the mark of when you built that world, and confirm before making changes from that view that there is nothing that might've changed the view." — S547, How does it work L118-L119
- Quote: "The garbage collector checks for dangling layers in the namespace. It's a lock free GC, but on-conflict (when there is an ongoing manifest insertion, or an ongoing garbage collection), the methods can throw errors." — S550, class GarbageCollector doc comment L16-L18
- Quote: "We take the approach of listing all manifests in a namespace and storing its digests in a Set, then we list all the layers and those that are not in the Set get removed." — S547, How does it work L58-L59
- Conditions/notes: endpoint `POST /v2/<name>/gc` (src/router.ts L705); modes "unreferenced" and "untagged". Cost per run: list every manifest and every layer of the repository and read every manifest. The price of optimism is availability: a push that overlaps a GC run fails and must be retried, and a GC run that overlaps a push aborts. The marks are plain objects written with unconditional puts; the protocol's correctness under R2's consistency model is argued in the document by four interleavings, not proven or model-checked.
- Relevance: the same idea as angos's run markers (F315) arrived at independently: optimistic marks instead of locks on an object store. Both show what must be built when there is no transactional metadata store; with one, GC fencing is a transaction.

### F350. cloudflare/serverless-registry (Q4, Q8): a "pull fallback" list of upstream registries is consulted when content is missing and the result is copied into R2 (intended for migration); a cross-repository mount is stored as a small symlink object that names the source repository instead of copying the layer.
- Quote: "You can configure the R2 registry to fallback to another registry if it doesn't exist in your R2 bucket. It will download from the registry and copy it into the R2 bucket. In the next pull it will be able to pull it directly from R2." — S546, Configuring Pull fallback L104-L106
- Quote: "customMetadata: { [symlinkHeader]: sourceName }, // Storing target repository name in metadata (to easily resolve recursive layer mounting)" — S549, func mountExistingLayer L657
- Conditions/notes: fallback credentials are per registry (basic or bearer, secret named by `password_env`). No freshness model is documented: once copied, content is served from R2. Blobs are stored per repository name, and a mounted layer is a pointer to another repository's object, so deleting the source repository's layer has to account for symlinks (the GC code reads the symlink header).
- Relevance: avoid per-repository blob keys with cross-repository pointers; a global content-addressed blob namespace makes mount and GC simpler.

**peryx (Rust), examined briefly; two peer-reviewed papers on Distribution-based registries (F351 to F361)**

### F351. peryx (Q1): peryx is a Rust artifact cache and private registry (PyPI and OCI) shipped as one executable; all index roles share one content-addressed blob store on the local filesystem or an S3-compatible store, and metadata always stays in a node-local redb file.
- Quote: "Peryx is an artifact cache and private registry. One executable serves each supported artifact format." — S555, intro L9
- Quote: "Each index role shares a content-addressed blob store." — S555, Properties L15
- Quote: "Cached indexes fetch on a miss, verify the digest, and serve later requests from local or object storage." — S555, Properties L16
- Quote: "Hosted indexes accept private uploads under scoped access controls." — S555, Properties L17
- Quote: "Metadata always stays in the local redb store; only the content-addressed blobs move." — S556, [blob] L676
- Quote: "edition = "2024" rust-version = "1.96"" — S567, [workspace.package] L12-L13
- Conditions/notes: workspace lints set `unsafe_code = "deny"` (S567 L23); the HTTP stack is axum 0.8 (S567). Availability is a config choice `none`, `dc` or `ha` (S555 L18); see F354 for what of it ships. Found through the third-party comparison recorded in F168; not listed on the OCI conformance site (F009).
- Relevance: the nearest class competitor (Rust, one binary, embedded key-value metadata plus content-addressed bytes); its metadata is node-local, so scale-out rests entirely on its own replication layer.

### F352. peryx (Q2): it implements the distribution-spec routes including the referrers API and `OCI-Subject`, addresses stored content by sha256 only (a blob digest in another algorithm is refused with 400 DIGEST_INVALID), and gates pull requests on the distribution-spec conformance suite built from a pinned commit with the sha512 data test switched off.
- Quote: "A blob digest in any other algorithm is rejected with 400 DIGEST_INVALID rather than served unverified; a manifest an upstream advertises under another algorithm is re-addressed under peryx's own sha256" — S557, Digest addressing L34-L36
- Quote: "GET /v2/<name>/referrers/<digest> and OCI-Subject on push" — S557, standards table, Referrers API row L27
- Quote: "git -C "$scratch/checkout" fetch --quiet --depth=1 origin fcfba1ec55526073f48b2f6d4e3d7eef410ddcbc" — S565, recipe conformance L1231
- Quote: "OCI_PASSWORD=conformance OCI_DATA_SHA512=false "$suite")" — S565, recipe conformance L1271
- Conditions/notes: the suite runs against a debug build with one hosted index; its results are not published. A referrers lookup accepts sha512 and unregistered algorithms as subject keys only (S558 L190-L196).
- Relevance: a second recent registry (after the list in F007 and F175) that declines sha512; supporting it end to end remains a differentiator.

### F353. peryx (Q3): outside `ha` mode there is no online garbage collector: a delete removes only the repository link, unreferenced bytes are reclaimed by an offline command that cannot run beside the server, and a crash between the blob write and its metadata row leaves an orphan object by design.
- Quote: "For a delete, peryx removes the target link and leaves the shared content store unchanged." — S564, cross-repository mount paragraph L57
- Quote: "The collector checks again after its disk walk, preserving bytes referenced by a concurrent publication." — S564, cross-repository mount paragraph L58-L59
- Quote: "Every offline cache command opens the metadata store, and the store admits one holder at a time, so none of them run alongside serve." — S563, cache commands L226-L227
- Quote: "Because the object write commits the blob before its metadata row, a crash between the two leaves an orphan object." — S556, [blob] > Backup and failure recovery L755
- Conditions/notes: the command is `peryx cache purge orphaned-blobs`, a dry run unless `--yes` is given (S563 L223). The scheduled reclamation job exists only as an HA component (F354). No duration or scale figure for a purge is published.
- Relevance: write order "bytes first, then the record" makes a crash leave garbage, never a dangling reference: copy the order, and beat the offline purge with an online collector.

### F354. peryx (Q3, Q4): its HA reclamation design closes the delete-versus-reference race with a reference revision and a per-digest guard, holds a deletion until every replica and backup has applied the metadata serial, and fences the single collector by ownership term; it runs only in `ha` mode, where the docs' own status table lists OCI write acknowledgements as not integrated and automatic failover as design only.
- Quote: "A delete and a reference commit race, so the collector that purges orphaned blobs arms a per-digest reference guard around its delete." — S562, Reference guard L56-L57
- Quote: "While a guard is armed, a commit that names the digest as a blob reference is rejected, whether or not it appends a replication journal entry." — S562, Reference guard L61-L62
- Quote: "Deletion waits until each live replica and configured backup has applied that serial. One lagging plane keeps the candidate pending." — S562, Frontier gate L39-L40
- Quote: "The job requires a nonzero ownership term, so it performs no work in dc, which has no ownership consensus." — S562, intro L11-L12
- Counter: the HA mode this depends on is incomplete for OCI — "OCI mutation paths do not call the distributed acknowledgement resolver" — S561, Release status table, row OCI write acknowledgements L30
- Counter: failover is not automatic — "No runtime worker calls the failover policy or submits the transfer after liveness marks a member dead" — S561, Release status table, row Automatic failed-home selection and transfer L27
- Conditions/notes: design documentation of a project whose first release is eight weeks old (F358); no test report, model check or production evidence for the protocol was found in the docs read. Tombstone states are Pending, Ready and Skipped; a returning reference abandons a candidate (S562 L64-L72).
- Relevance: the only design found in this survey that states the collector's proof obligations (a revision-stamped "unreferenced" verdict, a guard that makes a racing reference commit fail, replica frontiers before bytes go); a useful checklist for an online collector over a replicated record store.

### F355. peryx (Q4): pull-through freshness is a time window with conditional revalidation and single-flight: a cached tag is served for `cache_ttl_secs` (default 300), then revalidated by a body-less HEAD; a burst of pulls costs one upstream check; on 401, 429, 5xx or a transport error a stale tag is served within `max_stale_secs` (default 300); an upstream 404 is remembered for 30 seconds; concurrent misses of one layer share one upstream fetch that is streamed to clients while it is stored.
- Quote: "peryx streams image blobs into a content-addressed store while serving the client. Concurrent pulls of one uncached layer share one upstream fetch." — S559, intro L7-L8
- Quote: "The freshness window removes this request from fresh reads, and the single-flight gate reduces a burst of stale-tag pulls to one upstream check." — S559, Endpoint coverage L119-L120
- Quote: "A 404 now settles the lookup and is remembered for 30 seconds, so a deployment client polling once a second for a tag it is about to publish spends two calls a minute instead of a hundred and twenty." — S559, Endpoint coverage L125-L127
- Quote: "Serve a stale tag within max_stale_secs, else forward the failure" — S558, Tag lookups a registry refuses, row 401/429/5xx/transport L67
- Quote: "cache_ttl_secs is both a fallback and a ceiling." — S556, cache freshness L44
- Counter: the speed of cached tag reads is bought with bounded staleness, as the project says itself — "The microseconds buy you a tag that can be up to one freshness window stale, which is the trade a caching proxy exists to make." — S559, Endpoint coverage L115-L116
- Conditions/notes: defaults `cache_ttl_secs` 300 and `max_stale_secs` 300 are in the settings table (S556 L24, L27); the 30-second negative window is stated with its consequence (a tag pushed during the window appears when it closes, S558 L73-L76) but not with a reason for the number. A mismatched sha256 advertised by the upstream gives 502 and caches nothing (S558 L44).
- Relevance: the freshness model to copy: explicit TTL, conditional HEAD, stale-on-error with a bound, negative caching and single-flight, each with a named setting. Compare F026, F067, F128 and F188, which revalidate on every tag pull.

### F356. peryx (Q9): limits documented for the OCI path: a manifest push above 4 MiB gets 413, an idle upload session is reclaimed after one hour by a pass that runs every minute and retests under the session lock, an out-of-order chunk gets 416 with the resume coordinates, and an artifact over the index's size limit gets 403.
- Quote: "A manifest push whose body exceeds the 4 MiB cap answers 413 Payload Too Large with code SIZE_INVALID." — S569, manifest push L418
- Quote: "peryx marks it eligible for reclamation after one hour without a status GET or PATCH attempt. By default, a local worker runs the pass once per minute" — S558, Cancel an upload session L120-L121
- Quote: "Reclamation takes the same per-session lock a PATCH, a closing PUT, and a status GET hold, and it retests the idle timestamp once it has that lock." — S558, Reclaiming an idle session L132-L133
- Quote: "The metadata store contains the session record, and the filesystem contains its staged bytes. After a restart, peryx reads both, and the client resumes from the recorded offset." — S558, Upload sessions L104-L106
- Conditions/notes: no reason is given for 4 MiB, one hour or one minute. The size limit is `max_artifact_size_bytes` per index (S556 L485); an over-limit chunked upload also removes the session (S558 L137-L140). Session ids are 128-bit random values (S558 L102-L103).
- Relevance: the session-reclaim rule (select, lock, retest, delete bytes, then the record) is the race-free shape to copy; the constants are unexplained like everyone else's.

### F357. peryx (Q10): the project publishes a pull-through comparison on one named host against Distribution in proxy mode, zot with on-demand sync and direct Docker Hub pulls: six images warm in 0.7 s (Distribution 2.7 s, zot 6.6 s, direct 7.4 s) and cold in 6.7 s (13.5 s, 51.2 s); ten clients pulling one image cold in 2.2 s against 6 to 12 s; one cached 30 MB layer streamed at 6,838 MB/s against zot's 2,772.
- Quote: "Against Docker Hub, peryx serves a warm pull in 0.7 s against 7.4 s to pull Docker Hub yourself, and ahead of distribution (2.7 s) and zot (6.6 s)." — S559, Pulling images L45-L46
- Quote: "peryx fills its cache in 6.7 s, ahead of direct's 7.4 s despite also verifying and storing every layer" — S559, Pulling images L47-L48
- Quote: "distribution pays 13.5 s and zot's on-demand sync far more, 51.2 s." — S559, Pulling images L48-L49
- Quote: "peryx's ten clients share the upstream fetches and finish cold in 2.2 s and warm in 0.7 s, against 6 to 12 s cold for the others." — S559, Concurrent pull fleet L91-L93
- Quote: "peryx serves a single stream at 6,838 MB/s against zot's 2,772, and eight readers take it to 8,192." — S559, Layer throughput L68
- Quote: "model = "Mac16,10" cpu = "Apple M4" architecture = "arm64" cores = "10 (4 performance + 6 efficiency)" memory = "16 GB"" — S560, [host] L2-L6
- Counter: the hot-layer figures measure the loopback socket more than the registry, by the page's own account — "That neighbourhood is the socket, not the registry." — S559, Layer throughput L74
- Counter: cold rows include Docker Hub and the network — "the cold row carries the actual upstream fetch (the network, Docker Hub's own latency, and the proxy's store write), so it is marked network-bound and kept out of the regression gate" — S559, Benchmark setup L17-L18
- Conditions/notes: vendor-published and not independent: the authors benchmark their own registry against two competitors. Host macOS 27.0 with the stores on a removable 2.0 TB APFS SSD (S560 L7-L17); client crane for pulls and in-process HTTP for layer transfers; workloads: six official images cold then warm; 30 MB layer of python:3.12-slim, one and eight readers; ten clients pulling node:22-alpine. Versions of zot and Distribution are not stated on the page. The contributor guide documents the harness for the PyPI comparison (fixed seed, shuffled server-round schedule, every raw sample kept, at least five rounds from one machine and power state, S570 L70-L88) and says the OCI workload is exposed through the same kind of binary (S570 L10-L11); the published OCI data file holds a value, range, spread and outlier count per cell and no registry versions (site/data/ecosystems/oci/bench/report.toml at the same tag). A "shielded" run behind a local cache gives 0.6 s warm for peryx, 2.7 s Distribution, 6.7 s zot (S559 L53-L55). F168 records the rendered page as a secondary source for the zot and Distribution numbers.
- Relevance: the one published class-for-class pull-through comparison found, with a reusable harness shape (cold fill, warm serve, hot-layer throughput, fleet single-flight, per-endpoint latency); numbers to beat on comparable hardware, and a reminder to state competitor versions.

### F358. peryx (Q12, Q13): release binaries target Linux and macOS on x86_64 and aarch64 plus Windows on x86_64; the project uses calendar versions, its changelog starts at 2026.925.0 (2026-09-25) and the newest release is 2026.1118.1 (2026-11-18).
- Quote: "targets = ["aarch64-apple-darwin", "aarch64-unknown-linux-gnu", "x86_64-apple-darwin", "x86_64-unknown-linux-gnu", "x86_64-pc-windows-msvc"]" — S566, dist settings L8
- Quote: "## 2026.925.0 (2026-09-25)" — S568, oldest heading L128
- Quote: "## 2026.1118.1 (2026-11-18)" — S568, newest heading L8
- Conditions/notes: no aarch64 Windows and no musl target in the list. Eight changelog headings in eight weeks. Whether the Windows binary is tested with the OCI workload was not checked.
- Relevance: one of the few registries in this survey shipping native Windows and macOS server binaries (with zot F164, olareg F298 and Gitea F372); maturity is weeks, not years.

### F359. Distribution-based registry in production (Q10, peer-reviewed): a 75-day trace of the IBM Cloud registry (built on the open-source Docker registry, Swift backend) holds over 38 million requests and 181.3 TB transferred; GET requests dominate, more than half of all requests are for layers, and 65% and 80% of layers are smaller than 1 MB and 10 MB.
- Quote: "Our traces cover all availability zones and many components of the registry service over the course of 75 days, which totals to over 38 million requests and accounts for more than 181.3 TB of data transferred." — S571, 1 Introduction
- Quote: "The IBM Cloud container registry is a fully managed, highly available, high-performance, v2 registry based on the open-source Docker registry [4]." — S571, 2.2 IBM Cloud Container Registry
- Quote: "1. GET requests are dominant in all registries and more than half of the requests are for layers, opening an opportunity for effective layer caching and prefetching at the registry." — S571, Analysis Summary
- Quote: "2. 65% and 80% of all layers are smaller than 1 MB and 10 MB, respectively, making individual layers suitable for caching." — S571, Analysis Summary
- Quote: "5. Registry accesses to layers, manifests, repositories, and by users are heavily skewed." — S571, Analysis Summary
- Conditions/notes: FAST 2018 (peer-reviewed). Traces from 06/20/2017 to 09/02/2017, five geographic locations and seven availability zones, registry servers behind Nginx with OpenStack Swift as object store; the registry is the 2017 Docker registry with IBM's extensions, so the request mix is evidence about clients and workloads more than about v3 code. The paper also states the pull share of the workload in its introduction; that sentence is not quoted because the PDF text extraction loses its dash. The anonymized traces and a trace replayer are published by the authors.
- Relevance: a public, peer-reviewed workload model for sizing and for benchmarks: reads dominate, most layers are small, popularity is skewed; the small-object path and the manifest path decide latency for most requests.

### F360. Distribution storage backends (Q10, peer-reviewed): replaying 10,000 production requests against a registry on a 32-core, 64 GB node, layers under 1 MB took on average 0.008 s from memory in front of Swift, about 0.013 s from a local SSD filesystem, 0.07 s from Swift and 0.11 s by redirect to Swift; very large layers took 0.63 s from Swift and 0.59 s by redirect; a memory cache sized at 2% of ingress data reached a 40% hit ratio.
- Quote: "The registry is launched on a 32 core machine with 64 GB of main memory and 512 GB of SSD storage, and the Swift object store runs on a separate set of nodes of similar configuration." — S571, 5.2 Performance Analysis
- Quote: "This is followed by Local FS, which yields an average response time of around 0.013 s and Swift with an average response time of 0.07 s. Redirection performs the worst with average response time of 0.11 s." — S571, 5.2 Performance Analysis
- Quote: "layers less than 1 MB), the response time is the lowest (0.008 s on average) for Memory + Swift." — S571, 5.2 Performance Analysis
- Quote: "However, for very large layers, Swift and Redirection perform similarly, with average response latencies of 0.63 s and 0.59 s, respectively." — S571, 5.2 Performance Analysis
- Quote: "adding even a single level of LRU-based memory cache yields a hit ratio of 40% for dal with a cache size of 2% of ingress data and as high as 78% for fra and syd with a cache size of 10% of ingress data." — S571, 5.3 Two-level Cache, Hit ratio analysis
- Conditions/notes: FAST 2018. All nodes on 10 Gbps links; the trace replayer ran on six further nodes; the requests are 10,000 requests of the Dallas trace starting at 12 am on 2017-08-01. "Memory + Swift" is the authors' modification of the Swift driver (about 200 lines) that keeps layers under 1 MB in memory; Swift, local filesystem and redirection are stock Docker registry options of 2017. Swift was removed from Distribution in v3.0.0 (F002), so the absolute numbers do not transfer to S3-class stores; the ordering is the finding. Hit ratios come from a simulation over the traces.
- Relevance: a redirect costs a small blob more than serving it (0.11 s against 0.008 s to 0.013 s): redirect only above a size threshold and serve small blobs and manifests from a local tier. Compare F035, F279 and F324, which redirect by default.

### F361. Layer deduplication (Q8, Q10, peer-reviewed): content-addressed layers leave file-level redundancy untouched: about 97% of files across Docker Hub layers are duplicates, compressed layer tarballs deduplicate at a ratio of 1, and file-level deduplication after decompression reaches about 2.1; the highest ratio measured for DupHunter, 6.9, needs block-level deduplication with erasure coding and makes GET layer 6.37x slower.
- Quote: "a recent analysis of the Docker Hub image dataset showed that about 97% of files across layers are duplicates [72]." — S572, 1 Introduction
- Quote: "Putting the original compressed layer tarballs in any of the deduplication systems results, unsuprisingly, in a deduplication ratio of 1." — S572, 2.1 Redundancy in Container Images, Deduplication ratios
- Quote: "After decompression, all deduplication schemes yield significant deduplication ratios. Jdupes, Btrfs, and ZFS reduce the dataset to about half and achieve deduplication ratios of 2.1, 2.3, and 2.3, respectively." — S572, 2.1 Redundancy in Container Images, Deduplication ratios
- Quote: "As shown in Table 3, if file-level deduplication and 3-way replication are used, the deduplication ratio of B-mode 0 is 2.1 while the GET layer performance is 1.03× slower." — S572, 6 Evaluation, deduplication ratio against GET layer performance
- Quote: "The deduplication ratio with erasure coding and block-level deduplication is the highest (i.e., 6.9). However, the GET layer performance decreases by 6.37× because to restore a layer, its containing files, which are split into data chunks and spread across different nodes, must first be restored." — S572, 6 Evaluation, deduplication ratio against GET layer performance
- Counter: the same design is faster than no deduplication when it keeps full layer replicas on its primary servers, at little or no space saving — "B-mode 3 has the highest GET layer performance but does not provide any space savings since each layer in B-mode 3 has three full replicas." — S572, 6 Evaluation, deduplication ratio against GET layer performance
- Conditions/notes: USENIX ATC 2020 (peer-reviewed). Testbed section: a 16-node cluster, each node with 8 cores, 16 GB RAM, a 500 GB SSD and a 10 Gbps NIC (the introduction says a 6-node cluster); dataset 0.93 TB of popular Docker Hub images, 36,000 compressed layers, 2 TB decompressed; 300 clients on 10 nodes; results normalized to a registry without deduplication. The abstract's "up to 6.9×" storage reduction and "up to 2.8×" lower GET layer latency are two different modes, not one. The spelling "unsuprisingly" is the paper's.
- Relevance: a registry must return every blob byte for byte under its digest, so its own deduplication stops at identical blobs (as in F033, F203, F147). Anything finer belongs below the blob interface and is paid for on the read path: the one measured combination of block-level deduplication with erasure coding cost a 6.37x slower layer GET, which is the number an erasure-coded blob store has to stay clear of.

**Gitea and Forgejo, Pulp container, Sonatype Nexus Repository, Dragonfly, Uber Kraken, OpenShift image registry (F362 to F401)**

### F362. Gitea/Forgejo (Q1, Q8): the container registry is a module of the forge's single Go binary; metadata are rows in the forge's SQL database and blob bytes sit in the "packages" object storage (local disk, MinIO/S3-compatible or Azure Blob) under a SHA-256 key `aa/bb/<sha256>`, deduplicated across all package types and owners.
- Quote: "The package registry has a build-in deduplication of uploaded blobs. If two identical files are uploaded only one blob is saved on the filesystem." — S586, Deduplication
- Quote: "return path.Join(string(key)[0:2], string(key)[2:4], string(key))" — S577, func KeyToRelativePath L68-L71
- Quote: ";; Allows the storage driver to redirect to authenticated URLs to serve files directly ;; Currently, only `minio` and `azureblob` is supported." — S576, [packages] section L2875-L2876 (storage type options at L2870-L2874)
- Conditions/notes: Gitea v28.4.0; Forgejo v17.0.2 keeps the same model (same file names under `modules/packages` and `models/packages`). No external service beyond the forge's own database is required. A container "version" is a tag or a digest; each layer is a `package_file` row pointing at a `package_blob` row.
- Relevance: copy the single content-addressed blob namespace with database references; note that dedup is global, so every delete must go through reference counting (F366).

### F363. Gitea (Q2): the docs claim OCI compliance, but the v28.4.0 router carries a TODO for content discovery/referrers, accepts only manifests whose media type starts with `application/vnd.docker.` or `application/vnd.oci.`, and builds every digest as `sha256:`.
- Quote: "The container registry follows the OCI specs and supports all compatible images like Docker and Helm Charts." — S585, Container Registry (intro)
- Counter: the referrers API is not implemented — "// TODO: Content Discovery / References (not implemented yet)" — S574, func ContainerRoutes L556
- Quote: "return strings.HasPrefix(mt, "application/vnd.docker.") || strings.HasPrefix(mt, "application/vnd.oci.")" — S580, func IsMediaTypeValid L74-L76
- Quote: "_, _, hashSHA256, _ := h.Sums() return "sha256:" + hex.EncodeToString(hashSHA256)" — S583, func digestFromHashSummer L190-L193
- Conditions/notes: no route for `/v2/<name>/referrers/<digest>` exists in `ContainerRoutes` (L558-L584); no GitHub issue tracking referrers was found (three issue searches for the word referrers; the only hits were about image names and URLs), so the tracking issue is: not found. Version claimed: none; the code links distribution-spec `main` and `v1.1.1` sections in comments.
- Relevance: a registry that claims OCI compliance without running the conformance suite drifts; publish per-category conformance results instead of a sentence.

### F364. Forgejo (Q2, Q11): a user-run of the distribution-spec conformance program against Forgejo (reported 2026-11-18) passed 847 of 1034 checks and failed 184, including referrers, chunked/streamed blob push, blob mount, sha512 and manifest get/put categories.
- Quote: "It could be good to make Forgejo’s package registry more compliant following the results of this test." — S594, About your usage of Forgejo
- Quote: "Pass..........................: 847 FAIL..........................: 184 Error.........................: 0 Total.........................: 1034" — S594, Result summary
- Quote: "Referrers.....................: FAIL" — S594, Result summary > API conformance
- Quote: "Digest Algorithm sha512.......: FAIL" — S594, Result summary > Data conformance
- Counter: the project documents itself as following the OCI specs — "The container registry follows the OCI specs and supports all compatible images like Docker and Helm Charts." — S585, Container Registry (intro)
- Conditions/notes: reporter's run, not a project-published result; Forgejo version not stated beyond "tested on a next instance"; flags `OCI_API_BLOBS_UPLOAD_CANCEL=true OCI_DATA_SPARSE=true OCI_API_MANIFESTS_TAG_PARAM=true`; 3 skipped, 0 disabled. The program is whatever `go run github.com/opencontainers/distribution-spec/conformance` resolved to on the reporter's machine (version not stated); its per-API and per-data result table is not the four-category report of the v1.1 suite. Many FAIL rows may share one root cause (not analysed in the issue). No equivalent published run for Gitea was found.
- Relevance: gives the concrete list of checks an embedded registry misses; a new registry should run this exact program in CI and publish the table.

### F365. Gitea/Forgejo (Q2, Q11): tags are stored and looked up lower-cased, so two tags differing only in case collide and pushing one silently replaces the other and can delete the first manifest; the limitation is documented and reported as an open bug in both projects (2026-11-13).
- Quote: "The registry only supports case-insensitive tag names. So image:tag and image:Tag get treated as the same image and tag." — S585, Image naming convention > Note
- Quote: "Version: strings.ToLower(mci.Reference), LowerVersion: strings.ToLower(mci.Reference)," — S579, func createPackageAndVersion L272-L273
- Quote: "Pushing another manifest as v1.0.0-rc1 re-points v1.0.0-RC1 too. The first manifest is then deleted, because no tag names it any more: its digest answers 404." — S588, What happened?
- Conditions/notes: reproduced by the reporter on Gitea 28.3.0 and Forgejo 17.0.1; the lower-casing lines are still present at Gitea v28.4.0. The OCI tag grammar `[a-zA-Z0-9_][a-zA-Z0-9._-]{0,127}` is case-sensitive, and Gitea validates against that exact pattern (container.go L50) before dropping case.
- Relevance: data loss from reusing a generic "package version" column for tags; keep tag bytes exact and test case-distinct tags.

### F366. Gitea/Forgejo (Q3): there is no registry garbage collector; a cron task (`cleanup_packages`, default daily at midnight and at start-up) applies per-owner cleanup rules, removes expired upload sessions and uploaded-but-unreferenced blobs, then deletes blob rows that no file references and that are older than `OLDER_THAN` (24h) inside one transaction, and only afterwards deletes the bytes from storage, logging (not retrying) failures.
- Quote: "Whenever a package gets deleted, only the references to the underlying blobs are removed. The blobs get not removed at this moment, so they still require space on the filesystem." — S586, Deduplication
- Quote: ";; Unreferenced blobs created more than OLDER_THAN ago are subject to deletion ;OLDER_THAN = 24h" — S576, [cron.cleanup_packages] L2391-L2392
- Quote: "Where("package_file.id IS NULL AND package_blob.created_unix < ?", time.Now().Add(-olderThan).Unix())." — S656, func FindExpiredUnreferencedBlobs L94-L101 (called from services/packages/cleanup/cleanup.go L188)
- Quote: "if err := contentStore.Delete(packages_module.BlobHash256Key(pb.HashSHA256)); err != nil { log.Error("Error deleting package blob [%v]: %v", pb.ID, err) }" — S575, func CleanupExpiredData L205-L207
- Quote: "Fix: restore missing blob file when re-publishing a package (#39239)" — S650, changelog
- Conditions/notes: online (no read-only mode); the task defaults (`Schedule: "@midnight"`, `RunAtStart: true`, `OlderThan: 24 * time.Hour`) are in S648 L139-L147. Two weaknesses are inferences from the code, with no incident report found for them: (1) the age guard is the blob row's creation time, not the time it became unreferenced, and the cleanup does not take the per-hash upload lock, so an old unreferenced blob being re-referenced by a concurrent upload races the task's transaction; (2) a failed storage delete after the row is gone leaves an orphan object that no later run can find (this path never walks storage). The v28.0.0 changelog line above shows the inverse divergence (row present, file missing) did occur. No documented durations or complexity; cost is one LEFT JOIN over `package_blob`/`package_file` per run.
- Relevance: copy the shape (references in the database, rows deleted in a transaction, bytes deleted after commit); avoid losing the delete on failure (keep a durable tombstone/queue) and guard by last-unreferenced time, not creation time.

### F367. Gitea vs Forgejo (Q3, Q11): Gitea leaves the per-architecture digest "versions" of replaced multi-arch images behind (open issue since 2024-09-16); Forgejo added a second cleanup pass that scans every container version in one transaction, keeps two in-memory maps, and deletes unreferenced `sha256:*` versions older than the same age in batches of 500.
- Quote: "When I push out a new version of all my tags and manifest, those packages that are listed just by their naked sha hashes are left still there." — S589, Description
- Quote: "SHA256BatchSize = 500 SHA256Log = "cleanup dangling images with a sha256:* version"" — S592, var block L19-L24
- Quote: "compute before making the inventory to not race against ongoing image creations" — S592, func cleanupSHA256 L48-L50
- Quote: "Although it is theoretically possible that a sha256: is uploaded after the index manifest that references it, this is not the normal order of operations." — S592, func cleanupSHA256 L61-L63
- Conditions/notes: Forgejo's pass iterates all `package_version` rows of type container in ascending id order and unmarshals each index manifest's metadata JSON; the maps grow with the number of digest versions (no stated bound). The second quote is the code's own statement of the ordering assumption the sweep rests on.
- Relevance: an untagged-manifest sweep needs an explicit referenced-by index (index manifest to child, subject to referrer); a full scan with in-memory maps is the thing to avoid at scale.

### F368. Gitea/Forgejo (Q9): the hard limits are constants in code: manifest body 10 MiB (413 above it), `_catalog` page capped at 100, tag list unbounded unless `n` is given, tags at most 128 characters, registry bearer token valid 24 hours, per-owner and per-type size limits default to unlimited; only the manifest and tag limits cite the spec.
- Quote: "const maxManifestSize = 10 * 1024 * 1024" — S573, const L40-L42 (the comment above it links the pushing-manifests section of the distribution spec)
- Quote: "if n <= 0 || n > 100 { n = 100 }" — S573, func GetRepositoryList L203-L206
- Quote: "ExpiresAt: jwt.NewNumericDate(now.Add(24 * time.Hour))," — S581, func CreateAuthorizationToken L37
- Quote: ";; Maximum size of a Container upload (`-1` means no limits, format `1000`, `1 MB`, `1 GiB`) ;LIMIT_SIZE_CONTAINER = -1" — S576, [packages] section L2895-L2896
- Quote: "const MaxManifestSize = 10 * 1024 * 1024" — S647, const L34-L36
- Conditions/notes: the spec section the comment links asks registries to accept manifests of at least 4 MiB; no reason is given for choosing 10 MiB. No rate limits, upload chunk minimum or concurrency limits exist in the container code. Forgejo adds quota enforcement on push routes (F371).
- Relevance: catalogue entry: 10 MiB manifest cap, unjustified; unbounded tag list by default is a bound a new registry must not copy.

### F369. Gitea (Q5, Q6): authentication is a bearer token minted at `/v2/token` from the forge's own users, access tokens and scopes (`read:package`, `write:package`); anonymous pulls run as a "ghost" user; visibility is inherited from the owner (user or organisation), so a single image cannot be private under a public owner; there is no CVE scanning or signature verification in the registry.
- Quote: "// If the current user is anonymous, the ghost user is used unless RequireSignInView is enabled." — S573, func Authenticate L153-L154
- Quote: "A package always belongs to an owner (a user or organisation), not a repository." — S587, Repository-Packages
- Quote: "Found that there isn't a way to make a container image private meaning anyone can pull the image even if the image is part of a private repo." — S590, Description
- Conditions/notes: no OIDC-for-registry, robot accounts or per-repository policies beyond the forge's token scopes and owner visibility. Search/UI is the forge's package pages; `_catalog` lists what the caller may see. A Trivy-scanning request (go-gitea/gitea issue 21811, opened 2022-11-14) is open; cosign signatures can be pushed as ordinary OCI manifests but nothing verifies them (no code found under `routers/api/packages/container`).
- Relevance: per-owner visibility is the most-commented open Gitea issue with "container registry" in its title (20 comments at 2026-11-20); per-repository policy is expected.

### F370. Gitea (Q8, Q10): the upload path is node-local and hash-heavy: chunked uploads append to a file in the server's temp directory with the resumable hash state saved in the database, every blob is hashed with MD5, SHA-1, SHA-256 and SHA-512 at once, and a global lock per blob hash serialises concurrent uploads of the same content.
- Quote: "func uploadPathTempDir() *tempdir.TempDir { return setting.AppDataTempDir("package-upload") }" — S578, L33-L35
- Quote: "combinedWriter := io.MultiWriter(md5, sha1, sha256, sha512)" — S582, func NewMultiHasher L42-L48
- Quote: "// There will be concurrent uploading for the same blob, so it needs a global lock per blob hash" — S583, func saveAsPackageBlob L25-L28
- Quote: "const DefaultMemorySize = 32 * 1024 * 1024" — S584, L29
- Conditions/notes: an upload session therefore lives on one node's disk; nothing in the code replicates the temp file, so a multi-node deployment needs session affinity or a shared temp directory (inference from the code; no doc statement found). Cross-repository mount is a database insert after an access check (`IsBlobAccessibleForUser`, container.go L243-L266). No published benchmark for either project was found.
- Relevance: avoid node-local upload state in a registry meant to run as a fleet; avoid computing digests nobody asked for on the hot path.

### F371. Forgejo vs Gitea (Q2, Q9, Q11): Forgejo is a separate code base that diverges in the registry: it enforces package quota on push routes (413 when exceeded), still returned an off-by-one `Range` for an upload-session GET in 17.0.1 (fixed in Gitea), and fixes concurrent-upload database errors in 17.0.2.
- Quote: "Evaluate quota against the package owner (org or user the package is pushed to), not the uploader (ctx.Doer)." — S593, func enforcePackagesQuota L96-L98
- Quote: "When I GET an open blob upload session, the `Range` header is one byte past what was uploaded." — S595, Description
- Quote: "This breaks resuming an upload, which is what the GET is for." — S595, Description
- Quote: "fix: prevent `Record has changed since last read in table 'package_version'` for concurrent container uploads" — S652, 17.0.2 bug fixes
- Conditions/notes: issue reproduced on Forgejo 17.0.1 with SQLite and local storage; state on 17.0.2 not verified. Other open Forgejo registry reports seen in the tracker on 2026-11-20: empty tag list for a package with tags (issue 13272), unique-constraint 500 on upload (issue 14160), digest links of `docker manifest create` images returning 404 (issue 14327).
- Relevance: resumable-upload semantics (Range, Location) are where embedded registries break real clients; cover them with the conformance program.

### F372. Gitea/Forgejo (Q12, Q13): Gitea's newest release is v28.4.0 (2026-11-15) with binaries for Linux (amd64, arm64, arm-5, arm-6, riscv64), macOS (amd64, arm64), Windows (amd64, arm64) and FreeBSD (amd64), the linux-amd64 binary being 114 MB; Forgejo's newest is v17.0.2 (2026-11-18), Linux-only (amd64, arm64, arm-6), Windows support having been dropped in 2024.
- Quote: "gitea-28.4.0-linux-amd64 sha256:154e775bd6bb589fc14cd93a689d8fbc64090aaa3ec537f0e3af298b13c6d386 114 MB 2026-11-15T19:39:29Z" — S591, asset list
- Quote: "In 2024, it was decided to drop support for the Microsoft Windows operating system and some specific code has been removed." — S596, Why are there no mentions of Windows binaries?
- Quote: ""draft":false,"prerelease":false,"created_at":"2026-11-18T19:33:02+01:00","published_at":"2026-11-18T19:33:02+01:00"" — S597, release record for tag v17.0.2
- Quote: "(packages) Accept OCI 1.1 container image names (#40363) (#40369)" — S651, changelog
- Conditions/notes: Gitea's previous line was v1.27.x (v1.27.3 on 2026-08-29) and the next tag after it is v28.0.0 (2026-09-30) per the project's release feed; the reason for the numbering jump was not looked up. The 114 MB figure is the whole forge, not a registry. Forgejo asset names per the same API record: `forgejo-17.0.2-linux-amd64`, `-linux-arm64`, `-linux-arm-6`. The changelog line quoted is the one container-registry change in v28.4.0.
- Relevance: an embedded registry inherits a 100 MB+ binary and the host's platform matrix; a dedicated static binary can be far smaller.

### F373. Pulp container (Q1): the registry is a plugin of Pulp 3: a Django REST API, an aiohttp content-serving app and task workers, all coordinated through PostgreSQL, with artifacts stored on a filesystem or S3 and deduplicated; it is a multi-process service, not a single binary.
- Quote: "Pulp's architecture has three components to it: a REST API, a content serving application, and the tasking system." — S599, Architecture
- Quote: "All necessary information about tasks is stored in Pulp's Postgres database as a single source of truth." — S599, Distributed Tasking System
- Quote: "- Host content either locally or on S3 - De-duplicate all saved content" — S598, Features L26-L27
- Conditions/notes: pulp_container 2.30.1 requires pulpcore >=3.118.0,<3.130 and Python >=3.11 (pyproject.toml). Redis is optional (`CACHE_ENABLED = False` by default in pulpcore settings.py L338).
- Relevance: an example of metadata held fully in a relational database with blobs as content-addressed artifacts; the cost is that every write is a database task (F377).

### F374. Pulp container (Q2): Pulp states it implements the OCI distribution and image specs, but 2.30.1 registers no referrers route and resolves manifests "by digest" only when the reference starts with `sha256:`.
- Quote: "Pulp is a Container and Artifact Registry that implements OCI Distribution Specification and OCI Image Format Specification." — S598, Welcome to Pulp Container L4-L5
- Counter: the route table has blobs, uploads, manifests, signatures, catalog and tags only — "router.register(rf"v2/{re_path}/manifests", Manifests, basename="manifests") router.register(rf"extensions/v2/{re_path}/signatures", Signatures, basename="signatures")" — S600, L38-L39
- Quote: "if pk[:7] != "sha256:": # fetching manifest by tag name" — S601, func handle_safe_method L1320-L1321
- Conditions/notes: `grep -i referrers` over `registry_api.py`, `urls.py` and `models.py` at 2.30.1 returns nothing; `artifactType` is read only to classify a manifest (models.py L294). No conformance-suite result published by the project was found. Spec version claimed: none.
- Relevance: same pattern as F363: a compliance sentence without a published conformance table.

### F375. Pulp container (Q3): there is no registry GC; content that no repository version references becomes an "orphan" and is deleted by pulpcore's orphan-cleanup task after `ORPHAN_PROTECTION_TIME` (default 1440 minutes), and the docs warn the window must exceed the longest task or content created by that task can be deleted before it finishes; blobs and manifests pushed but not yet committed to a repository version are tracked in `pending_blobs`/`pending_manifests`.
- Quote: "The time, specified in minutes, for how long Pulp will hold orphan Content and Artifacts before they become candidates for deletion by an orphan cleanup task." — S602, ORPHAN_PROTECTION_TIME
- Quote: "This should ideally be longer than your longest running task. Otherwise any content created during that task could be cleaned up before the task finishes." — S602, ORPHAN_PROTECTION_TIME
- Quote: "# how long to protect ephemeral items in minutes ORPHAN_PROTECTION_TIME = 24 * 60" — S603, L304-L305
- Quote: "Remove pending blobs and manifests when committing the content to the repository." — S604, func remove_pending_content L760-L761
- Conditions/notes: online; correctness rests on a wall-clock grace period, and the docs themselves describe the failure when a task outlives it. Repository versions are snapshots ("every operation is a restorable snapshot", docs/index.md), so space is reclaimed only after the versions that reference content are gone and the orphan task has run. No durations or cost figures published.
- Relevance: a time-based grace period is the documented weak point; prefer a reference that a writer holds for as long as it is working.

### F376. Pulp container (Q4): mirroring is done by sync tasks from any v2 registry (mirror or additive, with filters); pull-through caching exists since 2.18.0 (2024-02-02) but is still a tech preview in 2.30.1, re-checks the upstream tag on every tag pull (HEAD since 2.30.0), serves the local tag if the upstream check fails, and keeps its repositories read-only.
- Quote: "This feature is provided as a tech preview and could change in backwards incompatible ways in the future." — S605, Configure Pull-Through Caching (warning)
- Quote: "if we fail to fetch the manifest on the upstream and we don't have a local tag already saved, then continue raising the HTTP exception" — S601, func handle_safe_method L1333-L1334
- Quote: "Pull-through manifest resolution now uses HEAD checks, reducing Docker Hub 429s for repeated tag resolutions." — S606, 2.30.0 (2026-10-08) Features
- Quote: "Repositories and their content remain manageable through standard Pulp API endpoints. The repositories are read-only and public by default." — S605, note under Configuring the caching
- Conditions/notes: one remote per pull-through distribution; `includes`/`excludes` patterns restrict upstream repositories. Freshness model: no TTL, an upstream round trip per tag pull; digest pulls go upstream only when the manifest is missing locally (L1391-L1396). Stampede handling: a missing manifest dispatches a background `download_image_data` task per request with `exclusive_resources=[repository]` (serialised by the task system; no in-flight dedup found). Where upstream credentials are kept was not checked.
- Relevance: revalidating the tag on every pull and falling back to the cached tag when the upstream is down is a clear freshness rule worth copying; the lack of a TTL makes every pull depend on upstream latency and rate limits.

### F377. Pulp container (Q8, Q11): every manifest push is turned into a repository-exclusive database task that creates a new repository version; when that task is cancelled the client gets HTTP 429, and users report throttled concurrent pushes and intermittent 404s on layer push.
- Quote: "elif immediate_task.state == "canceled": raise Throttled() elif immediate_task.state in ["waiting", "running"]: if has_task_completed(immediate_task, wait_in_seconds=2):" — S601, class Manifests put L1740-L1743
- Quote: "Push some images into Pulp. These images do have many layers, and are pushed concurrently from a couple of hosts, all NATed behind the same IP address." — S608, To Reproduce
- Quote: "We occasionally (with no real discernible pattern) see 404 Not Found during image pushes." — S609, Describe the bug
- Quote: "The registry supports cross repository blob mounting. When uploading blobs that already exist in the registry as a part of a different repository, the content is not being uploaded but rather referenced from another repository to reduce network traffic." — S607, Push Images
- Conditions/notes: issue 1396 was against pulpcore 3.21.2 (2023) and is closed; issue 1730 (pulp-container 2.19.2) is open with 18 comments. The 2-second wait and the 429 are constants in 2.30.1.
- Relevance: serialising all writes to a repository through a task queue bounds push concurrency per repository; a new registry should make tag updates a single conditional record write.

### F378. Pulp container (Q5, Q7): token authentication is on by default with a 300-second token; with it disabled there are no private repositories and only superusers can push; Pulp hosts and mirrors cosign signatures, SBOMs and attestations and can sign with a signing service, but no pull-time verification or enforcement was found.
- Quote: "TOKEN_EXPIRATION_TIME = getattr(settings, "TOKEN_EXPIRATION_TIME", 300)" — S610, L26
- Quote: "All users are permitted to pull content from the Registry without any limitations because the concept of private repositories is not adopted once token authentication is disabled." — S611, Basic Authentication
- Quote: "During the sync task, Pulp will automatically mirror cosign signatures or atomic signatures (accessible via the signatures extensions API)." — S612, Mirror cosign Signatures
- Quote: "Push and delete permission is restricted to those with the *Owner* or *Collaborator* roles on the image's namespace or distribution." — S654, Role-based Access Control
- Conditions/notes: authorisation is role-based on namespaces and distributions (owner, collaborator, consumer roles); distributions are public unless marked `private=True` (same page). No OIDC for registry clients or robot accounts found in the plugin docs. Search/UI: REST API and `_catalog` only; no CVE scanning in the plugin.
- Relevance: signatures as first-class mirrored content is worth copying; enforcement is left to clients.

### F379. Pulp container (Q9, Q10, Q12, Q13): constants are few and unexplained (catalog/tag page default 100, capped at 10 times that; signature payload 4 MB; 10 parallel signing tasks); the only timing found is a data-migration command handling 16115 manifests in about 2 minutes on a local filesystem; newest release 2.30.1 (2026-10-28), Linux, Python >= 3.11.
- Quote: "if self.n > 10 * api_settings.PAGE_SIZE: self.n = 10 * api_settings.PAGE_SIZE" — S601, class ContainerCatalogPagination L585-L586
- Quote: "MEGABYTE = 1_000_000 SIGNATURE_PAYLOAD_MAX_SIZE = 4 * MEGABYTE" — S613, L73-L74
- Quote: "Successfully handled 16115 manifests. real 2m4.823s" — S614, comment with test results
- Quote: "## 2.30.1 (2026-10-28) {: #2.30.1 } #### Bugfixes {: #2.30.1-bugfix } - Allow anonymous pull-through for public distributions" — S606, Changelog L11-L15
- Quote: "Operating System :: POSIX :: Linux" — S615, classifiers L18
- Conditions/notes: the timing is for `pulpcore-manager container-handle-image-data` after syncing 128 repositories with 33k manifests (24k image manifests, 9k manifest lists) on a local filesystem, reported in an issue opened 2024-03-22; hardware not stated; it is not a registry throughput benchmark. No pull/push benchmark, binary size or idle memory figure was found. `PAGE_SIZE` is 100 in pulpcore settings.py L188. No manifest size limit constant was found in the plugin.
- Relevance: nothing to beat on published performance; the page-size cap (1000) is another unexplained constant for the catalogue.

### F380. Sonatype Nexus Repository (Q1, Q13; brief's assumption outdated): the open-source code base ("Nexus Repository Core") no longer contains the Docker format: it covers maven, raw and APT with an embedded H2 database, and Docker is only in the free-but-closed Community Edition binary and the paid editions.
- Quote: "This is the open source codebase of Nexus Repository Core. This contains functionality for maven, raw, and APT repository formats, and uses an embedded H2 database that is appropriate for small workloads." — S616, Sonatype Nexus Repository Core
- Quote: "Community Edition includes additional format support such as npm, Docker, NuGet, PyPI and many others. It also allows use of an external PostgreSQL database that allows you to deploy Nexus Repository under Kubernetes." — S616, Sonatype Nexus Repository Core
- Conditions/notes: README at tag `release-3.98.1-01` (2026-11-05). Java (requires Java 21, F389); blob stores are a file system or cloud object storage (S3 and Azure are named in the cleanup docs); metadata in H2 or external PostgreSQL. Docker repositories come as hosted, proxy and group types.
- Relevance: Nexus is no longer an open implementation of a container registry; treat its docs as vendor documentation of a closed product.

### F381. Sonatype Nexus Repository (Q9): the free Community Edition is capped at 40,000 components and 100,000 requests per day and stops accepting new components above either threshold; the embedded H2 database is supported up to 200,000 requests per day or 100,000 components and not in containers.
- Quote: "It supports up to 40,000 total components and 100,000 requests per day." — S617, Community Edition Limitations
- Quote: "For the small percentage of larger or high-use teams that exceed these limits, Community Edition's built-in safeguards will pause the addition of new components until usage returns below both thresholds." — S617, Community Edition Limitations
- Quote: "Nexus Repository running with the embedded H2 database supports up to 200,000 requests per day or 100,000 components." — S618, Embedded H2 Database Limitations
- Conditions/notes: the stated reason for the Community Edition caps is commercial ("aligns our free Community Edition tier with teams and smaller organizations"). What counts as one component for a Docker image (tag, manifest or layer) is not stated on this page; Sonatype's "Components and Assets in Docker" page was not read.
- Relevance: these are licence limits, not engineering limits; they bound how far the free product can be compared.

### F382. Sonatype Nexus Repository (Q2): Docker repositories support OCI image spec 1.0.0 and 1.0.1 since release 3.71; no statement of distribution-spec 1.1, referrers or sha512 support was found in the Docker registry documentation.
- Quote: "The OCI specification versions 1.0.0 and 1.0.1 are supported within Docker repositories. This format support is available as of the 3.71 release." — S619, Supporting OCI Images
- Conditions/notes: the same section lists what changed for OCI: octet-stream blob uploads, `Location` on manifest upload, `DELETE /v2/{name}/manifests/{tag}`, optional `mediaType`. Docker Registry API V1 can still be enabled per repository. Conformance-suite status: not found.
- Relevance: a registry that stops at OCI 1.0 cannot serve referrers-based signatures and SBOMs.

### F383. Sonatype Nexus Repository (Q3): garbage collection is a chain of scheduled tasks: cleanup policies soft-delete tags, "Docker - Delete unused manifests and images" soft-deletes layers and manifests no tag references, and "Admin - Compact blob store" hard-deletes; the docs warn that images pulled or pushed only by digest are deleted irreversibly.
- Quote: "This task deletes Docker tags, manifests, and layers (blobs) that are no longer referenced by any tags or manifests." — S620, Docker - Delete unused manifests and images
- Quote: "In hosted repositories, this data loss is irreversible. To prevent images from being deleted by the Docker - Delete unused manifests and images task, ensure that images are referenced by at least one tag." — S620, Docker - Delete unused manifests and images (warning)
- Quote: "While Nexus Repository no longer displays soft deleted components in the user interface, they are not immediately deleted from storage and still use storage space." — S621, Hard Deleting Components with the Compact Blob Store Task
- Conditions/notes: online tasks, no read-only mode documented. Reclaiming space needs all three stages plus "Docker - Delete incomplete uploads" for abandoned sessions. Until 3.80.0 the compact task did not delete from S3 blob stores (same page).
- Relevance: soft-delete then compact gives an undo window (copy); "untagged means garbage" deletes digest-pinned deployments (avoid: digest references and referrers must count as roots or be protected by policy).

### F384. Sonatype Nexus Repository (Q3, Q11): a cleanup-policy bug in releases 3.31.0 to 3.41.0 deleted binaries unintentionally on H2/PostgreSQL deployments; Sonatype published a critical advisory and fixed it in 3.41.1.
- Quote: "Sonatype has discovered a critical bug that can cause cleanup policies to unintentionally delete binaries in Nexus Repository deployments using H2 or PostgreSQL." — S622, Critical Cleanup Policy Bug Advisory
- Quote: "Your deployment must be using Nexus Repository 3.31.0 to 3.41.0" — S622, Who is Impacted?
- Conditions/notes: affected policies were those with Component Age or Component Usage criteria; not Docker-specific.
- Relevance: retention policies evaluated against the wrong timestamp are a data-loss class; test retention with an oracle and a dry-run mode.

### F385. Sonatype Nexus Repository (Q10): the one published performance record is for cleanup: 10.6 ms per component (SQL-based) against 17.2 ms (Java-based), more than 10 hours for 3.2 million components, on an 8 vCPU/32 GB node with a 4 vCPU/32 GB PostgreSQL.
- Quote: "The average time per component for SQL-based cleanup was 10.6 ms versus 17.2 ms for Java-based cleanup." — S623, Summary of Findings
- Quote: "The total process could take more than 10 hours with the SQL-based approach and more than 15 hours with the Java-based approach for 3.2 Million components." — S623, Summary of Findings
- Quote: "A Nexus Repository node deployed in an AWS m5d.2xlarge EC2 instance (8 vCPU, 32 GB RAM)" — S623, Test Environment and Setup
- Conditions/notes: database "External Amazon PostgreSQL RDS backed by db.r5.xlarge (4 vCPU, 32GB RAM)"; workload "~3.2 million Maven components in hosted and proxy Maven repositories" with a policy that flagged all components; Nexus version and date of the test not stated (page last modified 2025-08-07). Maven, not Docker. Vendor-measured, not independent. No pull/push throughput figures found.
- Relevance: a number to beat for policy-driven deletion: about 94 components per second at best (derived here as 1 / 10.6 ms; the page gives no rate).

### F386. Sonatype Nexus Repository (Q4): a proxy repository caches what clients request and re-checks the remote after configured maximum ages; group repositories merge hosted and proxy members under one URL; pushing through a group is a paid feature.
- Quote: "If the content is not cached, Nexus Repository retrieves it from the remote repository, stores it locally, and serves it to the client." — S624, Proxy Repository Type
- Quote: "Nexus Repository uses the greater of the Maximum component age and Maximum metadata age values before rechecking the remote repository." — S624, Proxy Repository Type
- Quote: "With Sonatype Nexus Repository Pro, you can push images to a group repository." — S625, Pushing Images to a Group Repository
- Conditions/notes: one remote URL per proxy repository; several upstreams are combined by a group whose members are listed "in the desired order" (Grouping Docker Repositories page). Whether a Docker proxy repository refuses pushes is not stated on the pages read (push is described for hosted repositories and, in Pro, for groups with a writable member). Behaviour when the remote is down and stampede handling: not found on these pages. ECR proxying supports IAM keys and short-lived STS credentials from 3.95 (Proxy Repository for Docker page).
- Relevance: the TTL pair (component age, metadata age) is the common freshness model; how two TTLs combine (here: the greater wins) is a rule to state explicitly.

### F387. Sonatype Nexus Repository (Q5, Q7): Docker clients authenticate through a "Docker Bearer Token Realm" that must be enabled (also for anonymous pulls), authorisation is Nexus's privilege model, and signature trust is not handled by the product.
- Quote: "In Sonatype Nexus Repository, you must enable the Docker Bearer Token Realm to access Docker repositories with a Docker client or another container image manager, such as Docker Desktop, Docker Engine, or Podman." — S626, Docker Bearer Token Realm and Access to Docker Repositories
- Quote: "Docker Content Trust is not directly handled by Nexus Repository." — S627, Docker Content Trust
- Conditions/notes: anonymous pulls are "controlled by the nx-view read privilege" (same authentication page). No cosign/notation verification or SBOM handling was found in the Docker registry documentation.
- Relevance: none to copy for trust.

### F388. Sonatype Nexus Repository (Q9): documented operating constants: at most 20 Docker port connectors per instance, unused-asset-blob cleanup every 30 minutes with a 60-minute upload grace, cleanup-policy task hourly, compaction retention "Blobs Older Than" 0 days by default; reasons are given for the first two only.
- Quote: "For better server performance, limit your configuration to only use 20 port connectors. Each port connector must reserve system resources to monitor the exposed port which cannot be used to serve requests to your clients." — S619, Port Connectors
- Quote: "Nexus Repository creates one copy of the task for each format to run every 30 minutes. Use the delay minute/hour property to configure the interval before an asset is soft-deleted. This delay avoids soft-delete files that take a significant amount of time to upload to the repository." — S620, Admin - Cleanup unused asset blobs
- Quote: "When configuring the Admin - Compact blob store task you may set the Blobs Older Than property to temporarily retain deleted artifacts for a designated number of days before deleting them from storage. The default value is 0 days." — S621, Retain Soft-Deleted Blobs for a Period of Time Before Hard Deleting
- Conditions/notes: the grace property is `nexus.assetBlobCleanupTask.blobCreatedDelayMinute=60`; before 3.62.0 the task ignored blob creation time (same section). Manifest size limit, upload session expiry default ("Age in Hours" is a task parameter) and pagination defaults: not found in the docs.
- Relevance: another wall-clock upload grace (60 minutes) standing in for knowing that an upload is still in progress.

### F389. Sonatype Nexus Repository (Q11, Q12, Q13): newest self-hosted release is 3.98.3 (2026-11-19); the 3.98 notes list a database-migrator defect that breaks OCI pulls after H2/PostgreSQL migration; supported operating systems are Linux, Windows and macOS on Java 21, "Intel and AMD" CPUs.
- Quote: "3.98.0 – November 3, 2026 3.98.1 – November 5, 2026 3.98.2 – November 18, 2026 3.98.3 – November 19, 2026" — S628, Sonatype Nexus Repository 3.98.0 – 3.98.3 Release Notes
- Quote: "Database migrations between H2 and PostgreSQL may not correctly transform OCI-related tables. After migration, OCI pull requests can fail with an HTTP 500 Internal Server Error across hosted, proxy, and group repositories." — S628, Database Migrator 3.94.0 – 3.98.0 (known issues)
- Quote: "Sonatype Nexus Repository supports the Linux, Windows, and MacOS operating systems. Sonatype has not tested nor supported other operating systems." — S618, Operating systems
- Quote: "Nexus Repository is tested on and supports OpenJDK and requires Java 21. Nexus Repository is compatible with both Intel and AMD CPU architectures." — S618, Supported Java Versions
- Conditions/notes: arm64 is not mentioned on the requirements page. The public source tag lags the binary (release-3.98.1-01 on 2026-11-05).
- Relevance: database migrations between engines are a recurring breakage source; one record store avoids the class.

### F390. Dragonfly (Q1): Dragonfly is not a registry but a P2P distribution layer in front of one, made of Manager, Scheduler, Seed Peer and Peer services; the Manager needs MySQL and Redis, and since 2026-07 a "lightweight" deployment drops the Manager and both databases.
- Quote: "Dragonfly services could be divided into four categories: Manager, Scheduler, Seed Peer and Peer." — S629, Architecture
- Quote: "Dragonfly now supports a lightweight deployment that removes the Manager, and with it MySQL and Redis." — S630, post body (first paragraph)
- Quote: "Dragonfly client written in Rust. It can serve as both a peer and a seed peer." — S655, introduction
- Quote: "module d7y.io/dragonfly/v2 go 1.25.0" — S653, L1-L3
- Conditions/notes: Manager and Scheduler are Go (dragonflyoss/dragonfly v2.5.4, 2026-11-13); the peer/seed-peer daemon is Rust (dragonflyoss/client v1.5.14). Persistent-task features still need Redis in the lightweight model (same post).
- Relevance: the registry need not do P2P itself, but it should serve range requests and stable blob URLs so that a layer like this can sit in front of it.

### F391. Dragonfly (Q4): it plays the pull-through/mirror role only: containerd is pointed at the local daemon with `pull` and `resolve` capabilities, the first request for a task makes a Seed Peer fetch from the source registry, later requests are served from peers; pushes do not go through it; images can be preheated, including from Harbor.
- Quote: "[host."http://127.0.0.1:4001"] capabilities = ["pull", "resolve"]" — S631, Proxy all registries to Dragonfly (hosts.toml)
- Quote: "First-time Download in P2P Cluster: If this is the first download, the Scheduler triggers the Seed Peer to download directly from the source." — S629, How it works
- Quote: "fn default_proxy_registry_mirror_addr() -> String { "https://index.docker.io".to_string() }" — S632, L418-L422
- Quote: "Preheating requires calling the container registry to parse the image manifest and construct the URL for downloading blobs." — S649, Preheat (v2.2.0 notes)
- Quote: "Dragonfly 2.0 is compatible with dragonfly 1.0 integrated harbor preheat interface." — S637, P2P Preheat
- Quote: "Improve task ID calculation for OCI registry blob downloads to reduce redundant downloads and storage across registries." — S638, New features and enhancements
- Conditions/notes: freshness is the upstream's (manifests/tags are proxied; blobs are content-addressed tasks; from v2.5.0 the task id for registry blobs is derived from the blob digest, `default_enable_task_id_based_blob_digest` being `true` in S632 L424-L430). Cached tasks expire by TTL and disk thresholds (F393). Behaviour when the source registry is down: a Peer that already has the pieces cached locally returns the file "without contacting the Scheduler" (Introduction, How it works); manifests and tags still come from the source registry, and no documented guarantee for an upstream outage was found.
- Relevance: preheat needs the registry to expose manifests to a trusted caller and to emit push events; plan for both.

### F392. Dragonfly (Q10): published numbers are sparse and conditions thin: the project's 2023 walkthrough shows one layer taking 1.47 s from source and 37.48 ms from a remote peer's cache in a kind cluster, and the CNCF graduation post (2026-01-15) claims tens of millions of container launches per day in production and up to 90% storage bandwidth saved.
- Quote: "When pull image back-to-source for the first time through dragonfly, peer uses cluster-a’s scheduler and seed peer. It takes 1.47s to download the 82cbeb56bf8065dfb9ff5a0c6ea212ab3a32f413a137675df59d496e68eaf399 layer." — S633, Containerd pull image back-to-source for the first time through dragonfly in cluster A
- Quote: "When pull image hits cache of remote peer, peer uses cluster-a’s scheduler and seed peer. It takes 37.48ms to download the 82cbeb56bf8065dfb9ff5a0c6ea212ab3a32f413a137675df59d496e68eaf399 layer." — S633, Containerd pull image hits the cache of remote peer in cluster A
- Quote: "Dragonfly continues to advance container image distribution, supporting tens of millions of container launches per day in production, saving storage bandwidth by up to 90%, and reducing launch time from minutes to seconds, with large-scale adoption across different cloud native scenarios." — S634, post body
- Conditions/notes: the 2023 test pulls `ghcr.io/dragonflyoss/dragonfly2/scheduler:v2.0.5` on kind (Kubernetes-in-Docker) worker nodes; layer size, hardware and repetitions are not stated (one run shown). The graduation figures are unattributed aggregate claims without workload definition. A peer-reviewed paper exists (F394) but was not read.
- Relevance: no rigorous registry-relevant benchmark to compare against; the claims define what users expect from a P2P front.

### F393. Dragonfly client (Q9): defaults are constants in the Rust client's config, mostly without reasons: 8 concurrent pieces, 360 s piece timeout, GC every 900 s, task TTL 30 days, disk GC between 80% and 60% usage, 50 GB/s download and back-to-source bandwidth limits, 10 GB/s prefetch limit (the only one with a stated reason).
- Quote: "fn default_gc_policy_task_ttl() -> Duration { Duration::from_secs(2_592_000) }" — S632, L334-L338
- Quote: "fn default_gc_interval() -> Duration { Duration::from_secs(900) }" — S632, L328-L332
- Quote: "fn default_download_concurrent_piece_count() -> u32 { 8 }" — S632, L185-L189
- Quote: "Returns the default rate limit of the prefetch speed in GB/MB/KB per second, default is 10GB/s. The prefetch request has lower priority so limit the rate to avoid occupying the bandwidth impact other download tasks." — S632, L411-L412
- Quote: "The client supports outbound bandwidth, inbound bandwidth, back-to-source bandwidth, prefetch bandwidth, upload request, download request, and adaptive rate limiting to better protect source services and improve system stability under high load." — S638, Comprehensive Rate Limiting
- Conditions/notes: client v1.5.14. v2.5.0 also added configurable request rate limits on the Manager and Scheduler gRPC servers (same post).
- Relevance: cache eviction by TTL plus high/low disk watermarks is a stated rule (copy the shape, measure the numbers).

### F394. Dragonfly (Q12, Q13): Dragonfly graduated in CNCF on 2025-10-28; newest releases are v2.5.4 (2026-11-13) and Rust client v1.5.14 (2026-11-13), shipped for Linux x86_64 and aarch64 (gnu and musl) only; a paper on its design was accepted by IEEE Transactions on Networking in 2025.
- Quote: "Dragonfly was accepted to CNCF on November 13, 2018, moved to the Incubating maturity level on April 9, 2020, and then moved to the Graduated maturity level on October 28, 2025." — S629, Milestones
- Quote: "dragonfly-client-v1.5.14-x86_64-unknown-linux-musl.tar.gz sha256:d514da77d57dd77e02ac903a0072e921aa19350bbef273f5be36ae41b235653a 33.1 MB 2026-11-12T16:26:00Z" — S636, asset list
- Quote: "Empowering Dragonfly: A Lightweight and Scalable Distribution System for Large Models with High Concurrency" — S635, Title
- Conditions/notes: release dates from the projects' release feeds; assets listed: `aarch64`/`x86_64` × `unknown-linux-gnu`/`unknown-linux-musl` as tar.gz, deb and rpm (the x86_64 musl tarball is 33.1 MB; what it contains was not inspected). 2025–2026 changes: v2.3.0 (2025-07), v2.4.0 (2026-01), v2.5.0 (2026-06: Hugging Face/ModelScope protocols, blocklist, rate limiting, registry proxy simplification), lightweight deployment (2026-07).
- Relevance: active, graduated project; integration target rather than competitor.

### F395. Uber Kraken (Q1, Q4): Kraken is a P2P registry front made of agent, origin, tracker, proxy and build-index services that keeps no data of its own beyond caches: blobs and tags live in pluggable backends (S3, GCS, ECR, HDFS, read-only HTTP or another registry), with rule-based asynchronous replication between clusters.
- Quote: "Kraken is a P2P-powered Docker registry that focuses on scalability and availability." — S639, introduction L9
- Quote: "Storage backends are used by Origin and Build-Index for data persistence. Kraken has support for S3, GCS, ECR, HDFS, http (readonly), and Docker Registry (readonly) as backends." — S640, Configuring Storage Backend For Origin And Build-Index L183
- Quote: "Powers image replication between clusters (simple duplicated queues with retry)" — S639, Architecture > Build-Index L94
- Conditions/notes: Go; five service types plus a backend; "Other than pluggable storage, Kraken only has an optional dependency on DNS" (README features). A read-only registry backend makes Kraken a pull-through distribution layer for an existing registry; pushes go through the proxy to origins and the build-index.
- Relevance: separates "who stores" from "who distributes"; hash-ring origins over an object store resemble a stateless registry tier.

### F396. Uber Kraken (Q10): the README publishes a benchmark and production figures: a 3 GB two-layer image pulled by 2600 hosts at once finished with p50 10 s, p99 18 s, p99.9 22 s; the busiest cluster distributes more than 1 million blobs per day and, at peak, 20K blobs of 100 MB–1 GB in under 30 s.
- Quote: "The following data is from a test where a 3G Docker image with 2 layers is downloaded by 2600 hosts concurrently (5200 blob downloads), with 300MB/s speed limit on all agents (using 5 trackers and 5 origins):" — S639, Benchmark L99-L101
- Quote: "- p50 = 10s (at speed limit) - p99 = 18s - p99.9 = 22s" — S639, Benchmark L105-L107
- Quote: "In our busiest cluster, Kraken distributes more than 1 million blobs per day, including 100k 1G+ blobs. At its peak production load, Kraken distributes 20K 100MB-1G blobs in under 30 sec." — S639, introduction L14-L16
- Conditions/notes: hardware, network, Kraken version and date of the test are not stated, and nothing in the README says it was re-measured for v0.1.37. Self-reported by the vendor. Scale claim: "Supports at least 15k hosts per cluster".
- Relevance: the reference numbers for burst distribution (2600 concurrent pullers); a central registry would need a CDN or P2P tier to match them.

### F397. Uber Kraken (Q8, Q9): its limits are stated with reasons: blobs are limited to 20 GB in practice because peers cap connections per blob; tags have no consistency guarantee and mutating a tag is unreliable (cached lookups, replication not triggered); idle torrents are evicted by time-to-idle (5 m in memory, 6 h on disk).
- Quote: "Theoretically, Kraken should distribute blobs of any size without significant performance degradation, but at Uber, we enforce a 20G limit and cannot endorse the production use of ultra-large blobs (i.e. 100G+)." — S639, Limitations L182-L184
- Quote: "Mutating tags (e.g. updating a `latest` tag) is allowed, however, a few things will not work: tag lookups immediately afterwards will still return the old value due to Nginx caching, and replication probably won't trigger." — S639, Limitations L176-L178
- Quote: "No consistency guarantees: the client should use unique tags" — S639, Architecture > Build-Index L93
- Quote: "SeederTTI (time-to-idle) is the duration a completed torrent will exist without being read from before being removed from in-memory archive." — S640, Seeder TTI L95
- Conditions/notes: the reason given for the blob cap: "Peers enforce connection limits on a per blob basis, and new peers might be starved for connections if no peers become seeders relatively soon." Origin file-count cap example `capacity: 1000000`.
- Relevance: tag immutability is assumed by P2P/caching fronts; a registry that offers linearizable tag updates should also expose "immutable tag" as a policy so such fronts stay correct.

### F398. Uber Kraken (Q2, Q3): the registry API is the 2019 Docker distribution library with a Kraken storage driver whose delete returns "path not found" and whose walk is unimplemented, so there is no delete, catalog walk or garbage collection through the registry interface; manifests accepted are Docker schema 2, manifest list, OCI manifest and OCI index.
- Quote: "replace github.com/docker/distribution => github.com/docker/distribution v0.0.0-20191024225408-dee21c0394b5" — S641, L131
- Quote: "// Walk is not implemented. func (d *KrakenStorageDriver) Walk(ctx context.Context, path string, f driver.WalkFn) error { return errors.New("walk not implemented") }" — S642, L366-L369
- Quote: "func (d *KrakenStorageDriver) Delete(ctx context.Context, path string) error { log.Debugf("(*KrakenStorageDriver).Delete %s", path) return driver.PathNotFoundError{" — S642, L352-L354
- Conditions/notes: OCI distribution-spec conformance, referrers and sha512: nothing found (the embedded library predates OCI 1.1). Space is reclaimed by cache eviction on agents/origins and by whatever lifecycle the backend store has.
- Relevance: a distribution front can stay simple by refusing deletes; the system of record still needs a real GC.

### F399. Uber Kraken (Q12, Q13): the project is maintained in 2026 with weekly tags (v0.1.37 on 2026-11-16) carrying small fixes; server binaries are built for Linux only.
- Quote: "docs: point contact section to GitHub issues and drop Go Report Card badge" — S643, What's Changed
- Quote: "# Binaries that require Linux build" — S644, L66
- Conditions/notes: release feed shows v0.1.31 (2026-10-05) through v0.1.37 (2026-11-16) one week apart. No release binaries are attached; deployment is by container images and a Helm chart. Architectures of published images: not found.
- Relevance: alive but in maintenance mode; its design claims date from 2018–2019.

### F400. OpenShift image registry (Q3): pruning is two-level: a scheduled pruner (daily by default, keeping 3 revisions per tag and anything younger than 60 minutes) removes image objects through the cluster API and then their blobs, and a separate "hard prune" removes blobs the API no longer knows about and must be run with the registry in read-only mode.
- Quote: "If the registry is not running in read-only mode, any pushes happening at the same time as the prune will either:" — S645, Procedure step: Switch the registry to read-only mode L59-L60
- Quote: "- fail and cause new orphans, or - succeed although the images cannot be pulled (because some of the referenced blobs were deleted)." — S645, Procedure step: Switch the registry to read-only mode L63-L65
- Quote: "`spec.keepTagRevisions`:: The number of revisions per tag to keep. This is an optional field, default is `3`. The initial value is `3`." — S646, pruning custom resource fields L76
- Conditions/notes: docs branch enterprise-4.21. Image metadata lives in the cluster's etcd (the hard-prune module speaks of blobs "not referenced by the" cluster's etcd); that the registry derives from Distribution is taken from the brief, its source repository was not read. `keepYoungerThanDuration` default `60m`, schedule default "daily at midnight" (same module, L74-L77).
- Relevance: even with metadata in a database, this design still needs a stop-the-world sweep for storage the database lost track of; the goal is to make that state unreachable.

### F401. OpenShift image registry (Q3, Q11): the docs enumerate how orphaned blobs arise, including an interrupted pruner and pruner bugs, i.e. the two-step sequence (delete metadata, then delete bytes) is not crash-safe.
- Quote: "- The standard image pruner deleting an image manifest, but is interrupted before it deletes the related blobs." — S645, Hard pruning the registry L27-L28
- Quote: "- A bug in the registry pruner, which fails to remove the intended blobs, causing the image objects referencing them to be removed and the blobs becoming orphans." — S645, Hard pruning the registry L30-L31
- Conditions/notes: other listed causes: deleting an image object directly, pushes that upload blobs but never the manifest, and quota refusals.
- Relevance: the same orphan class as F366; a durable deletion intent recorded with the metadata change removes it.

## Comparison table
Columns are the 13 questions of the brief. One row per registry; the OCI distribution-spec row is the yardstick the others are measured against.

| registry | Q1 architecture | Q2 OCI spec | Q3 GC | Q4 proxy/mirror/replication | Q5 auth | Q6 search/UI/CVE | Q7 trust | Q8 layout/dedupe | Q9 limits | Q10 performance | Q11 pain points | Q12 platforms | Q13 releases |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| CNCF Distribution v3.1.2 | Go, one `registry` binary, stateless; drivers filesystem/s3/azure/gcs (+inmemory); metadata = link files in the storage driver, no DB; optional Redis descriptor cache (F001, F002, F003, F004) | Claims OCI Distribution Spec, no version; CI runs conformance v1.0.1 with 4 categories; no referrers API (PR #4828 open), no `OCI-Subject`, no `OCI-Chunk-Min-Length`; sha512 blobs re-hashed, sha512 manifests rejected; not on the OCI results site (F005, F006, F007, F009, F013) | Offline stop-the-world mark and sweep CLI; registry must be read-only; O(manifests x tags) with --delete-untagged; 4-year data-loss bug #3178; cache not coordinated; online GC requested since 2016 (F016..F022) | Pull-through cache only: one upstream, no pushes, TTL 168h, tag checked upstream on every pull, serves cached tag if upstream down but cannot start without it, no stampede coalescing, shared-blob expiry bug open since 2017; no replication (F023..F028) | silly / token (external JWT server) / htpasswd; no RBAC, users or robots; anonymous is a token-server decision (F029) | `/v2/_catalog` (100 default, 1000 max) and tags list (all tags without `n`, clamp 1000); no search, UI or CVE scanning (F030, F031) | None: no signature verification or enforcement; only structural and URL validation (F032) | `/docker/registry/v2/blobs/<alg>/<2 hex>/<digest>/data` + per-repo link files; global blob dedupe; mount = one link, needs `from`; S3 commit = copy + delete (F033, F034, F035) | Manifest 4 MiB hard-coded; upload purge 168h/24h; no HTTP timeouts, no rate limits; unbounded notification queue; see Constants (F011, F022, F036, F037, F038) | No official benchmark results; PR numbers: S3 walk 50k blobs 89.74 min to 9.85 s; 4 MiB GET 1.994 ms to 1.160 ms CPU (main); catalog 57.79 s for one page on S3 (2.8.1); tags list regression with 100k tags (F039..F045) | Proxy cache bugs, GC leftovers and downtime, storage walks, slow 2.8 to 3 migration (F046) | Linux only: amd64, arm64, armv6, armv7, ppc64le, s390x, riscv64; macOS from source; no Windows (F047) | v3.0.0 2025-04-03; v3.1.0 2026-04-06; v3.1.1 2026-05-01; v3.1.2 2026-09-24 (F048) |
| OCI distribution-spec (yardstick) | n/a | v1.1.1 (2025-01-29): 4 categories, Pull mandatory; referrers + artifactType filter; manifests >= 4 MB, 413; main (unreleased) adds digest-algorithm, tag params, upload cancel, proxying `ns` (F008, F010, F012, F013, F014, F015) | n/a | main adds a Registry Proxying section (F015) | out of scope of the spec | tags list with `n`/`last`/Link; no catalog endpoint (F008, F014) | referrers API carries signatures and SBOMs (F012) | n/a | tag <= 128 chars; manifest floor 4 MB (F010) | n/a | n/a | n/a | v1.1.1 newest release; main at 9727462 (2026-08-31) (F015) |
| Harbor | Go; about 10 containers: core, jobservice, registry (fork of Distribution v2.8.3, not v3), registryctl, portal, nginx, PostgreSQL 18, Valkey, optional Trivy adapter and exporter; metadata in PostgreSQL, bytes through Distribution drivers; Compose or Helm; HA by external PostgreSQL/Redis (F049, F050, F051, F104, F106) | only claim found: v1.1.0-rc2 (v2.9.0); conformance run nightly with all 4 categories enabled but the public report is 404 and Harbor is not on the OCI conformant list; referrers API with artifactType served from the DB; own layer assumes sha256; no tag delete over /v2/; no referrers tag schema (F054, F055, F056, F102) | online since v2.1: mark/sweep over DB rows, fixed 2 h window, blob status machine with version column; all-or-nothing mark; O(tags) per manifest until v2.15 stopped writing tag files; reports of 18-day runs, deleted live blobs, orphans on S3-compatible stores (F057 to F065, F103) | proxy-cache project: one upstream, no push, upstream HEAD on every tag pull, serves cached copy when upstream is down, double download on miss, trimmed manifest lists, 429 limiter instead of single-flight; replication by policy, push or pull, deletions only for event rules (F066 to F074, F105) | one of DB, LDAP/AD, OIDC; project-scoped RBAC with 5 built-in roles, custom roles from v2.16; robot accounts (30-day default); OIDC CLI secret; anonymous pull on public projects (F075, F076, F077) | search over project and repository names only; Angular UI; Trivy through a scanner adapter, Security Hub, SBOM on push; catalog and tag list paginated in memory (F078, F079) | cosign and notation signatures and SBOMs as accessories; pull gate checks that a signature exists, no cryptographic verification; CVE-severity pull gate (F080, F081) | Distribution v2 layout with global blob dedupe; references mirrored in PostgreSQL (blob, artifact_blob, project_blob); tags only in the DB since v2.15; mount adds a project_blob row (F063, F082, F083) | manifest 4 MiB (reason given); upload purge 168h/24h; token 30 min; job limit 24 h; registry client timeout 30 min; GC workers 1 to 10; no request rate limits (F084 to F089; Constants) | first-party k6 reports at 500 VUs: v2.15.4 API calls 2.2 s to 3.62 s average at 125.36/s to 186.23/s, pull 13.07/s, push 7.91/s (1 KiB artifact by script default), catalog 5.87 s at 40000 repositories; v2.14.4 get-artifact-by-tag 3105.41/s; cache layer about 600 to over 4000 manifest GETs/s; stack minimum 2 CPU / 4 GB (F052, F090 to F098) | GC duration and failure, proxy-cache defects, PostgreSQL contention, slow lists, replication failures, offline upgrades (F062, F072, F100, F101, F103) | Linux containers only; amd64, plus arm64 from v2.16.0; no Windows or macOS server (F099) | v2.16.0 2026-11-10; v2.15.0 2026-03-20; v2.14.0 2025-09-17; v2.13.0 2025-04-10; patches v2.15.5 2026-11-12, v2.14.6 2026-10-23 (F053) |
| Zot | Go; one binary, one config file; minimal and full builds. Blobs on local FS or S3/GCS/Azure through Distribution v3 drivers. Truth is index.json per repo; optional MetaDB (BoltDB, DynamoDB, Redis). No external service on one node; object store plus DynamoDB or Redis when instances share storage (F107, F108, F109, F110) | Claims full conformance; published v1.1 result for v2.1.19: Pass, 848 of 854, 0 failed, 2 disabled, 4 skipped. Referrers with artifactType filter. sha512 passes. OCI-only unless docker2s2 compat is set (F111, F112, F113, F114, F115) | Online, on by default; per-repo mark and sweep every 1 h; safety is mtime older than gcDelay (1 h); holds the store-wide write lock; data-loss races with S3 dedupe fixed in v2.1.22 and v2.1.23; can stall silently; no published cost (F116 to F125) | sync extension: periodic and on-demand; several upstreams with fallback URLs; tags revalidated on every request unless manifestCheckInterval; stale served when upstream fails; single-flight since v2.1.21; streaming since v2.1.23; mirror also takes pushes; static-hash scale-out; HA by mutual sync (F126 to F135) | htpasswd, LDAP, bearer, OIDC workload identity, OpenID login, API keys, mTLS; glob policies over users, groups, actions with anonymous, default, admin and CEL conditions; anonymous pull yes; no robot accounts; no stored RBAC (F136 to F140) | GraphQL search extension; embedded Trivy (15 min scan loop, DB refresh at least 2 h, not on object storage); UI and zli (F141, F142, F143) | cosign and notation verified and reported through search; no enforcement at push or pull; signatures and SBOMs as referrers; sync onlySigned is a presence check (F144, F145, F133) | OCI layout directory per repo; dedupe by hard links locally, by zero-byte placeholders plus cache DB on object stores; mount by digest from the dedupe cache; fsync opt-in (F146 to F150) | manifest 4 MiB; HTTP read and write 60 s absolute; no default page size; rate limit opt-in; upload expiry equals gcDelay; see Constants (F151 to F157) | only a sample zb output: 2202.7788 req/s catalog, 105.00491 req/s 1 MB push (c=2, n=100, localhost, hardware unstated); binary 221 MB full, 86.6 MB minimal; no RSS published; third-party pull-through numbers in F168 (F158 to F162, F168) | remote dedupe stalls and losses, store-lock latency, GC not freeing or stalling, pull-through re-pulls and first-pull waits, 60 s timeouts (F163) | linux, darwin, freebsd, windows on amd64 and arm64; S3 driver not on Windows (F164) | v2.1.23 on 2026-11-02; monthly patch line; 2026 was hardening plus GCS, Azure, quotas, streaming; v2.2.0 milestone open (F165, F166, F167) |
| Project Quay v3.18.3 | Python monolith: nginx + 3 gunicorn apps + memcached + ~30 workers in one container (F169); needs PostgreSQL, Redis, object storage; 13 storage drivers, local only for PoC (F170, F171); Go rewrite on Distribution v3 + SQLite for the mirror registry (F172) | claims OCI 1.1 (F173); CI runs pull, push, discovery, not content management, at spec v1.1.0-rc1 with tag cache off (F174); referrers + artifactType since 3.12, default 3.13 (F173, F176); sha256 only, blob DELETE refused (F175); gaps: stale tag list, missing-subject error, no OCI-Subject (F174, F176) | online, continuous, no manual run (F178); one random repo per 30 s tick, time machine 2w (F179, F180); time-based push protection 1 h raised to 6 h (F181); TOCTOU data-loss races patched 2026, new 500 open (F182, F183); orphaned storage acknowledged (F184); Go collector deleted in-flight multi-arch children (F185); no durations published | proxy cache org: one upstream, HEAD on every pull, expiry 86400 s re-armed (F187, F188); upstream-down broken until 3.18.0 (F189); accepts pushes (F190); eviction only via quota (F191); no single-flight, large blobs failed (F192); mirroring via skopeo, strips signatures (F193, F194); geo-replication with one shared DB and Redis (F195) | own token service, 6 credential backends, 1 h JWT (F196); org/team/robot RBAC with read/write/admin (F197); robots, keyless OIDC exchange (F198); anonymous pull on, push-created repos private (F199) | REST search, 10 x 10 results (F200); React UI; Clair v4 as separate service with own PostgreSQL (F201) | stores cosign artifacts, no server-side verification or enforcement found (F202); mirroring removes signatures (F193) | `<alg>/<2 hex>/<hex>` objects, global dedupe by checksum with per-repo link rows (F203); manifests as DB rows per repo (F204); mount needs `from` + read access (F205); chunks stored as separate objects and assembled at commit (F221) | layer 20G, S3 50 GB ceiling at 5 MB parts (F206); manifest limit implicit nginx 1m (F207); pages 100, tag cache 120 s (F177); upload purge 2 d, MPU 1 d (F186); rate limits off by default (F208); gunicorn 30 s timeout (F209) | vendor benchmark 0.22-0.56 img/s, 37.62-95.56 MB/s, 171 MB images, 2 replicas (F210); no results with the public tool (F211); hosted: ~5,000 req/s, 7K-9K DB connections, 60M images (F212); min 6 GB RAM per container (F213); DB-shaped slow paths (F214) | database connection storms and migration incidents (F215); upgrade breakage (F216); storage-driver and SQLite limits (F217); GC races and orphans (F182-F184); proxy cache failures (F189, F192) | Linux containers: amd64, ppc64le, s390x, arm64 since 3.17; no Windows/macOS server (F218) | v3.18.3 2026-11-13; 3.14 to 3.18 in 2025-2026; 5.0 in draft; mirror-registry-v3.0.0 tag 2026-11-09 (F219, F220, F172) |
| GitLab Container Registry | Go fork of Distribution, one binary; metadata in PostgreSQL 16+ (optional, "prefer" default since 2026-05), bytes in filesystem, S3, GCS or Azure; Redis optional (F222 to F227) | Conformance run at spec v1.0.1 only, four workflows; no referrers endpoint, `subject` stored; media-type allowlist; sha256 and sha512 only (F228 to F232) | Online, event-driven review queues filled by triggers, 24h delay, one task per run; legacy mode is offline mark-and-sweep; open data-loss bug 2381; referrer race fixed 2026-10 (F233 to F249) | No pull-through (removed 2023; Dependency Proxy is in Rails); Geo is one-way, event-driven, DR only; notifications best-effort (F250 to F252) | JWT from GitLab only; no RBAC in the registry; tag protection and immutability as patterns in the token, max 5 (F253) | Private `/gitlab/v1/` API: tag details with filter, sort, size; UI and scanning are GitLab's (F254 to F256) | Displays cosign signatures via `subject`; no verification or enforcement found (F257) | Distribution byte layout, global dedupe, links as rows; 8 tables x 64 hash partitions; mount is a row insert; pulls redirected to pre-signed URLs (F223, F258, F259, F279) | 4 MiB manifest body; optional payload limit (256KB on GitLab.com); upload purge 168h/24h; GC 24h/5s/10s/5s; S3 350 req/s (F260 to F265; Constants) | GitLab.com about 1500 reads/s and 60 writes/s (2024), 5+ PB (2021); offline GC estimate 64 days; import 8 min to 970 h; 72 s to list 500k tags (F266 to F272) | Import downtime and duration; GC queue load; upgrade breakage in 19.0; mixed GC modes destroy data; manual schema migrations before 18.3 (F240, F244, F248, F273 to F275, F280) | Linux only; amd64 and arm64 packages; CI builds linux 386/amd64/arm/arm64 (F276) | v4.43.0-gitlab 2026-10-23; database default "prefer" since GitLab 19.0 (F224, F277, F278) |
| olareg | Go; one binary or embedded library; OCI Layout directory per repository or memory; no database, no services (F281, F282) | conformance action in CI (stable and dev); site: spec v1.1, 48 of 52 workflows; referrers with artifactType filter; sha512 (F284, F285, F286) | online, in-process, per-repository stop-the-world mark-and-sweep every 15 min; 1 h grace by file mtime; race fixes in 0.1.2 and 0.2.0 (F287, F288, F289) | none; read-only mode for a preloaded directory (F290) | basic auth (bcrypt file) or opaque 5-min tokens; ACL by path, verb, member, anonymous (F291) | none; tag list sorted per request with no default page size (F292) | none; signatures are plain referrers (F293) | per-repository layout, index.json rewritten per change, no fsync; no cross-repository dedupe; mount copies bytes (F282, F283, F294) | manifest 8 MiB; referrers 4 MiB; 1000 uploads per repository with LRU eviction; no blob limit; rate limit off (F295, F296) | none published; binary about 10 MB (F297) | four issues ever; missing shared store (F282, F289) | Linux (5 arches), macOS amd64/arm64, Windows amd64 binaries; 8 Linux image platforms (F298) | v0.2.2 2026-06-24; v0.2.0 2026-04-13 added auth (F299) |
| trow | Rust (axum); SQLite plus blob files on one volume; no object storage; single replica; beta (F300, F301) | CI runs conformance v1.1.1 with all four categories; referrers without artifactType filter; SHA-256 only in practice; not on the conformance site (F302, F303) | online task every 10 min; orphans found by SQL over a trigger-maintained table; 1-day idle threshold; LRU eviction of proxied blobs at 80% of max_size (F304) | proxies any registry (f/ prefix or ns parameter); HEAD per pull; serves cached copy on upstream failure; proxied repositories refuse writes; whole image fetched on a miss; 200 ms polling for concurrent fetches (F305, F306) | one username/password; 3600 s bearer token; no per-repository policy (F307) | catalog and tag list from SQL; no UI, search or scanning (F310) | admission webhook with allow/deny prefixes; no signature checks (F308) | global blobs by digest; manifests in SQLite; per-repository association rows (F309) | manifest 4 MiB; unlimited default page; 30 s shutdown; 256Mi chart limit (F310) | none published; single-threaded runtime, one SQLite writer (F301, F311) | no data migration at 0.7.0; old install/TLS/push issues (F300, F311) | Linux amd64 and arm64 images; no binaries, no macOS or Windows artifacts (F311) | v0.10.0 2026-04-13; SQLite since v0.7.0 2025-01-28 (F300, F311) |
| angos | Rust (hyper); no database and no lock service; all state in a directory or S3 bucket; stateless replicas; one binary with server, worker, scrub, prune (F312) | claims OCI 1.1; CI conformance with every toggle on four backends; referrers; sha512; not on the conformance site (F322) | online; DELETE answers 202; scrub with run markers and 300 s grace; prune for retention; O(all keys) walk; data-loss fixes in 1.2.0 (F315, F316, F317) | read-only pull-through repositories; HEAD per pull for mutable tags; 503 when uncached and upstream down; blob fetched twice on a miss; upstream fallback list; queued replication, last-writer-wins (F318, F319) | mTLS, registry tokens, OIDC, basic; CEL policies plus webhook; default deny (F320) | web UI; listings API; Trivy or Grype reports stored as referrers (F321) | no signature verification found (F321) | global content-addressed blobs; write-once metadata keys; tags as timestamp-ordered entries (needs synchronized clocks) (F313, F314) | manifest 5 MiB; blob 100 GiB; request timeout 3600 s; S3 part 50 MiB; GC grace 300 s; no chunk minimum, small chunks staged (F323, F326) | none published; presigned-URL redirects on S3; upload completion copies bytes (F324) | storage layout changed repeatedly in 2026; scrub must match the server version (F316, F325) | Linux and macOS, amd64 and arm64; no Windows (F325) | v1.13.0 2026-11-17; v1.0.0 2026-01-01; lock-free redesign in 1.6.0 (F312, F325) |
| nora | Rust; one binary for many package formats; no database; local disk or S3/GCS; single instance by design (F327) | labelled Spec 1.1; no referrers, no mount, two-level names only; no conformance run found (F330) | background mark-and-sweep, off by default, every 24 h, 7-day grace; issue 584 write-vs-GC race (F331) | revalidate every pull by default or a TTL; serve stale; 30 s negative cache; single-flight; a name can be both hosted and proxied (F332) | htpasswd users; token roles read, write, admin; OIDC for CI; anonymous Docker pull off by default (F336) | web UI; in-memory search; no scanner (F336) | curation of proxied packages; no image signature verification found (F336) | blob keys per image name; no cross-repository dedupe (F335) | reasons stated: 100 upload sessions, 2 GB per session, 30 min TTL, 4 MiB manifest; default-on rate limits (F333, F334) | 30.8 MB binary, 62–69 ms start, about 18 MB idle RSS (v1.4.6); k6: 1,825 req/s local disk, 131 req/s S3 with 40% errors (F328, F329) | GC race (584); S3 small-object throughput (F329, F331) | Linux amd64 and arm64 only (F337) | v1.4.7 2026-11-18; 1.0.0 2026-07-13 (F327, F337) |
| mcronce/oci-registry | Rust; pull-through cache only; filesystem or S3 (F338) | conformance suite never run; SHA-256 only (F338) | cache expiry by invalidation time; cleanup loop every 300 s (F340) | any upstream; 14-day default invalidation; no push; no single-flight (F338, F340) | none (F338) | not found | not found | not examined | 14-day invalidation default (F340) | anecdote: about 60 MiB for five replicas against about 130 MiB for Distribution (F339) | not examined | x86-64 and aarch64 Dockerfiles; not examined further | v0.4.5 2024-04-23; unmaintained since (F338) |
| container_registry-rs | Rust crate for axum; local filesystem; basic auth (F341) | follows spec v1.0.1 best effort (F341) | not found | none | basic auth (F341) | none | none | not examined | not examined | not found | not examined | library crate | 0.3.1 2024-08-14 (F341) |
| rk8s distribution | Rust; filesystem blobs plus PostgreSQL; work in progress (F342) | lite implementation; not examined (F342) | not examined | not examined | JWT; GitHub OAuth; user management (F342) | not examined | not examined | not examined | JWT lifetime 3600 s in the example config (F342) | not found | not examined | not examined | rk8s v0.1.3 2026-04-26 (F342) |
| go-containerregistry pkg/registry | Go library; in-memory maps, optional blobs on disk; test fixture (F344) | referrers opt-in without artifactType filter; no catalog pagination; no blob existence check (F345) | none (F346) | none | none | none | none | maps in memory; flat blob directory with --disk (F344) | no limits; uploads buffered in memory (F346) | not found | not examined | Go library; not examined | v0.22.1 2026-09-04 (F344) |
| cloudflare/serverless-registry | TypeScript Worker; R2 bucket is the only state (F347) | referrers implemented; conformance status not found (F347) | on demand per repository; lock-free marks in R2; conflict is an error, never a delete (F349) | pull fallback copies missing content into R2 (F350) | one username/password or JWT public key; no per-repository scoping (F347) | none found | none found | per-repository keys; mount is a symlink object (F350) | 500 MB layer via docker push; chunks 5 MiB to 5 GiB, 100 MB per request; 2 h upload state (F348) | not found | not examined | Cloudflare Workers runtime only | no releases; commit a73605d 2026-08-17 (F347) |
| peryx (Rust; examined briefly) | Rust, one executable for PyPI and OCI; content-addressed blobs on local disk or S3-compatible store; metadata in a node-local redb file; modes none, dc, ha (F351) | distribution-spec routes with referrers and OCI-Subject; sha256 only; conformance suite at a pinned commit in CI, sha512 test off; not on the OCI site (F352) | offline orphan purge outside ha; ha design with reference revision, per-digest guard, replica frontier gate and term fencing; ha incomplete for OCI (F353, F354) | pull-through with TTL window (300 s), HEAD revalidation, stale-on-error bound, 30 s negative cache, single-flight, streaming on miss; replication in dc and ha modes (F354, F355) | not examined (scoped tokens and role grants exist in the docs tree) | not examined (web UI and search exist in the docs tree) | not examined | one blob store, sha256 keys, per (index, repository, digest) links; mount copies a link after a pull-permission check (F351, F353) | manifest 4 MiB with 413; upload idle 1 h, pass every minute; artifact size limit per index (F356) | own comparison on an Apple M4: six images warm 0.7 s against 2.7 s Distribution and 6.6 s zot; 6,838 MB/s hot layer (F357) | not examined | Linux and macOS on x86_64 and aarch64; Windows x86_64 (F358) | 2026.1118.1 on 2026-11-18; first changelog entry 2026-09-25 (F358) |
| Gitea packages (container) | Go, part of the forge binary; rows in the forge SQL DB, bytes in packages storage local/MinIO/Azure (F362) | Docs claim OCI; no referrers (code TODO), sha256 only, tags case-folded (F363, F365); no conformance result found | No GC; daily cron: cleanup rules + unreferenced blobs older than 24h, bytes deleted after DB commit, failures logged (F366); digest versions of replaced multi-arch images not cleaned (F367) | none found | Bearer token from forge users/scopes, 24h; ghost user for anonymous; visibility per owner (F369) | Forge UI; `_catalog` max 100; no CVE scanning (F369) | none (F369) | `aa/bb/<sha256>` content store, global dedupe, mount = DB insert (F362, F370) | manifest 10 MiB, catalog 100, tag list unbounded, token 24h (F368) | not found; node-local upload temp files, four hashes per blob (F370) | private images (20 comments), tag case overwrite, uncleaned digest versions (F365, F367, F369) | Linux, macOS, Windows, FreeBSD; amd64, arm64 and more (F372) | v28.4.0, 2026-11-15 (F372) |
| Forgejo packages (container) | as Gitea; separate code base (F362, F371) | user-run conformance: 847 pass / 184 fail of 1034; referrers and sha512 fail (F364) | as Gitea plus a full-scan sweep of unreferenced `sha256:*` versions in batches of 500 (F367) | none found | as Gitea, plus quota (F371) | as Gitea | none | as Gitea | manifest 10 MiB; quota 413 (F368, F371) | not found | Range off by one on upload GET, empty tag lists, unique-constraint 500s (F371) | Linux only: amd64, arm64, arm-6 (F372) | v17.0.2, 2026-11-18 (F372) |
| Pulp container | Python/Django plugin: API + content app + workers on PostgreSQL; artifacts on FS or S3 (F373) | Claims OCI distribution spec; no referrers route; sha256-only digest handling (F374) | No GC; orphan cleanup after 1440 min protection, docs warn of deletion during long tasks (F375) | Sync (mirror/additive); pull-through is tech preview, revalidates tag each pull, falls back to local tag, read-only (F376) | Token auth, 300 s; RBAC; no private repos without token auth (F378) | REST API and `_catalog`; no CVE scanning (F378) | Hosts/mirrors cosign signatures, SBOMs; signing service; no enforcement found (F378) | Global artifact dedupe; cross-repo mount; every push = repository-exclusive task and new repository version (F377) | page 100 (max 1000), signature 4 MB, 2 s task wait (F379) | only a migration timing: 16115 manifests in 2m4.8s (F379) | throttled concurrent pushes, intermittent 404 on layer push (F377) | Linux (F379) | 2.30.1, 2026-10-28 (F379) |
| Sonatype Nexus Repository | Java; Docker format not in the open-source Core any more; H2 or PostgreSQL; file/S3/Azure/Google blob stores (F380) | OCI image spec 1.0.0/1.0.1 since 3.71; referrers not found (F382) | Task chain: policies soft-delete tags, Docker GC task soft-deletes untagged, compact hard-deletes; digest-only images lost (F383); cleanup bug 3.31–3.41 (F384) | proxy repos with max-age recheck; groups; push to group paid (F386) | Docker Bearer Token Realm; privileges (F387) | UI and search (closed product) | Content trust not handled (F387) | blob stores with soft delete; dedupe not documented | CE 40,000 components / 100,000 requests per day; 20 port connectors (F381, F388) | cleanup 10.6 ms per component, >10 h for 3.2M (F385) | cleanup data loss advisory; migrator breaks OCI pulls (F384, F389) | Linux, Windows, macOS; Intel/AMD (F389) | 3.98.3, 2026-11-19 (F389) |
| Dragonfly | Not a registry: P2P layer (Manager, Scheduler, Seed Peer, Peer); Go control plane, Rust client; MySQL+Redis unless lightweight (F390) | n/a (proxies a registry) | cache eviction by TTL and disk watermarks (F393) | pull-only mirror for containerd, preheat, Harbor integration; no pushes (F391) | upstream registry's | Manager console | n/a | piece-based tasks keyed by blob digest (F391) | client defaults in code (F393) | 1.47 s vs 37.48 ms one layer; aggregate claims (F392) | not researched | Linux x86_64/aarch64 (F394) | v2.5.4 and client v1.5.14, 2026-11-13; CNCF graduated 2025-10-28 (F394) |
| Uber Kraken | Go; agent, origin, tracker, proxy, build-index over pluggable backends (F395) | Docker distribution library of 2019; OCI manifest/index accepted; no referrers (F398) | none through the registry API: delete not found, walk unimplemented; cache TTI eviction (F397, F398) | read-only registry/HTTP backends; async cross-cluster replication (F395) | not found (README: uploader authentication via TLS) | none | none | tags in build-index without consistency; blobs on origin hash ring (F397) | 20 GB blob cap, 15k hosts, TTIs (F397) | 2600 hosts, 3 GB image: p50 10 s, p99 18 s, p99.9 22 s (F396) | tag mutation unreliable (F397) | Linux (F399) | v0.1.37, 2026-11-16, weekly tags (F399) |
| OpenShift image registry | Distribution derivative; image metadata in cluster etcd (F400) | not researched | scheduled pruner via API + read-only "hard prune" for orphans (F400, F401) | not researched | cluster RBAC (not researched) | not researched | not researched | not researched | pruner: daily, keep 3 tag revisions, 60m (F400) | not found | orphans from interrupted pruner (F401) | not researched | docs branch enterprise-4.21 |

### Digests across registries

These five tables restate findings side by side; every cell rests on the findings it cites and adds nothing to them.

**Digest 1: garbage collection, and what protects a push in flight**

| registry | mode | how "unreferenced" is decided | protection of a push in flight | documented failure |
|---|---|---|---|---|
| CNCF Distribution | offline CLI, registry read-only or stopped (F016) | walk of every repository, manifest and blob through the storage driver (F017) | none: the operator must stop writes (F016) | `--delete-untagged` deleted platform manifests of tagged indexes, 2020 to 2024 (F019); blob removal not coordinated with the descriptor cache (F020) |
| Harbor | online job since v2.1 (F057) | database rows: blobs with no project association (F058) | fixed 2-hour window plus a blob status machine with a version column (F057, F059) | user reports of referenced blobs deleted, cause unconfirmed (F064); orphaned bytes on S3-compatible stores (F103); runs of days (F062) |
| zot | online, periodic per repository, under the store-wide write lock (F116, F119) | reachability from the repository's index.json (F117) | modification time younger than gcDelay, 1 hour (F118) | S3 dedupe placeholders: blobs pushed one second earlier deleted, fixed in v2.1.23 (F120); remote dedupe content losses (F121) |
| Quay | online, continuous workers (F178, F179) | references in the database; storage deleted afterwards, outside the transaction (F180) | time: hidden temporary tags and UploadedBlob rows, 1 hour raised to 6 (F181) | TOCTOU races patched in March 2026 with a Redis lock that does not hold without Redis (F182); new HTTP 500 open (F183); mirror-registry collector broke multi-arch pushes (F185) |
| GitLab registry | online, one review task per event; legacy mode offline (F233, F234) | trigger-filled review queues over database references (F235, F236, F237) | review delay of one day; row locks for manifest races; the blob existence race is left open (F236, F241, F242) | open bug: manifest and layer rows not atomic, layers deleted a day later (F244); referrer race fixed in v4.43.0 (F245) |
| olareg | online ticker, one repository at a time, requests to it blocked meanwhile (F287) | reachability from index.json (F288) | grace period (F288, F295) | races fixed in v0.1.2 and v0.2.0 (F289) |
| trow | online, every 10 minutes (F304) | SQL over trigger-maintained manifest-to-blob rows (F309) | one day without access (F304) | none found |
| angos | online, separate scrub and prune commands beside live servers (F315) | walk of every key in three passes (F317) | grace period 300 s plus run markers; needs synchronized clocks (F315, F316) | scrub bugs that deleted live data, recorded in the 2026 changelog (F316) |
| nora | background mark and sweep, off by default, every 24 hours (F331) | orphan snapshot (F331) | 7-day grace period (F331) | blob pushed after the snapshot deleted (issue 584), the reason for the grace period (F331) |
| cloudflare/serverless-registry | on request, per repository (F349) | marks written to the object store by collector and by each manifest insertion (F349) | each side re-checks the other's mark; a conflict is an error, not a deletion (F349) | none found |
| peryx | offline purge command; scheduled reclamation only in `ha` mode (F353, F354) | reference inventory stamped with a revision (F354) | per-digest guard that rejects a racing reference commit; replica frontier gate; term fencing (F354) | none found; `ha` mode incomplete for OCI (F354) |
| Gitea / Forgejo | daily cron cleanup, no collector (F366) | database rows no file references, older than 24 hours (F366) | age only (F366) | Gitea leaves replaced per-architecture versions behind (F367); storage delete failures are logged, not retried (F366) |
| Pulp container | orphan-cleanup task (F375) | content no repository version references (F375) | ORPHAN_PROTECTION_TIME, 1440 minutes, which must exceed the longest task (F375) | the window caveat itself (F375) |
| Nexus Repository | chain of scheduled tasks: soft delete, then compaction (F383) | tags, then manifests and layers no tag references (F383) | 60-minute upload grace (F388) | digest-only images deleted irreversibly (F383); cleanup bug in 3.31.0 to 3.41.0 deleted binaries (F384) |
| OpenShift image registry | scheduled pruner through the cluster API, plus a hard prune that needs read-only mode (F400) | API objects, then blobs the API no longer knows (F400) | anything younger than 60 minutes is kept (F400) | interrupted pruner leaves orphans (F401) |
| Uber Kraken | none through the registry interface (F398) | n/a | n/a | n/a |

**Digest 2: pull-through cache semantics**

| registry | upstreams | accepts pushes | freshness of a tag | upstream down | concurrent misses |
|---|---|---|---|---|---|
| CNCF Distribution | one per registry instance (F023) | no (F023) | checked upstream on every pull; cached content expires after 168h (F024, F026) | cached tag served; cannot start without the upstream (F026) | each request fetches from the upstream (F027) |
| Harbor | one endpoint per proxy-cache project (F066) | no (F066) | upstream HEAD on every pull, no TTL; 7-day retention policy (F067, F071) | cached copy served (F067) | not coalesced; each blob downloaded twice on a miss; optional limiter answers 429 (F068, F070) |
| zot | several, each with fallback URLs (F127) | yes (F127) | revalidated on every tag request unless an interval is set (F128) | cached image served; 503 if nothing is cached, since v2.1.22 (F129) | singleflight since v2.1.21; streaming opt-in since v2.1.23 (F130, F131) |
| Quay | one per organization (F187) | yes (F190) | upstream HEAD on every tag pull; expiration 86400 s re-armed (F188) | timed out in 3.12 to 3.15, fixed in v3.18.0 (F189) | no single-flight (F192) |
| GitLab registry | none; proxy mode removed in 2023 (F250) | n/a | n/a | n/a | n/a |
| trow | any registry, without configuration (F305) | no (F305) | upstream HEAD on every tag pull (F305) | cached copy returned (F305) | waiters poll every 200 ms with no deadline (F306) |
| angos | per repository (F318) | no (F318) | mutable tags revalidated on each pull, immutable tags never (F318) | cached content served, uncached gets 503 (F318) | a blob miss is fetched twice (F318) |
| nora | several, as a fallback chain (F332) | yes: a pushed tag is authoritative and never revalidated (F332) | revalidated on every pull by default, optional staleness window (F332) | last cached manifest served, marked stale (F332) | collapsed into one upstream fetch (F332) |
| mcronce/oci-registry | any (F338) | no (F338) | invalidation time per upstream, 14 days by default (F340) | not found | none: fetched twice (F340) |
| peryx | per cached index (F351) | only hosted indexes accept uploads (F351) | served from cache for 300 s, then a HEAD; 404 remembered 30 s (F355) | stale tag within 300 s, else the failure is forwarded (F355) | single-flight, streamed while stored (F355) |
| Pulp container | any v2 registry as remote; tech preview (F376) | no (F376) | upstream tag re-checked on every tag pull (F376) | local tag served (F376) | not found |
| Nexus Repository | per proxy repository; groups merge members (F386) | group push is a paid feature (F386) | re-check after configured maximum ages (F386) | not found | not found |

**Digest 3: published performance numbers (conditions in the findings)**

| registry | figure | finding |
|---|---|---|
| CNCF Distribution | S3 walk of about 50k blobs: 89.74 minutes (extrapolated) to 9.85 seconds (PR description, 2021) | F040 |
| CNCF Distribution | manifest verify 128,746,104 ns/op to 21,460,250.5 ns/op with 10 ms injected per storage operation (main, unreleased) | F041 |
| CNCF Distribution | `_catalog?n=50` took 57.79 s with 1300+ repositories on S3 (user, 2.8.1) | F043 |
| CNCF Distribution | repositories of about 100,000 tags: a sync job of 30-50 seconds on 2.8.3 takes up to 30 minutes on 3.1.2 (user) | F031 |
| Harbor | k6, 500 virtual users, v2.15.4, AWS m8a.4xlarge workers: API calls 125.36/s to 186.23/s at 2.2 s to 3.62 s average; pull 13.07/s; push 7.91/s | F090, F091 |
| Harbor | v2.14.4, same stated environment: get-artifact-by-tag 3105.41/s at 122.68 ms | F092 |
| Harbor | `/v2/_catalog` 75.97/s at 5.87 s average with 40000 repositories (v2.15.4) | F093 |
| Harbor | manifest GETs about 600/s without and over 4000/s with the Redis cache layer | F094 |
| Harbor | deleting one manifest: about 39 s on S3; over an hour with more than 60,000 tags on GCS | F061 |
| zot | sample zb output: 2202.7788 requests/s Get Catalog, 105.00491 requests/s 1 MB push, concurrency 2, 100 requests, localhost, hardware unstated | F158 |
| zot | binary 221 MB full, 86.6 MB minimal (linux/amd64, v2.1.23) | F160 |
| Quay | 0.22 to 0.56 image pulls/s (37.62 to 95.56 MB/s) for 171 MB, 100-layer images on 2 replicas of 2 to 16 vCPU (vendor, 2026-10-29) | F210 |
| GitLab registry | offline GC estimated at 64 days for GitLab.com; 15k blobs on S3 from 2 hours to 93 seconds | F233, F269 |
| GitLab registry | import to the database: 8 minutes for 100 GiB on S3 up to 970 hours for 609 TiB on S3 | F270 |
| GitLab registry | GitLab.com, May 2024: about 1500 read and about 60 write requests per second | F267 |
| GitLab registry | tag listing 72 seconds with about 500,000 tags in one repository (user, v4.34.0) | F272 |
| nora | 30.8 MB binary, 62-69 ms start, about 18 MB idle RSS (v1.4.6) | F328 |
| nora | k6, 50 virtual users, 100 KB raw files: 1,825 requests/s local disk; 131 requests/s with 40% errors on an S3-compatible store | F329 |
| peryx | own comparison on an Apple M4: six images warm 0.7 s against Distribution 2.7 s and zot 6.6 s; hot 30 MB layer 6,838 MB/s against zot 2,772 | F357 |
| Nexus Repository | cleanup 10.6 ms per component, more than 10 hours for 3.2 million components | F385 |
| Uber Kraken | 3 GB image pulled by 2600 hosts at once: p50 10 s, p99 18 s, p99.9 22 s | F396 |
| Dragonfly | one layer 1.47 s from source, 37.48 ms from a remote peer's cache (kind cluster, 2023) | F392 |
| third party | pull-through comparison of zot and Distribution on one Apple M4 host (secondary) | F168 |
| Distribution-based registry, production trace (peer-reviewed) | 75 days, over 38 million requests, 181.3 TB transferred; 65% of layers under 1 MB and 80% under 10 MB | F359 |
| Distribution-based registry, backends (peer-reviewed) | layers under 1 MB: 0.008 s from memory in front of Swift, about 0.013 s local SSD, 0.07 s Swift, 0.11 s by redirect; 32-core, 64 GB registry node, 2017 | F360 |
| DupHunter (peer-reviewed) | block-level deduplication with erasure coding: ratio 6.9, GET layer 6.37x slower; file-level with 3 replicas: ratio 2.1, 1.03x slower | F361 |

**Digest 4: manifest size limit**

| registry | limit | status above it | reason given | finding |
|---|---|---|---|---|
| OCI distribution-spec v1.1.1 | registries SHOULD support at least 4 megabytes | SHOULD be 413 | n/a | F010 |
| CNCF Distribution | 4 MiB, hard-coded | MANIFEST_INVALID, not 413 | no | F011 |
| Harbor | 4 MiB | 413 | yes | F084 |
| zot | 4 MiB since v2.1.16 | 413 | yes | F151 |
| Quay | none in the registry code; nginx body limit only (1m by inference) | 413 from nginx, by inference | no | F207 |
| GitLab registry | 4 MiB read limit; optional payloadsizelimit, 256KB on GitLab.com | not stated in the finding | no | F260 |
| olareg | 8 MiB | 413 | no | F295 |
| trow | 4 MiB | MANIFEST_INVALID, 413 is a TODO | no | F310 |
| angos | 5 MiB | not stated in the finding | no | F323 |
| nora | 4 MiB | not stated in the finding | yes: "matches common registry limits" | F333 |
| peryx | 4 MiB | 413 with code SIZE_INVALID | no | F356 |
| Gitea / Forgejo | 10 MiB | 413 | cites the spec | F368 |

**Digest 5: conformance, referrers and digest algorithms**

| registry | conformance evidence | referrers API | digests beyond sha256 | finding |
|---|---|---|---|---|
| CNCF Distribution | CI pinned to the v1.0.1 suite, four categories | no (PR open) | blobs re-hashed to sha256; sha512 manifests rejected | F005, F006, F007 |
| Harbor | nightly run with four categories; public report returns 404 | yes, from its database, with artifactType filter | own layer assumes sha256 | F054, F055, F056 |
| zot | published v1.1 result: 848 of 854 checks pass, 0 fail (v2.1.19) | yes, with artifactType filter | sha512 passes conformance | F112, F113, F114 |
| Quay | CI without content management, at v1.1.0-rc1 | yes since v3.12.0, default since v3.13.0 | sha256 only | F173, F174, F175, F176 |
| GitLab registry | CI at v1.0.1 only | no endpoint; referrers only through its own tag-list API | database represents sha256 and sha512 | F228, F229, F232 |
| olareg | OCI site: 48 of 52 workflows (v0.2.2, spec v1.1) | yes, with artifactType filter | sha512 accepted | F284, F285, F286 |
| trow | CI runs the v1.1.1 action with four categories | yes since v0.7.0, no artifactType filter | hashes with SHA-256 only | F302, F303 |
| angos | CI runs the conformance binary with every toggle on | yes | sha512 enabled in its CI run | F322 |
| nora | no run found | no | not found | F330 |
| peryx | CI at a pinned suite commit, sha512 test off | yes | no: sha256 only | F352 |
| Gitea / Forgejo | user run against Forgejo: 847 of 1034 checks pass | no | sha256 only | F363, F364 |
| Pulp container | no run found | no route registered | by-digest lookups only for `sha256:` | F374 |
| Nexus Repository | no statement of distribution-spec 1.1 | no statement | no statement | F382 |
| Uber Kraken | built on the 2019 Docker distribution library | no | not found | F398 |

## Catalogue of constants
Every limit, default and interval found, with the place that defines it and whether a reason is written down there. "none" in the last column means no reason is given at the definition or in the documentation read.

| registry | constant | value | where (URL) | justification given? |
|---|---|---|---|---|
| Distribution | max manifest body (`maxManifestBodySize`) | 4 MiB (`4 * 1024 * 1024`), not configurable | https://github.com/distribution/distribution/blob/v3.1.2/registry/handlers/manifests.go#L31 | none in code; equals the spec's "at least 4 megabytes" guidance (F010) |
| Distribution | max blob upload body | unlimited (limit `-1`) | https://github.com/distribution/distribution/blob/v3.1.2/registry/handlers/blobupload.go#L169 | none |
| Distribution | catalog default page (`defaultReturnedEntries`) | 100 entries | https://github.com/distribution/distribution/blob/v3.1.2/registry/handlers/catalog.go#L16 | none |
| Distribution | catalog max page (`catalog.maxentries`, `defaultMaxEntries`) | 1000 entries; larger `n` gets 400 | https://github.com/distribution/distribution/blob/v3.1.2/configuration/configuration.go#L14-L15 | yes, indirectly: advisory GHSA-hqxw-f8mx-cpmw (unbounded `n` allocated a slice of n strings); the number 1000 itself has no reason |
| Distribution | tags max page (`tags.maxtags`, `defaultMaxTags`) | 1000 tags; larger `n` is clamped; no `n` returns all tags | https://github.com/distribution/distribution/blob/v3.1.2/configuration/configuration.go#L17-L18 | none for the number; clamping justified by the spec's Link-header rule (code comment) |
| Distribution | upload purge age (`uploadpurging.age`) | 168h | https://github.com/distribution/distribution/blob/v3.1.2/registry/handlers/app.go#L1072 | none |
| Distribution | upload purge interval (`uploadpurging.interval`) | 24h | https://github.com/distribution/distribution/blob/v3.1.2/registry/handlers/app.go#L1073 | none |
| Distribution | upload purge start jitter | random 0 to 59 minutes; 30 minutes if the random source fails | https://github.com/distribution/distribution/blob/v3.1.2/registry/handlers/app.go#L1137-L1143 | none ("sleep 30min for failure case") |
| Distribution | upload with missing `startedat` | treated as started 10000 hours in the future | https://github.com/distribution/distribution/blob/v3.1.2/registry/storage/purgeuploads.go#L25-L26 | "default to far in future to protect against missing startedat" |
| Distribution | proxy cache TTL (`proxy.ttl`, `repositoryTTL`) | 168h (7 days); 0 = never expire | https://github.com/distribution/distribution/blob/v3.1.2/registry/proxy/proxyregistry.go#L26 | none |
| Distribution | proxy cache write timeout (`proxy.cachewritetimeout`) | 5 minutes | https://github.com/distribution/distribution/blob/v3.1.2/registry/proxy/proxyregistry.go#L60-L61 | "ensures that cache writes don't hang indefinitely if the storage backend is slow"; the value has no reason; undocumented in configuration.md |
| Distribution | proxy scheduler state save period (`indexSaveFrequency`) | 5 s | https://github.com/distribution/distribution/blob/v3.1.2/registry/proxy/scheduler/scheduler.go#L21 | none |
| Distribution | S3 multipart min part (`minChunkSize`) | 5 MiB | https://github.com/distribution/distribution/blob/v3.1.2/registry/storage/driver/s3-aws/s3.go#L48-L50 | "S3 API requires multipart upload chunks to be at least 5MB" |
| Distribution | S3 default part (`chunksize`, `defaultChunkSize`) | 10 MiB (`2 * minChunkSize`) | https://github.com/distribution/distribution/blob/v3.1.2/registry/storage/driver/s3-aws/s3.go#L52 | none; docs: "faster connections benefit from larger chunk sizes" |
| Distribution | S3 max part (`maxChunkSize`) | 5 GiB on 64-bit; `math.MaxInt32` on arm/386 | https://github.com/distribution/distribution/blob/v3.1.2/registry/storage/driver/s3-aws/s3_64bit.go | "S3 API requires max upload chunk to be 5GB" |
| Distribution | S3 multipart copy part (`multipartcopychunksize`) | 32 MiB | https://github.com/distribution/distribution/blob/v3.1.2/registry/storage/driver/s3-aws/s3.go#L55-L58 | "Empirically, 32 MB is optimal." (no measurement recorded) |
| Distribution | S3 multipart copy threshold (`multipartcopythresholdsize`) | 32 MiB | https://github.com/distribution/distribution/blob/v3.1.2/registry/storage/driver/s3-aws/s3.go#L64-L67 | "Empirically, 32 MB is optimal." (no measurement recorded) |
| Distribution | S3 multipart copy concurrency (`multipartcopymaxconcurrency`) | 100 | https://github.com/distribution/distribution/blob/v3.1.2/registry/storage/driver/s3-aws/s3.go#L60-L62 | none |
| Distribution | S3 list page (`listMax`) | 1000 keys | https://github.com/distribution/distribution/blob/v3.1.2/registry/storage/driver/s3-aws/s3.go#L70-L71 | "the largest amount of objects you can request from S3 in a list call" |
| Distribution | S3 pre-signed redirect URL lifetime | 20 minutes, not configurable | https://github.com/distribution/distribution/blob/v3.1.2/registry/storage/driver/s3-aws/s3.go#L1057 | none |
| Distribution | Azure signed URL lifetime | 20 minutes ("default expiration") | https://github.com/distribution/distribution/blob/v3.1.2/registry/storage/driver/azure/azure.go#L467 | none |
| Distribution | Azure max write chunk (`maxChunkSize`) | 4 MiB | https://github.com/distribution/distribution/blob/v3.1.2/registry/storage/driver/azure/azure.go#L40 | none at the definition (a later comment says uploads <= 4 MiB let Azure compute MD5) |
| Distribution | Azure copy-status retries (`max_retries`, `retry_delay`) | 5 retries; 100ms multiplied by retry number | https://github.com/distribution/distribution/blob/v3.1.2/docs/content/storage-drivers/azure.md | none |
| Distribution | GCS chunk (`chunksize`) | code default 16 MiB, min 256 KiB, multiple of 256 KiB; docs say default 5242880 | https://github.com/distribution/distribution/blob/v3.1.2/registry/storage/driver/gcs/gcs.go#L50-L51 | none; docs and code disagree |
| Distribution | GCS concurrency (`maxconcurrency`) | default 50, minimum 25; undocumented | https://github.com/distribution/distribution/blob/v3.1.2/registry/storage/driver/gcs/gcs.go#L52-L53 | none |
| Distribution | GCS retries (`maxTries`) | 5 | https://github.com/distribution/distribution/blob/v3.1.2/registry/storage/driver/gcs/gcs.go#L58 | none |
| Distribution | filesystem concurrency (`maxthreads`) | default 100, minimum 25 | https://github.com/distribution/distribution/blob/v3.1.2/registry/storage/driver/filesystem/driver.go#L24-L29 | "Each operation spawns a new thread and may cause thread exhaustion issues if many are done in parallel." (docs); the numbers have no reason |
| Distribution | in-memory blob descriptor cache (`blobdescriptorsize`, `DefaultSize`) | 10000 descriptors; 0 = unlimited | https://github.com/distribution/distribution/blob/v3.1.2/registry/storage/cache/memory/memory.go#L16-L18 | none |
| Distribution | Redis descriptor cache expiry | none (keys never expire) | https://github.com/distribution/distribution/blob/v3.1.2/docs/content/about/configuration.md | "the registry does not set an expiration value on keys" so Redis must use allkeys-lru |
| Distribution | blob read buffer (`fileReaderBufferSize`) | 4 MiB per GET | https://github.com/distribution/distribution/blob/v3.1.2/registry/storage/filereader.go#L13-L18 | none: "TODO(stevvooe): Set an optimal buffer size here"; PR #5081: "That commit message records no measurement or specific reasoning." |
| Distribution | tag lookup concurrency (`storage.tag.concurrencylimit`, `DefaultConcurrencyLimit`) | `runtime.GOMAXPROCS(0)` | https://github.com/distribution/distribution/blob/v3.1.2/registry/storage/registry.go#L15 | none for the default; the knob exists because lookup reads every tag link |
| Distribution | HTTP server read/write/idle timeouts | not set (Go defaults: none) | https://github.com/distribution/distribution/blob/v3.1.2/registry/registry.go#L169-L172 | none |
| Distribution | HTTP drain timeout (`http.draintimeout`) | 0 = no graceful drain | https://github.com/distribution/distribution/blob/v3.1.2/registry/registry.go#L325-L327 | none |
| Distribution | request rate limit / concurrency limit | none (no option) | https://github.com/distribution/distribution/blob/v3.1.2/configuration/configuration.go#L137-L187 | n/a |
| Distribution | notification endpoint timeout / threshold / backoff | 1 s / 10 failures / 1 s when unset | https://github.com/distribution/distribution/blob/v3.1.2/notifications/endpoint.go#L26-L36 | "reasonable default" (comment), no reason for the values |
| Distribution | notification queue length | unbounded, in memory | https://github.com/distribution/distribution/blob/v3.1.2/notifications/sinks.go#L12-L14 | none ("It is unbounded and thread safe") |
| Distribution | storage driver health check interval (`defaultCheckInterval`) | 10 s; threshold unset = 1 failure | https://github.com/distribution/distribution/blob/v3.1.2/registry/handlers/app.go#L55-L56 | none |
| Distribution | JWT validity leeway (`Leeway`) | 60 s on nbf and exp | https://github.com/distribution/distribution/blob/v3.1.2/registry/auth/token/token.go#L21-L23 | "to account for clock skew as per https://tools.ietf.org/html/rfc7519#section-4.1.5" |
| Distribution | generated HTTP secret size (`randomSecretSize`) | 32 random bytes | https://github.com/distribution/distribution/blob/v3.1.2/registry/handlers/app.go#L51-L53 | none |
| Distribution | repository name length (`RepositoryNameTotalLengthMax`) | 255 characters | https://github.com/distribution/distribution/blob/v3.1.2/vendor/github.com/distribution/reference/reference.go#L38-L39 | none at the definition; the OCI spec notes many clients cap host + name at 255 |
| OCI distribution-spec v1.1.1 | manifest size a registry should accept | at least 4 megabytes; 413 above the registry's limit | https://github.com/opencontainers/distribution-spec/blob/v1.1.1/spec.md#pushing-manifests | none |
| OCI distribution-spec v1.1.1 | tag length | at most 128 characters, `[a-zA-Z0-9_][a-zA-Z0-9._-]{0,127}` | https://github.com/opencontainers/distribution-spec/blob/v1.1.1/spec.md#pulling-manifests | none |
| OCI distribution-spec v1.1.1 | referrers tag schema `<ref>` | digest limited to 64 characters | https://github.com/opencontainers/distribution-spec/blob/v1.1.1/spec.md#referrers-tag-schema | none |
| OCI distribution-spec main (unreleased) | tags per manifest push via `tag` parameters | SHOULD support at least 10; MAY answer 414 | https://github.com/opencontainers/distribution-spec/blob/97274622c11112caa21efb8c52acca3c6b8fa7f1/spec.md#pushing-manifests | none |
| Harbor | GC reserve time window | 2 hours (`DefaultGCTimeWindowHours = int64(2)`); override only by env `GC_TIME_WINDOW_HOURS` | https://github.com/goharbor/harbor/blob/v2.16.0/src/common/const.go#L166-L167 | purpose only: "To avoid damaging the uploading artifact" (docs); value not derived |
| Harbor | GC: blob stuck in `deleting` is released | after the same 2 hours | https://github.com/goharbor/harbor/blob/v2.16.0/src/server/middleware/blob/head_blob.go#L65-L66 | none |
| Harbor | GC workers | default 1, maximum 10 | https://github.com/goharbor/harbor/blob/v2.16.0/src/jobservice/job/impl/gc/garbage_collection.go#L160-L161 and https://github.com/goharbor/harbor/blob/v2.16.0/src/server/v2.0/handler/gc.go#L292-L293 | none |
| Harbor | GC job concurrency and retry | 1 concurrent job, MaxFails 1, no retry | https://github.com/goharbor/harbor/blob/v2.16.0/src/jobservice/job/impl/gc/garbage_collection.go#L81-L94 | none |
| Harbor | GC manual trigger rate (UI) | once per minute | https://goharbor.io/docs/2.16.0/administration/garbage-collection/ | "To avoid triggering the garbage collection process too frequently" |
| Harbor | GC mark paging | projects 50 per page, blobs 1000 per page | https://github.com/goharbor/harbor/blob/v2.16.0/src/jobservice/job/impl/gc/garbage_collection.go#L637-L647 | none |
| Harbor | GC registry-cache (Redis) timeouts | dial 30 s, read 70 s, write 10 s | https://github.com/goharbor/harbor/blob/v2.16.0/src/jobservice/job/impl/gc/garbage_collection.go#L52-L55 | none |
| Harbor | max manifest PUT body | 4 MiB (`4 << 20`) | https://github.com/goharbor/harbor/blob/v2.16.0/src/common/const.go#L253-L255 | yes: "OCI manifests are sub-MiB in practice, so 4MiB is a generous ceiling." |
| Harbor | max audit-log payload | 4 MiB (`4 << 20`) | https://github.com/goharbor/harbor/blob/v2.16.0/src/common/const.go#L257-L258 | none |
| Harbor | Beego max memory / max upload size | 128 GB each (`1 << 37`) | https://github.com/goharbor/harbor/blob/v2.16.0/src/common/const.go#L246-L251 | none |
| Harbor | blob size cap at the front proxy | none (`client_max_body_size 0`) | https://github.com/goharbor/harbor/blob/v2.16.0/make/photon/prepare/templates/nginx/nginx.https.conf.jinja#L83 | none |
| Harbor | nginx `/v2/` proxy timeouts | send 900 s, read 900 s | https://github.com/goharbor/harbor/blob/v2.16.0/make/photon/prepare/templates/nginx/nginx.https.conf.jinja#L191-L192 | none |
| Harbor | nginx worker_connections | 3096 | https://github.com/goharbor/harbor/blob/v2.16.0/make/photon/prepare/templates/nginx/nginx.https.conf.jinja#L5 | none |
| Harbor | core to registry transport idle pool | MaxIdleConns 1024, MaxIdleConnsPerHost 1024 | https://github.com/goharbor/harbor/blob/v2.16.0/src/server/registry/proxy.go#L36-L44 | yes: default of 2 "closes almost every connection after use under high concurrency ... eventually exhausting ephemeral ports (EADDRNOTAVAIL)" |
| Harbor | upload purging (embedded registry) | enabled, age 168h, interval 24h, dryrun false | https://github.com/goharbor/harbor/blob/v2.16.0/make/harbor.yml.tmpl#L309-L316 | none ("default is one week") |
| Harbor | upload session accepted-size record (Redis) | 24 hours | https://github.com/goharbor/harbor/blob/v2.16.0/src/controller/blob/controller.go#L102 | none ("keep the size of blob in redis with 24 hours") |
| Harbor | DB pool, installer defaults | max_idle_conns 100, max_open_conns 900, conn_max_lifetime 5m, conn_max_idle_time 0 | https://github.com/goharbor/harbor/blob/v2.16.0/make/harbor.yml.tmpl#L53-L63 | partial: "the default number of connections is 1024 for postgres of harbor" |
| Harbor | DB pool, code defaults | max idle 2, max open 0 (unlimited), lifetime 5m, idle time 0 | https://github.com/goharbor/harbor/blob/v2.16.0/src/lib/config/metadata/metadatalist.go#L110-L113 | none |
| Harbor | registry to Redis pool | maxidle 100, maxactive 500, idletimeout 60s; read/write/dial timeout 10s | https://github.com/goharbor/harbor/blob/v2.16.0/make/photon/prepare/templates/registry/config.yml.jinja#L38-L48 | none |
| Harbor | filesystem storage driver maxthreads | 100 | https://github.com/goharbor/harbor/blob/v2.16.0/make/harbor.yml.tmpl#L78-L79 | none |
| Harbor | job workers per jobservice | 10 | https://github.com/goharbor/harbor/blob/v2.16.0/make/harbor.yml.tmpl#L141-L142 | partial (docs): "since each worker consumes a certain amount of network/CPU/IO resources, set the value of this attribute based on the hardware resource of the host" |
| Harbor | max job duration | 24 hours (task then marked error, keeps running) | https://github.com/goharbor/harbor/blob/v2.16.0/make/harbor.yml.tmpl#L143-L144 and https://github.com/goharbor/harbor/blob/v2.16.0/make/photon/prepare/templates/jobservice/config.yml.jinja#L68-L70 | none |
| Harbor | dangling execution reaper | 168 hours | https://github.com/goharbor/harbor/blob/v2.16.0/make/photon/prepare/templates/jobservice/config.yml.jinja#L71-L72 | none |
| Harbor | job log retrieval cap | 10 MB | https://github.com/goharbor/harbor/blob/v2.16.0/make/photon/prepare/templates/jobservice/config.yml.jinja#L74-L75 | none |
| Harbor | job log sweeper | 1 day | https://github.com/goharbor/harbor/blob/v2.16.0/make/harbor.yml.tmpl#L150-L151 | none |
| Harbor | webhook job | max retry 3, HTTP client timeout 3 s | https://github.com/goharbor/harbor/blob/v2.16.0/make/harbor.yml.tmpl#L153-L157 | none |
| Harbor | log rotation | 50 files of 200M | https://github.com/goharbor/harbor/blob/v2.16.0/make/harbor.yml.tmpl#L165-L170 | none |
| Harbor | cache layer | disabled; expiry 24 hours | https://github.com/goharbor/harbor/blob/v2.16.0/make/harbor.yml.tmpl#L327-L331 | none ("keep cache for one day by default") |
| Harbor | Trivy scan timeout | 5m0s, hard upper bound 30 minutes | https://github.com/goharbor/harbor/blob/v2.16.0/make/harbor.yml.tmpl#L124-L126 and https://github.com/goharbor/harbor/blob/v2.16.0/src/pkg/scan/job.go#L54 | none |
| Harbor | registry bearer token lifetime | 30 minutes | https://github.com/goharbor/harbor/blob/v2.16.0/src/lib/config/metadata/metadatalist.go#L122 | none |
| Harbor | robot token lifetime | 30 days (may be set to never expire) | https://github.com/goharbor/harbor/blob/v2.16.0/src/lib/config/metadata/metadatalist.go#L153-L154 | none |
| Harbor | UI session timeout | 60 minutes | https://github.com/goharbor/harbor/blob/v2.16.0/src/lib/config/metadata/metadatalist.go#L197 | none |
| Harbor | JWT validation leeway | 60 s | https://github.com/goharbor/harbor/blob/v2.16.0/src/common/const.go#L260-L261 | none |
| Harbor | LDAP connection timeout | 5 s | https://github.com/goharbor/harbor/blob/v2.16.0/src/lib/config/metadata/metadatalist.go#L94 | none |
| Harbor | password hashing work factor | PBKDF2-HMAC-SHA256, 600000 iterations (legacy 4096) | https://github.com/goharbor/harbor/blob/v2.16.0/src/common/utils/encrypt.go#L45-L54 | yes: "the OWASP-recommended minimum iteration count" |
| Harbor | registry HTTP client timeout | 30 minutes (env `REGISTRY_HTTP_CLIENT_TIMEOUT`) | https://github.com/goharbor/harbor/blob/v2.16.0/src/lib/config/systemconfig.go#L33-L38 | none |
| Harbor | execution status refresh interval | 30 s | https://github.com/goharbor/harbor/blob/v2.16.0/src/lib/config/metadata/metadatalist.go#L199 | none |
| Harbor | audit-log retention maximum | 240000 hours | https://github.com/goharbor/harbor/blob/v2.16.0/src/common/const.go#L220-L221 | none |
| Harbor | default project storage quota | -1 (unlimited) | https://github.com/goharbor/harbor/blob/v2.16.0/src/lib/config/metadata/metadatalist.go#L164 | none |
| Harbor | REST API page size | default 10, maximum 100 | https://github.com/goharbor/harbor/blob/v2.16.0/api/v2.0/swagger.yaml#L6597-L6605 | none |
| Harbor | legacy API page size | default 500, maximum 500 | https://github.com/goharbor/harbor/blob/v2.16.0/src/common/api/base.go#L35-L36 | none |
| Harbor | `/v2/_catalog` without `n` | unbounded: every non-empty repository is returned | https://github.com/goharbor/harbor/blob/v2.16.0/src/server/registry/catalog.go#L56-L74 | none |
| Harbor | tag retention rules per project | 15 | https://github.com/goharbor/harbor/blob/v2.16.0/src/server/v2.0/handler/retention.go#L163-L164 | none |
| Harbor | tag immutability rules per project | 15 | https://goharbor.io/docs/2.16.0/working-with-projects/working-with-images/create-tag-immutability-rules/ | none |
| Harbor | proxy cache: wait for blobs before caching a manifest | 10 checks, 20 s apart | https://github.com/goharbor/harbor/blob/v2.16.0/src/controller/proxy/controller.go#L41-L45 | none |
| Harbor | proxy cache: wait for children of a manifest list | 20 checks, 20 s apart (comment says 30 mins) | https://github.com/goharbor/harbor/blob/v2.16.0/src/controller/proxy/controller.go#L41-L45 | partial: "manifest list depends on manifest ready" |
| Harbor | proxy cache: manifest-list body kept in Redis | 7 days | https://github.com/goharbor/harbor/blob/v2.16.0/src/controller/proxy/controller.go#L46-L47 | none ("keep manifest list in cache for one week") |
| Harbor | proxy cache: tag-ensure retry | 60 tries, 10 s apart | https://github.com/goharbor/harbor/blob/v2.16.0/src/server/middleware/repoproxy/proxy.go#L65-L66 | none |
| Harbor | proxy cache: referrer cache TTL | 7 days | https://github.com/goharbor/harbor/blob/v2.16.0/src/server/middleware/repoproxy/proxy.go#L68 | none |
| Harbor | proxy cache: upstream-connection counter expiry | 3600 s | https://github.com/goharbor/harbor/blob/v2.16.0/src/pkg/proxy/connection/limit.go#L43 | yes (terse): "set expire to avoid always lock" |
| Harbor | proxy cache: default retention policy | 7 days | https://goharbor.io/docs/2.16.0/administration/configure-proxy-cache/ | none |
| Harbor | proxy cache: max_upstream_conn, proxy_speed_kb | -1 (no limit) unless set per project | https://github.com/goharbor/harbor/blob/v2.16.0/api/v2.0/swagger.yaml#L7729-L7736 | n/a |
| Harbor | replication chunk size | 10 MiB (`10 * 1024 * 1024`), env `REPLICATION_CHUNK_SIZE` | https://github.com/goharbor/harbor/blob/v2.16.0/src/controller/replication/transfer/image/transfer.go#L44 | none |
| Harbor | replication copy retries | 5 per blob, 5 per chunk, back-off from 2 s doubling | https://github.com/goharbor/harbor/blob/v2.16.0/src/controller/replication/transfer/image/transfer.go#L48-L61 | none |
| Harbor | metrics exporter | max requests 30, cache time 23 s, cache clean interval 14400 s | https://github.com/goharbor/harbor/blob/v2.16.0/make/photon/prepare/templates/exporter/env.jinja#L5-L7 | none |
| Harbor | minimum / recommended resources | 2 CPU, 4 GB, 40 GB / 4 CPU, 8 GB, 160 GB | https://goharbor.io/docs/2.16.0/install-config/installation-prereqs/ | none |
| Harbor | request rate limits (registry and API) | none built in | see F089 | n/a |
| Zot | GC delay (grace period for unreferenced blobs) `DefaultGCDelay` | 1 h | https://github.com/project-zot/zot/blob/v2.1.23/pkg/storage/constants/constants.go#L24 | partly: docs say the knob depends on "client network speeds and the size of blobs"; none for 1 h |
| Zot | GC interval `DefaultGCInterval` | 1 h | https://github.com/project-zot/zot/blob/v2.1.23/pkg/storage/constants/constants.go#L25 | none |
| Zot | GC per-repository task start jitter | random, up to 30 s | https://github.com/project-zot/zot/blob/v2.1.23/pkg/storage/gc/gc.go#L97-L100 | none ("default value") |
| Zot | GC time window `gcTimeWindow` | unset (any time) | https://github.com/project-zot/zot/blob/v2.1.23/pkg/api/config/config.go#L48-L50 | yes: lock contention on high-load registries (examples/README.md L166-L175) |
| Zot | retention delay for untagged manifests and orphan referrers | equals gcDelay (24 h before v2.1.9) | https://github.com/project-zot/zot/blob/v2.1.23/pkg/cli/server/root.go#L1186-L1190 | none |
| Zot | upload session expiry | none of its own; removed by GC when mtime is older than gcDelay | https://github.com/project-zot/zot/blob/v2.1.23/pkg/storage/gc/gc.go#L1241-L1244 | yes: "Do not delete blob uploads which have been updated recently" |
| Zot | minimum upload chunk size | none advertised (no OCI-Chunk-Min-Length in tree); S3 multipart parts at least 5 MB via the Distribution driver | https://zotregistry.dev/v2.1.23/articles/storage/ | yes for S3: "The S3 API requires multipart upload chunks to be at least 5MB" |
| Zot | max manifest PUT body `MaxManifestBodySize` | 4 MiB (`4 * 1024 * 1024`) | https://github.com/project-zot/zot/blob/v2.1.23/pkg/api/constants/consts.go#L23-L25 | yes: "OCI manifest JSON is always small metadata; 4 MiB is well above any realistic manifest." |
| Zot | max tag= query parameters on a digest push `MaxManifestDigestQueryTags` | 46 = (8192 - 2048) / (4 + 128 + 1) | https://github.com/project-zot/zot/blob/v2.1.23/pkg/api/constants/consts.go#L15-L22 | yes: derivation from an about 8 KiB request-target budget in the comment |
| Zot | max API-key creation body `MaxAPIKeyBodySize` | 8 KiB | https://github.com/project-zot/zot/blob/v2.1.23/pkg/api/constants/consts.go#L26-L27 | none (release notes for v2.1.16 say 4 KiB) |
| Zot | max image-trust key or certificate upload `MaxImageTrustBodySize` | 8 MiB | https://github.com/project-zot/zot/blob/v2.1.23/pkg/api/constants/consts.go#L28-L29 | none |
| Zot | max token request form body `MaxTokenRequestBodySize` | 64 KiB | https://github.com/project-zot/zot/blob/v2.1.23/pkg/api/constants/consts.go#L30-L31 | none |
| Zot | max GraphQL request body `MaxSearchBodySize` | 1 MiB | https://github.com/project-zot/zot/blob/v2.1.23/pkg/api/constants/consts.go#L32-L37 | yes: largest own query is a few hundred bytes; bounds buffered memory per request |
| Zot | GraphQL parser token limit | 2000 | https://github.com/project-zot/zot/blob/v2.1.23/pkg/extensions/extension_search.go#L32-L37 | yes: richest own query is under 300 tokens |
| Zot | GraphQL complexity limit | 500 | https://github.com/project-zot/zot/blob/v2.1.23/pkg/extensions/extension_search.go#L38-L42 | yes: "The richest known real query costs on the order of 30; 500 leaves ample headroom." |
| Zot | GraphQL query cache entries | 64 | https://github.com/project-zot/zot/blob/v2.1.23/pkg/extensions/extension_search.go#L43-L49 | yes: keeps worst-case cache memory in the tens of MiB |
| Zot | CVE list page limit `maxCvePageLimit` | 4096 (`4 * 1024`) | https://github.com/project-zot/zot/blob/v2.1.23/pkg/extensions/search/cve/pagination.go#L100 | none |
| Zot | catalog and tag-list page size | no default and no maximum (-1 when n is absent) | https://github.com/project-zot/zot/blob/v2.1.23/pkg/api/routes.go#L335 | none |
| Zot | tag length | 128 characters | https://github.com/project-zot/zot/blob/v2.1.23/pkg/regexp/regexp.go#L9-L11 | yes: the OCI distribution-spec limit |
| Zot | HTTP read timeout (whole request including body) | 60 s | https://github.com/project-zot/zot/blob/v2.1.23/pkg/cli/server/root.go#L41 | partly: docs give the purpose (slow or stalled clients), none for 60 s |
| Zot | HTTP write timeout (whole response) | 60 s | https://github.com/project-zot/zot/blob/v2.1.23/pkg/cli/server/root.go#L42 | partly: as above |
| Zot | HTTP shutdown drain timeout | 30 s | https://github.com/project-zot/zot/blob/v2.1.23/pkg/cli/server/root.go#L43 | none |
| Zot | HTTP idle timeout (fixed) | 120 s | https://github.com/project-zot/zot/blob/v2.1.23/pkg/api/controller.go#L39 | none |
| Zot | HTTP read-header timeout (fixed) | 5 s | https://github.com/project-zot/zot/blob/v2.1.23/pkg/api/controller.go#L40 | none |
| Zot | blob response copy chunk | 10 MiB | https://github.com/project-zot/zot/blob/v2.1.23/pkg/api/routes.go#L3206 | none |
| Zot | request rate limit | off by default; opt-in requests per second, optional per-method | https://github.com/project-zot/zot/blob/v2.1.23/pkg/api/config/config.go#L434-L442 | n/a |
| Zot | failed-authentication delay `failDelay` | 0 s by default | https://github.com/project-zot/zot/blob/v2.1.23/pkg/api/config/config.go#L893 | yes: "to prevent automated attacks" (examples/README.md L938) |
| Zot | browser session cookie max age | 7200 s (2 h) | https://github.com/project-zot/zot/blob/v2.1.23/pkg/api/cookiestore.go#L26 | none |
| Zot | API key expiry | none by default (optional expirationDate) | https://github.com/project-zot/zot/blob/v2.1.23/examples/README.md#L797-L812 | none |
| Zot | LDAP connection retries | 8, with linear backoff of 1 s steps | https://github.com/project-zot/zot/blob/v2.1.23/pkg/api/ldap.go#L107-L118 | none |
| Zot | OIDC provider metadata refresh / HTTP timeout | 1 min / 10 s | https://github.com/project-zot/zot/blob/v2.1.23/pkg/api/bearer_oidc.go#L31-L34 | none |
| Zot | scheduler workers | NumCPU x 4 | https://github.com/project-zot/zot/blob/v2.1.23/pkg/scheduler/scheduler.go#L549-L556 | none |
| Zot | scheduler queue length per priority | 400 tasks | https://github.com/project-zot/zot/blob/v2.1.23/pkg/scheduler/scheduler.go#L63 | none |
| Zot | scheduler dispatch tick | 50 ms | https://github.com/project-zot/zot/blob/v2.1.23/pkg/scheduler/scheduler.go#L64 | none |
| Zot | CVE background scan interval | 15 min, hardcoded | https://github.com/project-zot/zot/blob/v2.1.23/pkg/extensions/extension_search.go#L29 | none |
| Zot | CVE database update interval minimum | 2 h (shorter values are raised) | https://github.com/project-zot/zot/blob/v2.1.23/pkg/cli/server/root.go#L1084-L1092 | none |
| Zot | scrub interval default / minimum | 24 h / 2 h | https://github.com/project-zot/zot/blob/v2.1.23/pkg/cli/server/root.go#L1146-L1157 | none |
| Zot | sync overall timeout `DefaultSyncTimeout` | 3 h | https://github.com/project-zot/zot/blob/v2.1.23/pkg/extensions/sync/constants/constants.go#L20 | partly: docs say it keeps a sync alive after the client disconnects; none for 3 h |
| Zot | sync response-header timeout | 30 s | https://github.com/project-zot/zot/blob/v2.1.23/pkg/extensions/sync/constants/constants.go#L21 | none |
| Zot | sync concurrent requests per upstream host `reqConcurrent` | 3 (regclient default) | https://github.com/project-zot/zot/blob/v2.1.23/pkg/extensions/config/sync/config.go#L82-L93 | yes for the knob: the default "causes head-of-line blocking" under many concurrent pulls |
| Zot | sync request rate per upstream host `reqPerSec` | unlimited | https://github.com/project-zot/zot/blob/v2.1.23/pkg/extensions/config/sync/config.go#L94-L97 | none |
| Zot | sync retries | none unless maxRetries and retryDelay are set | https://github.com/project-zot/zot/blob/v2.1.23/examples/README.md#L1502-L1504 | none |
| Zot | on-demand tag revalidation `manifestCheckInterval` | 0 (check upstream on every tag request) | https://github.com/project-zot/zot/blob/v2.1.23/examples/README.md#L1583-L1589 | yes: "a tag is mutable and may have been repushed" |
| Zot | streaming pull-through concurrent blobs `DefaultMaxConcurrentStreams` | 32 | https://github.com/project-zot/zot/blob/v2.1.23/pkg/extensions/sync/constants/constants.go#L24-L26 | none |
| Zot | event HTTP sink timeout | 30 s | https://github.com/project-zot/zot/blob/v2.1.23/pkg/extensions/events/common.go#L10 | none |
| Zot | repository quotas `maxRepos`, `maxRepoBytes` | 0 (off) | https://github.com/project-zot/zot/blob/v2.1.23/pkg/api/config/config.go#L32-L33 | n/a |
| Zot | storage path limits | 255 per component; 4095 local path; 1024 object key | https://github.com/project-zot/zot/blob/v2.1.23/pkg/storage/imagestore/imagestore.go#L44-L46 | none |
| Zot | upload session id length | 255 | https://github.com/project-zot/zot/blob/v2.1.23/pkg/storage/imagestore/upload_session.go#L12 | none |
| Zot | dedupe cache lock check timeout | 10 s | https://github.com/project-zot/zot/blob/v2.1.23/pkg/storage/constants/constants.go#L19 | none |
| Zot | cluster proxy hops | 1 (a request that already carries the hop header is refused with 508) | https://github.com/project-zot/zot/blob/v2.1.23/pkg/api/proxy.go#L61-L68 | yes: "there should be no multi-hop" |
| Zot | default log level | debug | https://github.com/project-zot/zot/blob/v2.1.23/pkg/api/config/config.go#L895 | none |
| Zot | default listen address | 127.0.0.1:8080 | https://github.com/project-zot/zot/blob/v2.1.23/pkg/api/config/config.go#L891-L892 | none |
| Zot | config, htpasswd and TLS file reload debounce / stat check | 150 ms / 1 s | https://github.com/project-zot/zot/blob/v2.1.23/pkg/cli/server/config_reloader.go#L23-L27 | yes: debounce "coalesces bursts of fsnotify events into one reload"; the stat poll is a backstop |
| Zot | CI gate on binary growth | fail at 1% or more | https://github.com/project-zot/zot/blob/v2.1.23/.github/workflows/compare-binary-size.yaml#L25-L44 | none |
| Zot | zb benchmark tool constants | test blobs 1 MiB, 10 MiB, 100 MiB, 1 GiB; client HTTP timeout 30 s; default concurrency 1 and requests 1 (cmd/zb/README.md) | https://github.com/project-zot/zot/blob/v2.1.23/cmd/zb/perf.go#L31-L52 | none |
| Zot | example systemd limits | LimitNOFILE=500000, MemoryHigh=30G, MemoryMax=32G | https://zotregistry.dev/v2.1.23/install-guides/install-guide-linux/ | none |
| Quay | MAXIMUM_LAYER_SIZE (max blob) | "20G" | https://github.com/quay/quay/blob/v3.18.3/config.py#L677-L678 | none ("Maximum size allowed for layers in the registry.") |
| Quay | nginx client_max_body_size, blob and /v1 locations | `{{ maximum_layer_size }}` (20G) | https://github.com/quay/quay/blob/v3.18.3/conf/nginx/server-base.conf.jnj#L308 | none |
| Quay | manifest body limit | not set; nginx default 1m applies to the manifests location (inference) | https://github.com/quay/quay/blob/v3.18.3/conf/nginx/server-base.conf.jnj#L357-L384 | none |
| Quay | nginx client_max_body_size, /push and build-manager server | 5M | https://github.com/quay/quay/blob/v3.18.3/conf/nginx/server-base.conf.jnj#L75-L78 | none |
| Quay | PUSH_TEMP_TAG_EXPIRATION_SEC (push protection window) | 6 h (1 h before v3.17.5 / v3.18.1) | https://github.com/quay/quay/blob/v3.18.3/config.py#L500-L503 | yes: "to protect big multiarch images from not being GCed while pushes are still in progress" |
| Quay | DEFAULT_TAG_EXPIRATION (time machine) | "2w"; options 0s, 1d, 1w, 2w, 4w | https://github.com/quay/quay/blob/v3.18.3/config.py#L688-L693 | none |
| Quay | GARBAGE_COLLECTION_FREQUENCY | 30 s | https://github.com/quay/quay/blob/v3.18.3/config.py#L536-L537 | none |
| Quay | GC_CANDIDATE_COUNT (repos sampled per GC tick) | 500 | https://github.com/quay/quay/blob/v3.18.3/data/model/oci/tag.py#L45 | none |
| Quay | GC policy list cap and cache | 100 distinct policies, cached 600 s | https://github.com/quay/quay/blob/v3.18.3/data/model/repository.py#L207-L215 | yes: "This sucks but it's the only way to limit memory" |
| Quay | REPOSITORY_GC_TIMEOUT / NAMESPACE_GC_TIMEOUT (lock TTL and queue reservation) | 3 h + 60 s padding | https://github.com/quay/quay/blob/v3.18.3/workers/gc/gcworker.py#L20-L21 | none |
| Quay | repository/namespace GC queue poll | 60 s | https://github.com/quay/quay/blob/v3.18.3/workers/repositorygcworker.py#L15-L17 | none |
| Quay | GC delete batch sizes | 10 rows per iteration; 500 ids per chunk delete | https://github.com/quay/quay/blob/v3.18.3/data/model/gc.py#L146-L168 | none |
| Quay | BLOB_DELETE lock TTL (GC side) | 120 s | https://github.com/quay/quay/blob/v3.18.3/data/model/storage.py#L257 | none |
| Quay | BLOB_DELETE lock TTL (push side) | 30 s | https://github.com/quay/quay/blob/v3.18.3/data/model/storage.py#L366-L370 | none |
| Quay | BLOB_DELETE_LOCK_ACQUIRE_TIMEOUT (push waits for GC) | 3 s, then HTTP 503 | https://github.com/quay/quay/blob/v3.18.3/data/model/storage.py#L38-L39 | partly: "Total seconds a blob writer waits for a contended BLOB_DELETE lock before giving up." |
| Quay | GlobalLock default TTL | 600 s | https://github.com/quay/quay/blob/v3.18.3/util/locking.py#L77-L79 | none |
| Quay | BLOBUPLOAD_DELETION_DATE_THRESHOLD (upload session purge age) | 2 days (60 * 48 minutes) | https://github.com/quay/quay/blob/v3.18.3/workers/blobuploadcleanupworker/blobuploadcleanupworker.py#L16-L17 | none (a comment in the same file says "typically a week") |
| Quay | BLOBUPLOAD_CLEANUP_FREQUENCY | 1 h; lock TTL 20 min | https://github.com/quay/quay/blob/v3.18.3/workers/blobuploadcleanupworker/blobuploadcleanupworker.py#L18-L19 | none |
| Quay | MPU_CLEANUP_TTL (stale S3 multipart uploads) | 1 day, checked every 6 h, not configurable | https://github.com/quay/quay/blob/v3.18.3/workers/blobuploadcleanupworker/blobuploadcleanupworker.py#L21-L27 | yes: "if the threshold is set too low, then MPUs still in progress might be deleted causing push failures" |
| Quay | storage minimum chunk size (multipart part) | 5 MiB (`minimum_chunk_size_mb` overrides) | https://github.com/quay/quay/blob/v3.18.3/storage/cloud.py#L136 | yes: "Minimum size of upload part size on S3 is 5MB" (L310) |
| Quay | S3 maximum chunk size | 5 GB (`maximum_chunk_size_gb`) | https://github.com/quay/quay/blob/v3.18.3/storage/cloud.py#L874-L877 | none ("# 5gb default") |
| Quay | RadosGW / IBM Cloud maximum chunk size | 100 MiB | https://github.com/quay/quay/blob/v3.18.3/storage/cloud.py#L1102-L1115 | partly: "100 MiB, as with IBM Storage driver"; the adjacent comment explains the 10000-part / 50 GiB ceiling |
| Quay | storage IO buffer | 64 kB | https://github.com/quay/quay/blob/v3.18.3/storage/basestorage.py#L42-L43 | none |
| Quay | direct download URL validity | 60 s | https://github.com/quay/quay/blob/v3.18.3/storage/cloud.py#L250-L252 | none |
| Quay | V2_PAGINATION_SIZE and effective page maximum | 50 configured, effective 100 (`max(..., 100)`) | https://github.com/quay/quay/blob/v3.18.3/endpoints/v2/__init__.py#L91-L93 | partly: "minimally server 100 tags per page" |
| Quay | REST tag list page | default 50, max 100 | https://github.com/quay/quay/blob/v3.18.3/endpoints/api/tag.py#L172-L181 | none |
| Quay | search paging | 10 per page, 10 pages | https://github.com/quay/quay/blob/v3.18.3/config.py#L772-L776 | none |
| Quay | data model cache TTLs | repository blob 60s, catalog page 60s, active repo tags 120s, value limit 1MiB | https://github.com/quay/quay/blob/v3.18.3/config.py#L742-L750 | none |
| Quay | in-container memcached size | 64 MB | https://github.com/quay/quay/blob/v3.18.3/conf/supervisord.conf.jnj#L282 | none |
| Quay | registry JWT lifetime | 1 h (`TOKEN_VALIDITY_LIFETIME_S`) | https://github.com/quay/quay/blob/v3.18.3/endpoints/v2/v2auth.py#L59 | none |
| Quay | REGISTRY_JWT_AUTH_MAX_FRESH_S | 3660 s | https://github.com/quay/quay/blob/v3.18.3/config.py#L508-L511 | yes: "At most signed one hour, accounting for clock skew" |
| Quay | SIGNED_GRANT_EXPIRATION_SEC | 1 day | https://github.com/quay/quay/blob/v3.18.3/config.py#L505-L506 | partly: "One day to complete a push/pull" |
| Quay | instance service key expiration / refresh | 120 min / 55 min | https://github.com/quay/quay/blob/v3.18.3/config.py#L638-L643 | partly: "Should be the expiration / 2 minus some additional window time." |
| Quay | SESSION_TIMEOUT / FRESH_LOGIN_TIMEOUT | "31d" (L197) / "10m" (L827) | https://github.com/quay/quay/blob/v3.18.3/config.py#L197 | none |
| Quay | rate-limit zones (when FEATURE_RATE_LIMITS) | very light 60 / 600 r/s, light 50 / 500 r/s, heavy 5 / 50 r/s (HTTP/1 per address / HTTP/2 per connection); auth 120 r/s; status 429 | https://github.com/quay/quay/blob/v3.18.3/conf/nginx/rate-limiting.conf.jnj#L64-L90 | partly: "the limit is very low on purpose but should allow for the burst of traffic required for a registry operation" |
| Quay | rate-limit bursts per endpoint | blobs 50/100, manifests 10/50, tags 2/2, _catalog 1/5, other /v2 20/80, /api 25/100 | https://github.com/quay/quay/blob/v3.18.3/conf/nginx/server-base.conf.jnj#L220-L439 | partly: tags restricted "due to how heavy an operation it can be"; blobs "will receive a high volume of traffic, so we set the burst much higher" |
| Quay | FEATURE_RATE_LIMITS default | False | https://github.com/quay/quay/blob/v3.18.3/config.py#L784-L785 | none |
| Quay | nginx proxy timeouts | blobs read/send 2000 s; _catalog read 10 s; storage proxy read 60 s | https://github.com/quay/quay/blob/v3.18.3/conf/nginx/server-base.conf.jnj#L304-L305 | none |
| Quay | nginx worker_connections / keepalive_timeout | 10240 / 5 s (0 on registry locations) | https://github.com/quay/quay/blob/v3.18.3/conf/nginx/root-base.conf#L13 | partly: "Disables HTTP 1.1 keep-alive and forces round-robin." |
| Quay | gunicorn registry workers | CPU x 4, min 8, max 64 (web: x 2, 2-32; secscan: x 2, 2-4) | https://github.com/quay/quay/blob/v3.18.3/conf/gunicorn_registry.py#L18 | none |
| Quay | gunicorn worker timeout | gunicorn default 30 s at v3.18.3; 5.0 draft: registry 300 s, web 60 s, range 30-300 | https://github.com/quay/quay-docs/blob/1350e7d1257ae9a124ceeef3a1f44720a2396c9e/modules/config-fields-rate-limit-performance.adoc | yes (5.0 docs): longer timeouts hold CPU and memory per in-flight request |
| Quay | outbound HTTP client pool | 100 connections, pool size 100 | https://github.com/quay/quay/blob/v3.18.3/config.py#L10-L15 | none ("a large request pool") |
| Quay | DB connect timeout | 10 s | https://github.com/quay/quay/blob/v3.18.3/data/database.py#L66 | none |
| Quay | proxy cache expiration (staleness) | 86400 s | https://github.com/quay/quay/blob/v3.18.3/data/database.py#L68 | none |
| Quay | proxy cache upstream timeouts | (5 s connect, 30 s read); blobs (5 s, 120 s) | https://github.com/quay/quay/blob/v3.18.3/proxy/__init__.py#L173-L175 | none |
| Quay | proxy cache upstream token lifetime assumption | 1 h, renewal threshold 10 s | https://github.com/quay/quay/blob/v3.18.3/proxy/__init__.py#L20-L21 | partly: "1 hour, in seconds - Quay's default" |
| Quay | proxy cache blob download worker | poll 10 s, reservation 20 min | https://github.com/quay/quay/blob/v3.18.3/workers/proxycacheblobworker.py#L35-L36 | none |
| Quay | REPO_MIRROR_INTERVAL / ORG_MIRROR_INTERVAL | 30 s | https://github.com/quay/quay/blob/v3.18.3/config.py#L584-L593 | none |
| Quay | mirror sync interval (per repository) | 24 h default | https://github.com/quay/quay-docs/blob/a4420c1359090a2735003589fe74240db605737e/modules/mirroring-creating-repository.adoc | none |
| Quay | SKOPEO_TIMEOUT_SECONDS | 300 s default, max 43200 s | https://github.com/quay/quay/blob/v3.18.3/util/repomirror/skopeomirror.py#L22 | none (was hard-coded until 3.15, PROJQUAY-7249) |
| Quay | MAX_SYNC_RETRIES / MAX_SYNC_DURATION | 3 / 12 h | https://github.com/quay/quay/blob/v3.18.3/data/model/repo_mirror.py#L24-L33 | yes: "for certain very big images, 2 hours might not be enough" |
| Quay | REPO_MIRROR_MAX_MANIFEST_LIST_SIZE / MAX_MANIFEST_ENTRIES | 10 MB / 1000 | https://github.com/quay/quay/blob/v3.18.3/config.py#L605-L611 | yes: "Prevents DoS via oversized manifests from malicious registries." |
| Quay | ORG_MIRROR_MAX_DISCOVERY_DURATION | 1800 s | https://github.com/quay/quay/blob/v3.18.3/config.py#L587-L590 | partly: "Increase for large source registries with many repositories." |
| Quay | storage replication worker | poll 10 s, reservation 20 min, backoff x4 up to 30 s, 3 retries | https://github.com/quay/quay/blob/v3.18.3/workers/storagereplication.py#L22-L23 | none |
| Quay | work queue defaults | 5 retries per item; worker poll 30 s, reservation 300 s, retry after 300 s | https://github.com/quay/quay/blob/v3.18.3/workers/queueworker.py#L51-L58 | none |
| Quay | SECURITY_SCANNER_INDEXING_INTERVAL / V4_REINDEX_THRESHOLD / MAX_SCAN_RETRIES | 30 s / 300 s / 5 | https://github.com/quay/quay/blob/v3.18.3/config.py#L552-L564 | none |
| Quay | QUOTA_TOTAL_DELAY_SECONDS | 30 min | https://github.com/quay/quay/blob/v3.18.3/config.py#L932-L935 | yes: "Rolling deployments can cause incorrect totals, so this field should be set to a time longer than it takes for the rolling deployment to complete." |
| Quay | ACTION_LOG_ROTATION_THRESHOLD | "30d" | https://github.com/quay/quay/blob/v3.18.3/config.py#L492 | none |
| Quay | blob response Cache-Control max-age | HEAD 31436000 s, GET 31536000 s | https://github.com/quay/quay/blob/v3.18.3/endpoints/v2/blob.py#L66 | none (the two values differ by 100000 s, apparently a typo) |
| Quay (Go mirror registry) | GC interval | 30 s | https://github.com/quay/quay/blob/mirror-registry-v3.0.0/internal/gc/gc.go#L31-L41 | none |
| Quay (Go mirror registry) | stale upload threshold | 48 h | https://github.com/quay/quay/blob/mirror-registry-v3.0.0/internal/gc/sqlite_collector.go#L14 | none |
| Quay (Go mirror registry) | manifest GC cascade iterations | max 100 per cycle, warning when hit | https://github.com/quay/quay/blob/mirror-registry-v3.0.0/internal/gc/sqlite_collector.go#L100-L129 | partly: "deleting a manifest list orphans its children, which need another pass to detect" |
| GitLab registry | max manifest request body | 4 MiB (`4 << 20`) | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/handlers/manifests.go#L43 | none |
| GitLab registry | `validation.manifests.payloadsizelimit` | 0 = disabled by default; 256KB on GitLab.com | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/docs/configuration.md#payloadsizelimit ; https://gitlab.com/gitlab-com/runbooks/-/blob/6ec746844dbd1375929ad305dcd118902c517211/docs/registry/manifest-invalid-request-body-too-large.md | none for 256KB |
| GitLab registry | `validation.manifests.referencelimit` | 0 = disabled by default (example 150) | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/docs/configuration.md#referencelimit | none |
| GitLab registry | config payload cached in database up to (`ConfigSizeLimit`) | 256000 bytes | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/datastore/datastore.go#L4 | "safety limit", no measurement |
| GitLab registry | `gc.reviewafter` / `review_after` default | 24h (1 day) | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/docs/configuration.md#gc ; https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/datastore/migrations/structure.sql#L46 | yes, qualitative: protect in-flight pushes, allow re-tagging (F247) |
| GitLab registry | review-time jitter (`gc_review_after`) | 5 to 61 s (docs: up to 60 s) | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/datastore/migrations/structure.sql#L38 | "To avoid scheduling multiple tasks to the exact same time" |
| GitLab registry | GC worker interval (`gc.blobs.interval`, `gc.manifests.interval`) | 5s | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/gc/agent.go#L30 | none; docs suggest 1s or 500ms for backlogs |
| GitLab registry | `gc.maxbackoff` (idle and error) | 24h, plus up to 33% jitter | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/gc/agent.go#L25-L31 | none |
| GitLab registry | GC agent start jitter | up to 60 s | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/gc/agent.go#L26 | "to ease concurrency in clustered environments" |
| GitLab registry | `gc.transactiontimeout` | 10s | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/gc/worker/worker.go#L27 | "to avoid stalled or long-running transactions" |
| GitLab registry | `gc.blobs.storagetimeout` | 5s (specification text says 2 seconds) | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/gc/worker/blobworker.go#L18 | bound the time a row lock is held during a remote delete |
| GitLab registry | failed GC task postponement | 5 minutes doubling to 24 hours | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/gc/worker/worker.go#L149-L151 | none |
| GitLab registry | `gc.errorcooldownperiod` | 0 (example 30m) | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/docs/configuration.md#gc | slow workers across deployments during intermittent errors |
| GitLab registry | API-side GC lock look-ahead window | 1 hour | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/handlers/manifests.go#L824 | lock only records "reviewed (and possibly deleted) soon" |
| GitLab registry | API-side GC lock timeout (then 503) | 5 s | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/handlers/manifests.go#L825 | "we should not let transactions open (and clients waiting) for too long" |
| GitLab registry | upload purge age / interval / first-run jitter | 168h / 24h / 0-59 min | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/handlers/app.go#L2078-L2085 ; https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/handlers/app.go#L2146 | none |
| GitLab registry | tag list page size (`/gitlab/v1/.../tags/list/`) | default 100, max 1000 | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/handlers/repositories.go#L82-L83 | none |
| GitLab registry | catalog page size (`/v2/_catalog`) | default 100, max 1000 | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/handlers/catalog.go#L26-L27 | none |
| GitLab registry | tag name filter length | 128 characters | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/docs/spec/gitlab/api.md#list-repository-tags | follows the OCI tag pattern |
| GitLab registry | rename: max sub-repositories | 3000 (1000, then 2000 in v4.39.0, 3000 in v4.43.0) | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/handlers/repositories.go#L95 | none |
| GitLab registry | rename lease TTL / dry-run timeout | 60 s / 5 s | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/datastore/repositorylease.go#L24 ; https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/handlers/repositories.go#L94 | none |
| GitLab registry | tag protection / immutability patterns in token | max 5 patterns, max 100 characters each | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/handlers/manifests.go#L1769-L1773 | none |
| GitLab registry | JWT clock-skew leeway | 60 s | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/auth/token/token.go#L28 | cites RFC 7519 section 4.1.5 |
| GitLab (Rails) | registry token lifetime | 5 minutes default; 15 on GitLab.com | https://gitlab.com/gitlab-org/gitlab/-/blob/v19.6.0-ee/doc/administration/packages/container_registry.md#increase-token-duration ; https://gitlab.com/gitlab-org/gitlab/-/blob/v19.6.0-ee/doc/user/gitlab_com/_index.md#container-registry | none |
| GitLab (Rails) | tags deleted per bulk API call | 20 | https://gitlab.com/gitlab-org/gitlab/-/blob/v19.6.0-ee/doc/administration/instance_limits.md#container-repository-tag-deletion-limit | "each tag deletion triggers network requests to the container registry" |
| GitLab (Rails) | repository deletion retries | 10, with backoff | https://gitlab.com/gitlab-org/gitlab/-/blob/v19.6.0-ee/doc/administration/packages/container_registry.md#max-retries-for-deleting-container-images | detect persistent errors |
| GitLab registry | S3 minimum chunk | 5 MiB (`5 << 20`) | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/storage/driver/s3-aws/common/common.go#L18 | S3 API minimum part size |
| GitLab registry | S3 default chunk | 10 MiB (`2 * MinChunkSize`); docs advise 25-50 MB when large pushes fail | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/storage/driver/s3-aws/common/common.go#L24 | none |
| GitLab registry | S3 maximum chunk | 5 GiB (`5 << 30`) | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/storage/driver/s3-aws/common/common.go#L22 | S3 maximum part size |
| GitLab registry | S3 multipart copy chunk and threshold | 32 MiB each | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/storage/driver/s3-aws/common/common.go#L26-L38 | "Empirically, 32 MB is optimal." (no data) |
| GitLab registry | S3 multipart copy concurrency | 100 | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/storage/driver/s3-aws/common/common.go#L33 | none |
| GitLab registry | S3 `maxrequestspersecond` | 350 (max 3500; 0 disables) | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/storage/driver/s3-aws/common/common.go#L40-L45 | "350 is 10% of the requestsPerSecondUpperLimit" with a link to AWS documentation |
| GitLab registry | S3 retries | 5 | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/storage/driver/s3-aws/common/common.go#L49 | none |
| GitLab registry | S3 keys per delete / per list | 1000 / 1000 | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/storage/driver/s3-aws/common/common.go#L53 ; https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/storage/driver/s3-aws/v2/s3.go#L62 | S3 API limits |
| GitLab registry | S3 max part-list pages (`maxListRespLoop`) | 10000 | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/storage/driver/s3-aws/v2/s3.go#L56-L59 | "to prevent infinite loops" |
| GitLab registry | GCS chunk (min / default) | 256 KiB / 5 MiB (`20 * minChunkSize`) | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/storage/driver/gcs/common.go#L46-L47 | none |
| GitLab registry | GCS concurrency (default / min / delete / walk) and tries | 50 / 25 / 150 / 100; 5 tries | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/storage/driver/gcs/common.go#L48-L52 | none |
| GitLab registry | filesystem `maxthreads` | default 100, minimum 25 | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/storage/driver/filesystem/driver.go#L24-L29 | none |
| GitLab registry | pre-signed URL expiry (`storage.redirect.expirydelay`) | 20 minutes | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/storage/driver/storagedriver.go#L233 | none |
| GitLab registry | URL cache `min_url_validity` / `default_url_validity` | 10m / 20m | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/docs/configuration.md#urlcache | none |
| GitLab registry | Redis cache TTL (all keys) | 6 hours, not configurable | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/handlers/app.go#L91 | none ("We intend to fine-tune this value") |
| GitLab registry | Redis cache operation timeout | 500 ms | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/datastore/repository.go#L33 | none |
| GitLab registry | size-with-descendants cache TTL / timed-out marker TTL | 5 minutes / 24 hours | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/datastore/repository.go#L46 ; https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/datastore/repository.go#L449 | "to avoid consecutive failures" |
| GitLab registry | Redis pool size | 10 | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/configuration/configuration.go#L1268 | none |
| GitLab registry | database pool `maxopen` / `maxidle` / `maxlifetime` | 0 = unlimited / 0 = none / 0 = unlimited (example 25 / 25 / 5m) | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/docs/configuration.md#pool | none |
| GitLab registry | minimum PostgreSQL version | 16.0 | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/datastore/datastore.go#L8 | "guaranteed to work" |
| GitLab registry | hash partitions per partitioned table | 64 | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/datastore/migrations/structure.sql#L6300 | none for the number |
| GitLab registry | repository name / path length | 255 characters each | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/datastore/migrations/structure.sql#L6250-L6251 | none |
| GitLab registry | media type id width | SMALLINT | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/docs/supported-media-types.md | none; cited as the reason not to accept unknown types on public registries |
| GitLab registry | load balancing: replica lag limits | 30 s and 8 MiB (configuration.go declares 1 minute) | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/datastore/loadbalancing.go#L57-L61 | bytes: "This matches the Rails default" |
| GitLab registry | load balancing: up-to-date replica lookup timeout | 100 ms | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/datastore/loadbalancing.go#L32-L37 | "a starting point/educated guess that matches the one used in GitLab Rails" |
| GitLab registry | load balancing: replica quarantine | 5 minutes after 5 flaps in 60 s or 3 consecutive failures | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/datastore/loadbalancing.go#L63-L70 | none |
| GitLab registry | primary LSN record TTL (read-your-writes) | 1 hour | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/datastore/repository.go#L49 | none |
| GitLab registry | notification queue size per endpoint | 3000 events, then drop | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/notifications/endpoint.go#L14-L20 | yes: about 3000 bytes per event, production queue rarely above 150, about 8.5 MiB of RAM |
| GitLab registry | notification queue purge timeout at shutdown | 5 s | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/notifications/endpoint.go#L22-L34 | "Value chosen arbitrary", kept under the Kubernetes kill threshold |
| GitLab registry | notification retries (`maxretries`) | 10 | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/docs/configuration.md#endpoints | none |
| GitLab registry | HTTP `ReadHeaderTimeout` | 60 s | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/registry.go#L168-L172 | "prevent slow-loris attack, but be conservative at it" |
| GitLab registry | health check interval / database check timeout | 10 s / 5 s | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/handlers/app.go#L83-L88 | "Chosen arbitrary" |
| GitLab registry | rate limiter | off by default; example 5000 per minute, burst 8000, per IP | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/docs/configuration.md#rate-limiter | none |
| GitLab registry | background migrations `jobinterval` / `maxjobretries` | 1m / 0 | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/docs/configuration.md#backgroundmigrations | none |
| GitLab registry | importer tag concurrency | 1 by default, allowed 1 to 5, GCS only | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/root.go#L77 | none |
| GitLab registry | importer `--pre-import-skip-recent` | 72h | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/registry/root.go#L78 | lets an interrupted pre-import resume |
| GitLab.com | per-repository migration time limit | 30 minutes | https://gitlab.com/gitlab-org/container-registry/-/work_items/374 | bound the read-only window |
| GitLab.com | database statement timeout | 15 seconds | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/docs/database-migrations.md | none |
| olareg | manifest body max | 8 MiB (`1024 * 1024 * 8`); 413 above | https://github.com/olareg/olareg/blob/v0.2.2/config/config.go#L13 | none |
| olareg | referrers response max | 4 MiB, then paged | https://github.com/olareg/olareg/blob/v0.2.2/config/config.go#L16 | "OCI recommends 4MiB" (L86) |
| olareg | referrers page cache | 5 min; 1000 entries | https://github.com/olareg/olareg/blob/v0.2.2/config/config.go#L14-L15 | none |
| olareg | concurrent upload sessions per repository | 1000; overflow evicts least recently used down to 90% | https://github.com/olareg/olareg/blob/v0.2.2/config/config.go#L17 | none |
| olareg | upload session expiry | GC grace period (1 h) since last use; pruning timer at +10% | https://github.com/olareg/olareg/blob/v0.2.2/internal/store/dir.go#L153-L155 | none |
| olareg | GC frequency | 15 min | https://github.com/olareg/olareg/blob/v0.2.2/config/config.go#L121-L123 | none |
| olareg | GC grace period | 1 h by blob file mtime | https://github.com/olareg/olareg/blob/v0.2.2/config/config.go#L124-L126 | "time to preserve recently pushed manifests and blobs" |
| olareg | index.json re-read interval | 1 s | https://github.com/olareg/olareg/blob/v0.2.2/internal/store/store.go#L22 | none |
| olareg | GC change-window slack | 250 ms | https://github.com/olareg/olareg/blob/v0.2.2/internal/store/dir.go#L231 | none |
| olareg | HTTP read-header timeout | 5 s (no other timeouts) | https://github.com/olareg/olareg/blob/v0.2.2/olareg.go#L95 | none |
| olareg | rate limit | off (0); when set, per source IP per 1 s, `Retry-After: 1`; counters kept 10 s | https://github.com/olareg/olareg/blob/v0.2.2/olareg.go#L42-L46 | none |
| olareg | opaque token lifetime | 5 min | https://github.com/olareg/olareg/blob/v0.2.2/config/auth.go#L37 | docs: password hashing once per token lifetime instead of per request |
| olareg | auth file poll interval | 5 s | https://github.com/olareg/olareg/blob/v0.2.2/config/auth.go#L36 | none |
| olareg | upload session id | 16 random bytes, base64url | https://github.com/olareg/olareg/blob/v0.2.2/internal/store/store.go#L138 | none |
| olareg | blob size max | none | https://github.com/olareg/olareg/blob/v0.2.2/blob.go#L259 | n/a |
| olareg | tag list page size | unbounded unless `n` is given | https://github.com/olareg/olareg/blob/v0.2.2/tag.go#L48-L51 | n/a |
| olareg | delete API | disabled by default | https://github.com/olareg/olareg/blob/v0.2.2/config/config.go#L95 | none (changed in v0.1.1) |
| olareg | listen port | 5000 | https://github.com/olareg/olareg/blob/v0.2.2/cmd/olareg/serve.go#L75 | none |
| trow | manifest body max | 4 MiB | https://github.com/Trow-Registry/trow/blob/v0.10.0/src/routes/manifest.rs#L153 | none |
| trow | GC interval | 600 s | https://github.com/Trow-Registry/trow/blob/v0.10.0/src/registry/garbage_collect.rs#L21 | none |
| trow | stale upload age | 1 day | https://github.com/Trow-Registry/trow/blob/v0.10.0/src/registry/garbage_collect.rs#L86 | none |
| trow | orphan blob idle threshold | 1 day since last access | https://github.com/Trow-Registry/trow/blob/v0.10.0/src/registry/garbage_collect.rs#L117 | none |
| trow | proxy cache high-water mark | 80% of `registry_proxies.max_size` (unset by default) | https://github.com/Trow-Registry/trow/blob/v0.10.0/src/registry/garbage_collect.rs#L38 | none |
| trow | proxied blob eviction batch | 500 | https://github.com/Trow-Registry/trow/blob/v0.10.0/src/registry/garbage_collect.rs#L161 | none |
| trow | bearer token lifetime | 3600 s | https://github.com/Trow-Registry/trow/blob/v0.10.0/src/routes/response/trow_token.rs#L22 | none |
| trow | graceful shutdown | 30 s | https://github.com/Trow-Registry/trow/blob/v0.10.0/src/main.rs#L203 | none |
| trow | concurrent-fetch wait poll | 200 ms, no deadline | https://github.com/Trow-Registry/trow/blob/v0.10.0/src/registry/storage.rs#L110 | none |
| trow | SQLite writer connections | 1 | https://github.com/Trow-Registry/trow/blob/v0.10.0/src/init_db.rs#L12 | none |
| trow | hashing buffer | 1 MiB | https://github.com/Trow-Registry/trow/blob/v0.10.0/src/utils/digest.rs#L12 | none |
| trow | catalog and tag page default | unlimited (`i64::MAX`) | https://github.com/Trow-Registry/trow/blob/v0.10.0/src/routes/catalog.rs#L31 | none |
| trow | Argon2 parameters | mem_cost 4066, time_cost 3 | https://github.com/Trow-Registry/trow/blob/v0.10.0/src/lib.rs#L124-L125 | none |
| trow | chart memory limit | 256Mi | https://github.com/Trow-Registry/trow/blob/v0.10.0/charts/trow/values.yaml#L126-L128 | none |
| trow | listen address | `[::]:8000` | https://github.com/Trow-Registry/trow/blob/v0.10.0/src/main.rs#L46 | none |
| angos | max_manifest_size | 5 MiB | https://github.com/project-angos/angos/blob/v1.13.0/doc/reference/configuration.md?plain=1#L122 | none |
| angos | max_blob_size | 100 GiB; 413 above | https://github.com/project-angos/angos/blob/v1.13.0/doc/reference/configuration.md?plain=1#L123 | none |
| angos | gc_grace_secs | 300 s | https://github.com/project-angos/angos/blob/v1.13.0/doc/reference/configuration.md?plain=1#L139 | yes: "must keep a value that exceeds clock skew plus the longest write stall" |
| angos | prune upload window (`-u`) | 1 h | https://github.com/project-angos/angos/blob/v1.13.0/doc/how-to/run-storage-maintenance.md?plain=1#L80 | yes: "must exceed the longest push you expect" |
| angos | scrub and prune concurrency | 25 | https://github.com/project-angos/angos/blob/v1.13.0/doc/how-to/run-storage-maintenance.md?plain=1#L48 | none |
| angos | writer backoff on a GC run marker | 5 attempts, 50 ms to 800 ms jittered | https://github.com/project-angos/angos/blob/v1.13.0/src/registry/metadata_store/gc.rs#L36-L40 | "a run only covers one batch, so the wait is short" |
| angos | max_concurrent_requests | 64 (Tokio worker threads) | https://github.com/project-angos/angos/blob/v1.13.0/doc/reference/configuration.md?plain=1#L115 | yes: I/O-bound work; rule of thumb 8-16x CPU cores |
| angos | query_timeout; grace period; handshake_timeout | 3600 s; 60 s; 10 s | https://github.com/project-angos/angos/blob/v1.13.0/doc/reference/configuration.md?plain=1#L90-L92 | none |
| angos | job concurrency: cache; replication; scan; index | 4; 4; 2; 4 | https://github.com/project-angos/angos/blob/v1.13.0/doc/reference/configuration.md?plain=1#L116-L119 | none |
| angos | blob_stream_frame_size | 128 KiB | https://github.com/project-angos/angos/blob/v1.13.0/doc/reference/configuration.md?plain=1#L124 | yes: fewer allocations and body writes per blob |
| angos | shutdown_drain_secs | 30 s | https://github.com/project-angos/angos/blob/v1.13.0/doc/reference/configuration.md?plain=1#L133 | none |
| angos | S3 multipart_part_size | 50 MiB (allowed 5 MiB to 5 GiB) | https://github.com/project-angos/angos/blob/v1.13.0/doc/reference/configuration.md?plain=1#L276 | partly: S3's 5 MiB minimum and 5 GiB per-part ceiling |
| angos | S3 multipart copy | threshold 5 GB; chunk 100 MB; 4 jobs | https://github.com/project-angos/angos/blob/v1.13.0/doc/reference/configuration.md?plain=1#L277-L279 | partly: keeps completion inside S3 copy limits |
| angos | S3 retries | 3 attempts; backoff 50 ms doubling to 1 s | https://github.com/project-angos/angos/blob/v1.13.0/doc/explanation/storage-backends.md?plain=1#L345-L346 | yes: "so a throttled bucket is not hammered" |
| angos | S3 timeouts | 900 s per operation; 300 s per attempt | https://github.com/project-angos/angos/blob/v1.13.0/doc/reference/configuration.md?plain=1#L282-L283 | yes: streamed upload bodies exempt because the client paces them |
| angos | S3 circuit breaker | 5 consecutive failures; 10 s cooldown | https://github.com/project-angos/angos/blob/v1.13.0/doc/reference/configuration.md?plain=1#L284-L285 | none |
| angos | presign_ttl_secs | 1800 s | https://github.com/project-angos/angos/blob/v1.13.0/doc/reference/configuration.md?plain=1#L287 | none |
| angos | job queue | 5 attempts; backoff 100 ms to 10 s; claim lease 60 s | https://github.com/project-angos/angos/blob/v1.13.0/doc/reference/configuration.md?plain=1#L183-L186 | partly: lease refreshed at a third of its length |
| angos | token service ttl_secs | 3600 s (at most 86400 s) | https://github.com/project-angos/angos/blob/v1.13.0/doc/reference/configuration.md?plain=1#L431 | none |
| angos | OIDC | JWKS refresh 3600 s; clock skew tolerance 60 s | https://github.com/project-angos/angos/blob/v1.13.0/doc/reference/configuration.md?plain=1#L383-L385 | none |
| angos | upstream (pull-through) | connect 30 s; read inactivity 300 s; 5 redirects | https://github.com/project-angos/angos/blob/v1.13.0/doc/reference/configuration.md?plain=1#L507-L509 | yes for the read timeout: bounds a stall, not the whole transfer |
| angos | pull history | 1000 pulls per target; compaction after 3600 s | https://github.com/project-angos/angos/blob/v1.13.0/doc/reference/configuration.md?plain=1#L214-L216 | none |
| angos | revision_cache_entries | 100000 | https://github.com/project-angos/angos/blob/v1.13.0/doc/reference/configuration.md?plain=1#L136 | sizing note only (about 200 bytes each) |
| nora | manifest body max | 4 MiB | https://github.com/getnora-io/nora/blob/v1.4.7/nora-registry/src/registry/docker.rs#L549 | "4 MiB matches common registry limits" |
| nora | concurrent upload sessions | 100; overflow answered 429 TOOMANYREQUESTS | https://github.com/getnora-io/nora/blob/v1.4.7/nora-registry/src/registry/docker.rs#L495 | "prevent memory exhaustion" |
| nora | Retry-After on upload overflow | random 3 to 10 s | https://github.com/getnora-io/nora/blob/v1.4.7/nora-registry/src/registry/docker.rs#L519-L523 | yes: a fixed value re-synchronizes refused clients |
| nora | upload session size max | 2048 MB | https://github.com/getnora-io/nora/blob/v1.4.7/nora-registry/src/registry/docker.rs#L497 | none |
| nora | upload session TTL | 30 min | https://github.com/getnora-io/nora/blob/v1.4.7/nora-registry/src/registry/docker.rs#L499 | none |
| nora | request body limit | 2048 MB | https://github.com/getnora-io/nora/blob/v1.4.7/nora-registry/src/config/server.rs#L85-L87 | "2GB - enough for any Docker image" |
| nora | proxy temp file max age | 4 h | https://github.com/getnora-io/nora/blob/v1.4.7/nora-registry/src/registry/docker.rs#L693-L697 | yes: long enough for slow multi-GB downloads, short enough to reclaim orphans |
| nora | negative (miss) cache | 30 s TTL; 10,000 entries | https://github.com/getnora-io/nora/blob/v1.4.7/nora-registry/src/config/registry/docker.rs#L212-L215 | partly: only saved upstream requests are lost on overflow |
| nora | upstream download resume | 5 attempts; pause n x 500 ms | https://github.com/getnora-io/nora/blob/v1.4.7/nora-registry/src/registry/docker.rs#L4622-L4625 | none |
| nora | upstream tag list pages | 200 | https://github.com/getnora-io/nora/blob/v1.4.7/nora-registry/src/registry/docker.rs#L4978-L4980 | yes: a listing that stopped early would look complete |
| nora | GC | off by default; interval 86400 s; grace 604800 s | https://github.com/getnora-io/nora/blob/v1.4.7/nora-registry/src/config/gc.rs#L36-L46 | yes: must exceed the slowest blob-to-manifest gap; "matches docker-distribution's reference default" |
| nora | rate limits (on by default) | auth 1/s burst 5; writes 200/s burst 500; reads 1000/s burst 2000 | https://github.com/getnora-io/nora/blob/v1.4.7/nora-registry/src/config/rate_limit.rs#L55-L72 | none |
| nora | keep-alive timeout | 120 s | https://github.com/getnora-io/nora/blob/v1.4.7/nora-registry/src/config/server.rs#L50-L53 | yes: above traefik's 90 s and nginx's 60 s idle timeouts |
| nora | header read timeout | 30 s | https://github.com/getnora-io/nora/blob/v1.4.7/nora-registry/src/config/server.rs#L45-L47 | yes: a slow request head cannot hold a connection |
| nora | max connections | 4096 | https://github.com/getnora-io/nora/blob/v1.4.7/nora-registry/src/config/server.rs#L56-L57 | "Keep it below the process's open-file limit" |
| nora | shutdown drain | 15 s | https://github.com/getnora-io/nora/blob/v1.4.7/nora-registry/src/config/server.rs#L38-L42 | yes: fits Kubernetes' default 30 s grace period |
| nora | Docker upstream | timeout 300 s; read timeout 60 s; metadata_ttl -1 (revalidate every pull) | https://github.com/getnora-io/nora/blob/v1.4.7/nora-registry/src/config/registry/docker.rs#L200-L210 | none |
| mcronce/oci-registry | manifest and blob invalidation time | 14 days each | https://github.com/mcronce/oci-registry/blob/v0.4.5/src/upstream.rs#L74-L80 | none |
| mcronce/oci-registry | cache cleanup interval | 300 s | https://github.com/mcronce/oci-registry/blob/v0.4.5/src/main.rs#L99 | none |
| go-containerregistry pkg/registry | any limit | none: uploads buffered in memory without bound | https://github.com/google/go-containerregistry/blob/v0.22.1/pkg/registry/blobs.go#L167-L168 | n/a |
| cloudflare/serverless-registry | minimum chunk | 5 MiB | https://github.com/cloudflare/serverless-registry/blob/a73605d6d5587286eb9cfcb910148f7d1423f218/src/chunk.ts#L6 | none |
| cloudflare/serverless-registry | maximum chunk | 5 GiB | https://github.com/cloudflare/serverless-registry/blob/a73605d6d5587286eb9cfcb910148f7d1423f218/src/chunk.ts#L9 | none |
| cloudflare/serverless-registry | maximum chunk per upload request | 100 MB | https://github.com/cloudflare/serverless-registry/blob/a73605d6d5587286eb9cfcb910148f7d1423f218/src/chunk.ts#L12 | none |
| cloudflare/serverless-registry | maximum layer through docker push | 500 MB | https://github.com/cloudflare/serverless-registry/blob/a73605d6d5587286eb9cfcb910148f7d1423f218/README.md?plain=1#L170 | yes: request body size limit of the Workers plan |
| cloudflare/serverless-registry | upload state expiry | 2 h | https://github.com/cloudflare/serverless-registry/blob/a73605d6d5587286eb9cfcb910148f7d1423f218/src/registry/r2.ts#L188 | none |
| cloudflare/serverless-registry | staged small chunk expiry | 1 h (full push-compatibility mode) | https://github.com/cloudflare/serverless-registry/blob/a73605d6d5587286eb9cfcb910148f7d1423f218/src/registry/r2.ts#L890-L895 | none |
| peryx | manifest push body cap | 4 MiB (413, code SIZE_INVALID) | https://github.com/tox-dev/peryx/blob/v2026.1118.1/site/content/ecosystems/oci/reference/endpoints.md#L418 | none for the number; the status code choice is explained |
| peryx | upload session idle reclamation | 1 hour without a status GET or PATCH | https://github.com/tox-dev/peryx/blob/v2026.1118.1/site/content/ecosystems/oci/reference/registry-behavior.md#L120 | none |
| peryx | upload reclamation pass interval | once per minute (schedule configurable) | https://github.com/tox-dev/peryx/blob/v2026.1118.1/site/content/ecosystems/oci/reference/registry-behavior.md#L121 | none |
| peryx | `cache_ttl_secs` (freshness window, fallback and ceiling) | 300 s | https://github.com/tox-dev/peryx/blob/v2026.1118.1/site/content/core/operations/configuration.md#L24 | partly: a ceiling so that an upstream max-age cannot pin a page for a year (L44-L51) |
| peryx | `max_stale_secs` (stale-on-error bound) | 300 s | https://github.com/tox-dev/peryx/blob/v2026.1118.1/site/content/core/operations/configuration.md#L27 | none for the number |
| peryx | negative cache for an upstream 404 on a tag | 30 s | https://github.com/tox-dev/peryx/blob/v2026.1118.1/site/content/ecosystems/oci/reference/registry-behavior.md#L65 | "bounds how far behind a publication the repository can be" (consequence stated, number not derived) |
| peryx | upload session id | 128-bit random, 32 lowercase hex characters | https://github.com/tox-dev/peryx/blob/v2026.1118.1/site/content/ecosystems/oci/reference/registry-behavior.md#L102 | none |
| Gitea | max manifest body | 10 MiB (`10 * 1024 * 1024`), 413 above | https://github.com/go-gitea/gitea/blob/v28.4.0/routers/api/packages/container/container.go#L40-L42 | none for the value; comment links the spec's pushing-manifests section |
| Gitea | `_catalog` page size | default 100, max 100 | https://github.com/go-gitea/gitea/blob/v28.4.0/routers/api/packages/container/container.go#L203-L206 | none |
| Gitea | tags/list page size | unbounded when `n` is absent (`n := -1`) | https://github.com/go-gitea/gitea/blob/v28.4.0/routers/api/packages/container/container.go#L746-L749 | none |
| Gitea | tag length | 1 to 128 characters (`{0,127}` after the first) | https://github.com/go-gitea/gitea/blob/v28.4.0/routers/api/packages/container/container.go#L48-L50 | cites distribution-spec v1.1.1 pulling-manifests |
| Gitea | registry bearer token lifetime | 24 h | https://github.com/go-gitea/gitea/blob/v28.4.0/services/packages/auth.go#L37 | none |
| Gitea | cleanup task schedule | `@midnight`, also at start-up | https://github.com/go-gitea/gitea/blob/v28.4.0/services/cron/tasks_basic.go#L139-L147 | none |
| Gitea | unreferenced-blob and upload-session age before deletion (`OLDER_THAN`) | 24 h | https://github.com/go-gitea/gitea/blob/v28.4.0/custom/conf/app.example.ini#L2391-L2392 | none for the value: "Unreferenced blobs created more than OLDER_THAN ago are subject to deletion" |
| Gitea | container upload size limit (`LIMIT_SIZE_CONTAINER`) | -1 (no limit) | https://github.com/go-gitea/gitea/blob/v28.4.0/custom/conf/app.example.ini#L2895-L2896 | none |
| Gitea | in-memory buffer before spilling a request body to a temp file | 32 MiB | https://github.com/go-gitea/gitea/blob/v28.4.0/modules/packages/hashed_buffer.go#L29 | none |
| Forgejo | max manifest body | 10 MiB | https://codeberg.org/forgejo/forgejo/src/tag/v17.0.2/services/packages/container/manifest.go#L34-L36 | none |
| Forgejo | dangling-digest cleanup batch | 500 rows | https://codeberg.org/forgejo/forgejo/src/tag/v17.0.2/services/packages/container/cleanup_sha256.go#L20 | none |
| Pulp container | registry token lifetime (`TOKEN_EXPIRATION_TIME`) | 300 s | https://github.com/pulp/pulp_container/blob/2.30.1/pulp_container/app/authorization.py#L26 | none |
| Pulp container | catalog and tags page size | default `PAGE_SIZE` (100), max 10 x `PAGE_SIZE` | https://github.com/pulp/pulp_container/blob/2.30.1/pulp_container/app/registry_api.py#L581-L588 | none |
| Pulp container | wait for the repository task on manifest push | 2 s, then the response is returned; cancelled task gives 429 | https://github.com/pulp/pulp_container/blob/2.30.1/pulp_container/app/registry_api.py#L1740-L1743 | none |
| Pulp container | signature payload max size | 4 MB (`4 * 1_000_000`) | https://github.com/pulp/pulp_container/blob/2.30.1/pulp_container/constants.py#L73-L74 | none |
| Pulp container | parallel signing tasks (`MAX_PARALLEL_SIGNING_TASKS`) | 10 | https://github.com/pulp/pulp_container/blob/2.30.1/pulp_container/app/settings.py#L9-L10 | none (comment states meaning only) |
| Pulp (pulpcore) | orphan protection time | 1440 min | https://github.com/pulp/pulpcore/blob/3.124.0/pulpcore/app/settings.py#L304-L305 | partly: "This should ideally be longer than your longest running task." |
| Pulp (pulpcore) | cache entry TTL (`CACHE_SETTINGS.EXPIRES_TTL`), cache off by default | 600 s | https://github.com/pulp/pulpcore/blob/3.124.0/pulpcore/app/settings.py#L338-L341 | none |
| Nexus Repository CE | usage caps | 40,000 components and 100,000 requests per day | https://help.sonatype.com/en/ce-onboarding.html | commercial: "aligns our free Community Edition tier with teams and smaller organizations" |
| Nexus Repository | embedded H2 support limit | 200,000 requests per day or 100,000 components | https://help.sonatype.com/en/sonatype-nexus-repository-system-requirements.html | none ("Workloads beyond these limitations are not supported") |
| Nexus Repository | Docker port connectors per instance | 20 | https://help.sonatype.com/en/docker-registry.html | "Each port connector must reserve system resources to monitor the exposed port" |
| Nexus Repository | unused asset blob cleanup | every 30 min; blob-created delay 60 min | https://help.sonatype.com/en/tasks.html | "This delay avoids soft-delete files that take a significant amount of time to upload" |
| Nexus Repository | compact blob store "Blobs Older Than" | 0 days | https://help.sonatype.com/en/cleanup-policies.html | none |
| Dragonfly client | concurrent pieces per download | 8 | https://github.com/dragonflyoss/client/blob/v1.5.14/dragonfly-client-config/src/dfdaemon.rs#L185-L189 | none |
| Dragonfly client | piece download timeout | 360 s | https://github.com/dragonflyoss/client/blob/v1.5.14/dragonfly-client-config/src/dfdaemon.rs#L173-L177 | none |
| Dragonfly client | GC interval | 900 s | https://github.com/dragonflyoss/client/blob/v1.5.14/dragonfly-client-config/src/dfdaemon.rs#L328-L332 | none |
| Dragonfly client | task TTL | 30 days (2,592,000 s) | https://github.com/dragonflyoss/client/blob/v1.5.14/dragonfly-client-config/src/dfdaemon.rs#L334-L338 | none |
| Dragonfly client | disk GC high / low threshold | 80% / 60% | https://github.com/dragonflyoss/client/blob/v1.5.14/dragonfly-client-config/src/dfdaemon.rs#L381-L391 | none |
| Dragonfly client | download and back-to-source bandwidth limit | 50 GB/s each | https://github.com/dragonflyoss/client/blob/v1.5.14/dragonfly-client-config/src/dfdaemon.rs#L160-L171 | none |
| Dragonfly client | prefetch bandwidth limit | 10 GB/s | https://github.com/dragonflyoss/client/blob/v1.5.14/dragonfly-client-config/src/dfdaemon.rs#L411-L416 | "The prefetch request has lower priority so limit the rate to avoid occupying the bandwidth impact other download tasks." |
| Uber Kraken | blob size limit (operational) | 20 G | https://github.com/uber/kraken/blob/v0.1.37/README.md#L182-L186 | "Peers enforce connection limits on a per blob basis, and new peers might be starved for connections if no peers become seeders relatively soon." |
| Uber Kraken | seeder time-to-idle (example config) | 5m | https://github.com/uber/kraken/blob/v0.1.37/docs/CONFIGURATION.md#L93-L101 | none |
| Uber Kraken | on-disk cache and download cleanup time-to-idle (example config) | 6h each | https://github.com/uber/kraken/blob/v0.1.37/docs/CONFIGURATION.md#L103-L113 | none |
| Uber Kraken | origin cache capacity (example config) | 1000000 files | https://github.com/uber/kraken/blob/v0.1.37/docs/CONFIGURATION.md#L115-L120 | none |
| OpenShift image registry | pruner defaults | daily at midnight; keepTagRevisions 3; keepYoungerThanDuration 60m | https://github.com/openshift/openshift-docs/blob/0d60203614d4f683b27ce00852668dce37160993/modules/pruning-images.adoc#L74-L77 | none |

## Assumptions in the brief that the sources contradict

**CNCF Distribution v3.1.2, with the OCI distribution-spec as yardstick**

- "Distribution v3 GA changed things in 2025–2026": v3.0.0 GA is 2025-04-03; the 2026 releases are v3.1.0 (tag pagination, CVE fixes), v3.1.1 and v3.1.2. None of them adds the referrers API, sha512 manifests or online GC (F006, F007, F018, F048).
- "referrers API + artifactType filtering" for Distribution: not implemented in any release as of 2026-11-20; PR #4828 is open (F006).
- "conformance-suite status per category": the four categories are the v1.0/v1.1 suite; the suite on distribution-spec main is rewritten into about 52 named API and Data workflows, and the public results site already reports olareg and Zot in that form (F009, F015). Distribution's own CI is pinned to the v1.0.1 action (F005).
- Lead "layer links (`_layers`) never removed": outdated for v3: `MarkAndSweep` deletes unmarked layer links (F021); open reports of stale `_layers` remain.
- Lead "#3178 fixed in which release": fixed by PR #4285 (2024-04-21), shipped in v3.0.0; never backported to 2.8 (F019).
- Lead "in-flight de-duplication of concurrent blob fetches (the `inflight` map)": the map exists but only prevents double caching; concurrent requests are not coalesced and each fetches from the upstream (F027).
- Lead "`maxthreads` in storage/driver/base/regulator.go": the regulator is generic; the `maxthreads` default 100 / minimum 25 lives in the filesystem driver, GCS has `maxconcurrency` 50 / 25, and S3 and Azure are not regulated (Constants).
- Lead "tags/list pagination": `n` and `last` already worked in v3.0.0 (in memory, after reading all tags); v3.1.0 changed the mechanism and added the `maxtags` cap (F031).
- Lead "ttl 0, exec credential helper": both exist in v3.1.2 (`ttl: 0` disables expiry; `proxy.exec` with `command` and `lifetime`) (F024, F028).
- Lead "notifications ... maxretries/queue": there is no maxretries or queue-size option; retries are unbounded behind a circuit breaker (threshold, backoff) and the queue is unbounded (F037).
- "digest algorithms beyond sha256" as part of conformance: the released v1.1.1 conformance tests have no sha512 case; sha512 rows exist only in the new suite on main (F008, F009).

**Harbor v2.16.0**

- "Harbor 2.13/2.14 features" as the newest : the newest release is v2.16.0 (2026-11-10), after v2.15.0 (2026-03-20); the changes that matter most for design are in 2.15 and 2.16 (F053, F063).
- Harbor would track Distribution v3 : v2.16.0 still builds its registry from a fork of Distribution v2.8.3; the v3 upgrade is an explicit non-goal of the GC proposal (F050).
- Redis as a required service : v2.16.0 bundles Valkey instead (and PostgreSQL 18); the docs' component table still shows Redis 7.2.6 and PostgreSQL 15.10 (F051).
- GC sweeps tags through the registry v2 API : true until v2.14; since v2.15.0 Harbor no longer writes tag files to storage and GC can skip tag deletion (F063). GC workers are configurable from 1 to 10 since v2.9 (F060).
- "cosign/notation signature verification, enforcement at pull" : enforcement exists but only checks that a signature accessory is present; nothing is verified cryptographically (F080).
- "sha512 digest support (which release)" : no release adds it in Harbor's own layer; no statement found (F056).
- Server is amd64 only : arm64 installers ship from v2.16.0 (F099).
- goharbor/perf holds the results : the repository holds k6 scripts; the numbers are on its wiki pages, one per release (F090).
- A conformance report is published : the README link returns 404 and Harbor is absent from the OCI conformant-products list (F054).
- Proxy cache evicts by TTL : there is no cache TTL; a 7-day tag-retention policy plus GC removes content (F071).

**zot v2.1.23**

- "zot 2.1.x features such as scale-out clustering, event sinks, retention policies": retention policies shipped in v2.0.0 (2023-12-18), scale-out clustering in v2.1.0 (2024-07-09) and event sinks in v2.1.3 (2025-05-22); only event sinks fall inside 2025 to 2026 (F125, F134, F156, F165).
- "dedupe via hard links and a BoltDB/Redis/DynamoDB cache": hard links apply to local filesystems only; on S3, GCS and Azure dedupe leaves zero-byte placeholder objects, and the "cache" database is required to read them, so it is state, not a cache (F110, F147).
- "blobs on local filesystem ... or S3": GCS (v2.1.15, 2026-03-08) and Azure Blob Storage (v2.1.18, 2026-06-24) are also supported (F108).
- "zot's zb benchmark tool and published results": no result table or dashboard is published; the docs and README carry one sample output without hardware or version, and CI benchmark data stays in an Actions cache (F158, F159).
- "conformance-suite status per category (pull, push, content discovery, content management)": the current suite at distribution-spec main, which zot runs and submits, reports per-API and per-data rows instead of those four categories (F112).
- "online inline GC": the docs say inline, but the implementation is periodic background tasks per repository that take the store-wide write lock; nothing is collected inline with a delete (F116, F117, F119).
- Windows: the brief's lead questioned Windows support; native Windows binaries for amd64 and arm64 have shipped since v2.1.3, with the S3 driver excluded (F164).
- Size: the "minimal" build is not small any more (86.6 MB on linux/amd64 at v2.1.23) (F160).
- Sync internals: mirroring uses regclient and, since v2.1.22, always preserves upstream digests; `preserveDigest` is deprecated and ignored (F126, F115).

**Project Quay v3.18.3**

- "A Quay proxy-cache organization cannot be pushed to" (lead in the brief): wrong for every release to date; proxy organizations accept pushes and whether to change that was still being discussed on 2026-11-19 (F190).
- "Quay is Python plus Go helpers": no longer the whole picture; since 2026 the repository contains a Go registry built on CNCF Distribution v3 with SQLite metadata, tagged mirror-registry-v3.0.0 on 2026-11-09, and tracker epics describe a wider Go rewrite (F172).
- "mirror-registry v3.0.0 is a release of quay/mirror-registry": it is a tag in quay/quay with source archives only; quay/mirror-registry's newest release is v2.0.12 (2026-09-03) (F172).
- "PUSH_TEMP_TAG_EXPIRATION_SEC is one hour": six hours since v3.17.5 (2026-09-29) and v3.18.1 (2026-10-05); the product documentation still explains GC timing in terms of one hour (F181).
- "Pull-through caches serve cached content when the upstream is down": documented from the start, but not true in 3.12 to 3.15; fixed in v3.18.0 (F189).
- "Next version after 3.18 is 3.19": the documentation's main branch and tracker fix versions name version 5 / v5.0.0 (F219).
- "MySQL status?": deprecated since 3.6 and "planned for removal", still accepted by the code at v3.18.3 (F170).

**GitLab Container Registry v4.43.0-gitlab**

- "Is the database mandatory in 2026?" : no. It is the default only for new installations ("prefer" mode, GitLab 19.0); existing registries keep legacy filesystem metadata, and its removal was taken off the milestone and "is being reassessed" (F224).
- "Referrers API and artifactType filtering (when added)" : the OCI referrers endpoint was never added. `subject` is stored and referrers are listed, with an artifact-type filter, only through GitLab's private tag-list API (F229). Conformance is still run at spec v1.0.1 (F228).
- "Storage drivers s3/s3_v2 and azure/azure_v2" : `s3` and `azure` are now only aliases of the v2 drivers; the v1 implementations are gone (F227).
- "Design docs under docs/spec/gitlab/ include database layout and import tool" : the import tool, upstream differences and storage usage documents are in `docs/`; `docs/database-layout.md` is a five-line pointer to a generated diagram. The schema of record is `registry/datastore/migrations/structure.sql`.
- "`gc.blobs.storagetimeout`" : the specification still says the default is 2 seconds; the code and configuration reference say 5s (F238).
- "Tag delete endpoint `DELETE /v2/<name>/tags/reference/<tag>`" (described in docs/upstream-differences.md) : removed in v4.0.0 (2024-04-29); the document is stale. Tags are deleted with `DELETE /v2/<name>/manifests/<tag>`.

**olareg v0.2.2; the Rust registries trow, angos, nora and smaller ones; go-containerregistry pkg/registry; cloudflare/serverless-registry**

- The brief names `oci-distribution` as a Rust registry : it is a client crate, renamed `oci-client` (repository oras-project/rust-oci-client); it is not a server (F343).
- The brief's examples of Rust registries (`oci-distribution`/`trow`) : they miss the two most active Rust registries of 2026, which are angos (project-angos/angos, formerly simple-registry, v1.13.0 on 2026-11-17) and nora (getnora-io/nora, v1.4.7 on 2026-11-18); trow's last release is v0.10.0 from 2026-04-13 (F312, F327, F311).
- Lead "olareg has no auth built in" : basic auth and opaque-token auth with per-repository ACLs were added in v0.2.0 on 2026-04-13 (F291).
- Lead "trow has no GC or a simple one" : since v0.7.2 (2025-02-21) it has an online collector for stale uploads, orphan blobs and LRU eviction of proxied blobs (F304).
- Lead "trow has a proxy-registry configuration" : since v0.9.0 (2025-08-06) any registry is proxied without configuration; the config file only adds credentials and a size cap (F305).
- The brief asks for conformance status per category (pull, push, content discovery, content management) : the OCI conformance site now reports newer entries per workflow (52 workflows for olareg at spec v1.1) and keeps the four categories only for older entries (F284).
- Candidate "mcronce/oci-registry" as a maintained Rust registry : its last release is v0.4.5 from 2024-04-23 and it is a pull-through cache that cannot accept pushes (F338).

**peryx (Rust), examined briefly; two peer-reviewed papers on Distribution-based registries**

- The brief's list of Rust registries (`oci-distribution`/`trow`) : besides trow, the maintained Rust registry servers found are angos, nora and peryx (F312, F327, F351); `oci-distribution` is a client crate (F343).

**Gitea and Forgejo, Pulp container, Sonatype Nexus Repository, Dragonfly, Uber Kraken, OpenShift image registry**

- "Sonatype Nexus OSS" as an open implementation of a container registry: the open-source code base is now "Nexus Repository Core" with maven, raw and APT only; the Docker format ships in the closed, usage-capped Community Edition and paid editions (F380, F381).
- "Forgejo/Gitea packages" as one target: they are two diverging code bases (different module paths, licences for new files, cleanup passes, quota, bug sets); findings for one do not transfer automatically (F367, F371).
- Gitea version numbers: the newest release is v28.4.0, not a 1.x number; the 1.27 line ended at v1.27.3 on 2026-08-29 (F372).
- "Dragonfly's registry pieces": Dragonfly has no registry; it is a pull-side P2P mirror plus preheat in front of a registry, CNCF-graduated since 2025-10-28 (F390, F391, F394).
- Pulp pull-through caching is not a finished feature: it is still labelled tech preview in 2.30.1 (F376).

## Not verified / open questions

**All registries**

- What the verification covers: every quoted passage was matched by script against the downloaded source and against a fresh fetch of its cited URL on 2026-11-20 (for GitLab file pages, GitHub file pages and the Quay tracker, the raw or API form of the same URL, because those pages are rendered by scripts; for the two papers, text extracted from the PDF fetched from the cited URL). That shows the words are in the source. Whether a claim follows from its quotes was checked by a second reading for every claim line and for the full text of about 75 of the 401 findings, chosen for performance, garbage collection and limits; the rest were not re-read in full.
- The catalogue of constants: 348 of its 407 rows were checked against the cited lines of the downloaded file (346 by script: the value or the constant's name appears there; 2 by hand); the other rows cite documentation pages, issues or files that were fetched singly and were not checked that way.
- Comment counts and open or closed states of GitHub issues come from the GitHub issue search API on 2026-11-20.
- No number in these notes was reproduced: nothing was built, run or measured.
- Open-source registries named by the sources and not examined: Django-OCI, Keppel, Keygen and OpenRegistry (listed on the OCI conformance site, F009); ferro-oci-server and summ (Rust, found by repository search; see the list for the small registries below). peryx was examined only for Q1 to Q4, Q9, Q10, Q12 and Q13 (F351 to F358).
- Not found for any registry: the duration of a full garbage-collection run at a stated registry size on stated hardware (nearest: GitLab's 15k-blob figure and 64-day estimate, F269 and F233; Harbor's per-manifest delete times, F061; the Nexus cleanup record, F385), and a whole-registry throughput figure with hardware, version, load and number of runs all stated (nearest: Harbor's k6 reports, F090 to F093; the vendor's Quay sizing post, F210; nora's footprint table and load test, F328 and F329; peryx's comparison, F357). What each of these leaves out is in its Conditions/notes line.

**CNCF Distribution v3.1.2, with the OCI distribution-spec as yardstick**

- Duration of a full `garbage-collect` run at a stated scale (repositories, manifests, blobs, backend): not found in Distribution docs, release notes or the issues read (#1609, #2212, #3178, #3366, #3851, #4382, #4461); only the blob-enumeration timings of PR #3480 (F040).
- Idle RSS, CPU footprint, requests per second or latency percentiles for the registry as a whole: not found (docs/, README, release notes, PRs #3480, #4066, #4906, #4919, #5081).
- HTTP status actually returned for a manifest over 4 MiB (the code appends MANIFEST_INVALID; whether the client sees 400 or a closed connection was not tested by running the registry) (F011).
- Behaviour of `--delete-untagged` on manifests carrying `subject` (untagged referrers): inferred from code, not tested (F006).
- Total number of tests in the new conformance suite behind PR #4946's "779 -> 950 passing tests": not stated in the PR text read.
- What CVE-2026-35172, CVE-2026-33540 and CVE-2026-41888 map to: the v3.1.0/v3.1.1 notes give only the numbers; the advisory list shows GHSA-f2g3-hh2r-cwgc and GHSA-3p65-76g6-3w7r published 2026-04-06 and GHSA-6pjf-3r9x-m592 published 2026-05-01, but the CVE-to-GHSA mapping was not confirmed on the advisory pages.
- Machine model, core count and network for the PR #5081 and PR #3480 measurements: not stated in the quoted parts of the PRs.
- Whether the conformance action commit 10efb871 is exactly tag v1.0.1: taken from the workflow's own comment, not checked against the spec repository's tags.
- Year of the conformance site's "Last generated: Aug 5" line: taken from the oci-conformance commit page timestamp (2026-08-05T16:54:13-05:00), not printed on the site itself.
- Docker Hub / GHCR image sizes for `registry:3`: not looked up (release tarball sizes used instead, F045).

**Harbor v2.16.0**

- Per-category OCI conformance results for any Harbor version : README report URL and the workflow's S3 bucket both returned 404 on 2026-11-20; conformance.opencontainers.org has no Harbor entry; the suite was not run here.
- Behaviour with sha512 digests (push, pull, referrers) : no doc or code statement; not tested.
- Blob size used in the published k6 runs : the wiki pages do not say whether `BLOB_SIZE` / `BLOBS_COUNT_PER_ARTIFACT` were overridden; 1 KiB x 1 is the script default (F091).
- Cause of the v2.14.4 to v2.15.4 API throughput drop : the PBKDF2 change is an inference from code and timing; no project statement found (F092).
- Replica counts, resource limits and database sizing of the perf test beds : "standard deploy" only; chart defaults read from harbor-helm v1.19.4 (F090).
- Idle RSS, CPU and binary size per component : not published; only installer size and stack minimums found (F052, F098).
- GC speed after the v2.15 tag change : the proposal lists the S3 benchmark as future work; no numbers found in PRs 22382 and 22507, release notes or wiki.
- Root cause of issue 15445 (GC deleted referenced blobs) : never confirmed by maintainers; the standby-query explanation is a user's (F064).
- Whether issue 22111 (blobs left on S3-compatible stores) reproduces on AWS S3 and on v2.16.0 : maintainers asked, no answer recorded (F103).
- Contents of Harbor's Distribution fork (tag v2.8.3-harbor.3) beyond the Redis patch name : the fork repository was not downloaded.
- Replication throughput or lag numbers : none published in docs, wiki or release notes.
- Proxy-cache overhead against the 50% design target : no measurement found (F105).
- Whether BuildKit cache export (issue 14774) still fails on v2.16.0 : not tested.
- Windows or macOS server support : no statement in docs; inferred from container-only packaging (F099).
- Close dates of issues 19991, 22739, 23025, 16915, 20505, 15048 : state taken from GitHub issue search on 2026-11-20, dates not recorded.

**zot v2.1.23**

- GC duration or cost at a stated registry size: not found in the v2.1.23 docs, the What's New page, the repository or the issues read.
- Idle RSS and CPU published by the project: not found; only a 2023 user report and an example systemd unit (F161).
- A public page with CI benchmark history: not found (https://project-zot.github.io/zot/dev/bench/ answered 404 on 2026-11-20).
- Versions of zot and Distribution used in the third-party comparison (F168): not stated on the page; its numbers were not reproduced.
- The release in which sha512 digests were first accepted: not found (v2.1.19 already passes the sha512 conformance rows).
- Whether v2.1.23 bounds the number of distinct queued on-demand syncs after the GHSA-c45h-jrgv-35rc fix: code read only as far as the singleflight grouping; no explicit cap seen, not confirmed.
- Signature enforcement: a lint for mandatory signatures was proposed in a pull request linked from issue #4098; no such option exists in the v2.1.23 tree (grep for it found nothing); its later status was not checked.
- An explicit documentation statement that a repository filled by sync also accepts pushes: not found; inferred from the active/active recipe and from the absence of a proxy-only repository type (F127).
- Contents of the open v2.2.0 milestone: not read.
- Issue comment counts and open/closed states come from the GitHub issue search API on 2026-11-20 and were not cross-checked on each issue page.
- The keying of the rate limiter was read in tollbooth v7.0.2 source, not confirmed by running zot.
- Helm chart defaults (resources, probes) and the zui repository were not examined.
- Legacy four-category conformance status (pull, push, content discovery, content management) for zot: not found in those terms.

**Project Quay v3.18.3**

- Conformance results: no published pass/fail report for any category was found; the claim rests on the CI job definition (F174). Whether the Python registry would pass content management if enabled: not tested.
- The referrers missing-subject behaviour, the absence of `OCI-Subject`, sha256-only digests and the implicit 1m manifest limit (F175, F176, F207) are inferred from code and templates at v3.18.3, not from a running instance.
- GC durations or throughput at scale (repositories per hour, time to reclaim a given volume): not found in documentation, tracker or blogs; only the scheduling constants are known (F179).
- Benchmark conditions gap: the 2026 vendor article does not state the Quay version; no push, tag-listing or GC benchmark was found anywhere (F210).
- Binary size, idle RSS and idle CPU of the Quay container or of the Go mirror registry: not found; only the 6 GB minimum and the 100-200 MB target are published (F172, F213).
- PROJQUAY-10909 (the GC race report behind PR #5390) and PROJQUAY-13529 return "does not exist or you do not have permission" to anonymous readers; their content was taken from the PR text and the changelog only.
- The enhancement proposal was fetched from the `main` branch of quay/enhancements; the cited permalink uses that branch's head commit (4e971ee, 2026-10-06) on the assumption that the file is unchanged at that commit.
- The two vendor developer-blog articles (S375, S376) refuse automated fetches (HTTP 403); they were read from Internet Archive snapshots (the 2026 one captured on 2026-11-20 for this research).
- GitHub issues on quay/quay are development chores (298 issues, the most-commented being dependency dashboards); operational pain points were therefore ranked from the vendor tracker by watcher count, which under-represents users who never file there.
- Vendor release dates ("Issued" lines in the release-note modules) differ from the upstream changelog and GitHub dates by days to weeks; upstream dates are used in F219.
- Clair internals, the Quay Operator, builders and billing were out of scope and not examined.

**GitLab Container Registry v4.43.0-gitlab**

- Method caveat : issue and epic quotes are taken from the `description` field returned by GitLab's public REST API (the cited pages render it client-side); issue comments require authentication through the API and were not read, so later corrections in comments may be missing.
- Resource footprint (binary size, idle memory, CPU) : not found in the registry repository, GitLab admin docs or runbooks; only "roughly 20" pods as GitLab.com's baseline.
- Measured API latency before and after the database (tag listing, manifest GET) : not found; GitLab publishes only qualitative statements and the `1+3N` request count.
- Online GC throughput on GitLab.com (tasks per second, queue depth, bytes per day) and GitLab.com's actual `gc` intervals : not found beyond "terabytes of data ... each day"; dashboards are private.
- Whether the tag-listing slowness in issue 2137 was fixed : the issue is closed (2026-02-19) but I found no matching CHANGELOG entry; issue comments need authentication through the API and were not read.
- Whether issue 2381 (layers deleted by online GC after an interrupted push) is fixed on master after v4.43.0 : open on 2026-11-20; code at the tag is unchanged.
- sha512 pushes end to end : the datastore encodes sha512 digests, but no test result or conformance statement was found.
- Whether an untagged referrer manifest is itself collected after 24 hours : inferred from the `IsDangling` query only; not confirmed by documentation or a test.
- Dependency Proxy behaviour when the upstream is down, and cache-stampede handling : not found in the user page read; the feature is outside the registry.
- Signature verification or pull-time enforcement : none found in registry or GitLab user docs; absence is not proof.
- The reason for 64 partitions and for the 24 hour review delay as numbers : no measurement found.
- Epic 13237 (registry rate limiting) : API returned 403; not read.
- Windows or macOS builds of the server : none found in CI or packaging; I did not try to compile.

**olareg v0.2.2; the Rust registries trow, angos, nora and smaller ones; go-containerregistry pkg/registry; cloudflare/serverless-registry**

- olareg: outputs of its CI conformance runs : not read; only the workflow definition and the OCI conformance site were used. Why the site marks the tag-parameter workflows as failing although RELEASE.md says v0.2.0 added pushing a digest with tags: not resolved.
- olareg: issue 276 (linux/arm/v6 image crash, opened 2026-11-18) : title seen in the issue search; body not read.
- olareg, trow, angos, mcronce, serverless-registry: benchmarks : searched READMEs, docs, changelogs and (for olareg, trow, angos, nora) the most-commented issues; none found except nora's (F328, F329) and mcronce's anecdote (F339).
- trow: per-category conformance results : the CI job exists (F302) but its logs were not read; trow is not on the OCI conformance site.
- trow: whether untagged manifests are ever collected : no such pass in src/registry/garbage_collect.rs; other code paths not searched exhaustively.
- trow: `registry_proxies.offline` : the field is parsed (src/configuration.rs L33) but a search of src/ at v0.10.0 finds no reader, so the "offline mode" announced for v0.6.0 may no longer have any effect; not confirmed by running it.
- angos: the rename from simple-registry : inferred from GitHub redirecting https://github.com/simple-registry/simple-registry and https://github.com/project-angos/simple-registry to project-angos/angos; no written statement found in the changelog or the v1.0.0 release notes.
- angos: correctness of the run-marker protocol on object stores without strong list-after-write consistency : the docs argue it; not independently checked, and no model or proof is published.
- angos: doc/reference/api-endpoints.md and the web UI reference : not read in full; search capabilities beyond listings unknown.
- nora: sha512 support, OCI conformance, per-repository token scoping, issue 693 (load-test harness) : not examined.
- nora: high availability : ARCHITECTURE.md says "No high availability" by design, while the maintainer's decision table in issue 130 lists "HA / multi-instance" with shared S3 and "Stateless NORA + shared storage"; which is current was not resolved.
- nora: the code comment that a 7-day GC grace "matches docker-distribution's reference default" : which Distribution setting is meant is not stated in the comment; compare with the Distribution section (upload purge age).
- mcronce/oci-registry, container_registry-rs, rk8s distribution : only README and a few source lines were read; storage layout, limits and GC were not examined.
- Other Rust registries found by GitHub repository search and not examined : ferro-oci-server (crates.io v1.0.0, 2026-06-09, repository abyo-software/ferro-protocols; a web search result says it claims 75/75 conformance specs, unverified), summcr/summ (v0.1.1, 2026-09-13), tox-dev/peryx (PyPI and OCI, releases through 2026-11-18), waynr/portfolio (last push 2023), DBCDK/ha-registry (archived).
- cloudflare/serverless-registry: conformance status, sha512, and the R2 consistency guarantees its GC marks rely on : not examined.
- Platform support (Q12) : "no Windows" and "no macOS" statements are based on release assets and build scripts; no build was attempted (builds were out of scope), so whether the sources compile on those platforms is unknown.
- Dates : release dates come from GitHub release feeds and changelog headings as fetched on 2026-11-20; issue counts and "most-commented" lists come from the GitHub issue search API on the same day.

**peryx (Rust), examined briefly; two peer-reviewed papers on Distribution-based registries**

- The two papers (F359 to F361) : quotes were checked against text extracted from the PDFs fetched from the cited URLs; figures that exist only in the papers' charts were not read off. Later peer-reviewed work on registry workloads was not searched for.
- peryx authentication, authorization, search, web UI, retention and trash features : present in the docs tree (site/content/core/access, core/repositories) but not read.
- peryx behaviour under the conformance suite : the CI recipe was read; no result table is published and the suite was not run here.
- Versions of zot and Distribution in peryx's comparison, and its behaviour on Linux server hardware : not stated on the page; all published numbers are from one Apple M4 laptop-class host over loopback.
- Whether peryx's HA reclamation protocol has been model-checked or fault-tested : not found in the docs read (an availability-harness page exists in the contributing docs and was not read).

**Gitea and Forgejo, Pulp container, Sonatype Nexus Repository, Dragonfly, Uber Kraken, OpenShift image registry**

- Gitea/Forgejo referrers tracking issue: not found (GitHub issue search for "referrers" in go-gitea/gitea returned 0 hits twice; Codeberg search surfaced only the general compliance issue 15324).
- Gitea conformance-suite result: not found (no published run; only the Forgejo user-run in F364). Whether the 184 Forgejo failures share few root causes: not analysed.
- Gitea/Forgejo published benchmarks, idle memory, CPU: not found (docs and release notes searched).
- Gitea multi-node behaviour of upload sessions: inferred from the temp-file code path (F370), no doc statement found.
- Reason for Gitea's jump from v1.27.x to v28.0.0: not looked up.
- Forgejo 17.0.2 state of issue 15258 (Range off by one): not verified against the 17.0.2 source.
- Pulp container: manifest size limit (none found in the plugin), conformance results, registry throughput benchmarks, published image architectures: not found. RBAC details were not quoted (docs/admin/learn/rbac.md exists).
- Nexus: default values of Docker proxy "Maximum component age"/"Maximum metadata age", behaviour when the remote is down, stampede handling, manifest size limit, blob dedupe across repositories, arm64 support: not found on the pages read. Date and product version of the cleanup performance test: not stated.
- Dragonfly: the IEEE Transactions on Networking paper was not read (only the project's announcement); no hardware-specified benchmark of image pulls found; most-reported operational issues not researched.
- Kraken: authentication/authorisation model beyond the README's one line, published image architectures, date and hardware of the README benchmark: not found.
- OpenShift image registry: only the pruning docs were read (optional scope); pull-through, auth and limits not researched.
