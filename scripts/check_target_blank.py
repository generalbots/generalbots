#!/usr/bin/env python3
"""Reverse-tabnabbing gate (issue #2327).

A page opened from `target="_blank"` without `rel="noopener"` keeps a live
`window.opener` handle back to the suite tab and can navigate it to a phishing
page; without `noreferrer` the destination also receives the suite URL in the
`Referer` header. Both are one attribute away, so a forgotten attribute is a
regression, not a style choice.

This gate scans every anchor the suite ships — static HTML and JS template
strings alike, because the tag text is the same in both — and fails when a
`target="_blank"` anchor lacks `noopener` (or `noreferrer`).

Usage
-----
    python3 scripts/check_target_blank.py --check
    python3 scripts/check_target_blank.py --list
    python3 scripts/check_target_blank.py --self-test
    python3 scripts/check_target_blank.py --root <repo>

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
# Third-party bundles are not ours to edit and never anchor to the suite.
SKIP_SEGMENTS = ("vendor", "node_modules")

ANCHOR = re.compile(r"<a\b[^>]*>", re.IGNORECASE | re.DOTALL)
TARGET_BLANK = re.compile(r"""target\s*=\s*["']_blank["']""", re.IGNORECASE)
REL = re.compile(r"""rel\s*=\s*["']([^"']*)["']""", re.IGNORECASE)


def iter_anchors(text: str):
    for m in ANCHOR.finditer(text):
        yield m.start(), m.group(0)


def check_anchor(tag: str) -> str | None:
    """Return a reason when the tag violates the rule, else None."""
    if not TARGET_BLANK.search(tag):
        return None
    rel_match = REL.search(tag)
    if rel_match is None:
        return "target=\"_blank\" without rel=\"noopener noreferrer\""
    tokens = rel_match.group(1).lower().split()
    missing = [t for t in ("noopener", "noreferrer") if t not in tokens]
    if missing:
        return f"rel lacks {' and '.join(missing)}"
    return None


def line_of(text: str, index: int) -> int:
    return text.count("\n", 0, index) + 1


def scan_file(path: Path) -> list[tuple[int, str, str]]:
    try:
        text = path.read_text(encoding="utf-8", errors="replace")
    except OSError as exc:
        print(f"warning: cannot read {path}: {exc}", file=sys.stderr)
        return []
    found = []
    for index, tag in iter_anchors(text):
        reason = check_anchor(tag)
        if reason:
            snippet = " ".join(tag.split())[:120]
            found.append((line_of(text, index), reason, snippet))
    return found


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


def check(root: Path) -> int:
    violations = 0
    for path in candidates(root):
        for line, reason, snippet in scan_file(path):
            rel = path.relative_to(root)
            print(f"{rel}:{line}: {reason}\n    {snippet}")
            violations += 1
    if violations:
        print(f"\n{violations} target=\"_blank\" anchor(s) missing noopener/noreferrer.")
        return 1
    print("no unguarded target=\"_blank\" anchors")
    return 0


def list_sites(root: Path) -> int:
    total = 0
    for path in candidates(root):
        for line, _reason, snippet in scan_file(path):
            print(f"{path.relative_to(root)}:{line}: {snippet}")
            total += 1
    print(f"\n{total} site(s)")
    return 0


def self_test() -> int:
    good = [
        '<a href="https://x.test" target="_blank" rel="noopener noreferrer">ok</a>',
        '<a href="/metrics" target="_blank" rel="noreferrer noopener">ok</a>',
        '<a href="/internal">same tab</a>',
    ]
    bad = [
        '<a href="https://x.test" target="_blank">no rel</a>',
        "<a href='https://x.test' target='_blank' rel='noopener'>no noreferrer</a>",
        '<a href="${appUrl}" target="_blank" style="color:red">js template</a>',
    ]
    failures = 0
    for tag in good:
        if check_anchor(tag) is not None:
            print(f"self-test: false positive on {tag}")
            failures += 1
    for tag in bad:
        if check_anchor(tag) is None:
            print(f"self-test: missed violation in {tag}")
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
