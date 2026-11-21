//! The seams (plan RQ0.4): each simulator passes its trait's conformance suite (AC-RQ0.4.2) and
//! injects every fault its trait can meet (AC-RQ0.4.3, T-RQ0.4.9), and the behaviors the plan's
//! tests name.

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

use std::sync::Arc;
use std::time::{Duration, SystemTime};

use reliquary_core::seams::bytes::sim::{SimBytes, SimBytesFixture, SimLimits};
use reliquary_core::seams::bytes::{self, Plan};
use reliquary_core::seams::clock::{self, SimClock, SimClockFixture};
use reliquary_core::seams::conformance::{self, Report};
use reliquary_core::seams::entropy::{self, SimEntropy, SimEntropyFixture, Stream};
use reliquary_core::seams::records::{self, SimRecords, SimRecordsFixture};
use reliquary_core::seams::secrets::{self, SimSecrets, SimSecretsFixture};
use reliquary_core::seams::topics::{self, FaultyTopics, SimTopicsFixture};
use reliquary_core::seams::upstream::{self, SimUpstream, SimUpstreamFixture};

/// 2026-01-01T00:00:00Z: where every simulated wall clock here starts.
fn epoch() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_767_225_600)
}

/// A simulated store's grant key: 32 bytes of the stream of `seed`.
fn grant_key(seed: u64) -> [u8; 32] {
    let mut key = [0u8; 32];
    Stream::new(seed).fill(&mut key).unwrap();
    key
}

/// A second: what a simulated caller waits, and a simulated clock's quantum.
const PATIENCE: Duration = Duration::from_secs(1);
/// Steps a simulated case has: far more than any case takes.
const STEPS: u64 = 1 << 20;

fn bytes_fixture() -> Result<SimBytesFixture, String> {
    let clock = SimClock::new(epoch(), 64);
    let limits = SimLimits {
        // Pieces and runs that do not divide one another, so cuts fall at every place in both.
        plan: Plan { piece: 3, run: 4 },
        objects: 64,
        uploads: 8,
        bytes: 1 << 20,
        key: 64,
    };
    let bytes = SimBytes::in_memory(Arc::new(clock.clone()), &grant_key(7), limits)
        .map_err(|e| e.to_string())?;
    Ok(SimBytesFixture {
        bytes,
        clock,
        patience: PATIENCE,
        length: 17,
        steps: STEPS,
    })
}

