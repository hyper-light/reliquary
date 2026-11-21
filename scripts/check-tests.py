#!/usr/bin/env python3
"""Every test cites a requirement the registry holds, and no test skips unseen (CLAUDE.md §6;
plan RQ0.3: AC-RQ0.3.4, AC-RQ0.3.5, T-RQ0.3.5, T-RQ0.3.7).

Read from the workspace's crates (crates/, every .rs file: tests/, benches/, examples/ and each
`#[cfg(test)]` module), a test being a function under `#[test]` or `#[<path>::test]`:
- Its doc comment (the `///` or `/** */` lines among its attributes) names at least one ID of the
  form `AC-…` or `T-…` (AC-RQ0.3.4, T-T9.1.1), and every such ID it names is in
  docs/requirements.toml. A test with none fails, and so does one naming an ID the registry lacks.
- It skips only through the skip record: a `return` in its body (outside a closure, an async block
  or a nested function, whose `return`s are their own) is allowed only when its statement calls
  `skip(…)` (`return reliquary_testkit::skip(…)…;`), which writes the line scripts/test.sh prints
  after the suite. `#[ignore]` hides a test from the suite, and a `cfg` on a test, or on the module
  or file holding it, other than one true whenever `test` is (`cfg(test)`, `cfg(any(test, …))`),
  leaves it out where the cfg is false: each fails. A test that cannot run on a platform is
  compiled on every one and skips through the record where it cannot run.
Words in comments and literals are not read (scripts/rust_tokens.py).
"""
import pathlib
import re
import sys

# scripts/rust_tokens.py, beside this file: an embedded Python (the Windows leg's) puts no script's
# directory on its path.
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from rust_tokens import matching, tokens

ROOT = pathlib.Path(__file__).resolve().parent.parent
ID = re.compile(r"\b(?:AC|T)-[A-Z]+[0-9]+(?:\.[0-9]+)+\b")
REGISTERED = re.compile(r'\bid = "((?:AC|T)-[^"]+)"')
ITEM_WORDS = {"pub", "crate", "async", "const", "unsafe", "extern", "default", "safe"}


def registry(path):
    return set(REGISTERED.findall(path.read_text(encoding="utf-8")))


def attribute(toks, at):
    """The attribute opening at `toks[at]` (`#`, then `!`?, then `[`): (inner, its path as text, the
    tokens inside its parentheses, the index past its `]`)."""
    k = at + 1
    inner = toks[k].text == "!" if k < len(toks) else False
    if inner:
        k += 1
    end = matching(toks, k)
    body = toks[k + 1:end]
    path = []
    i = 0
    while i < len(body) and (body[i].kind == "word" or body[i].text == ":"):
        path.append(body[i].text)
        i += 1
    args = []
    if i < len(body) and body[i].text == "(":
        close = matching(body, i)
        args = body[i + 1:close]
    return inner, "".join(path), args, end + 1


def cfg_hides(args):
    """Whether a `cfg(…)` with these arguments can be false while `test` is true."""
    words = [t.text for t in args]
    if words == ["test"]:
        return False
    if words[:2] == ["any", "("]:
        depth = 0
        for t in args[2:]:
            if t.text == "(":
                depth += 1
            elif t.text == ")":
                depth -= 1
            elif depth == 0 and t.text == "test":
                return False
    return True


def mentions(args, word):
    return any(t.kind == "word" and t.text == word for t in args)


