//! AC-RQ0.4.4: no seam carries a whole object in memory. What moves through each one moves in
//! bounded pieces, so the heap's peak while more and more moves through it stays where it was for
//! the least: for `Bytes` (T-RQ0.4.1) and `Upstream`, objects of one piece, 64 and 1024; for
//! `Records` and `Topics`, tables and topics of one page, 64 and 1024, read a page at a time; and for
//! `Secrets`, `Clock` and `Entropy`, 64 and 1024 times the calls of the least, or a caller's buffer
//! 64 and 1024 times larger.
//!
//! The process's allocator here is hyper-measure's counting one, so each peak is counted, not
//! estimated, on the thread that makes the calls. What a seam's far side keeps (a store's objects,
//! an upstream's blobs, a table's rows, a topic's messages) is made before the count begins: it is
//! the other party's, not the seam's traffic. `Bytes` keeps its objects in files, as Mantle keeps
//! them on its disks.

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

use std::fs::{self, File, OpenOptions};
use std::io::{ErrorKind, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};

use amphora_api::{Consistency, Records as _, Scan, Tx};
use hyper_measure::alloc::{self, Counting};
use quiver_api::Topics as _;
use reliquary_core::seams::bytes::sim::{SimBytes, SimLimits, Store};
use reliquary_core::seams::bytes::{Bytes, Hasher, Plan, Put, Whole};
use reliquary_core::seams::clock::{Clock, SimClock};
use reliquary_core::seams::entropy::{Entropy, SimEntropy, Stream};
use reliquary_core::seams::records::{self, SimRecords};
use reliquary_core::seams::secrets::{self, Secrets, SimSecrets};
use reliquary_core::seams::topics::FaultyTopics;
use reliquary_core::seams::upstream::{SimUpstream, Upstream};

#[global_allocator]
static COUNTING: Counting = Counting;

/// Bodies kept as files in a directory of the test's own.
struct FileStore {
    dir: tempfile::TempDir,
}

impl FileStore {
    fn path(&self, body: u64) -> PathBuf {
        self.dir.path().join(body.to_string())
    }
}

impl Store for FileStore {
    fn append(&self, body: u64, bytes: &[u8]) -> Result<(), String> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.path(body))
            .map_err(|e| e.to_string())?;
        file.write_all(bytes).map_err(|e| e.to_string())
    }

    fn truncate(&self, body: u64, length: u64) -> Result<(), String> {
        match OpenOptions::new().write(true).open(self.path(body)) {
            Ok(file) => file.set_len(length).map_err(|e| e.to_string()),
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }

    fn read_at(&self, body: u64, at: u64, into: &mut [u8]) -> Result<usize, String> {
        let mut file = File::open(self.path(body)).map_err(|e| e.to_string())?;
        file.seek(SeekFrom::Start(at)).map_err(|e| e.to_string())?;
        let mut n = 0;
        while n < into.len() {
            match file.read(&mut into[n..]) {
                Ok(0) => break,
                Ok(got) => n += got,
                Err(e) if e.kind() == ErrorKind::Interrupted => {}
                Err(e) => return Err(e.to_string()),
            }
        }
        Ok(n)
    }

    fn remove(&self, body: u64) -> Result<(), String> {
        match fs::remove_file(self.path(body)) {
            Err(e) if e.kind() != ErrorKind::NotFound => Err(e.to_string()),
            _ => Ok(()),
        }
    }
}

/// The piece the caller moves: the store's plan's, and the caller's buffer.
const PIECE: usize = 16 << 10;
/// A page of rows, or a batch of messages, a caller reads at a time.
const PAGE: usize = 16;
/// How much more than the least each seam's test moves.
const SCALES: [u64; 2] = [64, 1024];

fn epoch() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_767_225_600)
}

/// A deadline no simulated call reaches: no fault is armed, so no call waits.
fn deadline(clock: &SimClock) -> Instant {
    clock.now() + Duration::from_secs(3600)
}

/// `at` as an index.
fn ix(at: u64) -> usize {
    usize::try_from(at).unwrap()
}

