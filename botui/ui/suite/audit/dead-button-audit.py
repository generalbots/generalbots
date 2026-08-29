#!/usr/bin/env python3
"""Issue #1211: Vibe dead-button sweep for suite apps.

Connects to the already-running Chrome on CDP :9222, opens each suite app
route inside the desktop shell, clicks every clickable/interactive element,
and flags "dead buttons" — elements that produce neither a network request
nor a DOM change when clicked.

Usage (from repo root):
  python3 botui/ui/suite/audit/dead-button-audit.py [--apps crm,drive] [--base URL] [--max-per-app N]

Notes:
  - Uses the running Chrome (CDP 9222). Each use-case opens a NEW tab and is
    left open as trace evidence. Never close the browser.
  - Apps are opened via the desktop route (e.g. /suite/desktop.html#/crm or
    /crm) so the desktop shell bootstraps the app modules.
  - A button is "dead" when clicking it fires no network request AND the
    visible DOM changes by less than a small threshold.
"""

from __future__ import annotations

import argparse
import asyncio
import json
import sys
from urllib.parse import urljoin

from playwright.async_api import async_playwright

try:
    import websockets  # noqa: F401  (playwright dep; imported to surface errors early)
except ImportError:  # pragma: no cover
    pass

CDP = "http://127.0.0.1:9222"
DEFAULT_BASE = "http://localhost:3000"

# Desktop routes for the most common suite apps. Each entry is (route, selector
# used to confirm the app shell opened). Apps without a shell selector fall
# back to just waiting for the network idle marker.
APPS = {
    "about": "/about",
    "admin": "/admin",
    "analytics": "/analytics",
    "attendant": "/attendant",
    "automations": "/automations",
    "banking": "/banking",
    "billing": "/billing",
    "biometry": "/biometry",
    "browser": "/browser",
    "calculator": "/calculator",
    "calendar": "/calendar",
    "campaigns": "/campaigns",
    "canvas": "/canvas",
    "chat": "/chat",
    "clock": "/clock",
    "compliance": "/compliance",
    "concierge": "/concierge",
    "crm": "/crm",
    "dashboards": "/dashboards",
    "database": "/database",
    "docs": "/docs",
    "drive": "/drive",
    "email": "/email",
    "fraud": "/fraud",
    "goals": "/goals",
    "governance": "/governance",
    "handoff": "/handoff",
    "hr": "/hr",
    "integrations": "/integrations",
    "jukebox": "/jukebox",
    "kyc": "/kyc",
    "learn": "/learn",
    "lists": "/lists",
    "mail": "/mail",
    "meet": "/meet",
    "memory": "/memory",
    "minutes": "/minutes",
    "monitoring": "/monitoring",
    "notepad": "/notepad",
    "notes": "/notes",
    "o365": "/o365",
    "paper": "/paper",
    "people": "/people",
    "photos": "/photos",
    "plan": "/plan",
    "player": "/player",
    "plugins": "/plugins",
    "pos": "/pos",
    "products": "/products",
    "project": "/project",
    "research": "/research",
    "retail": "/retail",
    "sales": "/sales",
    "settings": "/settings",
    "sheet": "/sheet",
    "slides": "/slides",
    "snapshot": "/snapshot",
    "social": "/social",
    "sources": "/sources",
    "store": "/store",
    "tasks": "/tasks",
    "tax": "/tax",
    "terminal": "/terminal",
    "tickets": "/tickets",
    "timeclock": "/timeclock",
    "timer": "/timer",
    "tools": "/tools",
    "vibe": "/vibe",
    "video": "/video",
    "vision": "/vision",
    "weather": "/weather",
    "workspace": "/workspace",
}

# Interactive elements we consider "buttons" for this sweep.
CLICKABLE = [
    "button",
    "a[href]",
    "[role='button']",
    "[onclick]",
    "[hx-get]",
    "[hx-post]",
    "[hx-put]",
    "[hx-delete]",
    "[data-action]",
    "input[type='submit']",
    "input[type='button']",
]


