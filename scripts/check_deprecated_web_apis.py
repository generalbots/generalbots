#!/usr/bin/env python3
"""Deprecated web API gate (issue #2340).

`document.write` is parser-blocking and can clear a document; `String#substr`
is a legacy alias of `slice`; and a raw `document.execCommand` call site is a
migration liability because its behaviour differs per browser and it bypasses
the one place where a modern replacement will be swapped in.

The UI allows exactly these exceptions, and this gate fails on anything else:

* `botui/ui/suite/js/gb-shared.js` — the single `document.execCommand` call
  (`gbFormat`) and the single `execCommand("copy")` fallback (`gbCopyText`).
* `botui/ui/suite/<app>/vendored` and any `vendor/` path segment.

Usage
-----
    python3 scripts/check_deprecated_web_apis.py --check
    python3 scripts/check_deprecated_web_apis.py --list
    python3 scripts/check_deprecated_web_apis.py --self-test
    python3 scripts/check_deprecated_web_apis.py --root <repo>

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
# `settings/index.html` still bootstraps its scripts with `document.write` and
# loads the security bootstrap twice (v6 and v7). That is tracked as its own
# issue (#2332) because the fix changes load order; #2340 must not duplicate it.
EXEMPT = {
    Path("botui/ui/suite/js/gb-shared.js"),
    Path("botui/ui/suite/settings/index.html"),
}

RULES = (
    ("document.write", re.compile(r"document\s*\.\s*write\s*\("),
     "document.write is legacy — build DOM, or use gbPrintHtml() for print views"),
    ("substr", re.compile(r"\.\s*substr\s*\("),
     "String#substr is deprecated — use slice()/substring()"),
    ("execCommand", re.compile(r"\bexecCommand\s*\("),
     "raw document.execCommand — go through gbFormat() in js/gb-shared.js"),
)


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
        if path.relative_to(root) in EXEMPT:
            continue
        files.append(path)
    return sorted(files)


def scan_text(text: str):
    """Yield (line_number, rule_name, hint) for every hit in `text`."""
    for name, pattern, hint in RULES:
        for m in pattern.finditer(text):
            yield text.count("\n", 0, m.start()) + 1, name, hint


def check(root: Path) -> int:
    violations = 0
    for path in candidates(root):
        try:
            text = path.read_text(encoding="utf-8", errors="replace")
        except OSError as exc:
            print(f"warning: cannot read {path}: {exc}", file=sys.stderr)
            continue
        for line, name, hint in sorted(scan_text(text)):
            print(f"{path.relative_to(root)}:{line}: {name} — {hint}")
            violations += 1
    if violations:
        print(f"\n{violations} deprecated API use(s) outside js/gb-shared.js.")
        return 1
    print("no deprecated web API usage")
    return 0


def list_sites(root: Path) -> int:
    total = 0
    for path in candidates(root):
        try:
            text = path.read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        for line, name, _hint in sorted(scan_text(text)):
            print(f"{path.relative_to(root)}:{line}: {name}")
            total += 1
    print(f"\n{total} site(s)")
    return 0


def self_test() -> int:
    bad = [
        'win.document.write("<p>hi</p>");',
        'var s = value.substr(2, 4);',
        'document.execCommand("bold");',
    ]
    good = [
        'var s = value.slice(2, 6);',
        'gbFormat("bold");',
        'gbPrintHtml(html);',
    ]
    failures = 0
    for line in bad:
        if not list(scan_text(line)):
            print(f"self-test: missed {line}")
            failures += 1
    for line in good:
        if list(scan_text(line)):
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
