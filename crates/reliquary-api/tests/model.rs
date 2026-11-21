//! The reference model on its own (plan RQ0.5): it answers every endpoint of distribution spec
//! v1.1.1 with the spec's statuses and codes as one function of its state (AC-RQ0.5.1); generated
//! histories reach every request kind, every error code and every answer it lists while its
//! invariants hold (T-RQ0.5.1, T-RQ0.5.5); histories of none, one and only refused requests pass
//! (T-RQ0.5.3); its grammars agree with the specs' expressions (AC-RQ0.5.1); and the manifests the
//! generator writes are the structures it says they are (AC-RQ0.5.2).

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
use std::path::Path;

use hyper_sim::rng::Seeded;
use regex::Regex;
use reliquary_api::answer::{ANSWERS, ErrorCode, Status};
use reliquary_api::check::{self, Budget, Verdict};
use reliquary_api::generate::{self, Generator, Mix};
use reliquary_api::grammar::{Digest, DigestError, Name, Tag};
use reliquary_api::harness::{self, Coverage, Gap, Hunt};
use reliquary_api::history::{Client, Plan};
use reliquary_api::model::Model;
use reliquary_api::request::{Endpoint, Malformation, ManifestBody, Op, OpKind, Request};

const SEED: u64 = 0x5eed_0005;

/// A mix that reaches every refusal: few names and tags, so requests meet each other; a manifest
/// bound and a staging budget small enough to be passed.
fn coverage_mix() -> Mix {
    Mix {
        repositories: 3,
        principals: 2,
        tags: 6,
        contents: 6,
        largest_blob: 32,
        invalid_one_in: 3,
        manifest_bound: Some(2048),
        staging_budget: Some(64),
    }
}

fn status_code(status: Status) -> u16 {
    match status {
        Status::Ok => 200,
        Status::Created => 201,
        Status::Accepted => 202,
        Status::NoContent => 204,
        Status::PartialContent => 206,
        Status::BadRequest => 400,
        Status::Unauthorized => 401,
        Status::Forbidden => 403,
        Status::NotFound => 404,
        Status::MethodNotAllowed => 405,
        Status::ContentTooLarge => 413,
        Status::RangeNotSatisfiable => 416,
        Status::TooManyRequests => 429,
    }
}

/// Generated histories against the model alone, run until every request kind, every error code
/// and every answer the model lists has occurred, with the model's invariants checked after every
/// request: a tag names a manifest that exists, a manifest's blobs exist (T-RQ0.5.1, AC-RQ0.5.1,
/// AC-T9.7.4).
#[test]
fn every_kind_and_code_occurs_while_the_invariants_hold() {
    let mix = coverage_mix();
    let mut coverage = Coverage::default();
    let mut steps = 0u64;
    let mut index = 0;
    while index < 600 && !coverage.gaps(&[]).is_empty() {
        let seed = harness::history_seed(SEED, index);
        let (config, requests) = generate::sequential(seed, &mix, 150);
        let mut model = Model::new(config);
        for (i, request) in requests.iter().enumerate() {
            let answer = model.step(request);
            coverage.count(request, &answer);
            if let Err(broken) = model.check() {
                panic!("history {seed:#x}, request {i} ({request}) broke an invariant: {broken:?}");
            }
            steps += 1;
        }
        index += 1;
    }
    let gaps = coverage.gaps(&[]);
    assert!(
        gaps.is_empty(),
        "after {index} histories, {steps} requests: never produced {gaps:?}"
    );
    assert_eq!(coverage.kinds.len(), OpKind::ALL.len());
    assert_eq!(coverage.codes.len(), ErrorCode::ALL.len());
}

/// Each answer is one function of the state: deciding twice on one state gives one decision and
/// changes nothing, and two models fed the same requests stay equal (AC-RQ0.5.1).
#[test]
fn each_answer_is_a_function_of_the_state() {
    let mix = coverage_mix();
    for index in 0..20 {
        let (config, requests) =
            generate::sequential(harness::history_seed(SEED ^ 1, index), &mix, 120);
        let mut one = Model::new(config.clone());
        let mut other = Model::new(config);
        for request in &requests {
            let before = one.clone();
            let first = one.decide(request);
            let second = one.decide(request);
            assert_eq!(first, second, "{request}");
            assert_eq!(one, before, "deciding {request} changed the state");
            assert_eq!(one.step(request), other.step(request), "{request}");
            assert_eq!(one, other);
        }
    }
}

