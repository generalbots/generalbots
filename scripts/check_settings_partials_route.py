#!/usr/bin/env python3
"""#2935 criterion 3 — Settings partials route guard (standalone, CI-callable).

Two modes:

* Offline (default): pin the source chain that decides whether the ten
  ``/suite/settings/partials/*.html`` requests are 200 or 404 in a deployment:
  the eleven shipped files, the exact clean URLs in the Settings boot document,
  the botui ``/suite/:dir/partials/*path`` proxy to botserver, the botserver
  ``ServeDir`` rooted at ``BOTUI_PATH``, and the whole-suite-tree release
  packaging. A stale/unset ``BOTUI_PATH`` on a host is the documented root
  cause of the source-vs-host gap; the offline checks make that chain visible.
* Live (``--base-url``): request the same eleven URLs over HTTP and require 200
  with ``text/html`` (zero 404s) — the smoke test CI runs against a started
  suite. Requests go through botui, exercising the proxy chain end to end.

This guard is intentionally standalone: it is not registered in
``scripts/prod_readiness_gates.py``. CI invokes it directly.

Usage:
    python3 scripts/check_settings_partials_route.py
    python3 scripts/check_settings_partials_route.py --base-url http://127.0.0.1:3000

Exit codes: 0 = every check passed, 1 = at least one check failed,
2 = usage error.
"""

from __future__ import annotations

import argparse
import sys
import urllib.error
import urllib.request
from pathlib import Path
from urllib.parse import urljoin

ROOT = Path(__file__).resolve().parents[1]
PARTIAL_DIR = ROOT / "botui/ui/suite/settings/partials"
INDEX = ROOT / "botui/ui/suite/settings/index.html"
REGISTRY = ROOT / "botui/ui/suite/js/window-manager/01_registry.js"
UI_ROUTER = ROOT / "botui/src/ui_server/mod.rs"
PROXY = ROOT / "botui/src/ui_server/proxy.rs"
AXUM = ROOT / "botserver/src/main_module/server/run_axum_server.rs"
LINUX_PKG = ROOT / "desktop/linux/package.sh"
RELEASE = ROOT / ".github/workflows/release.yml"

# The exact ten URLs from issue #2935 (browser console on the host).
BOOT_PARTIALS = (
    "01_profile_security.html",
    "06_dialogs.html",
    "07_os.html",
    "08_shared_links.html",
    "10_system.html",
    "12_model.html",
    "17_limits.html",
    "13_instructions.html",
    "14_shortcuts.html",
    "15_os_appliance.html",
    "16_protection.html",
)
BOOT_PREFIX = "/suite/settings/partials/"

FAILURES: list[str] = []


def check(label: str, condition: bool, detail: str = "") -> None:
    if condition:
        print(f"[PASS] {label}")
    else:
        FAILURES.append(label)
        suffix = f" — {detail}" if detail else ""
        print(f"[FAIL] {label}{suffix}")


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def partial_refs(src: str) -> list[str]:
    """Extract every hx-get partial URL, retaining any query string."""
    marker = 'hx-get="' + BOOT_PREFIX
    refs: list[str] = []
    at = 0
    while True:
        start = src.find(marker, at)
        if start == -1:
            return refs
        end = src.find('"', start + len(marker))
        if end == -1:
            refs.append("<unterminated hx-get>")
            return refs
        refs.append(BOOT_PREFIX + src[start + len(marker):end])
        at = end + 1


def count_in(src: str, needle: str) -> int:
    return src.count(needle)


def check_shipped_files() -> None:
    check("#2935 the issue names ten partials plus the gbgov limits tab", len(BOOT_PARTIALS) == 11)
    for name in BOOT_PARTIALS:
        path = PARTIAL_DIR / name
        if not path.is_file():
            check(f"#2935 partial {name} ships", False, "file missing")
            continue
        body = read(path)
        check(f"#2935 partial {name} ships as non-trivial HTML",
              len(body) > 500 and "<" in body and ">" in body,
              f"{len(body)} bytes")