fn checked(report: Report, faults: &[&'static str]) {
    let missing = report.injected.missing(faults);
    let seam = report.seam;
    let injected: Vec<_> = report.injected.iter().collect();
    report.into_result().unwrap();
    assert!(
        missing.is_empty(),
        "{seam}: the suite injected none of {missing:?} (injected: {injected:?})"
    );
}

/// AC-RQ0.4.2, AC-RQ0.4.3, T-RQ0.4.9: the clock simulator passes the clock suite, injecting every
/// fault a clock can meet.
#[test]
fn the_clock_simulator_passes_its_suite() {
    let report = conformance::run("Clock", &clock::cases(), || {
        Ok(SimClockFixture {
            clock: SimClock::new(epoch(), 8),
            quantum: PATIENCE,
            sleeps: 8,
            steps: STEPS,
        })
    });
    checked(report, clock::FAULTS);
}

/// AC-RQ0.4.2, AC-RQ0.4.3, T-RQ0.4.9: the entropy simulator passes the entropy suite, injecting
/// every fault a source can meet.
#[test]
fn the_entropy_simulator_passes_its_suite() {
    let report = conformance::run("Entropy", &entropy::cases(), || {
        let clock = SimClock::new(epoch(), 8);
        Ok(SimEntropyFixture {
            entropy: SimEntropy::new(11, Arc::new(clock.clone())),
            clock,
            patience: PATIENCE,
            draws: 1000,
            steps: STEPS,
        })
    });
    checked(report, entropy::FAULTS);
}

/// AC-RQ0.4.2, AC-RQ0.4.3, T-RQ0.4.9: the bytes simulator passes the bytes suite, injecting every
/// fault its trait can meet.
#[test]
fn the_bytes_simulator_passes_its_suite() {
    let report = conformance::run("Bytes", &bytes::suite::cases(), bytes_fixture);
    checked(report, bytes::FAULTS);
}

fn records_fixture() -> Result<SimRecordsFixture, String> {
    let clock = SimClock::new(epoch(), 8);
    let limits = records::SimLimits {
        rows: 256,
        changes: 256,
    };
    Ok(SimRecordsFixture {
        records: SimRecords::new(clock.clone(), limits),
        clock,
        patience: PATIENCE,
        rows: 11,
        page: 4,
    })
}

/// AC-RQ0.4.2, AC-RQ0.4.3, T-RQ0.4.9: the records simulator passes the records suite, injecting
/// every fault Amphora's `Records` can meet.
#[test]
fn the_records_simulator_passes_its_suite() {
    let report = conformance::run("Records", &records::cases(), records_fixture);
    checked(report, records::FAULTS);
}

fn secrets_fixture() -> Result<SimSecretsFixture, String> {
    let clock = SimClock::new(epoch(), 8);
    let limits = secrets::SimLimits {
        leases: 4,
        life: Duration::from_secs(60),
    };
    let secrets = SimSecrets::new(Arc::new(clock.clone()), limits);
    secrets.keep("registry.example/robot", b"s3cr3t-token-value", true);
    secrets.keep("registry.example/admin", b"never-leased", false);
    Ok(SimSecretsFixture {
        secrets,
        clock,
        patience: PATIENCE,
        steps: STEPS,
        name: "registry.example/robot".into(),
        value: b"s3cr3t-token-value".to_vec(),
        denied: "registry.example/admin".into(),
    })
}

/// AC-RQ0.4.2, AC-RQ0.4.3, T-RQ0.4.9: the secrets simulator passes the secrets suite, injecting
/// every fault its trait can meet.
#[test]
fn the_secrets_simulator_passes_its_suite() {
    let report = conformance::run("Secrets", &secrets::cases(), secrets_fixture);
    checked(report, secrets::FAULTS);
}

fn upstream_fixture() -> Result<SimUpstreamFixture, String> {
    let clock = SimClock::new(epoch(), 8);
    Ok(SimUpstreamFixture {
        upstream: SimUpstream::new(Arc::new(clock.clone())),
        clock,
        patience: PATIENCE,
        steps: STEPS,
        length: 13,
        buffer: 4,
    })
}

/// AC-RQ0.4.2, AC-RQ0.4.3, T-RQ0.4.9: the upstream simulator passes the upstream suite, injecting
/// every fault another registry can show.
#[test]
fn the_upstream_simulator_passes_its_suite() {
    let report = conformance::run("Upstream", &upstream::cases(), upstream_fixture);
    checked(report, upstream::FAULTS);
}

fn topics_fixture() -> Result<SimTopicsFixture, String> {
    let clock = SimClock::new(epoch(), 8);
    Ok(SimTopicsFixture {
        topics: FaultyTopics::new(quiver_api::sim::SimTopics::new(), Arc::new(clock.clone())),
        clock,
        patience: PATIENCE,
        steps: STEPS,
        topic: quiver_api::Topic {
            class: quiver_api::Class::Ephemeral,
            retain_messages: 5,
            retain_bytes: 1 << 20,
        },
        messages: 9,
    })
}

/// AC-RQ0.4.2, AC-RQ0.4.3, T-RQ0.4.9: the topics simulator (Quiver's, with faults injected)
/// passes the topics suite, injecting every fault Quiver's `Topics` can meet.
#[test]
fn the_topics_simulator_passes_its_suite() {
    let report = conformance::run("Topics", &topics::cases(), topics_fixture);
    checked(report, topics::FAULTS);
}

/// AC-RQ0.4.2: with no fault armed, the fault-injecting wrapper is Quiver's simulator unchanged:
/// it passes Quiver's own conformance suite, every clause.
#[test]
fn ac_rq0_4_2_faulty_topics_with_nothing_armed_passes_quivers_own_suite() {
    use quiver_api::conformance::{self as quiver, TopicsFixture};
    struct Wrapped(FaultyTopics<quiver_api::sim::SimTopics>);
    impl TopicsFixture for Wrapped {
        fn topics(&self) -> &dyn quiver_api::Topics {
            &self.0
        }
        fn serves(&self, _: quiver_api::Class) -> bool {
            true
        }
    }
    let clock = SimClock::new(epoch(), 8);
    let made = clock.clone();
    let report =
        futures::executor::block_on(quiver::run("Topics", &quiver::cases::<Wrapped>(), || {
            let clock = made.clone();
            async move {
                Wrapped(FaultyTopics::new(
                    quiver_api::sim::SimTopics::new(),
                    Arc::new(clock),
                ))
            }
        }));
    let n = report.into_result().unwrap();
    assert_eq!(n, quiver::cases::<Wrapped>().len());
    assert_eq!(
        clock.elapsed(),
        Duration::ZERO,
        "nothing was held: no fault was armed"
    );
}

// ------------------------------------------------------------------------- the plan's tests

use reliquary_core::seams::bytes::{
    Access, Bytes as _, Deleted, Method, Op as BytesOp, Progress, Put, Redeemed, Refusal, Sha256,
    Whole,
};
use reliquary_core::seams::clock::Clock as _;
use reliquary_core::seams::fault::Fault;

/// A simulated store of objects, its clock, and a deadline no call here reaches unless a fault
/// holds it.
fn store(plan: Plan) -> (SimBytes, SimClock) {
    let clock = SimClock::new(epoch(), 64);
    let limits = SimLimits {
        plan,
        objects: 64,
        uploads: 8,
        bytes: 1 << 20,
        key: 64,
    };
    let bytes = SimBytes::in_memory(Arc::new(clock.clone()), &grant_key(3), limits).unwrap();
    (bytes, clock)
}

fn due(clock: &SimClock) -> std::time::Instant {
    clock.now() + PATIENCE
}

/// A byte's place in memory.
fn ix(at: u64) -> usize {
    usize::try_from(at).unwrap()
}

fn whole(data: &[u8]) -> Whole {
    Whole {
        length: data.len() as u64,
        sha256: Sha256::of(data),
    }
}

/// Puts `data` at `key`: begun, sent a piece of `plan` at a time, finished.
fn put(
    bytes: &SimBytes,
    clock: &SimClock,
    key: &str,
    data: &[u8],
    put: Put,
) -> Result<reliquary_core::seams::bytes::Meta, Refusal> {
    clock
        .run(
            async {
                let upload = bytes.begin(key, put, due(clock)).await?;
                let mut at = 0;
                while at < data.len() as u64 {
                    let end = data.len().min(ix(at) + upload.plan.piece);
                    let piece = &data[ix(at)..end];
                    at = bytes
                        .send(&upload.id, at, piece, due(clock))
                        .await?
                        .received;
                }
                bytes.finish(&upload.id, whole(data), due(clock)).await
            },
            STEPS,
        )
        .unwrap()
}

/// Every byte of the object at `key`.
fn get(bytes: &SimBytes, clock: &SimClock, key: &str) -> Result<Vec<u8>, Refusal> {
    clock
        .run(
            async {
                let mut reading = bytes.get(key, None, due(clock)).await?;
                let mut out = Vec::new();
                let mut into = [0u8; 5];
                loop {
                    let n = reading.body.read(&mut into, due(clock)).await?;
                    if n == 0 {
                        return Ok(out);
                    }
                    out.extend_from_slice(&into[..n]);
                }
            },
            STEPS,
        )
        .unwrap()
}

/// T-RQ0.4.2: put an object, then put another to its key create-once: refused as existing, both
/// when the key holds the object at the begin and when it came to hold it before the finish, and
/// the first object's bytes and facts are unchanged.
#[test]
fn t_rq0_4_2_create_once_on_a_key_that_exists_is_refused_and_the_first_object_is_unchanged() {
    let (bytes, clock) = store(Plan { piece: 4, run: 4 });
    let first = b"the first object".as_slice();
    let second = b"another object, longer".as_slice();
    let once = |identity| Put {
        create_once: true,
        identity,
    };
    let meta = put(&bytes, &clock, "k", first, once(1)).unwrap();
    assert_eq!(
        put(&bytes, &clock, "k", second, once(2)),
        Err(Refusal::Exists("k".into()))
    );
    // Begun while the key was empty; the first object lands before its finish.
    let late = clock
        .run(bytes.begin("k2", once(3), due(&clock)), STEPS)
        .unwrap()
        .unwrap();
    let meta2 = put(&bytes, &clock, "k2", first, once(4)).unwrap();
    let finished = clock
        .run(
            async {
                bytes.send(&late.id, 0, &second[..4], due(&clock)).await?;
                bytes
                    .finish(&late.id, whole(&second[..4]), due(&clock))
                    .await
            },
            STEPS,
        )
        .unwrap();
    assert_eq!(finished, Err(Refusal::Exists("k2".into())));
    for (key, meta) in [("k", &meta), ("k2", &meta2)] {
        assert_eq!(get(&bytes, &clock, key).unwrap(), first);
        let head = clock.run(bytes.head(key, due(&clock)), STEPS).unwrap();
        assert_eq!(head.as_ref(), Ok(meta));
    }
}

/// T-RQ0.4.3: for several plans, cut a put at every byte, resume it from the durable offset the
/// store names, and finish: the object is whole, the durable offset is the last run's end at or
/// before the cut, and no byte before it was sent twice; only the bytes of the run the cut fell in
/// were sent again.
#[test]
fn t_rq0_4_3_a_put_resumes_from_each_durable_offset_after_a_cut() {
    let data: Vec<u8> = (0..23u8).collect();
    let length = data.len() as u64;
    for (piece, run) in [(1, 1), (3, 4), (4, 4), (5, 2), (7, 3), (23, 23)] {
        for cut in 0..=length {
            let (bytes, clock) = store(Plan { piece, run });
            let mut sent = vec![0u32; data.len()];
            bytes.arm(BytesOp::Send, Fault::Cut(cut)).unwrap();
            let key = "resumed";
            let ended = clock
                .run(
                    async {
                        let put = Put {
                            create_once: true,
                            identity: 9,
                        };
                        let upload = bytes.begin(key, put, due(&clock)).await?;
                        let mut at = 0;
                        // Sends until the cut, then from where the store says it stands.
                        let mut durable = None;
                        while at < length {
                            let end = data.len().min(ix(at) + piece);
                            for n in &mut sent[ix(at)..end] {
                                *n += 1;
                            }
                            match bytes
                                .send(&upload.id, at, &data[ix(at)..end], due(&clock))
                                .await
                            {
                                Ok(progress) => at = progress.received,
                                Err(Refusal::Cut { at: offset }) => {
                                    let status = bytes.status(&upload.id, due(&clock)).await?;
                                    assert_eq!(
                                        status,
                                        Progress {
                                            received: offset,
                                            durable: offset
                                        }
                                    );
                                    durable = Some(offset);
                                    at = offset;
                                }
                                Err(other) => return Err(other),
                            }
                        }
                        bytes.finish(&upload.id, whole(&data), due(&clock)).await?;
                        Ok(durable)
                    },
                    STEPS,
                )
                .unwrap()
                .unwrap();
            let durable = ended.unwrap_or_else(|| panic!("plan {piece}/{run}: no cut at {cut}"));
            assert_eq!(durable, cut / run * run, "plan {piece}/{run}, cut at {cut}");
            let below = ix(durable);
            assert!(
                sent[..below].iter().all(|n| *n == 1),
                "plan {piece}/{run}, cut at {cut}: a byte before the durable offset {durable} was sent twice"
            );
            assert!(sent.iter().all(|n| *n == 1 || *n == 2));
            let twice = sent.iter().filter(|n| **n == 2).count() as u64;
            assert!(
                twice < run + piece as u64,
                "plan {piece}/{run}, cut at {cut}: {twice} bytes sent twice"
            );
            assert_eq!(
                get(&bytes, &clock, key).unwrap(),
                data,
                "plan {piece}/{run}, cut at {cut}"
            );
        }
    }
}

/// T-RQ0.4.5: two threads each put an object create-once to one key and finish at once, many
/// times: each time exactly one wins, the other is told the key exists, and the object is the
/// winner's.
#[test]
fn t_rq0_4_5_of_two_create_once_puts_at_once_exactly_one_wins() {
    for round in 0..200u64 {
        let (bytes, clock) = store(Plan { piece: 8, run: 8 });
        let start = std::sync::Barrier::new(2);
        let ends: Vec<_> = std::thread::scope(|s| {
            let racers: Vec<_> = [b"aaaaaaaaaaaa".as_slice(), b"bbbbbbbbbbbbbbbb".as_slice()]
                .into_iter()
                .enumerate()
                .map(|(i, data)| {
                    let (bytes, clock, start) = (bytes.clone(), clock.clone(), &start);
                    s.spawn(move || {
                        let put = Put {
                            create_once: true,
                            identity: u128::from(round * 2 + i as u64),
                        };
                        let upload = clock
                            .run(bytes.begin("raced", put, due(&clock)), STEPS)
                            .unwrap()
                            .unwrap();
                        clock
                            .run(bytes.send(&upload.id, 0, &data[..8], due(&clock)), STEPS)
                            .unwrap()
                            .unwrap();
                        clock
                            .run(bytes.send(&upload.id, 8, &data[8..], due(&clock)), STEPS)
                            .unwrap()
                            .unwrap();
                        start.wait();
                        let end = clock
                            .run(bytes.finish(&upload.id, whole(data), due(&clock)), STEPS)
                            .unwrap();
                        (end, data)
                    })
                })
                .collect();
            racers.into_iter().map(|r| r.join().unwrap()).collect()
        });
        let won: Vec<_> = ends.iter().filter(|(end, _)| end.is_ok()).collect();
        assert_eq!(won.len(), 1, "round {round}: {ends:?}");
        let lost: Vec<_> = ends.iter().filter(|(end, _)| end.is_err()).collect();
        assert_eq!(
            lost[0].0,
            Err(Refusal::Exists("raced".into())),
            "round {round}"
        );
        assert_eq!(
            get(&bytes, &clock, "raced").unwrap(),
            won[0].1,
            "round {round}"
        );
    }
}

/// T-RQ0.4.6: a commit whose answer is lost after it applied, on each seam with one: the caller
/// reconciles by reading, which shows it applied, and asking again never applies it twice.
#[test]
fn t_rq0_4_6_an_answer_lost_after_a_commit_is_reconciled_by_reading_and_never_applied_twice() {
    // Bytes: a finish.
    let (bytes, clock) = store(Plan { piece: 8, run: 8 });
    let data = b"lost answer".as_slice();
    let upload = clock
        .run(
            async {
                let put = Put {
                    create_once: true,
                    identity: 5,
                };
                let upload = bytes.begin("k", put, due(&clock)).await?;
                bytes.send(&upload.id, 0, &data[..8], due(&clock)).await?;
                bytes.send(&upload.id, 8, &data[8..], due(&clock)).await?;
                Ok::<_, Refusal>(upload)
            },
            STEPS,
        )
        .unwrap()
        .unwrap();
    bytes.arm(BytesOp::Finish, Fault::Lose).unwrap();
    let lost = clock.run(bytes.finish(&upload.id, whole(data), due(&clock)), STEPS);
    assert_eq!(lost.unwrap(), Err(Refusal::Lost));
    let head = clock
        .run(bytes.head("k", due(&clock)), STEPS)
        .unwrap()
        .unwrap();
    assert_eq!(
        head.sha256,
        Sha256::of(data),
        "reading shows the finish applied"
    );
    let again = clock.run(bytes.finish(&upload.id, whole(data), due(&clock)), STEPS);
    assert_eq!(again.unwrap(), Err(Refusal::NoUpload(upload.id.0.clone())));
    assert_eq!(get(&bytes, &clock, "k").unwrap(), data);

    // Records: a commit.
    use amphora_api::{Cond, Consistency, Records as _, Refusal as RecordsRefusal, Tx, Write};
    let clock = SimClock::new(epoch(), 8);
    let records = SimRecords::new(
        clock.clone(),
        records::SimLimits {
            rows: 8,
            changes: 8,
        },
    );
    let tx = Tx {
        conds: vec![Cond::Absent {
            family: "tags".into(),
            key: b"latest".to_vec(),
        }],
        writes: vec![Write::Put {
            family: "tags".into(),
            key: b"latest".to_vec(),
            value: b"sha256:a".to_vec(),
        }],
    };
    records.arm(records::Op::Commit, Fault::Lose).unwrap();
    assert_eq!(
        records.commit(tx.clone(), due(&clock)),
        Err(RecordsRefusal::UnknownOutcome)
    );
    let row = records
        .get("tags", b"latest", Consistency::Linearizable, due(&clock))
        .unwrap()
        .expect("reading shows the commit applied");
    assert_eq!(
        records.commit(tx, due(&clock)),
        Err(RecordsRefusal::Conflict {
            family: "tags".into(),
            key: b"latest".to_vec(),
            at: Some(row.rev),
        })
    );
    assert_eq!(
        records.head(due(&clock)),
        Ok(row.rev),
        "nothing applied twice"
    );
}

/// T-RQ0.4.7: the store refuses Full and Busy, injected and at its own bounds: the caller gets the
/// typed refusal, and the store saw one attempt for each call: nothing was asked again.
#[test]
fn t_rq0_4_7_full_and_busy_are_passed_to_the_caller_typed_and_never_asked_again() {
    let (bytes, clock) = store(Plan { piece: 8, run: 8 });
    let busy = Refusal::Busy {
        retry_after: Some(PATIENCE),
    };
    let full = Refusal::Full {
        what: "bytes",
        limit: 0,
    };
    for (op, refusal) in [(BytesOp::Head, busy), (BytesOp::Head, full)] {
        let before = bytes.attempts(op);
        bytes.arm(op, Fault::Refuse(refusal.clone())).unwrap();
        assert_eq!(
            clock.run(bytes.head("k", due(&clock)), STEPS).unwrap(),
            Err(refusal)
        );
        assert_eq!(bytes.attempts(op), before + 1);
    }
    // At the store's own bound: 8 uploads open, a ninth refused Full, after one attempt.
    for i in 0..8u128 {
        let put = Put {
            create_once: false,
            identity: i,
        };
        clock
            .run(bytes.begin(&format!("k{i}"), put, due(&clock)), STEPS)
            .unwrap()
            .unwrap();
    }
    let before = bytes.attempts(BytesOp::Begin);
    let ninth = Put {
        create_once: false,
        identity: 8,
    };
    let refused = clock
        .run(bytes.begin("k8", ninth, due(&clock)), STEPS)
        .unwrap();
    assert!(
        matches!(
            refused,
            Err(Refusal::Full {
                what: "uploads",
                limit: 8
            })
        ),
        "{refused:?}"
    );
    assert_eq!(bytes.attempts(BytesOp::Begin), before + 1);
    assert_eq!(bytes.uploads_open(), 8, "no open upload was dropped");
}

/// T-RQ0.4.8: a grant changed in its key, its method, its size, its expiry or its holder is
/// refused by the simulator's Mantle side, and the grant as issued is honored.
#[test]
fn t_rq0_4_8_a_grant_changed_in_key_method_size_or_expiry_is_refused() {
    let (bytes, clock) = store(Plan { piece: 8, run: 8 });
    let data = b"a granted object".as_slice();
    put(
        &bytes,
        &clock,
        "k",
        data,
        Put {
            create_once: true,
            identity: 1,
        },
    )
    .unwrap();
    let expires = clock.wall() + Duration::from_secs(60);
    let access = Access {
        method: Method::Get,
        length: data.len() as u64,
    };
    let grant = clock
        .run(
            bytes.grant("k", access, "puller", expires, due(&clock)),
            STEPS,
        )
        .unwrap()
        .unwrap();
    let mut changed = Vec::new();
    let mut g = grant.clone();
    g.key = "k-other".into();
    changed.push(("key", g));
    let mut g = grant.clone();
    g.access.method = Method::Put;
    changed.push(("method", g));
    let mut g = grant.clone();
    g.access.length += 1;
    changed.push(("size", g));
    // A microsecond: the smallest change every platform's clock holds (Windows' counts in 100 ns,
    // where one nanosecond more is the same time), checked to be a change.
    let mut g = grant.clone();
    g.expires += Duration::from_micros(1);
    assert_ne!(
        g.expires, grant.expires,
        "the changed expiry is the same time"
    );
    changed.push(("expiry", g));
    let mut g = grant.clone();
    g.holder = "mallory".into();
    changed.push(("holder", g));
    for (what, hostile) in &changed {
        let redeemed = clock
            .run(bytes.redeem(hostile, &hostile.holder, due(&clock)), STEPS)
            .unwrap();
        assert!(
            matches!(redeemed, Err(Refusal::Forbidden(_))),
            "a grant with its {what} changed: {redeemed:?}"
        );
    }
    let honored = clock
        .run(bytes.redeem(&grant, "puller", due(&clock)), STEPS)
        .unwrap();
    assert!(matches!(honored, Ok(Redeemed::Reading(_))), "{honored:?}");
    clock.advance(Duration::from_secs(60)).unwrap();
    let late = clock
        .run(bytes.redeem(&grant, "puller", due(&clock)), STEPS)
        .unwrap();
    assert!(matches!(late, Err(Refusal::Expired)), "{late:?}");
}

/// T-RQ0.4.9, AC-RQ0.4.3: run every seam's suite against its simulator and count the faults each
/// injected, by kind: every kind its trait can meet is above zero.
#[test]
fn t_rq0_4_9_every_suite_injects_every_fault_kind_of_its_seam() {
    let runs = [
        (
            conformance::run("Clock", &clock::cases(), || {
                Ok(SimClockFixture {
                    clock: SimClock::new(epoch(), 8),
                    quantum: PATIENCE,
                    sleeps: 8,
                    steps: STEPS,
                })
            }),
            clock::FAULTS,
        ),
        (
            conformance::run("Entropy", &entropy::cases(), || {
                let clock = SimClock::new(epoch(), 8);
                Ok(SimEntropyFixture {
                    entropy: SimEntropy::new(11, Arc::new(clock.clone())),
                    clock,
                    patience: PATIENCE,
                    draws: 1000,
                    steps: STEPS,
                })
            }),
            entropy::FAULTS,
        ),
        (
            conformance::run("Bytes", &bytes::suite::cases(), bytes_fixture),
            bytes::FAULTS,
        ),
        (
            conformance::run("Records", &records::cases(), records_fixture),
            records::FAULTS,
        ),
        (
            conformance::run("Secrets", &secrets::cases(), secrets_fixture),
            secrets::FAULTS,
        ),
        (
            conformance::run("Upstream", &upstream::cases(), upstream_fixture),
            upstream::FAULTS,
        ),
        (
            conformance::run("Topics", &topics::cases(), topics_fixture),
            topics::FAULTS,
        ),
    ];
    for (report, faults) in runs {
        for kind in faults {
            let n = report.injected.get(kind);
            eprintln!("{}: {kind}: {n}", report.seam);
            assert!(
                n > 0,
                "{}: no fault of kind {kind} was injected",
                report.seam
            );
        }
        report.into_result().unwrap();
    }
}

/// A bytes store whose create-once is broken, to show the suite finds it: it forgets the condition.
struct Careless(SimBytes);

impl reliquary_core::seams::bytes::Bytes for Careless {
    fn begin<'a>(
        &'a self,
        key: &'a str,
        put: Put,
        deadline: std::time::Instant,
    ) -> reliquary_core::seams::BoxFuture<'a, Result<reliquary_core::seams::bytes::Upload, Refusal>>
    {
        self.0.begin(
            key,
            Put {
                create_once: false,
                ..put
            },
            deadline,
        )
    }
    fn send<'a>(
        &'a self,
        upload: &'a reliquary_core::seams::bytes::UploadId,
        at: u64,
        piece: &'a [u8],
        deadline: std::time::Instant,
    ) -> reliquary_core::seams::BoxFuture<'a, Result<Progress, Refusal>> {
        self.0.send(upload, at, piece, deadline)
    }
    fn status<'a>(
        &'a self,
        upload: &'a reliquary_core::seams::bytes::UploadId,
        deadline: std::time::Instant,
    ) -> reliquary_core::seams::BoxFuture<'a, Result<Progress, Refusal>> {
        self.0.status(upload, deadline)
    }
    fn finish<'a>(
        &'a self,
        upload: &'a reliquary_core::seams::bytes::UploadId,
        whole: Whole,
        deadline: std::time::Instant,
    ) -> reliquary_core::seams::BoxFuture<'a, Result<reliquary_core::seams::bytes::Meta, Refusal>>
    {
        self.0.finish(upload, whole, deadline)
    }
    fn abort<'a>(
        &'a self,
        upload: &'a reliquary_core::seams::bytes::UploadId,
        deadline: std::time::Instant,
    ) -> reliquary_core::seams::BoxFuture<'a, Result<(), Refusal>> {
        self.0.abort(upload, deadline)
    }
    fn get<'a>(
        &'a self,
        key: &'a str,
        range: Option<reliquary_core::seams::range::Range>,
        deadline: std::time::Instant,
    ) -> reliquary_core::seams::BoxFuture<'a, Result<reliquary_core::seams::bytes::Reading, Refusal>>
    {
        self.0.get(key, range, deadline)
    }
    fn head<'a>(
        &'a self,
        key: &'a str,
        deadline: std::time::Instant,
    ) -> reliquary_core::seams::BoxFuture<'a, Result<reliquary_core::seams::bytes::Meta, Refusal>>
    {
        self.0.head(key, deadline)
    }
    fn delete<'a>(
        &'a self,
        key: &'a str,
        deadline: std::time::Instant,
    ) -> reliquary_core::seams::BoxFuture<'a, Result<Deleted, Refusal>> {
        self.0.delete(key, deadline)
    }
    fn grant<'a>(
        &'a self,
        key: &'a str,
        access: Access,
        holder: &'a str,
        expires: SystemTime,
        deadline: std::time::Instant,
    ) -> reliquary_core::seams::BoxFuture<'a, Result<reliquary_core::seams::bytes::Grant, Refusal>>
    {
        self.0.grant(key, access, holder, expires, deadline)
    }
}

