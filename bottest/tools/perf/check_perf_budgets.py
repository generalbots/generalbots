#!/usr/bin/env python3
"""Offline CI performance-budget gate (#2459, bounded slice).

The budgets live once, in ``perf_budgets.json`` next to this script. This gate
does three things and nothing more:

1. ``--validate-config`` checks the config schema (fail closed on a bad table).
2. ``--self-test`` proves the gate is non-vacuous: a compliant report passes,
   a regression past a budget fails, and a missing metric fails closed.
3. ``--report CI.json`` ingests a CI perf JSON report and pass/fails it against
   every declared budget.

It deliberately does NOT collect telemetry, emit metrics, render dashboards,
fire alerts or talk to a server. Those need a running stack and are out of
scope for this slice.

Config shape (see perf_budgets.json)::

    {
      "metrics": {"open_p75_ms": {"unit": "ms", "direction": "max", ...}, ...},
      "core_metrics": ["open_p75_ms", ...],
      "apps": {"docs": {"budgets": {"open_p75_ms": 1000, ...}}, ...}
    }

CI perf report shape (``apps`` is the only required key)::

    {
      "profile": "reference-laptop",
      "measured_at": "2026-11-26T00:00:00Z",
      "trace_url": "https://ci.example/trace/123",
      "apps": {
        "docs": {"open_p75_ms": 812, "keystroke_p95_ms": 12.4, ...},
        "slides": {...}, "sheets": {...}
      }
    }

A metric is PASS when ``value <= budget`` for ``direction: max`` (lower is
better) or ``value >= budget`` for ``direction: min`` (higher is better, e.g.
scroll fps). The comparison is inclusive: a value exactly at budget passes.

Exit codes: 0 all checks met · 1 a budget breach / failed self-test ·
2 the config or the report is unusable (never silently green).
"""

from __future__ import annotations

import argparse
import json
import math
import sys
from pathlib import Path
from typing import Any

HERE = Path(__file__).resolve().parent
DEFAULT_CONFIG = HERE / "perf_budgets.json"

MAX = "max"
MIN = "min"
DIRECTIONS = (MAX, MIN)


def load_config(path: Path) -> dict[str, Any]:
    """Read the budgets JSON. Raises ValueError on unreadable/non-object input."""
    try:
        raw = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as error:
        raise ValueError(f"{path}: invalid JSON: {error}") from error
    except OSError as error:
        raise ValueError(f"{path}: cannot read config: {error}") from error
    if not isinstance(raw, dict):
        raise ValueError(f"{path}: config top level must be an object")
    return raw


def load_report(path: Path) -> dict[str, Any]:
    """Read a CI perf JSON report. Raises ValueError on unreadable/non-object input."""
    try:
        raw = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as error:
        raise ValueError(f"{path}: invalid JSON: {error}") from error
    except OSError as error:
        raise ValueError(f"{path}: cannot read report: {error}") from error
    if not isinstance(raw, dict) or not isinstance(raw.get("apps"), dict):
        raise ValueError(f"{path}: report must be an object with an 'apps' object")
    return raw


def _is_number(value: Any) -> bool:
    return isinstance(value, (int, float)) and not isinstance(value, bool) and math.isfinite(value)


def validate_config(config: dict[str, Any]) -> list[str]:
    """Return every schema/policy problem; an empty list means the table is sound."""
    problems: list[str] = []

    if config.get("schema_version") != 1:
        problems.append("schema_version must be 1")
    if config.get("issue") != 2459:
        problems.append("issue must be 2459")
    if not isinstance(config.get("reference_profile"), dict):
        problems.append("reference_profile must be an object")

    metrics = config.get("metrics")
    if not isinstance(metrics, dict) or not metrics:
        problems.append("metrics must be a non-empty object")
        return problems

    for name, definition in sorted(metrics.items()):
        if not isinstance(name, str) or not name:
            problems.append("metrics: empty metric name")
            continue
        if not isinstance(definition, dict):
            problems.append(f"metrics.{name}: must be an object")
            continue
        if not isinstance(definition.get("unit"), str) or not definition["unit"]:
            problems.append(f"metrics.{name}.unit must be a non-empty string")
        if definition.get("direction") not in DIRECTIONS:
            problems.append(f"metrics.{name}.direction must be one of {DIRECTIONS}")
        if not isinstance(definition.get("label"), str) or not definition["label"]:
            problems.append(f"metrics.{name}.label must be a non-empty string")

    core = config.get("core_metrics")
    if not isinstance(core, list) or not core:
        problems.append("core_metrics must be a non-empty list")
    else:
        for name in core:
            if name not in metrics:
                problems.append(f"core_metrics.{name}: not declared in metrics")

    apps = config.get("apps")
    if not isinstance(apps, dict) or not apps:
        problems.append("apps must be a non-empty object")
        return problems

    for app, entry in sorted(apps.items()):
        if not isinstance(app, str) or not app:
            problems.append("apps: empty app name")
            continue
        if not isinstance(entry, dict):
            problems.append(f"apps.{app}: must be an object")
            continue
        budgets = entry.get("budgets")
        if not isinstance(budgets, dict):
            problems.append(f"apps.{app}.budgets must be an object")
            continue
        for name in sorted(metrics):
            if name not in budgets:
                problems.append(f"apps.{app}.budgets.{name}: missing required budget")
        for name, value in sorted(budgets.items()):
            if name not in metrics:
                problems.append(f"apps.{app}.budgets.{name}: not declared in metrics")
                continue
            if not _is_number(value) or value <= 0:
                problems.append(f"apps.{app}.budgets.{name}: must be a positive finite number")

    return problems


