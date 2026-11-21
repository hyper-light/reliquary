//! The `Bytes` contract, clause by clause (`docs/design/seams.md`, "Bytes"): the cases every
//! implementation passes, the simulator now and Mantle's when it lands (plan RQ0.4, RQ1.8).
//!
//! Every size a case uses is its fixture's, as the test that makes the fixture states it: how long
//! a caller waits, how long an object is (several of the store's pieces and runs), how many uploads
//! the store holds open. Each case's contents and operation identities are drawn from its keys, so a
//! case is a function of its fixture.

use std::time::{Duration, Instant};

use super::{
    Access, Bytes, Deleted, Grant, Meta, Method, Op, Plan, Progress, Put, Reading, Redeemed,
    Refusal, Sha256, Upload, Whole,
};
use crate::seams::BoxFuture;
use crate::seams::clock::Clock;
use crate::seams::conformance::{Case, Failure, Fixture, both, ensure, fine, refused, same};
use crate::seams::entropy::Stream;
use crate::seams::fault::Fault;
use crate::seams::range::{Range, Span};

/// What a bytes store's conformance suite runs against.
pub trait BytesFixture: Fixture {
    fn bytes(&self) -> &dyn Bytes;
    /// The clock the store's deadlines and grants' expiries are kept by.
    fn clock(&self) -> &dyn Clock;
    /// Lets `d` pass by the store's clock.
    fn pass(&self, d: Duration) -> Result<(), Failure>;
    /// Arms `fault` for the store's next call of `op`.
    fn arm(&self, op: Op, fault: Fault<Refusal>) -> Result<(), Failure>;
    /// How many calls of `op` the store has had.
    fn attempts(&self, op: Op) -> u64;
    /// Mantle's side: `holder` presents `grant`.
    fn redeem<'a>(
        &'a self,
        grant: &'a Grant,
        holder: &'a str,
        deadline: Instant,
    ) -> BoxFuture<'a, Result<Redeemed, Refusal>>;
    /// How long a case's caller waits for an answer: each deadline is the clock's now plus this.
    fn patience(&self) -> Duration;
    /// How long a case's objects are: several of the store's pieces and runs, and at least two
    /// bytes, so an object has a first byte and a last that are not the same.
    fn length(&self) -> u64;
    /// The most uploads the store holds open at once.
    fn uploads_max(&self) -> usize;
}

fn count(n: usize) -> u64 {
    u64::try_from(n).unwrap_or(u64::MAX)
}

fn deadline<F: BytesFixture>(f: &F) -> Result<Instant, Failure> {
    f.clock()
        .now()
        .checked_add(f.patience())
        .ok_or_else(|| "a deadline past what the platform's time can say".to_owned())
}

fn digest_of(parts: &[&[u8]]) -> Sha256 {
    let mut hasher = super::Hasher::default();
    for part in parts {
        hasher.update(part);
    }
    hasher.finish()
}

/// `length` bytes no one would guess, drawn for `key`.
fn content(key: &str, length: u64) -> Result<Vec<u8>, Failure> {
    let seed = digest_of(&[key.as_bytes()]).0;
    let mut first = [0u8; size_of::<u64>()];
    for (into, from) in first.iter_mut().zip(seed) {
        *into = from;
    }
    let mut bytes = vec![0u8; usize::try_from(length).map_err(|e| e.to_string())?];
    Stream::new(u64::from_le_bytes(first))
        .fill(&mut bytes)
        .map_err(|e| e.to_string())?;
    Ok(bytes)
}

/// The identity of the put of `data` to `key`: the same put asked again has the same one.
fn identity(key: &str, data: &[u8]) -> u128 {
    let digest = digest_of(&[key.as_bytes(), data]).0;
    let mut first = [0u8; size_of::<u128>()];
    for (into, from) in first.iter_mut().zip(digest) {
        *into = from;
    }
    u128::from_le_bytes(first)
}

fn whole(data: &[u8]) -> Whole {
    Whole {
        length: count(data.len()),
        sha256: Sha256::of(data),
    }
}

fn put_of(key: &str, data: &[u8], create_once: bool) -> Put {
    Put {
        create_once,
        identity: identity(key, data),
    }
}

/// The bytes of `data` from byte `at`, at most a piece of `plan`.
fn piece_at(data: &[u8], at: u64, plan: Plan) -> Result<&[u8], Failure> {
    let start = usize::try_from(at).map_err(|e| e.to_string())?;
    let rest = data
        .get(start..)
        .ok_or_else(|| format!("byte {at} is past the data"))?;
    rest.get(..plan.piece.min(rest.len()))
        .ok_or_else(|| "a piece past the data".to_owned())
}

