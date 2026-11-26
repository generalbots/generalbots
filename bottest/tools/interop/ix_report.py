#!/usr/bin/env python3
"""Conversion report, structural-loss gate and report validator (#2463).

Test-only and stdlib-only; every function here is a pure function of its
inputs (no clock, host name, locale, file path or filesystem ordering leaks
into the output).

The report is deterministic JSON with sorted keys. It lists:

* ``losses``         -- features present in the source but reduced/dropped in
                        the GB-converted output (headings, tables, images,
                        formulas and fonts);
* ``font_decisions`` -- a machine-readable metric-compatible fallback decision
                        for every font used by the source document;
* ``gate``           -- PASS/FAIL with per-feature checks. The gate **fails**
                        when a required feature is *silently* dropped: a count
                        feature going to zero, or a font disappearing with no
                        documented fallback actually present in the output.
                        A font drop with a recorded metric-compatible fallback
                        that appears in the converted output is a WARN, not a
                        silent drop, so the gate passes with a visible warning.

:func:`validate` re-derives losses, decisions and gate from the stored feature
vectors, so a tampered report cannot pass the validator: any mismatch between
stored and recomputed content is reported as invalid.
"""

from __future__ import annotations

import json
from typing import Any

import ix_fonts

SCHEMA = "gb-interop-conversion-report/1"
ISSUE = 2463
GENERATOR = "bottest/tools/interop/interop.py"
COUNT_FEATURES = ("headings", "tables", "images", "formulas")


def _features(extract: dict[str, Any]) -> dict[str, Any]:
    features = extract["features"]
    for name in COUNT_FEATURES:
        value = features[name]
        if not isinstance(value, int) or value < 0:
            raise ValueError(f"feature {name} must be a non-negative int")
    if not isinstance(features["fonts"], list):
        raise ValueError("feature fonts must be a list")
    return features


def compute_font_decisions(source: dict[str, Any]) -> list[dict[str, object]]:
    """Return one fallback decision per unique source font, sorted by name."""
    return ix_fonts.decide_all(source["fonts"])


def compute_losses(source: dict[str, Any], converted: dict[str, Any],
                   decisions: list[dict[str, object]]) -> list[dict[str, Any]]:
    """Return structural losses, ordered by the fixed feature schema."""
    losses: list[dict[str, Any]] = []
    for name in COUNT_FEATURES:
        before, after = int(source[name]), int(converted[name])
        if before > after:
            losses.append({
                "feature": name,
                "type": "count",
                "source": before,
                "converted": after,
                "delta": after - before,
                "severity": "dropped" if after == 0 else "partial",
            })
    converted_fonts = set(converted["fonts"])
    by_name = {str(item["font"]): item for item in decisions}
    for font in sorted(set(source["fonts"]) - converted_fonts):
        decision = by_name.get(font, {})
        fallback = decision.get("fallback")
        documented = bool(
            decision.get("warning") and fallback
            and fallback in converted_fonts
        )
        losses.append({
            "feature": "fonts",
            "type": "font",
            "font": font,
            "source": "present",
            "converted": "absent",
            "fallback": fallback,
            "strategy": decision.get("strategy", "none"),
            "documented": documented,
        })
    return losses


