#!/usr/bin/env python3
"""JWT hardening guards (issue #2125).

Two primitives are dangerous enough to be enforced mechanically instead of in
reviewer memory:

A. dangerous-decode   `jsonwebtoken::dangerous` / `insecure_decode` return
                      claims without checking a signature. No production file
                      may use them.
B. payload-only-identity
                      a file that base64url-decodes a JWT payload segment and
                      reads `sub` / `user_id` / `email` must go through the
                      shared verifier (`botsecurity_core::verified_jwt`), which
                      pins HS256, rejects `alg: none`, verifies the HMAC in
                      constant time and requires a live `exp`. Sites that verify
                      inline before decoding are listed in
                      `scripts/jwt_hardening_allowlist.txt` with a reason.

Test files (`*_tests.rs`, `tests.rs`) are skipped: they assert on these very
strings and mint tokens on purpose.

Usage:
    python3 scripts/check_jwt_hardening.py --check
    python3 scripts/check_jwt_hardening.py --self-test

Exit codes: 0 = clean, 1 = violations, 2 = usage error.
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SCAN_ROOT = Path("botserver")
ALLOWLIST = Path("scripts/jwt_hardening_allowlist.txt")

DANGEROUS = ("jsonwebtoken::dangerous", "insecure_decode")
PAYLOAD_DECODERS = (
    "URL_SAFE_NO_PAD",
    "base64url_decode(",
    "base64_url_decode(",
    "decode_jwt_payload(",
)
IDENTITY_CLAIMS = ('get("sub")', 'get("user_id")', 'get("email")')
SHARED_VERIFIER = ("verify_hs256_jwt", "verify_saas_jwt")

RULES = ("dangerous-decode", "payload-only-identity")


def is_test_file(relative: Path) -> bool:
    name = relative.name
    return name == "tests.rs" or name.endswith("_tests.rs") or "tests" in relative.parts


def scan_text(relative: Path, text: str) -> list[str]:
    """Violations for one file, as `<rule>: <reason>` strings."""
    if is_test_file(relative):
        return []

    violations: list[str] = []

    for pattern in DANGEROUS:
        if pattern in text:
            violations.append(f"dangerous-decode: uses `{pattern}`")

    decodes_payload = any(marker in text for marker in PAYLOAD_DECODERS)
    reads_identity = any(marker in text for marker in IDENTITY_CLAIMS)
    verifies = any(marker in text for marker in SHARED_VERIFIER)
    if decodes_payload and reads_identity and not verifies:
        violations.append(
            "payload-only-identity: decodes a JWT payload and reads identity "
            "claims without calling the shared verifier"
        )

    return violations


def rust_files(root: Path) -> list[Path]:
    scan = root / SCAN_ROOT
    if not scan.is_dir():
        return []
    return sorted(p.relative_to(root) for p in scan.rglob("*.rs") if p.is_file())


def read_allowlist(root: Path) -> list[tuple[str, str]]:
    """`(path, reason)` entries; blank lines and `#` comments are ignored."""
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


def run_check(root: Path = ROOT) -> int:
    allowlist = read_allowlist(root)
    allowed = {path for path, _ in allowlist}
    used: set[str] = set()
    failures = 0
    for relative in rust_files(root):
        text = (root / relative).read_text(encoding="utf-8", errors="replace")
        for violation in scan_text(relative, text):
            if violation.startswith("payload-only-identity") and relative.as_posix() in allowed:
                used.add(relative.as_posix())
                continue
            print(f"{relative}: {violation}", file=sys.stderr)
            failures += 1

    # The allowlist is shrink-only: an entry that no longer suppresses a
    # violation means the file was fixed and the line must go.
    for path, _ in allowlist:
        if path not in used:
            print(f"{ALLOWLIST}: stale entry (no violation anymore): {path}", file=sys.stderr)
            failures += 1

    if failures:
        print(
            f"\n{failures} JWT hardening violation(s). Route the file through "
            "botsecurity_core::verified_jwt or justify it in "
            f"{ALLOWLIST}.",
            file=sys.stderr,
        )
        return 1
    print("jwt-hardening: clean")
    return 0


def self_test() -> int:
    """Prove both rules fire and that the allowlisted shape passes."""
    failures = 0

    dangerous = scan_text(Path("botserver/src/x.rs"), "let d = jsonwebtoken::dangerous::insecure_decode(t);")
    if not any(v.startswith("dangerous-decode") for v in dangerous):
        print("self-test: dangerous-decode did not fire", file=sys.stderr)
        failures += 1

    unverified = scan_text(
        Path("botserver/src/y.rs"),
        'let p = URL_SAFE_NO_PAD.decode(seg)?;\nlet sub = p.get("sub");',
    )
    if not any(v.startswith("payload-only-identity") for v in unverified):
        print("self-test: payload-only-identity did not fire", file=sys.stderr)
        failures += 1

    verified = scan_text(
        Path("botserver/src/z.rs"),
        'let c = verify_hs256_jwt(token, secret)?;\nlet sub = c.sub;',
    )
    if verified:
        print(f"self-test: shared verifier flagged: {verified}", file=sys.stderr)
        failures += 1

    test_file = scan_text(
        Path("botserver/src/z_tests.rs"),
        'assert!(!src.contains("jsonwebtoken::dangerous"));\nlet s = p.get("sub");',
    )
    if test_file:
        print(f"self-test: test file flagged: {test_file}", file=sys.stderr)
        failures += 1

    if failures:
        return 1
    print("jwt-hardening self-test: ok")
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
