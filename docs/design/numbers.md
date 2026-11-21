# The rule on numbers

Status: design, 2026-11-20; the gate built (plan RQ0.2). This is Mantle's rule and Mantle's gate (mantle
`docs/design/constants.md`, research/01 §5), taken whole and made stricter where Reliquary starts with no
numbers. The inventory is [`constants.md`](constants.md); its checker is `scripts/check-contracts.py`.

## The rule

The person's words: "We DO NOT TOLERATE ARBITRARY CONSTANTS, and enforce learned and data derived
approaches. This DOES NOT mean 'data derived' approaches learned from a specific machine that are
effectively a constant. We do NOT assume hardware or underlying machine." And, to Mantle on
2026-11-12: "Learned and derived approaches or exceedingly well researched ONLY."

Every number Reliquary's production code runs with has one row in `docs/design/constants.md`: its
file and key, its kind, its value, and what establishes it.

A kind the rule accepts:

| Kind | What it is |
|---|---|
| format | a field width, tag, version or length of an on-disk or wire encoding; changing it changes the format |
| external | fixed by a specification, an interface, or a stated property of a system Reliquary must meet |
| derived | computed by a stated model from measured inputs or structural facts |
| cited | prescribed by peer-reviewed work or a primary source, used as it prescribes |
| learned | measured by the running system; the row names what it measures, on what, where it is kept and how it follows change |
| counted | bytes or items a structure allocates, counted and pinned by a test: a property of the code, not of a machine |

A kind the rule refuses, which fails the gate:

| Kind | What it is |
|---|---|
| picked | set by hand, or a bound or floor taken as the operating value |
| knee | read off one measured curve by a chosen criterion and frozen |
| once | measured once, on one machine |
| copied | another system's tuning |
| guessed | a guessed timeout, deadline or wait |
| inherited | derived from a refused input |
| open | nothing establishes it yet |

Reliquary started with no numbers, so nothing is forgiven: `scripts/arbitrary-open.txt` stays empty
and the gate refuses an item in it. What the gate reads, and each way it fails, is in
[`constants.md`](constants.md).

## What replaces a constant, by class of tunable

Each class below is where a registry usually keeps a constant. Each row says what Reliquary has in its
place, the estimator or model, and where that model's inputs are measured. A learned value is kept
under what it describes, a path, a device, a peer, a repository or a workload class, never the machine
alone (AC-RQ0.8.4). crates/reliquary/tests/numbers.rs holds these tables and the lists after them to the
plan and to the code.

### Waits

| What | Usually | In Reliquary | Its inputs, and where they are measured | Kind | Kept under | Plan |
|---|---|---|---|---|---|---|
| How long to wait for an answer or the next byte | a timeout | liveness and progress at a cold start and after a run-out; then the slowest answer of its kind the path has shown | each call's time to its answer and between its bytes, and the other party's signs of life, measured by the caller as the call runs | learned | the path (peer and route) and the kind of call | RQ0.8 |
| How long a connection or handshake may take | a timeout | RFC 6298's estimator on that path's own samples, doubled per timeout in a row | each handshake's round trip, measured by the side that connects | cited, learned | the path | RQ0.8 |
| How many retries | a count | none of its own: a refusal that can pass is asked again while the trouble passes within what that path's troubles have taken; a refusal that cannot is the caller's | how long each of the path's troubles lasted, measured by the caller from the first refusal to the next answer | learned | the path | RQ0.8 |
| When to retry (`Retry-After`) | a fixed delay | the queue's measured drain | the bottleneck's queue and its completions a second, measured where it admits | derived | the bottleneck | RQ4.8 |
| Webhook and scanner waits and retries | a timeout, a retry count, a backoff | the receiver's own answers, then RQ0.8's waits; a backlog is a durable cursor, not a queue | the receiver's answers (status, `Retry-After`) and its signs of life, measured by the sender | learned | the peer (the receiver) | RQ7.13, RQ7.15 |

### Concurrency