/// AC-RQ0.4.2: the bytes suite is not vacuous: a store that forgets create-once's condition fails
/// the clauses that state it, BYTES-6 and BYTES-7, and only those.
#[test]
fn ac_rq0_4_2_the_bytes_suite_fails_a_store_that_forgets_create_once() {
    use reliquary_core::seams::bytes::suite::BytesFixture;
    use reliquary_core::seams::conformance::Fixture;
    struct Broken(SimBytesFixture, Careless);
    impl Fixture for Broken {
        fn drive<'a, T>(
            &'a self,
            work: reliquary_core::seams::BoxFuture<'a, T>,
        ) -> Result<T, String> {
            self.0.drive(work)
        }
        fn injected(&self) -> reliquary_core::seams::fault::Counts {
            self.0.injected()
        }
    }
    impl BytesFixture for Broken {
        fn bytes(&self) -> &dyn reliquary_core::seams::bytes::Bytes {
            &self.1
        }
        fn clock(&self) -> &dyn reliquary_core::seams::clock::Clock {
            self.0.clock()
        }
        fn pass(&self, d: Duration) -> Result<(), String> {
            self.0.pass(d)
        }
        fn arm(&self, op: BytesOp, fault: Fault<Refusal>) -> Result<(), String> {
            self.0.arm(op, fault)
        }
        fn attempts(&self, op: BytesOp) -> u64 {
            self.0.attempts(op)
        }
        fn redeem<'a>(
            &'a self,
            grant: &'a reliquary_core::seams::bytes::Grant,
            holder: &'a str,
            deadline: std::time::Instant,
        ) -> reliquary_core::seams::BoxFuture<'a, Result<Redeemed, Refusal>> {
            self.0.redeem(grant, holder, deadline)
        }
        fn patience(&self) -> Duration {
            self.0.patience()
        }
        fn length(&self) -> u64 {
            self.0.length()
        }
        fn uploads_max(&self) -> usize {
            self.0.uploads_max()
        }
    }
    let report = conformance::run("Bytes", &bytes::suite::cases(), || {
        let fixture = bytes_fixture()?;
        let careless = Careless(fixture.bytes.clone());
        Ok(Broken(fixture, careless))
    });
    let failed: Vec<_> = report.failed.iter().map(|(id, _)| *id).collect();
    assert_eq!(failed, ["BYTES-6", "BYTES-7"], "{:?}", report.failed);
}

