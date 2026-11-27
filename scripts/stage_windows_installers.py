#!/usr/bin/env python3
"""Stage the complete Windows offline cache embedded in the gblite MSI.

Reads botserver/3rdparty.toml, downloads the Windows asset of every in-scope
`[components.*]` / `[models.*]` entry into botserver/botserver-installers/ and
writes SHA256SUMS-windows-installers. `--check` verifies the cache and its sums
without downloading, so CI can prove first boot needs no network.

Design choices (x86_64 MSI, CPU-only local inference): an entry with `url_win`
uses its `filename_win` / `sha256_win`; entries without one (platform-neutral
GGUF models, Windows-only llm_win_cpu_x64) use the generic triple. Excluded,
with reasons below: components with no Windows build, Linux/macOS and ARM64
llama.cpp variants, GPU-only assets and entries with unusable Windows pins.
CUDA runtimes are left out: with no CUDA engine shipped they add ~2 x 30 MB for
no benefit. An empty upstream digest is never invented: it is warned about and
recorded, as in check_component_checksums.py; a present digest is enforced.

Usage: python3 scripts/stage_windows_installers.py [--check] [--cache DIR]
Exit 0 = cache complete and verified, 1 = gap or integrity failure.
"""

from __future__ import annotations

import argparse
import hashlib
import os
import sys
import time
import tomllib
import urllib.error
import urllib.request
from dataclasses import dataclass
from pathlib import Path
from urllib.parse import urlsplit

ROOT = Path(__file__).resolve().parent.parent
DEFAULT_MANIFEST = ROOT / "botserver" / "3rdparty.toml"
DEFAULT_CACHE = ROOT / "botserver" / "botserver-installers"
SUMS_NAME = "SHA256SUMS-windows-installers"

# Key -> reason. These entries have no usable Windows build/pin for this MSI.
EXCLUDE = {
    "alm": "Forgejo publishes no Windows asset (manifest v17: Linux only)",
    "alm_ci": "Forgejo runner publishes no Windows asset (manifest: Linux only)",
    "table_editor": "no Windows build",
    "webmail": "Roundcube needs WSL/Docker on Windows (manifest note)",
    "libpq_win": "url_win (17.4) and filename_win (18.6) disagree; the workflow "
                 "stages the pinned 17.4 runtime DLLs and `tables` the 18.6 ZIP",
    "botmodels": "url_win carries an unresolved {tag} placeholder resolved at "
                 "install time; optional wizard-only component",
    "llm_win_cpu_arm64": "ARM64 build; this MSI targets x86_64",
    "llm_win_cuda12": "GPU-only llama.cpp build; Lite ships CPU-only inference",
    "llm_win_cuda13": "GPU-only llama.cpp build; Lite ships CPU-only inference",
    "llm_win_vulkan": "GPU-only llama.cpp build; Lite ships CPU-only inference",
    "cudart_win_cuda12": "CUDA runtime for an excluded GPU build (~30 MB, unused)",
    "cudart_win_cuda13": "CUDA runtime for an excluded GPU build (~30 MB, unused)",
}
# The generic `llm` entry is the Linux x64 CPU build; these prefixes are all
# non-Windows llama.cpp builds. The one Windows CPU build is kept explicitly.
NON_WINDOWS_LLAMA = ("llm_linux", "llm_macos")
LINUX_ONLY_NAMES = {"llm", "llm_linux_s390x"}

@dataclass(frozen=True)
class Asset:
    key: str
    filename: str
    url: str
    digest: str
    pinned: bool

def skip_reason(section: str, key: str) -> str | None:
    if key in EXCLUDE:
        return EXCLUDE[key]
    if section != "components":
        return None  # GGUF models are platform-neutral
    if key.startswith(NON_WINDOWS_LLAMA) or key in LINUX_ONLY_NAMES:
        return "Linux/macOS-only llama.cpp build"
    if key.endswith("_cached"):
        return "Linux bundled cache copy; no Windows build"
    return None

def select(manifest: Path) -> tuple[list[Asset], list[str], dict[str, str]]:
    """Resolve the Windows triple of every in-scope manifest entry."""
    with manifest.open("rb") as handle:
        document = tomllib.load(handle)
    assets: list[Asset] = []
    warnings: list[str] = []
    skipped: dict[str, str] = {}
    seen: dict[str, str] = {}
    for section in ("components", "models"):
        for key, entry in sorted(document.get(section, {}).items()):
            if not isinstance(entry, dict):
                continue
            reason = skip_reason(section, key)
            if reason:
                skipped[key] = reason
                continue
            win = section == "components" and bool(entry.get("url_win"))
            url = entry.get("url_win") if win else entry.get("url")
            filename = (entry.get("filename_win") if win else entry.get("filename"))
            digest = (entry.get("sha256_win", "") if win else entry.get("sha256", ""))
            if not isinstance(url, str) or not url.startswith("https://"):
                raise ValueError(f"{section}.{key}: missing or non-https Windows URL")
            if not isinstance(filename, str) or not filename or Path(filename).name != filename:
                filename = Path(urlsplit(url).path).name
            if not filename or ".." in filename:
                raise ValueError(f"{section}.{key}: unsafe filename")
            digest = str(digest).strip().lower()
            pinned = bool(digest)
            if pinned and (len(digest) != 64 or any(c not in "0123456789abcdef" for c in digest)):
                raise ValueError(f"{section}.{key}: invalid SHA-256 pin {digest!r}")
            if not pinned:
                warnings.append(f"{section}.{key} ({filename}): empty upstream SHA-256; recorded unpinned")
            if filename in seen and seen[filename] != url:
                raise ValueError(f"filename collision for {filename}: {seen[filename]} vs {url}")
            seen[filename] = url
            assets.append(Asset(key, filename, url, digest, pinned))

    # Non-vacuous coverage: no in-scope entry with a download URL may be lost.
    missing = [
        f"{section}.{key}"
        for section in ("components", "models")
        for key, entry in sorted(document.get(section, {}).items())
        if isinstance(entry, dict)
        and not skip_reason(section, key)
        and (entry.get("url_win") or entry.get("url"))
        and key not in {a.key for a in assets}
    ]
    if missing:
        raise ValueError(f"in-scope entries not staged: {', '.join(missing)}")
    if not assets:
        raise ValueError("no Windows assets selected; check the manifest and exclusions")
    return assets, warnings, skipped

