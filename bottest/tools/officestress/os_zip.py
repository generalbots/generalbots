#!/usr/bin/env python3
"""Deterministic ZIP packaging for the ``officestress`` corpus (#2531).

ZIP output is byte-for-byte stable across Python versions and operating
systems because:

* every member is ``ZIP_STORED`` (deflate output is zlib-version dependent);
* every ``ZipInfo`` carries a fixed DOS-epoch timestamp and fixed creator
  fields (``create_system=0`` avoids the Unix/Windows host bit);
* member order is fixed by the caller and no extra/comment fields are added.

A seeded filler member inflates the archive to an exact target byte count.
"""

from __future__ import annotations

import hashlib
import random
import zipfile
from pathlib import Path

CHUNK = 1 << 20
ZIP_EPOCH = (1980, 1, 1, 0, 0, 0)
FILLER_MEMBER = "corpus/filler.bin"
FILLER_SALT = 0x5EED


def zip_overhead(names: tuple[str, ...]) -> int:
    """Local headers + central directory + EOCD for stored members."""
    return sum(76 + 2 * len(name.encode("ascii")) for name in names) + 22


def sha256_file(path: Path) -> str:
    """Stream a file through SHA-256 and return its hex digest."""
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(CHUNK), b""):
            digest.update(chunk)
    return digest.hexdigest()


def stored_info(name: str) -> zipfile.ZipInfo:
    """A ZipInfo whose bytes are independent of host OS, clock and locale."""
    info = zipfile.ZipInfo(name, date_time=ZIP_EPOCH)
    info.compress_type = zipfile.ZIP_STORED
    info.external_attr = 0o644 << 16
    info.create_system = 0
    info.create_version = 20
    info.extract_version = 20
    return info


def filler_bytes(seed: int, length: int) -> bytes:
    """Deterministic pseudo-random filler for a requested length."""
    rng = random.Random(seed ^ FILLER_SALT)
    return rng.randbytes(length)


def write_archive(
    path: Path,
    parts: tuple[tuple[str, bytes], ...],
    filler_length: int,
    seed: int,
) -> None:
    """Write parts plus a seeded filler member to ``path`` (stored ZIP)."""
    with zipfile.ZipFile(path, "w", compression=zipfile.ZIP_STORED) as archive:
        for name, payload in parts:
            archive.writestr(stored_info(name), payload)
        with archive.open(stored_info(FILLER_MEMBER), "w") as sink:
            blob = filler_bytes(seed, filler_length)
            for offset in range(0, len(blob), CHUNK):
                sink.write(blob[offset : offset + CHUNK])


def build_archive(
    path: Path,
    parts: tuple[tuple[str, bytes], ...],
    target_bytes: int,
    seed: int,
) -> dict[str, object]:
    """Write an exact-size archive and return its size/SHA-256 metadata.

    ``parts`` must already be fully rendered bytes, so no RNG is consumed
    during packaging; the filler has its own salted RNG. Sizing is linear, so
    a single correction pass absorbs any implementation-specific overhead.
    """
    names = tuple(name for name, _ in parts) + (FILLER_MEMBER,)
    fixed = sum(len(payload) for _, payload in parts) + zip_overhead(names)
    filler_length = target_bytes - fixed
    if filler_length <= 0:
        raise ValueError(f"skeleton ({fixed} bytes) exceeds target ({target_bytes} bytes)")

    path.parent.mkdir(parents=True, exist_ok=True)
    write_archive(path, parts, filler_length, seed)
    actual = path.stat().st_size
    if actual != target_bytes:
        filler_length -= actual - target_bytes
        if filler_length <= 0:
            raise ValueError(f"invalid corrected filler length {filler_length}")
        write_archive(path, parts, filler_length, seed)
        actual = path.stat().st_size
    if actual != target_bytes:
        raise ValueError(f"{path}: wrote {actual} bytes, expected {target_bytes}")

    with zipfile.ZipFile(path) as archive:
        if archive.namelist() != list(names) or archive.testzip() is not None:
            raise ValueError(f"{path}: archive failed structural validation")
    return {"size_bytes": actual, "sha256": sha256_file(path)}
