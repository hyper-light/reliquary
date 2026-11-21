//! The rule on numbers, as a gate (plan RQ0.2; docs/design/constants.md): scripts/check-contracts.py
//! on fixture trees, a rule at a time and every violation at once, and on this tree, whose every
//! production number has a row of a kind the rule accepts and whose open list is empty.

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
use std::process::Command;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

/// What the checker said: whether it passed, its exit status and everything it printed.
struct Said {
    passed: bool,
    status: Option<i32>,
    text: String,
}

impl Said {
    /// Its failures: every line but the closing count.
    fn failures(&self) -> Vec<&str> {
        self.text
            .lines()
            .filter(|l| !l.starts_with("check-contracts:"))
            .collect()
    }

    /// The failures holding each of `parts`.
    fn lines_with(&self, parts: &[&str]) -> Vec<&str> {
        self.failures()
            .into_iter()
            .filter(|l| parts.iter().all(|p| l.contains(p)))
            .collect()
    }

    /// The failure holding each of `parts`, which must be exactly one.
    fn one(&self, parts: &[&str]) -> &str {
        let lines = self.lines_with(parts);
        assert_eq!(lines.len(), 1, "one line with {parts:?} in:\n{}", self.text);
        lines[0]
    }
}

/// scripts/check-contracts.py run with `args`.
fn run(args: &[&str]) -> Said {
    let out = Command::new("python3")
        .arg(root().join("scripts/check-contracts.py"))
        .args(args)
        .output()
        .expect("python3 runs scripts/check-contracts.py");
    Said {
        passed: out.status.success(),
        status: out.status.code(),
        text: format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ),
    }
}

/// A tree the checker runs on: its Rust files, its inventory's lines and its open list.
struct Tree {
    dir: tempfile::TempDir,
    inventory: String,
    open: String,
}

const LIB: &str = "crates/fixture/src/lib.rs";

impl Tree {
    fn new() -> Self {
        Self {
            dir: tempfile::tempdir().unwrap(),
            inventory:
                "# Operating constants\n\n| Number | Kind | Value | Basis |\n|---|---|---|---|\n"
                    .to_owned(),
            open: "# Open items: none.\n".to_owned(),
        }
    }

    fn file(&mut self, rel: &str, text: &str) -> &mut Self {
        let path = self.dir.path().join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
        self
    }

    /// A row of the inventory, as `| `file` `key` | kind | `value` | basis |`.
    fn row(&mut self, file: &str, key: &str, kind: &str, value: &str, basis: &str) -> &mut Self {
        self.line(&format!(
            "| `{file}` `{key}` | {kind} | `{value}` | {basis} |"
        ))
    }

    fn line(&mut self, line: &str) -> &mut Self {
        self.inventory.push_str(line);
        self.inventory.push('\n');
        self
    }

    fn open_item(&mut self, item: &str) -> &mut Self {
        self.open.push_str(item);
        self.open.push('\n');
        self
    }

    fn check(&self) -> Said {
        let root = self.dir.path();
        std::fs::create_dir_all(root.join("docs/design")).unwrap();
        std::fs::create_dir_all(root.join("scripts")).unwrap();
        std::fs::write(root.join("docs/design/constants.md"), &self.inventory).unwrap();
        std::fs::write(root.join("scripts/arbitrary-open.txt"), &self.open).unwrap();
        run(&["--root", root.to_str().unwrap()])
    }
}

/// A tree whose one crate's lib.rs is `lib`.
fn with_lib(lib: &str) -> Tree {
    let mut tree = Tree::new();
    tree.file(LIB, lib);
    tree
}

// ---------------------------------------------------------------------------------------------------
// The plan's tests (T-RQ0.2.1 to T-RQ0.2.7).

/// Add `const RETRIES: u32 = 3;` with no row → the gate fails naming the file and the key
/// (T-RQ0.2.1).
#[test]
fn t_rq0_2_1_a_number_with_no_row_fails_naming_its_file_and_key() {
    let said = with_lib("pub const RETRIES: u32 = 3;\n").check();
    assert!(!said.passed, "{}", said.text);
    said.one(&["crates/fixture/src/lib.rs:1: RETRIES = 3 has no row in docs/design/constants.md"]);
}

