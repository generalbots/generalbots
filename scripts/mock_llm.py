#!/usr/bin/env python3
"""Deterministic OpenAI-compatible mock for local chat tests (#1756, #2166, #2173).

Hardened for CI reliability (including windows-latest):
- Threaded server, reusable address, explicit timeouts, unbuffered/log-safe behavior
- OpenAI-compatible endpoints: /, /health, /ready, /v1/models, /v1/chat/completions, /v1/embeddings
- Supports streaming (SSE) and non-streaming chat completions
- Deterministic default reply aligned with desktop-local smoke/e2e harness
- Request counters exposed via /health for smoke assertions/artifacts

Modes:
    MOCK_MODE=answer   (default) deterministic assistant content, one completion
    MOCK_MODE=tool     one tool_call (MOCK_TOOL, default drive_fetch) then stop

Env:
    MOCK_HOST=127.0.0.1   MOCK_PORT=8090
    MOCK_MODEL=mock-gpt   MOCK_REPLY="Hello from mock LLM."
    MOCK_MODE=answer|tool MOCK_TOOL=drive_fetch
    MOCK_LOG=/path/to/mock-llm.log  (optional structured request log file)

Usage:
    python -u scripts/mock_llm.py
    MOCK_PORT=18080 MOCK_REPLY="Hello from mock LLM." python -u scripts/mock_llm.py
"""
from __future__ import annotations

import hashlib
import json
import os
import signal
import sys
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from typing import Any

HOST = os.environ.get("MOCK_HOST", "127.0.0.1")
PORT = int(os.environ.get("MOCK_PORT", "8090"))
MODEL = os.environ.get("MOCK_MODEL", "mock-gpt")
MODE = os.environ.get("MOCK_MODE", "answer")
TOOL = os.environ.get("MOCK_TOOL", "drive_fetch")
DEFAULT_REPLY = os.environ.get("MOCK_REPLY", "Hello from mock LLM.")
LOG_PATH = os.environ.get("MOCK_LOG", "").strip()

_STATE_LOCK = threading.Lock()
_STATE: dict[str, Any] = {
    "requests": 0,
    "chat_requests": 0,
    "embedding_requests": 0,
    "stream_requests": 0,
    "non_stream_requests": 0,
    "errors": 0,
    "started_at": int(time.time()),
}


def _inc(key: str, amount: int = 1) -> None:
    with _STATE_LOCK:
        _STATE[key] = int(_STATE.get(key, 0)) + amount


def _snapshot() -> dict[str, Any]:
    with _STATE_LOCK:
        return dict(_STATE)


def _now() -> int:
    return int(time.time())


def _write_log(line: str) -> None:
    msg = line.rstrip()
    print(msg, file=sys.stderr, flush=True)
    if LOG_PATH:
        try:
            with open(LOG_PATH, "a", encoding="utf-8") as f:
                f.write(msg + "\n")
        except Exception:
            # Logging must never break mock responses.
            pass


def _user_text(messages: Any) -> str:
    if not isinstance(messages, list):
        return ""
    for item in reversed(messages):
        if not isinstance(item, dict):
            continue
        if item.get("role") != "user":
            continue
        content = item.get("content")
        if isinstance(content, str):
            return content
        if isinstance(content, list):
            parts: list[str] = []
            for part in content:
                if isinstance(part, dict):
                    if part.get("type") == "text" and isinstance(part.get("text"), str):
                        parts.append(part["text"])
                    elif isinstance(part.get("text"), str):
                        parts.append(part["text"])
                elif isinstance(part, str):
                    parts.append(part)
            return "".join(parts)
        return str(content or "")
    return ""


def _reply_for(user_text: str, digest: str) -> str:
    # Preserve #1756 deterministic scenarios used by local/e2e tests.
    if "ATTACH-TEST" in user_text:
        return "Here is the generated file: [sample.docx](vmfile:output/sample.docx)"
    if "[Steering]" in user_text:
        return "DONE - steering applied."
    # #2166/#2173 smoke expects a stable, non-fallback assistant reply.
    # Keep DEFAULT_REPLY deterministic and include a digest marker for traceability.
    if DEFAULT_REPLY:
        return DEFAULT_REPLY
    return f"MOCK-OK {digest}"


class MockServer(ThreadingHTTPServer):
    daemon_threads = True
    allow_reuse_address = True


