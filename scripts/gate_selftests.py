#!/usr/bin/env python3
"""Fixture-based self-tests for the policy gates (#2121).

A gate that cannot prove it fails closed is a gate nobody can trust: a typo in
a regex or an inverted condition turns it into a script that prints "ok" on
every input. Every gate therefore carries `--self-test`, which plants a
violation in a temporary copy of the smallest input the gate reads, runs the
gate's real CLI against it and asserts exit 1 — then asserts exit 0 on the same
input without the violation.

Design notes
------------
* The fixture lives in `TemporaryDirectory`; nothing is written to the repo.
* The gate runs as a subprocess, so the test exercises argument parsing and
  the exit code, not an internal helper the CLI might not call.
* Gates whose input paths are module constants accept a root override in the
  environment (`GB_ADEQUACY_ROOT`, `GB_PRODUCT_ROOT`), which is how the fixture
  tree is injected. That is also useful for reviewing a branch in a worktree.
* Everything is offline and deterministic: no network, no clock, no cargo.

Usage:
    python3 scripts/gate_selftests.py --gate adequacy
    python3 scripts/gate_selftests.py --list
"""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPTS = ROOT / "scripts"
TIMEOUT = 300
SHA = "0123456789abcdef0123456789abcdef01234567"
CLEAN, VIOLATION = 0, 1


def _gate(script: str, args: list[str], cwd: Path = ROOT, env: dict[str, str] | None = None) -> int:
    """Exit code of one gate invocation; output is captured, not printed."""
    cmd = [sys.executable, str(SCRIPTS / script), *args]
    merged = {**os.environ, **(env or {})}
    try:
        proc = subprocess.run(
            cmd, cwd=cwd, env=merged, capture_output=True, text=True, timeout=TIMEOUT
        )
    except (OSError, subprocess.TimeoutExpired) as error:
        print(f"self-test: cannot run {script}: {error}", file=sys.stderr)
        return 99
    return proc.returncode


def _sh(script: str, args: list[str], cwd: Path) -> int:
    """Exit code of a shell gate invocation."""
    try:
        proc = subprocess.run(
            ["bash", str(SCRIPTS / script), *args],
            cwd=cwd, capture_output=True, text=True, timeout=TIMEOUT,
        )
    except (OSError, subprocess.TimeoutExpired) as error:
        print(f"self-test: cannot run {script}: {error}", file=sys.stderr)
        return 99
    return proc.returncode


class Case:
    """Collects the expectations of one gate so every failure is reported."""

    def __init__(self, name: str) -> None:
        self.name = name
        self.failures: list[str] = []

    def expect(self, label: str, expected: int, actual: int) -> None:
        if actual != expected:
            self.failures.append(f"{label}: expected exit {expected}, got {actual}")

    def finish(self) -> int:
        if self.failures:
            for failure in self.failures:
                print(f"self-test {self.name}: {failure}", file=sys.stderr)
            print(f"{self.name} self-test: FAILED ({len(self.failures)})", file=sys.stderr)
            return 1
        print(f"{self.name} self-test: ok (fails closed on a planted violation)")
        return 0


def adequacy(tmp: Path) -> Case:
    """Ratchet gate: a new `unwrap` above the recorded ceiling must fail."""
    case = Case("adequacy")
    root = tmp / "tree"
    (root / "src").mkdir(parents=True)
    bad = root / "src" / "bad.rs"
    bad.write_text("pub fn f(v: Option<u8>) -> u8 {\n    v.unwrap()\n}\n", encoding="utf-8")
    env = {"GB_ADEQUACY_ROOT": str(root)}
    baseline = str(root / "issues" / "adequacy" / "baseline.json")
    case.expect("record baseline", CLEAN, _gate("adequacy_guard.py", ["--update", "--baseline", baseline], env=env))
    case.expect("at ceiling", CLEAN, _gate("adequacy_guard.py", ["--baseline", baseline], env=env))
    bad.write_text(
        "pub fn f(v: Option<u8>) -> u8 {\n    v.unwrap()\n}\n\n"
        "pub fn g(v: Option<u8>) -> u8 {\n    v.expect(\"x\")\n}\n",
        encoding="utf-8",
    )
    case.expect("growth", VIOLATION, _gate("adequacy_guard.py", ["--baseline", baseline], env=env))
    return case


def i18n_lint(tmp: Path) -> Case:
    """Catalog gate: a violation absent from the baseline must fail."""
    case = Case("i18n-lint")
    real = ROOT / ".github" / "i18n-baseline.json"
    recorded = json.loads(real.read_text(encoding="utf-8")).get("fingerprints", [])
    if not recorded:
        case.failures.append("the committed baseline is empty; the gate cannot be proven")
        return case
    empty = tmp / "empty-baseline.json"
    empty.write_text(json.dumps({"fingerprints": []}), encoding="utf-8")
    case.expect("new violation", VIOLATION, _gate("i18n_lint.py", ["--check", "--baseline", str(empty)]))
    case.expect("baseline clean", CLEAN, _gate("i18n_lint.py", ["--check"]))
    return case


