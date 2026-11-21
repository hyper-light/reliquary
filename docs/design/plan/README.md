# Reliquary's plan

The phased plan for Reliquary, from an empty workspace to a registry that runs production kiln, on a
laptop and across regions, on one code path. Fourteen phases, RQ0 to RQ13; 126 items; each item with
what to build, when it is done, and the tests that show it. The design it implements is
[`../architecture.md`](../architecture.md); the rule on numbers is [`../numbers.md`](../numbers.md);
the sources are in [`../../research/`](../../research/).

In kiln each phase is a goal on the Reliquary agent's goal stack, and each item is a step of that
goal's plan. This directory is the text those were made from, and `docs/requirements.toml` holds
every ID.

## In kiln

The Reliquary agent's goal stack holds one goal per phase, each turned into a plan whose steps are
the phase's items, with their criteria, tests, what they wait for and the findings they rest on.

| Phase | Kiln plan | Its research |
|---|---|---|
| RQ0 | `01a1fa09c6087239961e72c87e38f8d2` | `01a1fa09c60970a28bff314b5b4c6cbc` |
| RQ1 | `01a1fa0acd59700f83ae3b747a0ede5a` | `01a1fa0acd9b7677ac5a6864a9754b9d` |
| RQ2 | `01a1fa0b4b0672c09d3565472a665976` | `01a1fa0b4b1771f0b6ce30ed439df405` |
| RQ3 | `01a1fa0b5c3c75d7b7a3803e3954e99b` | `01a1fa0b5c3d7326ab33a0c2c7e51b44` |
| RQ4 | `01a1fa0b9785720b958e3de7d47d1197` | `01a1fa0b97867611921436aad89da9c1` |
| RQ5 | `01a1fa0bccbc77f896b78457c6a2429a` | `01a1fa0bccbe724e8b9a722eeef6c091` |
| RQ6 | `01a1fa0dff4d70cf8f7fd79c9c862d26` | `01a1fa0dff4e77e081338e49896f3f09` |
| RQ7 | `01a1fa0e13fc7459922ade1d6aa3d589` | `01a1fa0e14e172d58ba5dfdb080c823b` |
| RQ8 | `01a1fa0e58f4725fb5b0c26592d0c690` | `01a1fa0e58f576c0b26da44f74cb703d` |
| RQ9 | `01a1fa0e70c976999842634e461447f2` | `01a1fa0e70ca778aad9f9839da033e3a` |
| RQ10 | `01a1fa0e88b276c288689c4784cba1c3` | `01a1fa0e88da737da0d2fcbe07ad52b3` |
| RQ11 | `01a1fa0e991472f996a32a6f07679f34` | `01a1fa0e991472f9999b4d81b81c2503` |
| RQ12 | `01a1fa0eaa3177b0832dc0b61bbea4ae` | `01a1fa0eaa3177b0867c564719b421f0` |
| RQ13 | `01a1fa0eeaa07794b8f6ec1f652244ce` | `01a1fa0eec3276169d504d34f078d1ee` |

A step's goal can be split into its criteria and tests as it is taken up. The stack holds at most 500
goals, met ones included, so split a step when it is under way, not all of them ahead.

## How the plan is ordered

1. **The smallest thing that works comes first.** Phase 1 is a registry on one machine that Docker
   can push to and pull from. Everything after deepens one axis of it, and every phase ends in
   something a person or kiln can use that day.
