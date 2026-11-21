# Observability: source notes

Researched 2026-11-20 for plan item RQ0.9. Question: what do the primary sources say about naming
metrics, their units and labels; how an exposition is written and read; how kiln, which observes
Reliquary, reads metrics, events and log lines; how kiln's other products agreed their names with
kiln's Observe owner; and what a bounded event buffer that drops its oldest entry can be built on?
Method: each source was saved to `~/.cache/reliquary-research/observability/src-<id>/`. External
pages were fetched with curl on 2026-11-20. Internal files were exported with `git show` at the
revision named. Crates are copied from cargo's registry at the version named. Every `Quote:` line
below was checked by script to be an exact, whitespace-normalised substring of its saved source
(HTML stripped of tags for S6). See "Checking" at the end.

Reading rules: a quote is copied character for character from the saved source, with line breaks
inside it folded to one space. In a Rust source, a comment's markers (`//`, `///`, `//!`) at the
start of a line fold with the line break before them. A code quote is a line of the file named.

## Sources

| id | type | title | URL or place | version/date |
|---|---|---|---|---|
| S1 | standard | OpenMetrics 1.0, `specification/OpenMetrics.md` | https://raw.githubusercontent.com/OpenObservability/OpenMetrics/main/specification/OpenMetrics.md | main, fetched 2026-11-20 |
| S2 | docs | Prometheus, "Metric and label naming" (`docs/practices/naming.md`) | https://raw.githubusercontent.com/prometheus/docs/main/docs/practices/naming.md | main, fetched 2026-11-20 |
| S3 | docs | Prometheus, "Instrumentation" (`docs/practices/instrumentation.md`) | https://raw.githubusercontent.com/prometheus/docs/main/docs/practices/instrumentation.md | main, fetched 2026-11-20 |
| S4 | standard | RFC 8259, "The JavaScript Object Notation (JSON) Data Interchange Format" | https://www.rfc-editor.org/rfc/rfc8259.txt | December 2017 |
| S5 | code | kiln, `crates/kiln-obs/src/prom.rs` and `crates/kiln/src/observe/scrape.rs` | kiln repository | `e64eeb7cb29ba8b49e733f49ee89e37fb96bc923` (origin/main, 2026-11-20) |
| S6 | docs | Linux man-pages, syslog(2) | https://man7.org/linux/man-pages/man2/syslog.2.html | fetched 2026-11-20 |
| S7 | code | kiln, `crates/kiln/src/observe/spans.rs` | kiln repository | `e64eeb7cb` |
| S8 | code | kiln, `crates/kiln/src/log_lines.rs` | kiln repository | `e64eeb7cb` |
| S9 | docs | kiln, `docs/observe.md` | kiln repository | `e64eeb7cb` |
| S10 | design | Amphora, `docs/design/replication.md` | Amphora repository | `db991a05d1769648d1de0390ac80b04c979d150e` (origin/main) |
| S11 | code | concurrent-queue 2.5.0, `src/lib.rs` and `src/bounded.rs` | crates.io, `concurrent-queue` 2.5.0 | 2.5.0 |
| S12 | code | tracing-subscriber 0.3.23, `src/filter/directive.rs` | crates.io, `tracing-subscriber` 0.3.23 | 0.3.23 |

## Findings

### F1. An OpenMetrics family carries its HELP, TYPE and UNIT, a unit is the end of its name, and an exposition that is partly invalid is invalid whole.
- Quote: "A MetricFamily MUST have a name, HELP, TYPE, and UNIT metadata." — S1, "MetricFamily"
- Quote: "If non-empty, it MUST be a suffix of the MetricFamily name separated by an underscore." — S1, "Unit"
- Quote: "Partial or invalid expositions MUST be considered erroneous in their entirety." — S1, "Data transmission & wire formats"
- Quote: "Expositions MUST end with EOF and SHOULD end with 'EOF\n'." — S1, "Overall Structure"
- Quote: "application/openmetrics-text; version=1.0.0; charset=utf-8" — S1, "Overall Structure" (the content type)
- Relevance: the encoder writes every family's metadata, checks every family before it writes any, and ends in `# EOF` (design/observability.md O3). The scrape test reads what it serves with a reader written from S1's ABNF (T-RQ0.9.4).

