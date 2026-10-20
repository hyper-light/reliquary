# Status

Updated 2026-10-20. A piece is done when its tests pass in the gate (`scripts/gates.sh`: Linux, with the
macOS and Windows lints) and the evidence its requirement names is recorded.

## Implemented

- **The bar** (kiln's CP-M1): a Cargo workspace whose lints deny panics, this repository's rules
  (`CLAUDE.md`), the requirement registry (`docs/requirements.toml`), the design's starting point
  (`docs/design/README.md`), the gate (`scripts/gates.sh`, run by kiln CI from `.kiln/ci.yaml`), and the
  vendoring tool (`scripts/vendor.py`, kiln's).
- **The `reliquary` binary** says what it is (`reliquary --version`, `reliquary --help`), and nothing more yet.

## Next

- **CP-T9.1**: Kiln's registry (`ci/registry`) moves here; the OCI distribution conformance suite passes; Shards' agent, harness and MCP artifacts are checked on push; kiln vendors it back.
- **CP-T9.2**: Skills and MCP servers, first-class, with the MCP Registry's API.
- **CP-T9.3**: The catalog and search.

## Open

Everything else the track's plan names (kiln plan `01a157fe104d7444a38c808b89f2fae0`): each goal, its criteria and how each is
measured are in `docs/requirements.toml`.
