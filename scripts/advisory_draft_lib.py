#!/usr/bin/env python3
"""Pure helpers for the dependency advisory drafter (#2725).

Every decision here is a function of the caller-supplied reports, waivers and
`today` date only: no network, no implicit clock, no repository state. That is
what lets `draft_advisories.py --self-test` exercise the whole pipeline offline
and keeps the CI job reproducible.

The drafter never *publishes* anything. It normalizes scanner findings,
deduplicates them against published advisories / already-open draft issues,
applies expiring waivers, and renders a report plus draft issue bodies.
"""
from __future__ import annotations

import json
import re
import tomllib
from dataclasses import dataclass
from datetime import date
from pathlib import Path
from string import Template
from typing import Any

CVE_RE = re.compile(r"CVE-\d{4}-\d{4,}", re.I)
CWE_RE = re.compile(r"CWE-\d+", re.I)
ADVISORY_ID_RE = re.compile(
    r"\b(?:RUSTSEC-\d{4}-\d+|GHSA-[0-9a-z]{4}-[0-9a-z]{4}-[0-9a-z]{4}"
    r"|PYSEC-\d{4}-\d+|CVE-\d{4}-\d{4,})\b",
    re.I,
)

DRAFT_STATUSES = ("new", "expired-waiver")


# --------------------------------------------------------------------------- #
# Data model
# --------------------------------------------------------------------------- #
@dataclass(frozen=True)
class Finding:
    advisory_id: str
    ecosystem: str
    package: str
    installed_version: str
    vulnerable_range: str
    aliases: tuple[str, ...]
    summary: str
    references: tuple[str, ...]
    severity: str
    source_file: str

    @property
    def cves(self) -> tuple[str, ...]:
        haystack = " ".join((self.advisory_id, *self.aliases, self.summary))
        return tuple(sorted({m.upper() for m in CVE_RE.findall(haystack)}))

    @property
    def cwes(self) -> tuple[str, ...]:
        return tuple(sorted({m.upper() for m in CWE_RE.findall(self.summary)}))

    @property
    def identity(self) -> str:
        """Stable key for dedup: the canonical (lexically smallest) alias."""
        return min((self.advisory_id.upper(), *(a.upper() for a in self.aliases)))


@dataclass(frozen=True)
class Waiver:
    advisory_id: str
    package: str
    reason: str
    approver: str
    expires: date

    def active_on(self, today: date) -> bool:
        return self.expires >= today


@dataclass(frozen=True)
class Decision:
    finding: Finding
    status: str  # new | waived | expired-waiver | published | already-drafted
    waiver: Waiver | None
    note: str


# --------------------------------------------------------------------------- #
# Report normalization
# --------------------------------------------------------------------------- #
def _text(value: Any) -> str:
    if value is None:
        return ""
    return value if isinstance(value, str) else str(value)


def _ref_url(ref: Any) -> str:
    if isinstance(ref, dict):
        return _text(ref.get("url"))
    return _text(ref)


def _osv_vuln(vuln: dict, package: str, version: str, source: str) -> Finding:
    ranges: list[str] = []
    for affected in vuln.get("affected") or []:
        if not isinstance(affected, dict):
            continue
        for rng in affected.get("ranges") or []:
            for event in rng.get("events") or []:
                for key in ("introduced", "fixed", "last_affected"):
                    if key in event:
                        ranges.append(f"{key}={event[key]}")
        ranges.extend(f"version={v}" for v in affected.get("versions") or [])
    db = vuln.get("database_specific") or {}
    return Finding(
        advisory_id=_text(vuln.get("id")) or "UNKNOWN",
        ecosystem="osv",
        package=package,
        installed_version=version,
        vulnerable_range=", ".join(dict.fromkeys(ranges)) or "unknown",
        aliases=tuple(_text(a) for a in vuln.get("aliases") or []),
        summary=_text(vuln.get("summary")) or _text(vuln.get("details")),
        references=tuple(
            url for url in (_ref_url(r) for r in vuln.get("references") or []) if url
        ),
        severity=_text(db.get("severity")) or _text(vuln.get("severity")),
        source_file=source,
    )


def findings_from_osv(doc: Any, source: str) -> list[Finding]:
    results = doc.get("results") if isinstance(doc, dict) else None
    if not isinstance(results, list):
        raise ValueError(f"{source}: expected osv-scanner JSON with a 'results' array")
    out: list[Finding] = []
    for result in results:
        for pkg in (result or {}).get("packages") or []:
            meta = pkg.get("package") or {}
            package = _text(meta.get("name")) or "?"
            version = _text(meta.get("version")) or "?"
            out.extend(
                _osv_vuln(v, package, version, source)
                for v in pkg.get("vulnerabilities") or []
                if isinstance(v, dict)
            )
    return out


def findings_from_pip_audit(doc: Any, source: str) -> list[Finding]:
    if not isinstance(doc, list):
        raise ValueError(f"{source}: expected pip-audit JSON array")
    out: list[Finding] = []
    for entry in doc:
        package = _text(entry.get("name")) or "?"
        version = _text(entry.get("version")) or "?"
        for vuln in entry.get("vulns") or []:
            fix = ", ".join(_text(f) for f in vuln.get("fix_versions") or [])
            out.append(
                Finding(
                    advisory_id=_text(vuln.get("id")) or "UNKNOWN",
                    ecosystem="python",
                    package=package,
                    installed_version=version,
                    vulnerable_range=f"versions < {fix}" if fix else "unknown",
                    aliases=tuple(_text(a) for a in vuln.get("aliases") or []),
                    summary=_text(vuln.get("description")),
                    references=tuple(
                        url
                        for url in (_ref_url(r) for r in vuln.get("references") or [])
                        if url
                    ),
                    severity=_text(vuln.get("severity")),
                    source_file=source,
                )
            )
    return out


