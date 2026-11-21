# Reliquary's design

The target, from track T9 of kiln's critical path (kiln plan `01a157fe104d7444a38c808b89f2fae0`) and
the person's direction of 2026-11-20: best along every axis, an order of magnitude over every
competitor, one binary from a laptop to regions, the Hex runtime served, and hyper-quic for Shards.
Each decision is recorded with its sources, its evidence and what would falsify it (CLAUDE.md §5).

- [`architecture.md`](architecture.md): the shape and the decisions, D1 to D28.
- [`numbers.md`](numbers.md): the rule on numbers, and what replaces each thing that is usually a constant.
- [`constants.md`](constants.md): every number production code runs with, its kind and what establishes it; the gate's inventory.
- [`observability.md`](observability.md): how Reliquary tells an operator about itself, O1 to O6. The list
  of every metric, event and dashboard row is [`../observability.md`](../observability.md).
- [`gates.md`](gates.md): the structural gates and the platform matrix, G1 to G6: the gate's order, no
  recursion through any call, `unsafe` only where CLAUDE.md names it, the dependency graph, citations and
  the skip record, and what runs on which platform.
- [`model.md`](model.md): the reference model and the differential harness, H1 to H14: the answer to every
  request, and how an implementation, a cluster or regions are held to it.
- [`seams.md`](seams.md): the seams to other products, S1 to S11: Amphora's `Records`, Quiver's `Topics`,
  and Reliquary's own `Bytes`, `Secrets`, `Upstream`, `Clock` and `Entropy`, each contract clause by clause,
  with its simulator's faults.
- [`sim.md`](sim.md): the simulated world, W1 to W12: one seeded world, nodes running a core over the
  simulated seams, clients, a nemesis of every fault kind, and the verdicts a run is judged by.
- [`plan/`](plan/README.md): the phased plan, RQ0 to RQ13, 126 items, each with its criteria and tests.
- [`../research/`](../research/): the source notes the decisions cite.

## What it is

Reliquary is a registry that puts the best agents and skills in everyone's hands. It holds:
- `reliquary-api`: the trait others reach Reliquary through, with a simulator and a conformance suite;
- `reliquary-core`: the sans-IO cores, and the seams Reliquary reaches others through;
- `reliquary-wire` and `reliquary-client`: the native protocol and the client Shards, Hex and kiln link;
- `reliquary-node`: the OCI door, the native door and the shells of the seams;
- `reliquary-sim` and `reliquary-bench`;
- the `reliquary` binary.

Each crate appears with its first code (architecture.md D24).

Records and bytes stay apart: its records (manifests, tags, repositories, referrers, the catalog,
signatures and policies) are in Amphora; its bytes (layers, artifacts, images) are on Mantle through
the `Bytes` seam; its events on Quiver; its credentials for upstream registries and its signing
leases from Veil. Kiln's registry moves here (plan RQ2.9); kiln vendors Reliquary back at
`vendor/reliquary` and reaches it only through kiln-seams' `Artifacts`.

## The target

- **Distribution:** OCI Distribution v1.1.1, passing its conformance suite, and its next version's
  additions as they are tested by that suite.
- **Two doors:** the OCI door for every stock client; the native door, on hyper-quic, for Shards,
  Hex and kiln, where a pull is one exchange.
