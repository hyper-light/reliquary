//! `docs/design/seams.md` states each seam's contract, and this test holds it to the code (plan
//! RQ0.4). For every seam, the doc's clauses are its conformance cases (the same ids, in the same
//! order, in the same words: AC-RQ0.4.1); its "Methods:" are its traits' methods, read from their
//! sources; its "Faults:" are the kinds its simulator injects, and every refusal its trait names
//! is one of them (AC-RQ0.4.3); and no method takes a timeout as a number, while every method that
//! waits takes a deadline (AC-RQ0.4.5). A section for a seam that isn't one, or a seam without a
//! section, fails it too.

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
use std::path::{Path, PathBuf};

use reliquary_core::seams::conformance::Case;
use reliquary_core::seams::fault::{CUT, DELAY, LOSE, VANISH};
use reliquary_core::seams::{bytes, clock, entropy, records, secrets, topics, upstream};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// One seam: its section's title, the traits the doc lists the methods of and the files they are
/// in, the file its refusal is in, the kinds its simulator injects, its cases, and its methods
/// that never wait.
struct Seam {
    title: &'static str,
    traits: Vec<(PathBuf, &'static str)>,
    refusal: PathBuf,
    faults: &'static [&'static str],
    cases: Vec<(&'static str, &'static str)>,
    instant: &'static [&'static str],
    /// Its methods take no deadline of their own: its callers hold each call to theirs with
    /// `clock::within` (Quiver's `Topics`).
    held_by_caller: bool,
}

fn said<F>(cases: &[Case<F>]) -> Vec<(&'static str, &'static str)> {
    cases.iter().map(|c| (c.id, c.says)).collect()
}

fn seams() -> Vec<Seam> {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/seams");
    let amphora = root().join("vendor/amphora/crates/api/src/lib.rs");
    let quiver = root().join("vendor/quiver/crates/quiver-api/src/lib.rs");
    vec![
        Seam {
            title: "Topics",
            traits: vec![(quiver.clone(), "Topics"), (quiver.clone(), "Subscription")],
            refusal: quiver,
            faults: topics::FAULTS,
            cases: said(&topics::cases::<topics::SimTopicsFixture>()),
            instant: &[],
            held_by_caller: true,
        },
        Seam {
            title: "Clock",
            traits: vec![(src.join("clock.rs"), "Clock")],
            refusal: src.join("clock.rs"),
            faults: clock::FAULTS,
            cases: said(&clock::cases::<clock::SimClockFixture>()),
            instant: &["now", "wall"],
            held_by_caller: false,
        },
        Seam {
            title: "Entropy",
            traits: vec![(src.join("entropy.rs"), "Entropy")],
            refusal: src.join("entropy.rs"),
            faults: entropy::FAULTS,
            cases: said(&entropy::cases::<entropy::SimEntropyFixture>()),
            instant: &[],
            held_by_caller: false,
        },
        Seam {
            title: "Bytes",
            traits: vec![
                (src.join("bytes.rs"), "Bytes"),
                (src.join("bytes.rs"), "Body"),
            ],
            refusal: src.join("bytes.rs"),
            faults: bytes::FAULTS,
            cases: said(&bytes::suite::cases::<bytes::sim::SimBytesFixture>()),
            instant: &[],
            held_by_caller: false,
        },
        Seam {
            title: "Records",
            traits: vec![(amphora.clone(), "Records")],
            refusal: amphora,
            faults: records::FAULTS,
            cases: said(&records::cases::<records::SimRecordsFixture>()),
            instant: &[],
            held_by_caller: false,
        },
        Seam {
            title: "Secrets",
            traits: vec![(src.join("secrets.rs"), "Secrets")],
            refusal: src.join("secrets.rs"),
            faults: secrets::FAULTS,
            cases: said(&secrets::cases::<secrets::SimSecretsFixture>()),
            instant: &[],
            held_by_caller: false,
        },
        Seam {
            title: "Upstream",
            traits: vec![
                (src.join("upstream.rs"), "Upstream"),
                (src.join("upstream.rs"), "Body"),
            ],
            refusal: src.join("upstream.rs"),
            faults: upstream::FAULTS,
            cases: said(&upstream::cases::<upstream::SimUpstreamFixture>()),
            instant: &[],
            held_by_caller: false,
        },
    ]
}

