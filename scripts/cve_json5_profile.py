#!/usr/bin/env python3
"""Dependency-free validation of the CVE JSON 5.1 required profile (#2715).

The full official schema check is optional (`--schema` in cve_json5.py). This
gate always runs and covers the rules that matter for our publication policy:
correct root shape, CNA identity agreement, an English description, affected
products, HTTPS references, a CVSS entry, a CWE and the ban on self-assigned
CVE strings outside `cveMetadata.cveId`.
"""

from __future__ import annotations

from typing import Any

from cve_json5_common import (
    CVE_RE, GBSA_RE, UUID_RE, CWE_RE, valid_timestamp, walk,
)


def validate_profile(record: Any) -> list[str]:
    """Return a list of problems; empty means the required profile is satisfied."""
    errors: list[str] = []
    if not isinstance(record, dict):
        return ["<root>: record must be an object"]
    if set(record) != {"dataType", "dataVersion", "cveMetadata", "containers"}:
        errors.append("<root>: unexpected or missing top-level fields")
    if record.get("dataType") != "CVE_RECORD" or record.get("dataVersion") != "5.1":
        errors.append("dataType/dataVersion: expected CVE_RECORD / 5.1")

    meta = record.get("cveMetadata")
    containers = record.get("containers")
    if not isinstance(meta, dict) or not isinstance(containers, dict):
        return errors + ["cveMetadata/containers: must be objects"]

    for key in ("cveId", "assignerOrgId", "assignerShortName",
                "dateReserved", "datePublished", "dateUpdated", "state"):
        if not meta.get(key):
            errors.append(f"cveMetadata.{key}: mandatory")
    if not CVE_RE.match(str(meta.get("cveId", ""))):
        errors.append("cveMetadata.cveId: invalid CVE ID")
    if not UUID_RE.match(str(meta.get("assignerOrgId", ""))):
        errors.append("cveMetadata.assignerOrgId: must be a CVE UUID v4")
    if meta.get("state") != "PUBLISHED":
        errors.append("cveMetadata.state: must be PUBLISHED")
    for key in ("dateReserved", "datePublished", "dateUpdated"):
        if not valid_timestamp(meta.get(key)):
            errors.append(f"cveMetadata.{key}: invalid timestamp")

    cna = containers.get("cna")
    if not isinstance(cna, dict):
        return errors + ["containers.cna: mandatory"]

    provider = cna.get("providerMetadata")
    if not isinstance(provider, dict):
        errors.append("containers.cna.providerMetadata: mandatory")
    else:
        if provider.get("orgId") != meta.get("assignerOrgId"):
            errors.append("providerMetadata.orgId must match assignerOrgId")
        if provider.get("shortName") != meta.get("assignerShortName"):
            errors.append("providerMetadata.shortName must match assignerShortName")

    descriptions = cna.get("descriptions")
    if not isinstance(descriptions, list) or not any(
        isinstance(item, dict) and str(item.get("lang", "")).startswith("en")
        and str(item.get("value", "")).strip() for item in descriptions
    ):
        errors.append("descriptions: an English description is mandatory")

    affected = cna.get("affected")
    if not isinstance(affected, list) or not affected:
        errors.append("affected: at least one product is mandatory")
    else:
        for item in affected:
            if not isinstance(item, dict) or not item.get("product") or not item.get("versions"):
                errors.append("affected: each product needs product and versions")

    references = cna.get("references")
    if not isinstance(references, list) or not references or any(
        not isinstance(item, dict) or not str(item.get("url", "")).startswith("https://")
        for item in references
    ):
        errors.append("references: at least one HTTPS URL is mandatory")

    metrics = cna.get("metrics")
    if not isinstance(metrics, list) or not metrics or not any(
        isinstance(item, dict) and ("cvssV4_0" in item or "cvssV3_1" in item)
        for item in metrics
    ):
        errors.append("metrics: a CVSS entry is mandatory")

    problems = cna.get("problemTypes")
    if not isinstance(problems, list) or not problems:
        errors.append("problemTypes: at least one CWE is mandatory")
    else:
        for item in problems:
            for desc in item.get("descriptions", []) if isinstance(item, dict) else []:
                if not CWE_RE.match(str(desc.get("cweId", ""))):
                    errors.append("problemTypes: invalid CWE ID")

    source = cna.get("source")
    if not isinstance(source, dict) or not GBSA_RE.match(str(source.get("advisory", ""))):
        errors.append("source.advisory: GBSA-YYYY-NNNN is mandatory")

    allowed = str(meta.get("cveId", ""))
    for path, value in walk(record):
        if isinstance(value, str) and "CVE-" in value and value != allowed:
            errors.append(f"{path}: no self-assigned CVE string outside cveMetadata.cveId")
    return errors
