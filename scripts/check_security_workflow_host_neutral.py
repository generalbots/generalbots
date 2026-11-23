#!/usr/bin/env python3
"""Fail when a security audit job is skipped off GitHub (#2744).

`security.yml` used to gate every audit job behind `github.server_url`, so the
Forgejo/ALM mirror could deploy code that never passed cargo-deny, OSV,
gitleaks, pip-audit or the workflow gates. Only GitHub-only artefacts (SARIF
uploads, the Security tab) may be host-gated.

Usage: python3 scripts/check_security_workflow_host_neutral.py [--workflow PATH]
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

DEFAULT_WORKFLOW = Path(".github/workflows/security.yml")
AUDIT_JOBS = (
    "supply-chain",
    "osv",
    "secrets",
    "python",
    "sbom",
    "workflow-lint",
)
JOB_HEADER = re.compile(r"^  ([A-Za-z0-9_-]+):\s*$")


def job_if_conditions(text: str) -> dict[str, list[str]]:
    """Job-level `if:` expressions keyed by job id (step-level `if:` is ignored)."""
    conditions: dict[str, list[str]] = {}
    current: str | None = None
    for raw in text.splitlines():
        header = JOB_HEADER.match(raw)
        if header:
            current = header.group(1)
            conditions.setdefault(current, [])
            continue
        if current and raw.startswith("    if:"):
            conditions[current].append(raw.strip())
    return conditions


def problems(conditions: dict[str, list[str]]) -> list[str]:
    found = []
    for name in AUDIT_JOBS:
        gated = [value for value in conditions.get(name, []) if "github.server_url" in value]
        if gated:
            found.append(f"{name}: audit job is host-gated: {gated[0]}")
        if name not in conditions:
            found.append(f"{name}: audit job is missing from security.yml")
    return found


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--workflow", type=Path, default=DEFAULT_WORKFLOW)
    args = parser.parse_args()
    text = args.workflow.read_text(encoding="utf-8")
    issues = problems(job_if_conditions(text))
    if issues:
        print("Security audit jobs must run on GitHub and Forgejo (#2744):")
        for issue in issues:
            print(f"  - {issue}")
        return 1
    print(f"All {len(AUDIT_JOBS)} security audit jobs are host-neutral")
    return 0


if __name__ == "__main__":
    sys.exit(main())
