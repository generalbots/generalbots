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

A metric is either a statistics map (``p50``/``p95``/``p99``/``max``) or a bare
number when the SLO declares ``stat: value``. The cost block is optional: when
present, the conversation cost is computed from the tokens and the price table
in ``ops/slo.yaml``, and that computed value — not a self-reported one — is what
the cost budget is checked against.

Usage::

    python3 scripts/check_slo.py --snapshot run.json
    python3 scripts/check_slo.py --snapshot run.json --json
    python3 scripts/check_slo.py --snapshot run.json --baseline perf/baselines/local-mock.json
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

ROOT = Path(__file__).resolve().parent.parent
DEFAULT_CONFIG = ROOT / "ops" / "slo.yaml"
STATS = ("p50", "p95", "p99", "max")


# --------------------------------------------------------------------------
# Config loading: PyYAML when available, a strict subset parser otherwise so
# the gate never depends on a package the runner may not have installed.
# --------------------------------------------------------------------------
def _strip_comment(line: str) -> str:
    out = []
    quoted = False
    for index, char in enumerate(line):
        if char == '"':
            quoted = not quoted
        if char == "#" and not quoted and (index == 0 or line[index - 1] in " \t"):
            break
        out.append(char)
    return "".join(out).rstrip()


def _scalar(token: str) -> Any:
    token = token.strip()
    if len(token) >= 2 and token[0] == token[-1] and token[0] in "\"'":
        return token[1:-1]
    lowered = token.lower()
    if lowered in {"true", "false"}:
        return lowered == "true"
    if lowered in {"null", "~", ""}:
        return None
    if token.startswith("[") and token.endswith("]"):
        inner = token[1:-1].strip()
        return [_scalar(part) for part in inner.split(",")] if inner else []
    for caster in (int, float):
        try:
            return caster(token)
        except ValueError:
            continue
    return token


def _parse_simple_yaml(text: str) -> dict[str, Any]:
    """Parse the YAML subset used by ``ops/slo.yaml`` (no anchors, no blocks)."""
    lines: list[tuple[int, str]] = []
    for raw in text.splitlines():
        if not raw.strip() or raw.lstrip().startswith("#"):
            continue
        stripped = _strip_comment(raw)
        if not stripped.strip():
            continue
        indent = len(stripped) - len(stripped.lstrip(" "))
        lines.append((indent, stripped.strip()))

    def parse_block(index: int, indent: int) -> tuple[Any, int]:
        if index >= len(lines):
            return {}, index
        if lines[index][1].startswith("- "):
            items: list[Any] = []
            while index < len(lines) and lines[index][0] == indent and lines[index][1].startswith("- "):
                content = lines[index][1][2:].strip()
                index += 1
                if ":" not in content:
                    items.append(_scalar(content))
                    continue
                key, _, value = content.partition(":")
                item: dict[str, Any] = {}
                if value.strip():
                    item[key.strip()] = _scalar(value)
                elif index < len(lines) and lines[index][0] > indent:
                    nested, index = parse_block(index, lines[index][0])
                    item[key.strip()] = nested
                else:
                    item[key.strip()] = None
                while index < len(lines) and lines[index][0] > indent:
                    nested, index = parse_block(index, lines[index][0])
                    if isinstance(nested, dict):
                        item.update(nested)
                    else:
                        break
                items.append(item)
            return items, index

        mapping: dict[str, Any] = {}
        while index < len(lines) and lines[index][0] == indent and not lines[index][1].startswith("- "):
            key, sep, value = lines[index][1].partition(":")
            if not sep:
                raise ValueError(f"expected 'key: value', got {lines[index][1]!r}")
            index += 1
            if value.strip():
                mapping[key.strip()] = _scalar(value)
            elif index < len(lines) and lines[index][0] > indent:
                nested, index = parse_block(index, lines[index][0])
                mapping[key.strip()] = nested
            else:
                mapping[key.strip()] = None
        return mapping, index

    parsed, _ = parse_block(0, lines[0][0] if lines else 0)
    if not isinstance(parsed, dict):
        raise ValueError("the SLO config must be a mapping at the top level")
    return parsed


def load_config(path: Path, force_fallback: bool = False) -> dict[str, Any]:
    text = path.read_text(encoding="utf-8")
    if not force_fallback:
        try:
            import yaml  # type: ignore[import-not-found]
        except ImportError:
            pass
        else:
            loaded = yaml.safe_load(text)
            if not isinstance(loaded, dict):
                raise ValueError("the SLO config must be a mapping at the top level")
            return loaded
    return _parse_simple_yaml(text)


# --------------------------------------------------------------------------
# Snapshot and cost
# --------------------------------------------------------------------------
def load_snapshot(source: str) -> dict[str, Any]:
    if source == "-":
        raw = sys.stdin.read()
        label = "<stdin>"
    else:
        path = Path(source)
        if not path.is_file():
            raise ValueError(f"snapshot not found: {source}")
        raw = path.read_text(encoding="utf-8")
        label = source
    try:
        snapshot = json.loads(raw)
    except json.JSONDecodeError as error:
        raise ValueError(f"snapshot {label} is not valid JSON: {error}") from error
    if not isinstance(snapshot, dict) or not isinstance(snapshot.get("metrics"), dict):
        raise ValueError(f"snapshot {label} has no 'metrics' object")
    return snapshot


