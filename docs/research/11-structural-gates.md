# 11. The structural gates: what they read, and what the readings rest on

Status: research note, 2026-11-20, for plan RQ0.3 (`docs/design/gates.md`). Quotes are verbatim; each
claim carries its source; a conclusion derived here says so and from what.

## Sources

[1] Mantle's `scripts/check-recursion.py` at `89d365cc9bf3b1c6393662fa3647c2bf546b9176` (origin/perf-10k,
2026-11-18; sha256 of the file `0230c6a249808036056dc8dbae51b673bdce06d3cf4fa63acb0d93d782b68514`), and
Mantle's `deny.toml` and `scripts/gates.sh` at `f41d8344`.

[2] rustc 1.99.1, `compiler/rustc_middle/src/ty/vtable.rs` and `compiler/rustc_codegen_ssa/src/meth.rs`
(https://raw.githubusercontent.com/rust-lang/rust/1.99.1/…).

[3] LLVM Language Reference, `load` instruction, at llvmorg-21.1.0
(https://raw.githubusercontent.com/llvm/llvm-project/llvmorg-21.1.0/llvm/docs/LangRef.rst).

[4] R. Tarjan, "Depth-first search and linear graph algorithms", SIAM J. Comput. 1(2):146–160, 1972.

[5] J. Dean, D. Grove, C. Chambers, "Optimization of Object-Oriented Programs Using Static Class
Hierarchy Analysis", ECOOP 1995, LNCS 952.

[6] cargo-deny 0.22.0's documentation, `docs/src/checks/advisories/cfg.md` and
`docs/src/checks/licenses/cfg.md` (https://raw.githubusercontent.com/EmbarkStudios/cargo-deny/0.22.0/…),
and its release notes for 0.21.0 and 0.22.0 (GitHub releases).

[7] Hex's `crates/hex-testkit/src/lib.rs` and `scripts/gates.sh` at `cfe430167` (origin/main).

[8] The Rust Programming Language, ch. 11.2, "Controlling How Tests Are Run"
(https://raw.githubusercontent.com/rust-lang/book/main/src/ch11-02-running-tests.md).

[9] POSIX.1-2024, `write()` (https://pubs.opengroup.org/onlinepubs/9799919799/functions/write.html).

[10] Python 3.14 documentation, "Using Python on Windows", "Finding modules"
(https://raw.githubusercontent.com/python/cpython/3.14/Doc/using/windows.rst).

[11] kiln's runners and CI labels for `hyper-light/reliquary`, read 2026-11-20 (`runners`,
`ci.workflows.labels`, `ci.routing.explain`); Docker Hub's tag API and Red Hat's registry for the images
of `.kiln/distros.yaml`, read 2026-11-20.

## Findings

F1. Mantle's check reads direct calls alone. [1]: "What a direct-call graph does not see: - Indirect
calls: a `dyn Trait` method (a vtable slot loaded and called), a function pointer, and so any recursion
that passes through one." Plan RQ0.3's T-RQ0.3.2 asks for exactly that case: two functions calling each
other through a trait object.

F2. A Rust vtable starts with three entries, then the methods. [2], `vtable.rs`: "pub const
COMMON_VTABLE_ENTRIES: &'tcx [VtblEntry<'tcx>] = &[VtblEntry::MetadataDropInPlace, VtblEntry::MetadataSize,
VtblEntry::MetadataAlign];" So slot 0 is the drop, and a method's slot is a fixed offset in every
vtable of its trait.

F3. A call through a trait object is a load from a vtable at a byte offset, marked invariant. [2],
`meth.rs`: "let gep = bx.inbounds_ptradd(llvtable, bx.const_usize(vtable_byte_offset)); let ptr =
bx.load(llty, gep, ptr_align); // VTable loads are invariant. bx.set_invariant_load(ptr);" [3]: "If a
load instruction tagged with the ``!invariant.load`` metadata is executed, the memory location
referenced by the load has to contain the same value at all points in the program where the memory
location is dereferenceable". Derived: in -O0 IR, a call whose callee is such a load is a call through a
vtable slot at that offset (measured on the pinned toolchain: `@vtable.N` constants, the slot load and
the call, in the fixtures of crates/reliquary/tests/gates.rs).

F4. Resolving a virtual call to every method a class hierarchy allows at that call is class hierarchy
analysis [5], an over-approximation of what can run: a cycle it finds is a cycle the code allows, and
one it misses must pass through code it does not read.

F5. Strongly connected components in linear time, depth first [4]; Mantle walks it on an explicit stack
[1], as CLAUDE.md §11 asks of Reliquary's own code.

F6. cargo-deny's exceptions can be scoped and made to expire. [6]: "A list of PackageSpecs of the
crates that are allowed to directly depend on the crate(s) that the advisory applies to." and "Allows an
advisory to be ignored only until an expiry is reached, after which the advisory is reported as normal".
Its licenses check skips dev-dependencies by default: "If `true`, licenses are checked even for
`dev-dependencies`. By default this is false as `dev-dependencies` are not used by downstream crates,
nor part of binary artifacts." 0.21.0 "fixed an issue where `--target` could not be passed multiple
times in the CLI".

F7. Hex's skip record [7]: "A test that can't run here … says so with [`skip`] … a line on stderr, which
libtest captures and shows only for a test that fails, and the same line appended to the skip record,
which libtest's capture can't reach. scripts/gates.sh points `HEX_SKIPS` at [`RECORD`] under the target
directory, empties it before `cargo test` and prints it after, pass or fail". [8]: "By default, if a
test passes, Rust's test library captures anything printed to standard output."

F8. An append lands whole at the end. [9]: "If the O_APPEND flag of the file status flags is set, the
file offset shall be set to the end of the file prior to each write and no intervening file
modification operation shall occur between changing the file offset and the write operation."

F9. The embeddable Python's path is its `._pth` file's. [10]: "Include a ``._pth`` file alongside your
executable containing the directories to include. This will ignore paths listed in the registry and
environment variables". Derived: a script run by it imports its sibling modules only once it puts its
own directory on `sys.path`.

F10. What runs where, read 2026-11-20 [11]: macOS arm64 machines (several with a docker executor),
one Windows x86_64 machine (jupiter: cargo and git, no Python), one Linux x86_64 machine (adacachy,
which runs production) and kiln's Kubernetes pods (Linux x86_64). No Linux arm64 machine and no
Windows arm64 machine. Images: `debian:trixie`, `almalinux:10`, `opensuse/leap:16.0`, `alpine:3.24`
and `rust:1.99.1-trixie` exist for amd64 and arm64; `archlinux:latest` for amd64 alone;
`registry.access.redhat.com/ubi10/ubi` answers.
