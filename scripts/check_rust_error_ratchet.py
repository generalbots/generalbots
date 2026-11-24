#!/usr/bin/env python3
"""Ratchet guard for Rust error-handling debt (issue #2543).

AGENTS.md forbids ``unwrap()``, ``expect()``, ``panic!()`` and silent error
masking in production Rust. The tree violates this at scale (1,319
unwrap/expect, 2,315 unwrap_or_default, 256 swallowed Results per the issue),
so a one-shot fix is not realistic. This guard makes the *direction* of change
executable: the counts may stay equal or fall, never rise.

Counts are textual proxies only -- the guard never invokes cargo or clippy.
The ceiling lives in ``botserver/clippy-baseline.toml``; the ``[rules]`` totals
are enforced, the per-file hotspot snapshot (top 50 files) is informational:
it lets a reviewer find hotspots without making file moves fail the ratchet.

Counted rules in non-test ``.rs`` files under ``botserver/src`` and
``botserver/crates``:
  unwrap              -- ``.unwrap(`` calls
  expect              -- ``.expect(`` calls
  panic               -- ``panic!`` macro invocations
  unwrap_or_default   -- ``.unwrap_or_default(`` calls (Result *and* Option)
  discarded_result    -- ``.ok();`` statements and ``let _ = ...;`` bindings

Heuristic limits (deliberately documented, not hidden):
  * Comments are stripped before counting; ``//`` inside a string literal
    truncates the rest of that line, so a call after a URL-like string can be
    missed (false negative). It never invents a call.
  * ``#[cfg(test)]`` items are removed by brace matching; ``cfg_attr(test,
    ...)``, macro-generated code and test-only code behind a custom feature
    are not recognised (false positives).
  * Files under ``tests/`` / ``test/`` / ``testdata/`` and ``*_test.rs`` /
    ``*_tests.rs`` are skipped by path; a test helper in a production module
    without ``#[cfg(test)]`` is counted (false positive).
  * ``unwrap_or_default`` cannot be typed by a regex: Option calls are counted
    too (false positives), while ``unwrap_or_else(Default::default)`` and
    ``unwrap_or_default`` behind an alias are missed (false negatives).
  * ``discarded_result`` counts every ``let _ = ...;`` binding, including
    non-Result values, and de-duplicates ``let _ = x.ok();`` on one source line
    so it counts once. Multi-line statements may be miscounted.
  * A file rename or a change in formatting can move counts between rules;
    only the totals are gated, so such churn cannot raise the ceiling.

Usage:
    python3 scripts/check_rust_error_ratchet.py
    python3 scripts/check_rust_error_ratchet.py --update
    python3 scripts/check_rust_error_ratchet.py --self-test
    python3 scripts/check_rust_error_ratchet.py --check-baseline-diff --base <ref>

The history mode is the integrity half of the ratchet. --update refuses raises
only relative to the baseline file it reads, so hand-raising a ceiling in that
committed file would otherwise silently become the accepted normal. History
mode compares every shrink-only ceiling table ([rules] and [clippy]) in the
working-tree baseline with the same file at <ref> (or with the offline copy
named by GB_RATCHET_BASE_FILE) and fails when any ceiling increased. Keys added
or removed between the two revisions are reported, not failed.

The only unwrap/expect/panic text in this file is inside the self-test fixture
string; no such call is ever executed by the guard.

Exit codes: 0 = at/below baseline, 1 = growth, 2 = usage/baseline error.
"""
from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
import tomllib

ROOT = os.path.abspath(
    os.environ.get("GB_RATCHET_ROOT") or os.path.join(os.path.dirname(__file__), "..")
)
DEFAULT_BASELINE = os.path.join(ROOT, "botserver", "clippy-baseline.toml")
SCAN_DIRS = ("botserver/src", "botserver/crates")
SKIP_DIRS = {
    ".git", ".venv", "__pycache__", "node_modules", "target", "vendor", "work",
    "test", "tests", "testdata", "benches",
}
SKIP_FILE_STEMS = {"test", "tests"}
RULES = ("unwrap", "expect", "panic", "unwrap_or_default", "discarded_result")
# Every shrink-only ceiling table in clippy-baseline.toml. [files.*] is an
# informational hotspot snapshot (counts move between files on a rename), so it
# is deliberately not a ceiling table and must never gate the history check.
CEILING_KEYS = {
    "rules": RULES,
    "clippy": ("unwrap_used", "expect_used", "panic"),
}
HOTSPOT_LIMIT = 50
RULE_NOTES = {
    "unwrap": "calls to .unwrap(...)",
    "expect": "calls to .expect(...)",
    "panic": "panic! macro invocations",
    "unwrap_or_default": ".unwrap_or_default(...) calls (Result and Option)",
    "discarded_result": ".ok(); statements and let _ = ...; bindings",
}

