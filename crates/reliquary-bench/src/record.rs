//! A measurement's record, kept in `bench/` (CLAUDE.md §7): what was measured, on what hardware, from
//! which commit, when, under what load, and whether it is of record (AC-RQ0.7.4). The host's name is
//! not read: a record names the machine only as the operator states it (`--machine`).

use std::path::Path;

use serde::Serialize;

use crate::driver::Outcome;
use crate::gate::GateOutcome;

/// The hardware and system a record was made on.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Host {
    pub os: &'static str,
    pub arch: &'static str,
    pub cpu: Option<String>,
    /// Cores the process may run on at once.
    pub cores: Option<usize>,
    pub memory_bytes: Option<u64>,
    pub kernel: Option<String>,
    /// The machine's name, as the operator states it.
    pub machine: Option<String>,
}

/// Format: the record's layout, raised when a field changes meaning.
pub const FORMAT: u32 = 1;

/// A record.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Record {
    pub format: u32,
    pub tool: String,
    pub date: String,
    pub host: Host,
    /// The load rule, as the record was held to it.
    pub rule: String,
    #[serde(flatten)]
    pub what: What,
}

/// What a record holds.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum What {
    /// A point, or an A|B.
    Measured(Box<Outcome>),
    /// CI's gate on instruction counts.
    Gate(Box<GateOutcome>),
}

/// The load rule's words in a record.
#[must_use]
pub fn rule() -> String {
    format!(
        "of record only if the one-minute load average less this run's own share stayed under {} at \
         every reading (CLAUDE.md §7) and every build names a clean commit",
        crate::load::LIMIT
    )
}

/// The date now, in RFC 3339 at the second, UTC; the Unix epoch if the clock reads before it.
#[must_use]
pub fn now() -> String {
    let seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX));
    rfc3339(seconds).unwrap_or_else(|| "1970-01-01T00:00:00Z".to_owned())
}

/// Days from 0000-03-01 to 1970-01-01 in the proleptic Gregorian calendar (Hinnant,
/// "chrono-Compatible Low-Level Date Algorithms", `civil_from_days`): the algorithm counts years from
/// March, so a leap day ends its year.
const EPOCH_SHIFT: i64 = 719_468;
/// Days in a Gregorian era of 400 years (Hinnant, same).
const ERA_DAYS: i64 = 146_097;
/// Years in an era.
const ERA_YEARS: i64 = 400;
/// Days in a common year.
const YEAR_DAYS: i64 = 365;
/// Days in four years less one, a century less one, an era less one: where an era's day of the era
/// crosses a leap day, a century's missing one and the era's own (Hinnant, same).
const FOUR_YEARS_LESS_ONE: i64 = 1_460;
/// See [`FOUR_YEARS_LESS_ONE`].
const CENTURY_LESS_ONE: i64 = 36_524;
/// See [`FOUR_YEARS_LESS_ONE`].
const ERA_LESS_ONE: i64 = 146_096;
/// Years in a leap cycle, and in a century.
const LEAP_YEARS: i64 = 4;
/// See [`LEAP_YEARS`].
const CENTURY_YEARS: i64 = 100;
/// The days of the five months from March as a line through `153·m + 2` over 5 (Hinnant, same).
const FIVE_MONTHS_DAYS: i64 = 153;
/// See [`FIVE_MONTHS_DAYS`].
const FIVE_MONTHS: i64 = 5;
/// See [`FIVE_MONTHS_DAYS`].
const FIVE_MONTHS_OFFSET: i64 = 2;
/// Months counted from March: March is 0, so January is 10 and the calendar's month is `m + 3` before
/// it and `m − 9` from it.
const JANUARY_FROM_MARCH: i64 = 10;
/// See [`JANUARY_FROM_MARCH`].
const MARCH: i64 = 3;
/// See [`JANUARY_FROM_MARCH`].
const MARCH_TO_DECEMBER: i64 = 9;
/// February, the month after which a year counted from March has begun.
const FEBRUARY: i64 = 2;
/// Seconds in an hour and in a minute (POSIX.1-2017, Base Definitions §4.16).
const HOUR_SECONDS: i64 = 3_600;
/// See [`HOUR_SECONDS`].
const MINUTE_SECONDS: i64 = 60;

