#!/usr/bin/env python3
"""Create and destroy a throwaway cloud account for the CI suite smoke.

Commands:
  create   POST /api/cloud/auth/signup with a UUID-suffixed bot name; saves
           ids, email, password and JWT to $GB_ACCOUNT_FILE (default /tmp).
  cleanup  Revokes the org through the API, removes the mock identity and
           deletes the rows signup created. Best effort: never masks a smoke
           failure, never prints the token.

The password is generated at runtime (or supplied via GB_SMOKE_PASSWORD);
it is never hardcoded, stored in the repo or reused outside this job.
"""

from __future__ import annotations

import json
import os
import secrets
import subprocess
import sys
import urllib.error
import urllib.parse
import urllib.request
import uuid

API = os.environ.get("GB_API_BASE", "http://127.0.0.1:8080").rstrip("/")
ACCOUNT_FILE = os.environ.get("GB_ACCOUNT_FILE", "/tmp/gb-smoke-account.json")
DIRECTORY = os.environ.get("GB_MOCK_ZITADEL_URL", "http://127.0.0.1:8099").rstrip("/")
# CI-only throwaway credential: prefer an injected value; otherwise generate
# one per run and persist it only in the 0600 account file for cleanup.
PASSWORD = os.environ.get("GB_SMOKE_PASSWORD") or (secrets.token_urlsafe(18) + "Aa1!")


def request(method: str, path: str, body=None, token: str = "") -> tuple[int, dict]:
    data = json.dumps(body).encode() if body is not None else None
    headers = {"Accept": "application/json"}
    if data:
        headers["Content-Type"] = "application/json"
    headers["Origin"] = API
    if token:
        headers["Authorization"] = "Bearer " + token
    req = urllib.request.Request(API + path, data=data, headers=headers, method=method)
    try:
        with urllib.request.urlopen(req, timeout=30) as resp:
            raw = resp.read().decode(errors="replace")
            try:
                return resp.status, json.loads(raw)
            except ValueError:
                return resp.status, {}
    except urllib.error.HTTPError as err:
        raw = err.read().decode(errors="replace")
        try:
            return err.code, json.loads(raw)
        except ValueError:
            return err.code, {"error": raw[:200]}


def create() -> int:
    suffix = uuid.uuid4().hex[:8]
    email = f"smoke-{suffix}@example.test"
    bot_name = f"smoke-{suffix}"
    status, data = request("POST", "/api/cloud/auth/signup", {
        "email": email,
        "name": "CI Smoke",
        "bot_name": bot_name,
        "password": PASSWORD,
        "plan": "free",
    })
    if status >= 400 or not data.get("status") == "ok":
        print(f"CREATE FAIL: signup returned {status}: {str(data)[:300]}")
        return 1
    token = data.get("token") or ""
    if not token:
        status, login = request("POST", "/api/cloud/auth/login",
                                {"email": email, "password": PASSWORD})
        token = login.get("token", "") if status < 400 else ""
        if not token:
            print(f"CREATE FAIL: no token after login ({status})")
            return 1
    account = {
        "email": email,
        "password": PASSWORD,
        "bot_name": bot_name,
        "org_id": data.get("org_id", ""),
        "branch_id": data.get("branch_id", ""),
        "token": token,
    }
    with open(ACCOUNT_FILE, "w") as out:
        json.dump(account, out)
    os.chmod(ACCOUNT_FILE, 0o600)
    print(f"CREATE OK: bot={bot_name} org={account['org_id']} token_len={len(token)}")
    return 0


def cleanup() -> int:
    try:
        with open(ACCOUNT_FILE) as src:
            account = json.load(src)
    except (OSError, ValueError):
        print("CLEANUP: no account file; nothing to do")
        return 0

    token = account.get("token", "")
    org_id = account.get("org_id", "")
    branch_id = account.get("branch_id", "")
    email = account.get("email", "")

    if org_id:
        status, _ = request("DELETE", "/api/cloud/organizations/" + org_id, token=token)
        print(f"CLEANUP: org DELETE -> {status}")

    try:
        encoded = urllib.parse.quote(email, safe="")
        req = urllib.request.Request(
            f"{DIRECTORY}/management/v1/users/by-login/{encoded}", method="DELETE")
        with urllib.request.urlopen(req, timeout=15) as resp:
            print(f"CLEANUP: identity DELETE -> {resp.status}")
    except Exception as exc:
        print(f"CLEANUP: identity delete skipped ({type(exc).__name__})")

    database_url = os.environ.get("DATABASE_URL", "")
    if database_url and org_id:
        statements = [
            "DELETE FROM workspace_resources WHERE workspace_id IN "
            f"(SELECT id FROM cloud_workspaces WHERE org_id='{org_id}')",
            f"DELETE FROM cloud_workspaces WHERE org_id='{org_id}'",
            f"DELETE FROM user_organizations WHERE org_id='{org_id}'",
            f"DELETE FROM billing_recurring WHERE org_id='{branch_id}'",
            f"DELETE FROM crm_contacts WHERE org_id='{org_id}'",
            f"DELETE FROM bots WHERE org_id='{org_id}'",
            f"DELETE FROM branches WHERE id='{branch_id}'",
            f"DELETE FROM organizations WHERE id='{org_id}'",
            f"DELETE FROM users WHERE email='{email}'",
        ]
        sql = "; ".join(statements) + ";"
        proc = subprocess.run(["psql", database_url, "-v", "ON_ERROR_STOP=0", "-c", sql],
                              capture_output=True, text=True, timeout=60)
        print(f"CLEANUP: SQL rc={proc.returncode}")
        if proc.stderr.strip():
            print("CLEANUP: SQL warnings: " + proc.stderr.strip().splitlines()[0][:180])

    try:
        os.remove(ACCOUNT_FILE)
    except OSError:
        pass
    return 0


if __name__ == "__main__":
    command = sys.argv[1] if len(sys.argv) > 1 else ""
    if command == "create":
        sys.exit(create())
    if command == "cleanup":
        sys.exit(cleanup())
    print("usage: ci-suite-account.py create|cleanup")
    sys.exit(2)
