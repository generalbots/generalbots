#!/usr/bin/env python3
"""Validate security advisory source files against the single template (#2724).

An advisory is a Markdown file with YAML front matter. This gate is the
publication guard: it rejects free-form bulletins, checks every mandatory
field (including two-person review and revision history), recomputes the CVSS
v4.0 base score from the vector and refuses to publish when vector and score
(or score and severity rating) disagree.

Usage:
    python3 scripts/validate_advisory.py [PATH ...] [--json]
    python3 scripts/validate_advisory.py --self-test

PATH may be a file or a directory (default: security/advisories). Files named
TEMPLATE.md, README.md or CHECKLIST.md are documentation, not advisories, and
are skipped during directory discovery.

Exit codes: 0 all advisories valid, 1 one or more invalid, 2 tooling error.
Requires PyYAML (already used by scripts/slo_common.py and others).
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any

from cvss4 import base_score, severity_for

try:
    import yaml
except ImportError:
    print("error: PyYAML is required to validate advisories", file=sys.stderr)
    raise SystemExit(2)

DEFAULT_DIR = Path("security/advisories")
SKIP_NAMES = {"TEMPLATE.md", "README.md", "CHECKLIST.md"}
PUBLISHED = "published"
STATUSES = {"draft", "review", PUBLISHED, "withdrawn"}
SEVERITIES = {"none", "low", "medium", "high", "critical"}
EXPLOITABILITY = {"none", "poc", "active"}
REQUIRED_SCALARS = (
    "id", "title", "summary", "status", "severity", "product",
    "attack_prerequisites", "impact", "remediation", "workarounds",
    "exploitability_status",
)
REQUIRED_LISTS = (
    "affected_versions", "fixed_versions", "cwe", "timeline",
    "credits", "references", "revision_history",
)
REQUIRED_SECTIONS = (
    "summary", "affected products", "impact", "remediation",
    "workarounds", "exploitability", "timeline", "credits",
    "references", "revision history",
)
ID_RE = re.compile(r"^GBSA-\d{4}-\d{4}$")
CWE_RE = re.compile(r"^CWE-\d+$")
CAPEC_RE = re.compile(r"^CAPEC-\d+$")


def discover(paths: list[Path]) -> tuple[list[Path], list[str]]:
    """Resolve CLI paths into advisory files."""
    files: list[Path] = []
    problems: list[str] = []
    for path in paths or [DEFAULT_DIR]:
        if path.is_dir():
            files.extend(sorted(p for p in path.glob("*.md") if p.name not in SKIP_NAMES))
        elif path.is_file():
            files.append(path)
        else:
            problems.append(f"{path}: path does not exist")
    return files, problems


def split_front_matter(text: str, name: str) -> tuple[dict[str, Any], str, list[str]]:
    """Return (front matter, body, errors). Missing/invalid YAML is an error."""
    lines = text.splitlines()
    if not lines or lines[0].strip() != "---":
        return {}, text, [f"{name}:1: missing YAML front matter (free-form bulletin)"]
    end = next((i for i in range(1, len(lines)) if lines[i].strip() == "---"), None)
    if end is None:
        return {}, text, [f"{name}:1: unterminated YAML front matter"]
    try:
        data = yaml.safe_load("\n".join(lines[1:end]))
    except yaml.YAMLError as error:
        return {}, text, [f"{name}:2: invalid YAML front matter: {error}"]
    if not isinstance(data, dict):
        return {}, text, [f"{name}:2: front matter must be a YAML mapping"]
    return data, "\n".join(lines[end + 1:]), []


def _nonempty_list(data: dict[str, Any], key: str, path: str, errors: list[str]) -> list[Any]:
    value = data.get(key)
    if not isinstance(value, list) or not value:
        errors.append(f"{path}: '{key}' must be a non-empty list")
        return []
    if any(isinstance(item, str) and not item.strip() for item in value):
        errors.append(f"{path}: '{key}' contains an empty entry")
    return value


def _entries(data: dict[str, Any], key: str, fields: tuple[str, ...],
             path: str, errors: list[str]) -> None:
    for index, item in enumerate(_nonempty_list(data, key, path, errors), start=1):
        if not isinstance(item, dict):
            errors.append(f"{path}: '{key}[{index}]' must be a mapping")
            continue
        for field in fields:
            if not str(item.get(field, "")).strip():
                errors.append(f"{path}: '{key}[{index}].{field}' is mandatory")


def validate(data: dict[str, Any], body: str, name: str) -> list[str]:
    errors: list[str] = []
    path = name

    for key in REQUIRED_SCALARS:
        value = data.get(key)
        if not isinstance(value, str) or not value.strip():
            errors.append(f"{path}: '{key}' is mandatory")

    for key in REQUIRED_LISTS:
        _nonempty_list(data, key, path, errors)

    identifier = str(data.get("id", ""))
    if identifier and not ID_RE.match(identifier):
        errors.append(f"{path}: 'id' must be GBSA-YYYY-NNNN (no self-assigned CVE IDs)")
    for text in (identifier, str(data.get("title", ""))):
        if "CVE-" in text:
            errors.append(f"{path}: do not self-assign CVE IDs before #2715 chooses a CNA route")

    status = data.get("status")
    if status not in STATUSES:
        errors.append(f"{path}: 'status' must be one of {sorted(STATUSES)}")
    severity = data.get("severity")
    if severity not in SEVERITIES:
        errors.append(f"{path}: 'severity' must be one of {sorted(SEVERITIES)}")
    exploitability = data.get("exploitability_status")
    if exploitability not in EXPLOITABILITY:
        errors.append(f"{path}: 'exploitability_status' must be one of {sorted(EXPLOITABILITY)}")

    for item in _nonempty_list(data, "cwe", path, errors):
        if not isinstance(item, str) or not CWE_RE.match(item):
            errors.append(f"{path}: CWE entry {item!r} must look like 'CWE-79'")
    if "capec" in data:
        capec = data["capec"]
        if capec is not None and (not isinstance(capec, list) or
                                  any(not isinstance(i, str) or not CAPEC_RE.match(i) for i in capec)):
            errors.append(f"{path}: CAPEC entries must look like 'CAPEC-63'")

    cvss = data.get("cvss")
    computed: float | None = None
    if not isinstance(cvss, dict):
        errors.append(f"{path}: 'cvss' mapping is mandatory")
    else:
        if cvss.get("version") != "4.0":
            errors.append(f"{path}: cvss.version must be '4.0'")
        vector = cvss.get("vector")
        try:
            computed = base_score(vector)
        except ValueError as error:
            errors.append(f"{path}: invalid cvss.vector: {error}")
        declared = cvss.get("base_score")
        if not isinstance(declared, (int, float)) or isinstance(declared, bool):
            errors.append(f"{path}: cvss.base_score must be a number")
        elif computed is not None and abs(float(declared) - computed) > 1e-9:
            errors.append(
                f"{path}: CVSS vector and score disagree — vector scores {computed:.1f}, "
                f"front matter declares {declared}"
            )
        rating = severity_for(computed) if computed is not None else None
        if rating is not None and cvss.get("base_severity") != rating:
            errors.append(f"{path}: cvss.base_severity must be '{rating}' for score {computed:.1f}")
        if rating is not None and severity is not None and severity != rating:
            errors.append(f"{path}: 'severity' must match the CVSS rating '{rating}'")

    _entries(data, "timeline", ("date", "event"), path, errors)
    _entries(data, "credits", ("name", "role"), path, errors)
    _entries(data, "revision_history", ("version", "date", "change"), path, errors)

    review = data.get("review")
    if not isinstance(review, dict):
        errors.append(f"{path}: 'review' mapping is mandatory (two-person review)")
    else:
        author = str(review.get("author", "")).strip()
        reviewer = str(review.get("security_reviewer", "")).strip()
        if not author:
            errors.append(f"{path}: review.author is mandatory")
        if not reviewer:
            errors.append(f"{path}: review.security_reviewer is mandatory")
        if author and reviewer and author.casefold() == reviewer.casefold():
            errors.append(f"{path}: author and security reviewer must be different people")
        if status == PUBLISHED:
            if review.get("approved") is not True:
                errors.append(f"{path}: publication requires review.approved: true")
            if not str(review.get("reviewed_at", "")).strip():
                errors.append(f"{path}: publication requires review.reviewed_at")
            if not str(data.get("published", "")).strip():
                errors.append(f"{path}: publication requires a 'published' date")

    embargo = data.get("embargo", {})
    if embargo is not None:
        if not isinstance(embargo, dict):
            errors.append(f"{path}: 'embargo' must be a mapping")
        elif embargo.get("early_notice") is True:
            recipients = embargo.get("recipients")
            if not isinstance(recipients, list) or not recipients:
                errors.append(f"{path}: early notice requires non-empty embargo.recipients")
            if not str(embargo.get("until", "")).strip():
                errors.append(f"{path}: early notice requires an embargo.until date")

    headings = {
        line[2:].strip().casefold()
        for line in body.splitlines() if line.startswith("## ")
    }
    for section in REQUIRED_SECTIONS:
        if section not in headings:
            errors.append(f"{path}: body is missing '## {section.title()}' (use TEMPLATE.md)")

    return errors


def check_files(files: list[Path]) -> dict[str, list[str]]:
    results: dict[str, list[str]] = {}
    for file in files:
        try:
            text = file.read_text(encoding="utf-8")
        except OSError as error:
            results[str(file)] = [f"{file}: cannot read file: {error}"]
            continue
        data, body, errors = split_front_matter(text, str(file))
        if not errors:
            errors = validate(data, body, str(file))
        results[str(file)] = errors
    return results


def _fixture(status: str = PUBLISHED, **overrides: Any) -> str:
    data: dict[str, Any] = {
        "id": "GBSA-2026-0001", "title": "Example advisory", "summary": "Summary.",
        "status": status, "severity": "critical", "published": "2026-11-23",
        "updated": "2026-11-23", "product": "General Bots Server",
        "affected_versions": ["< 6.8.12"], "fixed_versions": ["6.8.12"],
        "cwe": ["CWE-79"], "capec": ["CAPEC-63"],
        "cvss": {
            "version": "4.0", "base_score": 9.3, "base_severity": "critical",
            "vector": "CVSS:4.0/AV:N/AC:L/AT:N/PR:N/UI:N/VC:H/VI:H/VA:H/SC:N/SI:N/SA:N",
        },
        "attack_prerequisites": "None.", "impact": "Full compromise.",
        "remediation": "Upgrade.", "workarounds": "None.",
        "exploitability_status": "none",
        "timeline": [{"date": "2026-11-01", "event": "Reported"}],
        "credits": [{"name": "Reporter", "role": "reporter"}],
        "references": ["https://www.first.org/cvss/v4.0/"],
        "embargo": {"early_notice": False, "recipients": [], "until": None},
        "review": {
            "author": "Alice", "security_reviewer": "Bob", "approved": True,
            "reviewed_at": "2026-11-22",
        },
        "revision_history": [{"version": 1, "date": "2026-11-23", "change": "Initial"}],
    }
    data.update(overrides)
    body = "\n".join(f"## {s.title()}\n" for s in REQUIRED_SECTIONS)
    return "---\n" + yaml.safe_dump(data, sort_keys=False) + "---\n" + body


def self_test() -> int:
    import copy
    import tempfile

    body = "\n".join(f"## {s.title()}" for s in REQUIRED_SECTIONS)
    base = yaml.safe_load(_fixture().split("---")[1])

    def with_change(label: str, mutate: Any, reason: str,
                    text: str | None = None) -> tuple[str, str, str | None]:
        doc = copy.deepcopy(base)
        mutate(doc)
        return (label, text or "---\n" + yaml.safe_dump(doc) + "---\n" + body, reason)

    cases: list[tuple[str, str, str | None]] = [("valid", _fixture(), None)]
    cases.append(with_change("score_mismatch", lambda d: d["cvss"].update(base_score=9.2), "disagree"))
    cases.append(with_change("bad_rating", lambda d: d["cvss"].update(base_severity="high"), "base_severity"))
    cases.append(with_change("bad_top_severity", lambda d: d.update(severity="low"), "'severity'"))
    cases.append(with_change("missing_reviewer", lambda d: d["review"].pop("security_reviewer"),
                             "security_reviewer"))
    cases.append(with_change("same_person", lambda d: d["review"].update(security_reviewer="ALICE"),
                             "different people"))
    cases.append(with_change("unapproved", lambda d: d["review"].update(approved=False), "approved"))
    cases.append(with_change("broken_vector", lambda d: d["cvss"].update(
        vector=d["cvss"]["vector"].replace("/AT:N", "")), "mandatory base metric"))
    cases.append(with_change("missing_cwe", lambda d: d.pop("cwe"), "'cwe'"))
    cases.append(with_change("no_history", lambda d: d.update(revision_history=[]), "revision_history"))
    cases.append(with_change("self_assigned_cve", lambda d: d.update(id="CVE-2026-0001"), "CVE"))
    cases.append(with_change("embargo_no_recipients", lambda d: d["embargo"].update(
        early_notice=True, recipients=[], until="2026-12-23"), "recipients"))
    cases.append(with_change("no_front_matter", lambda d: None, "front matter",
                             "# Bulletin\n\nNo front matter here."))

    with tempfile.TemporaryDirectory() as tmp:
        failures = 0
        for label, text, reason in cases:
            file = Path(tmp) / f"{label}.md"
            file.write_text(text, encoding="utf-8")
            data, draft, errors = split_front_matter(text, str(file))
            if not errors:
                errors = validate(data, draft, str(file))
            passed = not errors
            expected_pass = reason is None
            matched = passed == expected_pass and (reason is None or any(reason in e for e in errors))
            if not matched:
                failures += 1
                print(f"FAIL {label}: expected {reason or 'accept'}, got {errors}")
            else:
                print(f"ok {label}: {'accepted' if passed else 'rejected'}")
        if failures:
            print(f"self-test FAILED: {failures} case(s)")
            return 1
        print(f"self-test ok: {len(cases)} advisory validation cases")
        return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("paths", nargs="*", type=Path, help="advisory file(s) or directory")
    parser.add_argument("--json", action="store_true", help="emit JSON results")
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()

    if args.self_test:
        return self_test()

    files, problems = discover(args.paths)
    results = check_files(files)
    errors = [message for messages in results.values() for message in messages] + problems
    if args.json:
        print(json.dumps({"advisories": {k: v for k, v in results.items()}, "errors": problems},
                         indent=2))
    else:
        for name, messages in results.items():
            if messages:
                for message in messages:
                    print(f"error: {message}")
                print(f"REJECT {name}")
            else:
                print(f"PASS {name}")
        for problem in problems:
            print(f"error: {problem}")
        print(f"\n{len(results)} advisory file(s), {len(errors)} problem(s)")
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