/// Sends `data` from byte `from` to its end, a piece at a time, noting each byte sent in `sent`.
async fn send_from<F: BytesFixture>(
    f: &F,
    upload: &Upload,
    data: &[u8],
    from: u64,
    mut sent: Option<&mut Vec<u32>>,
) -> Result<Result<Progress, Refusal>, Failure> {
    let end = count(data.len());
    let mut at = from;
    let mut progress = upload.progress;
    while at < end {
        let piece = piece_at(data, at, upload.plan)?;
        if let Some(sent) = sent.as_deref_mut() {
            note(sent, at, piece.len())?;
        }
        progress = match f.bytes().send(&upload.id, at, piece, deadline(f)?).await {
            Ok(progress) => progress,
            Err(refusal) => return Ok(Err(refusal)),
        };
        ensure(progress.received > at, || {
            format!("a send at {at} moved the upload nowhere")
        })?;
        at = progress.received;
    }
    Ok(Ok(progress))
}

/// Notes that `n` bytes from byte `at` were sent once more.
fn note(sent: &mut [u32], at: u64, n: usize) -> Result<(), Failure> {
    let start = usize::try_from(at).map_err(|e| e.to_string())?;
    let bytes = sent
        .get_mut(start..)
        .and_then(|rest| rest.get_mut(..n))
        .ok_or_else(|| "a piece past the object".to_owned())?;
    for byte in bytes {
        *byte = byte.saturating_add(1);
    }
    Ok(())
}

/// Puts `data` at `key`: begun, sent in pieces, finished whole.
async fn put<F: BytesFixture>(
    f: &F,
    key: &str,
    data: &[u8],
    create_once: bool,
) -> Result<(Meta, Plan), Failure> {
    let upload = fine(
        "a begin",
        f.bytes()
            .begin(key, put_of(key, data, create_once), deadline(f)?)
            .await,
    )?;
    fine(
        "a send",
        send_from(f, &upload, data, upload.progress.received, None).await?,
    )?;
    let meta = fine(
        "a finish",
        f.bytes()
            .finish(&upload.id, whole(data), deadline(f)?)
            .await,
    )?;
    Ok((meta, upload.plan))
}

/// An upload of `data` to `key`, begun and sent whole, not finished.
async fn staged<F: BytesFixture>(
    f: &F,
    key: &str,
    data: &[u8],
    create_once: bool,
) -> Result<Upload, Failure> {
    let upload = fine(
        "a begin",
        f.bytes()
            .begin(key, put_of(key, data, create_once), deadline(f)?)
            .await,
    )?;
    fine(
        "a send",
        send_from(f, &upload, data, upload.progress.received, None).await?,
    )?;
    Ok(upload)
}

/// Every byte `reading` gives, read into a buffer of `buffer` bytes, each read no more than it.
async fn read_all<F: BytesFixture>(
    f: &F,
    reading: &mut Reading,
    buffer: usize,
) -> Result<Result<Vec<u8>, (Vec<u8>, Refusal)>, Failure> {
    let mut out = Vec::new();
    let mut into = vec![0u8; buffer];
    loop {
        let n = match reading.body.read(&mut into, deadline(f)?).await {
            Ok(n) => n,
            Err(refusal) => return Ok(Err((out, refusal))),
        };
        if n == 0 {
            return Ok(Ok(out));
        }
        let got = into
            .get(..n)
            .ok_or_else(|| format!("a read said {n} bytes into a buffer of {buffer}"))?;
        out.extend_from_slice(got);
        ensure(count(out.len()) <= reading.span.length, || {
            "a reading gave more bytes than its span".into()
        })?;
    }
}

/// The object at `key`, or the bytes of `range` of it, read whole.
async fn read<F: BytesFixture>(
    f: &F,
    key: &str,
    range: Option<Range>,
    buffer: usize,
) -> Result<(Reading, Vec<u8>), Failure> {
    let mut reading = fine("a get", f.bytes().get(key, range, deadline(f)?).await)?;
    let bytes = fine(
        "a reading",
        read_all(f, &mut reading, buffer).await?.map_err(|(_, r)| r),
    )?;
    Ok((reading, bytes))
}

fn b1<F: BytesFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        let data = content("b1", f.length())?;
        let (meta, plan) = put(f, "b1", &data, false).await?;
        same("its length", meta.length, count(data.len()))?;
        same("its SHA-256", meta.sha256, Sha256::of(&data))?;
        same(
            "head",
            fine("a head", f.bytes().head("b1", deadline(f)?).await)?,
            meta.clone(),
        )?;
        let (reading, back) = read(f, "b1", None, plan.piece).await?;
        same("the reading's facts", reading.meta, meta)?;
        same(
            "the reading's span",
            reading.span,
            Span::whole(count(data.len())),
        )?;
        ensure(back == data, || {
            "the bytes read back are not those put".into()
        })
    })
}

fn b2<F: BytesFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        let data = content("b2", f.length())?;
        let upload = fine(
            "a begin",
            f.bytes()
                .begin("b2", put_of("b2", &data, false), deadline(f)?)
                .await,
        )?;
        let over = upload
            .plan
            .piece
            .checked_add(1)
            .ok_or_else(|| "a plan's piece as large as memory".to_owned())?;
        let piece = vec![0u8; over];
        let refusal = refused(
            "a piece over the plan",
            f.bytes().send(&upload.id, 0, &piece, deadline(f)?).await,
        )?;
        ensure(matches!(refusal, Refusal::TooLarge { .. }), || {
            format!("refused {refusal:?}, not TooLarge")
        })?;
        same(
            "the upload after it",
            fine("a status", f.bytes().status(&upload.id, deadline(f)?).await)?,
            upload.progress,
        )
    })
}