/// Add a row whose kind is `picked`, and one of each other kind the rule refuses → the gate fails on
/// each, naming the kind (T-RQ0.2.2).
#[test]
fn t_rq0_2_2_a_row_of_a_refused_kind_fails() {
    for kind in [
        "picked",
        "knee",
        "once",
        "copied",
        "guessed",
        "inherited",
        "open",
    ] {
        let mut tree = with_lib("pub const RETRIES: u32 = 3;\n");
        tree.row(LIB, "RETRIES", kind, "3", "Three tries.");
        let said = tree.check();
        assert!(!said.passed, "{kind}: {}", said.text);
        said.one(&[&format!("RETRIES = 3: {kind}, which the rule refuses")]);
    }
}

/// The fixture's test of a derived constant `WINDOW`, so that only the basis is wrong.
const WINDOW: &str = "pub const WINDOW: u32 = 16;\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn window_is_its_model() {\n        assert_eq!(super::WINDOW, 2 * 8);\n    }\n}\n";

/// Add a `derived` row whose basis reads "measured on fedora" → the gate fails with "one machine's
/// measurement"; so does a `learned` row measured on one machine, and each with a point read off one
/// curve (T-RQ0.2.3, AC-RQ0.2.1).
#[test]
fn t_rq0_2_3_a_learned_or_derived_row_measured_on_one_machine_or_curve_fails() {
    let derived = "Model: two round trips of eight. Inputs: the round trip, {}. Test: `crates/fixture/src/lib.rs` `window_is_its_model`.";
    let learned = "Measures: the slowest answer. On: each path. Kept: per path. Follows: {}.";
    for (kind, basis) in [("derived", derived), ("learned", learned)] {
        for (phrase, what) in [
            ("measured on fedora", "one machine's measurement"),
            ("measured once", "one machine's measurement"),
            ("timed on this machine", "one machine's measurement"),
            (
                "the knee of the latency curve",
                "a point read off one curve",
            ),
            ("read off the measured curve", "a point read off one curve"),
        ] {
            let mut tree = with_lib(WINDOW);
            tree.row(LIB, "WINDOW", kind, "16", &basis.replace("{}", phrase));
            let said = tree.check();
            assert!(!said.passed, "{kind}, {phrase}: {}", said.text);
            said.one(&[&format!("WINDOW: a {kind} row whose basis is {what}")]);
            assert_eq!(said.failures().len(), 1, "{kind}, {phrase}: {}", said.text);
        }
    }
}

/// Change a listed constant's value in code only → the gate fails and shows both values
/// (T-RQ0.2.4).
#[test]
fn t_rq0_2_4_a_value_changed_in_code_alone_fails_showing_both() {
    let mut tree = with_lib("pub const RETRIES: u32 = 4;\n");
    tree.row(LIB, "RETRIES", "external", "3", "A fixture's.");
    let said = tree.check();
    assert!(!said.passed, "{}", said.text);
    said.one(&["RETRIES: the row says `3`, the code `4` (crates/fixture/src/lib.rs:1)"]);
}

const WAIT_AND_ROOM: &str = "use std::time::Duration;\n\npub fn wait() -> Duration {\n    Duration::from_millis(500)\n}\n\npub fn room() -> Vec<u8> {\n    Vec::with_capacity(64)\n}\n";

/// A literal passed as a duration or a capacity (`Duration::from_millis(500)`, `with_capacity(64)`)
/// → found, failing with no row and passing with one; a changed literal leaves its row naming nothing
/// (T-RQ0.2.5).
#[test]
fn t_rq0_2_5_a_literal_duration_or_capacity_is_held_to_a_row() {
    let said = with_lib(WAIT_AND_ROOM).check();
    assert!(!said.passed, "{}", said.text);
    said.one(&["lib.rs:4: wait:from_millis(500) = 500 has no row"]);
    said.one(&["lib.rs:8: room:with_capacity(64) = 64 has no row"]);

    let mut tree = with_lib(WAIT_AND_ROOM);
    tree.row(
        LIB,
        "wait:from_millis(500)",
        "external",
        "500",
        "A fixture's.",
    )
    .row(
        LIB,
        "room:with_capacity(64)",
        "external",
        "64",
        "A fixture's.",
    );
    let said = tree.check();
    assert!(said.passed, "{}", said.text);

    tree.file(LIB, &WAIT_AND_ROOM.replace("500", "600"));
    let said = tree.check();
    assert!(!said.passed, "{}", said.text);
    said.one(&["wait:from_millis(500) names no number in production code"]);
    said.one(&["wait:from_millis(600) = 600 has no row"]);
}