/// The spec's endpoint table (distribution spec v1.1.1, "Endpoints"): each ID, its success
/// statuses and its failure statuses.
fn spec_endpoints() -> BTreeMap<&'static str, (Vec<u16>, Vec<u16>)> {
    BTreeMap::from([
        ("end-1", (vec![200], vec![404, 401])),
        ("end-2", (vec![200], vec![404])),
        ("end-3", (vec![200], vec![404])),
        ("end-4a", (vec![202], vec![404])),
        ("end-4b", (vec![201, 202], vec![404, 400])),
        ("end-5", (vec![202], vec![404, 416])),
        ("end-6", (vec![201], vec![404, 400])),
        ("end-7", (vec![201], vec![404, 413])),
        ("end-8a", (vec![200], vec![404])),
        ("end-8b", (vec![200], vec![404])),
        ("end-9", (vec![202], vec![404, 400, 405])),
        ("end-10", (vec![202], vec![404, 400, 405])),
        ("end-11", (vec![201, 202], vec![404])),
        ("end-12a", (vec![200], vec![404, 400])),
        ("end-12b", (vec![200], vec![404, 400])),
        ("end-13", (vec![204], vec![404])),
    ])
}

/// The model answers every endpoint of distribution spec v1.1.1 (and the upload cancel of its
/// `main`): each has a success in the table of answers, each status the model gives is one the
/// spec's table lists for that endpoint or one model.md gives a reason for, and the spec's
/// statuses the model never gives are exactly those model.md says it never gives (AC-RQ0.5.1).
#[test]
fn the_answers_cover_every_endpoint_of_the_spec() {
    let spec = spec_endpoints();
    let ids: BTreeSet<&str> = Endpoint::ALL.iter().map(|e| e.id()).collect();
    for id in spec.keys() {
        assert!(ids.contains(id), "{id} has no endpoint in the model");
    }
    // Statuses outside the spec's table, each with its reason in model.md (H2, H7, H9): a
    // malformed request, authentication, authorization and the staging budget anywhere; ranges on
    // a blob read; a closing PUT's last chunk out of order.
    let anywhere: BTreeSet<u16> = BTreeSet::from([400, 401, 403, 429]);
    let mut never: BTreeSet<(&str, u16)> = BTreeSet::new();
    for (id, (success, failure)) in &spec {
        let given: BTreeSet<u16> = ANSWERS
            .iter()
            .filter(|(e, _, _)| e.id() == *id)
            .map(|(_, s, _)| status_code(*s))
            .collect();
        for status in success.iter().chain(failure) {
            if !given.contains(status) {
                never.insert((id, *status));
            }
        }
        for status in &given {
            let listed = success.contains(status) || failure.contains(status);
            let ranged = *id == "end-2" && matches!(status, 206 | 416);
            // The closing PUT's last chunk out of order: v1.1.1 says a chunk out of order is 416,
            // and the spec's main adds 416 to end-6 for the final chunk (model.md, H2).
            let last_chunk = *id == "end-6" && *status == 416;
            assert!(
                listed || anywhere.contains(status) || ranged || last_chunk,
                "{id} answers {status}, which neither the spec's table nor model.md gives"
            );
        }
        assert!(
            success.iter().any(|s| given.contains(s)),
            "{id} never succeeds"
        );
    }
    // model.md, "What the spec lists that Reliquary never answers".
    let documented: BTreeSet<(&str, u16)> = BTreeSet::from([
        ("end-1", 404),
        ("end-4a", 404),
        ("end-4b", 202),
        ("end-4b", 404),
        ("end-7", 404),
        ("end-11", 404),
        ("end-12a", 404),
        ("end-12b", 404),
    ]);
    assert_eq!(never, documented);
}

