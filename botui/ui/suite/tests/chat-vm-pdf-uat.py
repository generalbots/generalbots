#!/usr/bin/env python3
"""Blocking CI UAT: chat PDF upload -> terminal consent -> document created.

This is an automated CI harness (not a manual CDP session): it drives the real
Suite chat in one Playwright tab attached to Chrome over CDP 9222, sends one
turn at a time, reads the reply/consent card before deciding the next step,
and asserts five criteria. It exits non-zero if any criterion FAILs and writes
a machine-readable report for CI artifact upload.

The runner has no Incus, so CI installs scripts/ci-incus-stub.sh as `incus` on
PATH and starts botserver with GB_CI_INCUS_ROOT pointing at a throwaway fake
root. The harness copies the very same fixture bytes it uploads through the
composer into that fake root at data/sample-upload.pdf; the terminal command
reads those bytes and writes output/created-from-pdf.txt. The shim is a TEST
DOUBLE, not a container runtime -- no real VM or isolation is claimed.

Usage:
  python3 botui/ui/suite/tests/chat-vm-pdf-uat.py --self-test
  GB_TEST_TOKEN=... GB_CI_INCUS_ROOT=/tmp/... \
    python3 botui/ui/suite/tests/chat-vm-pdf-uat.py \
      --cdp http://127.0.0.1:9222 --base http://localhost:3000 \
      --bot mybot --pdf botui/ui/suite/tests/fixtures/sample-upload.pdf \
      --report /tmp/opencode/chat-vm-pdf-uat-report.json
"""

from __future__ import annotations

import argparse
import asyncio
import base64
import json
import os
import sys
from pathlib import Path

DEFAULT_CDP = "http://127.0.0.1:9222"
DEFAULT_BASE = "http://localhost:3000"
DEFAULT_REPORT = "/tmp/opencode/chat-vm-pdf-uat-report.json"
MARKER = "GB-PDF-MARKER-7F3A"
PDF_NAME = "sample-upload.pdf"
DOC_NAME = "created-from-pdf.txt"
INPUT_REL = Path("opt/vibe/app/data") / PDF_NAME
OUTPUT_REL = Path("opt/vibe/app/output") / DOC_NAME
CRITERIA = (
    "pdf_uploaded_and_attached",
    "terminal_consent_proposed",
    "approval_produced_ok_result",
    "document_created_from_pdf",
    "assistant_reported_document",
)


def build_command(marker: str = MARKER) -> str:
    """The exact command the consent card must execute, relative to cwd."""
    return (
        "mkdir -p output && "
        "grep -a -o '{marker}' data/{pdf} > output/{doc} && "
        "printf 'source=%s\\n' '{pdf}' >> output/{doc} && echo created"
    ).format(marker=marker, pdf=PDF_NAME, doc=DOC_NAME)


def turn_text() -> str:
    # propose_terminal_from_text() keys on this prefix and the backticked
    # payload; the LLM is intentionally bypassed for this turn.
    return "Use vm_exec to run `" + build_command() + "` to create a document from " + PDF_NAME + "."


def fixture_path(explicit: str) -> Path:
    if explicit:
        return Path(explicit)
    return Path(__file__).resolve().parent / "fixtures" / PDF_NAME


def results_table(results: dict) -> str:
    lines = ["| Criterion | Observed | Status |", "| --- | --- | --- |"]
    for key in CRITERIA:
        item = results.get(key, {"status": "FAIL", "observed": "not reached"})
        observed = str(item.get("observed", ""))[:220].replace("|", "/").replace("\n", " ")
        lines.append(f"| {key} | {observed} | {item.get('status', 'FAIL')} |")
    return "\n".join(lines)


def set_result(results: dict, key: str, ok: bool, observed: str) -> None:
    results[key] = {"status": "PASS" if ok else "FAIL", "observed": observed}


def write_report(path: str, results: dict, extra: dict) -> None:
    parent = os.path.dirname(path)
    if parent:
        os.makedirs(parent, exist_ok=True)
    failures = [key for key in CRITERIA if results.get(key, {}).get("status") != "PASS"]
    payload = {"criteria": results, "failures": failures, "ok": not failures}
    payload.update(extra)
    with open(path, "w", encoding="utf-8") as out:
        json.dump(payload, out, indent=2)


