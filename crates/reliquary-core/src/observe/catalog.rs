//! Every metric and event Reliquary emits, with what each means: the list `docs/observability.md`
//! is held to (plan RQ0.9, AC-RQ0.9.1), and the one file where a `reliquary_` name is written
//! (`tests/observe.rs` fails on one anywhere else under `crates/*/src`, as it does on tracing's
//! macros outside `observe/`). The conventions these follow are `docs/design/observability.md`'s;
//! a new name, label value or event is agreed with kiln's Observe owner first (CLAUDE.md §10).

/// What a metric measures, in a base unit (Prometheus, "Metric and label naming", Base Units): the
/// unit's word ends the metric's name and is its family's `# UNIT` (OpenMetrics 1.0, "Unit").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    /// A count of things, which is no unit.
    None,
    Seconds,
    Bytes,
    /// A share, from 0 to 1, never a percentage.
    Ratio,
}

impl Unit {
    /// The word a name ends in and `# UNIT` states; none for a count.
    #[must_use]
    pub const fn word(self) -> Option<&'static str> {
        match self {
            Self::None => None,
            Self::Seconds => Some("seconds"),
            Self::Bytes => Some("bytes"),
            Self::Ratio => Some("ratio"),
        }
    }
}

/// A metric's type, as its family's `# TYPE` names it (OpenMetrics 1.0, "Metric Types").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Counts what happens: its sample is named `<family>_total`, and it only goes up.
    Counter,
    /// A value now.
    Gauge,
}

impl Kind {
    /// The word `# TYPE` states.
    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Self::Counter => "counter",
            Self::Gauge => "gauge",
        }
    }
}

/// A label: its name and the closed set of values it takes, which Reliquary chooses and a client
/// never does (`docs/design/observability.md`, O2).
#[derive(Debug)]
pub struct Label {
    pub name: &'static str,
    pub values: &'static [&'static str],
}

/// A metric: the name a query names it by (a counter's ends in `_total`), the component it belongs
/// to and is drawn in, its type, unit and labels, and the help its family states.
#[derive(Debug)]
pub struct Metric {
    pub name: &'static str,
    pub component: &'static str,
    pub kind: Kind,
    pub unit: Unit,
    pub labels: &'static [Label],
    pub help: &'static str,
}

impl Metric {
    /// The family's name, as OpenMetrics' text names it: a counter's without its `_total`, none for
    /// a counter whose name lacks it.
    #[must_use]
    pub fn family(&self) -> Option<&'static str> {
        match self.kind {
            Kind::Counter => self.name.strip_suffix("_total"),
            Kind::Gauge => Some(self.name),
        }
    }

    /// How many series the metric has: the product of its labels' value counts, every one of which
    /// is exported from the start (Prometheus, "Instrumentation", Avoid missing metrics); none past
    /// what a `usize` holds.
    #[must_use]
    pub fn series(&self) -> Option<usize> {
        self.labels
            .iter()
            .try_fold(1_usize, |n, label| n.checked_mul(label.values.len()))
    }
}

/// An event's severity: `error`, a failure a person must look at; `warn`, something let go or
/// refused; `info`, a change of state. What is chatter below these is no event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Error,
    Warn,
    Info,
}

impl Level {
    /// The word an event line's `level` holds.
    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warn => "warn",
            Self::Info => "info",
        }
    }
}

/// What an event's field holds.
#[derive(Debug)]
pub enum Values {
    /// A count: a whole number, at least zero.
    Count,
    /// One of a closed set Reliquary chooses: the only kind of value a field kiln makes a tag of
    /// may hold.
    Closed(&'static [&'static str]),
    /// What a request or the world chooses (a repository, a digest, an error's words): an
    /// attribute of the event alone, under a name kiln never makes a tag.
    Free,
}

/// An event's field: its name and what it holds.
#[derive(Debug)]
pub struct Field {
    pub name: &'static str,
    pub values: Values,
}

/// An event: the dotted target it goes on, the name its `event` field gives it, the component it
/// belongs to, its level, its fields, and what it means.
#[derive(Debug)]
pub struct Event {
    pub target: &'static str,
    pub name: &'static str,
    pub component: &'static str,
    pub level: Level,
    pub fields: &'static [Field],
    pub help: &'static str,
}

/// What every target of Reliquary's begins with: the event ring's layer takes the events of these
/// alone.
pub const TARGET_PREFIX: &str = "reliquary.";

/// Why the event ring let an event go: `reliquary_events_dropped_total`'s `reason`, and the
/// counts `events.dropped` carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dropped {
    /// The oldest line, evicted to make room for a newer one.
    Evicted,
    /// Longer than the ring's line bound.
    Oversize,
    /// Emitted after the ring closed.
    Closed,
}

