# aws-lc-rs, vendored

The cryptography under hyper-tls, hyper-quic, hyper-transport and Mantle's crates, at the copy kiln
links: its release with the changes Mantle makes to it, so Mantle builds on this one copy (kiln's
docs/vendoring.md; plan RQ0.1).

- **Upstream:** https://github.com/aws/aws-lc-rs, tag `v1.18.2`, commit
  `9df497501436596d4da77109217806b3af7f56cb`, ISC and (Apache-2.0 or ISC) (`LICENSE`): the commit kiln's
  `vendor/MANIFEST.toml` names at kiln `e64eeb7c` (2026-11-20).
- **What is here:** the crate's folder, `aws-lc-rs`, at upstream's path, and the repository's
  `LICENSE`. Left out: the crate's `third_party/`, the NIST vectors only its own tests read (kiln's copy
  holds them; Reliquary doesn't run the crate's suite). aws-lc-sys is not here: Reliquary takes
  crates.io's 0.46.0, as kiln does, which the crate names by version (the first change below), built as
  kiln builds it: no CPU jitter entropy and never a system AWS-LC (Reliquary's `.cargo/config.toml`).
- **How it's built:** Reliquary's root `Cargo.toml` points `aws-lc-rs` at `vendor/aws-lc-rs/aws-lc-rs`
  (`[patch.crates-io]`), and Mantle's root manifest does the same here (`vendor/mantle/UPSTREAM.md`).

## Changes from upstream

Each change is recorded in `vendor/patches/aws-lc-rs/`, applied in this order on upstream's tree. They
are kiln's, byte for byte; kiln's `vendor/aws-lc-rs/UPSTREAM.md` says each in full.

| File | Change | Why | Recorded as |
|---|---|---|---|
| `aws-lc-rs/Cargo.toml` | the manifest standalone, as crates.io's package of `1.18.2` states it: its versions and lints resolved, aws-lc-sys and aws-lc-fips-sys by version | the repository's manifest inherits from its workspace and takes aws-lc-sys by path from a folder no one vendors; kiln's, carried as kiln records it | `0001-its-manifest-standalone-as-crates-io-s-package-states-it.patch` |
| `aws-lc-rs/src/digest.rs`, `src/digest/sha.rs`, `src/digest/state.rs`, `src/hmac.rs`, `src/aead.rs`, `src/aead/tls.rs`, `src/aead/cbc_hmac.rs`, `src/aead/aead_ctx.rs`, `src/aead/unbound_key.rs`, `src/key_wrap.rs`, `src/key_wrap/tests.rs`, `tests/jwe_rfc7516.rs`, `tests/tls13_vectored_seal.rs`, `Cargo.toml` | Mantle's: MD5 for legacy use, fallible digests, a digest's state saved and resumed, one-shot HMAC, TLS 1.3 vectored sealing, AES-CBC-HMAC, AES-192 key wrap, a cloneable `LessSafeKey` | what Mantle builds on; kiln's, carried as kiln records it | `0002-mantle-s-changes-md5-fallible-digests-a-digest-s-state-one.patch` |

## Taking a newer upstream

`python3 scripts/vendor.py aws-lc-rs <commit>` takes the crate at that commit and re-applies the
changes above by three-way merge. Take the commit kiln's `vendor/MANIFEST.toml` names, with kiln's
changes, and move `[kiln]`'s commit with it.
