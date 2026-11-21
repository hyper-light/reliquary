//! docs/design/numbers.md held to the plan and to the code (plan RQ0.2, AC-RQ0.2.5): each class of
//! tunable names what replaces its constant and where that model's inputs are measured; the lists of
//! what an operator states, what a node measures and what it derives name plan items that exist and
//! code that exists; the configuration type is what the document says it is.

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
use std::path::{Path, PathBuf};
use std::process::Command;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(root().join(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"))
}

/// A table's header and rows, each a list of cells.
struct Table {
    header: Vec<String>,
    rows: Vec<Vec<String>>,
}

impl Table {
    /// The cell of `row` under the column `name`.
    fn cell<'a>(&self, row: &'a [String], name: &str) -> &'a str {
        let at = self
            .header
            .iter()
            .position(|h| h == name)
            .unwrap_or_else(|| panic!("no column {name:?} in {:?}", self.header));
        row.get(at).map_or("", String::as_str)
    }
}

/// A row's cells: split on each `|` that is not escaped, a `\|` read as `|`.
fn cells(line: &str) -> Vec<String> {
    let inner = line.trim().strip_prefix('|').unwrap_or(line.trim());
    let inner = inner.strip_suffix('|').unwrap_or(inner);
    let mut out = Vec::new();
    let mut cell = String::new();
    let mut chars = inner.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' && chars.peek() == Some(&'|') {
            cell.push('|');
            chars.next();
        } else if c == '|' {
            out.push(cell.trim().to_owned());
            cell.clear();
        } else {
            cell.push(c);
        }
    }
    out.push(cell.trim().to_owned());
    out
}

/// The first table under each `###` heading, by the heading's text.
fn tables(doc: &str) -> BTreeMap<String, Table> {
    let mut out = BTreeMap::new();
    let mut heading: Option<&str> = None;
    let mut lines: Vec<&str> = Vec::new();
    for line in doc.lines().chain(std::iter::once("")) {
        if line.starts_with('|') {
            lines.push(line);
            continue;
        }
        if let (Some(h), [header, _separator, rows @ ..]) = (heading, lines.as_slice()) {
            out.insert(
                h.to_owned(),
                Table {
                    header: cells(header),
                    rows: rows.iter().map(|l| cells(l)).collect(),
                },
            );
            heading = None;
        }
        lines.clear();
        if let Some(h) = line.strip_prefix("### ") {
            heading = Some(h.trim());
        } else if line.starts_with("## ") {
            heading = None;
        }
    }
    out
}

/// Every plan item's ID the registry holds (`id = "RQ…"`).
fn plan_items() -> BTreeSet<String> {
    read("docs/requirements.toml")
        .lines()
        .filter_map(|l| l.strip_prefix("id = \"RQ"))
        .filter_map(|l| l.strip_suffix('"'))
        .map(|id| format!("RQ{id}"))
        .collect()
}

/// The plan items a cell names: each `RQn.m` in it.
fn named_items(cell: &str) -> Vec<String> {
    cell.split(|c: char| !(c.is_ascii_alphanumeric() || c == '.'))
        .filter(|w| w.starts_with("RQ"))
        .map(|w| w.trim_end_matches('.').to_owned())
        .collect()
}

/// Whether `text` holds `word` as a whole identifier.
fn has_word(text: &str, word: &str) -> bool {
    let ident = |c: Option<char>| c.is_some_and(|c| c.is_ascii_alphanumeric() || c == '_');
    text.match_indices(word).any(|(at, _)| {
        let before = text.get(..at).and_then(|s| s.chars().next_back());
        let after = text.get(at + word.len()..).and_then(|s| s.chars().next());
        !ident(before) && !ident(after)
    })
}

const ACCEPTED: [&str; 6] = [
    "format", "external", "derived", "cited", "learned", "counted",
];
const CLASSES: [&str; 7] = [
    "Waits",
    "Concurrency",
    "Sizes of chunks, pieces and windows",
    "Batches",
    "Queues and caches",
    "Rates",
    "Lifetimes",
];
const LISTS: [&str; 3] = [
    "What an operator states",
    "What a node measures",
    "What a node derives",
];
/// What a learned value may be kept under (AC-RQ0.2.3, AC-RQ0.8.4): never the machine alone.
const SUBJECTS: [&str; 5] = ["path", "device", "peer", "repository", "workload class"];

fn doc() -> BTreeMap<String, Table> {
    tables(&read("docs/design/numbers.md"))
}

/// Read each class of tunable (waits, concurrency, sizes, batches, queues and caches, rates,
/// lifetimes) → each has rows, each row names what replaces its constant and where its inputs are
/// measured, of kinds the rule accepts, a learned one kept under a path, device, peer, repository or
/// workload class, and plan items that exist (AC-RQ0.2.5, AC-RQ0.2.3).
#[test]
fn ac_rq0_2_5_each_class_of_tunable_names_its_model_and_where_its_inputs_are_measured() {
    let doc = doc();
    let items = plan_items();
    for class in CLASSES {
        let table = doc
            .get(class)
            .unwrap_or_else(|| panic!("numbers.md has no class {class:?}"));
        assert!(!table.rows.is_empty(), "{class}: no rows");
        for row in &table.rows {
            let what = table.cell(row, "What");
            for column in [
                "In Reliquary",
                "Its inputs, and where they are measured",
                "Kept under",
            ] {
                assert!(
                    !table.cell(row, column).is_empty(),
                    "{class}, {what}: no {column:?}"
                );
            }
            let kinds = table.cell(row, "Kind");
            for kind in kinds.split(',').map(str::trim) {
                assert!(ACCEPTED.contains(&kind), "{class}, {what}: kind {kind:?}");
            }
            if kinds.contains("learned") {
                let kept = table.cell(row, "Kept under");
                assert!(
                    SUBJECTS.iter().any(|s| kept.contains(s)),
                    "{class}, {what}: learned, kept under {kept:?}"
                );
            }
            let plan = named_items(table.cell(row, "Plan"));
            assert!(!plan.is_empty(), "{class}, {what}: no plan item");
            for item in plan {
                assert!(
                    items.contains(&item),
                    "{class}, {what}: {item} is not in the registry"
                );
            }
        }
    }
}

