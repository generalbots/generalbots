#!/usr/bin/env python3
"""Production readiness gate (#2121).

The epic is only credible if its claims are executable. This is the single
entry point that runs every readiness check the repository already owns and
reports one row per check with the issue that owns it:

    python3 scripts/prod_readiness.py                  # table
    python3 scripts/prod_readiness.py --json           # machine output
    python3 scripts/prod_readiness.py --markdown       # rewrite the scoreboard
    python3 scripts/prod_readiness.py --check-markdown # fail if the page drifted

Statuses
--------
``PASS``  the check ran and passed.
``FAIL``  the check ran and failed — this is a regression, and the gate exits 1.
``SKIP``  the check does not exist yet (or its baseline is not committed). The
          row is still printed with the owning issue and what has to land, so a
          missing gate is visible instead of silent. ``SKIP`` is never a pass.

Exit codes: 0 = no FAIL, 1 = at least one FAIL, 3 = the committed scoreboard
does not match the gate output (``--check-markdown``).

Adding a check: append a ``Gate`` to ``GATES`` (or a child to ``CHILDREN`` when
it belongs to a workstream) — the table, the JSON and the BotBook scoreboard are
all rendered from the same list, so nothing has to be edited twice.
"""

from __future__ import annotations

import argparse
import json
import re
import shutil
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOC = "botbook/src/10-configuration-deployment/production-readiness.md"
EPIC = 2121
TIMEOUT = 300

PASS, FAIL, SKIP = "PASS", "FAIL", "SKIP"

# Markers that prove the Security workflow still runs every scan. Used as the
# offline fallback when the gitleaks binary is not installed in this
# environment (CI installs it in `security.yml`, not here).
SECURITY_SCANS = ("cargo-deny", "osv-scanner", "gitleaks", "pip-audit", "syft", "actionlint")


@dataclass(frozen=True)
class Gate:
    """One readiness check."""

    id: str
    title: str
    owner: int
    proves: str
    steps: tuple[tuple[str, ...], ...] = ()
    requires: tuple[str, ...] = ()
    evidence: tuple[str, ...] = ()
    min_files: int = 1
    ok_detail: str = ""
    fixed_detail: bool = False


@dataclass(frozen=True)
class Result:
    gate: Gate
    status: str
    detail: str