/// The table of answers in docs/design/model.md is the code's ANSWERS, row for row (AC-RQ0.5.1:
/// a doc-truth test).
#[test]
fn the_design_record_lists_the_answers_the_model_gives() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/design/model.md");
    let text = std::fs::read_to_string(&path).expect("docs/design/model.md");
    let start = text
        .find("<!-- answers:begin -->")
        .expect("model.md marks its table of answers");
    let end = text
        .find("<!-- answers:end -->")
        .expect("model.md ends its table of answers");
    let mut documented = BTreeSet::new();
    for line in text.get(start..end).unwrap().lines() {
        let cells: Vec<&str> = line.split('|').map(str::trim).collect();
        // | end-N | statuses | codes |
        if cells.len() < 4 || !cells[1].starts_with("end-") {
            continue;
        }
        let endpoint = Endpoint::ALL
            .iter()
            .find(|e| e.id() == cells[1])
            .unwrap_or_else(|| panic!("model.md names an endpoint the code lacks: {}", cells[1]));
        for answer in cells[2].split(',').map(str::trim) {
            let (status, code) = match answer.split_once(' ') {
                Some((status, code)) => (status, Some(code)),
                None => (answer, None),
            };
            let status: u16 = status.parse().expect("a status");
            let status = [
                Status::Ok,
                Status::Created,
                Status::Accepted,
                Status::NoContent,
                Status::PartialContent,
                Status::BadRequest,
                Status::Unauthorized,
                Status::Forbidden,
                Status::NotFound,
                Status::MethodNotAllowed,
                Status::ContentTooLarge,
                Status::RangeNotSatisfiable,
                Status::TooManyRequests,
            ]
            .into_iter()
            .find(|s| status_code(*s) == status)
            .unwrap_or_else(|| panic!("model.md names a status the code lacks: {status}"));
            let code = code.map(|code| {
                *ErrorCode::ALL
                    .iter()
                    .find(|c| c.as_str() == code)
                    .unwrap_or_else(|| panic!("model.md names a code the spec lacks: {code}"))
            });
            documented.insert((*endpoint, status, code));
        }
    }
    let code: BTreeSet<_> = ANSWERS.iter().copied().collect();
    assert_eq!(code.len(), ANSWERS.len(), "ANSWERS lists a row twice");
    let missing: Vec<_> = code.difference(&documented).collect();
    let extra: Vec<_> = documented.difference(&code).collect();
    assert!(
        missing.is_empty() && extra.is_empty(),
        "model.md lacks {missing:?} and lists {extra:?} that the code does not give"
    );
}

/// The model's code holds no I/O, clock, thread, lock, interior mutability or cache: what it
/// answers can depend on nothing but its state and the request (AC-RQ0.5.1).
#[test]
fn the_model_has_no_io_no_concurrency_and_no_cache() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let forbidden = [
        "std::fs",
        "std::net",
        "std::io",
        "std::thread",
        "std::time",
        "std::env",
        "Mutex",
        "RwLock",
        "Atomic",
        "Cell",
        "OnceLock",
        "LazyLock",
        "thread_local",
        "static mut",
        "Instant",
        "SystemTime",
    ];
    for file in ["model.rs", "grammar.rs", "request.rs", "answer.rs"] {
        let text = std::fs::read_to_string(src.join(file)).expect("the model's source");
        for word in forbidden {
            assert!(!text.contains(word), "{file} uses {word}");
        }
    }
}

/// Histories with no requests, with one request, and with only refused requests pass, compared
/// in order and checked for linearizability (T-RQ0.5.3).
#[test]
fn histories_of_none_one_and_only_refused_requests_pass() {
    let mix = coverage_mix();
    let budget = Budget(100_000);
    let (config, requests) = generate::sequential(SEED, &mix, 400);
    let mut coverage = Coverage::default();

    // None.
    let mut model = Model::new(config.clone());
    assert!(harness::compare(&config, &mut model, &[], &mut coverage).is_ok());
    let empty = Plan::sequential(Client(0), Vec::new()).run(&mut Model::new(config.clone()));
    assert_eq!(
        check::check(&Model::new(config.clone()), &empty.unwrap(), budget),
        Verdict::Linearizable { order: Vec::new() }
    );

    // One, of every kind the history holds.
    let mut kinds = BTreeSet::new();
    for request in &requests {
        if !kinds.insert(request.op.kind()) {
            continue;
        }
        let one = std::slice::from_ref(request);
        let mut model = Model::new(config.clone());
        assert!(harness::compare(&config, &mut model, one, &mut coverage).is_ok());
        let history = Plan::sequential(Client(0), one.to_vec())
            .run(&mut Model::new(config.clone()))
            .unwrap();
        assert!(matches!(
            check::check(&Model::new(config.clone()), &history, budget),
            Verdict::Linearizable { .. }
        ));
    }
    assert_eq!(kinds.len(), OpKind::ALL.len(), "the history lacks a kind");

    // Only refusals: every request broken, and of those the model refuses, all in one history.
    let broken = Mix {
        invalid_one_in: 1,
        ..mix
    };
    let (config, requests) = generate::sequential(SEED ^ 2, &broken, 400);
    let mut model = Model::new(config.clone());
    let refused: Vec<Request> = requests
        .into_iter()
        .filter(|request| model.step(request).error.is_some())
        .collect();
    assert!(refused.len() > 100, "only {} refusals", refused.len());
    let mut only = Coverage::default();
    let mut model = Model::new(config.clone());
    assert!(harness::compare(&config, &mut model, &refused, &mut only).is_ok());
    let answered: u64 = only.answers.values().sum();
    let refusals: u64 = only
        .answers
        .iter()
        .filter(|((_, _, code), _)| code.is_some())
        .map(|(_, n)| n)
        .sum();
    assert_eq!(
        answered, refusals,
        "a request of the refused history succeeded"
    );
    let history = Plan::sequential(Client(0), refused)
        .run(&mut Model::new(config.clone()))
        .unwrap();
    assert!(matches!(
        check::check(&Model::new(config), &history, budget),
        Verdict::Linearizable { .. }
    ));
}

