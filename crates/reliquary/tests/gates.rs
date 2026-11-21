//! The structural gates (plan RQ0.3), each run on a fixture that plants what it must refuse: recursion,
//! direct and through trait objects, and calls through pointers that cannot recur passing (T-RQ0.3.1,
//! T-RQ0.3.2, AC-RQ0.3.3); `unsafe` outside the files CLAUDE.md names (T-RQ0.3.3); a dependency with an
//! advisory and one with a license outside the allow list (T-RQ0.3.4); a test citing an ID the
//! registry lacks, or none (T-RQ0.3.5, AC-RQ0.3.4); a skip, and the record printed after the suite, pass
//! or fail (T-RQ0.3.6, AC-RQ0.3.5); a test's early return, `#[ignore]` and `cfg` (T-RQ0.3.7); the gate's
//! order and its stop at the first failure (AC-RQ0.3.1); and the platforms and distributions CI runs
//! the gates on (AC-RQ0.3.2, AC-RQ0.3.7, T-RQ0.3.8, T-RQ0.3.9).

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
use std::process::{Command, Output};

/// The workspace's root, as cargo gives the crate's (never canonicalized: on Windows that is a `\\?\`
/// path, which cargo refuses in a manifest).
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap()
        .to_path_buf()
}

/// bash, the first on PATH: on Windows a bare `bash` would be found in System32 before PATH, and
/// that one is WSL's, not the bash the gate runs in.
fn bash() -> PathBuf {
    let name = if cfg!(windows) { "bash.exe" } else { "bash" };
    std::env::var_os("PATH")
        .and_then(|path| {
            std::env::split_paths(&path)
                .map(|dir| dir.join(name))
                .find(|candidate| candidate.is_file())
        })
        .unwrap_or_else(|| PathBuf::from(name))
}

fn script(name: &str) -> PathBuf {
    root().join("scripts").join(name)
}

/// What a command printed, both streams, and how it ended.
fn ran(command: &mut Command) -> (Output, String) {
    let out = command
        .output()
        .unwrap_or_else(|e| panic!("{command:?} didn't run: {e}"));
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    (out, text)
}

/// A command run in a fixture as the gate runs it, with none of this test run's own cargo settings
/// (a target directory shared with the outer build would wait on its lock).
fn command(program: impl AsRef<std::ffi::OsStr>, dir: &Path) -> Command {
    let mut command = Command::new(program);
    command
        .current_dir(dir)
        .env_remove("CARGO_TARGET_DIR")
        .env_remove("RUSTFLAGS")
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .env_remove("RELIQUARY_SKIPS");
    command
}

/// A workspace of one crate, `fixture`, with the given files beside its manifest (`src/lib.rs`, …),
/// pinned to this workspace's toolchain, and its lockfile made without the network.
fn fixture(files: &[(&str, &str)], dependencies: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let at = dir.path();
    std::fs::copy(
        root().join("rust-toolchain.toml"),
        at.join("rust-toolchain.toml"),
    )
    .unwrap();
    std::fs::write(
        at.join("Cargo.toml"),
        "[workspace]\nmembers = [\"crates/*\"]\nresolver = \"3\"\n",
    )
    .unwrap();
    let krate = at.join("crates").join("fixture");
    std::fs::create_dir_all(&krate).unwrap();
    std::fs::write(
        krate.join("Cargo.toml"),
        format!(
            "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\nedition = \"2024\"\nlicense = \"MIT\"\npublish = false\n\n[dependencies]\n{dependencies}\n"
        ),
    )
    .unwrap();
    for (name, text) in files {
        let path = krate.join(name);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }
    let (out, text) = ran(command("cargo", at).args(["generate-lockfile", "--offline"]));
    assert!(out.status.success(), "the fixture's lockfile: {text}");
    dir
}

fn recursion(workspace: &Path) -> (Output, String) {
    ran(command("python3", workspace)
        .arg(script("check-recursion.py"))
        .arg("--workspace")
        .arg(workspace))
}

/// Add a function that calls itself → the recursion gate fails and names it, with its file and line
/// (T-RQ0.3.1, AC-RQ0.3.3).
#[test]
fn t_rq0_3_1_a_function_that_calls_itself_fails_the_recursion_gate_named() {
    let dir = fixture(
        &[(
            "src/lib.rs",
            "pub fn walk(n: u32) -> u32 {\n    if n == 0 { 0 } else { walk(n - 1) }\n}\n\npub fn flat(n: u32) -> u32 {\n    (0..n).sum()\n}\n",
        )],
        "",
    );
    let (out, text) = recursion(dir.path());
    assert!(!out.status.success(), "{text}");
    assert!(
        text.contains(
            "a call-graph cycle (CLAUDE.md §11): rewrite it as flat work: fixture::walk\n"
        ),
        "{text}"
    );
    assert!(text.contains("src/lib.rs:1  fixture::walk"), "{text}");
    assert!(!text.contains("fixture::flat"), "{text}");
}

