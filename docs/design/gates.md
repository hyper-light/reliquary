# The structural gates and the platform matrix

Plan RQ0.3 (serves CP-T9.1). The gates CLAUDE.md's rules need beyond lints, and where they run. Sources
are `docs/research/11-structural-gates.md` ([F1]…[F10] below are its findings); each decision says what
would falsify it.

## G1. One gate, in one order, stopping at the first failure; the platform's part on each platform

`scripts/gates.sh` runs, in this order (AC-RQ0.3.1): contracts (`scripts/check-contracts.py`, RQ0.2),
fmt, clippy for each of the five targets, the production check, recursion, deny, the tests, the vendor
check; `set -e` ends it at the first failure with that failure's status. `gates.sh here` runs the gates
whose answer depends on the platform they run on: the production check of the host's target, recursion
in the host's build (the IR is the host's, and so is its `cfg`'d code), and the suite. The rest read the
same sources and give the same answer anywhere; clippy lints all five targets from any one host (zig
compiles AWS-LC's C for the others). kiln CI runs the whole gate on Linux x86_64 (`.kiln/ci.yaml`) and
`gates.sh here` on macOS arm64, Windows x86_64 and Linux arm64 (`.kiln/platforms.yaml`), on every push
to main and every pull request (AC-RQ0.3.2, T-RQ0.3.8).

The whole gate needs Python 3.11 or later for its vendor step (kiln's `scripts/vendor.py` reads TOML
with `tomllib`, and says so where it is older); Reliquary's own scripts run on 3.9 and later, as `gates.sh
here` does on a host whose Python is older (macOS's /usr/bin/python3 is 3.9).

Evidence: the order is Mantle's gate's [1]; `crates/reliquary/tests/gates.rs` checks the order and runs
the gate in a tree whose first gate fails, which stops there. Falsified by a gate whose answer differs by
platform yet runs once: then it joins `here`.

## G2. No recursion, read from the built code's call graph, through pointers too

`scripts/check-recursion.py` is Mantle's [1]: the workspace's libraries and binaries built as -O0 LLVM IR
with one codegen unit, every `define` a function and every direct call an edge, Tarjan's strongly
connected components on an explicit stack [F5], and a cycle with a function of Reliquary's failing the
gate with every function in it (AC-RQ0.3.3). Mantle's check leaves calls through pointers uncounted
[F1]; Reliquary's follows them, because T-RQ0.3.2 is such a call:

- A call through a trait object is a load from a vtable slot rustc marks `!invariant.load`, at a byte
  offset, called [F3]; the vtables are the module's `@vtable.N` constants, whose entries' offsets and
  functions are read from the constant, and whose trait the full debug information names
  (`<T as Trait>::{vtable}`), so the IR build takes full debug information for the workspace's crates.
  The call may reach the function at its offset in any vtable the workspace makes (class hierarchy
  analysis, [F4]), narrowed by what is known at the call: the method its source names at the call's
  debug location (a method call's location is its name), else a closure's call (`f(x)` reaches the Fn
  traits' vtables), and none when no vtable the workspace makes has that name (its trait's objects
  are made elsewhere); only a call whose source cannot be read reaches the entries with its LLVM
  signature. A call in std's or a dependency's instance for a trait object (its drop glue,
  `Pin<Box<dyn Future>>`'s poll) goes through the vtables of the traits its path names alone, a trait
  object inside another's generic arguments left out (the `dyn S` of `dyn Future<Output = Box<dyn S>>`
  is the future's output). rustc names a module's vtables `@vtable.0` to `@vtable.9`, then
  `@vtable.a` and on; every one is read.
- A boxed future is polled and dropped in std's shared instances, so a call there is made for each of
  Reliquary's functions above it, and reaches only the futures whose vtables that function, or what it
  calls, makes; a drop glue's are its future's own poll's. So `helper().await` reaches the helper's
  future and not the awaiting one, while an async function that boxes and awaits itself is a cycle, and
  so is the drop of the nested futures it builds.
- A call through a function pointer may reach any function of the workspace's IR whose address
  Reliquary's own code takes (outside vtables, and outside std's and dependencies' instances such as the
  formatting machinery) with its signature.

The wrapper that adds the IR flags to the workspace's compiles is a small Rust program
(`scripts/recursion-wrapper.rs`), compiled by the pinned rustc, because Windows cannot run a script as
cargo's RUSTC_WORKSPACE_WRAPPER; so the check runs on every platform, `gates.sh here` included.

Evidence: `gates.rs` plants a function calling itself (T-RQ0.3.1), two calling each other through
trait objects (T-RQ0.3.2) and a ring of twelve through `dyn Ring` (twelve vtables), each found and
named; and calls through pointers that cannot come back (a
boxed trait object dropped inside another trait's, a future awaiting a helper's boxed future, cases a
runner calls through function pointers) found to be no cycle, beside async recursion through a boxed
future found as one. On the workspace of 2026-11-20 (14,722 functions, 82 calls through trait objects,
141 vtables) it finds no cycle of Reliquary's. Not seen: calls made into Reliquary's code by std's or a dependency's compiled code
through pointers (formatting, a thread's start, an executor's poll of a spawned task), and code no build
instantiates. Falsified by a cycle through a pointer the check misses, or a cycle it reports that the
code cannot make: either is a fixture in `gates.rs` and a change here.

## G3. `unsafe` only in the files CLAUDE.md names

The workspace denies `unsafe_code`, and clippy's `undocumented_unsafe_blocks` holds each block to its
`// SAFETY:` comment. `scripts/check-unsafe.py` holds the rest of CLAUDE.md §12: any file under crates/
that uses `unsafe` (a block, a function, an impl, a trait, an extern block, an `unsafe(…)` attribute) or
allows the lints that deny it must be named in §12, in backticks; each file named must exist and use
`unsafe`. It reads tokens (`scripts/rust_tokens.py`, the Reference's lexical structure), so a word in a
comment or a literal is not code. It runs as part of the production check (T-RQ0.3.3). Falsified by an
`unsafe` it does not see, or one it reads in a comment or a literal.

## G4. The dependency graph: advisories, bans, licenses and sources

cargo-deny 0.22.0, installed by its digest (`scripts/install-cargo-deny`), checks `deny.toml` over the
five targets and the musl two, every feature (T-RQ0.3.4). Advisories, unmaintained and unsound included,
and yanked versions fail. Licenses: a permissive list, Mantle's [1] with OpenSSL's for AWS-LC;
dev-dependencies' licenses are not read [F6], as no artifact carries them. Bans: wildcards, and a second
TLS or crypto stack (openssl, native-tls). Sources: crates.io; another hyper-light product comes by name
and version, patched to its vendored copy.

An exception is of one kind only: an advisory, or a git source, in what a vendored product brings, which
Reliquary may not change. Each is allowed only for that product's crate that depends on it, expires
[F6], and is asked of the product's owner (STATUS.md, "Open"). Today these are the Amphora store's:
chrono, heapless 0.7 under postcard 1.1, and socket2 at a fixed commit. Evidence: `gates.rs` plants a
dependency with an advisory (smallvec 1.6.0, RUSTSEC-2021-0003) and one under GPL-3.0-only, each failing.
Falsified by an advisory or a license deny.toml lets through unnamed.

## G5. Every test cites a requirement, and skips only through the skip record

`scripts/check-tests.py` (part of `scripts/test.sh`, the gate's tests): every `#[test]`'s doc comment
names at least one `AC-…` or `T-…` ID, each in `docs/requirements.toml` (AC-RQ0.3.4, T-RQ0.3.5). A test
that cannot run where the gate runs skips through `reliquary_testkit::skip`, Hex's skip record [F7]: a
line on stderr and the same line appended (O_APPEND, so lines from processes at once land whole, [F8])
to the record, which `scripts/test.sh` empties before the suite and prints after it, pass or fail
(AC-RQ0.3.5, T-RQ0.3.6). libtest hides a passing test's output [F7], so without the record a skip that
passes is unseen. The audit refuses what would skip unseen: a `return` in a test whose statement records
no skip, `#[ignore]`, and a `cfg` on a test or on the module or file holding it that can be false while
`test` is true (T-RQ0.3.7): a test is compiled everywhere and skips through the record where it cannot
run. The suite runs with `--no-fail-fast`, so every binary's tests run and every skip is recorded.
Falsified by a test that skips, or is left out, without a line in the record.

## G6. The platform matrix

What runs where follows the runners kiln has [F10]. The whole gate runs in a Linux x86_64 pod; `gates.sh
here` on a macOS arm64 machine, on jupiter for Windows x86_64 (with python.org's embeddable Python 3.14.10
by its published digest, as jupiter has none, [F9]), and in a linux/arm64 Debian container on an arm64
machine's docker for Linux arm64. The suite runs in containers of Debian, Red Hat (UBI 10), SUSE (Leap
16.0), Arch and Alpine (musl), x86_64 in pods and arm64 in docker, each installing the pinned rustup and
toolchain by digest (`scripts/install-rust`), with the skip record printed after it (AC-RQ0.3.7,
T-RQ0.3.9). Two have no runner, each named in STATUS.md with what would close it: Windows on arm64 (no
such machine), and Arch Linux on arm64 (no official image). Falsified by a target or family the record
shows passing that never ran there.

Evidence, 2026-11-20, on copies of the shared tree, before any push (kiln's local CI run took no pack
of this size, `ci.uploads.pack`): the whole gate passed on macOS arm64 in 19m19s; `gates.sh here` passed
in Debian containers on Linux arm64 (natively) and Linux x86_64 (under emulation on an arm64 Mac); the
suite passed in Debian 13, RHEL 10.3 (UBI 10), openSUSE Leap 16.0 and Alpine 3.24 on arm64, and in Debian
13, openSUSE Leap 16.0, Arch Linux and Alpine 3.24 on x86_64, each with its skip record printed after
it. Under emulation two legs differed from CI's: Arch's pacman runs without its download sandbox,
which emulation cannot seccomp, and RHEL 10, which needs x86-64-v3 that emulation lacks, could not
start TLS, so the Red Hat family ran as RHEL 9.9 (UBI 9), its record naming the three tests that skipped
for want of Python 3.11. Windows x86_64 ran on jupiter, where its first runs found what this design now
answers: MSVC's vtable names, a binary's unhashed names, AWS-LC's NASM, and in three tests of RQ0.4's
and RQ0.9's, times finer than Windows' 100 ns clock.

## The Gates lines, approved

The person approved these as CLAUDE.md's Gates lines on 2026-11-20 (AC-RQ0.3.6), asked in the RQ0.3
session before CLAUDE.md changed, in this order:

```
python3 scripts/check-contracts.py
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked --target x86_64-unknown-linux-gnu -- -D warnings
cargo clippy --workspace --all-targets --locked --target aarch64-unknown-linux-gnu -- -D warnings
cargo clippy --workspace --all-targets --locked --target aarch64-apple-darwin -- -D warnings
cargo clippy --workspace --all-targets --locked --target x86_64-pc-windows-msvc -- -D warnings
cargo clippy --workspace --all-targets --locked --target aarch64-pc-windows-msvc -- -D warnings
bash scripts/check-production.sh --target <each of the five>
python3 scripts/check-recursion.py
cargo deny --locked check advisories bans licenses sources
bash scripts/test.sh
python3 scripts/vendor.py check
```

The same day the person chose G4's exceptions for what vendored Amphora brings, scoped to the crate that
brings each and expiring 2026-12-20, over checking the shipped graph alone or blocking every commit
until Amphora changes.