const NO_CHOICE: &str = "pub struct Tally {\n    pub count: u64,\n}\n\npub fn tally(xs: &[u64]) -> (Tally, u64, usize, Vec<u8>) {\n    let mut product = 1;\n    for x in xs {\n        product = x.saturating_mul(product);\n    }\n    let floor = xs.len().max(1);\n    let next = floor.checked_add(1).unwrap_or(floor);\n    (Tally { count: 0 }, product, next, Vec::with_capacity(0))\n}\n";

/// Zero as an empty count, and one as a first value, a `max(1)` floor or an increment → no row asked;
/// the same places holding 2 → each asks for one (T-RQ0.2.6).
#[test]
fn t_rq0_2_6_zero_and_one_as_no_choice_need_no_row() {
    let said = with_lib(NO_CHOICE).check();
    assert!(said.passed, "{}", said.text);
    assert!(
        said.text.contains("check-contracts: 0 production numbers"),
        "{}",
        said.text
    );

    let chosen = NO_CHOICE
        .replace("count: 0", "count: 2")
        .replace("product = 1", "product = 2")
        .replace("max(1)", "max(2)")
        .replace("checked_add(1)", "checked_add(2)")
        .replace("with_capacity(0)", "with_capacity(2)");
    let said = with_lib(&chosen).check();
    assert!(!said.passed, "{}", said.text);
    for key in [
        "tally:count = 2",
        "tally:product = 2",
        "tally:max(2) = 2",
        "tally:checked_add(2) = 2",
        "tally:with_capacity(2) = 2",
    ] {
        said.one(&[key, "has no row"]);
    }
    assert_eq!(said.failures().len(), 5, "{}", said.text);
}

const EVERY_VIOLATION: &str = "pub const UNLISTED: u32 = 7;\npub const DRIFTED: u32 = 4;\npub const PICKED: u32 = 8;\npub const ON_FEDORA: u32 = 16;\npub const KNEE: u32 = 32;\npub const COPIED: u32 = 10;\npub const UNPLACED: u32 = 5;\npub const UNMEASURED: u32 = 6;\npub const UNTESTED: u32 = 12;\npub const UNNAMED: u32 = 14;\npub const NO_SUCH_TEST: u32 = 18;\npub const ODD_KIND: u32 = 20;\npub const BLANK: u32 = 22;\npub const TWICE: u32 = 24;\npub const FORGIVEN: u32 = 26;\n\npub fn room() -> Vec<u8> {\n    Vec::with_capacity(64)\n}\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn on_fedora_is_its_model() {\n        assert_eq!(super::ON_FEDORA, 2 * 8);\n    }\n\n    #[test]\n    fn unnamed_is_its_model() {\n        assert_eq!(2 * 7, 14);\n    }\n\n    #[test]\n    fn room_is_its_model() {\n        assert_eq!(64, 64);\n    }\n}\n";

