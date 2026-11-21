//! The skip record's tests: what a skip writes, that skips from many threads land whole, and where the
//! record is.

use super::*;

/// Skip a test lacking a privilege, with tabs and line breaks in what it says → one line of four
/// fields in the record, the binary named without cargo's hash, and nothing else in the file
/// (T-RQ0.3.6, AC-RQ0.3.5).
#[test]
fn a_skip_writes_one_line_of_four_fields() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("nested").join("skips.tsv");
    skip_to(
        &path,
        "needs_raw_sockets",
        "CAP_NET_RAW",
        "socket(AF_INET, SOCK_RAW):\tEPERM\nas this runner's user",
    )
    .unwrap();
    let text = fs::read_to_string(&path).unwrap();
    assert_eq!(text.lines().count(), 1, "{text}");
    let fields: Vec<&str> = text.trim_end_matches('\n').split('\t').collect();
    assert_eq!(fields.len(), 4, "{text}");
    assert_eq!(fields[0], binary());
    assert!(
        !fields[0].contains('-')
            || !fields[0]
                .rsplit('-')
                .next()
                .unwrap()
                .bytes()
                .all(|b| b.is_ascii_hexdigit()),
        "the binary keeps cargo's hash: {}",
        fields[0]
    );
    assert_eq!(fields[1], "needs_raw_sockets");
    assert_eq!(fields[2], "CAP_NET_RAW");
    assert_eq!(
        fields[3],
        "socket(AF_INET, SOCK_RAW): EPERM as this runner's user"
    );
}

/// Skip from many threads at once, each many times → every line lands whole, none lost
/// (AC-RQ0.3.5).
#[test]
fn skips_from_many_threads_each_land_whole() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("skips.tsv");
    let (threads, each) = (16, 50);
    std::thread::scope(|scope| {
        for t in 0..threads {
            let path = &path;
            scope.spawn(move || {
                for i in 0..each {
                    skip_to(
                        path,
                        &format!("test_{t}_{i}"),
                        "nothing",
                        "a long reason, the same length every time",
                    )
                    .unwrap();
                }
            });
        }
    });
    let text = fs::read_to_string(&path).unwrap();
    let mut lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), threads * each);
    for line in &lines {
        assert_eq!(line.split('\t').count(), 4, "a torn line: {line:?}");
    }
    lines.sort_unstable();
    lines.dedup();
    assert_eq!(
        lines.len(),
        threads * each,
        "a line was written twice or lost"
    );
}

/// Ask where the record is → `RELIQUARY_SKIPS` when the gate sets it, else the record under the
/// target directory this test binary runs from (AC-RQ0.3.5).
#[test]
fn the_record_is_the_gates_or_under_the_target_directory() {
    let found = record().unwrap();
    match std::env::var_os(VAR) {
        Some(path) => assert_eq!(found, PathBuf::from(path)),
        None => {
            assert!(found.ends_with(RECORD), "{}", found.display());
            let target = found.parent().unwrap().parent().unwrap();
            assert!(
                target.join("CACHEDIR.TAG").is_file(),
                "{}",
                target.display()
            );
        }
    }
}