/// AC-RQ0.4.3: cut a clock's stream of wakes at every instant from before three sleeps' deadlines
/// to after them, a tenth of a quantum apart, each on a fresh clock: every sleep due by the cut
/// completes, and every later one is refused Ended, as is every sleep made after it.
#[test]
fn ac_rq0_4_3_a_clock_cut_at_any_instant_completes_what_is_due_and_ends_the_rest() {
    use std::task::{Context, Poll, Waker};
    let poll = |sleep: &mut clock::Sleep| match sleep
        .as_mut()
        .poll(&mut Context::from_waker(Waker::noop()))
    {
        Poll::Ready(outcome) => Some(outcome),
        Poll::Pending => None,
    };
    let step = PATIENCE / 10;
    for tenths in 0..=40u32 {
        let clock = SimClock::new(epoch(), 8);
        let start = clock.now();
        let deadlines = [start + PATIENCE, start + 2 * PATIENCE, start + 3 * PATIENCE];
        let mut sleeps: Vec<_> = deadlines.iter().map(|d| clock.sleep_until(*d)).collect();
        for sleep in &mut sleeps {
            assert_eq!(poll(sleep), None);
        }
        let cut = start + step * tenths;
        clock.cut_at(cut).unwrap();
        clock.advance(4 * PATIENCE).unwrap();
        for (deadline, sleep) in deadlines.iter().zip(&mut sleeps) {
            let want = if *deadline <= cut {
                Ok(())
            } else {
                Err(clock::Refusal::Ended)
            };
            assert_eq!(
                poll(sleep),
                Some(want),
                "cut at {tenths} tenths, the sleep until {deadline:?}"
            );
        }
        let mut later = clock.sleep_until(clock.now());
        assert_eq!(
            poll(&mut later),
            Some(Err(clock::Refusal::Ended)),
            "cut at {tenths} tenths"
        );
        assert_eq!(clock.injected().get(reliquary_core::seams::fault::CUT), 1);
    }
}

