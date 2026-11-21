#!/usr/bin/env python3
"""The rule on numbers, as a gate (docs/design/numbers.md; plan RQ0.2).

Every number Reliquary's production code runs with has a row in docs/design/constants.md: its file
and key, its kind, the value the code has, and what establishes it. This is Mantle's checker
(mantle scripts/check-contracts.py at f41d8344) with Reliquary's differences, each stated in
docs/design/constants.md:

- It reads what Mantle's reads, each `const` and `static` holding a number and each literal
  parameter, and every other literal but 0 and 1 wherever it stands: an operand, a comparison, a
  pattern, an array's length, any call's argument. An estimator's figures stand in its arithmetic.
- It reads every Rust file of crates/ outside a crate's tests/, benches/ and examples/. A vendored
  crate is its owner's to inventory, and is read only where docs/design/constants.md's "Vendored
  crates" section names its path, as Mantle's `VENDORED` names the crates on Mantle's write path.
- It forgives nothing: a row of a kind the rule refuses fails, and so does any item in
  scripts/arbitrary-open.txt, which stays empty. There is no `--known`.
- A `learned` row names what it measures, on what, where that is kept and how it follows a change;
  a `derived` row its model, its inputs and the test that computes its value; a `counted` row the
  test that pins its value. The test must exist and name the constant.

Test code is not production code: an item under a `cfg` that requires `test` or marked `#[test]`, a
file whose first item is `#![cfg(test)]`, and a file every declaration of which is test code (rustc
compiles a file only where a `mod` declaration or an `include!` names it).
crates/reliquary/tests/contracts.rs holds these rules to fixture trees.
"""
import hashlib
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
INVENTORY = "docs/design/constants.md"
OPEN_LIST = "scripts/arbitrary-open.txt"

# Kinds the rule accepts as a basis (docs/design/constants.md, "Kinds").
ACCEPTED = {"format", "external", "derived", "cited", "learned", "counted"}
# Kinds the rule refuses, and what each is.
REFUSED = {
    "picked": "set by hand, or a bound or floor taken as the operating value",
    "knee": "read off one measured curve by a chosen criterion and frozen",
    "once": "measured once, on one machine",
    "copied": "another system's tuning",
    "guessed": "a guessed timeout, deadline or wait",
    "inherited": "derived from a refused input",
    "open": "nothing establishes it yet",
}
# A word a negation stands before, which then names what the basis is not.
NOT = r"(?<!\bnot )(?<!\bnever )(?<!\bno )(?<!\bnor )"
# A basis the rule refuses, whatever kind its row claims: each pattern and what it names. The
# machines named are the house's (CLAUDE.md, "Working here"), as their hosts are written.
REFUSED_BASES = (
    (
        re.compile(
            r"\bmeasured once\b|\bonce,? on one\b"
            r"|\b(?:measured|timed|benchmarked|calibrated|profiled|tuned|observed)\s+on\s+"
            r"(?:(?:the|this|that|one|a|an|my|our)\s+(?:box|machine|host|runner|laptop|server|workstation|computer|mac|vm|instance)\b"
            r"|M\d+\b)",
            re.I,
        ),
        "one machine's measurement",
    ),
    (re.compile(r"\b(?:fedora|jupiter|adacachy)\b"), "one machine's measurement"),
    (
        re.compile(
            NOT + r"\b(?:knee|elbow|sweet spot)\b|\bread off (?:the|a|one) (?:measured )?(?:curve|graph|plot|sweep|chart)\b"
            r"|\bone (?:measured )?curve\b",
            re.I,
        ),
        "a point read off one curve",
    ),
    (re.compile(NOT + r"\b(?:picked|hand-set|hand-picked|arbitrar(?:y|ily)|guess(?:ed|es)?|rule of thumb)\b", re.I), "a picked value"),
)
# Another system whose tuning is not a basis, unless the row is `format` or `external` (a fact about
# that system Reliquary must meet: its encoding, its interface) or cites the primary source that
# measured or prescribes the value.
OTHER_SYSTEMS = re.compile(
    r"\b(?:RocksDB|LevelDB|botocore|AWS CLI|AWS SDKs?|quinn|quiche|Chromium|Google QUIC|tokio|HDFS|GFS|Tectonic"
    r"|CNCF Distribution|docker/distribution|Docker|containerd|Podman|CRI-O|skopeo|crane|ORAS|Harbor|Zot|Quay"
    r"|Artifactory|Nexus|GHCR|ECR|GCR|ACR|Dragonfly|Kraken|Spegel|eStargz|stargz|Nydus|SOCI|casync|desync"
    r"|nginx|Envoy|HAProxy|kubelet|Kubernetes|Mantle|Shards|Amphora|Quiver|Veil|Focal|Slates|kiln)\b"
)
PRIMARY = re.compile(
    r"\bRFC ?\d+|\bIETF\b|\bdraft-ietf|\bISO(?:/IEC)? ?\d+|\bIEEE\b|\bPOSIX\b|\bet al\.|\bdoi:"
    r"|\bresearch/\d+|\b(?:distribution|image|runtime)-spec\b"
)
# The labels a row's basis states its parts under: `Label: text`, up to the next label.
CLAUSE = re.compile(r"\b(Measures|On|Kept|Follows|Model|Inputs|Test):")
LEARNED_CLAUSES = ("Measures", "On", "Kept", "Follows")
TESTED_CLAUSES = {"derived": ("Model", "Inputs", "Test"), "counted": ("Test",)}
# What a learned value is kept under (AC-RQ0.2.3; plan RQ0.8): never the machine alone.
SUBJECTS = re.compile(
    r"\b(?:paths?|devices?|peers?|repositor(?:y|ies)|workload class(?:es)?|class(?:es)? of (?:requests?|workloads?))\b",
    re.I,
)

