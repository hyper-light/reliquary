//! An event as one line of JSON (RFC 8259): what the event ring holds and its sink writes
//! (`docs/observability.md`, "Event lines"). The keys are those kiln's log reader takes for
//! itself (`timestamp`, `level`, `message`; kiln's `docs/observe.md` §8.1), then `target`,
//! `event` and the event's own fields. The time is in seconds since the epoch, the base unit,
//! which kiln's reader tells from milliseconds and nanoseconds by its size.

use std::fmt::{self, Write as _};
use std::time::{SystemTime, UNIX_EPOCH};

use super::catalog::{Dropped, EVENTS_DROPPED_EVENT};

/// A line being written: its time, level and target, then its fields as they are recorded.
pub(super) struct Line {
    text: String,
}

impl Line {
    /// A line of an event at `at`, of `level`, on `target`.
    pub(super) fn begin(at: SystemTime, level: &str, target: &str) -> Self {
        let mut text = String::new();
        text.push_str("{\"timestamp\":");
        seconds(at, &mut text);
        text.push_str(",\"level\":");
        string(level, &mut text);
        text.push_str(",\"target\":");
        string(target, &mut text);
        Self { text }
    }

    /// A field holding text.
    pub(super) fn text(&mut self, name: &str, value: &str) {
        self.key(name);
        string(value, &mut self.text);
    }

    /// A field holding what `value`'s `Debug` writes, as text.
    pub(super) fn debug(&mut self, name: &str, value: &dyn fmt::Debug) {
        self.key(name);
        self.quoted(format_args!("{value:?}"));
    }

    /// A field holding what `value`'s `Display` writes, as text.
    pub(super) fn display(&mut self, name: &str, value: &dyn fmt::Display) {
        self.key(name);
        self.quoted(format_args!("{value}"));
    }

    /// A field holding a whole number.
    pub(super) fn whole(&mut self, name: &str, value: impl fmt::Display) {
        self.key(name);
        // A String's writes never fail, and an integer's Display makes none.
        let _ = write!(self.text, "{value}");
    }

    /// A field holding a real number: as a number when it is one, else as the text of what it is
    /// (`NaN`, `inf`), which JSON has no number for.
    pub(super) fn real(&mut self, name: &str, value: f64) {
        if value.is_finite() {
            self.whole(name, value);
        } else {
            self.display(name, &value);
        }
    }

    /// A field holding `true` or `false`.
    pub(super) fn truth(&mut self, name: &str, value: bool) {
        self.whole(name, value);
    }

    /// The line, ended.
    pub(super) fn end(mut self) -> Box<[u8]> {
        self.text.push_str("}\n");
        self.text.into_bytes().into_boxed_slice()
    }

    fn key(&mut self, name: &str) {
        self.text.push(',');
        string(name, &mut self.text);
        self.text.push(':');
    }

    /// `args` as a JSON string. A `Debug` or `Display` that fails leaves what it wrote, and the
    /// line says it failed rather than end the value unmarked.
    fn quoted(&mut self, args: fmt::Arguments<'_>) {
        self.text.push('"');
        if Escaped(&mut self.text).write_fmt(args).is_err() {
            self.text.push_str(" (its formatting failed here)");
        }
        self.text.push('"');
    }
}

/// The line `events.dropped` is written as: what the ring let go since its previous batch, by
/// reason, at `at`.
pub(super) fn dropped(at: SystemTime, counts: impl Fn(Dropped) -> u64) -> Box<[u8]> {
    let event = &EVENTS_DROPPED_EVENT;
    let mut line = Line::begin(at, event.level.word(), event.target);
    line.text("event", event.name);
    line.text("message", event.help);
    for &reason in Dropped::ALL {
        line.whole(reason.word(), counts(reason));
    }
    line.end()
}

/// `at` as seconds since the epoch, to the nanosecond; a clock before 1970 as the negative
/// seconds it is.
fn seconds(at: SystemTime, text: &mut String) {
    // A String's writes never fail.
    let _ = match at.duration_since(UNIX_EPOCH) {
        Ok(after) => write!(text, "{}.{:09}", after.as_secs(), after.subsec_nanos()),
        Err(before) => {
            let before = before.duration();
            write!(text, "-{}.{:09}", before.as_secs(), before.subsec_nanos())
        }
    };
}

/// `s` as a JSON string.
fn string(s: &str, text: &mut String) {
    text.push('"');
    for c in s.chars() {
        escape(c, text);
    }
    text.push('"');
}

/// `c` as it stands inside a JSON string: a quotation mark, a reverse solidus and every control
/// character escaped, as RFC 8259 §7 requires of U+0000 to U+001F (and here of the other control
/// characters too, which it allows), the rest as it is.
fn escape(c: char, text: &mut String) {
    match c {
        '"' => text.push_str("\\\""),
        '\\' => text.push_str("\\\\"),
        '\n' => text.push_str("\\n"),
        '\r' => text.push_str("\\r"),
        '\t' => text.push_str("\\t"),
        c if c.is_control() => {
            // A String's writes never fail.
            let _ = write!(text, "\\u{:04x}", u32::from(c));
        }
        c => text.push(c),
    }
}