/// The calendar date of a day counted from 1970-01-01: Hinnant's `civil_from_days`, checked.
fn civil(days: i64) -> Option<(i64, i64, i64)> {
    let z = days.checked_add(EPOCH_SHIFT)?;
    let era = z.checked_div_euclid(ERA_DAYS)?;
    let doe = z.checked_rem_euclid(ERA_DAYS)?;
    let yoe = doe
        .checked_sub(doe.checked_div(FOUR_YEARS_LESS_ONE)?)?
        .checked_add(doe.checked_div(CENTURY_LESS_ONE)?)?
        .checked_sub(doe.checked_div(ERA_LESS_ONE)?)?
        .checked_div(YEAR_DAYS)?;
    let year = yoe.checked_add(era.checked_mul(ERA_YEARS)?)?;
    let doy = doe.checked_sub(
        YEAR_DAYS
            .checked_mul(yoe)?
            .checked_add(yoe.checked_div(LEAP_YEARS)?)?
            .checked_sub(yoe.checked_div(CENTURY_YEARS)?)?,
    )?;
    let mp = FIVE_MONTHS
        .checked_mul(doy)?
        .checked_add(FIVE_MONTHS_OFFSET)?
        .checked_div(FIVE_MONTHS_DAYS)?;
    let day = doy
        .checked_sub(
            FIVE_MONTHS_DAYS
                .checked_mul(mp)?
                .checked_add(FIVE_MONTHS_OFFSET)?
                .checked_div(FIVE_MONTHS)?,
        )?
        .checked_add(1)?;
    let month = if mp < JANUARY_FROM_MARCH {
        mp.checked_add(MARCH)?
    } else {
        mp.checked_sub(MARCH_TO_DECEMBER)?
    };
    let year = if month <= FEBRUARY {
        year.checked_add(1)?
    } else {
        year
    };
    Some((year, month, day))
}

/// Unix seconds as RFC 3339 at the second, UTC (RFC 3339 §5.6).
fn rfc3339(seconds: i64) -> Option<String> {
    let day_seconds = crate::changes::DAY_SECONDS;
    let (year, month, day) = civil(seconds.checked_div_euclid(day_seconds)?)?;
    let of_day = seconds.checked_rem_euclid(day_seconds)?;
    let hour = of_day.checked_div(HOUR_SECONDS)?;
    let minute = of_day
        .checked_rem(HOUR_SECONDS)?
        .checked_div(MINUTE_SECONDS)?;
    let second = of_day.checked_rem(MINUTE_SECONDS)?;
    Some(format!(
        "{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z"
    ))
}

/// This machine's hardware and system, as far as each platform says without privilege.
#[must_use]
pub fn host(machine: Option<String>) -> Host {
    Host {
        os: std::env::consts::OS,
        arch: std::env::consts::ARCH,
        cpu: cpu(),
        cores: std::thread::available_parallelism().ok().map(usize::from),
        memory_bytes: memory(),
        kernel: kernel(),
        machine,
    }
}

/// The first line of a file that starts with `key`, its value after the colon.
fn field(path: &str, key: &str) -> Option<String> {
    let text = std::fs::read_to_string(path).ok()?;
    text.lines()
        .find(|l| l.starts_with(key))
        .and_then(|l| l.split_once(':'))
        .map(|(_, v)| v.trim().to_owned())
}

/// What `sysctl -n key` prints.
fn sysctl(key: &str) -> Option<String> {
    let out = std::process::Command::new("sysctl")
        .args(["-n", key])
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_owned())
        .filter(|s| !s.is_empty())
}

fn cpu() -> Option<String> {
    match std::env::consts::OS {
        "linux" => field("/proc/cpuinfo", "model name").or_else(|| {
            let implementer = field("/proc/cpuinfo", "CPU implementer")?;
            let part = field("/proc/cpuinfo", "CPU part")?;
            Some(format!("Arm implementer {implementer} part {part}"))
        }),
        "macos" => sysctl("machdep.cpu.brand_string"),
        "windows" => std::env::var("PROCESSOR_IDENTIFIER").ok(),
        _ => None,
    }
}

