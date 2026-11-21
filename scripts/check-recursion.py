#!/usr/bin/env python3
"""Call-graph cycles in the built code (CLAUDE.md §11; plan RQ0.3, AC-RQ0.3.3, T-RQ0.3.1, T-RQ0.3.2):
no function of Reliquary's calls itself, directly or through others, through a trait object or a
function pointer included.

This is Mantle's scripts/check-recursion.py (Mantle 89d365cc9bf3, on its perf-10k line): its reader
of rustc's v0 symbols, its IR build, its strongly connected components and its report, unchanged
but where this says otherwise. What Reliquary adds: it follows calls through trait objects and
function pointers, which Mantle's check counts and leaves (docs/design/gates.md, G2); it checks
any workspace (`--workspace`), so its tests plant cycles in fixtures (crates/reliquary/tests/gates.rs);
cargo runs a small compiled program as the wrapper (scripts/recursion-wrapper.rs), so the check runs
on Windows too; and there is no list of known cycles: Reliquary has none to forgive.

The code read is what the pinned toolchain builds: every workspace crate's libraries and binaries,
without cfg(test), tests/, benches/ or examples/, with every feature, as LLVM IR after rustc's own
-O0 pipeline. That is a dev build, which inlines nothing but `#[inline(always)]`: such a function's
calls become its caller's, so a cycle through one stays a cycle, named by the functions it was
inlined into. Build scripts and proc-macros run on the building machine, not in the registry, and
are not read. The build has its own target directory, check-recursion under the target directory
(CARGO_TARGET_DIR's when set), and cargo runs the
wrapper as its RUSTC_WORKSPACE_WRAPPER, which adds `--emit=llvm-ir -C codegen-units=1` to the
workspace's compiles alone: one .ll a crate, found by the stem of the outputs cargo reports,
dependencies built as usual. A changed source rebuilds its crate and the crates above it, as any
build does; the wrapper's flags are not part of cargo's fingerprint, so a unit cargo finds fresh
whose IR is missing is cleaned and built again.

Each `define` is a function and each direct `call` or `invoke` an edge. Symbols are rustc's v0
mangling, the pinned toolchain's default, read here without crate disambiguators. A function is
Reliquary's when the item it instantiates is defined in a workspace crate (a method, a closure, a
trait impl, a derive), and a drop glue shim is Reliquary's when the type it drops names a workspace
type: a recursive type's glue is that type's recursion. Strongly connected components are found by
Tarjan's algorithm on an explicit stack; a component with a function of Reliquary's is a cycle of
ours, reported with every function in it and every call through a pointer that joins it. A generic
function of std, core, alloc or a dependency is instantiated in the crate that uses it, so a closure
passed to an iterator adapter that calls back into its own function is a cycle through the
adapter's instance, and it is ours: the report names the adapter's functions beside ours. A
component of std's or a dependency's generic code alone is not ours and is only counted.

Calls through pointers, as the -O0 IR has them:
- A call through a trait object loads its callee from a vtable slot (a load rustc marks
  `!invariant.load`, at a byte offset from the vtable pointer) and calls it. Rust's vtables are the
  module's `@vtable.N` constants: each entry's offset and function are read from the constant. The
  call may reach the function at its offset in any vtable the workspace's code makes, narrowed by
  what the call site says: the method named in the source at the call's debug location (a method
  call's location is its method name), else a closure's call (`f(x)` reaches the Fn traits'
  entries), else the entries whose LLVM signature (return type and argument count) is the call's.
  Inside the drop glue of `dyn Trait`, slot 0 (drop_in_place) reaches the drop glue of the vtables
  of `Trait` alone. A poll through `dyn Future` reaches the futures whose vtable is made in the
  poller or in what it calls directly: a future polled by the function that made it, or by its
  caller (`helper().await`), and so an async function that boxes and awaits itself, which is
  async recursion.
- A call through a function pointer may reach any function of the workspace's IR whose address
  Reliquary's own code takes (outside vtables, and outside std's and dependencies' instances, such
  as the formatting machinery's) with the call's LLVM signature.
Each is an over-approximation of what can run, so a cycle found through one is a cycle the code
allows, and the report says which calls joined it.

What this does not see:
- Calls made by std's or a dependency's compiled code into Reliquary's through pointers: the
  formatting machinery calling a `Display` impl, a thread's start, an executor polling a spawned
  task. The vtables those calls use are made in Reliquary's code and read here, but the call sites
  are not.
- Code this build does not instantiate: a generic function with no instance in production code,
  and code under a `cfg` of another platform (the IR is the host's; kiln CI runs the gate on each
  platform, .kiln/ci.yaml).
"""
import filecmp
import json
import os
import pathlib
import re
import subprocess
import sys
import tempfile
from collections import deque

ROOT = pathlib.Path(__file__).resolve().parent.parent
WRAPPER_SOURCE = pathlib.Path(__file__).resolve().parent / "recursion-wrapper.rs"
# The IR build: the dev profile (no optimization, so the IR's calls are the source's) with full debug
# information for the workspace's crates, which gives each function's file and line, each call's,
# and each vtable's type and trait (`<T as Trait>::{vtable}`, which line tables alone leave out),
# and none for dependencies, which are not read.
CARGO_BUILD = (
    "cargo", "build", "--workspace", "--lib", "--bins", "--all-features", "--locked",
    "--message-format=json-render-diagnostics",
    "--config", 'profile.dev.debug="full"',
    "--config", 'profile.dev.package."*".debug=false',
)
# The unit kinds whose code runs in the registry: libraries and binaries (cargo's target kinds).
CODE_KINDS = {"lib", "rlib", "dylib", "cdylib", "staticlib", "bin"}
# Steps one symbol's reading may take: a v0 symbol's back-references point only backward, but
# nested ones can repeat work; rustc's symbols take a few hundred.
DEMANGLE_STEPS = 100_000

B62 = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"
BASIC = {
    "a": "i8", "b": "bool", "c": "char", "d": "f64", "e": "str", "f": "f32", "h": "u8",
    "i": "isize", "j": "usize", "l": "i32", "m": "u32", "n": "i128", "o": "u128", "s": "i16",
    "t": "u16", "u": "()", "v": "...", "x": "i64", "y": "u64", "z": "!", "p": "_",
}
INTEGER = set("ahijlmnostxy")
# The compiler's drop glue for a type, as the pinned toolchain names it.
DROP_GLUE = "core::ptr::drop_glue::<"

# A line of a function's body that calls: `call`/`invoke`, maybe as a value and maybe a tail call;
# its callee is the first `@name(` or `%name(` after the keyword (a named return type is followed
# by a space, an argument by `,` or `)`).
CALL = re.compile(r'^\s+(?:%(?:"[^"]*"|[-\w$.]+) = )?(?:(?:tail|musttail|notail) )?(?:call|invoke)\b')
CALLEE = re.compile(r'([@%])("(?:[^"\\]|\\.)*"|[-\w$.]+)\(')
DEFINE = re.compile(r'^define\b[^@]*@("(?:[^"\\]|\\.)*"|[-\w$.]+)\(')
DBG = re.compile(r"!dbg !(\d+)")
SUBPROGRAM = re.compile(r"^!(\d+) = (?:distinct )?!DISubprogram\(")
SP_FILE = re.compile(r"\bfile: !(\d+)")
SP_LINE = re.compile(r"\bline: (\d+)")
DIFILE = re.compile(r'^!(\d+) = !DIFile\(filename: "((?:[^"\\]|\\.)*)", directory: "((?:[^"\\]|\\.)*)"')


