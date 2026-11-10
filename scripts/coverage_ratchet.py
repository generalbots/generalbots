#!/usr/bin/env python3
"""Committed coverage baseline for the ratchet (#2119).

`scripts/check_coverage.py` owns the floors and the CLI; this module owns the
*baseline* it compares against. A floor is a constant, a ratchet is a promise:
once a crate has been measured at N%, a PR that lands N-1% must fail even while
N-1% is still above the floor.

The baseline lives in `issues/coverage/baseline.json` — data next to the other
committed quality baselines, never a repo-root file:

    {
      "meta": {"issue": 2119, ...},
      "crates": {"botlib": {"lines": 80.0, "functions": 80.0, "branches": 70.0}}
    }

`record_baseline()` is the only writer and it takes `max(stored, measured)` per
metric, so a regenerated baseline can never lower the bar (a silent reset to a
green number is exactly the failure mode this gate exists to prevent).
"""

from __future__ import annotations

import json
from pathlib import Path

METRICS = ("lines", "functions", "branches")


class BaselineError(Exception):
    """The baseline file cannot be read or is not a valid ratchet."""


def load_baseline(path: Path) -> dict:
    """Returns `{crate: {metric: percent}}`; a missing file is an empty ratchet."""
    if not path.is_file():
        return {}
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise BaselineError(f"{path}: {error}") from error
    crates = payload.get("crates")
    if not isinstance(crates, dict):
        raise BaselineError(f"{path}: missing the `crates` object")
    parsed = {}
    for crate, entry in crates.items():
        if not isinstance(entry, dict):
            raise BaselineError(f"{path}: `{crate}` must be an object")
        values = {}
        for metric in METRICS:
            value = entry.get(metric)
            if not isinstance(value, (int, float)) or not 0 <= value <= 100:
                raise BaselineError(f"{path}: `{crate}.{metric}` must be a percentage")
            values[metric] = float(value)
        parsed[crate] = values
    return parsed


def compare_baseline(crate: str, result: dict, baseline: dict) -> list[str]:
    """Every metric that moved *down* since the baseline was recorded."""
    recorded = baseline.get(crate)
    if recorded is None:
        return ["no baseline entry"]
    drops = []
    for metric in METRICS:
        current = result["percent"][metric]
        if current is None:
            continue
        if current < recorded[metric]:
            drops.append(
                f"{metric} {current:.2f}% is below the {recorded[metric]:.2f}% baseline"
            )
    return drops


def record_baseline(path: Path, crate: str, result: dict) -> dict:
    """Raises the stored value for `crate`; never lowers it (the ratchet)."""
    baseline = load_baseline(path)
    current = baseline.get(crate, {})
    updated = {}
    for metric in METRICS:
        measured = result["percent"][metric]
        previous = current.get(metric, 0.0)
        if measured is None:
            updated[metric] = round(previous, 2)
        else:
            updated[metric] = round(max(previous, measured), 2)
    baseline[crate] = updated
    path.parent.mkdir(parents=True, exist_ok=True)
    payload = {
        "meta": {
            "issue": 2119,
            "tool": "cargo-llvm-cov",
            "note": (
                "Ratchet baseline: scripts/check_coverage.py fails when a crate "
                "measures below these values. Raise with --record-baseline; the "
                "writer never lowers a value."
            ),
        },
        "crates": {key: baseline[key] for key in sorted(baseline)},
    }
    path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    print(f"baseline: recorded {crate} {updated}")
    return updated
