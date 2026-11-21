//! The event ring's `tracing` layer (plan RQ0.9, T-RQ0.9.5): each event emitted on one of
//! Reliquary's targets becomes the line the ring holds (`line`), offered to the ring on the
//! emitting thread, which takes no lock and never waits. Events of other targets pass it by, and
//! so does every span: the layer filters nothing for the layers beside it.

use std::fmt;
use std::sync::Arc;
use std::time::SystemTime;

use tracing::field::{Field, Visit};
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::layer::{Context, Layer};

use super::catalog::TARGET_PREFIX;
use super::line::Line;
use super::ring::EventRing;

/// Offers each event of Reliquary's targets to a ring.
pub struct RingLayer {
    ring: Arc<EventRing>,
    now: fn() -> SystemTime,
}

impl RingLayer {
    /// A layer offering `ring` each event of Reliquary's targets, timed by `now`: the wall clock
    /// in a process, a fixed one in a test.
    #[must_use]
    pub fn new(ring: Arc<EventRing>, now: fn() -> SystemTime) -> Self {
        Self { ring, now }
    }
}

impl<S: Subscriber> Layer<S> for RingLayer {
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let metadata = event.metadata();
        if !metadata.target().starts_with(TARGET_PREFIX) {
            return;
        }
        let mut line = Line::begin((self.now)(), word(*metadata.level()), metadata.target());
        event.record(&mut Fields(&mut line));
        // What becomes of the line, kept or let go, the ring counts.
        let _offered = self.ring.offer(line.end());
    }
}

/// The word an event line's `level` holds.
fn word(level: Level) -> &'static str {
    if level == Level::ERROR {
        "error"
    } else if level == Level::WARN {
        "warn"
    } else if level == Level::INFO {
        "info"
    } else if level == Level::DEBUG {
        "debug"
    } else {
        "trace"
    }
}

/// Writes each field an event records into its line, a number as a number.
struct Fields<'a>(&'a mut Line);

impl Visit for Fields<'_> {
    fn record_f64(&mut self, field: &Field, value: f64) {
        self.0.real(field.name(), value);
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.0.whole(field.name(), value);
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.0.whole(field.name(), value);
    }

    fn record_i128(&mut self, field: &Field, value: i128) {
        self.0.whole(field.name(), value);
    }

    fn record_u128(&mut self, field: &Field, value: u128) {
        self.0.whole(field.name(), value);
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.0.truth(field.name(), value);
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.0.text(field.name(), value);
    }

    fn record_error(&mut self, field: &Field, value: &(dyn std::error::Error + 'static)) {
        self.0.display(field.name(), &value);
    }

    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        self.0.debug(field.name(), value);
    }
}
