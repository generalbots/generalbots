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

Contract mode (#2119)
---------------------
`--contracts` runs the static half of the same idea, before any build: every
registry app must be classified in both product files, present in the smoke
list derived above, and documented in `issues/app-contracts/apps.toml` (routes
that really exist, or `no_api` with a reason and an issue). No server, no
cargo -- seconds, and it fails a new app that nobody documented.

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


def check_integrations_api(base: str) -> None:
    """#2100 — an Integrations app whose API is missing renders empty.

    The provider catalogue is the app's first request: it must answer `200`
    with providers, and the workspace context must answer `200` with either a
    bot id or the documented `no_active_bot` payload. A 404 here is the blank
    pane, so it fails the run instead of reaching a user.
    """
    root = base.rstrip("/")
    status, _, body = fetch(f"{root}/api/apps/integrations/catalog")
    if status != 200:
        raise SystemExit(
            f"FAIL integrations: GET /api/apps/integrations/catalog -> {status} "
            "(expected 200; the Explore pane would be empty)"
        )
    try:
        payload = json.loads(body)
    except json.JSONDecodeError as error:
        raise SystemExit(
            f"FAIL integrations: catalog is not JSON ({error}); the app cannot render it"
        ) from error
    providers = payload.get("providers")
    if not isinstance(providers, list) or not providers:
        raise SystemExit(
            "FAIL integrations: GET /api/apps/integrations/catalog returned no providers"
        )
    unusable = [
        str(item.get("id"))
        for item in providers
        if not isinstance(item, dict) or not item.get("id")
    ]
    if unusable:
        raise SystemExit(
            "FAIL integrations: catalog providers without an id: " + ", ".join(unusable[:5])
        )

    status, _, body = fetch(f"{root}/api/apps/integrations/context")
    if status != 200:
        raise SystemExit(
            f"FAIL integrations: GET /api/apps/integrations/context -> {status} "
            "(expected 200, or the documented no_active_bot payload)"
        )
    try:
        context = json.loads(body)
    except json.JSONDecodeError as error:
        raise SystemExit(
            f"FAIL integrations: context is not JSON ({error})"
        ) from error
    if not context.get("bot_id") and context.get("reason") != "no_active_bot":
        raise SystemExit(
            "FAIL integrations: context has no bot_id and no documented "
            f"no_active_bot reason (got {sorted(context)})"
        )
    print(
        f"ok   {'integrations':<24} {len(providers)} providers, context "
        f"{'bound' if context.get('bot_id') else 'no_active_bot'}"
    )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--suite-base", help="botserver base URL")
    parser.add_argument("--static-base", help="botui static base URL")
    parser.add_argument(
        "--catalog",
        default=None,
        help="catalog URL (default: <suite-base>/api/apps/catalog)",
    )
    parser.add_argument(
        "--contracts",
        action="store_true",
        help="static app-contract check (#2119): no servers needed",
    )
    parser.add_argument(
        "--self-test",
        action="store_true",
        help="prove the app-contract rules fail when violated",
    )
    args = parser.parse_args()

    sys.path.insert(0, str(Path(__file__).resolve().parent))
    import app_contracts

    if args.self_test:
        return app_contracts.self_test()

    if args.contracts:
        registry = registry_entries()
        try:
            return app_contracts.report(
                registry,
                app_contracts.validate(
                    registry,
                    app_contracts.load_contracts(),
                    app_contracts.classification(),
                    app_contracts.route_literals(),
                    {app_id for app_id, _ in suite_paths()},
                ),
            )
        except app_contracts.ContractError as error:
            print(f"error: {error}", file=sys.stderr)
            return 2

    if not args.suite_base or not args.static_base:
        parser.error("--suite-base and --static-base are required (or use --contracts)")

    suite_all = suite_paths()
    static = static_pages()

    catalog = args.catalog or f"{args.suite_base.rstrip('/')}/api/apps/catalog"
    # The shipping profile may compile only a subset of the registry
    # (e.g. desktop-local). Probe the catalog first so we only assert the
    # surfaces the profile actually ships, while still validating that every
    # shipped (compiled) registry app is present and answers.
    status, _, body = fetch(catalog)
    if status != 200:
        raise SystemExit(f"FAIL catalog: GET {catalog} -> {status} (expected 200)")
    try:
        payload = json.loads(body)
    except json.JSONDecodeError as error:
        raise SystemExit(f"FAIL catalog: GET {catalog} is not JSON ({error})") from error
    catalog_entries = {item.get("id"): item for item in payload.get("apps", [])}
    registry_ids = {app_id for app_id, _ in suite_all}
    missing = sorted(app_id for app_id in registry_ids if app_id not in catalog_entries)
    if missing:
        raise SystemExit(
            "FAIL catalog: GET /api/apps/catalog does not list "
            f"{', '.join(missing)}; the launcher cannot open them"
        )
    compiled_ids = {
        app_id for app_id in registry_ids
        if catalog_entries.get(app_id, {}).get("compiled") is True
    }
    uncompiled = sorted(registry_ids - compiled_ids)
    if uncompiled:
        print(
            "smoke_app_surfaces: skipping non-compiled registry apps in this profile: "
            + ", ".join(uncompiled)
        )
    suite = [
        (label, path) for label, path in suite_all
        if label == "desktop-shell" or label in compiled_ids
    ]

    verify_files(suite)
    verify_files(static)

    print(f"smoke_app_surfaces: {len(suite)} suite + {len(static)} static surfaces")
    for label, path in suite:
        check_surface(args.suite_base, label, path)
    for label, path in static:
        check_surface(args.static_base, label, path)

    check_catalog(catalog, compiled_ids)
    # #2100 — an app can answer its page and still render empty because the
    # API behind it is missing from the profile.
    if "integrations" in compiled_ids:
        check_integrations_api(args.suite_base)
    else:
        print("smoke_app_surfaces: integrations not compiled in this profile, skipping API check")

    print("smoke_app_surfaces: every shipped app surface answered")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
