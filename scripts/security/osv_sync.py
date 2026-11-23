#!/usr/bin/env python3
"""Offline CLI for the OSV + OpenVEX advisory feeds (#2718).

One source, four surfaces (never re-authored by hand):

  * security/osv/<GBSA-ID>.json   OSV record for each canonical advisory
  * security/osv/index.json       discovery index served at /security/osv/
  * security/vex/<GBVEX-ID>.json  OpenVEX statement per hand-authored claim
  * security/vex/index.json       discovery index for the VEX documents

The CSAF feed (#2714) and the OSV feed are both rendered from
`botlib/src/security/advisory/data/advisories.json`, so `check` proves the
1:1 ID parity required by #2718 instead of trusting a hand-maintained list.
NVD/CVE and OSV.dev submission stays a documented handoff (see the botbook
page); `emit-submission` only writes the payload a human or CI posts.

Usage:
    python3 scripts/security/osv_sync.py generate
    python3 scripts/security/osv_sync.py check
    python3 scripts/security/osv_sync.py emit-submission --out /tmp/osv-queue
    python3 scripts/security/osv_sync.py --self-test
Exit codes: 0 clean, 1 policy/generation failure, 2 usage error.
"""

from __future__ import annotations

import argparse
import json
import sys
import tempfile
from pathlib import Path

from advisory_source import SourceError, load_bundle, public_base
from osv_records import (
    OPENVEX_CONTEXT,
    OSV_SCHEMA,
    VEX_JUSTIFICATIONS,
    SyncError,
    _severity,
    _timestamp,
    _write,
    generate,
    load_vex_source,
)

ROOT = Path(__file__).resolve().parents[2]
OSV_DIR = ROOT / "security/osv"
VEX_DIR = ROOT / "security/vex"
VEX_SOURCE = VEX_DIR / "statements.json"
SYNC_DOC = ROOT / "botbook/src/09-security/advisory-feeds-osv-nvd.md"


def _read_json(path: Path) -> dict:
    try:
        document = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, ValueError) as error:
        raise SyncError(f"cannot read {path}: {error}") from error
    if not isinstance(document, dict):
        raise SyncError(f"{path}: expected a JSON object")
    return document


def check(osv_dir: Path, vex_dir: Path, bundle: dict, source: dict, doc: Path) -> list[str]:
    """Enforce every #2718 acceptance criterion against the generated files."""
    errors: list[str] = []
    canonical_ids = sorted(record["id"] for record in bundle["advisories"])
    index_path = osv_dir / "index.json"
    if not index_path.is_file():
        return ["security/osv/index.json is missing (run osv_sync.py generate)"]
    index = _read_json(index_path)
    if index.get("ids") != canonical_ids:
        errors.append("OSV index ids differ from the canonical/CSAF advisory ids")
    published_ids = sorted(path.stem for path in osv_dir.glob("GBSA-*.json") if path.is_file())
    if published_ids != canonical_ids:
        errors.append(f"OSV files {published_ids} do not match advisory ids {canonical_ids}")
    by_id = {record["id"]: record for record in bundle["advisories"]}
    for advisory_id in set(canonical_ids) & set(published_ids):
        path = osv_dir / f"{advisory_id}.json"
        document = _read_json(path)
        record = by_id[advisory_id]
        if document.get("id") != advisory_id:
            errors.append(f"{path.name}: embedded id does not match file name")
        if document.get("schema_version") != OSV_SCHEMA:
            errors.append(f"{path.name}: wrong OSV schema_version")
        if not document.get("affected"):
            errors.append(f"{path.name}: no affected package/ranges")
        if document.get("severity") != _severity(record):
            errors.append(f"{path.name}: severity/CVSS differs from the canonical advisory")
        modified = _timestamp(record.get("updated") or record["published"], "modified")
        if document.get("modified") != modified:
            errors.append(f"{path.name}: modified date is stale after an advisory revision")

    vex_ids = sorted(item["id"] for item in source["statements"])
    vex_index_path = vex_dir / "index.json"
    if not vex_index_path.is_file():
        errors.append("security/vex/index.json is missing (run osv_sync.py generate)")
    else:
        vex_index = _read_json(vex_index_path)
        if vex_index.get("ids") != vex_ids:
            errors.append("VEX index ids differ from the VEX source")
    vex_files = sorted(path.stem for path in vex_dir.glob("GBVEX-*.json") if path.is_file())
    if vex_files != vex_ids:
        errors.append(f"VEX files {vex_files} do not match VEX source ids {vex_ids}")
    by_vex_id = {item["id"]: item for item in source["statements"]}
    not_affected = 0
    for vex_id in set(vex_ids) & set(vex_files):
        path = vex_dir / f"{vex_id}.json"
        document = _read_json(path)
        statement = by_vex_id[vex_id]
        if document.get("@context") != OPENVEX_CONTEXT:
            errors.append(f"{path.name}: wrong OpenVEX @context")
        expected_updated = _timestamp(statement["updated"], "updated")
        if document.get("last_updated") != expected_updated:
            errors.append(f"{path.name}: last_updated is stale after a VEX revision")
        entries = document.get("statements", [])
        if len(entries) != 1:
            errors.append(f"{path.name}: expected exactly one OpenVEX statement")
            continue
        entry = entries[0]
        if (
            entry.get("status") != "not_affected"
            or entry.get("justification") not in VEX_JUSTIFICATIONS
        ):
            errors.append(f"{path.name}: invalid status/justification")
            continue
        if not isinstance(entry.get("impact_statement"), str) or not entry.get("products"):
            errors.append(f"{path.name}: missing impact statement or products")
            continue
        products = entry.get("database_specific", {}).get("products", [])
        if not products or not all(item.get("version_range") for item in products):
            errors.append(f"{path.name}: VEX must state precise product version ranges")
            continue
        not_affected += 1
    if not_affected < 1:
        errors.append("no not_affected VEX example covering a dependency CVE")

    if not doc.is_file() or "OSV" not in doc.read_text(encoding="utf-8"):
        errors.append(f"OSV/NVD sync procedure is missing at {doc}")
    return errors


