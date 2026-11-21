//! `reliquary bench` run as a process, its workers the same binary (plan RQ0.7). These hold what a run
//! shows whatever the machine's load: the report's rows, the record's names, the verdicts that may and
//! may not be given, the gate's status following its verdicts. The statistics themselves are held exactly
//! by reliquary-bench's own seeded tests.

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

use std::io::Write;
use std::process::{Command, Output, Stdio};

fn bench(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_reliquary"))
        .arg("bench")
        .args(args)
        .output()
        .expect("the binary runs")
}

fn text(out: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

fn has_word(text: &str, word: &str) -> bool {
    text.split(|c: char| !c.is_alphanumeric())
        .any(|w| w.eq_ignore_ascii_case(word))
}

/// Measure a point of a synthetic operation → the report has a row a state with its median, the
/// wider side of its interval, its units, its tails or dashes, order in time on the first row, each
/// count with why, and no mean (AC-RQ0.7.1, AC-RQ0.7.7); a build that names no commit is not of record
/// and its record says so (AC-RQ0.7.4).
#[test]
fn a_point_reports_its_states_counts_and_standing_and_no_mean() {
    let dir = tempfile::tempdir().unwrap();
    let out = bench(&[
        "run",
        "spin",
        "--set",
        "work=2000",
        "--metric",
        "throughput",
        "--seed",
        "1",
        "--record",
        dir.path().to_str().unwrap(),
    ]);
    let said = text(&out);
    assert_eq!(out.status.code(), Some(0), "{said}");
    for needle in [
        "round:",
        "Mantle's design §5a",
        "levels:",
        "rounds:",
        "process starts:",
        "KJ13",
        "median",
        "interval",
        "units",
        "p50",
        "p99.9",
        "order in time",
    ] {
        assert!(said.contains(needle), "{needle} missing: {said}");
    }
    assert!(!has_word(&said, "mean"), "{said}");
    assert!(said.contains("NOT OF RECORD"), "{said}");
    assert!(said.contains("names no clean commit"), "{said}");
    let files: Vec<_> = std::fs::read_dir(dir.path()).unwrap().collect();
    assert_eq!(files.len(), 1);
    let record: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(files[0].as_ref().unwrap().path()).unwrap())
            .unwrap();
    assert_eq!(record["kind"], "measured");
    assert_eq!(record["standing"]["standing"], "not of record");
    assert!(record["host"]["os"].is_string() && record["host"]["cores"].is_number());
    assert!(record["date"].as_str().unwrap().ends_with('Z'));
    assert!(!record["readings"].as_array().unwrap().is_empty());
    assert!(record["tool"].as_str().unwrap().contains("unknown commit"));
}

/// A|B of two settings, B's work four times A's → never "B better", and the verdict with both
/// intervals (T-RQ0.7.6, AC-RQ0.7.2). A machine busy with other work may leave the intervals meeting,
/// so "not separated" is a right answer here; that a planted slowdown wider than the interval is "A
/// better" is held exactly by reliquary-bench's seeded tests (`ab.rs`, `driver.rs`).
#[test]
fn an_a_b_with_a_planted_slowdown_never_calls_b_better() {
    let out = bench(&[
        "ab",
        "spin",
        "--a-set",
        "work=2000",
        "--b-set",
        "work=8000",
        "--metric",
        "throughput",
        "--seed",
        "2",
    ]);
    let said = text(&out);
    assert_eq!(out.status.code(), Some(0), "{said}");
    let verdict = said
        .lines()
        .find(|l| l.starts_with("verdict: "))
        .unwrap_or_else(|| panic!("no verdict: {said}"));
    assert!(!verdict.starts_with("verdict: B better"), "{said}");
    assert!(
        verdict.starts_with("verdict: A better") || verdict.starts_with("verdict: not separated"),
        "{verdict}"
    );
    assert!(
        verdict.contains("(A ") && verdict.contains(", B "),
        "{verdict}"
    );
}

/// Open loop at the power point → each operation's latency from its intended start, its rate and
/// why said, the tails stated or dashed (AC-RQ0.7.5).
#[test]
fn an_open_loop_point_states_its_rate_and_latency_from_intended_starts() {
    let out = bench(&[
        "run",
        "spin",
        "--set",
        "work=2000",
        "--load",
        "power",
        "--metric",
        "latency:p99",
        "--seed",
        "3",
    ]);
    let said = text(&out);
    assert_eq!(out.status.code(), Some(0), "{said}");
    assert!(said.contains("open loop, Poisson at"), "{said}");
    assert!(said.contains("Kleinrock"), "{said}");
    assert!(said.contains("latency from the intended start"), "{said}");
}

/// The gate on instruction counts → each gated bench named with a verdict and both intervals, and the
/// status 1 exactly when a bench rose unaccepted (AC-RQ0.7.6). The counter is cachegrind where
/// Valgrind is installed, else the OS's where it counts a process's instructions (macOS).
#[test]
fn the_gates_status_follows_its_verdicts() {
    let dir = tempfile::tempdir().unwrap();
    let gated = dir.path().join("gated.toml");
    std::fs::write(
        &gated,
        "[[bench]]\nname = \"spin\"\nsettings = [\"work=1000\"]\nwhy = \"the test's\"\n",
    )
    .unwrap();
    let valgrind = Command::new("valgrind")
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success());
    let counter = if valgrind {
        "cachegrind"
    } else if cfg!(target_os = "macos") {
        "os"
    } else {
        return reliquary_testkit::skip(
            "the_gates_status_follows_its_verdicts",
            "an instruction counter: Valgrind on PATH, or macOS's own account",
            "neither is here",
        )
        .expect("the skip record");
    };
    let bin = env!("CARGO_BIN_EXE_reliquary");
    let out = bench(&[
        "gate",
        gated.to_str().unwrap(),
        "--a",
        bin,
        "--b",
        bin,
        "--counter",
        counter,
        "--scratch",
        dir.path().to_str().unwrap(),
        "--seed",
        "4",
    ]);
    let said = text(&out);
    let line = said
        .lines()
        .find(|l| l.starts_with("spin: "))
        .unwrap_or_else(|| panic!("no line for spin: {said}"));
    assert!(line.contains("A [") && line.contains("B ["), "{line}");
    let failed = said.contains("FAILED:");
    assert_eq!(
        out.status.code(),
        Some(if failed { 1 } else { 0 }),
        "{said}"
    );
    assert_eq!(failed, line.contains("rose: fails the gate"), "{said}");
}

/// A worker given a line that is not the protocol's → it refuses, says why, and ends with a failure,
/// never a panic (AC-RQ0.7.1; hostile input to the worker's parser).
#[test]
fn a_worker_refuses_a_hostile_line() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_reliquary"))
        .args(["bench", "worker", "--bench", "judge", "--seed", "1"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"{\"do\":\"round\",\"ops\":\"lots\"}\n")
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert_eq!(out.status.code(), Some(1), "{}", text(&out));
    let lines: Vec<String> = String::from_utf8(out.stdout)
        .unwrap()
        .lines()
        .map(str::to_owned)
        .collect();
    assert!(lines[0].contains("hello"), "{lines:?}");
    assert!(lines[1].contains("refused"), "{lines:?}");
}

/// Misuse of `reliquary bench` → status 2 and the usage (AC-RQ0.7.1).
#[test]
fn misuse_is_status_two_with_the_usage() {
    let out = bench(&["run", "spin", "--metric", "speed"]);
    assert_eq!(out.status.code(), Some(2), "{}", text(&out));
    assert!(text(&out).contains("usage: reliquary bench"));
}
