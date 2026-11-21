# Amphora, vendored

Where Reliquary keeps its records (architecture D2): Amphora's `amphora-api`, the trait Reliquary
reaches records through (`Records`: rows with revisions, conditional transactions, one change feed),
for the seam and its simulator (plan RQ0.4); and Amphora's embedded store, which keeps them in
Reliquary's own process (plan RQ1.8) (kiln's docs/vendoring.md; plan RQ0.1).

- **Upstream:** https://github.com/hyper-light/amphora, branch `main`, commit
  `db991a05d1769648d1de0390ac80b04c979d150e` (2026-11-07), MIT (`LICENSE`): the head of Amphora's
  `main` on 2026-11-20. kiln vendors no Amphora at the kiln revision Reliquary's pins follow
  (`vendor/MANIFEST.toml`, `[kiln]`), so this pin is Amphora's own (`kiln_holds_none`), until kiln
  takes Amphora.
- **What is here:** upstream's root manifest and `LICENSE`; `crates/api`; the store, `crates/amphora`;
  the engine the store is built on, `crates/engine`, `engine-ext`, `engine-macros` and
  `engine-parser` (Turso's four crates, Amphora's own since 2026-10-20, `crates/engine/ORIGIN.md`); and
  Amphora's record of what it vendors (`vendor/MANIFEST.toml`, `vendor/patches`), so Amphora's pins are
  checked as this copy holds them (the first change below). Left out: Amphora's copy of hyper-raft (the
  store builds on `vendor/hyper-raft`), its docs, benchmarks, scripts and CI. Three of the store's unit
  tests include Amphora's docs, so the store's own tests don't build here; Reliquary builds the store,
  not its tests.
- **How it's built:** Reliquary's root `Cargo.toml` points `amphora` and `amphora-api` at their folders
  here (`[patch.crates-io]`), and `hyper-durable`, `hyper-raft`, `hyper-log` and `hyper-block`, which
  the store names by version, at `vendor/hyper-raft`.

## Changes from upstream

Each change is recorded in `vendor/patches/amphora/`, applied in this order on upstream's tree. Each is
Reliquary's, for building Amphora on kiln's one hyper-raft; the second changes Amphora's code, and is
offered to Amphora's owner (through the coordinator).

| File | Change | Why | Recorded as |
|---|---|---|---|
| `vendor/MANIFEST.toml`, `vendor/patches/hyper-raft/*` | Amphora's hyper-raft pin moved from `d5d5f8f6` to `e0613f52`, and its records of changes to hyper-raft made kiln's four (Reliquary's `vendor/patches/hyper-raft`) in place of Amphora's own four | Reliquary holds one copy of hyper-raft, kiln's, and a product it vendors must pin that copy with the same recorded changes (`scripts/vendor.py check`), as kiln holds Hex's (kiln's `vendor/patches/hex/0001`). Amphora's own changes but its fifth aren't called by what is built here (its third is hyper-rt's Darwin FIFO reactor; its fourth, a hyper-transport method the store doesn't call); the fifth is the second change below | `0001-amphora-s-pins-at-reliquary-s-one-hyper-raft.patch` |
| `crates/amphora/src/database/replicated.rs` | the held count told through hyper-durable's `Replica::suspect` and `trust`, their refusal of the election part on ticks (`Error::Settings("elections run on ticks")`) taken as the held part done (`held_alone`), in place of `Replica::suspect_held` and `trust_held` | those two are Amphora's own change to hyper-durable (its `vendor/patches/hyper-raft/0005`), which kiln's hyper-raft at `e0613f52` lacks. There, `suspect` and `trust` take the same held part (`RawNode::suspect_held`, `trust_held`) first, then tell the core, which a replica that elects on ticks refuses before any change, with an error that doesn't fence it; Amphora's replicated store elects on ticks always (`Elections::Ticks`). Its two tests of the held count (`a_verdict_told_while_following_stays_in_force_once_leading`, `a_member_the_detector_suspects_isnt_counted_in_a_held_quorum_until_trusted`) pass on kiln's hyper-raft with this change (2026-11-20) | `0002-the-store-s-held-verdicts-on-kiln-s-hyper-durable.patch` |

## Taking a newer upstream

`python3 scripts/vendor.py amphora <commit>` takes Amphora at that commit and re-applies the changes
above by three-way merge; a commit whose pins are kiln's, or whose store builds on kiln's hyper-durable,
makes a change nothing, and the re-vendor drops it. When kiln vendors Amphora, take the commit kiln
names and remove `kiln_holds_none`.