/// Run the checker on one tree holding one of each violation → each is reported once, by its key,
/// and nothing else is (T-RQ0.2.7).
#[test]
fn t_rq0_2_7_one_tree_with_each_violation_reports_each_once_by_name() {
    let fixture = "A fixture's.";
    let mut tree = with_lib(EVERY_VIOLATION);
    tree.row(LIB, "GONE", "external", "1", fixture)
        .row(LIB, "DRIFTED", "external", "3", fixture)
        .row(LIB, "PICKED", "picked", "8", "Eight felt right.")
        .row(LIB, "ON_FEDORA", "derived", "16", "Model: two round trips of eight. Inputs: the round trip, measured on fedora. Test: `crates/fixture/src/lib.rs` `on_fedora_is_its_model`.")
        .row(LIB, "KNEE", "learned", "32", "Measures: latency. On: each path. Kept: per path. Follows: by aging. Frozen at the knee of the latency curve.")
        .row(LIB, "COPIED", "cited", "10", "RocksDB's default.")
        .row(LIB, "UNPLACED", "learned", "5", "Measures: answers. On: the node. Kept: in memory. Follows: by aging.")
        .row(LIB, "UNMEASURED", "learned", "6", "Measures: answers. On: each peer. Kept: in memory.")
        .row(LIB, "UNTESTED", "derived", "12", "Model: three fours. Inputs: four.")
        .row(LIB, "UNNAMED", "derived", "14", "Model: two sevens. Inputs: seven. Test: `crates/fixture/src/lib.rs` `unnamed_is_its_model`.")
        .row(LIB, "NO_SUCH_TEST", "derived", "18", "Model: two nines. Inputs: nine. Test: `crates/fixture/src/lib.rs` `absent`.")
        .row(LIB, "room:with_capacity(64)", "derived", "64", "Model: a frame. Inputs: its fields. Test: `crates/fixture/src/lib.rs` `room_is_its_model`.")
        .row(LIB, "ODD_KIND", "tuned", "20", "Tuned.")
        .row(LIB, "BLANK", "external", "22", "")
        .row(LIB, "TWICE", "external", "24", fixture)
        .row(LIB, "TWICE", "external", "24", fixture)
        .row(LIB, "FORGIVEN", "external", "26", fixture)
        .line("| `crates/fixture/src/lib.rs` `MALFORMED` | external | 30 | A value without its backticks. |")
        .line("\n## Vendored crates\n\n- `vendor/absent`: a fixture's.")
        .open_item("crates/fixture/src/lib.rs FORGIVEN");
    let said = tree.check();
    assert!(!said.passed, "{}", said.text);
    let each: &[&[&str]] = &[
        &["UNLISTED = 7 has no row"],
        &["GONE names no number in production code"],
        &["DRIFTED: the row says `3`, the code `4`"],
        &["PICKED = 8: picked, which the rule refuses"],
        &["ON_FEDORA: a derived row whose basis is one machine's measurement"],
        &["KNEE: a learned row whose basis is a point read off one curve"],
        &["COPIED: a cited row whose basis is another system's tuning (RocksDB)"],
        &[
            "UNPLACED: a learned row's On: names no path, device, peer, repository or workload class",
        ],
        &["UNMEASURED: a learned row names", "it has no Follows:"],
        &["UNTESTED: a derived row names", "it has no Test:"],
        &["UNNAMED: its test unnamed_is_its_model does not name UNNAMED"],
        &["NO_SUCH_TEST: its Test: names no #[test] fn absent in crates/fixture/src/lib.rs"],
        &["room:with_capacity(64): a derived value is held by its test only as a named constant"],
        &["ODD_KIND: unknown kind tuned"],
        &["BLANK: states no basis"],
        &["TWICE: a second row"],
        &["FORGIVEN: Reliquary forgives no number"],
        &["a row must be | `file` `key` | kind | `value` | basis |"],
        &["vendor/absent: no such vendored path"],
    ];
    for parts in each {
        said.one(parts);
    }
    assert_eq!(said.failures().len(), each.len(), "{}", said.text);
}

// ---------------------------------------------------------------------------------------------------
// The criteria the plan's tests do not reach alone (AC-RQ0.2.1 to AC-RQ0.2.4).

/// Run the gate with `--known`, and with an open list naming an item → the first is refused as a usage
/// error, the second fails; this tree's open list names nothing (AC-RQ0.2.2).
#[test]
fn ac_rq0_2_2_the_open_list_is_empty_and_the_gate_refuses_one_that_is_not() {
    let said = run(&["--known", "scripts/arbitrary-open.txt"]);
    assert_eq!(said.status, Some(2), "{}", said.text);
    assert!(said.text.contains("There is no --known"), "{}", said.text);

    let mut tree = with_lib("pub const KEPT: u32 = 8;\n");
    tree.row(LIB, "KEPT", "external", "8", "A fixture's.")
        .open_item("crates/fixture/src/lib.rs KEPT");
    let said = tree.check();
    assert!(!said.passed, "{}", said.text);
    said.one(&["scripts/arbitrary-open.txt:2: crates/fixture/src/lib.rs KEPT: Reliquary forgives no number"]);

    let ours = std::fs::read_to_string(root().join("scripts/arbitrary-open.txt")).unwrap();
    let items: Vec<&str> = ours
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect();
    assert!(
        items.is_empty(),
        "scripts/arbitrary-open.txt names {items:?}"
    );
}

