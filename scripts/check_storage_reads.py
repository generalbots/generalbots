#!/usr/bin/env python3
"""LocalStorage read gate (issue #2310).

`localStorage` outlives a deploy: a schema change, an evicted-then-truncated
write or a hand edit leaves one corrupt key, and `JSON.parse` of it throws.
When that read sits in the desktop boot chain the whole shell dies and the user
cannot open the app to clear the key — the failure is total and manual.

The suite's safe reader is `gbStoreGet(key, fallback)` in `js/gb-shared.js`: it
parses, and on a corrupt value drops the key once, warns with the key name and
returns the fallback. This gate fails on any raw `JSON.parse(localStorage...)`
in the UI so a new read cannot reintroduce the crash.

Usage
-----
    python3 scripts/check_storage_reads.py --check
    python3 scripts/check_storage_reads.py --list
    python3 scripts/check_storage_reads.py --self-test
    python3 scripts/check_storage_reads.py --root <repo>

Exit codes: 0 clean, 1 violations found.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SCAN_ROOT = Path("botui/ui")
SUFFIXES = (".html", ".js")
SKIP_SEGMENTS = ("vendor", "node_modules")

# `JSON.parse(localStorage.getItem(...))`, `JSON.parse(localStorage['k'])`, and
# `JSON.parse( window.localStorage ... )` are all the same hazard.
RAW_READ = re.compile(
    r"JSON\.parse\(\s*" r"(?:window\.)?" r"localStorage\s*(?:\.\s*getItem\s*\(|\[)",
)
SAFE_READ = "gbStoreGet("


def candidates(root: Path) -> list[Path]:
    base = root / SCAN_ROOT
    if not base.is_dir():
        return []
    files = []
    for path in base.rglob("*"):
        if not path.is_file() or path.suffix.lower() not in SUFFIXES:
            continue
        if any(part in SKIP_SEGMENTS for part in path.parts):
            continue
        files.append(path)
    return sorted(files)


def scan_text(text: str) -> list[int]:
    return [text.count("\n", 0, m.start()) + 1 for m in RAW_READ.finditer(text)]


def check(root: Path) -> int:
    violations = 0
    for path in candidates(root):
        try:
            text = path.read_text(encoding="utf-8", errors="replace")
        except OSError as exc:
            print(f"warning: cannot read {path}: {exc}", file=sys.stderr)
            continue
        for line in scan_text(text):
            print(f"{path.relative_to(root)}:{line}: raw JSON.parse(localStorage...) — use {SAFE_READ} instead")
            violations += 1
    if violations:
        print(f"\n{violations} raw localStorage read(s); the suite reader is gbStoreGet(key, fallback).")
        return 1
    print("no raw JSON.parse(localStorage...) reads")
    return 0


def list_sites(root: Path) -> int:
    total = 0
    for path in candidates(root):
        try:
            text = path.read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        for line in scan_text(text):
            print(f"{path.relative_to(root)}:{line}")
            total += 1
    print(f"\n{total} site(s)")
    return 0


def self_test() -> int:
    bad = [
        'var a = JSON.parse(localStorage.getItem("k") || "[]");',
        "var b = JSON.parse(window.localStorage.getItem(k));",
        "var c = JSON.parse(localStorage['k']);",
    ]
    good = [
        'var a = gbStoreGet("k", []);',
        "var b = JSON.parse(payload);",
        "localStorage.setItem('k', JSON.stringify(v));",
    ]
    failures = 0
    for line in bad:
        if not scan_text(line):
            print(f"self-test: missed {line}")
            failures += 1
    for line in good:
        if scan_text(line):
            print(f"self-test: false positive on {line}")
            failures += 1
    if failures:
        return 1
    print("self-test ok")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--list", action="store_true")
    parser.add_argument("--self-test", action="store_true")
    parser.add_argument("--root", type=Path, default=ROOT)
    args = parser.parse_args()
    if not (args.check or args.list or args.self_test):
        parser.print_help()
        return 2
    status = 0
    if args.self_test:
        status |= self_test()
    if args.list:
        status |= list_sites(args.root)
    if args.check:
        status |= check(args.root)
    return status


if __name__ == "__main__":
    raise SystemExit(main())