UNWRAP_RE = re.compile(r"\.unwrap\s*\(")
EXPECT_RE = re.compile(r"\.expect\s*\(")
PANIC_RE = re.compile(r"\bpanic\s*!\s*\(")
UNWRAP_OR_DEFAULT_RE = re.compile(r"\.unwrap_or_default\s*\(")
OK_CALL_RE = re.compile(r"\.ok\s*\(\s*\)\s*;")
LET_UNDERSCORE_RE = re.compile(r"\blet\s+_\s*=")
PATTERNS = {
    "unwrap": UNWRAP_RE,
    "expect": EXPECT_RE,
    "panic": PANIC_RE,
    "unwrap_or_default": UNWRAP_OR_DEFAULT_RE,
}
BLOCK_COMMENT_RE = re.compile(r"/\*.*?\*/", re.DOTALL)
CFG_TEST_RE = re.compile(r"#\[cfg\(\s*test\s*\)\]")

HEADER = """# Rust error-handling ratchet baseline (issue #2543).
# Shrink-only ceiling generated by scripts/check_rust_error_ratchet.py.
# [rules] counts are textual proxies in non-test Rust, NOT clippy diagnostics.
# [files] is a truncated top-50 hotspot snapshot; only [rules] totals are gated.
# Fix production sites, then run `python3 scripts/check_rust_error_ratchet.py --update`.
# Never raise a number by hand: the guard refuses growth.
"""


def strip_comments(text: str) -> str:
    """Remove // and /* */ comments, preserving line numbers.

    Lexical approximation: cuts at `//` even inside a string, which can hide
    a call after a URL but never invents one.
    """
    text = BLOCK_COMMENT_RE.sub(lambda match: "\n" * match.group(0).count("\n"), text)
    return "\n".join(line.split("//", 1)[0] for line in text.split("\n"))


def strip_cfg_test(text: str) -> str:
    """Remove `#[cfg(test)]` items by brace matching, preserving line numbers."""
    pieces: list[str] = []
    pos = 0
    for match in CFG_TEST_RE.finditer(text):
        if match.start() < pos:
            continue
        brace = text.find("{", match.end())
        semi = text.find(";", match.end())
        if brace == -1 or (semi != -1 and semi < brace):
            end = semi + 1 if semi != -1 else match.end()
        else:
            depth = 0
            end = len(text)
            for index in range(brace, len(text)):
                if text[index] == "{":
                    depth += 1
                elif text[index] == "}":
                    depth -= 1
                    if depth == 0:
                        end = index + 1
                        break
        pieces.append(text[pos:match.start()])
        pieces.append("\n" * text[match.start():end].count("\n"))
        pos = end
    pieces.append(text[pos:])
    return "".join(pieces)


def is_production_path(rel: str) -> bool:
    """Best-effort path heuristic: skip tests directories and test file stems."""
    parts = rel.split("/")
    if any(part in {"tests", "test", "testdata", "benches"} for part in parts[:-1]):
        return False
    stem = os.path.splitext(parts[-1])[0]
    return stem not in SKIP_FILE_STEMS and not stem.endswith(("_test", "_tests"))


def count_text(text: str) -> dict[str, int]:
    """Count all rules on one (already cleaned) source string."""
    cleaned = strip_cfg_test(strip_comments(text))
    counts = {rule: 0 for rule in RULES}
    for rule, pattern in PATTERNS.items():
        counts[rule] = len(pattern.findall(cleaned))
    for line in cleaned.split("\n"):
        if LET_UNDERSCORE_RE.search(line):
            counts["discarded_result"] += 1
        else:
            counts["discarded_result"] += len(OK_CALL_RE.findall(line))
    return counts


