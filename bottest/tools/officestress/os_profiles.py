#!/usr/bin/env python3
"""Memory profiles, case ceilings and the offline CI gate for #2531.

Pure data + pure functions: no I/O and no third-party imports. ``evaluate``
turns a *measured* report into a PASS/FAIL/SKIP verdict and never invents RSS
numbers. A report with status ``SKIP`` can only ever produce a ``SKIP`` verdict,
so an environment without a running server cannot be mistaken for a PASS.

The 2/4/8 GB profiles are cgroup class limits. Per-case single-import ceilings
come from ``ops/office-slo.yaml`` (the 4 GB reference) and scale linearly with
the profile so the same case can be admitted on a different class. The ratifed
numbers stay in this module only because ``ops/office-slo.yaml`` is declarative
and is validated separately by ``scripts/check_office_memory.py``; keep the two
in sync.
"""

from __future__ import annotations

from typing import Any

ISSUE = 2531
SCHEMA = 1

REFERENCE_PROFILE = "laptop-4gb"
REFERENCE_LIMIT_MB = 4096
PROFILES: dict[str, int] = {
    "laptop-2gb": 2048,
    "laptop-4gb": 4096,
    "laptop-8gb": 8192,
}

# Acceptance corpus: case id -> (generator kind, whole MB, 4 GB ceiling MiB).
CASE_SPECS: dict[str, tuple[str, int, int]] = {
    "docs-large": ("docx", 25, 900),
    "sheets-large": ("xlsx", 25, 1200),
    "slides-large": ("pptx", 50, 1100),
}

REGRESSION_PCT = 10.0
SOAK_CYCLES = 20
MAX_SOAK_CYCLES = 100
SOAK_RETURN_TOLERANCE_PCT = 10.0
MAX_SESSION_CACHE_GROWTH_MB = 16.0

NUMERIC_CASE_FIELDS = (
    "baseline_rss_mb", "peak_rss_mb", "return_rss_mb", "exit_code",
    "cycles", "session_cache_growth_mb", "events_with_content",
)
OPTIONAL_CASE_FIELDS = ("browser_heap_mb", "wire_bytes")

# Keys that must never appear in an observations payload: no document content,
# tenant names, paths or other PII may enter a report.
FORBIDDEN_OBSERVATION_KEYS = frozenset({
    "content", "text", "document", "body", "value", "cells", "paragraphs",
    "rows", "payload", "data", "snippet", "raw", "html", "xml",
})


def profile_names() -> tuple[str, ...]:
    """Return the declared profiles in ascending memory order."""
    return tuple(sorted(PROFILES, key=PROFILES.__getitem__))


def ceiling_for(case_id: str, profile: str) -> float:
    """Scale a 4 GB case ceiling to ``profile`` (rounded to whole MiB)."""
    if case_id not in CASE_SPECS:
        raise ValueError(f"unknown case {case_id!r}")
    if profile not in PROFILES:
        raise ValueError(f"unknown profile {profile!r}")
    base = CASE_SPECS[case_id][2]
    return float(round(base * PROFILES[profile] / REFERENCE_LIMIT_MB))


def profile_rows(peak_rss_mb: float) -> list[dict[str, Any]]:
    """Record an observed peak against every 2/4/8 GB profile."""
    rows: list[dict[str, Any]] = []
    for name in profile_names():
        limit = PROFILES[name]
        rows.append({
            "name": name,
            "limit_mb": limit,
            "observed_peak_mb": round(peak_rss_mb, 2),
            "fits_limit": peak_rss_mb <= limit,
            "headroom_mb": round(limit - peak_rss_mb, 2),
        })
    return rows


def _is_number(value: Any) -> bool:
    return isinstance(value, (int, float)) and not isinstance(value, bool)


