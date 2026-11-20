#!/usr/bin/env python3
"""console.log guard (issue #2304).

Suite production JS must not ship direct console.log calls (debug noise /
internal leakage). Use gbLog.debug(...) (gated by gb-debug/GB_DEBUG) or
gbLog.info/warn/error as appropriate; console.warn/error remain fine.

Usage:
    python3 scripts/check_console_log.py --check
    python3 scripts/check_console_log.py --list
    python3 scripts/check_console_log.py --self-test
    python3 scripts/check_console_log.py --root <repo>

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
# `tests/` holds Node/vm test harnesses (e.g. webtest-apps/00_harness.js) whose
# only way to report results is console.log/process.stdout — they are not
# shipped as production suite JS. Matches the i18n guard's EXCLUDED_PARTS.
SKIP_SEGMENTS = ("vendor", "node_modules", "tests")
SKIP_SUFFIXES = (".orig",)
ALLOWED_FILES = {
    Path("botui/ui/suite/js/gb-shared.js"),  # defines gbLog.debug wrapper
}

CONSOLE_LOG = re.compile(r"\bconsole\.log\s*\(")


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
        if any(str(path).endswith(s) for s in SKIP_SUFFIXES):
            continue
        files.append(path)
    return sorted(files)


def is_allowed(root: Path, path: Path) -> bool:
    try:
        rel = path.relative_to(root)
    except ValueError:
        rel = path
    return rel in ALLOWED_FILES


def scan_text(text: str) -> list[int]:
    return [text.count("\n", 0, m.start()) + 1 for m in CONSOLE_LOG.finditer(text)]


def check(root: Path) -> int:
    violations = 0
    for path in candidates(root):
        if is_allowed(root, path):
            continue
        try:
            text = path.read_text(encoding="utf-8", errors="replace")
        except OSError as exc:
            print(f"warning: cannot read {path}: {exc}", file=sys.stderr)
            continue
        for line in scan_text(text):
            print(f"{path.relative_to(root)}:{line}: console.log call; use gbLog.debug/warn/error")
            violations += 1
    if violations:
        print(f"\n{violations} console.log call(s); use gbLog.debug (GB_DEBUG-gated) or gbLog.warn/error.")
        return 1
    print("no direct console.log calls in suite JS (outside vendor/.orig/wrapper)")
    return 0


def list_sites(root: Path) -> int:
    total = 0
    for path in candidates(root):
        if is_allowed(root, path):
            continue
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
        'console.log("x")',
        'console.log (a,b)',
        'if (x) console.log(x);',
    ]
    good = [
        'gbLog.debug("x")',
        'console.warn("x")',
        'console.error("x")',
        '// console.log mentioned in text',
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
    parser.add_argument("--root", type=Path, default=ROOT)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--check", action="store_true", default=True)
    mode.add_argument("--list", action="store_true")
    mode.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    root = args.root.resolve()
    if args.self_test:
        return self_test()
    if args.list:
        return list_sites(root)
    return check(root)


if __name__ == "__main__":
    raise SystemExit(main())
