#!/usr/bin/env python3
"""Flake detection and the retry/quarantine policy (#2119).

A flaky test is a test that reports both `ok` and `FAILED` for the same code.
It is worse than a failing test: it teaches the team to re-run CI, which is how
real failures get merged. This script makes the policy executable:

* **detect** — read N repeated runs of the same workflow and list every test
  that flipped. Two inputs are supported, both offline:
    - `--logs DIR`      one `<run-id>.log` per run (`gh run view <id> --log`);
    - `--runs FILE`     the `gh api .../actions/runs` JSON (job/step level).
* **policy** — `issues/flakes/quarantine.toml` records every quarantined test.
  An entry needs `test`, `issue`, `owner`, `since` and `reason`; the SLA is 14
  days (fix it or delete it, never leave it ignored forever).
* **gate** — the run fails when a test flaked at least `max_flakes` times in the
  window without a quarantine entry, when a quarantine entry is incomplete, or
  when an entry is past its SLA. A quarantined test that no longer flakes is
  reported so the entry can be removed (never silently deleted).

Retry policy (enforced by the numbers below, documented for reviewers):
a failing test gets **one** retry; two consecutive failures are a real failure.
A test that needs that retry more than `max_flakes` times in `window` runs is
quarantined with `#[ignore = "flaky: <issue>"]`, an owner and a date.

Usage:
    gh run list --workflow ci.yml --limit 20 --json databaseId \\
        --jq '.[].databaseId' | while read id; do
        gh run view "$id" --log > "/tmp/flakes/$id.log"; done
    python3 scripts/flake_report.py --logs /tmp/flakes
    python3 scripts/flake_report.py --quarantine
    python3 scripts/flake_report.py --self-test

Exit codes: 0 = no unquarantined flake over the threshold and a valid registry,
1 = policy violation, 2 = usage/input error.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from collections import defaultdict
from datetime import date, datetime
from pathlib import Path

try:
    import tomllib
except ModuleNotFoundError:  # Python < 3.11
    try:
        import tomli as tomllib  # type: ignore
    except ModuleNotFoundError:
        print("flake_report: Python 3.11+ (tomllib) is required", file=sys.stderr)
        raise SystemExit(2)

ROOT = Path(__file__).resolve().parents[1]
QUARANTINE = ROOT / "issues" / "flakes" / "quarantine.toml"
DEFAULT_WINDOW = 20
DEFAULT_MAX_FLAKES = 3
DEFAULT_SLA_DAYS = 14

# `cargo test` prints one line per test; libtest also prints them for ignored
# tests, which are not results at all.
TEST_LINE = re.compile(r"^test ([\w:<>.\- ]+) \.\.\. (ok|FAILED|ignored)\s*$", re.MULTILINE)


class InputError(Exception):
    """The runs/quarantine input cannot be read."""


def results_from_logs(directory: Path, window: int) -> dict[str, list[tuple[str, bool]]]:
    """`{test: [(run id, passed)]}` from `gh run view --log` dumps."""
    if not directory.is_dir():
        raise InputError(f"{directory} is not a directory")
    logs = sorted(p for p in directory.glob("*.log") if p.is_file())
    if not logs:
        raise InputError(f"{directory} has no *.log files")
    logs = logs[-window:]
    results: dict[str, list[tuple[str, bool]]] = defaultdict(list)
    for log in logs:
        for name, verdict in TEST_LINE.findall(log.read_text(encoding="utf-8", errors="replace")):
            if verdict == "ignored":
                continue
            results[name.strip()].append((log.stem, verdict == "ok"))
    if not results:
        raise InputError(f"{directory}: no `test ... ok|FAILED` lines found")
    return dict(results)


def results_from_runs(path: Path, window: int) -> dict[str, list[tuple[str, bool]]]:
    """`{job > step: [(run id, passed)]}` from the Actions runs API."""
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise InputError(f"{path}: {error}") from error
    runs = payload.get("workflow_runs", payload) if isinstance(payload, dict) else payload
    if not isinstance(runs, list) or not runs:
        raise InputError(f"{path}: expected a non-empty list of runs")
    results: dict[str, list[tuple[str, bool]]] = defaultdict(list)
    for run in runs[-window:]:
        run_id = str(run.get("id", run.get("run_id", "?")))
        for job in run.get("jobs", []):
            job_name = job.get("name", "?")
            for step in job.get("steps", []):
                conclusion = step.get("conclusion")
                if conclusion in (None, "skipped", "cancelled"):
                    continue
                key = f"{job_name} > {step.get('name', '?')}"
                results[key].append((run_id, conclusion == "success"))
    if not results:
        raise InputError(f"{path}: no job/step conclusions found")
    return dict(results)


def flaky(results: dict[str, list[tuple[str, bool]]]) -> dict[str, dict]:
    """Tests that both passed and failed, with their failure count."""
    report = {}
    for name, outcomes in results.items():
        passed = sum(1 for _, ok in outcomes if ok)
        failed = len(outcomes) - passed
        if passed and failed:
            report[name] = {
                "runs": len(outcomes),
                "passed": passed,
                "failed": failed,
                "last": outcomes[-1][0],
            }
    return report


def load_quarantine(path: Path) -> dict:
    if not path.is_file():
        raise InputError(f"{path} is missing")
    try:
        return tomllib.loads(path.read_text(encoding="utf-8"))
    except (OSError, tomllib.TOMLDecodeError) as error:
        raise InputError(f"{path}: {error}") from error


def check_registry(registry: dict, today: date) -> list[str]:
    """Every quarantine entry must be complete and inside its SLA."""
    meta = registry.get("meta", {})
    sla_days = int(meta.get("sla_days", DEFAULT_SLA_DAYS))
    errors = []
    entries = registry.get("flake", [])
    seen = set()
    for index, entry in enumerate(entries, start=1):
        where = f"[[flake]] #{index}"
        test = str(entry.get("test", "")).strip()
        if not test:
            errors.append(f"{where} has no `test`")
            continue
        if test in seen:
            errors.append(f"{where} duplicates `{test}`")
        seen.add(test)
        if not isinstance(entry.get("issue"), int):
            errors.append(f"{test}: quarantine needs an `issue` number")
        if not str(entry.get("owner", "")).strip():
            errors.append(f"{test}: quarantine needs an `owner`")
        reason = str(entry.get("reason", "")).strip()
        if len(reason) < 20:
            errors.append(f"{test}: quarantine needs a real `reason` (>= 20 chars)")
        since = str(entry.get("since", "")).strip()
        try:
            started = datetime.strptime(since, "%Y-%m-%d").date()
        except ValueError:
            errors.append(f"{test}: `since` must be YYYY-MM-DD (got {since!r})")
            continue
        age = (today - started).days
        if age > sla_days:
            errors.append(
                f"{test}: quarantined {age} days ago (SLA {sla_days}); fix it or delete it"
            )
    return errors


def check_flakes(report: dict[str, dict], registry: dict) -> tuple[list[str], list[str]]:
    """Unquarantined flakes over the threshold, and stale quarantine entries."""
    meta = registry.get("meta", {})
    max_flakes = int(meta.get("max_flakes", DEFAULT_MAX_FLAKES))
    quarantined = {
        str(entry.get("test", "")).strip() for entry in registry.get("flake", [])
    }
    errors = []
    for name, stats in sorted(report.items()):
        if stats["failed"] >= max_flakes and name not in quarantined:
            errors.append(
                f"{name}: {stats['failed']} flakes in {stats['runs']} runs and no "
                "quarantine entry (add one to issues/flakes/quarantine.toml)"
            )
    stale = sorted(quarantined - set(report))
    return errors, stale


def render(report: dict[str, dict]) -> None:
    if not report:
        print("flake report: no test flipped result")
        return
    print(f"{'test':<70} {'runs':>5} {'pass':>5} {'fail':>5} last run")
    for name, stats in sorted(report.items()):
        print(
            f"{name:<70} {stats['runs']:>5} {stats['passed']:>5} "
            f"{stats['failed']:>5} {stats['last']}"
        )


def run(args: argparse.Namespace) -> int:
    registry = load_quarantine(Path(args.quarantine))
    errors = check_registry(registry, date.today())
    if errors:
        print("flake policy: FAILED", file=sys.stderr)
        for error in errors:
            print(f"  - {error}", file=sys.stderr)
        return 1
    if args.quarantine_only:
        entries = len(registry.get("flake", []))
        print(f"flake policy: {entries} quarantined test(s), all inside the SLA")
        return 0

    if args.logs:
        results = results_from_logs(Path(args.logs), args.window)
    else:
        results = results_from_runs(Path(args.runs), args.window)
    report = flaky(results)
    render(report)
    errors, stale = check_flakes(report, registry)
    for name in stale:
        print(f"warning: `{name}` is quarantined but did not flake in this window")
    if errors:
        print("flake policy: FAILED", file=sys.stderr)
        for error in errors:
            print(f"  - {error}", file=sys.stderr)
        return 1
    print(f"flake report: {len(report)} flaky test(s), policy satisfied")
    return 0


def self_test() -> int:
    """Proves detection, the threshold and the SLA rules fire."""
    failures = 0

    def expect(condition: bool, label: str) -> None:
        nonlocal failures
        if not condition:
            failures += 1
            print(f"self-test FAILED: {label}")

    results = {
        "crate::tests::flaky": [("1", True), ("2", False), ("3", True), ("4", False)],
        "crate::tests::stable": [("1", True), ("2", True), ("3", True)],
        "crate::tests::broken": [("1", False), ("2", False), ("3", False)],
    }
    report = flaky(results)
    expect(set(report) == {"crate::tests::flaky"}, "only the flipping test is reported")
    expect(report["crate::tests::flaky"]["failed"] == 2, "failure count is kept")

    registry = {
        "meta": {"max_flakes": 2, "sla_days": 14},
        "flake": [
            {
                "test": "crate::tests::flaky",
                "issue": 2119,
                "owner": "quality",
                "since": "2026-01-01",
                "reason": "order dependent when the cache is warm",
            }
        ],
    }
    errors, stale = check_flakes(report, registry)
    expect(errors == [], "a quarantined flake passes")
    expect(stale == [], "no stale entry in the window")

    errors, _ = check_flakes(report, {"meta": {"max_flakes": 2}, "flake": []})
    expect(len(errors) == 1, "an unquarantined flake over the threshold fails")

    _, stale = check_flakes({"crate::tests::stable": {"runs": 3, "passed": 3, "failed": 0, "last": "3"}}, registry)
    expect(stale == ["crate::tests::flaky"], "a quarantine entry that stopped flaking is reported")

    today = date(2026, 6, 1)
    expect(check_registry(registry, date(2026, 1, 5)) == [], "a fresh entry is accepted")
    expired = dict(registry, flake=[dict(registry["flake"][0], since="2026-01-01")])
    expect(check_registry(expired, today) != [], "an entry past the SLA fails")
    missing_issue = dict(registry, flake=[{"test": "t", "owner": "o", "since": "2026-05-30",
                                          "reason": "a long enough reason string"}])
    expect(check_registry(missing_issue, today) != [], "an entry without an issue fails")
    missing_owner = dict(registry, flake=[{"test": "t", "issue": 1, "since": "2026-05-30",
                                           "reason": "a long enough reason string"}])
    expect(check_registry(missing_owner, today) != [], "an entry without an owner fails")
    vague = dict(registry, flake=[{"test": "t", "issue": 1, "owner": "o", "since": "2026-05-30",
                                   "reason": "flaky"}])
    expect(check_registry(vague, today) != [], "an entry without a real reason fails")

    if failures:
        print(f"self-test: {failures} failure(s)")
        return 1
    print("self-test: ok")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description="Flake detection and quarantine policy (#2119)")
    parser.add_argument("--logs", help="directory of `<run-id>.log` files")
    parser.add_argument("--runs", help="Actions runs JSON (job/step level)")
    parser.add_argument(
        "--quarantine",
        default=str(QUARANTINE),
        help="quarantine registry (default: issues/flakes/quarantine.toml)",
    )
    parser.add_argument("--window", type=int, default=DEFAULT_WINDOW, help="runs to consider")
    parser.add_argument(
        "--quarantine-only",
        action="store_true",
        help="validate the registry without CI history",
    )
    parser.add_argument("--self-test", action="store_true", help="prove the policy logic")
    args = parser.parse_args()

    if args.self_test:
        return self_test()
    try:
        if not args.quarantine_only and not args.logs and not args.runs:
            parser.error("--logs or --runs is required (or use --quarantine-only)")
        return run(args)
    except InputError as error:
        print(f"error: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main())
