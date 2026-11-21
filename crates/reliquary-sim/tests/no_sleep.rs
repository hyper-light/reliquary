//! No simulated wait is a wall-clock sleep (plan RQ0.6): the repository's tests, every file under a
//! crate's `tests/` and `benches/` and every `#[cfg(test)]` item of its `src`, read for a thread's
//! sleep and the runtimes' sleeps, hold none (AC-RQ0.6.4). A simulated wait is a sleep on the
//! `Clock` seam, which the world ends when its virtual time reaches it. Production code that must
//! wait in real time is not a simulated wait (the measurement tool's open loop holds each operation to
//! its intended start, `reliquary-bench`'s `drive.rs`), and the scan says it saw it and passed it by.

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

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

/// What waits on the wall clock, each written in two parts so this file holds none of them whole:
/// std's thread sleeps and timed parks, and the sleeps and timers of the runtimes a crate might
/// reach for (tokio, async-std, smol, futures-timer).
fn sleeps() -> Vec<String> {
    [
        ("thread::", "sleep"),
        ("sleep", "_ms("),
        ("park", "_timeout("),
        ("time::", "sleep("),
        ("time::", "sleep_until("),
        ("task::", "sleep("),
        ("Timer::", "after("),
        ("Delay::", "new("),
        ("tokio::time::", "interval("),
    ]
    .iter()
    .map(|(a, b)| format!("{a}{b}"))
    .collect()
}

/// Every `.rs` file under `dir`.
fn rust_files(dir: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "rs") {
                found.push(path);
            }
        }
    }
    found.sort();
    found
}

/// `text` with every comment and every string and character literal blanked to spaces, lines kept:
/// what is left is the code, so a word inside a literal or a comment neither matches nor opens a
/// brace.
fn code_of(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let blank = |c: char| if c == '\n' { '\n' } else { ' ' };
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let next = chars.get(i + 1).copied();
        if c == '/' && next == Some('/') {
            while i < chars.len() && chars[i] != '\n' {
                out.push(' ');
                i += 1;
            }
            continue;
        }
        if c == '/' && next == Some('*') {
            let mut depth = 0;
            while i < chars.len() {
                if chars[i] == '/' && chars.get(i + 1) == Some(&'*') {
                    depth += 1;
                    out.push_str("  ");
                    i += 2;
                } else if chars[i] == '*' && chars.get(i + 1) == Some(&'/') {
                    depth -= 1;
                    out.push_str("  ");
                    i += 2;
                    if depth == 0 {
                        break;
                    }
                } else {
                    out.push(blank(chars[i]));
                    i += 1;
                }
            }
            continue;
        }
        // A raw string: r"…", r#"…"#, br#"…"#.
        let raw_at = if c == 'r' {
            Some(i + 1)
        } else if c == 'b' && next == Some('r') {
            Some(i + 2)
        } else {
            None
        };
        let word_before = i > 0 && (chars[i - 1].is_alphanumeric() || chars[i - 1] == '_');
        if let Some(mut j) = raw_at.filter(|_| !word_before) {
            let mut hashes = 0;
            while chars.get(j) == Some(&'#') {
                hashes += 1;
                j += 1;
            }
            if chars.get(j) == Some(&'"') {
                let close: String = std::iter::once('"')
                    .chain(std::iter::repeat_n('#', hashes))
                    .collect();
                let rest: String = chars[j + 1..].iter().collect();
                let len = rest.find(&close).map_or(rest.chars().count(), |at| {
                    rest.get(..at).unwrap_or("").chars().count()
                });
                let end = (j + 1 + len + close.chars().count()).min(chars.len());
                for &ch in &chars[i..end] {
                    out.push(blank(ch));
                }
                i = end;
                continue;
            }
        }
        if c == '"' {
            out.push(' ');
            i += 1;
            while i < chars.len() && chars[i] != '"' {
                if chars[i] == '\\' {
                    out.push(' ');
                    i += 1;
                }
                if i < chars.len() {
                    out.push(blank(chars[i]));
                    i += 1;
                }
            }
            out.push(' ');
            i += 1;
            continue;
        }
        // A character literal, not a lifetime: '\…' or 'x'.
        if c == '\'' && (next == Some('\\') || chars.get(i + 2) == Some(&'\'')) {
            let mut j = i + 1;
            if chars.get(j) == Some(&'\\') {
                j += 2;
            }
            while j < chars.len() && chars[j] != '\'' {
                j += 1;
            }
            for _ in i..=j.min(chars.len() - 1) {
                out.push(' ');
            }
            i = j + 1;
            continue;
        }
        out.push(c);
        i += 1;
    }
    out
}

