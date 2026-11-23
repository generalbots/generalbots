#!/usr/bin/env python3
"""Package the static advisory tree into a verifiable offline bundle (#2720)."""
import argparse
import hashlib
import tarfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SOURCES = ["security/csaf", "security/osv", "security/vex", "security/pages", "security/advisories"]


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", required=True)
    args = parser.parse_args()
    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    manifest = []
    with tarfile.open(out, "w:gz") as archive:
        for source in SOURCES:
            base = ROOT / source
            if not base.exists():
                continue
            for path in sorted(base.rglob("*")):
                if not path.is_file():
                    continue
                relative = path.relative_to(ROOT)
                digest = hashlib.sha256(path.read_bytes()).hexdigest()
                manifest.append(f"{digest}  {relative.as_posix()}")
                archive.add(path, arcname=relative.as_posix())
        manifest_path = out.parent / "MANIFEST.sha256"
        manifest_path.write_text("\n".join(manifest) + "\n", encoding="utf-8")
        archive.add(manifest_path, arcname="MANIFEST.sha256")
    print(f"build_offline_bundle: {len(manifest)} file(s) -> {out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