/// A `learned` row with what it measures, on what, where it is kept and how it follows a change →
/// passes; one lacking any of them, or measured on the node alone → fails naming what it lacks
/// (AC-RQ0.2.3).
#[test]
fn ac_rq0_2_3_a_learned_row_names_what_on_what_where_and_how_it_follows() {
    let whole = [
        ("Measures", "the slowest answer of a call kind"),
        ("On", "each path, a peer and its route"),
        ("Kept", "the path's record, merged across processes"),
        (
            "Follows",
            "the record ages, so a slower or faster path moves it within its samples",
        ),
    ];
    let basis = |skip: Option<&str>, on: &str| {
        whole
            .iter()
            .filter(|(label, _)| Some(*label) != skip)
            .map(|(label, text)| format!("{label}: {}.", if *label == "On" { on } else { text }))
            .collect::<Vec<_>>()
            .join(" ")
    };
    let mut tree = with_lib("pub const SEEN: u32 = 9;\n");
    tree.row(LIB, "SEEN", "learned", "9", &basis(None, whole[1].1));
    let said = tree.check();
    assert!(said.passed, "{}", said.text);

    for (label, _) in whole {
        let mut tree = with_lib("pub const SEEN: u32 = 9;\n");
        tree.row(LIB, "SEEN", "learned", "9", &basis(Some(label), whole[1].1));
        let said = tree.check();
        assert!(!said.passed, "{label}: {}", said.text);
        said.one(&["SEEN: a learned row names", &format!("it has no {label}:")]);
    }

    for alone in ["the node", "this machine", "the host it runs on"] {
        let mut tree = with_lib("pub const SEEN: u32 = 9;\n");
        tree.row(LIB, "SEEN", "learned", "9", &basis(None, alone));
        let said = tree.check();
        assert!(!said.passed, "{alone}: {}", said.text);
        said.one(&[
            "SEEN: a learned row's On: names no path, device, peer, repository or workload class",
        ]);
    }
}

const TWICE_EIGHT: &str = "pub const SIXTEEN: u32 = 2 * EIGHT;\npub const EIGHT: u32 = 8;\n\npub fn room() -> Vec<u8> {\n    Vec::with_capacity(16)\n}\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn sixteen_is_two_eights() {\n        assert_eq!(super::SIXTEEN, 2 * 8);\n    }\n\n    #[test]\n    fn sixteen_alone() {\n        assert_eq!(16, 2 * 8);\n    }\n}\n";

/// A `derived` row with its model, its inputs and a test that computes it and names it → passes; one
/// lacking a part, naming a test that is not there or does not name it, or on a literal rather than
/// a named constant → fails; a `counted` row likewise names the test that pins it (AC-RQ0.2.4).
#[test]
fn ac_rq0_2_4_a_derived_row_names_its_model_inputs_and_the_test_that_computes_it() {
    let test = |name: &str| format!("Test: `crates/fixture/src/lib.rs` `{name}`.");
    let model = "Model: two of a pair. Inputs: EIGHT.";
    let tree = |sixteen: &str, room: &str| {
        let mut tree = with_lib(TWICE_EIGHT);
        tree.row(LIB, "EIGHT", "external", "8", "A fixture's.")
            .row(LIB, "SIXTEEN", "derived", "2 * EIGHT", sixteen)
            .row(LIB, "room:with_capacity(16)", "external", "16", room);
        tree.check()
    };
    let said = tree(
        &format!("{model} {}", test("sixteen_is_two_eights")),
        "A fixture's.",
    );
    assert!(said.passed, "{}", said.text);

    for (basis, phrase) in [
        (
            format!("Inputs: EIGHT. {}", test("sixteen_is_two_eights")),
            "it has no Model:".to_owned(),
        ),
        (
            format!("Model: two of a pair. {}", test("sixteen_is_two_eights")),
            "it has no Inputs:".to_owned(),
        ),
        (model.to_owned(), "it has no Test:".to_owned()),
        (
            format!("{model} {}", test("absent")),
            "names no #[test] fn absent".to_owned(),
        ),
        (
            format!("{model} {}", test("sixteen_alone")),
            "its test sixteen_alone does not name SIXTEEN".to_owned(),
        ),
        (
            format!("{model} Test: `crates/fixture/src/gone.rs` `sixteen_is_two_eights`."),
            "names crates/fixture/src/gone.rs, which does not exist".to_owned(),
        ),
    ] {
        let said = tree(&basis, "A fixture's.");
        assert!(!said.passed, "{basis}: {}", said.text);
        said.one(&["SIXTEEN: ", &phrase]);
    }

    let said = tree(
        &format!("{model} {}", test("sixteen_is_two_eights")),
        "A fixture's.",
    );
    assert!(said.passed, "{}", said.text);
    let mut literal = with_lib(TWICE_EIGHT);
    literal
        .row(LIB, "EIGHT", "external", "8", "A fixture's.")
        .row(LIB, "SIXTEEN", "external", "2 * EIGHT", "A fixture's.")
        .row(
            LIB,
            "room:with_capacity(16)",
            "derived",
            "16",
            &format!("{model} {}", test("sixteen_alone")),
        );
    let said = literal.check();
    assert!(!said.passed, "{}", said.text);
    said.one(&["room:with_capacity(16): a derived value is held by its test only as a named constant: name it"]);

    let mut counted = with_lib(TWICE_EIGHT);
    counted
        .row(LIB, "EIGHT", "counted", "8", "The bytes of one entry.")
        .row(
            LIB,
            "SIXTEEN",
            "counted",
            "2 * EIGHT",
            &format!("Two entries. {}", test("sixteen_is_two_eights")),
        )
        .row(
            LIB,
            "room:with_capacity(16)",
            "external",
            "16",
            "A fixture's.",
        );
    let said = counted.check();
    assert!(!said.passed, "{}", said.text);
    said.one(&["EIGHT: a counted row names the test that pins its value (Test:): it has no Test:"]);
    assert_eq!(said.failures().len(), 1, "{}", said.text);
}