/// The lines of `code` (comments and literals blanked) inside each `#[cfg(test)]` module, and the
/// files of each `#[cfg(test)] mod name;`, by name.
fn test_items(code: &str) -> (BTreeSet<usize>, Vec<String>) {
    let mut lines = BTreeSet::new();
    let mut files = Vec::new();
    let mut from = 0;
    let attribute = "#[cfg(test)]";
    while let Some(at) = code.get(from..).and_then(|rest| rest.find(attribute)) {
        let start = from + at;
        let after = code.get(start + attribute.len()..).unwrap_or("");
        // The item the attribute is on: past other attributes and visibility, a `mod`.
        let item = after.trim_start();
        let item = item.strip_prefix("pub ").unwrap_or(item).trim_start();
        if let Some(rest) = item.strip_prefix("mod ") {
            let name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            let rest = rest.get(name.len()..).unwrap_or("").trim_start();
            if rest.starts_with(';') {
                files.push(name);
            } else if rest.starts_with('{') {
                let open = code.len() - rest.len();
                let mut depth = 0usize;
                let mut end = code.len();
                for (k, c) in code.get(open..).unwrap_or("").char_indices() {
                    match c {
                        '{' => depth += 1,
                        '}' => {
                            depth -= 1;
                            if depth == 0 {
                                end = open + k;
                                break;
                            }
                        }
                        _ => {}
                    }
                }
                let first = code.get(..start).unwrap_or("").matches('\n').count();
                let last = code.get(..end).unwrap_or("").matches('\n').count();
                lines.extend(first..=last);
            }
        }
        from = start + attribute.len();
    }
    (lines, files)
}

/// Every test of every crate in crates/ (its tests/ and benches/ files, and each `#[cfg(test)]` module
/// of its src, in its own file or another), read outside comments and literals, holds no wall-clock
/// sleep. The reading is not vacuous: it read this crate's tests and its world, found the seam's
/// simulated sleeps to tell apart, and saw the production code that waits in real time and passed it
/// by (AC-RQ0.6.4).
#[test]
fn no_test_waits_on_the_wall_clock() {
    let crates = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let files = rust_files(crates);
    let patterns = sleeps();
    // Files a `#[cfg(test)] mod name;` makes test code whole.
    let mut test_files: BTreeSet<PathBuf> = BTreeSet::new();
    let mut tests_read = BTreeSet::new();
    let mut found = Vec::new();
    let mut passed_by = Vec::new();
    let mut simulated = 0;
    let mut codes = Vec::new();
    for file in &files {
        let code = code_of(&fs::read_to_string(file).unwrap());
        let (_, named) = test_items(&code);
        let dir = if file
            .file_name()
            .is_some_and(|n| n == "lib.rs" || n == "main.rs" || n == "mod.rs")
        {
            file.parent().unwrap().to_path_buf()
        } else {
            file.with_extension("")
        };
        for name in named {
            test_files.insert(dir.join(format!("{name}.rs")));
            test_files.insert(dir.join(name).join("mod.rs"));
        }
        codes.push((file.clone(), code));
    }
    for (file, code) in &codes {
        let in_tests_dir = file
            .components()
            .any(|c| c.as_os_str() == "tests" || c.as_os_str() == "benches");
        let whole = in_tests_dir || test_files.contains(file);
        let (lines, _) = test_items(code);
        for (n, line) in code.lines().enumerate() {
            let tested = whole || lines.contains(&n);
            if tested {
                tests_read.insert(file.clone());
                simulated += line.matches("sleep_until(").count();
            }
            for pattern in &patterns {
                if line.contains(pattern.as_str()) {
                    let at = format!("{}:{}", file.display(), n + 1);
                    if tested {
                        found.push(at);
                    } else {
                        passed_by.push(at);
                    }
                }
            }
        }
    }
    assert!(
        found.is_empty(),
        "wall-clock sleeps in tests:\n{}",
        found.join("\n")
    );
    // Paths compared by their parts, so a target's separator does not matter.
    let sim = |parts: &[&str]| -> PathBuf { parts.iter().collect() };
    let read = |parts: &[&str]| tests_read.iter().any(|f| f.ends_with(sim(parts)));
    assert!(read(&["reliquary-sim", "tests", "crash.rs"]));
    assert!(read(&["reliquary-sim", "src", "entry.rs"]));
    assert!(!read(&["reliquary-sim", "src", "world.rs"]));
    assert!(simulated > 0, "no simulated sleep found to tell apart");
    // What a reader of this test's output sees it passed by: production code, each by its place.
    println!("production code that waits in real time, passed by: {passed_by:?}");
}

/// The scan's own reading: words inside comments and literals are no code, a `#[cfg(test)]` module
/// is test code to its closing brace and no further, and a `#[cfg(test)] mod name;` names a file
/// (AC-RQ0.6.4).
#[test]
fn the_scan_reads_test_code_and_nothing_else() {
    let sleep = ["thread::", "sleep"].concat();
    let source = format!(
        "fn real() {{ let _ = \"{sleep}\"; }}\n// {sleep}\nfn more() {{ let c = '{{'; }}\n#[cfg(test)]\nmod tests {{\n    fn t() {{ let s = r#\"}}\"#; {sleep}(x); }}\n}}\nfn after() {{ {sleep}(y); }}\n#[cfg(test)]\nmod outside;\n"
    );
    let code = code_of(&source);
    assert!(
        !code.lines().next().unwrap().contains(&sleep),
        "a string read as code"
    );
    assert!(
        !code.lines().nth(1).unwrap().contains(&sleep),
        "a comment read as code"
    );
    let (lines, files) = test_items(&code);
    assert_eq!(lines, (3..=6).collect::<BTreeSet<usize>>());
    assert!(code.lines().nth(5).unwrap().contains(&sleep));
    assert!(code.lines().nth(7).unwrap().contains(&sleep) && !lines.contains(&7));
    assert_eq!(files, vec!["outside".to_owned()]);
}
