#!/usr/bin/env python3
"""Fragmented CDP viewport gate (#2421).

Opens one new tab per viewport against an already-running Chrome CDP endpoint
and prints one compact JSON line per viewport with the geometry the mobile
matrix needs: innerWidth/innerHeight, document scrollWidth vs innerWidth,
horizontal-overflow boolean and a screenshot path under /tmp/opencode/.

Fragmentation rules (AGENTS.md): each viewport is a separate function call, no
viewport loop hides state, existing tabs are never navigated or closed, and the
only websocket opened belongs to the tab this script created. Screenshots and
tabs are left in place as evidence.

Usage:
    python3 scripts/check_viewports.py --self-test
    python3 scripts/check_viewports.py --cdp http://127.0.0.1:9223 \
        --url http://localhost:3000/default --out-dir /tmp/opencode
"""

from __future__ import annotations

import argparse
import base64
import contextlib
import io
import json
import sys
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_CDP = "http://127.0.0.1:9223"
DEFAULT_URL = "http://localhost:3000/default"
DEFAULT_OUT = "/tmp/opencode"
RECORD_KEYS = (
    "viewport",
    "url",
    "inner_width",
    "inner_height",
    "scroll_width",
    "client_width",
    "horizontal_overflow",
    "pointer",
    "screenshot",
)
VIEWPORTS = (
    {"name": "phone", "width": 390, "height": 844, "pointer": "coarse", "mobile": True, "scale": 3},
    {"name": "phone-small", "width": 320, "height": 568, "pointer": "coarse", "mobile": True, "scale": 3},
    {"name": "tablet-p", "width": 768, "height": 1024, "pointer": "coarse", "mobile": True, "scale": 3},
    {"name": "tablet-l", "width": 1024, "height": 768, "pointer": "coarse", "mobile": True, "scale": 3},
    {"name": "desktop", "width": 1440, "height": 900, "pointer": "fine", "mobile": False, "scale": 1},
)
METRICS_JS = """(() => {
  const root = document.scrollingElement || document.documentElement;
  return {
    inner_width: window.innerWidth,
    inner_height: window.innerHeight,
    scroll_width: root.scrollWidth,
    client_width: root.clientWidth
  };
})()"""


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--cdp", default=DEFAULT_CDP, help="Chrome CDP endpoint (default: %(default)s)")
    parser.add_argument("--url", default=DEFAULT_URL, help="page to open in each new tab (default: %(default)s)")
    parser.add_argument("--out-dir", default=DEFAULT_OUT, help="screenshot destination (default: %(default)s)")
    parser.add_argument("--self-test", action="store_true", help="validate args and JSON shape without a browser")
    return parser


def emit(payload: dict[str, Any]) -> None:
    print(json.dumps(payload, sort_keys=True, separators=(",", ":")))


def make_record(
    viewport: dict[str, Any],
    target_url: str,
    metrics: dict[str, int] | None,
    screenshot: str | None,
    error: str | None = None,
) -> dict[str, Any]:
    record: dict[str, Any] = {
        "viewport": viewport["name"],
        "url": target_url,
        "inner_width": metrics["inner_width"] if metrics else None,
        "inner_height": metrics["inner_height"] if metrics else None,
        "scroll_width": metrics["scroll_width"] if metrics else None,
        "client_width": metrics["client_width"] if metrics else None,
        "horizontal_overflow": metrics["scroll_width"] > metrics["inner_width"] if metrics else None,
        "pointer": viewport["pointer"],
        "screenshot": screenshot,
    }
    if error is not None:
        record["error"] = error
    return record


