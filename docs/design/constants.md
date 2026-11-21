# Operating constants

Every number Reliquary's production code runs with has a row here: its file and key, its kind, the
value the code has, and what establishes it. The rule is the person's, in [`numbers.md`](numbers.md).
This is Mantle's inventory (mantle `docs/design/constants.md`, research/01 §5) kept for Reliquary's
code, and its gate is Mantle's checker, `scripts/check-contracts.py`, with the differences below
(decision D7; plan RQ0.2).

## Kinds

A row's kind is one the rule accepts:

- **format**: a field width, tag, version or length of an on-disk or wire encoding. Changing it
  changes the format.
- **external**: fixed by a specification or an interface (an RFC, the OCI specifications, an
  operating system's interface, an algorithm's definition), or a stated property of a system
  Reliquary must meet.
- **derived**: computed by a stated model from measured inputs or structural facts. A value derived
  from a refused one is `inherited`, not derived.
- **cited**: prescribed by peer-reviewed work or a primary source, used as it prescribes.
- **learned**: measured by the running system.
- **counted**: bytes or items a structure allocates, counted and pinned by a test: a property of the
  code, not of a machine.

or one it refuses, which fails the gate:

- **picked**: set by hand, or a bound or a floor taken as the operating value.
- **knee**: read off one measured curve by a chosen criterion and frozen.
- **once**: measured once, on one machine.
- **copied**: another system's tuning.
- **guessed**: a guessed timeout, deadline or wait.
- **inherited**: derived from a refused input.
- **open**: nothing establishes it yet.

## What a row says

A row is `` | `file` `key` | kind | `value` | basis | ``. Its value is the code's, token for token; a
value longer than a hundred characters is stated as the SHA-256 of its tokens (`sha256:`). Its basis
says what establishes the number, and for three kinds says it in labelled parts the gate reads:

- **learned**: `Measures:` what is measured; `On:` what it is measured on and kept under, a path, a
  device, a peer, a repository or a workload class, never the machine alone; `Kept:` where the
  measurement is kept; `Follows:` how it follows a change (AC-RQ0.2.3).
- **derived**: `Model:` the model; `Inputs:` its inputs; `Test:` `` `file` `function` ``, a `#[test]`
  that computes the value from them and names the constant (AC-RQ0.2.4). A derived number is therefore
  a named constant.
- **counted**: `Test:` `` `file` `function` ``, the `#[test]` that counts and pins it.

A row names its number by file and key. A constant's key is its name, after any scopes it is declared
in (`Type::NAME` in an impl). A literal's key is `Type::function:slot`: a struct field, an enum
variant, a binding or a command-line flag (`--seconds`) by name; a call's argument with its call
(`take(4)`, `from_millis(500)`); a size shift in parentheses (`(1 << 20)`); a literal array by its
text; any other literal by its value (`2`, `0.975`). `#2` and `#3` number the second and third of one
key in one scope.

## What the gate reads, and what it refuses

`scripts/check-contracts.py` reads every Rust file of `crates/` outside a crate's `tests/`, `benches/`
and `examples/` (a build script included), and each vendored path the "Vendored crates" section below
names. In them it reads:

- each `const` and `static` holding a number, of any type, and each of a numeric type however it is
  computed; a cell (an atomic, a lock, a lazy value) is filled at run time, so its literals are read
  where they stand;
- each of Mantle's literal parameters: a number written as the whole value of a struct field, an
  assignment or a binding, a count, duration or fallback argument (`with_capacity`, `channel`, `take`,
  `Duration::from_*`, `min`, `max`, `unwrap_or` and the rest of its `CALLS`), a command-line default
  or a size shift, and a literal array (a ladder, a table) outside a constant;
- and, past Mantle's checker, every other literal but 0 and 1, wherever it stands: an operand, a
  comparison, a pattern, an array's length, an enum's discriminant, any call's argument. An
  estimator's figures stand in its arithmetic (`srtt / 8`), where Mantle's checker does not read them
  yet, and RQ0.8 holds each of them to a row (AC-RQ0.8.6).

It leaves out zero as a parameter's whole value (the empty count, offset or sum); one as the argument
of `min`, `max`, `clamp`, `unwrap_or` or `map_or`, or as a `let` binding's value (at least one; a
count's or a product's first value); and 0 and 1 anywhere else (an increment, a first index, an
identity). Test code is not production code: an item under a `cfg` that requires `test` or marked
`#[test]`, a file whose first item is `#![cfg(test)]`, and a file every declaration of which is test
code.

