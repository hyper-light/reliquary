//! The metrics components report, as the OpenMetrics 1.0 text a scrape reads (OpenMetrics 1.0,
//! "Text format"; `docs/design/observability.md`, O3): each family's `# TYPE`, its `# UNIT` when
//! it has one, and its `# HELP`, then every series, and `# EOF` at the end. kiln's scrape asks
//! for this format first, as Prometheus's does.
//!
//! Every family is checked before any is written, and a family whose series don't match its
//! catalog entry refuses the whole exposition: "Partial or invalid expositions MUST be considered
//! erroneous in their entirety" (OpenMetrics 1.0, "Data transmission & wire formats").

use std::fmt::Write as _;

use super::catalog::Metric;

/// What a response that carries an exposition says it is (OpenMetrics 1.0, "Overall Structure").
pub const CONTENT_TYPE: &str = "application/openmetrics-text; version=1.0.0; charset=utf-8";

/// One series of a metric: its labels' values, in the order the metric declares its labels, and
/// its value. A value is one of the label's closed set, a `'static` word of the catalog's, never
/// text a request brought.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Series {
    pub labels: Vec<&'static str>,
    pub value: u64,
}

impl Series {
    /// The one series of a metric without labels.
    #[must_use]
    pub fn of(value: u64) -> Self {
        Self {
            labels: Vec::new(),
            value,
        }
    }
}

/// A metric and its series, as a component reports them for a scrape.
#[derive(Debug, Clone)]
pub struct Family {
    pub metric: &'static Metric,
    pub series: Vec<Series>,
}

/// Why an exposition can't be written.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Refused {
    #[error("{metric} is a counter, whose name ends in _total")]
    CounterName { metric: &'static str },
    #[error("{metric} is reported twice in one exposition")]
    RepeatedFamily { metric: &'static str },
    #[error("{metric} has {want} labels, and a series of it gives {got} values")]
    LabelCount {
        metric: &'static str,
        want: usize,
        got: usize,
    },
    #[error("{metric}'s label {label} takes none of the value {value:?}")]
    LabelValue {
        metric: &'static str,
        label: &'static str,
        value: &'static str,
    },
    #[error("{metric} reports the series {labels:?} twice")]
    RepeatedSeries {
        metric: &'static str,
        labels: Vec<&'static str>,
    },
}

/// Writes `families` to the end of `out` as one exposition, ending in `# EOF`.
///
/// # Errors
///
/// [`Refused`], and nothing written, when a family is reported twice, a counter's name lacks its
/// `_total`, or a series gives the wrong number of label values, a value its label doesn't take,
/// or the label values of another series.
pub fn encode(families: &[Family], out: &mut String) -> Result<(), Refused> {
    for (i, family) in families.iter().enumerate() {
        let metric = family.metric;
        if families
            .iter()
            .take(i)
            .any(|earlier| earlier.metric.name == metric.name)
        {
            return Err(Refused::RepeatedFamily {
                metric: metric.name,
            });
        }
        check(family)?;
    }
    for family in families {
        write(family, out);
    }
    out.push_str("# EOF\n");
    Ok(())
}

/// Refuses a family whose name or series don't match its catalog entry.
fn check(family: &Family) -> Result<(), Refused> {
    let metric = family.metric;
    if metric.family().is_none() {
        return Err(Refused::CounterName {
            metric: metric.name,
        });
    }
    for (i, series) in family.series.iter().enumerate() {
        if series.labels.len() != metric.labels.len() {
            return Err(Refused::LabelCount {
                metric: metric.name,
                want: metric.labels.len(),
                got: series.labels.len(),
            });
        }
        for (label, &value) in metric.labels.iter().zip(&series.labels) {
            if !label.values.contains(&value) {
                return Err(Refused::LabelValue {
                    metric: metric.name,
                    label: label.name,
                    value,
                });
            }
        }
        if family
            .series
            .iter()
            .take(i)
            .any(|earlier| earlier.labels == series.labels)
        {
            return Err(Refused::RepeatedSeries {
                metric: metric.name,
                labels: series.labels.clone(),
            });
        }
    }
    Ok(())
}

/// Writes a family [`check`] passed: its metadata in the order OpenMetrics asks (TYPE, UNIT,
/// HELP), then a sample a series, named as its type has it: a counter's `<family>_total`, a
/// gauge's its family's.
fn write(family: &Family, out: &mut String) {
    let metric = family.metric;
    let name = metric.family().unwrap_or(metric.name);
    out.push_str("# TYPE ");
    out.push_str(name);
    out.push(' ');
    out.push_str(metric.kind.word());
    out.push('\n');
    if let Some(unit) = metric.unit.word() {
        out.push_str("# UNIT ");
        out.push_str(name);
        out.push(' ');
        out.push_str(unit);
        out.push('\n');
    }
    out.push_str("# HELP ");
    out.push_str(name);
    out.push(' ');
    escape(metric.help, out);
    out.push('\n');
    for series in &family.series {
        out.push_str(metric.name);
        if !series.labels.is_empty() {
            out.push('{');
            for (i, (label, value)) in metric.labels.iter().zip(&series.labels).enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push_str(label.name);
                out.push_str("=\"");
                escape(value, out);
                out.push('"');
            }
            out.push('}');
        }
        // A String's writes never fail.
        let _ = writeln!(out, " {}", series.value);
    }
}

