#!/usr/bin/env python3
"""Offline gate for the public security status page, incident history and hall
of fame (#2721). No network, no dependencies.

Failures (all fatal):
  * an incident row that is not published, not past embargo or has no fix;
  * an incident row that disagrees with the canonical advisory source;
  * a published advisory missing from the incident history;
  * a hall-of-fame entry without recorded opt-in, or carrying private data;
  * a status/hall-of-fame page with broken i18n keys, a missing nav link or a
    table that disagrees with the incident index;
  * an email (other than the public security mailbox), internal IP or token
    shaped string in any published file.

Usage: python3 scripts/check_security_status_pages.py
Exit codes: 0 clean, 1 policy failure, 2 usage/IO error.
"""

from __future__ import annotations

import html
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
INCIDENTS = ROOT / "security/incidents/index.json"
ADVISORIES = ROOT / "botlib/src/security/advisory/data/advisories.json"
HALL = ROOT / "security/pages/hall-of-fame.json"
STATUS_PAGE = ROOT / "security/pages/status.html"
HALL_PAGE = ROOT / "security/pages/hall-of-fame.html"

PUBLIC_MAILBOX = "security@generalbots.org"
RATING = re.compile(r"^(critical|high|medium|low|info)$")
DATE = re.compile(r"^\d{4}-\d{2}-\d{2}$")
ADVISORY_ID = re.compile(r"^GBSA-\d{4}-\d{4}$")
IPV4 = re.compile(r"\b(?:\d{1,3}\.){3}\d{1,3}\b")
EMAIL = re.compile(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b")
TOKEN = re.compile(
    r"(?:VAULT_TOKEN\s*=|ghp_[A-Za-z0-9]|github_pat_|AKIA[0-9A-Z]{16}|"
    r"lv-[A-Za-z0-9_-]{10,}|-----BEGIN [A-Z ]*PRIVATE KEY-----|Bearer\s+[A-Za-z0-9._-]{12,})"
)
CATALOG = re.compile(
    r'<script type="application/json" id="gb-i18n">(.*?)</script>', re.DOTALL
)
I18N_KEY = re.compile(r'data-i18n="([^"]+)"')
REQUIRED_LINKS = ("/security/", "status.html", "hall-of-fame.html")


class Gate:
    def __init__(self) -> None:
        self.errors: list[str] = []
        self.checks = 0

    def check(self, condition: bool, message: str) -> None:
        self.checks += 1
        if not condition:
            self.errors.append(message)

    def load_json(self, path: Path) -> object:
        try:
            return json.loads(path.read_text(encoding="utf-8"))
        except (OSError, ValueError) as exc:
            self.check(False, f"{path.name}: cannot parse JSON ({exc})")
            return {}


def text_of(fragment: str) -> str:
    """Strip tags and unescape entities from an HTML fragment."""
    return html.unescape(re.sub(r"<[^>]+>", "", fragment)).strip()


def page_rows(page: str) -> list[list[str]]:
    body = re.search(r"<tbody>(.*?)</tbody>", page, re.DOTALL)
    if body is None:
        return []
    rows: list[list[str]] = []
    for row in re.findall(r"<tr>(.*?)</tr>", body.group(1), re.DOTALL):
        cells = re.findall(r"<td[^>]*>(.*?)</td>", row, re.DOTALL)
        rows.append([text_of(cell) for cell in cells])
    return rows


def page_catalogs(page: str) -> dict[str, dict[str, str]]:
    match = CATALOG.search(page)
    if match is None:
        return {}
    try:
        parsed = json.loads(match.group(1))
    except ValueError:
        return {}
    if not isinstance(parsed, dict):
        return {}
    return parsed


def privacy_scan(gate: Gate, label: str, value: str) -> None:
    gate.check(IPV4.search(value) is None, f"{label}: internal IP shaped string")
    gate.check(TOKEN.search(value) is None, f"{label}: token shaped string")
    for found in EMAIL.findall(value):
        gate.check(
            found.lower() == PUBLIC_MAILBOX,
            f"{label}: private email address published ({found})",
        )


def check_incidents(gate: Gate, incidents: dict, advisories: dict) -> None:
    gate.check(incidents.get("schema") == "generalbots.security.incident-index/1.0",
               "incident index: unexpected schema version")
    rows = incidents.get("incidents")
    gate.check(isinstance(rows, list), "incident index: 'incidents' must be a list")
    if not isinstance(rows, list):
        return

    source = advisories.get("advisories")
    gate.check(isinstance(source, list), "advisory source: 'advisories' must be a list")
    if not isinstance(source, list):
        return
    by_id = {item.get("id"): item for item in source if isinstance(item, dict)}
    seen: set[str] = set()

    for row in rows:
        if not isinstance(row, dict):
            gate.check(False, "incident index: row is not an object")
            continue
        rid = str(row.get("advisory", ""))
        label = f"incident {rid or '<missing id>'}"
        gate.check(bool(ADVISORY_ID.fullmatch(rid)), f"{label}: invalid advisory id")
        gate.check(rid not in seen, f"{label}: duplicate row")
        seen.add(rid)
        gate.check(row.get("status") == "resolved", f"{label}: status must be resolved")
        gate.check(row.get("embargo") == "ended", f"{label}: embargo has not ended")
        gate.check(bool(row.get("fixed_version")), f"{label}: no fixed version")
        gate.check(bool(DATE.fullmatch(str(row.get("date", "")))), f"{label}: invalid date")
        gate.check(bool(RATING.fullmatch(str(row.get("severity", "")))), f"{label}: invalid severity")
        gate.check(isinstance(row.get("affected_versions"), list) and bool(row["affected_versions"]),
                   f"{label}: affected versions missing")
        gate.check(str(row.get("advisory_url", "")).startswith("https://generalbots.org/security/advisories/"),
                   f"{label}: advisory URL must be the canonical HTTPS page")
        for key in ("advisory", "date", "severity", "product", "component",
                    "affected_versions", "fixed_version", "status", "embargo"):
            gate.check(key in row, f"{label}: missing field {key}")

        original = by_id.get(rid)
        gate.check(original is not None, f"{label}: no canonical advisory with this id")
        if original is None:
            continue
        gate.check(original.get("published") == row.get("date"),
                   f"{label}: date differs from canonical advisory")
        gate.check(original.get("severity") == row.get("severity"),
                   f"{label}: severity differs from canonical advisory")
        gate.check(original.get("product") == row.get("product"),
                   f"{label}: product differs from canonical advisory")
        gate.check(original.get("component") == row.get("component"),
                   f"{label}: component differs from canonical advisory")
        ranges = [item.get("range") for item in original.get("affected_versions", [])
                  if isinstance(item, dict)]
        gate.check(ranges == list(row.get("affected_versions", [])),
                   f"{label}: affected ranges differ from canonical advisory")
        remediation = original.get("remediation")
        if isinstance(remediation, dict):
            gate.check(remediation.get("recommended_version") == row.get("fixed_version"),
                       f"{label}: fixed version differs from canonical advisory")
        gate.check("exploit" not in str(row).lower() and "poc" not in str(row).lower(),
                   f"{label}: exploit detail must not be in the public index")

    for aid, original in by_id.items():
        if original.get("severity") in ("critical", "high", "medium", "low", "info"):
            gate.check(aid in seen, f"advisory {aid}: published advisory missing from incident history")


def check_hall(gate: Gate, hall: dict) -> None:
    entries = hall.get("entries")
    gate.check(isinstance(entries, list), "hall of fame: 'entries' must be a list")
    gate.check("0" in hall.get("incentive", "") and "bounty" in hall.get("incentive", "").lower(),
               "hall of fame: must state no bounty and reward budget 0")
    if not isinstance(entries, list):
        return
    for entry in entries:
        if not isinstance(entry, dict):
            gate.check(False, "hall of fame: entry is not an object")
            continue
        label = f"hall entry {entry.get('handle', '<anonymous>')}"
        gate.check(entry.get("opt_in") is True, f"{label}: missing recorded opt-in")
        gate.check(bool(entry.get("handle")) and bool(ADVISORY_ID.fullmatch(str(entry.get("advisory", "")))),
                   f"{label}: handle or advisory id invalid")
        gate.check(bool(DATE.fullmatch(str(entry.get("date", "")))), f"{label}: invalid date")
        gate.check(entry.get("role") in ("reporter", "researcher", "internal"),
                   f"{label}: invalid role")
        for private in ("email", "mail", "ip", "phone", "token", "secret", "address"):
            gate.check(private not in entry, f"{label}: private field '{private}' must not be published")


def check_page(gate: Gate, path: Path, incidents: dict, is_status: bool) -> None:
    try:
        page = path.read_text(encoding="utf-8")
    except OSError as exc:
        gate.check(False, f"{path.name}: cannot read ({exc})")
        return
    label = path.name
    gate.check(page.count("<h1") == 1, f"{label}: expected exactly one h1")
    gate.check('lang="pt-BR"' in page, f"{label}: lang must default to pt-BR")
    gate.check('class="skip"' in page, f"{label}: skip link missing")
    for href in REQUIRED_LINKS:
        gate.check(f'href="{href}"' in page, f"{label}: missing link {href}")
    privacy_scan(gate, label, page)

    catalogs = page_catalogs(page)
    gate.check(set(catalogs) == {"pt-BR", "en"}, f"{label}: expected pt-BR and en catalogs")
    keys = set(I18N_KEY.findall(page))
    for lang, catalog in catalogs.items():
        gate.check(keys.issubset(set(catalog)),
                   f"{label}: keys missing from {lang}: {sorted(keys - set(catalog))}")
    if len(catalogs) == 2:
        gate.check(set(catalogs["pt-BR"]) == set(catalogs["en"]),
                   f"{label}: pt-BR and en key sets differ")
    for catalog in catalogs.values():
        for value in catalog.values():
            privacy_scan(gate, f"{label} catalog", value)

    if is_status:
        gate.check("CTIR Gov" in page and "CERT.br" in page,
                   f"{label}: escalation contacts missing")
        gate.check("embargo" in page.lower(), f"{label}: embargo note missing")
        gate.check("EXEMPLO" in page and "EXAMPLE" in page,
                   f"{label}: example badges missing")
        rows = page_rows(page)
        expected = incidents.get("incidents", [])
        gate.check(len(rows) == len(expected),
                   f"{label}: {len(rows)} rows but {len(expected)} incident records")
        for row, record in zip(rows, expected):
            rid = str(record.get("advisory", ""))
            gate.check(len(row) == 6, f"{label}: row for {rid} has {len(row)} cells, expected 6")
            if len(row) != 6:
                continue
            gate.check(rid in row[0], f"{label}: row advisory id mismatch ({rid})")
            gate.check(str(record.get("date")) in row[1], f"{label}: row date mismatch ({rid})")
            gate.check(str(record.get("severity")) in row[2], f"{label}: row severity mismatch ({rid})")
            gate.check(str(record.get("product")) in row[3] and str(record.get("component")) in row[3],
                       f"{label}: row product mismatch ({rid})")
            for version in record.get("affected_versions", []):
                gate.check(str(version) in row[4], f"{label}: affected range mismatch ({rid})")
            gate.check(str(record.get("fixed_version")) in row[5],
                       f"{label}: fixed version mismatch ({rid})")
    else:
        lower = page.lower()
        gate.check("budget" in lower and "bounty" in lower, f"{label}: no-bounty notice missing")
        gate.check("opt-in" in lower or "opt_in" in lower, f"{label}: opt-in notice missing")
        gate.check("remo" in lower, f"{label}: removal process missing")
        gate.check("hall-of-fame.json" in page, f"{label}: registry link missing")
        gate.check("mailto:" + PUBLIC_MAILBOX in page,
                   f"{label}: public removal contact missing")


def main() -> int:
    gate = Gate()
    incidents = gate.load_json(INCIDENTS)
    advisories = gate.load_json(ADVISORIES)
    hall = gate.load_json(HALL)
    if not all(isinstance(item, dict) for item in (incidents, advisories, hall)):
        print("ERROR: a required JSON document is missing or malformed")
        return 2
    check_incidents(gate, incidents, advisories)
    check_hall(gate, hall)
    check_page(gate, STATUS_PAGE, incidents, True)
    check_page(gate, HALL_PAGE, incidents, False)

    if gate.errors:
        print(f"FAIL: {len(gate.errors)} of {gate.checks} checks failed")
        for error in gate.errors:
            print(f"  - {error}")
        return 1
    print(f"PASS: {gate.checks} checks green (incident history, status page, hall of fame)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
