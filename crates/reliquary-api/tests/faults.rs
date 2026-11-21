//! The harness finds planted bugs (plan RQ0.5): registries that are the model with one fault each,
//! a lost tag update, a manifest accepted with a missing blob, a blob readable through a repository
//! it was never pushed to, and stale tag reads, are each reported, by the comparison of sequential
//! histories and by the linearizability check of concurrent ones, while the model run against
//! itself never is (T-RQ0.5.2, AC-RQ0.5.3). A failure shrinks to a 1-minimal history that replays
//! from the seed the hunt reports (AC-RQ0.5.4, T-RQ0.5.6).

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

use std::collections::{BTreeSet, VecDeque};

use reliquary_api::answer::{Answer, Body, ErrorCode, Status};
use reliquary_api::check::{self, Budget, Verdict};
use reliquary_api::generate::{self, Mix, Race, Shape};
use reliquary_api::grammar::{self, Algorithm, Digest, Name, Reference};
use reliquary_api::harness::{
    self, Coverage, Failure, Hunt, Interleaving, Registry, Tangle, Unexplained,
};
use reliquary_api::history::{Client, Plan, PlanError};
use reliquary_api::model::{Change, Config, Decision, Model, Stored};
use reliquary_api::request::{ManifestBody, Op, OpKind, Request};

const SEED: u64 = 0x5eed_fa17;
const LENGTH: usize = 120;

/// Few names, tags and contents, so tags move, blobs meet other repositories, and manifests meet
/// missing blobs.
fn hunting_mix() -> Mix {
    Mix {
        repositories: 2,
        principals: 1,
        tags: 3,
        contents: 4,
        largest_blob: 16,
        invalid_one_in: 4,
        manifest_bound: None,
        staging_budget: None,
    }
}

/// A lost tag update: a move of a tag that names another manifest is acknowledged and not made.
struct LostTagMove(Model);

impl Registry for LostTagMove {
    fn call(&mut self, _: Client, request: &Request) -> Answer {
        let Decision { answer, change } = self.0.decide(request);
        let change = match change {
            Change::Store {
                repository,
                digest,
                manifest,
                tag: Some(tag),
            } if self
                .0
                .repository(&repository)
                .and_then(|r| r.tag(&tag))
                .is_some_and(|named| *named != digest) =>
            {
                Change::Store {
                    repository,
                    digest,
                    manifest,
                    tag: None,
                }
            }
            other => other,
        };
        self.0.apply(change);
        answer
    }
}

/// A manifest accepted with a missing blob: what a manifest references is never looked for.
struct UncheckedReferences(Model);

impl Registry for UncheckedReferences {
    fn call(&mut self, _: Client, request: &Request) -> Answer {
        let decision = self.0.decide(request);
        if decision.answer.error == Some(ErrorCode::ManifestBlobUnknown)
            && let Op::PutManifest {
                name,
                reference,
                manifest,
            } = &request.op
            && let Some(repository) = Name::parse(name, None)
        {
            let sha256 = Digest::of(Algorithm::Sha256, &manifest.bytes);
            let (digest, tag) = match Reference::parse(reference) {
                Ok(Reference::Digest(digest)) => (digest, None),
                Ok(Reference::Tag(tag)) => (sha256, Some(tag)),
                Err(_) => (sha256, None),
            };
            let subject = match &manifest.body {
                ManifestBody::Image(image) => image.subject.as_ref().map(|s| s.digest.clone()),
                ManifestBody::Index(index) => index.subject.as_ref().map(|s| s.digest.clone()),
                _ => None,
            };
            self.0.apply(Change::Store {
                repository,
                digest: digest.clone(),
                manifest: Stored {
                    content_type: grammar::essence(&manifest.content_type).to_owned(),
                    bytes: manifest.bytes.clone(),
                    blobs: Vec::new(),
                    children: Vec::new(),
                    subject: subject.clone(),
                    artifact_type: None,
                    annotations: Vec::new(),
                },
                tag,
            });
            return Answer::success(Status::Created, Body::ManifestCreated { digest, subject });
        }
        self.0.apply(decision.change);
        decision.answer
    }
}

