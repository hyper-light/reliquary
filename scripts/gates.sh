#!/usr/bin/env bash
# Every gate of CLAUDE.md, in order, and the first failure ends it with its status (plan RQ0.3,
# AC-RQ0.3.1): the numbers, the format, the lints of each of the five targets, the production check,
# recursion, the dependency graph, the tests with the skip record after them, and the vendored trees.
# Run it before each commit; kiln CI runs it on every push and pull request (.kiln/ci.yaml).
#
# `gates.sh here` runs the gates whose answer depends on the platform they run on: the production
# check of this host's target, recursion in this host's build, and the tests. kiln CI runs it on each
# of the five targets and in each Linux distribution's container; the others read the same sources
# and give the same answer anywhere, the lints of every target included, so they run once.
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.."
gate() { printf '::: %s\n' "$*" >&2; "$@"; }
# aws-lc-sys assembles with NASM on x86-64 Windows; with none, from the objects it ships, as kiln's own
# gate builds it (kiln's ci/gate.sh at 8299f4ced).
case "$(uname -s)" in
MINGW* | MSYS* | CYGWIN*)
  if [ -z "${AWS_LC_SYS_PREBUILT_NASM:-}" ] && ! command -v nasm >/dev/null; then
    export AWS_LC_SYS_PREBUILT_NASM=1
  fi
  ;;
esac
if [ "${1:-}" = "here" ]; then
  gate bash scripts/check-production.sh
  gate python3 scripts/check-recursion.py
  gate bash scripts/test.sh
  exit 0
elif [ "$#" -ne 0 ]; then
  printf 'usage: gates.sh [here]\n' >&2
  exit 2
fi
# Reliquary's five targets (CLAUDE.md §12), which rust-toolchain.toml installs.
targets=(x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu aarch64-apple-darwin x86_64-pc-windows-msvc aarch64-pc-windows-msvc)
gate python3 scripts/check-contracts.py
gate cargo fmt --all --check
# aws-lc-sys, under the vendored aws-lc-rs (vendor/aws-lc-rs/UPSTREAM.md), compiles AWS-LC's C for each
# target: a target other than this host's gets zig as its C compiler and archiver (scripts/zig-cc and
# scripts/zig-ar, Hex's, as Hex's gate compiles it). A check links nothing.
host=$(rustc -vV | sed -n 's/^host: //p')
for t in "${targets[@]}"; do
  [ "$t" = "$host" ] && continue
  command -v zig >/dev/null || { printf '::: zig is needed to compile C for %s (scripts/zig-cc; scripts/install-zig puts it on PATH)\n' "$t" >&2; exit 1; }
  v=$(printf %s "$t" | tr - _)
  export "CC_$v=$PWD/scripts/zig-cc" "AR_$v=$PWD/scripts/zig-ar"
done
for t in "${targets[@]}"; do
  gate cargo clippy --workspace --all-targets --locked --target "$t" -- -D warnings
done
gate bash scripts/check-production.sh $(printf -- '--target %s ' "${targets[@]}")
gate python3 scripts/check-recursion.py
# cargo-deny at its pinned release, by its digest (scripts/install-cargo-deny), against deny.toml.
deny=$(sh scripts/install-cargo-deny)/cargo-deny
gate "$deny" --locked check advisories bans licenses sources
gate bash scripts/test.sh
gate python3 scripts/vendor.py check