/// `text` as OpenMetrics' escaped-string: a line feed, a quotation mark and a backslash escaped
/// (OpenMetrics 1.0, "Escaping").
fn escape(text: &str, out: &mut String) {
    for c in text.chars() {
        match c {
            '\n' => out.push_str("\\n"),
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c => out.push(c),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observe::catalog::{
        EVENTS_BUDGET_BYTES, EVENTS_DROPPED, EVENTS_EMITTED, Kind, Label, Unit,
    };

    fn encoded(families: &[Family]) -> Result<String, Refused> {
        let mut out = String::new();
        encode(families, &mut out).map(|()| out)
    }

    /// Encode a counter with a label, a gauge with a unit, and nothing → the exact text
    /// OpenMetrics' examples have: TYPE, UNIT, HELP, `_total` on the counter's samples, `# EOF`
    /// last (T-RQ0.9.4).
    #[test]
    fn t_rq0_9_4_families_are_written_as_openmetrics_text() {
        let text = encoded(&[
            Family {
                metric: &EVENTS_DROPPED,
                series: vec![
                    Series {
                        labels: vec!["evicted"],
                        value: 3,
                    },
                    Series {
                        labels: vec!["oversize"],
                        value: 0,
                    },
                    Series {
                        labels: vec!["closed"],
                        value: 0,
                    },
                ],
            },
            Family {
                metric: &EVENTS_BUDGET_BYTES,
                series: vec![Series::of(4096)],
            },
        ])
        .unwrap();
        let (dropped, budget) = (EVENTS_DROPPED.name, EVENTS_BUDGET_BYTES.name);
        let family = dropped.strip_suffix("_total").unwrap();
        let want = format!(
            "# TYPE {family} counter\n\
             # HELP {family} {}\n\
             {dropped}{{reason=\"evicted\"}} 3\n\
             {dropped}{{reason=\"oversize\"}} 0\n\
             {dropped}{{reason=\"closed\"}} 0\n\
             # TYPE {budget} gauge\n\
             # UNIT {budget} bytes\n\
             # HELP {budget} {}\n\
             {budget} 4096\n\
             # EOF\n",
            EVENTS_DROPPED.help, EVENTS_BUDGET_BYTES.help
        );
        assert_eq!(text, want);
        assert_eq!(encoded(&[]).unwrap(), "# EOF\n");
    }

    /// Encode a help and a label value holding a newline, a quote and a backslash → each escaped as
    /// OpenMetrics says, one line each (T-RQ0.9.4).
    #[test]
    fn t_rq0_9_4_help_and_label_values_are_escaped() {
        const ODD: Metric = Metric {
            name: "odd_total",
            component: "odd",
            kind: Kind::Counter,
            unit: Unit::None,
            labels: &[Label {
                name: "word",
                values: &["a\"b\\c\nd"],
            }],
            help: "Line one\nhas \"quotes\" and a \\.",
        };
        let text = encoded(&[Family {
            metric: &ODD,
            series: vec![Series {
                labels: vec!["a\"b\\c\nd"],
                value: 1,
            }],
        }])
        .unwrap();
        assert!(
            text.contains("# HELP odd Line one\\nhas \\\"quotes\\\" and a \\\\.\n"),
            "{text}"
        );
        assert!(
            text.contains("odd_total{word=\"a\\\"b\\\\c\\nd\"} 1\n"),
            "{text}"
        );
    }

    /// Encode a family twice, a series with a label value outside its set, one with too few values,
    /// a series twice, and a counter named without `_total` → each refused, naming the metric, and
    /// nothing written (AC-RQ0.9.2: a value the catalog doesn't list never reaches a scrape).
    #[test]
    fn ac_rq0_9_2_a_series_the_catalog_doesnt_declare_refuses_the_exposition() {
        let emitted = || Family {
            metric: &EVENTS_EMITTED,
            series: vec![Series::of(1)],
        };
        let dropped = |labels: Vec<&'static str>| Family {
            metric: &EVENTS_DROPPED,
            series: vec![Series { labels, value: 1 }],
        };
        let mut out = String::new();
        assert_eq!(
            encode(&[emitted(), emitted()], &mut out),
            Err(Refused::RepeatedFamily {
                metric: EVENTS_EMITTED.name
            })
        );
        assert_eq!(
            encode(&[dropped(vec!["library/alpine"])], &mut out),
            Err(Refused::LabelValue {
                metric: EVENTS_DROPPED.name,
                label: "reason",
                value: "library/alpine"
            })
        );
        assert_eq!(
            encode(&[dropped(vec![])], &mut out),
            Err(Refused::LabelCount {
                metric: EVENTS_DROPPED.name,
                want: 1,
                got: 0
            })
        );
        let twice = Family {
            metric: &EVENTS_DROPPED,
            series: vec![
                Series {
                    labels: vec!["evicted"],
                    value: 1,
                },
                Series {
                    labels: vec!["evicted"],
                    value: 2,
                },
            ],
        };
        assert_eq!(
            encode(&[twice], &mut out),
            Err(Refused::RepeatedSeries {
                metric: EVENTS_DROPPED.name,
                labels: vec!["evicted"]
            })
        );
        const UNNAMED: Metric = Metric {
            name: "events_emitted",
            ..EVENTS_EMITTED
        };
        assert_eq!(
            encode(
                &[Family {
                    metric: &UNNAMED,
                    series: vec![Series::of(1)]
                }],
                &mut out
            ),
            Err(Refused::CounterName {
                metric: UNNAMED.name
            })
        );
        assert!(out.is_empty(), "{out}");
    }
}
