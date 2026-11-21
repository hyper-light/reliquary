#!/usr/bin/env bash
# The gate's tests (CLAUDE.md, "Gates"; plan RQ0.3: AC-RQ0.3.4, AC-RQ0.3.5): every test cites a
# requirement the registry holds and skips only through the skip record (scripts/check-tests.py);
# then the workspace's whole suite, every test binary whatever fails before it; and after it, pass
# or fail, the skip record: every test that skipped, what it lacked and why (crates/reliquary-testkit).
# Exits with the first failure's status. `--workspace DIR` runs it on another workspace (its tests'
# fixtures, crates/reliquary/tests/gates.rs); other arguments go to `cargo test`.
set -euo pipefail
scripts=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
workspace=$(cd -- "$scripts/.." && pwd)
if [ "${1:-}" = "--workspace" ]; then
  workspace=$(cd -- "$2" && pwd)
  shift 2
fi
cd -- "$workspace"
python3 "$scripts/check-tests.py" --root "$workspace"
target=$(cargo metadata --format-version 1 --no-deps --locked |
  python3 -c 'import json, sys; print(json.load(sys.stdin)["target_directory"])')
export RELIQUARY_SKIPS="$target/tmp/reliquary-skips.tsv"
rm -f -- "${RELIQUARY_SKIPS:?}"
status=0
cargo test --workspace --locked --no-fail-fast "$@" || status=$?
if [ -s "$RELIQUARY_SKIPS" ]; then
  printf '::: %s skipped (binary, test, missing, why):\n' "$(wc -l <"$RELIQUARY_SKIPS" | tr -d ' ')" >&2
  cat -- "$RELIQUARY_SKIPS" >&2
else
  printf '::: no test skipped\n' >&2
fi
exit "$status"