def base62(s, pos):
    """A <base-62-number> at `pos`: its value and the position after it."""
    if s[pos] == "_":
        return 0, pos + 1
    value = 0
    while s[pos] != "_":
        value = value * 62 + B62.index(s[pos])
        pos += 1
    return value + 1, pos + 1


def identifier(s, pos):
    """An <undisambiguated-identifier> at `pos`: its text and the position after it."""
    punycode = s[pos] == "u"
    if punycode:
        pos += 1
    end = pos + 1
    while s[pos] != "0" and s[end].isdigit():  # "0", or a number with no leading zero
        end += 1
    length = int(s[pos:end])
    pos = end + 1 if s[end] == "_" else end
    text = s[pos:pos + length]
    if len(text) != length:
        raise ValueError("short identifier")
    if punycode:
        basic, _, encoded = text.rpartition("_")
        text = (basic + "-" + encoded).encode("ascii").decode("punycode")
    return text, pos + length


def disambiguator(s, pos):
    """An optional `s<base-62-number>` at `pos`: its value (0 when absent) and the position after."""
    if s[pos] != "s":
        return 0, pos
    value, pos = base62(s, pos + 1)
    return value + 1, pos


def demangle(symbol):
    """A v0 symbol read: (its path as text, the crate defining the item it instantiates, every crate
    its path names), or None for a symbol of another form or one this reader cannot parse.

    The grammar (rustc's "v0 Symbol Format", RFC 2603) is read by a stack of tasks, each a
    production still to read, so nesting deepens the stack and never the call stack. A task's
    `show` is false inside an impl's own path, which says where the impl is and is not printed."""
    if not symbol.startswith("_R"):
        return None
    s = symbol[2:]
    if not s or s[0].isdigit():  # an encoding version: none is defined
        return None
    out, crates, defining = [], set(), [None]
    pos = 0
    tasks = [("path", True, True, True)]
    steps = 0
    try:
        while tasks:
            steps += 1
            if steps > DEMANGLE_STEPS:
                return None
            task = tasks.pop()
            kind, show = task[0], task[1]
            if kind == "text":
                if show:
                    out.append(task[2])
            elif kind == "seek":
                pos = task[2]
            elif kind == "path":
                in_value, defines = task[2], task[3]
                tag = s[pos]
                pos += 1
                if tag == "C":
                    _, pos = disambiguator(s, pos)
                    name, pos = identifier(s, pos)
                    crates.add(name)
                    if defines and defining[0] is None:
                        defining[0] = name
                    if show:
                        out.append(name)
                elif tag == "N":
                    namespace = s[pos]
                    pos += 1
                    tasks.append(("name", show, namespace))
                    tasks.append(("path", show, in_value, defines))
                elif tag == "I":
                    tasks.append(("args", show, in_value, 0))
                    tasks.append(("path", show, in_value, defines))
                elif tag in "MX":
                    _, pos = disambiguator(s, pos)
                    tasks.append(("text", show, ">"))
                    if tag == "X":
                        tasks.append(("path", show, False, False))
                        tasks.append(("text", show, " as "))
                    tasks.append(("type", show))
                    tasks.append(("text", show, "<"))
                    tasks.append(("path", False, False, defines))
                elif tag == "Y":
                    tasks.append(("text", show, ">"))
                    tasks.append(("path", show, False, defines))
                    tasks.append(("text", show, " as "))
                    tasks.append(("type", show))
                    tasks.append(("text", show, "<"))
                elif tag == "B":
                    target, pos = base62(s, pos)
                    if target >= pos - 1:
                        return None
                    tasks.append(("seek", show, pos))
                    tasks.append(("path", show, in_value, defines))
                    pos = target
                else:
                    return None
            elif kind == "name":
                number, pos = disambiguator(s, pos)
                name, pos = identifier(s, pos)
                if show:
                    namespace = task[2]
                    if namespace.islower():
                        out.append("::" + name if name else "")
                    else:
                        label = {"C": "closure", "S": "shim"}.get(namespace, namespace)
                        out.append("::{" + label + (":" + name if name else "") + "#" + str(number) + "}")
            elif kind == "args":
                in_value, count = task[2], task[3]
                if count == 0 and show:
                    out.append("::<" if in_value else "<")
                if s[pos] == "E":
                    pos += 1
                    if show:
                        out.append(">")
                    continue
                if count and show:
                    out.append(", ")
                tasks.append(("args", show, in_value, count + 1))
                if s[pos] == "L":
                    _, pos = base62(s, pos + 1)
                    if show:
                        out.append("'_")
                elif s[pos] == "K":
                    pos += 1
                    tasks.append(("const", show))
                else:
                    tasks.append(("type", show))
            elif kind == "type":
                tag = s[pos]
                if tag in BASIC:
                    pos += 1
                    if show:
                        out.append(BASIC[tag])
                elif tag in "RQ":
                    pos += 1
                    if s[pos] == "L":
                        _, pos = base62(s, pos + 1)
                    if show:
                        out.append("&" if tag == "R" else "&mut ")
                    tasks.append(("type", show))
                elif tag in "PO":
                    pos += 1
                    if show:
                        out.append("*const " if tag == "P" else "*mut ")
                    tasks.append(("type", show))
                elif tag in "AS":
                    pos += 1
                    if show:
                        out.append("[")
                    tasks.append(("text", show, "]"))
                    if tag == "A":
                        tasks.append(("const", show))
                        tasks.append(("text", show, "; "))
                    tasks.append(("type", show))
                elif tag == "T":
                    pos += 1
                    if show:
                        out.append("(")
                    tasks.append(("tuple", show, 0))
                elif tag == "F":
                    pos += 1
                    if s[pos] == "G":
                        _, pos = base62(s, pos + 1)
                    prefix = ""
                    if s[pos] == "U":
                        pos += 1
                        prefix = "unsafe "
                    if s[pos] == "K":
                        pos += 1
                        if s[pos] == "C":
                            pos += 1
                            abi = "C"
                        else:
                            abi, pos = identifier(s, pos)
                        prefix += 'extern "' + abi.replace("_", "-") + '" '
                    if show:
                        out.append(prefix + "fn(")
                    tasks.append(("fn", show, 0))
                elif tag == "D":
                    pos += 1
                    if s[pos] == "G":
                        _, pos = base62(s, pos + 1)
                    if show:
                        out.append("dyn ")
                    tasks.append(("dyn", show, 0))
                elif tag == "B":
                    target, pos = base62(s, pos + 1)
                    if target >= pos - 1:
                        return None
                    tasks.append(("seek", show, pos))
                    tasks.append(("type", show))
                    pos = target
                else:
                    tasks.append(("path", show, False, False))
            elif kind == "tuple":
                count = task[2]
                if s[pos] == "E":
                    pos += 1
                    if show:
                        out.append(",)" if count == 1 else ")")
                    continue
                if count and show:
                    out.append(", ")
                tasks.append(("tuple", show, count + 1))
                tasks.append(("type", show))
            elif kind == "fn":
                count = task[2]
                if s[pos] == "E":
                    pos += 1
                    if show:
                        out.append(")")
                    if s[pos] == "u":
                        pos += 1
                    else:
                        if show:
                            out.append(" -> ")
                        tasks.append(("type", show))
                    continue
                if count and show:
                    out.append(", ")
                tasks.append(("fn", show, count + 1))
                tasks.append(("type", show))
            elif kind == "dyn":
                count = task[2]
                if s[pos] == "E":
                    if s[pos + 1] != "L":
                        return None
                    _, pos = base62(s, pos + 2)
                    continue
                if count and show:
                    out.append(" + ")
                tasks.append(("dyn", show, count + 1))
                tasks.append(("bindings", show, 0))
                tasks.append(("path", show, False, False))
            elif kind == "bindings":
                count = task[2]
                if s[pos] != "p":
                    if count and show:
                        out.append(">")
                    continue
                name, pos = identifier(s, pos + 1)
                if show:
                    # Inside the trait's own generic arguments, if it has them, as rustc-demangle does.
                    opened = count == 0 and out[-1] == ">"
                    if opened:
                        out.pop()
                    out.append((", " if count or opened else "<") + name + " = ")
                tasks.append(("bindings", show, count + 1))
                tasks.append(("type", show))
            elif kind == "const":
                tag = s[pos]
                pos += 1
                if tag == "B":
                    target, pos = base62(s, pos)
                    if target >= pos - 1:
                        return None
                    tasks.append(("seek", show, pos))
                    tasks.append(("const", show))
                    pos = target
                elif tag == "p":
                    if show:
                        out.append("_")
                elif tag in INTEGER or tag in "bc":
                    negative = s[pos] == "n"
                    if negative:
                        pos += 1
                    end = s.index("_", pos)
                    value = int(s[pos:end], 16) if end > pos else 0
                    pos = end + 1
                    if tag == "b":
                        text = "true" if value else "false"
                    elif tag == "c":
                        text = repr(chr(value))
                    else:
                        text = ("-" if negative else "") + str(value)
                    if show:
                        out.append(text)
                else:
                    return None
            else:
                return None
    except (IndexError, ValueError, UnicodeError):
        return None
    return "".join(out), defining[0], crates