2. **Data decides.** A design choice with more than one candidate is made by an A|B record under
   the measurement rules of RQ0.7, kept in `bench/`. A phase whose shape depends on data starts by
   collecting it (RQ4.2's cost table, RQ10.1's corpus measurements, RQ11.1's release history).
3. **No number is picked.** Every number in production code has a row in `docs/design/constants.md`
   whose kind is format, external, derived, cited, learned or counted, and a gate refuses the rest
   (RQ0.2). A value measured once on one machine and frozen is refused like a guess. Waits, limits,
   sizes and rates are learned on the path, peer, device or repository they belong to, and follow
   it when it changes (RQ0.8). The numbers that appear in this plan's criteria are kiln's own
   criteria, a specification's, or the person's order-of-magnitude bar, and each says which.
4. **One code path.** The laptop, the fleet and three regions run the same core; the same
   differential suite passes on one node (RQ1), three nodes (RQ9) and three regions (RQ12).
5. **Work side by side.** Phases that do not need each other run at once; items inside a phase say
   what they wait for. After RQ1, seven phases can be under way together.
6. **Other products are asked, not edited.** Where an item needs a change in Mantle, Amphora,
   Shards, Hex, Veil or kiln, the item is the request, the seam and its tests; the change is its
   owner's.

## The phases

| Phase | Goal, in a line | Serves (kiln's T9) | Needs | Runs beside |
|---|---|---|---|---|
| [RQ0 The ground](phase-00-ground.md) | Gates, seams with simulators, a model, a simulated world, a measurement tool | CP-T9.1 | none | |
| [RQ1 The slice](phase-01-slice.md) | One binary; `docker push` and `pull`; OCI conformance; nothing acknowledged lost | CP-T9.1 | RQ0 | |
| [RQ2 Artifacts](phase-02-artifacts.md) | Agents, harnesses, MCP servers, microVMs, tar and zip checked on push; kiln's registry moves here | CP-T9.1 | RQ1 | RQ3 to RQ9 |
| [RQ3 The native door](phase-03-native-door.md) | hyper-quic for Shards, Hex and kiln; a pull in one exchange; Hex's image store | CP-T9.1, CP-T9.6 | RQ1 | RQ2, RQ4 to RQ9 |
| [RQ4 Speed](phase-04-speed.md) | Ten times the best competitor, or at the hardware's limit, on one node | AC-T9.1.4, CP-T9.10 | RQ1 | RQ2, RQ3, RQ5 to RQ9 |
| [RQ5 Skills and MCP](phase-05-skills-mcp.md) | Skills and MCP servers first-class; the MCP Registry API | CP-T9.2 | RQ1, RQ2 | RQ3, RQ4, RQ6 to RQ9 |
| [RQ6 The catalog](phase-06-catalog.md) | Search with facets; a page per artifact | CP-T9.3 | RQ1 | RQ2 to RQ5, RQ7 to RQ9 |
| [RQ7 Trust, identity and audit](phase-07-trust.md) | Signatures as facts, fresh roots, signed tags, policy at pull, provenance; identities without stored secrets; an audit trail; events never lost; a plug for scanners | CP-T9.4, CP-T9.10 | RQ1 | RQ2 to RQ6, RQ8, RQ9 |
| [RQ8 Pull-through](phase-08-pull-through.md) | Caches and mirrors of other registries | CP-T9.5 | RQ1 | RQ2 to RQ7, RQ9 |
| [RQ9 A cluster](phase-09-cluster.md) | Many nodes; online garbage collection; quotas; retention; a disk or machine lost | CP-T9.7 | RQ1 | RQ2 to RQ8 |
| [RQ10 Fast distribution](phase-10-fast-distribution.md) | Lazy loading, chunks, working sets, peers | CP-T9.6 | RQ3, RQ9 | RQ4 to RQ8 |
| [RQ11 Kiln's artifacts](phase-11-kiln-artifacts.md) | Upgrades by delta; CI caches and artifacts; one store | CP-T9.8 | RQ7, RQ9, RQ10 | RQ12 |
| [RQ12 Regions](phase-12-regions.md) | Replication; pulls in-region; a region lost | CP-T9.9 | RQ8, RQ9 | RQ10, RQ11 |
| [RQ13 Scale and release](phase-13-scale-release.md) | Hex's scale simulated, drills, the release, the comparison, production | CP-T9.10 | all | |

```
RQ0 ─ RQ1 ─┬─ RQ2 ──┬─ RQ5 ─────────────────────────┐
           ├─ RQ3 ──┼──────────┬─ RQ10 ─┬─ RQ11 ────┤
           ├─ RQ4 ──┼──────────│────────│───────────┤
           ├─ RQ6 ──┼──────────│────────│───────────┼─ RQ13
           ├─ RQ7 ──┼──────────│────────┘           │
           ├─ RQ8 ──┼──────────│─────── RQ12 ───────┤
           └─ RQ9 ──┴──────────┴─────── (RQ12) ─────┘
```

## Reading an item

```
### RQ1.3 Blob uploads
Needs: RQ1.1, RQ1.2 · Serves: AC-T9.1.2      what it waits for; which of kiln's criteria it serves

A paragraph: what to build, in the words an engineer would use.

Done when:                                    acceptance criteria, each checkable
- AC-RQ1.3.1 ...
Tests:                                        each "do X → expect Y", tagged by what it exercises
- T-RQ1.3.1 [happy] ... → ...
```

IDs follow kiln's own: phase `RQn` beside `CP-T9.n`, item `RQn.m`, criterion `AC-RQn.m.k` beside
`AC-T9.n.k`, test `T-RQn.m.k` beside `T-T9.n.k`. A criterion that restates one of kiln's ends with
its ID in parentheses. Code, commits, tests and benchmarks cite these IDs.

Test tags say which behavior a test is for:

| Tag | It shows |
|---|---|
| `happy` | the thing works as meant |
| `refusal` | what must not be accepted is refused, typed |
| `edge` | boundaries: empty, one, exactly at a limit, the largest |
| `race` | two things at once end in a state some order of them explains |
| `deadlock` | nothing ends with work waiting and nothing running |
| `livelock` | contenders that keep retrying all finish or are refused |
| `exhaustion` | a resource at its bound gives a typed refusal, and service returns when it lifts |
| `crash` | power cut or kill at a write: nothing acknowledged is lost, nothing partial is seen |
| `failure` | a dependency slow, wrong or gone |
| `hostile` | input or a peer built to break it |
| `model` | the implementation agrees with a reference or a model checker |
| `perf` | a recorded measurement under RQ0.7's rules |
| `non-vacuity` | the tests above really exercised what they claim, by counter |

## What "ten times" means

The person's bar is an order of magnitude over every competitor on every key metric. It is judged
as [Phase 4](phase-04-speed.md) states: for each metric and each point of the comparison's matrix,
Reliquary's interval lies wholly beyond a tenth (latency, cost per operation, footprint) or ten times
(throughput) of the best competitor's median at that point; where the best competitor already sits at
the hardware's limit measured that day (the link, the device's durable bandwidth, one core's
SHA-256), the bar is to sit there too, and the record says which bar applied. Kiln's own criterion,
pull p99 no worse than CNCF Distribution's (AC-T9.10.2), is the floor beneath it.

## The plan's scale sizes

Tests that must show a cost does not grow with what the registry holds run at three sizes. None is
picked:

| Size | What it is | From |
|---|---|---|
| first | 100,000 artifacts | kiln's AC-T9.3.2 |
| second | the registry a fleet of 10,000 nodes running a million workloads draws on | Hex's stated target (hex `docs/design/README.md`), with the artifact count taken from RQ10.1's recorded mix |
| third | the largest public registry's published size: Docker Hub's last first-party repository count, 7 million (2020), at its present 20 billion pulls a month | research/05 F48 |

## What the plan asks of other owners

| Of | Request | First needed by |
|---|---|---|
| Mantle | An in-process node entry point (a node opened by a call, reached through the loopback peer) | RQ1.8 |
| Mantle | A grant its native door honors: one principal, one object, one method, one length, one life | RQ3.5 |
| Mantle | A grant's expiry judged against the latest wall time its node has read, so a clock stepped back never brings back a grant it refused as expired (`docs/design/sim.md` W9) | RQ3.5 |
| Mantle | Its measurement rounds as a crate others link | RQ0.7 |
| Amphora | What it provides across regions, and the records' bounds Reliquary must fit | RQ1.2, RQ12.1 |
| Amphora | Amphora on kiln's one hyper-raft: its change to hyper-durable (`Replica::suspect_held`, `trust_held`) upstream, or Reliquary's change to its store (`vendor/patches/amphora/0002`) taken, so Reliquary's copy carries no change of its own; and its `scripts/vendor.py` made kiln's (`vendor.py check` says it isn't, at `db991a05`) | RQ1.8 |
| kiln | Amphora vendored at a revision Reliquary's pin then follows; until then Reliquary holds Amphora's pins to kiln's hyper-raft by a recorded change | RQ1.8 |
| kiln | `scripts/vendor.py <project> --abort` working on Windows: its `shutil.rmtree` of the kept repository is refused git's read-only object files (`PermissionError`, WinError 5), so a stopped re-vendor can't be given up there; until kiln's copy does, Reliquary's test of it records a skip on Windows (`t_rq0_1_5_a_stopped_re_vendor_is_given_up_with_abort`) | RQ0.1 |
| Shards | A skill artifact type beside agent, harness and mcp | RQ5.1 |
| Shards | `reliquary-client` as the transport for its registry commands and `… FROM` directives; where an image carries its Agentfile | RQ3.7, RQ2.6 |
| Shards and Hex | How a guest maps a lazily filled disk on each host platform | RQ10.4 |
| Hex | The image-store seam in its `-api`; the Firecracker pair's artifact shape | RQ3.8, RQ2.5 |
| Veil | Its `-api`: leased upstream credentials and signing without holding a key | RQ8.6, RQ7.3, RQ7.10, RQ7.13 |
| kiln | `Artifacts` on Reliquary behind a setting; the registry's move; runners' mirrors; the consumers of each phase | RQ1.10, RQ2.9, RQ8.9 and each phase's last item |
| kiln | Its identity behind the `Identity` seam: workload tokens, machine credentials and people's sign-in when Reliquary is embedded | RQ7.11, RQ7.12 |
| Quiver | Topics for Reliquary's events, with the delivery guarantee RQ7.13 states | RQ7.13 |
| kiln's Observe owner | Metric and event names | RQ0.9 |
| The person | New lines for CLAUDE.md's Gates list; every push, merge and deploy | RQ0.3, RQ13.8 |

## Open decisions the plan leaves to data

Each is made by the item named, by an A|B record, and written into `docs/design/` with what would
falsify it.

| Decision | Candidates | Made by |
|---|---|---|
| How a manifest read is kept fresh | no cache; change-feed invalidation with a freshness proof; revision-validated | RQ4.4 |
| Cache eviction | the rules the literature offers, on the replayed trace | RQ4.3, RQ4.4 |
| Redirect or relay for a blob | per request, from measured path costs | RQ4.5 |
| HTTP/2 as the OCI door's default | HTTP/2 against HTTP/1.1 per workload | RQ4.6 |
| The catalog's index | inverted segments; trigram store; the records store's queries | RQ6.2 |
| Chunking | fixed against content-defined, and the size, per kind of content | RQ10.1, RQ10.3 |
| The lazy device path per host platform | in-process block backend; NBD; the kernel's own on Linux | RQ10.4, RQ10.6 |
| Peer topology | tree against mesh, per kind of transfer | RQ10.7 |
| Upgrade deltas | chunks against per-pair patches | RQ11.1 |
| Relay or direct bytes on the native door | through Reliquary; straight to Mantle by grant | RQ3.5 |
| A share inside the eviction rule | fixed, where our traces show the result insensitive; a downsized simulation on sampled requests; an online climber | RQ4.4 |
| A learned value against the best cited fixed rule | each estimator against the rule its sources measured | RQ0.8 |
| A second request to another holder after the path's learned tail wait (hedging; research/06 F47) | none; a hedge at the measured percentile, its extra load bounded by that percentile | RQ10.7 and RQ8.1, as an A|B inside each; adopted only if it wins |