def module_files(path, toks):
    """The files of the modules a file declares (`mod name;`), each with whether a `cfg` on it can
    hide its tests: [(file, hidden)]."""
    found, prelude, k = [], [], 0
    base = path.parent if path.name in ("lib.rs", "main.rs", "mod.rs") else path.parent / path.stem
    while k < len(toks):
        t = toks[k]
        if t.text == "#" and k + 1 < len(toks) and toks[k + 1].text in ("[", "!"):
            inner, name, args, k = attribute(toks, k)
            if not inner:
                prelude.append((name, args))
            continue
        if t.kind == "word" and t.text == "mod" and k + 2 < len(toks) and toks[k + 2].text == ";":
            name = toks[k + 1].text
            hidden = any(n == "cfg" and cfg_hides(a) for n, a in prelude)
            explicit = [a for n, a in prelude if n == "path" and a]
            candidates = [base / f"{name}.rs", base / name / "mod.rs"]
            for target in candidates:
                if target.exists():
                    found.append((target.resolve(), hidden))
                    break
        if t.kind != "doc" and not (t.kind == "word" and t.text in ITEM_WORDS):
            prelude = []
        k += 1
    return found


def excluded(toks, start, end):
    """The token ranges inside a test body whose `return`s are not the test's: nested functions'
    bodies, closures' block bodies and async blocks."""
    ranges = []
    k = start
    while k < end:
        t = toks[k]
        if t.kind == "word" and t.text == "fn":
            j = k + 1
            while j < end and toks[j].text != "{" and toks[j].text != ";":
                j += 1
            if j < end and toks[j].text == "{":
                close = matching(toks, j)
                ranges.append((j, close))
                k = close
        elif t.kind == "word" and t.text == "async":
            j = k + 1
            if j < end and toks[j].text == "move":
                j += 1
            if j < end and toks[j].text == "{":
                close = matching(toks, j)
                ranges.append((j, close))
                k = close
        elif t.text == "|" and k > start and toks[k - 1].text in ("(", ",", "=", "{", ";", "move", "return", ">", ":", "[", "!"):
            # A closure's parameters, `|…|`, then maybe `-> T`, then a block body.
            j = k + 1
            while j < end and toks[j].text != "|":
                j += 1
            j += 1
            if j < end and toks[j].text == "-":
                while j < end and toks[j].text != "{":
                    j += 1
            if j < end and toks[j].text == "{":
                close = matching(toks, j)
                ranges.append((j, close))
                k = close
        k += 1
    return ranges


def early_returns(toks, start, end):
    """The lines of `return`s in a test body whose statement records no skip."""
    skipped = excluded(toks, start, end)
    lines = []
    for k in range(start, end):
        t = toks[k]
        if not (t.kind == "word" and t.text == "return"):
            continue
        if any(a < k < b for a, b in skipped):
            continue
        depth, j, records = 0, k + 1, False
        while j < end:
            u = toks[j]
            if u.text in "([{":
                depth += 1
            elif u.text in ")]}":
                depth -= 1
                if depth < 0:
                    break
            elif u.text == ";" and depth == 0:
                break
            if u.kind == "word" and u.text == "skip" and j + 1 < end and toks[j + 1].text == "(":
                records = True
            j += 1
        if not records:
            lines.append(t.line)
    return lines