fn b3<F: BytesFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        let data = content("b3", f.length())?;
        let upload = fine(
            "a begin",
            f.bytes()
                .begin("b3", put_of("b3", &data, false), deadline(f)?)
                .await,
        )?;
        let first = piece_at(&data, 0, upload.plan)?;
        let after = fine(
            "the first piece",
            f.bytes().send(&upload.id, 0, first, deadline(f)?).await,
        )?;
        let next = piece_at(&data, after.received, upload.plan)?;
        for at in [0, after.received.saturating_add(1)] {
            same(
                &format!("a send at byte {at}"),
                f.bytes().send(&upload.id, at, next, deadline(f)?).await,
                Err(Refusal::Offset {
                    expected: after.received,
                }),
            )?;
        }
        same(
            "the upload after them",
            fine("a status", f.bytes().status(&upload.id, deadline(f)?).await)?,
            after,
        )
    })
}

fn b4<F: BytesFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        let length = f.length();
        for cut in 0..=length {
            let key = format!("b4-{cut}");
            let data = content(&key, length)?;
            let upload = fine(
                "a begin",
                f.bytes()
                    .begin(&key, put_of(&key, &data, false), deadline(f)?)
                    .await,
            )?;
            let mut sent = vec![0u32; data.len()];
            f.arm(Op::Send, Fault::Cut(cut))?;
            let cut_at = match send_from(f, &upload, &data, 0, Some(&mut sent)).await? {
                Err(Refusal::Cut { at }) => at,
                other => return Err(format!("a put cut at byte {cut} ended {other:?}")),
            };
            let progress = fine("a status", f.bytes().status(&upload.id, deadline(f)?).await)?;
            same(
                "the upload after the cut",
                progress,
                Progress {
                    received: cut_at,
                    durable: cut_at,
                },
            )?;
            ensure(
                cut_at <= cut && cut.saturating_sub(cut_at) < upload.plan.run,
                || {
                    format!(
                        "cut at {cut}, the durable offset is {cut_at}: not within the run the cut fell in"
                    )
                },
            )?;
            fine(
                "the rest",
                send_from(f, &upload, &data, cut_at, Some(&mut sent)).await?,
            )?;
            fine(
                "the finish",
                f.bytes()
                    .finish(&upload.id, whole(&data), deadline(f)?)
                    .await,
            )?;
            let below = usize::try_from(cut_at).map_err(|e| e.to_string())?;
            ensure(
                sent.iter().take(below).all(|n| *n == 1) && sent.iter().all(|n| *n >= 1),
                || {
                    format!(
                        "cut at {cut}: a byte before {cut_at} was sent twice, or one not at all"
                    )
                },
            )?;
            let (_, back) = read(f, &key, None, upload.plan.piece).await?;
            ensure(back == data, || {
                format!("cut at {cut}: the object is not the bytes put")
            })?;
        }
        Ok(())
    })
}

fn b5<F: BytesFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        let data = content("b5", f.length())?;
        let other = content("b5-other", f.length())?;
        let upload = staged(f, "b5", &data, false).await?;
        let right = whole(&data);
        let wrong = [
            Whole {
                length: right.length.saturating_add(1),
                ..right
            },
            Whole {
                sha256: Sha256::of(&other),
                ..right
            },
        ];
        for whole in wrong {
            let refusal = refused(
                "a finish not the upload's",
                f.bytes().finish(&upload.id, whole, deadline(f)?).await,
            )?;
            ensure(matches!(refusal, Refusal::Mismatch(_)), || {
                format!("refused {refusal:?}, not Mismatch")
            })?;
        }
        same(
            "the key after them",
            f.bytes().head("b5", deadline(f)?).await,
            Err(Refusal::NotFound("b5".into())),
        )?;
        fine(
            "the finish",
            f.bytes().finish(&upload.id, right, deadline(f)?).await,
        )
        .map(drop)
    })
}

fn b6<F: BytesFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        let first = content("b6", f.length())?;
        let (meta, plan) = put(f, "b6", &first, false).await?;
        let second = content("b6-second", f.length())?;
        same(
            "a create-once begin on a key that holds an object",
            f.bytes()
                .begin("b6", put_of("b6", &second, true), deadline(f)?)
                .await
                .map(|u| u.id),
            Err(Refusal::Exists("b6".into())),
        )?;
        // Begun while the key was empty, finished once an object had come.
        let late = staged(f, "b6-late", &second, true).await?;
        let (there, _) = put(f, "b6-late", &first, false).await?;
        same(
            "a create-once finish on a key that came to hold an object",
            f.bytes()
                .finish(&late.id, whole(&second), deadline(f)?)
                .await,
            Err(Refusal::Exists("b6-late".into())),
        )?;
        same(
            "the object that came",
            fine("a head", f.bytes().head("b6-late", deadline(f)?).await)?,
            there,
        )?;
        let (reading, back) = read(f, "b6", None, plan.piece).await?;
        same("the first object's facts", reading.meta, meta)?;
        ensure(back == first, || "the first object changed".into())
    })
}

