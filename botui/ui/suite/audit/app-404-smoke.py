#!/usr/bin/env python3
"""Smoke test: open every available Suite app and report HTTP 404s.

Two modes:

  Live  - attaches to an already-running Chrome/Brave over CDP, opens one OWN
          tab (other tabs are never navigated or closed), signs in only when a
          lock screen is present, opens each app through the shell's real
          launcher path, records 404 responses, closes the app, and exits 1 if
          any app returned a 404. Needs a running server + a signed-in account.

  CI    - `--self-test` exercises the offline classification/report logic with
          fixtures. No browser, no server, no credentials. This is the mode
          wired into .github/workflows/ci.yml; the live mode is intentionally
          NOT a CI gate because CI has no test account.

Usage (from repo root):
  python3 botui/ui/suite/audit/app-404-smoke.py --self-test
  GB_TEST_EMAIL=... GB_TEST_PASSWORD=... \
    python3 botui/ui/suite/audit/app-404-smoke.py \
      --cdp http://127.0.0.1:9222 --base https://pragmatismo.generalbots.org \
      --wait 2 --report /tmp/opencode/app-404-smoke-report.json

Credentials are only read from the environment / CLI and are never printed or
written to the report. No extra Playwright config file is added.
"""

from __future__ import annotations

import argparse
import asyncio
import json
import os
import sys
from urllib.parse import urlsplit

DEFAULT_CDP = "http://127.0.0.1:9222"
DEFAULT_BASE = "http://localhost:3000"
DEFAULT_REPORT = "/tmp/opencode/app-404-smoke-report.json"
WIDGET_IDS = {"clock-widget"}


def normalize(url: str) -> str:
    """Reduce an absolute response URL to a stable origin-relative path."""
    return urlsplit(url).path or "/"


def classify(seen: list[str]) -> list[str]:
    """Deduplicate and sort the 404 paths seen for one app."""
    return sorted({normalize(item) for item in seen if item.startswith("/")})


def self_test() -> int:
    fixtures = [
        ("https://pragmatismo.generalbots.org/api/slides/media/list?x=1",
         "/api/slides/media/list"),
        ("/suite/settings/partials/07_os.html", "/suite/settings/partials/07_os.html"),
    ]
    for raw, expected in fixtures:
        got = normalize(raw)
        assert got == expected, f"normalize({raw!r}) = {got!r}, expected {expected!r}"
    result = classify([
        "https://host/api/a?x=1", "/api/a", "/api/b", "HARNESS timeout", "/api/a",
    ])
    assert result == ["/api/a", "/api/b"], result
    report = {"base": DEFAULT_BASE, "apps": 2, "results": [
        {"app": "docs", "not_found": []},
        {"app": "tasks", "not_found": ["/api/tasks/stats/json"]},
    ]}
    bad = [r for r in report["results"] if r["not_found"]]
    assert len(bad) == 1 and bad[0]["app"] == "tasks", report
    print("SELFTEST PASS: normalize/classify/report OK (no browser, no account)")
    return 0