def validate_observations(document: Any) -> list[str]:
    """Validate an external browser-heap/wire-bytes observations payload."""
    problems: list[str] = []
    if not isinstance(document, dict) or not isinstance(document.get("cases"), dict):
        return ["observations must be an object with a 'cases' object"]
    for case_id, values in document["cases"].items():
        if case_id not in CASE_SPECS:
            problems.append(f"observations: unknown case {case_id!r}")
        if not isinstance(values, dict):
            problems.append(f"observations: case {case_id!r} must be an object")
            continue
        forbidden = sorted(set(values) & FORBIDDEN_OBSERVATION_KEYS)
        if forbidden:
            problems.append(f"observations: case {case_id!r} has content/PII keys {forbidden}")
        extra = sorted(set(values) - set(OPTIONAL_CASE_FIELDS))
        if extra:
            problems.append(f"observations: case {case_id!r} has unknown keys {extra}")
        for field in OPTIONAL_CASE_FIELDS:
            if field in values and (not _is_number(values[field]) or values[field] < 0):
                problems.append(f"observations: {case_id}.{field} must be non-negative")
    return problems


def validate_report(report: Any) -> list[str]:
    """Return every schema problem; an empty list means the report is usable."""
    problems: list[str] = []
    if not isinstance(report, dict):
        return ["report must be an object"]
    if report.get("schema") != SCHEMA:
        problems.append(f"schema must be {SCHEMA}")
    if report.get("issue") != ISSUE:
        problems.append(f"issue must be {ISSUE}")
    if report.get("status") not in ("MEASURED", "SKIP"):
        problems.append("status must be MEASURED or SKIP")
    if report.get("profile") not in PROFILES:
        problems.append(f"profile must be one of {profile_names()}")
    if report.get("mode") not in ("measure", "soak"):
        problems.append("mode must be measure or soak")
    cases = report.get("cases")
    if not isinstance(cases, list):
        return problems + ["cases must be a list"]
    if report.get("status") == "SKIP" and cases:
        problems.append("a SKIP report must not carry measured cases")
    seen: set[str] = set()
    for case in cases:
        if not isinstance(case, dict):
            problems.append("every case must be an object")
            continue
        case_id = case.get("id")
        if case_id not in CASE_SPECS:
            problems.append(f"unknown case id {case_id!r}")
        elif case_id in seen:
            problems.append(f"duplicate case id {case_id!r}")
        else:
            seen.add(case_id)
        for field in NUMERIC_CASE_FIELDS:
            if not _is_number(case.get(field)):
                problems.append(f"{case_id}.{field} must be numeric")
        for field in OPTIONAL_CASE_FIELDS:
            value = case.get(field)
            if value is not None and not _is_number(value):
                problems.append(f"{case_id}.{field} must be numeric or null")
    return problems