### F2. A counter's sample is named `_total`; its `_created` is only a SHOULD; and names that look like another type's suffixes are to be avoided.
- Quote: "The MetricPoint's Total Value Sample MetricName MUST have the suffix \"_total\"." — S1, "Counter" (text format)
- Quote: "A MetricPoint in a Metric with the type Counter SHOULD have a Timestamp value called Created." — S1, "Counter" (data model)
- Quote: "Exposers SHOULD avoid names that could be confused with the suffixes that text format sample metric names use." — S1, "Suffixes"
- Quote: "Label names beginning with underscores are RESERVED and MUST NOT be used unless specified by this standard." — S1, "Label"
- Relevance: counters end in `_total`, no family ends in a type's suffix word, and no label begins with `_` (the lint). Reliquary writes no `_created` samples, which kiln skips anyway (F6).

### F3. Prometheus asks for base units, with seconds and bytes as the names' last word, a count ending in `total`, and a share as a ratio from 0 to 1.
- Quote: "...SHOULD use base units (e.g. seconds, bytes, meters - not milliseconds, megabytes, kilometers)." — S2, "Metric names"
- Quote: "Note that an accumulating count has `total` as a suffix, in addition to the unit if applicable." — S2, "Metric names"
- Quote: "Values are 0–1 (rather than 0–100)." — S2, "Base Units" (the row for Percent, whose base unit is `ratio`)
- Quote: "To avoid confusion combining different metrics, always use _bytes_, even where _bits_ appear more common." — S2, "Base Units"
- Relevance: `Unit` holds `seconds`, `bytes` and `ratio` alone, and the lint refuses `_ms` and the like, naming the base unit to use (T-RQ0.9.3).

### F4. Prometheus warns against labels of many values, naming user IDs and unbounded sets, and against missing series.
- Quote: "Do not use labels to store dimensions with high cardinality (many different label values), such as user IDs, email addresses, or other unbounded sets of values." — S2, "Labels"
- Quote: "As a general guideline, try to keep the cardinality of your metrics below 10, and for metrics that exceed that, aim to limit them to a handful across your whole system." — S3, "Do not overuse labels"
- Quote: "To avoid this, export a default value such as `0` for any time series you know may exist in advance." — S3, "Avoid missing metrics"
- Relevance: a label's values are a closed set in code, never a repository, tag, digest or principal (AC-RQ0.9.2). Each metric's series count is stated in docs/observability.md, and every series is exported from the start.

### F5. kiln's scrape asks for OpenMetrics first, as Prometheus's does, and reads it.
- Quote: "A scrape asks for OpenMetrics or the text format, as Prometheus's `Accept` does, gzip allowed," — S5, `scrape.rs`
- Quote: "const ACCEPT: &str = \"application/openmetrics-text;version=1.0.0;q=0.5,application/openmetrics-text;version=0.0.1;q=0.4,text/plain;version=0.0.4;q=0.3,*/*;q=0.1\";" — S5, `scrape.rs`
- Quote: "OpenMetrics (`application/openmetrics-text`, OpenMetrics 1.0) reads alike, but for what it changes" — S5, `prom.rs`
- Relevance: Reliquary serves OpenMetrics 1.0.0 text, and the scrape test sends kiln's own `Accept`.

### F6. kiln skips a `_created` sample.
- Quote: "a counter's, histogram's or summary's `_created` sample is its creation's time, not a value: skipped (as Prometheus skips it by default)." — S5, `prom.rs`
- Relevance: a `_created` sample would cost every scrape a line its reader drops, so Reliquary writes none.

### F7. kiln makes an event of each event on a listed target that names itself with an `event` field, keeps ids as attributes, and makes tags only of a fixed list of field names.
- Quote: "And each event of those targets that names itself (an `event` field:" — S7, module doc
- Quote: "An event without one is the log's alone, so a target's own chatter stays out." — S7, module doc
- Quote: "Fields that become tags: few values each. Any other (an id, a path, an error's words) is an attribute of the event alone." — S7, `TAGS`
- Quote: "pub const TAGS: &[&str] = &[\"runner\", \"from\", \"to\", \"what\", \"result\", \"reason\", \"executor\", \"kind\", \"status\", \"policy\", \"repo\", \"machine\", \"direction\", \"state\"];" — S7
- Quote: "`kiln.<span name>.duration` (seconds of wall time from its creation to its close; a gauge, one" — S7, module doc
- Relevance: events go on `reliquary.<component>` with an `event` field. A field holding a request's value never takes a TAGS name. Spans stay off the event targets, since a listed target's spans become `kiln.<span>` metrics (O4).