/// The variants of `enum Refusal` in the source file at `path` (relative to the workspace's root),
/// read from the source itself, not from any list the simulators keep.
fn refusal_variants(path: &str) -> Vec<String> {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(path),
    )
    .unwrap();
    let code: String = source
        .lines()
        .map(|line| line.split_once("//").map_or(line, |(code, _)| code))
        .collect::<Vec<_>>()
        .join("\n");
    let at = code.find("pub enum Refusal").expect("an enum Refusal");
    let open = at + code.get(at..).unwrap().find('{').unwrap();
    let mut depth = 0i64;
    let mut variants = Vec::new();
    for line in code.get(open + 1..).unwrap().lines() {
        let trimmed = line.trim();
        if depth == 0 && trimmed.starts_with('}') {
            break;
        }
        if depth == 0 && !trimmed.starts_with('#') {
            let name: String = trimmed
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() {
                variants.push(name);
            }
        }
        depth += i64::try_from(line.matches(['{', '(']).count()).unwrap();
        depth -= i64::try_from(line.matches(['}', ')']).count()).unwrap();
    }
    variants
}

/// AC-RQ0.4.3, T-RQ0.4.9: for each of the seven seams, read every refusal its trait names from the
/// trait's own source (Amphora's and Quiver's from their vendored `-api` crates), run its suite
/// against its simulator, and check what the simulator injected: every one of those refusals, an
/// answer delayed past its caller's deadline, an answer lost after its effect applied, and a stream
/// cut at a byte (a row, a message, an instant), each at least once.
#[test]
fn ac_rq0_4_3_every_simulator_injects_every_refusal_its_trait_names_a_delay_a_lost_answer_and_a_cut()
 {
    use reliquary_core::seams::fault::{CUT, DELAY, LOSE};
    let clock_fixture = || {
        Ok(SimClockFixture {
            clock: SimClock::new(epoch(), 8),
            quantum: PATIENCE,
            sleeps: 8,
            steps: STEPS,
        })
    };
    let entropy_fixture = || {
        let clock = SimClock::new(epoch(), 8);
        Ok(SimEntropyFixture {
            entropy: SimEntropy::new(11, Arc::new(clock.clone())),
            clock,
            patience: PATIENCE,
            draws: 1000,
            steps: STEPS,
        })
    };
    let runs = [
        (
            "crates/reliquary-core/src/seams/bytes.rs",
            conformance::run("Bytes", &bytes::suite::cases(), bytes_fixture),
        ),
        (
            "vendor/amphora/crates/api/src/lib.rs",
            conformance::run("Records", &records::cases(), records_fixture),
        ),
        (
            "vendor/quiver/crates/quiver-api/src/lib.rs",
            conformance::run("Topics", &topics::cases(), topics_fixture),
        ),
        (
            "crates/reliquary-core/src/seams/secrets.rs",
            conformance::run("Secrets", &secrets::cases(), secrets_fixture),
        ),
        (
            "crates/reliquary-core/src/seams/upstream.rs",
            conformance::run("Upstream", &upstream::cases(), upstream_fixture),
        ),
        (
            "crates/reliquary-core/src/seams/clock.rs",
            conformance::run("Clock", &clock::cases(), clock_fixture),
        ),
        (
            "crates/reliquary-core/src/seams/entropy.rs",
            conformance::run("Entropy", &entropy::cases(), entropy_fixture),
        ),
    ];
    for (source, report) in runs {
        let refusals = refusal_variants(source);
        assert!(!refusals.is_empty(), "{source}: no refusals read");
        let mut said = Vec::new();
        for kind in refusals
            .iter()
            .map(String::as_str)
            .chain([DELAY, LOSE, CUT])
        {
            let n = report.injected.get(kind);
            assert!(
                n > 0,
                "{}: the simulator never injected {kind} (read from {source})",
                report.seam
            );
            said.push(format!("{kind}={n}"));
        }
        eprintln!(
            "{}: {} refusals read from {source}; injected: {}",
            report.seam,
            refusals.len(),
            said.join(", ")
        );
        report.into_result().unwrap();
    }
}