# What Reliquary's reader adds to Mantle's: a byte offset from a pointer (`getelementptr i8`), a
# load of a pointer (a vtable slot's when rustc marks it `!invariant.load`), a constant
# expression's offset into a global, a call's debug location, a vtable's constant, the scopes a
# debug location names, and the target's pointer size.
GEP = re.compile(r'^\s+(%(?:"[^"]*"|[-\w$.]+)) = getelementptr\b[^,]*?\bi8, ptr ((?:%|@)(?:"[^"]*"|[-\w$.]+)), i64 (-?\d+)')
LOAD_PTR = re.compile(r'^\s+(%(?:"[^"]*"|[-\w$.]+)) = load ptr, ptr (.+?), align ')
CONST_GEP = re.compile(r'^getelementptr\b[^(]*\(i8, ptr (@(?:"[^"]*"|[-\w$.]+)), i64 (-?\d+)\)')
NAME = re.compile(r'^(?:%|@)(?:"[^"]*"|[-\w$.]+)$')
GLOBAL_REF = re.compile(r'@("(?:[^"\\]|\\.)*"|[-\w$.]+)')
DBG_CALL = re.compile(r"!dbg !(\d+)")
# A module's private vtables: rustc names them `@vtable.0` to `@vtable.9`, then `@vtable.a` and on.
VTABLE = re.compile(r'^@(vtable\.[0-9A-Za-z]+) = [^<]*<\{[^}]*\}> <\{(.*)\}>, align \d+(?:, !dbg !(\d+))?')
GLOBAL_EXPRESSION = re.compile(r"^!(\d+) = !DIGlobalVariableExpression\(var: !(\d+)")
GLOBAL_VARIABLE = re.compile(r'^!(\d+) = (?:distinct )?!DIGlobalVariable\(name: "((?:[^"\\]|\\.)*)"')
LOCATION = re.compile(r"^!(\d+) = !DILocation\(line: (\d+)(?:, column: (\d+))?, scope: !(\d+)")
SCOPE = re.compile(r"^!(\d+) = (?:distinct )?!(?:DISubprogram|DILexicalBlock|DILexicalBlockFile)\(")
SCOPE_FILE = re.compile(r"\bfile: !(\d+)")
POINTER_BITS = re.compile(r'^target datalayout = "(?:[^"]*-)?p:(\d+)')
ARRAY = re.compile(r"^\[(\d+) x i8\]")
INTEGER_TYPE = re.compile(r"^i(\d+)\b")
# What a type's text carries besides the type: parameter and return attributes, linkage, calling
# conventions and fast-math flags, as LLVM's assembly writes them.
ATTRIBUTES = re.compile(
    r"\b(?:range|dereferenceable|dereferenceable_or_null|nofpclass|align|sret|byval|elementtype|captures|initializes)\s*\([^()]*(?:\([^()]*\))?[^()]*\)"
    r"|\balign \d+"
    r"|\b(?:internal|private|linkonce_odr|linkonce|weak_odr|weak|external|available_externally|dso_local|dso_preemptable|hidden|protected|default|unnamed_addr|local_unnamed_addr|noundef|zeroext|signext|inreg|noalias|nonnull|nocapture|noinline|readonly|writeonly|returned|swiftself|swifterror|immarg|noext|fastcc|coldcc|tailcc|ccc|cc \d+|fast|nnan|ninf|nsz|arcp|contract|afn|reassoc|tail|musttail|notail|win64cc|x86_64_sysvcc|aarch64_vector_pcs|preserve_mostcc)\b"
)
# The traits whose vtables a call written `f(x)` reaches: the closures' (core::ops::function).
FN_TRAITS = ("core::ops::function::Fn", "core::ops::function::FnMut", "core::ops::function::FnOnce")
FUTURE = "core::future::future::Future"
# The method at each slot of std's traits whose vtable entries are closure or coroutine bodies, which
# no symbol names: (trait, slot) → method (std's trait definitions: `poll` is Future's one method, and
# each Fn trait adds its own call to its supertrait's).
STD_SLOTS = {
    (FUTURE, 3): "poll",
    ("core::ops::function::FnOnce", 3): "call_once",
    ("core::ops::function::FnMut", 3): "call_once",
    ("core::ops::function::FnMut", 4): "call_mut",
    ("core::ops::function::Fn", 3): "call_once",
    ("core::ops::function::Fn", 4): "call_mut",
    ("core::ops::function::Fn", 5): "call",
}
IDENT = re.compile(r"(?:r#)?[A-Za-z_][A-Za-z0-9_]*")
IR_ESCAPE = re.compile(r"\\([0-9A-Fa-f]{2})")


def ir_text(text):
    """A string of the IR's metadata as it reads: LLVM writes a backslash, a quote and every byte it
    does not print as `\\XX` (a Windows path's separators are `\\5C`), the bytes UTF-8's."""
    out, at = bytearray(), 0
    for m in IR_ESCAPE.finditer(text):
        out += text[at:m.start()].encode("utf-8")
        out.append(int(m.group(1), 16))
        at = m.end()
    out += text[at:].encode("utf-8")
    return out.decode("utf-8", "replace")