def evaluate(config: dict[str, Any], report: dict[str, Any]) -> dict[str, Any]:
    """Compare a CI perf report against every configured per-app budget."""
    results: list[dict[str, Any]] = []
    breaches: list[dict[str, str]] = []
    warnings: list[str] = []

    apps = config["apps"]
    metrics = config["metrics"]
    report_apps = report["apps"]

    for app in sorted(apps):
        budgets = apps[app]["budgets"]
        measured = report_apps.get(app)
        if not isinstance(measured, dict):
            breaches.append({
                "kind": "missing_app",
                "app": app,
                "metric": "-",
                "detail": "report has no measurements for this budgeted app",
            })
            for name in sorted(budgets):
                results.append({
                    "app": app, "metric": name, "value": None,
                    "budget": budgets[name], "direction": metrics[name]["direction"],
                    "unit": metrics[name]["unit"], "met": False,
                })
            continue

        for name in sorted(budgets):
            budget = budgets[name]
            definition = metrics[name]
            direction = definition["direction"]
            unit = definition["unit"]
            value = measured.get(name)
            result = {
                "app": app, "metric": name, "value": value, "budget": budget,
                "direction": direction, "unit": unit, "met": False,
            }
            if not _is_number(value):
                breaches.append({
                    "kind": "missing_metric" if name not in measured else "invalid_metric",
                    "app": app, "metric": name,
                    "detail": "metric is missing or not a finite number in the report",
                })
            elif (direction == MAX and value > budget) or (direction == MIN and value < budget):
                breaches.append({
                    "kind": "budget",
                    "app": app, "metric": name,
                    "detail": f"{value} {'>' if direction == MAX else '<'} {budget} {unit}",
                })
            else:
                result["met"] = True
            results.append(result)

    for app in sorted(report_apps):
        if app not in apps:
            warnings.append(f"report app {app!r} has no budget in config; ignored")

    return {
        "ok": not breaches,
        "profile": report.get("profile"),
        "measured_at": report.get("measured_at"),
        "trace_url": report.get("trace_url"),
        "checked": len(results),
        "passed": sum(1 for entry in results if entry["met"]),
        "results": results,
        "breaches": breaches,
        "warnings": warnings,
    }


def render_text(report: dict[str, Any]) -> str:
    lines = [
        f"perf budget check: profile={report['profile'] or '-'} "
        f"checked={report['checked']} passed={report['passed']} "
        f"breaches={len(report['breaches'])}"
    ]
    if report.get("trace_url"):
        lines.append(f"  trace: {report['trace_url']}")
    for entry in report["results"]:
        status = "PASS" if entry["met"] else "FAIL"
        value = entry["value"]
        shown = "-" if value is None else str(value)
        op = "<=" if entry["direction"] == MAX else ">="
        lines.append(
            f"  {status} {entry['app']}.{entry['metric']} "
            f"{shown} {op} {entry['budget']} {entry['unit']}"
        )
    for warning in report["warnings"]:
        lines.append(f"  WARN {warning}")
    for breach in report["breaches"]:
        lines.append(f"  BREACH {breach['kind']}: {breach['app']}.{breach['metric']} {breach['detail']}")
    lines.append("RESULT: " + ("ok" if report["ok"] else "FAILED"))
    return "\n".join(lines)


def main() -> int:
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    parser.add_argument("--config", type=Path, default=DEFAULT_CONFIG, help="budgets JSON file")
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--report", type=Path, help="CI perf JSON report to check")
    mode.add_argument("--validate-config", action="store_true", help="validate the config schema only")
    mode.add_argument("--self-test", action="store_true", help="prove the gate is non-vacuous")
    parser.add_argument("--json", action="store_true", help="emit the verdict as JSON")
    args = parser.parse_args()

    if args.self_test:
        from perf_selftest import self_test

        return self_test(args.config)

    try:
        config = load_config(args.config)
    except ValueError as error:
        print(f"config error: {error}", file=sys.stderr)
        return 2
    problems = validate_config(config)
    if problems:
        for problem in problems:
            print(f"config error: {problem}", file=sys.stderr)
        return 2

    if args.validate_config:
        if args.json:
            print(json.dumps({"ok": True, "problems": []}))
        else:
            print(f"config OK — {len(config['apps'])} apps, {len(config['metrics'])} metrics")
        return 0

    try:
        report = load_report(args.report)
    except ValueError as error:
        print(f"report error: {error}", file=sys.stderr)
        return 2

    verdict = evaluate(config, report)
    if args.json:
        print(json.dumps(verdict, indent=2, sort_keys=True))
    else:
        print(render_text(verdict))
    return 0 if verdict["ok"] else 1


if __name__ == "__main__":
    sys.exit(main())