/// `text` from byte `from` to byte `to`.
fn cut(text: &str, from: usize, to: usize) -> &str {
    text.get(from..to).expect("a cut at a character's boundary")
}

/// The doc's section titled `title`: from its heading to the next of its level.
fn section<'a>(doc: &'a str, title: &str) -> Option<&'a str> {
    let heading = format!("\n## {title}\n");
    let start = doc.find(&heading)? + heading.len();
    let rest = cut(doc, start, doc.len());
    Some(rest.find("\n## ").map_or(rest, |end| cut(rest, 0, end)))
}

/// The paragraph of `section` that starts with `label`, joined into one line.
fn paragraph(section: &str, label: &str) -> Option<String> {
    let start = section.find(&format!("\n{label}"))? + 1;
    let rest = cut(section, start, section.len());
    let end = rest.find("\n\n").unwrap_or(rest.len());
    Some(
        cut(rest, 0, end)
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" "),
    )
}

/// The items between backticks in `text`, in order.
fn ticked(text: &str) -> Vec<String> {
    text.split('`')
        .skip(1)
        .step_by(2)
        .map(str::to_owned)
        .collect()
}

/// Whether `id` is a clause's id: a seam's name in capitals, a dash, and a number (`BYTES-3`).
fn clause_id(id: &str) -> bool {
    id.split_once('-').is_some_and(|(seam, n)| {
        !seam.is_empty()
            && seam.chars().all(|c| c.is_ascii_uppercase())
            && !n.is_empty()
            && n.chars().all(|c| c.is_ascii_digit())
    })
}

/// The clauses of `section`: each `- **ID** words` line.
fn clauses(section: &str) -> Vec<(String, String)> {
    section
        .lines()
        .filter_map(|line| line.strip_prefix("- **"))
        .filter_map(|line| line.split_once("** "))
        .filter(|(id, _)| clause_id(id))
        .map(|(id, says)| (id.to_owned(), says.to_owned()))
        .collect()
}

