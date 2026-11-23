#!/usr/bin/env python3
"""Offline generator and validation gate for the CSAF 2.0 advisory feed (#2714).

One source, three static surfaces (never re-authored by hand):

  * security/csaf/provider-metadata.json   CSAF 2.0 provider metadata
  * security/csaf/<year>/<GBSA-ID>.json    one CSAF advisory per canonical record
  * security/csaf/changes.csv              stable IDs and release timestamps

`check` is the gate: it regenerates in memory and fails when a published file is
missing, stale, hand-edited, carries an invented CVE or contains secret-like
material. No network calls and no third-party packages.

Usage:
    python3 scripts/security/csaf_sync.py generate
    python3 scripts/security/csaf_sync.py check
    python3 scripts/security/csaf_sync.py --self-test
Exit codes: 0 clean, 1 policy/generation failure, 2 usage error.
"""

from __future__ import annotations

import argparse
import csv
import io
import json
import sys
import tempfile
from pathlib import Path

from advisory_source import SourceError, load_bundle, public_base
from csaf_records import (
    CATEGORY,
    CHANGES_FILE,
    CSAFError,
    CSAF_VERSION,
    CVE_RE,
    METADATA_VERSION,
    PROVIDER_FILE,
    feed_files,
    generate,
    scan_secrets,
)

ROOT = Path(__file__).resolve().parents[2]
FEED_DIR = ROOT / "security/csaf"


def _read_json(payload: bytes, label: str) -> dict:
    try:
        document = json.loads(payload)
    except ValueError as error:
        raise CSAFError(f"{label}: invalid JSON: {error}") from error
    if not isinstance(document, dict):
        raise CSAFError(f"{label}: expected a JSON object")
    return document


def _check_structure(files: dict[str, bytes], bundle: dict) -> list[str]:
    """Validate the generated documents themselves, independent of disk state."""
    errors: list[str] = []
    meta = _read_json(files[PROVIDER_FILE], PROVIDER_FILE)
    required = ("canonical_url", "last_updated", "list_on_CSAF_aggregators",
                "mirror_on_CSAF_aggregators", "metadata_version", "publisher", "role",
                "distributions")
    errors.extend(f"{PROVIDER_FILE}: missing {key}" for key in required if key not in meta)
    if meta.get("metadata_version") != METADATA_VERSION:
        errors.append(f"{PROVIDER_FILE}: metadata_version is not {METADATA_VERSION}")
    if not str(meta.get("canonical_url", "")).endswith("/.well-known/csaf/provider-metadata.json"):
        errors.append(f"{PROVIDER_FILE}: canonical_url is not the well-known URL")
    if meta.get("role") != "csaf_provider":
        errors.append(f"{PROVIDER_FILE}: role must be csaf_provider")
    distributions = meta.get("distributions")
    if not isinstance(distributions, list) or not any(
        isinstance(item, dict) and item.get("directory_url") for item in distributions
    ):
        errors.append(f"{PROVIDER_FILE}: no directory_url distribution pointer")

    rows = list(csv.DictReader(io.StringIO(files[CHANGES_FILE].decode("utf-8"))))
    canonical_ids = sorted(record["id"] for record in bundle["advisories"])
    if sorted(row.get("id", "") for row in rows) != canonical_ids:
        errors.append(f"{CHANGES_FILE}: ids differ from the canonical advisory set")
    for row in rows:
        if not row.get("date") or not row.get("path"):
            errors.append(f"{CHANGES_FILE}: row has no date or path")

    for name, payload in files.items():
        if not name.endswith(".json") or name == PROVIDER_FILE:
            continue
        doc = _read_json(payload, name)
        tracking = doc.get("document", {}).get("tracking", {})
        for key in ("id", "status", "version", "initial_release_date",
                    "current_release_date", "revision_history"):
            if key not in tracking:
                errors.append(f"{name}: document.tracking.{key} missing")
        if tracking.get("id") not in canonical_ids:
            errors.append(f"{name}: tracking.id is not a canonical advisory id")
        if doc.get("document", {}).get("csaf_version") != CSAF_VERSION:
            errors.append(f"{name}: csaf_version is not {CSAF_VERSION}")
        if doc.get("document", {}).get("category") != CATEGORY:
            errors.append(f"{name}: document.category is not {CATEGORY}")
        branches = doc.get("product_tree", {}).get("branches", [])
        if not any(branch.get("branches") for branch in branches):
            errors.append(f"{name}: no affected product version ranges")
        vulnerabilities = doc.get("vulnerabilities", [])
        if not vulnerabilities:
            errors.append(f"{name}: no vulnerabilities")
            continue
        for vulnerability in vulnerabilities:
            if "cve" in vulnerability:
                value = vulnerability["cve"]
                if not isinstance(value, str) or not CVE_RE.fullmatch(value):
                    errors.append(f"{name}: invented/invalid CVE {value!r}")
            scores = vulnerability.get("scores", [])
            if not any("cvss_v3" in score or "cvss_v4_0" in score for score in scores):
                errors.append(f"{name}: no CVSS v3.1/v4.0 score")
            if not all(score.get("products") for score in scores):
                errors.append(f"{name}: score has no products")
            if not vulnerability.get("cwe"):
                errors.append(f"{name}: no CWE")
            if not vulnerability.get("remediations"):
                errors.append(f"{name}: no remediation")
            if not any(threat.get("category") == "exploit_status"
                       for threat in vulnerability.get("threats", [])):
                errors.append(f"{name}: no exploitability assessment")
        for secret in scan_secrets(payload):
            errors.append(f"{name}: generated file contains {secret}")
    return errors


