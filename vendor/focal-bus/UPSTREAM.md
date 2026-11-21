# Focal's event bus, vendored

The bus Quiver is built on, held because Quiver's record of what it vendors names it
(`vendor/quiver/UPSTREAM.md`): this repository holds one copy of each hyper-light product, and a product
whose record its copy holds takes each one it names at that copy, with the same recorded changes
(`scripts/vendor.py check`). Nothing in Reliquary links the bus, so only what kiln's recorded changes
edit is here, and those changes are kiln's, byte for byte (kiln's docs/vendoring.md; plan RQ0.1, RQ0.4).

- **Upstream:** https://github.com/hyper-light/focal, branch `bus-replicated`, commit
  `9aded7cdf0a596f35039acb9b4e610c52f9d1f0a` (2026-10-28), MIT (`LICENSE`): the commit kiln's
  `vendor/MANIFEST.toml` names at kiln `e64eeb7c` (2026-11-20), and the one Quiver's record names.
- **What is here:** upstream's root manifest and `LICENSE`, and the six files kiln's changes edit, at
  upstream's paths: of the bus, `crates/focal-bus/Cargo.toml`, `src/member.rs` and `src/pki.rs`; of its
  memory budget, `crates/focal-memory/src/budget.rs`, `disk.rs` and `owner.rs`. No crate is here whole,
  and nothing here is built: no crate of Reliquary's names the bus, and `[patch.crates-io]` points no
  name here. Left out: the rest of upstream's tree.

## Changes from upstream

Each change is recorded in `vendor/patches/focal-bus/`, applied in this order on upstream's tree. They
are kiln's, byte for byte (none is kiln's layout alone), and Quiver's record of the bus is the same six
(`vendor/quiver/UPSTREAM.md`). Each was made for Quiver; kiln's `vendor/focal-bus/UPSTREAM.md` says each
change's why in full.

| File | Change | Why | Recorded as |
|---|---|---|---|
| `Cargo.toml` | The shared crates (`hyper-*`) named by version, without a path into Focal's `vendor/` | whoever builds the bus points each name at its one copy of hyper-raft (`[patch.crates-io]`), not Focal's own; kiln's, carried as kiln records it | `0001-the-shared-crates-by-name-at-the-copy-of-hyper-raft-beside.patch` |
| `crates/focal-bus/src/member.rs` | A member's liveness stream takes room for one peer where its group has none | Quiver's persisted class runs on one node as a group of one, and hyper-liveness refuses a stream of no peers; kiln's, carried as kiln records it | `0002-a-member-s-liveness-stream-takes-room-for-one-peer-so-a.patch` |
| `crates/focal-bus/src/member.rs`, `crates/focal-bus/Cargo.toml` | A member's log on a given file (`NodeFile`) or on hyper-sim's block device (`Member::open_simulated`, the `sim` feature); `Member::cut`, `serve` and `in_flight`; `Member::plant` (the `mutants` feature); the owed acknowledgements redirected in order as a member stops leading | Quiver's persisted class runs in a deterministic simulation, and a `HashMap`'s order sent the redirects differently each run; kiln's, carried as kiln records it | `0003-a-member-s-log-on-a-simulated-device-its-cuts-and-work-in.patch` |
| `crates/focal-bus/src/member.rs` | Under the `sim` feature, the member's diagnosis (`Member::core_state`): the core's state, each member's progress, the outbox, the Raft traffic per peer, the liveness stream's measures | Quiver's simulation of its persisted class traces a node that never catches up to its cause with it; kiln's, carried as kiln records it | `0004-the-simulation-s-diagnosis-of-a-member.patch` |
| `crates/focal-bus/src/pki.rs` | A directory's identities may be made as Ed25519 (`KeyKind`, `load_or_make_with`); `load_or_make` makes them as before, ECDSA P-256 | an ECDSA signature's length varies with its nonce, so a simulated handshake's flight differed from one run of a seed to the next; Ed25519's are 64 bytes every time (RFC 8032); kiln's, carried as kiln records it | `0005-identities-may-be-made-as-ed25519.patch` |
| `crates/focal-memory/src/budget.rs`, `crates/focal-memory/src/disk.rs`, `crates/focal-memory/src/owner.rs` | The memory budget's five atomic updates call `try_update`, `fetch_update`'s name since Rust 1.99.0, with the same arguments and result | Rust 1.99.0 deprecates `fetch_update`; kiln's, carried as kiln records it | `0006-the-memory-budget-s-atomic-updates-by-try-update-fetch.patch` |

## Taking a newer upstream

`python3 scripts/vendor.py focal-bus <commit>` takes these files at that commit and re-applies the
changes above by three-way merge. Take the commit kiln's `vendor/MANIFEST.toml` names, the one Quiver's
record names too, with kiln's changes, and move `[kiln]`'s commit with it. A file a new change of kiln's
edits is added to `take`; when Reliquary comes to link the bus, its crates are taken whole, as kiln takes
them, and patched by name to this copy.
