#!/usr/bin/env python3
"""Fail when a security/crypto dependency is not pinned exactly (#2672, IMP-10).

Caret ranges let a patch release of a crypto primitive enter the build without
an explicit review. Every crate in the list below must use `version = "=x.y.z"`
in the workspace root manifest.

Usage: python3 scripts/check_security_dependency_pins.py [--manifest PATH]
"""

from __future__ import annotations

import argparse
import sys
import tomllib
from pathlib import Path

DEFAULT_MANIFEST = Path("Cargo.toml")
SECURITY_CRATES = (
    "aes-gcm",
    "argon2",
    "base64",
    "governor",
    "hex",
    "hmac",
    "jsonwebtoken",
    "native-tls",
    "rand",
    "rcgen",
    "ring",
    "rustls",
    "sha1",
    "sha2",
    "x509-parser",
)


def pinned_versions(manifest: Path) -> dict[str, str]:
    data = tomllib.loads(manifest.read_text(encoding="utf-8"))
    deps = data.get("workspace", {}).get("dependencies", {})
    result: dict[str, str] = {}
    for name in SECURITY_CRATES:
        spec = deps.get(name)
        if isinstance(spec, str):
            result[name] = spec
        elif isinstance(spec, dict):
            result[name] = str(spec.get("version", ""))
    return result


def problems(versions: dict[str, str]) -> list[str]:
    found = []
    for name in SECURITY_CRATES:
        value = versions.get(name)
        if not value:
            found.append(f"{name}: missing from workspace dependencies")
        elif not value.startswith("="):
            found.append(f"{name}: must be an exact pin, got '{value}'")
    return found


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", type=Path, default=DEFAULT_MANIFEST)
    args = parser.parse_args()
    issues = problems(pinned_versions(args.manifest))
    if issues:
        print("Security/crypto dependencies must be pinned exactly (IMP-10):")
        for issue in issues:
            print(f"  - {issue}")
        return 1
    print(f"All {len(SECURITY_CRATES)} security/crypto dependencies are exactly pinned")
    return 0


if __name__ == "__main__":
    sys.exit(main())
