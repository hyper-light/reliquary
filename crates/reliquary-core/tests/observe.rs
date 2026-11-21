//! Observability's conventions, held to the code (plan RQ0.9): `docs/observability.md` and the
//! catalog to each other, dashboards included (AC-RQ0.9.1, AC-RQ0.9.4); every name to the
//! conventions (AC-RQ0.9.2); names written only in the catalog; a scrape of a running stub read
//! as OpenMetrics (T-RQ0.9.4); and a flood of events into a stalled sink (T-RQ0.9.5).

#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing,
        clippy::arithmetic_side_effects,
        clippy::disallowed_macros
    )
)]

use std::collections::{BTreeMap, BTreeSet};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::{Arc, mpsc};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use reliquary_core::observe::catalog::{
    self, EVENTS, Event, Field, Kind, Label, METRICS, Metric, Unit, Values,
};
use reliquary_core::observe::layer::RingLayer;
use reliquary_core::observe::openmetrics::{self, CONTENT_TYPE};
use reliquary_core::observe::ring::{Bounds, EventRing};
use tracing_subscriber::Registry;
use tracing_subscriber::layer::SubscriberExt as _;

// ---------------------------------------------------------------------------------------------------
// The conventions, as docs/observability.md states them and docs/design/observability.md decides.

/// Base units, as names end in them (Prometheus, "Metric and label naming", Base Units).
const BASE: &[&str] = &["seconds", "bytes", "ratio"];

/// Words that are units but not base ones, each with the base unit to name it in instead.
const NOT_BASE: &[(&str, &str)] = &[
    ("ms", "seconds"),
    ("msec", "seconds"),
    ("millis", "seconds"),
    ("milliseconds", "seconds"),
    ("us", "seconds"),
    ("usec", "seconds"),
    ("micros", "seconds"),
    ("microseconds", "seconds"),
    ("ns", "seconds"),
    ("nsec", "seconds"),
    ("nanos", "seconds"),
    ("nanoseconds", "seconds"),
    ("mins", "seconds"),
    ("minutes", "seconds"),
    ("hours", "seconds"),
    ("days", "seconds"),
    ("kb", "bytes"),
    ("kib", "bytes"),
    ("kilobytes", "bytes"),
    ("mb", "bytes"),
    ("mib", "bytes"),
    ("megabytes", "bytes"),
    ("gb", "bytes"),
    ("gib", "bytes"),
    ("gigabytes", "bytes"),
    ("tb", "bytes"),
    ("tib", "bytes"),
    ("bits", "bytes"),
    ("percent", "ratio"),
    ("pct", "ratio"),
];

/// Words a family's samples take in OpenMetrics' text, by type, which no family may end in
/// (OpenMetrics 1.0, "Suffixes").
const TYPE_WORDS: &[&str] = &[
    "total", "created", "bucket", "count", "sum", "info", "gcount", "gsum",
];

/// Label names for what a request carries: a client chooses their values.
const REQUEST_LABELS: &[&str] = &[
    "repository",
    "repo",
    "name",
    "namespace",
    "tag",
    "reference",
    "digest",
    "blob",
    "manifest",
    "upload",
    "session",
    "principal",
    "user",
    "account",
    "subject",
    "client",
    "token",
    "address",
    "ip",
    "peer",
    "path",
    "url",
    "media_type",
    "artifact_type",
    "user_agent",
];

/// Label names the scraper (`job`, `instance`), the host (`host`, and kiln's `db`) and the format
/// (`le`, `quantile`) own.
const HOST_LABELS: &[&str] = &["job", "instance", "host", "db", "le", "quantile"];

/// Field names kiln makes tags of (kiln's crates/kiln/src/observe/spans.rs, `TAGS`, at kiln
/// e64eeb7cb): a field of one of these names holds a closed set or a count, never free text.
const KILN_TAGS: &[&str] = &[
    "runner",
    "from",
    "to",
    "what",
    "result",
    "reason",
    "executor",
    "kind",
    "status",
    "policy",
    "repo",
    "machine",
    "direction",
    "state",
];

/// The keys an event's line has of its own (observe::line), which no field may take.
const LINE_KEYS: &[&str] = &["timestamp", "level", "target", "event", "message"];

// ---------------------------------------------------------------------------------------------------
// The lint (AC-RQ0.9.2).

/// ASCII snake_case: lowercase letters, digits and single underscores, beginning with a letter
/// and not ending in an underscore.
fn snake(s: &str) -> bool {
    s.starts_with(|c: char| c.is_ascii_lowercase())
        && s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        && !s.contains("__")
        && !s.ends_with('_')
}

/// A help line: said, on one line, and fit for a table's cell.
fn text_problem(at: &str, text: &str) -> Option<String> {
    if text.trim().is_empty() {
        Some(format!("{at}: its help says nothing"))
    } else if text.contains('\n') || text.contains('|') || text.trim() != text {
        Some(format!(
            "{at}: its help is one line, without `|` or space at either end"
        ))
    } else {
        None
    }
}

/// Words of `name` that are units but not base ones, each told the base unit to use.
fn unit_problems(at: &str, name: &str) -> Vec<String> {
    name.split(['_', '.'])
        .filter_map(|word| NOT_BASE.iter().find(|(not, _)| *not == word))
        .map(|(word, base)| {
            format!("{at}: `{word}` is not a base unit: name it in `_{base}` (docs/observability.md, Units)")
        })
        .collect()
}

