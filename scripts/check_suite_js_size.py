#!/usr/bin/env python3
"""Fail when a non-vendor suite JS module exceeds the 450-line limit (#2305)."""
from __future__ import annotations

import os
import sys
from pathlib import Path

ROOT = Path("botui/ui/suite")
LIMIT = 450


def main() -> int:
    if not ROOT.is_dir():
        print(f"FAIL: suite root not found: {ROOT}", file=sys.stderr)
        return 2

    bad: list[tuple[str, int]] = []
    for dirpath, dirnames, filenames in os.walk(ROOT):
        dirnames[:] = [d for d in dirnames if d not in ("vendor", "node_modules")]
        for name in filenames:
            if not name.endswith(".js"):
                continue
            if name.endswith(".orig") or ".bak" in name:
                continue
            path = Path(dirpath) / name
            try:
                with path.open("r", encoding="utf-8", errors="ignore") as f:
                    n = sum(1 for _ in f)
            except OSError as exc:
                print(f"ERR reading {path}: {exc}", file=sys.stderr)
                return 2
            if n > LIMIT:
                bad.append((str(path), n))

    if bad:
        bad.sort(key=lambda item: (-item[1], item[0]))
        for path, n in bad:
            print(f"{n}\t{path}")
        print(
            f"FAIL: {len(bad)} JS module(s) under {ROOT} exceed {LIMIT} lines",
            file=sys.stderr,
        )
        return 1

    print(f"OK: no JS module under {ROOT} exceeds {LIMIT} lines")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
