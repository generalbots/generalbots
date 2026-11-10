#!/usr/bin/env python3
"""Single SaaS JWT secret resolver guard (issue #2140).

The SaaS JWT secret decides who is authenticated everywhere: cloud login mints
tokens with it and the settings, SSO, Vibe-domain, browser-policy, docs/slides
and email paths verify them. Two crates used to re-implement the lookup with a
repository-visible fallback, so a token signed with
`dev-secret-key-change-in-production-minimum-32-chars` was accepted by those
verifiers even after the login path had generated a random secret.

The gate makes the invariant mechanical:

A. single-resolver     only `botserver/crates/botsecurity-core/src/jwt_secret.rs`
                       may define `resolve_saas_jwt_secret`.
B. no-wrapper-resolver no file outside that module may define a
                       `*saas_jwt_secret` helper (the `ProjectDomains` and
                       `get_saas_jwt_secret` copies this issue removed).
C. no-hardcoded-default the placeholder literals may appear only in the
                       canonical module (as `DEV_FALLBACK_SECRET`) and in tests
                       or documentation.
D. no-second-env-read  `SAAS_JWT_SECRET` / `JWT_SECRET` may be read from the
                       environment only by the canonical module (the encryption
                       master key is the single documented exception).
E. consumers-share-it  `botserver/src`, `botsettings` and `botvibe` must call
                       the shared resolver/verifier and define no local copy.

Test files (`*_tests.rs`, `tests.rs`, a `tests/` path) are skipped: they mint
tokens with known secrets on purpose.

Usage:
    python3 scripts/check_jwt_secret.py --check
    python3 scripts/check_jwt_secret.py --self-test

Environment:
    GB_JWT_SECRET_ROOT   scan this tree instead of the repository root
                         (used by scripts/gate_selftests.py).

Exit codes: 0 = clean, 1 = violations, 2 = usage error.
"""

from __future__ import annotations

import argparse
import os
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SCAN_DIRS = ("botserver", "bottest", "botlib")

CANONICAL = "botserver/crates/botsecurity-core/src/jwt_secret.rs"
RESOLVER_DEF = re.compile(r"\bfn\s+resolve_saas_jwt_secret\b")
WRAPPER_DEF = re.compile(r"\bfn\s+\w*saas_jwt_secret\s*\(")
PLACEHOLDERS = ("dev-secret-key-change-in-production", "change-me-in-production")
ENV_READS = ('var("SAAS_JWT_SECRET")', 'var("JWT_SECRET")')
CONFIG_KEY_READ = 'get("saas_jwt_secret")'
SHARED_RESOLVER = ("verified_jwt::resolve_saas_jwt_secret", "verified_jwt::verify_saas_jwt")
CONSUMERS = {
    "botserver/src": "botserver routes",
    "botserver/crates/botsettings": "settings/2FA API",
    "botserver/crates/botvibe": "Vibe domain auth",
}
# The message-encryption master key derives from JWT_SECRET as a last resort;
# it is not a token verifier, so it may keep reading the variable.
ENV_ALLOWLIST = {"botserver/crates/botsecurity-crypto/src/encryption/master_key_lock.rs"}

RULES = (
    "single-resolver",
    "no-wrapper-resolver",
    "no-hardcoded-default",
    "no-second-env-read",
    "consumers-share-it",
)


def root() -> Path:
    override = os.environ.get("GB_JWT_SECRET_ROOT")
    return Path(override).resolve() if override else ROOT


def is_test_file(relative: Path) -> bool:
    name = relative.name
    return name == "tests.rs" or name.endswith("_tests.rs") or "tests" in relative.parts


def rust_files(base: Path) -> list[Path]:
    files: list[Path] = []
    for directory in SCAN_DIRS:
        scan = base / directory
        if not scan.is_dir():
            continue
        files.extend(
            path.relative_to(base) for path in scan.rglob("*.rs") if path.is_file()
        )
    return sorted(files)