/// What the conventions refuse in `metrics` and `events`, each naming what it is about.
fn lint(metrics: &[&Metric], events: &[&Event]) -> Vec<String> {
    let mut problems = Vec::new();
    let mut families = BTreeSet::new();
    for m in metrics {
        let at = format!("`{}`", m.name);
        if !snake(m.name) {
            problems.push(format!("{at}: a name is ASCII snake_case"));
        }
        if !snake(m.component) {
            problems.push(format!(
                "{at}: its component `{}` is ASCII snake_case",
                m.component
            ));
        }
        let prefix = format!("reliquary_{}_", m.component);
        if !m.name.starts_with(&prefix) {
            problems.push(format!(
                "{at}: its name begins `{prefix}`, Reliquary's and its component's"
            ));
        }
        let family = match (m.kind, m.name.strip_suffix("_total")) {
            (Kind::Counter, Some(family)) => family,
            (Kind::Counter, None) => {
                problems.push(format!("{at}: a counter's name ends in `_total`"));
                m.name
            }
            (Kind::Gauge, _) => m.name,
        };
        if !families.insert(family) {
            problems.push(format!(
                "{at}: its family `{family}` is another metric's too"
            ));
        }
        if let Some(last) = family.rsplit('_').next()
            && TYPE_WORDS.contains(&last)
        {
            problems.push(format!(
                "{at}: it ends in `_{last}`, a word another type's samples take"
            ));
        }
        problems.extend(unit_problems(&at, family));
        match m.unit.word() {
            Some(unit) if !family.ends_with(&format!("_{unit}")) => problems.push(format!(
                "{at}: its unit is {unit}, so its name ends in `_{unit}`{}",
                if m.kind == Kind::Counter {
                    " before `_total`"
                } else {
                    ""
                }
            )),
            None if family.rsplit('_').next().is_some_and(|w| BASE.contains(&w)) => {
                problems.push(format!("{at}: it ends in a unit and declares none"));
            }
            _ => {}
        }
        problems.extend(text_problem(&at, m.help));
        let mut names = BTreeSet::new();
        for label in m.labels {
            let l = label.name;
            if !names.insert(l) {
                problems.push(format!("{at}: label `{l}` is declared twice"));
            }
            if !snake(l) {
                problems.push(format!("{at}: label `{l}` is ASCII snake_case"));
            }
            if REQUEST_LABELS.contains(&l) {
                problems.push(format!(
                    "{at}: label `{l}` takes its values from a request; a label's values are a closed set Reliquary chooses (AC-RQ0.9.2)"
                ));
            }
            if HOST_LABELS.contains(&l) {
                problems.push(format!(
                    "{at}: label `{l}` is the scraper's, the host's or the format's"
                ));
            }
            if label.values.is_empty() {
                problems.push(format!("{at}: label `{l}` takes no value"));
            }
            let mut values = BTreeSet::new();
            for value in label.values {
                if !snake(value) {
                    problems.push(format!(
                        "{at}: label `{l}`'s value `{value}` is ASCII snake_case"
                    ));
                }
                if !values.insert(value) {
                    problems.push(format!("{at}: label `{l}` takes `{value}` twice"));
                }
            }
        }
        if m.series().is_none() {
            problems.push(format!("{at}: it has more series than a usize counts"));
        }
    }
    let mut names = BTreeSet::new();
    for e in events {
        let at = format!("event `{}`", e.name);
        if !names.insert(e.name) {
            problems.push(format!("{at} is declared twice"));
        }
        let target = format!("reliquary.{}", e.component);
        if e.target != target {
            problems.push(format!("{at}: its target is `{target}`, its component's"));
        }
        let what = e.name.strip_prefix(&format!("{}.", e.component));
        if !what.is_some_and(|what| what.split('.').all(snake)) {
            problems.push(format!(
                "{at}: its name is `{}.` and dotted snake_case words",
                e.component
            ));
        }
        problems.extend(text_problem(&at, e.help));
        let mut fields = BTreeSet::new();
        for Field { name, values } in e.fields {
            if !fields.insert(name) {
                problems.push(format!("{at}: field `{name}` is declared twice"));
            }
            if !snake(name) {
                problems.push(format!("{at}: field `{name}` is ASCII snake_case"));
            }
            if LINE_KEYS.contains(name) {
                problems.push(format!(
                    "{at}: field `{name}` is a key its line has of its own"
                ));
            }
            problems.extend(unit_problems(&format!("{at}, field `{name}`"), name));
            match values {
                Values::Free if KILN_TAGS.contains(name) => problems.push(format!(
                    "{at}: field `{name}` holds what a request chose, and kiln makes `{name}` a tag"
                )),
                Values::Closed([]) => {
                    problems.push(format!("{at}: field `{name}` takes no value"));
                }
                _ => {}
            }
        }
    }
    problems
}

// ---------------------------------------------------------------------------------------------------
// The doc-truth test (AC-RQ0.9.1, AC-RQ0.9.4).

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn doc() -> String {
    std::fs::read_to_string(root().join("docs/observability.md")).expect("docs/observability.md")
}

fn metrics() -> Vec<&'static Metric> {
    METRICS.iter().collect()
}

fn events() -> Vec<&'static Event> {
    EVENTS.iter().collect()
}

/// The text between `<!-- {name}:begin -->` and `<!-- {name}:end -->`.
fn block<'a>(doc: &'a str, name: &str) -> Option<&'a str> {
    let (_, after) = doc.split_once(&format!("<!-- {name}:begin -->"))?;
    let (inside, _) = after.split_once(&format!("<!-- {name}:end -->"))?;
    Some(inside)
}

/// A table's rows past its header and rule, each its cells, trimmed.
fn rows(block: &str) -> Vec<Vec<String>> {
    block
        .lines()
        .map(str::trim)
        .filter(|l| l.starts_with('|'))
        .skip(2)
        .map(|l| {
            l.trim_matches('|')
                .split('|')
                .map(|c| c.trim().to_owned())
                .collect()
        })
        .collect()
}

