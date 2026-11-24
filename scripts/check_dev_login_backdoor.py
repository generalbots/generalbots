#!/usr/bin/env python3
"""Guard against the static admin login backdoor (issue #2759).

`botcloud::lookup_admin_credentials_user_id` must authorize only via a
case-insensitive email match plus a salted SHA-256 digest from
`admin-credentials.json`. A `BOTSERVER_DEV_MODE` env check accepting a hardcoded
`admin@localhost` / `dev` pair is a fixed credential in any build that sets the
flag, so no first-party Rust source may contain that flag or that pair.

Test modules (`#[cfg(test)]`) may mention the legacy strings to assert they are
rejected; they are stripped before scanning, like in the error ratchet.

Usage:
    python3 scripts/check_dev_login_backdoor.py --check
    python3 scripts/check_dev_login_backdoor.py --self-test

Exit codes: 0 = clean, 1 = violations, 2 = usage error.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SCAN_ROOT = Path("botserver")
CFG_TEST_RE = re.compile(r"#\[cfg\(\s*test\s*\)\]")

# The env flag and the hardcoded password check are unique to the bypass.
MARKERS = (
    "BOTSERVER_DEV_MODE",
    'password == "dev"',
    'password.eq_ignore_ascii_case("dev")',
)
# `admin@localhost` is a legitimate identity elsewhere (bucket naming, admin
# claim checks), so the email half is only a marker in the login helper itself.
LOGIN_HELPER = "login_helpers.rs"
EMAIL_MARKER = "admin@localhost"


def is_test_path(relative: Path) -> bool:
    parts = relative.parts
    if any(part in {"tests", "test", "testdata", "benches", "target"} for part in parts[:-1]):
        return True
    return relative.name == "tests.rs" or relative.name.endswith(("_test.rs", "_tests.rs"))


def strip_cfg_test(text: str) -> str:
    """Remove `#[cfg(test)]` items by brace matching, preserving line numbers."""
    pieces: list[str] = []
    pos = 0
    for match in CFG_TEST_RE.finditer(text):
        if match.start() < pos:
            continue
        brace = text.find("{", match.end())
        semicolon = text.find(";", match.end())
        if brace == -1 or (semicolon != -1 and semicolon < brace):
            end = semicolon + 1 if semicolon != -1 else match.end()
        else:
            depth = 0
            end = len(text)
            for index in range(brace, len(text)):
                if text[index] == "{":
                    depth += 1
                elif text[index] == "}":
                    depth -= 1
                    if depth == 0:
                        end = index + 1
                        break
        pieces.append(text[pos:match.start()])
        pieces.append("\n" * text[match.start():end].count("\n"))
        pos = end
    pieces.append(text[pos:])
    return "".join(pieces)


def scan_text(text: str, source: str) -> list[str]:
    """`<source>:<line>: <marker>` violations in one production Rust file."""
    violations: list[str] = []
    cleaned = strip_cfg_test(text)
    markers = MARKERS + ((EMAIL_MARKER,) if source.endswith(LOGIN_HELPER) else ())
    for number, line in enumerate(cleaned.splitlines(), start=1):
        for marker in markers:
            if marker in line:
                violations.append(f"{source}:{number}: dev-login backdoor marker `{marker}`")
    return violations


def rust_files(root: Path) -> list[Path]:
    scan = root / SCAN_ROOT
    if not scan.is_dir():
        return []
    return sorted(
        path.relative_to(root)
        for path in scan.rglob("*.rs")
        if path.is_file() and not is_test_path(path.relative_to(root))
    )


def run_check(root: Path = ROOT) -> int:
    failures = 0
    for relative in rust_files(root):
        text = (root / relative).read_text(encoding="utf-8", errors="replace")
        for violation in scan_text(text, relative.as_posix()):
            print(violation, file=sys.stderr)
            failures += 1
    if failures:
        print(
            f"\n{failures} dev-login backdoor marker(s); no build may accept a "
            "hardcoded admin credential (#2759).",
            file=sys.stderr,
        )
        return 1
    print("dev-login-backdoor: clean")
    return 0


def self_test() -> int:
    """Prove the guard fires on the old code and passes the fixed shape."""
    failures = 0
    planted = (
        'if std::env::var("BOTSERVER_DEV_MODE").is_ok()\n'
        '    && email.eq_ignore_ascii_case("admin@localhost")\n'
        '    && password == "dev"\n'
        '{\n    return Some("1".to_string());\n}\n'
    )
    found = scan_text(planted, "botserver/crates/botcloud/src/api/login_helpers.rs")
    if not found:
        print("self-test: the legacy backdoor was not detected", file=sys.stderr)
        failures += 1

    clean = 'if !credentials_authorize(&json, email, password) { continue; }\n'
    if scan_text(clean, "botserver/crates/botcloud/src/api/login_helpers.rs"):
        print("self-test: the fixed predicate was flagged", file=sys.stderr)
        failures += 1

    with_test = (
        clean
        + '#[cfg(test)]\nmod tests {\n'
        + '    assert!(!credentials_authorize(&json, "admin@localhost", "dev"));\n}\n'
    )
    if scan_text(with_test, "botserver/crates/botcloud/src/api/login_helpers.rs"):
        print("self-test: test-module legacy strings were flagged", file=sys.stderr)
        failures += 1

    if failures:
        return 1
    print("dev-login-backdoor self-test: ok")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="scan the tree (default)")
    parser.add_argument("--self-test", action="store_true", help="prove each rule fires")
    args = parser.parse_args()
    if args.self_test:
        return self_test()
    return run_check()


if __name__ == "__main__":
    sys.exit(main())
