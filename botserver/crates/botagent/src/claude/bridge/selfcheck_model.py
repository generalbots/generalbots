import json, signal, sys
from http.server import BaseHTTPRequestHandler, HTTPServer
signal.alarm(120)
def ran(messages):
    for m in messages:
        c = m.get("content")
        if isinstance(c, list) and any(b.get("type") == "tool_result" for b in c):
            return True
    return False
class H(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"
    def log_message(self, *a):
        pass
    def body(self, obj):
        data = json.dumps(obj).encode()
        self.send_response(200)
        self.send_header("content-type", "application/json")
        self.send_header("content-length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)
    def do_GET(self):
        self.body({"data": [], "has_more": False})
    def ev(self, name, data):
        self.wfile.write(("event: %s\ndata: %s\n\n" % (name, json.dumps(data))).encode())
    def do_POST(self):
        req = json.loads(self.rfile.read(int(self.headers.get("content-length") or 0)) or b"{}")
        if "count_tokens" in self.path:
            return self.body({"input_tokens": 1})
        ask = bool(req.get("tools")) and not ran(req.get("messages", []))
        block = ({"type": "tool_use", "id": "toolu_gb_check", "name": "Bash",
                  "input": {"command": "true", "description": "permission self-check"}}
                 if ask else {"type": "text", "text": "ok"})
        stop = "tool_use" if ask else "end_turn"
        msg = {"id": "msg_gb_check", "type": "message", "role": "assistant", "model": req.get("model", "check"),
               "content": [], "stop_reason": None, "stop_sequence": None, "usage": {"input_tokens": 1, "output_tokens": 1}}
        if not req.get("stream"):
            msg.update(content=[block], stop_reason=stop)
            return self.body(msg)
        self.send_response(200)
        self.send_header("content-type", "text/event-stream")
        self.send_header("connection", "close")
        self.end_headers()
        self.ev("message_start", {"type": "message_start", "message": msg})
        start = dict(block, input={}) if ask else dict(block, text="")
        self.ev("content_block_start", {"type": "content_block_start", "index": 0, "content_block": start})
        delta = ({"type": "input_json_delta", "partial_json": json.dumps(block["input"])} if ask
                 else {"type": "text_delta", "text": "ok"})
        self.ev("content_block_delta", {"type": "content_block_delta", "index": 0, "delta": delta})
        self.ev("content_block_stop", {"type": "content_block_stop", "index": 0})
        self.ev("message_delta", {"type": "message_delta", "delta": {"stop_reason": stop, "stop_sequence": None},
                                  "usage": {"output_tokens": 1}})
        self.ev("message_stop", {"type": "message_stop"})
        self.wfile.flush()
        self.close_connection = True
HTTPServer(("127.0.0.1", int(sys.argv[1])), H).serve_forever()