def emit_submission(osv_dir: Path, out: Path, bundle: dict, base: str) -> list[str]:
    """Write the exact payload a human or CI job submits; never performs network I/O."""
    index = _read_json(osv_dir / "index.json")
    queue = {
        "schema": 1,
        "target": "https://osv.dev (partner/handoff channel)",
        "method": "POST",
        "note": "Payload only. Submission is performed by the security team or a scheduled CI job.",
        "base_url": base,
        "records": [
            _read_json(osv_dir / f"{advisory_id}.json") for advisory_id in index.get("ids", [])
        ],
    }
    out.mkdir(parents=True, exist_ok=True)
    _write(out, "submission.json", queue)
    return [str(out / "submission.json")]


def self_test() -> int:
    """Offline proof of parity, deterministic output and fail-closed checks."""
    try:
        bundle = load_bundle()
        source = load_vex_source(VEX_SOURCE)
        base = public_base()
        with tempfile.TemporaryDirectory(prefix="gb-osv-") as temp:
            root = Path(temp)
            osv_dir, vex_dir = root / "osv", root / "vex"
            doc = root / "sync.md"
            doc.write_text("# OSV and NVD sync procedure\n", encoding="utf-8")
            names = generate(osv_dir, vex_dir, bundle, source, base)
            if not names:
                raise SyncError("generate produced no artifacts")
            errors = check(osv_dir, vex_dir, bundle, source, doc)
            if errors:
                raise SyncError("clean tree failed check: " + "; ".join(errors))
            first_id = bundle["advisories"][0]["id"]
            first = _read_json(osv_dir / f"{first_id}.json")
            generate(osv_dir, vex_dir, bundle, source, base)
            if _read_json(osv_dir / f"{first_id}.json") != first:
                raise SyncError("generation is not deterministic")
            (osv_dir / f"{first_id}.json").unlink()
            if not check(osv_dir, vex_dir, bundle, source, doc):
                raise SyncError("missing OSV record was not detected")
            generate(osv_dir, vex_dir, bundle, source, base)
            vex_file = vex_dir / f"{source['statements'][0]['id']}.json"
            tampered = _read_json(vex_file)
            tampered["statements"][0]["justification"] = "not_a_real_justification"
            _write(vex_dir, vex_file.name, tampered)
            if not check(osv_dir, vex_dir, bundle, source, doc):
                raise SyncError("invalid VEX justification was not detected")
            out = emit_submission(osv_dir, root / "queue", bundle, base)
            if not _read_json(Path(out[0]))["records"]:
                raise SyncError("submission payload has no records")
    except (SyncError, SourceError, OSError, ValueError, KeyError) as error:
        print(f"osv_sync self-test FAILED: {error}", file=sys.stderr)
        return 1
    print(f"osv_sync self-test: ok ({len(names)} artifacts, parity + fail-closed verified)")
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", nargs="?", choices=("generate", "check", "emit-submission"))
    parser.add_argument("--osv-dir", type=Path, default=OSV_DIR)
    parser.add_argument("--vex-dir", type=Path, default=VEX_DIR)
    parser.add_argument("--vex-source", type=Path, default=VEX_SOURCE)
    parser.add_argument("--sync-doc", type=Path, default=SYNC_DOC)
    parser.add_argument("--base", default=public_base(), help="public bulletin origin")
    parser.add_argument("--out", type=Path, help="output directory for emit-submission")
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args(argv)
    try:
        if args.self_test:
            return self_test()
        if args.command is None:
            parser.error("a command (or --self-test) is required")
        bundle = load_bundle()
        source = load_vex_source(args.vex_source)
        if args.command == "generate":
            names = generate(args.osv_dir, args.vex_dir, bundle, source, args.base)
            print(f"generated {len(names)} OSV/VEX artifacts")
            return 0
        if args.command == "check":
            errors = check(args.osv_dir, args.vex_dir, bundle, source, args.sync_doc)
            if errors:
                for error in errors:
                    print(f"FAIL: {error}", file=sys.stderr)
                return 1
            print("check: ok (ID parity, VEX example, sync procedure)")
            return 0
        if args.out is None:
            parser.error("emit-submission requires --out")
        written = emit_submission(args.osv_dir, args.out, bundle, args.base)
        print(f"wrote {written[0]}")
        return 0
    except (SyncError, SourceError, OSError, ValueError, KeyError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