def apply_cost(snapshot: dict[str, Any], config: dict[str, Any]) -> dict[str, Any]:
    """Compute cost per conversation from tokens and the price table."""
    cost = snapshot.get("cost")
    if not isinstance(cost, dict):
        return {}
    model = str(cost.get("model", ""))
    models = (config.get("pricing") or {}).get("models") or {}
    if model not in models:
        raise ValueError(
            f"model {model!r} has no price in ops/slo.yaml; add it instead of guessing"
        )
    price = models[model] or {}
    price_in = float(price.get("input", 0.0))
    price_out = float(price.get("output", 0.0))

    conversations = int(cost.get("conversations", 0))
    tokens_in = float(cost.get("tokens_in", 0.0))
    tokens_out = float(cost.get("tokens_out", 0.0))
    if conversations <= 0:
        raise ValueError("cost.conversations must be a positive integer")

    per_conversation = (
        (tokens_in / conversations) / 1_000_000 * price_in
        + (tokens_out / conversations) / 1_000_000 * price_out
    )
    snapshot.setdefault("metrics", {})["cost.conversation_usd"] = per_conversation
    if "cost.tenant_day_usd" in cost:
        snapshot["metrics"]["cost.tenant_day_usd"] = float(cost["cost.tenant_day_usd"])

    return {
        "model": model,
        "conversations": conversations,
        "tokens_in": tokens_in,
        "tokens_out": tokens_out,
        "conversation_usd": round(per_conversation, 8),
        "price_in_per_million": price_in,
        "price_out_per_million": price_out,
    }


def resolve_metric(snapshot: dict[str, Any], metric: str, stat: str) -> tuple[float | None, str | None]:
    metrics = snapshot.get("metrics") or {}
    if metric not in metrics:
        return None, "metric not present in the snapshot"
    raw = metrics[metric]
    if isinstance(raw, dict):
        if stat in raw:
            return float(raw[stat]), None
        if stat == "value" and "value" in raw:
            return float(raw["value"]), None
        return None, f"stat {stat!r} not measured (available: {sorted(raw)})"
    if stat not in ("value", "max"):
        return None, f"scalar metric cannot answer stat {stat!r}"
    return float(raw), None


def evaluate(
    config: dict[str, Any],
    snapshot: dict[str, Any],
    baseline: dict[str, Any] | None = None,
    max_regression_pct: float | None = None,
    allow_missing: bool = False,
    profile: str | None = None,
    klass: str | None = None,
) -> dict[str, Any]:
    slos = config.get("slos") or []
    if not isinstance(slos, list) or not slos:
        raise ValueError("ops/slo.yaml declares no slos")

    if profile is None:
        candidate = snapshot.get("profile")
        declared = {str(s.get("profile")) for s in slos}
        profile = candidate if candidate in declared else None
    if max_regression_pct is None:
        policy = config.get("baseline_policy") or {}
        max_regression_pct = float(policy.get("regression_max_pct", 20.0))

    results: list[dict[str, Any]] = []
    breaches: list[dict[str, Any]] = []
    warnings: list[str] = []
    checked = 0

    for slo in slos:
        if profile and str(slo.get("profile")) != profile:
            continue
        if klass and str(slo.get("class")) != klass:
            continue
        slo_id = str(slo.get("id"))
        metric = str(slo.get("source") or slo_id)
        stat = str(slo.get("stat", "value"))
        op = str(slo.get("op", "lte"))
        target = float(slo.get("target"))
        value, problem = resolve_metric(snapshot, metric, stat)

        if value is None:
            if allow_missing:
                warnings.append(f"{slo_id}[{stat}]: skipped — {problem}")
                continue
            breaches.append(
                {
                    "kind": "missing",
                    "id": slo_id,
                    "stat": stat,
                    "target": target,
                    "value": None,
                    "unit": slo.get("unit"),
                    "window": slo.get("window"),
                    "detail": problem,
                }
            )
            continue

        checked += 1
        if op == "gte":
            met = value >= target
        else:
            met = value <= target
        entry = {
            "id": slo_id,
            "stat": stat,
            "op": op,
            "target": target,
            "value": value,
            "unit": slo.get("unit"),
            "window": slo.get("window"),
            "profile": slo.get("profile"),
            "met": met,
        }
        results.append(entry)
        if not met:
            breaches.append({**entry, "kind": "budget", "detail": f"{value} {op} {target} is false"})

        if baseline is not None:
            base_value, base_problem = resolve_metric(baseline, metric, stat)
            if base_value is None:
                warnings.append(f"{slo_id}[{stat}]: no baseline ({base_problem})")
                continue
            if base_value > 0 and op == "lte":
                delta_pct = (value - base_value) / base_value * 100.0
                if delta_pct > max_regression_pct:
                    breaches.append(
                        {
                            **entry,
                            "kind": "regression",
                            "baseline": base_value,
                            "delta_pct": round(delta_pct, 2),
                            "detail": (
                                f"{value} is {delta_pct:.1f}% over the baseline "
                                f"{base_value} (limit {max_regression_pct:.0f}%)"
                            ),
                        }
                    )

    budgets = config.get("cost_budgets") or {}
    quota = budgets.get("tenant_day_usd")
    alert_pct = float(budgets.get("alert_at_pct", 80))
    tenant_day, _ = resolve_metric(snapshot, "cost.tenant_day_usd", "value")
    if quota and tenant_day is not None:
        if tenant_day > float(quota) * alert_pct / 100.0:
            warnings.append(
                f"cost.tenant_day_usd {tenant_day} is past {alert_pct:.0f}% of the {quota} quota"
            )

    return {
        "ok": not breaches,
        "config_version": config.get("version"),
        "profile": profile,
        "checked": checked,
        "max_regression_pct": max_regression_pct,
        "breaches": breaches,
        "warnings": warnings,
        "slos": results,
        "cost": snapshot.get("_cost") or {},
    }


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
        lines.append(f"  BREACH {breach['kind']}: {breach['id']}[{breach['stat']}] {breach['detail']}")
    lines.append("RESULT: " + ("ok" if report["ok"] else "FAILED"))
    return "\n".join(lines)