def self_test() -> int:
    checks: list[str] = []
    parser = build_parser()
    defaults = parser.parse_args([])
    expected_defaults = (DEFAULT_CDP, DEFAULT_URL, DEFAULT_OUT, False)
    actual_defaults = (defaults.cdp, defaults.url, defaults.out_dir, defaults.self_test)
    if actual_defaults != expected_defaults:
        emit({"self_test": "FAIL", "check": "defaults", "expected": expected_defaults, "actual": actual_defaults})
        return 1
    checks.append("defaults")

    custom = parser.parse_args(
        [
            "--cdp", "http://127.0.0.1:9333",
            "--url", "http://localhost:4000/cloud",
            "--out-dir", "/tmp/other",
            "--self-test",
        ]
    )
    expected_custom = ("http://127.0.0.1:9333", "http://localhost:4000/cloud", "/tmp/other", True)
    actual_custom = (custom.cdp, custom.url, custom.out_dir, custom.self_test)
    custom_missing_flag = parser.parse_args(["--cdp", "http://127.0.0.1:9333"]).self_test
    if actual_custom != expected_custom or custom_missing_flag:
        emit({"self_test": "FAIL", "check": "custom args", "expected": expected_custom, "actual": actual_custom})
        return 1
    checks.append("custom args")

    sample = make_record(
        VIEWPORTS[0],
        DEFAULT_URL,
        {"inner_width": 390, "inner_height": 844, "scroll_width": 390, "client_width": 390},
        "/tmp/opencode/viewport_phone_390x844.png",
    )
    buffer = io.StringIO()
    with contextlib.redirect_stdout(buffer):
        emit(sample)
    try:
        decoded = json.loads(buffer.getvalue())
    except json.JSONDecodeError as error:
        emit({"self_test": "FAIL", "check": "JSON output", "error": str(error)})
        return 1
    if tuple(sorted(decoded)) != tuple(sorted(RECORD_KEYS)):
        emit({"self_test": "FAIL", "check": "JSON keys", "expected": list(RECORD_KEYS), "actual": list(decoded)})
        return 1
    if (
        decoded["horizontal_overflow"] is not False
        or decoded["inner_width"] != 390
        or decoded["screenshot"] != sample["screenshot"]
        or not isinstance(decoded["horizontal_overflow"], bool)
    ):
        emit({"self_test": "FAIL", "check": "JSON values", "record": decoded})
        return 1
    checks.append("JSON shape")

    failure = make_record(VIEWPORTS[0], DEFAULT_URL, None, None, "boom")
    if failure.get("error") != "boom" or failure["horizontal_overflow"] is not None:
        emit({"self_test": "FAIL", "check": "error shape", "record": failure})
        return 1
    checks.append("error shape")

    names = [viewport["name"] for viewport in VIEWPORTS]
    expected_names = ["phone", "phone-small", "tablet-p", "tablet-l", "desktop"]
    if names != expected_names:
        emit({"self_test": "FAIL", "check": "viewport matrix", "expected": expected_names, "actual": names})
        return 1
    checks.append("viewport matrix")

    emit({
        "self_test": "ok",
        "checks": checks,
        "viewports": [f"{viewport['name']}={viewport['width']}x{viewport['height']}" for viewport in VIEWPORTS],
    })
    return 0


def cdp_http(cdp: str, path: str, method: str = "GET") -> dict[str, Any]:
    request = urllib.request.Request(cdp.rstrip("/") + path, method=method)
    try:
        with urllib.request.urlopen(request, timeout=15) as response:
            body = response.read().decode("utf-8")
    except (urllib.error.URLError, OSError) as error:
        raise RuntimeError(f"CDP HTTP {path} failed: {error}") from error
    if not body:
        return {}
    try:
        decoded = json.loads(body)
    except json.JSONDecodeError as error:
        raise RuntimeError(f"CDP HTTP {path} returned non-JSON") from error
    return decoded if isinstance(decoded, dict) else {}


def open_tab(cdp: str, target_url: str) -> tuple[str, str]:
    path = "/json/new?" + urllib.parse.quote(target_url, safe=":/?=&%")
    info = cdp_http(cdp, path, method="PUT")
    ws_url = info.get("webSocketDebuggerUrl")
    target_id = info.get("id")
    if not isinstance(ws_url, str) or not ws_url or not isinstance(target_id, str):
        raise RuntimeError("CDP did not return webSocketDebuggerUrl/id for the new tab")
    return ws_url, target_id


def close_tab(cdp: str, target_id: str) -> None:
    """Close only the tab this helper created; never another agent's tab."""
    try:
        cdp_http(cdp, "/json/close/" + urllib.parse.quote(target_id, safe=""))
    except RuntimeError:
        pass


class TabSession:
    """One websocket for one tab created by open_tab; never closes a tab."""

    def __init__(self, ws_url: str) -> None:
        try:
            import websocket
        except ImportError as error:
            raise RuntimeError("websocket-client is required for CDP runs") from error
        try:
            self._ws = websocket.create_connection(ws_url, timeout=60, suppress_origin=True)
        except Exception as error:
            raise RuntimeError(f"CDP websocket connect failed: {error}") from error
        self._next_id = 0

    def call(self, method: str, params: dict[str, Any] | None = None) -> dict[str, Any]:
        self._next_id += 1
        ident = self._next_id
        try:
            self._ws.send(json.dumps({"id": ident, "method": method, "params": params or {}}))
            while True:
                raw = self._ws.recv()
                if not raw:
                    raise RuntimeError(f"CDP {method} closed the connection")
                message = json.loads(raw)
                if message.get("id") != ident:
                    continue
                if "error" in message:
                    raise RuntimeError(f"CDP {method} failed: {message['error']}")
                result = message.get("result", {})
                return result if isinstance(result, dict) else {}
        except RuntimeError:
            raise
        except Exception as error:
            raise RuntimeError(f"CDP {method} failed: {error}") from error

    def close(self) -> None:
        self._ws.close()


