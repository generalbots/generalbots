#!/usr/bin/env python3
"""Render CSAF 2.0 provider metadata and advisories from the canonical bundle (#2714).

Pure library: no CLI, no network and no writes outside the directory passed to
`generate()`. The CLI and the offline validation gate live in `csaf_sync.py`.

One advisory source (`botlib/src/security/advisory/data/advisories.json`)
produces every CSAF artifact, so IDs, CVSS vectors and version ranges can never
drift from the bulletin. No CVE is ever synthesized: the canonical records use
`GBSA-YYYY-NNNN` IDs, which CSAF carries in `vulnerabilities[].ids`.
"""

from __future__ import annotations

import json
import re
from pathlib import Path
from urllib.parse import urlsplit

CSAF_VERSION = "2.0"
METADATA_VERSION = "2.0"
CATEGORY = "csaf_security_advisory"
PROVIDER_FILE = "provider-metadata.json"
CHANGES_FILE = "changes.csv"
PUBLISHER = {
    "category": "vendor",
    "name": "General Bots",
    "namespace": "https://generalbots.org",
    "contact_details": "security@generalbots.org",
}
TLP = {"label": "WHITE"}
ADVISORY_ID_RE = re.compile(r"^GBSA-[0-9]{4}-[0-9]{4}$")
CVE_RE = re.compile(r"^CVE-[0-9]{4}-[0-9]{4,}$")
PLACEHOLDER_CVE_RE = re.compile(r"^CVE-[0-9]{4}-0{4,}$")
CWE_NAMES = {
    "CWE-22": "Improper Limitation of a Pathname to a Restricted Directory ('Path Traversal')",
    "CWE-79": "Improper Neutralization of Input During Web Page Generation ('Cross-site Scripting')",
    "CWE-200": "Exposure of Sensitive Information to an Unauthorized Actor",
}
# CVSS v3.1 metric abbreviations -> FIRST schema field names (derived from the
# vector only; no impact or exploitability claim is invented here).
V3_METRICS = {
    "AV": ("attackVector", {"N": "NETWORK", "A": "ADJACENT_NETWORK", "L": "LOCAL", "P": "PHYSICAL"}),
    "AC": ("attackComplexity", {"L": "LOW", "H": "HIGH"}),
    "PR": ("privilegesRequired", {"N": "NONE", "L": "LOW", "H": "HIGH"}),
    "UI": ("userInteraction", {"N": "NONE", "R": "REQUIRED"}),
    "S": ("scope", {"U": "UNCHANGED", "C": "CHANGED"}),
    "C": ("confidentialityImpact", {"N": "NONE", "L": "LOW", "H": "HIGH"}),
    "I": ("integrityImpact", {"N": "NONE", "L": "LOW", "H": "HIGH"}),
    "A": ("availabilityImpact", {"N": "NONE", "L": "LOW", "H": "HIGH"}),
}
SECRET_PATTERNS = (
    (re.compile(rb"-----BEGIN (?:RSA |EC |OPENSSH |DSA )?PRIVATE KEY-----"), "private key block"),
    (re.compile(rb"\bAKIA[0-9A-Z]{16}\b"), "AWS access key id"),
    (re.compile(rb"\bgh[pousr]_[A-Za-z0-9]{20,}\b"), "GitHub token"),
    (re.compile(rb"\bxox[baprs]-[A-Za-z0-9-]{10,}\b"), "Slack token"),
    (
        re.compile(rb"(?i)\b(?:api[_-]?key|password|secret|authorization)\b\s*[:=]\s*[\"'][^\"']{8,}[\"']"),
        "credential field",
    ),
    (re.compile(rb"\b(?:10|192\.168|172\.(?:1[6-9]|2[0-9]|3[01]))\.\d{1,3}\.\d{1,3}\b"), "private IP"),
)


class CSAFError(ValueError):
    """Any generation or policy failure; always fails closed."""


def _origin(base: str) -> str:
    parts = urlsplit(base)
    if parts.scheme not in ("http", "https") or not parts.netloc:
        raise CSAFError(f"bulletin base must be an http(s) URL: {base!r}")
    return f"{parts.scheme}://{parts.netloc}"


def _timestamp(value: str, label: str) -> str:
    if not isinstance(value, str) or not re.fullmatch(r"[0-9]{4}-[0-9]{2}-[0-9]{2}", value):
        raise CSAFError(f"{label}: expected YYYY-MM-DD, got {value!r}")
    return f"{value}T00:00:00Z"


def _advisory_id(record: dict) -> str:
    value = record.get("id", "")
    if not isinstance(value, str) or not ADVISORY_ID_RE.fullmatch(value):
        raise CSAFError(f"advisory id outside the GBSA token alphabet: {value!r}")
    return value


def _cwe(identifiers: list[str], label: str) -> dict:
    if not identifiers:
        raise CSAFError(f"{label}: no CWE recorded in the canonical advisory")
    first = identifiers[0]
    if not isinstance(first, str) or not re.fullmatch(r"CWE-[1-9][0-9]{0,5}", first):
        raise CSAFError(f"{label}: invalid CWE {first!r}")
    return {"id": first, "name": CWE_NAMES.get(first, first)}


