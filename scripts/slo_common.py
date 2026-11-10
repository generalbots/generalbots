#!/usr/bin/env python3
"""Shared pieces of the SLO gate (#2115): config, snapshot and evaluation.

``scripts/check_slo.py`` is the CLI; this module holds the parts that are worth
testing on their own — the YAML subset loader (so the gate never depends on a
package the runner may not have), the cost model and the budget evaluation.

Config: ``ops/slo.yaml``. Snapshot: JSON with a ``metrics`` object, one entry per
SLO metric, either a statistics map (``p50``/``p95``/``p99``/``max``) or a bare
number when the SLO declares ``stat: value``.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parent.parent
DEFAULT_CONFIG = ROOT / "ops" / "slo.yaml"
STATS = ("p50", "p95", "p99", "max")


# --------------------------------------------------------------------------
# Config loading: PyYAML when available, a strict subset parser otherwise.
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


def parse_simple_yaml(text: str) -> dict[str, Any]:
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
            while (
                index < len(lines)
                and lines[index][0] == indent
                and lines[index][1].startswith("- ")
            ):
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
        while (
            index < len(lines)
            and lines[index][0] == indent
            and not lines[index][1].startswith("- ")
        ):
            key, separator, value = lines[index][1].partition(":")
            if not separator:
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


def expand_objectives(config: dict[str, Any]) -> dict[str, Any]:
    """Normalize the flat ``objective.<id>.<key>`` map into the SLO list.

    ``ops/slo.yaml`` is shared with ``botanalytics::ops_slo`` (#2116), which
    reads the flat dotted keys without a YAML dependency. The gate wants the
    list form, so one file serves both consumers: the list is synthesized from
    the flat map when ``slos`` is absent.
    """
    if isinstance(config.get("slos"), list) and config["slos"]:
        return config
    flat = {
        str(key): value
        for key, value in config.items()
        if isinstance(key, str) and key.startswith("objective.")
    }
    if not flat:
        return config
    grouped: dict[str, dict[str, Any]] = {}
    for key, value in flat.items():
        _, identifier, field = key.split(".", 2)
        grouped.setdefault(identifier, {})[field] = value
    slos = []
    for identifier, fields in grouped.items():
        entry = {
            "id": identifier,
            "source": identifier,
            "stat": "value",
            "op": "lte",
            "profile": "local-mock",
        }
        entry.update(fields)
        slos.append(entry)
    config = dict(config)
    config["slos"] = slos
    return config


def load_config(path: Path = DEFAULT_CONFIG, force_fallback: bool = False) -> dict[str, Any]:
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
            return expand_objectives(loaded)
    return expand_objectives(parse_simple_yaml(text))


# --------------------------------------------------------------------------
# Snapshot, cost and evaluation
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


def resolve_metric(
    snapshot: dict[str, Any], metric: str, stat: str
) -> tuple[float | None, str | None]:
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
        declared = {str(slo.get("profile")) for slo in slos}
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
        met = value >= target if op == "gte" else value <= target
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
            breaches.append(
                {**entry, "kind": "budget", "detail": f"{value} {op} {target} is false"}
            )

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
    if quota and tenant_day is not None and tenant_day > float(quota) * alert_pct / 100.0:
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
