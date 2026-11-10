#!/usr/bin/env python3
"""Enforce the performance budgets from ``ops/slo.yaml`` (#2115).

The SLOs are declared once, in ``ops/slo.yaml``, and the human-readable table in
``botbook/src/11-hardware-scaling/slo.md`` mirrors it. This script is the gate:
it takes a metrics snapshot (JSON) and exits non-zero when a budget is breached,
so a performance regression fails the build instead of shipping invisibly.

Snapshot format (``metrics`` is the only required key)::

    {
      "profile": "local-mock",
      "measured_at": "2026-11-10T12:00:00Z",
      "window_days": 7,
      "metrics": {
        "chat.first_token_ms": {"p50": 620, "p95": 1500, "p99": 2900},
        "rest.api_ms": {"p50": 90, "p95": 210, "p99": 480},
        "turn.error_pct": 0.4,
        "ws.unexpected_disconnect_pct": 0.18
      },
      "cost": {"model": "gpt-4o-mini", "conversations": 400,
               "tokens_in": 800000, "tokens_out": 120000}
    }

The cost block is optional: when present, cost per conversation is computed from
the tokens and the price table in ``ops/slo.yaml``, and that computed value — not
a self-reported one — is what the cost budget is checked against.

Usage::

    python3 scripts/check_slo.py --snapshot run.json
    python3 scripts/check_slo.py --snapshot run.json --json
    python3 scripts/check_slo.py --snapshot run.json --baseline perf/baselines/local-mock.json
    python3 scripts/check_slo.py --profile hotpath --snapshot target/slo/hotpath.json
    python3 scripts/check_slo.py --self-test

Exit codes: 0 every budget met, 1 a breach, 2 the config or the snapshot is
unusable (never silently green).
"""

from __future__ import annotations

import argparse
import json
import sys
import tempfile
from pathlib import Path
from typing import Any

from slo_common import (
    DEFAULT_CONFIG,
    apply_cost,
    evaluate,
    load_config,
    load_snapshot,
)


def render_text(report: dict[str, Any]) -> str:
    lines = [
        f"SLO check: profile={report['profile'] or 'all'} checked={report['checked']} "
        f"breaches={len(report['breaches'])}"
    ]
    for entry in report["slos"]:
        status = "PASS" if entry["met"] else "FAIL"
        unit = entry["unit"] or ""
        lines.append(
            f"  {status} {entry['id']}[{entry['stat']}] "
            f"{entry['value']} {entry['op']} {entry['target']} {unit} ({entry['window']})"
        )
    for warning in report["warnings"]:
        lines.append(f"  WARN {warning}")
    for breach in report["breaches"]:
        lines.append(
            f"  BREACH {breach['kind']}: {breach['id']}[{breach['stat']}] {breach['detail']}"
        )
    lines.append("RESULT: " + ("ok" if report["ok"] else "FAILED"))
    return "\n".join(lines)