fn ticked(words: &[&str]) -> String {
    words
        .iter()
        .map(|w| format!("`{w}`"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// A metric's row, as the catalog says it.
fn metric_row(m: &Metric) -> Vec<String> {
    let labels = if m.labels.is_empty() {
        "—".to_owned()
    } else {
        m.labels
            .iter()
            .map(|Label { name, values }| format!("`{name}`: {}", ticked(values)))
            .collect::<Vec<_>>()
            .join("; ")
    };
    vec![
        format!("`{}`", m.name),
        m.kind.word().to_owned(),
        m.unit.word().unwrap_or("—").to_owned(),
        labels,
        m.series().map_or("?".to_owned(), |n| n.to_string()),
        m.help.to_owned(),
    ]
}

/// An event's row, as the catalog says it.
fn event_row(e: &Event) -> Vec<String> {
    let fields = if e.fields.is_empty() {
        "—".to_owned()
    } else {
        e.fields
            .iter()
            .map(|Field { name, values }| {
                let holds = match values {
                    Values::Count => "count".to_owned(),
                    Values::Free => "free".to_owned(),
                    Values::Closed(closed) => ticked(closed),
                };
                format!("`{name}`: {holds}")
            })
            .collect::<Vec<_>>()
            .join("; ")
    };
    vec![
        format!("`{}`", e.target),
        format!("`{}`", e.name),
        e.level.word().to_owned(),
        fields,
        e.help.to_owned(),
    ]
}

/// Compares a table of `doc` with the catalog's rows, keyed by column `key`, naming each
/// difference.
fn compare(
    what: &str,
    doc_rows: &[Vec<String>],
    code_rows: &[Vec<String>],
    columns: &[&str],
    key: usize,
) -> Vec<String> {
    let mut problems = Vec::new();
    let code: BTreeMap<&str, &Vec<String>> =
        code_rows.iter().map(|r| (r[key].as_str(), r)).collect();
    let mut documented = BTreeSet::new();
    for row in doc_rows {
        let Some(name) = row.get(key) else {
            problems.push(format!(
                "docs/observability.md has a {what} row without its {}",
                columns[key]
            ));
            continue;
        };
        if !documented.insert(name.as_str()) {
            problems.push(format!(
                "{what} {name} is listed twice in docs/observability.md"
            ));
        }
        match code.get(name.as_str()) {
            None => problems.push(format!(
                "{what} {name} is in docs/observability.md and not in the catalog (crates/reliquary-core/src/observe/catalog.rs)"
            )),
            Some(want) => {
                for (i, column) in columns.iter().enumerate() {
                    let said = row.get(i).map_or("(nothing)", String::as_str);
                    if said != want[i] {
                        problems.push(format!(
                            "{what} {name}: docs/observability.md's {column} says {said:?}, the catalog {:?}",
                            want[i]
                        ));
                    }
                }
            }
        }
    }
    for name in code.keys() {
        if !documented.contains(name) {
            problems.push(format!(
                "{what} {name} is in the catalog and not in docs/observability.md"
            ));
        }
    }
    problems
}

/// Every `reliquary_…` name in `text`.
fn metric_names(text: &str) -> Vec<&str> {
    text.match_indices("reliquary_")
        .filter_map(|(at, _)| text.get(at..))
        .filter_map(|from| {
            let end = from
                .find(|c: char| !(c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'))
                .unwrap_or(from.len());
            from.get(..end)
        })
        .collect()
}

/// The dashboard groups of `doc`: each component's name and its panels' rows.
fn groups(doc: &str) -> Vec<(String, Vec<Vec<String>>)> {
    doc.split("<!-- dashboard:")
        .skip(1)
        .filter_map(|after| after.split_once(" -->"))
        .filter_map(|(marker, _)| marker.strip_suffix(":begin"))
        .map(|component| {
            let panels = block(doc, &format!("dashboard:{component}")).map_or_else(Vec::new, rows);
            (component.to_owned(), panels)
        })
        .collect()
}

/// What differs between `doc` and the catalog: each metric and event in both and said alike, and
/// each component's dashboard group drawing every metric of it in PromQL and in kiln's queries and
/// showing every event of it in kiln's (AC-RQ0.9.1, AC-RQ0.9.4).
fn doc_truth(metrics: &[&Metric], events: &[&Event], doc: &str) -> Vec<String> {
    let mut problems = Vec::new();
    let Some(metric_block) = block(doc, "metrics") else {
        return vec!["docs/observability.md has no metrics table".to_owned()];
    };
    let Some(event_block) = block(doc, "events") else {
        return vec!["docs/observability.md has no events table".to_owned()];
    };
    problems.extend(compare(
        "metric",
        &rows(metric_block),
        &metrics.iter().map(|m| metric_row(m)).collect::<Vec<_>>(),
        &["Metric", "Type", "Unit", "Labels", "Series", "Help"],
        0,
    ));
    problems.extend(compare(
        "event",
        &rows(event_block),
        &events.iter().map(|e| event_row(e)).collect::<Vec<_>>(),
        &["Target", "Event", "Level", "Fields", "Help"],
        1,
    ));
    let groups = groups(doc);
    let components: BTreeSet<&str> = metrics
        .iter()
        .map(|m| m.component)
        .chain(events.iter().map(|e| e.component))
        .collect();
    for (component, panels) in &groups {
        if !components.contains(component.as_str()) {
            problems.push(format!(
                "docs/observability.md has a dashboard group for `{component}`, which no metric or event belongs to"
            ));
        }
        for panel in panels {
            if panel.len() != 3 {
                problems.push(format!(
                    "group `{component}`: a panel has its title, its PromQL and its kiln query: {panel:?}"
                ));
                continue;
            }
            for drawn in metric_names(&panel[1])
                .into_iter()
                .chain(metric_names(&panel[2]))
            {
                if !metrics
                    .iter()
                    .any(|m| m.name == drawn && m.component == component)
                {
                    problems.push(format!(
                        "group `{component}`'s panel {:?} draws `{drawn}`, which is no metric of `{component}`'s",
                        panel[0]
                    ));
                }
            }
        }
    }
    for component in components {
        let Some((_, panels)) = groups.iter().find(|(c, _)| c == component) else {
            problems.push(format!(
                "component `{component}` has no dashboard group in docs/observability.md: it ships its rows in the change that adds it (AC-RQ0.9.4)"
            ));
            continue;
        };
        let drawn = |column: usize, name: &str| {
            panels
                .iter()
                .filter_map(|p| p.get(column))
                .any(|cell| metric_names(cell).contains(&name))
        };
        for m in metrics.iter().filter(|m| m.component == component) {
            for (column, language) in [(1, "PromQL"), (2, "kiln")] {
                if !drawn(column, m.name) {
                    problems.push(format!(
                        "`{}` is drawn in no {language} panel of group `{component}` (AC-RQ0.9.4)",
                        m.name
                    ));
                }
            }
        }
        for e in events.iter().filter(|e| e.component == component) {
            if !panels
                .iter()
                .filter_map(|p| p.get(2))
                .any(|cell| cell.contains(e.name))
            {
                problems.push(format!(
                    "event `{}` is shown in no kiln panel of group `{component}` (AC-RQ0.9.4)",
                    e.name
                ));
            }
        }
    }
    problems
}

/// Read docs/observability.md against the catalog → nothing differs: every metric and event in
/// both and said alike, every component drawn in its group (AC-RQ0.9.1, AC-RQ0.9.4).
#[test]
fn ac_rq0_9_1_docs_observability_md_and_the_catalog_agree() {
    let problems = doc_truth(&metrics(), &events(), &doc());
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

/// Add a metric to the catalog without documenting it → the doc-truth test fails and names it
/// (T-RQ0.9.2).
#[test]
fn t_rq0_9_2_a_metric_the_doc_lacks_fails_the_doc_truth_test_and_is_named() {
    const UNDOCUMENTED: Metric = Metric {
        name: "reliquary_events_undocumented_total",
        help: "Events no page speaks of.",
        ..catalog::EVENTS_EMITTED
    };
    let mut with = metrics();
    with.push(&UNDOCUMENTED);
    let problems = doc_truth(&with, &events(), &doc());
    assert!(
        problems.iter().any(|p| p
            .contains("`reliquary_events_undocumented_total` is in the catalog and not in docs/observability.md")),
        "{problems:#?}"
    );
}

/// Add an event to the catalog without documenting it → the doc-truth test fails and names it
/// (AC-RQ0.9.1).
#[test]
fn ac_rq0_9_1_an_event_the_doc_lacks_fails_the_doc_truth_test_and_is_named() {
    const UNDOCUMENTED: Event = Event {
        name: "events.undocumented",
        ..catalog::EVENTS_DROPPED_EVENT
    };
    let mut with = events();
    with.push(&UNDOCUMENTED);
    let problems = doc_truth(&metrics(), &with, &doc());
    assert!(
        problems.iter().any(|p| p.contains(
            "event `events.undocumented` is in the catalog and not in docs/observability.md"
        )),
        "{problems:#?}"
    );
}

/// Document a metric and an event the code lacks → the doc-truth test fails and names each
/// (AC-RQ0.9.1).
#[test]
fn ac_rq0_9_1_a_documented_metric_or_event_the_code_lacks_fails_and_is_named() {
    let doc = doc()
        .replace(
            "<!-- metrics:end -->",
            "| `reliquary_events_ghost_total` | counter | — | — | 1 | Ghosts. |\n<!-- metrics:end -->",
        )
        .replace(
            "<!-- events:end -->",
            "| `reliquary.events` | `events.ghost` | info | — | A ghost. |\n<!-- events:end -->",
        );
    let problems = doc_truth(&metrics(), &events(), &doc);
    for name in [
        "metric `reliquary_events_ghost_total`",
        "event `events.ghost`",
    ] {
        assert!(
            problems.iter().any(|p| p.starts_with(name)
                && p.contains("is in docs/observability.md and not in the catalog")),
            "{name}: {problems:#?}"
        );
    }
}

/// Document a metric whose help, labels and series differ from the catalog's → each difference
/// named with its column (AC-RQ0.9.1).
#[test]
fn ac_rq0_9_1_a_row_that_says_other_than_the_catalog_fails_and_names_the_column() {
    let truth = doc();
    let row = metric_row(&catalog::EVENTS_DROPPED);
    let said = format!("| {} |", row.join(" | "));
    assert!(
        truth.contains(&said),
        "the doc's row is the catalog's: {said}"
    );
    let altered = said
        .replace("`closed`", "`late`")
        .replace("| 3 |", "| 4 |")
        .replace("Events the event ring let go", "Events lost");
    let problems = doc_truth(&metrics(), &events(), &truth.replace(&said, &altered));
    for column in ["Labels", "Series", "Help"] {
        assert!(
            problems.iter().any(|p| p
                .starts_with("metric `reliquary_events_dropped_total`: docs/observability.md's ")
                && p.contains(&format!("'s {column} says"))),
            "{column}: {problems:#?}"
        );
    }
}

/// Add a component's metric, documented, with no dashboard group → the test fails and names the
/// component; with a group drawing it in PromQL and kiln's queries → it passes (AC-RQ0.9.4).
#[test]
fn ac_rq0_9_4_a_component_without_its_dashboard_rows_fails_and_is_named() {
    const PULLS: Metric = Metric {
        name: "reliquary_pulls_total",
        component: "pulls",
        kind: Kind::Counter,
        unit: Unit::None,
        labels: &[],
        help: "Pulls answered.",
    };
    let mut with = metrics();
    with.push(&PULLS);
    let documented = doc().replace(
        "<!-- metrics:end -->",
        "| `reliquary_pulls_total` | counter | — | — | 1 | Pulls answered. |\n<!-- metrics:end -->",
    );
    let problems = doc_truth(&with, &events(), &documented);
    assert_eq!(
        problems,
        [
            "component `pulls` has no dashboard group in docs/observability.md: it ships its rows in the change that adds it (AC-RQ0.9.4)"
        ]
    );
    let half = format!(
        "{documented}\n<!-- dashboard:pulls:begin -->\n| Panel | PromQL | kiln |\n|---|---|---|\n| Pulls/s | `sum(rate(reliquary_pulls_total[5m]))` | — |\n<!-- dashboard:pulls:end -->\n"
    );
    assert_eq!(
        doc_truth(&with, &events(), &half),
        ["`reliquary_pulls_total` is drawn in no kiln panel of group `pulls` (AC-RQ0.9.4)"]
    );
    let drawn = half.replace(
        "| — |\n<!-- dashboard:pulls:end -->",
        "| `sum:reliquary_pulls_total{$host}.as_count()` |\n<!-- dashboard:pulls:end -->",
    );
    let problems = doc_truth(&with, &events(), &drawn);
    assert!(problems.is_empty(), "{problems:#?}");
}

/// A panel that draws another component's metric, or one no catalog has → named (AC-RQ0.9.4).
#[test]
fn ac_rq0_9_4_a_panel_drawing_what_its_component_lacks_is_named() {
    let doc = doc().replace(
        "| Drop markers |",
        "| Strays | `sum(rate(reliquary_pulls_total[5m]))` | — |\n| Drop markers |",
    );
    let problems = doc_truth(&metrics(), &events(), &doc);
    assert!(
        problems
            .iter()
            .any(|p| p.contains("draws `reliquary_pulls_total`, which is no metric of `events`'s")),
        "{problems:#?}"
    );
}

// ---------------------------------------------------------------------------------------------------
// The lint's tests (AC-RQ0.9.2, T-RQ0.9.1, T-RQ0.9.3).

/// Run the lint over the catalog → nothing refused (AC-RQ0.9.2).
#[test]
fn ac_rq0_9_2_the_catalog_passes_the_lint() {
    let problems = lint(&metrics(), &events());
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

fn labelled(name: &'static str) -> Metric {
    let labels: &'static [Label] = Box::leak(Box::new([Label {
        name,
        values: &["library_alpine"],
    }]));
    Metric {
        name: "reliquary_pulls_total",
        component: "pulls",
        kind: Kind::Counter,
        unit: Unit::None,
        labels,
        help: "Pulls answered.",
    }
}

/// Add a counter with a `repository` label → the lint fails, naming the label (T-RQ0.9.1).
#[test]
fn t_rq0_9_1_a_counter_with_a_repository_label_fails_the_lint() {
    let by_repository = labelled("repository");
    let problems = lint(&[&by_repository], &[]);
    assert_eq!(
        problems,
        [
            "`reliquary_pulls_total`: label `repository` takes its values from a request; a label's values are a closed set Reliquary chooses (AC-RQ0.9.2)"
        ]
    );
}

/// Add a counter labelled by each other thing a request carries (a tag, a digest, a principal and
/// the rest the conventions list) → the lint fails for each (AC-RQ0.9.2).
#[test]
fn ac_rq0_9_2_a_label_a_request_chooses_fails_the_lint() {
    for &name in REQUEST_LABELS {
        let metric = labelled(name);
        let problems = lint(&[&metric], &[]);
        assert!(
            problems
                .iter()
                .any(|p| p.contains(&format!("label `{name}` takes its values from a request"))),
            "{name}: {problems:#?}"
        );
    }
    let host = labelled("instance");
    assert!(lint(&[&host], &[])[0].contains("is the scraper's, the host's or the format's"));
}

/// Name a metric `reliquary_pull_ms` → the unit lint fails and asks for `_seconds` (T-RQ0.9.3).
#[test]
fn t_rq0_9_3_a_metric_named_reliquary_pull_ms_fails_the_unit_lint_asking_for_seconds() {
    const PULL_MS: Metric = Metric {
        name: "reliquary_pull_ms",
        component: "pull",
        kind: Kind::Gauge,
        unit: Unit::None,
        labels: &[],
        help: "How long the last pull took.",
    };
    assert_eq!(
        lint(&[&PULL_MS], &[]),
        [
            "`reliquary_pull_ms`: `ms` is not a base unit: name it in `_seconds` (docs/observability.md, Units)"
        ]
    );
}

/// Name metrics in megabytes and percent, declare a unit a name doesn't end in, name a counter
/// without `_total` and a gauge with it, and put a metric under another component → each refused,
/// saying what to do (AC-RQ0.9.2).
#[test]
fn ac_rq0_9_2_the_lint_refuses_each_break_of_the_conventions() {
    let gauge = |name: &'static str, unit: Unit| Metric {
        name,
        component: "pull",
        kind: Kind::Gauge,
        unit,
        labels: &[],
        help: "A gauge.",
    };
    let cases = [
        (
            gauge("reliquary_pull_size_mb", Unit::None),
            "name it in `_bytes`",
        ),
        (
            gauge("reliquary_pull_done_percent", Unit::None),
            "name it in `_ratio`",
        ),
        (
            gauge("reliquary_pull_wait", Unit::Seconds),
            "so its name ends in `_seconds`",
        ),
        (
            gauge("reliquary_pull_wait_seconds", Unit::None),
            "it ends in a unit and declares none",
        ),
        (
            gauge("reliquary_pull_total", Unit::None),
            "a word another type's samples take",
        ),
        (
            gauge("reliquary_push_wait_seconds", Unit::Seconds),
            "its name begins `reliquary_pull_`",
        ),
        (
            gauge("reliquary_pull_Wait", Unit::None),
            "a name is ASCII snake_case",
        ),
        (
            Metric {
                kind: Kind::Counter,
                ..gauge("reliquary_pull_pulls", Unit::None)
            },
            "a counter's name ends in `_total`",
        ),
    ];
    for (metric, said) in &cases {
        let problems = lint(&[metric], &[]);
        assert!(
            problems.iter().any(|p| p.contains(said)),
            "{}: {said}: {problems:#?}",
            metric.name
        );
    }
}

/// Declare events off their component's target, not named under it, with a free field kiln makes
/// a tag of, a field taking the line's own key, and a field in milliseconds → each refused
/// (AC-RQ0.9.2).
#[test]
fn ac_rq0_9_2_the_lint_refuses_events_that_break_the_conventions() {
    let event = |target, name, fields: &'static [Field]| Event {
        target,
        name,
        component: "pulls",
        level: catalog::Level::Info,
        fields,
        help: "An event.",
    };
    let cases = [
        (
            event("reliquary.push", "pulls.done", &[]),
            "its target is `reliquary.pulls`",
        ),
        (
            event("reliquary.pulls", "done", &[]),
            "its name is `pulls.`",
        ),
        (
            event(
                "reliquary.pulls",
                "pulls.done",
                &[Field {
                    name: "repo",
                    values: Values::Free,
                }],
            ),
            "kiln makes `repo` a tag",
        ),
        (
            event(
                "reliquary.pulls",
                "pulls.done",
                &[Field {
                    name: "level",
                    values: Values::Count,
                }],
            ),
            "a key its line has of its own",
        ),
        (
            event(
                "reliquary.pulls",
                "pulls.done",
                &[Field {
                    name: "took_ms",
                    values: Values::Count,
                }],
            ),
            "name it in `_seconds`",
        ),
    ];
    for (e, said) in &cases {
        let problems = lint(&[], &[e]);
        assert!(
            problems.iter().any(|p| p.contains(said)),
            "{}: {said}: {problems:#?}",
            e.name
        );
    }
    let free_repository = event(
        "reliquary.pulls",
        "pulls.done",
        &[Field {
            name: "repository",
            values: Values::Free,
        }],
    );
    assert!(lint(&[], &[&free_repository]).is_empty());
}

// ---------------------------------------------------------------------------------------------------
// What the code emits is what the catalog says (AC-RQ0.9.1). A metric is a `reliquary_` name, which
// is written only in the catalog. An event is emitted only through tracing's macros, which appear
// only in observe/. A string beginning `reliquary.` is free outside the catalog (a seed stream's
// label, say): without a macro it emits nothing.

/// tracing's macros that emit events or make spans.
const MACROS: &[&str] = &[
    "trace",
    "debug",
    "info",
    "warn",
    "error",
    "event",
    "span",
    "trace_span",
    "debug_span",
    "info_span",
    "warn_span",
    "error_span",
];

/// What a scan found: its problems, the files it read, and the metric names it found in the
/// catalog.
struct Scan {
    problems: Vec<String>,
    files: usize,
    catalog_names: usize,
}

/// Every `.rs` file under `crates/*/src` of `root`, walked with a list rather than by recursion
/// (CLAUDE.md §11).
fn sources(root: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(root.join("crates"))
        .unwrap()
        .map(|e| e.unwrap().path().join("src"))
        .filter(|p| p.is_dir())
        .collect();
    while let Some(dir) = dirs.pop() {
        for entry in std::fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                dirs.push(path);
            } else if path.extension().is_some_and(|e| e == "rs") {
                found.push(path);
            }
        }
    }
    found.sort();
    found
}

