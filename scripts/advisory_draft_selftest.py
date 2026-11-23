#!/usr/bin/env python3
"""Offline fixtures for `draft_advisories.py --self-test` (#2725).

No network, no repository state, fixed clock: proves normalization, dedup,
waiver expiry, deduplication and rendering behave as the workflow expects.
"""
from __future__ import annotations

import json
from datetime import date, timedelta
from pathlib import Path
from tempfile import TemporaryDirectory

from advisory_draft_lib import (
    classify,
    dedupe,
    findings_from_deny,
    findings_from_osv,
    findings_from_pip_audit,
    ids_in_text,
    load_ids,
    load_waivers,
)
from advisory_draft_render import render_draft, render_report

# --------------------------------------------------------------------------- #
# Offline self-test
# --------------------------------------------------------------------------- #
def run_self_test() -> int:
    today = date(2026, 11, 23)
    with TemporaryDirectory(prefix="gb-advisory-") as tmp:
        root = Path(tmp)
        osv = {
            "results": [
                {
                    "source": {"path": "Cargo.lock"},
                    "packages": [
                        {
                            "package": {"name": "newcrate", "version": "1.2.0"},
                            "vulnerabilities": [
                                {
                                    "id": "RUSTSEC-2026-9999",
                                    "aliases": ["CVE-2026-12345"],
                                    "summary": "buffer overflow CWE-120 in newcrate",
                                    "references": [{"url": "https://example.test/adv"}],
                                    "affected": [
                                        {"ranges": [{"events": [{"introduced": "0"}, {"fixed": "1.2.1"}]}]}
                                    ],
                                    "database_specific": {"severity": "HIGH"},
                                }
                            ],
                        }
                    ],
                }
            ]
        }
        pip = [
            {
                "name": "waivedpkg",
                "version": "1.0.0",
                "vulns": [
                    {"id": "GHSA-aaaa-bbbb-cccc", "aliases": ["CVE-2026-22222"],
                     "description": "issue", "fix_versions": ["1.1.0"], "references": []}
                ],
            },
            {
                "name": "expiredpkg",
                "version": "2.0.0",
                "vulns": [
                    {"id": "PYSEC-2026-7777", "aliases": [], "description": "issue",
                     "fix_versions": [], "references": []}
                ],
            },
            {
                "name": "publishedpkg",
                "version": "3.0.0",
                "vulns": [
                    {"id": "GHSA-dddd-eeee-ffff", "aliases": [], "description": "issue",
                     "fix_versions": [], "references": []}
                ],
            },
            {
                "name": "draftedpkg",
                "version": "4.0.0",
                "vulns": [
                    {"id": "RUSTSEC-2026-8888", "aliases": [], "description": "issue",
                     "fix_versions": [], "references": []}
                ],
            },
        ]
        deny = {
            "diagnostics": {
                "advisories": [
                    {
                        "advisory": {"id": "RUSTSEC-2026-8888", "title": "alias of drafted finding",
                                     "url": "https://example.test/dup", "aliases": []},
                        "package": {"name": "draftedpkg"},
                        "versions": ["4.0.0"],
                    }
                ]
            }
        }
        (root / "osv.json").write_text(json.dumps(osv), encoding="utf-8")
        (root / "pip.json").write_text(json.dumps(pip), encoding="utf-8")
        (root / "deny.json").write_text(json.dumps(deny), encoding="utf-8")
        (root / "waivers.toml").write_text(
            "\n".join(
                [
                    "[[waiver]]",
                    'id = "GHSA-aaaa-bbbb-cccc"',
                    'approver = "@security-team"',
                    f'expires = "{(today + timedelta(days=30)).isoformat()}"',
                    'reason = "not reachable"',
                    "",
                    "[[waiver]]",
                    'id = "PYSEC-2026-7777"',
                    'approver = "@security-team"',
                    'expires = "2026-01-01"',
                    'reason = "temporary"',
                ]
            ),
            encoding="utf-8",
        )
        (root / "published.txt").write_text("GHSA-dddd-eeee-ffff\n", encoding="utf-8")
        (root / "drafts.txt").write_text("RUSTSEC-2026-8888\n", encoding="utf-8")

        findings = dedupe(
            findings_from_osv(osv, "osv.json")
            + findings_from_pip_audit(pip, "pip.json")
            + findings_from_deny(deny, "deny.json")
        )
        waivers, errors = load_waivers(root / "waivers.toml")
        assert not errors, errors
        decisions = classify(
            findings,
            waivers,
            load_ids(root / "published.txt"),
            load_ids(root / "drafts.txt"),
            today,
        )
        got = {d.finding.advisory_id: d.status for d in decisions}
        expected = {
            "RUSTSEC-2026-9999": "new",
            "GHSA-aaaa-bbbb-cccc": "waived",
            "PYSEC-2026-7777": "expired-waiver",
            "GHSA-dddd-eeee-ffff": "published",
            "RUSTSEC-2026-8888": "already-drafted",
        }
        assert got == expected, f"classification mismatch: {got} != {expected}"
        assert len(findings) == 5, f"alias dedup failed: {len(findings)} findings"
        report = render_report(decisions, today, "self-test")
        assert "drafts needed: **2**" in report, "report did not count expired waiver as draft"
        assert "@security-team" in report and "expired" in report.lower(), "waivers not audited"
        template = "ID=$advisory_id PKG=$package RANGE=$vulnerable_range CVE=$cves CWE=$cwes REF=$references"
        body = render_draft(decisions[0].finding, template, "report.md")
        assert "ID=RUSTSEC-2026-9999" in body and "CVE=CVE-2026-12345" in body, body
        assert "CWE=CWE-120" in body, body
        assert ids_in_text("see GHSA-aaaa-bbbb-cccc and CVE-2026-12345") == {
            "GHSA-AAAA-BBBB-CCCC", "CVE-2026-12345"
        }
        print("self-test: normalize/dedupe/classify/report/draft PASS (5 findings, 2 drafts)")
    return 0