/// Add two functions that call each other through trait objects, each made one where they are
/// joined → the gate fails, naming both and the calls through vtables that join them (T-RQ0.3.2,
/// AC-RQ0.3.3).
#[test]
fn t_rq0_3_2_two_functions_calling_each_other_through_trait_objects_fail_named() {
    let dir = fixture(
        &[(
            "src/lib.rs",
            r"pub trait Even { fn even(&self, n: u32) -> bool; }
pub trait Odd { fn odd(&self, n: u32) -> bool; }
pub struct E<'a> { pub odd: &'a dyn Odd }
pub struct O<'a> { pub even: &'a dyn Even }
impl Even for E<'_> {
    fn even(&self, n: u32) -> bool { n == 0 || self.odd.odd(n.saturating_sub(1)) }
}
impl Odd for O<'_> {
    fn odd(&self, n: u32) -> bool { n != 0 && self.even.even(n.saturating_sub(1)) }
}
pub struct Zero;
impl Even for Zero { fn even(&self, n: u32) -> bool { n == 0 } }
pub fn parity(n: u32) -> bool {
    let zero = Zero;
    let odd = O { even: &zero };
    let even = E { odd: &odd };
    let start: &dyn Even = &even;
    start.even(n)
}
",
        )],
        "",
    );
    let (out, text) = recursion(dir.path());
    assert!(!out.status.success(), "{text}");
    assert!(
        text.contains("rewrite it as flat work: <fixture::E as fixture::Even>::even | <fixture::O as fixture::Odd>::odd\n"),
        "{text}"
    );
    assert!(text.contains("calls <fixture::E as fixture::Even>::even → <fixture::O as fixture::Odd>::odd: through a trait object (slot 3, `odd`)"), "{text}");
    assert!(text.contains("calls <fixture::O as fixture::Odd>::odd → <fixture::E as fixture::Even>::even: through a trait object (slot 3, `even`)"), "{text}");
    assert!(
        !text.contains("fixture::Zero as"),
        "Zero's even calls nothing: {text}"
    );
}

/// Twelve types each calling the next through `dyn Ring`, each made a trait object, so the module
/// holds twelve vtables (rustc names those past the tenth `@vtable.a`, `@vtable.b`) → one cycle naming
/// all twelve, every vtable read (T-RQ0.3.2, AC-RQ0.3.3).
#[test]
fn a_ring_of_twelve_trait_objects_is_one_cycle_with_every_vtable_read() {
    let mut src = String::from("pub trait Ring { fn step(&self, n: u32) -> u32; }\n");
    for i in 0..12 {
        src.push_str(&format!(
            "pub struct R{i}<'a>(pub Option<&'a dyn Ring>);\nimpl Ring for R{i}<'_> {{\n    fn step(&self, n: u32) -> u32 {{\n        match self.0 {{ Some(next) if n > 0 => next.step(n - 1), _ => n }}\n    }}\n}}\n"
        ));
    }
    src.push_str("pub fn spin(n: u32) -> u32 {\n    let r11 = R11(None);\n");
    for i in (0..11).rev() {
        src.push_str(&format!("    let r{i} = R{i}(Some(&r{}));\n", i + 1));
    }
    src.push_str("    let start: &dyn Ring = &r0;\n    start.step(n)\n}\n");
    let dir = fixture(&[("src/lib.rs", &src)], "");
    let (out, text) = recursion(dir.path());
    assert!(!out.status.success(), "{text}");
    let mut ring: Vec<String> = (0..12)
        .map(|i| format!("<fixture::R{i} as fixture::Ring>::step"))
        .collect();
    ring.sort();
    let named = format!(
        "a call-graph cycle (CLAUDE.md §11): rewrite it as flat work: {}\n",
        ring.join(" | ")
    );
    assert!(text.contains(&named), "{text}");
    assert!(text.contains(" 12 vtables;"), "{text}");
}