SUFFIX = r"(?:u8|u16|u32|u64|u128|usize|i8|i16|i32|i64|i128|isize|f32|f64)?"
# The types a constant of no literal still counts by: a number of a numeric type, however computed.
NUMERIC = {
    "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize", "f32",
    "f64", "Duration", "NonZero", "NonZeroU8", "NonZeroU16", "NonZeroU32", "NonZeroU64", "NonZeroU128",
    "NonZeroUsize", "Wrapping", "Saturating",
}
# A constant holding a cell is filled at run time: its literals are read where they stand.
CELLS = ("Atomic", "OnceLock", "OnceCell", "LazyLock", "LazyCell", "Cell", "RefCell", "Mutex", "RwLock")
# A value longer than this is stated as the SHA-256 of its tokens.
VALUE_WIDTH = 100

# Mantle's literal parameters: calls whose first argument is a count, a duration or a fallback.
CALLS = {
    "from_secs", "from_millis", "from_micros", "from_nanos", "from_secs_f64", "from_secs_f32",
    "with_capacity", "channel", "sync_channel", "bounded", "take", "repeat", "sleep", "reserve",
    "resize", "truncate", "min", "max", "clamp", "unwrap_or", "map_or",
}
# Of those, the calls whose argument one is "at least one" (a floor, a divisor), not a choice.
FLOORS = {"min()", "max()", "clamp()", "unwrap_or()", "map_or()"}
# Tokens after which a literal begins a value of its own, and tokens that end one: a run of literals
# between them (`64 << 10`, `7 * 24 * 3600`) is one number.
STARTS = {"(", "[", "{", ",", "=", ":", ";", "=>", "return", "break", ""}
ENDS = {",", ";", ")", "]", "}", "=>", ""}
# Words before a `(` that does not open a call, and before a `-` that is a sign.
KEYWORDS = {
    "if", "while", "match", "return", "in", "for", "loop", "else", "let", "mut", "move", "as", "break",
    "continue", "where", "impl", "dyn", "fn", "unsafe", "async", "await", "ref", "box", "yield",
}

TOKEN = re.compile(
    r"(?P<nl>\n)|(?P<ws>[ \t\r\f]+)|(?P<lc>//[^\n]*)|(?P<bc>/\*)"
    r"|(?P<raw>b?r(?P<hashes>#*)\")"
    r"|(?P<str>b?\"(?:[^\"\\]|\\.)*\")"
    r"|(?P<chr>b?'(?:[^'\\\n]|\\(?:u\{[0-9a-fA-F]{1,6}\}|x[0-9a-fA-F]{2}|.))')"
    r"|(?P<life>'[A-Za-z_][A-Za-z0-9_]*)"
    r"|(?P<num>(?:0x[0-9a-fA-F_]+|0b[01_]+|0o[0-7_]+|[0-9][0-9_]*(?:\.(?![.A-Za-z_])[0-9_]*)?(?:[eE][+-]?[0-9_]+)?)" + SUFFIX + r")"
    r"|(?P<id>[A-Za-z_][A-Za-z0-9_]*)"
    r"|(?P<p>::|<<=|>>=|<<|>>|=>|->|==|!=|<=|>=|\.\.=|\.\.\.|\.\.|&&|\|\||[-+*/%^&|]=|.)",
    re.S,
)
ROW = re.compile(r"^\|\s*`([^`]+)`\s+`([^`]+)`\s*\|\s*([a-z]+)\s*\|\s*(`+)(.*?)\4\s*\|(.*)\|\s*$")
# The inventory's rows name Rust files; its other tables are not read.
TABLE_ROW = re.compile(r"^\|\s*`(?:crates|vendor)/[^`]+`\s+`")
# A vendored path the gate reads: a bullet of the inventory's "Vendored crates" section.
VENDORED_SECTION = "## Vendored crates"
VENDORED_PATH = re.compile(r"^-\s+`(vendor/[^`]+)`")
# A constant's key: its name, after the scopes it is declared in; a literal parameter's has a `:`.
CONSTANT_KEY = re.compile(r"(?:[A-Za-z_][A-Za-z0-9_]*::)*([A-Za-z_][A-Za-z0-9_]*)(?:#\d+)?")
# The directories of a crate that hold no production code (CLAUDE.md: "Production code").
NOT_PRODUCTION = {"tests", "benches", "examples", "target"}


