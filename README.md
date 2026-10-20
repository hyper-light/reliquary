# reliquary
Artifact registry for agents, skills, and more.

Reliquary is the registry for agents, skills and more: build once, equip every agent. It serves OCI images, Shards and Firecracker microVM images, and every artifact the Agentfile spec
names: agents, harnesses, skills and MCP servers. Its records are in Amphora and its bytes on Mantle.

**Status:** the repository has its bar (`CLAUDE.md`, the gate, the requirement registry); the
`reliquary` binary says what it is, and nothing more yet. `STATUS.md` says what works and what's next;
`docs/design/` is the design.

```sh
cargo build --release      # target/release/reliquary
bash scripts/gates.sh      # every gate, as kiln CI runs them
```

MIT licensed (`LICENSE`).
