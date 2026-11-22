#!/usr/bin/env python3
"""CSS hygiene gate for the design-system sweeps (#2323, #2324).

Two rules, both enforced with a shrink-only per-file budget so the existing
debt can only go down:

  raw-hex       6-digit hex colour literals in *component* CSS. The token
                definition files (css/base/**, public/themes/**,
                identity-tokens.css, config-colors.css and theme-* files) are
                exempt: those files are where colours are *allowed* to live.
                Everywhere else a colour must be var(--token).
  bang-important `!important` declarations in component CSS (token
                definition files exempt). The cascade must be resolved through
                specificity, not brute force.

The budget file `scripts/css_hygiene_budget.txt` records the count per file as
`<rule>:<relative-path> <count>`. It is shrink-only: a file that falls to zero
must have its line removed, and a file that appears with *more* literals than
its budget (or a new file with any) is a violation. This is robust to line
number shifts, unlike a line-based allowlist.

Usage:
    python3 scripts/check_css_hygiene.py --check
    python3 scripts/check_css_hygiene.py --write-budget
    python3 scripts/check_css_hygiene.py --self-test

Exit codes: 0 clean, 1 violations (or a stale budget entry), 2 usage error.
"""

from __future__ import annotations

import argparse
import re
import sys
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SUITE = Path("botui/ui/suite")
BUDGET = Path("scripts/css_hygiene_budget.txt")
SKIP_DIRS = {"vendor", "node_modules"}

HEX6 = re.compile(r"#[0-9a-fA-F]{6}\b")
BANG = re.compile(r"!important\b")

# Files where colour literals are the definition, not a component violation.
TOKEN_PATH_PARTS = ("css/base/", "public/themes/")
TOKEN_FILE_NAMES = {"identity-tokens.css", "config-colors.css", "palette-tokens.css", "high-contrast.css"}
BANG_ROOTS = (Path("botui/ui/suite/vibe"), Path("botui/ui/suite/settings"))


def is_token_file(rel: Path) -> bool:
    text = rel.as_posix()
    return (
        any(part in text for part in TOKEN_PATH_PARTS)
        or rel.name in TOKEN_FILE_NAMES
        or any(part.startswith("theme-") for part in rel.parts)
    )


def css_files() -> list[Path]:
    base = ROOT / SUITE
    return [
        path
        for path in base.rglob("*.css")
        if not (SKIP_DIRS & set(path.relative_to(base).parts))
    ]


def scan() -> dict[str, dict[str, int]]:
    counts: dict[str, dict[str, int]] = {
        "raw-hex": defaultdict(int),
        "bang-important": defaultdict(int),
    }
    for path in css_files():
        rel = path.relative_to(ROOT).as_posix()
        try:
            text = path.read_text(encoding="utf-8")
        except OSError:
            continue
        if not is_token_file(Path(rel).relative_to(SUITE)):
            n = len(HEX6.findall(text))
            if n:
                counts["raw-hex"][rel] = n
        # #2323 — the whole component surface is budgeted now, not only the
        # vibe/settings hotspots. Token definition files remain exempt.
        if not is_token_file(Path(rel).relative_to(SUITE)):
            n = len(BANG.findall(text))
            if n:
                counts["bang-important"][rel] = n
    return counts


def load_budget() -> dict[tuple[str, str], int]:
    budget: dict[tuple[str, str], int] = {}
    if not (ROOT / BUDGET).exists():
        return budget
    for raw in (ROOT / BUDGET).read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        key, _, value = line.partition(" ")
        rule, sep, path = key.partition(":")
        if not sep or not value.isdigit():
            continue
        budget[(rule, path)] = int(value)
    return budget


def current_keys(counts: dict[str, dict[str, int]]) -> set[tuple[str, str]]:
    return {(rule, path) for rule, paths in counts.items() for path in paths}


def check(counts: dict[str, dict[str, int]]) -> int:
    budget = load_budget()
    current = current_keys(counts)
    failures: list[str] = []

    for (rule, path), allowed in sorted(budget.items()):
        actual = counts.get(rule, {}).get(path, 0)
        if actual == 0:
            failures.append(f"STALE BUDGET {rule}:{path} (budget {allowed}, now 0 — delete the line)")
        elif actual > allowed:
            failures.append(f"BUDGET EXCEEDED {rule}:{path} {actual} > {allowed}")

    for rule, path in sorted(current - set(budget)):
        failures.append(f"NEW {rule}:{path} = {counts[rule][path]} — no budget entry; convert to a token")

    for failure in failures[:80]:
        print(f"FAIL: {failure}")
    if len(failures) > 80:
        print(f"... and {len(failures) - 80} more")
    if failures:
        print(f"css hygiene: {len(failures)} violation(s)")
        return 1
    print(f"css hygiene: OK ({sum(len(p) for p in counts.values())} budgeted files)")
    return 0


def write_budget(counts: dict[str, dict[str, int]]) -> int:
    lines = [
        "# CSS hygiene budget (#2323, #2324) — shrink-only.",
        "# Format: <rule>:<relative-path> <count>. Delete a line when its count hits zero.",
        "# raw-hex = 6-digit colour literal outside token-definition files.",
        "# bang-important = !important in vibe/** and settings/** CSS.",
        "",
    ]
    for rule in ("raw-hex", "bang-important"):
        for path, count in sorted(counts[rule].items()):
            lines.append(f"{rule}:{path} {count}")
    (ROOT / BUDGET).write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"wrote {sum(len(p) for p in counts.values())} budget entries to {BUDGET}")
    return 0


def self_test() -> int:
    assert len(HEX6.findall("color: #d4f505; border: #abc;")) == 1
    assert len(BANG.findall("a{color:red!important} b{color:blue !important}")) == 2
    assert is_token_file(Path("css/base/01_root.css"))
    assert is_token_file(Path("public/themes/light.css"))
    assert is_token_file(Path("css/theme-sentient/01_theme.css"))
    assert not is_token_file(Path("vibe/vibe-windows.css"))
    print("self-test: OK")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--write-budget", action="store_true")
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if not (args.check or args.write_budget or args.self_test):
        parser.print_help()
        return 2
    if args.self_test:
        return self_test()
    counts = scan()
    if args.write_budget:
        return write_budget(counts)
    return check(counts)


if __name__ == "__main__":
    sys.exit(main())
