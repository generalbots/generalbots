#!/usr/bin/env python3
"""Locale/date/time/number raw-formatting guard (issue #2329).

Unprefixed/raw locale-formatting calls render per browser/collaborator locale
instead of the suite locale contract. Suite code must use centralized helpers
(`gbFormatDate/Time/DateTime/Number/Currency` or i18n-format helpers).

This gate fails on raw `.toLocaleDateString(`, `.toLocaleTimeString(`, or
`.toLocaleString(` in suite UI files outside formatter modules, vendor, tests
and node_modules.

Usage:
    python3 scripts/check_locale_format_calls.py --check
    python3 scripts/check_locale_format_calls.py --list
    python3 scripts/check_locale_format_calls.py --self-test
    python3 scripts/check_locale_format_calls.py --root <repo>

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
SKIP_SEGMENTS = ("vendor", "node_modules", "tests")
ALLOWED_FILES = {
    Path("botui/ui/suite/js/gb-shared.js"),
    Path("botui/ui/suite/js/gb-datetime.js"),
    Path("botui/ui/suite/js/i18n-format.js"),
}

RAW_CALL = re.compile(r"\.toLocale(?:Date|Time)?String\s*\(")


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


def is_allowed(root: Path, path: Path) -> bool:
    try:
        rel = path.relative_to(root)
    except ValueError:
        rel = path
    return rel in ALLOWED_FILES


def scan_text(text: str) -> list[int]:
    return [text.count("\n", 0, m.start()) + 1 for m in RAW_CALL.finditer(text)]


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
            print(f"{path.relative_to(root)}:{line}: raw locale-format call; use gbFormatDate/Time/DateTime/Number/Currency")
            violations += 1
    if violations:
        print(f"\n{violations} raw locale-format call(s); centralize in formatter helpers/modules.")
        return 1
    print("no raw locale-format calls outside formatter modules/tests/vendor")
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
        'd.toLocaleDateString()',
        'd.toLocaleString("pt-BR")',
        'now.toLocaleTimeString([], { hour: "2-digit" })',
        'n.toLocaleString(undefined, { style: "currency" })',
    ]
    good = [
        'gbFormatDate(d)',
        'gbFormatDateTime(d, { locale: "pt-BR" })',
        'Intl.DateTimeFormat("en", opts).format(d)',
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