fn b7<F: BytesFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        let a = content("b7-a", f.length())?;
        let b = content("b7-b", f.length())?;
        let upload_a = staged(f, "b7", &a, true).await?;
        let upload_b = staged(f, "b7", &b, true).await?;
        let (end_a, end_b) = both(
            f.bytes().finish(&upload_a.id, whole(&a), deadline(f)?),
            f.bytes().finish(&upload_b.id, whole(&b), deadline(f)?),
        )
        .await;
        let winner = match (end_a, end_b) {
            (Ok(meta), Err(Refusal::Exists(_))) => (meta, &a),
            (Err(Refusal::Exists(_)), Ok(meta)) => (meta, &b),
            other => return Err(format!("two create-once finishes ended {other:?}")),
        };
        let (_, back) = read(f, "b7", None, upload_a.plan.piece).await?;
        same("the object's facts", winner.0.sha256, Sha256::of(winner.1))?;
        ensure(back == *winner.1, || {
            "the object is not the winner's".into()
        })
    })
}

fn b8<F: BytesFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        let before = content("b8", f.length())?;
        let (meta, plan) = put(f, "b8", &before, false).await?;
        let after = content("b8-after", f.length())?;
        let upload = staged(f, "b8", &after, false).await?;
        same(
            "head while the next put is unfinished",
            fine("a head", f.bytes().head("b8", deadline(f)?).await)?,
            meta,
        )?;
        let (_, back) = read(f, "b8", None, plan.piece).await?;
        ensure(back == before, || {
            "a get saw an unfinished put's bytes".into()
        })?;
        let next = fine(
            "the finish",
            f.bytes()
                .finish(&upload.id, whole(&after), deadline(f)?)
                .await,
        )?;
        same(
            "head after the finish",
            fine("a head", f.bytes().head("b8", deadline(f)?).await)?,
            next,
        )
    })
}

fn b9<F: BytesFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        let data = content("b9", f.length())?;
        let (meta, plan) = put(f, "b9", &data, false).await?;
        let length = meta.length;
        let last = length
            .checked_sub(1)
            .filter(|last| *last > 0)
            .ok_or_else(|| "the fixture's objects must be at least two bytes".to_owned())?;
        let byte = |at: u64| -> Result<Vec<u8>, Failure> {
            let at = usize::try_from(at).map_err(|e| e.to_string())?;
            Ok(data.get(at..=at).map(<[u8]>::to_vec).unwrap_or_default())
        };
        // Each selects the bytes `want`, from `first`: the span's length is theirs.
        let satisfied = [
            (
                Range::From {
                    first: 0,
                    last: Some(0),
                },
                0,
                byte(0)?,
            ),
            (
                Range::From {
                    first: last,
                    last: Some(last),
                },
                last,
                byte(last)?,
            ),
            (Range::Suffix(1), last, byte(last)?),
            (
                Range::From {
                    first: 0,
                    last: Some(u64::MAX),
                },
                0,
                data.clone(),
            ),
        ];
        for (range, first, want) in satisfied {
            let span = Span {
                first,
                length: count(want.len()),
            };
            let (reading, back) = read(f, "b9", Some(range), plan.piece).await?;
            same(&format!("the span of {range:?}"), reading.span, span)?;
            ensure(back == want, || format!("the bytes of {range:?}"))?;
        }
        let refusals = [
            (
                Range::From {
                    first: length,
                    last: None,
                },
                Refusal::Unsatisfiable { length },
            ),
            (Range::Suffix(0), Refusal::Unsatisfiable { length }),
        ];
        for (range, want) in refusals {
            same(
                &format!("a get of {range:?}"),
                f.bytes()
                    .get("b9", Some(range), deadline(f)?)
                    .await
                    .map(|r| r.span),
                Err(want),
            )?;
        }
        let invalid = refused(
            "a range whose last byte is before its first",
            f.bytes()
                .get(
                    "b9",
                    Some(Range::From {
                        first: last,
                        last: Some(0),
                    }),
                    deadline(f)?,
                )
                .await,
        )?;
        ensure(matches!(invalid, Refusal::Invalid { .. }), || {
            format!("refused {invalid:?}, not Invalid")
        })?;
        // An empty object: only a suffix selects anything of it, which is all of it, no bytes.
        put(f, "b9-empty", &[], false).await?;
        same(
            "a get of an empty object from its first byte",
            f.bytes()
                .get(
                    "b9-empty",
                    Some(Range::From {
                        first: 0,
                        last: None,
                    }),
                    deadline(f)?,
                )
                .await
                .map(|r| r.span),
            Err(Refusal::Unsatisfiable { length: 0 }),
        )?;
        let (reading, back) = read(f, "b9-empty", Some(Range::Suffix(1)), plan.piece).await?;
        same("a suffix of an empty object", reading.span, Span::whole(0))?;
        same("its bytes", back.len(), 0)
    })
}

