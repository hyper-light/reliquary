# Mantle, vendored

Where Reliquary keeps its bytes (architecture D3, D4): Mantle's client, protocol, HTTP codec and CRCs,
built on this repository's one copy of hyper-raft and of aws-lc-rs, at the revision kiln vendors Mantle
(kiln's docs/vendoring.md; plan RQ0.1).

- **Upstream:** https://github.com/hyper-light/mantle, branch `main`, commit
  `87ab09e1c94149c7b669cb0de5975e228125c867` (2026-11-08), MIT (`LICENSE`): the commit kiln's
  `vendor/MANIFEST.toml` names at kiln `e64eeb7c` (2026-11-20).
- **What is here:** upstream's root manifest, `Cargo.lock` and `LICENSE`, and six crates whole, at
  upstream's paths: `client` (`mantle-client`), `protocol`, `http`, `crc`, and `s3` and `codec`, which
  `client` and `protocol` take. Of `cert`, `chunk`, `disk`, `ec`, `engine`, `mantle`, `node` and
  `range`, their manifests alone, which the first change below edits: nothing here builds them, and the
  change is carried so these changes are kiln's, byte for byte. Left out: the rest of upstream's tree,
  Mantle's own `vendor/` among it (its copies of hyper-raft's crates and of aws-lc-rs and aws-lc-sys).
- **What it builds on:** hyper-raft's crates at `vendor/hyper-raft` and aws-lc-rs at
  `vendor/aws-lc-rs`, as kiln's copy of Mantle does: Mantle's crates take hyper-raft's by path, which
  the first change below points at `../../../hyper-raft/crates/hyper-*`.
- **How it's built:** Reliquary's root `Cargo.toml` points each name its crates take at its folder
  here (`[patch.crates-io]`). The folder is a workspace of its own, which Reliquary's excludes.

## Changes from upstream

Each change is recorded in `vendor/patches/mantle/`, applied in this order on upstream's tree. They are
kiln's, byte for byte, but kiln's third (`0003-hyper-rt-from-kiln-s-own-runtime-crates-kiln-rt.patch`),
which is kiln's layout alone (`Kiln-only:`): it points five crates at kiln's own copy of hyper-rt, which
Reliquary doesn't hold, so Mantle's crates here take hyper-rt from `vendor/hyper-raft`. kiln's
`vendor/mantle/UPSTREAM.md` says each change's why in full.

| File | Change | Why | Recorded as |
|---|---|---|---|
| `Cargo.toml`, `Cargo.lock`, the manifests of `crates/cert`, `chunk`, `client`, `disk`, `ec`, `engine`, `http`, `mantle`, `node`, `protocol` and `range` | each of hyper-raft's crates taken by path from the copy beside this one (`../../../hyper-raft/crates/hyper-*`), not from Mantle's `vendor/`; the root's `[patch.crates-io]` points aws-lc-rs at `../aws-lc-rs/aws-lc-rs`; `Cargo.lock` resolved on them | one copy of each: two linked copies of hyper-raft fail the one-revision test (AC-RQ0.1.3), and Mantle's own copies aren't here; kiln's, carried as kiln records it | `0001-its-crates-on-kiln-s-one-hyper-raft-and-aws-lc-rs.patch` |
| `crates/protocol/src/bounds.rs` | `Limits::acknowledgement` stated as `Acknowledgement::Decimated`, `Limits::fec_window` as `None` | hyper-raft at this revision adds both; a node's transport states them as every connection had them before; kiln's, carried as kiln records it | `0002-hyper-transport-s-acknowledgement-and-erasure-coding-stated.patch` |

## Taking a newer upstream

`python3 scripts/vendor.py mantle <commit>` takes Mantle at that commit and re-applies the changes above
by three-way merge. Take the commit kiln's `vendor/MANIFEST.toml` names, with kiln's changes, and move
`[kiln]`'s commit with it. A crate this repository comes to build is added to `take` whole.
