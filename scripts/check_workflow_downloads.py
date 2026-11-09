#!/usr/bin/env python3
"""Fail when a workflow downloads a file without verifying it (#2001).

The #2001 self-review of the release pipeline found the last two unverified
downloads (mdBook in `ci.yml`, the relocatable CPython interpreter in
`release.yml`) and pinned them. This guard keeps that property: every `run:`
step that writes a downloaded file must also verify a SHA-256 in the same step.

Two shapes are accepted, because both are real roots of trust:

* a literal digest — `echo "<sha256>  <path>" | sha256sum -c -`
* a checksum asset from the same release — the step fetches `SHA256SUMS*`
  next to the artifact and checks the artifact against it
  (`sha256sum -c`, `sha256sum -c -`, or `sha256sum -c SHA256SUMS-...`)

Downloads that are not written to a file (`-o /dev/null`, plain GET probes,
local `127.0.0.1` health checks) are ignored. Genuine exceptions belong in
`scripts/workflow_download_allowlist.txt` as `workflow:marker  reason`.

Usage: python3 scripts/check_workflow_downloads.py [--dir DIR]
Exit codes: 0 all downloads verified, 1 otherwise.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

DEFAULT_DIR = Path(".github/workflows")
DEFAULT_ALLOWLIST = Path("scripts/workflow_download_allowlist.txt")

# `curl ... -o file`, `curl --output file`, `wget -O file`
DOWNLOAD = re.compile(
    r"\b(?:curl|wget)\b[^\n]*?"
    r"(?:\s-o\s+|\s--output\s+|\s-O\s+)(?P<path>[^\s'\"]+)"
)
# `curl ... > file`
REDIRECT = re.compile(r"\b(?:curl|wget)\b[^\n]*?>\s*(?P<path>[^\s'\"]+)")
VERIFY = re.compile(r"sha256sum\s+-c\b", re.IGNORECASE)


def load_allowlist(path: Path) -> dict[str, str]:
    allowed: dict[str, str] = {}
    if not path.is_file():
        return allowed
    for raw in path.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        parts = line.split(None, 1)
        allowed[parts[0]] = parts[1] if len(parts) > 1 else ""
    return allowed


def run_blocks(lines: list[str]) -> list[tuple[int, list[str]]]:
    """(first-line-number, body) for every `run: |` block."""
    blocks: list[tuple[int, list[str]]] = []
    i = 0
    while i < len(lines):
        match = re.match(r"^(\s*)run:\s*\|", lines[i])
        if not match:
            i += 1
            continue
        indent = len(match.group(1))
        start = i + 2
        body: list[str] = []
        i += 1
        while i < len(lines):
            line = lines[i]
            if line.strip() and (len(line) - len(line.lstrip())) <= indent:
                break
            body.append(line)
            i += 1
        blocks.append((start, body))
    return blocks


def unverified(workflow: Path, allowlist: dict[str, str]) -> list[str]:
    lines = workflow.read_text(encoding="utf-8").splitlines()
    problems: list[str] = []
    for start, body in run_blocks(lines):
        # Join `\`-continued lines first: `curl -fsSL \\` + `  -o file \\` is a
        # single command, and the target sits on a later physical line.
        text = re.sub(r"\\\n", " ", "\n".join(body))
        targets = [
            m.group("path")
            for pattern in (DOWNLOAD, REDIRECT)
            for m in pattern.finditer(text)
        ]
        targets = [t for t in targets if t not in ("/dev/null", "-")]
        if not targets or VERIFY.search(text):
            continue
        marker = f"{workflow.name}:{Path(targets[0]).name}"
        if marker in allowlist:
            continue
        problems.append(
            f"{workflow}: step starting at line {start} downloads "
            f"{', '.join(sorted(set(targets)))} without `sha256sum -c`"
        )
    return problems


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--dir", default=str(DEFAULT_DIR))
    parser.add_argument("--allowlist", default=str(DEFAULT_ALLOWLIST))
    args = parser.parse_args()

    workflows = sorted(Path(args.dir).glob("*.yml"))
    if not workflows:
        print(f"check_workflow_downloads: no workflows under {args.dir}", file=sys.stderr)
        return 2

    allowlist = load_allowlist(Path(args.allowlist))
    problems = [p for wf in workflows for p in unverified(wf, allowlist)]
    if problems:
        print("unverified workflow downloads:", file=sys.stderr)
        for problem in problems:
            print(f"  - {problem}", file=sys.stderr)
        print(
            "Pin a literal digest (`echo \"<sha256>  <path>\" | sha256sum -c -`) "
            "or verify against the release's SHA256SUMS asset.",
            file=sys.stderr,
        )
        return 1

    print(f"ok: every file download across {len(workflows)} workflow(s) is verified")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
