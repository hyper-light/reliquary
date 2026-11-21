# Records

One JSON file a run, named `<date>-<bench>-<kind>.json` (`kind`: `point`, `ab` or `gate`), written by
`reliquary bench … --record bench/records` and never written over. A record says its standing at its
top, `"standing": {"standing": "of record"}` or `"not of record"` with why; only a record of record is
cited by an acceptance criterion (AC-RQ0.7.4; `crates/reliquary-bench/tests/records.rs`).
