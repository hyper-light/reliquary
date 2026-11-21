"""Rust source as the gates read it (scripts/check-unsafe.py, scripts/check-tests.py): tokens with
comments and literals set apart, so a word inside a string or a comment is never read as code.

What it reads, from the Reference's lexical structure (doc.rust-lang.org/reference/tokens.html,
comments.html): line comments (`//`, doc `///` and `//!`), block comments (nested, doc `/**` and
`/*!`), string literals with their prefixes (`b`, `c`, raw `r#"…"#`), character and byte literals
told apart from lifetimes, identifiers (raw `r#…` included), and punctuation one character a
token. Numbers are read as words. It never fails: an unclosed literal or comment runs to the end
of the file, which rustc would refuse anyway.
"""
from collections import namedtuple

# kind: "word", "punct", "doc" (a doc comment's text), "string", "char" or "lifetime".
Token = namedtuple("Token", "kind text line")

WORD_START = set("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_")
WORD = WORD_START | set("0123456789")


def tokens(text):
    """The file's tokens, in order, with each one's 1-based line."""
    out = []
    i, line, n = 0, 1, len(text)
    while i < n:
        c = text[i]
        if c == "\n":
            line += 1
            i += 1
        elif c in " \t\r":
            i += 1
        elif text.startswith("//", i):
            end = text.find("\n", i)
            end = n if end < 0 else end
            body = text[i:end]
            if (body.startswith("///") and not body.startswith("////")) or body.startswith("//!"):
                out.append(Token("doc", body[3:], line))
            i = end
        elif text.startswith("/*", i):
            start, depth, j = line, 1, i + 2
            while j < n and depth:
                if text.startswith("/*", j):
                    depth += 1
                    j += 2
                elif text.startswith("*/", j):
                    depth -= 1
                    j += 2
                else:
                    if text[j] == "\n":
                        line += 1
                    j += 1
            body = text[i:j]
            if (body.startswith("/**") and not body.startswith("/***") and body != "/**/") or body.startswith("/*!"):
                out.append(Token("doc", body[3:-2], start))
            i = j
        elif c in WORD_START or (c.isalpha() and ord(c) > 127):
            raw_string = string_prefix(text, i)
            if raw_string is not None:
                end, lines = raw_string
                out.append(Token("string", text[i:end], line))
                line += lines
                i = end
                continue
            j = i + 1
            if text.startswith("r#", i) and i + 2 < n and text[i + 2] in WORD_START:
                j = i + 2
            while j < n and (text[j] in WORD or (text[j].isalnum() and ord(text[j]) > 127)):
                j += 1
            out.append(Token("word", text[i:j].removeprefix("r#"), line))
            i = j
        elif c.isdigit():
            j = i + 1
            while j < n and (text[j] in WORD or text[j] == "."):
                if text[j] == "." and not (j + 1 < n and text[j + 1].isdigit()):
                    break
                j += 1
            out.append(Token("word", text[i:j], line))
            i = j
        elif c == '"':
            end, lines = quoted(text, i + 1, '"')
            out.append(Token("string", text[i:end], line))
            line += lines
            i = end
        elif c == "'":
            # A character literal ('a', '\n', '\u{1F600}') or a lifetime or label ('a, 'static).
            if i + 2 < n and text[i + 1] == "\\":
                end, lines = quoted(text, i + 1, "'")
                out.append(Token("char", text[i:end], line))
                line += lines
                i = end
            elif i + 2 < n and text[i + 2] == "'":
                out.append(Token("char", text[i:i + 3], line))
                i += 3
            else:
                j = i + 1
                while j < n and (text[j] in WORD or (text[j].isalnum() and ord(text[j]) > 127)):
                    j += 1
                if j < n and text[j] == "'" and j > i + 1:
                    out.append(Token("char", text[i:j + 1], line))
                    i = j + 1
                else:
                    out.append(Token("lifetime", text[i:j], line))
                    i = j
        else:
            out.append(Token("punct", c, line))
            i += 1
    return out


def quoted(text, i, close):
    """The end of a literal whose body starts at `i` and ends at `close`, escapes skipped, and the
    line breaks inside it."""
    n, lines = len(text), 0
    while i < n:
        c = text[i]
        if c == "\\":
            if i + 1 < n and text[i + 1] == "\n":
                lines += 1
            i += 2
            continue
        if c == "\n":
            lines += 1
        if c == close:
            return i + 1, lines
        i += 1
    return n, lines


def string_prefix(text, i):
    """A string literal with a prefix starting at `i` (`b"…"`, `c"…"`, `r"…"`, `r#"…"#`, `br#"…"#`,
    `cr"…"`, `b'…'`): its end and the line breaks inside it; None when `i` starts a word."""
    n = len(text)
    j = i
    if text.startswith(("br", "cr"), j):
        j += 2
        raw = True
    elif text[j] in "bc":
        j += 1
        raw = False
        if j < n and text[j] == "r":
            j += 1
            raw = True
    elif text[j] == "r":
        j += 1
        raw = True
    else:
        return None
    if raw:
        hashes = 0
        while j < n and text[j] == "#":
            hashes += 1
            j += 1
        if j >= n or text[j] != '"':
            return None
        close = '"' + "#" * hashes
        end = text.find(close, j + 1)
        end = n if end < 0 else end + len(close)
        return end, text.count("\n", i, end)
    if j < n and text[j] == '"':
        return quoted(text, j + 1, '"')
    if j < n and text[j] == "'" and text[i] == "b":
        return quoted(text, j + 1, "'")
    return None


def matching(toks, at):
    """The index of the token closing the bracket token at `at` (`(`, `[` or `{`), or len(toks)."""
    pairs = {"(": ")", "[": "]", "{": "}"}
    stack = []
    for k in range(at, len(toks)):
        t = toks[k]
        if t.kind != "punct":
            continue
        if t.text in pairs:
            stack.append(pairs[t.text])
        elif stack and t.text == stack[-1]:
            stack.pop()
            if not stack:
                return k
    return len(toks)