/// Read the lists of what an operator states, what a node measures and what it derives → each has
/// rows; each row names plan items that exist; each "In code" is "—" or a file that declares the name
/// it gives (AC-RQ0.2.5).
#[test]
fn ac_rq0_2_5_the_lists_name_plan_items_and_code_that_exist() {
    let doc = doc();
    let items = plan_items();
    for list in LISTS {
        let table = doc
            .get(list)
            .unwrap_or_else(|| panic!("numbers.md has no list {list:?}"));
        assert!(!table.rows.is_empty(), "{list}: no rows");
        for row in &table.rows {
            let name = row.first().map_or("", String::as_str);
            let plan = named_items(table.cell(row, "Plan"));
            assert!(!plan.is_empty(), "{list}, {name}: no plan item");
            for item in plan {
                assert!(
                    items.contains(&item),
                    "{list}, {name}: {item} is not in the registry"
                );
            }
            let code = table.cell(row, "In code");
            if code != "—" {
                let spans: Vec<&str> = code.split('`').skip(1).step_by(2).collect();
                let [file, item] = spans.as_slice() else {
                    panic!("{list}, {name}: In code {code:?} is not `file` `name`")
                };
                let text = std::fs::read_to_string(root().join(file))
                    .unwrap_or_else(|e| panic!("{list}, {name}: {file}: {e}"));
                let last = item.rsplit("::").next().unwrap_or(*item);
                assert!(
                    has_word(&text, last),
                    "{list}, {name}: {file} declares no {item}"
                );
            }
        }
    }
}

/// Read what an operator states → nothing a node can measure: no timeout, retry count, queue length,
/// cache size, chunk size or concurrency (AC-RQ0.2.5; AC-RQ1.9.2).
#[test]
fn ac_rq0_2_5_the_operator_states_nothing_a_node_can_measure() {
    let doc = doc();
    let table = &doc["What an operator states"];
    let refused = [
        "timeout",
        "retry",
        "retries",
        "queue",
        "cache",
        "chunk",
        "concurrency",
        "pool",
        "batch",
        "window",
    ];
    for row in &table.rows {
        let said = format!(
            "{} {}",
            table.cell(row, "Setting"),
            table.cell(row, "What it is")
        )
        .to_lowercase();
        for word in refused {
            assert!(
                !has_word(&said, word),
                "the operator would state {word}: {said:?}"
            );
        }
    }
}

/// Read the configuration type's line, then pass each operator setting to `reliquary` → the document
/// says there is none yet, no setting is marked built, and the binary refuses each as a misuse
/// (AC-RQ0.2.5). When `reliquary serve` comes (RQ1.9), the line names the type, and this test holds
/// the operator's list to its fields.
#[test]
fn ac_rq0_2_5_the_configuration_type_is_none_yet_and_the_binary_takes_no_setting() {
    let text = read("docs/design/numbers.md");
    assert!(
        text.contains("The configuration type: none yet."),
        "numbers.md names a configuration type: hold the operator's list to its fields here"
    );
    let doc = doc();
    let table = &doc["What an operator states"];
    for row in &table.rows {
        let setting = table.cell(row, "Setting").trim_matches('`');
        assert_eq!(
            table.cell(row, "In code"),
            "—",
            "{setting}: built, with no configuration type"
        );
        let out = Command::new(env!("CARGO_BIN_EXE_reliquary"))
            .args([format!("--{setting}"), "1".to_owned()])
            .output()
            .expect("the binary runs");
        assert_eq!(out.status.code(), Some(2), "--{setting}: {out:?}");
    }
}

/// Read the inventory's `learned` rows → each measures something "What a node measures" lists
/// (AC-RQ0.2.5, AC-RQ0.2.3).
#[test]
fn ac_rq0_2_5_every_learned_number_is_a_measurement_the_list_names() {
    let doc = doc();
    let measured: Vec<String> = doc["What a node measures"]
        .rows
        .iter()
        .filter_map(|r| r.first().cloned())
        .collect();
    let inventory = read("docs/design/constants.md");
    for line in inventory
        .lines()
        .filter(|l| l.starts_with("| `crates/") || l.starts_with("| `vendor/"))
    {
        let row = cells(line);
        if row.get(1).map(String::as_str) != Some("learned") {
            continue;
        }
        let basis = row.get(3).map_or("", String::as_str);
        let measures = basis.split("Measures:").nth(1).unwrap_or("");
        let measures = measures.split(" On:").next().unwrap_or(measures);
        assert!(
            measured.iter().any(|m| has_word(measures, m)),
            "{}: measures {measures:?}, which numbers.md's list does not name",
            row[0]
        );
    }
}
