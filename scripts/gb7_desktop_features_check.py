#!/usr/bin/env python3
"""CI guard for the GB7 feature record (#1649).

Fails when:
- a feature is missing from / extra to the published contract,
- a `shipped` feature has no end-to-end evidence,
- a referenced test file does not exist,
- the checked-in page differs from what the record generates,
- the removed features GB7-22 / GB7-24 reappear.

Usage:
    python3 scripts/gb7_desktop_features_check.py
    python3 scripts/gb7_desktop_features_check.py --record /tmp/bad.json
"""

from __future__ import annotations

import argparse
import importlib.util
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
RECORD = ROOT / "desktop" / "gb7" / "features.json"
PAGE = ROOT / "desktop" / "gb7" / "index.html"
GENERATOR = ROOT / "desktop" / "gb7" / "generate_features.py"

EXPECTED_IDS = {
    "GB7-08",
    "GB7-09",
    "GB7-10",
    "GB7-11",
    "GB7-12",
    "GB7-13",
    "GB7-14",
    "GB7-15",
    "GB7-16",
    "GB7-18",
    "GB7-19",
    "GB7-20",
    "GB7-25",
    "GB7-31",
}
REMOVED_IDS = {"GB7-22", "GB7-24"}
ALLOWED_STATUSES = {"shipped", "partial", "not_shipped"}


def load_generator():
    spec = importlib.util.spec_from_file_location("gb7_generate", GENERATOR)
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    spec.loader.exec_module(module)
    return module


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--record", type=Path, default=RECORD)
    parser.add_argument("--page", type=Path, default=PAGE)
    parser.add_argument("--self-test", action="store_true", help="prove the gate fails closed")
    args = parser.parse_args()

    if args.self_test:
        from gate_selftests import run_self_test

        return run_self_test("gb7-desktop-features")

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
        if not evidence.get("issue"):
            errors.append(f"{fid}: no issue links this feature to the epic")
        if status == "shipped" and not evidence.get("e2e"):
            errors.append(f"{fid}: shipped without an executed-test evidence artifact")
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
                f"{args.page} is stale: run "
                "python3 desktop/gb7/generate_features.py"
            )

    if errors:
        print("GB7 feature record: FAIL")
        for error in errors:
            print(f"  - {error}")
        return 1

    epic = record.get("epic") or {}
    if not epic.get("issue"):
        errors.append("record has no parent epic")
    for removed in record.get("epic", {}).get("removed_from_scope", []):
        if removed in ids:
            errors.append(f"{removed} is removed from scope but listed")

    shipped = sum(1 for feature in features if feature.get("status") == "shipped")
    print(
        f"GB7 feature record: OK ({len(features)} features, {shipped} shipped, "
        f"page matches the record)"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