/// Calls through pointers that cannot come back (a boxed trait object dropped inside another trait's
/// object, a boxed future awaiting a helper's boxed future, a table of cases a runner calls through
/// function pointers) → no cycle; an async function that boxes and awaits itself → its poll and its
/// drop each named as a cycle, and so is a future whose own `poll` polls a boxed future of its kind
/// (AC-RQ0.3.3).
#[test]
fn calls_through_pointers_are_cycles_only_where_they_can_come_back() {
    let dir = fixture(
        &[(
            "src/lib.rs",
            r"use std::future::Future;
use std::pin::Pin;
pub type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;
pub trait Other { fn other(&self, n: u32) -> bool; }
pub trait Service { fn serve(&self) -> u32; }
pub struct X;
impl Other for X { fn other(&self, n: u32) -> bool { n > 3 } }
pub struct Holder { pub h: Box<dyn Other + Send> }
impl Service for Holder { fn serve(&self) -> u32 { u32::from(self.h.other(2)) } }
pub fn boxed() -> Box<dyn Service> { Box::new(Holder { h: Box::new(X) }) }
pub fn helper(n: u32) -> BoxFuture<u32> { Box::pin(async move { n.wrapping_add(1) }) }
pub fn case(n: u32) -> BoxFuture<u32> { Box::pin(async move { helper(n).await }) }
pub struct Case { pub check: fn(u32) -> BoxFuture<u32> }
pub const CASES: &[Case] = &[Case { check: case }, Case { check: helper }];
pub async fn run(n: u32) -> u32 {
    let mut sum = 0u32;
    for c in CASES { sum = sum.wrapping_add((c.check)(n).await); }
    sum
}
pub fn start(n: u32) -> BoxFuture<u32> { Box::pin(run(n)) }
pub fn countdown(n: u32) -> BoxFuture<u32> {
    Box::pin(async move { if n == 0 { 0 } else { countdown(n - 1).await } })
}
pub struct Spin(pub u32);
impl Future for Spin {
    type Output = u32;
    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<u32> {
        let n = self.0;
        if n == 0 { return std::task::Poll::Ready(0); }
        let mut inner: Pin<Box<dyn Future<Output = u32>>> = Box::pin(Spin(n - 1));
        inner.as_mut().poll(cx)
    }
}
",
        )],
        "",
    );
    let (out, text) = recursion(dir.path());
    assert!(!out.status.success(), "{text}");
    let cycles: Vec<&str> = text
        .lines()
        .filter_map(|l| {
            l.strip_prefix("a call-graph cycle (CLAUDE.md §11): rewrite it as flat work: ")
        })
        .collect();
    assert_eq!(
        cycles,
        [
            "<fixture::Spin as core::future::future::Future>::poll",
            "core::ptr::drop_glue::<fixture::countdown::{closure#0}>",
            "fixture::countdown::{closure#0}"
        ],
        "{text}"
    );
    assert!(text.contains(" 1 through function pointers"), "{text}");
}

/// Add `unsafe` in a file CLAUDE.md §12 does not name, beside one it names → the gate fails naming the
/// first and its line, and passes the second; a named file that uses none is to come off the list
/// (T-RQ0.3.3).
#[test]
fn t_rq0_3_3_unsafe_in_a_file_claude_md_does_not_name_fails() {
    let dir = tempfile::tempdir().unwrap();
    let at = dir.path();
    let src = at.join("crates").join("x").join("src");
    std::fs::create_dir_all(&src).unwrap();
    std::fs::write(
        at.join("CLAUDE.md"),
        "# Rules\n\n## 12. Portable\n\nWhere an OS interface needs `unsafe`, its file is named here:\n- `crates/x/src/sys.rs`: mmap.\n- `crates/x/src/gone.rs`: once.\n\n## 13. Lean code\n\nSee `crates/x/src/lib.rs`.\n",
    )
    .unwrap();
    std::fs::write(
        src.join("sys.rs"),
        "// SAFETY: the map is ours.\npub fn f() { unsafe { g() } }\nunsafe fn g() {}\n",
    )
    .unwrap();
    std::fs::write(
        src.join("lib.rs"),
        "// unsafe in a comment, and \"unsafe\" in a string, are no use of it.\npub mod sys;\n#[allow(unsafe_code)]\npub fn h() -> u8 { unsafe { *(&raw const X) } }\nstatic X: u8 = 1;\n",
    )
    .unwrap();
    std::fs::write(src.join("gone.rs"), "pub fn safe() {}\n").unwrap();
    let (out, text) = ran(command("python3", at)
        .arg(script("check-unsafe.py"))
        .arg("--root")
        .arg(at));
    assert!(!out.status.success(), "{text}");
    assert!(
        text.contains(
            "crates/x/src/lib.rs:3: `allow(unsafe_code)` in a file CLAUDE.md §12 does not name"
        ),
        "{text}"
    );
    assert!(
        text.contains("crates/x/src/lib.rs:4: `unsafe` in a file CLAUDE.md §12 does not name"),
        "{text}"
    );
    assert!(
        !text.contains("crates/x/src/lib.rs:1"),
        "a comment or a string read as code: {text}"
    );
    assert!(!text.contains("crates/x/src/sys.rs:"), "{text}");
    assert!(
        text.contains("CLAUDE.md §12 names crates/x/src/gone.rs, which does not use `unsafe`"),
        "{text}"
    );
    assert!(text.contains("3 files read"), "{text}");
}

