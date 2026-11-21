# Observability: the conventions and their decisions

Plan RQ0.9 (serves CP-T9.1). This file decides how Reliquary tells an operator about itself
(CLAUDE.md §10). The list of every metric, event and dashboard row is
[`../observability.md`](../observability.md), and `crates/reliquary-core/tests/observe.rs` holds
that list and the code to each other. The sources are [research/09](../research/09-observability.md),
cited below as F1 to F15. kiln's Observe owner agrees the names before they land; who that is,
and the agreement of 2026-11-20, are at the end.

## O1. Names in base units, a component's prefix, a type's suffix

**Chosen.** Every metric is `reliquary_<component>_<quantity>[_<unit>][_total]` in ASCII
snake_case. The component is the part of Reliquary that owns the metric and the dashboard group
that draws it. Units are base units only (`seconds`, `bytes`, `ratio` from 0 to 1, none for a
count), as the family name's last word and its `# UNIT`. A counter ends in `_total`. No family
ends in a word another type's samples take. `catalog::Unit` holds the three base units alone. The
lint in `tests/observe.rs` refuses a name with a word that is a unit but not a base one, naming
the base unit to use (T-RQ0.9.3). It also refuses a declared unit the name doesn't end in, a
counter without `_total`, a gauge with it, and a name outside its component's prefix.

**Rests on.** Prometheus's naming practice (F3: base units, `total` on an accumulating count,
`ratio` for a share), OpenMetrics' unit and suffix rules (F1, F2), and kiln's own names, which its
scrape reads under their Prometheus spelling (F5).

**Falsified by.** A consumer, kiln's Observe first, that can't read or aggregate a family named
so, or Observe's owner asking for another scheme.

## O2. Labels come from closed sets Reliquary chooses

**Chosen.** A label's values are a closed set declared in the catalog. A series carries them as
`&'static str` words, so text a request brought can't become a value by accident. The encoder
refuses an exposition holding a value outside its label's set, or a series of the wrong arity
(AC-RQ0.9.2). The lint refuses a label named for what a request carries (`repository`, `tag`,
`digest`, `principal` and the rest docs/observability.md lists) and the names the scraper, the
host or the format own (`job`, `instance`, `host`, `db`, `le`, `quantile`, a leading `_`). Each
metric's series count, the product of its labels' value counts, is stated in
docs/observability.md and checked against the catalog, so a label's cost is on the page. Every
series is exported from the start at zero. A new label value is agreed with Observe's owner first,
as Amphora's are.

**Rests on.** Prometheus: "Do not use labels to store dimensions with high cardinality" and
"export a default value such as `0`" (F4). kiln makes tags only of a closed list of field names
(F7). Amphora's closed sets were agreed one value at a time (F10).

**Falsified by.** An operator question that only a per-repository or per-principal series
answers. That goes to an event's free attribute or to the catalog (RQ6), never to a label, so
this decision stands unless the observability host can bound a label's cardinality itself and
Observe's owner asks for one.

## O3. OpenMetrics 1.0.0 text, checked whole before it is written

**Chosen.** `/metrics` serves OpenMetrics 1.0.0 text (`application/openmetrics-text;
version=1.0.0; charset=utf-8`). Every family has `# TYPE`, its `# UNIT` when it has one, and
`# HELP`, in that order. A counter's samples are `<family>_total`, and the exposition ends in
`# EOF`. There are no `_created` samples. `openmetrics::encode` checks every family before it
writes any, and refuses the exposition whole on a family reported twice, a counter named without
`_total`, or a series its catalog entry doesn't declare.

**Rests on.** OpenMetrics: a family's metadata, the unit as suffix, `# EOF`, and "Partial or
invalid expositions MUST be considered erroneous in their entirety" (F1, F2). kiln's scrape asks
for OpenMetrics first and skips `_created` (F5, F6).

**Falsified by.** A reader Reliquary must serve that takes only Prometheus's 0.0.4 text (then the
door negotiates by `Accept`), or one that needs `_created` to see a counter's reset.

## O4. Events on `reliquary.<component>`, spans on `reliquary.trace.<component>`

**Chosen.** Events go through `tracing` on the target `reliquary.<component>`, each with an
`event` field named `<component>.<what>` in kiln's dotted style. The level is `error` (a person
must look), `warn` (something let go or refused) or `info` (a change of state). Field names are
snake_case, with quantities in base units. A field that holds what a request chose is an attribute
and never takes a name kiln makes a tag of; those carry closed sets or counts. No field takes a
key the event's line has of its own. Spans go on `reliquary.trace.<component>`. tracing's macros
appear only in `crates/reliquary-core/src/observe/`, and a `reliquary_` name only in the catalog;
the test fails on either elsewhere, so what the code emits is what the catalog lists.

**Rests on.** kiln's bridge: an event on a listed target that names itself in an `event` field
becomes an Observe event, ids stay attributes, and tags come of a fixed list of names (F7).
Amphora's events on `amphora.floor`, with spans kept off it because kiln makes metrics of a listed
target's spans (F10). A target directive matches every target its text begins, so the spans'
target must not begin with an event target (F14).

**Falsified by.** kiln choosing events another way (by a field, not a target), or Observe's owner
asking for spans on the event targets.

## O5. The event ring: bounded, lock-free, drops the oldest, says what it lost

