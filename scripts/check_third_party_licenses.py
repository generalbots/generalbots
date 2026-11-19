#!/usr/bin/env python3
"""Fail when a bundled runtime component has no license entry (#2480).

`botserver/3rdparty.toml` is the provenance record for every binary the
installer downloads or the appliance image carries. Shipping those binaries is
distribution, so each one needs its license recorded — several of them are
AGPL or BUSL, which carry source-offer obligations that cannot be satisfied by
a note buried in a README.

`LICENSES-desktop.md` is that record: one row per component, with the SPDX
identifier, the upstream source URL and where the license text is shipped
inside the image. This gate keeps the two files in sync:

  * a component in `3rdparty.toml` with no row fails;
  * a row whose SPDX id or source URL is empty fails;
  * a row naming a component that no longer exists fails (stale entry);
  * the copies served to Settings → About → Licenses must match the canonical
    record and NOTICE byte for byte, so the UI cannot show a stale table.

Usage:
    python3 scripts/check_third_party_licenses.py [--toml PATH] [--licenses PATH] [--served DIR]

Exit codes: 0 the record is complete, 1 otherwise.
"""

from __future__ import annotations

import argparse
import sys
import tomllib
from pathlib import Path

DEFAULT_TOML = Path("botserver/3rdparty.toml")
DEFAULT_LICENSES = Path("LICENSES-desktop.md")
DEFAULT_SERVED = Path("botui/ui/suite/about/licenses")
DEFAULT_NOTICE = Path("desktop/os/licenses/NOTICE")

# Column order of the table in `LICENSES-desktop.md`.
HEADER = ("component | name | version | license | source | text")


def component_keys(toml_path: Path) -> list[str]:
    """Every `[components.<key>]` entry, sorted."""
    data = tomllib.loads(toml_path.read_text(encoding="utf-8"))
    return sorted((data.get("components") or {}).keys())


def license_rows(md_path: Path) -> dict[str, list[str]]:
    """`component key -> cells` for every data row of the license table."""
    rows: dict[str, list[str]] = {}
    for raw in md_path.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if not line.startswith("|") or line.startswith("|---") or "---" in line[:4]:
            continue
        cells = [cell.strip() for cell in line.strip("|").split("|")]
        if len(cells) != 6:
            continue
        if cells[0].lower() == "component":
            continue
        rows[cells[0].strip("`")] = cells
    return rows


def problems(keys: list[str], rows: dict[str, list[str]]) -> list[str]:
    found: list[str] = []
    for key in keys:
        cells = rows.get(key)
        if cells is None:
            found.append(f"{key}: missing from LICENSES-desktop.md")
            continue
        if not cells[3] or cells[3] == "-":
            found.append(f"{key}: no SPDX license identifier")
        if not cells[4] or not cells[4].startswith("http"):
            found.append(f"{key}: no upstream source URL")
        if not cells[5]:
            found.append(f"{key}: no license text location")
    for key in sorted(set(rows) - set(keys)):
        found.append(f"{key}: listed in LICENSES-desktop.md but not in 3rdparty.toml")
    return found


def served_copy_problems(
    served: Path, licenses: Path, notice: Path
) -> list[str]:
    """The About → Licenses view must mirror the canonical shipped texts."""
    found: list[str] = []
    for name, source in (("LICENSES-desktop.md", licenses), ("NOTICE", notice)):
        copy = served / name
        if not copy.is_file():
            found.append(f"{copy}: missing served copy of {name}")
        elif copy.read_bytes() != source.read_bytes():
            found.append(f"{copy}: differs from {source}; refresh the served copy")
    return found


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--toml", type=Path, default=DEFAULT_TOML)
    parser.add_argument("--licenses", type=Path, default=DEFAULT_LICENSES)
    parser.add_argument("--served", type=Path, default=DEFAULT_SERVED)
    args = parser.parse_args()

    for path in (args.toml, args.licenses):
        if not path.is_file():
            print(f"missing {path}", file=sys.stderr)
            return 1
    if not Path("LICENSE").is_file():
        print("missing LICENSE (the MIT text for General Bots itself)", file=sys.stderr)
        return 1

    keys = component_keys(args.toml)
    rows = license_rows(args.licenses)
    found = problems(keys, rows)
    found.extend(served_copy_problems(args.served, args.licenses, DEFAULT_NOTICE))
    if found:
        print("Third-party license record is incomplete (#2480):", file=sys.stderr)
        for item in found:
            print(f"  {item}", file=sys.stderr)
        print(
            "Add or fix the row in LICENSES-desktop.md; every component in "
            "botserver/3rdparty.toml must name its license and source.",
            file=sys.stderr,
        )
        return 1
    print(f"{len(keys)} components have a license entry")
    return 0


if __name__ == "__main__":
    sys.exit(main())