def self_test(pdf: Path) -> int:
    data = pdf.read_bytes()
    if MARKER.encode("ascii") not in data:
        print(f"SELFTEST FAIL: {pdf} does not contain {MARKER}")
        return 1
    if data[:5] != b"%PDF-":
        print(f"SELFTEST FAIL: {pdf} is not a PDF")
        return 1
    command = build_command()
    if MARKER not in command or f"data/{PDF_NAME}" not in command or f"output/{DOC_NAME}" not in command:
        print("SELFTEST FAIL: build_command() lost the marker or paths")
        return 1
    probe = {}
    set_result(probe, CRITERIA[0], False, "deliberate negative")
    if not probe[CRITERIA[0]]["status"] == "FAIL":
        print("SELFTEST FAIL: result table does not fail closed")
        return 1
    table = results_table(probe)
    if "FAIL" not in table:
        print("SELFTEST FAIL: report table hides failures")
        return 1
    print(f"SELFTEST PASS: valid PDF ({len(data)} bytes), marker, command, report OK (no browser)")
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


def stage_pdf_in_fake_root(pdf: Path, root: Path) -> str:
    """Stand in for the real VM's Drive->workspace staging step."""
    target = root / INPUT_REL
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_bytes(pdf.read_bytes())
    output = root / OUTPUT_REL
    if output.exists():
        output.unlink()
    return str(target)


async def capture_bot_text(page) -> str:
    return str(await page.evaluate(
        """() => {
            const pane = document.getElementById('messages');
            if (!pane) return '';
            const bots = Array.from(pane.querySelectorAll('.message.bot .bot-message'));
            return bots.length ? bots[bots.length - 1].textContent.trim() : '';
        }"""
    ))