/// What scripts/install-cargo-deny lacks when it cannot install cargo-deny here.
const DENY_LACKS: &str = "cargo-deny (scripts/install-cargo-deny: the network, or a POSIX sh)";

/// cargo-deny at its pinned release, or why the runner can't install it.
fn cargo_deny() -> Result<PathBuf, String> {
    let (out, text) = ran(Command::new("sh")
        .arg(script("install-cargo-deny"))
        .current_dir(root()));
    if !out.status.success() {
        return Err(text);
    }
    let dir = String::from_utf8(out.stdout).unwrap();
    let exe = if cfg!(windows) {
        "cargo-deny.exe"
    } else {
        "cargo-deny"
    };
    Ok(Path::new(dir.trim()).join(exe))
}

/// Install cargo-deny twice at once into an empty cache, as the suite's two deny tests do where the
/// gate has not installed it first → both succeed and both give the same working binary, neither
/// reading the other's half-written download (T-RQ0.3.4). Needs the network.
#[test]
fn t_rq0_3_4_two_installs_of_cargo_deny_at_once_each_give_a_working_one() {
    let test = "t_rq0_3_4_two_installs_of_cargo_deny_at_once_each_give_a_working_one";
    let cache = tempfile::tempdir().unwrap();
    let install = || {
        Command::new("sh")
            .arg(script("install-cargo-deny"))
            .current_dir(root())
            .env("XDG_CACHE_HOME", cache.path())
            .env_remove("RUNNER_TEMP")
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
    };
    let (Ok(first), Ok(second)) = (install(), install()) else {
        return reliquary_testkit::skip(test, "a POSIX sh", "sh didn't start")
            .expect("the skip record takes the line");
    };
    let (first, second) = (
        first.wait_with_output().unwrap(),
        second.wait_with_output().unwrap(),
    );
    let said = |out: &Output| {
        format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        )
    };
    if !first.status.success() && !second.status.success() && said(&first).contains("curl") {
        return reliquary_testkit::skip(test, "the network (GitHub's releases)", &said(&first))
            .expect("the skip record takes the line");
    }
    for out in [&first, &second] {
        assert!(out.status.success(), "{}", said(out));
        let dir = String::from_utf8(out.stdout.clone()).unwrap();
        let exe = if cfg!(windows) {
            "cargo-deny.exe"
        } else {
            "cargo-deny"
        };
        let (version, text) = ran(Command::new(Path::new(dir.trim()).join(exe)).arg("--version"));
        assert!(version.status.success(), "{text}");
        assert!(text.contains("cargo-deny 0.22.0"), "{text}");
    }
}

/// Add a dependency with a RustSec advisory (smallvec 1.6.0, RUSTSEC-2021-0003) → `cargo deny` fails
/// the advisories check, naming it, under Reliquary's deny.toml (T-RQ0.3.4). Needs the network: the
/// crate's index and the advisory database.
#[test]
fn t_rq0_3_4_a_dependency_with_an_advisory_fails_cargo_deny() {
    let test = "t_rq0_3_4_a_dependency_with_an_advisory_fails_cargo_deny";
    let deny = match cargo_deny() {
        Ok(deny) => deny,
        Err(why) => {
            return reliquary_testkit::skip(test, DENY_LACKS, &why)
                .expect("the skip record takes the line");
        }
    };
    let dir = tempfile::tempdir().unwrap();
    let at = dir.path();
    std::fs::copy(
        root().join("rust-toolchain.toml"),
        at.join("rust-toolchain.toml"),
    )
    .unwrap();
    std::fs::copy(root().join("deny.toml"), at.join("deny.toml")).unwrap();
    std::fs::create_dir_all(at.join("src")).unwrap();
    std::fs::write(at.join("src").join("lib.rs"), "pub use smallvec;\n").unwrap();
    std::fs::write(
        at.join("Cargo.toml"),
        "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\nedition = \"2024\"\nlicense = \"MIT\"\npublish = false\n\n[dependencies]\nsmallvec = \"=1.6.0\"\n",
    )
    .unwrap();
    // What needs the network, each step skipping through the record without it; then the check offline.
    let (out, text) = ran(command("cargo", at).arg("fetch"));
    if !out.status.success() {
        return reliquary_testkit::skip(test, "the network (crates.io)", &text)
            .expect("the skip record takes the line");
    }
    let (out, text) = ran(command(&deny, at).args(["fetch", "db"]));
    if !out.status.success() {
        return reliquary_testkit::skip(test, "the network (the RustSec advisory database)", &text)
            .expect("the skip record takes the line");
    }
    let (out, text) =
        ran(command(&deny, at).args(["--locked", "--offline", "check", "advisories"]));
    assert!(!out.status.success(), "{text}");
    assert!(text.contains("RUSTSEC-2021-0003"), "{text}");
    assert!(text.contains("advisories FAILED"), "{text}");
}

