#!/usr/bin/env python3
"""Markdown rendering for the dependency advisory drafter (#2725).

Kept separate from normalization so the pure decision logic can be reviewed
without the presentation layer. Import the shared types from
`advisory_draft_lib`.
"""
from __future__ import annotations

from datetime import date
from string import Template

from advisory_draft_lib import DRAFT_STATUSES, Decision, Finding

# --------------------------------------------------------------------------- #
# Rendering
# --------------------------------------------------------------------------- #
def _table(headers: list[str], rows: list[list[str]]) -> list[str]:
    if not rows:
        return ["_None._", ""]
    lines = ["| " + " | ".join(headers) + " |", "|" + "|".join("---" for _ in headers) + "|"]
    lines.extend("| " + " | ".join(cell.replace("|", "\\|") for cell in row) + " |" for row in rows)
    lines.append("")
    return lines


def render_report(decisions: list[Decision], today: date, token: str) -> str:
    drafts = [d for d in decisions if d.status in DRAFT_STATUSES]
    active = [d for d in decisions if d.status == "waived"]
    expired = [d for d in decisions if d.status == "expired-waiver"]
    sections: list[str] = [
        "# Dependency advisory draft report",
        "",
        f"- Generated: {today.isoformat()} (run `{token}`)",
        f"- Findings: {len(decisions)} | drafts needed: **{len(drafts)}** | "
        f"waived: {len(active)} | expired waivers: {len(expired)}",
        "- **Nothing here is published.** Draft issues go to human triage "
        "and the two-person review gate (#2724); publication is a separate, "
        "manual step.",
        "",
        "## New findings (draft advisory required)",
        "",
    ]
    sections += _table(
        ["Advisory", "Ecosystem", "Package", "Installed", "Range", "CVE / CWE", "Source"],
        [
            [
                d.finding.advisory_id,
                d.finding.ecosystem,
                d.finding.package,
                d.finding.installed_version,
                d.finding.vulnerable_range,
                ", ".join((*d.finding.cves, *d.finding.cwes)) or "-",
                d.finding.source_file,
            ]
            for d in drafts
        ],
    )
    sections += ["## Active waivers (visible, expiry enforced)", ""]
    sections += _table(
        ["Advisory", "Approver", "Expires", "Reason"],
        [
            [d.finding.advisory_id, d.waiver.approver, d.waiver.expires.isoformat(), d.waiver.reason]
            for d in active
        ],
    )
    sections += ["## Expired waivers (suppression no longer honoured)", ""]
    sections += _table(
        ["Advisory", "Approver", "Expired on", "Action"],
        [
            [d.finding.advisory_id, d.waiver.approver, d.waiver.expires.isoformat(), d.note]
            for d in expired
        ],
    )
    sections += ["## Deduplicated / already tracked", ""]
    sections += _table(
        ["Advisory", "Status", "Note"],
        [
            [d.finding.advisory_id, d.status, d.note]
            for d in decisions
            if d.status in ("published", "already-drafted")
        ],
    )
    return "\n".join(sections).rstrip() + "\n"


def render_draft(finding: Finding, template: str, report_ref: str) -> str:
    values = {
        "advisory_id": finding.advisory_id,
        "ecosystem": finding.ecosystem,
        "package": finding.package,
        "installed_version": finding.installed_version,
        "vulnerable_range": finding.vulnerable_range,
        "cves": ", ".join(finding.cves) or "none reported",
        "cwes": ", ".join(finding.cwes) or "none reported",
        "severity": finding.severity or "unspecified",
        "source_file": finding.source_file,
        "summary": finding.summary or "No summary provided by the source advisory.",
        "aliases": ", ".join(finding.aliases) or "none",
        "references": "\n".join(f"- {url}" for url in finding.references) or "- none provided",
        "report_ref": report_ref,
    }
    return Template(template).safe_substitute(values)