/// A blob readable through a repository it was never pushed to: a read finds the blob in any
/// repository that links it.
struct GlobalBlobs(Model);

impl Registry for GlobalBlobs {
    fn call(&mut self, _: Client, request: &Request) -> Answer {
        let decision = self.0.decide(request);
        let unknown = matches!(
            decision.answer.error,
            Some(ErrorCode::BlobUnknown | ErrorCode::NameUnknown)
        );
        let read = match &request.op {
            Op::GetBlob {
                digest,
                range: None,
                ..
            } => Some((digest, true)),
            Op::HeadBlob { digest, .. } => Some((digest, false)),
            _ => None,
        };
        if unknown
            && let Some((digest, get)) = read
            && let Ok(digest) = Digest::parse(digest)
            && let Some(content) = self
                .0
                .repositories()
                .find_map(|(_, repository)| repository.blob(&digest).cloned())
        {
            let size = u64::try_from(content.len()).unwrap();
            let content = if get { Some(content) } else { None };
            return Answer::success(
                Status::Ok,
                Body::Blob {
                    digest,
                    size,
                    content,
                },
            );
        }
        self.0.apply(decision.change);
        decision.answer
    }
}

/// Stale tag reads: every other read of a manifest by tag comes from a replica that has applied
/// all but the last `lag` changes.
struct StaleTags {
    primary: Model,
    replica: Model,
    behind: VecDeque<Change>,
    lag: usize,
    reads: u64,
}

impl StaleTags {
    fn new(config: &Config, lag: usize) -> Self {
        Self {
            primary: Model::new(config.clone()),
            replica: Model::new(config.clone()),
            behind: VecDeque::new(),
            lag,
            reads: 0,
        }
    }
}

impl Registry for StaleTags {
    fn call(&mut self, _: Client, request: &Request) -> Answer {
        let by_tag = matches!(
            &request.op,
            Op::GetManifest { reference, .. } | Op::HeadManifest { reference, .. }
                if !reference.contains(':')
        );
        if by_tag {
            self.reads += 1;
            if self.reads.is_multiple_of(2) {
                return self.replica.decide(request).answer;
            }
        }
        let Decision { answer, change } = self.primary.decide(request);
        self.primary.apply(change.clone());
        self.behind.push_back(change);
        while self.behind.len() > self.lag {
            let change = self.behind.pop_front().unwrap();
            self.replica.apply(change);
        }
        answer
    }
}

fn sequential_failure<R: Registry>(make: impl FnMut(&Config) -> R) -> Box<Failure> {
    let hunt = Hunt {
        seed: SEED,
        mix: hunting_mix(),
        histories: 200,
        length: LENGTH,
        shrink_tries: 5000,
    };
    harness::hunt(&hunt, make).expect_err("the planted fault was not reported")
}

fn tangle<R: Registry>(
    plans: impl FnMut(u64, u64) -> Result<(Config, Plan), PlanError>,
    make: impl FnMut(&Config) -> R,
) -> Box<Tangle> {
    let hunt = Interleaving {
        seed: SEED,
        histories: 150,
        budget: Budget(500_000),
        shrink_tries: 600,
    };
    match harness::hunt_concurrent(&hunt, plans, make) {
        Err(Unexplained::Tangle(tangle)) => tangle,
        other => panic!("the planted fault was not reported as a tangle: {other:?}"),
    }
}

fn shape() -> Shape {
    Shape {
        clients: 3,
        setup: 30,
        per_client: 10,
        lose_one_in: 0,
    }
}

fn race() -> Race {
    Race {
        writers: 2,
        readers: 1,
        manifests: 3,
        moves: 4,
        reads: 6,
        lose_one_in: 0,
    }
}

