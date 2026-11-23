#!/usr/bin/env python3
"""MinIO provenance gate for #2612 (CVE-2025-62506).

The upstream release that fixes the CVE publishes no binary assets, so the
install pin is the immutable source archive rather than a downloadable
executable. This module holds the constants and checks specialised to that
exception; `check_component_checksums.py` owns the generic component gate.

The archive digest was obtained by fetching the GitHub tag archive twice; both
copies were byte-identical. No binary digest is fabricated here: the compiled
binary's digest depends on the Go toolchain and is written by
`build-minio-fixed.sh` to a `minio.sha256` sidecar at build time.
"""

from __future__ import annotations

import hashlib
import subprocess
from pathlib import Path

MINIO_FIXED_TAG = "RELEASE.2025-10-15T17-29-55Z"
MINIO_FIXED_COMMIT = "9e49d5e7a648f00e26f2246f4dc28e6b07f8c84a"
MINIO_SOURCE_SHA = "be6d0bd3696c3a13a35f02d3a0280b64319c67918b4501c5c3d87f96d000085c"
MINIO_CVE = "CVE-2025-62506"
VULNERABLE_MINIO_TAGS = ("RELEASE.2025-09-07T16-13-09Z",)


def is_sha256(value: str) -> bool:
    """True only for a 64-hex SHA-256 pin."""
    value = value.strip().lower()
    return len(value) == 64 and all(char in "0123456789abcdef" for char in value)


def minio_problems(components: dict) -> list[str]:
    """Fail closed on a vulnerable or unverifiable MinIO pin.

    The CVE fix has no upstream binary, so the only safe install is the binary
    built from the pinned source and shipped in the offline cache. Guard all
    three things that could silently put the vulnerable binary back: a
    download URL, a stale source pin, or a fabricated source digest.
    """
    found: list[str] = []
    for name, entry in sorted(components.items()):
        for field in ("url", "url_win", "url_arm64"):
            value = str(entry.get(field, "") or "")
            if any(tag in value for tag in VULNERABLE_MINIO_TAGS):
                found.append(
                    f"error: {name}.{field} points at a MinIO release affected by "
                    f"{MINIO_CVE}; remove the download URL and use the offline cache"
                )

    # A partial manifest (a self-test fixture, or a build without the drive
    # component) has no drive entry; there is then no MinIO install to guard.
    drive = components.get("drive")
    if not isinstance(drive, dict):
        return found

    for field in ("url", "url_win", "url_arm64"):
        value = str(drive.get(field, "") or "").strip()
        if value:
            found.append(
                f"error: drive.{field} = '{value}'; #2612 requires an empty URL "
                "because upstream publishes no fixed binary"
            )

    expected = {
        "source_tag": MINIO_FIXED_TAG,
        "source_commit": MINIO_FIXED_COMMIT,
        "source_sha256": MINIO_SOURCE_SHA,
        "cve_fixed": MINIO_CVE,
    }
    for field, want in expected.items():
        actual = str(drive.get(field, "") or "").strip().lower()
        if actual != want.lower():
            found.append(
                f"error: drive.{field} = '{actual}', expected '{want}' "
                f"(the pinned source of the {MINIO_CVE} fix)"
            )

    source_url = str(drive.get("source_url", "") or "").strip()
    if not source_url.startswith("https://") or MINIO_FIXED_TAG not in source_url:
        found.append(
            "error: drive.source_url must be the HTTPS GitHub archive of "
            f"{MINIO_FIXED_TAG}"
        )
    if not is_sha256(str(drive.get("source_sha256", "") or "")):
        found.append("error: drive.source_sha256 must be a 64-hex SHA-256")

    build_script = str(drive.get("build_script", "") or "").strip()
    if not build_script:
        found.append("error: drive.build_script must name the source-build script")

    for field in ("sha256", "sha256_win", "sha256_arm64"):
        value = str(drive.get(field, "") or "").strip()
        if value and not is_sha256(value):
            found.append(f"error: drive.{field} is not a 64-hex SHA-256")
    return found


def verify_minio_binary(binary: Path) -> list[str]:
    """Check the cached binary's build-time sidecar and its reported version."""
    errors: list[str] = []
    if not binary.is_file():
        return [f"error: cached MinIO binary is missing: {binary}"]

    sidecar = binary.with_name(f"{binary.name}.sha256")
    if not sidecar.is_file():
        return [f"error: missing {sidecar}; rerun scripts/build-minio-fixed.sh"]
    expected = sidecar.read_text(encoding="utf-8").strip().lower()
    if not is_sha256(expected):
        return [f"error: {sidecar} does not contain a 64-hex SHA-256"]

    digest = hashlib.sha256()
    with binary.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    actual = digest.hexdigest()
    if actual != expected:
        errors.append(
            f"error: {binary} SHA-256 mismatch: sidecar {expected}, on-disk {actual}"
        )

    try:
        result = subprocess.run(
            [str(binary), "--version"],
            capture_output=True,
            text=True,
            timeout=30,
            check=False,
        )
        version = f"{result.stdout}\n{result.stderr}"
    except (OSError, subprocess.SubprocessError) as error:
        return errors + [f"error: cannot run {binary} --version: {error}"]
    if result.returncode != 0:
        errors.append(f"error: {binary} --version exited {result.returncode}")
    if MINIO_FIXED_TAG not in version or MINIO_FIXED_COMMIT[:12] not in version:
        errors.append(
            f"error: {binary} does not report {MINIO_FIXED_TAG} / "
            f"{MINIO_FIXED_COMMIT[:12]} ({MINIO_CVE} fix); rebuild from source"
        )
    return errors
