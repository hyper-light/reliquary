# Measuring Reliquary

Status: built 2026-11-20 (plan RQ0.7; architecture D23). Sources: Mantle's measurement design at
`f41d8344` (research/07 S40, F38, the design followed), research/07 (change points, instruction counts,
the load average, layout bias, interleaved trials, interval overlap, open loops), research/06 F48 to
F51, research/01 §5. The code is `crates/reliquary-bench`; `reliquary bench` runs it.

Every performance claim Reliquary makes rests on a record this tool writes (CLAUDE.md §7), and every
design choice with candidates is decided by its A|B (plan README, "Data decides"). So the tool must
state what it measured with intervals that mean what they say, refuse to state what it cannot, and give
a verdict between two builds only when the data separate them.

## M1. A point is judged as Mantle's design judges it

**Decision.** A point is measured in units (rounds, or process starts, or builds: M3), one value each.
Before any interval, the units are tested for order in time: the lag-1 autocorrelation against
`±1.96/√n` and the exact runs test above and below the median, each at 5%; ordered units carry no
interval. Then Hartigan and Hartigan's dip test says one state or two, split where the within-group sum
of squares is least. Each state reports its median with Le Boudec's order-statistic 95% interval, its
units out of the point's, its share with Clopper and Pearson's interval when there are two, and its
latency quantiles over its own operations with their distribution-free intervals; a state without an
interval, and a quantile its operations cannot support, show a dash; units ordered in time state no
interval of a share or a tail either. No mean and no coefficient of variation is reported (Mantle's
design §2 to §6; research/07 F38). `rounds.rs` carries
`mantle_disk::rounds` at Mantle `87ab09e1` with its reference values, until Mantle publishes it as a
crate (the plan's request to Mantle).

Two numbers differ from Mantle's code, each to the rule on numbers: a point runs at most 28 units,
`fewest_for_two_states`, which Mantle's design derives and its code at that revision still holds at 30;
and the most units the exact sums hold is 122, the largest `n` with `40·2^n` in a `u128`, where Mantle's
code holds a round 120 beneath it (`docs/design/constants.md`).

**Evidence.** `rounds.rs`'s tests: the dip against seventeen reference values from R's `diptest` and a
linear program, the median's ranks against Le Boudec's Table A.1, the runs test's rejection regions at
10, 20, 30 and 50, two states found and split (T-RQ0.7.2), drifting units ordered with no interval
(T-RQ0.7.3). `tails.rs`: a p99.9 of 3,687 operations is a dash and of 3,688 a number (T-RQ0.7.4).
`drive.rs`: over 200 simulated runs of a known distribution the p50, p99 and p99.9 intervals miss the
true quantile within their 5% (T-RQ0.7.1).

**Falsified by.** A reference sample whose dip, ranks or runs region differs from the published; a
report that states a number where its state's operations cannot support one.

## M2. A round is a count of operations

**Decision.** A round runs the most of: `⌈fewest(q)/10⌉`, so the report's highest quantile has its
interval after the fewest units (6 operations for a median, 368 for a p99, 3,688 for a p99.9);
`⌈N·½/precision⌉`, so the operations still in flight as a round ends cost it at most the precision;
for a latency metric, whose round is valued at its own quantile, the operations that quantile needs
within the round (368 for a p99), so a round's "p99" is never its maximum; and, from a sizing round not
kept, `⌈(1.96·cv/precision)²⌉`, Jain's sample size from the operations' spread (Mantle's design §5a;
research/07 F38). A process's first round is discarded (Georges et al.). The record states each term.
A process's CPU time is not a metric: Linux and Windows advance it by a scheduler's tick, 10 and 15.6
ms, so a round of microseconds reads none (measured on jupiter, 2026-11-20).

**Falsified by.** A point whose units settle only because its rounds were made longer than these terms
ask, or one whose quantile is stated with fewer operations than its interval needs.

## M3. How many builds, process starts and rounds follows from the variation at each level

**Decision.** A dimensioning experiment repeats every level the fewest times its order in time can be
tested, ten rounds in each of ten process starts, and every build given when there are several. From it
Kalibera and Jones's estimators give each level's own variance `T_i²`, and each level but the top is
repeated `⌈√((c_{i+1}/c_i)·(T_i²/T_{i+1}²))⌉` times, `c_i` the time a repetition at that level adds by
itself, measured in the same run (KJ13 Eq. 1 to 3; Mantle's research/21 §2.4). The top level, the
highest with variation of its own, is repeated until its units are independent and every state's
interval is within the precision, from 10 to 28 units; the interval is stated from its units alone
(KJ13 Eq. 4), each the median of the units below it. A level that adds no variation is not repeated.
A|B dimensions both arms: the higher of their top levels is repeated until settled, each level below it
runs the larger of the arms' counts, and neither arm is repeated less than its own plan asks. Builds as
the top level are measured once each, as many as were given. Each process start adds 0 to 4,095
bytes to its environment, drawn anew, the range over which Mytkowicz et al. found a program's stack
alignment, and speed, to move (research/07 F30, F31, F39). Builds are a level only when several are
given, with the time a build takes stated (`--build-seconds`); one build is one sample of code layout
however often it runs (STABILIZER, research/07 F32), so a record of one build says its interval covers
process starts and rounds only. Every count and why is in the record: no "best of N" has a picked N.

**Evidence.** `levels.rs`: Eq. 3 reproduces KJ13's Table 5 (bloat6: 10); the estimators recover a
seeded experiment's variances; variation between process starts and not rounds repeats process starts
(T-RQ0.7.9), variation between rounds alone keeps one process. `driver.rs` holds the same through the
whole driver on simulated workers.

**Falsified by.** A point whose interval at the top level fails to cover a rerun's median more often than
its 95% says, which would mean a level's variation went unrepeated.

## M4. A|B: interleaved, randomized, a verdict only when intervals separate

**Decision.** Two builds, or two settings of one, run in cycles of one unit each, the order drawn anew for
each cycle (Abedi and Brecht's randomized multiple interleaved trials, research/07 F33), on one machine
in one session. Each arm is judged alone (M1). The verdict is "A better" or "B better" only when each
arm's units are independent and in one state with an interval, and the two intervals do not meet;
otherwise "not separated", with why, both intervals beside it. No mean is computed, so no verdict can
come from one.

Requiring two 95% intervals not to meet is a test at a level far below 5%: about 0.56% for normal,
equal-error estimates (Payton, Greenstone and Schenker, after Schenker and Gentleman; research/07 F35,
F36). So an A|A run reports a difference less often than the 5% AC-RQ0.7.3 allows, and the price is
power: a real difference must be larger to separate.

A real A|B on the Mac this was built on, at a one-minute load average near 10 (2026-11-20, not of
record): B ran 10% more work, its median throughput 14% below A's, and each interval ±12 to 16%, so the
two met: "not separated", the rule's price in power on a busy machine.

**Evidence.** Simulated A|A runs, 20,000 of each shape, 28 units a side, drawn as `ab.rs`'s A|A test
draws them from SplitMix64 seed 2026 (2026-11-20, this change's `judge` and `ab::compare`, before it was
committed): bell-shaped units 41 differences (0.21%), skewed 58 (0.29%), two states 5 (0.03%; both arms
had an interval in 4,219 of those runs). `ab.rs`'s test holds 2,000 such
runs under a one-sided binomial test at 5% (T-RQ0.7.7); a planted 8% slowdown is "A better" with the
intervals separated (T-RQ0.7.6); means four times apart with medians alike are "not separated"
(AC-RQ0.7.2).

**Falsified by.** An A|A rate above 5% on real runs; or a regression the nightly detector (M8) finds
that an A|B of the same builds called "not separated", which would move the verdict to intervals of
about 83%, the level that makes non-overlap a 5% test (research/07 F36).

## M5. Of record only under the load rule, with a clean commit

**Decision.** A record is of record when the one-minute load average, less the run's own share, is under
2 at every reading (CLAUDE.md §7), and every build names a clean commit. The first reading is taken
before any of the run's own work; the run's share is its processes' CPU over wall time decayed as the
kernel decays the average, `e^(−Δt/60 s)` (research/07 F26, F27). A platform with no load average
(Windows) makes no record of record, and says so. In a container the average read is the host's
(research/07 F28): a busy host makes the record not of record, as it should. A build names its commit
when `scripts/bench-build.sh` made it from one (`RELIQUARY_COMMIT`). A record not of record is written
and kept with the load that made it so, and an acceptance criterion never cites it:
`crates/reliquary-bench/tests/records.rs` fails the gate on a line that names an `AC-…` criterion and
such a record.

**Evidence.** `load.rs`: a run begun at load 5.31 is not of record and names it (T-RQ0.7.8); the run's
own threads are not held against it; no load average, or an unnamed or dirty commit, is not of record.
`tests/records.rs` refuses a fixture citation of a record not of record.

**Falsified by.** A record of record whose rerun on a quiet machine lies outside its interval, which
would mean the rule let a disturbed run through.

## M6. Latency from intended starts

**Decision.** Open loop, each operation's intended start is drawn ahead from a Poisson process
(hyper-measure's `Arrivals`), and its latency runs from that start, never from when the driver got to
it, so an operation queued behind a stall is charged the stall; the driver's lateness is recorded beside
and its worst is in the report (research/06 F48; research/07 F37, F38). The driver waits for an intended
start by parking until as long before it as its parks have woken late in this process, then yielding,
so a coarse timer (Windows's, which made a run on jupiter late by about 15 ms a start) does not make
every start late. Where no rate is stated, the rate is half the slower arm's
closed-loop rate measured first, an M/M/1 queue's point of greatest power (Kleinrock), the operating
point Mantle's admission seeks. A closed loop is a control.

**Evidence.** `drive.rs`: a server stalling for a second mid-run puts more than 900 of 10,000 operations
past a millisecond and the p99 past half a second under open loop, and under a closed loop only the one
operation the stall caught, the p99 under a millisecond (T-RQ0.7.5).

## M7. CI gates instruction counts

**Decision.** On every change, `.kiln/instructions.yaml` builds the change (B) and its base (A) with
`scripts/bench-build.sh` at paths of one length and runs `reliquary bench gate bench/gated.toml`.
Instructions are counted by Valgrind's cachegrind, which counts each instruction a process runs on its
synthetic CPU, so a pod needs no performance-counter privilege, which a container is refused by Docker's
default seccomp profile (research/07 F17 to F25). Cachegrind counts a whole process, so an operation's
count is the difference of two process starts, one running a discarded first operation and one running
it and `N` more; `N` is the fewest that holds the difference within the precision given how much a
start's count varies between processes. Units interleave as in M4 and are judged as in M1. A bench
fails when B rose: its interval separated above A's, or, where an arm cannot be judged (ordered, in two
states, too few units), every unit of B above every unit of A, which for exchangeable units of one
distribution happens once in `C(m + n, m)` and so needs neither independence nor one state. An arm that
cannot be judged with no such rise passes, said as undecided: Mantle's two checks of order mark about
one independent arm in ten (its design §2), so failing there would fail about one A|A gate in five. A
rise passes when a commit of the change carries `Bench-accepted: <bench>: <why>`. A bench the base
build does not run passes as new. Cachegrind's count is read from its `summary:` line at the place its
`events:` line gives `Ir` (research/07 F40). The base's own
`reliquary bench gate` judges the change, so a change to the judging cannot pass its own gate. A push is
gated against the commit the branch held before it, a pull request against where it leaves its base. Valgrind runs on Linux alone (research/07 F21); on macOS the gate
counts with the OS's own account of the process, round by round, and on Windows it is refused, naming
the platform. The benches gated are Reliquary's own code with no setting a person picks: today `judge`.

**Evidence.** `gate.rs`: 2,000 seeded A|A gates of counts a few tens apart fail within the 5% level by a
one-sided binomial test; each decision fails or passes as stated. Live on the Mac, with the OS's count,
at a load average near 27 (2026-11-20, not of record): ten A|A gates of `judge`, one build as both arms,
all passed (three judged, seven undecided); three gates of `spin` against a build planted with 10% more
work all failed (two by intervals, one by every unit of B above every unit of A). Live on Linux under
cachegrind (Debian 13 arm64 in a container on the same Mac, Valgrind 3.24.0, 2026-11-20): three A|A gates
of `judge` passed, both arms judged, about 44,060 instructions an operation within ±30, one operation a
process start as the start's count varied by 13 to 19 instructions; two gates against a separate build of
`spin` with 10% more iterations failed, A about 7,560 and B about 8,325 instructions an operation, and the
same with `--accept spin` passed as accepted.

**Falsified by.** A gate that fails a change whose code path does not touch the gated bench more often
than its 5% level, by its own A|A runs; or a rise the nightly finds that the gate passed.

## M8. A nightly series and its change points

**Decision.** `.kiln/nightly.yaml` measures main's tip on the bench host alone, open loop at the power
point, and feeds the p99's median to kiln's metrics when the record is of record. kiln keeps the series
per runner fingerprint (kiln's docs/ci.md W16-R07), so a series compares one class of hardware with
itself. `reliquary bench changes` reads it back, one point a UTC day, at most 366, and runs E-Divisive
means (Matteson and James; α = 1; the joint `τ, κ` search; bisection as a loop; research/07 F1 to F5):
a split is kept when no more than 5% of 199 shuffles within its segments reach its divergence (`ecp`'s
defaults, F6), each side at least three points, the fewest at which a two-sample test can reach 5% and
MongoDB's minimum too (F8). The shuffles are drawn from a generator seeded by the history's own commits
and values, so the same history gives the same change points every time and on every platform
(AC-RQ0.7.8); Hunter met the permutation test's nondeterminism with Student's t-test (F12), which assumes
normal metrics. An alert names its commit range, `git log <last before>..<first after>`, the medians on
either side and the last commit of the history it read, since a change point can move as nights are
added (F7); `bench/changes.toml` holds the ranges a person has looked at, and only a change for the worse
not there fails the run.

**Evidence.** `changes.rs`: a planted shift is found at its commit range, the same twice (T-RQ0.7.10);
two shifts give two; a steady series none; the search equals `Q̂`'s definition over every split.
`cli.rs`: kiln's answer read, an unacknowledged worsening alerts with its range and history, an
acknowledged one does not.

**Falsified by.** A change point that moves between two runs over one history; an alert whose range does
not hold the commit a bisection finds.

## Open

- **Live.** The two workflows are validated by kiln (`ci.workflows.validate`, no diagnostics, labels
  included) and the tool is tested, but neither workflow has run: they run once the person pushes them.
  The nightly needs its slot on kiln's bench calendar and the metrics API's answer as the job token reads
  it, which only a live run shows.
- **Builds as a level.** The tool measures builds given to it; making builds that differ in code layout
  (a linker's section order, STABILIZER's re-randomization) is not built.
- **A container's own pressure.** In a container the load rule reads the host's average; the
  container's own CPU pressure (PSI `cpu.pressure`, research/07 F29) would say what the run met itself.
- **Mantle's crate.** `rounds.rs` is carried until Mantle publishes its rounds as a crate others link.
- **Tails over pooled operations.** A state's quantiles are read over its operations together, and
  their intervals treat the operations as independent, as Mantle's design states them; operations
  within a unit queue behind one another, and across process starts they share no process, so these
  intervals can be narrower than a rerun shows. The units' own medians carry the sound intervals.
- **Waits on a worker.** The driver reads a round's answer until the worker answers or ends; a worker
  runs only the operations and schedule stated, so the wait is bounded by the stated work and the
  worker's own liveness, not by a clock of the driver's. A worker alive and stuck would hold it.
