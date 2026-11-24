#!/usr/bin/env python3
"""Static audit of the office chat-verb bindings (#2448).

Proves, without building or running botserver, that every `bind(...)` in
office_endpoints.rs is:

  1. UNIQUE   - no verb name or bind signature is declared twice;
  2. REAL     - the verb name exists in the curated command catalog
                (`cmd(...)` entries in apps/commands);
  3. BACKED   - the (method, path) pair exists in the endpoint inventory
                (`ApiEndpoint { method, path }` entries under core/bot).

Both `"/api/x"` and `"api/x"` spellings are normalised before comparison.

Duplicate (method, path) pairs across *different* verbs are intentional
aliases in this codebase (e.g. docs.save/docs.update both target
/api/docs/save). They are allowlisted below; any NEW duplicate pair fails.
Run with `--verbose` to list the accepted aliases.

Exit 0 only on a clean audit; exit 1 on any violation.
"""

from __future__ import annotations

import argparse
import re
import sys
from collections import Counter, defaultdict
from pathlib import Path
from typing import NamedTuple

DEFAULT_ROOT = Path(__file__).resolve().parent.parent

REL_BINDINGS = "botserver/src/apps/commands/verb_endpoints/table/office_endpoints.rs"
REL_CATALOG = "botserver/src/apps/commands"
REL_INVENTORY = "botserver/src/core/bot"
SKIP_CATALOG_FILES = {"app_command.rs"}  # the cmd() definition, not a catalog entry
SKIP_NAME_PARTS = ("tests",)

# (method, normalised path) -> verbs that are allowed to share it.
INTENTIONAL_ALIASES: dict[tuple[str, str], frozenset[str]] = {
    ("POST", "api/docs/save"): frozenset({"docs.update", "docs.save"}),
    ("POST", "api/sheet/cells"): frozenset({"sheet.cells.set", "sheet.merge"}),
    ("POST", "api/slides/element/add"): frozenset(
        {"slides.element.add", "slides.media.add"}
    ),
}

PAIR_RE = re.compile(r'\(\s*"([^"]*)"\s*,\s*"([^"]*)"\s*\)')
CMD_RE = re.compile(r'\bcmd\(\s*"[^"]*"\s*,\s*"([^"]+)"')
ENDPOINT_RE = re.compile(r"\bApiEndpoint\s*\{")
BIND_RE = re.compile(r"\bbind\s*\(")
ESCAPES = {"n": "\n", "r": "\r", "t": "\t", '"': '"', "\\": "\\", "'": "'"}


class Bind(NamedTuple):
    index: int
    line: int
    verb: str
    method: str
    path: str
    renames: tuple[tuple[str, str], ...]
    defaults: tuple[tuple[str, str], ...]


def after_string(text: str, i: int) -> int:
    """Return the offset just past the double-quoted string starting at i."""
    j = i + 1
    while j < len(text):
        if text[j] == "\\":
            j += 2
            continue
        if text[j] == '"':
            return j + 1
        j += 1
    raise ValueError(f"unterminated string literal at offset {i}")


def mask_comments(text: str) -> str:
    """Blank // and /* */ comments without touching string literals.

    Masking preserves offsets and length so line numbers stay accurate.
    """
    out = list(text)
    i, n = 0, len(text)
    while i < n:
        ch = text[i]
        if ch == '"':
            i = after_string(text, i)
            continue
        if text.startswith("//", i):
            while i < n and text[i] != "\n":
                out[i] = " "
                i += 1
            continue
        if text.startswith("/*", i):
            depth = 1
            out[i] = out[i + 1] = " "
            i += 2
            while i < n and depth:
                if text.startswith("/*", i):
                    out[i] = out[i + 1] = " "
                    i += 2
                    depth += 1
                elif text.startswith("*/", i):
                    out[i] = out[i + 1] = " "
                    i += 2
                    depth -= 1
                else:
                    if text[i] != "\n":
                        out[i] = " "
                    i += 1
            if depth:
                raise ValueError("unterminated block comment")
            continue
        i += 1
    return "".join(out)


def balanced_blocks(masked: str, pattern: re.Pattern[str]) -> list[tuple[int, str]]:
    """Return (start offset, full block text) for each matched balanced call."""
    blocks: list[tuple[int, str]] = []
    for match in pattern.finditer(masked):
        start = match.end() - 1
        opener = masked[start]
        closer = ")" if opener == "(" else "}"
        depth, j, end = 0, start, None
        while j < len(masked):
            ch = masked[j]
            if ch == '"':
                j = after_string(masked, j)
                continue
            if ch == opener:
                depth += 1
            elif ch == closer:
                depth -= 1
                if depth == 0:
                    end = j + 1
                    break
            j += 1
        if end is None:
            raise ValueError(f"unbalanced '{opener}{closer}' at offset {start}")
        blocks.append((match.start(), masked[match.start() : end]))
    return blocks