fn b10<F: BytesFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        let data = content("b10", f.length())?;
        let (_, plan) = put(f, "b10", &data, false).await?;
        for buffer in 1..=plan.piece {
            let (_, back) = read(f, "b10", None, buffer).await?;
            ensure(back == data, || {
                format!("read into a buffer of {buffer}, the bytes are not those put")
            })?;
        }
        Ok(())
    })
}

fn b11<F: BytesFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        let missing = Refusal::NotFound("b11".into());
        same(
            "a head of a key with no object",
            f.bytes().head("b11", deadline(f)?).await,
            Err(missing.clone()),
        )?;
        same(
            "a get of a key with no object",
            f.bytes()
                .get("b11", None, deadline(f)?)
                .await
                .map(|r| r.span),
            Err(missing.clone()),
        )?;
        same(
            "a delete of a key with no object",
            f.bytes().delete("b11", deadline(f)?).await,
            Ok(Deleted::Absent),
        )?;
        put(f, "b11", &content("b11", f.length())?, false).await?;
        same(
            "a delete of an object",
            f.bytes().delete("b11", deadline(f)?).await,
            Ok(Deleted::Removed),
        )?;
        same(
            "a head after the delete",
            f.bytes().head("b11", deadline(f)?).await,
            Err(missing),
        )
    })
}

fn b12<F: BytesFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        let first = content("b12", f.length())?;
        let (_, plan) = put(f, "b12", &first, false).await?;
        let mut deleted = fine("a get", f.bytes().get("b12", None, deadline(f)?).await)?;
        fine("a delete", f.bytes().delete("b12", deadline(f)?).await)?;
        let back = fine(
            "the reading",
            read_all(f, &mut deleted, plan.piece)
                .await?
                .map_err(|(_, r)| r),
        )?;
        ensure(back == first, || {
            "a reading lost its object to a delete".into()
        })?;
        put(f, "b12", &first, false).await?;
        let mut overwritten = fine("a get", f.bytes().get("b12", None, deadline(f)?).await)?;
        let second = content("b12-second", f.length())?;
        put(f, "b12", &second, false).await?;
        let back = fine(
            "the reading",
            read_all(f, &mut overwritten, plan.piece)
                .await?
                .map_err(|(_, r)| r),
        )?;
        ensure(back == first, || {
            "a reading lost its object to an overwrite".into()
        })?;
        let (_, now) = read(f, "b12", None, plan.piece).await?;
        ensure(now == second, || "the overwrite did not take".into())
    })
}

/// A wall time `f`'s patience from now: a grant's expiry.
fn expiry<F: BytesFixture>(f: &F) -> Result<std::time::SystemTime, Failure> {
    f.clock()
        .wall()
        .checked_add(f.patience())
        .ok_or_else(|| "an expiry past what the platform's time can say".to_owned())
}

fn b13<F: BytesFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        let data = content("b13", f.length())?;
        let (meta, plan) = put(f, "b13", &data, false).await?;
        let get = Access {
            method: Method::Get,
            length: meta.length,
        };
        let grant = fine(
            "a grant",
            f.bytes()
                .grant("b13", get, "holder", expiry(f)?, deadline(f)?)
                .await,
        )?;
        let Redeemed::Reading(mut reading) = fine(
            "a grant redeemed",
            f.redeem(&grant, "holder", deadline(f)?).await,
        )?
        else {
            return Err("a get's grant gave no reading".into());
        };
        let back = fine(
            "its reading",
            read_all(f, &mut reading, plan.piece)
                .await?
                .map_err(|(_, r)| r),
        )?;
        ensure(back == data, || {
            "a grant's reading is not the object".into()
        })?;
        let mut changed = Vec::new();
        let mut key = grant.clone();
        key.key.push_str("-other");
        changed.push(("its key", key));
        let mut method = grant.clone();
        method.access.method = Method::Put;
        changed.push(("its method", method));
        let mut length = grant.clone();
        length.access.length = length.access.length.saturating_add(1);
        changed.push(("its length", length));
        let mut expires = grant.clone();
        expires.expires = expiry(f)?
            .checked_add(f.patience())
            .ok_or_else(|| "an expiry past what the platform's time can say".to_owned())?;
        changed.push(("its expiry", expires));
        let mut holder = grant.clone();
        holder.holder = "another".into();
        changed.push(("its holder", holder));
        for (what, hostile) in &changed {
            let refusal = refused(
                &format!("a grant with {what} changed"),
                f.redeem(hostile, &hostile.holder, deadline(f)?).await,
            )?;
            ensure(matches!(refusal, Refusal::Forbidden(_)), || {
                format!("with {what} changed, refused {refusal:?}, not Forbidden")
            })?;
        }
        let refusal = refused(
            "a grant presented by another than its holder",
            f.redeem(&grant, "another", deadline(f)?).await,
        )?;
        ensure(matches!(refusal, Refusal::Forbidden(_)), || {
            format!("presented by another, refused {refusal:?}, not Forbidden")
        })?;
        let put = Access {
            method: Method::Put,
            length: meta.length,
        };
        let granted = fine(
            "a put's grant",
            f.bytes()
                .grant("b13-put", put, "holder", expiry(f)?, deadline(f)?)
                .await,
        )?;
        let Redeemed::Upload(upload) = fine(
            "a put's grant redeemed",
            f.redeem(&granted, "holder", deadline(f)?).await,
        )?
        else {
            return Err("a put's grant gave no upload".into());
        };
        fine(
            "a granted put",
            send_from(f, &upload, &data, 0, None).await?,
        )?;
        fine(
            "a granted put's finish",
            f.bytes()
                .finish(&upload.id, whole(&data), deadline(f)?)
                .await,
        )?;
        f.pass(f.patience())?;
        same(
            "a grant past its expiry",
            f.redeem(&grant, "holder", deadline(f)?).await.map(drop),
            Err(Refusal::Expired),
        )
    })
}

