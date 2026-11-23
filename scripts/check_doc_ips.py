#!/usr/bin/env python3
"""Fail when a tracked Markdown file contains a real IP literal (#2674).

Production addresses belong in the private operator record, not in the
repository. Run from the repo root with no arguments to scan all git-tracked
``*.md`` files, or pass explicit paths::

    python3 scripts/check_doc_ips.py
    python3 scripts/check_doc_ips.py INFRA.md

Allowlisted values are documentation placeholders only: loopback, the null
address, broadcast, netmasks, the well-known public DNS resolvers used in
copy/paste examples, and the RFC 5737 documentation networks.
"""

from __future__ import annotations

import ipaddress
import re
import subprocess
import sys
from pathlib import Path

IPV4 = re.compile(r"(?<![\w.])((?:\d{1,3}\.){3}\d{1,3})(?![\w.])")

ALLOWED = {
    ipaddress.IPv4Address(value)
    for value in (
        "0.0.0.0",
        "127.0.0.1",
        "255.255.255.0",
        "255.255.255.255",
        "8.8.8.8",
        "8.8.4.4",
        "1.1.1.1",
        "9.9.9.9",
    )
}
# Migration/version names that happen to look like dotted quads.
VERSION_LITERALS = {"6.5.15.1"}

DOC_NETWORKS = tuple(
    ipaddress.IPv4Network(value)
    for value in (
        "192.0.2.0/24",          # RFC 5737 documentation
        "198.51.100.0/24",
        "203.0.113.0/24",
        "10.0.0.0/8",            # RFC 1918 private examples
        "172.16.0.0/12",
        "192.168.0.0/16",
        "127.0.0.0/8",
        "169.254.0.0/16",        # link-local (cloud metadata examples)
        "100.64.0.0/10",         # CGNAT
        "172.0.0.0/8",           # reserved / documented over-blocking example
        "224.0.0.0/4",           # multicast
        "240.0.0.0/4",           # reserved
    )
)


def tracked_markdown() -> list[Path]:
    result = subprocess.run(
        ["git", "ls-files", "*.md"],
        check=True,
        capture_output=True,
        text=True,
    )
    return [Path(line) for line in result.stdout.splitlines() if line]


def scan(path: Path) -> list[tuple[int, str]]:
    findings: list[tuple[int, str]] = []
    try:
        text = path.read_text(encoding="utf-8")
    except (OSError, UnicodeDecodeError) as error:
        print(f"skip {path}: {error}", file=sys.stderr)
        return findings
    for number, line in enumerate(text.splitlines(), 1):
        for match in IPV4.finditer(line):
            # `6.5.15.1-consolidated` is a migration name, not an address;
            # `6.5.15.1:83` / `6.5.15.1/up.sql` are migration line references.
            tail = line[match.end():]
            if tail[:1] in {"-", "_"} or re.match(r"[:][0-9]|/[/\\]?[0-9]", tail):
                continue
            try:
                address = ipaddress.IPv4Address(match.group(1))
            except ipaddress.AddressValueError:
                continue
            if match.group(1) in VERSION_LITERALS:
                continue
            if address in ALLOWED or any(address in network for network in DOC_NETWORKS):
                continue
            findings.append((number, match.group(1)))
    return findings


def main(argv: list[str]) -> int:
    paths = [Path(arg) for arg in argv] if argv else tracked_markdown()
    failures = 0
    for path in paths:
        for number, address in scan(path):
            print(f"{path}:{number}: real IP literal {address} — use a placeholder")
            failures += 1
    if failures:
        print(f"check_doc_ips: FAIL — {failures} literal(s)")
        return 1
    print(f"check_doc_ips: OK — {len(paths)} file(s) clean")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