/// Whether `line` invokes `name!` as a macro of its own (not as the end of a longer name).
fn invokes(line: &str, name: &str) -> bool {
    let call = format!("{name}!");
    line.match_indices(&call).any(|(at, _)| {
        let before = line.get(..at).and_then(|b| b.chars().next_back());
        let after = line
            .get(at + call.len()..)
            .and_then(|a| a.trim_start().chars().next());
        !before.is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
            && matches!(after, Some('(' | '{' | '['))
    })
}

/// Scans `root`'s sources under `crates/*/src`: a string beginning `reliquary_` only in the
/// catalog, and tracing's event and span macros (and `#[instrument]`) only in observe/.
fn scan(root: &Path) -> Scan {
    let observe = root.join("crates/reliquary-core/src/observe");
    let catalog = observe.join("catalog.rs");
    let mut scan = Scan {
        problems: Vec::new(),
        files: 0,
        catalog_names: 0,
    };
    for file in sources(root) {
        scan.files += 1;
        let text = std::fs::read_to_string(&file).unwrap();
        // As the repository names it, `/` between components on every platform.
        let shown = file
            .strip_prefix(root)
            .unwrap_or(&file)
            .components()
            .map(|c| c.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/");
        for (i, line) in text.lines().enumerate() {
            if line.trim_start().starts_with("//") {
                continue;
            }
            let names = line.matches("\"reliquary_").count();
            if file == catalog {
                scan.catalog_names += names;
            } else if names > 0 {
                scan.problems.push(format!(
                    "{shown}:{}: a `reliquary_` name is written only in crates/reliquary-core/src/observe/catalog.rs",
                    i + 1
                ));
            }
            if !file.starts_with(&observe) {
                for name in MACROS.iter().filter(|m| invokes(line, m)) {
                    scan.problems.push(format!(
                        "{shown}:{}: `{name}!` emits outside crates/reliquary-core/src/observe/: an event is the catalog's",
                        i + 1
                    ));
                }
                if line.contains("#[instrument") || line.contains("tracing::instrument") {
                    scan.problems.push(format!(
                        "{shown}:{}: `instrument` makes a span outside crates/reliquary-core/src/observe/",
                        i + 1
                    ));
                }
            }
        }
    }
    scan
}

/// Scan every source under crates/*/src → no `reliquary_` name outside the catalog and no tracing
/// macro outside observe/; and the scan read the catalog's own names, every metric's, so it looked
/// where they are (AC-RQ0.9.1).
#[test]
fn ac_rq0_9_1_names_and_events_are_written_only_in_the_catalog() {
    let scan = scan(&root());
    assert!(scan.problems.is_empty(), "{}", scan.problems.join("\n"));
    assert!(scan.files > METRICS.len(), "read {} files", scan.files);
    assert!(
        scan.catalog_names >= METRICS.len(),
        "found {} metric names in the catalog",
        scan.catalog_names
    );
}

/// Scan a tree with a stray metric name and stray macros outside observe/, beside a catalog and an
/// observe/ that may hold them, and a seed's label beginning `reliquary.` → each stray named by
/// file and line, and the label left alone (AC-RQ0.9.1).
#[test]
fn ac_rq0_9_1_the_scan_names_a_stray_name_and_a_stray_macro() {
    let tree = tempfile::tempdir().unwrap();
    let write = |path: &str, text: &str| {
        let path = tree.path().join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    };
    write(
        "crates/reliquary-core/src/observe/catalog.rs",
        "const A: &str = \"reliquary_a_total\";\nconst T: &str = \"reliquary.a\";\n",
    );
    write(
        "crates/reliquary-core/src/observe/layer.rs",
        "fn f() { tracing::info!(target: T, \"fine here\"); }\n",
    );
    write(
        "crates/reliquary/src/door/pull.rs",
        "// a comment may say \"reliquary_x\" and info!(x)\nfn g() {\n    let n = \"reliquary_pulls_total\";\n    tracing::warn!(target: \"reliquary.pulls\", event = \"pulls.done\");\n    debug_span!(\"x\");\n    compile_error!(\"not a log\");\n    let seed = \"reliquary.seed\";\n}\n#[tracing::instrument]\nfn h() {}\n",
    );
    let scan = scan(tree.path());
    assert_eq!(scan.files, 3);
    assert_eq!(scan.catalog_names, 1);
    let shown: Vec<String> = scan
        .problems
        .iter()
        .map(|p| p.split(": ").next().unwrap().to_owned())
        .collect();
    assert_eq!(
        shown,
        [
            "crates/reliquary/src/door/pull.rs:3",
            "crates/reliquary/src/door/pull.rs:4",
            "crates/reliquary/src/door/pull.rs:5",
            "crates/reliquary/src/door/pull.rs:9"
        ],
        "{:#?}",
        scan.problems
    );
}

// ---------------------------------------------------------------------------------------------------
// OpenMetrics 1.0's text format, read from its ABNF ("Text format", "ABNF") apart from the
// encoder, to judge what a scrape gets (T-RQ0.9.4).

mod om {
    /// A family as the text gives it.
    #[derive(Debug, Default)]
    pub struct Family {
        pub name: String,
        pub kind: Option<String>,
        pub unit: Option<String>,
        pub help: Option<String>,
        pub samples: Vec<Sample>,
    }

    /// A sample: its name, its labels in order, and its value.
    #[derive(Debug)]
    pub struct Sample {
        pub name: String,
        pub labels: Vec<(String, String)>,
        pub value: f64,
    }

    const TYPES: &[&str] = &[
        "counter",
        "gauge",
        "histogram",
        "gaugehistogram",
        "stateset",
        "info",
        "summary",
        "unknown",
    ];

    fn name_char(c: char, first: bool) -> bool {
        c.is_ascii_alphabetic() || c == '_' || c == ':' || (!first && c.is_ascii_digit())
    }

    /// `metricname`: an initial character, then name characters.
    fn metric_name(s: &str) -> Result<(), String> {
        let mut chars = s.chars();
        match chars.next() {
            Some(c) if name_char(c, true) && chars.all(|c| name_char(c, false)) => Ok(()),
            _ => Err(format!("{s:?} is no metricname")),
        }
    }

    /// `label-name`: as a metricname, without colons.
    fn label_name(s: &str) -> Result<(), String> {
        metric_name(s)?;
        if s.contains(':') {
            return Err(format!("{s:?} is no label-name"));
        }
        Ok(())
    }

    /// An `escaped-string` up to an unescaped quotation mark (when `quoted`) or the end, its
    /// value, and what follows the closing mark.
    fn escaped(s: &str, quoted: bool) -> Result<(String, &str), String> {
        let mut value = String::new();
        let mut chars = s.char_indices();
        while let Some((at, c)) = chars.next() {
            match c {
                '"' if quoted => return Ok((value, s.get(at + 1..).unwrap_or(""))),
                '"' => return Err("an unescaped quotation mark".to_owned()),
                '\n' => return Err("a line feed in a string".to_owned()),
                '\\' => match chars.next() {
                    Some((_, 'n')) => value.push('\n'),
                    Some((_, '"')) => value.push('"'),
                    Some((_, '\\')) => value.push('\\'),
                    Some((_, other)) => value.push(other),
                    None => return Err("a backslash at the end".to_owned()),
                },
                c => value.push(c),
            }
        }
        if quoted {
            Err("a string without its closing quotation mark".to_owned())
        } else {
            Ok((value, ""))
        }
    }

    /// `number`: a realnumber, an infinity, or NaN, case aside.
    fn number(s: &str) -> Result<f64, String> {
        let lower = s.to_ascii_lowercase();
        let unsigned = lower.strip_prefix(['+', '-']).unwrap_or(&lower);
        let real = |t: &str| {
            let (mantissa, exponent) = match t.split_once('e') {
                Some((m, e)) => (m, Some(e)),
                None => (t, None),
            };
            let exponent_ok = exponent.is_none_or(|e| {
                let digits = e.strip_prefix(['+', '-']).unwrap_or(e);
                !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit())
            });
            let mantissa_ok = match mantissa.split_once('.') {
                Some((whole, part)) => {
                    (!whole.is_empty() || !part.is_empty())
                        && whole.chars().all(|c| c.is_ascii_digit())
                        && part.chars().all(|c| c.is_ascii_digit())
                        && (exponent.is_none() || !part.is_empty() || !whole.is_empty())
                }
                None => !mantissa.is_empty() && mantissa.chars().all(|c| c.is_ascii_digit()),
            };
            mantissa_ok && exponent_ok
        };
        let ok = lower == "nan" || unsigned == "inf" || unsigned == "infinity" || real(unsigned);
        if !ok {
            return Err(format!("{s:?} is no number"));
        }
        s.parse::<f64>().map_err(|e| format!("{s:?}: {e}"))
    }

    /// A sample line.
    fn sample(line: &str) -> Result<Sample, String> {
        let end = line
            .find(['{', ' '])
            .ok_or_else(|| format!("{line:?} has no value"))?;
        let (name, mut rest) = line
            .split_at_checked(end)
            .ok_or_else(|| format!("{line:?} splits nowhere at {end}"))?;
        metric_name(name)?;
        let mut labels = Vec::new();
        if let Some(inside) = rest.strip_prefix('{') {
            rest = inside;
            if let Some(after) = rest.strip_prefix('}') {
                rest = after;
            } else {
                loop {
                    let (label, after) = rest
                        .split_once("=\"")
                        .ok_or_else(|| format!("{line:?}: a label without =\""))?;
                    label_name(label)?;
                    let (value, after) = escaped(after, true)?;
                    if labels.iter().any(|(l, _): &(String, String)| l == label) {
                        return Err(format!("{line:?}: label {label} twice"));
                    }
                    labels.push((label.to_owned(), value));
                    if let Some(after) = after.strip_prefix(',') {
                        rest = after;
                    } else if let Some(after) = after.strip_prefix('}') {
                        rest = after;
                        break;
                    } else {
                        return Err(format!("{line:?}: labels end in }}"));
                    }
                }
            }
        }
        let rest = rest
            .strip_prefix(' ')
            .ok_or_else(|| format!("{line:?}: a space before the value"))?;
        let (value, after) = rest.split_once(' ').unwrap_or((rest, ""));
        let value = number(value)?;
        if !after.is_empty() {
            let (timestamp, exemplar) = after.split_once(' ').unwrap_or((after, ""));
            if !timestamp.is_empty() {
                number(timestamp)?;
            }
            if !exemplar.is_empty() && !exemplar.starts_with("# {") {
                return Err(format!("{line:?}: what follows the value"));
            }
        }
        Ok(Sample {
            name: name.to_owned(),
            labels,
            value,
        })
    }

    /// Whether a sample named `name` is one of `family`'s, by its type's suffixes ("Suffixes").
    fn belongs(family: &Family, name: &str) -> bool {
        let suffixes: &[&str] = match family.kind.as_deref() {
            Some("counter") => &["_total", "_created"],
            Some("info") => &["_info"],
            Some("histogram") => &["_bucket", "_count", "_sum", "_created"],
            Some("gaugehistogram") => &["_bucket", "_gcount", "_gsum"],
            Some("summary") => &["", "_count", "_sum", "_created"],
            _ => &[""],
        };
        name.strip_prefix(family.name.as_str())
            .is_some_and(|suffix| suffixes.contains(&suffix))
    }

    /// Reads an exposition into its families, refusing what the text format refuses.
    pub fn parse(text: &str) -> Result<Vec<Family>, String> {
        if text.contains('\r') {
            return Err("a carriage return".to_owned());
        }
        if text.starts_with('\u{feff}') {
            return Err("a byte order mark".to_owned());
        }
        let body = text
            .strip_suffix("# EOF\n")
            .or_else(|| text.strip_suffix("# EOF"))
            .ok_or("no `# EOF` at its end")?;
        if !body.is_empty() && !body.ends_with('\n') {
            return Err("`# EOF` is a line of its own".to_owned());
        }
        let mut families: Vec<Family> = Vec::new();
        for (i, line) in body.lines().enumerate() {
            let at = i + 1;
            if let Some(meta) = line.strip_prefix("# ") {
                let (word, rest) = meta
                    .split_once(' ')
                    .ok_or_else(|| format!("line {at}: {line:?}"))?;
                let (name, value) = rest
                    .split_once(' ')
                    .ok_or_else(|| format!("line {at}: {line:?} has no value"))?;
                metric_name(name).map_err(|e| format!("line {at}: {e}"))?;
                if families.last().is_none_or(|f| f.name != name) {
                    if families.iter().any(|f| f.name == name) {
                        return Err(format!("line {at}: family {name} again, after another"));
                    }
                    families.push(Family {
                        name: name.to_owned(),
                        ..Family::default()
                    });
                }
                let family = families.last_mut().unwrap();
                if !family.samples.is_empty() {
                    return Err(format!("line {at}: {name}'s metadata after its samples"));
                }
                match word {
                    "TYPE" => {
                        if family.kind.is_some() || family.unit.is_some() || family.help.is_some() {
                            return Err(format!("line {at}: TYPE twice, or after UNIT or HELP"));
                        }
                        if !TYPES.contains(&value) {
                            return Err(format!("line {at}: no type {value:?}"));
                        }
                        family.kind = Some(value.to_owned());
                    }
                    "UNIT" => {
                        if family.unit.is_some() || family.help.is_some() {
                            return Err(format!("line {at}: UNIT twice, or after HELP"));
                        }
                        if !value.chars().all(|c| name_char(c, false)) {
                            return Err(format!("line {at}: no unit {value:?}"));
                        }
                        if !value.is_empty() && !name.ends_with(&format!("_{value}")) {
                            return Err(format!(
                                "line {at}: {name} doesn't end in its unit, _{value}"
                            ));
                        }
                        family.unit = Some(value.to_owned());
                    }
                    "HELP" => {
                        if family.help.is_some() {
                            return Err(format!("line {at}: HELP twice"));
                        }
                        let (help, _) =
                            escaped(value, false).map_err(|e| format!("line {at}: {e}"))?;
                        family.help = Some(help);
                    }
                    _ => return Err(format!("line {at}: # begins only TYPE, UNIT, HELP and EOF")),
                }
            } else if line.starts_with('#') {
                return Err(format!("line {at}: # begins only TYPE, UNIT, HELP and EOF"));
            } else {
                let sample = sample(line).map_err(|e| format!("line {at}: {e}"))?;
                if !families.last().is_some_and(|f| belongs(f, &sample.name)) {
                    if families.iter().any(|f| f.name == sample.name) {
                        return Err(format!(
                            "line {at}: family {} again, after another",
                            sample.name
                        ));
                    }
                    families.push(Family {
                        name: sample.name.clone(),
                        ..Family::default()
                    });
                }
                let family = families.last_mut().unwrap();
                if family
                    .samples
                    .iter()
                    .any(|s| s.name == sample.name && s.labels == sample.labels)
                {
                    return Err(format!("line {at}: a series twice"));
                }
                family.samples.push(sample);
            }
        }
        Ok(families)
    }
}

