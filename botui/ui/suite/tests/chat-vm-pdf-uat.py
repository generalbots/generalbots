#!/usr/bin/env python3
"""Blocking CI UAT: a chat phrase makes the desktop agent VM create a PDF.

The user sends one natural turn -- "make me a pdf of brazil in one paragraph" --
and the chat agent VM produces a PDF the user can access. This harness drives
the real Suite chat in one Playwright tab attached to Chrome over CDP 9222,
one turn at a time, reads the reply/consent card before deciding the next step,
and asserts five criteria. It exits non-zero if any criterion FAILs and writes
a machine-readable report for CI artifact upload.

GitHub-hosted runners have no Incus, so CI installs scripts/ci-incus-stub.sh as
`incus` on PATH and starts botserver with GB_CI_INCUS_ROOT pointing at a
throwaway fake root. The stub executes the agent's shell command locally under
that root, so the produced PDF is inspected there. The shim is a TEST DOUBLE,
not a container runtime -- no real VM or isolation is claimed.

Usage:
  python3 botui/ui/suite/tests/chat-vm-pdf-uat.py --self-test
  GB_TEST_TOKEN=... GB_CI_INCUS_ROOT=/tmp/... \
    python3 botui/ui/suite/tests/chat-vm-pdf-uat.py \
      --cdp http://127.0.0.1:9222 --base http://localhost:3000 \
      --bot mybot --report /tmp/opencode/chat-vm-pdf-uat-report.json
"""

from __future__ import annotations

import argparse
import asyncio
import json
import os
import sys
import time
from pathlib import Path

DEFAULT_CDP = "http://127.0.0.1:9222"
DEFAULT_BASE = "http://localhost:3000"
DEFAULT_REPORT = "/tmp/opencode/chat-vm-pdf-uat-report.json"
PHRASE = "make me a pdf of brazil in one paragraph"
VM_APP = Path("opt/vibe/app")
CRITERIA = (
    "phrase_sent_to_chat",
    "agent_vm_action_proposed",
    "approval_produced_ok_result",
    "pdf_created_in_agent_vm",
    "assistant_reported_accessible_pdf",
)


def vm_root() -> Path:
    return Path(os.environ.get("GB_CI_INCUS_ROOT", "/tmp/gb-ci-incus-root"))


def turn_text() -> str:
    """The exact user phrase -- no command, no upload, no LLM bypass."""
    return PHRASE


def set_result(results: dict, key: str, ok: bool, observed: str) -> None:
    results[key] = {"status": "PASS" if ok else "FAIL", "observed": observed}


def results_table(results: dict) -> str:
    lines = ["| Criterion | Observed | Status |", "| --- | --- | --- |"]
    for key in CRITERIA:
        item = results.get(key, {"status": "FAIL", "observed": "not reached"})
        observed = str(item.get("observed", ""))[:220].replace("|", "/").replace("\n", " ")
        lines.append(f"| {key} | {observed} | {item.get('status', 'FAIL')} |")
    return "\n".join(lines)


def write_report(path: str, results: dict, extra: dict) -> None:
    parent = os.path.dirname(path)
    if parent:
        os.makedirs(parent, exist_ok=True)
    failures = [key for key in CRITERIA if results.get(key, {}).get("status") != "PASS"]
    payload = {"criteria": results, "failures": failures, "ok": not failures,
               "phrase": PHRASE}
    payload.update(extra)
    with open(path, "w", encoding="utf-8") as out:
        json.dump(payload, out, indent=2)


def find_pdfs(root: Path, since: float):
    """Any PDF the agent wrote under the fake VM workspace after the turn."""
    app = root / VM_APP
    if not app.is_dir():
        return []
    found = []
    for candidate in sorted(app.rglob("*.pdf")):
        try:
            stat = candidate.stat()
            if stat.st_mtime < since - 2 or stat.st_size < 200:
                continue
            head = candidate.read_bytes()[:5]
            if head != b"%PDF-":
                continue
            body = candidate.read_bytes()
            found.append({
                "path": str(candidate),
                "bytes": stat.st_size,
                "names_brazil": (b"brazil" in body.lower() or b"brasil" in body.lower()),
            })
        except OSError:
            continue
    return found


def self_test() -> int:
    if turn_text() != PHRASE or "pdf" not in PHRASE or "brazil" not in PHRASE:
        print("SELFTEST FAIL: the phrase is not the user's PDF request")
        return 1
    probe = {}
    set_result(probe, CRITERIA[0], False, "deliberate negative")
    if probe[CRITERIA[0]]["status"] != "FAIL":
        print("SELFTEST FAIL: result table does not fail closed")
        return 1
    if "FAIL" not in results_table(probe):
        print("SELFTEST FAIL: report table hides failures")
        return 1
    root = vm_root()
    app = root / VM_APP
    if not app.is_dir():
        app.mkdir(parents=True, exist_ok=True)
    sample = app / "output" / "self-test.pdf"
    sample.parent.mkdir(parents=True, exist_ok=True)
    sample.write_bytes(b"%PDF-1.4\n" + b"x" * 400 + b"Brazil")
    found = find_pdfs(root, time.time() - 10)
    sample.unlink()
    if not found or not found[0]["names_brazil"]:
        print("SELFTEST FAIL: PDF discovery did not find the agent-produced file")
        return 1
    print("SELFTEST PASS: phrase, report and VM PDF discovery OK (no browser)")
    return 0