/// Measures `moved` at the least (once to warm up, then counted) and at each of [`SCALES`]: the
/// least's peak, after the peak at each scale is held to it.
fn held_flat(seam: &str, moved: impl Fn(u64) -> i64) -> i64 {
    assert!(alloc::installed(), "the counting allocator is this test's");
    moved(1);
    let least = moved(1);
    for scale in SCALES {
        let peak = moved(scale);
        eprintln!(
            "{seam}: {scale} times the least moved: peak heap {peak} bytes (the least: {least})"
        );
        assert!(
            peak <= least,
            "{seam}: the heap's peak grew with what moved: {peak} bytes at {scale} times, {least} at one"
        );
    }
    least
}

/// Puts an object of `pieces` pieces through `Bytes` and gets it back, its bytes drawn from a
/// seeded stream and checked against the same stream as they come back: the heap's peak while it
/// moved, counted from before the caller's piece was allocated.
fn bytes_moved(pieces: u64) -> i64 {
    let clock = SimClock::new(epoch(), 8);
    let store = FileStore {
        dir: tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).unwrap(),
    };
    let dir = store.dir.path().to_owned();
    let piece = u64::try_from(PIECE).unwrap();
    let length = pieces * piece;
    let limits = SimLimits {
        plan: Plan {
            piece: PIECE,
            run: piece,
        },
        objects: 1,
        uploads: 1,
        bytes: length,
        key: 64,
    };
    let bytes = SimBytes::new(Arc::new(clock.clone()), &[1; 32], Arc::new(store), limits).unwrap();
    let deadline = deadline(&clock);
    let (mut source, mut check) = (Stream::new(pieces), Stream::new(pieces));
    // Each call's future is boxed and freed; the step budget is one poll per call and a few more.
    let steps = pieces * 4 + 16;

    // The test's own copy to check against is not the caller's: it is made before the count.
    let mut expect = vec![0u8; PIECE];
    alloc::begin();
    let mut buffer = vec![0u8; PIECE];
    let outcome = clock.run(
        async {
            let put = Put {
                create_once: true,
                identity: 1,
            };
            let upload = bytes.begin("big", put, deadline).await?;
            let mut hasher = Hasher::default();
            let mut at = 0;
            while at < length {
                source.fill(&mut buffer).unwrap();
                hasher.update(&buffer);
                at = bytes
                    .send(&upload.id, at, &buffer, deadline)
                    .await?
                    .received;
            }
            let whole = Whole {
                length,
                sha256: hasher.finish(),
            };
            let meta = bytes.finish(&upload.id, whole, deadline).await?;
            assert_eq!(meta.length, length);
            let mut reading = bytes.get("big", None, deadline).await?;
            let mut back = 0;
            loop {
                let n = reading.body.read(&mut buffer, deadline).await?;
                if n == 0 {
                    break;
                }
                check.fill(&mut expect[..n]).unwrap();
                assert!(
                    buffer[..n] == expect[..n],
                    "the bytes at {back} came back changed"
                );
                back += u64::try_from(n).unwrap();
            }
            assert_eq!(back, length, "the object came back short");
            // Deleted and read no more, the object's bytes leave the store.
            drop(reading);
            bytes.delete("big", deadline).await?;
            Ok::<(), reliquary_core::seams::bytes::Refusal>(())
        },
        steps,
    );
    drop((buffer, expect));
    let counts = alloc::end();
    outcome.unwrap().unwrap();
    assert_eq!(
        bytes.removals_refused(),
        0,
        "the store refused to let a body go"
    );
    assert_eq!(
        fs::read_dir(&dir).unwrap().count(),
        0,
        "a body the store no longer holds was left in its directory"
    );
    counts.peak
}

/// T-RQ0.4.1, AC-RQ0.4.4: put an object of one piece, then of 64 and of 1024 (16 MiB, hundreds of
/// times what the smallest's put and get held at once) through `Bytes`, and get each back:
/// the bytes come back the same, and the heap's peak is the same for each: one piece, and the
/// overhead counted for an object of one piece.
#[test]
fn t_rq0_4_1_a_large_object_moves_in_pieces_and_the_heaps_peak_does_not_grow() {
    let budget = held_flat("Bytes", bytes_moved);
    let piece = i64::try_from(PIECE).unwrap();
    assert!(
        budget >= piece,
        "the caller's piece is in the count: {budget}"
    );
    let largest = 1024 * u64::try_from(PIECE).unwrap();
    assert!(
        largest >= 256 * u64::try_from(budget).unwrap(),
        "an object of {largest} bytes is not many times the {budget} bytes the smallest held"
    );
    eprintln!(
        "Bytes: one piece of {piece} bytes and a counted overhead of {} bytes",
        budget - piece
    );
}