impl Dropped {
    pub const ALL: &'static [Self] = &[Self::Evicted, Self::Oversize, Self::Closed];

    /// The label value, and the name of `events.dropped`'s field, that stand for the reason.
    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Self::Evicted => "evicted",
            Self::Oversize => "oversize",
            Self::Closed => "closed",
        }
    }
}

/// The component every metric and event of the event ring belongs to.
const EVENTS_COMPONENT: &str = "events";

pub const EVENTS_EMITTED: Metric = Metric {
    name: "reliquary_events_emitted_total",
    component: EVENTS_COMPONENT,
    kind: Kind::Counter,
    unit: Unit::None,
    labels: &[],
    help: "Events emitted on Reliquary's targets and offered to the event ring, kept or not.",
};

pub const EVENTS_DROPPED: Metric = Metric {
    name: "reliquary_events_dropped_total",
    component: EVENTS_COMPONENT,
    kind: Kind::Counter,
    unit: Unit::None,
    labels: &[Label {
        name: "reason",
        values: &[
            Dropped::Evicted.word(),
            Dropped::Oversize.word(),
            Dropped::Closed.word(),
        ],
    }],
    help: "Events the event ring let go before its sink took them, by reason (evicted: the oldest, to make room for a newer one; oversize: longer than the ring's line bound; closed: emitted after the ring closed).",
};

pub const EVENTS_HELD_BYTES: Metric = Metric {
    name: "reliquary_events_held_bytes",
    component: EVENTS_COMPONENT,
    kind: Kind::Gauge,
    unit: Unit::Bytes,
    labels: &[],
    help: "Bytes of event lines the event ring holds for its sink now.",
};

pub const EVENTS_BUDGET_BYTES: Metric = Metric {
    name: "reliquary_events_budget_bytes",
    component: EVENTS_COMPONENT,
    kind: Kind::Gauge,
    unit: Unit::Bytes,
    labels: &[],
    help: "The most bytes of event lines the event ring holds: its lines times its line bound.",
};

pub const EVENTS_DROPPED_EVENT: Event = Event {
    target: "reliquary.events",
    name: "events.dropped",
    component: EVENTS_COMPONENT,
    level: Level::Warn,
    fields: &[
        Field {
            name: Dropped::Evicted.word(),
            values: Values::Count,
        },
        Field {
            name: Dropped::Oversize.word(),
            values: Values::Count,
        },
        Field {
            name: Dropped::Closed.word(),
            values: Values::Count,
        },
    ],
    help: "The event ring let events go since its sink's previous batch: the first line of the next batch, with how many for each reason.",
};

/// Every metric Reliquary exposes.
pub const METRICS: &[Metric] = &[
    EVENTS_EMITTED,
    EVENTS_DROPPED,
    EVENTS_HELD_BYTES,
    EVENTS_BUDGET_BYTES,
];

/// Every event Reliquary emits.
pub const EVENTS: &[Event] = &[EVENTS_DROPPED_EVENT];

#[cfg(test)]
mod tests {
    use super::*;

    /// Read the drop counter's label values and the marker's field names → each reason's word, in
    /// `Dropped::ALL`'s order: the counter, the event and the ring speak of one set
    /// (AC-RQ0.9.1).
    #[test]
    fn ac_rq0_9_1_the_drop_reasons_are_one_set_in_the_counter_and_the_event() {
        let words: Vec<&str> = Dropped::ALL.iter().map(|d| d.word()).collect();
        let label: Vec<&str> = EVENTS_DROPPED.labels[0].values.to_vec();
        let fields: Vec<&str> = EVENTS_DROPPED_EVENT.fields.iter().map(|f| f.name).collect();
        assert_eq!(label, words);
        assert_eq!(fields, words);
    }

    /// Ask each metric for its family and series → a counter's family drops `_total`, a gauge's is
    /// its name, and the drop counter has one series a reason (AC-RQ0.9.1).
    #[test]
    fn ac_rq0_9_1_a_metrics_family_and_series_follow_from_its_declaration() {
        assert_eq!(
            EVENTS_EMITTED.family(),
            EVENTS_EMITTED.name.strip_suffix("_total")
        );
        assert_eq!(EVENTS_HELD_BYTES.family(), Some(EVENTS_HELD_BYTES.name));
        assert_eq!(EVENTS_DROPPED.series(), Some(Dropped::ALL.len()));
        assert_eq!(EVENTS_EMITTED.series(), Some(1));
        let unnamed = Metric {
            name: "events_emitted",
            ..EVENTS_EMITTED
        };
        assert_eq!(unnamed.family(), None);
    }
}