- **Artifacts checked on push:** agents, harnesses and MCP servers by the Agentfile spec's rules
  (Shards' AGENTFILE_ARCH §12.17); skills as Claude Code and the Agent Skills format define them;
  Shards' EROFS microVM images and Firecracker's kernel and rootfs pairs; images built from
  Agentfiles; tar and zip archives, with one file readable out of them; OCI images and indexes.
- **The MCP Registry's API**, so MCP clients discover servers here.
- **The catalog:** full-text search, facets by kind, platform, publisher and signature, and a page
  per artifact with its versions, asks, signatures, SBOM, provenance and pulls.
- **Trust:** Sigstore signatures as referrers, verified once into facts; TUF for trust roots and for
  what each tag points at; Rego policies decided ahead of pulls; provenance from kiln CI; a
  verifiable history of every tag.
- **Speed:** ten times the best competitor or at the hardware's measured limit; pull-through
  caches; peers; lazy loading with every block verified; chunk-level sharing; working sets.
- **On Mantle:** online garbage collection with no pause, exact quotas, retention per namespace.
- **Regions:** replication by a stated rule, pulls in-region, a region lost without a failed pull.
- **No picked numbers:** every number has a kind and a gate (numbers.md).
- **Compared with** CNCF Distribution, Harbor and Zot on the same hardware, and ECR.

## Decisions

| Decision | In architecture.md | Built or re-decided by | Status |
|---|---|---|---|
| Two doors over one core | D1 | RQ1.6, RQ3 | decided 2026-11-20; open |
| Records as rows, conditional transactions, one change feed | D2 | RQ1.2 | decided; open |
| Bytes under keys of their own; digests in the records | D3 | RQ1.3, RQ1.8 | decided; open |
| Bytes bypass Reliquary where they can; the rule is derived per request | D4 | RQ3.5, RQ4.5 | by data |
| No special node | D5 | RQ9.1 | decided; open |
| Online garbage collection by three conditional steps | D6 | RQ9.3, RQ9.4 | decided; open |
| Every number has a kind | D7 | RQ0.2 | decided; the gate built |
| Waits by liveness, progress and the slowest seen | D8 | RQ0.8 | decided; open |
| Learned admission per bottleneck | D9 | RQ4.8 | decided; open |
| Artifact checks on push, shared with Shards | D10 | RQ2 | decided; open |
| Whole against chunked storage | D11 | RQ10.1, RQ10.3 | by data |
| Ranges verified by a hash tree | D12 | RQ10.2 | decided; open |
| Lazy loading at the block device | D13 | RQ10.4, RQ10.6 | by data, with Shards' and Hex's owners |
| Verified peers | D14 | RQ10.7 | by data |
| Trust as facts, decided ahead, checked twice | D15 | RQ7.1, RQ7.4, RQ7.5, RQ7.6, RQ7.7 | decided; open |
| Signed tags and a verifiable history | D16 | RQ7.3, RQ7.8 | decided; open |
| Pull-through freshness and budgets | D17 | RQ8 | decided; open |
| Tenancy without oracles | D18 | RQ9.5, RQ13.4 | decided; open |
| Regions: homed by default | D19 | RQ12 | decided; open, with Amphora's owner |
| A derived, per-node catalog index | D20 | RQ6.2 | by data |
| The house's runtime, codecs and simulator | D21 | RQ0.1, RQ1.6 | decided; open |
| Skills and MCP servers | D22 | RQ5 | decided; open, with Shards' owner |
| Measurement and A|B | D23, [`measurement.md`](measurement.md) M1 to M8 | RQ0.7 | built, on branch rq0-ground, not merged; the gate and the nightly ran by hand in kiln CI on 2026-11-21; the nightly alarm moved to RQ4.5 (the person, 2026-11-21) |
| Crates | D24 | each phase | decided; open |
| Identity proved, not stored | D25 | RQ7.11, RQ7.12 | decided; open |
| Audit in the change's own commit; events from the change feed | D26 | RQ7.13, RQ7.14 | decided; open |
| Both families of signature; signing on push; scanners outside | D27 | RQ7.9, RQ7.10, RQ7.15 | decided; open, with Veil's owner |
| Learning is not assumed to win | D28 | RQ0.8, RQ4.4 | by data |
| Observability: base units, closed labels, OpenMetrics, events by target, the event ring, one catalog | observability.md O1 to O6 | RQ0.9 | decided 2026-11-20; names agreed with kiln's Observe owner 2026-11-20 |
| The reference model, its answers and the differential harness | [`model.md`](model.md) H1 to H14 | RQ0.5 | built 2026-11-20, not committed |
| Seams to other products, each with a simulator that injects every fault and a suite | [`seams.md`](seams.md) S1 to S11 | RQ0.4 | built 2026-11-20, not committed |
| The simulated world: one seeded world, nodes running a core over the seams, a nemesis, verdicts by history and liveness | [`sim.md`](sim.md) W1 to W12 | RQ0.6 | built 2026-11-20, not committed |
