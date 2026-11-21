# Observability

What Reliquary tells an operator about itself: every metric it exposes, every event it emits, and
the dashboard rows that draw them (plan RQ0.9). The catalog in code is
`crates/reliquary-core/src/observe/catalog.rs`, and `crates/reliquary-core/tests/observe.rs`
holds this page and the catalog to each other. It fails on a metric or event the catalog has
and this page lacks, or the reverse. It fails on a row that says something of one that the
catalog doesn't, and on a component with no dashboard group. Those checks serve AC-RQ0.9.1 and
AC-RQ0.9.4. The same test holds every name to the conventions below (AC-RQ0.9.2). Why the
conventions are these, with their sources, is in
[`design/observability.md`](design/observability.md). They are agreed with kiln's Observe owner
before a name lands (CLAUDE.md §10).

## Conventions

- **Metric names**: `reliquary_<component>_<quantity>[_<unit>][_total]`, in ASCII snake_case.
  The component is the part of Reliquary that owns the metric, and the dashboard group it is
  drawn in.
- **Units**: base units only, as the name's last word (before a counter's `_total`) and the
  family's `# UNIT`: `seconds`, `bytes`, or `ratio` (0 to 1). A count has no unit. A name with a
  word `ms`, `msec`, `millis`, `milliseconds`, `us`, `usec`, `micros`, `microseconds`, `ns`, `nsec`,
  `nanos`, `nanoseconds`, `mins`, `minutes`, `hours` or `days` fails, told to use `_seconds`.
  `kb`, `kib`, `kilobytes`, `mb`, `mib`, `megabytes`, `gb`, `gib`, `gigabytes`, `tb`, `tib` or
  `bits` fails, told to use `_bytes`. `percent` or `pct` fails, told to use `_ratio`.
- **Types**: a counter's name ends in `_total`, and a gauge's never does. No family ends in a word
  another type's samples take: `total`, `created`, `bucket`, `count`, `sum`, `info`, `gcount`,
  `gsum`.
- **Labels**: values from a closed set Reliquary states in code, each listed below, never a value
  a client chooses. A label named for what a request carries fails: `repository`, `repo`, `name`,
  `namespace`, `tag`, `reference`, `digest`, `blob`, `manifest`, `upload`, `session`,
  `principal`, `user`, `account`, `subject`, `client`, `token`, `address`, `ip`, `peer`, `path`,
  `url`, `media_type`, `artifact_type` or `user_agent`. Reliquary takes none of `job`, `instance`,
  `host`, `db`, `le` or `quantile`, nor a name beginning with `_`: the scraper, the host and the
  format own those. Every series of every metric is exported from the start, at zero. A new value
  is agreed with kiln's Observe owner first.
- **Exposition**: OpenMetrics 1.0.0 text, `application/openmetrics-text; version=1.0.0;
  charset=utf-8`. Every family has its `# TYPE`, its `# UNIT` when it has one, and its `# HELP`.
  `# EOF` comes last, and there are no `_created` samples.
- **Events**: emitted through `tracing` on target `reliquary.<component>`, each with an `event`
  field named `<component>.<what>`. The level is `error` for a failure a person must look at,
  `warn` for something let go or refused, and `info` for a change of state. Field names are
  snake_case, and a quantity's name is in a base unit. A field holding what a request chose (a
  repository, a digest) never takes a name kiln makes a tag of: `runner`, `from`, `to`, `what`,
  `result`, `reason`, `executor`, `kind`, `status`, `policy`, `repo`, `machine`, `direction` or
  `state`. No field takes a key the event's line already has: `timestamp`, `level`, `target`,
  `event` or `message`.