# --------------------------------------------------------------------------
# Self-test: prove the gate fails closed, without fixtures in the repository.
# --------------------------------------------------------------------------
def _self_test() -> int:
    checks = 0
    failures: list[str] = []

    def check(condition: bool, label: str) -> None:
        nonlocal checks
        checks += 1
        if not condition:
            failures.append(label)

    try:
        config = load_config(DEFAULT_CONFIG)
        fallback = load_config(DEFAULT_CONFIG, force_fallback=True)
        check(config == fallback, "fallback YAML parser disagrees with PyYAML")
        check(len(config["slos"]) >= 10, "config declares the SLO table")
        check(config["pricing"]["models"]["mock"]["input"] == 0.0, "mock model is priced at zero")
    except (OSError, ValueError, KeyError) as error:
        print(f"self-test: cannot load ops/slo.yaml: {error}", file=sys.stderr)
        return 1

    local = [s for s in config["slos"] if s.get("profile") == "local-mock"]

    def snapshot_at(factor: float, profile: str = "local-mock") -> dict[str, Any]:
        metrics: dict[str, Any] = {}
        for slo in local:
            value = round(float(slo["target"]) * factor, 6)
            stat = slo["stat"]
            if stat == "value":
                metrics[slo["source"]] = value
            else:
                metrics.setdefault(slo["source"], {})[stat] = value
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
        baseline = snapshot_at(0.5)
        regressed = snapshot_at(0.7)
        regression_report = evaluate(config, regressed, baseline=baseline)
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
        expected = round((1_000_000 / 100) / 1_000_000 * 0.15 + (1_000_000 / 100) / 1_000_000 * 0.60, 8)
        check(
            abs(priced["metrics"]["cost.conversation_usd"] - expected) < 1e-9,
            "cost per conversation follows the price table",
        )
        cost_report = evaluate(config, priced)
        check(cost_report["ok"] is True, "a priced run inside the cost budget passes")

        unknown = {"profile": "local-mock", "metrics": dict(passing["metrics"]),
                   "cost": {"model": "unpriced-model", "conversations": 1,
                            "tokens_in": 1, "tokens_out": 1}}
        try:
            apply_cost(unknown, config)
        except ValueError:
            check(True, "an unpriced model is refused")
        else:
            check(False, "an unpriced model must be refused")

        missing = {"profile": "local-mock", "metrics": {}}
        missing_report = evaluate(config, missing)
        check(missing_report["ok"] is False, "missing metrics fail closed")
        tolerant = evaluate(config, missing, allow_missing=True)
        check(tolerant["ok"] is True and tolerant["warnings"], "--allow-missing downgrades to warnings")

    for failure in failures:
        print(f"self-test FAILED: {failure}", file=sys.stderr)
    print(f"self-test: {checks - len(failures)}/{checks} checks passed")
    return 0 if not failures else 1


# --------------------------------------------------------------------------
def run(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description="Enforce the SLO budgets (#2115).")
    parser.add_argument("--config", default=str(DEFAULT_CONFIG), help="SLO definition file")
    parser.add_argument("--snapshot", default="-", help="metrics snapshot JSON, or - for stdin")
    parser.add_argument("--baseline", help="baseline snapshot for the regression gate")
    parser.add_argument("--max-regression-pct", type=float, help="override baseline_policy.regression_max_pct")
    parser.add_argument("--profile", help="only evaluate SLOs for this profile")
    parser.add_argument("--class", dest="klass", help="only evaluate SLOs of this class")
    parser.add_argument("--allow-missing", action="store_true", help="warn instead of failing on absent metrics")
    parser.add_argument("--json", action="store_true", help="print the report as JSON")
    parser.add_argument("--self-test", action="store_true", help="prove the checker fails closed")
    args = parser.parse_args(argv)

    if args.self_test:
        return _self_test()

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