def workflow_pinning(tmp: Path) -> Case:
    """Actions gate: a mutable tag must fail, a commit SHA must pass."""
    case = Case("workflow-pinning")
    workflows = tmp / ".github" / "workflows"
    workflows.mkdir(parents=True)
    wf = workflows / "ci.yml"
    wf.write_text(
        "jobs:\n  build:\n    steps:\n"
        f"      - uses: actions/checkout@{SHA} # v4.2.2\n",
        encoding="utf-8",
    )
    case.expect("pinned", CLEAN, _gate("check_workflow_pinning.py", [], cwd=tmp))
    wf.write_text("jobs:\n  build:\n    steps:\n      - uses: actions/checkout@v4\n", encoding="utf-8")
    case.expect("mutable tag", VIOLATION, _gate("check_workflow_pinning.py", [], cwd=tmp))
    return case


def workflow_downloads(tmp: Path) -> Case:
    """Download gate: a `curl -o` without `sha256sum -c` must fail."""
    case = Case("workflow-downloads")
    workflows = tmp / "wf"
    workflows.mkdir()
    allowlist = tmp / "allowlist.txt"
    allowlist.write_text("# empty\n", encoding="utf-8")
    wf = workflows / "release.yml"
    verified = (
        "jobs:\n  build:\n    steps:\n      - name: fetch\n        run: |\n"
        "          curl -fsSL https://example.invalid/tool.tgz -o tool.tgz\n"
        f'          echo "{SHA}  tool.tgz" | sha256sum -c -\n'
    )
    wf.write_text(verified, encoding="utf-8")
    args = ["--dir", str(workflows), "--allowlist", str(allowlist)]
    case.expect("verified", CLEAN, _gate("check_workflow_downloads.py", args))
    wf.write_text(verified.replace(f'          echo "{SHA}  tool.tgz" | sha256sum -c -\n', ""), encoding="utf-8")
    case.expect("unverified", VIOLATION, _gate("check_workflow_downloads.py", args))
    return case


def osv_allowlist(tmp: Path) -> Case:
    """Advisory gate: an ID outside the dated allowlist must fail."""
    case = Case("osv-allowlist")
    report = tmp / "osv.json"
    report.write_text(
        json.dumps({"results": [{"packages": [{
            "package": {"name": "serde", "version": "1.0.0"},
            "groups": [{"ids": ["RUSTSEC-2024-0001"]}],
        }]}]}),
        encoding="utf-8",
    )
    covered = tmp / "covered.txt"
    covered.write_text("RUSTSEC-2024-0001  reachable path reviewed 2026-11-10\n", encoding="utf-8")
    other = tmp / "other.txt"
    other.write_text("RUSTSEC-2020-9999  unrelated advisory\n", encoding="utf-8")
    case.expect(
        "covered", CLEAN,
        _gate("check_osv_allowlist.py", ["--report", str(report), "--allowlist", str(covered)]),
    )
    case.expect(
        "unknown", VIOLATION,
        _gate("check_osv_allowlist.py", ["--report", str(report), "--allowlist", str(other)]),
    )
    return case


def component_checksums(tmp: Path) -> Case:
    """Provenance gate: an unhashed component outside the allowlist must fail."""
    case = Case("component-checksums")
    toml = tmp / "3rdparty.toml"
    toml.write_text('[components.minio]\nversion = "1"\nurl = "https://example.invalid/m"\nsha256 = ""\n', encoding="utf-8")
    empty = tmp / "allow.txt"
    empty.write_text("# no acknowledged gaps\n", encoding="utf-8")
    acknowledged = tmp / "allow-ok.txt"
    acknowledged.write_text("minio  upstream publishes no digest yet\n", encoding="utf-8")
    case.expect(
        "unacknowledged", VIOLATION,
        _gate("check_component_checksums.py", ["--toml", str(toml), "--allowlist", str(empty)]),
    )
    case.expect(
        "acknowledged", CLEAN,
        _gate("check_component_checksums.py", ["--toml", str(toml), "--allowlist", str(acknowledged)]),
    )
    return case


def _product_fixture(tmp: Path) -> Path:
    """Copy the profile inputs (matrix, profiles, registry, Cargo.toml)."""
    root = tmp / "product"
    (root / "botserver").mkdir(parents=True)
    shutil.copy2(ROOT / "botserver" / "Cargo.toml", root / "botserver" / "Cargo.toml")
    for path in (ROOT / "botserver").glob(".product*"):
        shutil.copy2(path, root / "botserver" / path.name)
    shutil.copytree(ROOT / "botserver" / "src" / "apps" / "registry", root / "botserver" / "src" / "apps" / "registry")
    return root


def product_profile(tmp: Path) -> Case:
    """Profile gate: a registry id that leaves the inventory must fail."""
    case = Case("product-profile")
    root = _product_fixture(tmp)
    env = {"GB_PRODUCT_ROOT": str(root)}
    case.expect("consistent", CLEAN, _gate("product_profile_check.py", [], env=env))
    registry = root / "botserver" / "src" / "apps" / "registry" / "entries.rs"
    lines = registry.read_text(encoding="utf-8").splitlines(keepends=True)
    dropped = next((i for i, line in enumerate(lines) if 'id: "' in line), None)
    if dropped is None:
        case.failures.append("fixture has no registry id to drop")
        return case
    registry.write_text("".join(lines[:dropped] + lines[dropped + 1 :]), encoding="utf-8")
    case.expect("id removed", VIOLATION, _gate("product_profile_check.py", [], env=env))
    return case


