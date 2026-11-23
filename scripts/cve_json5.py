#!/usr/bin/env python3
"""Generate CVE JSON 5.1 submission records from canonical advisories (#2715).

A CVE ID is never invented here. A record is emitted only when the advisory
front matter declares a ``cve.cve_id`` that is already present in the
evidence-backed CNA assignment registry. IDs are sponsored by an existing CNA
(CERT.br/NIC.br with CTIR Gov coordination, or the GitHub CNA as an interim
route) — see botbook/src/09-security/cve-issuance-strategy.md. When the ID is
sponsored, the generated JSON is the payload submitted to that CNA; the CNA
publishes the record in the CVE List.

Usage:
    python3 scripts/cve_json5.py generate [--advisories DIR] [--registry FILE]
                                          [--out DIR] [--schema FILE]
    python3 scripts/cve_json5.py validate FILE [FILE ...] [--schema FILE]
    python3 scripts/cve_json5.py self-test

Exit codes: 0 clean, 1 validation/generation error, 2 bad input/tooling error.
No network and no third-party dependency at runtime; ``jsonschema`` is optional
and used only for the full official-schema check when a local schema is given.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any

sys.path.insert(0, str(Path(__file__).resolve().parent))

try:
    import yaml
except ImportError:
    print("error: PyYAML is required to read advisory front matter", file=sys.stderr)
    raise SystemExit(2)

try:
    from cvss4 import base_score, severity_for
except ImportError:
    base_score = None  # type: ignore[assignment]
    severity_for = None  # type: ignore[assignment]

from cve_json5_common import (
    CVE_RE, GBSA_RE, UUID_RE, SHORT_NAME_RE, CWE_RE, CREDIT_TYPES,
    DATA_VERSION, GenerationError, valid_timestamp,
)
from cve_json5_profile import validate_profile
from cve_json5_schema import validate_against_schema

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_ADVISORIES = ROOT / "security" / "advisories"
DEFAULT_REGISTRY = DEFAULT_ADVISORIES / "cve-assignments.json"
SKIP_NAMES = {"TEMPLATE.md", "CHECKLIST.md", "README.md"}
RANGE_RE = re.compile(r"^\s*(?:>=|>)?\s*(v?\d[\w.+-]*)\s*,\s*<\s*(v?\d[\w.+-]*)\s*$")
UPPER_RE = re.compile(r"^\s*<\s*(v?\d[\w.+-]*)\s*$")


def read_json(path: Path) -> Any:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise GenerationError(f"{path}: cannot read JSON: {error}") from error


def load_registry(path: Path) -> dict[str, dict[str, Any]]:
    """Return CNA assignments keyed by CVE ID; reject unevidenced entries."""
    data = read_json(path)
    if not isinstance(data, dict) or not isinstance(data.get("assignments"), list):
        raise GenerationError(f"{path}: expected an object with an 'assignments' list")
    registry: dict[str, dict[str, Any]] = {}
    required = ("cve_id", "advisory", "short_name", "org_id",
                "date_reserved", "date_published", "evidence")
    for index, item in enumerate(data["assignments"], start=1):
        label = f"{path}: assignments[{index}]"
        if not isinstance(item, dict):
            raise GenerationError(f"{label}: must be an object")
        for key in required:
            if not isinstance(item.get(key), str) or not item[key].strip():
                raise GenerationError(f"{label}: '{key}' is mandatory")
        if not CVE_RE.match(item["cve_id"]):
            raise GenerationError(f"{label}: invalid CVE ID {item['cve_id']!r}")
        if item["cve_id"] in registry:
            raise GenerationError(f"{label}: duplicate CVE ID {item['cve_id']}")
        if not GBSA_RE.match(item["advisory"]):
            raise GenerationError(f"{label}: 'advisory' must be a GBSA-YYYY-NNNN ID")
        if not SHORT_NAME_RE.match(item["short_name"]):
            raise GenerationError(f"{label}: invalid CNA short name")
        if not UUID_RE.match(item["org_id"]):
            raise GenerationError(f"{label}: 'org_id' must be a CVE programme UUID v4")
        for key in ("date_reserved", "date_published"):
            if not valid_timestamp(item[key]):
                raise GenerationError(f"{label}: '{key}' must be an RFC 3339 timestamp")
        registry[item["cve_id"]] = item
    return registry


def load_advisory(path: Path) -> dict[str, Any]:
    """Parse the YAML front matter of one canonical advisory."""
    lines = path.read_text(encoding="utf-8").splitlines()
    if not lines or lines[0].strip() != "---":
        raise GenerationError(f"{path}: missing YAML front matter")
    end = next((i for i in range(1, len(lines)) if lines[i].strip() == "---"), None)
    if end is None:
        raise GenerationError(f"{path}: unterminated YAML front matter")
    data = yaml.safe_load("\n".join(lines[1:end]))
    if not isinstance(data, dict):
        raise GenerationError(f"{path}: front matter must be a mapping")
    if not GBSA_RE.match(str(data.get("id", ""))):
        raise GenerationError(f"{path}: 'id' must be GBSA-YYYY-NNNN (never a CVE ID)")
    if "CVE-" in str(data.get("title", "")):
        raise GenerationError(f"{path}: a title must not self-assign a CVE ID")
    return data


def affected_versions(values: Any, name: str) -> list[dict[str, str]]:
    """Translate template range strings into CVE affected-version entries."""
    if not isinstance(values, list) or not values:
        raise GenerationError(f"{name}: 'affected_versions' must be a non-empty list")
    entries: list[dict[str, str]] = []
    for raw in values:
        text = str(raw).strip()
        match = RANGE_RE.match(text)
        if match:
            entry = {"version": match.group(1), "lessThan": match.group(2)}
        else:
            match = UPPER_RE.match(text)
            if match:
                entry = {"version": "0", "lessThan": match.group(1)}
            elif re.match(r"^v?\d[\w.+-]*$", text):
                entry = {"version": text}
            else:
                raise GenerationError(f"{name}: unparseable affected version {text!r}")
        entry["status"] = "affected"
        entry["versionType"] = "semver"
        entries.append(entry)
    return entries


def metric(cvss: Any, name: str) -> dict[str, Any]:
    """Build a CVE metrics entry and recompute the CVSS v4.0 base score."""
    if not isinstance(cvss, dict):
        raise GenerationError(f"{name}: 'cvss' must be a mapping")
    version = str(cvss.get("version", ""))
    if version not in {"4.0", "3.1"}:
        raise GenerationError(f"{name}: cvss.version must be 4.0 or 3.1")
    vector = str(cvss.get("vector", ""))
    score = cvss.get("base_score")
    severity = str(cvss.get("base_severity", "")).upper()
    if not vector.startswith(f"CVSS:{version}/"):
        raise GenerationError(f"{name}: CVSS vector does not match version {version}")
    if not isinstance(score, (int, float)) or isinstance(score, bool):
        raise GenerationError(f"{name}: cvss.base_score must be a number")
    if version == "4.0":
        if base_score is None or severity_for is None:
            raise GenerationError(f"{name}: scripts/cvss4.py is required for CVSS 4.0")
        try:
            computed = base_score(vector)
        except ValueError as error:
            raise GenerationError(f"{name}: invalid CVSS vector: {error}") from error
        if abs(float(score) - computed) > 1e-9:
            raise GenerationError(f"{name}: CVSS vector/score disagree ({computed:.1f} vs {score})")
        if severity != severity_for(computed).upper():
            raise GenerationError(f"{name}: CVSS severity disagrees with the vector")
        return {"cvssV4_0": {
            "version": version, "vectorString": vector,
            "baseScore": float(score), "baseSeverity": severity,
        }}
    if severity not in {"NONE", "LOW", "MEDIUM", "HIGH", "CRITICAL"}:
        raise GenerationError(f"{name}: invalid CVSS 3.1 base severity")
    return {"cvssV3_1": {
        "version": version, "vectorString": vector,
        "baseScore": float(score), "baseSeverity": severity,
    }}


def build_record(data: dict[str, Any], assignment: dict[str, Any],
                 name: str) -> dict[str, Any]:
    """Join an approved advisory with its CNA assignment into a CVE 5.1 record."""
    identifier = str(data.get("id", ""))
    declared = data.get("cve")
    if not isinstance(declared, dict) or not CVE_RE.match(str(declared.get("cve_id", ""))):
        raise GenerationError(f"{name}: no CNA-assigned cve.cve_id in front matter")
    if declared["cve_id"] != assignment["cve_id"]:
        raise GenerationError(f"{name}: CVE ID is not in the assignment registry")
    if assignment["advisory"] != identifier:
        raise GenerationError(f"{name}: registry entry belongs to {assignment['advisory']}")

    summary = str(data.get("summary", "")).strip()
    if not summary:
        raise GenerationError(f"{name}: 'summary' is mandatory")
    fixed = data.get("fixed_versions")
    if not isinstance(fixed, list) or not fixed:
        raise GenerationError(f"{name}: 'fixed_versions' is mandatory")
    description = f"{summary} Fixed in {', '.join(str(v) for v in fixed)}."

    urls = [f"https://generalbots.org/security/advisories/{identifier}/"]
    for raw in data.get("references", []) or []:
        url = str(raw)
        if url.startswith("https://"):
            urls.append(url)
    references = [{"url": url} for url in dict.fromkeys(urls)]

    cwes = data.get("cwe", [])
    if not isinstance(cwes, list) or not cwes:
        raise GenerationError(f"{name}: at least one CWE ID is mandatory")
    problem_types = [{"descriptions": [{
        "lang": "en", "type": "CWE", "description": str(cwe), "cweId": str(cwe),
    }]} for cwe in cwes]
    for entry in problem_types:
        if not CWE_RE.match(entry["descriptions"][0]["cweId"]):
            raise GenerationError(f"{name}: invalid CWE ID")

    credits = []
    for item in data.get("credits", []) or []:
        if isinstance(item, dict) and item.get("name"):
            credits.append({
                "lang": "en",
                "value": f"{item['name']} ({item.get('role', 'reporter')})",
                "type": CREDIT_TYPES.get(str(item.get("role", "")), "finder"),
            })

    product = str(data.get("product", "")).strip()
    if not product:
        raise GenerationError(f"{name}: 'product' is mandatory")
    published = assignment["date_published"]
    short_name = assignment["short_name"]
    org_id = assignment["org_id"]
    return {
        "dataType": "CVE_RECORD",
        "dataVersion": DATA_VERSION,
        "cveMetadata": {
            "cveId": assignment["cve_id"],
            "assignerOrgId": org_id,
            "assignerShortName": short_name,
            "dateReserved": assignment["date_reserved"],
            "datePublished": published,
            "dateUpdated": published,
            "state": "PUBLISHED",
        },
        "containers": {"cna": {
            "providerMetadata": {
                "orgId": org_id, "shortName": short_name, "dateUpdated": published,
            },
            "title": str(data.get("title", ""))[:256],
            "descriptions": [{"lang": "en", "value": description}],
            "affected": [{
                "vendor": "General Bots",
                "product": product,
                "defaultStatus": "unaffected",
                "versions": affected_versions(data.get("affected_versions"), name),
            }],
            "references": references,
            "metrics": [metric(data.get("cvss"), name)],
            "problemTypes": problem_types,
            "credits": credits,
            "source": {"advisory": identifier},
        }},
    }



def check_schema(record: dict[str, Any], schema: Path | None,
                 errors: list[str]) -> None:
    if schema is None:
        return
    passed, full_errors, skipped = validate_against_schema(record, schema)
    if skipped:
        errors.append("official schema check skipped: install 'jsonschema' for --schema")
    elif not passed:
        errors.extend(f"official schema: {error}" for error in full_errors)


def generate(args: argparse.Namespace) -> int:
    registry = load_registry(args.registry)
    files = sorted(p for p in args.advisories.glob("*.md") if p.name not in SKIP_NAMES)
    args.out.mkdir(parents=True, exist_ok=True)
    emitted = 0
    failures = 0
    for path in files:
        try:
            data = load_advisory(path)
        except (GenerationError, OSError) as error:
            print(f"error: {error}", file=sys.stderr)
            failures += 1
            continue
        declared = data.get("cve")
        if not isinstance(declared, dict) or not declared.get("cve_id"):
            continue  # an advisory without a CNA assignment is not a CVE record
        try:
            record = build_record(data, registry[declared["cve_id"]], str(path))
        except KeyError:
            errors = [f"{path}: CVE ID is not in the CNA assignment registry"]
            record = None
        except GenerationError as error:
            errors = [str(error)]
            record = None
        else:
            errors = validate_profile(record)
            check_schema(record, args.schema, errors)
        if record is not None and not errors:
            target = args.out / f"{declared['cve_id']}.json"
            target.write_text(json.dumps(record, indent=2, ensure_ascii=False) + "\n",
                              encoding="utf-8")
            print(f"PASS {path} -> {target}")
            emitted += 1
        else:
            failures += 1
            for error in errors:
                print(f"error: {error}", file=sys.stderr)
            print(f"REJECT {path}", file=sys.stderr)
    print(f"\n{len(files)} advisory file(s), {emitted} CVE record(s), {failures} failure(s)")
    return 1 if failures else 0


def validate(args: argparse.Namespace) -> int:
    failures = 0
    for path in args.files:
        try:
            record = read_json(path)
        except GenerationError as error:
            print(f"error: {error}", file=sys.stderr)
            failures += 1
            continue
        errors = validate_profile(record)
        if isinstance(record, dict):
            check_schema(record, args.schema, errors)
        if errors:
            failures += 1
            for error in errors:
                print(f"error: {path}: {error}", file=sys.stderr)
            print(f"REJECT {path}", file=sys.stderr)
        else:
            print(f"PASS {path}")
    return 1 if failures else 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    subs = parser.add_subparsers(dest="command", required=True)
    gen = subs.add_parser("generate", help="generate CVE JSON 5 records")
    gen.add_argument("--advisories", type=Path, default=DEFAULT_ADVISORIES)
    gen.add_argument("--registry", type=Path, default=DEFAULT_REGISTRY)
    gen.add_argument("--out", type=Path, required=True)
    gen.add_argument("--schema", type=Path)
    gen.set_defaults(func=generate)
    val = subs.add_parser("validate", help="validate CVE JSON 5 records")
    val.add_argument("files", nargs="+", type=Path)
    val.add_argument("--schema", type=Path)
    val.set_defaults(func=validate)
    test = subs.add_parser("self-test", help="run offline generator self-tests")
    test.set_defaults(func=lambda _args: self_test())
    args = parser.parse_args(argv)
    try:
        return args.func(args)
    except (GenerationError, OSError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 2


def self_test() -> int:
    from cve_json5_selftest import run
    return run()


if __name__ == "__main__":
    raise SystemExit(main())
