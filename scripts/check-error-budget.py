#!/usr/bin/env python3
"""Deterministic error-budget scanner for issue #2543 (offline, no cargo).

Counts textual proxies for error-handling debt in *production* Rust under a
bounded set of first-party crates:

  unwrap             -- ``.unwrap(`` calls
  expect             -- ``.expect(`` calls
  panic              -- ``panic!`` macro invocations
  unwrap_or_default  -- ``.unwrap_or_default(`` calls (Result *and* Option)
  allow              -- ``#[allow(...)]`` lint-suppression attributes

`#[cfg(test)]` items and test files/directories are excluded. Comments are
stripped first. The tool never invokes cargo/clippy; it is a stable proxy that
feeds the Rust-side ``Budget::evaluate`` ratchet (botcore::error_budget).

Emits two deterministic JSON files (no timestamps, sorted keys) under OUT_DIR
(default ``/tmp/opencode/error-budget``):

  error-budget-report.json    per-crate counts + totals for this run
  error-budget-baseline.json  shrink-only ceiling (first run / --update-baseline)

Exit codes: 0 = at/below baseline, 1 = a rule grew (regression), 2 = usage error.

Heuristic limits (documented, deliberately not hidden):
  * ``//`` inside a string literal truncates the rest of the line (false
    negative); a call is never invented.
  * ``#[cfg(test)]`` blocks are removed by brace matching; ``cfg_attr(test,
    ...)`` and macro-generated test code are not recognised (false positive).
  * Only method-call form ``.unwrap(`` is counted; free ``unwrap(...)`` calls
    and function definitions are ignored (false negative).
  * ``#[allow`` counts every suppression attribute, including ones that are
    legitimately scoped; the point is to keep the *count* from growing.
"""
from __future__ import annotations

import argparse
import json
import os
import re
import sys

DEFAULT_CRATES = (
    "botcore",
    "botlog",
    "botsettings",
    "botcoresecrets",
    "botsecurity-crypto",
    "botbasic_core",
)
MAX_CRATES = 32
RULES = ("unwrap", "expect", "panic", "unwrap_or_default", "allow")
RULE_NOTES = {
    "unwrap": "'.unwrap(' calls",
    "expect": "'.expect(' calls",
    "panic": "'panic!' macro invocations",
    "unwrap_or_default": "'.unwrap_or_default(' calls (Result and Option)",
    "allow": "'#[allow(...)]' lint-suppression attributes",
}

CRATES_ROOT = "botserver/crates"
SKIP_DIRS = {
    ".git", ".venv", "__pycache__", "node_modules", "target", "vendor",
    "work", "test", "tests", "testdata", "benches",
}
SKIP_FILE_STEMS = {"test", "tests"}

PATTERNS = {
    "unwrap": re.compile(r"\.unwrap\s*\("),
    "expect": re.compile(r"\.expect\s*\("),
    "panic": re.compile(r"\bpanic\s*!\s*\("),
    "unwrap_or_default": re.compile(r"\.unwrap_or_default\s*\("),
    "allow": re.compile(r"#\s*\[\s*allow\b"),
}
BLOCK_COMMENT_RE = re.compile(r"/\*.*?\*/", re.DOTALL)
CFG_TEST_RE = re.compile(r"#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]")

ROOT = os.path.abspath(
    os.environ.get("GB_ERROR_BUDGET_ROOT")
    or os.path.join(os.path.dirname(os.path.abspath(__file__)), "..")
)


def strip_comments(text: str) -> str:
    """Remove // and /* */ comments while preserving line numbers."""
    text = BLOCK_COMMENT_RE.sub(
        lambda m: "\n" * m.group(0).count("\n"), text
    )
    return "\n".join(line.split("//", 1)[0] for line in text.split("\n"))


def strip_cfg_test(text: str) -> str:
    """Blank out ``#[cfg(test)]`` items by brace matching, keeping line numbers."""
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
    """Skip test directories and test-named files."""
    parts = rel.split("/")
    if any(part in {"tests", "test", "testdata", "benches"} for part in parts[:-1]):
        return False
    stem = os.path.splitext(parts[-1])[0]
    return stem not in SKIP_FILE_STEMS and not stem.endswith(("_test", "_tests"))


def count_text(text: str) -> dict[str, int]:
    """Count every rule on one (already cleaned) source string."""
    cleaned = strip_cfg_test(strip_comments(text))
    return {rule: len(pattern.findall(cleaned)) for rule, pattern in PATTERNS.items()}


def scan_crate(crate: str) -> dict[str, int]:
    """Aggregate counts over the production .rs files of a single crate."""
    counts = {rule: 0 for rule in RULES}
    base = os.path.join(ROOT, CRATES_ROOT, crate)
    if not os.path.isdir(base):
        print(f"check-error-budget: crate not found: {crate}", file=sys.stderr)
        return counts
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
                    file_counts = count_text(handle.read())
            except OSError as error:
                print(f"check-error-budget: cannot read {rel}: {error}", file=sys.stderr)
                continue
            for rule in RULES:
                counts[rule] += file_counts[rule]
    return counts