### F8. kiln's own log-line queue takes no lock, lets a line go at its bound, counts it, and says how many as its writer next writes; its bounds are cited.
- Quote: "Pushing takes no lock and makes no call" — S8, module doc
- Quote: "A line past the queue's bound ([`LINES`], [`LINES_BYTES`]) is let go and counted (`kiln_log_lines_dropped_total`), and the writer says how many as it next writes." — S8, module doc
- Quote: "Past that journald drops a service's lines itself, so a queue holding more would hold only lines it drops." — S8, `LINES`
- Relevance: the event ring follows the same shape (lock-free, counted, said in the stream) with the plan's rule, the oldest let go, where kiln and journald let the newest go (design/observability.md O5, "Against").

### F9. kiln reads a JSON log line's message, level and time for itself, telling seconds from finer units by their size, and keeps the rest as attributes.
- Quote: "A JSON line's message, level (a name, pino's numbers, syslog's), time (RFC 3339, or seconds, milliseconds, microseconds or nanoseconds since the epoch, told apart by their size), trace and span are its record's; its other fields its attributes, nested keys joined by dots." — S9, §8.1
- Relevance: an event line's keys are `timestamp` (seconds since the epoch), `level` and `message`, then `target`, `event` and the fields.

### F10. Amphora's names were agreed with the Kiln Metrics agent by keyed message, recorded with the date, and its dashboard rows are written in PromQL and in kiln's queries.
- Quote: "The names are the Kiln Metrics agent's agreement (kiln agent 01a14bf7, message key `amphora-class-metrics/names/v1`, 2026-11-06)" — S10, "Metrics"
- Quote: "Dashboard row **Amphora replicated class**, one panel each, in PromQL (any host's form) and in kiln's own queries." — S10, "Metrics"
- Quote: "spans stay off that target, as kiln makes `kiln.<span>` metrics of a listed target's spans." — S10, "The replicated application floor", "Metrics"
- Relevance: Reliquary's agreement is asked and recorded the same way (AC-RQ0.9.3), and its dashboard groups take the same two languages (AC-RQ0.9.4).

### F11. concurrent-queue's `force_push` on a full bounded queue displaces the head, the oldest item, and returns it; it fails only on a closed queue.
- Quote: "If the queue is full, one item from the queue is replaced with the provided item. The displaced item is returned as `Some(T)`." — S11, `lib.rs`, `force_push`
- Quote: "If the queue is closed, an error is returned." — S11, `lib.rs`, `force_push`
- Quote: "let head = tail.wrapping_sub(self.one_lap);" — S11, `bounded.rs`, `force_push` (the slot taken is the one a lap behind the tail: the head's)
- Quote: "If the capacity is zero, this constructor will panic." — S11, `lib.rs`, `bounded`
- Relevance: the ring is `bounded(lines)` with `force_push`. Zero lines is refused before the constructor is called. The unit and flood tests check that the oldest goes (T-RQ0.9.5).

### F12. The kernel's own log buffer is cyclic: it keeps the newest messages.
- Quote: "The kernel has a cyclic buffer of size LOG_BUF_LEN in which messages given as arguments to the kernel function printk() are stored (regardless of their log level)." — S6, "The kernel log buffer"
- Relevance: a buffer that lets its oldest go has long practice behind it (O5).

### F13. JSON requires a quotation mark, a reverse solidus and the control characters below U+0020 escaped inside a string.
- Quote: "All Unicode characters may be placed within the quotation marks, except for the characters that MUST be escaped: quotation mark, reverse solidus, and the control characters (U+0000 through U+001F)." — S4, §7
- Relevance: the line writer escapes these (and the other control characters, which §7 allows), so a line is one line whatever its fields hold.

### F14. A tracing target directive matches every target its text begins.
- Quote: "if !meta.target().starts_with(&target[..]) {" — S12, `StaticDirective::cares_about`
- Relevance: listing `reliquary.events` takes `reliquary.events.anything` too, so spans go on `reliquary.trace.<component>`, which no event target begins.

### F15. kiln keeps a log's other fields as `@`-addressable attributes, and a search names one as `@key:value`.
- Quote: "`attributes` (a flat map of `@`-addressable fields)" — S9, §3 "Logs"
- Quote: "A value list in a search, `service:(web OR api)` or `@evt.name:(a OR \"b c\")`, is the key's condition on each value" — S9, §6
- Relevance: a collected event line's `target` and `event` are attributes (F9), so the drop-marker panel searches `@target:reliquary.events @event:events.dropped`.

## Not verified

- How often the first lines of a flood carry its cause: no source measured it. This is the case
  against letting the oldest go (O5, "Against").

## Checking

`python3 -I ~/.cache/reliquary-research/observability-tools/check_quotes.py docs/research/09-observability.md`
checks each `Quote:` above against its saved source. The record of its run is in
`docs/design/observability.md`, "Evidence".