/// Fetches a blob of `pieces` pieces whole from an upstream into one piece-sized buffer, held to
/// its digest as it comes: the heap's peak while it moved.
fn upstream_moved(pieces: u64) -> i64 {
    let clock = SimClock::new(epoch(), 8);
    let upstream = SimUpstream::new(Arc::new(clock.clone()));
    let length = ix(pieces) * PIECE;
    let mut content = vec![0u8; length];
    Stream::new(pieces).fill(&mut content).unwrap();
    let digest = upstream.push("repo", "application/vnd.oci.image.layer.v1.tar", &content);
    drop(content);
    let deadline = deadline(&clock);
    alloc::begin();
    let mut buffer = vec![0u8; PIECE];
    let outcome = clock.run(
        async {
            let mut fetched = upstream.blob("repo", &digest, None, deadline).await?;
            let mut total = 0;
            loop {
                let n = fetched.body.read(&mut buffer, deadline).await?;
                if n == 0 {
                    return Ok::<usize, reliquary_core::seams::upstream::Refusal>(total);
                }
                total += n;
            }
        },
        pieces * 4 + 16,
    );
    drop(buffer);
    let counts = alloc::end();
    assert_eq!(
        outcome.unwrap().unwrap(),
        length,
        "the blob came back whole"
    );
    counts.peak
}

/// AC-RQ0.4.4: fetch a blob of one piece, 64 and 1024 (16 MiB) from an upstream through `Upstream`,
/// whole and held to its digest: the heap's peak is the same for each.
#[test]
fn ac_rq0_4_4_an_upstream_body_moves_in_pieces_and_the_heaps_peak_does_not_grow() {
    held_flat("Upstream", upstream_moved);
}

/// Scans a family of `pages` pages of rows a page at a time, and reads its change feed a page at a
/// time: the heap's peak while it read them.
fn records_read(pages: u64) -> i64 {
    let clock = SimClock::new(epoch(), 8);
    let rows = ix(pages) * PAGE;
    let limits = records::SimLimits {
        rows,
        changes: rows,
    };
    let store = SimRecords::new(clock.clone(), limits);
    let deadline = deadline(&clock);
    for n in 0..rows {
        let tx = Tx {
            conds: vec![],
            writes: vec![amphora_api::Write::Put {
                family: "rows".into(),
                key: format!("row-{n:08}").into_bytes(),
                value: n.to_le_bytes().to_vec(),
            }],
        };
        store.commit(tx, deadline).unwrap();
    }
    alloc::begin();
    let mut seen = 0;
    let mut after = None;
    loop {
        let scan = Scan {
            prefix: Vec::new(),
            after: after.take(),
            limit: PAGE,
            at: Consistency::Linearizable,
        };
        // Read until a page comes back empty, so every table's read goes on past its first page.
        let page = store.scan("rows", scan, deadline).unwrap();
        if page.rows.is_empty() {
            break;
        }
        seen += page.rows.len();
        after = page.rows.last().map(|row| row.key.clone());
    }
    let mut changed = 0;
    let mut from = 0;
    loop {
        let read = store.changes("rows", from, PAGE, deadline).unwrap();
        let Some(last) = read.changes.last().map(|c| c.rev) else {
            break;
        };
        changed += read.changes.len();
        from = last;
    }
    let counts = alloc::end();
    assert_eq!(
        (seen, changed),
        (rows, rows),
        "every row and change was read"
    );
    counts.peak
}

/// AC-RQ0.4.4: scan a family of one page of rows, 64 and 1024 through `Records`, and read its change
/// feed, a page at a time: the heap's peak is the same for each.
#[test]
fn ac_rq0_4_4_records_move_a_page_at_a_time_and_the_heaps_peak_does_not_grow() {
    held_flat("Records", records_read);
}