def totals_of(report: dict[str, dict[str, int]]) -> dict[str, int]:
    """Sum per-crate counts into a single totals dict."""
    totals = {rule: 0 for rule in RULES}
    for counts in report.values():
        for rule in RULES:
            totals[rule] += counts[rule]
    return totals


def grown_rules(current: dict[str, int], ceiling: dict[str, int]) -> list[str]:
    """Rules whose current count exceeds the ceiling (pure, shrink-only)."""
    return [rule for rule in RULES if current[rule] > ceiling[rule]]


def write_json(path: str, payload: dict) -> None:
    """Write deterministic JSON (sorted keys, trailing newline)."""
    with open(path, "w", encoding="utf-8") as handle:
        json.dump(payload, handle, sort_keys=True, indent=2)
        handle.write("\n")


def load_json(path: str) -> dict | None:
    """Load a JSON file, returning None when absent or corrupt."""
    if not os.path.isfile(path):
        return None
    try:
        with open(path, "r", encoding="utf-8") as handle:
            return json.load(handle)
    except (OSError, json.JSONDecodeError) as error:
        print(f"check-error-budget: cannot load {path}: {error}", file=sys.stderr)
        return None


def clamp_baseline(current: dict, previous: dict) -> dict:
    """Produce a shrink-only baseline: every value is min(current, previous)."""
    result = {"schema": 1, "crates": {}, "totals": {}}
    current_crates = current.get("crates", {})
    prev_crates = previous.get("crates", {})
    for crate, counts in current_crates.items():
        prev = prev_crates.get(crate, counts)
        result["crates"][crate] = {
            rule: min(counts[rule], prev.get(rule, counts[rule])) for rule in RULES
        }
    prev_totals = previous.get("totals", {})
    result["totals"] = {
        rule: min(current["totals"][rule], prev_totals.get(rule, current["totals"][rule]))
        for rule in RULES
    }
    return result


def self_test() -> int:
    """Synthetic checks for the counting heuristics; returns exit code."""
    sample = (
        "fn ok() { let _ = value.unwrap(); }\n"
        "fn bad() { value.expect(\"x\"); panic!(\"x\"); }\n"
        "fn d() { let _ = opt.unwrap_or_default(); #[allow(dead_code)] fn z() {} }\n"
        "#[cfg(test)]\nmod hidden { fn t() { value.unwrap(); } }\n"
        "// value.expect(\"comment\");\n"
    )
    got = count_text(sample)
    expected = {"unwrap": 1, "expect": 1, "panic": 1, "unwrap_or_default": 1, "allow": 1}
    if got != expected:
        print(f"check-error-budget self-test FAIL: {got} != {expected}")
        return 1
    if grown_rules({"unwrap": 2, **{r: 0 for r in RULES[1:]}},
                   {"unwrap": 1, **{r: 0 for r in RULES[1:]}}) != ["unwrap"]:
        print("check-error-budget self-test FAIL: growth not detected")
        return 1
    print("check-error-budget self-test OK (counting, cfg(test) strip, growth)")
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--crates", nargs="+", default=list(DEFAULT_CRATES),
                        help="crate names under botserver/crates (bounded)")
    parser.add_argument("--out-dir", default=os.path.join("/tmp", "opencode", "error-budget"),
                        help="directory for the JSON report and baseline")
    parser.add_argument("--max-crates", type=int, default=MAX_CRATES,
                        help="maximum number of crates allowed in one run")
    parser.add_argument("--update-baseline", action="store_true",
                        help="lower the baseline to current counts (never raises it)")
    parser.add_argument("--self-test", action="store_true", help="run heuristic self-test")
    args = parser.parse_args(argv)

    if args.self_test:
        return self_test()

    crates = sorted(set(args.crates))
    if not crates or len(crates) > args.max_crates:
        print(f"check-error-budget: crate count must be 1..{args.max_crates}", file=sys.stderr)
        return 2

    per_crate = {crate: scan_crate(crate) for crate in crates}
    totals = totals_of(per_crate)
    report = {"schema": 1, "crates": per_crate, "totals": totals}

    os.makedirs(args.out_dir, exist_ok=True)
    report_path = os.path.join(args.out_dir, "error-budget-report.json")
    baseline_path = os.path.join(args.out_dir, "error-budget-baseline.json")
    write_json(report_path, report)

    baseline = load_json(baseline_path)
    if baseline is None or args.update_baseline:
        verb = "created" if baseline is None else "updated"
        baseline = report if baseline is None else clamp_baseline(report, baseline)
        write_json(baseline_path, baseline)
        print(f"check-error-budget: baseline {verb} at {baseline_path}")
        print("check-error-budget: " + ", ".join(f"{r}={totals[r]}" for r in RULES))
        return 0

    grown = grown_rules(totals, baseline.get("totals", {}))
    summary = ", ".join(f"{r}={totals[r]}" for r in RULES)
    if grown:
        details = ", ".join(
            f"{r} {totals[r]} > {baseline.get('totals', {}).get(r, 0)}"
            for r in grown
        )
        print(f"check-error-budget: FAIL — {summary}; regression: {details}")
        return 1
    print(f"check-error-budget: OK — {summary} (at/below baseline)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
