#!/usr/bin/env python3
"""Fail when a bundled runtime component or model has no pinned SHA-256 (#2001, #2708).

`botserver/3rdparty.toml` is the provenance record for every binary the
installer downloads or unpacks: version + URL + SHA-256. The installer now
enforces the hash (and refuses an unhashed asset under
`GB_REQUIRE_CHECKSUMS=1`), so an entry without a hash is an unverified binary
in the shipping payload. #2708 extends the same rule to `[models.*]`: those
GGUF files are fetched into data/llm and loaded by llama-server, so an empty
pin there is an unverified model. Model gaps are not allowlisted — they fail
the gate unconditionally.

The component gate is a ratchet: the gaps that exist today are listed, with a
reason, in `scripts/component_checksums_allowlist.txt`. A new component without
a hash fails CI; removing a hash from a component that had one fails CI too.
Closing a gap means adding the hash and deleting the line.

For vendored JavaScript (#2742), pass --js-manifest: every pinned asset is
re-hashed against the file on disk, and any unlisted *.min.js in the vendor
tree fails. That is a direct tamper seal, not an allowlist: the manifest itself
is the source of truth.

Usage:
    python3 scripts/check_component_checksums.py [--toml PATH] [--allowlist PATH]
    python3 scripts/check_component_checksums.py --js-manifest PATH [--vendor-dir PATH]

Exit codes: 0 clean, 1 otherwise.
"""

from __future__ import annotations

import argparse
import hashlib
import sys
import tomllib
from pathlib import Path

DEFAULT_TOML = Path("botserver/3rdparty.toml")
DEFAULT_ALLOWLIST = Path("scripts/component_checksums_allowlist.txt")
DEFAULT_VENDOR_DIR = Path("botui/ui/suite/js/vendor")

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
    """(name, what-is-missing) for every unhashed asset that can install.

    Works for both `[components.*]` and `[models.*]` (#2708)."""
    found: list[tuple[str, str]] = []
    # An asset with no download URL cannot be fetched at all, so a missing
    # hash is not an unverified download.
    for name, entry in sorted(components.items()):
        if entry.get("url") and not str(entry.get("sha256", "")).strip():
            found.append((name, "url"))
        if entry.get("url_win") and not str(entry.get("sha256_win", "")).strip():
            found.append((name, "url_win"))
        if entry.get("url_arm64") and not str(entry.get("sha256_arm64", "")).strip():
            found.append((name, "url_arm64"))
    return found


def insecure_urls(components: dict) -> list[tuple[str, str]]:
    """(name, url-field) for any download URL that is not HTTPS (#2611, #2708).

    A plaintext URL lets an on-path attacker replace the archive before any
    checksum is computed, so the hash (when one exists) is no longer a
    provenance guarantee. This is an unconditional failure — no allowlist.
    """
    found: list[tuple[str, str]] = []
    for name, entry in sorted(components.items()):
        for field in ("url", "url_win", "url_arm64"):
            value = entry.get(field)
            if isinstance(value, str) and value and not value.startswith("https://"):
                found.append((name, field))
    return found


def verify_vendored_js(manifest_path: Path, vendor_dir: Path) -> tuple[list[str], int]:
    """Verify copied JS assets against a pinned SHA-256 manifest (#2742).

    A changed file detects tampering; a new unlisted `*.min.js` stops an
    unaudited bundle from being dropped into the served tree. Both fail closed.
    Returns (errors, number-of-pinned-assets).
    """
    errors: list[str] = []
    try:
        with manifest_path.open("rb") as handle:
            assets = tomllib.load(handle).get("assets", {})
    except (OSError, tomllib.TOMLDecodeError) as error:
        return [f"error: cannot read JS manifest {manifest_path}: {error}"], 0

    if not assets:
        return [f"error: {manifest_path} has no [assets.*] entries"], 0

    pinned: set[str] = set()
    for key, entry in sorted(assets.items()):
        path = str(entry.get("path", "")).strip()
        version = str(entry.get("version", "")).strip()
        url = str(entry.get("url", "")).strip()
        digest = str(entry.get("sha256", "")).strip().lower()
        if not path or not version or not url or not digest:
            errors.append(f"error: JS asset '{key}' needs path, version, url and sha256")
            continue
        rel = Path(path)
        if rel.is_absolute() or ".." in rel.parts:
            errors.append(f"error: JS asset '{key}' has an unsafe path '{path}'")
            continue
        if not url.startswith("https://"):
            errors.append(
                f"error: JS asset '{key}' has a plaintext source URL; use https://"
            )
        pinned.add(path)
        target = vendor_dir / rel
        if not target.is_file():
            errors.append(f"error: JS asset '{key}' file is missing: {path}")
            continue
        actual = hashlib.sha256(target.read_bytes()).hexdigest()
        if actual != digest:
            errors.append(
                f"error: JS asset '{key}' SHA-256 mismatch for {path}: "
                f"manifest {digest} != on-disk {actual}"
            )

    if vendor_dir.is_dir():
        for target in sorted(vendor_dir.rglob("*.min.js")):
            rel = target.relative_to(vendor_dir).as_posix()
            if rel not in pinned:
                errors.append(
                    f"error: untracked vendored bundle {rel}; add it to {manifest_path}"
                )

    return errors, len(assets)


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
    parser.add_argument(
        "--js-manifest",
        type=Path,
        help="verify vendored JS assets against this SHA-256 manifest (#2742)",
    )
    parser.add_argument(
        "--vendor-dir",
        type=Path,
        default=DEFAULT_VENDOR_DIR,
        help=f"root of the vendored JS tree (default: {DEFAULT_VENDOR_DIR})",
    )
    args = parser.parse_args()

    if args.self_test:
        from gate_selftests import run_self_test

        return run_self_test("component-checksums")

    if args.js_manifest is not None:
        errors, count = verify_vendored_js(args.js_manifest, args.vendor_dir)
        for error in errors:
            print(error)
        if errors:
            print(f"\n{len(errors)} vendored JS integrity error(s)")
            return 1
        print(f"ok: {count} vendored JS asset(s) match {args.js_manifest}")
        return 0

    if not args.toml.is_file():
        print(f"error: {args.toml} not found")
        return 1
    with args.toml.open("rb") as handle:
        document = tomllib.load(handle)
    components = document.get("components", {})
    models = document.get("models", {})

    # #2708 — a model URL is as much a supply-chain entry point as a component
    # URL: llama-server loads the downloaded GGUF. Check both tables.
    insecure = insecure_urls(components) + insecure_urls(models)
    if insecure:
        for name, field in insecure:
            print(
                f"error: asset '{name}' uses a plaintext {field} in {args.toml}; "
                "every download URL must start with https:// (#2611)"
            )
        print(f"\n{len(insecure)} insecure download URL(s)")
        return 1

    found = gaps(components)
    model_gaps = gaps(models)
    if args.list:
        for name, variant in found:
            print(f"{name}\t{variant}")
        for name, variant in model_gaps:
            print(f"model:{name}\t{variant}")
        return 0

    # Models are deliberately outside the ratchet allowlist: an empty model
    # hash must fail the gate even when a reason is written for a component.
    if model_gaps:
        for name, variant in model_gaps:
            print(
                f"error: model '{name}' has no SHA-256 for {variant}; "
                "add the hash to botserver/3rdparty.toml — unverified GGUF files "
                "must never reach llama-server (#2708)"
            )
        print(f"\n{len(model_gaps)} unhashed model(s)")
        return 1

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
        f"{len(components)} components and {len(models)} models"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