/// The model run against itself is never reported: sequential histories compare equal, and
/// concurrent ones, with lost answers too, are linearizable (T-RQ0.5.2: no false alarm, so a report
/// is a fault's).
#[test]
fn the_model_against_itself_is_never_reported() {
    let hunt = Hunt {
        seed: SEED,
        mix: hunting_mix(),
        histories: 60,
        length: LENGTH,
        shrink_tries: 0,
    };
    let coverage = harness::hunt(&hunt, |c| Model::new(c.clone()))
        .unwrap_or_else(|failure| panic!("{failure}"));
    assert_eq!(coverage.histories, 60);
    let concurrent = Interleaving {
        seed: SEED,
        histories: 40,
        budget: Budget(500_000),
        shrink_tries: 0,
    };
    let lossy = Shape {
        lose_one_in: 8,
        ..shape()
    };
    let coverage = harness::hunt_concurrent(
        &concurrent,
        |seed, _| generate::concurrent(seed, &hunting_mix(), lossy),
        |c| Model::new(c.clone()),
    )
    .unwrap_or_else(|unexplained| panic!("{unexplained:?}"));
    assert!(coverage.lost > 0, "no answer was lost");
    harness::hunt_concurrent(
        &concurrent,
        |seed, _| generate::tag_race(seed, &race()),
        |c| Model::new(c.clone()),
    )
    .unwrap_or_else(|unexplained| panic!("{unexplained:?}"));
}

/// Each planted fault is reported by comparing sequential histories with the model, at the request
/// that shows it: a lost tag update at a read of the tag, a manifest with a missing blob at its own
/// push, a blob read through a repository it was never pushed to at the read, a stale tag at a read
/// of the tag (T-RQ0.5.2, AC-RQ0.5.3).
#[test]
fn each_planted_fault_is_reported_by_the_comparison() {
    let reads_tag = |failure: &Failure| {
        matches!(
            &failure.divergence.request.op,
            Op::GetManifest { reference, .. } | Op::HeadManifest { reference, .. }
                if !reference.contains(':')
        )
    };
    let lost = sequential_failure(|c| LostTagMove(Model::new(c.clone())));
    assert!(reads_tag(&lost), "{lost}");

    let unchecked = sequential_failure(|c| UncheckedReferences(Model::new(c.clone())));
    assert_eq!(
        unchecked.divergence.expected.error,
        Some(ErrorCode::ManifestBlobUnknown),
        "{unchecked}"
    );
    assert_eq!(unchecked.divergence.observed.status, Status::Created);

    let global = sequential_failure(|c| GlobalBlobs(Model::new(c.clone())));
    assert!(
        matches!(
            global.divergence.request.op.kind(),
            OpKind::GetBlob | OpKind::HeadBlob
        ),
        "{global}"
    );
    assert_eq!(global.divergence.observed.status, Status::Ok);

    let stale = sequential_failure(|c| StaleTags::new(c, 1));
    assert!(reads_tag(&stale), "{stale}");
}

/// Each planted fault is reported by the linearizability check of concurrent histories, and the
/// stale read as a read of the tag that no order of the history explains (T-RQ0.5.2,
/// AC-RQ0.5.3).
#[test]
fn each_planted_fault_is_reported_by_the_linearizability_check() {
    let plans = |seed, _| generate::concurrent(seed, &hunting_mix(), shape());
    let failing_kind = |tangle: &Tangle| {
        tangle.history.operations[tangle.operation]
            .request
            .op
            .kind()
    };

    let lost = tangle(plans, |c| LostTagMove(Model::new(c.clone())));
    assert!(
        matches!(
            failing_kind(&lost),
            OpKind::GetManifest | OpKind::HeadManifest | OpKind::ListTags | OpKind::DeleteManifest
        ),
        "{lost}"
    );

    let unchecked = tangle(plans, |c| UncheckedReferences(Model::new(c.clone())));
    assert_eq!(failing_kind(&unchecked), OpKind::PutManifest, "{unchecked}");

    let global = tangle(plans, |c| GlobalBlobs(Model::new(c.clone())));
    assert!(
        matches!(failing_kind(&global), OpKind::GetBlob | OpKind::HeadBlob),
        "{global}"
    );

    let stale = tangle(
        |seed, _| generate::tag_race(seed, &race()),
        |c| StaleTags::new(c, 1),
    );
    assert!(
        matches!(
            failing_kind(&stale),
            OpKind::GetManifest | OpKind::HeadManifest
        ),
        "{stale}"
    );
}