/// A reading rule on a crate of its own: the crate, its files (path and text), and what the checker
/// must say of it, each once.
type Case = (
    &'static str,
    &'static [(&'static str, &'static str)],
    &'static [&'static str],
);

/// Mantle's split of test code from production code, its literal parameters, and Reliquary's further
/// reading: every other literal but 0 and 1, wherever it stands.
const READS: &[Case] = &[
    (
        "ordinary",
        &[("src/lib.rs", "const MISSED: u64 = 1;\n")],
        &["src/lib.rs:1: MISSED = 1 has no row"],
    ),
    (
        "a_marked_test_file",
        &[
            (
                "src/lib.rs",
                "#[cfg(test)]\n#[path = \"checks.rs\"]\nmod checks;\n",
            ),
            ("src/checks.rs", "#![cfg(test)]\nconst MISSED: u64 = 1;\n"),
        ],
        &[],
    ),
    (
        "marked_after_its_docs",
        &[
            ("src/lib.rs", "mod checks;\n"),
            (
                "src/checks.rs",
                "//! Checks.\n\n#![cfg(test)]\nconst MISSED: u64 = 1;\n",
            ),
        ],
        &[],
    ),
    (
        "only_test_declarations",
        &[
            (
                "src/lib.rs",
                "#[cfg(test)]\n#[path = \"shared.rs\"]\nmod checks;\n",
            ),
            ("src/shared.rs", "const MISSED: u64 = 1;\n"),
        ],
        &[],
    ),
    (
        "a_production_declaration_too",
        &[
            (
                "src/lib.rs",
                "#[cfg(test)]\n#[path = \"shared.rs\"]\nmod checks;\n#[path = \"shared.rs\"]\nmod real;\n",
            ),
            ("src/shared.rs", "const MISSED: u64 = 1;\n"),
        ],
        &["src/shared.rs:1: MISSED = 1 has no row"],
    ),
    (
        "an_include_too",
        &[
            (
                "src/lib.rs",
                "include!(\"shared.rs\");\n#[cfg(test)]\n#[path = \"shared.rs\"]\nmod checks;\n",
            ),
            ("src/shared.rs", "const MISSED: u64 = 1;\n"),
        ],
        &["src/shared.rs:1: MISSED = 1 has no row"],
    ),
    (
        "a_marker_not_first",
        &[
            ("src/lib.rs", "mod checks;\n"),
            ("src/checks.rs", "const MISSED: u64 = 1;\n#![cfg(test)]\n"),
        ],
        &["MISSED = 1 has no row"],
    ),
    (
        "another_inner_attribute",
        &[
            ("src/lib.rs", "mod checks;\n"),
            ("src/checks.rs", "#![cfg(unix)]\nconst MISSED: u64 = 1;\n"),
        ],
        &["MISSED = 1 has no row"],
    ),
    (
        "a_test_function",
        &[(
            "src/lib.rs",
            "#[cfg(test)]\nfn f() {\n    let missed = 3;\n}\n",
        )],
        &[],
    ),
    (
        "a_test_attribute",
        &[("src/lib.rs", "#[test]\nfn f() {\n    let missed = 3;\n}\n")],
        &[],
    ),
    (
        "after_a_test_module",
        &[(
            "src/lib.rs",
            "#[cfg(test)]\nmod tests {\n    const T: u8 = 2;\n}\nconst MISSED: u64 = 1;\n",
        )],
        &["MISSED = 1 has no row"],
    ),
    (
        "all_of_test_and_unix",
        &[(
            "src/lib.rs",
            "#[cfg(all(unix, test))]\nconst MISSED: u64 = 1;\n",
        )],
        &[],
    ),
    (
        "not_test",
        &[("src/lib.rs", "#[cfg(not(test))]\nconst MISSED: u64 = 1;\n")],
        &["MISSED = 1 has no row"],
    ),
    (
        "a_struct_constant",
        &[("src/lib.rs", "const MISSED: Rounds = Rounds { min: 3 };\n")],
        &["MISSED = Rounds { min: 3 } has no row"],
    ),
    (
        "a_literal_field",
        &[(
            "src/lib.rs",
            "fn plan() -> Plan {\n    Plan { depth: 4 }\n}\n",
        )],
        &["plan:depth = 4 has no row"],
    ),
    (
        "a_command_line_default",
        &[(
            "src/lib.rs",
            "struct A {\n    #[arg(long, default_value_t = 16)]\n    clients: usize,\n    #[arg(long, default_value = \"30\")]\n    seconds: u64,\n}\n",
        )],
        &[
            "A:--clients = 16 has no row",
            "A:--seconds = \"30\" has no row",
        ],
    ),
    (
        "a_zero_is_no_choice",
        &[("src/lib.rs", "fn f() -> S {\n    S { count: 0 }\n}\n")],
        &[],
    ),
    (
        "a_ladder",
        &[(
            "src/lib.rs",
            "fn f() -> Vec<u32> {\n    vec![1, 4, 16]\n}\n",
        )],
        &["f:[1, 4, 16] = [1, 4, 16] has no row"],
    ),
    (
        "a_comparison",
        &[("src/lib.rs", "pub fn f(n: u32) -> bool {\n    n > 3\n}\n")],
        &["f:3 = 3 has no row"],
    ),
    (
        "an_operand",
        &[(
            "src/lib.rs",
            "pub fn rto(srtt: u64) -> u64 {\n    srtt / 8\n}\n",
        )],
        &["rto:8 = 8 has no row"],
    ),
    (
        "operands_are_read_one_by_one",
        &[(
            "src/lib.rs",
            "pub fn f(x: u64) -> u64 {\n    x * 2 + 3\n}\n",
        )],
        &["f:2 = 2 has no row", "f:3 = 3 has no row"],
    ),
    (
        "a_whole_run",
        &[(
            "src/lib.rs",
            "pub fn week() -> u64 {\n    7 * 24 * 3600\n}\n",
        )],
        &["week:7 * 24 * 3600 = 7 * 24 * 3600 has no row"],
    ),
    (
        "a_shift",
        &[(
            "src/lib.rs",
            "pub fn f() -> Vec<u8> {\n    Vec::with_capacity(1 << 20)\n}\n",
        )],
        &["f:with_capacity(1 << 20) = 1 << 20 has no row"],
    ),
    (
        "a_sign",
        &[("src/lib.rs", "pub fn f() -> S {\n    S { offset: -5 }\n}\n")],
        &["f:offset = -5 has no row"],
    ),
    (
        "any_call_s_argument",
        &[(
            "src/lib.rs",
            "pub fn f(s: &str) -> Option<u32> {\n    u32::from_str_radix(s, 16).ok()\n}\n",
        )],
        &["f:from_str_radix(16) = 16 has no row"],
    ),
    (
        "an_array_s_length",
        &[("src/lib.rs", "pub fn f() -> [u8; 32] {\n    [0; 32]\n}\n")],
        &["f:32 = 32 has no row", "f:32#2 = 32 has no row"],
    ),
    (
        "a_discriminant",
        &[("src/lib.rs", "pub enum Kind {\n    A = 2,\n    B = 3,\n}\n")],
        &["Kind:A = 2 has no row", "Kind:B = 3 has no row"],
    ),
    (
        "an_attribute_s_argument",
        &[("src/lib.rs", "#[repr(align(64))]\npub struct Line(u8);\n")],
        &["align(64) = 64 has no row"],
    ),
    (
        "a_tuple_s_field",
        &[(
            "src/lib.rs",
            "pub fn f(p: (u8, u8, u8)) -> u8 {\n    p.2\n}\n",
        )],
        &[],
    ),
    (
        "an_increment",
        &[(
            "src/lib.rs",
            "pub fn f(n: u32) -> Option<u32> {\n    n.checked_add(1)\n}\n",
        )],
        &[],
    ),
    (
        "a_cell",
        &[(
            "src/lib.rs",
            "use std::sync::atomic::AtomicU64;\npub static HITS: AtomicU64 = AtomicU64::new(0);\npub static SEED: AtomicU64 = AtomicU64::new(7);\n",
        )],
        &["new(7) = 7 has no row"],
    ),
    (
        "an_associated_constant",
        &[(
            "src/lib.rs",
            "pub struct S;\nimpl S {\n    pub const N: usize = 4;\n}\n",
        )],
        &["S::N = 4 has no row"],
    ),
    (
        "one_name_under_two_cfgs",
        &[(
            "src/lib.rs",
            "#[cfg(unix)]\nconst W: u32 = 4;\n#[cfg(windows)]\nconst W: u32 = 8;\n",
        )],
        &["W = 4 has no row", "W#2 = 8 has no row"],
    ),
    (
        "a_build_script",
        &[
            ("src/lib.rs", ""),
            ("build.rs", "const B: u32 = 3;\nfn main() {}\n"),
        ],
        &["build.rs:1: B = 3 has no row"],
    ),
    (
        "tests_benches_and_examples",
        &[
            ("src/lib.rs", ""),
            ("tests/t.rs", "const T: u32 = 3;\n"),
            ("benches/b.rs", "const B: u32 = 3;\n"),
            ("examples/e.rs", "const E: u32 = 3;\n"),
        ],
        &[],
    ),
];