/// Add a dependency whose license is outside the allow list (a path crate under GPL-3.0-only) →
/// `cargo deny` fails the licenses check, naming the license (T-RQ0.3.4). No network: a path
/// crate's license is its manifest's.
#[test]
fn t_rq0_3_4_a_license_outside_the_allow_list_fails_cargo_deny() {
    let test = "t_rq0_3_4_a_license_outside_the_allow_list_fails_cargo_deny";
    let deny = match cargo_deny() {
        Ok(deny) => deny,
        Err(why) => {
            return reliquary_testkit::skip(test, DENY_LACKS, &why)
                .expect("the skip record takes the line");
        }
    };
    // The crate under GPL-3.0-only first, then the fixture that depends on it.
    let dir = tempfile::tempdir().unwrap();
    let at = dir.path();
    std::fs::create_dir_all(at.join("copyleft").join("src")).unwrap();
    std::fs::write(
        at.join("copyleft").join("src").join("lib.rs"),
        "pub fn f() {}\n",
    )
    .unwrap();
    std::fs::write(
        at.join("copyleft").join("Cargo.toml"),
        "[package]\nname = \"copyleft\"\nversion = \"0.1.0\"\nedition = \"2024\"\nlicense = \"GPL-3.0-only\"\npublish = false\n",
    )
    .unwrap();
    std::fs::copy(
        root().join("rust-toolchain.toml"),
        at.join("rust-toolchain.toml"),
    )
    .unwrap();
    std::fs::copy(root().join("deny.toml"), at.join("deny.toml")).unwrap();
    std::fs::create_dir_all(at.join("src")).unwrap();
    std::fs::write(at.join("src").join("lib.rs"), "pub use copyleft;\n").unwrap();
    std::fs::write(
        at.join("Cargo.toml"),
        "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\nedition = \"2024\"\nlicense = \"MIT\"\npublish = false\n\n[dependencies]\ncopyleft = { path = \"copyleft\" }\n",
    )
    .unwrap();
    let (out, text) = ran(command("cargo", at).args(["generate-lockfile", "--offline"]));
    assert!(out.status.success(), "{text}");
    let (out, text) = ran(command(&deny, at).args(["--locked", "--offline", "check", "licenses"]));
    assert!(!out.status.success(), "{text}");
    assert!(text.contains("GPL-3.0-only"), "{text}");
    assert!(text.contains("licenses FAILED"), "{text}");
}

/// A fixture tree for scripts/check-tests.py: crates/x with these files, and a registry holding
/// AC-RQ0.3.4 and T-RQ0.3.5.
fn tests_tree(files: &[(&str, &str)]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let at = dir.path();
    std::fs::create_dir_all(at.join("docs")).unwrap();
    std::fs::write(
        at.join("docs").join("requirements.toml"),
        "[[own]]\nid = \"RQ0.3\"\ncriteria = [\n  { id = \"AC-RQ0.3.4\", text = \"…\" },\n]\ntests = [\n  { id = \"T-RQ0.3.5\", text = \"…\" },\n]\n",
    )
    .unwrap();
    for (name, text) in files {
        let path = at.join("crates").join("x").join(name);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }
    dir
}

fn check_tests(at: &Path) -> (Output, String) {
    ran(command("python3", at)
        .arg(script("check-tests.py"))
        .arg("--root")
        .arg(at))
}

