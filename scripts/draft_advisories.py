#!/usr/bin/env python3
"""Draft security advisories from scheduled dependency-scan reports (#2725).

Reads OSV (Cargo.lock + npm), pip-audit and cargo-deny JSON, normalizes and
deduplicates findings, applies expiring waivers, writes a Markdown report and
one draft issue body per NEW finding, and optionally opens those drafts as
GitHub/Forgejo issues for human triage.

Invariants
----------
* This script NEVER publishes an advisory, CSAF document or OSV record. The
  only side effect is a draft issue (label ``advisory-draft``) and a report.
* Every skip is audited: published, already-drafted, waived and expired-waiver
  decisions all appear in the report with their reason.
* Waivers carry an approver and an expiry; an expired waiver stops suppressing
  and the finding is drafted again.

Usage:
    python3 scripts/draft_advisories.py --self-test
    python3 scripts/draft_advisories.py \
        --osv-report /tmp/osv.json --osv-report /tmp/osv-npm.json \
        --pip-audit-report /tmp/pip.json --deny-report /tmp/deny.json \
        --report /tmp/advisory-draft-report.md --drafts-dir /tmp/advisory-drafts

Exit codes: 0 clean, 1 triage/waiver/API error, 2 bad input.
"""
from __future__ import annotations

import argparse
import json
import os
import sys
import urllib.error
import urllib.parse
import urllib.request
from datetime import date
from pathlib import Path
from typing import Any

sys.path.insert(0, str(Path(__file__).resolve().parent))

import advisory_draft_lib as lib  # noqa: E402  (path set above)
from advisory_draft_render import render_draft, render_report  # noqa: E402

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_TEMPLATE = ROOT / "scripts" / "advisory_draft_template.md"
DEFAULT_WAIVERS = ROOT / "scripts" / "advisory_waivers.toml"
DEFAULT_PUBLISHED = ROOT / "scripts" / "published_advisories.txt"
DEFAULT_STATE = ROOT / "scripts" / "advisory_drafts_state.json"


class ApiError(Exception):
    """A REST call failed; surfaced with the status code, never a raw token."""


def read_json(path: Path) -> Any:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise ValueError(f"{path}: cannot read JSON: {error}") from error


def write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def api_request(method: str, url: str, token: str, payload: dict | None = None) -> Any:
    body = json.dumps(payload).encode("utf-8") if payload is not None else None
    request = urllib.request.Request(
        url,
        data=body,
        method=method,
        headers={
            "Authorization": f"Bearer {token}",
            "Accept": "application/json",
            "Content-Type": "application/json",
            "User-Agent": "generalbots-advisory-drafter/1.0",
        },
    )
    try:
        with urllib.request.urlopen(request, timeout=30) as response:
            raw = response.read().decode("utf-8")
    except urllib.error.HTTPError as error:
        raise ApiError(f"{method} {urllib.parse.urlsplit(url).path}: HTTP {error.code}") from error
    except (urllib.error.URLError, TimeoutError, OSError) as error:
        raise ApiError(f"{method} {urllib.parse.urlsplit(url).path}: {error}") from error
    if not raw:
        return None
    try:
        return json.loads(raw)
    except json.JSONDecodeError as error:
        raise ApiError(f"{method} {urllib.parse.urlsplit(url).path}: invalid JSON") from error


def existing_issue_ids(api: str, repo: str, token: str) -> set[str]:
    """Advisory IDs already present in open or closed draft issues."""
    url = (
        f"{api}/repos/{repo}/issues?labels=advisory-draft&state=all&per_page=100"
    )
    issues = api_request("GET", url, token)
    if not isinstance(issues, list):
        raise ApiError("issue lookup returned a non-list response")
    found: set[str] = set()
    for issue in issues:
        if "pull_request" in issue:
            continue
        found |= lib.ids_in_text(
            " ".join(
                (
                    str(issue.get("title", "")),
                    str(issue.get("body", "")),
                )
            )
        )
    return found


def ensure_labels(api: str, repo: str, token: str) -> None:
    """Create the triage label if missing; dedup queries filter on it."""
    labels = (
        ("advisory-draft", "f9d0c4", "Automated dependency advisory draft - needs human triage"),
        ("security", "ee0701", "Security work"),
    )
    for name, color, description in labels:
        try:
            api_request(
                "POST",
                f"{api}/repos/{repo}/labels",
                token,
                {"name": name, "color": color, "description": description},
            )
        except ApiError:
            # 422 means the label already exists; any other error is retried
            # on the next run and the report still reaches the artifact.
            continue


def open_draft(api: str, repo: str, token: str, body: str, title: str) -> int:
    issue = api_request(
        "POST",
        f"{api}/repos/{repo}/issues",
        token,
        {"title": title, "body": body, "labels": ["advisory-draft", "security"]},
    )
    number = issue.get("number") if isinstance(issue, dict) else None
    if not isinstance(number, int):
        raise ApiError("issue creation response has no issue number")
    return number