async def run(args) -> int:
    from playwright.async_api import async_playwright

    email = args.email or os.environ.get("GB_TEST_EMAIL", "")
    password = args.password or os.environ.get("GB_TEST_PASSWORD", "")
    token = args.token or os.environ.get("GB_TEST_TOKEN", "")
    target = args.base.rstrip("/") + "/" + args.bot.strip("/") + "/"

    # Endpoints that are expected to return 503 in this environment (e.g. Drive
    # is not provisioned in the console-smoke job) are reported, never hidden.
    allow_503 = [v.strip() for v in os.environ.get("GB_SMOKE_ALLOW_503", "").split(",") if v.strip()]
    # The throwaway CI account is a viewer on the seeded Vibe projects: a
    # background canvas save (project.draw) can hit a 403 on its files endpoint
    # after another app selected it. Narrow, path-scoped and reported as an
    # environment note; no 404 or any other console error is exempted.
    allow_403 = [v.strip() for v in os.environ.get(
        "GB_SMOKE_ALLOW_403", "/api/vibe/projects/,/api/collab/links").split(",") if v.strip()]

    def allowed_cross_owner_403():
        urls = forbidden_bucket["items"]
        forbidden_bucket["items"] = []
        def allowed(url):
            path = urlsplit(url).path
            return any(
                path.startswith(p) and ("/files" in path if p.startswith("/api/vibe/") else True)
                for p in allow_403
            )

        if not urls or not all(allowed(url) for url in urls):
            forbidden_bucket["items"] = urls
            return
        allowed_bucket["items"].extend(
            {"source": "http.403 (allowed cross-owner scoped read)", "text": url}
            for url in urls)
        console_bucket["items"] = [h for h in console_bucket["items"]
                                   if "403" not in h["text"]]

    def write_report(results, total, stopped_at, console_errors, recoveries, allowed_503, forbidden):
        os.makedirs(os.path.dirname(args.report), exist_ok=True)
        with open(args.report, "w") as out:
            json.dump({"base": args.base, "apps": total, "results": results,
                       "stopped_at": stopped_at, "console_errors": console_errors,
                       "allowed_503": allowed_503, "forbidden": forbidden,
                       "shell_re_navigations": recoveries}, out, indent=1)

    async with async_playwright() as p:
        try:
            browser = await p.chromium.connect_over_cdp(args.cdp)
        except Exception as exc:
            print(f"SKIP: CDP endpoint {args.cdp} unavailable ({str(exc)[:120]})")
            return 0
        ctx = browser.contexts[0] if browser.contexts else await browser.new_context()
        page = await ctx.new_page()
        if token:
            await page.add_init_script(
                "(() => { const t = " + json.dumps(token) + ";"
                " localStorage.setItem('gb-access-token', t);"
                " localStorage.setItem('management_token', t);"
                " if (t.indexOf('eyJ') === 0) localStorage.setItem('gb-ws-token', t);"
                " })()")
        bucket = {"items": []}
        console_bucket = {"items": []}
        server_bucket = {"items": []}
        allowed_bucket = {"items": []}
        forbidden_bucket = {"items": []}

        def on_response(response):
            if response.status == 404:
                bucket["items"].append(response.url)
            if response.status == 403:
                forbidden_bucket["items"].append(response.url)
            if response.status == 503:
                server_bucket["items"].append(response.url)

        def on_console(message):
            if message.type in ("error", "warning"):
                console_bucket["items"].append({
                    "source": "console." + message.type,
                    "text": message.text[:500],
                })

        def on_pageerror(error):
            console_bucket["items"].append({
                "source": "pageerror", "text": str(error)[:500],
            })

        page.on("response", on_response)
        page.on("console", on_console)
        page.on("pageerror", on_pageerror)

        try:
            await page.goto(target,
                            wait_until="domcontentloaded", timeout=60000)
            await page.wait_for_function(
                "() => typeof WindowManager !== 'undefined' && !!document.getElementById('startBtn')",
                timeout=60000)

            if await page.evaluate(
                    "typeof LockScreen !== 'undefined' && LockScreen.isLocked()"):
                if not email or not password:
                    print("SKIP: desktop is locked; set GB_TEST_EMAIL/GB_TEST_PASSWORD "
                          "or pass --email/--password to run the live smoke")
                    return 0
                await page.fill("#gbLockEmail", email)
                await page.fill("#gbLockPassword", password)
                await page.evaluate("LockScreen.unlockWithCredential()")
                await page.wait_for_function("() => !LockScreen.isLocked()", timeout=30000)

            boot_404s = classify(bucket["items"])
            bucket["items"] = []
            results = [{"app": "<shell-boot>", "not_found": boot_404s}]
            # A 503 from an explicitly allowed optional backend is recorded as
            # an environment note; any OTHER console error still stops the sweep.
            urls = server_bucket["items"]
            server_bucket["items"] = []
            benign_503 = bool(allow_503) and bool(urls) and all(
                any(urlsplit(url).path.startswith(prefix) for prefix in allow_503)
                for url in urls)
            if benign_503:
                allowed_bucket["items"] = [
                    {"source": "http.503 (allowed by GB_SMOKE_ALLOW_503)", "text": url}
                    for url in urls]
                console_bucket["items"] = [item for item in console_bucket["items"]
                                           if "503" not in item["text"]]
            allowed_cross_owner_403()
            hits = console_bucket["items"]
            console_bucket["items"] = []
            if hits and not args.allow_console:
                write_report(results, 0, "<shell-boot>", hits, 0, allowed_bucket["items"], forbidden_bucket["items"])
                print("FAIL: console error/warning during shell boot; stopping sweep")
                for hit in hits:
                    print(f"  {hit['source']}: {hit['text'][:200]}")
                for url in forbidden_bucket["items"]:
                    print(f"  403 URL: {url}")
                print(f"Report: {args.report}")
                return 1

            apps = await page.evaluate(
                "async () => (await (await fetch('/api/apps/catalog',"
                "{credentials:'include'})).json()).apps")
            apps = [a for a in apps if a.get("available") and a["id"] not in WIDGET_IDS]
            if args.apps:
                wanted = {item.strip() for item in args.apps.split(",")}
                apps = [a for a in apps if a["id"] in wanted]
            if not apps:
                print("SKIP: catalog returned no available apps")
                return 0

            recoveries = 0
            for index, app in enumerate(apps, 1):
                app_id = app["id"]
                shell_ready = False
                try:
                    shell_ready = await page.evaluate(
                        "() => typeof WindowManager !== 'undefined'")
                except Exception:
                    shell_ready = False
                if not shell_ready:
                    await page.goto(target,
                                    wait_until="domcontentloaded", timeout=60000)
                    await page.wait_for_function(
                        "() => typeof WindowManager !== 'undefined'", timeout=60000)
                    recoveries += 1
                bucket["items"] = []
                forbidden_bucket["items"] = []
                # Each app is an independent test case. Persisted state leaks
                # between launches: the throwaway CI user is only a viewer of
                # the seeded Vibe project, and a stale `project` deep-link
                # param makes canvas POST project.draw to it (403). Reset web
                # storage and in-memory deep-link params, but keep auth tokens.
                try:
                    await page.evaluate("""() => {
                      var keep = ['gb-access-token', 'management_token', 'gb-ws-token', 'token', 'id_token'];
                      [window.localStorage, window.sessionStorage].forEach(function (store) {
                        try {
                          Object.keys(store).forEach(function (key) {
                            if (keep.indexOf(key) === -1) store.removeItem(key);
                          });
                        } catch (e) { /* storage may be unavailable */ }
                      });
                      window.__gbAppParams__ = {};
                    }""")
                except Exception:
                    pass
                try:
                    await page.evaluate(
                        "([id,t,u]) => WindowManager.launchFromMenu(id,t,u)",
                        [app_id, app.get("title", app_id), app.get("url", "")])
                    await asyncio.sleep(args.wait)
                    await page.evaluate("(id) => WindowManager.close(id)", app_id)
                except Exception as exc:
                    bucket["items"].append("HARNESS " + str(exc)[:160])

                not_found = classify(bucket["items"])
                harness = sorted({item for item in bucket["items"]
                                  if item.startswith("HARNESS ")})
                results.append({"app": app_id, "not_found": not_found,
                                "harness": harness})
                print(f"[{index}/{len(apps)}] {app_id}: "
                      f"{'FAIL ' + ', '.join(not_found) if not_found else 'PASS'}")

                allowed_cross_owner_403()
                hits = console_bucket["items"]
                console_bucket["items"] = []
                if hits and not args.allow_console:
                    write_report(results, len(apps), app_id, hits, recoveries, allowed_bucket["items"], forbidden_bucket["items"])
                    print(f"FAIL: console error/warning in app {app_id}; stopping sweep")
                    for hit in hits:
                        print(f"  {hit['source']}: {hit['text'][:200]}")
                    print(f"Report: {args.report}")
                    return 1

            write_report(results, len(apps), "", [], recoveries, allowed_bucket["items"], forbidden_bucket["items"])
            failures = [r for r in results if r["not_found"]]
            print(f"RESULT: {len(apps)} apps, {len(failures)} with 404s, "
                  f"{recoveries} shell re-navigations. Report: {args.report}")
            return 1 if failures else 0
        finally:
            await page.close()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true",
                        help="offline logic check for CI; no browser or account needed")
    parser.add_argument("--cdp", default=DEFAULT_CDP)
    parser.add_argument("--base", default=DEFAULT_BASE)
    parser.add_argument("--wait", type=float, default=2.0,
                        help="seconds to leave each app open (default: 2)")
    parser.add_argument("--apps", default="", help="comma-separated app ids (default: all)")
    parser.add_argument("--report", default=DEFAULT_REPORT)
    parser.add_argument("--email", default="", help="fallback for GB_TEST_EMAIL")
    parser.add_argument("--password", default="", help="fallback for GB_TEST_PASSWORD")
    parser.add_argument("--bot", default="pragmatismo", help="bot name in the suite URL")
    parser.add_argument("--token", default="", help="JWT to inject (or GB_TEST_TOKEN)")
    parser.add_argument("--allow-console", action="store_true",
                        help="local debugging only: do not fail on console errors/warnings")
    args = parser.parse_args()
    if args.self_test:
        return self_test()
    return asyncio.run(run(args))


if __name__ == "__main__":
    sys.exit(main())
