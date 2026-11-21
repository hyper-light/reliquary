# Quiver, vendored

Where Reliquary's events go (architecture D26): Quiver's `quiver-api`, the trait Reliquary reaches its
events through (`Topics`: topics of messages in order, read from any offset kept or followed by a
subscription told what it missed), with its simulator and conformance suite, for the seam (plan RQ0.4,
`docs/design/plan/phase-00-ground.md`) (kiln's docs/vendoring.md; plan RQ0.1).

- **Upstream:** https://github.com/hyper-light/quiver, branch `main`, commit
  `966ea139a18d590afd64cc3e30ba3807302d42b1` (2026-11-12), MIT (`LICENSE`): the commit kiln's
  `vendor/MANIFEST.toml` names at kiln `e64eeb7c` (2026-11-20).
- **What is here:** upstream's root manifest and `LICENSE`; `crates/quiver-api`, whole; and Quiver's
  record of what it vendors (`vendor/MANIFEST.toml`, `vendor/patches`), so Quiver's pins are checked as
  this copy holds them (`scripts/vendor.py`'s `holds_its_pins`), as kiln checks them on its own copy.
  Left out: Quiver's node (`crates/quiver`), its benchmarks (`crates/quiver-bench`, `bench/`), its
  copies of Focal's bus and hyper-raft (`vendor/focal-bus`, `vendor/hyper-raft`: this repository holds
  each beside this folder, at the revision Quiver's record names), and its `Cargo.lock`, docs, scripts
  and CI.
  `quiver-api`'s doc-truth test (`tests/doc_truth.rs`) reads Quiver's `docs/design/topics.md`, which
  isn't taken; Reliquary builds the crate, not its tests.
- **What it builds on:** `event-listener` and `futures`, from crates.io; no crate of hyper-light's.
- **How it's built:** Reliquary's root `Cargo.toml` points `quiver-api` at its folder here
  (`[patch.crates-io]`), as kiln patches the same name. `quiver-api` is Quiver's `-api` crate, so a
  crate of Reliquary's names it by the rule (CLAUDE.md §9), and the manifest has no exception line for
  it. The folder is a workspace of its own, which Reliquary's excludes.

## Changes from upstream

Each change is recorded in `vendor/patches/quiver/`, applied in this order on upstream's tree. They are
kiln's, byte for byte (none is kiln's layout alone): each brings Quiver's record of what it vendors to
what kiln holds, which is what this repository holds. kiln's `vendor/quiver/UPSTREAM.md` says each
change's why in full.

| File | Change | Why | Recorded as |
|---|---|---|---|
| `vendor/patches/hyper-raft/0005-hyper-transport-says-which-suite-and-group-a-connection.patch` | Quiver's record of kiln's fifth hyper-raft change, byte for byte | Reliquary's one hyper-raft carries that change (`vendor/patches/hyper-raft/0005`: hyper-transport says which suite and group a connection negotiated, for Hex), and Quiver's record of what it vendors, which this copy holds, must record the same changes to it (`scripts/vendor.py check`); kiln's, carried as kiln records it | `0001-quiver-records-kiln-s-hyper-raft-change-for-hex-s-suite.patch` |
| `vendor/MANIFEST.toml` | Quiver's hyper-raft pin moved from `fb74b595` to `e0613f52` | Reliquary holds one copy of hyper-raft, at the revision kiln takes (`e0613f52`, the one Shards and Hex pin), and a product it vendors pins that copy (AC-RQ0.1.3); kiln's, carried as kiln records it | `0002-quiver-s-hyper-raft-pin-moves-with-kiln-s.patch` |
| `vendor/patches/focal-bus/0006-the-memory-budget-s-atomic-updates-by-try-update-fetch.patch` | Quiver's record of kiln's sixth focal-bus change, byte for byte | Reliquary's focal-bus carries kiln's six changes, that one among them (`vendor/focal-bus/UPSTREAM.md`: the memory budget's atomic updates by `try_update`, `fetch_update`'s name since Rust 1.99), and Quiver's record of the bus must be the same (`scripts/vendor.py check`); kiln's, carried as kiln records it | `0003-quiver-records-kiln-s-focal-bus-change-for-the-memory-budget.patch` |

## Taking a newer upstream

`python3 scripts/vendor.py quiver <commit>` takes Quiver at that commit and re-applies the changes above
by three-way merge; a commit whose record already holds kiln's makes a change nothing, and the re-vendor
drops it. Take the commit kiln's `vendor/MANIFEST.toml` names, with kiln's changes, and move `[kiln]`'s
commit with it. A newer Quiver may name Focal's bus or hyper-raft at another revision: this repository
then takes them at kiln's, and `scripts/vendor.py check` holds Quiver's record to them.
