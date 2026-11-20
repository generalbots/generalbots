#!/usr/bin/env python3
"""Report the *real* state of GitHub Actions runs without miscounting jobs.

The failure this prevents: an ad-hoc `curl .../jobs | jq 'length'` printed
`56` and it was reported as "56 queued/running jobs". The API returns the
**total number of jobs in the run** (including completed, cancelled and
skipped ones); only a job whose `status` is `queued`/`in_progress`/`waiting`
is actually still running. A second trap is that `curl` without `-L` follows
the repository-move redirect only as a 301 JSON body, so `.jobs` is missing
and `jq` silently prints `n/a`.

This tool:
  * uses `gh api`, which authenticates, follows redirects and paginates;
  * validates the payload shape (a redirect/error body is an input error,
    never an empty job list);
  * merges every page and checks the merged count against `total_count`;
  * prints total/active/success/failed/cancelled/skipped as *separate*
    numbers, with job names for the ones that matter;
  * exits 1 when a job failed (or the run failed), 2 when a required run is
    still active or the input is unusable.

Usage:
    python3 scripts/ci_run_status.py 44720664790 44720664798
    python3 scripts/ci_run_status.py --require-complete 44719260878
    python3 scripts/ci_run_status.py --json 44719260878
    python3 scripts/ci_run_status.py --self-test

Exit codes: 0 = green (or explicitly allowed cancellation), 1 = failed,
2 = still running with --require-complete / input error.
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from typing import Any

ACTIVE_STATUSES = frozenset({"queued", "in_progress", "waiting", "pending", "requested"})
FAILURE_CONCLUSIONS = frozenset({"failure", "timed_out", "action_required", "startup_failure"})


class InputError(Exception):
    """The Actions API response cannot be trusted."""


def gh_api(endpoint: str, *, paginate: bool = False) -> Any:
    """Call `gh api` (auth + redirects) and return parsed JSON."""
    command = ["gh", "api", endpoint]
    if paginate:
        command += ["--paginate", "--slurp"]
    try:
        process = subprocess.run(command, capture_output=True, text=True, check=False)
    except OSError as error:
        raise InputError(f"cannot run gh: {error}") from error
    if process.returncode != 0:
        detail = process.stderr.strip().splitlines()
        raise InputError(f"gh api {endpoint}: {detail[-1] if detail else process.returncode}")
    try:
        return json.loads(process.stdout)
    except json.JSONDecodeError as error:
        raise InputError(f"gh api {endpoint}: invalid JSON ({error})") from error


def validate_pages(pages: Any, run_id: str) -> tuple[int, list[dict[str, Any]]]:
    """Reject error/redirect bodies and a truncated job list."""
    if not isinstance(pages, list) or not pages:
        raise InputError(f"run {run_id}: expected a non-empty list of job pages")
    total: int | None = None
    jobs: list[dict[str, Any]] = []
    for page in pages:
        if not isinstance(page, dict) or not isinstance(page.get("jobs"), list):
            keys = sorted(page) if isinstance(page, dict) else type(page).__name__
            raise InputError(f"run {run_id}: not a jobs response (keys={keys}); "
                             "a redirect/error body is not an empty job list")
        if "total_count" not in page:
            raise InputError(f"run {run_id}: jobs response is missing total_count")
        page_total = page["total_count"]
        if not isinstance(page_total, int) or page_total < 0:
            raise InputError(f"run {run_id}: invalid total_count {page_total!r}")
        if total is None:
            total = page_total
        jobs.extend(job for job in page["jobs"] if isinstance(job, dict))
    if total is None or len(jobs) != total:
        raise InputError(f"run {run_id}: fetched {len(jobs)} jobs but API reported {total}; "
                         "refusing to report a partial count")
    return total, jobs


def summarize(run: dict[str, Any], jobs: list[dict[str, Any]], total: int) -> dict[str, Any]:
    """Bucket every job by status first, then by conclusion."""
    counts = {
        "total": total,
        "active": 0,
        "success": 0,
        "failure": 0,
        "cancelled": 0,
        "skipped": 0,
        "other": 0,
    }
    active_jobs: list[dict[str, str]] = []
    failed_jobs: list[dict[str, str]] = []
    cancelled_jobs: list[dict[str, str]] = []
    for job in jobs:
        name = str(job.get("name", "?"))
        status = str(job.get("status", "?"))
        conclusion = job.get("conclusion")
        url = str(job.get("html_url", ""))
        entry = {"name": name, "status": status,
                 "conclusion": str(conclusion) if conclusion is not None else "-", "url": url}
        if status in ACTIVE_STATUSES:
            counts["active"] += 1
            active_jobs.append(entry)
        elif conclusion in FAILURE_CONCLUSIONS:
            counts["failure"] += 1
            failed_jobs.append(entry)
        elif conclusion == "cancelled":
            counts["cancelled"] += 1
            cancelled_jobs.append(entry)
        elif conclusion in ("skipped", "neutral"):
            counts["skipped"] += 1
        elif conclusion == "success":
            counts["success"] += 1
        else:
            counts["other"] += 1
            failed_jobs.append(entry)
    run_conclusion = run.get("conclusion")
    return {
        "id": str(run.get("id", "?")),
        "name": str(run.get("name", "?")),
        "status": str(run.get("status", "?")),
        "conclusion": str(run_conclusion) if run_conclusion is not None else "running",
        "head_sha": str(run.get("head_sha", "?"))[:10],
        "url": str(run.get("html_url", "")),
        "counts": counts,
        "active_jobs": active_jobs,
        "failed_jobs": failed_jobs,
        "cancelled_jobs": cancelled_jobs,
    }


def load_summary(repo: str, run_id: str) -> dict[str, Any]:
    run = gh_api(f"repos/{repo}/actions/runs/{run_id}")
    if not isinstance(run, dict) or "status" not in run or "id" not in run:
        raise InputError(f"run {run_id}: not a workflow-run response: {str(run)[:200]}")
    pages = gh_api(f"repos/{repo}/actions/runs/{run_id}/jobs?per_page=100", paginate=True)
    total, jobs = validate_pages(pages, run_id)
    return summarize(run, jobs, total)


def print_report(summary: dict[str, Any]) -> None:
    counts = summary["counts"]
    print(f"{summary['name']} run {summary['id']} @ {summary['head_sha']}: "
          f"{summary['status']}/{summary['conclusion']}")
    print(f"  jobs: total={counts['total']} active={counts['active']} "
          f"success={counts['success']} failed={counts['failure']} "
          f"cancelled={counts['cancelled']} skipped={counts['skipped']} "
          f"other={counts['other']}")
    print(f"  {summary['url']}")
    for label, key in (("FAILED", "failed_jobs"), ("ACTIVE", "active_jobs"),
                       ("CANCELLED", "cancelled_jobs")):
        entries = summary[key]
        if not entries:
            continue
        print(f"  {label} ({len(entries)}):")
        for entry in entries:
            print(f"    - {entry['name']}: {entry['status']}/{entry['conclusion']} {entry['url']}")


def self_test() -> None:
    """The exact regression: 56 total jobs must never be reported as active."""
    jobs = []
    for index in range(56):
        jobs.append({
            "name": f"job-{index}",
            "status": "completed" if index < 43 else "in_progress",
            "conclusion": "success" if index < 43 else None,
            "html_url": f"https://example.test/job/{index}",
        })
    pages = [{"total_count": 56, "jobs": jobs[:20]},
             {"total_count": 56, "jobs": jobs[20:]}]
    total, merged = validate_pages(pages, "test")
    result = summarize({"id": 1, "name": "CI", "status": "in_progress", "conclusion": None,
                        "head_sha": "a" * 40, "html_url": "https://example.test/run/1"},
                       merged, total)
    assert result["counts"]["total"] == 56
    assert result["counts"]["active"] == 13, result["counts"]
    assert result["counts"]["success"] == 43, result["counts"]
    for bad in ({"message": "Moved Permanently", "url": "https://example.test"},
                {"total_count": 56}, {"jobs": [], "total_count": "56"}):
        try:
            validate_pages([bad], "test")
        except InputError:
            pass
        else:
            raise AssertionError(f"accepted unusable payload: {bad}")
    try:
        validate_pages([{"total_count": 56, "jobs": jobs[:20]}], "test")
    except InputError:
        pass
    else:
        raise AssertionError("accepted a partial job list")
    print("ci_run_status: self-test OK (total=56, active=13, redirect rejected)")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("run_ids", nargs="*", help="GitHub Actions run ids")
    parser.add_argument("--repo", default="GeneralBots/BotServer", help="owner/repo")
    parser.add_argument("--require-complete", action="store_true",
                        help="exit 2 while any job is still active")
    parser.add_argument("--allow-cancelled", action="store_true",
                        help="treat a cancelled run as green (superseded push)")
    parser.add_argument("--json", action="store_true", help="print machine-readable JSON")
    parser.add_argument("--self-test", action="store_true", help="run built-in regression checks")
    args = parser.parse_args()
    if args.self_test:
        self_test()
        return 0
    if not args.run_ids:
        parser.error("provide at least one run id (or --self-test)")

    summaries: list[dict[str, Any]] = []
    for run_id in args.run_ids:
        try:
            summary = load_summary(args.repo, run_id)
        except InputError as error:
            print(f"ci_run_status: {error}", file=sys.stderr)
            return 2
        summaries.append(summary)
        if not args.json:
            print_report(summary)

    if args.json:
        print(json.dumps(summaries if len(summaries) > 1 else summaries[0], indent=2))
    if any(s["counts"]["failure"] or s["counts"]["other"] for s in summaries):
        return 1
    if any(s["conclusion"] == "failure" for s in summaries):
        return 1
    if any(s["conclusion"] == "cancelled" and not args.allow_cancelled for s in summaries):
        return 1
    if args.require_complete and any(s["counts"]["active"] for s in summaries):
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
