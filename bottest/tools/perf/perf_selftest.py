#!/usr/bin/env python3
"""Non-vacuity self-test for the #2459 performance-budget gate.

Kept in its own module so ``check_perf_budgets.py`` stays focused and small.
Invoked through ``check_perf_budgets.py --self-test``; no fixtures are written
to the repository — all reports live in a temporary directory.
"""

from __future__ import annotations

import copy
import json
import sys
import tempfile
from pathlib import Path
from typing import Any

from check_perf_budgets import (
    DEFAULT_CONFIG,
    MAX,
    MIN,
    evaluate,
    load_config,
    load_report,
    validate_config,
)


def _compliant_report(config: dict[str, Any]) -> dict[str, Any]:
    """A report where every app sits exactly at each budget (boundary case)."""
    return {
        "profile": "self-test",
        "measured_at": "2026-11-26T00:00:00Z",
        "apps": {
            app: dict(entry["budgets"]) for app, entry in config["apps"].items()
        },
    }


def self_test(config_path: Path = DEFAULT_CONFIG) -> int:
    """Prove the gate is non-vacuous without any fixtures in the repository."""
    checks = 0
    failures: list[str] = []

    def check(condition: bool, label: str) -> None:
        nonlocal checks
        checks += 1
        if not condition:
            failures.append(label)

    try:
        config = load_config(config_path)
    except ValueError as error:
        print(f"self-test: cannot load {config_path}: {error}", file=sys.stderr)
        return 1

    check(validate_config(config) == [], "shipped config validates with no problems")
    check(len(config["apps"]) >= 3, "config declares the three office apps")
    check(
        config["metrics"]["scroll_fps"]["direction"] == MIN,
        "scroll fps is a higher-is-better budget",
    )
    check(
        all(
            config["metrics"][name]["direction"] == MAX
            for name in config["core_metrics"]
            if name != "scroll_fps"
        ),
        "latency/memory budgets are lower-is-better",
    )

    baseline = _compliant_report(config)

    with tempfile.TemporaryDirectory(prefix="perf-budget-selftest-") as tmp:
        def roundtrip(name: str, payload: dict[str, Any]) -> dict[str, Any]:
            path = Path(tmp) / name
            path.write_text(json.dumps(payload), encoding="utf-8")
            return load_report(path)

        ok_report = evaluate(config, roundtrip("pass.json", baseline))
        check(ok_report["ok"] is True, "a report at every budget passes")
        expected = sum(len(entry["budgets"]) for entry in config["apps"].values())
        check(ok_report["checked"] == expected, "every declared app/metric is evaluated")
        check(ok_report["passed"] == expected, "every declared app/metric passes at budget")

        # A small change still inside the budget must pass: the gate is not
        # "any change fails".
        within = copy.deepcopy(baseline)
        within["apps"]["docs"]["open_p75_ms"] = round(
            config["apps"]["docs"]["budgets"]["open_p75_ms"] * 0.95, 3
        )
        check(evaluate(config, within)["ok"] is True, "a 5% improvement stays green")

        # Non-vacuity: a regression 10% past a lower-is-better budget fails and
        # names the exact app/metric.
        regressed = copy.deepcopy(baseline)
        regressed["apps"]["docs"]["open_p75_ms"] = round(
            config["apps"]["docs"]["budgets"]["open_p75_ms"] * 1.10, 3
        )
        bad = evaluate(config, regressed)
        check(bad["ok"] is False, "a 10% regression past the open budget fails")
        check(
            any(
                breach["kind"] == "budget"
                and breach["app"] == "docs"
                and breach["metric"] == "open_p75_ms"
                for breach in bad["breaches"]
            ),
            "the breach names the regressed app/metric",
        )

        # Direction guard: a dropped fps (higher-is-better) must fail too.
        dropped = copy.deepcopy(baseline)
        dropped["apps"]["sheets"]["scroll_fps"] = round(
            config["apps"]["sheets"]["budgets"]["scroll_fps"] * 0.90, 3
        )
        check(evaluate(config, dropped)["ok"] is False, "a 10% scroll-fps drop fails")

        # Fail closed: an omitted metric cannot be silently ignored.
        missing = copy.deepcopy(baseline)
        del missing["apps"]["slides"]["keystroke_p95_ms"]
        missing_report = evaluate(config, missing)
        check(missing_report["ok"] is False, "an omitted metric fails closed")
        check(
            any(breach["kind"] == "missing_metric" for breach in missing_report["breaches"]),
            "the omitted metric is reported as missing",
        )

        # Schema guard: a table with a missing budget and a non-positive budget
        # is rejected instead of producing a vacuous gate.
        broken = copy.deepcopy(config)
        del broken["apps"]["sheets"]["budgets"]["memory_per_tab_mb"]
        broken["apps"]["docs"]["budgets"]["scroll_fps"] = -1
        problems = validate_config(broken)
        check(any("memory_per_tab_mb" in item for item in problems), "missing budget is rejected")
        check(any("scroll_fps" in item for item in problems), "invalid budget is rejected")

        # Malformed report JSON is rejected, never treated as an empty pass.
        bad_path = Path(tmp) / "bad.json"
        bad_path.write_text("{not json", encoding="utf-8")
        try:
            load_report(bad_path)
        except ValueError:
            check(True, "a malformed report is rejected")
        else:
            check(False, "a malformed report is rejected")

    if failures:
        for label in failures:
            print(f"self-test FAIL: {label}", file=sys.stderr)
        print(f"self-test: {len(failures)}/{checks} checks failed", file=sys.stderr)
        return 1
    print(f"self-test OK — {checks} checks passed")
    return 0


if __name__ == "__main__":
    sys.exit(self_test())