async def wait_for_bot_change(page, previous: str, timeout_ms: int) -> str:
    await page.evaluate("(t) => { window.__gbUatBefore = t; }", previous)
    await page.wait_for_function(
        """() => {
            const pane = document.getElementById('messages');
            if (!pane) return false;
            const bots = Array.from(pane.querySelectorAll('.message.bot .bot-message'));
            const text = bots.length ? bots[bots.length - 1].textContent.trim() : '';
            return text.length > 0 && text !== window.__gbUatBefore ? text : false;
        }""",
        timeout=timeout_ms,
    )
    return await capture_bot_text(page)


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

    pdf = fixture_path(args.pdf)
    if not pdf.is_file():
        print(f"FATAL: fixture PDF not found: {pdf}")
        return 1
    token = token_from(args)
    if not token:
        print("FATAL: no CI JWT (set GB_TEST_TOKEN or GB_ACCOUNT_FILE)")
        return 1
    root = Path(os.environ.get("GB_CI_INCUS_ROOT", "/tmp/gb-ci-incus-root"))
    staged = stage_pdf_in_fake_root(pdf, root)

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

            # ONE turn: attach the PDF and request terminal document creation.
            await page.set_input_files("#messageFile", str(pdf))
            await page.fill("#messageInput", turn_text())
            await page.locator("#messageInput").press("Enter")

            # (a) the WS frame carried the exact uploaded bytes.
            def upload_frame():
                for frame in sent_frames:
                    if PDF_NAME in frame and "content_base64" in frame:
                        return frame
                return None

            frame = await wait_for(upload_frame, args.wait)
            ui_text = str(await page.locator("#messages").inner_text())
            attached = frame is not None and (f"📎 {PDF_NAME}" in ui_text or PDF_NAME in ui_text)
            observed = "no WS upload frame"
            if frame is not None:
                try:
                    payload = json.loads(frame)
                    files = payload.get("files") or ([payload["file"]] if payload.get("file") else [])
                    item = next(f for f in files if f.get("name") == PDF_NAME)
                    raw = base64.b64decode(item["content_base64"])
                    same = raw == pdf.read_bytes() and MARKER.encode() in raw
                    observed = (
                        f"WS frame carried {PDF_NAME} ({len(raw)} bytes), "
                        f"bytes_match_fixture={same}, chat_bubble_has_attachment={PDF_NAME in ui_text}"
                    )
                    attached = attached and same
                except Exception as exc:
                    observed = f"WS frame parse failed: {str(exc)[:120]}"
                    attached = False
            set_result(results, CRITERIA[0], bool(attached), observed)

            # (b) the deterministic proposal produces a consent card.
            await page.wait_for_selector(".gb-consent-card", timeout=args.wait * 1000)
            card_text = str(await page.locator(".gb-consent-card .gb-consent-detail").first.inner_text())
            proposed = DOC_NAME in card_text and MARKER in card_text
            set_result(
                results,
                CRITERIA[1],
                proposed,
                f"consent card command={card_text[:200]!r} expected_marker={MARKER}",
            )
            shot += 1
            await page.screenshot(path=f"/tmp/opencode/uat-vm-pdf-{shot:02d}-consent.png")

            # Approve: server-side consent record + real terminal executor.
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
                    " return e && e.textContent.indexOf('Exit code: 0') >= 0 ? true : false; }",
                    timeout=args.wait * 1000,
                )
                card_outcome = str(await page.locator(".gb-consent-outcome").first.inner_text())
            except Exception:
                card_outcome = str(await page.locator(".gb-consent-outcome").first.inner_text()) \
                    if await page.locator(".gb-consent-outcome").count() else "no outcome"
            result_post = [r for r in requests if "/result" in r.url and "/api/computer/actions/" in r.url]
            posted_status = ""
            if result_post:
                try:
                    posted_status = str(json.loads(result_post[-1].post_data or "{}").get("status", ""))
                except (ValueError, TypeError):
                    posted_status = "unparseable"
            ok_result = (
                exec_payload.get("status") == "ok"
                and exec_payload.get("exit_code") == 0
                and "Exit code: 0" in card_outcome
                and posted_status == "ok"
            )
            set_result(
                results,
                CRITERIA[2],
                ok_result,
                f"exec_status={exec_payload.get('status')} exit={exec_payload.get('exit_code')} "
                f"card={card_outcome[:80]!r} result_post={posted_status}",
            )
            shot += 1
            await page.screenshot(path=f"/tmp/opencode/uat-vm-pdf-{shot:02d}-approved.png")

            # (d) the document really exists under the fake VM root, with
            # content derived from the uploaded PDF bytes.
            output = root / OUTPUT_REL
            doc_text = output.read_text(errors="replace") if output.is_file() else ""
            created = (
                output.is_file()
                and MARKER in doc_text
                and f"source={PDF_NAME}" in doc_text
            )
            set_result(
                results,
                CRITERIA[3],
                created,
                f"staged={staged} document={output} exists={output.is_file()} content={doc_text[:120]!r}",
            )

            # (e) follow-up turn: the assistant reports the completed terminal
            # result injected by /api/computer/actions/<id>/result.
            before = await capture_bot_text(page)
            await page.fill("#messageInput", "Report the document you just created: its filename and the marker you read.")
            await page.locator("#messageInput").press("Enter")
            reply = await wait_for_bot_change(page, before, args.llm_wait * 1000)
            # The first change can be the first streamed token; wait for the
            # turn to settle before judging what the assistant reported.
            try:
                await page.wait_for_function(
                    "() => typeof ChatState === 'undefined' || !ChatState.isStreaming",
                    timeout=15000,
                )
            except Exception:
                pass
            await asyncio.sleep(1.0)
            reply = await capture_bot_text(page)
            reported = DOC_NAME in reply or MARKER in reply
            set_result(
                results,
                CRITERIA[4],
                reported,
                f"bot_reply={reply[:220]!r}",
            )
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
        "pdf": str(pdf),
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
    parser.add_argument("--pdf", default="", help="path to the sample PDF fixture")
    parser.add_argument("--report", default=DEFAULT_REPORT)
    parser.add_argument("--token", default="", help="CI JWT (or GB_TEST_TOKEN)")
    parser.add_argument("--wait", type=float, default=30.0, help="seconds to wait for UI/network steps")
    parser.add_argument("--llm-wait", type=float, default=120.0, help="seconds to wait for the LLM reply")
    args = parser.parse_args()
    if args.self_test:
        return self_test(fixture_path(args.pdf))
    return asyncio.run(run(args))


if __name__ == "__main__":
    sys.exit(main())