fn b14<F: BytesFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        let data = content("b14", f.length())?;
        let length = count(data.len());
        let access = Access {
            method: Method::Put,
            length,
        };
        let grant = fine(
            "a put's grant",
            f.bytes()
                .grant("b14", access, "holder", expiry(f)?, deadline(f)?)
                .await,
        )?;
        let Redeemed::Upload(upload) = fine(
            "a put's grant redeemed",
            f.redeem(&grant, "holder", deadline(f)?).await,
        )?
        else {
            return Err("a put's grant gave no upload".into());
        };
        let short = data
            .get(..data.len().saturating_sub(1))
            .ok_or_else(|| "an object shorter than a byte".to_owned())?;
        fine("a short body", send_from(f, &upload, short, 0, None).await?)?;
        let refusal = refused(
            "a granted put's finish one byte short",
            f.bytes()
                .finish(&upload.id, whole(short), deadline(f)?)
                .await,
        )?;
        ensure(matches!(refusal, Refusal::Mismatch(_)), || {
            format!("refused {refusal:?}, not Mismatch")
        })?;
        same(
            "the key after it",
            f.bytes().head("b14", deadline(f)?).await,
            Err(Refusal::NotFound("b14".into())),
        )?;
        fine(
            "the last byte",
            send_from(f, &upload, &data, length.saturating_sub(1), None).await?,
        )?;
        let refusal = refused(
            "a byte past a granted put's length",
            f.bytes()
                .send(&upload.id, length, &data, deadline(f)?)
                .await,
        )?;
        ensure(matches!(refusal, Refusal::TooLarge { .. }), || {
            format!("refused {refusal:?}, not TooLarge")
        })?;
        fine(
            "the finish",
            f.bytes()
                .finish(&upload.id, whole(&data), deadline(f)?)
                .await,
        )
        .map(drop)
    })
}

fn b15<F: BytesFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        let data = content("b15", f.length())?;
        let patience = f.patience();
        let past = patience
            .checked_add(patience)
            .ok_or_else(|| "a patience as long as time".to_owned())?;
        // Applied, its answer held past the deadline.
        let held = staged(f, "b15", &data, false).await?;
        f.arm(Op::Finish, Fault::Delay(past))?;
        let due = deadline(f)?;
        same(
            "a finish whose answer comes after the deadline",
            f.bytes().finish(&held.id, whole(&data), due).await,
            Err(Refusal::Late),
        )?;
        ensure(f.clock().now() >= due, || {
            "told late before its deadline".into()
        })?;
        same(
            "the object, read after",
            fine("a head", f.bytes().head("b15", deadline(f)?).await)?.sha256,
            Sha256::of(&data),
        )?;
        same(
            "the finish asked again",
            f.bytes()
                .finish(&held.id, whole(&data), deadline(f)?)
                .await
                .map(drop),
            Err(Refusal::NoUpload(held.id.0.clone())),
        )?;
        // Never arrived: nothing applied, the upload still open.
        let vanished = staged(f, "b15-vanished", &data, false).await?;
        f.arm(Op::Finish, Fault::Vanish)?;
        same(
            "a finish that never arrives",
            f.bytes()
                .finish(&vanished.id, whole(&data), deadline(f)?)
                .await
                .map(drop),
            Err(Refusal::Late),
        )?;
        same(
            "the key, read after",
            f.bytes().head("b15-vanished", deadline(f)?).await,
            Err(Refusal::NotFound("b15-vanished".into())),
        )?;
        fine(
            "the finish asked again",
            f.bytes()
                .finish(&vanished.id, whole(&data), deadline(f)?)
                .await,
        )?;
        // Held, but within the caller's patience: answered.
        f.arm(Op::Head, Fault::Delay(patience))?;
        fine(
            "a head answered at its deadline",
            f.bytes().head("b15", deadline(f)?).await,
        )
        .map(drop)
    })
}

