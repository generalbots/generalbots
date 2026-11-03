#!/usr/bin/env python3
"""Append a compressed stack to the desktop app (#1633).

    python3 scripts/build-self-extractor.py \
        --app target/release/botapp \
        --payload dist/payload \
        --out dist/generalbots

The output is a normal executable with:

    [ app ][ payload.tar.zst ][ magic(12) + offset(u64) + length(u64) + sha256 ]

`generalbots --verify` checks the digest, `--extract-only` unpacks the payload
into the per-user stack. Requires the `zstd` CLI (apt install zstd).
"""

from __future__ import annotations

import argparse
import hashlib
import os
import shutil
import subprocess
import sys
import tarfile
import tempfile
from pathlib import Path

MAGIC = b"GB6PAYLOAD!!"


def build_archive(payload_dir: Path, archive: Path, level: int) -> None:
    with tempfile.TemporaryDirectory(prefix="gb6-payload-") as tmp:
        tar_path = Path(tmp) / "payload.tar"
        with tarfile.open(tar_path, "w") as tar:
            for entry in sorted(payload_dir.iterdir()):
                tar.add(entry, arcname=entry.name)
        zstd = shutil.which("zstd")
        if not zstd:
            raise SystemExit(
                "the zstd CLI is required (apt install zstd) to build the payload"
            )
        subprocess.run(
            [zstd, f"-{level}", "--force", "--quiet", "-o", str(archive), str(tar_path)],
            check=True,
        )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--app", type=Path, required=True)
    parser.add_argument("--payload", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--level", type=int, default=19)
    args = parser.parse_args()

    if not args.app.is_file():
        raise SystemExit(f"missing app binary: {args.app}")
    if not args.payload.is_dir():
        raise SystemExit(f"missing payload directory: {args.payload}")

    with tempfile.TemporaryDirectory(prefix="gb6-build-") as tmp:
        archive = Path(tmp) / "payload.tar.zst"
        build_archive(args.payload, archive, args.level)
        offset = args.app.stat().st_size
        length = archive.stat().st_size

        digest = hashlib.sha256()
        outer = hashlib.sha256()
        args.out.parent.mkdir(parents=True, exist_ok=True)
        with args.out.open("wb") as output:
            with args.app.open("rb") as app_file:
                while True:
                    chunk = app_file.read(1024 * 1024)
                    if not chunk:
                        break
                    outer.update(chunk)
                    output.write(chunk)
            with archive.open("rb") as payload_file:
                while True:
                    chunk = payload_file.read(1024 * 1024)
                    if not chunk:
                        break
                    digest.update(chunk)
                    outer.update(chunk)
                    output.write(chunk)
            trailer = (
                MAGIC
                + offset.to_bytes(8, "little")
                + length.to_bytes(8, "little")
                + digest.digest()
            )
            outer.update(trailer)
            output.write(trailer)

        os.chmod(args.out, 0o755)

    print(
        f"built {args.out} ({args.out.stat().st_size} bytes; "
        f"app {offset} + payload {length})"
    )
    print(f"sha256 {outer.hexdigest()}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
