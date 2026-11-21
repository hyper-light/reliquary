//! What Reliquary's tests share: the skip record (plan RQ0.3, AC-RQ0.3.5, T-RQ0.3.6), kept as Hex
//! keeps its own (Hex's crates/hex-testkit, HEX-GATE-1). A test that cannot run where the gate runs
//! (it needs a privilege, a device, a tool or the network, and the runner lacks it) says so with
//! [`skip`] and returns: a line on stderr, which libtest captures and shows only for a test that
//! fails, and the same line appended to the skip record, which libtest's capture can't reach.
//! scripts/test.sh points `RELIQUARY_SKIPS` at [`RECORD`] under the target directory, empties it
//! before `cargo test` and prints it after, pass or fail, so a CI log names every skipped test, what
//! it lacked and why. scripts/check-tests.py refuses a test's other early returns, `#[ignore]` and a
//! `cfg` that leaves a test out: a test is compiled everywhere and skips through here.
//!
//! The record grows by one line a skip, and scripts/test.sh empties it before each suite; a line is
//! as long as the four fields its test passes, each a text the test itself holds.

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

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// The variable naming the skip record, which scripts/test.sh sets.
pub const VAR: &str = "RELIQUARY_SKIPS";

/// The skip record under the target directory: where scripts/test.sh points `RELIQUARY_SKIPS`, and
/// where a skip goes without it.
pub const RECORD: &str = "tmp/reliquary-skips.tsv";

/// Says that `test` skipped, lacking `missing`, and `why`: on stderr and in the skip record
/// ([`record`]), one line of four tab-separated fields (the test binary, `test`, `missing`, `why`).
/// Refused when the record can't be found or written, so a skip that can't be said fails its test
/// instead of passing unseen.
pub fn skip(test: &str, missing: &str, why: &str) -> io::Result<()> {
    skip_to(&record()?, test, missing, why)
}

/// As [`skip`], to the record at `path`.
pub fn skip_to(path: &Path, test: &str, missing: &str, why: &str) -> io::Result<()> {
    let line = line(&binary(), test, missing, why);
    io::stderr()
        .lock()
        .write_all(format!("SKIP {line}").as_bytes())?;
    append(path, &line)
}

/// The skip record this test process writes: `RELIQUARY_SKIPS` when set, else [`RECORD`] under the
/// target directory the running test binary sits in (the nearest directory above it that holds
/// cargo's `CACHEDIR.TAG`).
pub fn record() -> io::Result<PathBuf> {
    if let Some(path) = std::env::var_os(VAR) {
        return Ok(PathBuf::from(path));
    }
    let exe = std::env::current_exe()?;
    exe.ancestors()
        .skip(1)
        .find(|dir| dir.join("CACHEDIR.TAG").is_file())
        .map(|target| target.join(RECORD))
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!(
                    "no target directory (one holding CACHEDIR.TAG) above {}, and {VAR} unset",
                    exe.display()
                ),
            )
        })
}

/// Appends `line` to the record at `path`, making its directory first: one write of the whole line
/// to a file opened for appending, so lines from test processes running at once land whole (an
/// append's write goes to the end of the file in one step: POSIX `write`, O_APPEND; Windows,
/// FILE_APPEND_DATA).
fn append(path: &Path, line: &str) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?
        .write_all(line.as_bytes())
}

/// The record's line: four fields, each with its tabs and line breaks made spaces.
fn line(binary: &str, test: &str, missing: &str, why: &str) -> String {
    let flat = |s: &str| s.replace(['\t', '\n', '\r'], " ");
    format!(
        "{}\t{}\t{}\t{}\n",
        flat(binary),
        flat(test),
        flat(missing),
        flat(why)
    )
}

/// The running test binary's name, without the hash cargo appends (`gates-0123456789abcdef`).
fn binary() -> String {
    let exe = std::env::current_exe().ok();
    let stem = exe
        .as_deref()
        .and_then(Path::file_stem)
        .and_then(|s| s.to_str())
        .unwrap_or("an unnamed test binary");
    match stem.rsplit_once('-') {
        Some((name, hash)) if !hash.is_empty() && hash.bytes().all(|b| b.is_ascii_hexdigit()) => {
            name.to_owned()
        }
        _ => stem.to_owned(),
    }
}

#[cfg(test)]
mod tests;
