#!/usr/bin/env python3
"""Structural validation of the static CSAF 2.0 feed (#2714).

Checks fields the consumers depend on. Full OASIS schema validation is a CI
concern; this script intentionally needs no third-party packages or network.
"""
import csv
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
FEED = ROOT / "security" / "csaf"


def main() -> int:
    errors = []
    meta = json.loads((FEED / "provider-metadata.json").read_text(encoding="utf-8"))
    for key in ("metadata_version", "url", "publisher", "distribution", "categories"):
        if key not in meta:
            errors.append(f"provider-metadata.json: missing {key}")
    with (FEED / "changes.csv").open(encoding="utf-8", newline="") as handle:
        rows = list(csv.DictReader(handle))
    if not rows:
        errors.append("changes.csv: no advisories")
    for row in rows:
        for key in ("date", "id", "path"):
            if not row.get(key):
                errors.append(f"changes.csv: missing {key}")
        doc_path = FEED / row.get("path", "")
        if not doc_path.is_file():
            errors.append(f"changes.csv: missing document {doc_path}")
            continue
        doc = json.loads(doc_path.read_text(encoding="utf-8"))
        tracking = doc.get("document", {}).get("tracking", {})
        for key in ("id", "status", "version", "initial_release_date", "revision_history"):
            if key not in tracking:
                errors.append(f"{doc_path.name}: tracking.{key} missing")
        if tracking.get("id") != row.get("id"):
            errors.append(f"{doc_path.name}: tracking.id disagrees with changes.csv")
        vulnerabilities = doc.get("vulnerabilities", [])
        if not vulnerabilities:
            errors.append(f"{doc_path.name}: no vulnerabilities")
        for vulnerability in vulnerabilities:
            scores = vulnerability.get("scores", [])
            if not any("cvss_v4_0" in score or "cvss_v3_1" in score for score in scores):
                errors.append(f"{doc_path.name}: no CVSS v3.1/v4.0 score")
            if "cwe" not in vulnerability:
                errors.append(f"{doc_path.name}: no CWE")
    for error in errors:
        print(error)
    print(f"validate_csaf: {'FAIL' if errors else 'OK'} — {len(rows)} advisory row(s), {len(errors)} error(s)")
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