def evaluate(report: Any, baseline: Any = None) -> dict[str, Any]:
    """Evaluate a report against ceilings, cgroup limits, soak and regression.

    ``baseline`` is an optional prior report dict (or ``{"cases": {id: peak}}``
    mapping). No baseline means the 10% regression criterion is UNVERIFIED, not
    PASS — it is reported as such without failing the run.
    """
    problems = validate_report(report)
    if problems:
        return {
            "verdict": "FAIL", "checked": 0, "cases": [], "breaches": [
                {"case": "-", "detail": problem} for problem in problems
            ], "regression": "UNVERIFIED",
        }
    if report["status"] == "SKIP":
        return {
            "verdict": "SKIP", "checked": 0, "cases": [], "breaches": [],
            "regression": "UNVERIFIED", "skip_reason": report.get("skip_reason", ""),
        }

    baseline_peaks = _baseline_peaks(baseline)
    profile = report["profile"]
    limit = PROFILES[profile]
    required_cycles = SOAK_CYCLES if report["mode"] == "soak" else 1
    breaches: list[dict[str, str]] = []
    results: list[dict[str, Any]] = []

    for case in report["cases"]:
        case_id = case["id"]
        ceiling = ceiling_for(case_id, profile)
        local: list[str] = []
        peak = float(case["peak_rss_mb"])
        if peak > ceiling:
            local.append(f"peak RSS {peak:.1f} MB exceeds ceiling {ceiling:.0f} MB")
        if peak > limit:
            local.append(f"peak RSS {peak:.1f} MB exceeds {profile} cgroup limit {limit} MB")
        if int(case["cycles"]) < required_cycles:
            local.append(f"only {int(case['cycles'])} cycles, need {required_cycles}")
        if report["mode"] == "soak":
            baseline_rss = float(case["baseline_rss_mb"])
            returned = float(case["return_rss_mb"])
            if baseline_rss <= 0:
                local.append("baseline_rss_mb must be positive")
            else:
                allowed = baseline_rss * (1 + SOAK_RETURN_TOLERANCE_PCT / 100.0)
                if returned > allowed:
                    local.append(
                        f"RSS returned to {returned:.1f} MB, over {allowed:.1f} MB "
                        f"({SOAK_RETURN_TOLERANCE_PCT:.0f}% of baseline)"
                    )
            growth = float(case["session_cache_growth_mb"])
            if growth > MAX_SESSION_CACHE_GROWTH_MB:
                local.append(f"session cache grew {growth:.1f} MB, over {MAX_SESSION_CACHE_GROWTH_MB} MB")
        if int(case["exit_code"]) != 0:
            local.append(f"workload exited with code {int(case['exit_code'])}")
        if case.get("error"):
            local.append(f"workload error: {case['error']}")
        if int(case["events_with_content"]) != 0:
            local.append("a structured event included document content")

        regression: float | None = None
        if baseline_peaks is not None and case_id in baseline_peaks:
            base_peak = baseline_peaks[case_id]
            if base_peak > 0:
                regression = (peak - base_peak) / base_peak * 100.0
                if regression > REGRESSION_PCT:
                    local.append(
                        f"peak regressed {regression:.1f}% over baseline "
                        f"(limit {REGRESSION_PCT:.0f}%)"
                    )

        if local:
            breaches.append({"case": case_id, "detail": "; ".join(local)})
        results.append({
            "case": case_id,
            "peak_rss_mb": round(peak, 2),
            "ceiling_mb": ceiling,
            "regression_pct": None if regression is None else round(regression, 2),
            "met": not local,
        })

    return {
        "verdict": "FAIL" if breaches else "PASS",
        "checked": len(results),
        "cases": results,
        "breaches": breaches,
        "regression": "ENFORCED" if baseline_peaks is not None else "UNVERIFIED",
    }


def _baseline_peaks(baseline: Any) -> dict[str, float] | None:
    if baseline is None:
        return None
    cases = baseline.get("cases") if isinstance(baseline, dict) else None
    if not isinstance(cases, dict):
        return None
    peaks: dict[str, float] = {}
    for case_id, value in cases.items():
        if isinstance(value, dict) and _is_number(value.get("peak_rss_mb")):
            peaks[case_id] = float(value["peak_rss_mb"])
        elif _is_number(value):
            peaks[case_id] = float(value)
    return peaks


def render(result: dict[str, Any]) -> str:
    """Human-readable gate output (no document content included)."""
    lines = [
        f"Office load-stress gate: verdict={result['verdict']} "
        f"checked={result['checked']} breaches={len(result['breaches'])} "
        f"regression={result['regression']}"
    ]
    for case in result["cases"]:
        status = "PASS" if case["met"] else "FAIL"
        regression = case["regression_pct"]
        suffix = "" if regression is None else f", regression {regression:.1f}%"
        lines.append(
            f"  {status} {case['case']}: peak {case['peak_rss_mb']} MB "
            f"(ceiling {case['ceiling_mb']:.0f} MB{suffix})"
        )
    for breach in result["breaches"]:
        lines.append(f"  BREACH {breach['case']}: {breach['detail']}")
    if result["verdict"] == "SKIP":
        lines.append(f"  SKIP reason: {result.get('skip_reason', 'runtime environment absent')}")
    return "\n".join(lines)
