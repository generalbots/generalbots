#!/usr/bin/env python3
"""Guard that CI audits every Python runtime requirement, not just `==` pins (#2675).

The `python` job in .github/workflows/security.yml used to filter
botmodels/requirements.txt with ``grep '=='`` before handing it to pip-audit
with ``--disable-pip --no-deps``. pip-audit refuses non-pinned lines under
those flags ("requirement X is not pinned to an exact version"), so every
``>=``, ``~=``, ``<`` and unpinned dependency was silently skipped.

This guard asserts the job feeds the whole requirements file to pip-audit
without those flags, so pip resolves each specifier to the version it would
install and audits it. It also records that requirements.txt really contains
non-``==`` specifiers, so the rule cannot become vacuous.

Usage:
    python3 scripts/check_pip_audit_coverage.py             # check the repo
    python3 scripts/check_pip_audit_coverage.py --self-test  # synthetic cases

Exit codes: 0 clean, 1 a problem was found.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
WORKFLOW = ROOT / ".github" / "workflows" / "security.yml"
REQUIREMENTS = ROOT / "botmodels" / "requirements.txt"

REQ_FILE = "botmodels/requirements.txt"
BAD_FLAGS = ("--disable-pip", "--no-deps")


def requirement_lines(text: str) -> list[str]:
    """Effective runtime requirement specifiers (no blanks, comments or extras)."""
    lines: list[str] = []
    for raw in text.splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        line = line.split("  #", 1)[0].strip()  # trailing `pkg==1  # reason`
        if line:
            lines.append(line)
    return lines


def run_blocks(text: str) -> list[str]:
    """Body of every `run: |` block in a workflow."""
    lines = text.splitlines()
    blocks: list[str] = []
    i = 0
    while i < len(lines):
        match = re.match(r"^(\s*)run:\s*\|", lines[i])
        if not match:
            i += 1
            continue
        indent = len(match.group(1))
        i += 1
        body: list[str] = []
        while i < len(lines):
            line = lines[i]
            if line.strip() and (len(line) - len(line.lstrip())) <= indent:
                break
            body.append(line)
            i += 1
        blocks.append("\n".join(body))
    return blocks


def job_block(text: str, job_id: str) -> str | None:
    """The `  <job_id>:` block, up to the next 2-space key under `jobs:`."""
    lines = text.splitlines()
    start = None
    for i, line in enumerate(lines):
        if re.match(rf"^  {re.escape(job_id)}:\s*$", line):
            start = i
            break
    if start is None:
        return None
    end = len(lines)
    for i in range(start + 1, len(lines)):
        if re.match(r"^  \S", lines[i]):
            end = i
            break
    return "\n".join(lines[start:end])


def pip_audit_step(text: str) -> str | None:
    for block in run_blocks(text):
        if "pip-audit" in block:
            return block
    return None


def check(workflow_text: str, requirements_text: str) -> list[str]:
    problems: list[str] = []

    python_job = job_block(workflow_text, "python")
    if python_job is None:
        return ["no `python` job found in .github/workflows/security.yml"]

    step = pip_audit_step(python_job)
    if step is None:
        return ["no pip-audit step found in the `python` job"]

    # Only inspect executable lines: the step's comment documents the forbidden
    # flags/patterns and must not trip the checks itself.
    code = "\n".join(line for line in step.splitlines() if not line.lstrip().startswith("#"))

    # The whole requirements file must be the audit input.
    if not re.search(r"(?:--requirement|-r)\s+" + re.escape(REQ_FILE) + r"\b", code):
        problems.append(
            f"pip-audit step does not pass the full {REQ_FILE} via --requirement/-r"
        )

    # These flags make pip-audit reject non-pinned lines (or stop it resolving
    # specifiers) -- exactly how >= / ~= / unpinned deps got skipped.
    for flag in BAD_FLAGS:
        if re.search(r"(?<![\w-])" + re.escape(flag) + r"(?![\w-])", code):
            problems.append(f"pip-audit step still uses {flag}")

    if "requirements-exact" in code or re.search(r"grep[^\n]*==", code):
        problems.append("pip-audit step still filters the requirements file (grep '==')")

    if re.search(r"\|\|\s*true", code):
        problems.append("pip-audit step swallows failures with `|| true`")
    if re.search(r"continue-on-error\s*:\s*true", python_job):
        problems.append("the `python` job sets continue-on-error: true")

    reqs = requirement_lines(requirements_text)
    non_pinned = [r for r in reqs if "==" not in r]
    if not reqs:
        problems.append(f"{REQ_FILE} has no runtime dependencies")
    if not non_pinned:
        problems.append(
            f"{REQ_FILE} has no non-'==' specifier; add one or retire this guard"
        )

    return problems


SELF_REQS = (
    "# comment\nfastapi==0.115.0\nemail-validator>=2.2.0\n\n"
    "av>=16,<19\nrequests\n"
)

GOOD_STEP = """      - name: Audit all runtime requirements
        run: |
          /tmp/audit-venv/bin/pip-audit \\
            --requirement botmodels/requirements.txt \\
            --progress-spinner off \\
            --ignore-vuln GHSA-4j2p-28q2-5m79
"""

BAD_STEP = """      - name: Audit pinned requirements
        run: |
          grep -E '^[A-Za-z0-9_.-]+==' botmodels/requirements.txt \\
            > /tmp/requirements-exact.txt
          /tmp/audit-venv/bin/pip-audit \\
            --requirement /tmp/requirements-exact.txt \\
            --no-deps \\
            --disable-pip \\
            --progress-spinner off
"""


def wrap(step: str, job: str = "python") -> str:
    return (
        "jobs:\n"
        f"  {job}:\n"
        "    runs-on: ubuntu-latest\n"
        "    steps:\n"
        + step
        + "  next-job:\n"
        "    runs-on: ubuntu-latest\n"
    )


def self_test() -> None:
    assert check(wrap(GOOD_STEP), SELF_REQS) == [], "good step should pass"
    bad = check(wrap(BAD_STEP), SELF_REQS)
    for expected in (
        "does not pass the full",
        "--disable-pip",
        "--no-deps",
        "still filters",
    ):
        assert any(expected in p for p in bad), f"bad step not flagged for {expected!r}"
    assert check("jobs: {}\n", SELF_REQS), "missing job should be flagged"
    assert check(wrap(GOOD_STEP), "fastapi==0.115.0\n"), "vacuous guard should be flagged"
    assert check(
        wrap(GOOD_STEP.replace("            --progress-spinner off",
                               "            --progress-spinner off || true")),
        SELF_REQS,
    ), "`|| true` should be flagged"
    assert check(
        wrap(GOOD_STEP).replace("    runs-on:", "    continue-on-error: true\n    runs-on:"),
        SELF_REQS,
    ), "continue-on-error should be flagged"
    assert requirement_lines(SELF_REQS) == [
        "fastapi==0.115.0",
        "email-validator>=2.2.0",
        "av>=16,<19",
        "requests",
    ], "requirement parser should drop comments/blanks"
    print("self-test: OK")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()

    if args.self_test:
        self_test()
        return 0

    problems = check(WORKFLOW.read_text(encoding="utf-8"), REQUIREMENTS.read_text(encoding="utf-8"))
    if problems:
        for problem in problems:
            print(f"FAIL: {problem}")
        return 1
    print(f"OK: pip-audit audits the full {REQ_FILE} (all specifiers, all deps)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
