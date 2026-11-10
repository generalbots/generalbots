#!/usr/bin/env python3
"""#2101 app-surface smoke check.

The build ships a suite of applications. Until now nothing proved that the
launcher's apps actually answer in the profile that ships: `unit-tests` ran
`cargo test -p botserver --bin botserver` (no HTTP), and the desktop-local build
was compiled but never asked for a single app page.

This script drives the surface list from the code instead of a hand-written
list, so a new app cannot be forgotten:

- **suite surfaces** -- every `url: "/suite/..."` in the app registry
  (`botserver/src/apps/registry/*.rs`) plus the desktop shell itself. The
  desktop-local botserver serves `/suite` from `botui/ui/suite` (or from the
  embedded copy) and is the profile that ships in the installers.
- **cloud + login surfaces** -- every top-level page under `botui/ui/cloud`
  and `botui/ui/login`, served statically (botui owns those ports; botserver
  never serves complete cloud/login pages).
- **catalog coverage** -- `GET /api/apps/catalog` must list every registry id,
  so a registry entry that no launcher can see is reported too.

Each surface must answer `200` with a non-empty HTML body. The run stops at the
first surface that does not respond (the CI job should fail loudly, not report a
summary that hides a blank app -- see #2100).

Usage (CI starts the servers; this script only probes them):

    python3 scripts/smoke_app_surfaces.py \
        --suite-base http://127.0.0.1:8080 \
        --static-base http://127.0.0.1:3100 \
        --catalog http://127.0.0.1:8080/api/apps/catalog

Exit codes: 0 every surface answered, 1 the first unresponsive surface, 2 a
local (checkout) problem such as a missing registry or a surface file.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
import urllib.error
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REGISTRY = ROOT / "botserver" / "src" / "apps" / "registry"
UI = ROOT / "botui" / "ui"

# The desktop shell is not a registry entry: it is the container every app is
# injected into, so it is asserted first (a shell that does not answer makes
# every app page meaningless).
SHELL_PATH = "/suite/desktop.html"

# `AppSpec` declares `id` before `url`, so pairing them in file order yields the
# id of each surface. The pattern below is scanned as one ordered stream, which
# makes a layout change fail loudly instead of silently mislabelling a surface.
ENTRY_RE = re.compile(r'\b(id|url):\s*"([^"]+)"')

TIMEOUT_SECS = 30


def registry_entries() -> list[tuple[str, str]]:
    """Return (app id, suite path) for every registry entry with a URL."""
    entries: list[tuple[str, str]] = []
    seen: set[str] = set()
    for path in sorted(REGISTRY.glob("*.rs")):
        pairs = ENTRY_RE.findall(path.read_text(encoding="utf-8"))
        if len(pairs) % 2 != 0:
            raise SystemExit(
                f"smoke_app_surfaces: {path.name} has an unpaired id/url; "
                "the registry layout changed and the surface list can no longer "
                "be derived"
            )
        for (first, app_id), (second, url) in zip(pairs[::2], pairs[1::2]):
            if first != "id" or second != "url":
                raise SystemExit(
                    f"smoke_app_surfaces: {path.name} declares `{first}: {app_id}` "
                    f"before `{second}: {url}`; expected `id` then `url`"
                )
            if not url.startswith("/suite/"):
                continue
            if url in seen:
                continue
            seen.add(url)
            entries.append((app_id, url))
    if not entries:
        raise SystemExit(f"smoke_app_surfaces: no app surfaces in {REGISTRY}")
    return entries


def suite_paths() -> list[tuple[str, str]]:
    """(label, path) for the shell and every registry app surface."""
    surfaces = [("desktop-shell", SHELL_PATH)]
    surfaces.extend(registry_entries())
    return surfaces


def static_pages() -> list[tuple[str, str]]:
    """(label, path) for every cloud/login page, plus the suite apps that are
    static files (they are served by botserver too, so they belong to the
    suite base and are covered above)."""
    surfaces: list[tuple[str, str]] = []
    for app in ("cloud", "login"):
        directory = UI / app
        if not directory.is_dir():
            raise SystemExit(f"smoke_app_surfaces: missing {directory}")
        for page in sorted(directory.glob("*.html")):
            surfaces.append((f"{app}-{page.stem}", f"/{app}/{page.name}"))
    if not surfaces:
        raise SystemExit("smoke_app_surfaces: no cloud/login pages found")
    return surfaces


def verify_files(surfaces: list[tuple[str, str]]) -> None:
    """Fail fast, before any server boots, when a surface file is missing."""
    missing = [
        (label, path)
        for label, path in surfaces
        # Registry URLs carry cache-busting queries (`?v=3`); the file on disk
        # is the path without them.
        if not (UI / path.split("?", 1)[0].lstrip("/")).is_file()
    ]
    if missing:
        for label, path in missing:
            print(f"smoke_app_surfaces: {label} has no file botui/ui{path}", file=sys.stderr)
        raise SystemExit(2)


def fetch(url: str) -> tuple[int, str, str]:
    request = urllib.request.Request(url, headers={"Accept": "text/html,*/*"})
    try:
        with urllib.request.urlopen(request, timeout=TIMEOUT_SECS) as response:
            body = response.read()
            content_type = response.headers.get("content-type", "")
            return response.status, content_type, body.decode("utf-8", "replace")
    except urllib.error.HTTPError as error:
        body = error.read().decode("utf-8", "replace")
        return error.code, error.headers.get("content-type", ""), body
    except urllib.error.URLError as error:
        raise SystemExit(f"FAIL: GET {url} did not answer ({error.reason})") from error


def check_surface(base: str, label: str, path: str) -> None:
    url = f"{base.rstrip('/')}{path}"
    status, content_type, body = fetch(url)
    if status != 200:
        raise SystemExit(f"FAIL {label}: GET {url} -> {status} (expected 200)")
    if not body.strip():
        raise SystemExit(f"FAIL {label}: GET {url} -> 200 with an empty body")
    if "<" not in body:
        raise SystemExit(f"FAIL {label}: GET {url} -> 200 but the body has no HTML")
    if "text/html" not in content_type:
        raise SystemExit(
            f"FAIL {label}: GET {url} -> 200 with content-type {content_type!r} "
            "(expected text/html)"
        )
    print(f"ok   {label:<24} {path}")


def check_catalog(url: str, ids: set[str]) -> None:
    status, _, body = fetch(url)
    if status != 200:
        raise SystemExit(f"FAIL catalog: GET {url} -> {status} (expected 200)")
    try:
        payload = json.loads(body)
    except json.JSONDecodeError as error:
        raise SystemExit(f"FAIL catalog: GET {url} is not JSON ({error})") from error
    entries = {item.get("id"): item for item in payload.get("apps", [])}
    missing = sorted(app_id for app_id in ids if app_id not in entries)
    if missing:
        raise SystemExit(
            "FAIL catalog: GET /api/apps/catalog does not list "
            f"{', '.join(missing)}; the launcher cannot open them"
        )
    # An app visible in a launcher but absent from the compiled feature set is
    # exactly the blank-screen class of bug this job exists to catch (#2100).
    uncompiled = sorted(
        app_id for app_id in ids if entries[app_id].get("compiled") is not True
    )
    if uncompiled:
        raise SystemExit(
            "FAIL catalog: the shipping profile does not compile "
            f"{', '.join(uncompiled)} (catalog reports compiled=false)"
        )
    print(f"ok   {'catalog':<24} {len(ids)} registry ids present and compiled")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--suite-base", required=True, help="botserver base URL")
    parser.add_argument("--static-base", required=True, help="botui static base URL")
    parser.add_argument(
        "--catalog",
        default=None,
        help="catalog URL (default: <suite-base>/api/apps/catalog)",
    )
    args = parser.parse_args()

    suite = suite_paths()
    static = static_pages()
    verify_files(suite)
    verify_files(static)

    print(f"smoke_app_surfaces: {len(suite)} suite + {len(static)} static surfaces")
    for label, path in suite:
        check_surface(args.suite_base, label, path)
    for label, path in static:
        check_surface(args.static_base, label, path)

    catalog = args.catalog or f"{args.suite_base.rstrip('/')}/api/apps/catalog"
    check_catalog(catalog, {app_id for app_id, _ in suite})

    print("smoke_app_surfaces: every shipped app surface answered")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