def wrapper(target, root):
    """The wrapper program, compiled from scripts/recursion-wrapper.rs beside the target directory
    (inside it, it would make the directory before cargo does, and cargo then neither tags it as
    its own nor cleans it) when it is missing or older than its source: its path, or a failure."""
    home = target.with_name(target.name + "-wrapper")
    exe = home / ("recursion-wrapper.exe" if os.name == "nt" else "recursion-wrapper")
    if exe.exists() and exe.stat().st_mtime >= WRAPPER_SOURCE.stat().st_mtime:
        return exe, None
    home.mkdir(parents=True, exist_ok=True)
    # Compiled in the workspace, by the toolchain its rust-toolchain.toml pins.
    built = subprocess.run(
        ["rustc", "--edition", "2024", "-C", "opt-level=1", str(WRAPPER_SOURCE), "-o", str(exe)],
        cwd=root, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    if built.returncode != 0:
        return None, f"check-recursion: the wrapper didn't compile:\n{built.stdout}"
    return exe, None


def members(root):
    """The workspace's packages' manifests, as cargo metadata names them."""
    run = subprocess.run(
        ["cargo", "metadata", "--no-deps", "--format-version", "1", "--locked"],
        cwd=root, stdout=subprocess.PIPE, text=True)
    if run.returncode != 0:
        return None
    return {str(pathlib.Path(p["manifest_path"]).resolve()) for p in json.loads(run.stdout)["packages"]}


def cargo_build(root, target, exe, manifests):
    """Builds the IR; returns cargo's exit code and the workspace's code units: (crate name, kind,
    package name, the IR files)."""
    env = dict(os.environ)
    env.pop("RUSTC_WRAPPER", None)
    env.update({
        "CARGO_TARGET_DIR": str(target),
        "CARGO_INCREMENTAL": "0",
        "RUSTC_WORKSPACE_WRAPPER": str(exe),
    })
    run = subprocess.run(CARGO_BUILD, cwd=root, env=env, stdout=subprocess.PIPE, text=True)
    deps = target / "debug" / "deps"
    units = []
    for line in run.stdout.splitlines():
        try:
            message = json.loads(line)
        except ValueError:
            continue
        if message.get("reason") != "compiler-artifact":
            continue
        if str(pathlib.Path(message["manifest_path"]).resolve()) not in manifests:
            continue
        kind = message["target"]["kind"][0]
        if kind not in CODE_KINDS:
            continue
        name = message["target"]["name"].replace("-", "_")
        # `path+file:///../crates/x#reliquary-x@0.1.0`, or `../crates/x#0.1.0` when the package is
        # named as its directory.
        url, _, fragment = message["package_id"].rpartition("#")
        package = fragment.partition("@")[0] if "@" in fragment else url.rstrip("/").rpartition("/")[2]
        stems = set()
        for filename in message["filenames"]:
            path = pathlib.Path(filename)
            if path.parent == deps and path.suffix == ".rmeta" and path.name.startswith("lib"):
                stems.add(path.stem[3:])
        executable = message.get("executable")
        if executable:
            # The binary in deps/ whose bytes the uplifted executable has: cargo hard-links it on
            # Linux and copies it on macOS and Windows. On MSVC targets its names in deps/ carry no
            # hash (`reliquary.exe`, `reliquary.ll`), the debug database's name being in the binary.
            for candidate in [*deps.glob(name + "-*"), deps / name, deps / (name + ".exe")]:
                if (candidate.is_file() and candidate.suffix in ("", ".exe")
                        and filecmp.cmp(candidate, executable, shallow=False)):
                    stems.add(candidate.stem)
        files = sorted(p for stem in stems for p in deps.glob(stem + "*.ll") if p.name.split(".", 1)[0] == stem)
        units.append((name, kind, package, files))
    return run.returncode, units


def build(root, target):
    """The IR files of every workspace unit and the workspace's crate names, or the failures. A unit
    cargo found fresh but whose IR is missing (a target directory built otherwise) is cleaned and
    built once more."""
    exe, failure = wrapper(target, root)
    if failure:
        return None, None, [failure]
    manifests = members(root)
    if manifests is None:
        return None, None, [f"check-recursion: cargo metadata failed in {root}"]
    code, units = cargo_build(root, target, exe, manifests)
    if code != 0:
        return None, None, [f"check-recursion: the IR build failed (cargo exit {code})"]
    missing = sorted({package for _, _, package, files in units if len(files) != 1})
    if missing:
        clean = ["cargo", "clean", "--target-dir", str(target)] + [a for p in missing for a in ("-p", p)]
        subprocess.run(clean, cwd=root, check=False)
        code, units = cargo_build(root, target, exe, manifests)
        if code != 0:
            return None, None, [f"check-recursion: the IR build failed (cargo exit {code})"]
    failures = [f"check-recursion: {name} ({kind}): {len(files)} IR files, not one" for name, kind, _, files in units if len(files) != 1]
    if not units:
        failures.append("check-recursion: the build reported no library or binary of the workspace")
    return [files[0] for _, _, _, files in units if len(files) == 1], {name for name, _, _, _ in units}, failures


def closing(text, start):
    """The index just past the bracket that closes the one at `start`, over (), {}, [] and <>, with
    quoted strings skipped; or len(text) when it never closes."""
    pairs = {"(": ")", "{": "}", "[": "]", "<": ">"}
    stack = []
    i = start
    while i < len(text):
        c = text[i]
        if c == '"':
            end = text.find('"', i + 1)
            i = len(text) if end < 0 else end + 1
            continue
        if c in pairs:
            stack.append(pairs[c])
        elif stack and c == stack[-1]:
            stack.pop()
            if not stack:
                return i + 1
        i += 1
    return len(text)


def arguments(text, start):
    """The number of top-level arguments in the parenthesized list opening at `start`."""
    end = closing(text, start)
    inner = text[start + 1:end - 1].strip()
    if not inner:
        return 0
    count, depth, i = 1, 0, 0
    while i < len(inner):
        c = inner[i]
        if c == '"':
            close = inner.find('"', i + 1)
            i = len(inner) if close < 0 else close + 1
            continue
        if c in "({[<":
            depth += 1
        elif c in ")}]>":
            depth -= 1
        elif c == "," and depth == 0:
            count += 1
        i += 1
    return count


def plain_type(text):
    """A return type's text without its attributes: what LLVM calls it (`i1`, `{ ptr, i64 }`)."""
    return " ".join(ATTRIBUTES.sub(" ", text).split())


def value(text):
    """A load's address operand: (base name, byte offset), the base a `%local` or an `@global`; or
    None when it is neither a name nor a constant offset from one."""
    text = text.strip()
    if NAME.match(text):
        return text, 0
    m = CONST_GEP.match(text)
    if m:
        return m.group(1), int(m.group(2))
    return None


def vtable_entries(initializer, pointer):
    """A vtable constant's entries: [(byte offset, symbol)] for each function pointer in it, from the
    elements of its packed struct (`[24 x i8] c"…"`, `ptr @f`, `ptr null`, `i64 8`)."""
    entries, offset, i = [], 0, 0
    text = initializer.strip()
    while i < len(text):
        while i < len(text) and text[i] in " ,":
            i += 1
        if i >= len(text):
            break
        rest = text[i:]
        if rest.startswith("ptr "):
            j = i + 4
            m = GLOBAL_REF.match(text, j)
            if m:
                entries.append((offset, m.group(1).strip('"')))
                j = m.end()
            else:
                while j < len(text) and text[j] not in ",(":
                    j += 1
                if j < len(text) and text[j] == "(":
                    j = closing(text, j)
            offset += pointer
            i = j
            continue
        m = ARRAY.match(rest) or INTEGER_TYPE.match(rest)
        if m is None:
            return None
        size = int(m.group(1)) if m.re is ARRAY else int(m.group(1)) // 8
        j = i + m.end()
        while j < len(text) and text[j] == " ":
            j += 1
        if text.startswith('c"', j):
            close = text.find('"', j + 2)
            j = len(text) if close < 0 else close + 1
        else:
            while j < len(text) and text[j] != ",":
                j += 1
        offset += size
        i = j
    return entries


class Function:
    """A defined function: its direct callees, its file and line, its calls through pointers, the
    functions and vtables it names, its signature and the module it is in."""
    __slots__ = ("callees", "site", "slots", "pointers", "taken", "vtables", "signature", "module")

    def __init__(self, module, signature):
        self.callees = set()
        self.site = None
        self.slots = []  # calls through a vtable slot: (offset, vtable or None, dbg, signature)
        self.pointers = []  # calls through a function pointer: signatures
        self.taken = set()
        self.vtables = set()
        self.signature = signature
        self.module = module


def read_ir(paths):
    """What the IR files define: functions by symbol, vtables by (module, name) with their entries,
    the functions the modules' other constants name, each dyn call's debug location by (module, id),
    the pointer size in bytes, and each vtable's name in the debug information
    (`<T as Trait>::{vtable}`) by (module, name)."""
    functions, vtables, constants, named = {}, {}, set(), {}
    locations, pointer = {}, 8
    for module, path in enumerate(paths):
        sites, files, symbol_dbg, scope_files = {}, {}, [], {}
        vtable_dbg, expressions, variables = {}, {}, {}
        wanted = set()
        current = None
        names = {}  # within the current function: %local → ("gep", base, offset) or ("slot", base, offset)
        with open(path, encoding="utf-8", errors="replace") as ir:
            for line in ir:
                if current is not None:
                    if line.startswith("}"):
                        current, names = None, {}
                        continue
                    if "@" in line:
                        call = CALL.match(line)
                        callee = call and CALLEE.search(line, call.end())
                        skip = callee.start(2) if callee and callee.group(1) == "@" else -1
                        for ref in GLOBAL_REF.finditer(line):
                            if ref.start(1) == skip:
                                continue
                            name = ref.group(1).strip('"')
                            if name.startswith("vtable."):
                                current.vtables.add((module, name))
                            elif not name.startswith("llvm."):
                                current.taken.add(name)
                    if "getelementptr" in line:
                        m = GEP.match(line)
                        if m:
                            names[m.group(1)] = ("gep", m.group(2), int(m.group(3)))
                            continue
                    if "load ptr" in line and "!invariant.load" in line:
                        m = LOAD_PTR.match(line)
                        if m:
                            operand = value(m.group(2))
                            if operand:
                                base, offset = operand
                                if base in names and names[base][0] == "gep":
                                    _, base, more = names[base]
                                    offset += more
                                names[m.group(1)] = ("slot", base, offset)
                        continue
                    if "call" not in line and "invoke" not in line:
                        continue
                    call = CALL.match(line)
                    callee = call and CALLEE.search(line, call.end())
                    if not callee:
                        continue
                    if callee.group(1) == "@":
                        if not callee.group(2).startswith("llvm."):
                            current.callees.add(callee.group(2).strip('"'))
                        continue
                    signature = (plain_type(line[call.end():callee.start()]), arguments(line, callee.end() - 1))
                    local = "%" + callee.group(2)
                    slot = names.get(local)
                    if slot and slot[0] == "slot":
                        dbg = DBG_CALL.search(line, callee.end())
                        base = slot[1]
                        vtable = (module, base[1:].strip('"')) if base.startswith("@vtable.") else None
                        current.slots.append((slot[2], vtable, (module, dbg.group(1)) if dbg else None, signature))
                        if dbg:
                            wanted.add(dbg.group(1))
                    else:
                        current.pointers.append(signature)
                elif line.startswith("define"):
                    m = DEFINE.match(line)
                    if m is None:
                        continue
                    symbol = m.group(1).strip('"')
                    head = line[len("define"):m.start(1) - 1]
                    signature = (plain_type(head), arguments(line, m.end() - 1))
                    current = functions.setdefault(symbol, Function(module, signature))
                    dbg = DBG.search(line)
                    if dbg:
                        symbol_dbg.append((symbol, dbg.group(1)))
                elif line.startswith("@"):
                    m = VTABLE.match(line)
                    if m:
                        entries = vtable_entries(m.group(2), pointer)
                        vtables[(module, m.group(1))] = entries if entries is not None else []
                        if m.group(3):
                            vtable_dbg[m.group(1)] = m.group(3)
                        continue
                    if "=" in line:
                        for ref in GLOBAL_REF.finditer(line, line.index("=")):
                            constants.add(ref.group(1).strip('"'))
                elif line.startswith("!"):
                    if "DISubprogram(" in line:
                        m = SUBPROGRAM.match(line)
                        f, n = SP_FILE.search(line), SP_LINE.search(line)
                        if m and f and n:
                            sites[m.group(1)] = (f.group(1), int(n.group(1)))
                    if "DIGlobalVariable" in line:
                        m = GLOBAL_EXPRESSION.match(line)
                        if m:
                            expressions[m.group(1)] = m.group(2)
                            continue
                        m = GLOBAL_VARIABLE.match(line)
                        if m and m.group(2).endswith("::{vtable}"):
                            variables[m.group(1)] = m.group(2)
                        continue
                    if "DILocation(" in line:
                        m = LOCATION.match(line)
                        if m and m.group(1) in wanted:
                            locations[(module, m.group(1))] = (int(m.group(2)), int(m.group(3) or 0), m.group(4))
                    elif SCOPE.match(line):
                        f = SCOPE_FILE.search(line)
                        if f:
                            scope_files[SCOPE.match(line).group(1)] = f.group(1)
                    elif "DIFile(" in line:
                        m = DIFILE.match(line)
                        if m:
                            files[m.group(1)] = (ir_text(m.group(2)), ir_text(m.group(3)))
                elif line.startswith("target datalayout"):
                    m = POINTER_BITS.match(line)
                    pointer = int(m.group(1)) // 8 if m else 8
        for symbol, dbg in symbol_dbg:
            site = sites.get(dbg)
            if site and site[0] in files and functions[symbol].site is None:
                functions[symbol].site = files[site[0]] + (site[1],)
        for name, dbg in vtable_dbg.items():
            text = variables.get(expressions.get(dbg))
            if text:
                named[(module, name)] = text
        for key in [k for k in locations if k[0] == module]:
            line_number, column, scope = locations[key]
            file_id = scope_files.get(scope)
            locations[key] = (files.get(file_id), line_number, column)
    return functions, vtables, constants, locations, pointer, named


def shown(site, root):
    """A function's file:line, relative to the workspace when it is in it."""
    if site is None:
        return "(no line)"
    filename, directory, line = site
    full = pathlib.Path(directory, filename)
    try:
        filename = full.resolve().relative_to(root).as_posix()
    except ValueError:
        if not filename.startswith("library/"):
            filename = full.as_posix()
    return f"{filename}:{line}"


def impl_parts(text):
    """A demangled `<Type as Trait<..>>::method::…` read as (trait path without its generic
    arguments, method name); (None, None) for any other path."""
    if not text.startswith("<"):
        return None, None
    depth, i, split = 0, 0, -1
    while i < len(text):
        if text.startswith("->", i):
            i += 2
            continue
        c = text[i]
        if c == "<":
            depth += 1
        elif c == ">":
            depth -= 1
            if depth == 0:
                break
        elif depth == 1 and text.startswith(" as ", i) and split < 0:
            split = i
        i += 1
    if split < 0 or i >= len(text):
        return None, None
    trait = text[split + 4:i]
    cut = trait.find("<")
    trait = trait if cut < 0 else trait[:cut]
    rest = text[i + 1:]
    if not rest.startswith("::"):
        return trait, None
    method = rest[2:].split("::", 1)[0]
    return trait, method


def vtable_trait(name):
    """The trait a vtable is for, read from its debug information's name, without the trait's generic
    arguments: `<T as a::Trait>::{vtable}`, or on MSVC targets, whose names are C++'s,
    `impl$<T, a::Trait>::vtable$`; None for any other name."""
    if name.startswith("impl$<") and name.endswith(">::vtable$"):
        inner = name[len("impl$<"):-len(">::vtable$")]
        depth, split = 0, -1
        for i, c in enumerate(inner):
            if c == "<":
                depth += 1
            elif c == ">":
                depth -= 1
            elif c == "," and depth == 0:
                split = i
        if split < 0:
            return None
        trait = inner[split + 1:].strip()
        cut = trait.find("<")
        return trait if cut < 0 else trait[:cut]
    trait, _ = impl_parts(name)
    return trait


def principals(text):
    """The traits of the trait objects a function's path names (`dyn a::T + Send` names `a::T`), each
    without its generic arguments, leaving out a trait object inside another's generic arguments (the
    `dyn S` of `dyn Future<Output = Box<dyn S>>` is the future's output, not called through here): the
    drop glue of `Box<dyn a::T>` and std's `<Pin<Box<dyn Future>> as Future>::poll` call through vtables
    of those traits alone. A walk over the text, its brackets counted."""
    found, depth, inside, i = set(), 0, [], 0
    while i < len(text):
        if text.startswith("->", i):
            i += 2
            continue
        c = text[i]
        if c == "<":
            depth += 1
        elif c == ">":
            depth -= 1
            while inside and inside[-1] > depth:
                inside.pop()
        elif text.startswith("dyn ", i) and (i == 0 or not (text[i - 1].isalnum() or text[i - 1] == "_")):
            rest = text[i + 4:]
            end = len(rest)
            for stop in ("<", " ", ">", ",", "(", ")"):
                cut = rest.find(stop)
                if 0 <= cut < end:
                    end = cut
            if end and not inside:
                found.add(rest[:end])
            if rest[end:end + 1] == "<":
                # The trait's generic arguments open here: what they name is not called through.
                inside.append(depth + 1)
            i += 4 + end
            continue
        i += 1
    return found


class Source:
    """The workspace's source lines, read once a file, for the method a call site names."""

    def __init__(self):
        self.files = {}

    def line(self, file, number):
        if file is None:
            return None
        filename, directory = file
        path = pathlib.Path(directory, filename)
        key = str(path)
        if key not in self.files:
            try:
                self.files[key] = path.read_text(encoding="utf-8").splitlines()
            except (OSError, UnicodeError):
                self.files[key] = None
        lines = self.files[key]
        if lines is None or not 1 <= number <= len(lines):
            return None
        return lines[number - 1]


def call_syntax(text, column):
    """What the source says at a call's debug location (a 1-based column): ("method", name) for
    `recv.name(…)`, ("path", last segment) for `a::b::name(…)`, ("plain", name) for `name(…)`, or
    None."""
    if text is None or column < 1 or column > len(text):
        return None
    i = column - 1
    segments = []
    while True:
        m = IDENT.match(text, i)
        if not m:
            return None
        segments.append(m.group(0).removeprefix("r#"))
        i = m.end()
        if text.startswith("::<", i):
            i = closing(text, i + 2)
        if text.startswith("::", i) and IDENT.match(text, i + 2):
            i += 2
            continue
        break
    while i < len(text) and text[i] == " ":
        i += 1
    if i >= len(text) or text[i] != "(":
        return None
    before = text[:column - 1].rstrip()
    if len(segments) == 1 and before.endswith("."):
        return "method", segments[0]
    if len(segments) > 1:
        return "path", segments[-1]
    return "plain", segments[0]


class Resolver:
    """Where calls through pointers may go: the functions at a vtable slot, narrowed as the module's
    docstring says, and the functions whose address Reliquary's code takes."""

    def __init__(self, functions, vtables, constants, locations, pointer, owner, named, ours):
        self.functions = functions
        self.ours = ours
        self.vtables = vtables
        self.locations = locations
        self.pointer = pointer
        self.owner = owner
        self.source = Source()
        self.texts = {}
        # Each vtable's traits, whether it is a closure's, and each entry's method name.
        self.traits, self.closures, self.methods = {}, set(), {}
        for key, entries in vtables.items():
            trait = vtable_trait(named.get(key, ""))
            traits = {trait} if trait else set()
            for offset, symbol in entries:
                text = self.text(symbol)
                trait, method = impl_parts(text)
                if text.startswith(DROP_GLUE):
                    method = "drop_in_place"
                if trait:
                    traits.add(trait)
                self.methods[(key, offset)] = method
            self.traits[key] = traits
            if traits & set(FN_TRAITS):
                self.closures.add(key)
            for offset, _ in entries:
                if self.methods.get((key, offset)) is None:
                    for trait in traits:
                        name = STD_SLOTS.get((trait, offset // pointer))
                        if name:
                            self.methods[(key, offset)] = name
        self.slots = {}
        for key, entries in vtables.items():
            for offset, symbol in entries:
                self.slots.setdefault(offset, []).append((key, symbol))
        # The functions whose address Reliquary's own code takes, outside vtables.
        taken = set(constants)
        for symbol, function in functions.items():
            if owner(symbol):
                taken |= function.taken
        self.taken = sorted(s for s in taken if s in functions)
        self.callers = {}
        for symbol, function in functions.items():
            for callee in function.callees:
                if callee in functions:
                    self.callers.setdefault(callee, set()).add(symbol)
        self.made = {}
        self.by_text = {}
        for symbol in functions:
            self.by_text.setdefault(self.text(symbol), symbol)

    def text(self, symbol):
        if symbol not in self.texts:
            read = demangle(symbol)
            self.texts[symbol] = read[0] if read else symbol
        return self.texts[symbol]

    def signature(self, symbol):
        function = self.functions.get(symbol)
        return function.signature if function else None

    def made_from(self, caller):
        """The vtables made by `caller` and by every function it calls, directly or through a function
        pointer, at any depth: a breadth-first walk with a queue, bounded by the number of functions.
        A drop glue's are its type's own poll's: what made the futures the type holds."""
        text = self.text(caller)
        if text.startswith(DROP_GLUE) and text.endswith(">"):
            caller = self.by_text.get(text[len(DROP_GLUE):-1], caller)
        if caller in self.made:
            return self.made[caller]
        seen, queue, made = {caller}, deque([caller]), set()
        budget = len(self.functions)
        while queue and budget > 0:
            budget -= 1
            function = self.functions.get(queue.popleft())
            if function is None:
                continue
            made |= function.vtables
            nexts = set(function.callees)
            for signature in function.pointers:
                nexts.update(self.pointer_call(signature))
            for callee in nexts:
                if callee not in seen and callee in self.functions:
                    seen.add(callee)
                    queue.append(callee)
        self.made[caller] = made
        return made

    def contexts(self, symbol):
        """The functions of Reliquary's a call in `symbol` is made for: `symbol` itself when it is
        Reliquary's, else the nearest of Reliquary's functions above it through std's and
        dependencies' instances (an await of a boxed future runs in std's `Pin<Box<dyn Future>>`
        poll, a boxed value's drop in std's drop glue). A walk up the direct calls with a queue,
        bounded by the number of functions."""
        if self.owner(symbol):
            return {symbol}
        seen, queue, found = {symbol}, deque([symbol]), set()
        budget = len(self.functions)
        while queue and budget > 0:
            budget -= 1
            for caller in self.callers.get(queue.popleft(), ()):
                if caller in seen:
                    continue
                seen.add(caller)
                if self.owner(caller):
                    found.add(caller)
                else:
                    queue.append(caller)
        return found

    def slot(self, caller, offset, vtable, dbg, signature):
        """The calls through vtable slot `offset` in `caller`: [(the function it is made for, the
        functions it may reach)] and the label naming how they were found."""
        if vtable is not None:
            found = [s for o, s in self.vtables.get(vtable, []) if o == offset]
            return [(caller, found)], f"through a trait object (its vtable known, slot {offset // self.pointer})"
        candidates = self.slots.get(offset, [])
        how = []
        # std's and dependencies' instances for a trait object (its drop glue, `Pin<Box<dyn Future>>`'s
        # poll) call through the trait objects their paths name and no others, so the call goes
        # through those traits' vtables alone, even when none has an entry at the slot. A path of
        # Reliquary's own may name a trait object for another reason (a generic argument).
        read = demangle(caller)
        named = principals(self.text(caller)) if read and read[1] not in self.ours else set()
        if named:
            candidates = [(k, s) for k, s in candidates if self.traits.get(k, set()) & named]
            how.append("dyn " + " + ".join(sorted(named)))
        location = self.locations.get(dbg) if dbg else None
        syntax = None
        if location:
            file, number, column = location
            syntax = call_syntax(self.source.line(file, number), column)
        # The source names the method (or, `f(x)`, a closure's call): the call reaches the entries of
        # that name at the slot and no others, none when no vtable the workspace makes has one (its
        # trait's objects are made elsewhere, and so are the functions they reach). Read nothing, and
        # it reaches the entries of its signature, or every entry at the slot.
        if syntax and syntax[0] in ("method", "path"):
            chosen = [(k, s) for k, s in candidates if self.methods.get((k, offset)) == syntax[1]]
            how.append(f"`{syntax[1]}`")
        elif syntax and syntax[0] == "plain":
            chosen = [(k, s) for k, s in candidates if k in self.closures]
            how.append(f"`{syntax[1]}(…)`, a closure")
        elif named:
            chosen = candidates
        else:
            chosen = [(k, s) for k, s in candidates if self.signature(s) == signature]
            how.append("any method of its signature" if chosen else "any method at the slot")
            chosen = chosen or candidates
        label = f"through a trait object (slot {offset // self.pointer}, {', '.join(how)})"
        if chosen and all(FUTURE in self.traits.get(k, set()) for k, _ in chosen):
            out = []
            for context in sorted(self.contexts(caller) or {caller}):
                made = self.made_from(context)
                out.append((context, [s for k, s in chosen if k in made]))
            return out, label + ", one of the futures made by what this calls"
        return [(caller, [s for _, s in chosen])], label

    def pointer_call(self, signature):
        """The functions a call through a function pointer may reach."""
        return [s for s in self.taken if self.signature(s) == signature]


def components(graph):
    """The strongly connected components of `graph` (node: successors) with a cycle: more than one
    node, or one that calls itself. Tarjan's algorithm, its depth-first walk on an explicit stack of
    (node, next successor) frames."""
    index, low, on_stack, stack, found = {}, {}, set(), [], []
    counter = 0
    for root in graph:
        if root in index:
            continue
        index[root] = low[root] = counter
        counter += 1
        stack.append(root)
        on_stack.add(root)
        frames = [(root, iter(graph[root]))]
        while frames:
            node, successors = frames[-1]
            advanced = False
            for successor in successors:
                if successor not in index:
                    index[successor] = low[successor] = counter
                    counter += 1
                    stack.append(successor)
                    on_stack.add(successor)
                    frames.append((successor, iter(graph[successor])))
                    advanced = True
                    break
                if successor in on_stack:
                    low[node] = min(low[node], index[successor])
            if advanced:
                continue
            frames.pop()
            if frames:
                parent = frames[-1][0]
                low[parent] = min(low[parent], low[node])
            if low[node] == index[node]:
                component = []
                while True:
                    member = stack.pop()
                    on_stack.discard(member)
                    component.append(member)
                    if member == node:
                        break
                if len(component) > 1 or node in graph[node]:
                    found.append(component)
    return found



def owners(ours):
    """Whose a symbol is: the workspace crate defining the item it instantiates, or for a drop glue
    shim the first workspace crate its type names; None for std's and dependencies'."""
    cache = {}

    def owner(symbol):
        if symbol not in cache:
            read = demangle(symbol)
            text, defining, named = read if read else (symbol, None, set())
            found = defining if defining in ours else None
            if found is None and text.startswith(DROP_GLUE):
                found = next((c for c in sorted(named) if c in ours), None)
            cache[symbol] = found
        return cache[symbol]

    return owner


def graph(functions, resolver):
    """The call graph, direct calls and calls through pointers alike, with how each call through a
    pointer was resolved: (graph, labels, counts)."""
    out = {symbol: {c for c in function.callees if c in functions} for symbol, function in functions.items()}
    labels = {}
    counts = {"direct": 0, "slots": 0, "slots_unresolved": 0, "pointers": 0, "pointers_unresolved": 0}

    def add(caller, callee, label):
        if callee not in out[caller]:
            out[caller].add(callee)
            labels.setdefault((caller, callee), label)

    for symbol, function in functions.items():
        counts["direct"] += len(function.callees)
        for offset, vtable, dbg, signature in function.slots:
            counts["slots"] += 1
            resolved, label = resolver.slot(symbol, offset, vtable, dbg, signature)
            reached = False
            for context, found in resolved:
                how = label if context == symbol else f"{label}, in {resolver.text(symbol)}"
                for callee in found:
                    if callee in functions:
                        reached = True
                        add(context, callee, how)
            if not reached:
                counts["slots_unresolved"] += 1
        for signature in function.pointers:
            counts["pointers"] += 1
            found = resolver.pointer_call(signature)
            if not found:
                counts["pointers_unresolved"] += 1
            for callee in found:
                add(symbol, callee, "through a function pointer (any function whose address is taken, of its signature)")
    return {symbol: sorted(callees) for symbol, callees in out.items()}, labels, counts


def cycles(functions, edges, labels, owner, root):
    """The cycles of the call graph: (ours, not ours), each a (stable name, [(text, site, owned)],
    [the calls through pointers inside it]) sorted by name; a symbol this reader cannot parse shows
    raw and is no one's."""
    mine, others = [], []
    for component in components(edges):
        members, texts = [], {}
        for symbol in component:
            read = demangle(symbol)
            texts[symbol] = read[0] if read else symbol
            members.append((texts[symbol], shown(functions[symbol].site, root), owner(symbol)))
        members.sort()
        inside = set(component)
        through = sorted(
            f"{texts[a]} → {texts[b]}: {labels[(a, b)]}"
            for a in component for b in edges[a] if b in inside and (a, b) in labels)
        own = sorted({text for text, _, who in members if who})
        if own:
            mine.append((" | ".join(own), members, through))
        else:
            others.append((" | ".join(sorted({text for text, _, _ in members})), members, through))
    return sorted(mine), sorted(others)


def describe(stable, members, through):
    lines = [f"a call-graph cycle (CLAUDE.md §11): rewrite it as flat work: {stable}"]
    for text, site, who in members:
        lines.append(f"    {site}  {text}" + ("" if who else "  [std or a dependency]"))
    for call in through:
        lines.append(f"    calls {call}")
    return "\n".join(lines)


# The reader on small inputs: v0 symbols and what they read as (the forms the IR holds: a method of
# an inherent impl, a trait impl, a closure in a generic function, drop glue, a shim, a function
# pointer type and a back-reference), Mantle's, and a module whose cycles are known: one direct, one
# through a vtable slot, and std's alone.
SYMBOLS = {
    "_RNvMs_NtCsjDp8xmfcqRr_9mantle_s34jsonNtB4_5Value5write": ("<mantle_s3::json::Value>::write", "mantle_s3"),
    "_RINvNtCs8Mbv00yxnRz_4core3ptr9drop_glueINtNtB4_6option6OptionNtNtCsaH3QtraahVu_11mantle_node4node9NodeErrorEEB11_": (
        "core::ptr::drop_glue::<core::option::Option<mantle_node::node::NodeError>>", "core"),
    "_RNvMNtNtCs8Mbv00yxnRz_4core3ptr7mut_ptrOy3subCs91YuFR6N3tf_7probe_x": ("<*mut u64>::sub", "core"),
    "_RINvMNtNtNtNtCs8Mbv00yxnRz_4core5slice4sort6stable5mergeINtB3_10MergeStateyE10merge_downNvYyNtNtBb_3cmp10PartialOrd2ltECs91YuFR6N3tf_7probe_x": (
        "<core::slice::sort::stable::merge::MergeState<u64>>::merge_down::<<u64 as core::cmp::PartialOrd>::lt>", "core"),
    "_RINvMNtCs8Mbv00yxnRz_4core3stre10split_onceNCNvNtCsjDp8xmfcqRr_9mantle_s35sigv412header_claim0EBL_": (
        "<str>::split_once::<mantle_s3::sigv4::header_claim::{closure#0}>", "core"),
    "_RNvXsJ_NtCs8Mbv00yxnRz_4core6optionINtB5_6OptionTjjEENtNtNtB7_3ops9try_trait3Try6branchCsjDp8xmfcqRr_9mantle_s3": (
        "<core::option::Option<(usize, usize)> as core::ops::try_trait::Try>::branch", "core"),
    "_RNCNvCs1_5crate1f0B3_": ("crate::f::{closure#0}", "crate"),
    "_RNvYNtCs1_1a1TNtCs2_1b5Trait1m": ("<a::T as b::Trait>::m", "b"),
}
MODULE = """\
target datalayout = "e-m:o-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-n32:64-S128-Fn32"
@vtable.0 = private unnamed_addr constant <{ [24 x i8], ptr }> <{ [24 x i8] c"\\00\\00\\00\\00\\00\\00\\00\\00\\08\\00\\00\\00\\00\\00\\00\\00\\08\\00\\00\\00\\00\\00\\00\\00", ptr @_RNvYNtCs1_1a1TNtCs1_1a5Trait1m }>, align 8
define internal void @_RNvCs1_1a4walk(ptr %n) unnamed_addr !dbg !10 {
start:
  call void @_RNvCs1_1a4step(ptr %n), !dbg !12
  %r = invoke i64 @_RNvCs1_1a4walk(ptr %n)
          to label %ok unwind label %cleanup
}
define internal void @_RNvCs1_1a4step(ptr %n) unnamed_addr {
start:
  tail call void @_RNvCs1_1a4walk(ptr %n)
  call void @llvm.memcpy.p0.p0.i64(ptr %n, ptr %n, i64 1, i1 false)
}
define internal void @_RNvCs1_4core4sort(ptr %n) unnamed_addr {
start:
  call void @_RNvCs1_4core4sort(ptr %n)
}
define internal void @_RNvCs1_1a4leaf(ptr %n) unnamed_addr {
start:
  call void @_RNvCs1_4core4sort(ptr %n)
}
define internal void @_RNvYNtCs1_1a1TNtCs1_1a5Trait1m(ptr align 8 %self) unnamed_addr {
start:
  %0 = getelementptr inbounds i8, ptr %self, i64 8
  %vt = load ptr, ptr %0, align 8
  %1 = getelementptr inbounds i8, ptr %vt, i64 24
  %2 = load ptr, ptr %1, align 8, !invariant.load !5, !nonnull !5
  call void %2(ptr %self)
  call void %f(ptr %self)
}
define internal void @_RNvCs1_1a4make(ptr %out) unnamed_addr {
start:
  store ptr @vtable.0, ptr %out, align 8
  ret void
}
!10 = distinct !DISubprogram(name: "walk", scope: !11, file: !11, line: 7, type: !13, unit: !1)
!11 = !DIFile(filename: "crates/fixture/src/lib.rs", directory: "/nowhere")
"""


def fixtures():
    """The reader on the inputs above: the failures where it reads other than it must."""
    failures = []
    for symbol, expect in SYMBOLS.items():
        read = demangle(symbol)
        got = None if read is None else (read[0], read[1])
        if got != expect:
            failures.append(f"scripts/check-recursion.py: fixture {symbol}: read {got}, not {expect}")
    with tempfile.TemporaryDirectory(prefix="check-recursion-") as directory:
        path = pathlib.Path(directory, "fixture.ll")
        path.write_text(MODULE, encoding="utf-8")
        functions, vtables, constants, locations, pointer, named = read_ir([path])
    owner = owners({"a"})
    resolver = Resolver(functions, vtables, constants, locations, pointer, owner, named, {"a"})
    edges, labels, counts = graph(functions, resolver)
    mine, others = cycles(functions, edges, labels, owner, pathlib.Path("/nowhere"))
    names = [stable for stable, _, _ in mine]
    want = ["<a::T as a::Trait>::m", "a::step | a::walk"]
    if names != want or [s for s, _, _ in others] != ["core::sort"]:
        failures.append(f"scripts/check-recursion.py: fixture module: cycles {names}, not {want}; others {[s for s, _, _ in others]}")
    if counts["slots"] != 1 or counts["pointers"] != 1:
        failures.append(f"scripts/check-recursion.py: fixture module: calls through pointers {counts}")
    site = shown(functions["_RNvCs1_1a4walk"].site if "_RNvCs1_1a4walk" in functions else None, pathlib.Path("/elsewhere"))
    if site != "/nowhere/crates/fixture/src/lib.rs:7":
        failures.append(f"scripts/check-recursion.py: fixture module: walk's site {site}")
    return failures


def main() -> int:
    args = sys.argv[1:]
    root = ROOT
    target = None
    while args:
        if args[0] == "--workspace" and len(args) > 1:
            root = pathlib.Path(args[1]).resolve()
            args = args[2:]
        elif args[0] == "--target-dir" and len(args) > 1:
            target = pathlib.Path(args[1]).resolve()
            args = args[2:]
        else:
            print("usage: check-recursion.py [--workspace DIR] [--target-dir DIR]", file=sys.stderr)
            return 2
    if target is None:
        # Beside the build's own output: CARGO_TARGET_DIR when it is set, as cargo's is.
        base = os.environ.get("CARGO_TARGET_DIR")
        target = (pathlib.Path(base) if base else root / "target") / "check-recursion"
    failures = fixtures()
    paths, ours, build_failures = build(root, target)
    failures.extend(build_failures)
    if paths is None:
        for failure in failures:
            print(failure, file=sys.stderr)
        return 1
    functions, vtables, constants, locations, pointer, named = read_ir(paths)
    owner = owners(ours)
    resolver = Resolver(functions, vtables, constants, locations, pointer, owner, named, ours)
    edges, labels, counts = graph(functions, resolver)
    mine, others = cycles(functions, edges, labels, owner, root)
    failures.extend(describe(stable, members, through) for stable, members, through in mine)
    for failure in failures:
        print(failure, file=sys.stderr)
    print(
        f"check-recursion: {len(functions)} functions, {counts['direct']} direct calls,"
        f" {counts['slots']} calls through trait objects ({counts['slots_unresolved']} reaching none of the workspace's vtables),"
        f" {counts['pointers']} through function pointers ({counts['pointers_unresolved']} reaching none of its functions),"
        f" {len(vtables)} vtables; {len(mine)} cycles of ours"
        f" ({len(others)} in std's or dependencies' generic code alone, not ours)",
        file=sys.stderr if failures else sys.stdout,
    )
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
