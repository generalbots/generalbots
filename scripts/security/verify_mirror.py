#!/usr/bin/env python3
"""Compare a mirror tree with the authoritative static advisory files (#2726)."""
import hashlib
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SOURCES = ["security/csaf", "security/osv", "security/pages", "security/advisories"]


def main(argv):
    if len(argv) != 1:
        print("usage: verify_mirror.py <mirror-dir>")
        return 2
    mirror = Path(argv[0])
    checked, failures = 0, 0
    for source in SOURCES:
        for path in sorted((ROOT / source).rglob("*")):
            if not path.is_file():
                continue
            relative = path.relative_to(ROOT)
            checked += 1
            copy = mirror / relative
            if not copy.is_file() or hashlib.sha256(copy.read_bytes()).digest() != hashlib.sha256(path.read_bytes()).digest():
                print(f"MISMATCH {relative}")
                failures += 1
    print(f"verify_mirror: {'FAIL' if failures else 'OK'} — {checked} file(s), {failures} mismatch(es)")
    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