def unescape(raw: str) -> str:
    out: list[str] = []
    i = 0
    while i < len(raw):
        if raw[i] == "\\" and i + 1 < len(raw):
            out.append(ESCAPES.get(raw[i + 1], raw[i + 1]))
            i += 2
        else:
            out.append(raw[i])
            i += 1
    return "".join(out)


def string_arg(arg: str) -> str:
    value = arg.strip()
    if len(value) < 2 or not value.startswith('"') or not value.endswith('"'):
        raise ValueError(f"expected string literal, got {arg!r}")
    return unescape(value[1:-1])


def split_args(body: str) -> list[str]:
    """Split a call body on top-level commas (ignoring strings/brackets)."""
    parts: list[str] = []
    depth, instr, esc, cur = 0, False, False, []
    for ch in body:
        if instr:
            cur.append(ch)
            if esc:
                esc = False
            elif ch == "\\":
                esc = True
            elif ch == '"':
                instr = False
            continue
        if ch == '"':
            instr = True
            cur.append(ch)
        elif ch in "([{":
            depth += 1
            cur.append(ch)
        elif ch in ")]}":
            depth -= 1
            cur.append(ch)
        elif ch == "," and depth == 0:
            parts.append("".join(cur).strip())
            cur = []
        else:
            cur.append(ch)
    if "".join(cur).strip():
        parts.append("".join(cur).strip())
    return parts


def pairs_arg(arg: str) -> tuple[tuple[str, str], ...]:
    """Parse a `&[("a", "b"), ...]` argument, rejecting unparsed residue."""
    value = arg.strip()
    if value.startswith("&"):
        value = value[1:].strip()
    if not (value.startswith("[") and value.endswith("]")):
        raise ValueError(f"expected a slice literal, got {arg!r}")
    inner = value[1:-1]
    pairs = tuple((unescape(a), unescape(b)) for a, b in PAIR_RE.findall(inner))
    residue = re.sub(r"[\s,]", "", PAIR_RE.sub("", inner))
    if residue:
        raise ValueError(f"unparsed content in slice {arg!r}: {residue!r}")
    return pairs


def read_masked(path: Path) -> str:
    return mask_comments(path.read_text(encoding="utf-8"))


def parse_binds(masked: str) -> list[Bind]:
    binds: list[Bind] = []
    for index, (offset, block) in enumerate(balanced_blocks(masked, BIND_RE)):
        line = masked.count("\n", 0, offset) + 1
        body = block[block.index("(") + 1 : -1]
        args = split_args(body)
        if len(args) != 5:
            raise ValueError(
                f"bind at line {line} has {len(args)} args, expected 5: {body[:120]!r}"
            )
        binds.append(
            Bind(
                index=index,
                line=line,
                verb=string_arg(args[0]),
                method=string_arg(args[1]),
                path=string_arg(args[2]),
                renames=pairs_arg(args[3]),
                defaults=pairs_arg(args[4]),
            )
        )
    return binds


def parse_catalog(catalog_dir: Path) -> set[str]:
    names: set[str] = set()
    for path in sorted(catalog_dir.rglob("*.rs")):
        if path.name in SKIP_CATALOG_FILES or any(p in path.name for p in SKIP_NAME_PARTS):
            continue
        names.update(CMD_RE.findall(read_masked(path)))
    return names


def parse_inventory(inventory_dir: Path) -> set[tuple[str, str]]:
    routes: set[tuple[str, str]] = set()
    for path in sorted(inventory_dir.rglob("*.rs")):
        rel = str(path.relative_to(inventory_dir))
        if "endpoint_inventory" not in rel or any(p in path.name for p in SKIP_NAME_PARTS):
            continue
        masked = read_masked(path)
        for offset, block in balanced_blocks(masked, ENDPOINT_RE):
            line = masked.count("\n", 0, offset) + 1
            method = re.search(r'\bmethod\s*:\s*"([^"]+)"', block)
            route = re.search(r'\bpath\s*:\s*"([^"]+)"', block)
            if method is None or route is None:
                raise ValueError(f"{path.name}:{line}: ApiEndpoint lacks method/path")
            routes.add((method.group(1), route.group(1).lstrip("/")))
    return routes