/// A failing sequential history shrinks to a 1-minimal one that fails the same way, the seed the
/// hunt reports replays the same divergence at the same request, and hunting again reports the
/// same failure (AC-RQ0.5.4, T-RQ0.5.6).
#[test]
fn a_failure_shrinks_to_a_minimal_history_and_replays_from_its_seed() {
    let make = |c: &Config| LostTagMove(Model::new(c.clone()));
    let failure = sequential_failure(make);
    assert!(failure.minimal, "{failure}");
    assert!(
        failure
            .to_string()
            .contains(&format!("{:#018x}", failure.seed))
    );

    let replayed = harness::replay(failure.seed, &hunting_mix(), LENGTH, make)
        .expect_err("the replay did not fail");
    assert_eq!(replayed, failure.divergence);
    assert_eq!(sequential_failure(make), failure);

    let (config, _) = generate::sequential(failure.seed, &hunting_mix(), LENGTH);
    let signature = failure.divergence.signature();
    let fails = |requests: &[Request]| {
        harness::compare(
            &config,
            &mut make(&config),
            requests,
            &mut Coverage::default(),
        )
        .err()
        .is_some_and(|d| d.signature() == signature)
    };
    assert!(fails(&failure.shrunk));
    assert_eq!(
        failure.shrunk_divergence.as_ref().map(|d| d.signature()),
        Some(signature)
    );
    for i in 0..failure.shrunk.len() {
        let mut without = failure.shrunk.clone();
        without.remove(i);
        assert!(!fails(&without), "request {i} is not needed:\n{failure}");
    }
    assert!(failure.shrunk.len() <= failure.divergence.index + 1);
}

/// A concurrent history no order explains shrinks to a 1-minimal plan that fails the same way, and
/// its seed and index replay the same history and the same verdict at the same operation
/// (AC-RQ0.5.4, T-RQ0.5.6).
#[test]
fn a_tangle_shrinks_to_a_minimal_plan_and_replays_from_its_seed() {
    let plans = |seed, _| generate::tag_race(seed, &race());
    let make = |c: &Config| StaleTags::new(c, 1);
    let found = tangle(plans, make);
    assert!(found.minimal, "{found}");

    let (history, verdict) =
        harness::replay_concurrent(found.seed, found.index, Budget(500_000), plans, make).unwrap();
    assert_eq!(history, found.history);
    assert_eq!(
        verdict,
        Verdict::NotLinearizable {
            operation: found.operation
        }
    );

    let kind = found.history.operations[found.operation].request.op.kind();
    let (config, _) = generate::tag_race(found.seed, &race()).unwrap();
    let initial = Model::new(config.clone());
    let fails = |plan: &Plan| {
        let history = plan.run(&mut make(&config)).unwrap();
        match check::check(&initial, &history, Budget(500_000)) {
            Verdict::NotLinearizable { operation } => {
                history.operations[operation].request.op.kind() == kind
            }
            _ => false,
        }
    };
    assert!(fails(&found.shrunk), "{found}");
    let n = found.shrunk.operations().len();
    for i in 0..n {
        let keep: BTreeSet<usize> = (0..n).filter(|k| *k != i).collect();
        assert!(
            !fails(&found.shrunk.keeping(&keep)),
            "operation {i} is not needed:\n{found}"
        );
    }
    assert!(n < found.history.operations.len(), "{found}");
}
