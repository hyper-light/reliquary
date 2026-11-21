# 12. The simulated world: what it is built on, and what its runs found

Status: research note, 2026-11-20, for plan RQ0.6 (`docs/design/sim.md`). Quotes are verbatim; each
claim carries its source; a conclusion derived here says so and from what. F13 to F15 are measured by
this step's own runs, each named by its test and seed.

## Sources

[1] hyper-raft's `docs/sim.md` at `e0613f524b0df198cc61dc1e82b33c22d604cf7b`, the revision Reliquary
vendors (`vendor/hyper-raft/UPSTREAM.md`), read with `git show e0613f52:docs/sim.md` in a hyper-raft
checkout: the design of `hyper-sim` and `hyper-check`.

[2] `hyper-sim`'s source at the same revision, as vendored: `vendor/hyper-raft/crates/hyper-sim/src/`
(`clock.rs`, `net.rs`, `twice.rs`, `storage.rs`).

[3] `amphora-api` at Amphora `db991a05`, as vendored: `vendor/amphora/crates/api/src/lib.rs`, the
`Refusal` enum.

[4] Reliquary's `docs/design/seams.md` (plan RQ0.4) and `crates/reliquary-core/src/seams/bytes/sim.rs`
as RQ0.4 left them (2026-11-20).

