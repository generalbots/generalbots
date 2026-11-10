#!/usr/bin/env python3
"""SSRF fetch-site gate (issue #2131).

The project's only allowed way to fetch a caller-supplied URL is

    botsecurity_core::validation::fetch_user_url(url, &policy)

It rejects non-http(s) schemes, credentials in the URL, obfuscated IPv4,
loopback/private/link-local/CGNAT/metadata addresses, re-validates every
redirect hop and re-checks the connected peer address. A fetch site that calls
`client.get(...)` / `reqwest::get(...)` with a *variable* URL and does not call
the guard is therefore a server-side request forgery hole.

This gate lists every outbound fetch call under `botserver/` and classifies it:

  guarded    the enclosing function calls the shared guard
  fixed      the URL is a literal (or a `format!`/`concat!` whose first
             fragment is an absolute URL, so the host cannot be influenced)
  allowlist  the file is listed in `scripts/ssrf_allowlist.txt` with a reason
  VIOLATION  none of the above — the gate fails

The allowlist is shrink-only: an entry that no longer suppresses a violation is
itself a failure, so fixing a file forces the line to be deleted.

Usage
-----
    python3 scripts/check_ssrf.py --check
    python3 scripts/check_ssrf.py --list
    python3 scripts/check_ssrf.py --self-test
    python3 scripts/check_ssrf.py --root <repo>

Exit codes: 0 clean, 1 violations found.
"""

from __future__ import annotations

import argparse
import re
import sys
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SCAN_ROOT = Path("botserver")
ALLOWLIST = Path("scripts/ssrf_allowlist.txt")

# The shared guard. Any one of these in the enclosing function is enough.
GUARD_MARKERS = (
    "fetch_user_url",
    "fetch_internal_service_url",
    "validate_url_ssrf_strict",
    "validate_url_ssrf",
)

# `reqwest::get(x)` or a client-ish receiver followed by `.get(x)`. The
# receiver list keeps `HashMap::get`, `Vec::get` and friends out of the scan.
FETCH_CALL = re.compile(
    r"(?:reqwest::get\s*\(|"
    r"(?:\bself\.client\b|\bself\.http_client\b|\bclient\b|\bhttp_client\b|"
    r"\breqwest_client\b)\s*\.\s*get\s*\()"
)

FN_START = re.compile(r"^(?P<indent>\s*)(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?fn\s+\w+")

LITERAL_URL = re.compile(r'^\s*(?:b)?"https?://')
FORMAT_PREFIX = re.compile(r'^\s*(?:format|concat)!\s*\(\s*(?:b)?"https?://')


@dataclass
class Site:
    path: str
    line: int
    function: str
    argument: str
    status: str


def is_test_file(relative: Path) -> bool:
    name = relative.name
    return (
        name == "tests.rs"
        or name.endswith("_tests.rs")
        or "tests" in relative.parts
        or name == "test.rs"
    )


def rust_files(root: Path) -> list[Path]:
    scan = root / SCAN_ROOT
    if not scan.is_dir():
        return []
    files = []
    for path in sorted(scan.rglob("*.rs")):
        if "target" in path.parts:
            continue
        relative = path.relative_to(root)
        if is_test_file(relative):
            continue
        files.append(relative)
    return files


def call_argument(text: str, open_paren: int, limit: int = 600) -> str:
    """Text inside the parentheses starting at `open_paren`, paren-balanced."""
    depth = 0
    out: list[str] = []
    for ch in text[open_paren : open_paren + limit]:
        if ch == "(":
            depth += 1
            if depth == 1:
                continue
        elif ch == ")":
            depth -= 1
            if depth == 0:
                break
        out.append(ch)
    return "".join(out)


def enclosing_function(lines: list[str], line_index: int) -> tuple[str, int, int]:
    """`(name, start, end)` of the function containing `line_index` (0-based)."""
    start = None
    indent = ""
    for index in range(line_index, -1, -1):
        match = FN_START.match(lines[index])
        if match:
            start = index
            indent = match.group("indent")
            name = lines[index].strip()
            break
    if start is None:
        return ("<file>", 0, len(lines) - 1)
    # Body ends at the first line after `start` that closes at the same indent.
    end = len(lines) - 1
    closing = indent + "}"
    for index in range(start + 1, len(lines)):
        if lines[index] == closing or lines[index].rstrip() == closing:
            end = index
            break
    return (name, start, end)


def classify(argument: str, body: str) -> str:
    if LITERAL_URL.match(argument) or FORMAT_PREFIX.match(argument):
        return "fixed"
    if any(marker in body for marker in GUARD_MARKERS):
        return "guarded"
    return "violation"