def check_file(path, relative, hidden_file, known):
    """The failures of one file's tests, and how many tests it holds."""
    toks = tokens(path.read_text(encoding="utf-8", errors="replace"))
    failures, count = [], 0
    hidden = hidden_file
    modules = []  # (index of the closing brace, hides)
    prelude = []  # (kind, name or doc text, args, line)
    k = 0
    while k < len(toks):
        while modules and k > modules[-1][0]:
            modules.pop()
        t = toks[k]
        if t.kind == "doc":
            prelude.append(("doc", t.text, None, t.line))
            k += 1
            continue
        if t.text == "#" and k + 1 < len(toks) and toks[k + 1].text in ("[", "!"):
            line = t.line
            inner, name, args, k = attribute(toks, k)
            if inner:
                if name == "cfg" and cfg_hides(args):
                    hidden = True
            else:
                prelude.append(("attr", name, args, line))
            continue
        if t.kind == "word" and t.text in ITEM_WORDS:
            k += 1
            continue
        if t.kind == "word" and t.text == "mod" and k + 2 < len(toks) and toks[k + 2].text == "{":
            hides = any(kind == "attr" and name == "cfg" and cfg_hides(args) for kind, name, args, _ in prelude)
            modules.append((matching(toks, k + 2), hides))
            prelude = []
            k += 3
            continue
        attrs = [(name, args, line) for kind, name, args, line in prelude if kind == "attr"]
        is_test = any(name == "test" or name.endswith("::test") for name, _, _ in attrs)
        if is_test and t.kind == "word" and t.text == "fn" and k + 1 < len(toks) and toks[k + 1].kind == "word":
            count += 1
            name = toks[k + 1].text
            where = f"{relative}:{toks[k + 1].line}: {name}"
            docs = " ".join(text for kind, text, _, _ in prelude if kind == "doc")
            cited = ID.findall(docs)
            if not cited:
                failures.append(f"{where}: cites no requirement: name an AC-… or T-… ID of docs/requirements.toml in its doc comment (CLAUDE.md §6)")
            for unknown in sorted(set(cited) - known):
                failures.append(f"{where}: cites {unknown}, which docs/requirements.toml does not hold")
            for attr, args, line in attrs:
                if attr == "ignore":
                    failures.append(f"{where}: #[ignore] hides it from the suite: skip through the skip record where it cannot run")
                elif attr == "cfg" and cfg_hides(args):
                    failures.append(f"{where}: a cfg leaves it out where the cfg is false: compile it everywhere and skip through the skip record")
                elif attr == "cfg_attr" and (mentions(args, "ignore") or mentions(args, "test")):
                    failures.append(f"{where}: a cfg_attr ignores it or makes it a test only sometimes: skip through the skip record instead")
            if hidden or any(hides for _, hides in modules):
                failures.append(f"{where}: it is in a module or file under a cfg that leaves it out where the cfg is false: skip through the skip record instead")
            j = k + 2
            while j < len(toks) and toks[j].text not in ("{", ";"):
                j += 1
            if j < len(toks) and toks[j].text == "{":
                close = matching(toks, j)
                for line in early_returns(toks, j + 1, close):
                    failures.append(f"{relative}:{line}: {name} returns early without recording a skip: return reliquary_testkit::skip(…) where it cannot run")
            prelude = []
            k += 2
            continue
        prelude = []
        k += 1
    return failures, count


def main():
    args = sys.argv[1:]
    root = ROOT
    if args[:1] == ["--root"] and len(args) == 2:
        root = pathlib.Path(args[1]).resolve()
    elif args:
        print("usage: check-tests.py [--root DIR]", file=sys.stderr)
        return 2
    known = registry(root / "docs" / "requirements.toml")
    files = sorted(p for p in (root / "crates").rglob("*.rs") if "/target/" not in f"/{p.relative_to(root).as_posix()}")
    # A module's file is hidden when a cfg on its `mod` can be false while `test` is true, and so is
    # every module below it: a walk over the declarations with a queue, each file once.
    hidden, queue, declared = set(), [], {}
    for path in files:
        children = module_files(path, tokens(path.read_text(encoding="utf-8", errors="replace")))
        declared[path.resolve()] = children
        queue.extend(child for child, hides in children if hides)
    while queue:
        path = queue.pop()
        if path in hidden:
            continue
        hidden.add(path)
        queue.extend(child for child, _ in declared.get(path, []))
    failures, tests = [], 0
    for path in files:
        found, count = check_file(path, path.relative_to(root).as_posix(), path.resolve() in hidden, known)
        failures.extend(found)
        tests += count
    for failure in failures:
        print(failure, file=sys.stderr)
    print(f"check-tests: {tests} tests in {len(files)} files, each citing what docs/requirements.toml holds ({len(known)} IDs)"
          if not failures else f"check-tests: {len(failures)} failures in {tests} tests",
          file=sys.stderr if failures else sys.stdout)
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