It fails when a number has no row; when a row names no number; when a row's value is not the code's,
showing both; when a row's kind is one the rule refuses; when a row of an accepted kind states a basis
the rule refuses: one machine's measurement, a point read off one curve, a picked value, or another
system's tuning in a row other than `format` or `external` without the primary source that measured
it; when a `learned`, `derived` or `counted` row lacks a part above, or its test does not exist or
does not name its constant; and when `scripts/arbitrary-open.txt` lists anything (AC-RQ0.2.1,
AC-RQ0.2.2).

## Nothing is forgiven

Mantle's gate forgives the open items its list names, and the list only shrinks. Reliquary started
with no numbers and forgives none: `scripts/arbitrary-open.txt` stays empty, the gate refuses an item
in it, and the checker has no `--known`.

## Where the gate runs

`scripts/gates.sh` runs the checker first, as Mantle's gate does (plan RQ0.3), and
crates/reliquary/tests/contracts.rs runs it over the whole tree inside the suite, so it holds wherever
the suite runs. CLAUDE.md's Gates lines name it once the person approves them (AC-RQ0.3.6).

## Vendored crates

A vendored crate is its owner's to inventory. Mantle's inventory holds the hyper-raft crates on
Mantle's write path; aws-lc-rs is cryptography as published (plan principle 6: other products are asked,
not edited). Reliquary's gate reads a vendored path only where this section names it in a bullet
(`` - `vendor/...`: why ``), as Mantle's checker reads the crates its `VENDORED` names. A path joins
the list, with a row for each number in it, when Reliquary answers for the numbers there:
- an estimator RQ0.8 takes from hyper-timing, Shards or Mantle rather than write a second
  (AC-RQ0.8.6: every figure inside an estimator has a row here);
- Rust that a change Reliquary itself records in `vendor/patches/` adds (CLAUDE.md §1: vendored code
  Reliquary changes follows the same rule). Today that is `vendor/patches/amphora/0002`, which adds no
  number.

None is named yet. Since nothing is forgiven here, a number of a refused kind in a named path is
replaced, by its owner or by a change Reliquary records, before the path joins. Mantle names 85 of
the hyper-raft crates' numbers open at its own revision of them (mantle `scripts/arbitrary-open.txt` at
f41d8344: 55 in hyper-quic, 15 in hyper-rt, 3 in hyper-timing).

## crates/reliquary

