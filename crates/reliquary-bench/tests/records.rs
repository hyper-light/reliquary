//! The records in `bench/` (plan RQ0.7): each names its hardware, commit, date and load, and an
//! acceptance criterion cites only a record of record (AC-RQ0.7.4). A citation is a line of the
//! repository's documents that names an `AC-…` criterion and a `bench/records/…` file.

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

use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

/// The documents a citation may stand in: Markdown and TOML outside the build's and vendored trees.
fn documents(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut todo = vec![root.to_path_buf()];
    while let Some(dir) = todo.pop() {
        for entry in std::fs::read_dir(&dir).unwrap().flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();
            if path.is_dir() {
                if !matches!(name.as_str(), "target" | "vendor" | ".git" | "node_modules") {
                    todo.push(path);
                }
            } else if name.ends_with(".md") || name.ends_with(".toml") {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

/// Each `bench/records/….json` a line names.
fn records_named(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = line;
    while let Some(at) = rest.find("bench/records/") {
        let (_, tail) = rest.split_at(at);
        let end = tail
            .find(|c: char| c.is_whitespace() || matches!(c, '`' | ')' | ']' | '"' | ',' | ';'))
            .unwrap_or(tail.len());
        let (path, after) = tail.split_at(end);
        let path = path.trim_end_matches('.');
        if path.ends_with(".json") {
            out.push(path.to_owned());
        }
        rest = after;
    }
    out
}

/// The units a citation stands in: each table row alone, and each paragraph of prose, its lines
/// together, since a sentence wraps across lines (the documents wrap near a hundred columns).
fn citation_units(text: &str) -> Vec<(usize, String)> {
    let mut units = Vec::new();
    let mut paragraph: Option<(usize, String)> = None;
    for (n, line) in text.lines().enumerate() {
        let at = n + 1;
        if line.trim_start().starts_with('|') {
            units.extend(paragraph.take());
            units.push((at, line.to_owned()));
        } else if line.trim().is_empty() {
            units.extend(paragraph.take());
        } else {
            match paragraph.as_mut() {
                Some((_, p)) => {
                    p.push(' ');
                    p.push_str(line);
                }
                None => paragraph = Some((at, line.to_owned())),
            }
        }
    }
    units.extend(paragraph);
    units
}

/// Every citation by an acceptance criterion of a record that is missing or not of record.
fn bad_citations(root: &Path) -> Vec<String> {
    let mut bad = Vec::new();
    for doc in documents(root) {
        let text = std::fs::read_to_string(&doc).unwrap_or_default();
        for (line, unit) in citation_units(&text) {
            if !unit.contains("AC-") {
                continue;
            }
            for rec in records_named(&unit) {
                let at = format!("{}:{line}", doc.display());
                match std::fs::read_to_string(root.join(&rec)) {
                    Err(_) => bad.push(format!("{at}: cites {rec}, which does not exist")),
                    Ok(json) => {
                        let v: serde_json::Value =
                            serde_json::from_str(&json).unwrap_or_else(|e| panic!("{rec}: {e}"));
                        let standing = v["standing"]["standing"].as_str().unwrap_or("unsaid");
                        if standing != "of record" {
                            bad.push(format!(
                                "{at}: an acceptance criterion cites {rec}, which is {standing}"
                            ));
                        }
                    }
                }
            }
        }
    }
    bad
}

/// The repository's acceptance criteria cite no record that is missing or not of record
/// (AC-RQ0.7.4).
#[test]
fn criteria_cite_only_records_of_record() {
    let bad = bad_citations(&root());
    assert!(bad.is_empty(), "{bad:#?}");
}

/// A criterion citing a record made over the load rule → refused, naming the record and its standing;
/// citing a record of record → accepted; citing a missing record → refused (AC-RQ0.7.4, T-RQ0.7.8).
#[test]
fn a_citation_of_a_record_not_of_record_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let records = dir.path().join("bench/records");
    std::fs::create_dir_all(&records).unwrap();
    std::fs::write(
        records.join("busy.json"),
        r#"{"standing": {"standing": "not of record", "why": ["the one-minute load average was 7.10 before the first unit"]}}"#,
    )
    .unwrap();
    std::fs::write(
        records.join("quiet.json"),
        r#"{"standing": {"standing": "of record"}}"#,
    )
    .unwrap();
    std::fs::write(
        dir.path().join("STATUS.md"),
        "- pull p99 (AC-RQ4.1.3): `bench/records/busy.json`.\n\
         \n\
         - the table (AC-RQ4.2.3): bench/records/quiet.json\n\
         \n\
         - an aside, no criterion: bench/records/busy.json\n\
         \n\
         - a lost run (AC-RQ4.1.3): bench/records/gone.json\n\
         \n\
         The p99 a criterion names (AC-RQ4.1.3) as a sentence that wraps\n\
         before its record, bench/records/busy.json, is still its citation.\n\
         \n\
         | AC-RQ4.2.3 | bench/records/quiet.json |\n\
         | an aside | bench/records/busy.json |\n",
    )
    .unwrap();
    let bad = bad_citations(dir.path());
    assert_eq!(bad.len(), 3, "{bad:#?}");
    assert!(
        bad[0].contains("busy.json, which is not of record"),
        "{bad:#?}"
    );
    assert!(
        bad[1].contains("gone.json, which does not exist"),
        "{bad:#?}"
    );
    assert!(
        bad[2].contains("STATUS.md:9: ") && bad[2].contains("busy.json"),
        "{bad:#?}"
    );
}

/// Every record kept in bench/records names its hardware, the commit its tool was built from, its date
/// and the load it ran under, and says whether it is of record; at least one is kept (AC-RQ0.7.4).
#[test]
fn every_kept_record_names_hardware_commit_date_and_load() {
    let dir = root().join("bench/records");
    let mut seen = 0;
    for entry in std::fs::read_dir(&dir).unwrap().flatten() {
        let path = entry.path();
        if path.extension().is_none_or(|e| e != "json") {
            continue;
        }
        seen += 1;
        let v: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        let name = path.display();
        assert!(
            v["host"]["os"].is_string() && v["host"]["arch"].is_string(),
            "{name}: hardware"
        );
        let commit = v["tool"]
            .as_str()
            .and_then(|t| t.split_once('('))
            .and_then(|(_, rest)| rest.split_once(')'))
            .map(|(c, _)| c);
        assert!(commit.is_some_and(|c| !c.is_empty()), "{name}: commit");
        assert!(
            v["date"].as_str().is_some_and(|d| d.ends_with('Z')),
            "{name}: date"
        );
        assert!(
            v["readings"]
                .as_array()
                .is_some_and(|r| !r.is_empty() && r.iter().all(|x| x["when"].is_string())),
            "{name}: the load's readings"
        );
        assert!(v["rule"].is_string(), "{name}: the load rule");
        assert!(v["standing"]["standing"].is_string(), "{name}: standing");
    }
    assert!(seen > 0, "no record is kept in bench/records");
}