fn b16<F: BytesFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        let data = content("b16", f.length())?;
        let put = put_of("b16", &data, false);
        f.arm(Op::Begin, Fault::Lose)?;
        same(
            "a begin whose answer is lost",
            f.bytes()
                .begin("b16", put, deadline(f)?)
                .await
                .map(|u| u.id),
            Err(Refusal::Lost),
        )?;
        let upload = fine(
            "the begin asked again",
            f.bytes().begin("b16", put, deadline(f)?).await,
        )?;
        let again = fine(
            "the begin asked a third time",
            f.bytes().begin("b16", put, deadline(f)?).await,
        )?;
        same("the upload asked again", again.id, upload.id.clone())?;
        fine("a send", send_from(f, &upload, &data, 0, None).await?)?;
        f.arm(Op::Finish, Fault::Lose)?;
        same(
            "a finish whose answer is lost",
            f.bytes()
                .finish(&upload.id, whole(&data), deadline(f)?)
                .await
                .map(drop),
            Err(Refusal::Lost),
        )?;
        same(
            "the object, read after",
            fine("a head", f.bytes().head("b16", deadline(f)?).await)?.sha256,
            Sha256::of(&data),
        )?;
        same(
            "the finish asked again",
            f.bytes()
                .finish(&upload.id, whole(&data), deadline(f)?)
                .await
                .map(drop),
            Err(Refusal::NoUpload(upload.id.0.clone())),
        )?;
        f.arm(Op::Delete, Fault::Lose)?;
        same(
            "a delete whose answer is lost",
            f.bytes().delete("b16", deadline(f)?).await,
            Err(Refusal::Lost),
        )?;
        same(
            "the delete asked again",
            f.bytes().delete("b16", deadline(f)?).await,
            Ok(Deleted::Absent),
        )
    })
}

/// One refusal of every kind the store gives.
fn every_refusal<F: BytesFixture>(f: &F) -> Vec<Refusal> {
    let n = f.length();
    vec![
        Refusal::NotFound("k".into()),
        Refusal::NoUpload("u".into()),
        Refusal::Exists("k".into()),
        Refusal::Offset { expected: n },
        Refusal::Mismatch("m".into()),
        Refusal::Unsatisfiable { length: n },
        Refusal::Invalid {
            what: "a key",
            why: "w".into(),
        },
        Refusal::TooLarge {
            what: "a piece",
            size: n,
            limit: n,
        },
        Refusal::Full {
            what: "bytes",
            limit: n,
        },
        Refusal::Busy {
            retry_after: Some(f.patience()),
        },
        Refusal::Forbidden("f".into()),
        Refusal::Expired,
        Refusal::Cut { at: n },
        Refusal::Late,
        Refusal::Lost,
        Refusal::Unavailable("u".into()),
    ]
}

fn b17<F: BytesFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        for refusal in every_refusal(f) {
            let before = f.attempts(Op::Head);
            f.arm(Op::Head, Fault::Refuse(refusal.clone()))?;
            same(
                "a head the store refuses",
                f.bytes().head("b17", deadline(f)?).await,
                Err(refusal.clone()),
            )?;
            same(
                &format!("attempts of a head refused {refusal:?}"),
                f.attempts(Op::Head),
                before.saturating_add(1),
            )?;
        }
        Ok(())
    })
}

fn b18<F: BytesFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        let data = content("b18", f.length())?;
        let (meta, plan) = put(f, "b18", &data, false).await?;
        for cut in 0..=meta.length {
            f.arm(Op::Read, Fault::Cut(cut))?;
            let mut reading = fine("a get", f.bytes().get("b18", None, deadline(f)?).await)?;
            let (got, refusal) = match read_all(f, &mut reading, plan.piece).await? {
                Err(ended) => ended,
                Ok(_) => return Err(format!("a reading cut at {cut} was not cut")),
            };
            same("its refusal", refusal, Refusal::Cut { at: cut })?;
            let before = usize::try_from(cut).map_err(|e| e.to_string())?;
            ensure(data.get(..before) == Some(got.as_slice()), || {
                format!("cut at {cut}, the bytes before it are not the object's")
            })?;
            if cut < meta.length {
                let rest = Range::From {
                    first: cut,
                    last: None,
                };
                let (_, tail) = read(f, "b18", Some(rest), plan.piece).await?;
                ensure(data.get(before..) == Some(tail.as_slice()), || {
                    format!("cut at {cut}, the bytes from it are not the object's")
                })?;
            }
        }
        Ok(())
    })
}

fn b19<F: BytesFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        let data = content("b19", f.length())?;
        let upload = staged(f, "b19", &data, false).await?;
        fine("an abort", f.bytes().abort(&upload.id, deadline(f)?).await)?;
        let gone = Refusal::NoUpload(upload.id.0.clone());
        same(
            "a send to an upload given up",
            f.bytes()
                .send(&upload.id, upload.progress.received, &data, deadline(f)?)
                .await,
            Err(gone.clone()),
        )?;
        same(
            "a finish of an upload given up",
            f.bytes()
                .finish(&upload.id, whole(&data), deadline(f)?)
                .await
                .map(drop),
            Err(gone),
        )?;
        same(
            "the key after it",
            f.bytes().head("b19", deadline(f)?).await,
            Err(Refusal::NotFound("b19".into())),
        )
    })
}

