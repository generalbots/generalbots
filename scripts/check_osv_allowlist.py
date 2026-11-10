#!/usr/bin/env python3
"""Fail when OSV reports an advisory that is not in the dated allowlist.

`cargo-deny` is the blocking Rust gate (see `deny.toml`); the OSV job adds
cross-ecosystem coverage and the SARIF trail. Because the OSV scan is allowed
to exit non-zero for advisories we already accepted, a *new* advisory would be
easy to miss in the upload. This check closes that gap: it compares every
advisory ID in the OSV JSON report against `scripts/osv_allowlist.txt` and
fails on anything unknown.

Input
-----
The report must be `osv-scanner scan source --format json` output. SARIF is
deliberately not accepted: its `ruleId` is the CVE (not the RustSec/GHSA ID
used by the allowlist), so a SARIF diff would either miss aliases or report
false new IDs.

Usage
-----
    python3 scripts/check_osv_allowlist.py \
        --report /tmp/osv.json \
        --allowlist scripts/osv_allowlist.txt

Exit codes: 0 clean, 1 unknown advisory(ies), 2 unreadable/invalid report.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

DEFAULT_ALLOWLIST = Path("scripts/osv_allowlist.txt")


def load_allowlist(path: Path) -> set[str]:
    allowed: set[str] = set()
    for lineno, raw in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
        line = raw.split("#", 1)[0].strip()
        if not line:
            continue
        advisory = line.split()[0]
        if not advisory:
            continue
        allowed.add(advisory)
    if not allowed:
        raise ValueError(f"{path} contains no advisory IDs")
    return allowed


def parse_osv(report: dict) -> dict[str, set[str]]:
    """Return advisory ID -> affected package@version set."""
    results = report.get("results")
    if results is None:
        raise ValueError(
            "report has no `results` key; pass osv-scanner JSON (`--format json`), not SARIF"
        )

    found: dict[str, set[str]] = {}
    for result in results or []:
        for package in result.get("packages") or []:
            meta = package.get("package") or {}
            label = f"{meta.get('name', '?')}@{meta.get('version', '?')}"
            advisory_ids: set[str] = set()
            for group in package.get("groups") or []:
                advisory_ids.update(group.get("ids") or [])
            for vulnerability in package.get("vulnerabilities") or []:
                advisory_id = vulnerability.get("id")
                if advisory_id:
                    advisory_ids.add(advisory_id)
            for advisory_id in advisory_ids:
                found.setdefault(advisory_id, set()).add(label)
    return found


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--report", type=Path, help="osv-scanner JSON report (required unless --self-test)")
    parser.add_argument(
        "--allowlist",
        type=Path,
        default=DEFAULT_ALLOWLIST,
        help=f"allowlist file (default: {DEFAULT_ALLOWLIST})",
    )
    parser.add_argument("--self-test", action="store_true", help="prove the gate fails closed")
    args = parser.parse_args()

    if args.self_test:
        from gate_selftests import run_self_test

        return run_self_test("osv-allowlist")

    if args.report is None:
        print("error: --report is required", file=sys.stderr)
        return 2

    try:
        allowed = load_allowlist(args.allowlist)
    except (OSError, ValueError) as error:
        print(f"error: cannot read allowlist {args.allowlist}: {error}")
        return 2

    try:
        report = json.loads(args.report.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        print(f"error: cannot read report {args.report}: {error}")
        return 2

    if not isinstance(report, dict):
        print(f"error: {args.report} is not an OSV JSON object")
        return 2

    try:
        found = parse_osv(report)
    except ValueError as error:
        print(f"error: {error}")
        return 2

    unknown = {advisory: pkgs for advisory, pkgs in found.items() if advisory not in allowed}
    stale = sorted(allowed - set(found))

    for advisory in sorted(unknown):
        packages = ", ".join(sorted(unknown[advisory]))
        print(f"error: {advisory} is not in {args.allowlist} (affects {packages})")
        print("       triage it, then add a dated entry with a reachability argument")

    for advisory in stale:
        print(f"note: {advisory} is allowlisted but no longer reported (fixed or withdrawn?)")

    if unknown:
        print(f"\n{len(unknown)} unknown advisory(ies); {len(found)} reported, {len(allowed)} allowlisted")
        return 1

    print(
        f"ok: {len(found)} advisory(ies) reported, all covered by "
        f"{len(allowed)} allowlisted entries ({len(stale)} stale)"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
