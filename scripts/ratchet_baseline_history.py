#!/usr/bin/env python3
"""Baseline-history helpers for the #2543 ratchet guard (split for <450-line rule).

Verifies that shrink-only ceiling tables in botserver/clippy-baseline.toml never
grow between the base ref and the working tree, so a hand-raised ceiling fails
CI even though --update only refuses raises relative to the file it reads.
"""
from __future__ import annotations

import os
import subprocess
import tomllib
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BASELINE_FILE = os.path.join(ROOT, "botserver", "clippy-baseline.toml")
CEILING_KEYS = {
    "rules": ("unwrap", "expect", "panic", "unwrap_or_default", "discarded_result"),
    "clippy": ("unwrap_used", "expect_used", "panic"),
}


def parse_ceilings(text: str, source: str) -> dict[tuple[str, str], int] | None:
    """Parse every shrink-only ceiling table, or None after printing a reason.

    Returns {(table, key): value}. A table or key absent from an older snapshot
    contributes no entry (the caller reports it as added/removed, never as a
    raise). Completeness of the current baseline is already enforced by
    load_baseline() and check_clippy_baseline.py; present values must still be
    non-negative integers so a malformed value fails closed.
    """
    try:
        data = tomllib.loads(text)
    except tomllib.TOMLDecodeError as error:
        print(f"check_rust_error_ratchet: cannot parse {source}: {error}", file=sys.stderr)
        return None
    if not isinstance(data, dict):
        print(f"check_rust_error_ratchet: {source} is not a TOML table", file=sys.stderr)
        return None
    ceilings: dict[tuple[str, str], int] = {}
    for table, keys in CEILING_KEYS.items():
        section = data.get(table)
        if section is None:
            continue
        if not isinstance(section, dict):
            print(f"check_rust_error_ratchet: {source} has a non-table [{table}]", file=sys.stderr)
            return None
        for key in keys:
            if key not in section:
                continue
            value = section[key]
            if not isinstance(value, int) or isinstance(value, bool) or value < 0:
                print(
                    f"check_rust_error_ratchet: {source} needs a non-negative "
                    f"integer {table}.{key}",
                    file=sys.stderr,
                )
                return None
            ceilings[(table, key)] = value
    if not ceilings:
        print(f"check_rust_error_ratchet: {source} has no ceiling tables", file=sys.stderr)
        return None
    return ceilings


def read_git_file(ref: str, rel_path: str) -> tuple[str | None, str | None]:
    """Return (content, None) or (None, reason) for ``git show ref:path``."""
    try:
        result = subprocess.run(
            ["git", "show", f"{ref}:{rel_path}"],
            cwd=ROOT, capture_output=True, text=True, check=False,
        )
    except OSError as error:
        return None, f"cannot run git: {error}"
    if result.returncode != 0:
        reason = result.stderr.strip() or result.stdout.strip() or "git show failed"
        return None, f"git show {ref}:{rel_path} failed: {reason}"
    return result.stdout, None


def ceiling_raises(before: dict[tuple[str, str], int],
                   after: dict[tuple[str, str], int]) -> list[tuple[str, str, int, int]]:
    """Pure shrink-only check: (table, key, before, after) for each increase."""
    return [
        (table, key, before[(table, key)], value)
        for (table, key), value in after.items()
        if (table, key) in before and value > before[(table, key)]
    ]