def app_budget(tmp: Path) -> Case:
    """Ceiling gate: one line above the recorded ceiling must fail."""
    case = Case("app-complexity-budget")
    budget = tmp / "budget.toml"
    case.expect("re-baseline", CLEAN, _gate("app_complexity_budget.py", ["--budget", str(budget), "--update"]))
    case.expect("within budget", CLEAN, _gate("app_complexity_budget.py", ["--budget", str(budget)]))
    text = budget.read_text(encoding="utf-8")
    marker = "loc = "
    at = text.rfind(marker)
    if at < 0:
        case.failures.append("fixture budget has no loc ceiling")
        return case
    start = at + len(marker)
    end = start
    while end < len(text) and text[end].isdigit():
        end += 1
    lowered = text[:start] + str(int(text[start:end]) - 1) + text[end:]
    budget.write_text(lowered, encoding="utf-8")
    case.expect("over ceiling", VIOLATION, _gate("app_complexity_budget.py", ["--budget", str(budget)]))
    return case


def gb7_desktop(tmp: Path) -> Case:
    """Record gate: dropping a contracted feature must fail."""
    case = Case("gb7-desktop-features")
    record = json.loads((ROOT / "desktop" / "gb7" / "features.json").read_text(encoding="utf-8"))
    copy = tmp / "features.json"
    copy.write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")
    case.expect("consistent", CLEAN, _gate("gb7_desktop_features_check.py", ["--record", str(copy)]))
    trimmed = dict(record)
    trimmed["features"] = record["features"][1:]
    copy.write_text(json.dumps(trimmed, indent=2) + "\n", encoding="utf-8")
    case.expect("feature dropped", VIOLATION, _gate("gb7_desktop_features_check.py", ["--record", str(copy)]))
    return case


def _commit(repo: Path, message: str) -> None:
    subprocess.run(["git", "add", "-A"], cwd=repo, check=False, capture_output=True)
    subprocess.run(
        ["git", "-c", "user.email=gate@example.invalid", "-c", "user.name=gate", "commit", "-qm", message],
        cwd=repo, check=False, capture_output=True,
    )


def rust_syntax(tmp: Path) -> Case:
    """Syntax gate: an unparsable changed file must fail."""
    case = Case("rust-syntax")
    if shutil.which("rustfmt") is None:
        case.failures.append("rustfmt is not installed; the gate cannot be proven")
        return case
    repo = tmp / "repo"
    repo.mkdir()
    subprocess.run(["git", "init", "-q"], cwd=repo, check=False, capture_output=True)
    source = repo / "lib.rs"
    source.write_text("pub fn ok() -> u8 {\n    1\n}\n", encoding="utf-8")
    _commit(repo, "clean")
    source.write_text("pub fn broken( -> u8 {\n", encoding="utf-8")
    _commit(repo, "broken")
    case.expect("unparsable", VIOLATION, _sh("verify-rust-syntax.sh", ["HEAD~1"], repo))
    source.write_text("pub fn ok() -> u8 {\n    2\n}\n", encoding="utf-8")
    _commit(repo, "fixed")
    case.expect("parsable", CLEAN, _sh("verify-rust-syntax.sh", ["HEAD~1"], repo))
    return case


SELFTESTS = {
    "adequacy": adequacy,
    "app-budget": app_budget,
    "component-checksums": component_checksums,
    "gb7-desktop-features": gb7_desktop,
    "i18n-lint": i18n_lint,
    "osv-allowlist": osv_allowlist,
    "product-profile": product_profile,
    "rust-syntax": rust_syntax,
    "workflow-downloads": workflow_downloads,
    "workflow-pinning": workflow_pinning,
}


def run_self_test(name: str) -> int:
    """Run one gate's fixture self-test and return a process exit code."""
    test = SELFTESTS.get(name)
    if test is None:
        print(f"self-test: unknown gate `{name}` (known: {', '.join(sorted(SELFTESTS))})", file=sys.stderr)
        return 2
    with tempfile.TemporaryDirectory(prefix=f"gate-selftest-{name}-") as directory:
        return test(Path(directory)).finish()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--gate", help="gate id to self-test")
    parser.add_argument("--all", action="store_true", help="run every gate self-test")
    parser.add_argument("--list", action="store_true", help="print the gate ids")
    args = parser.parse_args()

    if args.list:
        print("\n".join(sorted(SELFTESTS)))
        return 0
    if args.all:
        failures = [name for name in sorted(SELFTESTS) if run_self_test(name) != 0]
        if failures:
            print(f"gate self-tests: {len(failures)} failed: {', '.join(failures)}", file=sys.stderr)
            return 1
        print(f"gate self-tests: {len(SELFTESTS)} gates fail closed")
        return 0
    if args.gate:
        return run_self_test(args.gate)
    parser.print_usage(sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main())