| What | Usually | In Reliquary | Its inputs, and where they are measured | Kind | Kept under | Plan |
|---|---|---|---|---|---|---|
| How many requests to admit | a limit | the point of greatest power, Kleinrock's throughput over delay, probed | completions a second and their delay at each limit probed, measured at the bottleneck's admission | learned | the device or peer that is the bottleneck, on its node | RQ0.8, RQ4.8 |
| How many transfers at once | a pool size | as many as pay on the path, climbed and held | the path's throughput at each count of transfers, measured by the side that transfers | learned | the path, pulls and pushes apart | RQ3.6 |
| HTTP/2 abuse thresholds | counts and rates of resets | none: handlers running are bounded by the advertised stream limit until each has stopped, whoever reset the stream | the connection's own streams, counted as they open and stop | counted | the connection | RQ4.6 |

### Sizes of chunks, pieces and windows

| What | Usually | In Reliquary | Its inputs, and where they are measured | Kind | Kept under | Plan |
|---|---|---|---|---|---|---|
| Upload piece, part and run sizes | a size | Mantle's plan for the upload; the size that costs least a byte on the path | Mantle's plan, from its interface; the time a byte costs at each size tried, measured by the uploader | external, learned | the path | RQ1.3, RQ3.6 |
| Fetch size for a range read | a block size | the cost model on the path's rate and round trip, over a tree that fixes none | the path's rate and round trip, measured by the reader | derived | the path | RQ10.2, RQ10.4 |
| Chunk size and cut rule for stored recipes | a size | per kind of content, from measured sharing and index cost; recorded in each recipe | how much stored content of each kind shares at each cut rule, and what the index costs a chunk, measured over that content | derived | the recipe | RQ10.1, RQ10.3 |
| Largest manifest | 4 MiB | the memory one request is admitted with, never below the specification's 4 megabytes | the memory the process is allowed, measured at start; what admission grants one request | derived, external | the node's budget | RQ1.5 |
| How much a packed entry may unpack to | a ratio or a size cap | for deflate, at most 1032 times its packed size, which the format cannot exceed; the least of that, the declared size and the admitted budget | the entry's packed and declared sizes, read from the archive; the budget admission grants | derived, cited | the entry | RQ2.2 |
| HTTP/2 and QUIC windows | sizes | the connection's memory charge and the path's measured bandwidth and delay | the path's rate and round trip, measured on the connection; the memory admission charges it | derived | the connection | RQ4.6 |
| Redirect or relay a blob | a size threshold | per request: size, the path's rate, what a new connection costs that client | the blob's size; the client's path's rate and connection cost, measured by the node | derived | the client's path | RQ4.5 |
| Working set | a fraction | the blocks recorded starts read in common | the blocks each start of an image and command reads, recorded by the lazy loader | learned | the workload class: an image and its command | RQ10.5 |

### Batches

| What | Usually | In Reliquary | Its inputs, and where they are measured | Kind | Kept under | Plan |
|---|---|---|---|---|---|---|
| Group commit | a timer and a batch size | what waited while the last flush ran | the writes that arrived during the last flush, counted by the committer | derived | the committer | RQ4.7 |
| Benchmark rounds, samples, confidence | counts | Mantle's measurement design: counts from the quantile reported and the interval asked; repetitions at each level (build, process start, round) from the variation measured there, so no "best of N" has a picked N | the variation measured at each level of the run | cited, derived | the record | RQ0.7 |

### Queues and caches

| What | Usually | In Reliquary | Its inputs, and where they are measured | Kind | Kept under | Plan |
|---|---|---|---|---|---|---|
| Queue lengths | a size | what its callers can still use: service time against their deadlines | the queue's service time, measured as it serves; each caller's deadline, carried by its request | derived | the queue | RQ4.8 |
| Cache sizes | a size | a share of the memory the OS allows, after the floor that admits one unit of each class | the memory the process is allowed, measured at start and when it changes; the size of one unit of each class | derived | the node's budget | RQ1.9, RQ4.4 |
| Cache eviction | an age or a count | a rule chosen by A\|B on the replayed trace; for cached images, hit chance against bytes freed and refetch cost | the node's own request trace, replayed; each upstream's refetch cost, measured as it fetches | learned | the repository: its artifacts' own pull history | RQ4.4, RQ8.8 |
| A share inside a cache's eviction rule | a percentage | fixed only where the record shows the result insensitive across the range tested on our traces; otherwise learned by the learner that wins its A\|B | the replayed trace, as above | cited, learned | the workload class the trace is of | RQ4.4, RQ0.8 |

### Rates