class DeadButtonAudit:
    def __init__(self, base: str, max_per_app: int) -> None:
        self.base = base
        self.max_per_app = max_per_app
        self.results: dict[str, list[dict]] = {}
        self.skipped: list[str] = []

    async def run(self, apps: list[str]) -> dict:
        async with async_playwright() as p:
            browser = await p.chromium.connect_over_cdp(CDP)
            ctx = browser.contexts[0] if browser.contexts else None
            if ctx is None:
                self.skipped.append("no default browser context found")
                return self.results

            for name in apps:
                route = APPS.get(name)
                if not route:
                    self.skipped.append(f"unknown app {name!r}")
                    continue
                await self._audit_one(ctx, name, route)
        return self.results

    async def _audit_one(
        self, ctx, name: str, route: str
    ) -> None:
        page = await ctx.new_page()
        url = urljoin(self.base + "/", route.lstrip("/"))
        dead: list[dict] = []
        print(f"\n=== audit app {name!r} @ {url} ===")
        try:
            await page.goto(url, wait_until="domcontentloaded", timeout=25000)
            await page.wait_for_timeout(2500)

            # Give the desktop shell a moment to bootstrap modules.
            await page.wait_for_load_state("networkidle", timeout=12000)

            # Collect unique clickable elements (by selector-visible text).
            handles = await page.locator(",".join(CLICKABLE)).all()
            seen: set[str] = set()
            targets: list = []
            for h in handles:
                try:
                    if not await h.is_visible():
                        continue
                    tag = await h.evaluate("el => el.tagName.toLowerCase()")
                    txt = (await h.inner_text() or "").strip() or (
                        await h.get_attribute("title") or ""
                    )
                    key = f"{tag}:{txt}"
                    if key in seen:
                        continue
                    seen.add(key)
                    targets.append((key, h))
                except Exception:
                    continue

            for key, h in targets[: self.max_per_app]:
                result = await self._probe(page, h, key)
                if result is not None:
                    dead.append(result)

        except Exception as e:  # page-level crash/timeout
            self.skipped.append(f"{name}: page error {type(e).__name__}: {e}")
        finally:
            # Keep the tab open as evidence (do not close).
            pass

        self.results[name] = dead
        print(f"  -> {len(dead)} dead button(s) out of {len(targets[: self.max_per_app])} probed")
        for d in dead:
            print(f"     DEAD: {d['element']} :: {d['hint']}")

    async def _probe(self, page, handle, key: str) -> dict | None:
        try:
            before = await self._dom_signature(page)

            async def _net_mark():
                await page.wait_for_timeout(300)

            net_fired = False
            page.on("request", lambda req: _mark(req) if req.resource_type in (
                "xhr", "fetch", "document",
            ) else None)

            def _mark(req):  # noqa: ANN001
                nonlocal net_fired
                net_fired = True

            try:
                await handle.scroll_into_view_if_needed(timeout=4000)
            except Exception:
                pass
            try:
                await handle.click(timeout=4000)
            except Exception:
                # Click itself failed (element not actionable) — count as suspect.
                return {
                    "element": key,
                    "hint": "click raised an error / not actionable",
                }

            await page.wait_for_timeout(700)
            after = await self._dom_signature(page)

            if net_fired:
                return None  # caused a network request -> alive
            if abs(after - before) > 4:
                return None  # visible DOM changed -> alive

            return {
                "element": key,
                "hint": "no network request and no DOM change",
            }
        except Exception as e:
            return {"element": key, "hint": f"probe error {type(e).__name__}: {e}"}

    async def _dom_signature(self, page) -> float:
        """Cheap DOM fingerprint: count of interactive nodes plus body length."""
        try:
            return await page.evaluate(
                "() => document.querySelectorAll('button,a[href],[role=button]').length + document.body.innerText.length / 1000"
            )
        except Exception:
            return 0.0


async def main() -> int:
    parser = argparse.ArgumentParser(description="Suite dead-button sweep (#1211)")
    parser.add_argument("--base", default=DEFAULT_BASE, help="suite base URL")
    parser.add_argument("--apps", default=None, help="comma-separated subset of apps")
    parser.add_argument("--max-per-app", type=int, default=40, help="buttons probed per app")
    parser.add_argument("--report", default="/tmp/dead-buttons-report.json", help="output JSON path")
    args = parser.parse_args()

    apps = [a.strip() for a in args.apps.split(",") if a.strip()] if args.apps else list(APPS.keys())

    audit = DeadButtonAudit(args.base, args.max_per_app)
    results = await audit.run(apps)

    report = {"results": results, "skipped": audit.skipped, "probed_apps": len(apps)}
    with open(args.report, "w", encoding="utf-8") as f:
        json.dump(report, f, indent=2, ensure_ascii=False)

    total_dead = sum(len(v) for v in results.values())
    print(f"\n==== SUMMARY: {total_dead} dead button(s) across {len(apps)} app(s) ====")
    if audit.skipped:
        print("Skipped:", *audit.skipped, sep="\n  - ")
    return 0 if total_dead == 0 else 1


if __name__ == "__main__":
    sys.exit(asyncio.run(main()))