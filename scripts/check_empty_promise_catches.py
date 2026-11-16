#!/usr/bin/env python3
"""Empty promise rejection handler gate (issue #2333).

Silent empty `.catch(...) {}` handlers discard promise rejections with no log,
no UI feedback, and no recovery. #2301 covers empty `catch {}` blocks and
floating fetches; this gate covers the promise-shaped form so the silent-failure
sweep stays complete.

The gate scans suite JavaScript (static files and JS template contexts alike,
since they share the same source form) and fails when a handler body is empty.
Best-effort suppressions must carry a comment or log and therefore are not
empty; intentional handling must do real work.

Usage
-----
    python3 scripts/check_empty_promise_catches.py --check
    python3 scripts/check_empty_promise_catches.py --list
    python3 scripts/check_empty_promise_catches.py --self-test
    python3 scripts/check_empty_promise_catches.py --root <repo>

Exit codes: 0 clean, 1 violations found.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SCAN_ROOT = Path("botui/ui/suite")
SUFFIXES = (".js",)
# Third-party bundles are not ours to edit and may contain minified patterns.
SKIP_SEGMENTS = ("vendor", "node_modules")

# Matches: .catch(function (...) { })  .catch((e) => {})  .catch(e => {})
# Bodies containing only whitespace are treated as empty. A comment inside the
# body makes the handler non-empty, which is the required form for provably
# best-effort suppressions (plus a debug log in code).
EMPTY_CATCH = re.compile(
    r"\.catch\(\s*"
    r"(?:function\s*\([^)]*\)|\([^)]*\)|[A-Za-z_$]\w*)"
    r"\s*(?:=>)?\s*"
    r"\{\s*\}\s*\)"
)


def line_of(text: str, index: int) -> int:
    return text.count("\n", 0, index) + 1


def scan_text(text: str) -> list[tuple[int, str]]:
    found = []
    for m in EMPTY_CATCH.finditer(text):
        snippet = " ".join(m.group(0).split())[:140]
        found.append((line_of(text, m.start()), snippet))
    return found


def scan_file(path: Path) -> list[tuple[int, str]]:
    try:
        text = path.read_text(encoding="utf-8", errors="replace")
    except OSError as exc:
        print(f"warning: cannot read {path}: {exc}", file=sys.stderr)
        return []
    return scan_text(text)


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
        if path.name.endswith(".orig"):
            continue
        files.append(path)
    return sorted(files)


def check(root: Path) -> int:
    violations = 0
    for path in candidates(root):
        for line, snippet in scan_file(path):
            rel = path.relative_to(root)
            print(f"{rel}:{line}: empty promise .catch handler\n    {snippet}")
            violations += 1
    if violations:
        print(f"\n{violations} empty promise .catch handler(s) found.")
        return 1
    print("no empty promise .catch handlers")
    return 0


def list_sites(root: Path) -> int:
    total = 0
    for path in candidates(root):
        for line, snippet in scan_file(path):
            print(f"{path.relative_to(root)}:{line}: {snippet}")
            total += 1
    print(f"\n{total} site(s)")
    return 0


def self_test() -> int:
    good = [
        "promise.catch(function (err) { console.debug(err); });",
        "promise.catch((e) => { /* best-effort */ log(e); })",
        "promise.catch(e => { handle(e); })",
        "promise.catch(function () { /* intentional: documented */ console.warn('x'); })",
    ]
    bad = [
        "promise.catch(function () {})",
        "promise.catch(() => {})",
        "promise.catch((e) => { })",
        "promise.catch(function (err) {   })",
        "promise.catch(err => {})",
    ]
    failures = 0
    for sample in good:
        if scan_text(sample):
            print(f"self-test: false positive on {sample}")
            failures += 1
    for sample in bad:
        if not scan_text(sample):
            print(f"self-test: missed violation in {sample}")
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