def tokens(text):
    """The tokens of Rust source as (kind, text, line): idents, numbers, strings and punctuation,
    comments dropped. A raw string's text is its body."""
    out, i, n, line = [], 0, len(text), 1
    while i < n:
        m = TOKEN.match(text, i)
        kind = m.lastgroup
        if kind == "nl":
            line += 1
            i = m.end()
        elif kind in ("ws", "lc"):
            i = m.end()
        elif kind == "bc":
            depth, j = 1, m.end()
            while depth:
                a, b = text.find("/*", j), text.find("*/", j)
                if b < 0:
                    j = n
                    break
                if 0 <= a < b:
                    depth, j = depth + 1, a + 2
                else:
                    depth, j = depth - 1, b + 2
            line += text.count("\n", i, j)
            i = j
        elif kind in ("raw", "hashes"):
            end = '"' + (m.group("hashes") or "")
            j = text.find(end, m.end())
            j = n if j < 0 else j + len(end)
            out.append(("str", text[m.end():j - len(end)], line))
            line += text.count("\n", i, j)
            i = j
        elif kind in ("str", "chr"):
            out.append(("str", m.group(0), line))
            line += m.group(0).count("\n")
            i = m.end()
        else:
            out.append((kind, m.group(0), line))
            i = m.end()
    return out


def requires_test(toks):
    """Whether a `cfg` predicate's tokens (inside its parentheses) hold only in test builds:
    `test`, or `all(..)` with an argument that requires it."""
    if [t for _, t, _ in toks] == ["test"]:
        return True
    if len(toks) >= 3 and toks[0][1] == "all" and toks[1][1] == "(":
        depth, start, args = 0, 2, []
        for k in range(2, len(toks)):
            t = toks[k][1]
            if t in "([{":
                depth += 1
            elif t in ")]}":
                if depth == 0:
                    args.append(toks[start:k])
                    break
                depth -= 1
            elif t == "," and depth == 0:
                args.append(toks[start:k])
                start = k + 1
        return any(requires_test(a) for a in args if a)
    return False


def attribute_at(toks, i):
    """At a `#` starting an attribute: (end index past its `]`, inner, whether it makes its item
    test code, path string or None)."""
    inner = i + 1 < len(toks) and toks[i + 1][1] == "!"
    k = i + (2 if inner else 1)
    if k >= len(toks) or toks[k][1] != "[":
        return None
    j = matching(toks, k)
    body = toks[k + 1:j]
    words = [t for _, t, _ in body]
    test = words == ["test"] or (len(body) >= 3 and words[0] == "cfg" and words[1] == "(" and requires_test(body[2:-1]))
    path = None
    if len(body) == 3 and words[0] == "path" and words[1] == "=" and body[2][0] == "str":
        path = body[2][1].strip('"')
    return j + 1, inner, test, path


def item_end(toks, i):
    """The index past the item or statement starting at `i`: its `;` at depth zero, or the `}`
    closing its first brace when that brace comes first (a function, module, impl, type)."""
    depth, j = 0, i
    head = [t for _, t, _ in toks[i:i + 4]]
    semicolon_item = any(t in ("const", "static", "let", "use", "type") for t in head[:2]) and "fn" not in head[:3]
    while j < len(toks):
        t = toks[j][1]
        if t in "([{":
            if t == "{" and depth == 0 and not semicolon_item:
                return matching(toks, j) + 1
            depth += 1
        elif t in ")]}":
            if depth == 0:
                return j
            depth -= 1
        elif t == ";" and depth == 0:
            return j + 1
        j += 1
    return j


def matching(toks, j):
    """The index of the bracket closing the one at `j`."""
    depth = 0
    for k in range(j, len(toks)):
        t = toks[k][1]
        if t in "([{":
            depth += 1
        elif t in ")]}":
            depth -= 1
            if depth == 0:
                return k
    return len(toks) - 1