class Handler(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"
    server_version = "GBMockLLM/1.1"
    sys_version = ""

    def log_message(self, fmt: str, *args: Any) -> None:  # noqa: A003
        # Keep stdout clean; structured access is emitted via _write_log when useful.
        return

    def _send_bytes(self, data: bytes, code: int = 200, content_type: str = "application/json") -> None:
        try:
            self.send_response(code)
            self.send_header("Content-Type", content_type)
            self.send_header("Cache-Control", "no-store")
            self.send_header("Content-Length", str(len(data)))
            self.end_headers()
            self.wfile.write(data)
        except (BrokenPipeError, ConnectionAbortedError, ConnectionResetError, OSError):
            _inc("errors")

    def _send_json(self, payload: dict[str, Any], code: int = 200) -> None:
        data = json.dumps(payload, separators=(",", ":"), ensure_ascii=False).encode("utf-8")
        self._send_bytes(data, code=code, content_type="application/json")

    def _send_empty(self, code: int = 200) -> None:
        try:
            self.send_response(code)
            self.send_header("Content-Length", "0")
            self.end_headers()
        except (BrokenPipeError, ConnectionAbortedError, ConnectionResetError, OSError):
            _inc("errors")

    def _read_body(self) -> bytes:
        # Support Content-Length and (rare) chunked transfer for robustness.
        te = (self.headers.get("Transfer-Encoding") or "").lower()
        if "chunked" in te:
            chunks: list[bytes] = []
            try:
                while True:
                    line = self.rfile.readline()
                    if not line:
                        break
                    size_str = line.split(b";", 1)[0].strip()
                    try:
                        size = int(size_str, 16)
                    except Exception:
                        break
                    if size <= 0:
                        # consume trailing headers terminator
                        while True:
                            tail = self.rfile.readline()
                            if not tail or tail in (b"\r\n", b"\n"):
                                break
                        break
                    chunks.append(self.rfile.read(size))
                    self.rfile.read(2)  # CRLF
                return b"".join(chunks)
            except Exception:
                _inc("errors")
                return b"".join(chunks)

        length_raw = self.headers.get("Content-Length", "0") or "0"
        try:
            length = int(length_raw)
        except ValueError:
            length = 0
        if length <= 0:
            return b""
        try:
            return self.rfile.read(length)
        except Exception:
            _inc("errors")
            return b""

    def _send_sse_headers(self) -> None:
        self.send_response(200)
        self.send_header("Content-Type", "text/event-stream; charset=utf-8")
        self.send_header("Cache-Control", "no-cache, no-store")
        self.send_header("Connection", "close")
        self.send_header("X-Accel-Buffering", "no")
        self.end_headers()

    def _normalize_path(self) -> str:
        path = self.path.split("?", 1)[0] or "/"
        if len(path) > 1 and path.endswith("/"):
            path = path[:-1]
        return path

    def do_OPTIONS(self) -> None:  # noqa: N802
        _inc("requests")
        self._send_empty(204)

    def do_HEAD(self) -> None:  # noqa: N802
        _inc("requests")
        path = self._normalize_path()
        if path in ("/", "/health", "/ready", "/v1/health", "/v1/models", "/models"):
            self._send_empty(200)
            return
        self._send_empty(404)

    def do_GET(self) -> None:  # noqa: N802
        _inc("requests")
        path = self._normalize_path()
        if path in ("/", "/health", "/ready", "/v1/health", "/healthz", "/readyz"):
            self._send_json({"status": "ok", "ready": True, "mock": True, "model": MODEL, "mode": MODE, **_snapshot()})
            return
        if path in ("/v1/models", "/models"):
            created = _STATE.get("started_at", _now())
            self._send_json(
                {
                    "object": "list",
                    "data": [
                        {"id": MODEL, "object": "model", "created": created, "owned_by": "mock"},
                        {"id": "mock-deterministic", "object": "model", "created": created, "owned_by": "mock"},
                    ],
                }
            )
            return
        self._send_json({"error": {"message": f"not found: {path}", "type": "not_found", "param": None, "code": "not_found"}}, 404)

    def _parse_json_body(self, raw: bytes) -> dict[str, Any]:
        if not raw:
            return {}
        try:
            body = json.loads(raw.decode("utf-8", errors="replace"))
            if isinstance(body, dict):
                return body
            return {"input": body}
        except Exception:
            _inc("errors")
            return {}

    def _wants_stream(self, body: dict[str, Any]) -> bool:
        if "stream" in body:
            return bool(body.get("stream"))
        accept = (self.headers.get("Accept") or "").lower()
        if "text/event-stream" in accept:
            return True
        # OpenAI chat completions default is non-streaming; prefer JSON for omitted stream.
        return False

    def do_POST(self) -> None:  # noqa: N802
        _inc("requests")
        path = self._normalize_path()
        raw = self._read_body()
        body = self._parse_json_body(raw)

        if path.endswith("/embeddings"):
            _inc("embedding_requests")
            inp = body.get("input", "")
            if isinstance(inp, list):
                items = inp
            else:
                items = [inp]
            n = max(1, len(items))
            data = [
                {"object": "embedding", "index": i, "embedding": [0.0] * 8}
                for i in range(n)
            ]
            self._send_json(
                {
                    "object": "list",
                    "data": data,
                    "model": body.get("model") or MODEL,
                    "usage": {"prompt_tokens": n, "total_tokens": n},
                }
            )
            return

        # Treat chat/completions-compatible POSTs (common path variants) as chat.
        if not (path.endswith("/chat/completions") or path.endswith("/completions") or "/chat" in path):
            # Still accept unknown POSTs for robustness, but log clearly.
            pass
        _inc("chat_requests")
        messages = body.get("messages") or []
        if not isinstance(messages, list):
            messages = []
        user_text = _user_text(messages)
        digest = hashlib.sha256(user_text.encode("utf-8", errors="ignore")).hexdigest()[:12]
        stream = self._wants_stream(body)
        req_model = body.get("model") or MODEL
        created = _now()
        if stream:
            _inc("stream_requests")
        else:
            _inc("non_stream_requests")
        _write_log(
            f"REQ path={path} stream={stream} msgs={len(messages)} digest={digest} mode={MODE} model={req_model}"
        )

        if MODE == "tool":
            tool_call = {
                "index": 0,
                "id": f"call_mock_{digest}",
                "type": "function",
                "function": {"name": TOOL, "arguments": json.dumps({"path": "sanity/check.csv"}, separators=(",", ":"))},
            }
            if not stream:
                self._send_json(
                    {
                        "id": f"chatcmpl-mock-{digest}",
                        "object": "chat.completion",
                        "created": created,
                        "model": req_model,
                        "choices": [
                            {
                                "index": 0,
                                "message": {"role": "assistant", "content": None, "tool_calls": [tool_call]},
                                "finish_reason": "tool_calls",
                            }
                        ],
                        "usage": {"prompt_tokens": 1, "completion_tokens": 1, "total_tokens": 2},
                    }
                )
                return
            self._send_tool_stream(digest, tool_call, created=created, model=req_model)
            return

        reply = _reply_for(user_text, digest)
        if not stream:
            completion_tokens = max(1, len(reply.split()))
            self._send_json(
                {
                    "id": f"chatcmpl-mock-{digest}",
                    "object": "chat.completion",
                    "created": created,
                    "model": req_model,
                    "choices": [
                        {
                            "index": 0,
                            "message": {"role": "assistant", "content": reply},
                            "finish_reason": "stop",
                        }
                    ],
                    "usage": {"prompt_tokens": 1, "completion_tokens": completion_tokens, "total_tokens": completion_tokens + 1},
                }
            )
            return
        self._send_content_stream(digest, reply, created=created, model=req_model)

    def _send_content_stream(self, digest: str, reply: str, created: int | None = None, model: str | None = None) -> None:
        try:
            self._send_sse_headers()
            chunk_base = {
                "id": f"chatcmpl-mock-{digest}",
                "object": "chat.completion.chunk",
                "created": int(created if created is not None else _now()),
                "model": model or MODEL,
            }

            def send(delta: dict[str, Any], finish: str | None = None) -> None:
                payload = dict(chunk_base)
                payload["choices"] = [{"index": 0, "delta": delta, "finish_reason": finish}]
                data = json.dumps(payload, separators=(",", ":"), ensure_ascii=False)
                self.wfile.write(("data: " + data + "\n\n").encode("utf-8"))
                self.wfile.flush()

            # Initial role delta improves OpenAI client compatibility.
            send({"role": "assistant", "content": ""}, None)
            # Stream in small deterministic chunks for client compatibility.
            parts = [reply] if len(reply) <= 64 else [reply[i : i + 48] for i in range(0, len(reply), 48)]
            for part in parts:
                if part:
                    send({"content": part}, None)
            send({}, "stop")
            self.wfile.write(b"data: [DONE]\n\n")
            self.wfile.flush()
        except (BrokenPipeError, ConnectionAbortedError, ConnectionResetError, OSError):
            _inc("errors")
        finally:
            self.close_connection = True

    def _send_tool_stream(self, digest: str, tool_call: dict[str, Any], created: int | None = None, model: str | None = None) -> None:
        try:
            self._send_sse_headers()
            chunk_base = {
                "id": f"chatcmpl-mock-{digest}",
                "object": "chat.completion.chunk",
                "created": int(created if created is not None else _now()),
                "model": model or MODEL,
            }

            def send(delta: dict[str, Any], finish: str | None = None) -> None:
                payload = dict(chunk_base)
                payload["choices"] = [{"index": 0, "delta": delta, "finish_reason": finish}]
                data = json.dumps(payload, separators=(",", ":"), ensure_ascii=False)
                self.wfile.write(("data: " + data + "\n\n").encode("utf-8"))
                self.wfile.flush()

            send({"role": "assistant", "content": None, "tool_calls": [tool_call]}, None)
            send({}, "tool_calls")
            self.wfile.write(b"data: [DONE]\n\n")
            self.wfile.flush()
        except (BrokenPipeError, ConnectionAbortedError, ConnectionResetError, OSError):
            _inc("errors")
        finally:
            self.close_connection = True


def main() -> None:
    server = MockServer((HOST, PORT), Handler)

    def _shutdown(signum: int, _frame: Any) -> None:
        _write_log(f"mock shutting down signal={signum}")
        threading.Thread(target=server.shutdown, daemon=True).start()

    for sig_name in ("SIGTERM", "SIGINT"):
        sig = getattr(signal, sig_name, None)
        if sig is not None:
            try:
                signal.signal(sig, _shutdown)
            except Exception:
                pass

    _write_log(f"mock listening on http://{HOST}:{PORT}/v1 mode={MODE} model={MODEL}")
    try:
        server.serve_forever(poll_interval=0.5)
    except KeyboardInterrupt:
        pass
    finally:
        try:
            server.server_close()
        except Exception:
            pass
        _write_log("mock stopped")


if __name__ == "__main__":
    main()