| What | Usually | In Reliquary | Its inputs, and where they are measured | Kind | Kept under | Plan |
|---|---|---|---|---|---|---|
| How fast to ask an upstream | a rate | the upstream's own stated limits and `429` answers | the upstream's rate-limit headers and `429`s with `Retry-After`, read by the puller | external, learned | the peer (the upstream) | RQ8.1 |
| Background work (scrub, reconcile, fill, mirror watch) | a period | what spare capacity and the budget's remainder allow, yielding to requests | the node's spare capacity, measured at admission; the budget's remainder | derived | the node's budget | RQ9.4, RQ9.9, RQ10.4, RQ8.7 |

### Lifetimes

| What | Usually | In Reliquary | Its inputs, and where they are measured | Kind | Kept under | Plan |
|---|---|---|---|---|---|---|
| Token lifetime | minutes | the operator's statement; else the token specification's default, never below its floor | stated by the operator | external | the configuration | RQ1.7 |
| Grant and presigned URL lifetime | minutes | the transfer's bytes over the path's learned rate, renewed while it progresses | the transfer's size; the path's rate, measured by the node | derived | the transfer | RQ3.5, RQ4.5 |
| How fresh a cached tag is | a TTL | checked on every pull, pulls sharing a check, unless the operator states a window | stated by the operator, when stated | external | the configuration | RQ8.3 |
| How stale a signed tag statement may be | a TTL | the operator's statement of how stale a tag may be (AC-RQ7.3.4) | stated by the operator | external | the configuration | RQ7.3 |
| Machine credential and session lifetimes | days | the operator's statement, which must be given; an exchanged token ends with the token it came from | stated by the operator | external | the configuration | RQ7.11, RQ7.12 |
| Abandoned upload sessions | an age | reclaimed when their room is needed, least recently progressed first | each session's last progress, recorded as it writes; the room its opener's quota has left | derived | the opener's quota | RQ1.3 |
| Grace before collecting an unreferenced blob | hours | the longest gap that repository has shown between a blob's assertion and its manifest, extended while its pusher shows progress | the time from each blob's upload to the manifest that names it, measured where manifests commit | learned | the repository | RQ9.4 |

## What an operator states, what a node measures, what it derives

The operator states only what no node can measure. There is no setting for a timeout, a retry count,
a queue length, a cache size, a chunk size or a concurrency (AC-RQ1.9.2). A node measures the rest and
derives every budget and bound from what it measured, and `reliquary doctor` prints all three with their
sources (AC-RQ1.9.3). In each list, "In code" names the field, type or function that holds the item once
it is built (`` `file` `name` ``), and "—" until then.

The configuration type: none yet. The `reliquary` binary takes no setting until `reliquary serve`
(RQ1.9); this line then names its type, and the operator's list is held to its fields.

### What an operator states

| Setting | What it is | Plan | In code |
|---|---|---|---|
| `listen` | where each door listens | RQ1.9, RQ1.6 | — |
| `data` | where data lives | RQ1.9 | — |
| `grants` | who may do what: grants and the policies a pull is decided by | RQ1.7, RQ7.4 | — |
| `survive` | how many failures to survive | RQ1.9, RQ9.1 | — |
| `memory-ceiling` | optionally, a ceiling on memory, below what the OS allows | RQ1.9 | — |
| `regions` | regions and the rules for what replicates between them | RQ12.1 | — |
| `upstreams` | upstream registries, how clients' names map to them, and how fresh their tags must be | RQ8.2, RQ8.3 | — |
| `upstream-credentials` | which credentials reach a private upstream, by name in the `Secrets` seam | RQ8.6 | — |
| `mirrors` | which repositories are mirrored | RQ8.7 | — |
| `quotas` | quotas | RQ9.5 | — |
| `retention` | retention and immutability | RQ9.6 | — |
| `token-lifetime` | how long a token lives | RQ1.7 | — |
| `credential-lifetimes` | how long machine credentials and sessions live, which must be given | RQ7.11, RQ7.12 | — |
| `statement-lifetime` | how stale a signed tag statement may be | RQ7.3 | — |
| `trust` | trust rules for issuers and signers, and the rule for signing on push | RQ7.1, RQ7.2, RQ7.10 | — |
| `notify` | who is told of which changes | RQ7.13 | — |
| `audit` | how long the audit trail is kept | RQ7.14 | — |
| `scanners` | which scanners a namespace's images go to | RQ7.15 | — |

### What a node measures