def check_boot_urls() -> None:
    index = read(INDEX)
    refs = partial_refs(index)
    expected = [BOOT_PREFIX + name for name in BOOT_PARTIALS]
    check("#2935 boot document issues exactly eleven partial requests",
          len(refs) == 11, f"found {len(refs)}: {refs}")
    check("#2935 boot document requests each partial once",
          len(set(refs)) == len(refs))
    for url in expected:
        check(f"#2935 boot document keeps {url}", url in refs)
    for url in refs:
        check(f"#2935 no stale or renamed boot partial {url}", url in expected)
        check(f"#2935 no ?v= query on partial {url}", "?" not in url)

    registry = read(REGISTRY)
    entry = next((line for line in registry.splitlines() if 'id: "settings"' in line), "")
    check("#2935 Settings deep link fetches the boot document",
          'hxGet: "/suite/settings/index.html"' in entry, entry.strip())

    # Nested partials are not boot requests, but every reference in the whole
    # Settings surface must still resolve (catches a stale nested chain).
    for path in sorted(ROOT.glob("botui/ui/suite/settings/**/*.html")):
        for url in partial_refs(read(path)):
            check(f"#2935 no query string on {url} in {path.name}", "?" not in url)
            name = url[len(BOOT_PREFIX):]
            check(f"#2935 {url} referenced by {path.name} resolves",
                  (PARTIAL_DIR / name).is_file())


def check_serving_chain() -> None:
    ui = read(UI_ROUTER)
    generic = '.route("/suite/:dir/partials/*path", any(proxy_api))'
    check("#2935 both ui_server branches proxy /suite/:dir/partials/*path",
          count_in(ui, generic) == 2, f"found {count_in(ui, generic)}")
    check("#2935 SUITE_DIRS gets an explicit partials proxy route",
          "for dir in SUITE_DIRS" in ui
          and '.route(&format!("/suite/{dir}/partials/*path"), any(proxy_api))' in ui)
    check("#2935 proxy/serve precedence is documented in source",
          "matchit ranks static segments" in ui)

    proxy = read(PROXY)
    check("#2935 botui forwards requests to botserver",
          "pub async fn proxy_api(" in proxy and "state.client.base_url()" in proxy)

    axum = read(AXUM)
    mount = '.nest_service("/suite", ServeDir::new(&ui_path))'
    check("#2935 botserver static router reads BOTUI_PATH",
          "pub(super) fn build_base_router(" in axum
          and 'std::env::var("BOTUI_PATH")' in axum
          and "let ui_path_exists" in axum)
    check("#2935 /suite is served by ServeDir from BOTUI_PATH", mount in axum)
    check("#2935 path-existence check gates the ServeDir mount",
          axum.find("let ui_path_exists") < axum.find(mount))
    check("#2935 effective UI root is logged for diagnosis",
          "Serving UI from external folder" in axum)


def check_packaging() -> None:
    pkg = read(LINUX_PKG)
    linux_line = next(
        (line for line in pkg.splitlines() if '"$ROOT/botui/ui/suite"' in line), ""
    )
    check("#2935 Linux packaging recursively copies the whole suite tree",
          "cp -a" in linux_line and '"$DEST/ui/suite"' in linux_line,
          linux_line.strip())

    release = read(RELEASE)
    check("#2935 Linux release payload contains the full suite tree",
          "cp -a botui/ui/suite  dist/payload/ui/suite" in release)
    check("#2935 Windows release payload recursively copies the suite tree",
          "Copy-Item -Recurse botui/ui/suite  dist/payload/ui/suite" in release)
    check("#2935 CI smoke test points botserver at the packaged suite root",
          'BOTUI_PATH="$PWD/ui/suite"' in release)


def check_live(base_url: str) -> None:
    """GET the ten partials through the suite and require 200 text/html."""
    base = base_url.rstrip("/") + "/"
    for name in BOOT_PARTIALS:
        path = BOOT_PREFIX + name
        request = urllib.request.Request(
            urljoin(base, path.lstrip("/")),
            headers={"Cache-Control": "no-cache", "User-Agent": "gb-settings-route-guard"},
        )
        try:
            with urllib.request.urlopen(request, timeout=10) as response:
                status = response.status
                content_type = response.headers.get("Content-Type", "")
                body = response.read()
        except urllib.error.HTTPError as error:
            status = int(error.code)
            content_type = ""
            body = b""
        except (urllib.error.URLError, OSError) as error:
            check(f"#2935 live {path} reachable", False, type(error).__name__)
            continue
        check(f"#2935 live {path} returns 200 text/html (not 404)",
              status == 200 and content_type.startswith("text/html") and len(body) > 500,
              f"status={status} content-type={content_type or '-'} bytes={len(body)}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--base-url",
        metavar="URL",
        help="suite origin (port 3000) to smoke-test the ten partial URLs live",
    )
    args = parser.parse_args()

    check_shipped_files()
    check_boot_urls()
    check_serving_chain()
    check_packaging()
    if args.base_url:
        check_live(args.base_url)

    total = len(FAILURES)
    print("-" * 70)
    print(f"#2935 Settings route guard: {'OK' if total == 0 else 'FAILED'} "
          f"({total} failure(s))")
    if total:
        for label in FAILURES:
            print(f"  - {label}")
    return 1 if total else 0


if __name__ == "__main__":
    sys.exit(main())
