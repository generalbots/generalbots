#!/usr/bin/env python3
"""Prove the supply-chain audits run on BOTH CI hosts (#2744).

`.github/workflows/security.yml` used to gate every audit job behind
`github.server_url == 'https://github.com'`. Since AGENTS.md makes
`git push alm main` the primary CI path, an advisory regression could pass the
canonical mirror and be deployed by alm-ci without cargo-deny, the OSV allowlist
gate, gitleaks, pip-audit or the workflow guards ever running.

This gate parses both CI manifests and asserts:

1. every blocking audit job in the GitHub workflow is reachable when
   `github.server_url` is NOT github.com (no job-level or step-level skip);
2. the only GitHub-gated steps are the two SARIF / Security-tab uploads;
3. `.forgejo/workflows/security.yml` exists, declares the same audit jobs and
   has no server_url skip of its own;
4. every blocking audit command appears in BOTH manifests (parity), so adding
   a gate on one host alone fails the check;
5. neither manifest references repository secrets -- these audits are
   host-independent and must run with the default read-only token.

Usage:
    python3 scripts/check_supply_chain_gates.py
    python3 scripts/check_supply_chain_gates.py --self-test

Exit codes: 0 clean, 1 violation found.
"""

from __future__ import annotations

import argparse
import shutil
import sys
import tempfile
from pathlib import Path

try:
    import yaml
except ImportError:  # pragma: no cover - dependency is present in CI images
    print("error: PyYAML is required to parse the CI manifests", file=sys.stderr)
    raise SystemExit(1)

ROOT = Path(__file__).resolve().parents[1]
GITHUB_WORKFLOW = ROOT / ".github/workflows/security.yml"
FORGEJO_WORKFLOW = ROOT / ".forgejo/workflows/security.yml"

# Jobs that enforce a supply-chain guarantee. All must run on either host.
AUDIT_JOBS = (
    "supply-chain",
    "osv",
    "secrets",
    "python",
    "sbom",
    "workflow-lint",
)

# (command fragment, human-readable label). Every entry must be present in
# both manifests: it is a blocking check, not a reporting nicety.
PARITY_MARKERS = (
    ("cargo-deny --all-features check advisories bans licenses sources", "cargo-deny advisories/bans/licenses/sources"),
    ("python3 scripts/audit-report.py", "RustSec advisory DB record"),
    ("python3 scripts/check_osv_allowlist.py", "OSV dated allowlist gate"),
    ("scripts/osv_allowlist_npm.txt", "npm lockfile advisory baseline"),
    ("gitleaks git", "full-history secret scan"),
    ("pip-audit", "Python dependency audit"),
    ("syft dir:.", "SPDX SBOM generation"),
    ("actionlint", "workflow syntax lint"),
    ("python3 scripts/check_pip_audit_coverage.py", "pip-audit coverage"),
    ("python3 scripts/check_workflow_pinning.py", "uses: SHA pinning"),
    ("python3 scripts/check_component_checksums.py", "component checksum pins"),
    ("--js-manifest botui/ui/suite/js/vendor/vendored-js.toml", "vendored JS hash pins"),
    ("python3 scripts/check_third_party_licenses.py", "third-party license entries"),
    ("python3 scripts/check_workflow_downloads.py", "download checksum verification"),
    ("python3 scripts/check_supply_chain_gates.py", "this parity gate"),
)

SERVER_URL = "github.server_url"
GITHUB_ONLY = "github.server_url == 'https://github.com'"
SARIF_ACTION = "github/codeql-action/upload-sarif"


def load(path: Path) -> dict:
    """Read the `jobs` mapping of a CI manifest, with a clear error on bad YAML."""
    with path.open(encoding="utf-8") as handle:
        doc = yaml.safe_load(handle)
    if not isinstance(doc, dict) or not isinstance(doc.get("jobs"), dict):
        raise ValueError(f"{path}: expected a mapping with a 'jobs' key")
    return doc


def gate_of(mapping: object) -> str:
    value = mapping.get("if") if isinstance(mapping, dict) else None
    return value if isinstance(value, str) else ""


def steps_of(job: dict) -> list:
    steps = job.get("steps")
    return steps if isinstance(steps, list) else []


def check_github(path: Path) -> list[str]:
    """No audit job/step may be skipped when the host is not github.com."""
    errors: list[str] = []
    jobs = load(path)["jobs"]
    missing = [name for name in AUDIT_JOBS if name not in jobs]
    if missing:
        errors.append(f"github: missing audit jobs: {', '.join(missing)}")
        return errors

    for name in AUDIT_JOBS:
        job = jobs[name]
        if SERVER_URL in gate_of(job):
            errors.append(f"github: audit job '{name}' is skipped off GitHub")
        sarif_seen = 0
        for step in steps_of(job):
            if not isinstance(step, dict):
                continue
            condition = gate_of(step)
            label = str(step.get("name", "<unnamed>"))
            if SARIF_ACTION in str(step.get("uses", "")):
                sarif_seen += 1
                if GITHUB_ONLY not in condition:
                    errors.append(
                        f"github: SARIF step '{label}' in '{name}' must be GitHub-only"
                    )
            elif SERVER_URL in condition:
                errors.append(
                    f"github: audit step '{label}' in '{name}' is skipped off GitHub"
                )
        if name in ("osv", "secrets") and sarif_seen < 1:
            errors.append(f"github: job '{name}' has no SARIF upload to gate")
    return errors


