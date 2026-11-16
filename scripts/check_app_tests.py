#!/usr/bin/env python3
"""Budget/guard: ensure each small suite app has a webtest-apps test file (#2418).

Usage:
  python3 scripts/check_app_tests.py
Exits non-zero if a required app is missing its test file.
"""
from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
TEST_DIR = ROOT / "botui/ui/suite/tests/webtest-apps"

# Apps that must have a webtest-apps/<app>.js suite.
REQUIRED_APPS = [
    "timer",
    "notes",
    "notepad",
    "recycle",
    "concierge",
    "snapshot",
    "calculator",
    "weather",
    "store",
    "timeclock",
    "lists",
    "photos",
]


def main() -> int:
    missing = []
    present = []
    for app in REQUIRED_APPS:
        test_file = TEST_DIR / f"{app}.js"
        if test_file.exists():
            present.append(app)
        else:
            missing.append(app)

    print("webtest-apps budget (#2418)")
    print(f"  test dir: {TEST_DIR}")
    print(f"  required: {len(REQUIRED_APPS)}")
    print(f"  present : {len(present)}")
    for app in present:
        print(f"    ✓ {app} -> {TEST_DIR / (app + '.js')}")
    if missing:
        print(f"  missing : {len(missing)}")
        for app in missing:
            print(f"    ✗ {app} (expected {TEST_DIR / (app + '.js')})")
        return 1

    print("  result  : OK (all required apps have a suite)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