fn b20<F: BytesFixture>(f: &F) -> BoxFuture<'_, Result<(), Failure>> {
    Box::pin(async move {
        let mut open = Vec::new();
        for n in 0..f.uploads_max() {
            let key = format!("b20-{n}");
            let upload = fine(
                "a begin within the bound",
                f.bytes()
                    .begin(&key, put_of(&key, &[], false), deadline(f)?)
                    .await,
            )?;
            open.push(upload);
        }
        let refusal = refused(
            "a begin past the most uploads open",
            f.bytes()
                .begin("b20", put_of("b20", &[], false), deadline(f)?)
                .await
                .map(|u| u.id),
        )?;
        ensure(matches!(refusal, Refusal::Full { .. }), || {
            format!("refused {refusal:?}, not Full")
        })?;
        for upload in &open {
            fine(
                "an upload open within the bound",
                f.bytes().status(&upload.id, deadline(f)?).await,
            )?;
        }
        let first = open
            .first()
            .ok_or_else(|| "a store that holds no upload open".to_owned())?;
        fine("an abort", f.bytes().abort(&first.id, deadline(f)?).await)?;
        fine(
            "a begin once one was given up",
            f.bytes()
                .begin("b20", put_of("b20", &[], false), deadline(f)?)
                .await,
        )
        .map(drop)
    })
}

/// The `Bytes` contract, clause by clause (`docs/design/seams.md`, "Bytes").
#[must_use]
pub fn cases<F: BytesFixture>() -> Vec<Case<F>> {
    vec![
        Case {
            id: "BYTES-1",
            says: "a put sent in pieces and finished whole is read back the same, and head says its length and SHA-256",
            check: b1::<F>,
        },
        Case {
            id: "BYTES-2",
            says: "a piece larger than the upload's plan is refused TooLarge, and the upload is as it was",
            check: b2::<F>,
        },
        Case {
            id: "BYTES-3",
            says: "a send is taken only at the upload's next byte; one at any other is refused Offset, naming the next",
            check: b3::<F>,
        },
        Case {
            id: "BYTES-4",
            says: "a put cut at any byte keeps what was durable and nothing after it, within one run of the cut; sent on from there it finishes whole, and no byte before the durable offset is sent twice",
            check: b4::<F>,
        },
        Case {
            id: "BYTES-5",
            says: "a finish whose length or SHA-256 is not what the upload received is refused Mismatch: nothing appears, and the upload stays open",
            check: b5::<F>,
        },
        Case {
            id: "BYTES-6",
            says: "create-once on a key that holds an object is refused Exists, at begin and at finish, and the object there is unchanged",
            check: b6::<F>,
        },
        Case {
            id: "BYTES-7",
            says: "of two create-once puts of one key finishing at once, exactly one wins; the other is refused Exists",
            check: b7::<F>,
        },
        Case {
            id: "BYTES-8",
            says: "an object appears only whole, at its finish: until then its key shows what was there before",
            check: b8::<F>,
        },
        Case {
            id: "BYTES-9",
            says: "a range is answered as RFC 9110 has it: the first byte, the last, and a suffix; an end past the object stops at its last byte; a start at or past the end, or a zero suffix, is refused Unsatisfiable with the object's length; a last byte before the first is refused Invalid; of an empty object only a suffix selects, and it selects nothing",
            check: b9::<F>,
        },
        Case {
            id: "BYTES-10",
            says: "a reading fills its caller's buffer and no more, whatever its size: an object is never in memory whole",
            check: b10::<F>,
        },
        Case {
            id: "BYTES-11",
            says: "a get or head of a key with no object is refused NotFound; a delete removes an object, and of a key with none is answered Absent",
            check: b11::<F>,
        },
        Case {
            id: "BYTES-12",
            says: "a reading begun before a delete or an overwrite reads the object it began with, whole",
            check: b12::<F>,
        },
        Case {
            id: "BYTES-13",
            says: "a grant lets its holder alone get, or put exactly its length, at its key until it expires: with its key, method, length, expiry or holder changed it is refused Forbidden, and past its expiry Expired",
            check: b13::<F>,
        },
        Case {
            id: "BYTES-14",
            says: "a granted put of another length than its grant's is refused, and nothing appears",
            check: b14::<F>,
        },
        Case {
            id: "BYTES-15",
            says: "an answer delayed past its caller's deadline is refused Late at the deadline, and what the call did is learned by reading",
            check: b15::<F>,
        },
        Case {
            id: "BYTES-16",
            says: "an answer lost after its call applied is refused Lost; reading shows what it did, and asking again never applies it twice",
            check: b16::<F>,
        },
        Case {
            id: "BYTES-17",
            says: "every refusal the store gives reaches its caller as it is, after one attempt: the seam never asks again",
            check: b17::<F>,
        },
        Case {
            id: "BYTES-18",
            says: "a reading cut at any byte gives every byte before the cut, then is refused Cut there; a reading from there gives the rest",
            check: b18::<F>,
        },
        Case {
            id: "BYTES-19",
            says: "an upload given up is gone: a send or a finish to it is refused NoUpload, and nothing appears",
            check: b19::<F>,
        },
        Case {
            id: "BYTES-20",
            says: "the uploads a store holds open are bounded: past the most, a begin is refused Full, and none open is dropped",
            check: b20::<F>,
        },
    ]
}
