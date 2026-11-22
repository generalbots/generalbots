"""Loopback stand-in for the model during the Codex approval self-check.

A user message holding 'CMD: <command>' is answered with one shell tool
call, one holding 'PATCH: <path>' (used by the live bridge test) with an
apply_patch call that adds that file; anything else with a short
assistant message. It never leaves 127.0.0.1 and logs nothing."""
import json
import re
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

PORT = 18766


def text_of(item):
    parts = item.get("content") or []
    if isinstance(parts, str):
        return parts
    return " ".join(p.get("text", "") for p in parts if isinstance(p, dict))


def reply_for(body, n):
    tools = [t.get("name") or (t.get("function") or {}).get("name") or t.get("type") for t in body.get("tools", [])]
    items = body.get("input", [])
    last = items[-1] if items else {}
    cmd = None
    patch = None
    if last.get("type") not in ("function_call_output", "custom_tool_call_output"):
        for item in reversed(items):
            if item.get("role") == "user" and ("CMD:" in text_of(item) or "PATCH:" in text_of(item)):
                found = re.search(r"CMD: (.+)", text_of(item))
                cmd = found.group(1).strip() if found else None
                found = re.search(r"PATCH: (\S+)", text_of(item))
                patch = found.group(1) if found else None
                break
    if patch:
        body_text = "*** Begin Patch\n*** Add File: %s\n+patched\n*** End Patch\n" % patch
        kind = next((t.get("type") for t in body.get("tools", []) if t.get("name") == "apply_patch"), None)
        if kind == "custom":
            return {"type": "custom_tool_call", "id": "ct_%d" % n, "call_id": "call_%d" % n,
                    "name": "apply_patch", "input": body_text}
        if kind:
            return {"type": "function_call", "id": "fc_%d" % n, "call_id": "call_%d" % n,
                    "name": "apply_patch", "arguments": json.dumps({"input": body_text})}
        cmd = "apply_patch <<'EOF'\n%sEOF\n" % body_text
    if not cmd:
        return {"type": "message", "role": "assistant", "id": "msg_%d" % n,
                "content": [{"type": "output_text", "text": "self-check done"}]}
    if "shell_command" in tools:
        name, args = "shell_command", {"command": cmd}
    elif "exec_command" in tools:
        name, args = "exec_command", {"cmd": cmd}
    else:
        name, args = "shell", {"command": ["bash", "-lc", cmd]}
    return {"type": "function_call", "id": "fc_%d" % n, "call_id": "call_%d" % n,
            "name": name, "arguments": json.dumps(args)}


class Handler(BaseHTTPRequestHandler):
    count = 0

    def log_message(self, *args):
        pass

    def do_GET(self):
        self.send_response(404)
        self.end_headers()

    def do_POST(self):
        size = int(self.headers.get("Content-Length") or 0)
        body = json.loads(self.rfile.read(size) or b"{}")
        Handler.count += 1
        rid = "resp_%d" % Handler.count
        events = [
            {"type": "response.created", "response": {"id": rid}},
            {"type": "response.output_item.done", "item": reply_for(body, Handler.count)},
            {"type": "response.completed", "response": {"id": rid, "usage": {
                "input_tokens": 1, "input_tokens_details": None, "output_tokens": 1,
                "output_tokens_details": None, "total_tokens": 2}}},
        ]
        self.send_response(200)
        self.send_header("Content-Type", "text/event-stream")
        self.send_header("Cache-Control", "no-cache")
        self.end_headers()
        for event in events:
            self.wfile.write(("event: %s\ndata: %s\n\n" % (event["type"], json.dumps(event))).encode())
        self.wfile.flush()


ThreadingHTTPServer(("127.0.0.1", PORT), Handler).serve_forever()
