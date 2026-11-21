# Research notes

The source notes Reliquary's design and plan cite (CLAUDE.md §5). Each note lists its sources, then
its findings; every finding carries a verbatim quote and where it is. Notes 03 to 06 were gathered
on 2026-11-20 by research agents under written briefs; each saved the pages it read and checked by
script that every quote is an exact substring of its page, and each ends with what it could not
verify. `docs/design/architecture.md` cites these as "research/NN §x" or "research/NN Fn".

| Note | What it covers | Size |
|---|---|---|
| [01 Internal ground](01-internal-ground.md) | Kiln's plans for the track; kiln's present registry; the seams (kiln-seams, Amphora, Quiver, Veil); Mantle's two doors, client and in-process node; the house rule on numbers and its gate; the shared runtime and transport; Shards' artifact types and client; what Hex needs | 12 sources |
| [02 Specifications](02-specifications.md) | OCI Distribution v1.1.1 and its unreleased next version; the conformance suite's settings; images, indexes and artifacts; the token specification; the MCP Registry API; the Agent Skills format; TUF's attack definitions; tiled transparency logs; RFC 6298 | 12 sources |
| [03 Self-hosted registries](03-registries-self-hosted.md) | CNCF Distribution, Harbor, Zot, Quay, GitLab's registry, olareg, the Rust registries (angos, nora, peryx, trow) and others: architecture, conformance, garbage collection and its failures, proxying, auth, trust, limits and constants, published benchmarks, operational pain | 656 sources, 401 findings |
| [04 Lazy loading, chunking, P2P, deltas](04-lazy-loading-p2p-chunking.md) | Registry workload studies; eStargz, SOCI, zstd:chunked, Nydus, DADI, EROFS, composefs, Lambda's block loader, Firecracker; content-defined chunking and hashing; dedup side channels; Kraken, Dragonfly, Spegel, BitTorrent v2; binary deltas | 113 sources, 93 findings |
| [05 Garbage collection, replication, hosted registries, scale](05-gc-replication-cloud-scale.md) | Online collection in GitLab, Harbor, zot, containerd; hybrid logical clocks and last-writer-wins registers; cross-region behavior of ACR, ECR, Harbor; Docker Hub's limits and what counts as a pull; ECR, ACR, GHCR, Artifact Registry limits; scale anchors; outages | 59 sources, 53 findings |
| [06 Trust, hostile input, control](06-trust-hazards-control.md) | Sigstore bundles, cosign's two storage forms, Rekor's two generations, SLSA and in-toto, Notary Project signatures, registry-side signing, OSV and OpenVEX, workload identity; zip and tar hazards, HTTP/2 and HTTP/1.1 abuse, decompression bounds; adaptive concurrency, CoDel, power, cache eviction, hedging, honest measurement; skills, plugin marketplaces and MCP bundles in the wild | 100 sources, 58 findings |
| [07 Measurement](07-measurement.md) | E-Divisive change points and how MongoDB and Hunter run them in CI; instruction counts without privilege (cachegrind, perf's paranoia, Docker's and Kubernetes' seccomp); what the load average counts and what a container sees; layout and environment bias (Mytkowicz, STABILIZER); randomized interleaved trials; judging a difference by interval overlap; open against closed loops; the measurement design `reliquary bench` follows | 40 sources, 40 findings |
| [08 The seams](08-seams.md) | RFC 9110's byte ranges and 416; RFC 2104's HMAC; Mantle's native client: bounded memory, uploads that resume from the node's durable offset, operation identities, lost answers, absolute deadlines; Amphora's and Quiver's traits; kiln's seams, their presigned URLs and suites; hyper-measure's counting allocator; an operating system's entropy source, unready and short | 8 sources, 13 findings |
| [10 The reference model and how it is checked](10-model-and-checking.md) | Linearizability's definition and locality (Herlihy and Wing); Wing and Gong's search, Lowe's cache and just-in-time linearization, Porcupine and Knossos; NP-completeness and bounded searches; P-compositionality (Horn and Kroening); operations whose outcome is unknown (Jepsen's `:info`, Knossos); delta debugging and 1-minimality (Zeller and Hildebrandt); RFC 9110's byte ranges; RFC 6585's 429 | 11 sources, 19 findings |
| [11 Structural gates](11-structural-gates.md) | Mantle's recursion check and what it leaves; rustc's vtables and their invariant loads; class hierarchy analysis; cargo-deny's scoped, expiring exceptions; Hex's skip record and libtest's capture; O_APPEND; the embeddable Python's path; kiln's runners and the distributions' images | 11 sources, 10 findings |
| [12 The simulated world](12-simulation.md) | hyper-sim's design: processes as state machines, the world's choices, clocks that drift and step, partitions and duplication, crash states, liveness by progress, determinism checked, planted defects; Amphora's unknown outcomes; Windows' 100 ns wall time; TCP's reset; what this step's runs found | 7 sources, 15 findings |

Five things the notes establish that shape the plan most:

1. **No registry's published numbers can be compared as they stand** (03 F090, F158): the baselines
   are measured on our hardware first (plan RQ4.1).
2. **Every deployed timeout, window, chunk size and piece size found is a judgement** (03 "Catalogue
   of constants", 04 F68, 05 F3 to F12): Reliquary learns or derives each (`docs/design/numbers.md`).
3. **Collectors elsewhere stop the world or guess a window, and several have lost data** (03 F019,
   F120, F244; 05 F1 to F12): Reliquary's collection is conditional transactions, model-checked (plan
   RQ9.3).
4. **Learning is not always better than a fixed rule** (06 F43, F46, F53): a learned value must win
   an A|B against the best fixed rule a source measured, or the cited rule stands (architecture D28,
   plan RQ0.8, RQ4.4).
5. **Competitors' notifications lose events, their audit trails cost every pull, and the newer ones
   take a workload's own token in place of a stored secret** (03 F037, F096, F136, F198, F252; 06
   F19): plan RQ7.11 to RQ7.15.
