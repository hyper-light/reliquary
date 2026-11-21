# hyper-raft's shared crates, vendored

The runtime, simulator, measurement, transport and timing crates Reliquary is built on, and those
Amphora's store is built on, at the revision kiln vendors them, so kiln, Mantle (`vendor/mantle`),
Amphora's store (`vendor/amphora`) and Reliquary link one hyper-raft (kiln's docs/vendoring.md; plan
RQ0.1).

- **Upstream:** https://github.com/hyper-light/hyper-raft, commit
  `e0613f524b0df198cc61dc1e82b33c22d604cf7b` (2026-11-13), MIT (`LICENSE`): the commit kiln's
  `vendor/MANIFEST.toml` names at kiln `e64eeb7c` (2026-11-20), hyper-raft's branch
  `p/q35-handshake-progress-v1`, one commit past its `main` (`045cf3ff`), which Hex and Shards' control
  link take too (kiln's `vendor/hyper-raft/UPSTREAM.md` gives its history).
- **What is here:** upstream's root manifest and `LICENSE`, and fifteen crates, whole, at upstream's
  paths: `hyper-datagram`, `hyper-measure`, `hyper-quic`, `hyper-rt`, `hyper-rt-transport`,
  `hyper-sim`, `hyper-timing`, `hyper-tls` (less its tests and test certificate authority, which only
  its own tests read) and `hyper-transport`, which Reliquary and Mantle's crates build on; and
  `hyper-block`, `hyper-durable`, `hyper-liveness`, `hyper-log`, `hyper-raft` and `hyper-seal`, which
  Amphora's store builds on.
- **How it's built:** Reliquary's root `Cargo.toml` points each name its crates take at its folder
  here (`[patch.crates-io]`); hyper-raft's crates take one another by path, and Mantle's take these by
  path (kiln's first change to Mantle, `vendor/mantle/UPSTREAM.md`). The folder is a workspace of its
  own, which Reliquary's excludes.

## Changes from upstream

Each change is recorded in `vendor/patches/hyper-raft/`, applied in this order on upstream's tree. They
are kiln's, byte for byte, but kiln's second (`0002-hyper-rt-from-kiln-s-own-runtime-crates-kiln-rt.patch`),
which is kiln's layout alone (`Kiln-only:`): it points two crates at kiln's own copy of hyper-rt, which
Reliquary doesn't hold. kiln's `vendor/hyper-raft/UPSTREAM.md` says each change's why in full; a product
that vendors hyper-raft records the same changes, which kiln's `scripts/vendor.py` checks when it vendors
the product.

| File | Change | Why | Recorded as |
|---|---|---|---|
| `crates/hyper-durable/src/lib.rs`, `crates/hyper-durable/Cargo.toml` | `RamStore` compiled only for the crate's own tests or under the `test-store` feature | the write-ahead log is mandatory in Quiver and kiln (kiln's CP-T2.1, T-T2.1.2); kiln's, carried as kiln records it | `0001-ramstore-for-tests-alone-the-write-ahead-log-mandatory.patch` |
| `crates/hyper-rt-transport/src/driver.rs`, `crates/hyper-rt-transport/tests/received.rs` | `Driver::poll_received` and `Driver::received`: a turn's events handed out before its sends, and a test of both | kiln's net thread hands what comes in to its shards before the turn's sends; kiln's, carried as kiln records it | `0003-driver-poll-received-a-turn-s-events-before-its-sends.patch` |
| `crates/hyper-tls/src/conn.rs` | `ConnectionCommon::set_max_fragment_size`: the largest record a connection sends, set as it goes | a socket whose records follow the path's rate; kiln's, carried as kiln records it | `0004-hyper-tls-a-connection-s-largest-record-set-as-it-goes.patch` |
| `crates/hyper-transport/src/endpoint.rs` | `Endpoint::negotiated`: the TLS 1.3 suite and key exchange group a connection negotiated | every link of the house takes `TLS13_AES_256_GCM_SHA384` alone and a test at each end says so (Ada, 2026-11-11); Hex's change, which kiln holds | `0005-hyper-transport-says-which-suite-and-group-a-connection.patch` |

## Taking a newer upstream

`python3 scripts/vendor.py hyper-raft <commit>` takes these crates at that commit and re-applies the
changes above by three-way merge. Take the commit kiln's `vendor/MANIFEST.toml` names, with kiln's
changes as kiln then records them, and move `[kiln]`'s commit in `vendor/MANIFEST.toml` to that kiln
revision: `crates/reliquary/tests/vendored.rs` holds the pins to it.