def _severity(score: float) -> str:
    if score == 0.0:
        return "NONE"
    if score < 4.0:
        return "LOW"
    if score < 7.0:
        return "MEDIUM"
    if score < 9.0:
        return "HIGH"
    return "CRITICAL"


def _cvss(record: dict, product_ids: list[str]) -> tuple[dict, str]:
    """Return the CSAF score entry (or a v4 extension) and a derived note."""
    advisory_id = _advisory_id(record)
    entries = record.get("cvss", [])
    if not entries:
        raise CSAFError(f"{advisory_id}: no CVSS metric in the canonical advisory")
    metric = entries[0]
    version, vector, score = metric.get("version"), metric.get("vector", ""), metric.get("score")
    if version not in ("3.1", "4.0"):
        raise CSAFError(f"{advisory_id}: unsupported CVSS version {version!r}")
    if not isinstance(vector, str) or not vector.startswith(f"CVSS:{version}/"):
        raise CSAFError(f"{advisory_id}: CVSS vector disagrees with version {version}")
    if not isinstance(score, (int, float)) or isinstance(score, bool) or not 0 <= score <= 10:
        raise CSAFError(f"{advisory_id}: CVSS score out of range: {score!r}")
    value = {"version": version, "vectorString": vector, "baseScore": score,
             "baseSeverity": _severity(score)}
    if version == "3.1":
        for part in vector.split("/")[1:]:
            short, _, suffix = part.partition(":")
            if short in V3_METRICS:
                field, allowed = V3_METRICS[short]
                if suffix not in allowed:
                    raise CSAFError(f"{advisory_id}: invalid CVSS metric {part}")
                value[field] = allowed[suffix]
        key = "cvss_v3"
    else:
        # CSAF 2.0 predates CVSS v4.0: publish the verbatim vector as a
        # forward-compatible extension and repeat it in a note so strict
        # consumers that ignore extensions still see the metric.
        key = "cvss_v4_0"
    metrics = ", ".join(part for part in vector.split("/")[1:] if part.split(":")[0] in
                        ("AV", "AC", "AT", "PR", "UI", "VC", "VI", "VA"))
    note = (f"CVSS v{version} vector {vector} (base score {score}, {value['baseSeverity']}). "
            f"Base metrics: {metrics}.")
    return {"products": list(product_ids), key: value}, note


def _product_tree(record: dict, advisory_id: str) -> tuple[dict, list[str], list[str]]:
    product = record["product"]
    branches, affected, fixed = [], [], []
    for index, item in enumerate(record.get("affected_versions", []), 1):
        if not isinstance(item, dict) or not any(key in item for key in ("introduced", "fixed", "range")):
            raise CSAFError(f"{advisory_id}: affected version has no bound")
        name = item.get("range") or f">= {item.get('introduced', '0')}, < {item.get('fixed', '*')}"
        affected_id = f"CSAFPID-{index:04d}"
        affected.append(affected_id)
        branches.append({
            "category": "product_name",
            "name": product,
            "branches": [{
                "category": "product_version_range",
                "name": name,
                "product": {"name": f"{product} {name}", "product_id": affected_id},
            }],
        })
    if not branches:
        raise CSAFError(f"{advisory_id}: no affected products in the canonical advisory")
    recommended = record["remediation"]["recommended_version"]
    fixed_id = f"CSAFPID-{len(branches) + 1:04d}"
    branches.append({
        "category": "product_name",
        "name": product,
        "branches": [{
            "category": "product_version",
            "name": recommended,
            "product": {"name": f"{product} {recommended}", "product_id": fixed_id},
        }],
    })
    fixed.append(fixed_id)
    return {"branches": branches}, affected, fixed


def _cve(record: dict, advisory_id: str) -> str | None:
    """Emit a CVE only when the canonical record explicitly carries a real one."""
    value = record.get("cve")
    if value is None:
        return None
    if not isinstance(value, str) or not CVE_RE.fullmatch(value) or PLACEHOLDER_CVE_RE.fullmatch(value):
        raise CSAFError(f"{advisory_id}: invalid CVE alias {value!r} in canonical advisory")
    return value