[5] Microsoft, `FILETIME` structure
(https://learn.microsoft.com/en-us/windows/win32/api/minwinbase/ns-minwinbase-filetime).

[6] The Rust standard library, `std::time::SystemTime`, "Platform-specific behavior"
(https://doc.rust-lang.org/std/time/struct.SystemTime.html).

[7] RFC 9293, Transmission Control Protocol (https://www.rfc-editor.org/rfc/rfc9293.txt).

## Findings

F1. A process under simulation is a state machine whose durable state the world keeps. [1] §3.6: "A
process is a sans-io state machine behind a test adapter (`Process`): it is started on what its
devices hold, given events (a message, a timer, a completion), and gives outputs (messages, timers,
device operations, observations) through a context that lends it its node's clocks and its own random
stream. Its durable state lives in the world's devices, never in the process". Reliquary's node shell
and its `Core` are this shape (sim.md W2), the stores' simulators its devices (W4).

F2. Every input the world chooses. [1] §3.3: "This is P#'s architecture (every source of nondeterminism
declared and controlled by the tester, Deligiannis et al. §1–§2) applied to sans-io crates, where the
declaration is already made: a crate here never reads a clock, a socket or a disk, so every input it
gets is an event the world chose." Faults come at the boundary, not inside the code under test: "No
buggify points in production code. FDB needs them because Flow code makes its own choices; a sans-io
crate makes none, so the unusual-but-legal behaviours FDB buggifies — an operation that fails, a delay,
an odd tuning value — are the world's to give at the boundary". (sim.md W1, W6.)

F3. A node's clocks drift and its wall clock steps. [1] §3.2: "Each node's clocks are views of virtual
time: a monotonic clock with an offset and a rate in parts per million, never going backward, and a
wall clock that may also step forward or back (mantle's ±2 s steps; Antithesis's clock skips). The
rate's bound is the deployment's stated drift bound, the quantity a lease rests on (Gray and Cheriton
§5, mantle note 06 A7 [via])." [2] `clock.rs`: "The wall clock is the monotonic clock moved to the
node's epoch, and may also be stepped forward or back". (sim.md W3.)

F4. Duplication is a re-delivery, and a partition cuts in flight too. [1] §3.4: "Duplication as a
re-delivery of a message still held, not a copy (hyper-raft's `keep`), so it adds nothing to the
bound." and "Partitions of every shape VOPR draws: a random split of uniform size, uniform membership,
one node isolated, symmetric or asymmetric, with start and heal rates and minimum stabilities". [2]
`net.rs`: "A **partition** cuts at the send and at the arrival: a message in flight when its pair is
cut is lost when it arrives (focal's network)." (sim.md W5, W6.)

F5. Where crash bugs lie. [1] §3.5: "After every persistence point (CrashMonkey's B3, §4): one crash
state per flush or `fsync` in the run, each with nothing unflushed surviving — linear in the run, and
where every bug in Mohan et al.'s study lay." Derived: a write a store acknowledges is its persistence
point, so a power cut before and after each write of a history (AC-RQ0.6.3) covers the crash states of
that study's kind. (sim.md W12.)

F6. Liveness waits on progress after the faults, not on a picked count. [1] §3.8: "The liveness phase.
After the faults, every run heals to a core ... and must converge within a bound derived from the
timing law (§4.2)." [1] §4.2: "The bound is neither a literal nor a confidence level someone picks".
Derived for Reliquary: a client's attempt after the faults stop must be answered or refused; one that
waits out its client's patience unanswered fails the run (AC-RQ0.6.5). The patience and the counted
attempts are the run's own stated figures. (sim.md W7.)

F7. Determinism is checked, not assumed. [1] §3.9: "Run twice. Every simulation test's first seed runs
twice and compares a digest of every decision and every observation". [2] `twice.rs`: "a simulation's
first seed runs twice from its seed and once from the trace of the first run, and all three must give
the same digest". (sim.md W1, W8.)

F8. Every check is shown to catch a defect planted for it. [1] §4.6: "Each oracle and each strategy has
planted defects it must catch". (sim.md W11.)

F9. A deadline passed or an answer lost leaves a commit's outcome open, settled by reading. [3]:
"#[error(\"the deadline elapsed; a submitted transaction may still commit\")] Timeout" and
"#[error(\"the commit outcome is unknown; reconcile it before retrying the operation\")]
UnknownOutcome". [4] S7: "The caller settles it by reading". (sim.md W10.)

F10. A grant's expiry is judged at Mantle's side by its own wall clock. [4] S6: "Mantle's side refuses
any of them changed (`Forbidden`), a presenter other than the holder (`Forbidden`), and a grant past
its expiry by its own wall clock (`Expired`)". In [4]'s `sim.rs` as RQ0.4 left it, `redeem` compared
each presentation with the wall clock's reading at that moment: `if shared.clock.wall() >=
grant.expires`. (See F13.)

F11. Windows keeps wall time in 100-nanosecond intervals. [5], under its title: "Contains a 64-bit value
representing the number of 100-nanosecond intervals since January 1, 1601 (UTC)." [6], "Platform-specific
behavior": "The following system calls are currently being used by `now()` to find out the current
time:", its row for Windows "GetSystemTimePreciseAsFileTime / GetSystemTimeAsFileTime", and for UNIX and
Darwin "clock_gettime (Realtime Clock)". Derived: a `SystemTime` made from a reading finer than 100 ns is cut on
Windows and on no other target, so the world puts its wall readings on that grid (sim.md W3).

F12. A connection to a process that is gone is refused or reset; a machine with no power answers
nothing. [7] §3.5.2, "Reset Generation": "If the connection does not exist (CLOSED), then a reset is sent
in response to any incoming segment except another reset. A SYN segment that does not match an existing
connection is rejected by this means." Derived: a crashed process's machine refuses new
connections and resets open ones, so its clients and callers learn of it at once, while a machine whose
power is cut is silent until they give up (sim.md W6).

F13. Measured: RQ0.4's bytes simulator honored a grant it had refused as expired, once its wall clock
stepped back. `crates/reliquary-sim/tests/clocks.rs`, T-RQ0.6.6, seed `0x5eed_0606`, before the fix:
the bytes store refused a grant `Expired` at 89,609,612 ns of the run, its wall clock was stepped back
40 ms at 90 ms, and a second delivery of the same redemption was honored at 94,573,716 ns. The ledger
named it a lease honored twice. Fixed at its cause: the simulator judges expiry against the latest wall
time it has read (sim.md W9; seams.md S6).

F14. Measured: the recursion gate (`scripts/check-recursion.py`, docs/design/gates.md G2) found two
call-graph cycles in the model node's first form: `commit` → `decide` → `write` → `commit` (a decision
made stale by the node's own commit decided again) and `pump` → `scan` → `sync_failed` → `pump` (a log
read that could not be made). Each was bounded at run time and still a cycle the rule refuses; both
became steps a loop drains (sim.md W10).

F15. Measured: a crash that only drops what reaches a machine leaves a client waiting out its whole
patience, during which faults armed for the next write or answer find none. `tests/nemesis.rs` before
crashes refused and reset connections: a node crashed at 2 ms held the setup client 2 s while the calm
at 150 ms disarmed a power cut and two lost answers that never landed. Hence F12's model, and armed
faults that outlive the calm and land at their machine's next write or answer (sim.md W6).
