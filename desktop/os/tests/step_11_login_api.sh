#!/usr/bin/env bash
# One action: POST credentials to /api/auth/login and check authentication.
# Credentials come from UAT_LOGIN_USER / UAT_LOGIN_PASSWORD; they are never
# stored in a file, put in argv, echoed, or included in a recorded result.
set -uo pipefail
here="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=uat_common.sh
. "$here/uat_common.sh"

id="login-api"
url="${UAT_API_URL:-http://localhost:8080}/api/auth/login"
uat_step_banner "$id — POST /api/auth/login"
if ! uat_live_gate "$id"; then
    exit 3
fi
if [ -z "${UAT_LOGIN_USER:-}" ] || [ -z "${UAT_LOGIN_PASSWORD:-}" ]; then
    uat_record "$id" UNVERIFIED \
        "UAT_LOGIN_USER / UAT_LOGIN_PASSWORD not set; no credentials are hardcoded"
    exit 3
fi

dir="$(uat_init_results)" || exit 2
response="$dir/.login-response.$$.json"
# Minimal JSON escaping so quotes/backslashes in credentials cannot break the
# payload. Values stay in memory only and are never printed or recorded.
user="${UAT_LOGIN_USER//\\/\\\\}"
user="${user//\"/\\\"}"
pass="${UAT_LOGIN_PASSWORD//\\/\\\\}"
pass="${pass//\"/\\\"}"
body="$(printf '{"email":"%s","password":"%s"}' "$user" "$pass")"
code="$(uat_http_json_post_status "$url" "$body" "$response")"
rc=$?
if [ -f "$response" ]; then
    if grep -Eq '"success"[[:space:]]*:[[:space:]]*true' "$response"; then
        ok=1
    else
        ok=0
    fi
    rm -f "$response"
else
    ok=0
fi
printf 'HTTP %s; authenticated payload: %s\n' "${code:-no-response}" \
    "$([ "$ok" -eq 1 ] && printf yes || printf no)"
if [ "$rc" -eq 0 ] && [ "$code" = "200" ] && [ "$ok" -eq 1 ]; then
    uat_record "$id" PASS "login API returned HTTP 200 with success=true"
else
    uat_record "$id" FAIL "login API returned ${code:-no-response} without success=true"
    exit 1
fi