def scan_text(relative: str, text: str) -> list[Site]:
    """Every outbound fetch site in one file, with its classification."""
    sites: list[Site] = []
    lines = text.splitlines()
    for match in FETCH_CALL.finditer(text):
        line_no = text.count("\n", 0, match.start()) + 1
        open_paren = text.index("(", match.start())
        argument = call_argument(text, open_paren)
        name, start, end = enclosing_function(lines, line_no - 1)
        body = "\n".join(lines[start : end + 1])
        sites.append(
            Site(
                path=relative,
                line=line_no,
                function=name,
                argument=argument.strip().replace("\n", " ")[:120],
                status=classify(argument, body),
            )
        )
    return sites


def read_allowlist(root: Path) -> list[tuple[str, str]]:
    path = root / ALLOWLIST
    if not path.is_file():
        return []
    entries: list[tuple[str, str]] = []
    for raw in path.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        target, _, reason = line.partition("#")
        entries.append((target.strip(), reason.strip()))
    return entries


def collect(root: Path) -> list[Site]:
    sites: list[Site] = []
    for relative in rust_files(root):
        text = (root / relative).read_text(encoding="utf-8", errors="replace")
        sites.extend(scan_text(relative.as_posix(), text))
    return sites


def run_check(root: Path = ROOT, list_only: bool = False) -> int:
    allowlist = read_allowlist(root)
    allowed = {path for path, _ in allowlist}
    used: set[str] = set()
    failures = 0

    sites = collect(root)
    for site in sites:
        if site.status == "violation" and site.path in allowed:
            used.add(site.path)
            site.status = "allowlist"

    if list_only:
        for site in sites:
            print(f"{site.status:10} {site.path}:{site.line}  {site.argument}")
        print(f"\n{len(sites)} fetch site(s)")

    for site in sites:
        if site.status == "violation":
            print(
                f"{site.path}:{site.line}: unguarded fetch of a variable URL "
                f"({site.argument}) — call fetch_user_url() or justify it in "
                f"{ALLOWLIST}",
                file=sys.stderr,
            )
            failures += 1

    for path, reason in allowlist:
        if not reason:
            print(f"{ALLOWLIST}: entry without a reason: {path}", file=sys.stderr)
            failures += 1
        if path not in used:
            print(
                f"{ALLOWLIST}: stale entry (no unguarded fetch anymore): {path}",
                file=sys.stderr,
            )
            failures += 1

    if failures:
        print(
            f"\n{failures} SSRF fetch-site violation(s). Route the call through "
            "botsecurity_core::validation::fetch_user_url (public targets) or "
            "fetch_internal_service_url (allowlisted internal services).",
            file=sys.stderr,
        )
        return 1
    print(f"ssrf: clean ({len(sites)} fetch site(s))")
    return 0


def self_test() -> int:
    """Prove the three classifications and the violation path."""
    failures = 0

    unguarded = scan_text(
        "botserver/src/x.rs",
        "async fn fetch(url: &str) {\n    let r = client.get(url).send().await;\n}\n",
    )
    if not any(site.status == "violation" for site in unguarded):
        print("self-test: unguarded variable fetch not flagged", file=sys.stderr)
        failures += 1

    guarded = scan_text(
        "botserver/src/y.rs",
        "async fn fetch(url: &str) {\n"
        "    let r = fetch_user_url(url, &policy).await?;\n"
        "    let s = client.get(url).send().await;\n"
        "}\n",
    )
    if any(site.status == "violation" for site in guarded):
        print("self-test: guarded fetch flagged", file=sys.stderr)
        failures += 1

    literal = scan_text(
        "botserver/src/z.rs",
        'async fn probe() {\n    let r = reqwest::get("https://example.com/health").await;\n}\n',
    )
    if any(site.status == "violation" for site in literal):
        print("self-test: literal URL flagged", file=sys.stderr)
        failures += 1

    format_prefix = scan_text(
        "botserver/src/w.rs",
        'async fn probe() {\n'
        '    let u = format!("https://api.example.com/{}", id);\n'
        "    let r = client.get(&u).send().await;\n"
        "}\n",
    )
    if any(site.status == "violation" for site in format_prefix):
        print("self-test: fixed-host format! flagged", file=sys.stderr)
        failures += 1

    if failures:
        return 1
    print("ssrf self-test: ok")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="scan the tree (default)")
    parser.add_argument("--list", action="store_true", help="list every fetch site")
    parser.add_argument("--self-test", action="store_true", help="prove the rules fire")
    parser.add_argument("--root", default=str(ROOT), help="repository root")
    args = parser.parse_args()

    if args.self_test:
        return self_test()
    return run_check(Path(args.root), list_only=args.list)


if __name__ == "__main__":
    sys.exit(main())
