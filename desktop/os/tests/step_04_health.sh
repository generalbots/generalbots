#!/usr/bin/env bash
# One action: probe ONE component health endpoint (or TCP port) on the host.
# Usage: step_04_health.sh <component> <port> [http-path]
# Loopback only; no internal addresses or credentials are used or printed.
set -uo pipefail
here="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=uat_common.sh
. "$here/uat_common.sh"

comp="${1:-}"
port="${2:-}"
path="${3:-}"
case "$port" in
    ''|*[!0-9]*) printf 'usage: %s <component> <port> [http-path]\n' "$0" >&2; exit 2 ;;
esac
if [ "$port" -lt 1 ] || [ "$port" -gt 65535 ]; then
    printf 'port out of range: %s\n' "$port" >&2
    exit 2
fi
id="health:$comp"
uat_step_banner "$id — health probe localhost:$port$path"
if ! uat_live_gate "$id"; then
    exit 3
fi

if [ -n "$path" ]; then
    case "$path" in /*) ;; *) path="/$path" ;; esac
    code="$(uat_http_status "http://localhost:${port}${path}")"
    rc=$?
    printf 'HTTP %s on localhost:%s%s\n' "${code:-no-response}" "$port" "$path"
    if [ "$rc" -eq 0 ] && [ "$code" = "200" ]; then
        uat_record "$id" PASS "HTTP 200 on localhost:$port$path"
    else
        uat_record "$id" FAIL "expected HTTP 200 on localhost:$port$path, got ${code:-no-response}"
        exit 1
    fi
else
    if timeout 4 bash -c "exec 3<>/dev/tcp/localhost/$port" 2>/dev/null; then
        printf 'TCP localhost:%s open\n' "$port"
        uat_record "$id" PASS "TCP port $port accepts connections"
    else
        printf 'TCP localhost:%s closed or timed out\n' "$port" >&2
        uat_record "$id" FAIL "TCP port $port is not listening"
        exit 1
    fi
fi