def check_baseline_history(baseline_path: str, base_ref: str | None) -> int:
    """Fail when a committed ceiling rose relative to the base revision."""
    try:
        with open(baseline_path, "r", encoding="utf-8") as handle:
            head_text = handle.read()
    except OSError as error:
        print(f"check_rust_error_ratchet: cannot read baseline {baseline_path}: {error}", file=sys.stderr)
        return 2
    head = parse_ceilings(head_text, baseline_path)
    if head is None:
        return 2

    base_file = os.environ.get("GB_RATCHET_BASE_FILE")
    if base_file:
        try:
            with open(base_file, "r", encoding="utf-8") as handle:
                base_text = handle.read()
        except OSError as error:
            print(f"check_rust_error_ratchet: cannot read base baseline {base_file}: {error}", file=sys.stderr)
            return 2
        base = parse_ceilings(base_text, base_file)
        if base is None:
            return 2
        source = base_file
    else:
        if not base_ref:
            print(
                "check_rust_error_ratchet: --check-baseline-diff needs --base "
                "or GB_RATCHET_BASE_FILE",
                file=sys.stderr,
            )
            return 2
        rel = os.path.relpath(baseline_path, ROOT).replace(os.sep, "/")
        base_text, error = read_git_file(base_ref, rel)
        if error is not None:
            print(f"check_rust_error_ratchet: {error}", file=sys.stderr)
            return 2
        if base_text is None:
            print(f"check_rust_error_ratchet: git returned no content for {rel}", file=sys.stderr)
            return 2
        base = parse_ceilings(base_text, f"{base_ref}:{rel}")
        if base is None:
            return 2
        source = f"{base_ref}:{rel}"

    raised = ceiling_raises(base, head)
    added = sorted(key for key in head if key not in base)
    removed = sorted(key for key in base if key not in head)
    for table, key in added:
        print(f"check_rust_error_ratchet: note: {table}.{key} added since {source}; not a raise")
    for table, key in removed:
        print(f"check_rust_error_ratchet: note: {table}.{key} removed since {source}")

    if raised:
        print(f"check_rust_error_ratchet: FAIL — {len(raised)} ceiling(s) raised against {source}:")
        for table, key, before, after in raised:
            print(f"- [{table}] {key}: {before} -> {after} (+{after - before})")
        print("  regenerate with --update; never raise a ceiling by hand")
        return 1
    print(f"check_rust_error_ratchet: OK — no ceiling raised against {source}")
    return 0


def self_test_history(check):
    """History-guard assertions, shared with the main guard's --self-test."""

    base_ceiling_text = (
        '[rules]\nunwrap = 0\nexpect = 0\npanic = 0\n'
        'unwrap_or_default = 9\n'
        '[clippy]\nunwrap_used = 0\nexpect_used = 0\npanic = 0\n'
    )
    raised_ceiling_text = base_ceiling_text.replace(
        "unwrap_or_default = 9", "unwrap_or_default = 10"
    ).replace("expect_used = 0", "expect_used = 1")
    lowered_ceiling_text = base_ceiling_text.replace(
        "unwrap_or_default = 9", "unwrap_or_default = 8"
    )
    base_ceilings = parse_ceilings(base_ceiling_text, "base")
    raised_ceilings = parse_ceilings(raised_ceiling_text, "raised")
    lowered_ceilings = parse_ceilings(lowered_ceiling_text, "lowered")
    check(
        "history guard detects raises in [rules] and [clippy]",
        base_ceilings is not None and raised_ceilings is not None
        and ceiling_raises(base_ceilings, raised_ceilings) == [
            ("rules", "unwrap_or_default", 9, 10),
            ("clippy", "expect_used", 0, 1),
        ],
    )
    check(
        "history guard accepts a lowered ceiling",
        base_ceilings is not None and lowered_ceilings is not None
        and ceiling_raises(base_ceilings, lowered_ceilings) == [],
    )
    added_ceiling_text = raised_ceiling_text.replace(
        "[clippy]", "discarded_result = 6\n[clippy]"
    )
    added_ceilings = parse_ceilings(added_ceiling_text, "added")
    check(
        "history guard reports an added key without treating it as a raise",
        base_ceilings is not None and added_ceilings is not None
        and ("rules", "discarded_result") in added_ceilings
        and ("rules", "discarded_result") not in base_ceilings
        and ceiling_raises(base_ceilings, added_ceilings) == [
            ("rules", "unwrap_or_default", 9, 10),
            ("clippy", "expect_used", 0, 1),
        ],
    )