def check(feed_dir: Path, bundle: dict, base: str) -> list[str]:
    """Compare the published feed with a fresh generation; fail closed."""
    try:
        files = feed_files(bundle, base)
    except CSAFError as error:
        return [f"generator: {error}"]
    errors = _check_structure(files, bundle)
    for name, payload in files.items():
        path = feed_dir / name
        if not path.is_file():
            errors.append(f"{name}: missing (run csaf_sync.py generate)")
        elif path.read_bytes() != payload:
            errors.append(f"{name}: stale or hand-edited (regenerate from the canonical bundle)")
    for directory in sorted(feed_dir.iterdir()) if feed_dir.is_dir() else []:
        if directory.is_dir() and directory.name.isdigit():
            for path in directory.glob("*.json"):
                if f"{directory.name}/{path.name}" not in files:
                    errors.append(f"{directory.name}/{path.name}: stale advisory not in canonical bundle")
    return errors


def self_test() -> int:
    """Offline proof: determinism, parity and fail-closed tamper detection."""
    try:
        bundle = load_bundle()
        base = public_base()
        with tempfile.TemporaryDirectory(prefix="gb-csaf-") as temp:
            feed = Path(temp) / "csaf"
            names = generate(feed, bundle, base)
            if not names:
                raise CSAFError("generate produced no artifacts")
            errors = check(feed, bundle, base)
            if errors:
                raise CSAFError("clean tree failed check: " + "; ".join(errors))
            first = (feed / names[0]).read_bytes()
            generate(feed, bundle, base)
            if (feed / names[0]).read_bytes() != first:
                raise CSAFError("generation is not deterministic")
            advisory = next(name for name in names if name.endswith(".json") and name != PROVIDER_FILE)
            path = feed / advisory
            tampered = _read_json(path.read_bytes(), advisory)
            tampered["vulnerabilities"][0]["cve"] = "CVE-2026-0000"
            path.write_text(json.dumps(tampered, indent=2, sort_keys=True) + "\n", encoding="utf-8")
            if not check(feed, bundle, base):
                raise CSAFError("invented CVE placeholder was not detected")
            generate(feed, bundle, base)
            path.unlink()
            if not check(feed, bundle, base):
                raise CSAFError("missing advisory was not detected")
            generate(feed, bundle, base)
            changes = feed / CHANGES_FILE
            changes.write_text(changes.read_text(encoding="utf-8") + "2026-01-01,GBFAKE,x.json\n",
                               encoding="utf-8")
            if not check(feed, bundle, base):
                raise CSAFError("stale changes.csv row was not detected")
    except (CSAFError, SourceError, OSError, ValueError, KeyError) as error:
        print(f"csaf_sync self-test FAILED: {error}", file=sys.stderr)
        return 1
    print(f"csaf_sync self-test: ok ({len(names)} artifacts, parity + fail-closed verified)")
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", nargs="?", choices=("generate", "check"))
    parser.add_argument("--feed-dir", type=Path, default=FEED_DIR)
    parser.add_argument("--base", default=public_base(), help="public bulletin origin")
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args(argv)
    try:
        if args.self_test:
            return self_test()
        if args.command is None:
            parser.error("a command (or --self-test) is required")
        bundle = load_bundle()
        if args.command == "generate":
            names = generate(args.feed_dir, bundle, args.base)
            print(f"generated {len(names)} CSAF artifacts in {args.feed_dir}")
            return 0
        errors = check(args.feed_dir, bundle, args.base)
        for error in errors:
            print(f"FAIL: {error}", file=sys.stderr)
        if errors:
            return 1
        print("check: ok (provider metadata, advisories, changes.csv, no invented CVE/secrets)")
        return 0
    except (CSAFError, SourceError, OSError, ValueError, KeyError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
