#!/usr/bin/env bash
# Builds the `reliquary` binary a benchmark measures (CLAUDE.md §7, plan RQ0.7): release, from the
# commit named and nothing else in the working tree, its `--version` and every record it makes naming
# that commit (RELIQUARY_COMMIT). A build of anything but a clean commit names none, and its records
# are not of record (docs/design/measurement.md).
#
#   scripts/bench-build.sh <commit> <out-dir>      # writes <out-dir>/reliquary
#
# The checkout is a git worktree under BENCH_WORK (default ~/.cache/reliquary-bench-build, never /tmp),
# removed afterwards. Two builds compared under cachegrind go to directories whose paths are as long as
# each other's (`.../a` and `.../b`): cachegrind's counts move with a file name's length (research/07
# F18).
set -euo pipefail
rev=${1:?usage: scripts/bench-build.sh <commit> <out-dir>}
out=${2:?usage: scripts/bench-build.sh <commit> <out-dir>}
cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.."
commit=$(git rev-parse --verify "$rev^{commit}")
work=${BENCH_WORK:-$HOME/.cache/reliquary-bench-build}
mkdir -p "$work" "$out"
tree="$work/$commit"
cleanup() {
  git worktree remove --force "${tree:?}" 2>/dev/null || true
}
trap cleanup EXIT
# A tree a run before left behind, killed before its trap ran, goes first.
cleanup
git worktree prune
git worktree add --detach "${tree:?}" "$commit" >/dev/null
(
  cd -- "$tree"
  RELIQUARY_COMMIT=$commit nice -n 10 cargo build --release --locked -p reliquary --bin reliquary
)
target=${CARGO_TARGET_DIR:-$tree/target}
cp -- "$target/release/reliquary" "$out/reliquary"
"$out/reliquary" --version