class Source:
    """One file's tokens, split into production and test code, with its `mod` declarations and
    `include!`s: (target path, whether test code declares it)."""

    def __init__(self, path):
        self.path = path
        self.toks = tokens(path.read_text(encoding="utf-8"))
        self.marked = False
        self.test = [False] * len(self.toks)
        self.imports = []
        self.split()

    def split(self):
        toks, i = self.toks, 0
        # The inline modules open at each point: (closing index, directory part, test code).
        inline = []
        pending_test, pending_path = False, None
        first_item = True
        while i < len(toks):
            while inline and i > inline[-1][0]:
                inline.pop()
            in_test = any(t for _, _, t in inline)
            if toks[i][1] == "#":
                attr = attribute_at(toks, i)
                if attr:
                    end, inner, test, path = attr
                    if inner and test and first_item:
                        self.marked = True
                        self.test = [True] * len(toks)
                    elif not inner:
                        pending_test = pending_test or test
                        if path is not None:
                            pending_path = path
                    i = end
                    continue
            if toks[i][1] not in ("pub", "(", "crate", "super", "self", "in", ")"):
                first_item = False
            gated = pending_test or in_test
            if pending_test:
                for k in range(i, item_end(toks, i)):
                    self.test[k] = True
            # `mod name;` or `mod name {`, after any visibility.
            if toks[i][1] == "mod" and i + 2 < len(toks) and toks[i + 1][0] == "id":
                name, after = toks[i + 1][1], toks[i + 2][1]
                parts = [p for _, p, _ in inline]
                if after == ";":
                    self.imports.append((self.resolve(parts, name, pending_path), gated or self.marked))
                elif after == "{":
                    inline.append((matching(toks, i + 2), pending_path or name, gated))
                    pending_test, pending_path = False, None
                    i += 3
                    continue
            if toks[i][1] == "include" and i + 4 < len(toks) and toks[i + 1][1] == "!" and toks[i + 3][0] == "str":
                target = (self.path.parent / toks[i + 3][1].strip('"')).resolve()
                self.imports.append((target, gated or self.marked))
            if pending_test:
                i = item_end(toks, i)
                pending_test, pending_path = False, None
                continue
            if toks[i][1] in (";", "}", "{"):
                pending_path = None
            i += 1

    def resolve(self, parts, name, path):
        base = self.path.parent
        own = self.path.stem not in ("lib", "main", "mod")
        if path is not None:
            if parts:
                inner = pathlib.Path(*parts) / path
                return (base / self.path.stem / inner if own else base / inner).resolve()
            return (base / path).resolve()
        directory = base / self.path.stem if own else base
        directory = directory / pathlib.Path(*parts) if parts else directory
        file = directory / f"{name}.rs"
        return (file if file.exists() else directory / name / "mod.rs").resolve()


def sources(paths):
    """Each file's Source. A file is test code when it is marked, or every declaration naming it is
    test code or sits in a test file. Marking only grows, so the loop ends within one pass a file."""
    found = {p: Source(p) for p in sorted(set(paths))}
    declared = {}
    for source in found.values():
        for target, gated in source.imports:
            declared.setdefault(target, []).append((source.path, gated))
    test_files = {p for p, s in found.items() if s.marked}
    for _ in range(len(found) + 1):
        grown = {
            p for p in found
            if p not in test_files and declared.get(p) and all(g or importer in test_files for importer, g in declared[p])
        }
        if not grown:
            break
        test_files |= grown
    for p in test_files:
        found[p].test = [True] * len(found[p].toks)
    return found


def literal_value(text):
    """A literal's number, or None for one this cannot read: its underscores and suffix dropped. A
    hex literal's suffix is an integer's, since `f32` is hex digits there."""
    text = text.replace("_", "")
    try:
        if text[:2] in ("0x", "0o", "0b"):
            return int(re.sub(r"[ui](?:8|16|32|64|128|size)$", "", text), 0)
        text = re.sub(SUFFIX + r"$", "", text)
        return float(text) if re.search(r"[.eE]", text) else int(text, 10)
    except ValueError:
        return None


def literal_run(toks, j):
    """The index past a literal-only expression at `j`: numbers joined by arithmetic and shifts,
    each optionally cast with `as`."""
    k, want_number = j, True
    while k < len(toks):
        kind, t, _ = toks[k]
        if want_number and kind == "num":
            k, want_number = k + 1, False
        elif not want_number and kind == "id" and t == "as" and k + 1 < len(toks) and toks[k + 1][0] == "id":
            k += 2
        elif not want_number and kind == "p" and t in ("*", "<<", "+", "-", "/", ">>"):
            k, want_number = k + 1, True
        else:
            break
    if want_number:
        k -= 1
    return k


def casts(toks, j):
    """The index past any `as` casts at `j`."""
    while j + 1 < len(toks) and toks[j][1] == "as" and toks[j + 1][0] == "id":
        j += 2
    return j


def render(toks):
    """A value's canonical text: its tokens, a space apart, joined where Rust joins them."""
    text = " ".join(t for _, t, _ in toks)
    for a, b in ((" ( ", "("), ("( ", "("), (" )", ")"), (" ,", ","), (" ::", "::"), (":: ", "::"), (" .", "."), (". ", "."), (" [ ", "["), ("[ ", "["), (" ]", "]"), (" ;", ";"), ("& ", "&"), (" : ", ": "), ("::< ", "::<"), (" >(", ">("), (" >::", ">::")):
        text = text.replace(a, b)
    # A sign joins its number where it follows no value: `-5`, `[-1, -2]`, `= -3`.
    return re.sub(r"(^|[\[(,;:=] ?)- (?=[0-9])", r"\1-", text)


def stated(text):
    """A value as a row states it: its canonical text, or the SHA-256 of that text when long."""
    if len(text) <= VALUE_WIDTH:
        return text
    return "sha256:" + hashlib.sha256(text.encode()).hexdigest()[:16]


