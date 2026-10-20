# Reliquary's design

The target, from track T9 of kiln's critical path (kiln plan `01a157fe104d7444a38c808b89f2fae0`). Each decision below is
made by the goal named beside it, recorded here with its sources, its evidence and what would
falsify it (CLAUDE.md §5).

## What it is

Reliquary is a registry that puts the best agents and skills in everyone's hands. It holds:
- `reliquary`: distribution, artifacts, the catalog and trust;
- `reliquary-api`: the traits others use, with a simulator implementation;
- the `reliquary` binary.

Records and bytes stay apart: its records (manifests, tags, repositories, referrers, the catalog,
signatures and policies) are in Amphora; its bytes (layers, artifacts, images) are on Mantle through
the `Blobs` seam; its events on Quiver; its credentials for upstream registries from Veil. It
starts from kiln's own registry, which moves here; kiln vendors it back at `vendor/reliquary` and
reaches it only through kiln-seams' `Artifacts`.

## The target

- **Distribution:** OCI Distribution v1.1, passing its conformance suite.
- **Artifacts checked on push:** agents, harnesses and MCP servers by the Agentfile spec's rules
  (Shards' AGENTFILE_ARCH §12.17); skills as Claude Code and the Agent Skills format define them;
  Shards' EROFS root filesystems and Firecracker's kernel and rootfs pairs; OCI images and indexes.
- **The MCP Registry's API**, so MCP clients discover servers here.
- **The catalog:** full-text search, facets by kind, platform, publisher and signature, and a page
  per artifact with its versions, asks, signatures, SBOM, provenance and pulls.
- **Trust:** Sigstore signatures as referrers, TUF for metadata freshness, Rego policies at pull,
  provenance from kiln CI's attestations.
- **Speed:** pull-through caches, P2P between runners, lazy loading from seekable layers with
  chunk-level deduplication, recorded working sets prefetched.
- **On Mantle:** online garbage collection by leases, quotas and retention per namespace.
- **Compared with** CNCF Distribution, Harbor and Zot on the same hardware, and ECR.

## Decisions

| Decision | Made by | Status |
|---|---|---|
| Distribution and conformance, from kiln's registry | CP-T9.1 | open |
| Artifact checks on push | CP-T9.1 | open |
| Skills and the MCP Registry API | CP-T9.2 | open |
| The catalog's index and search | CP-T9.3 | open |
| Trust: signatures, TUF and policy at pull | CP-T9.4 | open |
| Pull-through caches and mirrors | CP-T9.5 | open |
| Fast distribution: P2P, lazy loading, prefetch | CP-T9.6 | open |
| Garbage collection, quotas and retention on Mantle | CP-T9.7 | open |