/// Reads a topic of `batches` batches of messages a batch at a time: the heap's peak while it read
/// them.
fn topics_read(batches: u64) -> i64 {
    let clock = SimClock::new(epoch(), 8);
    let topics = FaultyTopics::new(quiver_api::sim::SimTopics::new(), Arc::new(clock.clone()));
    let messages = batches * u64::try_from(PAGE).unwrap();
    let topic = quiver_api::Topic {
        class: quiver_api::Class::Ephemeral,
        retain_messages: messages,
        retain_bytes: 1 << 30,
    };
    clock
        .run(
            async {
                topics.create("events", topic).await.unwrap();
                for n in 0..messages {
                    topics
                        .publish("events", None, &n.to_le_bytes())
                        .await
                        .unwrap();
                }
            },
            messages * 4 + 16,
        )
        .unwrap();
    alloc::begin();
    let outcome = clock.run(
        async {
            let mut read = 0;
            let mut from = 0;
            loop {
                let batch = topics.read("events", from, PAGE).await.unwrap();
                if batch.messages.is_empty() {
                    return read;
                }
                read += batch.messages.len();
                from = batch.next;
            }
        },
        messages * 4 + 16,
    );
    let counts = alloc::end();
    assert_eq!(
        u64::try_from(outcome.unwrap()).unwrap(),
        messages,
        "every message was read"
    );
    counts.peak
}

/// AC-RQ0.4.4: read a topic of one batch of messages, 64 and 1024 through `Topics`, a batch at a
/// time: the heap's peak is the same for each.
#[test]
fn ac_rq0_4_4_topics_move_a_batch_at_a_time_and_the_heaps_peak_does_not_grow() {
    held_flat("Topics", topics_read);
}

/// Leases a credential and gives it back, `times` times in turn: the heap's peak while it did.
fn secrets_leased(times: u64) -> i64 {
    let clock = SimClock::new(epoch(), 8);
    let limits = secrets::SimLimits {
        leases: 1,
        life: Duration::from_secs(60),
    };
    let store = SimSecrets::new(Arc::new(clock.clone()), limits);
    store.keep("robot", b"a-credential-of-its-own-size", true);
    let deadline = deadline(&clock);
    alloc::begin();
    let outcome = clock.run(
        async {
            for _ in 0..times {
                let lease = store.lease("robot", deadline).await?;
                store.release(&lease.id, deadline).await?;
            }
            Ok::<(), secrets::Refusal>(())
        },
        times * 4 + 16,
    );
    let counts = alloc::end();
    outcome.unwrap().unwrap();
    counts.peak
}

/// AC-RQ0.4.4: lease a credential and give it back once, 64 times and 1024 times in turn through
/// `Secrets`: a lease holds no more than its credential, and nothing of one stays after it, so the
/// heap's peak is the same for each.
#[test]
fn ac_rq0_4_4_a_lease_holds_its_credential_alone_and_the_heaps_peak_does_not_grow() {
    held_flat("Secrets", secrets_leased);
}

/// Fills a caller's buffer of `scale` pieces from a source: the heap's peak while it did, the
/// buffer being the caller's, made before the count.
fn entropy_filled(scale: u64) -> i64 {
    let clock = SimClock::new(epoch(), 8);
    let source = SimEntropy::new(scale, Arc::new(clock.clone()));
    let mut buffer = vec![0u8; ix(scale) * PIECE];
    let deadline = deadline(&clock);
    alloc::begin();
    let outcome = clock.run(source.fill(&mut buffer, deadline), 16);
    let counts = alloc::end();
    outcome.unwrap().unwrap();
    counts.peak
}

/// AC-RQ0.4.4: fill a caller's buffer of one piece, 64 and 1024 through `Entropy`: the source
/// writes into the caller's buffer and holds none of its own, so the heap's peak is the same.
#[test]
fn ac_rq0_4_4_a_fill_writes_into_its_callers_buffer_and_the_heaps_peak_does_not_grow() {
    held_flat("Entropy", entropy_filled);
}

/// Sleeps a quantum `times` times in turn: the heap's peak while it did.
fn clock_slept(times: u64) -> i64 {
    let clock = SimClock::new(epoch(), 8);
    let quantum = Duration::from_secs(1);
    alloc::begin();
    for _ in 0..times {
        let wake = clock.now() + quantum;
        clock.run(clock.sleep_until(wake), 16).unwrap().unwrap();
    }
    let counts = alloc::end();
    counts.peak
}

/// AC-RQ0.4.4: sleep once, 64 times and 1024 times in turn through `Clock`: a sleep holds its
/// place alone and nothing of it stays after it, so the heap's peak is the same for each.
#[test]
fn ac_rq0_4_4_sleeps_hold_their_place_alone_and_the_heaps_peak_does_not_grow() {
    held_flat("Clock", clock_slept);
}