def compute_gate(source: dict[str, Any], converted: dict[str, Any],
                 decisions: list[dict[str, object]]) -> dict[str, Any]:
    """Re-derive the PASS/FAIL gate from extracted features and decisions."""
    failures: list[dict[str, Any]] = []
    warnings: list[dict[str, Any]] = []
    checks: list[dict[str, Any]] = []

    for name in COUNT_FEATURES:
        before, after = int(source[name]), int(converted[name])
        if before > 0 and after == 0:
            failures.append({
                "feature": name, "reason": "silent-drop",
                "source": before, "converted": after,
            })
            checks.append({"feature": name, "status": "FAIL"})
        elif before > after:
            warnings.append({
                "feature": name, "reason": "partial-loss",
                "source": before, "converted": after,
            })
            checks.append({"feature": name, "status": "WARN"})
        else:
            checks.append({"feature": name, "status": "OK"})

    converted_fonts = set(converted["fonts"])
    by_name = {str(item["font"]): item for item in decisions}
    for font in sorted(set(source["fonts"]) - converted_fonts):
        decision = by_name.get(font, {})
        fallback = decision.get("fallback")
        documented = bool(
            decision.get("warning") and fallback
            and fallback in converted_fonts
        )
        if documented:
            warnings.append({
                "feature": "fonts", "font": font,
                "reason": "documented-substitution",
                "fallback": fallback,
                "strategy": decision.get("strategy", "none"),
            })
            checks.append({"feature": "fonts", "font": font, "status": "WARN"})
        else:
            failures.append({
                "feature": "fonts", "font": font,
                "reason": "silent-drop",
                "fallback": fallback,
                "strategy": decision.get("strategy", "none"),
            })
            checks.append({"feature": "fonts", "font": font, "status": "FAIL"})

    return {
        "passed": not failures,
        "required_features": list(COUNT_FEATURES),
        "failures": failures,
        "warnings": warnings,
        "checks": checks,
    }


def build_report(source_extract: dict[str, Any],
                 converted_extract: dict[str, Any],
                 source_name: str = "-", converted_name: str = "-",
                 synthetic_only: bool = False) -> dict[str, Any]:
    """Assemble a complete, deterministic conversion report dictionary."""
    source = _features(source_extract)
    converted = _features(converted_extract)
    decisions = compute_font_decisions(source)
    losses = compute_losses(source, converted, decisions)
    gate = compute_gate(source, converted, decisions)
    return {
        "schema": SCHEMA,
        "issue": ISSUE,
        "generator": GENERATOR,
        "synthetic_only": bool(synthetic_only),
        "source": {
            "name": source_name,
            "kind": source_extract["kind"],
            "features": dict(source),
        },
        "converted": {
            "name": converted_name,
            "kind": converted_extract["kind"],
            "features": dict(converted),
        },
        "losses": losses,
        "font_decisions": decisions,
        "gate": gate,
    }


def render_json(report: dict[str, Any]) -> str:
    """Serialize a report as stable, newline-terminated JSON."""
    return json.dumps(report, sort_keys=True, indent=2, ensure_ascii=True) + "\n"


def _is_shape(report: Any) -> bool:
    if not isinstance(report, dict):
        return False
    if report.get("schema") != SCHEMA:
        return False
    for side in ("source", "converted"):
        entry = report.get(side)
        if not isinstance(entry, dict):
            return False
        try:
            _features(entry)
        except (KeyError, TypeError, ValueError):
            return False
    for key in ("losses", "font_decisions", "gate"):
        if not isinstance(report.get(key), (list, dict)):
            return False
    if not isinstance(report["gate"], dict):
        return False
    return True


def validate(report: Any) -> tuple[bool, list[str], dict[str, Any] | None]:
    """Re-derive the report from its feature vectors and check consistency.

    Returns ``(shape_ok, errors, recomputed_gate)``. ``shape_ok`` is True only
    when the schema is valid AND every stored derived field matches the
    recomputed one. ``recomputed_gate`` is the authoritative gate verdict, even
    when the stored report disagrees with it.
    """
    errors: list[str] = []
    if not _is_shape(report):
        return False, ["invalid schema or feature vectors"], None
    source = _features(report["source"])
    converted = _features(report["converted"])
    decisions = compute_font_decisions(source)
    losses = compute_losses(source, converted, decisions)
    gate = compute_gate(source, converted, decisions)
    if report["losses"] != losses:
        errors.append("losses do not match recomputed structural losses")
    if report["font_decisions"] != decisions:
        errors.append("font_decisions do not match recomputed decisions")
    if report["gate"] != gate:
        errors.append("gate does not match recomputed verdict")
    return not errors, errors, gate