def token_from(args) -> str:
    token = args.token or os.environ.get("GB_TEST_TOKEN", "")
    if token:
        return token
    account = Path(os.environ.get("GB_ACCOUNT_FILE", "/tmp/gb-smoke-account.json"))
    if account.is_file():
        try:
            return json.loads(account.read_text()).get("token", "")
        except (OSError, ValueError):
            return ""
    return ""


async def capture_bot_text(page) -> str:
    return str(await page.evaluate(
        """() => {
            const pane = document.getElementById('messages');
            if (!pane) return '';
            const bots = Array.from(pane.querySelectorAll('.message.bot .bot-message'));
            return bots.length ? bots[bots.length - 1].textContent.trim() : '';
        }"""
    ))


async def wait_for(predicate, timeout_s: float, interval: float = 0.35):
    """Small one-shot polls for network captures (not a browser state loop)."""
    for _ in range(max(1, int(timeout_s / interval))):
        value = predicate()
        if value:
            return value
        await asyncio.sleep(interval)
    return None


async def run(args) -> int:
    from playwright.async_api import async_playwright

    token = token_from(args)
    if not token:
        print("FATAL: no CI JWT (set GB_TEST_TOKEN or GB_ACCOUNT_FILE)")
        return 1
    root = vm_root()
    root.joinpath(VM_APP, "output").mkdir(parents=True, exist_ok=True)
    started = time.time()

    results: dict = {key: {"status": "FAIL", "observed": "not reached"} for key in CRITERIA}
    responses: list = []
    requests: list = []
    sent_frames: list = []
    shot = 0

    async with async_playwright() as p:
        try:
            browser = await p.chromium.connect_over_cdp(args.cdp)
        except Exception as exc:
            print(f"FATAL: cannot attach to CDP {args.cdp}: {str(exc)[:200]}")
            return 1
        ctx = browser.contexts[0] if browser.contexts else await browser.new_context()
        page = await ctx.new_page()

        def on_response(response):
            responses.append(response)

        def on_request(request):
            requests.append(request)

        def on_ws(websocket):
            def on_frame(payload):
                sent_frames.append(str(payload))
            websocket.on("framesent", on_frame)

        page.on("response", on_response)
        page.on("request", on_request)
        page.on("websocket", on_ws)
        await page.add_init_script(
            "(() => { const t = " + json.dumps(token) + ";"
            " localStorage.setItem('gb-access-token', t);"
            " localStorage.setItem('management_token', t);"
            " localStorage.setItem('gb-ws-token', t);"
            " })()"
        )

        try:
            target = args.base.rstrip("/") + "/" + args.bot.strip("/") + "/"
            await page.goto(target, wait_until="domcontentloaded", timeout=60000)
            await page.wait_for_function(
                """() => document.getElementById('messageInput') &&
                    typeof ChatState !== 'undefined' && !!ChatState.currentSessionId &&
                    ChatState.ws && ChatState.ws.readyState === 1""",
                timeout=60000,
            )
            shot += 1
            await page.screenshot(path=f"/tmp/opencode/uat-vm-pdf-{shot:02d}-loaded.png")

            # ONE user turn: the natural phrase, nothing else.
            await page.fill("#messageInput", turn_text())
            await page.locator("#messageInput").press("Enter")

            def phrase_frame():
                for frame in sent_frames:
                    if PHRASE in frame:
                        return frame
                return None

            frame = await wait_for(phrase_frame, args.wait)
            user_text = str(await page.locator("#messages").inner_text())
            sent = frame is not None and PHRASE in user_text
            set_result(results, CRITERIA[0], bool(sent),
                       f"ws_frame={'yes' if frame else 'no'} user_bubble_has_phrase={PHRASE in user_text}")

            # The agent VM action reaches a consent card; approve it.
            try:
                await page.wait_for_selector(".gb-consent-card", timeout=args.llm_wait * 1000)
                card_text = str(await page.locator(".gb-consent-card").first.inner_text())
                proposed = "pdf" in card_text.lower()
                set_result(results, CRITERIA[1], proposed, f"consent_card={card_text[:220]!r}")
            except Exception as exc:
                set_result(results, CRITERIA[1], False, f"no consent card: {str(exc)[:120]}")
            shot += 1
            await page.screenshot(path=f"/tmp/opencode/uat-vm-pdf-{shot:02d}-consent.png")

            if await page.locator('.gb-consent-card .gb-consent-btn[data-decision="allow"]').count():
                await page.locator(
                    '.gb-consent-card .gb-consent-btn[data-decision="allow"]'
                ).first.click()

                def exec_response():
                    for response in responses:
                        if "/api/terminal/exec" in response.url:
                            return response
                    return None

                response = await wait_for(exec_response, args.wait)
                exec_payload = {}
                if response is not None:
                    try:
                        exec_payload = await response.json()
                    except Exception as exc:
                        exec_payload = {"error": str(exc)[:120]}
                card_outcome = ""
                try:
                    await page.wait_for_function(
                        "() => { const e = document.querySelector('.gb-consent-outcome');"
                        " return e && /exit code:\\s*0/i.test(e.textContent) ? true : false; }",
                        timeout=args.wait * 1000,
                    )
                except Exception:
                    pass
                if await page.locator(".gb-consent-outcome").count():
                    card_outcome = str(await page.locator(".gb-consent-outcome").first.inner_text())
                result_post = [r for r in requests if "/result" in r.url and "/api/computer/actions/" in r.url]
                posted_status = ""
                if result_post:
                    try:
                        posted_status = str(json.loads(result_post[-1].post_data or "{}").get("status", ""))
                    except (ValueError, TypeError):
                        posted_status = "unparseable"
                ok_result = (
                    (exec_payload.get("status") == "ok" and exec_payload.get("exit_code") == 0)
                    or "exit code: 0" in card_outcome.lower()
                ) and (not posted_status or posted_status == "ok")
                set_result(results, CRITERIA[2], bool(ok_result),
                           f"exec_status={exec_payload.get('status')} exit={exec_payload.get('exit_code')} "
                           f"card={card_outcome[:80]!r} result_post={posted_status}")
            else:
                set_result(results, CRITERIA[2], False, "no approval button on the consent card")
            shot += 1
            await page.screenshot(path=f"/tmp/opencode/uat-vm-pdf-{shot:02d}-approved.png")

            # Wait for the agent VM to finish writing the PDF, then inspect it.
            pdfs = []
            for _ in range(max(1, int(args.llm_wait / 2))):
                pdfs = find_pdfs(root, started)
                if pdfs:
                    break
                await asyncio.sleep(0.5)
            detail = "; ".join(
                f"{p['path']} ({p['bytes']} bytes, brazil={p['names_brazil']})" for p in pdfs)
            set_result(results, CRITERIA[3], bool(pdfs),
                       f"vm_root={root} pdfs={detail[:220] or 'none'}")

            # Follow-up turn: the assistant reports the produced document.
            await page.fill(
                "#messageInput",
                "Report the PDF you just created: its filename and where I can open it.",
            )
            await page.locator("#messageInput").press("Enter")
            await asyncio.sleep(min(args.llm_wait, 30.0))
            reply = await capture_bot_text(page)
            ui_text = str(await page.locator("#messages").inner_text())
            reported = ".pdf" in (reply + ui_text).lower()
            set_result(results, CRITERIA[4], reported, f"bot_reply={reply[:220]!r}")
            shot += 1
            await page.screenshot(path=f"/tmp/opencode/uat-vm-pdf-{shot:02d}-report.png")

        except Exception as exc:
            set_result(results, "harness", False, f"{type(exc).__name__}: {str(exc)[:220]}")
        finally:
            await page.close()

    write_report(args.report, results, {
        "cdp": args.cdp,
        "base": args.base,
        "bot": args.bot,
        "phrase": PHRASE,
        "fake_root": str(root),
        "exec_response": exec_payload if "exec_payload" in locals() else {},
    })
    print(results_table(results))
    failures = [key for key in CRITERIA if results.get(key, {}).get("status") != "PASS"]
    if "harness" in results:
        failures.append("harness")
    print(f"RESULT: {'PASS' if not failures else 'FAIL'} ({len(CRITERIA) - len(failures)}/{len(CRITERIA)} criteria). "
          f"Report: {args.report}")
    return 1 if failures else 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true", help="offline logic check; no browser")
    parser.add_argument("--cdp", default=DEFAULT_CDP)
    parser.add_argument("--base", default=DEFAULT_BASE)
    parser.add_argument("--bot", default="default", help="bot name in the suite URL")
    parser.add_argument("--report", default=DEFAULT_REPORT)
    parser.add_argument("--token", default="", help="CI JWT (or GB_TEST_TOKEN)")
    parser.add_argument("--wait", type=float, default=30.0, help="seconds to wait for UI/network steps")
    parser.add_argument("--llm-wait", type=float, default=120.0, help="seconds to wait for the agent")
    args = parser.parse_args()
    if args.self_test:
        return self_test()
    return asyncio.run(run(args))


if __name__ == "__main__":
    sys.exit(main())