def count_sites() -> tuple[dict[str, int], dict[str, dict[str, int]]]:
    """Return (totals, per_file) over the first-party scan scope."""
    totals = {rule: 0 for rule in RULES}
    per_file: dict[str, dict[str, int]] = {}
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
                if not is_production_path(rel):
                    continue
                try:
                    with open(path, "r", encoding="utf-8", errors="replace") as handle:
                        counts = count_text(handle.read())
                except OSError as error:
                    print(f"check_rust_error_ratchet: cannot read {rel}: {error}", file=sys.stderr)
                    continue
                if sum(counts.values()) == 0:
                    continue
                per_file[rel] = counts
                for rule in RULES:
                    totals[rule] += counts[rule]
    return totals, per_file


def growth_rules(current: dict[str, int], ceilings: dict[str, int]) -> list[str]:
    """Rules whose current count exceeds its ceiling (pure, shrink-only)."""
    return [rule for rule in RULES if current[rule] > ceilings[rule]]


def load_baseline(path: str) -> dict | None:
    """Parse the TOML ceiling, or return None after printing a reason."""
    try:
        with open(path, "rb") as handle:
            data = tomllib.load(handle)
    except (OSError, tomllib.TOMLDecodeError) as error:
        print(f"check_rust_error_ratchet: cannot load baseline {path}: {error}", file=sys.stderr)
        return None
    rules = data.get("rules")
    if not isinstance(rules, dict):
        print(f"check_rust_error_ratchet: baseline {path} has no [rules] table", file=sys.stderr)
        return None
    totals: dict[str, int] = {}
    for rule in RULES:
        value = rules.get(rule)
        if not isinstance(value, int) or isinstance(value, bool) or value < 0:
            print(
                f"check_rust_error_ratchet: baseline {path} needs a non-negative "
                f"integer rules.{rule}",
                file=sys.stderr,
            )
            return None
        totals[rule] = value
    return {"totals": totals, "per_file": data.get("files", {})}


from ratchet_baseline_history import (  # noqa: E402
    parse_ceilings, read_git_file, ceiling_raises, check_baseline_history, self_test_history,
)


def write_baseline(path: str, totals: dict[str, int], per_file: dict[str, dict[str, int]]) -> None:
    """Write a deterministic, shrink-only baseline with per-file hotspots."""
    lines = [HEADER.rstrip("\n"), "", "issue = 2543", "scope = [", ]
    for scan_dir in SCAN_DIRS:
        lines.append(f'    "{scan_dir}",')
    lines.extend(["]", "", "[rules]"])
    for rule in RULES:
        lines.append(f"{rule} = {totals[rule]}")
    lines.extend(["", "[rule_notes]"])
    for rule in RULES:
        lines.append(f"{rule} = {json.dumps(RULE_NOTES[rule])}")
    ranked = sorted(per_file.items(), key=lambda item: (-sum(item[1].values()), item[0]))
    for rel, counts in ranked[:HOTSPOT_LIMIT]:
        lines.extend(["", f"[files.{json.dumps(rel)}]"])
        for rule in RULES:
            if counts[rule]:
                lines.append(f"{rule} = {counts[rule]}")
    parent = os.path.dirname(path)
    if parent:
        os.makedirs(parent, exist_ok=True)
    with open(path, "w", encoding="utf-8") as handle:
        handle.write("\n".join(lines) + "\n")


def show_top(per_file: dict[str, dict[str, int]], rule: str, limit: int = 10) -> None:
    ranked = sorted(per_file.items(), key=lambda item: (-item[1][rule], item[0]))
    for rel, counts in ranked[:limit]:
        if counts[rule]:
            print(f"  {counts[rule]:4d}  {rel}")