/// `source` without its comments: what the compiler reads of it, near enough for its signatures.
fn code(source: &str) -> String {
    source
        .lines()
        .map(|line| line.split_once("//").map_or(line, |(code, _)| code))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Where the bracket that opens first at or after byte `from` of `text` closes: the bytes inside
/// it, as (just past the open, at the close).
fn bracketed(text: &str, from: usize, open: char, close: char) -> (usize, usize) {
    let at = from
        + cut(text, from, text.len())
            .find(open)
            .expect("an opening bracket");
    let mut depth = 0;
    for (i, c) in cut(text, at, text.len()).char_indices() {
        if c == open {
            depth += 1;
        } else if c == close {
            depth -= 1;
            if depth == 0 {
                return (at + 1, at + i);
            }
        }
    }
    panic!("an unclosed bracket")
}

/// The methods of trait `name` in `source`: each one's name and its parameters.
fn methods(source: &str, name: &str) -> Vec<(String, Vec<(String, String)>)> {
    let code = code(source);
    let at = code
        .find(&format!("pub trait {name}"))
        .unwrap_or_else(|| panic!("no trait {name}"));
    let (start, end) = bracketed(&code, at, '{', '}');
    let block = cut(&code, start, end);
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(i) = cut(block, from, block.len()).find("fn ") {
        let name_at = from + i + "fn ".len();
        let method: String = cut(block, name_at, block.len())
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        let (params_from, params_to) = bracketed(block, name_at, '(', ')');
        let params = cut(block, params_from, params_to)
            .split(',')
            .map(str::trim)
            .filter(|p| !p.is_empty() && !p.ends_with("self"))
            .filter_map(|p| p.split_once(':'))
            .map(|(n, t)| (n.trim().to_owned(), t.trim().to_owned()))
            .collect();
        out.push((method, params));
        // Past the signature, and past the default body when the method has one.
        let tail = cut(block, params_to, block.len());
        from = match (tail.find(';'), tail.find('{')) {
            (Some(semi), Some(body)) if body > semi => params_to + semi + 1,
            (Some(semi), None) => params_to + semi + 1,
            (_, Some(_)) => bracketed(block, params_to, '{', '}').1 + 1,
            (None, None) => block.len(),
        };
    }
    out
}

/// The variants of `enum Refusal` in `source`.
fn refusals(source: &str) -> Vec<String> {
    let code = code(source);
    let at = code.find("pub enum Refusal").expect("an enum Refusal");
    let (start, end) = bracketed(&code, at, '{', '}');
    let mut out = Vec::new();
    let mut depth: i64 = 0;
    for line in cut(&code, start, end).lines() {
        let trimmed = line.trim();
        if depth == 0 && !trimmed.starts_with('#') && !trimmed.is_empty() {
            let name: String = trimmed
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() {
                out.push(name);
            }
        }
        depth += i64::try_from(line.matches(['{', '(']).count()).unwrap();
        depth -= i64::try_from(line.matches(['}', ')']).count()).unwrap();
    }
    out
}

fn doc() -> String {
    read(&root().join("docs/design/seams.md"))
}

/// AC-RQ0.4.1: each seam's clauses in docs/design/seams.md are its suite's cases, the same ids
/// in the same order in the same words.
#[test]
fn ac_rq0_4_1_each_seams_clauses_are_its_suites_cases() {
    let doc = doc();
    for seam in seams() {
        let section = section(&doc, seam.title)
            .unwrap_or_else(|| panic!("seams.md has no section for {}", seam.title));
        let documented = clauses(section);
        let coded: Vec<(String, String)> = seam
            .cases
            .iter()
            .map(|(id, says)| ((*id).to_owned(), (*says).to_owned()))
            .collect();
        for (doc_clause, case) in documented.iter().zip(&coded) {
            assert_eq!(
                doc_clause, case,
                "{}: the doc's clause and the suite's case",
                seam.title
            );
        }
        assert_eq!(
            documented.len(),
            coded.len(),
            "{}: the doc has {} clauses, the suite {} cases",
            seam.title,
            documented.len(),
            coded.len()
        );
    }
}

/// AC-RQ0.4.1: each seam's "Methods:" in docs/design/seams.md are its traits' methods, as their
/// sources declare them.
#[test]
fn ac_rq0_4_1_each_seams_methods_are_its_traits() {
    let doc = doc();
    for seam in seams() {
        let section = section(&doc, seam.title).unwrap();
        let line = paragraph(section, "Methods:")
            .unwrap_or_else(|| panic!("{}: no Methods: line", seam.title));
        let documented: BTreeSet<String> = ticked(&line).into_iter().collect();
        let declared: BTreeSet<String> = seam
            .traits
            .iter()
            .flat_map(|(file, name)| methods(&read(file), name))
            .map(|(method, _)| method)
            .collect();
        assert_eq!(
            documented, declared,
            "{}: the doc's methods and the traits'",
            seam.title
        );
    }
}

/// AC-RQ0.4.3: each seam's "Faults:" in docs/design/seams.md are the kinds its simulator injects,
/// and every refusal its trait names is one of them, so each can be injected.
#[test]
fn ac_rq0_4_3_each_seams_faults_are_every_refusal_and_the_kinds_its_simulator_injects() {
    let doc = doc();
    let phrases = [DELAY, VANISH, LOSE, CUT];
    for seam in seams() {
        let section = section(&doc, seam.title).unwrap();
        let line = paragraph(section, "Faults:")
            .unwrap_or_else(|| panic!("{}: no Faults: line", seam.title));
        let documented: Vec<String> = ticked(&line);
        let injected: Vec<String> = seam.faults.iter().map(|k| (*k).to_owned()).collect();
        assert_eq!(
            documented, injected,
            "{}: the doc's faults and the simulator's",
            seam.title
        );
        let named: BTreeSet<String> = refusals(&read(&seam.refusal)).into_iter().collect();
        let refusals_injected: BTreeSet<String> = seam
            .faults
            .iter()
            .filter(|k| !phrases.contains(k))
            .map(|k| (*k).to_owned())
            .collect();
        assert_eq!(
            named, refusals_injected,
            "{}: the refusals its trait names and those its simulator injects",
            seam.title
        );
    }
}

/// AC-RQ0.4.5: no method of any seam takes a timeout as a number, and every method that waits
/// takes a deadline (an `Instant`) its caller derives; those that never wait are named.
#[test]
fn ac_rq0_4_5_no_method_takes_a_timeout_and_each_that_waits_takes_a_deadline() {
    for seam in seams() {
        for (file, name) in &seam.traits {
            for (method, params) in methods(&read(file), name) {
                for (param, ty) in &params {
                    let timed = ty.contains("Duration")
                        || ["timeout", "_ms", "_ns", "_us", "_secs", "seconds"]
                            .iter()
                            .any(|word| param.contains(word));
                    assert!(
                        !timed,
                        "{name}::{method} takes {param}: {ty}, a timeout as a number"
                    );
                }
                let deadline = params.iter().any(|(_, ty)| ty == "Instant");
                if seam.held_by_caller {
                    assert!(
                        !deadline,
                        "{name}::{method} takes a deadline, yet is named as held by its caller"
                    );
                } else if seam.instant.contains(&method.as_str()) {
                    assert!(
                        !deadline,
                        "{name}::{method} is named as never waiting, yet takes a deadline"
                    );
                } else {
                    assert!(deadline, "{name}::{method} waits and takes no deadline");
                }
            }
        }
    }
}

/// AC-RQ0.4.1: docs/design/seams.md has a section for each seam and none for a seam that isn't
/// one: every section with a "Methods:" line is a seam's.
#[test]
fn ac_rq0_4_1_the_doc_has_a_section_for_each_seam_and_no_other() {
    let doc = doc();
    let titles: BTreeSet<&str> = seams().iter().map(|s| s.title).collect();
    let documented: BTreeSet<&str> = doc
        .split("\n## ")
        .skip(1)
        .filter(|section| section.contains("\nMethods:"))
        .filter_map(|section| section.lines().next())
        .collect();
    assert_eq!(documented, titles);
}

/// AC-RQ0.4.1: the checks above read what they claim: a planted clause, method or fault the code
/// lacks is found.
#[test]
fn ac_rq0_4_1_a_planted_difference_is_found() {
    let doc = doc();
    let bytes = section(&doc, "Bytes").unwrap();
    let planted = format!("{bytes}\n- **BYTES-99** a clause no case checks\n");
    assert_ne!(
        clauses(&planted).len(),
        bytes::suite::cases::<bytes::sim::SimBytesFixture>().len()
    );
    let methods_line = paragraph(bytes, "Methods:").unwrap();
    assert!(ticked(&methods_line).contains(&"begin".to_owned()));
    let source = read(&Path::new(env!("CARGO_MANIFEST_DIR")).join("src/seams/bytes.rs"));
    let declared = methods(&source, "Bytes");
    assert!(
        declared
            .iter()
            .any(|(m, p)| m == "begin" && p.iter().any(|(_, t)| t == "Instant"))
    );
    assert!(refusals(&source).contains(&"Exists".to_owned()));
}
