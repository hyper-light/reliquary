#!/usr/bin/env python3
"""`unsafe` only in the files CLAUDE.md names (CLAUDE.md §12; plan RQ0.3, T-RQ0.3.3).

The workspace denies `unsafe_code` (Cargo.toml's lints), so `unsafe` compiles only under an
`#[allow(unsafe_code)]` or `#[expect(unsafe_code)]`; clippy's `undocumented_unsafe_blocks` holds
each block to its `// SAFETY:` comment. This check holds the rest of §12: a file of the workspace's
crates (crates/, every target, tests included) may use `unsafe` (a block, a function, an impl, a
trait, an extern block, an `unsafe(…)` attribute) or allow the lints that deny it only when §12
names it, as a `path` in backticks with its reason. Each file §12 names must exist and use
`unsafe`, so the list says what the code does now. Words in comments and literals are not read
(scripts/rust_tokens.py).
"""
import pathlib
import re
import sys

# scripts/rust_tokens.py, beside this file: an embedded Python (the Windows leg's) puts no script's
# directory on its path.
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from rust_tokens import tokens

ROOT = pathlib.Path(__file__).resolve().parent.parent
# The lints whose allowing lets `unsafe` compile (rustc's lint names).
LINTS = {"unsafe_code", "unsafe_op_in_unsafe_fn"}
NAMED = re.compile(r"`([^`\s]+\.rs)`")


def named(claude):
    """The files §12 of CLAUDE.md names, each in backticks: its section, from its heading to the next."""
    text = claude.read_text(encoding="utf-8")
    section = re.search(r"^## 12\..*?(?=^## |\Z)", text, re.M | re.S)
    if section is None:
        return None
    return set(NAMED.findall(section.group(0)))


def uses(path):
    """The lines where a file uses `unsafe`, or allows the lints that deny it."""
    found = []
    toks = tokens(path.read_text(encoding="utf-8", errors="replace"))
    for k, t in enumerate(toks):
        if t.kind != "word":
            continue
        if t.text == "unsafe":
            found.append((t.line, "`unsafe`"))
        elif t.text in LINTS and k >= 2 and toks[k - 1].text in ("(", ",", "::"):
            # Inside an allow/expect/warn attribute: walk back to the attribute's name.
            j = k - 1
            depth = 0
            while j >= 0:
                if toks[j].text == ")":
                    depth += 1
                elif toks[j].text == "(":
                    if depth == 0:
                        break
                    depth -= 1
                j -= 1
            if j >= 1 and toks[j - 1].text in ("allow", "expect", "warn"):
                found.append((t.line, f"`{toks[j - 1].text}({t.text})`"))
    return found


def main():
    args = sys.argv[1:]
    root = ROOT
    if args[:1] == ["--root"] and len(args) == 2:
        root = pathlib.Path(args[1]).resolve()
    elif args:
        print("usage: check-unsafe.py [--root DIR]", file=sys.stderr)
        return 2
    allowed = named(root / "CLAUDE.md")
    if allowed is None:
        print("check-unsafe: CLAUDE.md has no §12, which names the files `unsafe` may be in", file=sys.stderr)
        return 1
    failures, users, files = [], set(), 0
    for path in sorted((root / "crates").rglob("*.rs")):
        relative = path.relative_to(root).as_posix()
        if "/target/" in f"/{relative}":
            continue
        files += 1
        found = uses(path)
        if found:
            users.add(relative)
        if found and relative not in allowed:
            for line, what in found:
                failures.append(f"{relative}:{line}: {what} in a file CLAUDE.md §12 does not name (CLAUDE.md §12)")
    for relative in sorted(allowed - users):
        failures.append(f"CLAUDE.md §12 names {relative}, which does not use `unsafe`: take it off the list")
    for failure in failures:
        print(failure, file=sys.stderr)
    print(f"check-unsafe: {files} files read, {len(users)} using `unsafe`, {len(allowed)} named by CLAUDE.md §12",
          file=sys.stderr if failures else sys.stdout)
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
