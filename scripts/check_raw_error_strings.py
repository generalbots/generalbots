#!/usr/bin/env python3
"""Ratchet guard for raw ``(StatusCode, String)`` error bodies (issue #2414).

Three quarters of the HTTP surface used to return a bare ``String`` body, so
internal DB/IO text leaked to clients and the suite had two error shapes. The
sanitised envelope is ``botsecurity_core::error_sanitizer::SafeErrorResponse``
(``error`` / ``message`` / ``code`` / ``request_id``); migrating a handler away
from a raw tuple is what lowers this count.

The count mirrors the evidence command in the issue exactly:

    grep -rn '(StatusCode, String)' --include=*.rs botserver/src botserver/crates | wc -l

This guard makes that number executable. ``issues/error-strings-baseline.json``
is a *ceiling*, not a snapshot: the count may stay equal or fall, never rise.
When a change removes sites, lower the ceiling in the same change (or run
``--update``), so the budget only ever goes down.

Usage:
    python3 scripts/check_raw_error_strings.py
    python3 scripts/check_raw_error_strings.py --update

Exit codes: 0 = at/below baseline, 1 = growth, 2 = usage/baseline error.
"""
from __future__ import annotations

import argparse
import json
import os
import sys

ROOT = os.path.abspath(
    os.environ.get("GB_ERROR_STRINGS_ROOT") or os.path.join(os.path.dirname(__file__), "..")
)
DEFAULT_BASELINE = os.path.join(ROOT, "issues", "error-strings-baseline.json")
SCAN_DIRS = ("botserver/src", "botserver/crates")
PATTERN = "(StatusCode, String)"
SKIP_DIRS = {".git", ".venv", "__pycache__", "node_modules", "target", "vendor", "work"}


def count_sites() -> tuple[int, dict[str, int]]:
    """Return (total, per_file) for the exact tuple type under SCAN_DIRS."""
    per_file: dict[str, int] = {}
    for scan_dir in SCAN_DIRS:
        base = os.path.join(ROOT, scan_dir)
        if not os.path.isdir(base):
            continue
        for dirpath, dirnames, filenames in os.walk(base):
            dirnames[:] = sorted(name for name in dirnames if name not in SKIP_DIRS)
            for name in sorted(filenames):
                if not name.endswith(".rs"):
                    continue
                path = os.path.join(dirpath, name)
                rel = os.path.relpath(path, ROOT).replace(os.sep, "/")
                with open(path, "r", encoding="utf-8", errors="replace") as handle:
                    count = handle.read().count(PATTERN)
                if count:
                    per_file[rel] = count
    return sum(per_file.values()), per_file


def load_ceiling(path: str) -> int:
    try:
        with open(path, "r", encoding="utf-8") as handle:
            data = json.load(handle)
    except (OSError, json.JSONDecodeError) as error:
        print(f"check_raw_error_strings: cannot load baseline {path}: {error}", file=sys.stderr)
        return -1
    ceiling = data.get("total") if isinstance(data, dict) else None
    if not isinstance(ceiling, int) or isinstance(ceiling, bool) or ceiling < 0:
        print(f"check_raw_error_strings: baseline {path} has no non-negative integer 'total'", file=sys.stderr)
        return -1
    return ceiling


def write_baseline(path: str, total: int, per_file: dict[str, int]) -> None:
    data = {
        "issue": 2414,
        "pattern": PATTERN,
        "scope": list(SCAN_DIRS),
        "total": total,
        "per_file": dict(sorted(per_file.items(), key=lambda item: (-item[1], item[0]))),
    }
    parent = os.path.dirname(path)
    if parent:
        os.makedirs(parent, exist_ok=True)
    with open(path, "w", encoding="utf-8") as handle:
        json.dump(data, handle, indent=2, ensure_ascii=False)
        handle.write("\n")


def main() -> int:
    parser = argparse.ArgumentParser(description="Raw error-string budget guard (#2414)")
    parser.add_argument("--baseline", default=DEFAULT_BASELINE, help="baseline JSON path")
    parser.add_argument(
        "--update",
        action="store_true",
        help="lower the ceiling to the current count (never raises it)",
    )
    args = parser.parse_args()

    try:
        total, per_file = count_sites()
    except OSError as error:
        print(f"check_raw_error_strings: scan failed: {error}", file=sys.stderr)
        return 2
    ceiling = load_ceiling(args.baseline)
    if ceiling < 0:
        return 2

    if args.update:
        if total > ceiling:
            print(
                f"check_raw_error_strings: refusing to raise the ceiling "
                f"({total} > baseline {ceiling}); fix the growth first"
            )
            return 1
        write_baseline(args.baseline, total, per_file)
        print(
            f"check_raw_error_strings: baseline lowered to {total} "
            f"(was {ceiling}) at {os.path.relpath(args.baseline, ROOT)}"
        )
        return 0

    if total > ceiling:
        print(
            f"check_raw_error_strings: FAIL — {total} raw error bodies > ceiling "
            f"{ceiling} (+{total - ceiling}) in {os.path.relpath(args.baseline, ROOT)}"
        )
        for path, count in sorted(per_file.items(), key=lambda item: (-item[1], item[0]))[:20]:
            print(f"  {count:4d}  {path}")
        print("  remediation: return SafeErrorResponse via botsecurity_core::error_sanitizer")
        return 1

    note = ""
    if total < ceiling:
        note = f" ({ceiling - total} fewer; lower the ceiling with --update)"
    print(
        f"check_raw_error_strings: OK — {total} raw error bodies at/below ceiling "
        f"{ceiling}{note}"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