/// Run the checker on a tree with a crate for each reading rule and no rows → each crate's numbers are
/// asked for, each once, and nothing else of it is (AC-RQ0.2.1: every number production code runs
/// with, none of test code's).
#[test]
fn ac_rq0_2_1_the_checker_reads_every_number_production_code_runs_with() {
    let mut tree = Tree::new();
    for (name, files, _) in READS {
        for (rel, text) in *files {
            tree.file(&format!("crates/{name}/{rel}"), text);
        }
    }
    let said = tree.check();
    for (name, _, expect) in READS {
        let crate_path = format!("crates/{name}/");
        let lines = said.lines_with(&[&crate_path]);
        for phrase in *expect {
            let with: Vec<&&str> = lines.iter().filter(|l| l.contains(phrase)).collect();
            assert_eq!(
                with.len(),
                1,
                "{name}: one line with {phrase:?} in {lines:#?}"
            );
        }
        assert_eq!(lines.len(), expect.len(), "{name}: {lines:#?}");
    }
}

/// A vendored crate's number → not read; named in the inventory's "Vendored crates" section → read
/// like Reliquary's own (AC-RQ0.2.1; docs/design/constants.md, "Vendored crates").
#[test]
fn ac_rq0_2_1_a_vendored_path_is_read_where_the_inventory_names_it() {
    let mut tree = Tree::new();
    tree.file("vendor/demo/src/lib.rs", "pub const V: u32 = 9;\n")
        .file("vendor/demo/tests/t.rs", "const T: u32 = 3;\n");
    let said = tree.check();
    assert!(said.passed, "{}", said.text);

    tree.line("\n## Vendored crates\n\n- `vendor/demo`: a fixture's estimator.");
    let said = tree.check();
    assert!(!said.passed, "{}", said.text);
    said.one(&["vendor/demo/src/lib.rs:1: V = 9 has no row"]);
    assert_eq!(said.failures().len(), 1, "{}", said.text);
}

/// Run the gate on this tree → every number Reliquary's production code runs with has a row of a
/// kind the rule accepts, every row names one, and nothing is forgiven (AC-RQ0.2.1, AC-RQ0.2.2).
#[test]
fn ac_rq0_2_1_every_production_number_here_has_a_row_of_an_accepted_kind() {
    let said = run(&[]);
    assert!(said.passed, "{}", said.text);
}