def apply_viewport(session: TabSession, viewport: dict[str, Any]) -> None:
    session.call("Emulation.setDeviceMetricsOverride", {
        "width": viewport["width"],
        "height": viewport["height"],
        "deviceScaleFactor": viewport["scale"],
        "mobile": viewport["mobile"],
    })
    session.call("Emulation.setTouchEmulationEnabled", {
        "enabled": viewport["mobile"],
        "maxTouchPoints": 1,
    })


def read_metrics(session: TabSession) -> dict[str, int]:
    response = session.call("Runtime.evaluate", {"expression": METRICS_JS, "returnByValue": True})
    value = response.get("result", {}).get("value", {})
    if not isinstance(value, dict):
        raise RuntimeError("CDP Runtime.evaluate did not return metrics")
    return {key: int(value[key]) for key in ("inner_width", "inner_height", "scroll_width", "client_width")}


def capture_screenshot(session: TabSession, out_dir: Path, viewport: dict[str, Any]) -> str:
    result = session.call("Page.captureScreenshot", {"format": "png", "captureBeyondViewport": False})
    data = result.get("data")
    if not isinstance(data, str) or not data:
        raise RuntimeError("CDP Page.captureScreenshot returned no image data")
    path = out_dir / f"viewport_{viewport['name']}_{viewport['width']}x{viewport['height']}.png"
    path.write_bytes(base64.b64decode(data))
    return str(path)


def wait_for_page(session: TabSession) -> None:
    """Wait until the tab has left about:blank and finished its first layout.

    `PUT /json/new?URL` does not reliably navigate, and reading window.innerWidth
    before a real page commits returns Chrome's 980px fallback (#2421). This is a
    bounded series of one-shot reads, never a long sleeping loop. The destination
    may be an auth redirect, so any non-blank committed URL is accepted.
    """
    import time

    for _ in range(20):
        try:
            state = session.call("Runtime.evaluate", {
                "expression": "document.readyState + '|' + location.href",
                "returnByValue": True,
            })
            value = state.get("result", {}).get("value", "")
            if isinstance(value, str) and value.startswith("complete|") and "about:blank" not in value:
                return
        except RuntimeError:
            pass
        time.sleep(0.25)
    raise RuntimeError("page did not commit a URL and reach readyState=complete")


def check_viewport(args: argparse.Namespace, viewport: dict[str, Any], out_dir: Path) -> bool:
    session: TabSession | None = None
    target_id: str | None = None
    try:
        ws_url, target_id = open_tab(args.cdp, "about:blank")
        session = TabSession(ws_url)
        apply_viewport(session, viewport)
        session.call("Page.enable")
        session.call("Page.navigate", {"url": args.url})
        wait_for_page(session)
        metrics = read_metrics(session)
        screenshot = capture_screenshot(session, out_dir, viewport)
        record = make_record(viewport, args.url, metrics, screenshot)
        emit(record)
        return not record["horizontal_overflow"]
    except Exception as error:
        emit(make_record(viewport, args.url, None, None, str(error)))
        return False
    finally:
        if session is not None:
            session.close()
        if target_id is not None:
            close_tab(args.cdp, target_id)


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    if args.self_test:
        return self_test()
    out_dir = Path(args.out_dir)
    if out_dir.resolve().is_relative_to(ROOT):
        emit({"error": "--out-dir must be outside the repository"})
        return 2
    out_dir.mkdir(parents=True, exist_ok=True)
    results = [
        check_viewport(args, VIEWPORTS[0], out_dir),
        check_viewport(args, VIEWPORTS[1], out_dir),
        check_viewport(args, VIEWPORTS[2], out_dir),
        check_viewport(args, VIEWPORTS[3], out_dir),
        check_viewport(args, VIEWPORTS[4], out_dir),
    ]
    return 0 if all(results) else 1


if __name__ == "__main__":
    sys.exit(main())