GATES: tuple[Gate, ...] = (
    Gate(
        id="security-auth",
        title="JWT claims are signature-verified before they pick a tenant",
        owner=2112,
        proves="A forged/`alg=none` token cannot select another tenant's branch",
        evidence=("bottest/tests/unit/security/*jwt*.rs", "bottest/tests/unit/security/*tenant*.rs"),
        ok_detail="regression test present (executed by the unit-tests job)",
    ),
    Gate(
        id="security-webhooks",
        title="WhatsApp/Teams inbound requests prove their origin",
        owner=2113,
        proves="An unauthenticated POST cannot create a bot turn",
        evidence=("bottest/tests/unit/security/*webhook*.rs",),
        ok_detail="regression test present (executed by the unit-tests job)",
    ),
    Gate(
        id="security-mfa",
        title="TOTP enrollment and verification share one secret encoding",
        owner=2114,
        proves="A code from the QR secret is accepted by the server",
        evidence=("bottest/tests/unit/security/*mfa*.rs", "bottest/tests/unit/security/*totp*.rs"),
        ok_detail="regression test present (executed by the unit-tests job)",
    ),
    Gate(
        id="slo-check",
        title="SLOs are defined, instrumented and gated",
        owner=2115,
        proves="A latency/error/cost regression fails instead of shipping silently",
        steps=(
            ("python3", "scripts/check_slo.py", "--self-test"),
            ("python3", "scripts/check_slo.py", "--snapshot", "perf/baselines/local-mock.json"),
        ),
        requires=("ops/slo.yaml", "perf/baselines/local-mock.json"),
    ),
    Gate(
        id="ops-runbooks",
        title="Operational runbooks exist for the on-call surface",
        owner=2116,
        proves="Analytics console, error budgets and runbooks are operable, not prose",
        evidence=("botbook/src/10-configuration-deployment/operations/runbook-*.md",),
        min_files=6,
        ok_detail="six runbooks published",
    ),
    Gate(
        id="data-restore",
        title="Restore drill proves migrations, tenancy and recovery together",
        owner=2117,
        proves="A production-like restore boots and stays tenant-isolated",
        evidence=("scripts/restore_drill.sh", "scripts/restore_drill.py"),
        ok_detail="restore drill script present",
    ),
    Gate(
        id="release-version",
        title="One version everywhere, and the tag is applied to the build",
        owner=2118,
        proves="A tag and `/api/product.version` cannot disagree",
        steps=(("python3", "scripts/release_version.py", "--check"), ("python3", "scripts/release_version.py", "selftest")),
    ),
    Gate(
        id="test-guards",
        title="Unit-suite policy: registered modules, reasons for skips, offline",
        owner=2119,
        proves="The offline suite stays hermetic and reviewable",
        steps=(("python3", "scripts/test_guards.py", "--check"), ("python3", "scripts/test_guards.py", "--self-test")),
    ),
    Gate(
        id="app-contracts",
        title="Every registry app declares its classification, smoke and API surface",
        owner=2119,
        proves="A shipped app is documented and an invented route fails the build",
        steps=(("python3", "scripts/smoke_app_surfaces.py", "--contracts"),),
        ok_detail="every registry app classified, smoked and documented",
        fixed_detail=True,
    ),
    Gate(
        id="flake-policy",
        title="Flaky tests are registered, owned and inside the quarantine SLA",
        owner=2119,
        proves="A flake is time-boxed instead of retried until it passes",
        steps=(
            ("python3", "scripts/flake_report.py", "--self-test"),
            ("python3", "scripts/flake_report.py", "--quarantine-only"),
        ),
        ok_detail="no flake outside its SLA",
        fixed_detail=True,
    ),
    Gate(
        id="coverage-ratchet",
        title="Coverage ratchet against a committed baseline",
        owner=2119,
        proves="A PR cannot lower the covered lines of the crate it touches",
        steps=(("python3", "scripts/check_coverage.py", "--self-test"),),
        requires=("issues/coverage/baseline.json",),
        ok_detail="baseline committed; the ratchet self-test fails on a decrease",
        fixed_detail=True,
    ),
    Gate(
        id="product-profile",
        title="Every app is classified always/preview/hidden and its feature compiles",
        owner=2120,
        proves="The supported surface is bounded and owned",
        steps=(
            ("python3", "scripts/product_profile_check.py"),
            ("python3", "scripts/app_complexity_budget.py"),
        ),
        ok_detail="every app classified, owned and within the complexity budget",
        fixed_detail=True,
    ),
    Gate(
        id="security-scan",
        title="Supply-chain and secret scans are wired in CI",
        owner=EPIC,
        proves="cargo-deny, OSV, gitleaks, pip-audit, SBOM and actionlint still run",
        requires=(".gitleaks.toml", "deny.toml", "scripts/osv_allowlist.txt"),
    ),
    Gate(
        id="adequacy-guard",
        title="AGENTS.md structural rules stay at or below the baseline",
        owner=1735,
        proves="No new `unwrap`, `table!` outside botschema, 450+ line file or CDN asset",
        steps=(("python3", "scripts/adequacy_guard.py", "--baseline", "issues/adequacy/baseline.json"),),
    ),
    Gate(
        id="i18n-lint",
        title="Fluent catalogs and the keys the UI asks for are consistent",
        owner=2090,
        proves="A new user-facing string cannot ship untranslated or unbalanced",
        steps=(("python3", "scripts/i18n_lint.py", "--check"),),
        ok_detail="no new catalog violation",
        fixed_detail=True,
    ),
    Gate(
        id="workflow-pinning",
        title="Every `uses:` is a commit SHA and no workflow trusts a fork",
        owner=1726,
        proves="A retargeted action tag cannot execute with the repository token",
        steps=(("python3", "scripts/check_workflow_pinning.py"),),
        ok_detail="every `uses:` is a commit SHA; no workflow trusts a fork",
        fixed_detail=True,
    ),
)

