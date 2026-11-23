#!/usr/bin/env python3
"""Offline validation gate for the static CSAF 2.0 feed (#2714).

The structural checks used to live here by hand. They now call the same
`csaf_sync.check` the generator's self-test uses, so a published file cannot
drift from `botlib/src/security/advisory/data/advisories.json` and no invented
CVE or secret can be shipped. No third-party packages and no network access.
"""
from __future__ import annotations

import sys
from pathlib import Path

from advisory_source import SourceError, load_bundle, public_base
from csaf_records import CSAFError
from csaf_sync import check

ROOT = Path(__file__).resolve().parents[2]
FEED = ROOT / "security" / "csaf"


def main() -> int:
    try:
        errors = check(FEED, load_bundle(), public_base())
    except (CSAFError, SourceError, OSError, ValueError, KeyError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 1
    for error in errors:
        print(error)
    print(f"validate_csaf: {'FAIL' if errors else 'OK'} — {len(errors)} error(s)")
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