/// Feed the reader expositions the text format refuses → each refused; and the format's own
/// example → read (T-RQ0.9.4: the reader that judges a scrape is no reader of everything).
#[test]
fn t_rq0_9_4_the_openmetrics_reader_refuses_what_the_format_refuses() {
    let example = "# TYPE acme_http_router_request_seconds summary\n# UNIT acme_http_router_request_seconds seconds\n# HELP acme_http_router_request_seconds Latency though all of ACME's HTTP request router.\nacme_http_router_request_seconds_sum{path=\"/api/v1\",method=\"GET\"} 9036.32\nacme_http_router_request_seconds_count{path=\"/api/v1\",method=\"GET\"} 807283.0\nacme_http_router_request_seconds_created{path=\"/api/v1\",method=\"GET\"} 1605281325.0\n# TYPE go_goroutines gauge\n# HELP go_goroutines Number of goroutines that currently exist.\ngo_goroutines 69\n# TYPE process_cpu_seconds counter\n# UNIT process_cpu_seconds seconds\n# HELP process_cpu_seconds Total user and system CPU time spent in seconds.\nprocess_cpu_seconds_total 4.20072246e+06\n# EOF\n";
    let read = om::parse(example).unwrap();
    assert_eq!(read.len(), 3);
    assert_eq!(read[0].samples.len(), 3);
    for bad in [
        "# TYPE foo counter\nfoo_total 1\n",
        "# TYPE foo counter\nfoo 1\n# EOF\n",
        "# TYPE foo counter\n# TYPE foo gauge\n# EOF\n",
        "# TYPE foo counter\n# UNIT foo seconds\n# EOF\n",
        "# HELP foo x\n# TYPE foo gauge\n# EOF\n",
        "# TYPE foo gauge\nfoo 1\n# TYPE bar gauge\nbar 1\nfoo 2\n# EOF\n",
        "# TYPE foo gauge\nfoo{a=\"x} 1\n# EOF\n",
        "# TYPE foo gauge\nfoo{a=\"x\",a=\"y\"} 1\n# EOF\n",
        "# TYPE foo gauge\nfoo 1.2.3\n# EOF\n",
        "# TYPE foo gauge\nfoo 1\r\n# EOF\n",
        "# TYPE foo gauge\n# comment\n# EOF\n",
        "# TYPE foo gauge\nfoo 1\nfoo 2\n# EOF\n",
    ] {
        assert!(om::parse(bad).is_err(), "read: {bad:?}");
    }
}

