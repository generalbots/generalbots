#!/usr/bin/env python3
"""Fail when a bundled runtime component has no pinned SHA-256 (#2001).

`botserver/3rdparty.toml` is the provenance record for every binary the
installer downloads or unpacks: version + URL + SHA-256. The installer now
enforces the hash (and refuses an unhashed asset under
`GB_REQUIRE_CHECKSUMS=1`), so an entry without a hash is an unverified binary
in the shipping payload.

The gate is a ratchet: the gaps that exist today are listed, with a reason, in
`scripts/component_checksums_allowlist.txt`. A new component without a hash
fails CI; removing a hash from a component that had one fails CI too. Closing a
gap means adding the hash and deleting the line.

Usage:
    python3 scripts/check_component_checksums.py [--toml PATH] [--allowlist PATH]

Exit codes: 0 every gap is acknowledged, 1 otherwise.
"""

from __future__ import annotations

import argparse
import sys
import tomllib
from pathlib import Path

DEFAULT_TOML = Path("botserver/3rdparty.toml")
DEFAULT_ALLOWLIST = Path("scripts/component_checksums_allowlist.txt")


def load_allowlist(path: Path) -> dict[str, str]:
    """`component  reason` per line; `#` starts a comment."""
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


def gaps(components: dict) -> list[tuple[str, str]]:
    """(component, what-is-missing) for every unhashed asset that can install."""
    found: list[tuple[str, str]] = []
    for name, entry in sorted(components.items()):
        if not str(entry.get("sha256", "")).strip():
            found.append((name, "url"))
        if entry.get("url_win") and not str(entry.get("sha256_win", "")).strip():
            found.append((name, "url_win"))
        if entry.get("url_arm64") and not str(entry.get("sha256_arm64", "")).strip():
            found.append((name, "url_arm64"))
    return found


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--toml", type=Path, default=DEFAULT_TOML)
    parser.add_argument("--allowlist", type=Path, default=DEFAULT_ALLOWLIST)
    parser.add_argument(
        "--list",
        action="store_true",
        help="print the gaps and exit 0 (used to refresh the allowlist)",
    )
    parser.add_argument("--self-test", action="store_true", help="prove the gate fails closed")
    args = parser.parse_args()

    if args.self_test:
        from gate_selftests import run_self_test

        return run_self_test("component-checksums")

    if not args.toml.is_file():
        print(f"error: {args.toml} not found")
        return 1
    with args.toml.open("rb") as handle:
        components = tomllib.load(handle).get("components", {})

    found = gaps(components)
    if args.list:
        for name, variant in found:
            print(f"{name}\t{variant}")
        return 0

    allowed = load_allowlist(args.allowlist)
    unacknowledged = [
        (name, variant)
        for name, variant in found
        if name not in allowed and "all-variants" not in allowed.get(name, "")
    ]
    stale = sorted(set(allowed) - {name for name, _ in found})

    if stale:
        print(
            "note: allowlist entries with no gap any more (delete them): "
            + ", ".join(stale)
        )
    if unacknowledged:
        for name, variant in unacknowledged:
            print(
                f"error: component '{name}' has no SHA-256 for {variant}; "
                "add the hash to botserver/3rdparty.toml or acknowledge the gap "
                f"in {args.allowlist} with a reason"
            )
        print(f"\n{len(unacknowledged)} unacknowledged checksum gap(s)")
        return 1

    print(
        f"ok: {len(found)} acknowledged checksum gap(s) across "
        f"{len(components)} components"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