# Child issue -> the gate(s) that prove it. The scoreboard's second table is
# rendered from this map, so an issue cannot be listed without a gate.
CHILDREN: tuple[tuple[int, str, tuple[str, ...]], ...] = (
    (2112, "security(auth): JWT signature", ("security-auth",)),
    (2113, "security(webhooks): WhatsApp/Teams auth", ("security-webhooks",)),
    (2114, "security(mfa): TOTP encoding", ("security-mfa",)),
    (2115, "perf(slo): SLOs and budgets", ("slo-check",)),
    (2116, "ops(analytics): console, budgets, runbooks", ("ops-runbooks",)),
    (2117, "data(integrity): migrations, restore, tenancy", ("data-restore",)),
    (2118, "release(auto-bump): one version", ("release-version",)),
    (2119, "test(quality): ratchet, contracts, fuzzing", ("test-guards", "app-contracts", "coverage-ratchet", "flake-policy")),
    (2120, "product(focus): supported matrix", ("product-profile",)),
)


def _last_line(text: str) -> str:
    """The tool's own summary: last real line, ignoring per-rule notes."""
    for line in reversed([line.strip() for line in text.splitlines()]):
        if line and not line.startswith(("- ", "note:", "remediation:", "run: ")):
            return _stable(line)
    return ""


def _stable(text: str) -> str:
    """Drop wall-clock/file-count noise so the scoreboard cannot drift."""
    text = re.sub(r"\s*\([^()]*\bscanned\b[^()]*\)", "", text)
    return re.sub(r"\s*\(?\bin [\d.]+s\b\)?", "", text).strip()


def _clip(text: str, width: int = 78) -> str:
    return text if len(text) <= width else text[: width - 1] + "…"


def run_commands(gate: Gate, steps: tuple[tuple[str, ...], ...] | None = None) -> Result:
    """Run each step in order; the first non-zero exit fails the gate."""
    detail = gate.ok_detail
    for step in steps or gate.steps:
        argv = [sys.executable if part == "python3" else part for part in step]
        try:
            proc = subprocess.run(argv, cwd=ROOT, capture_output=True, text=True, timeout=TIMEOUT)
        except subprocess.TimeoutExpired:
            return Result(gate, FAIL, f"`{' '.join(step)}` timed out after {TIMEOUT}s")
        except OSError as error:
            return Result(gate, SKIP, f"cannot run `{' '.join(step)}`: {error}")
        if proc.returncode != 0:
            reason = _last_line(proc.stderr) or _last_line(proc.stdout) or "no output"
            return Result(gate, FAIL, f"`{' '.join(step)}` exited {proc.returncode}: {reason}")
        if not gate.fixed_detail:
            detail = _last_line(proc.stdout) or detail
    return Result(gate, PASS, detail)


def check_evidence(gate: Gate) -> Result:
    """A gate that is installed by a file landing in the repository."""
    found = sorted({path for pattern in gate.evidence for path in ROOT.glob(pattern)})
    if len(found) < gate.min_files:
        expected = " or ".join(gate.evidence)
        return Result(gate, SKIP, f"not landed yet — expected {gate.min_files}× {expected}")
    names = ", ".join(path.relative_to(ROOT).as_posix() for path in found[:3])
    more = f" (+{len(found) - 3})" if len(found) > 3 else ""
    return Result(gate, PASS, f"{names}{more}" if names else gate.ok_detail)