def impl_label(head):
    """The type an impl's header names: the one after `for`, or the first, generics left out."""
    names, depth = [], 0
    after_for = None
    for _, t, _ in head:
        if t == "<":
            depth += 1
        elif t == ">":
            depth -= 1
        elif depth == 0 and t == "for":
            after_for = len(names)
        elif depth == 0 and t == "where":
            break
        elif depth == 0 and re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", t) and t not in ("dyn", "mut", "const", "unsafe"):
            names.append(t)
    if after_for is not None and after_for < len(names):
        return names[after_for]
    return names[0] if names else ""


# Tokens an item may follow: what ends the item before it, an attribute's `]`, a visibility.
ITEM_START = {"", ";", "}", "{", "]", "pub", ")", "unsafe", "async", "extern", "default"}


def scope_at(toks, i, before):
    """At a token opening a named scope (impl, trait, mod, fn, struct, enum): (closing index,
    label, kind); else None."""
    kind, t, _ = toks[i]
    if kind != "id" or i + 1 >= len(toks):
        return None
    if t in ("impl", "trait", "mod") and before in ITEM_START:
        j = i + 1
        while j < len(toks) and toks[j][1] not in ("{", ";"):
            j += 1
        if j < len(toks) and toks[j][1] == "{":
            head = toks[i + 1:j]
            label = impl_label(head) if t == "impl" else (head[0][1] if head else "")
            return matching(toks, j), label, t
        return None
    if t == "fn" and toks[i + 1][0] == "id":
        j, depth = i + 2, 0
        while j < len(toks):
            x = toks[j][1]
            if x in "([":
                depth += 1
            elif x in ")]":
                depth -= 1
            elif depth == 0 and x in ("{", ";"):
                break
            j += 1
        if j < len(toks) and toks[j][1] == "{":
            return matching(toks, j), toks[i + 1][1], "fn"
        return None
    if t in ("struct", "enum") and toks[i + 1][0] == "id" and before in ITEM_START:
        j = i + 2
        while j < len(toks) and toks[j][1] not in ("{", "(", ";"):
            j += 1
        if j < len(toks) and toks[j][1] in ("{", "("):
            return matching(toks, j), toks[i + 1][1], t
    return None


def constant_at(toks, i):
    """At `const` or `static` starting an item: (name, value tokens, end) of one read whole, a
    number of its own or of a numeric type; (None, None, end) for one whose literals are read where
    they stand (a cell, `const _`); None for a `const fn` or anything else."""
    k = i + 1
    if k >= len(toks) or toks[k][0] != "id" or toks[k][1] in ("fn", "unsafe", "async", "extern"):
        return None
    if toks[k][1] == "mut":
        k += 1
    name = toks[k][1]
    end = item_end(toks, i)
    eq = next((x for x in range(k, end) if toks[x][1] == "="), None)
    if eq is None:
        return None
    value = toks[eq + 1:end - 1] if toks[end - 1][1] == ";" else toks[eq + 1:end]
    typ = [x[1] for x in toks[k + 2:eq]]
    literal = any(x[0] == "num" for x in value)
    typed = "".join(typ) in NUMERIC or (any(x in NUMERIC for x in typ) and not any(x[0] == "str" for x in value))
    cell = any(x.startswith(CELLS) for x in typ)
    if name == "_" or cell:
        return None, None, end
    if literal or typed:
        return name, value, end
    return None, None, end


def unary(toks, j):
    """Whether the `-` at `j` is a sign: it follows no value."""
    if j == 0:
        return True
    kind, t, _ = toks[j - 1]
    if kind in ("num", "str", "life"):
        return False
    if kind == "id":
        return t in KEYWORDS
    return t not in (")", "]", "}", "?")


def callee(toks, start):
    """The call a value starting at `start` is a whole argument of: `name` or `name!`, or None for a
    tuple, an array, a block or a grouping."""
    depth, j = 0, start - 1
    while j >= 0:
        t = toks[j][1]
        if t in (")", "]", "}"):
            depth += 1
        elif t in ("(", "[", "{"):
            if depth == 0:
                break
            depth -= 1
        elif t == ";" and depth == 0:
            return None
        j -= 1
    if j < 1 or toks[j][1] != "(":
        return None
    kind, t, _ = toks[j - 1]
    if kind == "id" and t not in KEYWORDS:
        return t
    if t == "!" and j >= 2 and toks[j - 2][0] == "id":
        return toks[j - 2][1] + "!"
    return None


def assigned(toks, eq):
    """The name an `=` at `eq` assigns or binds: the binding after `let` (and `mut`), else the
    last identifier of the place."""
    j = eq - 1
    while j >= 0 and toks[j][1] not in (";", "{", "}"):
        j -= 1
    head = toks[j + 1:eq]
    words = [t for _, t, _ in head]
    if words[:1] == ["let"]:
        rest = [x for x in head[1:] if x[1] != "mut"]
        return rest[0][1] if rest and rest[0][0] == "id" else None
    if head and head[-1][0] == "id":
        return head[-1][1]
    return None