# --------------------------------------------------------------------------
# Self-test: prove the gate fails closed, without fixtures in the repository.
# --------------------------------------------------------------------------
def self_test(config_path: Path = DEFAULT_CONFIG) -> int:
    checks = 0
    failures: list[str] = []

    def check(condition: bool, label: str) -> None:
        nonlocal checks
        checks += 1
        if not condition:
            failures.append(label)

    try:
        config = load_config(config_path)
        fallback = load_config(config_path, force_fallback=True)
        check(config == fallback, "fallback YAML parser disagrees with PyYAML")
        check(len(config["slos"]) >= 10, "config declares the SLO table")
        check(config["pricing"]["models"]["mock"]["input"] == 0.0, "mock model is priced at zero")
    except (OSError, ValueError, KeyError) as error:
        print(f"self-test: cannot load {config_path}: {error}", file=sys.stderr)
        return 1

    local = [slo for slo in config["slos"] if slo.get("profile") == "local-mock"]

    def snapshot_at(factor: float, profile: str = "local-mock") -> dict[str, Any]:
        """Build a snapshot where every objective sits at ``factor`` x its budget.

        Two objectives may read the same snapshot metric (``ws_handshake`` and
        ``ws_disconnect_rate`` both watch the WebSocket stability rate, and the
        error objectives share ``turn.error_pct``), so one metric can only carry
        one value. It gets the value that is worst for the expected verdict:
        the smallest budget for a run that must pass, the largest for a run that
        must fail — either way no objective is judged against a friendlier
        number than its own.
        """
        budgets: dict[str, list[tuple[str, float]]] = {}
        for slo in local:
            budgets.setdefault(str(slo["source"]), []).append(
                (str(slo["stat"]), float(slo["target"]))
            )
        metrics: dict[str, Any] = {}
        for source, entries in budgets.items():
            scaled = [round(limit * factor, 6) for _, limit in entries]
            value = min(scaled) if factor <= 1 else max(scaled)
            stats = {stat for stat, _ in entries if stat != "value"}
            metrics[source] = {stat: value for stat in stats} if stats else value
        return {"profile": profile, "metrics": metrics}

    with tempfile.TemporaryDirectory(prefix="slo-selftest-") as tmp:
        passing = snapshot_at(0.5)
        failing = snapshot_at(2.0)
        passing_path = Path(tmp) / "passing.json"
        failing_path = Path(tmp) / "failing.json"
        passing_path.write_text(json.dumps(passing), encoding="utf-8")
        failing_path.write_text(json.dumps(failing), encoding="utf-8")

        ok_report = evaluate(config, load_snapshot(str(passing_path)))
        check(ok_report["ok"] is True, "a snapshot under every budget passes")
        check(ok_report["checked"] == len(local), "every local-mock SLO is evaluated")

        bad_report = evaluate(config, load_snapshot(str(failing_path)))
        check(bad_report["ok"] is False, "a snapshot over every budget fails")
        check(len(bad_report["breaches"]) >= len(local), "each breached SLO is reported")

        # A regression that is still inside the budget must fail when a baseline exists.
        regression_report = evaluate(config, snapshot_at(0.7), baseline=snapshot_at(0.5))
        kinds = {breach["kind"] for breach in regression_report["breaches"]}
        check("regression" in kinds, "a 40% rise over the baseline is a regression breach")
        check(
            all(breach["kind"] != "budget" for breach in regression_report["breaches"]),
            "the regression fixture stays inside the absolute budget",
        )

        # Cost is computed from tokens, never taken from the snapshot.
        priced = {
            "profile": "local-mock",
            "metrics": dict(passing["metrics"]),
            "cost": {
                "model": "gpt-4o-mini",
                "conversations": 100,
                "tokens_in": 1_000_000,
                "tokens_out": 1_000_000,
            },
        }
        apply_cost(priced, config)
        expected = round((1_000_000 / 100) / 1_000_000 * (0.15 + 0.60), 8)
        check(
            abs(priced["metrics"]["cost.conversation_usd"] - expected) < 1e-9,
            "cost per conversation follows the price table",
        )
        check(evaluate(config, priced)["ok"] is True, "a priced run inside the cost budget passes")

        unknown = {
            "profile": "local-mock",
            "metrics": dict(passing["metrics"]),
            "cost": {
                "model": "unpriced-model",
                "conversations": 1,
                "tokens_in": 1,
                "tokens_out": 1,
            },
        }
        try:
            apply_cost(unknown, config)
        except ValueError:
            check(True, "an unpriced model is refused")
        else:
            check(False, "an unpriced model must be refused")

        missing_report = evaluate(config, {"profile": "local-mock", "metrics": {}})
        check(missing_report["ok"] is False, "missing metrics fail closed")
        tolerant = evaluate(config, {"profile": "local-mock", "metrics": {}}, allow_missing=True)
        check(
            tolerant["ok"] is True and bool(tolerant["warnings"]),
            "--allow-missing downgrades to warnings",
        )

    for failure in failures:
        print(f"self-test FAILED: {failure}", file=sys.stderr)
    print(f"self-test: {checks - len(failures)}/{checks} checks passed")
    return 0 if not failures else 1


def run(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description="Enforce the SLO budgets (#2115).")
    parser.add_argument("--config", default=str(DEFAULT_CONFIG), help="SLO definition file")
    parser.add_argument("--snapshot", default="-", help="metrics snapshot JSON, or - for stdin")
    parser.add_argument("--baseline", help="baseline snapshot for the regression gate")
    parser.add_argument(
        "--max-regression-pct", type=float, help="override baseline_policy.regression_max_pct"
    )
    parser.add_argument("--profile", help="only evaluate SLOs for this profile")
    parser.add_argument("--class", dest="klass", help="only evaluate SLOs of this class")
    parser.add_argument(
        "--allow-missing", action="store_true", help="warn instead of failing on absent metrics"
    )
    parser.add_argument("--json", action="store_true", help="print the report as JSON")
    parser.add_argument("--self-test", action="store_true", help="prove the checker fails closed")
    args = parser.parse_args(argv)

    if args.self_test:
        return self_test(Path(args.config))

    try:
        config = load_config(Path(args.config))
        snapshot = load_snapshot(args.snapshot)
        snapshot["_cost"] = apply_cost(snapshot, config)
        baseline = load_snapshot(args.baseline) if args.baseline else None
        report = evaluate(
            config,
            snapshot,
            baseline=baseline,
            max_regression_pct=args.max_regression_pct,
            allow_missing=args.allow_missing,
            profile=args.profile,
            klass=args.klass,
        )
    except (OSError, ValueError) as error:
        print(f"check_slo: {error}", file=sys.stderr)
        return 2

    if args.json:
        print(json.dumps(report, indent=2, sort_keys=True))
    else:
        print(render_text(report))
    return 0 if report["ok"] else 1


if __name__ == "__main__":
    sys.exit(run(sys.argv[1:]))