/// Add a test citing test 9 of item 99 of phase RQ9, which the registry lacks, one citing nothing, and
/// one citing what the registry holds → the citation gate fails naming the first two and passes the
/// third (T-RQ0.3.5, AC-RQ0.3.4).
#[test]
fn t_rq0_3_5_a_test_citing_an_id_the_registry_lacks_fails() {
    let dir = tests_tree(&[(
        "tests/t.rs",
        "/// Do X → Y (T-RQ9.99.9).\n#[test]\nfn unknown() {}\n\n/// Do X → Y, citing nothing.\n#[test]\nfn uncited() {}\n\n/// Do X → Y (T-RQ0.3.5, AC-RQ0.3.4).\n#[test]\n#[should_panic]\nfn cited() { panic!() }\n\n// T-RQ0.3.5 in a comment is no citation of the next test.\n#[tokio::test]\nasync fn async_uncited() {}\n",
    )]);
    let (out, text) = check_tests(dir.path());
    assert!(!out.status.success(), "{text}");
    assert!(text.contains("crates/x/tests/t.rs:3: unknown: cites T-RQ9.99.9, which docs/requirements.toml does not hold"), "{text}");
    assert!(
        text.contains("crates/x/tests/t.rs:7: uncited: cites no requirement"),
        "{text}"
    );
    assert!(
        text.contains("crates/x/tests/t.rs:16: async_uncited: cites no requirement"),
        "{text}"
    );
    assert!(!text.contains(": cited:"), "{text}");
    assert!(text.contains("3 failures in 4 tests"), "{text}");
}

/// A test that returns early without recording a skip, an `#[ignore]`d test, a test under `cfg`, a
/// test in a module under `cfg`, and a module's file declared under `cfg` → each flagged; a return
/// that records its skip, and a closure's or an async block's own return, are not (T-RQ0.3.7).
#[test]
fn t_rq0_3_7_an_early_return_without_a_skip_ignore_and_cfg_are_flagged() {
    let dir = tests_tree(&[
        (
            "src/lib.rs",
            "#[cfg(unix)]\nmod unix_only;\n#[cfg(test)]\nmod tests {\n    /// (AC-RQ0.3.4)\n    #[test]\n    fn early() {\n        if std::env::var_os(\"X\").is_none() {\n            return;\n        }\n    }\n    /// (AC-RQ0.3.4)\n    #[test]\n    fn skips() {\n        let Some(_x) = std::env::var_os(\"X\") else {\n            return reliquary_testkit::skip(\"skips\", \"X\", \"unset\").unwrap();\n        };\n        let f = |n: u32| -> u32 { if n == 0 { return 1; } n };\n        let _ = async { return 2; };\n        let _ = f(0);\n    }\n    /// (AC-RQ0.3.4)\n    #[test]\n    #[ignore = \"slow\"]\n    fn ignored() {}\n    /// (AC-RQ0.3.4)\n    #[cfg(target_os = \"linux\")]\n    #[test]\n    fn linux_only() {}\n}\n#[cfg(all(test, windows))]\nmod windows_tests {\n    /// (AC-RQ0.3.4)\n    #[test]\n    fn hidden() {}\n}\n",
        ),
        (
            "src/unix_only.rs",
            "/// (AC-RQ0.3.4)\n#[test]\nfn in_a_hidden_file() {}\n",
        ),
    ]);
    let (out, text) = check_tests(dir.path());
    assert!(!out.status.success(), "{text}");
    assert!(
        text.contains("crates/x/src/lib.rs:9: early returns early without recording a skip"),
        "{text}"
    );
    assert!(
        text.contains("ignored: #[ignore] hides it from the suite"),
        "{text}"
    );
    assert!(
        text.contains("linux_only: a cfg leaves it out where the cfg is false"),
        "{text}"
    );
    assert!(
        text.contains("hidden: it is in a module or file under a cfg"),
        "{text}"
    );
    assert!(
        text.contains("in_a_hidden_file: it is in a module or file under a cfg"),
        "{text}"
    );
    assert!(
        !text.contains(" skips "),
        "a recorded skip, a closure's or an async block's return flagged: {text}"
    );
    assert!(text.contains("5 failures in 6 tests"), "{text}");
}

