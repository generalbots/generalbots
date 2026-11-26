#!/usr/bin/env python3
"""Minimal in-memory Zitadel stand-in for the CI suite-app smoke job.

Implements only the endpoints botserver's signup/login use:
  POST /management/v1/users/human/_import   -> userId + verifiedEmail echo
  POST /management/v1/users/{id}/password   -> ok
  POST /management/v1/users/_search         -> ok (service-token probe)
  POST /v2/sessions                          -> verified session with stable user id
  DELETE /management/v1/users/{id}           -> ok
  GET  /health                               -> ok

Never stores secrets on disk; all state is in memory and dies with the job.
"""

from http.server import BaseHTTPRequestHandler, HTTPServer
import json
import os
import urllib.parse
import uuid

USERS = {}


class Handler(BaseHTTPRequestHandler):
    def _body(self):
        length = int(self.headers.get("Content-Length", "0"))
        try:
            return json.loads(self.rfile.read(length) or b"{}")
        except ValueError:
            return {}

    def _send(self, status, payload):
        raw = json.dumps(payload).encode()
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(raw)))
        self.end_headers()
        self.wfile.write(raw)

    def do_POST(self):
        path = self.path.split("?", 1)[0]
        data = self._body()
        if path.endswith("/management/v1/users/human/_import"):
            email = data.get("userName", "")
            user_id = "zitadel-" + uuid.uuid4().hex
            USERS[email] = {"id": user_id, "password": data.get("password", "")}
            self._send(200, {"userId": user_id, "verifiedEmail": email})
        elif path.endswith("/password") and "/management/v1/users/" in path:
            # Signup provisions the human via _import, then persists the
            # password with the v1 management endpoint (v2 leaves an empty
            # hash). Update the stored record so the follow-up session check
            # in signup.rs can actually verify it.
            user_id = path.rsplit("/users/", 1)[1].rsplit("/password", 1)[0]
            password = data.get("password", "")
            for record in USERS.values():
                if record["id"] == user_id:
                    record["password"] = password
                    break
            self._send(200, {"status": "ok"})
        elif path.endswith("/v2/sessions"):
            checks = data.get("checks", {})
            login = checks.get("user", {}).get("loginName", "")
            password = checks.get("password", {}).get("password", "")
            record = USERS.get(login)
            if record and record["password"] == password:
                self._send(200, {"sessionId": "ci-session",
                                 "factors": {"user": {"userId": record["id"]}}})
            else:
                self._send(401, {"error": "COMMAND-3M0fs"})
        elif path.endswith("/management/v1/users/_search"):
            self._send(200, {"result": []})
        else:
            self._send(404, {"error": "not implemented in CI mock"})

    def do_DELETE(self):
        path = urllib.parse.unquote(self.path.split("?", 1)[0])
        marker = "/by-login/"
        if marker in path:
            USERS.pop(path.rsplit(marker, 1)[1], None)
        else:
            for email, record in list(USERS.items()):
                if record["id"] in path:
                    USERS.pop(email, None)
        self._send(200, {"status": "ok"})

    def do_GET(self):
        if self.path.split("?", 1)[0] == "/health":
            self._send(200, {"ok": True})
        else:
            self._send(404, {"error": "not implemented in CI mock"})

    def log_message(self, *_args):
        pass


if __name__ == "__main__":
    port = int(os.environ.get("GB_MOCK_ZITADEL_PORT", "8099"))
    print(f"mock Zitadel listening on {port}", flush=True)
    HTTPServer(("127.0.0.1", port), Handler).serve_forever()
