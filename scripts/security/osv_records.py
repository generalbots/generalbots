#!/usr/bin/env python3
"""Render OSV and OpenVEX records from the canonical advisory bundle (#2718).

Pure library: no CLI, no network, no writes outside the directories passed to
`generate()`. The CLI lives in `osv_sync.py`.
"""

from __future__ import annotations

import json
from pathlib import Path

OSV_SCHEMA = "1.4.0"
OPENVEX_CONTEXT = "https://openvex.dev/ns/v0.2.0"
VEX_JUSTIFICATIONS = frozenset({
    "component_not_present",
    "vulnerable_code_not_present",
    "vulnerable_code_not_in_execute_path",
    "vulnerable_code_cannot_be_controlled_by_adversary",
    "inline_mitigations_already_exist",
})
DATE_LEN = 10


class SyncError(ValueError):
    """Any generation or policy failure; always fails closed."""


def _date(value: str, label: str) -> str:
    if not isinstance(value, str) or len(value) != DATE_LEN:
        raise SyncError(f"{label}: expected YYYY-MM-DD, got {value!r}")
    try:
        year, month, day = (int(part) for part in value.split("-"))
    except ValueError as error:
        raise SyncError(f"{label}: invalid date {value!r}") from error
    if not (1970 <= year <= 9999 and 1 <= month <= 12 and 1 <= day <= 31):
        raise SyncError(f"{label}: out-of-range date {value!r}")
    return value


def _timestamp(value: str, label: str) -> str:
    return f"{_date(value, label)}T00:00:00Z"


def _safe_id(value: str, label: str) -> str:
    if not isinstance(value, str) or not value:
        raise SyncError(f"{label}: empty identifier")
    if any(char in value for char in ("/", "\\", "\x00", "..")):
        raise SyncError(f"{label}: unsafe identifier {value!r}")
    if not all(char.isascii() and (char.isalnum() or char in "-._") for char in value):
        raise SyncError(f"{label}: identifier outside the OSV token alphabet: {value!r}")
    return value


def _severity(record: dict) -> list[dict]:
    entries = []
    for item in record.get("cvss", []):
        vector = item.get("vector", "")
        version = item.get("version", "")
        if not isinstance(vector, str) or not vector.startswith("CVSS:"):
            raise SyncError(f"{record['id']}: invalid CVSS vector")
        kind = "CVSS_V4" if version == "4.0" else "CVSS_V3" if version in ("3.0", "3.1") else None
        if kind is None:
            raise SyncError(f"{record['id']}: unsupported CVSS version {version!r}")
        entries.append({"type": kind, "score": vector})
    return entries


def _ranges(record: dict) -> list[dict]:
    ranges = []
    for item in record.get("affected_versions", []):
        introduced = item.get("introduced")
        fixed = item.get("fixed")
        if introduced is None and fixed is None and not item.get("range"):
            raise SyncError(f"{record['id']}: affected version has no bound")
        events = [{"introduced": introduced if introduced is not None else "0"}]
        if fixed is not None:
            events.append({"fixed": fixed})
        entry = {"type": "SEMVER", "events": events}
        if item.get("range"):
            entry["database_specific"] = {"authored_range": item["range"]}
        ranges.append(entry)
    if not ranges:
        raise SyncError(f"{record['id']}: no affected version ranges")
    return ranges


def _references(record: dict, base: str) -> list[dict]:
    canonical = f"{base}/advisories/{record['id']}"
    urls = list(record.get("references", []))
    if canonical not in urls:
        urls.insert(0, canonical)
    refs = [{"type": "ADVISORY", "url": canonical}]
    refs.extend({"type": "WEB", "url": url} for url in urls if url != canonical)
    return refs


def build_osv(record: dict, base: str) -> dict:
    """Render one canonical advisory as an OSV 1.4 record."""
    advisory_id = _safe_id(record["id"], "advisory id")
    published = _timestamp(record["published"], f"{advisory_id}.published")
    modified = _timestamp(record.get("updated") or record["published"], f"{advisory_id}.updated")
    document = {
        "schema_version": OSV_SCHEMA,
        "id": advisory_id,
        "published": published,
        "modified": modified,
        "summary": record["title"],
        "details": record.get("summary") or record["title"],
        "severity": _severity(record),
        "affected": [
            {
                "package": {"ecosystem": "GeneralBots", "name": record["component"]},
                "ranges": _ranges(record),
                "versions": [],
                "database_specific": {"product": record["product"], "severity": record["severity"]},
            }
        ],
        "references": _references(record, base),
        "database_specific": {
            "source": "General Bots Security Bulletin",
            "cwe": list(record.get("cwe", [])),
            "canonical": f"{base}/advisories/{advisory_id}",
            "remediation": record["remediation"]["recommended_version"],
        },
    }
    if record.get("updated"):
        document["database_specific"]["revisions"] = [
            {"date": item["date"], "summary": item["summary"]}
            for item in record.get("revisions", [])
        ]
    return document