def findings_from_deny(doc: Any, source: str) -> list[Finding]:
    if not isinstance(doc, dict):
        raise ValueError(f"{source}: expected cargo-deny JSON object")
    diagnostics = doc.get("diagnostics") or {}
    items = diagnostics.get("advisories") if isinstance(diagnostics, dict) else None
    if items is None:
        items = doc.get("advisories")
    if not isinstance(items, list):
        raise ValueError(f"{source}: cargo-deny JSON has no 'diagnostics.advisories'")
    out: list[Finding] = []
    for item in items:
        adv = (item or {}).get("advisory") or {}
        package = (item.get("package") or {}).get("name") or adv.get("package") or "?"
        versions = ", ".join(_text(v) for v in item.get("versions") or [])
        urls = [u for u in [_text(adv.get("url")), _text(adv.get("link"))] if u]
        out.append(
            Finding(
                advisory_id=_text(adv.get("id")) or "UNKNOWN",
                ecosystem="rust",
                package=_text(package),
                installed_version=versions or "unknown",
                vulnerable_range=versions or "unknown",
                aliases=tuple(_text(a) for a in adv.get("aliases") or []),
                summary=_text(adv.get("title")) or _text(adv.get("description")),
                references=tuple(urls),
                severity=_text(item.get("severity")) or _text(adv.get("severity")),
                source_file=source,
            )
        )
    return out


def dedupe(findings: list[Finding]) -> list[Finding]:
    """One finding per advisory/alias family; merge aliases from twin reports."""
    merged: dict[str, Finding] = {}
    for finding in findings:
        key = finding.identity
        if key not in merged:
            merged[key] = finding
            continue
        first = merged[key]
        aliases = tuple(dict.fromkeys((*first.aliases, finding.advisory_id, *finding.aliases)))
        merged[key] = Finding(
            advisory_id=first.advisory_id,
            ecosystem=first.ecosystem,
            package=first.package,
            installed_version=first.installed_version,
            vulnerable_range=first.vulnerable_range,
            aliases=tuple(a for a in aliases if a.upper() != first.advisory_id.upper()),
            summary=first.summary or finding.summary,
            references=tuple(dict.fromkeys((*first.references, *finding.references))),
            severity=first.severity or finding.severity,
            source_file=first.source_file,
        )
    return [merged[key] for key in sorted(merged)]


# --------------------------------------------------------------------------- #
# Waivers and dedup inputs
# --------------------------------------------------------------------------- #
def load_waivers(path: Path) -> tuple[list[Waiver], list[str]]:
    if not path.is_file():
        return [], []
    try:
        data = tomllib.loads(path.read_text(encoding="utf-8"))
    except (OSError, tomllib.TOMLDecodeError) as error:
        return [], [f"{path}: cannot parse waivers: {error}"]
    out: list[Waiver] = []
    errors: list[str] = []
    seen: set[str] = set()
    for index, raw in enumerate(data.get("waiver") or [], start=1):
        if not isinstance(raw, dict):
            errors.append(f"waiver #{index}: expected a [[waiver]] table")
            continue
        wid = _text(raw.get("id"))
        approver = _text(raw.get("approver"))
        if not wid or not approver:
            errors.append(f"waiver #{index}: both id and approver are required")
            continue
        if wid in seen:
            errors.append(f"waiver '{wid}': duplicate id")
            continue
        try:
            expires = date.fromisoformat(_text(raw.get("expires")))
        except ValueError:
            errors.append(f"waiver '{wid}': expires must be an ISO date (YYYY-MM-DD)")
            continue
        seen.add(wid)
        out.append(
            Waiver(
                advisory_id=wid,
                package=_text(raw.get("package")),
                reason=_text(raw.get("reason")) or "no reason recorded",
                approver=approver,
                expires=expires,
            )
        )
    return out, errors


def load_ids(path: Path) -> set[str]:
    """Read a newline-separated ID list; everything after '#' is a comment."""
    if not path.is_file():
        return set()
    ids: set[str] = set()
    for raw in path.read_text(encoding="utf-8").splitlines():
        body = raw.split("#", 1)[0].strip()
        if body:
            ids.add(body.split()[0].upper())
    return ids


def ids_in_text(text: str) -> set[str]:
    return {m.upper() for m in ADVISORY_ID_RE.findall(text or "")}


def load_state_ids(path: Path) -> set[str]:
    if not path.is_file():
        return set()
    try:
        state = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return set()
    return {str(key).upper() for key in state if isinstance(state, dict)}


def classify(
    findings: list[Finding],
    waivers: list[Waiver],
    published_ids: set[str],
    existing_ids: set[str],
    today: date,
) -> list[Decision]:
    by_id = {w.advisory_id.upper(): w for w in waivers}
    decisions: list[Decision] = []
    for finding in findings:
        ids = {finding.advisory_id.upper(), *(a.upper() for a in finding.aliases)}
        waiver = by_id.get(finding.advisory_id.upper())
        if ids & published_ids:
            status, note = "published", "already published; deduplicated"
        elif ids & existing_ids:
            status, note = "already-drafted", "draft issue already open; not reopened"
        elif waiver and not waiver.active_on(today):
            status = "expired-waiver"
            note = (
                f"waiver expired {waiver.expires.isoformat()} (approver "
                f"{waiver.approver}); re-triage and draft required"
            )
        elif waiver:
            status = "waived"
            note = (
                f"active until {waiver.expires.isoformat()} by {waiver.approver}: "
                f"{waiver.reason}"
            )
        else:
            status, note = "new", "new finding; draft advisory required"
        decisions.append(Decision(finding, status, waiver, note))
    return decisions