// ---------------------------------------------------------------------------------------------------
// A scrape of a running stub (T-RQ0.9.4).

/// The `Accept` kiln's scrape sends, Prometheus's own (kiln's crates/kiln/src/observe/scrape.rs,
/// `ACCEPT`).
const KILN_ACCEPT: &str = "application/openmetrics-text;version=1.0.0;q=0.5,application/openmetrics-text;version=0.0.1;q=0.4,text/plain;version=0.0.4;q=0.3,*/*;q=0.1";

/// The time every line of these tests is stamped with.
fn at() -> SystemTime {
    UNIX_EPOCH + Duration::from_secs(1_795_207_863)
}

/// A stub that answers one scrape: the ring's metrics at `/metrics`, as a node's door will. What
/// it was asked.
fn serve_one(listener: &TcpListener, ring: &EventRing) -> String {
    let (mut stream, _) = listener.accept().unwrap();
    let mut request = Vec::new();
    let mut piece = [0_u8; 1024];
    while !request.windows(4).any(|w| w == b"\r\n\r\n") {
        let n = stream.read(&mut piece).unwrap();
        assert!(n > 0, "the scraper closed before its request ended");
        request.extend_from_slice(&piece[..n]);
    }
    let mut body = String::new();
    openmetrics::encode(&ring.families(), &mut body).unwrap();
    write!(
        stream,
        "HTTP/1.1 200 OK\r\nContent-Type: {CONTENT_TYPE}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
    .unwrap();
    String::from_utf8(request).unwrap()
}

/// Scrapes `addr` as kiln does: its status line, its headers (names in lowercase) and its body.
fn scrape(addr: SocketAddr) -> (String, BTreeMap<String, String>, String) {
    let mut stream = TcpStream::connect(addr).unwrap();
    write!(
        stream,
        "GET /metrics HTTP/1.1\r\nHost: {addr}\r\nAccept: {KILN_ACCEPT}\r\nConnection: close\r\n\r\n"
    )
    .unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    let (head, body) = response.split_once("\r\n\r\n").unwrap();
    let mut lines = head.split("\r\n");
    let status = lines.next().unwrap().to_owned();
    let headers = lines
        .map(|l| {
            let (name, value) = l.split_once(':').unwrap();
            (name.trim().to_ascii_lowercase(), value.trim().to_owned())
        })
        .collect();
    (status, headers, body.to_owned())
}

/// Emit events into a ring (one more than it holds), then scrape a stub serving its metrics →
/// a 200 of OpenMetrics' content type whose every sample parses, belongs to a family with a TYPE
/// and a HELP, is a metric of the catalog's, and says what the ring did (T-RQ0.9.4).
#[test]
fn t_rq0_9_4_a_scrape_of_a_running_stub_is_openmetrics_with_a_help_and_a_type() {
    let ring = Arc::new(
        EventRing::new(Bounds {
            lines: 2,
            line_bytes: 4096,
        })
        .unwrap(),
    );
    let subscriber = Registry::default().with(RingLayer::new(Arc::clone(&ring), at));
    tracing::subscriber::with_default(subscriber, || {
        for n in 0..3_u64 {
            tracing::info!(target: "reliquary.events", event = "events.scraped", n, "an event before the scrape");
        }
    });
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let stub = {
        let ring = Arc::clone(&ring);
        thread::spawn(move || serve_one(&listener, &ring))
    };
    let (status, headers, body) = scrape(addr);
    let asked = stub.join().unwrap();
    assert!(asked.starts_with("GET /metrics HTTP/1.1\r\n"), "{asked}");
    assert_eq!(status, "HTTP/1.1 200 OK");
    assert_eq!(
        headers.get("content-type").map(String::as_str),
        Some(CONTENT_TYPE)
    );
    let families = om::parse(&body).unwrap_or_else(|e| panic!("{e}\n{body}"));
    let mut samples = 0;
    for family in &families {
        assert!(
            family.kind.is_some(),
            "{} has no TYPE:\n{body}",
            family.name
        );
        assert!(
            family.help.as_deref().is_some_and(|h| !h.is_empty()),
            "{} has no HELP:\n{body}",
            family.name
        );
        for sample in &family.samples {
            samples += 1;
            assert!(
                METRICS.iter().any(|m| m.name == sample.name),
                "{} is no metric of the catalog's",
                sample.name
            );
        }
    }
    let catalog_series: usize = METRICS.iter().map(|m| m.series().unwrap()).sum();
    assert_eq!(
        samples, catalog_series,
        "every series of every metric:\n{body}"
    );
    let value = |name: &str, labels: &[(&str, &str)]| {
        families
            .iter()
            .flat_map(|f| &f.samples)
            .find(|s| {
                s.name == name
                    && s.labels
                        .iter()
                        .map(|(l, v)| (l.as_str(), v.as_str()))
                        .eq(labels.iter().copied())
            })
            .unwrap_or_else(|| panic!("{name} {labels:?}:\n{body}"))
            .value
    };
    assert_eq!(value(catalog::EVENTS_EMITTED.name, &[]), 3.0);
    assert_eq!(
        value(catalog::EVENTS_DROPPED.name, &[("reason", "evicted")]),
        1.0
    );
    assert_eq!(value(catalog::EVENTS_BUDGET_BYTES.name, &[]), 8192.0);
}

// ---------------------------------------------------------------------------------------------------
// A flood (T-RQ0.9.5).

/// Four threads emit a thousand events each through the layer into a ring of sixteen lines while
/// the sink takes nothing → every emitter ends while the sink is stalled (none waited for it); the
/// sink's batch is the `events.dropped` line saying 3,984 were evicted, then the newest sixteen,
/// each emitter's a run ending at its last; the counters say the same; and every event is
/// delivered or counted, none lost unsaid (T-RQ0.9.5).
#[test]
fn t_rq0_9_5_a_flood_drops_the_oldest_counts_them_and_never_holds_up_an_emitter() {
    const LINES: usize = 16;
    const EMITTERS: u64 = 4;
    const EACH: u64 = 1000;
    let ring = Arc::new(
        EventRing::new(Bounds {
            lines: LINES,
            line_bytes: 512,
        })
        .unwrap(),
    );
    let dispatch =
        tracing::Dispatch::new(Registry::default().with(RingLayer::new(Arc::clone(&ring), at)));
    let (release, stalled) = mpsc::channel::<()>();
    let sink = {
        let ring = Arc::clone(&ring);
        thread::spawn(move || {
            stalled.recv().unwrap();
            let mut out = Vec::new();
            let drained = ring.drain(at(), &mut out);
            (drained, String::from_utf8(out).unwrap())
        })
    };
    let emitters: Vec<_> = (0..EMITTERS)
        .map(|emitter| {
            let dispatch = dispatch.clone();
            thread::spawn(move || {
                tracing::dispatcher::with_default(&dispatch, || {
                    for n in 0..EACH {
                        tracing::info!(target: "reliquary.events", event = "events.flood", emitter, n, "a flood");
                    }
                });
            })
        })
        .collect();
    for emitter in emitters {
        emitter.join().unwrap();
    }
    release.send(()).unwrap();
    let (drained, batch) = sink.join().unwrap();

    let total = EMITTERS * EACH;
    let kept = u64::try_from(LINES).unwrap();
    let evicted = total - kept;
    assert_eq!(drained.lines, LINES);
    assert_eq!(drained.lost.evicted, evicted);
    assert_eq!((drained.lost.oversize, drained.lost.closed), (0, 0));
    let lines: Vec<serde_json::Value> = batch
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert_eq!(lines.len(), LINES + 1, "{batch}");
    let marker = &lines[0];
    assert_eq!(marker["target"], catalog::EVENTS_DROPPED_EVENT.target);
    assert_eq!(marker["event"], catalog::EVENTS_DROPPED_EVENT.name);
    assert_eq!(marker["level"], "warn");
    assert_eq!(marker["evicted"], evicted);
    let mut runs: BTreeMap<u64, Vec<u64>> = BTreeMap::new();
    for line in &lines[1..] {
        assert_eq!(line["target"], "reliquary.events");
        assert_eq!(line["event"], "events.flood");
        assert_eq!(line["level"], "info");
        assert_eq!(line["message"], "a flood");
        assert_eq!(line["timestamp"], 1_795_207_863.0);
        runs.entry(line["emitter"].as_u64().unwrap())
            .or_default()
            .push(line["n"].as_u64().unwrap());
    }
    for (emitter, ns) in &runs {
        let first = EACH - u64::try_from(ns.len()).unwrap();
        assert_eq!(
            ns,
            &(first..EACH).collect::<Vec<_>>(),
            "emitter {emitter} kept other than its newest"
        );
    }

    let families = ring.families();
    let count = |metric: &Metric, label: Option<&str>| {
        families
            .iter()
            .find(|f| f.metric.name == metric.name)
            .and_then(|f| f.series.iter().find(|s| s.labels.first().copied() == label))
            .unwrap()
            .value
    };
    assert_eq!(count(&catalog::EVENTS_EMITTED, None), total);
    assert_eq!(count(&catalog::EVENTS_DROPPED, Some("evicted")), evicted);
    assert_eq!(count(&catalog::EVENTS_HELD_BYTES, None), 0);
    let delivered = u64::try_from(drained.lines).unwrap();
    let lost: u64 = catalog::Dropped::ALL
        .iter()
        .map(|&reason| count(&catalog::EVENTS_DROPPED, Some(reason.word())))
        .sum();
    assert_eq!(total, delivered + lost, "every event delivered or counted");
}

/// Emit on another target, and open a span on Reliquary's → the ring takes neither (T-RQ0.9.5:
/// the ring holds Reliquary's events alone).
#[test]
fn t_rq0_9_5_the_ring_takes_only_events_on_reliquarys_targets() {
    let ring = Arc::new(
        EventRing::new(Bounds {
            lines: 4,
            line_bytes: 512,
        })
        .unwrap(),
    );
    let subscriber = Registry::default().with(RingLayer::new(Arc::clone(&ring), at));
    tracing::subscriber::with_default(subscriber, || {
        tracing::warn!(target: "hyper_quic::connection", "not Reliquary's");
        let span = tracing::info_span!(target: "reliquary.trace.events", "events.span");
        span.in_scope(|| {});
    });
    let mut out = Vec::new();
    assert_eq!(ring.drain(at(), &mut out).lines, 0);
    assert!(out.is_empty());
}
