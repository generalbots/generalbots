#!/usr/bin/env python3
"""Deterministic OpenAI-compatible mock for local chat tests (#1756).

The old ad-hoc harness lived in /tmp and always streamed a `drive_fetch`
tool call, so every request produced another tool round and the browser saw
several completion frames per user turn. This one answers once:

    MOCK_MODE=answer   one content stream, one finish_reason=stop, one [DONE]
    MOCK_MODE=tool     one tool_call (MOCK_TOOL, default drive_fetch) then stop

Attachment tests: include `ATTACH-TEST` in the prompt and the answer streams a
single `[sample.docx](vmfile:output/sample.docx)` link.

Usage:
    python3 scripts/mock_llm.py            # 127.0.0.1:8090
    MOCK_PORT=8091 MOCK_MODE=tool python3 scripts/mock_llm.py

Then point the bot's `llm-url` at http://127.0.0.1:<port>/v1/chat/completions
(Vault, never committed). Requests are logged to stderr, not the repo.
"""
import hashlib
import json
import os
import sys
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

MODEL = os.environ.get("MOCK_MODEL", "mock-deterministic")
MODE = os.environ.get("MOCK_MODE", "answer")
TOOL = os.environ.get("MOCK_TOOL", "drive_fetch")


def log(message: str) -> None:
    print(message, file=sys.stderr, flush=True)


class Handler(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def log_message(self, *args) -> None:
        return

    def _json(self, payload: dict, code: int = 200) -> None:
        data = json.dumps(payload).encode()
        self.send_response(code)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def do_GET(self) -> None:
        if self.path.startswith("/v1/models"):
            self._json({"object": "list", "data": [{"id": MODEL, "object": "model"}]})
        else:
            self._json({"error": "not found"}, 404)

    def do_POST(self) -> None:
        length = int(self.headers.get("Content-Length") or 0)
        raw = self.rfile.read(length)
        if self.path.endswith("/embeddings"):
            self._json({
                "object": "list",
                "model": MODEL,
                "data": [{"object": "embedding", "index": 0, "embedding": [0.0] * 8}],
            })
            return

        body = json.loads(raw or b"{}")
        messages = body.get("messages") or []
        user_text = ""
        for item in reversed(messages):
            if item.get("role") == "user":
                user_text = str(item.get("content") or "")
                break
        digest = hashlib.sha256(user_text.encode()).hexdigest()[:12]
        log(f"REQ stream={body.get('stream')} msgs={len(messages)} digest={digest} mode={MODE}")

        self.send_response(200)
        self.send_header("Content-Type", "text/event-stream")
        self.send_header("Cache-Control", "no-cache")
        self.send_header("Connection", "close")
        self.end_headers()

        chunk = {
            "id": f"chatcmpl-mock-{digest}",
            "object": "chat.completion.chunk",
            "created": 0,
            "model": MODEL,
        }

        def send(delta: dict, finish: str | None = None) -> None:
            payload = dict(chunk)
            payload["choices"] = [{"index": 0, "delta": delta, "finish_reason": finish}]
            self.wfile.write(("data: " + json.dumps(payload) + "\n\n").encode())
            self.wfile.flush()

        if MODE == "tool":
            send({"tool_calls": [{
                "index": 0,
                "id": f"call_mock_{digest}",
                "type": "function",
                "function": {"name": TOOL, "arguments": json.dumps({"path": "sanity/check.csv"})},
            }]})
            send({}, "tool_calls")
        else:
            if "ATTACH-TEST" in user_text:
                pieces = ["Here is the generated file: ", "[sample.docx](vmfile:output/sample.docx)"]
            elif "[Steering]" in user_text:
                pieces = ["DONE ", "- steering ", "applied."]
            else:
                pieces = [f"MOCK-OK {digest}"]
            for piece in pieces:
                send({"content": piece})
            send({}, "stop")

        self.wfile.write(b"data: [DONE]\n\n")
        self.wfile.flush()
        self.close_connection = True


def main() -> None:
    port = int(os.environ.get("MOCK_PORT", "8090"))
    log(f"mock listening on 127.0.0.1:{port} mode={MODE}")
    ThreadingHTTPServer(("127.0.0.1", port), Handler).serve_forever()


if __name__ == "__main__":
    main()
