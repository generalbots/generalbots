#!/usr/bin/env bash
# One action: GET the login page from the login UI (port 5000 by default).
# Login/signup are served only by the login domain; port 4000 must not serve them.
set -uo pipefail
here="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=uat_common.sh
. "$here/uat_common.sh"

id="login-page"
url="${UAT_LOGIN_URL:-http://localhost:5000}/login"
uat_step_banner "$id — GET $url"
if ! uat_live_gate "$id"; then
    exit 3
fi

code="$(uat_http_status "$url")"
rc=$?
printf 'HTTP %s\n' "${code:-no-response}"
if [ "$rc" -eq 0 ] && [ "$code" = "200" ]; then
    uat_record "$id" PASS "login page returned HTTP 200"
else
    uat_record "$id" FAIL "login page returned ${code:-no-response}, expected 200"
    exit 1
fi
