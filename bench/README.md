# Benchmarks and their records

What Reliquary's performance claims rest on (CLAUDE.md §7), measured by `reliquary bench` (plan RQ0.7;
the design is [`docs/design/measurement.md`](../docs/design/measurement.md)).

- **`records/`**: the records of runs, one JSON file a run, written by `--record bench/records`. Each
  names its hardware, the commit of every build it measured, its date and the load it ran under, and
  says whether it is **of record**: only when the one-minute load average, less the run's own share,
  stayed under 2 at every reading (CLAUDE.md §7) and every build names a clean commit. An acceptance
  criterion cites only a record of record; `crates/reliquary-bench/tests/records.rs` fails the gate
  on a citation of any other (AC-RQ0.7.4).
- **`gated.toml`**: the benches whose instructions an operation CI gates on every change, the change
  (B) against its base (A), interleaved, counted by cachegrind (AC-RQ0.7.6; `.kiln/ci.yaml`,
  `instructions`). A bench whose B rises, its interval separated above A's, fails the gate unless the
  change's commit message carries `Bench-accepted: <bench>: <why>`.
- **`changes.toml`**: the change points the nightly run's detector found that a person has looked at,
  each by its commit range with why. The nightly run (`.kiln/nightly.yaml`) feeds each night's latency
  to kiln's metrics, reads back the series, and alerts on a change for the worse that is not here
  (AC-RQ0.7.6).

## Making a record of record

```sh
bash scripts/bench-build.sh <commit> ~/.cache/reliquary-bench/b    # a release build naming its commit
~/.cache/reliquary-bench/b/reliquary bench run judge --metric latency:p99 --load power \
  --machine <name> --record bench/records
```

On a quiet machine (CLAUDE.md, "Working here": fedora or jupiter, in a slot of kiln's bench calendar),
never one that is building. A run on a busy machine still writes its record, marked not of record
with the load that made it so; it can be kept as evidence and is never cited.

`reliquary bench ab` compares two builds (`--a` and `--b`, each built by the script) or two settings
of one (`--a-set`, `--b-set`), and gives "A better", "B better" or "not separated" with both intervals.