def assigned_by_let(toks, eq):
    """Whether the `=` at `eq` is a `let` binding's."""
    j = eq - 1
    while j >= 0 and toks[j][1] not in (";", "{", "}"):
        j -= 1
    return j + 1 < len(toks) and toks[j + 1][1] == "let"


def clap_field(toks, i):
    """The field a `default_value_t` or `default_value` attribute at `i` sets: the first identifier
    past its `]` followed by a `:`."""
    j = i
    while j < len(toks) and toks[j][1] != "]":
        j += 1
    while j < len(toks):
        if toks[j][0] == "id" and toks[j][1] not in ("pub", "crate"):
            if j + 1 < len(toks) and toks[j + 1][1] == ":":
                return toks[j][1]
        j += 1
    return "?"


def slot_of(toks, start, end, scopes):
    """Mantle's literal parameter a value from `start` to `end` is the whole of, or None: `field`
    of a struct literal or an enum's discriminant, `name` of an assignment or binding, `call()` of
    a count, duration or fallback argument, `--name` of a command-line default, `<<` of a size
    shift."""
    prev = toks[start - 1] if start else ("", "", 0)
    prev2 = toks[start - 2] if start > 1 else ("", "", 0)
    prev3 = toks[start - 3] if start > 2 else ("", "", 0)
    after = toks[end][1] if end < len(toks) else ""
    run = [t for _, t, _ in toks[start:end]]
    if prev[1] == ":" and prev2[0] == "id" and after in (",", "}") and prev3[1] in ("{", ",", "("):
        return prev2[1]
    if prev[1] == "=" and prev2[0] == "id" and after in (",", "}") and scopes and scopes[-1][2] == "enum":
        return prev2[1]
    if prev[1] == "=" and after == ";":
        return assigned(toks, start - 1)
    if prev[1] == "=" and prev2[1] == "default_value_t":
        return "--" + clap_field(toks, start)
    if prev[1] == "(" and prev2[0] == "id" and prev2[1] in CALLS and after in (")", ","):
        return prev2[1] + "()"
    if "<<" in run and prev[1] in ("(", ",", "=", "{", "[", "return") and after in (")", ",", ";", "}", "]"):
        return "<<"
    return None


def literal_array(toks, i):
    """The index of the `]` closing a literal array at `i`: two or more literal-only elements, a
    ladder or a table written in code; None for any other bracket."""
    k, elements = i + 1, 0
    while k < len(toks):
        if toks[k][0] != "num":
            return None
        end = literal_run(toks, k)
        elements += 1
        if end >= len(toks):
            return None
        if toks[end][1] == "]":
            return end if elements >= 2 else None
        if toks[end][1] != ",":
            return None
        k = end + 1
        if k < len(toks) and toks[k][1] == "]":
            return k if elements >= 2 else None
    return None


def numbers(source):
    """Every number of one file's production code: (key, value, line) for each `const` and `static`
    holding a number, each literal parameter, and every other literal but 0 and 1."""
    toks, test = source.toks, source.test
    found, counts, scopes = [], {}, []

    def add(key, value, line):
        counts[key] = counts.get(key, 0) + 1
        if counts[key] > 1:
            key = f"{key}#{counts[key]}"
        found.append((stated(key), stated(value), line))

    def scoped(slot, joiner=":"):
        here = "::".join(label for _, label, _ in scopes if label)
        return f"{here}{joiner}{slot}" if here else slot

    i = 0
    while i < len(toks):
        while scopes and i > scopes[-1][0]:
            scopes.pop()
        kind, t, line = toks[i]
        if test[i]:
            i += 1
            continue
        before = toks[i - 1][1] if i else ""
        scope = scope_at(toks, i, before)
        if scope:
            scopes.append(scope)
            i += 1
            continue
        if t in ("const", "static") and kind == "id" and before in ITEM_START:
            item = constant_at(toks, i)
            if item and item[0] is not None:
                name, value, end = item
                add(scoped(name, "::"), render(value), line)
                i = end
                continue
        if kind == "str" and before == "=" and i > 1 and toks[i - 2][1] == "default_value" and re.search(r"[0-9]", t):
            add(scoped("--" + clap_field(toks, i)), t, line)
            i += 1
            continue
        if t == "[" and i and before not in ("]", ")") and toks[i - 1][0] != "id":
            close = literal_array(toks, i)
            if close is not None:
                text = render(toks[i:close + 1])
                add(scoped(text), text, line)
                i = close + 1
                continue
        if kind != "num":
            i += 1
            continue
        if before == ".":
            # A tuple's field (`pair.0`, `nested.0.1`), not a number.
            i += 1
            continue
        start = i - 1 if before == "-" and unary(toks, i - 1) else i
        lead = toks[start - 1][1] if start else ""
        end = literal_run(toks, i)
        follow = toks[end][1] if end < len(toks) else ""
        if not (lead in STARTS and follow in ENDS):
            end = casts(toks, i + 1)
        text = ("-" if start < i else "") + render(toks[i:end])
        number = literal_value(t) if end == casts(toks, i + 1) else None
        slot = slot_of(toks, start, end, scopes)
        if slot is not None:
            if number == 0 and start == i:
                slot = None
            elif number == 1 and start == i and (slot in FLOORS or (lead == "=" and assigned_by_let(toks, start - 1))):
                slot = None
            elif slot.endswith("()"):
                slot = f"{slot[:-1]}{text})"
            elif slot == "<<":
                slot = f"({text})"
            if slot is not None:
                add(scoped(slot), text, line)
            i = end
            continue
        if number in (0, 1) and start == i:
            i = end
            continue
        call = callee(toks, start) if lead in ("(", ",") and follow in (")", ",") else None
        add(scoped(f"{call}({text})" if call else text), text, line)
        i = end
    return found