/// A fixture crate whose suite has a test lacking a privilege the runner does not grant (here, the
/// variable a privileged runner would set), one that passes and one that fails, run as the gate runs
/// the suite → the run fails, and after the suite the skip record names the skipped test and what it
/// lacked; with the failing test gone the run passes and still prints it (T-RQ0.3.6, AC-RQ0.3.5).
#[test]
fn t_rq0_3_6_a_test_lacking_a_privilege_skips_into_the_record_printed_after_the_suite() {
    let testkit = root().join("crates").join("reliquary-testkit");
    let testkit = toml::Value::String(testkit.to_str().unwrap().to_owned()).to_string();
    let body = |failing: bool| {
        format!(
            "/// Use the privilege → it works, or the test skips saying what it lacked (T-RQ0.3.5).\n#[test]\nfn needs_a_privilege() {{\n    if std::env::var_os(\"FIXTURE_PRIVILEGE\").is_none() {{\n        return reliquary_testkit::skip(\"needs_a_privilege\", \"FIXTURE_PRIVILEGE, a privilege this runner lacks\", \"the runner grants none\").expect(\"the skip record\");\n    }}\n}}\n\n/// Add → the sum (AC-RQ0.3.4).\n#[test]\nfn passes() {{\n    assert_eq!(1 + 1, 2);\n}}\n{}",
            if failing {
                "\n/// Fail → a failure (AC-RQ0.3.4).\n#[test]\nfn fails() {\n    panic!(\"planted\");\n}\n"
            } else {
                ""
            }
        )
    };
    let dir = fixture(&[("src/lib.rs", ""), ("tests/t.rs", &body(true))], "");
    let at = dir.path();
    std::fs::write(
        at.join("crates").join("fixture").join("Cargo.toml"),
        format!("[package]\nname = \"fixture\"\nversion = \"0.1.0\"\nedition = \"2024\"\nlicense = \"MIT\"\npublish = false\n\n[dev-dependencies]\nreliquary-testkit = {{ path = {testkit} }}\n"),
    )
    .unwrap();
    std::fs::create_dir_all(at.join("docs")).unwrap();
    std::fs::write(
        at.join("docs").join("requirements.toml"),
        "[[own]]\nid = \"RQ0.3\"\ncriteria = [{ id = \"AC-RQ0.3.4\", text = \"…\" }]\ntests = [{ id = \"T-RQ0.3.5\", text = \"…\" }]\n",
    )
    .unwrap();
    let (out, text) = ran(command("cargo", at).args(["generate-lockfile", "--offline"]));
    assert!(out.status.success(), "{text}");
    let suite = || {
        ran(command(bash(), at)
            .arg(script("test.sh"))
            .arg("--workspace")
            .arg(at)
            .env_remove("FIXTURE_PRIVILEGE"))
    };
    let (out, text) = suite();
    assert!(
        !out.status.success(),
        "a failing test must fail the run: {text}"
    );
    let results = text.rfind("test result:").expect("the suite ran");
    let record = text
        .find("::: 1 skipped (binary, test, missing, why):")
        .unwrap_or_else(|| panic!("{text}"));
    assert!(record > results, "the record comes after the suite: {text}");
    assert!(
        text.get(record..).unwrap().contains("t\tneeds_a_privilege\tFIXTURE_PRIVILEGE, a privilege this runner lacks\tthe runner grants none"),
        "{text}"
    );
    std::fs::write(
        at.join("crates").join("fixture").join("tests").join("t.rs"),
        body(false),
    )
    .unwrap();
    let (out, text) = suite();
    assert!(out.status.success(), "{text}");
    assert!(text.contains("::: 1 skipped"), "{text}");
}

/// Read scripts/gates.sh → its gates in AC-RQ0.3.1's order: contracts, fmt, clippy for each of the
/// five targets, the production check, recursion, deny, tests, the vendor check; run it in a tree
/// whose first gate fails → it stops there with that status, no later gate run (AC-RQ0.3.1).
#[test]
fn ac_rq0_3_1_the_gate_runs_in_order_and_stops_at_the_first_failure() {
    let text = std::fs::read_to_string(script("gates.sh")).unwrap();
    let (_, all) = text
        .split_once("\nfi\n")
        .expect("the `here` branch, then the whole gate");
    let order = [
        "gate python3 scripts/check-contracts.py",
        "gate cargo fmt --all --check",
        "gate cargo clippy --workspace --all-targets --locked --target \"$t\" -- -D warnings",
        "gate bash scripts/check-production.sh",
        "gate python3 scripts/check-recursion.py",
        "gate \"$deny\" --locked check advisories bans licenses sources",
        "gate bash scripts/test.sh",
        "gate python3 scripts/vendor.py check",
    ];
    let mut at = 0;
    for step in order {
        let found = all
            .get(at..)
            .unwrap()
            .find(step)
            .unwrap_or_else(|| panic!("{step} is not in order in scripts/gates.sh"));
        at += found + step.len();
    }
    assert!(all.contains(
        "targets=(x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu aarch64-apple-darwin x86_64-pc-windows-msvc aarch64-pc-windows-msvc)"
    ));
    assert!(text.contains("set -euo pipefail"));

    let dir = tempfile::tempdir().unwrap();
    let scripts = dir.path().join("scripts");
    std::fs::create_dir_all(&scripts).unwrap();
    std::fs::copy(script("gates.sh"), scripts.join("gates.sh")).unwrap();
    std::fs::write(
        scripts.join("check-contracts.py"),
        "import sys\nsys.exit(3)\n",
    )
    .unwrap();
    let (out, text) = ran(command(bash(), dir.path()).arg(scripts.join("gates.sh")));
    assert_eq!(out.status.code(), Some(3), "{text}");
    assert!(
        text.contains("::: python3 scripts/check-contracts.py"),
        "{text}"
    );
    assert!(
        !text.contains("::: cargo fmt"),
        "a gate ran after the first failure: {text}"
    );
}