def check_security_scan(gate: Gate) -> Result:
    """Run gitleaks when available, otherwise prove the scans are still wired."""
    missing = [name for name in gate.requires if not (ROOT / name).is_file()]
    if missing:
        return Result(gate, FAIL, f"scan configuration missing: {', '.join(missing)}")
    if shutil.which("gitleaks"):
        return run_commands(gate, (("gitleaks", "detect", "--no-git", "--no-banner", "--redact"),))
    workflow = (ROOT / ".github/workflows/security.yml").read_text(encoding="utf-8")
    absent = [scan for scan in SECURITY_SCANS if scan not in workflow]
    if absent:
        return Result(gate, FAIL, f"security.yml no longer wires: {', '.join(absent)}")
    return Result(gate, PASS, f"security.yml wires {len(SECURITY_SCANS)} scans (gitleaks not installed here)")


def run_gate(gate: Gate) -> Result:
    if gate.id == "security-scan":
        return check_security_scan(gate)
    if gate.steps:
        needed = list(gate.requires) + [
            step[1] for step in gate.steps if len(step) > 1 and step[0] == "python3" and step[1].endswith(".py")
        ]
        missing = [name for name in needed if not (ROOT / name).exists()]
        if missing:
            return Result(gate, SKIP, f"not landed yet — expected {', '.join(missing)}")
        return run_commands(gate)
    return check_evidence(gate)


def run_all(only: list[str] | None = None) -> list[Result]:
    selected = [gate for gate in GATES if not only or gate.id in only]
    unknown = sorted(set(only or ()) - {gate.id for gate in GATES})
    if unknown:
        raise SystemExit(f"prod_readiness: unknown gate(s): {', '.join(unknown)}")
    return [run_gate(gate) for gate in selected]


def tally(results: list[Result]) -> dict[str, int]:
    counts = {PASS: 0, FAIL: 0, SKIP: 0}
    for result in results:
        counts[result.status] += 1
    return counts


def print_table(results: list[Result]) -> None:
    counts = tally(results)
    width = max(len(result.gate.id) for result in results)
    print(f"PRODUCTION READINESS GATE (#{EPIC}) — {len(results)} checks")
    print()
    print(f"{'STATUS':6}  {'CHECK'.ljust(width)}  {'OWNER':6}  DETAIL")
    print(f"{'-' * 6}  {'-' * width}  {'-' * 6}  {'-' * 78}")
    for result in results:
        owner = f"#{result.gate.owner}"
        print(f"{result.status:6}  {result.gate.id.ljust(width)}  {owner:6}  {_clip(result.detail)}")
    print()
    print(f"{len(results)} checks: {counts[PASS]} PASS · {counts[FAIL]} FAIL · {counts[SKIP]} SKIP")
    if counts[FAIL]:
        print("FAIL is a regression: fix it or land the gate — SKIP is never a pass.")


def as_json(results: list[Result]) -> dict:
    counts = tally(results)
    return {
        "epic": EPIC,
        "ok": counts[FAIL] == 0,
        "summary": counts,
        "checks": [
            {
                "id": result.gate.id,
                "title": result.gate.title,
                "owner": result.gate.owner,
                "proves": result.gate.proves,
                "status": result.status,
                "detail": result.detail,
            }
            for result in results
        ],
    }


def _child_status(results: list[Result], gate_ids: tuple[str, ...]) -> str:
    statuses = [result.status for result in results if result.gate.id in gate_ids]
    if not statuses:
        return SKIP
    if FAIL in statuses:
        return FAIL
    return SKIP if SKIP in statuses else PASS


