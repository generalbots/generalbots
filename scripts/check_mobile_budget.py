#!/usr/bin/env python3
"""Mobile-form-factor ratchet (#2421) — shrink-only, checked-in baseline.

Counts four structural debt signals across the suite UI (vendor and `.orig`
files are excluded) and fails when any count rises above its recorded baseline:

  vh100        `100vh` uses (viewport-height bugs on mobile browsers)
  hover_only   mouseenter/mouseleave/mouseover/mouseout handlers (no touch path)
  sub44        sub-44px sizing on interactive selectors (touch-target debt)
  bp_space     malformed breakpoints like `768 px` (spacing drift)

Usage:
    python3 scripts/check_mobile_budget.py --check
    python3 scripts/check_mobile_budget.py --write-baseline
    python3 scripts/check_mobile_budget.py --self-test

Exit codes: 0 clean, 1 a count rose above its baseline, 2 usage error.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SUITE = ROOT / "botui" / "ui" / "suite"
BASELINE = ROOT / "scripts" / "mobile_budget_baseline.json"

METRICS = ("vh100", "hover_only", "sub44", "bp_space")
SKIP_PARTS = ("/vendor/", "/node_modules/")

VH100 = re.compile(r"\b100vh\b")
HOVER = re.compile(r"\b(?:mouseenter|mouseleave|mouseover|mouseout)\b")
BP_SPACE = re.compile(r"\(\s*(?:min|max)-width\s*:\s*\d+\s+px")
SIZE = re.compile(r"\b(?:min-height|height|min-width|width)\s*:\s*(\d+(?:\.\d+)?)px\b")
RULE = re.compile(r"([^{}]+)\{([^{}]*)\}")
INTERACTIVE = re.compile(
    r"\b(?:button|a|input|select|textarea|btn|button|tab|icon|link|chip|action|"
    r"close|toggle|menu|nav|control|toolbar|handle|switch|checkbox|radio)\b",
    re.IGNORECASE,
)


def _source_files() -> list[Path]:
    files: list[Path] = []
    for path in sorted(SUITE.rglob("*")):
        if path.suffix not in {".css", ".js"}:
            continue
        rel = "/" + path.relative_to(ROOT).as_posix()
        if any(part in rel for part in SKIP_PARTS) or rel.endswith(".orig"):
            continue
        files.append(path)
    return files


def measure(files: list[Path] | None = None) -> dict[str, int]:
    counts = dict.fromkeys(METRICS, 0)
    for path in files if files is not None else _source_files():
        try:
            text = path.read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        counts["vh100"] += len(VH100.findall(text))
        counts["hover_only"] += len(HOVER.findall(text))
        counts["bp_space"] += len(BP_SPACE.findall(text))
        if path.suffix == ".css":
            for selector, body in RULE.findall(text):
                if not INTERACTIVE.search(selector):
                    continue
                for value in SIZE.findall(body):
                    number = float(value)
                    if 0 < number < 44:
                        counts["sub44"] += 1
    return counts


def load_baseline(path: Path = BASELINE) -> dict[str, int]:
    try:
        raw = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, ValueError):
        return {}
    values = raw.get("counts", {}) if isinstance(raw, dict) else {}
    if not isinstance(values, dict):
        return {}
    return {k: int(values[k]) for k in METRICS if k in values}


def write_baseline(counts: dict[str, int], path: Path = BASELINE) -> None:
    payload = {
        "issue": 2421,
        "note": "Shrink-only baseline. Lower counts are always accepted; raising one fails --check.",
        "counts": {k: counts[k] for k in METRICS},
    }
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def check(counts: dict[str, int], baseline: dict[str, int]) -> int:
    failed = 0
    for key in METRICS:
        if key not in counts:
            print(f"mobile-budget: {key} missing from measurement")
            failed += 1
            continue
        measured = counts[key]
        recorded = baseline.get(key)
        if recorded is None:
            print(f"mobile-budget: {key}={measured} has no baseline entry")
            failed += 1
        elif measured > recorded:
            print(f"mobile-budget: {key} rose {recorded} -> {measured}")
            failed += 1
        else:
            print(f"mobile-budget: {key}={measured} (baseline {recorded}) OK")
    return 1 if failed else 0


def self_test() -> int:
    """Prove a raised count fails and a lowered count passes, without a repo scan."""
    failures = 0

    def expect(condition: bool, label: str) -> None:
        nonlocal failures
        if not condition:
            failures += 1
            print(f"self-test FAILED: {label}")

    baseline = dict.fromkeys(METRICS, 10)
    expect(check(dict.fromkeys(METRICS, 10), baseline) == 0, "equal counts pass")
    expect(check(dict.fromkeys(METRICS, 5), baseline) == 0, "lower counts pass")
    bumped = dict.fromkeys(METRICS, 10)
    bumped["vh100"] = 11
    expect(check(bumped, baseline) == 1, "a bumped count fails")
    expect(check({k: 10 for k in METRICS if k != "sub44"}, baseline) == 1, "a missing metric fails")

    import tempfile

    with tempfile.TemporaryDirectory() as tmp:
        probe = Path(tmp) / "a.css"
        probe.write_text(
            "button { min-height: 28px; }\n"
            ".panel { min-height: 60px; }\n"
            "@media (max-width: 768 px) { }\n",
            encoding="utf-8",
        )
        measured = measure([probe])
        expect(measured["sub44"] == 1, "sub44 counts only small interactive targets")
        expect(measured["bp_space"] == 1, "bp_space detects `768 px`")
        probe.write_text("button { min-height: 48px; }\n", encoding="utf-8")
        expect(measure([probe])["sub44"] == 0, "sub44 ignores targets at or above 44px")

    if failures:
        print(f"self-test: {failures} failure(s)")
        return 1
    print("self-test: ok")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--check", action="store_true", help="fail if a count rose above baseline")
    group.add_argument("--write-baseline", action="store_true", help="record the current counts")
    group.add_argument("--self-test", action="store_true", help="prove the ratchet logic offline")
    args = parser.parse_args()

    if args.self_test:
        return self_test()
    counts = measure()
    if args.write_baseline:
        write_baseline(counts)
        print(f"mobile-budget: baseline written to {BASELINE.relative_to(ROOT)}")
        print(json.dumps(counts, sort_keys=True))
        return 0
    return check(counts, load_baseline())


if __name__ == "__main__":
    sys.exit(main())