def check_forgejo(path: Path) -> list[str]:
    """The ALM mirror must declare every audit job with no server_url gate."""
    if not path.is_file():
        return [f"forgejo: missing mirror workflow {path}"]
    errors: list[str] = []
    jobs = load(path)["jobs"]
    missing = [name for name in AUDIT_JOBS if name not in jobs]
    if missing:
        errors.append(f"forgejo: missing audit jobs: {', '.join(missing)}")
    for name in AUDIT_JOBS:
        if name not in jobs:
            continue
        job = jobs[name]
        if SERVER_URL in gate_of(job):
            errors.append(f"forgejo: audit job '{name}' is gated on server_url")
        for step in steps_of(job):
            if isinstance(step, dict) and SERVER_URL in gate_of(step):
                errors.append(
                    f"forgejo: step '{step.get('name', '<unnamed>')}' in '{name}' "
                    "is gated on server_url"
                )
    return errors


def check_parity(gh_text: str, fj_text: str) -> list[str]:
    errors: list[str] = []
    for marker, label in PARITY_MARKERS:
        for host, text in (("github", gh_text), ("forgejo", fj_text)):
            if marker not in text:
                errors.append(f"{host}: missing blocking audit '{label}' ({marker})")
    return errors


def check_no_secrets(gh_text: str, fj_text: str) -> list[str]:
    errors: list[str] = []
    for host, text in (("github", gh_text), ("forgejo", fj_text)):
        if "secrets." in text:
            errors.append(f"{host}: audit workflow must not reference secrets.*")
    return errors


def collect(gh_path: Path, fj_path: Path) -> list[str]:
    errors = check_github(gh_path)
    errors += check_forgejo(fj_path)
    if gh_path.is_file() and fj_path.is_file():
        errors += check_parity(
            gh_path.read_text(encoding="utf-8"), fj_path.read_text(encoding="utf-8")
        )
        errors += check_no_secrets(
            gh_path.read_text(encoding="utf-8"), fj_path.read_text(encoding="utf-8")
        )
    return errors


def run_self_test(gh_path: Path, fj_path: Path) -> int:
    """Prove the gate fails closed when a server_url skip is planted."""
    with tempfile.TemporaryDirectory(prefix="gb-supply-chain-") as tmp:
        tmp_dir = Path(tmp)
        gh_copy = tmp_dir / "security.github.yml"
        fj_copy = tmp_dir / "security.forgejo.yml"
        shutil.copyfile(gh_path, gh_copy)
        shutil.copyfile(fj_path, fj_copy)

        baseline = collect(gh_copy, fj_copy)
        if baseline:
            print("self-test: clean manifests unexpectedly failed:")
            for error in baseline:
                print(f"  {error}")
            return 1

        needle = (
            "  supply-chain:\n"
            "    name: cargo-deny (advisories, bans, licenses, sources)\n"
        )
        text = gh_copy.read_text(encoding="utf-8")
        if needle not in text:
            print("self-test: cannot locate the supply-chain job fixture")
            return 1
        injected = needle + f"    if: ${{{{ {GITHUB_ONLY} }}}}\n"
        gh_copy.write_text(text.replace(needle, injected, 1), encoding="utf-8")

        mutated = collect(gh_copy, fj_copy)
        expected = "github: audit job 'supply-chain' is skipped off GitHub"
        if expected not in mutated:
            print("self-test: gate did not catch the injected non-GitHub skip")
            return 1

    print("self-test: clean manifests PASS, injected server_url skip detected PASS")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--github", type=Path, default=GITHUB_WORKFLOW)
    parser.add_argument("--forgejo", type=Path, default=FORGEJO_WORKFLOW)
    parser.add_argument(
        "--self-test", action="store_true", help="prove the gate fails closed"
    )
    args = parser.parse_args()

    if args.self_test:
        return run_self_test(args.github, args.forgejo)

    try:
        errors = collect(args.github, args.forgejo)
    except (OSError, ValueError, yaml.YAMLError) as error:
        print(f"error: cannot validate CI manifests: {error}", file=sys.stderr)
        return 1
    if errors:
        for error in errors:
            print(f"error: {error}")
        print(f"\n{len(errors)} supply-chain gate violation(s)")
        return 1
    print(
        f"supply-chain gates: {len(AUDIT_JOBS)} audit jobs, "
        f"{len(PARITY_MARKERS)} parity checks -- clean on GitHub and Forgejo"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