| Measurement | On what, and where it is measured | Plan | In code |
|---|---|---|---|
| cores | the cores the OS grants the process, its affinity and its CPU quota, read at start | RQ1.9 | — |
| memory | the memory the process is allowed, its cgroup or job limit under the machine's, read at start and when it changes | RQ1.9 | — |
| devices | each device data lives on, through Mantle's probe | RQ1.9 | — |
| answers | each call's time to its answer and between its bytes, and the other party's signs of life, by path and kind of call, measured by the caller | RQ0.8 | — |
| handshakes | each handshake's round trip, by path, measured by the side that connects | RQ0.8 | — |
| troubles | how long each of a path's troubles lasted, measured by the caller | RQ0.8 | — |
| power | each bottleneck's completions a second and their delay at each limit probed, measured at its admission | RQ0.8, RQ4.8 | — |
| service | each queue's service time, measured as it serves | RQ4.8 | — |
| rate | each path's rate and round trip, measured on its transfers | RQ0.8 | — |
| transfers | each path's throughput at each count of transfers, and the time a byte costs at each piece size, measured by the side that transfers | RQ3.6 | — |
| gaps | each repository's gaps between a blob's upload and the manifest that names it, measured where manifests commit | RQ9.4 | — |
| upstream limits | each upstream's stated limits and `429` answers, read by the puller | RQ8.1 | — |
| trace | the node's own request trace, kept to replay for eviction's A\|B | RQ4.4, RQ8.8 | — |
| starts | the blocks each start of an image and command reads, recorded by the lazy loader | RQ10.5 | — |
| sharing | how much stored content of each kind shares at each cut rule, and what the index costs a chunk | RQ10.1 | — |
| spare | the node's spare capacity, measured at admission | RQ4.8, RQ9.9 | — |
| variation | a benchmark's variation at each level: build, process start and round | RQ0.7 | `crates/reliquary-bench/src/levels.rs` `level_variances` |

### What a node derives

| Value | Its model, from what was stated and measured | Plan | In code |
|---|---|---|---|
| budgets | each cache's share of the memory the process is allowed, after the floor that admits one unit of each class | RQ1.9, RQ4.4 | — |
| queue lengths | service time against the callers' deadlines | RQ4.8 | — |
| `Retry-After` | the queue's measured drain | RQ4.8 | — |
| largest manifest | the memory one request is admitted with, never below the specification's 4 megabytes | RQ1.5 | — |
| retransmission timeout | RFC 6298's estimator on the handshakes' round trips | RQ0.8 | — |
| fetch size | the cost model on the path's rate and round trip | RQ10.2 | — |
| cut rule | per kind of content, from its sharing and the index's cost | RQ10.3 | — |
| redirect or relay | per request, from the blob's size, the path's rate and a connection's cost | RQ4.5 | — |
| grant lifetime | the transfer's bytes over the path's rate, renewed while it progresses | RQ3.5, RQ4.5 | — |
| group commit | what waited while the last flush ran | RQ4.7 | — |
| background rate | spare capacity and the budget's remainder | RQ9.9 | — |
| windows | the connection's memory charge and the path's bandwidth and delay | RQ4.6 | — |
| unpack bound | the least of 1032 times the packed size, the declared size and the admitted budget | RQ2.2 | — |
| reclaim order | upload sessions by least recent progress, when their room is needed | RQ1.3 | — |
| benchmark counts | rounds and repetitions from the interval asked and the variation measured | RQ0.7 | `crates/reliquary-bench/src/levels.rs` `plan` |

## Numbers that are specifications'

Listed in the inventory as external or format, each with its source in research/02: a tag's 128
characters; SHA-256's 64 and SHA-512's 128 hex digits; the 4 megabytes a manifest push must be able
to reach; the 4,096 bytes of warnings; a token's 60-second default and floor; a skill's 64, 1,024
and 500 character limits; the fourteen error codes and their status codes. Which region wins a tie
is a fixed order of regions, part of the replication format (RQ12.4).

## Numbers in the plan's criteria

A criterion's threshold is a requirement, not a tuning. Each is one of: kiln's own criterion, cited
by its ID (10 ms, 50 ms, 1 s, 70%, 3 image sizes, 30%, 0); a specification's; or the person's
order-of-magnitude bar, judged as `plan/README.md` states. A test's inputs are stated against
derived bounds ("more headers than the bound") and not as figures.
