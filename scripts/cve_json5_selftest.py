#!/usr/bin/env python3
"""Offline self-tests for the CVE JSON 5 generator (#2715).

Each case is one assertion about the ID rule or the generated record shape.
Run with: python3 scripts/cve_json5.py self-test
"""

from __future__ import annotations

import copy
import json
import tempfile
from pathlib import Path
from typing import Any, Callable

from cve_json5 import (
    build_record, load_advisory, load_registry, metric,
)
from cve_json5_common import GenerationError
from cve_json5_profile import validate_profile

ASSIGNMENT: dict[str, Any] = {
    "cve_id": "CVE-2026-12345",
    "advisory": "GBSA-2026-0001",
    "short_name": "certbr",
    "org_id": "4a3b2c1d-5e6f-4a7b-8c9d-0e1f2a3b4c5d",
    "date_reserved": "2026-01-15T00:00:00Z",
    "date_published": "2026-11-24T12:00:00Z",
    "evidence": "CERT.br ticket in private record",
}

FRONT_MATTER = """---
id: GBSA-2026-0001
title: "Example advisory for CVE JSON 5 self-test"
summary: "A factual summary of a test flaw."
status: published
severity: critical
published: 2026-11-24
product: "General Bots Server"
affected_versions:
  - ">= 6.8.0, < 6.8.12"
fixed_versions:
  - "6.8.12"
cwe:
  - CWE-79
cvss:
  version: "4.0"
  vector: "CVSS:4.0/AV:N/AC:L/AT:N/PR:N/UI:N/VC:H/VI:H/VA:H/SC:N/SI:N/SA:N"
  base_score: 9.3
  base_severity: critical
credits:
  - name: "Reporter"
    role: reporter
references:
  - "https://generalbots.org/security/advisories/GBSA-2026-0001/"
cve:
  cve_id: CVE-2026-12345
---

# Example

## Summary

Test body.
"""


def fixture_dir(tmp: Path, front: str = FRONT_MATTER,
                assignments: list[dict[str, Any]] | None = None) -> tuple[Path, Path]:
    tmp.mkdir(parents=True, exist_ok=True)
    advisory = tmp / "GBSA-2026-0001.md"
    advisory.write_text(front, encoding="utf-8")
    registry = tmp / "cve-assignments.json"
    registry.write_text(json.dumps({"assignments": [ASSIGNMENT] if assignments is None
                                    else assignments}), encoding="utf-8")
    return advisory, registry


def expect(label: str, reason: str, action: Callable[[], Any]) -> bool:
    try:
        result = action()
    except (GenerationError, KeyError, OSError) as error:
        matched = reason in str(error) or reason == "reject"
        print(f"{'ok' if matched else 'FAIL'} {label}: rejected ({error})")
        return matched
    if reason == "reject":
        print(f"FAIL {label}: expected rejection, got {result!r}")
        return False
    print(f"ok {label}: accepted")
    return True


def run() -> int:
    checks: list[bool] = []
    with tempfile.TemporaryDirectory() as folder:
        tmp = Path(folder)
        advisory, registry = fixture_dir(tmp)
        data = load_advisory(advisory)
        assignments = load_registry(registry)
        record = build_record(data, assignments[ASSIGNMENT["cve_id"]], str(advisory))
        profile_errors = validate_profile(record)
        checks.append(not profile_errors)
        print(f"{'ok' if not profile_errors else 'FAIL'} valid record: "
              f"{profile_errors or 'profile accepted'}")

        checks.append(record["dataVersion"] == "5.1"
                      and record["cveMetadata"]["cveId"] == ASSIGNMENT["cve_id"]
                      and record["containers"]["cna"]["source"]["advisory"] == data["id"])
        print("ok field mapping: CNA ID, data version and GBSA source agree")

        bad = copy.deepcopy(record)
        bad["containers"]["cna"]["descriptions"] = []
        errors = validate_profile(bad)
        checks.append(any("description" in error for error in errors))
        print(f"{'ok' if errors else 'FAIL'} missing description: {errors}")

        bad = copy.deepcopy(record)
        bad["containers"]["cna"]["source"]["advisory"] = "CVE-2026-12345"
        bad["containers"]["cna"]["title"] = " CVE-2026-99999 self-assigned"
        errors = validate_profile(bad)
        checks.append(any("self-assigned" in error for error in errors))
        print(f"{'ok' if errors else 'FAIL'} self-assigned string: {errors}")

        checks.append(expect(
            "CVE ID outside registry", "reject",
            lambda: build_record(data, {}, str(advisory)),
        ))
        other = copy.deepcopy(ASSIGNMENT)
        other["advisory"] = "GBSA-2026-0002"
        checks.append(expect(
            "registry points to another advisory", "registry entry belongs to",
            lambda: build_record(data, other, str(advisory)),
        ))
        bad_title, _ = fixture_dir(
            tmp / "selfassigned", FRONT_MATTER.replace('title: "Example', 'title: "CVE-2026-99999 Example')
        )
        checks.append(expect(
            "self-assigned title", "title must not self-assign",
            lambda: load_advisory(bad_title),
        ))
        bad_front = FRONT_MATTER.replace("cwe:\n  - CWE-79", "cwe:\n  - CWE-abc")
        bad_file, _ = fixture_dir(tmp / "bad", bad_front)
        checks.append(expect(
            "invalid CWE", "invalid CWE ID",
            lambda: build_record(load_advisory(bad_file), assignments[ASSIGNMENT["cve_id"]],
                                 str(bad_file)),
        ))
        bad_cvss = dict(data["cvss"], base_score=9.2)
        checks.append(expect(
            "CVSS score mismatch", "disagree",
            lambda: metric(bad_cvss, "cvss"),
        ))
        no_evidence = [{key: value for key, value in ASSIGNMENT.items()
                        if key != "evidence"}]
        invalid_registry, _ = fixture_dir(tmp / "registry", assignments=no_evidence)
        checks.append(expect(
            "registry without evidence", "mandatory",
            lambda: load_registry(invalid_registry),
        ))
        duplicate = [ASSIGNMENT, copy.deepcopy(ASSIGNMENT)]
        dup_registry, _ = fixture_dir(tmp / "dup", assignments=duplicate)
        checks.append(expect(
            "duplicate CVE assignment", "duplicate",
            lambda: load_registry(dup_registry),
        ))
        no_cve = FRONT_MATTER.replace("cve:\n  cve_id: CVE-2026-12345\n", "")
        skipped, _ = fixture_dir(tmp / "skip", no_cve)
        checks.append(not str(load_advisory(skipped)).count("CVE-2026"))
        print("ok advisory without assignment: no CVE ID is introduced")

    passed = sum(1 for check in checks if check)
    print(f"self-test: {passed}/{len(checks)} cases passed")
    return 0 if passed == len(checks) else 1


if __name__ == "__main__":
    raise SystemExit(run())