def load_state(path: Path) -> dict:
    if not path.is_file():
        return {}
    try:
        state = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return {}
    return state if isinstance(state, dict) else {}


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--osv-report", type=Path, action="append", default=[], help="osv-scanner JSON (repeatable)")
    parser.add_argument("--pip-audit-report", type=Path, action="append", default=[], help="pip-audit JSON (repeatable)")
    parser.add_argument("--deny-report", type=Path, action="append", default=[], help="cargo-deny JSON (repeatable)")
    parser.add_argument("--waivers", type=Path, default=DEFAULT_WAIVERS)
    parser.add_argument("--published-ids", type=Path, default=DEFAULT_PUBLISHED)
    parser.add_argument("--existing-ids", type=Path, help="extra newline-separated IDs already drafted")
    parser.add_argument("--state", type=Path, default=DEFAULT_STATE)
    parser.add_argument("--template", type=Path, default=DEFAULT_TEMPLATE)
    parser.add_argument("--report", type=Path, help="output Markdown report path")
    parser.add_argument("--drafts-dir", type=Path, help="output directory for draft bodies")
    parser.add_argument("--today", help="override today (YYYY-MM-DD) for expiry testing")
    parser.add_argument("--run-url", default="scheduled", help="CI run reference for the report header")
    parser.add_argument("--create-issues", action="store_true", help="open draft issues via the REST API")
    parser.add_argument("--api-url", help="REST API base, e.g. https://api.github.com")
    parser.add_argument("--repo", help="owner/repo that receives the draft issues")
    parser.add_argument("--token-env", default="ADVISORY_TOKEN", help="environment variable holding the token")
    parser.add_argument("--self-test", action="store_true", help="run offline fixtures and exit")
    return parser


def collect_findings(args: argparse.Namespace) -> list:
    findings: list = []
    for path in args.osv_report:
        findings += lib.findings_from_osv(read_json(path), path.name)
    for path in args.pip_audit_report:
        findings += lib.findings_from_pip_audit(read_json(path), path.name)
    for path in args.deny_report:
        findings += lib.findings_from_deny(read_json(path), path.name)
    return lib.dedupe(findings)


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    if args.self_test:
        from advisory_draft_selftest import run_self_test

        return run_self_test()

    if not (args.report and args.drafts_dir):
        print("error: --report and --drafts-dir are required", file=sys.stderr)
        return 2

    if args.today:
        try:
            today = date.fromisoformat(args.today)
        except ValueError:
            print("error: --today must be YYYY-MM-DD", file=sys.stderr)
            return 2
    else:
        today = date.today()

    if args.create_issues and not (args.api_url and args.repo and os.environ.get(args.token_env)):
        print("error: --create-issues needs --api-url, --repo and the token env var", file=sys.stderr)
        return 2

    try:
        findings = collect_findings(args)
    except ValueError as error:
        print(f"error: {error}", file=sys.stderr)
        return 2

    waivers, errors = lib.load_waivers(args.waivers)
    if errors:
        for error in errors:
            print(f"error: {error}", file=sys.stderr)
        return 1

    existing_ids = lib.load_ids(args.published_ids) | lib.load_state_ids(args.state)
    if args.existing_ids:
        existing_ids |= lib.load_ids(args.existing_ids)
    published_ids = lib.load_ids(args.published_ids)

    if args.create_issues:
        try:
            existing_ids |= existing_issue_ids(args.api_url, args.repo, os.environ[args.token_env])
        except ApiError as error:
            print(f"error: {error}", file=sys.stderr)
            return 1

    decisions = lib.classify(findings, waivers, published_ids, existing_ids, today)
    write_text(args.report, render_report(decisions, today, args.run_url))

    try:
        template = args.template.read_text(encoding="utf-8")
    except OSError as error:
        print(f"error: cannot read template {args.template}: {error}", file=sys.stderr)
        return 2

    drafts = [d for d in decisions if d.status in lib.DRAFT_STATUSES]
    args.drafts_dir.mkdir(parents=True, exist_ok=True)
    bodies: list[tuple] = []
    for decision in drafts:
        body = render_draft(decision.finding, template, args.report.name)
        write_text(args.drafts_dir / f"{decision.finding.advisory_id}.md", body)
        bodies.append((decision, body))

    if args.create_issues:
        ensure_labels(args.api_url, args.repo, os.environ[args.token_env])
        state = load_state(args.state)
        failures = 0
        for decision, body in bodies:
            finding = decision.finding
            title = f"sec/advisory-draft: {finding.advisory_id} - {finding.package}"
            try:
                number = open_draft(args.api_url, args.repo, os.environ[args.token_env], body, title)
            except ApiError as error:
                print(f"error: {finding.advisory_id}: {error}", file=sys.stderr)
                failures += 1
                continue
            state[finding.advisory_id.upper()] = {
                "issue": number,
                "created": today.isoformat(),
            }
            print(f"opened draft issue #{number} for {finding.advisory_id}")
        write_text(args.state, json.dumps(dict(sorted(state.items())), indent=2) + "\n")
        if failures:
            print(f"error: {failures} draft issue(s) could not be opened", file=sys.stderr)
            return 1
    else:
        print("dry run: no issues opened, no advisories published (use --create-issues in CI)")

    print(
        f"findings={len(findings)} drafts={len(bodies)} waived="
        f"{sum(1 for d in decisions if d.status == 'waived')} "
        f"expired={sum(1 for d in decisions if d.status == 'expired-waiver')} "
        f"deduplicated={sum(1 for d in decisions if d.status in ('published', 'already-drafted'))}"
    )
    print(f"report: {args.report}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