**Chosen.** When no host takes Reliquary's events, they wait for a sink in `ring::EventRing`, a
`concurrent_queue` of a stated number of lines of at most a stated length, made whole at its
start. Each event is a JSON line (`timestamp` in seconds since the epoch, `level`, `target`, then
its fields), written on the emitting thread and offered with `force_push`. A full ring evicts its
oldest line. A line longer than the bound is let go whole, never cut, and so is one offered after
the ring has closed. Each is counted by reason (`reliquary_events_dropped_total`). The sink's next
batch begins with an `events.dropped` line saying how many went for each reason since its previous
batch. A drain takes at most one ring's worth, so it ends while emitters outpace it. An emitter
takes no lock and never waits for the sink (T-RQ0.9.5).

**Rests on.** The plan's rule: the oldest are dropped (T-RQ0.9.5). `force_push` displaces the
head of a full bounded queue and returns it (F11). kiln's own log queue shows the shape: no lock
on the push, a counted loss, said in the stream as the writer next writes (F8). The kernel's own
log buffer is cyclic (F12).

**Against.** kiln's queue and journald let the newest go at a burst (F8), which keeps a flood's
beginning, where its cause often is. A ring that lets the oldest go keeps the flood's end and
loses its beginning: the marker says how many it lost, not which. No source measured which end
of a flood carries its cause more often (research/09, "Not verified").

**Bounds.** The ring's lines and line bytes are its caller's to state. Its budget is their
product, `reliquary_events_budget_bytes`; it holds at most that many bytes of lines, plus one
queue slot a line allocated at its start. They become rows of numbers.md's stated list with the
shell that makes the ring (RQ1). Embedded in kiln, kiln's subscriber takes the events and the
ring isn't made.

**Falsified by.** A recorded incident whose cause was among a flood's evicted lines. The ring
would then also keep a flood's first lines (a head it never evicts) beside its last: a change to
the plan's rule, not a tuning of the bounds.

## O6. One catalog, one page, one test

**Chosen.** `crates/reliquary-core/src/observe/catalog.rs` declares every metric and event.
`docs/observability.md` lists them and the dashboard groups that draw them, each panel in PromQL
and in kiln's queries. `tests/observe.rs` fails on a metric or event either has and the other
lacks, naming it (AC-RQ0.9.1, T-RQ0.9.2). It fails on a row that says other than the catalog,
naming the column. It fails on a component with no group, a metric drawn in no panel of its
group in either language, an event shown in no kiln panel, and a panel drawing another
component's metric (AC-RQ0.9.4). A component added later ships its rows in the same change or
fails this test.

**Rests on.** CLAUDE.md §10. Amphora's rows in both languages, carried by kiln on a built-in
dashboard as kiln's change (F10). AC-RQ13.5.2 holds this test to enforce dashboards later.

**Falsified by.** A metric or event reaching an operator that the test didn't see: a name built
at run time from parts, or a tracing macro called under another name. Either is a hole in the
scan, closed when found.

## The agreement with kiln's Observe owner (AC-RQ0.9.3)

- **Proposed** on 2026-11-20 to the Kiln Metrics agent (kiln agent
  `01a14bf7c0ce7451a5fd7c6f0a9764d0`), the Observe owner CLAUDE.md §10 names, by kiln message,
  key `reliquary-observability/names/v1` (message
  `from-agent-01a1fa992b4b7212821444f495d867b7-70c4699bc96a8447a2177771bd7bd371`, revision
  18481). It carried O1 to O6 and the `events` component's four metrics, one event and dashboard
  group. It asked whether `@target:`/`@event:` is the right form for a collected line's search,
  and suggested a monitor on evictions. That agent had been paused at its weekly usage limit since
  2026-11-12, marked to be resumed by hand, and the message waited in its queue.
- **Owner named** on 2026-11-20. Asked how to close this criterion with Kiln Metrics paused, the
  person answered: "You’re the owner now. Period." That made this step's agent kiln's Observe
  owner for these names: kiln agent `01a1fa992b4b7212821444f495d867b7`, Reliquary's RQ0.9.
- **Agreed** on 2026-11-20 by that owner: the scheme, O1 to O6, and the `events` component's
  names, help lines, label values, event and dashboard group, as docs/observability.md lists
  them. kiln's own spec settles the proposal's open question: a collected line's fields are
  `@`-addressable attributes (F15), so the drop-marker panel searches
  `@target:reliquary.events @event:events.dropped`. The suggested monitor, any `reason:evicted`
  drop within 15 minutes, is kiln's to carry with the group. Kiln Metrics was told its queued
  message needs no answer (key `reliquary-observability/names/v1-settled`).
- A later component's new names, label values and events are agreed the same way: with Observe's
  owner, recorded here with the date.

## Evidence

On 2026-11-20, on Darwin 25.4.0 arm64 (Adas-MacBook-Pro-2), toolchain 1.99.1, a machine shared
with other builds:
- `cargo test -p reliquary-core --lib observe`: 15 unit tests pass (the catalog, line, ring and
  encoder).
- `cargo test -p reliquary-core --test observe`: 19 pass. They cover the doc-truth test and its
  refusals, the lint and its refusals (T-RQ0.9.1, T-RQ0.9.3), the scan and its fixture, the
  OpenMetrics reader and the formats it refuses, a scrape of a running stub (T-RQ0.9.4), and a
  flood of 4,000 events from four threads into sixteen lines with the sink stalled (T-RQ0.9.5).
  The flood passed 30 runs of 30.
- `cargo clippy -p reliquary-core --all-targets -- -D warnings`, for this host and for
  `x86_64-pc-windows-msvc`, and the production check's `-F` lints: clean.
- research/09's 41 quotes: all found in their saved sources by its check. A copy with one number
  altered and one source id swapped failed on exactly those two.