/// After a run the counters hold every request kind, every error code and every answer above
/// zero; a run that never breaks a request leaves codes at zero, and the check fails it, naming
/// them (T-RQ0.5.5, AC-RQ0.5.2).
#[test]
fn the_counters_cover_every_kind_and_code_and_fail_a_run_that_misses_one() {
    let hunt = Hunt {
        seed: SEED ^ 3,
        mix: coverage_mix(),
        histories: 400,
        length: 150,
        shrink_tries: 0,
    };
    let coverage = harness::hunt(&hunt, |config| Model::new(config.clone())).expect("no failure");
    assert_eq!(coverage.gaps(&[]), Vec::new());
    assert!(coverage.kinds.values().all(|n| *n > 0));
    assert!(coverage.codes.values().all(|n| *n > 0));

    let valid_only = Hunt {
        mix: Mix {
            invalid_one_in: 0,
            ..coverage_mix()
        },
        histories: 20,
        ..hunt
    };
    let coverage =
        harness::hunt(&valid_only, |config| Model::new(config.clone())).expect("no failure");
    let gaps = coverage.gaps(&[]);
    assert!(
        gaps.contains(&Gap::Code(ErrorCode::NameInvalid)),
        "{gaps:?}"
    );
    assert!(
        gaps.contains(&Gap::Code(ErrorCode::Unauthorized)),
        "{gaps:?}"
    );
}

/// Draws strings from an alphabet weighted toward a grammar's own characters.
fn strings(seed: u64, alphabet: &[u8], longest: u64, count: usize) -> Vec<String> {
    let mut rng = Seeded::new(seed);
    (0..count)
        .map(|_| {
            let length = rng.below(longest + 1);
            (0..length)
                .map(|_| {
                    let bound = u64::try_from(alphabet.len()).unwrap();
                    char::from(alphabet[usize::try_from(rng.below(bound)).unwrap()])
                })
                .collect()
        })
        .collect()
}

/// Names, tags and digests are accepted exactly when the specs' expressions match them, run by a
/// regex engine as the reference: the name expression and the tag expression of distribution spec
/// v1.1.1, and the digest grammar with its registered algorithms of image spec v1.1.1 (AC-RQ0.5.1,
/// the model's half of AC-RQ1.1.5).
#[test]
fn the_grammars_agree_with_the_specs_expressions() {
    let name =
        Regex::new(r"^[a-z0-9]+((\.|_|__|-+)[a-z0-9]+)*(/[a-z0-9]+((\.|_|__|-+)[a-z0-9]+)*)*$")
            .unwrap();
    let tag = Regex::new(r"^[a-zA-Z0-9_][a-zA-Z0-9._-]{0,127}$").unwrap();
    let grammar = Regex::new(r"^[a-z0-9]+([+._-][a-z0-9]+)*:[a-zA-Z0-9=_-]+$").unwrap();
    let registered = Regex::new(r"^(sha256:[a-f0-9]{64}|sha512:[a-f0-9]{128})$").unwrap();

    let mut checked = 0;
    for text in strings(1, b"ab09._-/_-A:", 24, 20_000) {
        assert_eq!(
            Name::parse(&text, None).is_some(),
            name.is_match(&text),
            "{text:?}"
        );
        checked += 1;
    }
    for text in strings(2, b"aZ09._-/: ", 140, 20_000) {
        assert_eq!(Tag::parse(&text).is_some(), tag.is_match(&text), "{text:?}");
        checked += 1;
    }
    let real = Digest::of(reliquary_api::grammar::Algorithm::Sha256, b"x").to_string();
    let mut digests = strings(3, b"sha2560af+._-:=Z ", 80, 20_000);
    digests.extend([
        real.clone(),
        real.to_ascii_uppercase(),
        format!("{real}0"),
        real.get(..real.len() - 1).unwrap().to_owned(),
        real.replace("sha256", "sha512"),
        Digest::of(reliquary_api::grammar::Algorithm::Sha512, b"x").to_string(),
        real.replace("sha256", "blake3"),
        "multihash+base58:QmRZxt2b1FVZPNqd8hsiykDL3TdBDeTSPX9Kv46HmX4Gx8".to_owned(),
    ]);
    let mut verdicts = BTreeMap::new();
    for text in digests {
        let expected = if !grammar.is_match(&text) {
            Err(DigestError::Malformed)
        } else if text.starts_with("sha256:") || text.starts_with("sha512:") {
            if registered.is_match(&text) {
                Ok(())
            } else {
                Err(DigestError::Malformed)
            }
        } else {
            Err(DigestError::Unregistered)
        };
        let parsed = Digest::parse(&text).map(|_| ());
        assert_eq!(parsed, expected, "{text:?}");
        *verdicts.entry(format!("{expected:?}")).or_insert(0) += 1;
        checked += 1;
    }
    // Each verdict was reached: the generated strings were not all of one kind.
    assert_eq!(verdicts.len(), 3, "{verdicts:?}");
    assert!(checked > 60_000);
    // The spec's own examples.
    assert!(Name::parse("library/ubuntu", None).is_some());
    assert!(Name::parse("library/ubuntu", Some(5)).is_none());
    assert!(Tag::parse(&"a".repeat(128)).is_some());
    assert!(Tag::parse(&"a".repeat(129)).is_none());
}