def rust_files(top):
    """Every Rust file under `top` (or `top` itself, a file) outside its own tests/, benches/ and
    examples/ (cargo's, at a crate's top) and any target/."""
    if top.is_file():
        return [top.resolve()]
    out = []
    for path in sorted(top.rglob("*.rs")):
        parts = path.relative_to(top).parts
        if parts[0] in NOT_PRODUCTION or "target" in parts:
            continue
        out.append(path.resolve())
    return out


def vendored_paths(root, text):
    """The vendored paths the inventory's "Vendored crates" section names, and its failures."""
    paths, failures, inside = [], [], False
    for n, line in enumerate(text.splitlines(), 1):
        if line.startswith("## "):
            inside = line.strip() == VENDORED_SECTION
            continue
        m = VENDORED_PATH.match(line) if inside else None
        if m:
            if (root / m.group(1)).exists():
                paths.append(root / m.group(1))
            else:
                failures.append(f"constants.md:{n}: {m.group(1)}: no such vendored path")
    return paths, failures


def production_numbers(root, vendored):
    """Every production number of crates/ and the vendored paths: {(file, key): (value, line)}."""
    files = []
    crates = root / "crates"
    for crate in sorted(p for p in crates.iterdir() if p.is_dir()) if crates.is_dir() else ():
        files.extend(rust_files(crate))
    for path in vendored:
        files.extend(rust_files(path))
    found = {}
    resolved = root.resolve()
    for path, source in sources(files).items():
        if all(source.test):
            continue
        rel = path.relative_to(resolved).as_posix()
        for key, value, line in numbers(source):
            found[(rel, key)] = (value, line)
    return found


def inventory(text):
    """The inventory's rows {(file, key): (kind, value, basis, line)}, and the rows it cannot read."""
    rows, failures = {}, []
    for n, line in enumerate(text.splitlines(), 1):
        if not TABLE_ROW.match(line):
            continue
        m = ROW.match(line)
        if not m:
            failures.append(f"constants.md:{n}: a row must be | `file` `key` | kind | `value` | basis |")
            continue
        key = (m.group(1), m.group(2))
        if key in rows:
            failures.append(f"constants.md:{n}: {key[0]} {key[1]}: a second row")
            continue
        rows[key] = (m.group(3), m.group(5).strip().replace("\\|", "|"), m.group(6).strip(), n)
    return rows, failures


def clauses(basis):
    """A basis's labelled parts, {label: text}."""
    marks = list(CLAUSE.finditer(basis))
    return {
        m.group(1): basis[m.end():marks[k + 1].start() if k + 1 < len(marks) else len(basis)].strip(" .;")
        for k, m in enumerate(marks)
    }


def test_names(root, clause, name):
    """Why a `Test:` clause does not name a test that computes `name`, or None: the clause names
    `path` `function`, the file holds a `#[test]` function of that name, and its body names `name`."""
    named = re.findall(r"`([^`]+)`", clause)
    if len(named) != 2:
        return "its Test: names no `file` `function`"
    rel, function = named[0], named[1].split("::")[-1]
    path = root / rel
    if not path.is_file():
        return f"its Test: names {rel}, which does not exist"
    toks = tokens(path.read_text(encoding="utf-8"))
    for i in range(len(toks) - 2):
        if toks[i][1] == "fn" and toks[i + 1][1] == function:
            j = i - 1
            while j >= 0 and toks[j][1] in ("pub", "async", "(", ")", "crate", "super", "in", "self"):
                j -= 1
            attributes = []
            while j >= 0 and toks[j][1] == "]":
                opener = j
                depth = 0
                while opener >= 0:
                    if toks[opener][1] == "]":
                        depth += 1
                    elif toks[opener][1] == "[":
                        depth -= 1
                        if depth == 0:
                            break
                    opener -= 1
                attributes.append([t for _, t, _ in toks[opener + 1:j]])
                j = opener - 2 if opener >= 1 and toks[opener - 1][1] == "#" else -1
            if ["test"] not in attributes:
                continue
            k = i + 2
            while k < len(toks) and toks[k][1] != "{":
                k += 1
            body = toks[k:matching(toks, k) + 1] if k < len(toks) else []
            if any(x[0] == "id" and x[1] == name for x in body):
                return None
            return f"its test {function} does not name {name}"
    return f"its Test: names no #[test] fn {function} in {rel}"


