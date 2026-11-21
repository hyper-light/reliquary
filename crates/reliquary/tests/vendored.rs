//! What Reliquary vendors (vendor/MANIFEST.toml; plan RQ0.1): each project at the revision kiln vendors
//! it with kiln's recorded changes (AC-RQ0.1.2), one revision of each shared crate linked (AC-RQ0.1.3),
//! each exception to the -api rule a line of the manifest (AC-RQ0.1.5), each crate Reliquary names
//! running here with an answer its specification fixes (AC-RQ0.1.4: it builds and runs on this target),
//! and kiln's vendoring tool refusing what it must on Reliquary's manifest (T-RQ0.1.1, T-RQ0.1.2,
//! T-RQ0.1.5).

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
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn manifest(root: &Path) -> toml::Table {
    let text =
        std::fs::read_to_string(root.join("vendor/MANIFEST.toml")).expect("vendor/MANIFEST.toml");
    text.parse().expect("vendor/MANIFEST.toml parses")
}

fn projects(manifest: &toml::Table) -> &toml::Table {
    manifest
        .get("projects")
        .and_then(toml::Value::as_table)
        .expect("[projects]")
}

fn text<'a>(table: &'a toml::Value, key: &str) -> Option<&'a str> {
    table.get(key).and_then(toml::Value::as_str)
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// What this runner lacks to run `program` so: nothing when it runs and succeeds; else what it said,
/// for the skip record (crates/reliquary-testkit).
fn lacks(program: &str, args: &[&str]) -> Option<String> {
    match Command::new(program).args(args).output() {
        Err(e) => Some(format!("{program} doesn't run here: {e}")),
        Ok(out) if !out.status.success() => Some(format!(
            "{program} {}: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        )),
        Ok(_) => None,
    }
}

/// What a runner lacks for a test, and what it said: the skip record's `missing` and `why`.
type Lacking = (&'static str, String);

const GIT: &str = "git";
const PYTHON: &str = "python3 3.11 or later (kiln's scripts/vendor.py reads TOML with tomllib)";
const NETWORK: &str = "the network (kiln's repository on GitHub)";

/// A command's output, or the test fails saying what it ran and what it printed.
fn ran(command: &mut Command) -> Output {
    let out = command
        .output()
        .unwrap_or_else(|e| panic!("{command:?} didn't run: {e}"));
    assert!(
        out.status.success(),
        "{command:?} failed ({}):\n{}{}",
        out.status,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    out
}

// ---------------------------------------------------------------------------------------------------
// Each crate Reliquary names runs here, with an answer its specification or its own contract fixes
// (AC-RQ0.1.4 on the target the tests run on; the gate checks the others).

/// Compute the CRC catalogue's check value of "123456789" with mantle-crc → CRC-32C's, CRC-32's and
/// CRC-64/NVME's published values (AC-RQ0.1.4).
#[test]
fn t_rq0_1_mantle_crc_gives_the_catalogues_check_values() {
    assert_eq!(mantle_crc::crc32c(b"123456789"), 0xE306_9283);
    assert_eq!(mantle_crc::crc32(b"123456789"), 0xCBF4_3926);
    assert_eq!(mantle_crc::crc64nvme(b"123456789"), 0xAE8B_1486_0A79_9888);
}

/// Parse an upload's request head with mantle-http → its method, target and length, the body where the
/// head ends; a head with whitespace before a field's colon → refused 400, as RFC 9112 §5.1 says
/// (AC-RQ0.1.4).
#[test]
fn t_rq0_1_mantle_http_parses_a_head_and_refuses_what_rfc_9112_refuses() {
    let request =
        b"PUT /v2/agents/blobs/uploads/1 HTTP/1.1\r\nHost: r\r\nContent-Length: 3\r\n\r\nabc";
    let head = mantle_http::parse(request).unwrap().expect("a whole head");
    assert_eq!(head.method(request), b"PUT");
    assert_eq!(head.origin(request), b"/v2/agents/blobs/uploads/1");
    assert_eq!(head.framing(), mantle_http::Framing::Length(3));
    assert_eq!(&request[head.len()..], b"abc");
    let refused = mantle_http::parse(b"GET / HTTP/1.1\r\nHost : r\r\n\r\n").unwrap_err();
    assert_eq!(refused.status(), 400);
}

/// Name a client by its id and read the id back with mantle-protocol → the same id; a second spelling of
/// it → none (AC-RQ0.1.4).
#[test]
fn t_rq0_1_mantle_protocol_names_a_client_one_way() {
    assert_eq!(
        mantle_protocol::client_id(&mantle_protocol::client_name(7)),
        Some(7)
    );
    assert_eq!(mantle_protocol::client_id("client-07"), None);
    assert_eq!(mantle_protocol::node_name(3), "node-3");
}

/// Size a client of three nodes with mantle-client → a least transport, and a memory bound that holds
/// it, as its contract says (its transport's budget and what it holds beside it) (AC-RQ0.1.4).
#[test]
fn t_rq0_1_mantle_client_bounds_its_memory_by_its_transport() {
    let least = mantle_client::least_transport(3).unwrap();
    let memory = mantle_client::memory(&mantle_client::stated(3, least)).expect("a bound");
    assert!(memory > least, "{memory} <= {least}");
}

/// Fold two round trips into hyper-timing's estimator → RFC 9002 §5.3's smoothed round trip and mean
/// deviation; before any, no tail (AC-RQ0.1.4).
#[test]
fn t_rq0_1_hyper_timing_follows_rfc_9002() {
    let mut rtt = hyper_timing::ExchangeRtt::new();
    assert_eq!(rtt.tail_ns(1), None);
    rtt.on_sample(100_000);
    assert_eq!((rtt.smoothed_ns(), rtt.variation_ns()), (100_000, 50_000));
    rtt.on_sample(200_000);
    // rttvar = 3/4 · 50,000 + 1/4 · |100,000 − 200,000|; smoothed = 7/8 · 100,000 + 1/8 · 200,000.
    assert_eq!((rtt.smoothed_ns(), rtt.variation_ns()), (112_500, 62_500));
}

/// Draw from hyper-sim's generator seeded 0 → SplitMix64's first two outputs; a zero bound draws
/// nothing (AC-RQ0.1.4).
#[test]
fn t_rq0_1_hyper_sim_draws_splitmix64() {
    let mut rng = hyper_sim::Seeded::new(0);
    assert_eq!(rng.next_u64(), 0xE220_A839_7B1D_CDAF);
    assert_eq!(rng.below(0), 0);
    assert_eq!(rng.next_u64(), 0x6E78_9E6A_A1B9_65F4);
}

/// Ask hyper-measure how many samples put a median's upper end at 95% coverage → 6: the least n with
/// 0.5ⁿ ≤ 0.025 (AC-RQ0.1.4).
#[test]
fn t_rq0_1_hyper_measure_counts_the_samples_a_quantile_needs() {
    use hyper_measure::quantile::{Level, samples_for};
    assert_eq!(
        samples_for(Level::P50, Level::new(95, 100).unwrap()).unwrap(),
        6
    );
}

/// Run a future on a hyper-rt shard → its output (AC-RQ0.1.4).
#[test]
fn t_rq0_1_hyper_rt_runs_a_future_on_a_shard() {
    // A test harness's fixed quanta, as hyper-rt's own tests state them (RuntimeConfig's doc).
    let config = hyper_rt::runtime::RuntimeConfig {
        shards: 1,
        tasks_per_shard: 64,
        timers_per_shard: 64,
        interests_per_shard: 64,
        ring_entries: 64,
        step_budget_ns: 1_000_000_000,
        timer_tick_ns: 100_000,
        batch: 64,
        pin: false,
        cores: Vec::new(),
        page_bytes: 4096,
        spin_ns: 0,
        wake_tracking: None,
        latency_objective_ns: None,
    };
    let mut runtime = hyper_rt::runtime::LocalRuntime::new(&config).unwrap();
    assert_eq!(runtime.block_on(async { 6 * 7 }).unwrap(), 42);
}

/// Size what a hyper-rt-transport driver holds as it is made, for a one-datagram batch and for the most
/// one `recvmmsg(2)` takes (`UIO_MAXIOV`, 1,024) → at least one receive buffer of 65,536 bytes
/// (`GRO_LEGACY_MAX_SIZE`), which takes the largest UDP payload whole: 65,527 bytes over IPv6 (RFC 768,
/// RFC 8200) (AC-RQ0.1.4).
#[test]
fn t_rq0_1_hyper_rt_transport_holds_the_largest_datagram_from_the_start() {
    assert_eq!(hyper_rt_transport::MAX_BATCH, 1_024);
    assert_eq!(hyper_rt_transport::RECEIVE_BYTES, 65_536);
    for batch in [1, hyper_rt_transport::MAX_BATCH] {
        let held = hyper_rt_transport::least_bytes(hyper_rt_transport::Io { batch });
        assert!(held >= 65_536, "batch {batch}: {held} bytes");
    }
}

/// Configure hyper-tls for TLS 1.3 with AES-256-GCM alone, the house's one suite → accepted; with a TLS
/// 1.2 suite alone → refused, no suite usable (AC-RQ0.1.4).
#[test]
fn t_rq0_1_hyper_tls_takes_tls_1_3_with_aes_256_gcm_alone() {
    use hyper_tls::crypto::{CryptoProvider, aws_lc_rs};
    use std::sync::LazyLock;
    // hyper-tls borrows its provider for the process's life.
    static AES_256_GCM: LazyLock<CryptoProvider> = LazyLock::new(|| CryptoProvider {
        cipher_suites: vec![aws_lc_rs::cipher_suite::TLS13_AES_256_GCM_SHA384],
        ..aws_lc_rs::default_provider()
    });
    static TLS12_ALONE: LazyLock<CryptoProvider> = LazyLock::new(|| CryptoProvider {
        cipher_suites: vec![aws_lc_rs::cipher_suite::TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384],
        ..aws_lc_rs::default_provider()
    });
    let tls13 = hyper_tls::ClientConfig::builder_with_provider(&AES_256_GCM)
        .with_protocol_versions(&[&hyper_tls::version::TLS13]);
    assert!(tls13.is_ok(), "{:?}", tls13.err());
    let tls12 = hyper_tls::ClientConfig::builder_with_provider(&TLS12_ALONE)
        .with_protocol_versions(&[&hyper_tls::version::TLS13]);
    assert!(
        tls12.is_err(),
        "a TLS 1.3 configuration took a TLS 1.2 suite alone"
    );
}

/// Encode and decode RFC 9000 Appendix A.1's sample variable-length integers with hyper-quic → its
/// bytes and values; 2⁶² → refused (AC-RQ0.1.4).
#[test]
fn t_rq0_1_hyper_quic_codes_rfc_9000s_varints() {
    use hyper_quic::VarInt;
    use hyper_quic::coding::Codec;
    let samples: [(u64, &[u8]); 4] = [
        (
            151_288_809_941_952_652,
            &[0xc2, 0x19, 0x7c, 0x5e, 0xff, 0x14, 0xe8, 0x8c],
        ),
        (494_878_333, &[0x9d, 0x7f, 0x3e, 0x7d]),
        (15_293, &[0x7b, 0xbd]),
        (37, &[0x25]),
    ];
    for (value, wire) in samples {
        let mut out = Vec::new();
        VarInt::from_u64(value).unwrap().encode(&mut out);
        assert_eq!(out, wire, "{value}");
        let mut read = wire;
        assert_eq!(VarInt::decode(&mut read).unwrap().into_inner(), value);
    }
    let mut longer: &[u8] = &[0x40, 0x25];
    assert_eq!(VarInt::decode(&mut longer).unwrap().into_inner(), 37);
    assert!(VarInt::from_u64(1 << 62).is_err());
}

/// Spread a 10 ms pause with hyper-transport by a draw of 0 and of 2⁶³ → 5 ms and 7.5 ms: equal jitter
/// over the pause's second half (AC-RQ0.1.4).
#[test]
fn t_rq0_1_hyper_transport_spreads_a_pause_over_its_second_half() {
    use std::time::Duration;
    let pause = Duration::from_millis(10);
    assert_eq!(hyper_transport::spread(pause, 0), Duration::from_millis(5));
    assert_eq!(
        hyper_transport::spread(pause, 1 << 63),
        Duration::from_micros(7_500)
    );
}

/// Check a transaction with amphora-api: a condition and a value at its bound → accepted; a value one
/// byte past it → refused TooLarge, naming the value and the bound (AC-RQ0.1.4).
#[test]
fn t_rq0_1_amphora_api_holds_a_transaction_to_its_bounds() {
    use amphora_api::{Cond, Refusal, Tx, VALUE_MAX, Write};
    let tx = |len: usize| Tx {
        conds: vec![Cond::Absent {
            family: "blobs".into(),
            key: b"sha256:ab".to_vec(),
        }],
        writes: vec![Write::Put {
            family: "manifests".into(),
            key: b"sha256:cd".to_vec(),
            value: vec![0; len],
        }],
    };
    assert_eq!(amphora_api::transaction(&tx(VALUE_MAX)), Ok(()));
    assert_eq!(
        amphora_api::transaction(&tx(VALUE_MAX + 1)),
        Err(Refusal::TooLarge {
            what: "value".into(),
            size: VALUE_MAX + 1,
            limit: VALUE_MAX
        })
    );
}

/// Open Amphora's embedded store, commit a tag's row only while its key is absent, then the same again →
/// the first commits and reads back at its revision; the second is refused Conflict on the tag's family:
/// a row is taken only when its condition holds (architecture D2), on kiln's hyper-raft (vendor/amphora)
/// (AC-RQ0.1.4).
#[test]
fn t_rq0_1_amphora_store_commits_a_row_only_while_its_key_is_absent() {
    use amphora_api::{Cond, Consistency, Records, Refusal, Tx, Write};
    use std::time::{Duration, Instant};
    // The bound Amphora's own tests give each call, which returns when its work is done.
    let until = || Instant::now() + Duration::from_secs(30);
    let dir = tempfile::tempdir().unwrap();
    let database =
        amphora::database::Database::open(&dir.path().join("records.db"), until()).unwrap();
    let records = database.namespace("registry").unwrap();
    let tag = || Tx {
        conds: vec![Cond::Absent {
            family: "tags".into(),
            key: b"agents/a:v1".to_vec(),
        }],
        writes: vec![Write::Put {
            family: "tags".into(),
            key: b"agents/a:v1".to_vec(),
            value: b"sha256:ab".to_vec(),
        }],
    };
    let rev = records.commit(tag(), until()).unwrap();
    let row = records
        .get("tags", b"agents/a:v1", Consistency::Linearizable, until())
        .unwrap()
        .expect("the tag's row");
    assert_eq!((row.value.as_slice(), row.rev), (&b"sha256:ab"[..], rev));
    let again = records.commit(tag(), until());
    assert!(
        matches!(&again, Err(Refusal::Conflict { family, .. }) if family == "tags"),
        "{again:?}"
    );
}

/// Digest all but the last block of FIPS 180-2's million "a"s with aws-lc-rs, save its state, resume it
/// and digest the rest → the standard's SHA-256 of them; a state asked for mid-block → refused (a state is
/// whole only between blocks). Mantle's change, which kiln records, is in the copy linked (AC-RQ0.1.4).
#[test]
fn t_rq0_1_aws_lc_rs_resumes_a_saved_digest() {
    use aws_lc_rs::digest::{Context, SHA256};
    let million = vec![b'a'; 1_000_000];
    let (head, tail) = million.split_at(1_000_000 - 64);
    let mut first = Context::new(&SHA256);
    first.update(head);
    let mut resumed = Context::resume(&first.state().unwrap()).unwrap();
    resumed.update(tail);
    assert_eq!(
        hex(resumed.finish().as_ref()),
        "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0"
    );
    let mut mid_block = Context::new(&SHA256);
    mid_block.update(b"ab");
    assert!(mid_block.state().is_err(), "a state taken mid-block");
}

// ---------------------------------------------------------------------------------------------------
// AC-RQ0.1.2: each project at kiln's revision, with kiln's recorded changes.

/// A project's recorded changes: file name → bytes.
type Changes = BTreeMap<String, Vec<u8>>;

fn kiln_only(patch: &[u8]) -> bool {
    String::from_utf8_lossy(patch)
        .lines()
        .take_while(|l| !l.starts_with("diff --git "))
        .any(|l| l.starts_with("Kiln-only: "))
}

/// Where Reliquary's pins differ from kiln's: a project kiln holds at another revision or from another
/// repository, or with other recorded changes (kiln's layout's alone aside); a project kiln doesn't hold
/// without `kiln_holds_none` saying why; one kiln holds that still says it doesn't.
fn pin_problems(
    ours: &toml::Table,
    our_changes: &BTreeMap<String, Changes>,
    kilns: &toml::Table,
    kiln_changes: &BTreeMap<String, Changes>,
) -> Vec<String> {
    let mut out = Vec::new();
    let empty = Changes::new();
    for (name, spec) in projects(ours) {
        let our_commit = text(spec, "commit").unwrap_or("?");
        let holds_none = text(spec, "kiln_holds_none");
        match projects(kilns).get(name) {
            Some(kiln) => {
                let kiln_commit = text(kiln, "commit").unwrap_or("?");
                if our_commit != kiln_commit {
                    out.push(format!("{name}: Reliquary takes it at {our_commit}, kiln at {kiln_commit}: the pins must agree"));
                }
                if text(spec, "repository") != text(kiln, "repository") {
                    out.push(format!("{name}: Reliquary takes it from {:?}, kiln from {:?}", text(spec, "repository"), text(kiln, "repository")));
                }
                if holds_none.is_some() {
                    out.push(format!("{name}: kiln holds it now, at {kiln_commit}: kiln_holds_none goes"));
                }
                let ours = our_changes.get(name).unwrap_or(&empty);
                let theirs: Changes = kiln_changes.get(name).unwrap_or(&empty).iter().filter(|(_, p)| !kiln_only(p)).map(|(n, p)| (n.clone(), p.clone())).collect();
                let differ: Vec<&String> = ours.keys().chain(theirs.keys()).collect::<BTreeSet<_>>().into_iter().filter(|n| ours.get(*n) != theirs.get(*n)).collect();
                if !differ.is_empty() {
                    out.push(format!("{name}: Reliquary records other changes than kiln does: {differ:?}"));
                }
            }
            None if holds_none.is_some_and(|why| !why.trim().is_empty()) => {}
            None => out.push(format!("{name}: kiln holds no {name}: take it at kiln's revision, or say why with kiln_holds_none")),
        }
    }
    out
}

fn our_changes(root: &Path, names: impl Iterator<Item = String>) -> BTreeMap<String, Changes> {
    let mut out = BTreeMap::new();
    for name in names {
        let mut changes = Changes::new();
        if let Ok(entries) = std::fs::read_dir(root.join("vendor/patches").join(&name)) {
            for entry in entries {
                let entry = entry.unwrap();
                changes.insert(
                    entry.file_name().to_string_lossy().into_owned(),
                    std::fs::read(entry.path()).unwrap(),
                );
            }
        }
        out.insert(name, changes);
    }
    out
}

/// git, never asking for credentials, in a repository of its own under kiln's vendor cache, where
/// scripts/vendor.py keeps its upstreams (`$KILN_VENDOR_CACHE`, else `~/.cache/kiln/vendor`).
fn kiln_git(bare: &Path, args: &[&str]) -> Command {
    let mut command = Command::new("git");
    command
        .arg("--git-dir")
        .arg(bare)
        .args(args)
        .env("GIT_TERMINAL_PROMPT", "0");
    command
}

/// kiln's manifest at a commit, and its recorded changes by project.
type Kilns = (toml::Table, BTreeMap<String, Changes>);

/// curl's words, through git, for a server it couldn't reach: the network lacking, not a wrong pin.
const UNREACHABLE: [&str; 8] = [
    "Could not resolve host",
    "Could not resolve proxy",
    "Failed to connect to",
    "Couldn't connect to server",
    "Connection timed out",
    "Connection refused",
    "Network is unreachable",
    "Operation timed out",
];

/// kiln's manifest and recorded changes at `commit`: fetched once, the commit's tree without its files
/// (a partial clone), each file read as it is asked for. Without git, or the network to fetch with,
/// what's lacking and what was said, for the skip record; any other failure fails the test.
fn kiln_at(repository: &str, commit: &str, names: &BTreeSet<String>) -> Result<Kilns, Lacking> {
    if let Some(why) = lacks("git", &["--version"]) {
        return Err((GIT, why));
    }
    // Where scripts/vendor.py keeps it: Python's home is the user's profile on Windows, HOME elsewhere.
    let home = if cfg!(windows) { "USERPROFILE" } else { "HOME" };
    let cache = std::env::var_os("KILN_VENDOR_CACHE")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os(home).map(|h| Path::new(&h).join(".cache/kiln/vendor")))
        .unwrap_or_else(|| panic!("KILN_VENDOR_CACHE or {home}"));
    let bare = cache.join("git").join("kiln-pins.git");
    if !bare.join("HEAD").is_file() {
        std::fs::create_dir_all(&bare).unwrap();
        ran(Command::new("git")
            .args(["init", "-q", "--bare"])
            .arg(&bare));
    }
    ran(&mut kiln_git(
        &bare,
        &["config", "remote.origin.url", repository],
    ));
    ran(&mut kiln_git(
        &bare,
        &["config", "remote.origin.promisor", "true"],
    ));
    ran(&mut kiln_git(
        &bare,
        &["config", "remote.origin.partialclonefilter", "blob:none"],
    ));
    let have = kiln_git(&bare, &["cat-file", "-e", &format!("{commit}^{{commit}}")])
        .output()
        .unwrap();
    if !have.status.success() {
        let fetch = kiln_git(
            &bare,
            &[
                "fetch",
                "-q",
                "--depth",
                "1",
                "--filter=blob:none",
                "origin",
                commit,
            ],
        )
        .output()
        .unwrap();
        let said = String::from_utf8_lossy(&fetch.stderr).trim().to_owned();
        if !fetch.status.success() && UNREACHABLE.iter().any(|s| said.contains(s)) {
            return Err((NETWORK, said));
        }
        assert!(
            fetch.status.success(),
            "fetching {commit} from {repository} failed: {said}"
        );
    }
    let show =
        |path: &str| ran(&mut kiln_git(&bare, &["show", &format!("{commit}:{path}")])).stdout;
    let manifest = String::from_utf8(show("vendor/MANIFEST.toml"))
        .unwrap()
        .parse()
        .unwrap();
    let mut changes = BTreeMap::new();
    for name in names {
        let listed = ran(&mut kiln_git(
            &bare,
            &[
                "ls-tree",
                "-r",
                "--name-only",
                commit,
                "--",
                &format!("vendor/patches/{name}"),
            ],
        ))
        .stdout;
        let mut these = Changes::new();
        for path in String::from_utf8(listed).unwrap().lines() {
            let file = path.rsplit('/').next().unwrap().to_owned();
            these.insert(file, show(path));
        }
        changes.insert(name.clone(), these);
    }
    Ok((manifest, changes))
}

/// Read kiln's manifest and changes at the commit `[kiln]` names → every project Reliquary vendors is at
/// kiln's revision with kiln's changes, or says why kiln holds none (AC-RQ0.1.2). Needs the network once
/// per kiln commit, as `scripts/vendor.py check` does.
#[test]
fn ac_rq0_1_2_each_project_is_at_kilns_revision_with_kilns_changes() {
    let test = "ac_rq0_1_2_each_project_is_at_kilns_revision_with_kilns_changes";
    let root = root();
    let ours = manifest(&root);
    let kiln = ours.get("kiln").expect("[kiln] in vendor/MANIFEST.toml");
    let (repository, commit) = (
        text(kiln, "repository").expect("[kiln] repository"),
        text(kiln, "commit").expect("[kiln] commit"),
    );
    let names: BTreeSet<String> = projects(&ours).keys().cloned().collect();
    let fetched = kiln_at(repository, commit, &names);
    if let Err((missing, why)) = &fetched {
        return reliquary_testkit::skip(test, missing, why).unwrap();
    }
    let (kilns, kiln_changes) = fetched.unwrap();
    let found = pin_problems(
        &ours,
        &our_changes(&root, names.iter().cloned()),
        &kilns,
        &kiln_changes,
    );
    assert!(
        found.is_empty(),
        "Reliquary's pins differ from kiln's at {commit}:\n{}",
        found.join("\n")
    );
    // Non-vacuity: the projects kiln holds were compared with kiln's, changes and all.
    for name in ["hyper-raft", "mantle", "aws-lc-rs"] {
        assert!(
            projects(&kilns).contains_key(name),
            "kiln at {commit} holds no {name}"
        );
        assert!(
            !kiln_changes[name].is_empty(),
            "no changes of kiln's to {name} were read"
        );
    }
}

fn planted(projects: &str) -> toml::Table {
    format!("format = 1\n{projects}").parse().unwrap()
}

const KILN: &str = "[projects.hyper-raft]\nrepository = \"https://github.com/hyper-light/hyper-raft\"\ncommit = \"e0613f524b0df198cc61dc1e82b33c22d604cf7b\"\n";

/// Point hyper-raft at a revision other than kiln's → the pin check fails naming both (T-RQ0.1.3).
#[test]
fn t_rq0_1_3_a_project_at_another_revision_than_kilns_fails_naming_both() {
    let ours = planted(
        "[projects.hyper-raft]\nrepository = \"https://github.com/hyper-light/hyper-raft\"\ncommit = \"045cf3ff82e1b5c5d2c8c28e2b1e0f6d2c1a0b9e\"\n",
    );
    let found = pin_problems(&ours, &BTreeMap::new(), &planted(KILN), &BTreeMap::new());
    assert_eq!(
        found,
        [
            "hyper-raft: Reliquary takes it at 045cf3ff82e1b5c5d2c8c28e2b1e0f6d2c1a0b9e, kiln at e0613f524b0df198cc61dc1e82b33c22d604cf7b: the pins must agree"
        ]
    );
}

/// A recorded change kiln doesn't make, one kiln makes that Reliquary lacks, and kiln's layout alone →
/// the first two named, the third not asked of Reliquary (AC-RQ0.1.2).
#[test]
fn t_rq0_1_3_other_recorded_changes_than_kilns_fail_and_kilns_layout_alone_does_not() {
    let mut ours = BTreeMap::new();
    ours.insert(
        "hyper-raft".to_owned(),
        Changes::from([(
            "0009-ours.patch".to_owned(),
            b"Subject: ours\n\ndiff --git a/x b/x\n".to_vec(),
        )]),
    );
    let mut kilns = BTreeMap::new();
    kilns.insert(
        "hyper-raft".to_owned(),
        Changes::from([
            (
                "0001-kilns.patch".to_owned(),
                b"Subject: kiln's\nWhy: w\n\ndiff --git a/y b/y\n".to_vec(),
            ),
            (
                "0002-layout.patch".to_owned(),
                b"Subject: kiln-rt\nWhy: w\nKiln-only: kiln's own crate\n\ndiff --git a/z b/z\n"
                    .to_vec(),
            ),
        ]),
    );
    let found = pin_problems(&planted(KILN), &ours, &planted(KILN), &kilns);
    assert_eq!(
        found,
        [
            "hyper-raft: Reliquary records other changes than kiln does: [\"0001-kilns.patch\", \"0009-ours.patch\"]"
        ]
    );
}

/// A project kiln doesn't vendor, with and without `kiln_holds_none`; one kiln vendors that still says it
/// doesn't → refused unless it says why; told to drop the line (AC-RQ0.1.2).
#[test]
fn t_rq0_1_3_a_project_kiln_holds_none_of_needs_its_reason() {
    let amphora = "[projects.amphora]\nrepository = \"https://github.com/hyper-light/amphora\"\ncommit = \"db991a05d1769648d1de0390ac80b04c979d150e\"\n";
    let found = pin_problems(
        &planted(amphora),
        &BTreeMap::new(),
        &planted(KILN),
        &BTreeMap::new(),
    );
    assert_eq!(
        found,
        [
            "amphora: kiln holds no amphora: take it at kiln's revision, or say why with kiln_holds_none"
        ]
    );
    let said = format!("{amphora}kiln_holds_none = \"kiln vendors no Amphora yet\"\n");
    assert!(
        pin_problems(
            &planted(&said),
            &BTreeMap::new(),
            &planted(KILN),
            &BTreeMap::new()
        )
        .is_empty()
    );
    let stale = format!("{KILN}kiln_holds_none = \"no longer so\"\n");
    let found = pin_problems(
        &planted(&stale),
        &BTreeMap::new(),
        &planted(KILN),
        &BTreeMap::new(),
    );
    assert_eq!(
        found,
        [
            "hyper-raft: kiln holds it now, at e0613f524b0df198cc61dc1e82b33c22d604cf7b: kiln_holds_none goes"
        ]
    );
}

// ---------------------------------------------------------------------------------------------------
// AC-RQ0.1.3: one revision of each shared crate, as `cargo metadata` resolves the workspace.

/// A package as `cargo metadata` gives it.
#[derive(Clone, Debug)]
struct Package {
    name: String,
    version: String,
    source: Option<String>,
    manifest_path: PathBuf,
}

/// The workspace as `cargo metadata` resolves it for every platform at once (each target's dependencies,
/// `cfg(loom)`'s too), asked once for all the tests that read it. It reads every package's manifest, so a
/// package no build of this host downloaded is downloaded here.
fn metadata(root: &Path) -> &'static serde_json::Value {
    static METADATA: std::sync::OnceLock<serde_json::Value> = std::sync::OnceLock::new();
    METADATA.get_or_init(|| {
        let out = ran(Command::new(env!("CARGO")).current_dir(root).args([
            "metadata",
            "--format-version",
            "1",
            "--locked",
        ]));
        serde_json::from_slice(&out.stdout).expect("cargo metadata's JSON")
    })
}

fn packages(metadata: &serde_json::Value) -> Vec<Package> {
    metadata["packages"]
        .as_array()
        .expect("packages")
        .iter()
        .map(|p| Package {
            name: p["name"].as_str().unwrap().to_owned(),
            version: p["version"].as_str().unwrap().to_owned(),
            source: p["source"].as_str().map(str::to_owned),
            manifest_path: PathBuf::from(p["manifest_path"].as_str().unwrap()),
        })
        .collect()
}

/// Each crate a vendored project provides, its project and its project's folder: every package manifest
/// under the folder, walked with a stack of its own, not by recursion (CLAUDE.md §11).
fn vendored(root: &Path, manifest: &toml::Table) -> BTreeMap<String, (String, PathBuf)> {
    let mut out = BTreeMap::new();
    for (project, spec) in projects(manifest) {
        let folder = root
            .join(text(spec, "path").expect("a project's path"))
            .canonicalize()
            .unwrap();
        let mut todo = vec![folder.clone()];
        while let Some(dir) = todo.pop() {
            for entry in std::fs::read_dir(&dir).unwrap() {
                let entry = entry.unwrap();
                if entry.file_type().unwrap().is_dir() {
                    todo.push(entry.path());
                } else if entry.file_name() == "Cargo.toml" {
                    let package: toml::Table = std::fs::read_to_string(entry.path())
                        .unwrap()
                        .parse()
                        .unwrap();
                    if let Some(name) = package
                        .get("package")
                        .and_then(|p| p.get("name"))
                        .and_then(toml::Value::as_str)
                    {
                        let before = out.insert(name.to_owned(), (project.clone(), folder.clone()));
                        assert!(
                            before.as_ref().is_none_or(|(p, _)| p == project),
                            "two projects provide {name}"
                        );
                    }
                }
            }
        }
    }
    out
}

/// What's wrong with the packages linked: a vendored crate linked more than once, or from anywhere but
/// its project's folder; a hyper-light product's crate taken from git.
fn one_revision_problems(
    packages: &[Package],
    vendored: &BTreeMap<String, (String, PathBuf)>,
) -> Vec<String> {
    let mut out = Vec::new();
    let place = |p: &Package| {
        format!(
            "{} {} at {}",
            p.version,
            p.source.as_deref().unwrap_or("by path"),
            p.manifest_path.display()
        )
    };
    for (name, (project, folder)) in vendored {
        let linked: Vec<&Package> = packages.iter().filter(|p| &p.name == name).collect();
        match linked.as_slice() {
            [] => {}
            [one]
                if one.source.is_none()
                    && one
                        .manifest_path
                        .canonicalize()
                        .is_ok_and(|m| m.starts_with(folder)) => {}
            [one] => out.push(format!(
                "{name} (vendored as {project}) is linked as {}, not from vendor/{project}",
                place(one)
            )),
            many => out.push(format!(
                "{name} (vendored as {project}) is linked {} times: {}",
                many.len(),
                many.iter().map(|p| place(p)).collect::<Vec<_>>().join("; ")
            )),
        }
    }
    for p in packages {
        if p.source.as_deref().is_some_and(|s| {
            s.to_ascii_lowercase()
                .starts_with("git+https://github.com/hyper-light/")
        }) {
            out.push(format!(
                "{} is a hyper-light product's crate taken from git ({}): vendor it",
                p.name,
                place(p)
            ));
        }
    }
    out
}

/// The crates the plan's first phases link (plan RQ0.1), each the vendored one.
const LINKED: [&str; 22] = [
    "amphora",
    "amphora-api",
    "amphora-engine",
    "aws-lc-rs",
    "hyper-block",
    "hyper-datagram",
    "hyper-durable",
    "hyper-log",
    "hyper-measure",
    "hyper-quic",
    "hyper-raft",
    "hyper-rt",
    "hyper-rt-transport",
    "hyper-sim",
    "hyper-timing",
    "hyper-tls",
    "hyper-transport",
    "mantle-client",
    "mantle-crc",
    "mantle-http",
    "mantle-protocol",
    "mantle-s3",
];

/// Resolve the workspace with `cargo metadata` → each vendored crate linked once, from its folder, and
/// the crates the plan names among them (AC-RQ0.1.3).
#[test]
fn ac_rq0_1_3_the_workspace_links_one_revision_of_each_shared_crate() {
    let root = root();
    let vendored = vendored(&root, &manifest(&root));
    let packages = packages(metadata(&root));
    let found = one_revision_problems(&packages, &vendored);
    assert!(
        found.is_empty(),
        "the workspace links more than one revision:\n{}",
        found.join("\n")
    );
    for name in LINKED {
        assert!(vendored.contains_key(name), "{name} isn't a vendored crate");
        assert_eq!(
            packages.iter().filter(|p| p.name == name).count(),
            1,
            "{name} isn't linked once"
        );
    }
}

/// Add a second hyper-quic, from a fork by git, beside the vendored one → the check fails and names
/// both paths (T-RQ0.1.4).
#[test]
fn t_rq0_1_4_a_second_hyper_quic_fails_naming_both_paths() {
    let root = root();
    let vendored = vendored(&root, &manifest(&root));
    let mut packages = packages(metadata(&root));
    // The vendored hyper-quic's manifest as cargo names it, which is what the check prints: on Windows
    // `canonicalize` gives the verbatim `\\?\C:\…` form instead, which the message never holds.
    let ours = packages
        .iter()
        .find(|p| p.name == "hyper-quic" && p.source.is_none())
        .map(|p| p.manifest_path.clone())
        .expect("the vendored hyper-quic is linked");
    assert!(
        ours.ends_with("vendor/hyper-raft/crates/hyper-quic/Cargo.toml"),
        "{}",
        ours.display()
    );
    let fork = "/home/someone/.cargo/git/checkouts/hyper-raft-0123456789abcdef/0123456/crates/hyper-quic/Cargo.toml";
    packages.push(Package {
        name: "hyper-quic".into(),
        version: "0.1.0".into(),
        source: Some("git+https://github.com/someone/hyper-raft?rev=0123456#0123456789abcdef0123456789abcdef01234567".into()),
        manifest_path: fork.into(),
    });
    let found = one_revision_problems(&packages, &vendored);
    assert!(
        found.iter().any(|p| p
            .starts_with("hyper-quic (vendored as hyper-raft) is linked 2 times")
            && p.contains(fork)
            && p.contains(&ours.display().to_string())),
        "{found:?}"
    );
}

/// A hyper-light product's crate taken from git, and a vendored crate from crates.io in its folder's
/// place → each named (AC-RQ0.1.3).
#[test]
fn t_rq0_1_4_a_products_crate_from_git_or_crates_io_fails() {
    let root = root();
    let vendored = vendored(&root, &manifest(&root));
    let mut packages: Vec<Package> = packages(metadata(&root))
        .into_iter()
        .filter(|p| p.name != "mantle-crc")
        .collect();
    packages.push(Package {
        name: "mantle-crc".into(),
        version: "0.1.1".into(),
        source: Some("registry+https://github.com/rust-lang/crates.io-index".into()),
        manifest_path: "/home/someone/.cargo/registry/src/index.crates.io-0/mantle-crc-0.1.1/Cargo.toml".into(),
    });
    packages.push(Package {
        name: "veil-api".into(),
        version: "0.1.0".into(),
        source: Some("git+https://github.com/Hyper-Light/veil?rev=0123456#0123456789abcdef0123456789abcdef01234567".into()),
        manifest_path: "/home/someone/.cargo/git/checkouts/veil/0123456/crates/api/Cargo.toml".into(),
    });
    let found = one_revision_problems(&packages, &vendored);
    assert!(
        found
            .iter()
            .any(|p| p.starts_with("mantle-crc (vendored as mantle) is linked as 0.1.1 registry+")),
        "{found:?}"
    );
    assert!(
        found
            .iter()
            .any(|p| p.starts_with("veil-api is a hyper-light product's crate taken from git")),
        "{found:?}"
    );
}

// ---------------------------------------------------------------------------------------------------
// AC-RQ0.1.5: each exception to the -api rule, one line of vendor/MANIFEST.toml.

/// The vendored hyper-light products' crates a workspace member names itself, but their `-api` crates:
/// the crates CLAUDE.md §9 makes exceptions of.
fn named_non_api(
    metadata: &serde_json::Value,
    vendored: &BTreeMap<String, (String, PathBuf)>,
    manifest: &toml::Table,
) -> BTreeMap<String, String> {
    let members: BTreeSet<&str> = metadata["workspace_members"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(serde_json::Value::as_str)
        .collect();
    let mut out = BTreeMap::new();
    for package in metadata["packages"].as_array().unwrap() {
        if !members.contains(package["id"].as_str().unwrap()) {
            continue;
        }
        for dependency in package["dependencies"].as_array().unwrap() {
            let name = dependency["name"].as_str().unwrap();
            let Some((project, _)) = vendored.get(name) else {
                continue;
            };
            let product = projects(manifest)
                .get(project)
                .and_then(|s| text(s, "repository"))
                .is_some_and(|r| r.starts_with("https://github.com/hyper-light/"));
            if product && !name.ends_with("-api") {
                out.insert(name.to_owned(), project.clone());
            }
        }
    }
    out
}

/// Where the exceptions differ from what the workspace names: a crate named with no line, a line for a
/// crate not named, a line in another project's table, a line that doesn't say what ends it or isn't one line.
fn exception_problems(named: &BTreeMap<String, String>, manifest: &toml::Table) -> Vec<String> {
    let mut lines = BTreeMap::new();
    for (project, spec) in projects(manifest) {
        for (name, why) in spec
            .get("api_exceptions")
            .and_then(toml::Value::as_table)
            .into_iter()
            .flatten()
        {
            lines.insert(
                name.clone(),
                (project.clone(), why.as_str().unwrap_or_default().to_owned()),
            );
        }
    }
    let mut out = Vec::new();
    for (name, project) in named {
        match lines.get(name) {
            None => out.push(format!("{name} ({project}) is named without an -api crate and has no exception line: add one to [projects.{project}.api_exceptions]")),
            Some((at, _)) if at != project => out.push(format!("{name}'s exception line is in [projects.{at}.api_exceptions], not {project}'s")),
            Some((_, why)) if why.contains('\n') || !why.contains("Ends when") => out.push(format!("{name}'s exception line must be one line saying why and what ends it: {why:?}")),
            Some(_) => {}
        }
    }
    for name in lines.keys().filter(|n| !named.contains_key(*n)) {
        out.push(format!(
            "{name} has an exception line but no member of the workspace names it: the line goes"
        ));
    }
    out
}

/// Read the workspace's own dependencies → every vendored product crate it names but an -api crate has
/// its line, and every line a crate it names (AC-RQ0.1.5).
#[test]
fn ac_rq0_1_5_each_exception_to_the_api_rule_is_one_line_of_the_manifest() {
    let root = root();
    let manifest = manifest(&root);
    let vendored = vendored(&root, &manifest);
    let named = named_non_api(metadata(&root), &vendored, &manifest);
    let found = exception_problems(&named, &manifest);
    assert!(found.is_empty(), "{}", found.join("\n"));
    assert!(
        ["hyper-rt", "mantle-client", "amphora"]
            .iter()
            .all(|c| named.contains_key(*c)),
        "{named:?}"
    );
}

/// Name a product's crate without a line; keep a line nothing names; write one without what ends it →
/// each refused, named (AC-RQ0.1.5).
#[test]
fn t_rq0_1_5_a_missing_a_stale_and_a_half_said_exception_are_each_named() {
    let manifest = planted(
        "[projects.mantle]\nrepository = \"https://github.com/hyper-light/mantle\"\n[projects.mantle.api_exceptions]\nmantle-http = \"the codec; no -api crate\"\nmantle-disk = \"no -api crate. Ends when Mantle publishes one\"\n",
    );
    let named = BTreeMap::from([
        ("mantle-client".to_owned(), "mantle".to_owned()),
        ("mantle-http".to_owned(), "mantle".to_owned()),
    ]);
    let found = exception_problems(&named, &manifest);
    assert_eq!(found.len(), 3, "{found:?}");
    assert!(
        found[0].starts_with(
            "mantle-client (mantle) is named without an -api crate and has no exception line"
        ),
        "{found:?}"
    );
    assert!(
        found[1].starts_with(
            "mantle-http's exception line must be one line saying why and what ends it"
        ),
        "{found:?}"
    );
    assert!(
        found[2].starts_with(
            "mantle-disk has an exception line but no member of the workspace names it"
        ),
        "{found:?}"
    );
}

// ---------------------------------------------------------------------------------------------------
// kiln's vendoring tool on Reliquary's manifest (T-RQ0.1.1, T-RQ0.1.2, T-RQ0.1.5): Reliquary's copy of
// scripts/vendor.py, run against an upstream made here, offline.

/// An upstream repository and a repository vendoring it as Reliquary does, with one recorded change.
struct Lab {
    _dir: tempfile::TempDir,
    upstream: PathBuf,
    ours: PathBuf,
    cache: PathBuf,
    first: String,
}

const LIB: &str =
    "pub fn one() -> u32 { 1 }\npub fn two() -> u32 { 2 }\npub fn three() -> u32 { 3 }\n";

fn git_in(dir: &Path, args: &[&str]) -> String {
    let out = ran(Command::new("git")
        .current_dir(dir)
        .args(args)
        .env(
            "GIT_CONFIG_GLOBAL",
            if cfg!(windows) { "NUL" } else { "/dev/null" },
        )
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_AUTHOR_NAME", "lab")
        .env("GIT_AUTHOR_EMAIL", "lab@lab.invalid")
        .env("GIT_COMMITTER_NAME", "lab")
        .env("GIT_COMMITTER_EMAIL", "lab@lab.invalid"));
    String::from_utf8(out.stdout).unwrap().trim().to_owned()
}

impl Lab {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let upstream = dir.path().join("upstream");
        let ours = dir.path().join("ours");
        let cache = dir.path().join("cache");
        for d in [&upstream, &ours.join("scripts"), &ours.join("vendor")] {
            std::fs::create_dir_all(d).unwrap();
        }
        std::fs::copy(
            root().join("scripts/vendor.py"),
            ours.join("scripts/vendor.py"),
        )
        .unwrap();
        git_in(&upstream, &["init", "-q", "-b", "main"]);
        std::fs::create_dir_all(upstream.join("src")).unwrap();
        std::fs::write(upstream.join("src/lib.rs"), LIB).unwrap();
        std::fs::write(upstream.join("LICENSE"), "MIT\n").unwrap();
        let first = Self::commit(&upstream, "one");
        // The upstream's path as a TOML string, by TOML's own rules: on Windows a path's `\U…` read in a
        // basic string unescaped is a unicode escape, and the manifest doesn't parse.
        let repository = toml::Value::String(upstream.display().to_string());
        let manifest = format!(
            "format = 1\ngate = []\n\n[projects.demo]\nkind = \"git\"\nrepository = {repository}\ncommit = \"{first}\"\npath = \"vendor/demo\"\ntake = [\"src\", \"LICENSE\"]\nown = [\"UPSTREAM.md\"]\n"
        );
        std::fs::write(ours.join("vendor/MANIFEST.toml"), manifest).unwrap();
        let lab = Self {
            _dir: dir,
            upstream,
            ours,
            cache,
            first,
        };
        let (ok, said) = lab.vendor(&["demo", &lab.first.clone(), "--no-gate"]);
        assert!(ok, "{said}");
        let lib = lab.ours.join("vendor/demo/src/lib.rs");
        std::fs::write(
            &lib,
            LIB.replace("two() -> u32 { 2 }", "two() -> u32 { 2 + 0 } // ours"),
        )
        .unwrap();
        let (ok, said) = lab.vendor(&[
            "record",
            "demo",
            "two says ours",
            "--why",
            "a change of Reliquary's",
        ]);
        assert!(ok, "{said}");
        std::fs::write(
            lab.ours.join("vendor/demo/UPSTREAM.md"),
            "| src/lib.rs | two | ours | 0001-two-says-ours.patch |\n",
        )
        .unwrap();
        lab
    }

    fn commit(dir: &Path, message: &str) -> String {
        git_in(dir, &["add", "-A"]);
        git_in(dir, &["commit", "-q", "-m", message]);
        git_in(dir, &["rev-parse", "HEAD"])
    }

    /// scripts/vendor.py run on this lab: whether it succeeded, and what it said.
    fn vendor(&self, args: &[&str]) -> (bool, String) {
        let out = Command::new("python3")
            .arg(self.ours.join("scripts/vendor.py"))
            .arg("--root")
            .arg(&self.ours)
            .args(args)
            .env("KILN_VENDOR_CACHE", &self.cache)
            .output()
            .expect("python3 runs scripts/vendor.py");
        (
            out.status.success(),
            String::from_utf8_lossy(&out.stderr).into_owned(),
        )
    }

    /// A new upstream revision that rewrites the line the recorded change edits, and the re-vendor to
    /// it: whether it succeeded, and what it said.
    fn re_vendor_onto_a_conflict(&self) -> (bool, String) {
        std::fs::write(
            self.upstream.join("src/lib.rs"),
            LIB.replace("two() -> u32 { 2 }", "two() -> u32 { 22 }"),
        )
        .unwrap();
        let second = Self::commit(&self.upstream, "upstream rewrites two");
        self.vendor(&["demo", &second, "--no-gate"])
    }

    /// Every file of ours, by path, but the cache's.
    fn snapshot(&self) -> BTreeMap<PathBuf, Vec<u8>> {
        let mut out = BTreeMap::new();
        let mut todo = vec![self.ours.clone()];
        while let Some(dir) = todo.pop() {
            for entry in std::fs::read_dir(&dir).unwrap() {
                let entry = entry.unwrap();
                if entry.file_type().unwrap().is_dir() {
                    todo.push(entry.path());
                } else {
                    out.insert(entry.path(), std::fs::read(entry.path()).unwrap());
                }
            }
        }
        out
    }
}

/// What this runner lacks to run kiln's scripts/vendor.py on a lab, if anything: git, and a python3
/// with tomllib.
fn lab_lacks() -> Option<Lacking> {
    if let Some(why) = lacks("git", &["--version"]) {
        return Some((GIT, why));
    }
    lacks("python3", &["-c", "import tomllib"]).map(|why| (PYTHON, why))
}

/// Re-vendor a project at the revision it's at, its recorded change re-applied → no byte changes
/// (T-RQ0.1.1).
#[test]
fn t_rq0_1_1_re_vendoring_at_the_current_revision_changes_no_byte() {
    let test = "t_rq0_1_1_re_vendoring_at_the_current_revision_changes_no_byte";
    if let Some((missing, why)) = lab_lacks() {
        return reliquary_testkit::skip(test, missing, &why).unwrap();
    }
    let lab = Lab::new();
    let before = lab.snapshot();
    let (ok, said) = lab.vendor(&["demo", &lab.first, "--no-gate"]);
    assert!(ok, "{said}");
    assert_eq!(lab.snapshot(), before, "{said}");
    let (ok, said) = lab.vendor(&["check"]);
    assert!(ok, "{said}");
}

/// Change one byte of a vendored file without recording it → the check fails naming the file, and a
/// re-vendor is refused, nothing changed (T-RQ0.1.2).
#[test]
fn t_rq0_1_2_an_unrecorded_byte_fails_the_check_naming_the_file() {
    let test = "t_rq0_1_2_an_unrecorded_byte_fails_the_check_naming_the_file";
    if let Some((missing, why)) = lab_lacks() {
        return reliquary_testkit::skip(test, missing, &why).unwrap();
    }
    let lab = Lab::new();
    let license = lab.ours.join("vendor/demo/LICENSE");
    std::fs::write(&license, "MIT.\n").unwrap();
    let (ok, said) = lab.vendor(&["check"]);
    assert!(!ok, "{said}");
    assert!(said.contains("vendor/demo: differs: LICENSE"), "{said}");
    let before = lab.snapshot();
    let (ok, said) = lab.vendor(&["demo", &lab.first, "--no-gate"]);
    assert!(!ok && said.contains("record them first"), "{said}");
    assert_eq!(lab.snapshot(), before);
}

/// Take a new upstream revision that rewrites the line a recorded change edits → the re-vendor stops,
/// naming the file and the change, nothing changed (T-RQ0.1.5).
#[test]
fn t_rq0_1_5_a_conflicting_upstream_revision_stops_naming_the_file_and_the_change() {
    let test = "t_rq0_1_5_a_conflicting_upstream_revision_stops_naming_the_file_and_the_change";
    if let Some((missing, why)) = lab_lacks() {
        return reliquary_testkit::skip(test, missing, &why).unwrap();
    }
    let lab = Lab::new();
    let before = lab.snapshot();
    let (ok, said) = lab.re_vendor_onto_a_conflict();
    assert!(!ok, "{said}");
    assert!(
        said.contains("src/lib.rs")
            && said.contains("0001-two-says-ours.patch")
            && said.contains("two says ours"),
        "{said}"
    );
    assert!(
        said.contains("Nothing was changed in vendor/demo"),
        "{said}"
    );
    assert_eq!(lab.snapshot(), before);
}

/// Give up a re-vendor stopped on a conflict with `--abort` → its kept work is gone (a second
/// `--abort` finds none) and vendor/demo is as it was (T-RQ0.1.5). On Windows, kiln's vendor.py at
/// this copy can't: its `shutil.rmtree` of the kept repository is refused git's read-only object
/// files (WinError 5), kiln's to fix; the test records that skip rather than pass it unseen.
#[test]
fn t_rq0_1_5_a_stopped_re_vendor_is_given_up_with_abort() {
    let test = "t_rq0_1_5_a_stopped_re_vendor_is_given_up_with_abort";
    if let Some((missing, why)) = lab_lacks() {
        return reliquary_testkit::skip(test, missing, &why).unwrap();
    }
    let lab = Lab::new();
    let before = lab.snapshot();
    let (ok, said) = lab.re_vendor_onto_a_conflict();
    assert!(!ok, "{said}");
    let (ok, said) = lab.vendor(&["demo", "--abort"]);
    if !ok && cfg!(windows) && said.contains("PermissionError: [WinError 5]") {
        return reliquary_testkit::skip(
            test,
            "kiln's scripts/vendor.py giving up a stopped re-vendor on Windows",
            "its abort's shutil.rmtree is refused git's read-only object files (WinError 5): kiln's to fix",
        )
        .unwrap();
    }
    assert!(
        ok && said.contains("demo: the re-vendor that waited on a conflict is gone"),
        "{said}"
    );
    assert_eq!(lab.snapshot(), before);
    let (ok, said) = lab.vendor(&["demo", "--abort"]);
    assert!(
        !ok && said.contains("no re-vendor of demo waits on a conflict"),
        "{said}"
    );
}