- **Spans** go on `reliquary.trace.<component>`, off the event targets.
- **Where names are written**: a string beginning `reliquary_` is written only in the catalog,
  and `tracing`'s event and span macros appear only in `crates/reliquary-core/src/observe/`. The
  test fails on either anywhere else under `crates/*/src`, unit tests included: a unit test names
  a metric by its catalog constant. A string beginning `reliquary.` is free elsewhere (a seed
  stream's label, say), since without a macro it emits nothing.

## Metrics

A metric's series are the product of its labels' values. Every series is exported from the
start.

<!-- metrics:begin -->
| Metric | Type | Unit | Labels | Series | Help |
|---|---|---|---|---|---|
| `reliquary_events_emitted_total` | counter | — | — | 1 | Events emitted on Reliquary's targets and offered to the event ring, kept or not. |
| `reliquary_events_dropped_total` | counter | — | `reason`: `evicted`, `oversize`, `closed` | 3 | Events the event ring let go before its sink took them, by reason (evicted: the oldest, to make room for a newer one; oversize: longer than the ring's line bound; closed: emitted after the ring closed). |
| `reliquary_events_held_bytes` | gauge | bytes | — | 1 | Bytes of event lines the event ring holds for its sink now. |
| `reliquary_events_budget_bytes` | gauge | bytes | — | 1 | The most bytes of event lines the event ring holds: its lines times its line bound. |
<!-- metrics:end -->

## Events

Fields are `name: what it holds`: `count` (a whole number), `free` (what a request or the world
chose), or the closed set of values it takes.

<!-- events:begin -->
| Target | Event | Level | Fields | Help |
|---|---|---|---|---|
| `reliquary.events` | `events.dropped` | warn | `evicted`: count; `oversize`: count; `closed`: count | The event ring let events go since its sink's previous batch: the first line of the next batch, with how many for each reason. |
<!-- events:end -->

### Event lines

When Reliquary runs without a host that takes its events, its events wait in the event ring for
a sink, each as one line of JSON (RFC 8259) ended by a newline. The keys are `timestamp` (seconds
since the epoch, to the nanosecond), `level`, `target`, then the event's own fields in the order
they were emitted, `event` and `message` among them. kiln's log reader takes `timestamp`, `level`
and `message` for itself and keeps the rest as attributes. A quotation mark, a reverse solidus
and every control character are escaped, so a line is one line whatever its fields hold.

The ring holds a stated number of lines of at most a stated length, set by its caller. It never
cuts a line. A line offered to a full ring evicts the oldest, so an emitter never waits for the
sink. A line longer than the bound is let go, and so is one offered after the ring has closed.
Each is counted by `reliquary_events_dropped_total`. The sink's next batch begins with an
`events.dropped` line saying how many went for each reason since its previous batch.
Embedded in kiln, kiln's subscriber takes the events and the ring isn't used.

## Dashboards

Each component has a group of panels, given in PromQL (any Prometheus-compatible host) and in
kiln's own queries. kiln carries each group on a built-in dashboard of its own: that is kiln's
change, as Amphora's groups are. Every metric is drawn in at least one panel of its component's
group, in both languages, and every event in at least one of its kiln queries. `$host` is kiln's
template variable for the machines a dashboard shows.

### Reliquary events

<!-- dashboard:events:begin -->
| Panel | PromQL | kiln |
|---|---|---|
| Events emitted/s | `sum(rate(reliquary_events_emitted_total[5m]))` | `sum:reliquary_events_emitted_total{$host}.as_count()` |
| Events dropped/s, by reason, stacked | `sum by (reason) (rate(reliquary_events_dropped_total[5m]))` | `sum:reliquary_events_dropped_total{$host} by {reason}.as_count()` |
| Event ring held, against its budget (bytes) | `max by (instance) (reliquary_events_held_bytes)`, `max by (instance) (reliquary_events_budget_bytes)` | `max:reliquary_events_held_bytes{$host} by {host}`, `max:reliquary_events_budget_bytes{$host} by {host}` |
| Drop markers | — | `logs("@target:reliquary.events @event:events.dropped")` |
<!-- dashboard:events:end -->