/// Format: bytes in a KiB, the unit of /proc/meminfo (proc(5)).
const KIB: u64 = 1_024;

fn memory() -> Option<u64> {
    match std::env::consts::OS {
        "linux" => field("/proc/meminfo", "MemTotal")
            .and_then(|v| v.split_whitespace().next()?.parse::<u64>().ok())
            .and_then(|kib| kib.checked_mul(KIB)),
        "macos" => sysctl("hw.memsize").and_then(|v| v.parse().ok()),
        _ => None,
    }
}

fn kernel() -> Option<String> {
    match std::env::consts::OS {
        "linux" => std::fs::read_to_string("/proc/sys/kernel/osrelease")
            .ok()
            .map(|s| s.trim().to_owned()),
        "macos" => sysctl("kern.osrelease"),
        _ => None,
    }
}

/// Why a record could not be written.
#[derive(Debug)]
pub struct WriteError(pub String);

impl std::fmt::Display for WriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for WriteError {}

/// Writes `record` as JSON to `path`, never over a record already there.
pub fn write(record: &Record, path: &Path) -> Result<(), WriteError> {
    let text = serde_json::to_string_pretty(record).map_err(|e| WriteError(e.to_string()))?;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| WriteError(format!("{}: {e}", path.display())))?;
    std::io::Write::write_all(&mut file, text.as_bytes())
        .and_then(|()| std::io::Write::write_all(&mut file, b"\n"))
        .map_err(|e| WriteError(format!("{}: {e}", path.display())))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The date is the proleptic Gregorian calendar's, against dates whose Unix seconds are known: the
    /// epoch, a leap day, the day after one, a century's missing leap day, an era's, and this one
    /// (AC-RQ0.7.4: a record names its date).
    #[test]
    fn dates_are_the_gregorian_calendars() {
        for (seconds, date) in [
            (0, "1970-01-01T00:00:00Z"),
            (951_782_400, "2000-02-29T00:00:00Z"),
            (951_868_799, "2000-02-29T23:59:59Z"),
            (951_868_800, "2000-03-01T00:00:00Z"),
            (4_107_542_400, "2100-03-01T00:00:00Z"),
            (4_107_456_000, "2100-02-28T00:00:00Z"),
            (1_795_208_591, "2026-11-20T21:03:11Z"),
            (-1, "1969-12-31T23:59:59Z"),
            (-62_135_596_800, "0001-01-01T00:00:00Z"),
        ] {
            assert_eq!(rfc3339(seconds).unwrap(), date, "{seconds}");
        }
        // Every day of 1600 to 2400 follows the one before it.
        let mut last = civil(-135_140).unwrap();
        for d in -135_139..157_000 {
            let (y, m, day) = civil(d).unwrap();
            let next_ok = (y, m, day) > last;
            assert!(
                next_ok && (1..=12).contains(&m) && (1..=31).contains(&day),
                "{d}"
            );
            last = (y, m, day);
        }
    }

    /// A record names its hardware and date, and is never written over (AC-RQ0.7.4).
    #[test]
    fn a_record_names_its_host_and_date_and_is_not_overwritten() {
        let h = host(Some("bench-host".to_owned()));
        assert_eq!(h.os, std::env::consts::OS);
        assert!(h.cores.is_some_and(|c| c > 0));
        let d = now();
        assert!(
            d.ends_with('Z') && d.len() == "2026-11-20T21:03:11Z".len(),
            "{d}"
        );
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("r.json");
        let gate = GateOutcome {
            counter: "os".to_owned(),
            seed: 1,
            precision: crate::rounds::PRECISION,
            commits: vec!["abc".to_owned(), "abc".to_owned()],
            benches: vec![],
            failed: vec![],
            readings: vec![],
            standing: crate::load::Standing::NotOfRecord {
                why: vec!["no load was read".to_owned()],
            },
        };
        let r = Record {
            format: FORMAT,
            tool: "reliquary bench test".to_owned(),
            date: d,
            host: h,
            rule: rule(),
            what: What::Gate(Box::new(gate)),
        };
        write(&r, &path).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("\"kind\": \"gate\""), "{text}");
        assert!(
            write(&r, &path).is_err(),
            "a second write over it is refused"
        );
    }
}