def self_test() -> int:
    """Prove counting, exclusion, shrink-only and history logic on synthetic input."""
    failures: list[str] = []
    checks: list[str] = []

    def check(name: str, condition: bool) -> None:
        checks.append(name)
        if condition:
            print(f"  ok: {name}")
        else:
            print(f"  FAIL: {name}")
            failures.append(name)

    sample = r'''
// documentation only: thing.unwrap(); panic!("doc"); let _ = ignored();
#[cfg(test)]
mod tests {
    #[test]
    fn covered() {
        let inner: Result<u8, ()> = Ok(1);
        inner.unwrap();
        if true { let nested = 2; inner.unwrap(); }
    }
}
pub fn production() -> u8 {
    let a = read_db().unwrap();
    let b = other.expect("context");
    let c = fallback.unwrap_or_default();
    cleanup().ok();
    let _ = also_cleanup();
    let _ = settings().ok();
    panic!("boom");
    a + b + c
}
'''
    counts = count_text(sample)
    expected = {"unwrap": 1, "expect": 1, "panic": 1, "unwrap_or_default": 1, "discarded_result": 3}
    check(f"synthetic counts {counts}", counts == expected)
    check(
        "tests/ and *_test.rs paths are excluded",
        not is_production_path("botserver/crates/botcore/tests/it.rs")
        and not is_production_path("botserver/crates/botcore/src/foo_test.rs")
        and is_production_path("botserver/crates/botcore/src/foo.rs"),
    )
    parsed = tomllib.loads('[rules]\nunwrap = 4\nexpect = 3\npanic = 2\n'
                           'unwrap_or_default = 9\ndiscarded_result = 6\n')
    ceilings = {rule: parsed["rules"][rule] for rule in RULES}
    check("equal counts are not growth", growth_rules(ceilings.copy(), ceilings) == [])
    lowered = ceilings.copy()
    lowered["unwrap"] = 3
    check("lower count is not growth", growth_rules(lowered, ceilings) == [])
    raised = ceilings.copy()
    raised["expect"] = 4
    check("raised count is growth", growth_rules(raised, ceilings) == ["expect"])
    self_test_history(check)

    if failures:
        print(f"check_rust_error_ratchet: self-test FAIL — {len(failures)} check(s)")
        return 1
    print(f"check_rust_error_ratchet: self-test OK — {len(checks)} checks passed")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description="Rust error-handling ratchet (#2543)")
    parser.add_argument("--baseline", default=DEFAULT_BASELINE, help="baseline TOML path")
    parser.add_argument(
        "--update", action="store_true",
        help="lower the ceiling to the current counts (refuses raises)",
    )
    parser.add_argument(
        "--self-test", action="store_true",
        help="check counting and shrink-only logic on synthetic strings",
    )
    parser.add_argument(
        "--check-baseline-diff", action="store_true",
        help="fail if a committed ceiling rose relative to the base revision",
    )
    parser.add_argument(
        "--base", default=None,
        help="git ref to compare the baseline against (or GB_RATCHET_BASE)",
    )
    args = parser.parse_args()

    if args.self_test:
        return self_test()

    if args.check_baseline_diff:
        return check_baseline_history(args.baseline, args.base or os.environ.get("GB_RATCHET_BASE"))

    try:
        totals, per_file = count_sites()
    except OSError as error:
        print(f"check_rust_error_ratchet: scan failed: {error}", file=sys.stderr)
        return 2
    baseline = load_baseline(args.baseline)
    if baseline is None:
        return 2
    ceilings = baseline["totals"]
    grown = growth_rules(totals, ceilings)
    rel = os.path.relpath(args.baseline, ROOT)

    if args.update:
        if grown:
            for rule in grown:
                print(
                    f"check_rust_error_ratchet: refusing to raise {rule} "
                    f"({totals[rule]} > baseline {ceilings[rule]}); fix the growth first"
                )
            return 1
        write_baseline(args.baseline, totals, per_file)
        summary = ", ".join(f"{rule}: {ceilings[rule]}->{totals[rule]}" for rule in RULES)
        print(f"check_rust_error_ratchet: baseline written to {rel} ({summary})")
        return 0

    if grown:
        print(f"check_rust_error_ratchet: FAIL — {len(grown)} rule(s) grew against {rel}:")
        for rule in grown:
            print(
                f"- {rule}: {totals[rule]} > baseline {ceilings[rule]} "
                f"(+{totals[rule] - ceilings[rule]})"
            )
            show_top(per_file, rule)
        print("  remediation: propagate with ? / typed errors, log discarded Results")
        return 1

    drops = [f"{rule}: {totals[rule]} < baseline {ceilings[rule]}"
             for rule in RULES if totals[rule] < ceilings[rule]]
    summary = ", ".join(f"{rule}={totals[rule]}" for rule in RULES)
    print(f"check_rust_error_ratchet: OK — {summary} at/below {rel}")
    for line in drops:
        print(f"  note: {line} — lower it with --update (counts only move down)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