def scan_text(relative: Path, text: str) -> list[str]:
    """Violations for one file, as `<rule>: <reason>` strings."""
    posix = relative.as_posix()
    canonical = posix == CANONICAL
    test_file = is_test_file(relative)
    violations: list[str] = []

    if RESOLVER_DEF.search(text) and not canonical:
        violations.append(
            "single-resolver: defines resolve_saas_jwt_secret; call "
            f"{CANONICAL} instead"
        )

    if not canonical and not test_file and WRAPPER_DEF.search(text):
        violations.append(
            "no-wrapper-resolver: defines a local *saas_jwt_secret helper "
            "instead of calling botsecurity_core::verified_jwt"
        )

    if not canonical and not test_file and posix.endswith(".rs"):
        for literal in PLACEHOLDERS:
            if literal in text:
                violations.append(
                    f"no-hardcoded-default: contains the placeholder literal `{literal}`"
                )

    if not canonical and not test_file and posix not in ENV_ALLOWLIST:
        for pattern in ENV_READS:
            if pattern in text:
                violations.append(
                    f"no-second-env-read: reads {pattern} directly; the only reader is "
                    f"{CANONICAL}"
                )

    if not canonical and not test_file and CONFIG_KEY_READ in text:
        violations.append(
            "single-resolver: reads the saas_jwt_secret config key directly; the "
            f"only reader is {CANONICAL}"
        )

    return violations


def consumer_violations(base: Path, files: list[Path]) -> list[str]:
    """Each consumer crate must reach the shared resolver, and only it."""
    violations: list[str] = []
    for prefix, label in CONSUMERS.items():
        owned = [path for path in files if path.as_posix().startswith(prefix)]
        if not owned:
            continue
        shared = any(
            marker in (base / path).read_text(encoding="utf-8", errors="replace")
            for path in owned
            for marker in SHARED_RESOLVER
        )
        if not shared:
            violations.append(
                f"consumers-share-it: {label} ({prefix}) does not call "
                "botsecurity_core::verified_jwt::{resolve_saas_jwt_secret, verify_saas_jwt}"
            )
    return violations


def run_check(base: Path | None = None) -> int:
    base = base or root()
    files = rust_files(base)
    if not files:
        print(f"jwt-secret: no Rust sources under {base}; nothing to check", file=sys.stderr)
        return 2

    failures = 0
    for relative in files:
        text = (base / relative).read_text(encoding="utf-8", errors="replace")
        for violation in scan_text(relative, text):
            print(f"{relative}: {violation}", file=sys.stderr)
            failures += 1

    for violation in consumer_violations(base, files):
        print(violation, file=sys.stderr)
        failures += 1

    if failures:
        print(
            f"\n{failures} SaaS JWT secret violation(s). The only resolver is "
            f"{CANONICAL}.",
            file=sys.stderr,
        )
        return 1
    print("jwt-secret: clean")
    return 0


def self_test() -> int:
    """Prove every rule fires and that the canonical module passes."""
    failures = 0

    def expect(rule: str, path: str, text: str, should_fire: bool = True) -> None:
        nonlocal failures
        fired = any(v.startswith(rule) for v in scan_text(Path(path), text))
        if fired != should_fire:
            verb = "did not fire" if should_fire else "fired"
            print(f"self-test: {rule} {verb} on {path}", file=sys.stderr)
            failures += 1

    expect("single-resolver", "botserver/crates/botvibe/src/x.rs", "pub fn resolve_saas_jwt_secret() -> String {\n    1\n}")
    expect("no-wrapper-resolver", "botserver/crates/botvibe/src/x.rs", "pub fn saas_jwt_secret() -> String { String::new() }")
    expect("no-hardcoded-default", "botserver/src/x.rs", 'let s = "dev-secret-key-change-in-production-minimum-32-chars";')
    expect("no-second-env-read", "botserver/src/x.rs", 'let s = std::env::var("SAAS_JWT_SECRET");')
    expect("single-resolver", "botserver/src/x.rs", 'json.get("saas_jwt_secret")')

    canonical = (
        "pub const DEV_FALLBACK_SECRET: &str = \"dev-secret-key-change-in-production-minimum-32-chars\";\n"
        'pub fn resolve_saas_jwt_secret() -> Result<String, JwtSecretError> { let _ = std::env::var("JWT_SECRET"); Ok(String::new()) }\n'
    )
    expect("single-resolver", CANONICAL, canonical, should_fire=False)
    expect("no-hardcoded-default", CANONICAL, canonical, should_fire=False)

    test_file = 'let s = std::env::var("JWT_SECRET"); // dev-secret-key-change-in-production'
    expect("no-hardcoded-default", "botserver/crates/x/src/y_tests.rs", test_file, should_fire=False)
    expect("no-second-env-read", "botserver/crates/x/src/y_tests.rs", test_file, should_fire=False)

    allowlisted = 'if let Ok(jwt) = std::env::var("JWT_SECRET") { }'
    expect(
        "no-second-env-read",
        "botserver/crates/botsecurity-crypto/src/encryption/master_key_lock.rs",
        allowlisted,
        should_fire=False,
    )

    if failures:
        return 1
    print("jwt-secret self-test: ok (every rule fails closed)")
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
