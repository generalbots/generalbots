#!/usr/bin/env python3
"""CI guard for the GB6 feature record (#1649).

Fails when:
- a feature is missing from / extra to the published contract,
- a `shipped` feature has no end-to-end evidence,
- a referenced test file does not exist,
- the checked-in page differs from what the record generates,
- the removed features GB6-22 / GB6-24 reappear.

Usage:
    python3 scripts/gb6_features_check.py
    python3 scripts/gb6_features_check.py --record /tmp/bad.json
"""

from __future__ import annotations

import argparse
import importlib.util
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
RECORD = ROOT / "desktop" / "gb6" / "features.json"
PAGE = ROOT / "desktop" / "gb6" / "index.html"
GENERATOR = ROOT / "desktop" / "gb6" / "generate_features.py"

EXPECTED_IDS = {
    "GB6-08",
    "GB6-09",
    "GB6-10",
    "GB6-11",
    "GB6-12",
    "GB6-13",
    "GB6-14",
    "GB6-15",
    "GB6-16",
    "GB6-18",
    "GB6-19",
    "GB6-20",
    "GB6-25",
    "GB6-31",
}
REMOVED_IDS = {"GB6-22", "GB6-24"}
ALLOWED_STATUSES = {"shipped", "partial", "not_shipped"}


def load_generator():
    spec = importlib.util.spec_from_file_location("gb6_generate", GENERATOR)
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    spec.loader.exec_module(module)
    return module


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--record", type=Path, default=RECORD)
    parser.add_argument("--page", type=Path, default=PAGE)
    args = parser.parse_args()

    errors: list[str] = []
    try:
        with args.record.open(encoding="utf-8") as handle:
            record = json.load(handle)
    except (OSError, json.JSONDecodeError) as error:
        print(f"FAIL: cannot read {args.record}: {error}")
        return 1

    features = record.get("features")
    if not isinstance(features, list):
        print("FAIL: the record has no features array")
        return 1

    ids = [feature.get("id") for feature in features]
    if len(ids) != len(set(ids)):
        errors.append("duplicate feature ids in the record")
    missing = EXPECTED_IDS - set(ids)
    extra = set(ids) - EXPECTED_IDS
    if missing:
        errors.append(f"features missing from the record: {sorted(missing)}")
    if extra:
        errors.append(f"features not in the published contract: {sorted(extra)}")
    removed = REMOVED_IDS & set(ids)
    if removed:
        errors.append(f"removed features reappeared: {sorted(removed)}")

    for feature in features:
        fid = feature.get("id", "?")
        status = feature.get("status")
        if status not in ALLOWED_STATUSES:
            errors.append(f"{fid}: invalid status {status!r}")
            continue
        evidence = feature.get("evidence") or {}
        if status == "shipped" and not evidence.get("e2e"):
            errors.append(f"{fid}: shipped without an end-to-end evidence artifact")
        for test in evidence.get("tests") or []:
            if not (ROOT / test).exists():
                errors.append(f"{fid}: evidence test not found: {test}")
        last = feature.get("last_verified") or {}
        if status == "shipped" and not last.get("date"):
            errors.append(f"{fid}: shipped without a last_verified date")

    generator = load_generator()
    expected = generator.render(record)
    if not args.page.exists():
        errors.append(f"generated page missing: {args.page}")
    else:
        actual = args.page.read_text(encoding="utf-8")
        if actual != expected:
            errors.append(
                f"{args.page.relative_to(ROOT)} is stale: run "
                "python3 desktop/gb6/generate_features.py"
            )

    if errors:
        print("GB6 feature record: FAIL")
        for error in errors:
            print(f"  - {error}")
        return 1

    shipped = sum(1 for feature in features if feature.get("status") == "shipped")
    print(
        f"GB6 feature record: OK ({len(features)} features, {shipped} shipped, "
        f"page matches the record)"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
