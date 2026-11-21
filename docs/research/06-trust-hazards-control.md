# Trust, hostile input and control literature: source notes
Researched 2026-11-20. Question: what do the primary sources say a from-scratch OCI registry must verify (signatures, attestations, vulnerability data), must survive (hostile archives, HTTP/2 and HTTP/1.1 abuse, decompression bombs), and can use to replace hand-picked constants with measured or derived values? Method: each page was downloaded with curl into `~/.cache/reliquary-research/tmp-trust/pages/<id>/`, converted to text, and every `Quote:` / `Counter:` line below was checked by script (`scripts/check_quotes.py` in that directory) to be an exact, whitespace-normalised substring of the saved text of the cited source; 195 of 195 passed on 2026-11-20, and a copy with one altered number and one wrong source id failed the same check.

Reading rules for this file: a quote is copied character for character from the saved page (Markdown markers, typos and curly quotes included); line breaks inside a quote are folded to one space. "PDF p.N" is the page of the PDF file, counted from 1. Where a publisher blocks automated fetches (ACM), the saved copy is the Internet Archive's unmodified capture of the publisher's page and the row says so.

## Sources
| id | type | title | URL | version/date |
|---|---|---|---|---|
| S1 | docs | Sigstore docs, "Sigstore Bundle Format" | https://docs.sigstore.dev/about/bundle/ | fetched 2026-11-20 |
| S2 | standard | cosign `specs/BUNDLE_SPEC.md` (Cosign Bundle Specification) | https://raw.githubusercontent.com/sigstore/cosign/main/specs/BUNDLE_SPEC.md | main, fetched 2026-11-20 |
| S3 | standard | cosign `specs/SIGNATURE_SPEC.md` | https://raw.githubusercontent.com/sigstore/cosign/main/specs/SIGNATURE_SPEC.md | main, fetched 2026-11-20 |
| S4 | docs | cosign `CHANGELOG.md` | https://raw.githubusercontent.com/sigstore/cosign/main/CHANGELOG.md | main (top entry v3.0.5), fetched 2026-11-20 |
| S5 | blog | Sigstore blog, "Cosign v3 is now available" (project's own blog) | https://blog.sigstore.dev/cosign-3-0-available/ | 2025-10-08 |
| S6 | docs | GitHub releases API, sigstore/cosign (release notes) | https://api.github.com/repos/sigstore/cosign/releases?per_page=20 | v3.1.4 published 2026-10-20; v3.1.1 2026-06-09; v3.0.1 2025-10-08 |
| S7 | blog | Sigstore blog, "Rekor v2 GA - Cheaper to run, simpler to maintain" | https://blog.sigstore.dev/rekor-v2-ga/ | 2025-10-10 |
| S8 | blog | Sigstore blog, "sigstore.dev and Rekor evolution" | https://blog.sigstore.dev/rekor-evolution/ | 2026-06-28 |
| S9 | blog | Sigstore blog, "Post-Quantum Cryptography for Sigstore" | https://blog.sigstore.dev/pqc-sigstore/ | 2026-10-02 |
| S10 | blog | Sigstore blog, "Rekor Public-Good Instance Hardening" | https://blog.sigstore.dev/rekor-public-instance-hardening/ | 2026-11-16 |
| S11 | standard | Sigstore architecture-docs, `client-spec.md` ("Sigstore Client") | https://raw.githubusercontent.com/sigstore/architecture-docs/main/client-spec.md | main, fetched 2026-11-20 |
| S12 | standard | Sigstore architecture-docs, `rekor-v2-spec.md` | https://raw.githubusercontent.com/sigstore/architecture-docs/main/rekor-v2-spec.md | main, fetched 2026-11-20 |
| S13 | standard | SLSA v1.2, "Build: Provenance" | https://slsa.dev/spec/v1.2/build-provenance | v1.2, "Status: Approved" |
| S14 | standard | in-toto Attestation Framework v1, `statement.md` | https://raw.githubusercontent.com/in-toto/attestation/main/spec/v1/statement.md | spec v1, main, fetched 2026-11-20 |
| S15 | docs | BuildKit, "Image attestation storage" | https://raw.githubusercontent.com/moby/buildkit/master/docs/attestations/attestation-storage.md | master, fetched 2026-11-20 |
| S16 | docs | `actions/attest` README | https://raw.githubusercontent.com/actions/attest/main/README.md | main, fetched 2026-11-20 |
| S17 | docs | `actions/attest-build-provenance` README | https://raw.githubusercontent.com/actions/attest-build-provenance/main/README.md | main (v4 wording), fetched 2026-11-20 |
| S18 | docs | GitHub Docs, "Artifact attestations" | https://docs.github.com/en/actions/concepts/security/artifact-attestations | fetched 2026-11-20 |
| S19 | standard | Notary Project, `signature-specification.md` | https://raw.githubusercontent.com/notaryproject/specifications/main/specs/signature-specification.md | main, fetched 2026-11-20 |
| S20 | vendor | Amazon ECR User Guide, "Signing an image" | https://docs.aws.amazon.com/AmazonECR/latest/userguide/image-signing.html | fetched 2026-11-20 |
| S21 | vendor | Amazon ECR User Guide, "Managed signing" | https://docs.aws.amazon.com/AmazonECR/latest/userguide/managed-signing.html | fetched 2026-11-20 |
| S22 | vendor | Microsoft Learn, "Sign container images by using Notation, Azure Key Vault, and a self-signed certificate" | https://learn.microsoft.com/en-us/azure/container-registry/container-registry-tutorial-sign-build-push | fetched 2026-11-20 (installs Notation v1.3.2) |
| S23 | standard | OSV schema, `docs/schema.md` | https://raw.githubusercontent.com/ossf/osv-schema/main/docs/schema.md | main, fetched 2026-11-20 |
| S24 | standard | OpenVEX Specification | https://raw.githubusercontent.com/openvex/spec/main/OPENVEX-SPEC.md | v0.2.0 |
| S25 | standard | IANA media type registration `application/spdx+json` | https://www.iana.org/assignments/media-types/application/spdx+json | registered 2021-11-03 |
| S26 | standard | IANA media type registration `application/vnd.cyclonedx+json` | https://www.iana.org/assignments/media-types/application/vnd.cyclonedx+json | registered 2021-01-28 |
| S27 | standard | CycloneDX, "Specification Overview" | https://cyclonedx.org/specification/overview/ | fetched 2026-11-20 (lists `version=1.7`) |
| S28 | docs | Trivy docs, "Discover VEX Attestation in OCI Registry" | https://raw.githubusercontent.com/aquasecurity/trivy/main/docs/guide/supply-chain/vex/oci.md | main, fetched 2026-11-20 |
| S29 | docs | GitHub Docs, "OpenID Connect" | https://docs.github.com/en/actions/concepts/security/openid-connect | fetched 2026-11-20 |
| S30 | docs | GitHub Docs, "Configuring OpenID Connect in Amazon Web Services" | https://docs.github.com/en/actions/how-tos/secure-your-work/security-harden-deployments/oidc-in-aws | fetched 2026-11-20 |
| S31 | standard | OCI Working Group: Auth, README | https://raw.githubusercontent.com/opencontainers/wg-auth/main/README.md | main (last commit 2026-01-05), fetched 2026-11-20 |
| S32 | paper | Fifield, "A better zip bomb" (author's page for the USENIX WOOT'19 paper) | https://www.bamsoftware.com/hacks/zipbomb/ | 2019-07-02, updated 2023-05-18 |
| S33 | paper | You et al., "My ZIP isn't your ZIP: Identifying and Exploiting Semantic Gaps Between ZIP Parsers", USENIX Security '25 (abstract page) | https://www.usenix.org/conference/usenixsecurity25/presentation/you | August 2025 |
| S34 | paper | same paper, PDF | https://www.usenix.org/system/files/usenixsecurity25-you.pdf | August 2025 |
| S35 | docs | snyk/zip-slip-vulnerability README (the discoverer's own list) | https://raw.githubusercontent.com/snyk/zip-slip-vulnerability/master/README.md | disclosure 2018-06-05 |
| S36 | standard | OCI Image Format Specification, `layer.md` | https://raw.githubusercontent.com/opencontainers/image-spec/v1.1.1/layer.md | v1.1.1 (released 2025-03-03) |
| S37 | standard | CVE record CVE-2018-15664 | https://cveawg.mitre.org/api/cve/CVE-2018-15664 | published 2019-05-23 |
| S38 | standard | CVE record CVE-2021-32760 | https://cveawg.mitre.org/api/cve/CVE-2021-32760 | published 2021-07-19 |
| S39 | standard | CVE record CVE-2021-29136 | https://cveawg.mitre.org/api/cve/CVE-2021-29136 | published 2021-04-06 |
| S40 | standard | CVE record CVE-2018-20990 | https://cveawg.mitre.org/api/cve/CVE-2018-20990 | published 2019-08-26 |
| S41 | standard | CVE record CVE-2021-38511 | https://cveawg.mitre.org/api/cve/CVE-2021-38511 | published 2021-08-10 |
| S42 | standard | CVE record CVE-2019-1002101 | https://cveawg.mitre.org/api/cve/CVE-2019-1002101 | published 2019-04-01 |
| S43 | standard | CVE record CVE-2021-37701 | https://cveawg.mitre.org/api/cve/CVE-2021-37701 | published 2021-08-31 |
| S44 | standard | CVE record CVE-2025-62518 | https://cveawg.mitre.org/api/cve/CVE-2025-62518 | published 2025-10-21 |
| S45 | blog | Edera, "TARmageddon (CVE-2025-62518)" (discoverer's write-up) | https://edera.dev/stories/tarmageddon | 2025-10-21 |
| S46 | standard | CVE record CVE-2022-2879 | https://cveawg.mitre.org/api/cve/CVE-2022-2879 | published 2022-10-14 |
| S47 | standard | CVE record CVE-2023-44487 | https://cveawg.mitre.org/api/cve/CVE-2023-44487 | published 2023-10-10 |
| S48 | vendor | Cloudflare, "HTTP/2 Rapid Reset: deconstructing the record-breaking attack" | https://blog.cloudflare.com/technical-breakdown-http2-rapid-reset-ddos-attack/ | 2023-10-10 |
| S49 | vendor | Google Cloud, "How it works: The novel HTTP/2 'Rapid Reset' DDoS attack" | https://cloud.google.com/blog/products/identity-security/how-it-works-the-novel-http2-rapid-reset-ddos-attack | 2023-10-10 |
| S50 | blog | Nowotarski, "HTTP/2 CONTINUATION Flood: Technical Details" (discoverer's write-up) | https://nowotarski.info/http2-continuation-flood-technical-details/ | 2024-04-03 |
| S51 | standard | CERT/CC VU#421644, "HTTP/2 CONTINUATION frames can be utilized for DoS attacks" | https://kb.cert.org/vuls/id/421644 | 2024-04-03, revised 2024-07-19 |
| S52 | standard | CVE record CVE-2025-8671 | https://cveawg.mitre.org/api/cve/CVE-2025-8671 | published 2025-08-13 |
| S53 | standard | CERT/CC VU#767506, "HTTP/2 implementations are vulnerable to "MadeYouReset" DoS attack through HTTP/2 control frames" | https://kb.cert.org/vuls/id/767506 | 2025-08-13, revised 2026-03-17 |
| S54 | blog | Gal Bar Nahum, "The MadeYouReset HTTP/2 Vulnerability - Introduction" (discoverer's write-up) | https://galbarnahum.com/posts/made-you-reset-intro | August 2025 |
| S55 | vendor | Netflix security bulletin NFLX-2019-002, "HTTP/2 Denial of Service Advisory" | https://raw.githubusercontent.com/Netflix/security-bulletins/master/advisories/third-party/2019-002.md | 2019-08-13 |
| S56 | standard | RFC 9113, HTTP/2 | https://www.rfc-editor.org/rfc/rfc9113.txt | June 2022 |
| S57 | standard | RFC 7541, HPACK | https://www.rfc-editor.org/rfc/rfc7541.txt | May 2015 |
| S58 | vendor | Imperva Hacker Intelligence Initiative, "HTTP/2: In-depth analysis of the top four flaws of the next generation web protocol" (discoverer's report, PDF) | https://www.imperva.com/docs/Imperva_HII_HTTP2.pdf | 2016 |
| S59 | standard | RFC 9112, HTTP/1.1 | https://www.rfc-editor.org/rfc/rfc9112.txt | June 2022 |
| S60 | docs | PortSwigger Web Security Academy, "HTTP request smuggling" | https://portswigger.net/web-security/request-smuggling | fetched 2026-11-20 |
| S61 | blog | Kettle, "HTTP Desync Attacks: Request Smuggling Reborn" (PortSwigger Research) | https://portswigger.net/research/http-desync-attacks-request-smuggling-reborn | 2019-08-07 |
| S62 | blog | Kettle, "HTTP/1.1 must die: the desync endgame" (PortSwigger Research) | https://portswigger.net/research/http1-must-die | 2025-08-06, updated 2025-10-17 |
| S63 | standard | RFC 8878, Zstandard Compression and the 'application/zstd' Media Type | https://www.rfc-editor.org/rfc/rfc8878.txt | February 2021 |
| S64 | standard | RFC 9659, Window Sizing for Zstandard Content Encoding (updates RFC 8878) | https://www.rfc-editor.org/rfc/rfc9659.txt | September 2024 |
| S65 | docs | zlib Technical Details | https://zlib.net/zlib_tech.html | fetched 2026-11-20 |
| S66 | docs | Netflix/concurrency-limits README | https://raw.githubusercontent.com/Netflix/concurrency-limits/main/README.md | main, fetched 2026-11-20 |
| S67 | code | Netflix/concurrency-limits `Gradient2Limit.java` | https://raw.githubusercontent.com/Netflix/concurrency-limits/main/concurrency-limits-core/src/main/java/com/netflix/concurrency/limits/limit/Gradient2Limit.java | main, fetched 2026-11-20 |
| S68 | code | Netflix/concurrency-limits `VegasLimit.java` | https://raw.githubusercontent.com/Netflix/concurrency-limits/main/concurrency-limits-core/src/main/java/com/netflix/concurrency/limits/limit/VegasLimit.java | main, fetched 2026-11-20 |
| S69 | code | Netflix/concurrency-limits `AIMDLimit.java` | https://raw.githubusercontent.com/Netflix/concurrency-limits/main/concurrency-limits-core/src/main/java/com/netflix/concurrency/limits/limit/AIMDLimit.java | main, fetched 2026-11-20 |
| S70 | docs | Envoy docs, "Adaptive Concurrency" | https://www.envoyproxy.io/docs/envoy/latest/configuration/http/http_filters/adaptive_concurrency_filter | envoy 1.41.0-dev |
| S71 | code | Envoy `adaptive_concurrency.proto` (v3 API) | https://raw.githubusercontent.com/envoyproxy/envoy/main/api/envoy/extensions/filters/http/adaptive_concurrency/v3/adaptive_concurrency.proto | main, fetched 2026-11-20 |
| S72 | paper | Zhou et al., "Overload Control for Scaling WeChat Microservices" (SoCC'18), arXiv abstract | https://arxiv.org/abs/1806.04075 | v3, 2018-12-24 |
| S73 | paper | same paper, PDF | https://arxiv.org/pdf/1806.04075 | v3 |
| S74 | paper | Cho et al., "Overload Control for µs-scale RPCs with Breakwater" (OSDI'20), abstract page | https://www.usenix.org/conference/osdi20/presentation/cho | November 2020 |
| S75 | standard | RFC 8289, Controlled Delay Active Queue Management (Experimental) | https://www.rfc-editor.org/rfc/rfc8289.txt | January 2018 |
| S76 | paper | Kleinrock, "Internet Congestion Control Using the Power Metric: Keep the Pipe Just Full, But No Fuller" (author's PDF) | https://www.lk.cs.ucla.edu/data/files/Kleinrock/Internet%20Congestion%20Control%20Using%20the%20Power%20Metric-Keep%20the%20Pipe%20Just%20Full%2C%20But%20No%20Fuller%20July%202018.pdf | July 2018 |
| S77 | paper | Cardwell et al., "BBR: Congestion-Based Congestion Control", ACM Queue | https://queue.acm.org/detail.cfm?id=3022184 (publisher returns 403 to curl; saved copy: https://web.archive.org/web/2024id_/https://queue.acm.org/detail.cfm?id=3022184) | vol. 14 issue 5, 2016-12-01 |
| S78 | paper | Yang et al., "FIFO queues are all you need for cache eviction" (SOSP'23), ACM abstract page | https://dl.acm.org/doi/10.1145/3600006.3613147 (saved copy: https://web.archive.org/web/2024id_/https://dl.acm.org/doi/10.1145/3600006.3613147) | October 2023 |
| S79 | paper | same paper, author's PDF | https://jasony.me/publication/sosp23-s3fifo.pdf | October 2023 |
| S80 | paper | Zhang et al., "SIEVE is Simpler than LRU: an Efficient Turn-Key Eviction Algorithm for Web Caches" (NSDI'24), abstract page | https://www.usenix.org/conference/nsdi24/presentation/zhang-yazhuo | April 2024 |
| S81 | blog | SIEVE authors' site, "SIEVE is simpler than LRU" | https://cachemon.github.io/SIEVE-website/blog/2023/12/17/sieve-is-simpler-than-lru/ | 2023-12-17 |
| S82 | paper | SIEVE paper, author's PDF | https://junchengyang.com/publication/nsdi24-SIEVE.pdf | April 2024 |
| S83 | paper | Einziger, Eytan, Friedman, Manes, "Adaptive Software Cache Management" (Middleware'18), ACM abstract page | https://dl.acm.org/doi/10.1145/3274808.3274816 (saved copy: https://web.archive.org/web/2023id_/https://dl.acm.org/doi/10.1145/3274808.3274816) | December 2018 |
| S84 | docs | Caffeine wiki, "Efficiency" | https://raw.githubusercontent.com/wiki/ben-manes/caffeine/Efficiency.md | fetched 2026-11-20 |
| S85 | paper | Megiddo & Modha, "ARC: A Self-Tuning, Low Overhead Replacement Cache" (FAST'03), abstract page | https://www.usenix.org/conference/fast-03/arc-self-tuning-low-overhead-replacement-cache | March 2003 |
| S86 | paper | Dean & Barroso, "The Tail at Scale", Communications of the ACM | https://cacm.acm.org/research/the-tail-at-scale/ (publisher returns 403 to curl; saved copy: https://web.archive.org/web/2024id_/https://cacm.acm.org/research/the-tail-at-scale/) | Vol. 56 No. 2, February 2013 |
| S87 | docs | giltene/wrk2 README | https://raw.githubusercontent.com/giltene/wrk2/master/README.md | master, fetched 2026-11-20 |
| S88 | paper | Kalibera & Jones, "Rigorous Benchmarking in Reasonable Time" (ISMM'13), Kent Academic Repository record | https://kar.kent.ac.uk/33611/ | June 2013 |
| S89 | paper | same paper, PDF | https://kar.kent.ac.uk/33611/45/p63-kaliber.pdf | June 2013 |
| S90 | paper | Daly et al., "Change Point Detection in Software Performance Testing" (ICPE'20), arXiv abstract | https://arxiv.org/abs/2003.00584 | 2020 |
| S91 | paper | Fleming et al., "Hunter: Using Change Point Detection to Hunt for Performance Regressions" (ICPE'23), arXiv abstract | https://arxiv.org/abs/2301.03034 | 2023 |
| S92 | preprint | Carbune et al., "SmartChoices: Augmenting Software with Learned Implementations", arXiv abstract | https://arxiv.org/abs/2304.13033 | v3, 2024-07-08 (v1 2023-04-12) |
| S93 | blog | Engineering at Meta, "Spiral: Self-tuning services via real-time machine learning" (vendor engineering post) | https://engineering.fb.com/2018/06/28/data-infrastructure/spiral-self-tuning-services-via-real-time-machine-learning/ | 2018-06-28 |
| S94 | docs | Claude Code docs, "Extend Claude with skills" | https://code.claude.com/docs/en/skills.md | fetched 2026-11-20 |
| S95 | docs | Claude Code docs, "Create and distribute a plugin marketplace" | https://code.claude.com/docs/en/plugin-marketplaces.md | fetched 2026-11-20 |
| S96 | docs | modelcontextprotocol/mcpb README | https://raw.githubusercontent.com/modelcontextprotocol/mcpb/main/README.md | main (latest release v2.1.2, 2025-12-04), fetched 2026-11-20 |
| S97 | vendor | Postmark, "Information Regarding Malicious "postmark-mcp" Package" (statement of the impersonated vendor) | https://postmarkapp.com/blog/information-regarding-malicious-postmark-mcp-package | 2025-09-25 |
| S98 | vendor | Snyk, "ToxicSkills" study (vendor's own study, vendor blog) | https://snyk.io/blog/toxicskills-malicious-ai-agent-skills-clawhub/ | 2026-02-05 |
| S99 | preprint | Liu et al., "Agent Skills in the Wild: An Empirical Study of Security Vulnerabilities at Scale", arXiv abstract | https://arxiv.org/abs/2601.10338 | 2026-01-15 |
| S100 | standard | CNCF ModelPack model-spec, `docs/spec.md` | https://raw.githubusercontent.com/modelpack/model-spec/main/docs/spec.md | main, fetched 2026-11-20 |

Supplementary lookups used for dates and versions only (no quote taken): GitHub releases API for `opencontainers/image-spec` (v1.1.1, 2025-03-03), `opencontainers/distribution-spec` (v1.1.1, 2025-01-29), `notaryproject/notation` (v1.3.2, 2025-04-27; v2.0.0-alpha.1, 2025-03-13), `modelcontextprotocol/mcpb` (v2.1.2, 2025-12-04), `facebook/zstd` (v1.5.7, 2025-02-19); GitHub contents API for `opencontainers/wg-auth/docs` (lists `USE_CASES.md`, `EXTERNAL_SPECS.md`, `implementations`); all fetched 2026-11-20.

## Findings

### Assumptions in the brief that are outdated on 2026-11-20
- Rekor v2 is GA (2025-10-10) but is not the default: the public-good instance stays on Rekor v1 "for the foreseeable future" (2026-06-28), and a post-quantum redesign with a new bundle format revision was announced 2026-10-02 (F5, F7).
- The first published cosign with the new form on by default is v3.0.1 (2025-10-08); v3.0.0 was never published. Current is v3.1.4 (2026-10-20); v3.1.1 deprecates the old-format flags for removal in v4 (F4).
- SLSA is at v1.2 (Approved); the provenance predicate type is unchanged (F9).
- BuildKit no longer stores attestations only as extra manifests in the index: with OCI media types it writes them as OCI artifacts with `artifactType` and `subject` (F12).
- `actions/attest-build-provenance` v4 is a wrapper over `actions/attest` (F13).
- Amazon ECR now recommends registry-side "managed signing"; Notation is its manual path (F15).
- GitHub's OIDC `sub` claim changes format for repositories created after 2026-07-15 (F19).
- The zstd 8 MB window figure became a MUST for the HTTP `zstd` content coding in RFC 9659, September 2024 (F34).
- "MadeYouReset" exists: CVE-2025-8671, public 2025-08-13 (F28).
- A Rust tar parser differential, CVE-2025-62518 ("TARmageddon"), was published 2025-10-21 (F24).

### A. Supply-chain trust

### F1. A Sigstore bundle is one JSON document holding the signature content and all verification material; in a registry its media type `application/vnd.dev.sigstore.bundle.v0.3+json` is both the layer media type and the manifest `artifactType`.
- Quote: "A Sigstore bundle is everything required to verify a signature on an artifact. This is satisfied by the Verification Material and signature Content." — S1, "Bundle"
- Quote: "The manifest must have an `artifactType` field which identifies the type of the artifact being referenced -- in this case, it's the Sigstore bundle media type." — S2, "Storage / Publishing"
- Conditions/notes: the spec's example manifest has `config` = the OCI empty descriptor (`application/vnd.oci.empty.v1+json`, size 2), one layer of the bundle media type, and a `subject`; the example in S1 shows `"mediaType": "application/vnd.dev.sigstore.bundle.v0.3+json"`. The S1 page is headed "Version 0.3.2": the document version moves while the media type stays `v0.3`.
- Relevance: the registry can recognise signatures and attestations by `artifactType` alone and index them per subject without parsing the bundle.

### F2. Cosign's bundle spec stores the bundle as an OCI 1.1 referrer and tells clients to fall back to the referrers tag schema when the referrers API answers 404.
- Quote: "A `404 Not Found` response indicates that the registry does not support the referrers API and the referrers tag scheme should be used as a fallback:" — S2, "Retrieval"
- Conditions/notes: the fallback tag is `sha256-<hex>` (an image index listing the artifacts), not the legacy `.sig` tag.
- Relevance: serve the referrers API natively and never answer it with 404, so clients do not create fallback tags; still accept pushes of `sha256-<hex>` index tags from old clients.

### F3. The legacy cosign scheme stores a signature at a tag computed from the subject digest: replace `:` with `-`, append `.sig`.
- Quote: "In this scheme, signatures are stored in an OCI registry in a predictable location, addressable by tag. The location of signatures corresponding to a specific object can be computed using the digest of the object." — S3, "Tag-based Discovery"
- Quote: "The **legacy `.att` tag**, which is the default for Cosign v2." — S28, "How It Works"
- Conditions/notes: the rules in S3 are "Replace the `:` character with a `-`" and "Append the `.sig` suffix"; attestations use `.att`.
- Relevance: garbage collection and tag listing must treat `sha256-<hex>.sig` / `.att` tags as attached to a subject, or a tag-retention rule will delete signatures.

### F4. The bundle-as-referrer form is cosign's default from v3 (first published as v3.0.1, 2025-10-08); the old form can still be selected until v4.
- Quote: "Cosign v3 is a minor change from Cosign v2.6.x, with all of the new capabilities of recent releases **on by default**, but will still allow you to disable them if you need the older functionality." — S4, "v3.0.0"
- Quote: "These new features include support for the standardized bundle format (`--new-bundle-fomat`), providing roots of trust for verification and service URLs for signing via one file (`--trusted-root`, `--signing-config`), and container signatures stored as an OCI Image 1.1 referring artifact." — S4, "v3.0.0"
- Quote: "v3.0.1 is an equivalent release to v3.0.0, which was never published due to a failure in our CI workflows." — S4, "v3.0.1"
- Quote: "Starting in Cosign v3, these flags will be on by default, but will still allow you to disable them if you want the old operation." — S5, body
- Quote: "An **OCI 1.1 referrer** holding a Sigstore bundle, which is the default for `cosign attest` since Cosign v3." — S28, "How It Works"
- Quote: "You may continue to use the deprecated flags with Cosign v3.x releases. The deprecated flags will be removed in a future Cosign v4 release." — S6, release notes of v3.1.1 (2026-06-09)
- Conditions/notes: v2.6.0 (2025-09-12) introduced the referrer upload behind `--new-bundle-format`. `CHANGELOG.md` on main stops at v3.0.5 although releases continue to v3.1.4 (S6).
- Relevance: both forms are in the field through 2026; conformance tests must cover a v2 client (tags) and a v3 client (referrers) against the same repository.

### F5. Rekor v2 (tile-backed log) reached GA on 2025-10-10, but the public-good instance keeps Rekor v1 as its default log; v2 is opt-in and its shard URLs rotate.
- Quote: "We are very excited to announce the General Availability of Rekor v2!" — S7, opening
- Quote: "the sigstore.dev public good instance will continue using Rekor v1 as the default log for the foreseeable future." — S8, opening
- Quote: "Even though we’re not enabling signing by default, Rekor v2 is running and available in production today." — S8, "Rekor v2 is Available (and Optimized)"
- Quote: "Rekor v2 does not act as a timestamping authority and Signed Entry Timestamps will not be returned as a response to an uploaded entry." — S12, "4.2 Timestamps"
- Counter: Rust client support for v2 is split across two crates — "Clients known to not have Rekor v2 support at time of writing sigstore/sigstore-rs sigstore/sigstore-ruby" — S8 (a heading followed by its two list items; the same post lists `sigstore/sigstore-rust` as compatible)
- Conditions/notes: S7 gives the first v2 shard as `https://log2025-1.rekor.sigstore.dev` and says it will be turned down when a 2026 instance is deployed.
- Relevance: a verifier in the registry must accept both v1 entries (signed entry timestamp, `integratedTime`) and v2 entries (checkpoint plus inclusion proof, RFC 3161 timestamp), and must take log keys and URLs from the TUF-distributed trusted root, never from a constant.

### F6. The Sigstore client specification states what a verifier must check: a trusted time, the certificate chain at that time, the embedded SCT, the identity policy, the log entry, and the signature over the artifact.
- Quote: "What must be true about the identity in a certificate (whom to trust)." — S11, "4. Verification" (first item of the policy)
- Quote: "If the verification policy uses the Timestamping Service, the Verifier MUST verify the timestamping response using the Timestamping Service root key material" — S11, "4.2.1. Timestamping Service"
- Quote: "For a signature with a given certificate to be considered valid, it must have a timestamp while every certificate in the chain up to the root is valid" — S11, "4.3. Certificate"
- Quote: "the Verifier MUST extract the `SignedCertificateTimestamp` embedded in the leaf certificate, and verify it as in" — S11, "4.3. Certificate" (continues: RFC 6962 §3.2, with the CT log's key)
- Quote: "The Verifier MUST then check the certificate against the verification policy." — S11, "4.3. Certificate"
- Quote: "Verifier MUST ensure that the artifact's digest/algorithm tuple is present in the list of subjects in the in-toto statement." — S11, "4.5. Signature Verification"
- Quote: "Clients that need to verify the inclusion of a given signing event SHOULD do so using using offline verification material provided by the signing client and without performing an online query of the transparency log." — S12, "4.1 Offline Verification"
- Quote: "Commitment by the log to its current state, containing the Merkle tree root hash and signed by Rekor's private key." — S12, terminology table, "Checkpoint"
- Quote: "SCT verification, a proof of inclusion in a certificate transparency log, is now on by default for verifying Fulcio certificates." — S4, "v2.0.0", "Breaking Changes"
- Counter: the log is optional under some policies — "The Verifier MAY choose not to require that signatures are in the Transparency Service. In this case, the Verifier MUST use a timestamp from the Timestamping Service during certificate verification." — S11
- Conditions/notes: S11 §4.3 says the Verifier "SHOULD check the `Issuer` X.509 extension" (OID 1.3.6.1.4.1.57264.1.8) and "will in most cases check the `SubjectAlternativeName` as well"; S11 §4.6 adds that with thresholds "Verifiers MUST ensure that multiple entries in the same log do not both count towards the threshold".
- Relevance: these are the verifier's test cases, one refusal type each: no trusted time, chain invalid at that time, SCT missing or bad, identity or issuer mismatch, inclusion proof or checkpoint bad, subject digest absent from the statement, signature bad.

### F7. Sigstore announced on 2026-10-02 a post-quantum redesign in which the log proof itself is the keyless signature and the bundle format gets a new revision.
- Quote: "The default “keyless” signing flow will truly be “keyless”, without an ephemeral key, short-lived certificate, or timestamp." — S9, "Core Innovations in PQC Sigstore"
- Quote: "Sigstore will leverage c2sp.org/tlog-proof structures in the new bundle format revision, so that tooling can be shared with the transparency community." — S9, "Core Innovations in PQC Sigstore"
- Conditions/notes: a proposal with a design document open for feedback; no dates for deployment are given in the post. ML-DSA is the named algorithm.
- Relevance: treat the bundle media type as versioned and the verifier as a set of per-version verifiers behind one trait; a `v0.3` assumption baked into storage or policy will not survive.

### F8. The public Rekor instance removed rarely used entry types on 2026-11-16, giving parser attack surface as the reason.
- Quote: "Effective immediately, we will disable the upload of a number of Rekor entry types, all of which are unsupported by Sigstore client tooling and have single-digit uploads in a 90-day window." — S10, "Hardening Sigstore Public Infrastructure"
- Quote: "Hardening the infrastructure and minimizing API surface reduces the risk of parsing vulnerabilities and denial-of-serivce attacks." — S10, same section
- Relevance: the same rule for the registry: accept only the artifact and envelope types it can fully parse and test; refuse the rest with a typed error.

### F9. SLSA v1.2 keeps the provenance predicate type `https://slsa.dev/provenance/v1`; the URI carries only the major version.
- Quote: "Important: Always use the above string for predicateType rather than what is in the URL bar. The predicateType URI will always resolve to the latest minor version of this specification." — S13, page introduction (before "Purpose")
- Conditions/notes: the string shown above that sentence is `"predicateType": "https://slsa.dev/provenance/v1"`. The page is marked "Status: Approved".
- Relevance: policy matches on the exact string; minor versions add fields, so the provenance parser must ignore unknown fields rather than refuse them.

### F10. In SLSA provenance the trust root is `runDetails.builder.id`, which names the whole trusted build platform.
- Quote: "The builder.id identifies this platform, representing the transitive closure of all entities that are trusted to faithfully run the build and record the provenance." — S13, "Model"
- Conditions/notes: the schema places it at `runDetails.builder.id`; `builder.version` and `builder.builderDependencies` are optional neighbours.
- Relevance: an admission policy is "signer identity X may speak for builder.id Y"; store both and index attestations by `builder.id`.

### F11. An in-toto Statement binds a predicate to subjects by digest only; `_type` is `https://in-toto.io/Statement/v1`.
- Quote: "The Statement is the middle layer of the attestation, binding it to a particular subject and unambiguously identifying the types of the [Predicate]." — S14, opening
- Quote: "represents a single software artifact. Each element MUST have `digest` set." — S14, "Fields", `subject`
- Conditions/notes: the same field text warns that subjects "are matched purely by digest, regardless of content type" (split across quote-block lines in the source, so not quoted as one passage).
- Relevance: the registry must check that a referrer's OCI `subject` digest appears in the statement's `subject` list; a mismatch is a typed refusal or a flagged record, never silently indexed.

### F12. BuildKit now stores attestations as OCI artifacts (an `artifactType` plus a `subject`) that are also listed in the image index with platform `unknown/unknown`; the older form is selected with `oci-artifact=false`.
- Quote: "BuildKit stores attestations as OCI artifacts when OCI media types are enabled. Set the image exporter option `oci-artifact=false` to use the legacy attestation image manifest format." — S15, opening
- Quote: "When OCI artifact storage is enabled, the manifest `artifactType` is set to `application/vnd.docker.attestation.manifest.v1+json`, the `subject` descriptor points to the target image manifest, and the `config` descriptor uses the OCI empty JSON descriptor." — S15, "Attestation Manifest"
- Quote: "To prevent container runtimes from accidentally pulling or running the image described in the manifest, the `platform` property of the attestation manifest will be set to `unknown/unknown`, as follows:" — S15, "Attestation Manifest Descriptor"
- Conditions/notes: layers are `application/vnd.in-toto+json` with an `in-toto.io/predicate-type` annotation; index descriptors carry `vnd.docker.reference.type: attestation-manifest` and `vnd.docker.reference.digest`. The BuildKit version that made this the default was not found in the page.
- Relevance: one attestation manifest can now be reached two ways (index child and referrer). Reference counting, deletion and replication must count it once and must not treat an `unknown/unknown` entry as a runnable platform.

### F13. GitHub's attest actions sign an in-toto statement with a Sigstore certificate and upload it to GitHub's attestations API; pushing it to the image's registry is an option that is off by default.
- Quote: "Once the attestation has been created and signed, it will be uploaded to the GH attestations API and associated with the repository from which the workflow was initiated." — S16, opening
- Quote: "If you want to publish the attestation to the container registry with the `push-to-registry` option, it is important that the `subject-name` specify the fully-qualified image name" — S16, "Container Image"
- Quote: "Existing applications may continue to use the `attest-build-provenance` action, but new implementations should use `actions/attest` instead." — S17, "Usage"
- Quote: "Private repositories that generate artifact attestations use GitHub's Sigstore instance. GitHub's Sigstore instance uses the same codebase as the Sigstore Public Good Instance, but it does not have a transparency log and only federates with GitHub Actions." — S18, "How GitHub generates artifact attestations"
- Quote: "Artifact attestations by itself provides SLSA v1.0 Build Level 2." — S18, "SLSA levels for artifact attestations"
- Conditions/notes: the manifest layout GitHub writes to the registry is not stated in S16–S18 (see open questions).
- Relevance: attestations from private GitHub repositories have no transparency-log entry, so a policy that requires a log proof must be selectable per trust root, not global.

### F14. A Notary Project signature is an OCI manifest with `artifactType: application/vnd.cncf.notary.signature`, a `subject`, and one JWS or COSE envelope layer; clients find it through the referrers API.
- Quote: "The signature manifest uses either the `artifactType` property or `config.mediaType` to indicate it's a Notary Project signature." — S19, "OCI Signatures"
- Quote: "**`artifactType`** (*string*): This OPTIONAL property, if present, MUST be `application/vnd.cncf.notary.signature`." — S19, "OCI Signatures" (manifest properties)
- Quote: "The client should be able to discover all the signatures belonging to an artifact (such as image manifest) by using [OCI Distribution Referrers API][oci-distribution-referrers]." — S19, "OCI Signature Discovery"
- Conditions/notes: envelope layer media types are `application/jose+json` and `application/cose`; the payload type is `application/vnd.cncf.notary.payload.v1+json`. Older signers (spec v1.1.0 or earlier) put the type in `config.mediaType` instead.
- Relevance: the referrers `artifactType` filter must fall back to `config.mediaType` when `artifactType` is absent, as the distribution spec requires, or old Notation signatures become invisible.

### F15. Adoption signals: ECR signs through AWS Signer, with Notation as the client for manual signing and a registry-side "managed signing" mode now recommended; ACR documents Notation with Key Vault.
- Quote: "Amazon ECR integrates with AWS Signer to provide two ways for you to sign your container images: managed signing (automatic, recommended) and manual signing" — S20, opening
- Quote: "Amazon ECR managed signing automatically signs your container images by generating cryptographic signatures using AWS Signer when images are pushed to Amazon ECR." — S21, opening
- Quote: "Notation is an open-source supply-chain security tool developed by the Notary Project community and backed by Microsoft." — S22, opening
- Conditions/notes: ECR managed signing allows "up to 10 per registry" signing rules (S21). Notation's latest stable release is v1.3.2 of 2025-04-27, with v2.0.0-alpha.1 of 2025-03-13 (GitHub releases API, fetched 2026-11-20): no stable release in the 18 months before this note.
- Relevance: signing at push inside the registry is now a shipped competitor feature; Reliquary's design should decide whether to offer it and under which identity.

### F16. An OSV record states affected versions as typed ranges (`SEMVER`, `ECOSYSTEM`, `GIT`) of events (`introduced`, `fixed`, `last_affected`, `limit`); only `SEMVER` can be evaluated without ecosystem logic or the repository.
- Quote: "In the `ranges` field, the `type` field is required. It specifies the type of version range being recorded and defines the interpretation of the `events` object's `introduced`, `fixed`, and any type-specific fields." — S23, "affected[].ranges[].type field"
- Quote: "`ECOSYSTEM`: The versions `introduced` and `fixed` are arbitrary, uninterpreted strings specific to the package ecosystem, which does not conform to SemVer 2.0’s version ordering." — S23, same section
- Quote: "`GIT` range inclusion queries cannot be answered without access to a copy of the underlying Git repository." — S23, same section
- Conditions/notes: the ecosystem table includes `crates.io`, `npm`, `PyPI`, `Go`, `Maven`, `Debian`, `Alpine`, `GitHub Actions` and `Docker Hardened Images`; `introduced: "0"` sorts before every version.
- Relevance: matching SBOM components to OSV needs one version comparator per ecosystem, each a hostile-input parser with its own tests; where no comparator exists the answer is "unknown", not "not affected".

### F17. OpenVEX v0.2.0 has four statuses, and `not_affected` is only valid with a justification label or an impact statement.
- Quote: "No remediation is required regarding this vulnerability. A `not_affected` status required the addition of a `justification` to the statement." — S24, "Status Labels"
- Quote: "For statements conveying a `not_affected` status, a VEX statement MUST include either a status justification or an impact_statement informing why the product is not affected by the vulnerability." — S24, statement fields table
- Conditions/notes: the statuses are `not_affected`, `affected`, `fixed`, `under_investigation`; justification labels include `component_not_present`, `vulnerable_code_not_in_execute_path`, `vulnerable_code_cannot_be_controlled_by_adversary`, `inline_mitigations_already_exist`.
- Relevance: a policy engine must refuse a `not_affected` statement that has neither field, and must record who signed the VEX: an exception is only as good as its signer.

### F18. `application/spdx+json` and `application/vnd.cyclonedx+json` are IANA-registered, but attestation tooling attaches SBOMs under one generic artifact type and distinguishes them by a predicate-type annotation.
- Quote: "Published specification: Current versions of the specification are available at https://spdx.github.io/spdx-spec/." — S25, registration for `application/spdx+json`
- Quote: "Published specification: The specification can be found on the CycloneDX website, https://cyclonedx.org/" — S26, registration for `application/vnd.cyclonedx+json`
- Quote: "The following media types are officially registered with IANA:" — S27, "Media Types" (the list that follows includes `application/vnd.cyclonedx+json`)
- Counter: SBOM, provenance and VEX referrers often share one `artifactType` — "Since a single artifact type covers every kind of attestation, an image usually exposes SBOM and provenance referrers next to the VEX one." — S28
- Quote: "Trivy uses the `in-toto.io/predicate-type` annotation to pick the VEX attestation without downloading the others, and falls back to inspecting the attestation itself when a referrer carries no annotation." — S28, "How It Works"
- Conditions/notes: S28 names the generic type `application/vnd.in-toto+json` ("as published by tools such as Docker Scout"); BuildKit's SBOM attestation uses predicate type `https://spdx.dev/Document` (S15). How often raw `application/spdx+json` is used as a referrer `artifactType` was not established from a primary source.
- Relevance: filtering referrers by `artifactType` is not enough to find "the SBOM"; the registry's index should also record the `in-toto.io/predicate-type` annotation, and the predicate type read from the statement when the annotation is absent.

### F19. With OIDC workload identity a CI job presents a short-lived, per-job token whose claims the receiving side matches against a preconfigured trust rule; no long-lived secret is stored.
- Quote: "Every time your job runs, GitHub's OIDC provider auto-generates an OIDC token. This token contains multiple claims to establish a security-hardened and verifiable identity about the specific workflow that is trying to authenticate." — S29, "How OIDC integrates with GitHub Actions"
- Quote: "OpenID Connect (OIDC) allows your GitHub Actions workflows to access resources in Amazon Web Services (AWS), without needing to store the AWS credentials as long-lived GitHub secrets." — S30, "Overview"
- Quote: "Repositories created after July 15, 2026 use an immutable default subject format that includes owner and repository IDs (not available on GitHub Enterprise Server)." — S29, note under the example token
- Conditions/notes: the documented example subject is `repo:octo-org/octo-repo:environment:prod`; the AWS audience is `sts.amazonaws.com`.
- Relevance: a registry that accepts federated tokens directly must match on `iss`, `aud` and `sub` exactly and must handle both subject formats; a rule written against the name-based `sub` will not match new repositories.

### F20. The OCI has an Auth working group whose mission is a standard for registry authentication, but it has published no specification yet.
- Quote: "This working group will define a standard to be supported by OCI compatible registries and clients when support for authentication and authorization is required." — S31, "Mission Statement"
- Quote: "Implementations of this in registries and clients have roughly followed standards set by Docker and have mostly focused on compatibility with specific implementations of registries or clients." — S31, "Mission Statement"
- Conditions/notes: on 2026-11-20 the repository's `docs/` holds only `USE_CASES.md`, `EXTERNAL_SPECS.md` and `implementations`; last commit 2026-01-05 (GitHub API).
- Relevance: the Docker token flow stays the compatibility target; keep the auth challenge logic behind a seam so an OCI standard can be added without touching the data path.

### B. Hostile input

### F21. Fifield's zip bomb overlaps file entries on one compressed kernel, so output grows quadratically with input: 42 kB to 5.5 GB, 10 MB to 281 TB, and 46 MB to 4.5 PB with Zip64, without recursion.
- Quote: "The output size increases quadratically in the input size, reaching a compression ratio of over 28 million (10 MB → 281 TB) at the limits of the zip format." — S32, "Summary"
- Quote: "It works by overlapping files inside the zip container, in order to reference a "kernel" of highly compressed data in multiple files, without making multiple copies of it." — S32, introduction
- Quote: "Furthermore, any such detection logic must be implemented inside the parser itself, not as a separate prefilter." — S32, discussion of defences
- Quote: "rather you should impose external limits on time and other resources so that the parser cannot misbehave too much, no matter what kind of attack it faces." — S32, discussion of defences
- Conditions/notes: plain DEFLATE "cannot achieve a compression ratio greater than 1032" (S32), which is why the construction matters; the page also warns that summing metadata sizes does not predict output because stored sizes may be false.
- Relevance: MCPB bundles and plugin archives are ZIP: extraction needs a counted output budget and an entry budget enforced during inflation, plus refusal of overlapping entries; the declared sizes are never the bound.

### F22. A 2025 differential study of 50 ZIP parsers in 19 languages found 14 ambiguity types in three categories, with 1221 of 1225 parser pairs disagreeing on at least one.
- Quote: "we developed a differential fuzzer ZipDiff and systematically identified parsing inconsistencies between 50 ZIP parsers across 19 programming languages." — S33, abstract
- Quote: "We summarize our findings as 14 distinct parsing ambiguity types in three categories with detailed analysis, systematizing current knowledge and uncovering 10 types of new parsing ambiguities." — S33, abstract
- Quote: "1221 out of a total of 1225 pairs of parsers are affected by at least one type of ambiguity, demonstrating the prevalence of inconsistencies across ZIP parsers." — S34, PDF p.7
- Quote: "14 distinct types and group them into three major categories: redundant metadata, file path processing, and ZIP structure positioning, as detailed in the following subsections." — S34, PDF p.7
- Conditions/notes: the paper proposes seven mitigations (PDF p.13), among them using one parser everywhere, normalising by extract-and-repack, and rejecting archives with ambiguous patterns such as unused bytes or conflicting central-directory and local-header fields. Exploits shown include VS Code extension impersonation and a signed nested JAR passing Spring Boot's verification.
- Relevance: what the registry scans or signs must be exactly what the client later extracts. Either refuse any ZIP that is not canonical (single EOCD, no gaps, central and local headers equal, no duplicate names), or re-pack and sign the re-packed bytes. Test with the paper's ambiguity classes.

### F23. Zip Slip (Snyk, public 2018-06-05) is path traversal through entry names during extraction and applies to tar and other archive formats as well.
- Quote: "It was discovered and responsibly disclosed by the Snyk Security team ahead of a public disclosure on 5th June 2018, and affects thousands of projects" — S35, opening
- Quote: "The vulnerability is exploited using a specially crafted archive that holds directory traversal filenames (e.g. `../../evil.sh`)." — S35, opening
- Relevance: every entry path is resolved against the destination root and refused if it leaves it; test names with `..`, absolute paths, backslashes, drive letters and NUL.

### F24. Archive extraction has repeatedly escaped its destination through symlinks and hard links in container tooling and in Rust and Node tar libraries; two further parser bugs concern header size.
- Quote: "the API endpoints behind the 'docker cp' command are vulnerable to a symlink-exchange attack with Directory Traversal, giving attackers arbitrary read-write access to the host filesystem with root privileges" — S37, CVE-2018-15664 (Docker)
- Quote: "pulling and extracting a specially-crafted container image can result in Unix file permission changes for existing files in the host’s filesystem" — S38, CVE-2021-32760 (containerd)
- Quote: "allows attackers to overwrite arbitrary host paths via a crafted image that causes symlink traversal when "umoci unpack" or "umoci raw unpack" is used." — S39, CVE-2021-29136 (umoci)
- Quote: "An issue was discovered in the tar crate before 0.4.16 for Rust. Arbitrary file overwrite can occur via a symlink or hardlink in a TAR archive." — S40, CVE-2018-20990 (Rust `tar`)
- Quote: "When symlinks are present in a TAR archive, extraction can create arbitrary directories via .. traversal." — S41, CVE-2021-38511 (Rust `tar` before 0.4.36)
- Quote: "The untar function can both create and follow symbolic links." — S42, CVE-2019-1002101 (kubectl cp)
- Quote: "This logic was insufficient when extracting tar files that contained both a directory and a symlink with the same name as the directory" — S43, CVE-2021-37701 (node-tar)
- Quote: "the parser incorrectly advances stream position based on ustar header size (often zero) instead of the PAX-specified size, causing it to interpret file content as legitimate tar headers." — S44, CVE-2025-62518 (astral-tokio-tar before 0.5.6)
- Quote: "While the active forks have been successfully patched (see also Astral Security Advisory), this disclosure highlights a major systemic challenge: the highly downloaded tokio-tar remains unpatched." — S45, opening
- Quote: "Reader.Read does not set a limit on the maximum size of file headers. A maliciously crafted archive could cause Read to allocate unbounded amounts of memory" — S46, CVE-2022-2879 (Go `archive/tar`)
- Conditions/notes: the Go fix "limits the maximum size of header blocks to 1 MiB" (S46). CVE-2018-15664 is a time-of-check race, not a parsing bug: its record says the daemon "does not do archive operations on a frozen filesystem (or from within a chroot)".
- Relevance: Reliquary's layer and bundle readers need (a) a tar parser tested against PAX/ustar size disagreement, (b) extraction that never follows a link created by the archive (resolve beneath a root handle), (c) a cap on header and PAX record size, (d) a named, maintained tar crate, checked against CVE-2025-62518.

### F25. The OCI layer specification defines whiteouts and duplicate-entry rules but says nothing about `..`, absolute paths or links that leave the root; confinement is left to the implementation.
- Quote: "Layer Changesets for the [media type](media-types.md) `application/vnd.oci.image.layer.v1.tar` MUST NOT include duplicate entries for file paths in the resulting [tar archive][tar-archive]." — S36, "Distributable Format"
- Quote: "Once a whiteout is applied, the whiteout itself MUST also be hidden." — S36, "Whiteouts"
- Quote: "Whiteout files MUST only apply to resources in lower/parent layers." — S36, "Whiteouts"
- Quote: "An opaque whiteout entry is a file with the name `.wh..wh..opq` indicating that all siblings are hidden in the lower layer." — S36, "Opaque Whiteout"
- Quote: "Extracting a layer with hardlink references to files outside of the layer may fail." — S36, "Hardlinks"
- Quote: "If the entry and the existing path are both directories, then the existing path's attributes MUST be replaced by those of the entry in the changeset." — S36, "Changeset over existing files"
- Conditions/notes: a search of `layer.md` at v1.1.1 for "travers", "absolute" and "escap" finds nothing; every `..` in the file is inside the opaque-whiteout name `.wh..wh..opq`. The duplicate-entry rule is a requirement on producers; the text does not say what a consumer does with a layer that violates it.
- Relevance: where the registry unpacks or indexes layers (lazy-pull tables of contents, scanning), path confinement and duplicate handling are its own stated rules with their own tests; the spec will not supply them.

### F26. Rapid Reset (CVE-2023-44487): a client opens streams and cancels them at once, so the concurrent-stream limit never binds; peaks reported were 201 million requests per second (Cloudflare) and 398 million (Google).
- Quote: "The HTTP/2 protocol allows a denial of service (server resource consumption) because request cancellation can reset many streams quickly, as exploited in the wild in August through October 2023." — S47, description
- Quote: "eventually peaked just above 201 million requests per second. This was nearly 3x bigger than our previous biggest attack on record." — S48, introduction
- Quote: "with the largest attack surpassing 398 million requests per second." — S49, introduction
- Quote: "instead the entire TCP connection needs to be closed when abuse is detected." — S49, "A multifaceted approach to mitigations"
- Quote: "For example, if a connection has more than 100 requests with more than 50% of the given requests canceled, it could be a candidate for a mitigation response." — S49, same section
- Quote: "we recommend that HTTP/2 servers should close connections that exceed the concurrent stream limit." — S49, same section
- Conditions/notes: Cloudflare says the attack came from "a botnet of merely 20,000 machines" (S48). Google warns that a rule on the rate of inbound RST_STREAM alone can be bypassed by batching, and that the graceful two-step GOAWAY "should not be used for building mitigations".
- Relevance: per-connection accounting of work started against work completed, with the connection closed (a GOAWAY that stops new streams at once) when the cancelled share passes a bound. The bound is a constant to derive from the measured cancel rate of real clients.

### F27. CONTINUATION flood (Nowotarski, 2024-04-03): a header block that never sets END_HEADERS makes the server read, decode or buffer headers without end, and the request never reaches the access log.
- Quote: "What if a client starts a new HTTP/2 stream and sends HEADERS and CONTINUATION frames but END_HEADERS flag is never set? This would create an infinite stream of headers that HTTP/2 server would need to parse and store in memory." — S50, technical details
- Quote: "Headers from CONTINUATION frames are stored in memory but there is a headers list size limit. At the same time, there is no headers timeout." — S50, list of outcomes
- Quote: "Many HTTP/2 implementations do not properly limit or sanitize the amount of CONTINUATION frames sent within a single stream." — S51, "Overview"
- Conditions/notes: in Go's case the decoder stopped emitting headers at the limit but kept decoding, so CPU was consumed; S51's solution text gives no single fix, only that analysis of raw traffic may be needed.
- Relevance: bound three things for one header block: total encoded bytes, frame count, and time to END_HEADERS. Exceeding any closes the connection, and the refusal is counted in a metric because no request log line will exist.

### F28. MadeYouReset (CVE-2025-8671, public 2025-08-13): the client provokes the server into resetting streams with invalid frames; the stream counts as closed while the backend keeps working, so the Rapid Reset mitigation of counting client resets does not see it.
- Quote: "Streams reset by the server are considered closed at the protocol level, even though backend processing continues. This allows a client to cause the server to handle an unbounded number of concurrent streams on a single connection." — S52, description
- Quote: "CERT/CC recommends that vendors who use HTTP/2 in their products review their implementation and limit the number/rate of RST_STREAMs sent from the server." — S53, "Solution"
- Quote: "Why does it matter? Because closed streams are not counted toward the MAX_CONCURRENT_STREAMS limit." — S54, "The Rapid Reset Vulnerability"
- Conditions/notes: triggers are "malformed frames or flow control errors" (S52).
- Relevance: the concurrency bound must be on work in flight, released only when the handler has stopped, not on protocol stream state; and server-sent resets per connection need a budget.

### F29. The 2019 family (CVE-2019-9511 to 9518) abuses frames that oblige the peer to answer or queue: pings, SETTINGS, resets, empty frames, tiny windows.
- Quote: "The attacker sends continual pings to an HTTP/2 peer, causing the peer to build an internal queue of responses." — S55, CVE-2019-9512 "Ping Flood"
- Quote: "Since the RFC requires that the peer reply with one acknowledgement per SETTINGS frame, an empty SETTINGS frame is almost equivalent in behavior to a ping." — S55, CVE-2019-9515 "Settings Flood"
- Quote: "Implementations SHOULD track the use of these features and set limits on their use. An endpoint MAY treat activity that is suspicious as a connection error (Section 5.4.1) of type ENHANCE_YOUR_CALM." — S56, §10.5 "Denial-of-Service Considerations"
- Conditions/notes: the advisory's own workaround is "an immediate workaround is to disable HTTP/2 support"; it also names Data Dribble (9511), Resource Loop (9513), Reset Flood (9514), 0-Length Headers Leak (9516), Internal Data Buffering (9517) and Empty Frames Flood (9518).
- Relevance: the outbound control-frame queue is a bounded queue with a stated overflow rule (close with ENHANCE_YOUR_CALM); the response path must respect TCP backpressure before buffering (9517).

### F30. Header compression lets a small request expand: the 2016 HPACK bomb turned a 4 KB header referenced 16K times into 64 MB per stream, and RFC 9113 notes there is no hard limit on a field block.
- Quote: "By sending a single header of size 4KB and then sending a request with 16K references to this one header, the request is decompressed to 64MB on the server side." — S58, PDF p.18, "HPACK Bomb"
- Quote: "Since there is no hard limit to the size of a field block, some endpoints could be forced to commit a large amount of available memory for field blocks." — S56, §10.5.1 "Limits on Field Block Size"
- Quote: "A server that receives a larger field block than it is willing to handle can send an HTTP 431 (Request Header Fields Too Large) status code [RFC6585]." — S56, §10.5.1
- Quote: "The field block MUST be processed to ensure a consistent connection state, unless the connection is closed." — S56, §10.5.1
- Quote: "The amount of memory used by the compressor is limited by the protocol using HPACK through the definition of the maximum size of the dynamic table." — S57, §7.3 "Memory Consumption"
- Conditions/notes: `SETTINGS_MAX_HEADER_LIST_SIZE` is advisory and "The initial value of this setting is unlimited" (S56 §6.5.2); `SETTINGS_HEADER_TABLE_SIZE` starts at 4,096 octets.
- Relevance: the bound is on decoded header-list bytes, counted while decoding, and the choice on overflow is 431 with the block still decoded, or closing the connection. Continuing to decode an unbounded block is the CONTINUATION flood (F27), so the decode itself needs the byte bound.

### F31. PortSwigger names the classic HTTP/1.1 smuggling classes by which header each hop trusts: CL.TE, TE.CL and TE.TE.
- Quote: "CL.TE: the front-end server uses the Content-Length header and the back-end server uses the Transfer-Encoding header." — S60, "How to perform an HTTP request smuggling attack"
- Quote: "TE.TE: the front-end and back-end servers both support the Transfer-Encoding header, but one of the servers can be induced not to process it by obfuscating the header in some way." — S60, same section
- Quote: "you can resolve all variants of this vulnerability by configuring the front-end server to exclusively use HTTP/2 to communicate to back-end systems, or by disabling back-end connection reuse entirely." — S61, "Defence"
- Relevance: a registry sits behind load balancers and in front of blob stores. Its HTTP/1.1 parser must reject what any neighbour might read differently, and it should speak HTTP/2 to its own back ends.

### F32. RFC 9112 turns the smuggling lessons into requirements: 400 for whitespace before the colon, 400 and close for un-framable requests, obs-fold rejected or replaced, bare CR invalid; a lone LF, however, MAY be accepted.
- Quote: "A server MUST reject, with a response status code of 400 (Bad Request), any received request message that contains whitespace between a header field name and colon." — S59, §5.1 "Field Line Parsing"
- Quote: "If a message is received with both a Transfer-Encoding and a Content-Length header field, the Transfer-Encoding overrides the Content-Length. Such a message might indicate an attempt to perform request smuggling (Section 11.2) or response splitting (Section 11.1) and ought to be handled as an error." — S59, §6.3 "Message Body Length", item 3
- Quote: "If a Transfer-Encoding header field is present in a request and the chunked transfer coding is not the final encoding, the message body length cannot be determined reliably; the server MUST respond with the 400 (Bad Request) status code and then close the connection." — S59, §6.3, item 4
- Quote: "If the unrecoverable error is in a request message, the server MUST respond with a 400 (Bad Request) status code and then close the connection." — S59, §6.3, item 5 (invalid Content-Length)
- Quote: "A server that receives an obs-fold in a request message that is not within a "message/http" container MUST either reject the message by sending a 400 (Bad Request)" — S59, §5.2 "Obsolete Line Folding" (the alternative allowed is replacing each obs-fold with SP)
- Quote: "A recipient of such a bare CR MUST consider that element to be invalid or replace each bare CR with SP before processing the element or forwarding the message." — S59, §2.2 "Message Parsing"
- Quote: "This specification has introduced new requirements on request parsing, particularly with regard to message framing in Section 6.3, to reduce the effectiveness of request smuggling." — S59, §11.2 "Request Smuggling"
- Counter: bare LF is not something the RFC makes a server reject — "Although the line terminator for the start-line and fields is the sequence CRLF, a recipient MAY recognize a single LF as a line terminator and ignore any preceding CR." — S59
- Conditions/notes: a request with both Content-Length and Transfer-Encoding "ought to be handled as an error", and §6.1 says a server "MAY reject" it; neither is a MUST. §7.1 asks recipients to guard chunk-size parsing against integer overflow.
- Relevance: choose the strict side wherever the RFC offers a choice: refuse CL together with TE, refuse obs-fold, refuse bare CR and bare LF, refuse duplicate or non-numeric Content-Length. Each is a typed 400 followed by connection close, and each is a parser test.

### F33. PortSwigger's 2025 research argues that patching individual HTTP/1.1 parser differences has not ended desync attacks.
- Quote: "Upstream HTTP/1.1 is inherently insecure and regularly exposes millions of websites to hostile takeover. Six years of attempted mitigations have hidden the issue, but failed to fix it." — S62, abstract
- Conditions/notes: the paper introduces `0.CL` and `Expect`-based desync classes with case studies on Akamai, Cloudflare and Netlify, and reports "over $200,000 in bug bounties in a two-week period".
- Relevance: add `Expect: 100-continue` handling and zero-length-body cases to the HTTP/1.1 conformance tests; uploads use `Expect` in real clients.

### F34. A zstd frame may declare a window up to 3.75 TB; decoders may refuse beyond their own limit, 8 MB is the interoperability figure, and RFC 9659 makes 8 MB mandatory for the HTTP `zstd` content coding.
- Quote: "The minimum Window_Size is 1 KB. The maximum Window_Size is (1<<41) + 7*(1<<38) bytes, which is 3.75 TB." — S63, §3.1.1.1.2 "Window Descriptor"
- Quote: "In order to protect decoders from unreasonable memory requirements, a decoder is allowed to reject a compressed frame that requests a memory size beyond the decoder's authorized range." — S63, §3.1.1.1.2
- Quote: "For improved interoperability, it's recommended for decoders to support values of Window_Size up to 8 MB and for encoders not to generate frames requiring a Window_Size larger than 8 MB." — S63, §3.1.1.1.2
- Quote: "decoders MUST support a Window_Size of up to and including 8 MB, and encoders MUST NOT generate frames requiring a Window_Size larger than 8 MB" — S64, §3 "Window Size"
- Counter: the 8 MB rule does not bind an attacker — "Note that decoders still need to take into account that they can receive oversized frames that do not follow the window size limit" — S64
- Conditions/notes: RFC 9659 applies to HTTP content coding, not to `application/vnd.oci.image.layer.v1.tar+zstd` layers; no OCI text fixing a window limit for layers was looked for here. Current zstd release is v1.5.7 (2025-02-19).
- Relevance: the decoder is given a maximum window before it reads a frame and refuses larger ones with a typed error; the limit for layers is a stated constant (8 MB is the cited figure; anything larger needs its own evidence).

### F35. DEFLATE cannot expand by more than 1032:1 per stream, so a gzip layer's output is bounded by its input size times 1032, and nesting multiplies it.
- Quote: "that the theoretical limit for the zlib format (as opposed to its implementation in the currently available sources) is 1032:1." — S65, "Maximum Compression Factor"
- Quote: "cannot achieve a compression ratio greater than 1032." — S32, introduction
- Conditions/notes: the derivation in S65 is that one length/distance pair yields at most 258 bytes from two bits.
- Relevance: a derived bound rather than a chosen one: output budget = min(declared uncompressed size, 1032 × compressed size) for a single gzip member, with nested archives charged to the same budget.

### C. Replacing constants

### F36. Netflix's concurrency-limits replaces a load-tested RPS limit with a concurrency limit estimated from latency, borrowing TCP congestion control; its algorithms still ship fixed defaults.
- Quote: "Java Library that implements and integrates concepts from TCP congestion control to auto-detect concurrency limits for services in order to achieve optimal throughput with optimal latency." — S66, "Overview"
- Quote: "RPS limits are then set somewhere below this tipping point (say 75% of this value) and enforced via a token bucket. However, in large distributed systems that auto-scale this value quickly goes out of date and the service falls over by becoming non-responsive as it is unable to gracefully shed excess load." — S66, "Background"
- Quote: "At the end of each sampling window the limit is increased by 1 if the queue is less than alpha (typically a value between 2-3) or decreased by 1 if the queue is greater than beta (typically a value between 4-6 requests)" — S66, "Vegas"
- Quote: "This algorithm attempts to address bias and drift when using minimum latency measurements." — S66, "Gradient2"
- Quote: "private double backoffRatio = 0.9;" — S69, `AIMDLimit.Builder`
- Counter: a minimum-latency baseline sheds too much under bursty RPC — "in an bias towards an impractically low base RTT resulting in excessive load shedding" — S67 (class comment; the sentence begins "We have also found that using minimum can result")
- Conditions/notes: Vegas estimates the queue as `L * (1 - minRTT/sampleRtt)`. Code defaults: Gradient2 initial 20, min 20, max 200, smoothing 0.2, queue size 4, long window 600, RTT tolerance 1.5 (S67); Vegas initial 20, max 1000, alpha `3 * log10(limit)`, beta `6 * log10(limit)`, probe multiplier 30 (S68); AIMD min 20, max 200, timeout 5 s (S69). The README covers Vegas and Gradient2; AIMD is described only in code.
- Relevance: copy the idea (limit concurrency, estimate it from latency), not the numbers: each default above is a constant the registry would have to measure or derive, and the README's "typically" is not evidence.

### F37. Envoy's adaptive concurrency filter measures minRTT by periodically pinning concurrency to a small value, then sets limit = gradient × limit + headroom with gradient = (minRTT + buffer) / sampleRTT.
- Quote: "The minRTT is periodically measured by pinning the concurrency limit to the configured min_concurrency and measuring the latency under these ideal conditions." — S70, "Calculating the minRTT"
- Quote: "Because the headroom value is so necessary to the proper function for the gradient controller, the headroom value is unconfigurable and pinned to the square-root of the concurrency limit." — S70, "Concurrency Limit Headroom"
- Quote: "// The concurrency limit set while measuring the minRTT. Defaults to 3." — S71, `MinimumRTTCalculationParams.min_concurrency`
- Counter: the measurement itself costs availability — "It is possible that there is a noticeable increase in request 503s during the minRTT measurement window because of the potentially significant drop in the concurrency limit." — S70
- Conditions/notes: defaults in S71: max limit 1000, request count per window 50, jitter 15%, buffer 25%, sample percentile p50. S70 also says the filter needs "full control over request concurrency" for the cluster.
- Relevance: a probe that deliberately starves the service is the cost of a clean baseline; a registry can instead take its no-queue service time from requests that found the queue empty, which needs no probe window. That is a design hypothesis to test, not a finding.

### F38. DAGOR (WeChat) detects overload from average queuing time, not response time or CPU, and sheds load by priority across services; its thresholds are empirical.
- Quote: "each microservice monitors its load status in real time and triggers load shedding in a collaborative manner among its relevant services when overload is detected." — S72, abstract
- Quote: "DAGOR has been used in the WeChat backend for five years. Experimental results show that DAGOR can benefit high success rate of service even when the system is experiencing overload, while ensuring fairness in the overload control." — S72, abstract
- Quote: "given the default timeout of each service task being 500 ms in WeChat, the threshold of the average request queuing time to indicate server overload is set to 20 ms." — S73, PDF p.5, §4.1
- Conditions/notes: monitoring refreshes "every second or every 2000 requests" (S73, PDF p.5).
- Relevance: queuing time is the signal to copy; the 20 ms threshold is tied to WeChat's 500 ms timeout (4%), which suggests deriving the threshold from the request deadline rather than fixing it.

### F39. Breakwater issues credits from server-side queueing delay and recovers from a demand spike to 1.4x capacity in under 20 ms.
- Quote: "When clients’ demand spikes unexpectedly to 1.4x capacity, Breakwater converges to stable performance in less than 20 ms with no congestion collapse while DAGOR and SEDA take 500 ms and 1.58 s to recover from congestion collapse, respectively." — S74, abstract
- Conditions/notes: evaluated on microsecond-scale RPCs; the abstract gives no parameter values.
- Relevance: server-driven admission (credits) avoids paying for requests that will be dropped; relevant to chunked uploads, where the registry can grant or withhold the next chunk.

### F40. CoDel controls a queue with the minimum sojourn time over an interval, and the RFC says it needs no tuning because its target is derived (5–10% of the RTT) and its one setting, the interval, is a worst-case RTT.
- Quote: "CoDel consists of an estimator, a setpoint, and a control loop. It requires no configuration in normal Internet deployments." — S75, abstract
- Quote: "Make AQM parameterless for normal operation, with no knobs for operators, users, or implementers to adjust." — S75, §1 "Introduction"
- Quote: "The only setting CoDel requires is the INTERVAL value, and as 100 ms satisfies that definition for normal Internet usage, CoDel can be parameter-free for consumer use." — S75, §4 "Overview of the CoDel AQM"
- Quote: "TARGET SHOULD be set to 5 ms for normal Internet traffic." — S75, §4.3 "Setting TARGET"
- Quote: "The calculations of Section 3.2 show that the best TARGET value is 5-10% of the RTT, with the low end of 5% preferred." — S75, §4.3
- Counter: the 100 ms is an assumption about the path and is replaced by measurement where the RTT can be observed — "Being located in the server makes it possible to monitor the actual RTT to use as CoDel's interval rather than making a "best guess" of RTT." — S75 (Appendix A, "Applying CoDel in the Data Center")
- Relevance: the pattern to copy for request queues: one measured quantity (the round trip or service time), the target as a stated fraction of it, and the fraction cited to an analysis. In a server the interval is measured, as Appendix A does.

### F41. Kleinrock's power is throughput divided by delay (in his later general form, goodness divided by badness), and its maximum marks the knee of the load curve.
- Quote: "Kleinrock developed the analytic machinery to do this using a quantity he called "power", which is the ratio of a normalized throughput to a normalized delay [KLEIN81]." — S75, §3.2 "Target Setpoint"
- Quote: "a performance metric, called Power, which we will use to mathematically (and therefore precisely) identify the knee of the curve." — S76, PDF p.11, §5 "Power Functions"
- Conditions/notes: S76 defines P(G) = G / B(G) (equation 5.1).
- Relevance: gives a concurrency limiter an objective that needs no threshold: move toward the load at which measured throughput/latency peaks.

### F42. BBR estimates the two quantities that define that operating point, bottleneck bandwidth and round-trip propagation time, and sends at their product; it still has stated gains.
- Quote: "A connection runs with the highest throughput and lowest delay when (rate balance) the bottleneck packet arrival rate equals BtlBw and (full pipe) the total data in flight is equal to the BDP (= BtlBw × RTprop)." — S77, "Characterizing the Bottleneck"
- Quote: "showed this operating point was optimal, maximizing delivered bandwidth while minimizing delay and loss, both for individual connections and for the network as a" — S77, "Congestion and Bottlenecks" (subject: "In 1979 Leonard Kleinrock"; the sentence ends "whole")
- Quote: "The rate and amount BBR sends is solely a function of the estimated BtlBw and RTprop, so the filters control adaptation in addition to estimating the bottleneck constraints." — S77, "Steady-state behavior"
- Counter: BBR's behaviour under loss rests on a chosen gain — "CUBIC's loss tolerance is a structural property of the algorithm, while BBR's is a configuration parameter." — S77 ("Google B4 WAN Deployment Experience")
- Conditions/notes: the article states a peak ProbeBW gain of "1.25 × BtlBw" and ProbeRTT reducing inflight "to four packets for at least one round trip".
- Relevance: the sequential-probing idea (probe for more capacity briefly, probe for the floor latency briefly, otherwise run at the estimate) is the template for a limiter that cannot observe both at once.

### F43. S3-FIFO (SOSP'23) uses three static FIFO queues and reports lower miss ratios than prior algorithms on 6594 traces; its one size constant (small queue = 10%) was chosen on 10 traces, and the authors found their adaptive variant worse on most traces.
- Quote: "Evaluated on 6594 cache traces from 14 datasets, we show that S3-FIFO has lower miss ratios than state-of-the-art algorithms across traces." — S78, abstract
- Quote: "FIFO queues enable S3-FIFO to achieve good scalability with 6× higher throughput compared to optimized LRU at 16 threads." — S78, abstract
- Quote: "choose S to use 10% of the cache space based on experiments with 10 traces and find that 10% generalizes well." — S79, PDF p.5, §4.1
- Quote: "the efficiency does not change much for most traces if S size is between 5% and 20% of the cache size." — S79, PDF p.12, "Is queue size sensitive?"
- Counter: adaptive sizing lost to the fixed 10% — "We compare S3-FIFO-d and S3-FIFO (not shown) and find that S3-FIFO is better than S3-FIFO-d on most traces except the 2% traces at the tail, on which using 10% cache size for S is far from optimal." — S79 (PDF p.13)
- Counter: there is a workload on which the design loses to plain LRU and FIFO — "Most objects in these traces are accessed only twice, and the second request falls out of the small FIFO queue S, which causes the second request to these objects to be cache misses." — S79 (PDF p.9)
- Conditions/notes: the same page says "Most, if not all, adaptive algorithms have many parameters" and "We find that small perturbations in the workload often cause the adaptive algorithm to overreact", and proposes "downsized simulations using spatial sampling" for systems that need the best parameter. Other fixed values: frequency capped at 3 (two bits), ghost queue as many entries as the main queue.
- Relevance: direct evidence against "adaptive is always better": a constant that is insensitive over a measured range (5–20%) can beat a controller. Reliquary's rule should allow a constant when its sensitivity is measured and recorded, and prefer miniature simulation on sampled traffic over online hill climbing where the miss-ratio curve may not be convex.

### F44. SIEVE (NSDI'24) is a one-queue algorithm with no parameters that beats nine algorithms on more than 45% of 1559 web-cache traces, and its authors say it is not scan-resistant.
- Quote: "Our evaluation on 1559 cache traces from 7 sources shows that SIEVE achieves up to 63.2% lower miss ratio than ARC." — S80, abstract
- Quote: "SIEVE has a lower miss ratio than 9 state-of-the-art algorithms on more than 45% of the 1559 traces, while the next best algorithm only has a lower miss ratio on 15%." — S80, abstract
- Quote: "have explicit or implicit parameters, e.g., the sizes of queues in LIRS, the learning rate in LeCaR and CACHEUS, and the decay rate and age granularity in LHD." — S82, PDF p.8, "ZERO parameter" (the sentence's subject: every compared algorithm except FIFO, LRU, CLOCK and Hyperbolic; the paragraph ends by saying SIEVE has no parameter and requires no tuning)
- Counter: on block workloads it can be worse than LRU — "The primary reason for this discrepancy is that SIEVE is not scan-resistant. In block cache workloads, which frequently feature scans, popular objects often intermingle with objects from scans." — S81
- Relevance: a parameter-free candidate for metadata and manifest caches with web-like popularity; not for a blob-chunk cache that sees sequential scans (layer pulls, replication, garbage collection).

### F45. Caffeine's W-TinyLFU sizes its admission window by hill climbing, following a 2018 paper that set out to remove the tuning parameter.
- Quote: "The goal is automatic tuning of the parameters for best performance based on the workload without any manual intervention." — S83, abstract
- Quote: "With these, we derive a parameterless software cache management policy that is competitive for all tested workloads." — S83, abstract
- Quote: "The size of the window vs main space is adaptively determined using a hill climbing optimization." — S84, "Window TinyLfu"
- Conditions/notes: the paper studies two approaches, hill climbing and an indicator of the workload's frequency/recency bias. The climber's step size and sampling period were not read (code not fetched).
- Relevance: the working example of an online-tuned cache parameter; read with F43's warning that hill climbing assumes a convex objective.

### F46. ARC (FAST'03) claims to balance recency and frequency online with no workload-specific tuning; later authors point out that its adaptation step is an implicit parameter.
- Quote: "ARC works uniformly well across varied workloads and cache sizes without any need for workload specific a priori knowledge or tuning." — S85, abstract
- Counter: the adaptation rule hides a constant — "ARC has no explicit parameters, its adaptive algorithm uses implicit parameters in deciding when and how much space to move between the queues." — S82 (PDF p.8)
- Conditions/notes: S79 (PDF p.13) asks of ARC's one-slot move per ghost hit "why one slot instead of half or two?".
- Relevance: "self-tuning" claims must be read for the step sizes they fix; Reliquary's catalogue of constants should list a controller's gains as constants too.

### F47. Dean and Barroso's hedged request: send a second copy only after the first has waited longer than the class's 95th-percentile latency, which adds about 5% load and cuts the tail.
- Quote: "One such approach is to defer sending a secondary request until the first request has been outstanding for more than the 95th-percentile expected latency for this class of requests. This approach limits the additional load to approximately 5% while substantially shortening the latency tail." — S86, "Hedged requests"
- Quote: "sending a hedging request after a 10ms delay reduces the 99.9th-percentile latency for retrieving all 1,000 values from 1,800ms to 74ms while sending just 2% more requests." — S86, "Hedged requests"
- Conditions/notes: the benchmark read 1,000 keys from a BigTable table across 100 servers.
- Relevance: the hedge delay for replica blob reads is a measured percentile per request class, not a timeout constant, and the extra load is bounded by construction (about 1 − percentile).

### F48. Coordinated omission is the load generator waiting for a slow response before sending the next request, so the slow period is under-sampled; wrk2 corrects it by timing from when each request should have been sent at a constant rate.
- Quote: "Since each connection will only begin to send a request after receiving a response, high latency responses result in the load generator coordinating with the server to avoid measurement during high latency periods." — S87, "A note about wrk2's latency measurement technique"
- Quote: "Rather than measure response latency from the time that the actual transmission of a request occurred, wrk2 measures response latency from the time the transmission *should* have occurred according to the constant throughput configured for the run." — S87, same note
- Relevance: every latency figure recorded in `bench/` comes from an open-loop generator at a stated rate; a closed-loop tool's percentiles are not comparable and should be refused by the bench harness.

### F49. Kalibera and Jones: repeat measurements only where the variation is, and only once iterations are independent, because confidence intervals from dependent samples are biased.
- Quote: "We capture experimentation cost with a novel mathematical model, which we use to identify the number of repetitions at each level of an experiment necessary and sufficient to obtain a given level of precision." — S88, abstract
- Quote: "If measurements are not i.i.d., the variance and confidence interval estimates will be biased." — S89, PDF p.5
- Quote: "We call a state independent if the execution times of the benchmark iterations are (statistically) independent and identically distributed." — S89, PDF p.5
- Relevance: "best of N" needs N chosen per level (build, process start, iteration) from measured variance; the bench record should state the levels and that independence was checked.

### F50. MongoDB replaced threshold alerts on CI performance results with E-Divisive change-point detection and reports a large drop in false positives.
- Quote: "This work describes our recent implementation of a change point detection system built upon the E-Divisive means algorithm." — S90, abstract
- Quote: "Using change point detection has had a dramatic impact on our ability to detect performance changes. Quantitatively, it has dramatically dropped our false positive rate for performance changes" — S90, abstract
- Conditions/notes: the abstract gives no number for the drop; a human still reviews the change points.
- Relevance: the performance gate in CI is a change-point test on the series of runs, not a fixed percentage threshold.

### F51. Hunter (DataStax) modifies E-Divisive to use a Student's t-test because the permutation test was not deterministic.
- Quote: "The main change we adopted was to replace the significance test using randomized permutations with a Student's t-test, as we discovered that the randomized approach did not produce deterministic results, at least not with a reasonable number of iterations." — S91, abstract
- Conditions/notes: the t-test assumes "normally-distributed performance metrics" (abstract); compared against PELT and DYNP.
- Relevance: a CI gate must be reproducible: the same history must give the same verdict; check normality before relying on the t-test.

### F52. Google's SmartChoices packages contextual-bandit learning so engineers can replace a heuristic by declaring context, arms and feedback.
- Quote: "While machine learning may outperform these heuristics, replacing existing heuristics in a production system safely and reliably can be prohibitively costly." — S92, abstract
- Quote: "Engineers have independently used SmartChoices to improve a wide range of software including caches, batch processing workloads, and UI layouts, resulting in better latency, throughput, and click-through rates." — S92, abstract
- Conditions/notes: preprint; no per-use numbers in the abstract.
- Relevance: the interface shape (context, arms, feedback, with a safe default arm) is the reusable part for any learned decision in the registry.

### F53. Meta's Spiral moved services from hand-coded heuristics to online-learned ones, and warns that wrong feedback teaches wrong behaviour.
- Quote: "To efficiently address this challenge, the system needed to become self-tuning rather than rely on manually hard-coded heuristics and parameters." — S93, "A new approach to meet Challenges of scale"
- Counter: a learned policy is only as good as its feedback signal — "if the feedback for what should and should not be cached is inaccurate or incomplete, the system will quickly learn to provide incorrect caching decisions, which will degrade performance." — S93
- Relevance: each learned value needs a stated objective, a bound it may not leave, and a fallback, and the feedback path needs its own test.

### D. Agent-artifact ecosystem

### F54. Claude Code reads many `SKILL.md` frontmatter fields beyond the Agent Skills format's six, and other upload paths reject the extra fields with a hard error.
- Quote: "If you include any field the spec doesn't allow, packaging or upload fails with a hard error instead of ignoring the field:" — S94, "Using skill frontmatter outside Claude Code"
- Quote: "Unexpected key(s) in SKILL.md frontmatter: argument-hint. Allowed properties are: allowed-tools, compatibility, description, license, metadata, name" — S94, same section
- Conditions/notes: the six spec fields are `name`, `description`, `license`, `compatibility`, `metadata`, `allowed-tools`. Claude Code additionally documents `when_to_use`, `argument-hint`, `arguments`, `disable-model-invocation`, `user-invocable`, `disallowed-tools`, `model`, `effort`, `context`, `agent`, `background`, `hooks`, `paths` and `shell`. Limits stated: `description` plus `when_to_use` is truncated at 1,536 characters in the listing; `compatibility` is at most 500 characters.
- Relevance: a skill record in the registry should keep the frontmatter as written, validate the six portable fields strictly, and report harness-specific fields per harness instead of refusing them; `hooks` and `allowed-tools` are the fields a policy must inspect, because they grant execution.

### F55. A Claude Code plugin marketplace is a `.claude-plugin/marketplace.json` file with `name`, `owner` and a `plugins` array whose entries name a plugin and a source.
- Quote: "A plugin marketplace is a directory or repository with a `.claude-plugin/marketplace.json` file that lists your plugins and where to fetch each one." — S95, opening
- Quote: "The file requires a `name`, an `owner`, and a `plugins` array." — S95, "Create a marketplace"
- Quote: "Each object in `plugins` is a plugin entry and needs a `name` and a `source`." — S95, "Create a marketplace"
- Conditions/notes: source types listed are a relative path, `github`, `git-subdir`, `url` (git), `archive` ("a zip file downloaded over HTTPS"), `npm` and `command`; git sources can be pinned to a `ref` or `sha`. No OCI source type is listed. Validation rejects a relative `source` that contains `..`.
- Relevance: Reliquary can serve a marketplace today without client changes by generating `marketplace.json` whose entries use the `archive` source pointing at digest-addressed blobs, or the `npm` source if it speaks that protocol.

### F56. An MCP Bundle (`.mcpb`, formerly `.dxt`) is a ZIP archive holding a local MCP server and a `manifest.json`.
- Quote: "MCP Bundles (`.mcpb`) are zip archives containing a local MCP server and a `manifest.json` that describes the server and its capabilities." — S96, opening
- Conditions/notes: the repository moved to `modelcontextprotocol/mcpb`; the npm package is `@anthropic-ai/mcpb`; latest release v2.1.2 (2025-12-04).
- Relevance: MCPB ingestion is exactly the hostile-ZIP case of F21–F23: a bundle is code that will run on a user's machine, in a format whose parsers disagree.

### F57. Public registries of agent skills and MCP servers were attacked or found unsafe in 2025–2026: an impersonating npm MCP server exfiltrated email, and two studies found a quarter to a third of public skills flawed.
- Quote: "A malicious actor created a fake package on npm impersonating our name, built trust over 15 versions, then added a backdoor in version 1.0.16 that secretly BCC’d emails to an external server." — S97, statement of 2025-09-25
- Quote: "13.4% of all skills, or 534 in total, all contain at least one critical-level security issue, including malware distribution, prompt injection attacks, and exposed secrets." — S98, introduction (3,984 skills from ClawHub and skills.sh, as of 2026-02-05)
- Quote: "The barrier to publishing a new agent skill on ClawHub? A SKILL.md Markdown file and a GitHub account that's one week old. No code signing. No security review. No sandbox by default." — S98
- Quote: "26.1% of skills contain at least one vulnerability, spanning 14 distinct patterns across four categories: prompt injection, data exfiltration, privilege escalation, and supply chain risks." — S99, abstract (31,132 skills analysed)
- Quote: "skills bundling executable scripts are 2.12x more likely to contain vulnerabilities than instruction-only skills (OR=2.12, p<0.001)" — S99, abstract
- Conditions/notes: S98 is a vendor's own study (it also reports "76 malicious payloads" confirmed by human review); S99 is a preprint whose detector has 86.7% precision and 82.5% recall, so its percentages carry that error. The postmark-mcp case is name impersonation followed by a late malicious version: trust accumulated over 15 clean versions.
- Relevance: the controls the evidence points to are publisher identity bound to a namespace (so a name cannot be taken by a stranger), signatures and provenance required at publish, a diff-aware review trigger when a new version adds network or execution capability, and a distinction between instruction-only and script-carrying skills.

### F58. CNCF ModelPack packages a model as an OCI image manifest with `artifactType: application/vnd.cncf.model.manifest.v1+json` and typed layers for weights, configuration, documentation, code and datasets.
- Quote: "This REQUIRED property MUST be `application/vnd.cncf.model.manifest.v1+json`." — S100, manifest properties, `artifactType`
- Conditions/notes: config media type `application/vnd.cncf.model.config.v1+json`; layer types such as `application/vnd.cncf.model.weight.v1.raw`, `.tar`, `.tar+gzip`, `.tar+zstd`. A released version number was not found (the releases API returned an empty list).
- Relevance: model artifacts need nothing special from a conformant registry except very large single layers and raw (non-tar) layer types; test multi-gigabyte chunked uploads and ranged reads with these media types.

## Tables

### 1. Hostile-input classes
| class | mechanism | primary source | what a server must do |
|---|---|---|---|
| ZIP overlapping-file bomb | many entries reference one compressed kernel; output grows quadratically (10 MB to 281 TB) | S32 | budget output bytes and entries during inflation, inside the parser; refuse overlapping entries; never trust declared sizes |
| ZIP parser differential | ambiguity in redundant metadata, path processing and structure positioning; 1221 of 1225 parser pairs differ | S33, S34 | accept only canonical archives or re-pack before signing and scanning; one parser for every decision |
| Zip Slip | entry name with `../` or absolute path | S35 | resolve each entry beneath the destination root; refuse on escape |
| TAR link escape | an entry is written through a symlink or hard link created earlier in the archive | S37–S43 | never follow archive-created links; refuse link targets outside the root; no check-then-use on paths |
| TAR header differential | PAX size and ustar size disagree; parser advances by the wrong one | S44, S45 | PAX overrides govern stream position; test disagreeing headers; use a maintained crate |
| TAR header exhaustion | unbounded header or PAX record size | S46 | cap header size (Go chose 1 MiB) |
| OCI whiteouts and duplicates | `.wh.` and `.wh..wh..opq` entries; duplicate paths | S36 | apply whiteouts to lower layers only, hide them; define and test the rule for duplicates and for paths leaving the root (the spec is silent) |
| HTTP/2 Rapid Reset | HEADERS then RST_STREAM; concurrency limit never binds | S47–S49 | count cancelled against completed work per connection; close the connection on abuse and on stream-limit violations |
| HTTP/2 CONTINUATION flood | header block without END_HEADERS | S50, S51 | bound header-block bytes, frames and time; close the connection; count the refusal |
| HTTP/2 MadeYouReset | client provokes server resets; backend work continues | S52–S54 | bound work in flight, not open streams; budget server-sent resets |
| HTTP/2 control-frame floods | PING, SETTINGS, empty frames, 1-byte windows force queued replies | S55, S56 | bounded control-frame queue; ENHANCE_YOUR_CALM; respect TCP backpressure |
| HPACK bomb | small encoded block expands through table references (4 KB to 64 MB) | S58, S56, S57 | bound decoded header-list bytes while decoding; 431 or close |
| HTTP/1.1 smuggling | CL.TE, TE.CL, TE.TE, obs-fold, whitespace before colon, bad Content-Length | S59–S62 | 400 and close for each; refuse CL with TE; prefer HTTP/2 to back ends |
| Bare CR, bare LF | line terminators parsed differently by neighbours | S59 | bare CR invalid (RFC MUST); bare LF refused by choice (RFC only says MAY accept) |
| zstd window | frame declares a window up to 3.75 TB | S63, S64 | set a maximum window before decoding; typed refusal above it |
| gzip / DEFLATE bomb | up to 1032:1 per stream, multiplied by nesting | S65, S32 | output budget from compressed size × 1032 and declared size, shared across nesting |

### 2. Adaptive mechanisms
| mechanism | what it measures | what it replaces | parameters it still fixes, and the justification given |
|---|---|---|---|
| Netflix Vegas limit (S66, S68) | queue estimate from minRTT against sample RTT | load-tested RPS limit | alpha and beta ("typically" 2–3 and 4–6 in the README; `3*log10` and `6*log10` in code), initial 20, max 1000, probe multiplier 30; no measurement cited |
| Netflix Gradient2 (S66, S67) | divergence of short and long RTT averages | same | smoothing 0.2, long window 600, tolerance 1.5, queue size 4, min 20, max 200; reason given for using averages, none for the values |
| Netflix AIMD (S69) | timeouts and drops | same | backoff ratio 0.9, timeout 5 s, min 20, max 200; none given |
| Envoy gradient controller (S70, S71) | minRTT from a pinned-concurrency probe; sample RTT percentile | static concurrency limits | probe concurrency 3, buffer 25%, jitter 15%, 50 requests per window, p50, max 1000, headroom = sqrt(limit); roles explained, values not justified |
| DAGOR (S72, S73) | average queuing time per window | per-service ad hoc shedding | 20 ms threshold against a 500 ms timeout, window 1 s or 2000 requests; called empirical, five years in production |
| Breakwater (S74) | server-side queueing delay | client-side rate limits, drop-based control | not stated in the abstract |
| CoDel (S75) | minimum sojourn time over an interval | queue-length limits and tuned AQM | INTERVAL 100 ms (worst-case terrestrial RTT, measured instead in the data-center variant), TARGET 5 ms (5% of interval, from the power analysis and simulation) |
| BBR (S77) | windowed max delivery rate and min RTT | packet loss as the congestion signal | ProbeBW peak gain 1.25, ProbeRTT at four packets; stated, with loss tolerance acknowledged as a configuration parameter |
| S3-FIFO (S78, S79) | nothing online (static queues) | LRU and adaptive policies | small queue 10% (10 traces, insensitive from 5% to 20%), frequency cap 3; the adaptive variant was worse on most traces |
| SIEVE (S80–S82) | nothing online | LRU | none; not scan-resistant |
| W-TinyLFU with hill climbing (S83, S84) | hit-rate change after a window resize | fixed admission-window size | climber step and period: not read |
| ARC (S85) | hits on ghost lists | fixed recency/frequency split | implicit: how much to move per ghost hit (S82, S79) |
| Hedged requests (S86) | 95th-percentile latency per request class | fixed retry timeouts | the percentile (95th), which sets the extra load (about 5%) |
| E-Divisive change points (S90, S91) | distribution shift in a result series | fixed regression thresholds | significance level; Hunter fixes the test (Student's t) for determinism |
| SmartChoices, Spiral (S92, S93) | per-decision feedback | hand-coded heuristics | the feedback definition and the fallback, which the sources call the hard part |

## Catalogue of constants
| system | constant | value | where (URL) | justification given? |
|---|---|---|---|---|
| CoDel | INTERVAL | 100 ms | https://www.rfc-editor.org/rfc/rfc8289.txt §4.2 | yes: worst-case RTT of the normal terrestrial Internet; Appendix A measures it instead |
| CoDel | TARGET | 5 ms | same, §4.3 | yes: 5–10% of RTT from the power analysis, plus simulation |
| HTTP/2 | SETTINGS_MAX_CONCURRENT_STREAMS floor | "no smaller than 100"; initially no limit | https://www.rfc-editor.org/rfc/rfc9113.txt §6.5.2 | partly: "so as to not unnecessarily limit parallelism" |
| HTTP/2 | SETTINGS_HEADER_TABLE_SIZE initial | 4,096 octets | same, §6.5.2 | no |
| HTTP/2 | SETTINGS_MAX_HEADER_LIST_SIZE initial | unlimited | same, §6.5.2 | no |
| Google Rapid Reset mitigation | abuse heuristic | more than 100 requests with more than 50% cancelled | https://cloud.google.com/blog/products/identity-security/how-it-works-the-novel-http2-rapid-reset-ddos-attack | no: offered as an example |
| Go archive/tar | maximum header block | 1 MiB | https://cveawg.mitre.org/api/cve/CVE-2022-2879 | no |
| Zstandard | maximum Window_Size in the format | (1<<41) + 7*(1<<38) bytes = 3.75 TB | https://www.rfc-editor.org/rfc/rfc8878.txt §3.1.1.1.2 | derived from the field widths |
| Zstandard | decoder window to support | 8 MB (recommended in RFC 8878; MUST for HTTP content coding in RFC 9659) | https://www.rfc-editor.org/rfc/rfc9659.txt §3 | interoperability; no measurement cited |
| DEFLATE | maximum expansion | 1032:1 | https://zlib.net/zlib_tech.html | yes: derived (258 bytes from 2 bits) |
| HPACK bomb (Imperva, against nghttpd before 1.7.1) | references per request, table size | 16K references to a 4 KB header = 64 MB | https://www.imperva.com/docs/Imperva_HII_HTTP2.pdf p.18 | measured in the authors' lab |
| Netflix Gradient2Limit | initial / min / max limit | 20 / 20 / 200 | https://raw.githubusercontent.com/Netflix/concurrency-limits/main/concurrency-limits-core/src/main/java/com/netflix/concurrency/limits/limit/Gradient2Limit.java | no |
| Netflix Gradient2Limit | smoothing, long window, RTT tolerance, queue size | 0.2, 600, 1.5, 4 | same | no |
| Netflix VegasLimit | initial / max limit, probe multiplier | 20 / 1000, 30 | https://raw.githubusercontent.com/Netflix/concurrency-limits/main/concurrency-limits-core/src/main/java/com/netflix/concurrency/limits/limit/VegasLimit.java | no |
| Netflix VegasLimit | alpha, beta | 3 × log10(limit), 6 × log10(limit) | same | README says "typically" 2–3 and 4–6 |
| Netflix AIMDLimit | backoff ratio, timeout, min / max | 0.9, 5 s, 20 / 200 | https://raw.githubusercontent.com/Netflix/concurrency-limits/main/concurrency-limits-core/src/main/java/com/netflix/concurrency/limits/limit/AIMDLimit.java | no |
| Envoy adaptive concurrency | minRTT probe concurrency | 3 | https://raw.githubusercontent.com/envoyproxy/envoy/main/api/envoy/extensions/filters/http/adaptive_concurrency/v3/adaptive_concurrency.proto | no |
| Envoy adaptive concurrency | buffer, jitter, request count, percentile, max limit | 25%, 15%, 50, p50, 1000 | same | no |
| Envoy adaptive concurrency | headroom | sqrt(limit), not configurable | https://www.envoyproxy.io/docs/envoy/latest/configuration/http/http_filters/adaptive_concurrency_filter | role explained; the square root is not justified |
| DAGOR | overload threshold on average queuing time | 20 ms (service timeout 500 ms) | https://arxiv.org/pdf/1806.04075 p.5 | "empirical", five years of production |
| DAGOR | monitoring window | 1 s or 2000 requests | same | reason given: follow load changes |
| BBR | ProbeBW peak pacing gain | 1.25 | https://queue.acm.org/detail.cfm?id=3022184 | partly: raised after cellular connections stuck at low rates |
| BBR | ProbeRTT inflight | four packets for at least one round trip | same | no |
| S3-FIFO | small queue share | 10% of the cache | https://jasony.me/publication/sosp23-s3fifo.pdf p.5, p.12 | yes: chosen on 10 traces, checked on 6594, insensitive from 5% to 20% |
| S3-FIFO | frequency cap | 3 (two bits) | same, p.5 | no |
| S3-FIFO-d (adaptive variant) | ghost size, trigger, step | 5% each, 100 hits with a 2× imbalance, 0.1% of cache | same, p.12 | no; the authors report tuning it "is very challenging" |
| Tail at Scale | hedge delay | 95th-percentile latency of the request class | https://cacm.acm.org/research/the-tail-at-scale/ | yes: bounds extra load to about 5% |
| Amazon ECR | managed signing rules | up to 10 per registry | https://docs.aws.amazon.com/AmazonECR/latest/userguide/managed-signing.html | no |
| Claude Code skills | listing text cap; `compatibility` length | 1,536 characters; 500 characters | https://code.claude.com/docs/en/skills.md | no |

## Not verified / open questions
- The manifest layout GitHub's `push-to-registry` writes (whether a referrer with the Sigstore bundle `artifactType`) is not stated in the READMEs or docs read; inspect a real image or the `@actions/attest` source.
- The BuildKit release that made OCI-artifact attestation storage the default was not found in the storage document.
- The approval date of SLSA v1.2 was not captured; the page shows only "Status: Approved".
- How common raw `application/spdx+json` or `application/vnd.cyclonedx+json` is as a referrer `artifactType`, compared with in-toto attestations of SBOMs, was not measured or sourced.
- No OCI text setting a zstd window limit for `tar+zstd` layers was searched for; RFC 9659 covers only the HTTP content coding.
- Breakwater's parameters and the W-TinyLFU climber's step and period were not read (paper body and Caffeine source not examined).
- The SIEVE "no parameter" sentence is paraphrased, because the PDF text renders the name in split small capitals; the adjacent sentence is quoted instead.
- The Tail at Scale, BBR, S3-FIFO and Adaptive Software Cache Management quotes were checked against Internet Archive captures of the publisher pages, since ACM refuses automated fetches; the capture dates were not recorded.
- Koi Security's reports on the postmark-mcp backdoor and on malicious ClawHub skills could not be fetched (the URLs now serve another vendor's page), so the incident is cited from Postmark's statement and Snyk's study only; the count of affected downloads was not verified.
- Imperva's HPACK bomb report is dated 2016 from its CVE numbers; the month was not verified.
- Whether a stable release of the OCI Auth working group's output is planned: no specification document exists in its repository as of 2026-11-20.
- The Sigstore post-quantum design is a proposal; its bundle media type and timeline are not published.
- Workload identity federation was read for GitHub Actions to AWS only; the Google Artifact Registry and Azure Container Registry federation documents were not fetched.
- Rapid Reset peak rates are the vendors' own reports (Cloudflare 201 million, Google 398 million requests per second); they were not cross-checked against an independent measurement.