/// What is written through it is escaped as JSON string contents.
struct Escaped<'a>(&'a mut String);

impl fmt::Write for Escaped<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            escape(c, self.0);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    fn text(line: Box<[u8]>) -> String {
        String::from_utf8(line.into_vec()).unwrap()
    }

    fn json(line: &str) -> serde_json::Value {
        serde_json::from_str(line.strip_suffix('\n').unwrap()).unwrap()
    }

    /// Write a line whose target and fields hold quotes, reverse solidi, newlines, every control
    /// character and text beyond ASCII → one line that ends at its one newline and that a JSON
    /// reader gives back exactly (T-RQ0.9.5's lines; CLAUDE.md §6, hostile input).
    #[test]
    fn t_rq0_9_5_a_line_of_hostile_text_is_one_line_a_json_reader_gives_back() {
        let hostile: String = (0_u32..=0x9f)
            .filter_map(char::from_u32)
            .chain("\"\\ é ✓ 𝄞 \u{2028}".chars())
            .collect();
        let mut line = Line::begin(UNIX_EPOCH, "warn", &hostile);
        line.text("event", &hostile);
        line.debug("debugged", &hostile);
        line.display("shown", &hostile);
        let written = text(line.end());
        assert_eq!(written.matches('\n').count(), 1, "{written}");
        assert!(written.ends_with("}\n"));
        let read = json(&written);
        assert_eq!(read["target"], hostile.as_str());
        assert_eq!(read["event"], hostile.as_str());
        assert_eq!(read["debugged"], format!("{hostile:?}"));
        assert_eq!(read["shown"], hostile.as_str());
    }

    /// Write times after the epoch, at it and before it → seconds with nine places, and a minus
    /// for a clock before 1970 (T-RQ0.9.5: the lines the ring holds).
    #[test]
    fn t_rq0_9_5_a_lines_time_is_seconds_since_the_epoch_to_the_nanosecond() {
        // Nanoseconds every platform's clock holds: Windows' counts in 100 ns.
        let after = UNIX_EPOCH + Duration::new(1_795_207_863, 312_000_700);
        let before = UNIX_EPOCH - Duration::new(2, 500_000_000);
        for (at, want) in [
            (after, "1795207863.312000700"),
            (UNIX_EPOCH, "0.000000000"),
            (before, "-2.500000000"),
        ] {
            let written = text(Line::begin(at, "info", "t").end());
            assert!(
                written.starts_with(&format!("{{\"timestamp\":{want},")),
                "{written}"
            );
            assert!(json(&written)["timestamp"].is_number(), "{written}");
        }
    }

    /// Write numbers, truths and reals that JSON has no number for → numbers where JSON has them,
    /// text where it doesn't, and the line still reads (T-RQ0.9.5: the lines the ring holds).
    #[test]
    fn t_rq0_9_5_a_lines_numbers_are_json_numbers_and_the_rest_text() {
        let mut line = Line::begin(UNIX_EPOCH, "info", "t");
        line.whole("count", u64::MAX);
        line.whole("signed", i64::MIN);
        line.real("share", 0.25);
        line.real("nothing", f64::NAN);
        line.real("endless", f64::INFINITY);
        line.truth("yes", true);
        let read = json(&text(line.end()));
        assert_eq!(read["count"], u64::MAX);
        assert_eq!(read["signed"], i64::MIN);
        assert_eq!(read["share"], 0.25);
        assert_eq!(read["nothing"], "NaN");
        assert_eq!(read["endless"], "inf");
        assert_eq!(read["yes"], true);
    }

    /// Write `events.dropped` with a count for each reason → the catalog's target, name, level and
    /// help, and each reason's count under its word (AC-RQ0.9.1).
    #[test]
    fn ac_rq0_9_1_the_drop_marker_is_the_catalogs_event() {
        let read = json(&text(dropped(UNIX_EPOCH, |reason| match reason {
            Dropped::Evicted => 7,
            Dropped::Oversize => 2,
            Dropped::Closed => 0,
        })));
        assert_eq!(read["target"], EVENTS_DROPPED_EVENT.target);
        assert_eq!(read["event"], EVENTS_DROPPED_EVENT.name);
        assert_eq!(read["level"], EVENTS_DROPPED_EVENT.level.word());
        assert_eq!(read["message"], EVENTS_DROPPED_EVENT.help);
        assert_eq!(read["evicted"], 7);
        assert_eq!(read["oversize"], 2);
        assert_eq!(read["closed"], 0);
    }

    /// A Debug that fails part-way → the line keeps what it wrote, says it failed, and still reads
    /// (T-RQ0.9.5: the lines the ring holds; CLAUDE.md §3).
    #[test]
    fn t_rq0_9_5_a_failing_debug_is_said_in_the_line() {
        struct Fails;
        impl fmt::Debug for Fails {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("half")?;
                Err(fmt::Error)
            }
        }
        let mut line = Line::begin(UNIX_EPOCH, "info", "t");
        line.debug("value", &Fails);
        let read = json(&text(line.end()));
        assert_eq!(read["value"], "half (its formatting failed here)");
    }
}