def norm(path: str) -> str:
    return path.lstrip("/")


def audit(bindings_path: Path, catalog_dir: Path, inventory_dir: Path, verbose: bool) -> int:
    violations: list[str] = []
    try:
        if not bindings_path.is_file():
            raise ValueError(f"bindings file not found: {bindings_path}")
        if not catalog_dir.is_dir():
            raise ValueError(f"catalog directory not found: {catalog_dir}")
        if not inventory_dir.is_dir():
            raise ValueError(f"inventory directory not found: {inventory_dir}")
        binds = parse_binds(read_masked(bindings_path))
        catalog = parse_catalog(catalog_dir)
        routes = parse_inventory(inventory_dir)
    except (OSError, UnicodeDecodeError, ValueError) as exc:
        print(f"FAIL audit input error: {exc}")
        return 1
    if not binds:
        print("FAIL no bind(...) calls found - audit would be vacuous")
        return 1

    by_verb: dict[str, list[Bind]] = defaultdict(list)
    by_pair: dict[tuple[str, str], list[Bind]] = defaultdict(list)
    for bind in binds:
        by_verb[bind.verb].append(bind)
        by_pair[(bind.method, norm(bind.path))].append(bind)

    for verb, group in sorted(by_verb.items()):
        if len(group) > 1:
            where = ", ".join(f"#{b.index}:{b.line}" for b in group)
            violations.append(f"FAIL duplicate verb {verb!r}: binds {where}")

    aliases: list[tuple[str, str]] = []
    for pair, group in sorted(by_pair.items()):
        if len(group) < 2:
            continue
        verbs = frozenset(b.verb for b in group)
        expected = INTENTIONAL_ALIASES.get(pair)
        if expected is not None and verbs == expected and len(verbs) == len(group):
            aliases.append(pair)
            continue
        where = ", ".join(f"{b.verb} (#{b.index}:{b.line})" for b in group)
        violations.append(
            f"FAIL duplicate (method,path) {' '.join(pair)!r}: {where}"
            " - not an allowlisted alias"
        )

    for bind in binds:
        if bind.verb not in catalog:
            violations.append(
                f"FAIL verb not in curated catalog: {bind.verb!r} (#{bind.index}:{bind.line})"
            )
        pair = (bind.method, norm(bind.path))
        if pair not in routes:
            violations.append(
                f"FAIL route not registered: {bind.method} {bind.path}"
                f" (#{bind.index}:{bind.line}, verb {bind.verb!r})"
            )

    covered_verbs = sum(1 for b in binds if b.verb in catalog)
    covered_routes = sum(1 for b in binds if (b.method, norm(b.path)) in routes)
    renames = sum(len(b.renames) for b in binds)
    defaults = sum(len(b.defaults) for b in binds)

    print("=== Office verb binding audit (#2448) ===")
    print(f"  bindings file           : {bindings_path}")
    print(f"  bind calls              : {len(binds)}")
    print(f"  unique verbs            : {len(by_verb)}")
    print(f"  unique (method,path)    : {len(by_pair)} ({len(aliases)} intentional aliases)")
    print(f"  renames / defaults      : {renames} / {defaults}")
    print(f"  catalog coverage        : {covered_verbs}/{len(binds)}")
    print(f"  route coverage          : {covered_routes}/{len(binds)}")
    if verbose:
        for pair in aliases:
            print(f"  alias                   : {' '.join(pair)} -> "
                  + ", ".join(sorted(INTENTIONAL_ALIASES[pair])))
    for violation in violations:
        print(violation)
    print("RESULT: " + ("FAIL" if violations else "PASS"))
    return 1 if violations else 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=DEFAULT_ROOT, help="workspace root")
    parser.add_argument("--bindings", type=Path, help="override office_endpoints.rs")
    parser.add_argument("--catalog", type=Path, help="override commands directory")
    parser.add_argument("--inventory", type=Path, help="override core/bot directory")
    parser.add_argument("--verbose", action="store_true", help="list allowlisted aliases")
    args = parser.parse_args()
    return audit(
        bindings_path=args.bindings or args.root / REL_BINDINGS,
        catalog_dir=args.catalog or args.root / REL_CATALOG,
        inventory_dir=args.inventory or args.root / REL_INVENTORY,
        verbose=args.verbose,
    )


if __name__ == "__main__":
    sys.exit(main())