def check_rows(root, rows):
    """Each row's kind and basis, as the rule has them."""
    failures = []
    for (rel, key), (kind, value, basis, n) in sorted(rows.items()):
        where = f"constants.md:{n}: {rel} {key}"
        if kind in REFUSED:
            failures.append(f"{where} = {value}: {kind}, which the rule refuses ({REFUSED[kind]})")
            continue
        if kind not in ACCEPTED:
            failures.append(f"{where}: unknown kind {kind}")
            continue
        if not basis:
            failures.append(f"{where}: states no basis")
            continue
        for pattern, what in REFUSED_BASES:
            if pattern.search(basis):
                failures.append(f"{where}: a {kind} row whose basis is {what}")
                break
        other = OTHER_SYSTEMS.search(basis)
        if kind not in ("format", "external") and other and not PRIMARY.search(basis):
            failures.append(f"{where}: a {kind} row whose basis is another system's tuning ({other.group(0)})")
        parts = clauses(basis)
        if kind == "learned":
            missing = [c for c in LEARNED_CLAUSES if not parts.get(c)]
            if missing:
                failures.append(
                    f"{where}: a learned row names what it measures (Measures:), on what (On:), where it is kept (Kept:) "
                    f"and how it follows a change (Follows:): it has no {', '.join(c + ':' for c in missing)}"
                )
            elif not SUBJECTS.search(parts["On"]):
                failures.append(f"{where}: a learned row's On: names no path, device, peer, repository or workload class")
        elif kind in TESTED_CLAUSES:
            missing = [c for c in TESTED_CLAUSES[kind] if not parts.get(c)]
            constant = CONSTANT_KEY.fullmatch(key)
            if missing:
                if kind == "derived":
                    need = "its model (Model:), its inputs (Inputs:) and the test that computes its value (Test:)"
                else:
                    need = "the test that pins its value (Test:)"
                failures.append(f"{where}: a {kind} row names {need}: it has no {', '.join(c + ':' for c in missing)}")
            elif constant is None:
                failures.append(f"{where}: a {kind} value is held by its test only as a named constant: name it")
            else:
                why = test_names(root, parts["Test"], constant.group(1))
                if why:
                    failures.append(f"{where}: {why}")
    return failures


def open_items(root):
    """scripts/arbitrary-open.txt's failures: it exists, and lists nothing (AC-RQ0.2.2)."""
    path = root / OPEN_LIST
    if not path.exists():
        return [f"{OPEN_LIST}: missing"]
    failures = []
    for n, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        item = line.strip()
        if item and not item.startswith("#"):
            failures.append(
                f"{OPEN_LIST}:{n}: {item}: Reliquary forgives no number: give its row a kind the rule accepts, and take it off this list"
            )
    return failures


def contracts(root):
    """Every failure of the tree at `root`, and its production numbers."""
    path = root / INVENTORY
    if not path.exists():
        return [f"{INVENTORY}: missing"] + open_items(root), production_numbers(root, [])
    text = path.read_text(encoding="utf-8")
    vendored, failures = vendored_paths(root, text)
    found = production_numbers(root, vendored)
    rows, unread = inventory(text)
    failures.extend(unread)
    for (rel, key), (value, line) in sorted(found.items()):
        row = rows.get((rel, key))
        if row is None:
            failures.append(f"{rel}:{line}: {key} = {value} has no row in {INVENTORY}")
        elif row[1] != value:
            failures.append(f"constants.md:{row[3]}: {rel} {key}: the row says `{row[1]}`, the code `{value}` ({rel}:{line})")
    for (rel, key), (_, _, _, n) in sorted(rows.items()):
        if (rel, key) not in found:
            failures.append(f"constants.md:{n}: {rel} {key} names no number in production code")
    failures.extend(check_rows(root, rows))
    failures.extend(open_items(root))
    return failures, found


def main(argv):
    args = argv[1:]
    root = ROOT
    if args[:1] == ["--root"] and len(args) == 2:
        root = pathlib.Path(args[1]).resolve()
    elif args:
        print(
            "usage: check-contracts.py [--root DIR]\n"
            "There is no --known: Reliquary forgives no number (docs/design/constants.md).",
            file=sys.stderr,
        )
        return 2
    failures, found = contracts(root)
    for failure in failures:
        print(failure, file=sys.stderr)
    if failures:
        print(f"check-contracts: {len(failures)} failures, {len(found)} production numbers read", file=sys.stderr)
        return 1
    print(f"check-contracts: {len(found)} production numbers, each with a row of a kind the rule accepts")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