def render_markdown(results: list[Result]) -> str:
    counts = tally(results)
    lines = [
        "# Production Readiness Gate 🟡 IN PROGRESS",
        "",
        f"<!-- Generated by `python3 scripts/prod_readiness.py --markdown` (#{EPIC}). Do not edit by hand. -->",
        "",
        "The production-readiness program "
        f"([#{EPIC}](https://github.com/generalbots/generalbots/issues/{EPIC})) "
        "is only credible if its claims are executable. This page is generated from "
        "`scripts/prod_readiness.py`, the single gate that runs every readiness check "
        "the repository owns and reports one row per check with the issue that owns it.",
        "",
        "| Status | Meaning |",
        "|---|---|",
        "| **PASS** | the check ran and passed |",
        "| **FAIL** | the check ran and failed — a regression; CI fails |",
        "| **SKIP** | the gate does not exist yet — the row names the issue and what must land |",
        "",
        f"Current run: **{counts[PASS]} PASS · {counts[FAIL]} FAIL · {counts[SKIP]} SKIP** "
        f"of {len(results)} checks.",
        "",
        "## Workstreams",
        "",
        "| Issue | Workstream | Gate | Status |",
        "|---|---|---|---|",
    ]
    for issue, title, gate_ids in CHILDREN:
        gates = ", ".join(f"`{gate_id}`" for gate_id in gate_ids)
        status = _child_status(results, gate_ids)
        lines.append(f"| [#{issue}](https://github.com/generalbots/generalbots/issues/{issue}) | {title} | {gates} | **{status}** |")
    lines += [
        "",
        "## Checks",
        "",
        "| Check | Proves | Owner | Status | Detail |",
        "|---|---|---|---|---|",
    ]
    for result in results:
        gate = result.gate
        detail = result.detail.replace("|", "\\|")
        lines.append(
            f"| `{gate.id}` | {gate.title} | [#{gate.owner}](https://github.com/generalbots/generalbots/issues/{gate.owner}) "
            f"| **{result.status}** | {detail} |"
        )
    lines += [
        "",
        "## How to run it",
        "",
        "```bash",
        "python3 scripts/prod_readiness.py                  # table, exit 1 on FAIL",
        "python3 scripts/prod_readiness.py --json           # machine output",
        "python3 scripts/prod_readiness.py --markdown       # rewrite this page",
        "python3 scripts/prod_readiness.py --check-markdown # fail when this page drifts",
        "```",
        "",
        "CI runs it as the `prod-readiness` job in `.github/workflows/ci.yml`: any "
        "**FAIL** fails the build, and so does a page that no longer matches the gate "
        "output. Regenerate the page instead of editing it.",
        "",
        "A **SKIP** row is the to-do list for the program: the owning issue lands the "
        "check (a script, a committed baseline or a regression test), and the row flips "
        "to **PASS** on the next run without touching this page by hand.",
        "",
    ]
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        prog="prod_readiness.py",
        description="Run every production-readiness check and report PASS/FAIL/SKIP.",
    )
    parser.add_argument("--json", action="store_true", help="print machine-readable results")
    parser.add_argument("--markdown", nargs="?", const=DOC, metavar="PATH", help="write the scoreboard page")
    parser.add_argument("--check-markdown", nargs="?", const=DOC, metavar="PATH", help="fail when the page drifted")
    parser.add_argument("--only", action="append", metavar="ID", help="run one gate (repeatable)")
    args = parser.parse_args(argv)

    results = run_all(args.only)
    counts = tally(results)
    if args.json:
        print(json.dumps(as_json(results), indent=2))
    else:
        print_table(results)

    code = 1 if counts[FAIL] else 0
    if args.markdown:
        path = ROOT / args.markdown
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(render_markdown(results), encoding="utf-8")
        print(f"wrote {args.markdown}")
    if args.check_markdown:
        path = ROOT / args.check_markdown
        if not path.is_file():
            print(f"prod_readiness: {args.check_markdown} is missing — run --markdown", file=sys.stderr)
            return 3
        if path.read_text(encoding="utf-8") != render_markdown(results):
            print(
                f"prod_readiness: {args.check_markdown} drifted from the gate output — "
                "run `python3 scripts/prod_readiness.py --markdown`",
                file=sys.stderr,
            )
            return 3
    return code


if __name__ == "__main__":
    raise SystemExit(main())