/// Every manifest the generator writes is what its structure says: an image manifest's or an
/// index's bytes are JSON with the fields the structure holds, bytes said not to be JSON are not,
/// and bytes said to be no manifest are JSON without a manifest's fields (AC-RQ0.5.2).
#[test]
fn the_generator_writes_the_manifests_it_describes() {
    let mut generator = Generator::new(SEED ^ 4, &coverage_mix());
    let mut seen = BTreeMap::new();
    for _ in 0..6000 {
        let request = generator.next_request();
        let Op::PutManifest { manifest, .. } = &request.op else {
            continue;
        };
        let parsed = serde_json::from_slice::<serde_json::Value>(&manifest.bytes);
        let kind = match &manifest.body {
            ManifestBody::Image(image) => {
                let json = parsed.expect("an image manifest is JSON");
                assert_eq!(json["schemaVersion"], 2);
                assert_eq!(json["config"]["digest"], image.config.digest.as_str());
                assert_eq!(json["config"]["size"], image.config.size);
                assert_eq!(json["layers"].as_array().unwrap().len(), image.layers.len());
                for (layer, written) in image.layers.iter().zip(json["layers"].as_array().unwrap())
                {
                    assert_eq!(written["digest"], layer.digest.as_str());
                    assert_eq!(written["size"], layer.size);
                    assert_eq!(written["mediaType"], layer.media_type.as_str());
                }
                assert_eq!(json["mediaType"].as_str(), image.media_type.as_deref());
                assert_eq!(
                    json["artifactType"].as_str(),
                    image.artifact_type.as_deref()
                );
                assert_eq!(
                    json["subject"]["digest"].as_str(),
                    image.subject.as_ref().map(|s| s.digest.as_str())
                );
                for (key, value) in &image.annotations {
                    assert_eq!(json["annotations"][key], value.as_str());
                }
                "image"
            }
            ManifestBody::Index(index) => {
                let json = parsed.expect("an index is JSON");
                assert_eq!(json["schemaVersion"], 2);
                let written = json["manifests"].as_array().unwrap();
                assert_eq!(written.len(), index.manifests.len());
                for (child, written) in index.manifests.iter().zip(written) {
                    assert_eq!(written["digest"], child.digest.as_str());
                    assert_eq!(written["size"], child.size);
                }
                assert_eq!(
                    json["artifactType"].as_str(),
                    index.artifact_type.as_deref()
                );
                "index"
            }
            ManifestBody::Malformed(Malformation::NotJson) => {
                assert!(parsed.is_err(), "bytes said not to be JSON parse");
                "not json"
            }
            ManifestBody::Malformed(Malformation::NotAManifest) => {
                let json = parsed.expect("bytes said to be no manifest are JSON");
                assert!(json.get("config").is_none() && json.get("manifests").is_none());
                "not a manifest"
            }
            ManifestBody::Opaque => "opaque",
        };
        *seen.entry(kind).or_insert(0) += 1;
    }
    assert_eq!(seen.len(), 5, "{seen:?}");
}