/// Read kiln CI's workflows → each of the five targets runs the gates a platform decides on every
/// push and pull request (Linux x86_64 within the whole gate), or STATUS.md names the target with no
/// runner and what would close it (AC-RQ0.3.2, T-RQ0.3.8).
#[test]
fn ac_rq0_3_2_each_target_runs_the_suite_in_ci_or_status_names_it() {
    let ci = std::fs::read_to_string(root().join(".kiln").join("ci.yaml")).unwrap();
    let platforms = std::fs::read_to_string(root().join(".kiln").join("platforms.yaml")).unwrap();
    let status = std::fs::read_to_string(root().join("STATUS.md")).unwrap();
    for workflow in [&ci, &platforms] {
        assert!(
            workflow.contains("push: { branches: [main] }")
                && workflow.contains("pull_request: {}")
        );
    }
    assert!(
        ci.contains("bash scripts/gates.sh"),
        "the whole gate, Linux x86_64"
    );
    // Each job: its key, up to the blank line before the next.
    for (job, requires) in [
        ("macos-arm64", "\"os == macos\", \"arch == aarch64\""),
        (
            "linux-arm64",
            "\"arch == aarch64\", \"executors has docker\"",
        ),
        ("windows-x86_64", "\"os == windows\", \"arch == x86_64\""),
    ] {
        let start = platforms
            .find(&format!("\n  {job}:\n"))
            .unwrap_or_else(|| panic!("no {job} job in .kiln/platforms.yaml"));
        let rest = platforms.get(start + 1..).unwrap();
        let body = rest.get(..rest.find("\n\n").unwrap_or(rest.len())).unwrap();
        assert!(body.contains(requires), "{job}: {body}");
        assert!(body.contains("scripts/gates.sh here"), "{job}: {body}");
    }
    assert!(
        status
            .lines()
            .any(|l| l.contains("aarch64-pc-windows-msvc") && l.contains("no runner")),
        "Windows on arm64 has no runner: STATUS.md must name it and what closes it"
    );
}

/// Read the distributions' workflow → the suite runs in a container of Debian, Red Hat, SUSE, Arch and
/// Alpine (musl) on x86_64, and of each on aarch64 but the one STATUS.md names with what it lacks
/// (AC-RQ0.3.7, T-RQ0.3.9).
#[test]
fn ac_rq0_3_7_each_distribution_family_runs_on_both_architectures_or_is_named() {
    let distros = std::fs::read_to_string(root().join(".kiln").join("distros.yaml")).unwrap();
    let status = std::fs::read_to_string(root().join("STATUS.md")).unwrap();
    let (x86, arm) = distros
        .split_once("\n  aarch64:\n")
        .expect("an x86_64 job, then an aarch64 job");
    for (family, named) in [
        ("debian", "Debian"),
        ("redhat", "Red Hat"),
        ("suse", "SUSE"),
        ("arch", "Arch Linux"),
        ("alpine", "Alpine"),
    ] {
        assert!(
            x86.contains(&format!("family: {family},")),
            "x86_64 lacks {family}"
        );
        if !arm.contains(&format!("family: {family},")) {
            assert!(
                status
                    .lines()
                    .any(|l| l.contains(named) && l.contains("arm64")),
                "aarch64 lacks {family}, and no line of STATUS.md names {named} on arm64"
            );
        }
    }
    assert!(arm.contains("\"arch == aarch64\", \"executors has docker\""));
    assert!(distros.contains("bash scripts/test.sh"));
    assert!(
        distros.contains("alpine") && distros.contains("musl-dev"),
        "Alpine is the musl family"
    );
}