def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()

def stage(asset: Asset, cache: Path, retries: int, timeout: int) -> str:
    """Download one asset with retries; enforce its pin when one exists."""
    target = cache / asset.filename
    if target.is_file() and (not asset.pinned or sha256(target) == asset.digest):
        return "cached"
    part = target.with_name(target.name + ".part")
    last: Exception | None = None
    for attempt in range(1, retries + 1):
        try:
            request = urllib.request.Request(
                asset.url, headers={"User-Agent": "generalbots-windows-stager/1"}
            )
            with urllib.request.urlopen(request, timeout=timeout) as response, part.open("wb") as out:
                while True:
                    chunk = response.read(1 << 20)
                    if not chunk:
                        break
                    out.write(chunk)
            if asset.pinned and sha256(part) != asset.digest:
                raise ValueError(f"SHA-256 mismatch; manifest {asset.digest}, download {sha256(part)}")
            os.replace(part, target)
            return "downloaded"
        except (OSError, urllib.error.URLError, ValueError) as error:
            last = error
            part.unlink(missing_ok=True)
            if attempt < retries:
                time.sleep(min(2 ** attempt, 10))
    raise RuntimeError(f"failed to stage {asset.filename} after {retries} attempts: {last}")

def write_sums(cache: Path) -> None:
    lines = [
        f"{sha256(path)}  {path.name}"
        for path in sorted(cache.iterdir())
        if path.is_file() and path.name != SUMS_NAME
    ]
    (cache / SUMS_NAME).write_text("\n".join(lines) + "\n", encoding="ascii")

def check(cache: Path, assets: list[Asset]) -> int:
    """Verify cached files, manifest pins and SHA256SUMS without downloading."""
    errors: list[str] = []
    files = {p.name for p in cache.iterdir() if p.is_file()} if cache.is_dir() else set()
    for asset in assets:
        if asset.filename not in files:
            errors.append(f"missing asset: {asset.filename} ({asset.key})")
        elif asset.pinned and sha256(cache / asset.filename) != asset.digest:
            errors.append(f"SHA-256 mismatch: {asset.filename} ({asset.key})")
    sums = cache / SUMS_NAME
    if SUMS_NAME not in files:
        errors.append(f"missing checksum file: {SUMS_NAME}")
    else:
        listed: dict[str, str] = {}
        for line in sums.read_text(encoding="ascii").splitlines():
            try:
                digest, name = line.split("  ", 1)
            except ValueError:
                errors.append(f"malformed checksum line: {line!r}")
                continue
            listed[name] = digest
        errors.extend(f"sum lists a missing file: {n}" for n in sorted(set(listed) - files))
        errors.extend(f"file absent from {SUMS_NAME}: {n}" for n in sorted(files - set(listed) - {SUMS_NAME}))
        for name, digest in listed.items():
            if name in files and sha256(cache / name) != digest:
                errors.append(f"checksum mismatch in {SUMS_NAME}: {name}")
    if errors:
        print(f"CHECK FAILED: {len(errors)} problem(s) in {cache}")
        for error in errors:
            print(f"  - {error}")
        return 1
    print(f"CHECK OK: {len(assets)} assets and {SUMS_NAME} verified in {cache}")
    return 0

def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, default=DEFAULT_MANIFEST)
    parser.add_argument("--cache", type=Path, default=DEFAULT_CACHE)
    parser.add_argument("--check", action="store_true", help="verify only; never download")
    parser.add_argument("--retries", type=int, default=3)
    parser.add_argument("--timeout", type=int, default=120)
    args = parser.parse_args()
    try:
        assets, warnings, skipped = select(args.manifest)
    except (OSError, UnicodeError, tomllib.TOMLDecodeError, ValueError) as error:
        print(f"error: cannot select Windows assets: {error}")
        return 1

    if args.check:
        return check(args.cache, assets)

    try:
        args.cache.mkdir(parents=True, exist_ok=True)
        statuses = {asset.filename: stage(asset, args.cache, args.retries, args.timeout) for asset in assets}
        write_sums(args.cache)
    except (OSError, ValueError, RuntimeError) as error:
        print(f"error: staging failed: {error}")
        return 1

    total = sum((args.cache / a.filename).stat().st_size for a in assets)
    print(f"Windows offline payload: {len(assets)} assets, {total / 1e9:.2f} GB -> {args.cache}")
    for asset in assets:
        size = (args.cache / asset.filename).stat().st_size / 1e6
        print(f"  {statuses[asset.filename]:<10} {size:9.1f} MB  {asset.filename}  {asset.digest[:12] or 'UNPINNED'}")
    for warning in warnings:
        print(f"WARN {warning}")
    print(f"Excluded {len(skipped)} manifest entries (no Windows build, non-x64 or GPU-only)\nWrote {args.cache / SUMS_NAME}")
    return 0

if __name__ == "__main__":
    sys.exit(main())
