#!/usr/bin/env python3
"""Offline gate for the public status page, incident history and hall of fame
(#2721). No network, no dependencies.

Failures (all fatal):
  * an incident row that is not published, not past embargo or has no fix;
  * a row that disagrees with the canonical advisory source, or a published
    advisory missing from the incident history;
  * a hall-of-fame entry without recorded opt-in, or carrying private data;
  * a status/hall-of-fame page whose table disagrees with the incident index,
    whose i18n keys are missing from any locale, or which dropped the site
    navigation (bulletin, status, hall of fame, reporting contact);
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
HALL = ROOT / "security/pages/security/hall-of-fame.json"
STATUS_PAGE = ROOT / "security/pages/security/status.html"
HALL_PAGE = ROOT / "security/pages/security/hall-of-fame.html"
BULLETIN = ROOT / "security/pages/security/index.html"
LANG_DIR = ROOT / "security/pages/lang"
LOCALES = ("pt-BR", "en", "es", "fr", "de", "it", "ru", "ja", "ko", "zh-CN")

PUBLIC_MAILBOX = "security@generalbots.org"
RATING = re.compile(r"^(critical|high|medium|low|info)$")
DATE_RE = re.compile(r"^\d{4}-\d{2}-\d{2}$")
ADVISORY_ID = re.compile(r"^GBSA-\d{4}-\d{4}$")
IPV4 = re.compile(r"\b(?:\d{1,3}\.){3}\d{1,3}\b")
EMAIL = re.compile(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b")
TOKEN = re.compile(
    r"(?:VAULT_TOKEN\s*=|ghp_[A-Za-z0-9]|github_pat_|AKIA[0-9A-Z]{16}|"
    r"lv-[A-Za-z0-9_-]{10,}|-----BEGIN [A-Z ]*PRIVATE KEY-----|Bearer\s+[A-Za-z0-9._-]{12,})"
)
I18N_ATTR = re.compile(r'data-i18n(?:-aria-label|-title|-placeholder)?="([^"]+)"')
LINKS_BY_PAGE = {"status.html": ("hall-of-fame.html",),
                 "hall-of-fame.html": ("status.html",)}
SEVERITY_PT = {
    "critical": "Crítica", "high": "Alta",
    "medium": "Média", "low": "Baixa", "info": "Informação",
}


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
        gate.check(bool(DATE_RE.fullmatch(str(row.get("date", "")))), f"{label}: invalid date")
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

    for aid in by_id:
        gate.check(aid in seen, f"advisory {aid}: published advisory missing from incident history")


def check_hall(gate: Gate, hall: dict) -> None:
    entries = hall.get("entries")
    gate.check(isinstance(entries, list), "hall of fame: 'entries' must be a list")
    incentive = hall.get("incentive", "").lower()
    gate.check("0" in incentive and "bounty" in incentive,
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
        gate.check(bool(DATE_RE.fullmatch(str(entry.get("date", "")))), f"{label}: invalid date")
        gate.check(entry.get("role") in ("reporter", "researcher", "internal"),
                   f"{label}: invalid role")
        for private in ("email", "mail", "ip", "phone", "token", "secret", "address"):
            gate.check(private not in entry, f"{label}: private field '{private}' must not be published")
        privacy_scan(gate, label, json.dumps(entry))


def check_catalogs(gate: Gate) -> dict[str, dict[str, str]]:
    catalogs: dict[str, dict[str, str]] = {}
    for locale in LOCALES:
        path = LANG_DIR / f"{locale}.json"
        try:
            data = json.loads(path.read_text(encoding="utf-8"))
        except (OSError, ValueError) as exc:
            gate.check(False, f"{path.name}: cannot parse ({exc})")
            continue
        gate.check(isinstance(data, dict) and all(isinstance(v, str) for v in data.values()),
                   f"{path.name}: expected a flat string map")
        if isinstance(data, dict):
            catalogs[locale] = data
    if len(catalogs) == len(LOCALES):
        reference = set(catalogs["pt-BR"])
        for locale, data in catalogs.items():
            gate.check(set(data) == reference, f"{locale}.json: key set differs from pt-BR")
    return catalogs


def check_page(gate: Gate, path: Path, incidents: dict, is_status: bool,
               catalogs: dict[str, dict[str, str]]) -> None:
    try:
        page = path.read_text(encoding="utf-8")
    except OSError as exc:
        gate.check(False, f"{path.name}: cannot read ({exc})")
        return
    label = path.name
    gate.check(page.count("<h1") == 1, f"{label}: expected exactly one h1")
    gate.check('lang="pt-BR"' in page, f"{label}: lang must default to pt-BR")
    gate.check('class="skip-link"' in page and 'href="#main"' in page,
               f"{label}: skip link missing")
    gate.check('id="main"' in page and "tabindex" in page, f"{label}: main#main not focusable")
    gate.check('data-i18n-loader' in page and 'i18n.js' in page,
               f"{label}: shared catalogue loader missing")
    gate.check('data-authoritative-locale="pt-BR"' in page,
               f"{label}: authoritative locale marker missing")
    gate.check('data-i18n="authoritative-note"' in page, f"{label}: authoritative note missing")
    gate.check("mailto:" + PUBLIC_MAILBOX in page, f"{label}: reporting contact missing")
    for href in LINKS_BY_PAGE[label]:
        gate.check(f'href="{href}"' in page, f"{label}: missing link {href}")
    gate.check('href="./"' in page, f"{label}: link back to the bulletin missing")
    for locale in LOCALES:
        gate.check(f'data-locale="{locale}"' in page, f"{label}: switcher lacks {locale}")
    gate.check("[key]" not in page, f"{label}: unresolved [key] placeholder")
    privacy_scan(gate, label, page)

    used = set(I18N_ATTR.findall(page))
    for locale, catalog in catalogs.items():
        gate.check(used.issubset(set(catalog)),
                   f"{label}: {locale}.json missing keys {sorted(used - set(catalog))}")
    for catalog in catalogs.values():
        for value in catalog.values():
            gate.check(not EMAIL.search(value) or
                       all(item.lower() == PUBLIC_MAILBOX for item in EMAIL.findall(value)),
                       f"{label} catalogue: private email address")
            gate.check(IPV4.search(value) is None and TOKEN.search(value) is None,
                       f"{label} catalogue: IP/token shaped string")

    if is_status:
        gate.check("CTIR Gov" in page and "CERT.br" in page,
                   f"{label}: escalation contacts missing")
        gate.check("embargo" in page.lower(), f"{label}: embargo note missing")
        gate.check("EXEMPLO" in page, f"{label}: example badge missing")
        gate.check("EXAMPLE" in catalogs.get("en", {}).get("status-example-badge", ""),
                   f"{label}: EN example badge missing")
        rows = page_rows(page)
        expected = incidents.get("incidents", [])
        gate.check(len(rows) == len(expected),
                   f"{label}: {len(rows)} rows but {len(expected)} incident records")
        for row, record in zip(rows, expected):
            rid = str(record.get("advisory", ""))
            gate.check(len(row) == 3, f"{label}: row for {rid} has {len(row)} cells, expected 3")
            if len(row) != 3:
                continue
            gate.check(rid in row[0], f"{label}: row advisory id mismatch ({rid})")
            gate.check(str(record.get("date")) in row[1], f"{label}: row date mismatch ({rid})")
            gate.check(SEVERITY_PT[str(record.get("severity"))] in row[1],
                       f"{label}: row severity mismatch ({rid})")
            gate.check(str(record.get("product")) in row[1] and str(record.get("component")) in row[1],
                       f"{label}: row product mismatch ({rid})")
            for version in record.get("affected_versions", []):
                gate.check(str(version) in row[2], f"{label}: affected range mismatch ({rid})")
            gate.check(str(record.get("fixed_version")) in row[2],
                       f"{label}: fixed version mismatch ({rid})")
    else:
        gate.check("recompensas" in page and "0" in page, f"{label}: no-bounty notice missing")
        en = catalogs.get("en", {})
        gate.check("bounty" in en.get("hof-no-rewards", "").lower()
                   and "0" in en.get("hof-no-rewards", ""),
                   f"{label}: EN no-bounty notice missing")
        gate.check("opt-in" in page, f"{label}: opt-in notice missing")
        gate.check("remo" in page, f"{label}: removal process missing")
        gate.check("hall-of-fame.json" in page, f"{label}: registry link missing")
        gate.check("mailto:" + PUBLIC_MAILBOX in page, f"{label}: public removal contact missing")


def check_bulletin(gate: Gate, catalogs: dict[str, dict[str, str]]) -> None:
    try:
        page = BULLETIN.read_text(encoding="utf-8")
    except OSError as exc:
        gate.check(False, f"{BULLETIN.name}: cannot read ({exc})")
        return
    for href in ("status.html", "hall-of-fame.html"):
        gate.check(f'href="{href}"' in page, f"{BULLETIN.name}: missing link {href}")
    for key in ("status-nav-title", "hof-nav-title"):
        gate.check(f'data-i18n="{key}"' in page, f"{BULLETIN.name}: missing {key} link")
        for locale, catalog in catalogs.items():
            gate.check(key in catalog, f"{BULLETIN.name}: {locale}.json missing {key}")


def main() -> int:
    gate = Gate()
    incidents = gate.load_json(INCIDENTS)
    advisories = gate.load_json(ADVISORIES)
    hall = gate.load_json(HALL)
    if not all(isinstance(item, dict) for item in (incidents, advisories, hall)):
        print("ERROR: a required JSON document is missing or malformed")
        return 2
    catalogs = check_catalogs(gate)
    check_incidents(gate, incidents, advisories)
    check_hall(gate, hall)
    check_page(gate, STATUS_PAGE, incidents, True, catalogs)
    check_page(gate, HALL_PAGE, incidents, False, catalogs)
    check_bulletin(gate, catalogs)

    if gate.errors:
        print(f"FAIL: {len(gate.errors)} of {gate.checks} checks failed")
        for error in gate.errors:
            print(f"  - {error}")
        return 1
    print(f"PASS: {gate.checks} checks green (incident history, status page, hall of fame, bulletin links)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