def build_advisory(record: dict, base: str) -> tuple[dict, str]:
    """Render one canonical advisory as a CSAF 2.0 document and its index row."""
    advisory_id = _advisory_id(record)
    published = _timestamp(record["published"], f"{advisory_id}.published")
    updated = _timestamp(record.get("updated") or record["published"], f"{advisory_id}.updated")
    revisions = record.get("revisions") or [{"date": record["published"], "summary": "Initial publication"}]
    tree, affected, fixed = _product_tree(record, advisory_id)
    score, cvss_note = _cvss(record, affected)
    cwe = _cwe(record.get("cwe", []), advisory_id)
    notes = [{"category": "description", "title": "Summary",
              "text": record.get("summary") or record["title"]},
             {"category": "details", "title": "CVSS vector", "text": cvss_note}]
    extra_cwes = record.get("cwe", [])[1:]
    if extra_cwes:
        notes.append({"category": "details", "title": "Additional CWEs",
                      "text": ", ".join(extra_cwes)})
    vulnerability = {
        "ids": [{"system_name": "General Bots Security Advisory", "text": advisory_id}],
        "notes": notes,
        "product_status": {"known_affected": affected, "recommended": fixed},
        "remediations": [{
            "category": "vendor_fix",
            "details": record["remediation"]["text"],
            "product_ids": affected,
        }],
        "scores": [score],
        "cwe": cwe,
        "threats": [{
            "category": "exploit_status",
            "details": cvss_note,
            "product_ids": affected,
        }],
    }
    cve = _cve(record, advisory_id)
    if cve:
        vulnerability["cve"] = cve
    references = [f"{base}/advisories/{advisory_id}", *record.get("references", [])]
    document = {
        "category": CATEGORY,
        "csaf_version": CSAF_VERSION,
        "distribution": {"tlp": dict(TLP)},
        "publisher": dict(PUBLISHER),
        "title": record["title"],
        "references": [{"category": "self" if index == 0 else "external",
                        "summary": "Canonical advisory" if index == 0 else "Related reference",
                        "url": url} for index, url in enumerate(dict.fromkeys(references))],
        "tracking": {
            "id": advisory_id,
            "status": "final",
            "version": str(len(revisions)),
            "initial_release_date": published,
            "current_release_date": updated,
            "revision_history": [
                {"date": _timestamp(item["date"], f"{advisory_id}.revision"),
                 "number": str(index), "summary": item["summary"]}
                for index, item in enumerate(revisions, 1)
            ],
        },
    }
    advisory = {"document": document, "product_tree": tree, "vulnerabilities": [vulnerability]}
    row = f"{updated},{advisory_id},{published[:4]}/{advisory_id}.json"
    return advisory, row


def provider_metadata(base: str, last_updated: str) -> dict:
    """Schema-valid CSAF 2.0 provider metadata with a directory pointer."""
    origin = _origin(base)
    return {
        "canonical_url": f"{origin}/.well-known/csaf/provider-metadata.json",
        "last_updated": last_updated,
        "list_on_CSAF_aggregators": True,
        "mirror_on_CSAF_aggregators": True,
        "metadata_version": METADATA_VERSION,
        "publisher": dict(PUBLISHER),
        "role": "csaf_provider",
        "distributions": [{"directory_url": f"{base.rstrip('/')}/csaf/"}],
    }


def scan_secrets(payload: bytes) -> list[str]:
    """Fail closed on credentials, key blocks and private addresses in feed files."""
    return [label for pattern, label in SECRET_PATTERNS if pattern.search(payload)]


def _encode(document: dict) -> bytes:
    return (json.dumps(document, indent=2, sort_keys=True, ensure_ascii=False) + "\n").encode("utf-8")


def feed_files(bundle: dict, base: str) -> dict[str, bytes]:
    """Every CSAF artifact, keyed by its path relative to `security/csaf/`."""
    records = sorted(bundle["advisories"], key=lambda record: record["id"])
    if not records:
        raise CSAFError("canonical bundle has no advisories")
    documents, rows = [], []
    for record in records:
        advisory, row = build_advisory(record, base)
        documents.append((f"{record['published'][:4]}/{record['id']}.json", advisory))
        rows.append(row)
    latest = max(row.split(",", 1)[0] for row in rows)
    files = {PROVIDER_FILE: _encode(provider_metadata(base, latest))}
    for name, document in documents:
        files[name] = _encode(document)
    files[CHANGES_FILE] = ("date,id,path\n" + "\n".join(sorted(rows)) + "\n").encode("utf-8")
    for name, payload in files.items():
        try:
            json.loads(payload) if name.endswith(".json") else None
        except ValueError as error:
            raise CSAFError(f"{name}: generated JSON is invalid: {error}") from error
        secrets = scan_secrets(payload)
        if secrets:
            raise CSAFError(f"{name}: generated file contains {', '.join(secrets)}")
    return files


def generate(feed_dir: Path, bundle: dict, base: str) -> list[str]:
    """Write every CSAF artifact atomically, removing stale generated advisories."""
    files = feed_files(bundle, base)
    expected = set(files)
    feed_dir.mkdir(parents=True, exist_ok=True)
    for directory in sorted(feed_dir.iterdir()):
        if directory.is_dir() and re.fullmatch(r"[0-9]{4}", directory.name):
            for path in directory.glob("*.json"):
                relative = f"{directory.name}/{path.name}"
                if relative not in expected:
                    path.unlink()
    for name, payload in files.items():
        target = feed_dir / name
        target.parent.mkdir(parents=True, exist_ok=True)
        temp = target.with_suffix(target.suffix + ".tmp")
        temp.write_bytes(payload)
        temp.replace(target)
    return sorted(files)
