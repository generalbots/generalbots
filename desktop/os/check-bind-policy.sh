#!/bin/sh
# #2477 — OS appliance bind-policy smoke test.
# The suite ports must be loopback-only in shell mode unless the operator has
# explicitly turned on GB_ALLOW_LAN=1 (the nftables gb_lan_allowed set is the
# matching host-side gate). Run inside the appliance after boot.
set -eu

PORTS="8080 3000 4000 5000"
LAN_ON="${GB_ALLOW_LAN:-0}"

if [ "$LAN_ON" = "1" ] || [ "$LAN_ON" = "true" ]; then
    echo "bind-policy: GB_ALLOW_LAN is on — skipping loopback-only assertion"
    exit 0
fi

if ! command -v ss >/dev/null 2>&1; then
    echo "bind-policy: ss is not installed — cannot assert" >&2
    exit 2
fi

failed=0
snapshot=$(ss -tlnp 2>/dev/null || true)
printf '%s\n' "$snapshot"

for port in $PORTS; do
    line=$(printf '%s\n' "$snapshot" | awk -v p=":$port" '$4 ~ p {print $4; exit}' || true)
    if [ -z "$line" ]; then
        echo "bind-policy: port $port is not listening (ok)"
        continue
    fi
    case "$line" in
        127.0.0.1:*|\[::1\]:*|localhost:*)
            echo "bind-policy: port $port loopback-only (ok)"
            ;;
        *)
            echo "bind-policy: FAIL port $port is exposed on $line" >&2
            failed=1
            ;;
    esac
done

if [ "$failed" -ne 0 ]; then
    echo "bind-policy: the appliance is publishing suite ports on a LAN interface" >&2
    exit 1
fi
echo "bind-policy: all suite ports loopback-only (PASS)"