| Number | Kind | Value | Basis |
|---|---|---|---|
| `crates/reliquary/src/lib.rs` `MISUSE` | external | `2` | The status for refused arguments, which shells and their scripts read as misuse: "All builtins return an exit status of 2 to indicate incorrect usage, generally invalid options or missing arguments" (Bash Reference Manual §3.7.5, "Exit Status", https://www.gnu.org/software/bash/manual/html_node/Exit-Status.html). crates/reliquary/tests/cli.rs pins it. |

## crates/reliquary-api

| Number | Kind | Value | Basis |
|---|---|---|---|
| `crates/reliquary-api/src/grammar.rs` `TAG_MAX_BYTES` | external | `128` | A tag's most characters: "`<reference>` as a tag MUST be at most 128 characters in length" (distribution spec v1.1.1, "Pulling manifests"; research/02 §1). Every character the tag grammar allows is ASCII, so characters and bytes count alike. crates/reliquary-api/tests/model.rs `the_grammars_agree_with_the_specs_expressions` holds the grammar to the spec's expression. |
| `crates/reliquary-api/src/grammar.rs` `HEX_DIGITS_PER_BYTE` | external | `2` | Base 16 writes each byte as two digits (RFC 4648 §8), and the image spec writes a registered digest's encoded part in lowercase hex: "When the _algorithm identifier_ is `sha256`, the _encoded_ portion MUST match `/[a-f0-9]{64}/`", 64 digits for SHA-256's 32 bytes (image spec v1.1.1, descriptor.md, "Registered algorithms"). |
| `crates/reliquary-api/src/generate.rs` `OWNER` | format | `0` | The number generated histories give the principal that owns every repository they name; the others are 1 to n. A history kept as a regression test names its principals by these numbers (plan AC-RQ1.11.5), so changing it changes what kept histories mean. |

## crates/reliquary-bench

The measurement tool's numbers (plan RQ0.7, `docs/design/measurement.md`): the statistics' definitions,
their sources' levels, and the formats it reads and writes. Its sources are in research/07.

| Number | Kind | Value | Basis |
|---|---|---|---|
| `crates/reliquary-bench/src/rounds.rs` `MAX_ROUNDS` | derived | `122` | Model: the exact binomial sums of the median's interval and of the runs test are counted in a `u128` and compared as `TAILS·sum ≤ 2^n`, so `n` is at most the largest with `TAILS·2^n` below `2^128`. Inputs: a `u128`'s 128 bits; `TAILS`. Test: `crates/reliquary-bench/src/rounds.rs` `max_rounds_is_the_most_whose_exact_sums_fit` |
| `crates/reliquary-bench/src/rounds.rs` `TAILS` | cited | `40` | Each tail of a two-sided test at 5%, 2.5%, one part in forty: the median's order-statistic interval (Le Boudec, *Performance Evaluation of Computer and Communication Systems*, Theorem 2.1) and the runs test's two-sided region (NIST/SEMATECH e-Handbook §1.3.5.13). |
| `crates/reliquary-bench/src/rounds.rs` `TAIL` | cited | `0.025` | The same tail as a probability, each end of Clopper and Pearson's interval of a state's share (Le Boudec, Theorem 2.4). |
| `crates/reliquary-bench/src/rounds.rs` `Z_95` | cited | `1.96` | The standard normal's 97.5% point: the `±1.96/√n` bound of an independent series' lag-1 autocorrelation (NIST/SEMATECH e-Handbook §1.3.3.1; Le Boudec §2.3.2), and the `z` of Jain's sample size that sizes a round (docs/research/07 F38). |
| `crates/reliquary-bench/src/rounds.rs` `FEWEST_TESTED` | derived | `10` | Model: the exact distribution of runs above and below the median for `n/2` rounds a side; the fewest `n` with a two-sided 5% rejection region. Inputs: the tail, `TAILS`. Test: `crates/reliquary-bench/src/rounds.rs` `fewest_tested_is_where_the_runs_test_can_first_reject` |
| `crates/reliquary-bench/src/rounds.rs` `PRECISION` | cited | `0.05` | ±5%, half of the 10% steady-state excursion of SNIA's Solid State Storage Performance Test Specification 2.0.2 §2.1.24, the precision Mantle's measurement design states a point to (docs/research/07 F38). |
| `crates/reliquary-bench/src/rounds.rs` `runs:2` | external | `2` | The runs test's exact distribution counts runs as `2k` or `2k + 1` (NIST/SEMATECH e-Handbook §1.3.5.13): `k` is half the count. |
| `crates/reliquary-bench/src/rounds.rs` `runs:is_multiple_of(2)` | external | `2` | Whether a count of runs is even, `2k`, in the same distribution. |
| `crates/reliquary-bench/src/rounds.rs` `runs:checked_mul(2)` | external | `2` | `P(R = 2k) ∝ 2·C(a−1, k−1)·C(b−1, k−1)`: either side may begin the `2k` runs (the same distribution). |
| `crates/reliquary-bench/src/rounds.rs` `runs:2#2` | external | `2` | The fewest runs rounds on both sides of the median make: two. |
| `crates/reliquary-bench/src/rounds.rs` `DIP_95` | cited | `sha256:291ba9db75193574` | The dip's 95% points for 4 to 200 observations, Hartigan and Hartigan, "The Dip Test of Unimodality", Ann. Statist. 13(1), 1985, Table 1 (9,999 uniform samples each). |
| `crates/reliquary-bench/src/tails.rs` `COVERAGE` | cited | `match Level::new(95, 100) { Ok(level) => level, Err(_) => Level::P50, }` | Every interval at 95%: Le Boudec's Theorem 2.1 and David and Nagaraja's distribution-free quantile interval, the coverage Mantle's measurement design states (docs/research/07 F38). |
| `crates/reliquary-bench/src/gate.rs` `runs_bench:misuse` | external | `2` | The status the `reliquary` binary refuses arguments with, which a base build that does not know a bench, or has no `bench` command, answers (`crates/reliquary/src/lib.rs` `MISUSE`; Bash Reference Manual §3.7.5, "Exit Status"). |
| `crates/reliquary-bench/src/changes.rs` `PERMUTATIONS` | cited | `199` | The shuffles a change point is tested against: `e.divisive`'s default `R` in `ecp`, the package of E-Divisive's authors, whose p-value resolution, 1/200, is a tenth of the level (Matteson and James 2014; docs/research/07 F6). |
| `crates/reliquary-bench/src/changes.rs` `LEVEL` | cited | `0.05` | The level a change point must reach: Matteson and James's `p0` and `ecp`'s `sig.lvl` (docs/research/07 F4, F6). |
| `crates/reliquary-bench/src/changes.rs` `MOST_POINTS` | external | `366` | One point a UTC day over kiln's widest window of a metric's history: `days` is clamped to 1..=365 (kiln `crates/kiln/src/ci/results.rs`, `repo_metrics`), which spans at most 366 UTC days. |
| `crates/reliquary-bench/src/changes.rs` `DAY_SECONDS` | external | `86_400` | Seconds in a day of Unix time (POSIX.1-2017, Base Definitions §4.16, "Seconds Since the Epoch"). |
| `crates/reliquary-bench/src/changes.rs` `best_split:2.0` | external | `2.0` | E-Divisive's `Ê`: twice the mean distance between the two sides (Matteson and James 2014, §2.1; docs/research/07 F1). |
| `crates/reliquary-bench/src/changes.rs` `best_split:2.0#2` | external | `2.0` | The first side's pairs, `C(m, 2) = m(m − 1)/2`, in `Ê`'s mean distance within it (same). |
| `crates/reliquary-bench/src/changes.rs` `best_split:2.0#3` | external | `2.0` | The second side's pairs, `C(n, 2) = n(n − 1)/2` (same). |
| `crates/reliquary-bench/src/changes.rs` `seed::OFFSET` | external | `0xcbf2_9ce4_8422_2325` | FNV-1a's 64-bit offset basis (Fowler, Noll and Vo; IETF draft-eastlake-fnv, "The FNV Non-Cryptographic Hash Algorithm"). |
| `crates/reliquary-bench/src/changes.rs` `seed::PRIME` | external | `0x0000_0100_0000_01b3` | FNV-1a's 64-bit prime, `2^40 + 2^8 + 0xb3` (same). |
| `crates/reliquary-bench/src/changes.rs` `seed:once(0xff)` | format | `0xff` | The separator between points in the seed's input: an octet no UTF-8 text holds ("The octet values C0, C1, F5 to FF never appear", RFC 3629 §1), so no commit name runs into the next. |
| `crates/reliquary-bench/src/driver.rs` `POWER_SHARE` | derived | `0.5` | Model: an M/M/1 queue's power, throughput over delay, `λ/T = λ(μ − λ)`, is greatest at `λ = μ/2` (Kleinrock, "Power and Deterministic Rules of Thumb for Probabilistic Problems in Computer Communications", IEEE ICC 1979). Inputs: `μ`, the closed-loop rate measured in the same session. Test: `crates/reliquary-bench/src/driver.rs` `the_power_share_is_where_an_mm1_queues_power_peaks` |
| `crates/reliquary-bench/src/driver.rs` `DRAIN` | derived | `0.5` | Model: as a round's last `N` operations finish one by one, the depth falls `N, N − 1, …, 1`, short of `N` by `(N − 1)/2` operations on average, at most `N/2` (docs/research/07 F38). Inputs: `N`, the operations in flight. Test: `crates/reliquary-bench/src/driver.rs` `the_drain_share_is_a_falling_depths_average_shortfall` |
| `crates/reliquary-bench/src/driver.rs` `SERIAL` | counted | `1` | A worker runs its operations one after another, so one is in flight. Test: `crates/reliquary-bench/src/worker.rs` `operations_run_one_at_a_time` |
| `crates/reliquary-bench/src/driver.rs` `MEDIAN` | format | `(1, 2)` | The median as a quantile, one half. |
| `crates/reliquary-bench/src/driver.rs` `PAD_MOST` | cited | `4_096` | The most environment a process start adds: Mytkowicz et al. (ASPLOS '09) varied the UNIX environment's size up to 4,096 bytes, over which it moved a program's stack alignment and speed (docs/research/07 F39). |
| `crates/reliquary-bench/src/cli.rs` `DECIMAL` | format | `10` | The base of a percentile's decimal places: p99.9 is 999 parts of 100 × 10. |
| `crates/reliquary-bench/src/report.rs` `PERCENT` | format | `100` | A percentage's whole. |
| `crates/reliquary-bench/src/report.rs` `UNITS` | external | `&[(1e9, "s"),(1e6, "ms"),(1e3, "µs")]` | Nanoseconds in a second, a millisecond and a microsecond (the SI's prefixes: BIPM, *The International System of Units*, 9th edition, Table 7). |
| `crates/reliquary-bench/src/worker.rs` `NS_PER_SECOND` | external | `1e9` | Nanoseconds in a second (the SI's nano, BIPM, 9th edition, Table 7). |
| `crates/reliquary-bench/src/load.rs` `LIMIT` | external | `2.0` | The person's load rule for a record of record: "load average under 2, nothing else building" (CLAUDE.md §7; kiln's master plan §3). |
| `crates/reliquary-bench/src/load.rs` `WINDOW_NS` | external | `60e9` | The one-minute load average's window: Linux decays it by `exp(−5/60)` every 5 seconds (`kernel/sched/loadavg.c`, `EXP_1`; docs/research/07 F26). |
| `crates/reliquary-bench/src/load.rs` `UPDATE_NS` | external | `5_000_000_000` | How often the kernel updates the load average: every 5 seconds (`include/linux/sched/loadavg.h`, `LOAD_FREQ`; docs/research/07 F26). `reliquary bench settle` reads it once an update. |
| `crates/reliquary-bench/src/record.rs` `FORMAT` | format | `1` | The record's layout, raised when a field changes meaning. |
| `crates/reliquary-bench/src/record.rs` `KIB` | external | `1_024` | `/proc/meminfo`'s unit: Linux writes its counts in kB of 1,024 bytes (`fs/proc/meminfo.c`, `show_val_kb`: pages shifted by `PAGE_SHIFT − 10`). |
| `crates/reliquary-bench/src/record.rs` `EPOCH_SHIFT` | external | `719_468` | Days from 0000-03-01 to 1970-01-01, the algorithm counting years from March so a leap day ends its year (Hinnant, "chrono-Compatible Low-Level Date Algorithms", `civil_from_days` (the proleptic Gregorian calendar)); `record.rs`'s `dates_are_the_gregorian_calendars` holds the dates it gives against the Unix seconds of known days. |
| `crates/reliquary-bench/src/record.rs` `ERA_DAYS` | external | `146_097` | Days in a Gregorian era of 400 years: 400 × 365 + 97 leap days (Hinnant, "chrono-Compatible Low-Level Date Algorithms", `civil_from_days` (the proleptic Gregorian calendar)). |
| `crates/reliquary-bench/src/record.rs` `ERA_YEARS` | external | `400` | Years in a Gregorian era, over which the calendar repeats (the Gregorian rule: every fourth year a leap year, but not a century's, unless the century divides by 400). |
| `crates/reliquary-bench/src/record.rs` `YEAR_DAYS` | external | `365` | Days in a common Gregorian year. |
| `crates/reliquary-bench/src/record.rs` `FOUR_YEARS_LESS_ONE` | external | `1_460` | Days in four years less one, where a day of the era crosses a leap day (Hinnant, "chrono-Compatible Low-Level Date Algorithms", `civil_from_days` (the proleptic Gregorian calendar)). |
| `crates/reliquary-bench/src/record.rs` `CENTURY_LESS_ONE` | external | `36_524` | Days in a century less one, where a day of the era crosses a century's missing leap day (Hinnant, "chrono-Compatible Low-Level Date Algorithms", `civil_from_days` (the proleptic Gregorian calendar)). |
| `crates/reliquary-bench/src/record.rs` `ERA_LESS_ONE` | external | `146_096` | Days in an era less one, where a day of the era crosses the era's own leap day (Hinnant, "chrono-Compatible Low-Level Date Algorithms", `civil_from_days` (the proleptic Gregorian calendar)). |
| `crates/reliquary-bench/src/record.rs` `LEAP_YEARS` | external | `4` | Years in the Gregorian leap cycle. |
| `crates/reliquary-bench/src/record.rs` `CENTURY_YEARS` | external | `100` | Years in a century, whose year is no leap year unless it divides by 400 (the Gregorian rule). |
| `crates/reliquary-bench/src/record.rs` `FIVE_MONTHS_DAYS` | external | `153` | Days in the five months from March to July, 31 + 30 + 31 + 30 + 31, which make the line `(153·m + 2)/5` through each month's first day of a year counted from March (Hinnant, "chrono-Compatible Low-Level Date Algorithms", `civil_from_days` (the proleptic Gregorian calendar)). |
| `crates/reliquary-bench/src/record.rs` `FIVE_MONTHS` | external | `5` | The five months of that line (Hinnant, "chrono-Compatible Low-Level Date Algorithms", `civil_from_days` (the proleptic Gregorian calendar)). |
| `crates/reliquary-bench/src/record.rs` `FIVE_MONTHS_OFFSET` | external | `2` | The line's offset, which puts each month's first day on it (Hinnant, "chrono-Compatible Low-Level Date Algorithms", `civil_from_days` (the proleptic Gregorian calendar)). |
| `crates/reliquary-bench/src/record.rs` `JANUARY_FROM_MARCH` | external | `10` | January's place in a year counted from March, March being 0. |
| `crates/reliquary-bench/src/record.rs` `MARCH` | external | `3` | March's number in the calendar, which month 0 of a year counted from March is. |
| `crates/reliquary-bench/src/record.rs` `MARCH_TO_DECEMBER` | external | `9` | The months from March to December: a month counted from March at or past January is the calendar's month less 9. |
| `crates/reliquary-bench/src/record.rs` `FEBRUARY` | external | `2` | February's number: a date in January or February belongs to the calendar year after the one counted from March. |
| `crates/reliquary-bench/src/record.rs` `HOUR_SECONDS` | external | `3_600` | Seconds in an hour of Unix time (POSIX.1-2017, Base Definitions §4.16, "Seconds Since the Epoch"). |
| `crates/reliquary-bench/src/record.rs` `MINUTE_SECONDS` | external | `60` | Seconds in a minute of Unix time (same). |
| `crates/reliquary-bench/src/rng.rs` `GOLDEN` | external | `0x9e37_79b9_7f4a_7c15` | SplitMix64's increment, `2^64` over the golden ratio, made odd (Steele, Lea and Flood, "Fast Splittable Pseudorandom Number Generators", OOPSLA 2014; Vigna's `splitmix64.c`). |
| `crates/reliquary-bench/src/rng.rs` `SplitMix64::next_u64:30` | external | `30` | SplitMix64's first shift (Vigna's `splitmix64.c`, the mixer of Steele et al., OOPSLA 2014). |
| `crates/reliquary-bench/src/rng.rs` `SplitMix64::next_u64:wrapping_mul(0xbf58_476d_1ce4_e5b9)` | external | `0xbf58_476d_1ce4_e5b9` | SplitMix64's first multiplier (same). |
| `crates/reliquary-bench/src/rng.rs` `SplitMix64::next_u64:27` | external | `27` | SplitMix64's second shift (same). |
| `crates/reliquary-bench/src/rng.rs` `SplitMix64::next_u64:wrapping_mul(0x94d0_49bb_1331_11eb)` | external | `0x94d0_49bb_1331_11eb` | SplitMix64's second multiplier (same). |
| `crates/reliquary-bench/src/rng.rs` `SplitMix64::next_u64:31` | external | `31` | SplitMix64's third shift (same). |
| `crates/reliquary-bench/src/rng.rs` `SplitMix64::below:64` | external | `64` | The top half of the 128-bit product of a 64-bit draw and `n` (Lemire, "Fast Random Integer Generation in an Interval", ACM TOMACS 29(1), 2019, §2). |
| `crates/reliquary-bench/src/rng.rs` `SplitMix64::unit:11` | format | `11` | 64 less 53: a draw's top 53 bits fill an IEEE 754 binary64's significand (IEEE 754-2019 §3.6). |
| `crates/reliquary-bench/src/rng.rs` `SplitMix64::unit:9_007_199_254_740_992.0` | format | `9_007_199_254_740_992.0` | `2^53`, so 53 bits make a fraction in [0, 1) (IEEE 754-2019 §3.6). |

## crates/reliquary-sim

The simulated world's numbers (plan RQ0.6, `docs/design/sim.md`). Every figure of a run, its
paths, bounds, budgets and faults, is its maker's: the world states none.

| Number | Kind | Value | Basis |
|---|---|---|---|
| `crates/reliquary-sim/src/clock.rs` `WALL_GRID_NS` | external | `100` | The resolution of the wall time every target can say: on Windows Rust's `SystemTime` is read with `GetSystemTimePreciseAsFileTime` (the standard library's `SystemTime`, "Platform-specific behavior"), a `FILETIME`, which "Contains a 64-bit value representing the number of 100-nanosecond intervals since January 1, 1601 (UTC)." (Microsoft, `FILETIME` structure, https://learn.microsoft.com/en-us/windows/win32/api/minwinbase/ns-minwinbase-filetime; research/12 F11). A wall reading put on this grid is the same `SystemTime` on every target, so a run's grants and leases expire alike everywhere (plan AC-RQ0.6.1); `crates/reliquary-sim/tests/determinism.rs` holds one seed's log to the same bytes on each. |