def load_vex_source(path: Path) -> dict:
    """Load and validate the hand-authored VEX claim list (never a bare KeyError)."""
    try:
        raw = path.read_text(encoding="utf-8")
    except OSError as error:
        raise SyncError(f"cannot read VEX source: {error}") from error
    try:
        source = json.loads(raw)
    except ValueError as error:
        raise SyncError(f"VEX source is not JSON: {error}") from error
    if not isinstance(source, dict) or not isinstance(source.get("statements"), list):
        raise SyncError("VEX source has no statements array")
    if not source["statements"]:
        raise SyncError("VEX source has no statements")
    for statement in source["statements"]:
        if not isinstance(statement, dict):
            raise SyncError("VEX statement is not an object")
        statement_id = _safe_id(statement.get("id", ""), "VEX id")
        cve = statement.get("cve", "")
        if not isinstance(cve, str) or not cve.startswith("CVE-"):
            raise SyncError(f"{statement_id}: missing CVE identifier")
        if statement.get("status") != "not_affected":
            raise SyncError(f"{statement_id}: only not_affected claims are published")
        if statement.get("justification") not in VEX_JUSTIFICATIONS:
            raise SyncError(f"{statement_id}: invalid justification")
        if not isinstance(statement.get("reason"), str) or len(statement["reason"].strip()) < 20:
            raise SyncError(f"{statement_id}: reason is too short to be meaningful")
        _date(statement.get("published", ""), f"{statement_id}.published")
        _date(statement.get("updated", ""), f"{statement_id}.updated")
        products = statement.get("products")
        if not isinstance(products, list) or not products:
            raise SyncError(f"{statement_id}: no products")
        for product in products:
            if not isinstance(product, dict) or not product.get("name"):
                raise SyncError(f"{statement_id}: invalid product")
            if not isinstance(product.get("purl"), str) or not product["purl"].startswith("pkg:"):
                raise SyncError(f"{statement_id}: product needs a purl")
            if not isinstance(product.get("version_range"), str) or not product["version_range"].strip():
                raise SyncError(f"{statement_id}: product needs a precise version range")
    return source


def build_openvex(statement: dict, base: str) -> dict:
    """Render one VEX claim as an OpenVEX document."""
    statement_id = _safe_id(statement["id"], "VEX id")
    published = _timestamp(statement["published"], f"{statement_id}.published")
    updated = _timestamp(statement["updated"], f"{statement_id}.updated")
    products = statement["products"]
    scopes = "; ".join(
        f"{item['name']} {item['version_range']} ({item['purl']})" for item in products
    )
    return {
        "@context": OPENVEX_CONTEXT,
        "@id": f"{base}/vex/{statement_id}",
        "author": "General Bots Security Team <security@generalbots.org>",
        "timestamp": published,
        "last_updated": updated,
        "version": int(statement["revision"]),
        "statements": [
            {
                "@id": f"{base}/vex/{statement_id}#{statement['cve']}",
                "vulnerability": {
                    "@id": f"https://nvd.nist.gov/vuln/detail/{statement['cve']}",
                    "name": statement["cve"],
                    "description": statement["description"],
                },
                "products": [item["purl"] for item in products],
                "status": statement["status"],
                "justification": statement["justification"],
                "impact_statement": f"{statement['reason']} Precise scope: {scopes}.",
                "database_specific": {
                    "title": statement["title"],
                    "products": products,
                    "references": list(statement.get("references", [])),
                },
            }
        ],
    }


def _osv_index(bundle: dict, base: str) -> dict:
    records = sorted(bundle["advisories"], key=lambda record: record["id"])
    return {
        "schema": 1,
        "source": "botlib/src/security/advisory/data/advisories.json",
        "format": "OSV",
        "schema_version": OSV_SCHEMA,
        "base_url": f"{base}/osv/",
        "count": len(records),
        "ids": [record["id"] for record in records],
        "modified": max(
            (record.get("updated") or record["published"] for record in records), default="1970-01-01"
        ),
    }


def _vex_index(source: dict, base: str) -> dict:
    statements = sorted(source["statements"], key=lambda item: item["id"])
    return {
        "schema": 1,
        "format": "OpenVEX",
        "context": OPENVEX_CONTEXT,
        "base_url": f"{base}/vex/",
        "count": len(statements),
        "ids": [item["id"] for item in statements],
        "cves": [item["cve"] for item in statements],
    }


def _write(directory: Path, name: str, document: dict) -> None:
    _safe_id(name, "output file name")
    directory.mkdir(parents=True, exist_ok=True)
    target = directory / name
    payload = (json.dumps(document, indent=2, sort_keys=True, ensure_ascii=False) + "\n").encode("utf-8")
    temp = target.with_suffix(target.suffix + ".tmp")
    temp.write_bytes(payload)
    temp.replace(target)


def generate(osv_dir: Path, vex_dir: Path, bundle: dict, source: dict, base: str) -> list[str]:
    """Write every OSV/VEX artifact atomically; validate all before writing."""
    osv_records = {record["id"]: build_osv(record, base) for record in bundle["advisories"]}
    vex_records = {item["id"]: build_openvex(item, base) for item in source["statements"]}
    if len(osv_records) != len(bundle["advisories"]):
        raise SyncError("duplicate advisory ids in canonical bundle")
    if len(vex_records) != len(source["statements"]):
        raise SyncError("duplicate VEX ids in VEX source")
    for advisory_id, document in osv_records.items():
        if document["id"] != advisory_id:
            raise SyncError(f"{advisory_id}: OSV id drift")
    files = []
    for advisory_id, document in osv_records.items():
        _write(osv_dir, f"{advisory_id}.json", document)
        files.append(f"osv/{advisory_id}.json")
    _write(osv_dir, "index.json", _osv_index(bundle, base))
    files.append("osv/index.json")
    for vex_id, document in vex_records.items():
        _write(vex_dir, f"{vex_id}.json", document)
        files.append(f"vex/{vex_id}.json")
    _write(vex_dir, "index.json", _vex_index(source, base))
    files.append("vex/index.json")
    return sorted(files)
