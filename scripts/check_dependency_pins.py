#!/usr/bin/env python3
"""IMP-10 — fail when a security/crypto dependency is not exactly pinned.

AGENTS.md requires critical deps (crypto, security) to use exact versions
(`=1.0.1`); a caret version or a bare version allows a silent upgrade to an
unreviewed release. `Cargo.lock` is not a substitute: it can be regenerated.

The set below is the security/crypto/rate-limiting surface of the root
workspace manifest (#2673). Adding a new crate to the set is a one-line change
here; removing a crate requires removing its pin from Cargo.toml too, which
this gate then reports as a missing pin.

Usage:
    python3 scripts/check_dependency_pins.py [--manifest PATH] [--self-test]

Exit codes: 0 all pinned, 1 violation(s) found.
"""

from __future__ import annotations

import argparse
import re
import sys
import tomllib
from pathlib import Path

DEFAULT_MANIFEST = Path("Cargo.toml")
CRATES = (
    "jsonwebtoken", "aes-gcm", "argon2", "hmac", "sha2",
    "ring", "rustls", "native-tls", "governor",
)


def check(manifest: Path) -> list[str]:
    try:
        with manifest.open("rb") as handle:
            data = tomllib.load(handle)
    except (OSError, tomllib.TOMLDecodeError) as error:
        return [f"cannot read {manifest}: {error}"]
    workspace = data.get("workspace", {}).get("dependencies", {})
    bad: list[str] = []
    for name in CRATES:
        entry = workspace.get(name)
        if entry is None:
            bad.append(f"{name}: not present in [workspace.dependencies]")
            continue
        version = entry if isinstance(entry, str) else entry.get("version", "")
        if not isinstance(version, str) or not version.startswith("="):
            bad.append(f"{name}: version {version!r} is not an exact '=' pin")
    return bad


def self_test() -> int:
    import tempfile

    tmp = Path(tempfile.mkdtemp())
    ok = tmp / "ok.toml"
    ok.write_text(
        '[workspace.dependencies]\n' +
        "".join(f'{c} = "=1.2.3"\n' for c in CRATES),
        encoding="utf-8",
    )
    bad = tmp / "bad.toml"
    bad.write_text(
        '[workspace.dependencies]\n' +
        "".join(f'{c} = "1.2"\n' for c in CRATES),
        encoding="utf-8",
    )
    assert check(ok) == [], check(ok)
    assert len(check(bad)) == len(CRATES), check(bad)
    print(f"self-test ok: {len(CRATES)} pinned crates pass, unpinned ones fail")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, default=DEFAULT_MANIFEST)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        return self_test()
    bad = check(args.manifest)
    if bad:
        for line in bad:
            print(f"error: {line}")
        print(f"\n{len(bad)} security/crypto dependency pin violation(s)")
        return 1
    print(f"ok: {len(CRATES)} security/crypto dependencies are exactly pinned")
    return 0


if __name__ == "__main__":
    sys.exit(main())