/// T-RQ0.4.4: ask an object for ranges at its first byte, at its last byte (by its position and by
/// a suffix), past its end, of zero length, and with a last byte before the first, through `Bytes`
/// and through `Upstream`: each is answered as RFC 9110 §14.1 has it, a satisfiable range with
/// exactly its bytes and an unsatisfiable one refused with the object's length.
#[test]
fn t_rq0_4_4_ranges_at_the_first_byte_the_last_past_the_end_and_of_zero_length_follow_rfc_9110() {
    use reliquary_core::seams::range::{Range, Span};
    use reliquary_core::seams::upstream::{Refusal as Up, Upstream as _};
    let data: Vec<u8> = (0..40u8).collect();
    let n = data.len() as u64;
    let (bytes, clock) = store(Plan { piece: 8, run: 8 });
    put(
        &bytes,
        &clock,
        "k",
        &data,
        Put {
            create_once: true,
            identity: 1,
        },
    )
    .unwrap();
    let upstream = SimUpstream::new(Arc::new(clock.clone()));
    let digest = upstream.push("repo", "application/octet-stream", &data);
    let satisfiable = [
        (
            Range::From {
                first: 0,
                last: Some(0),
            },
            Span {
                first: 0,
                length: 1,
            },
        ),
        (
            Range::From {
                first: n - 1,
                last: Some(n - 1),
            },
            Span {
                first: n - 1,
                length: 1,
            },
        ),
        (
            Range::Suffix(1),
            Span {
                first: n - 1,
                length: 1,
            },
        ),
        (
            Range::From {
                first: 5,
                last: Some(n + 100),
            },
            Span {
                first: 5,
                length: n - 5,
            },
        ),
    ];
    for (range, span) in satisfiable {
        let read = clock
            .run(
                async {
                    let mut reading = bytes.get("k", Some(range), due(&clock)).await?;
                    let mut out = vec![0u8; 64];
                    let mut got = Vec::new();
                    loop {
                        let k = reading.body.read(&mut out, due(&clock)).await?;
                        if k == 0 {
                            return Ok::<_, Refusal>((reading.span, got));
                        }
                        got.extend_from_slice(&out[..k]);
                    }
                },
                STEPS,
            )
            .unwrap()
            .unwrap();
        assert_eq!(read.0, span, "Bytes: {range:?}");
        assert_eq!(
            read.1,
            data[ix(span.first)..ix(span.end())],
            "Bytes: {range:?}"
        );
        let fetched = clock
            .run(
                upstream.blob("repo", &digest, Some(range), due(&clock)),
                STEPS,
            )
            .unwrap()
            .unwrap();
        assert_eq!(fetched.span, span, "Upstream: {range:?}");
    }
    for range in [
        Range::From {
            first: n,
            last: None,
        },
        Range::Suffix(0),
    ] {
        let got = clock
            .run(bytes.get("k", Some(range), due(&clock)), STEPS)
            .unwrap()
            .map(|r| r.span);
        assert_eq!(
            got,
            Err(Refusal::Unsatisfiable { length: n }),
            "Bytes: {range:?}"
        );
        let got = clock
            .run(
                upstream.blob("repo", &digest, Some(range), due(&clock)),
                STEPS,
            )
            .unwrap()
            .map(|f| f.span);
        assert_eq!(
            got,
            Err(Up::Unsatisfiable { length: n }),
            "Upstream: {range:?}"
        );
    }
    let backwards = Range::From {
        first: 9,
        last: Some(3),
    };
    let got = clock
        .run(bytes.get("k", Some(backwards), due(&clock)), STEPS)
        .unwrap()
        .map(|r| r.span);
    assert!(
        matches!(got, Err(Refusal::Invalid { .. })),
        "Bytes: {got:?}"
    );
    // Of an empty object only a suffix selects, and it selects nothing.
    put(
        &bytes,
        &clock,
        "empty",
        &[],
        Put {
            create_once: true,
            identity: 2,
        },
    )
    .unwrap();
    let got = clock
        .run(
            bytes.get(
                "empty",
                Some(Range::From {
                    first: 0,
                    last: None,
                }),
                due(&clock),
            ),
            STEPS,
        )
        .unwrap()
        .map(|r| r.span);
    assert_eq!(got, Err(Refusal::Unsatisfiable { length: 0 }));
    let got = clock
        .run(
            bytes.get("empty", Some(Range::Suffix(5)), due(&clock)),
            STEPS,
        )
        .unwrap()
        .map(|r| r.span);
    assert_eq!(got, Ok(Span::whole(0)));
}
