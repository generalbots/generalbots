#!/usr/bin/env bash
# Capture TPM PCR state into a SANITIZED diagnostics JSON (#2478, #2468).
#
# Sources, tried in order:
#   1. `tpm2_pcrread` when the tools are installed and a TPM answers;
#   2. /sys/kernel/security/tpm0/binary_bios_measurements, parsed with a bounded
#      TLV reader so raw event bytes are never printed or stored.
#
# The JSON contains ONLY sha256 digests of the raw PCR values, the PCR indices
# and a UTC timestamp. Raw values, event payloads and host details are never
# written to stdout, stderr or disk. No network access. Never fabricate PCRs:
# if neither source yields digests, an explicit skip with a reason is emitted.

set -uo pipefail

die() { printf 'capture-measured-boot: %s\n' "$*" >&2; exit 1; }

usage() {
    cat <<'USAGE'
Usage: desktop/os/diagnostics/capture-measured-boot.sh [options]

  --out FILE    diagnostics JSON path (default: $GB_DIAGNOSTICS_DIR/measured-boot.json)
  --check       validate an existing JSON (same --out path) without capturing
  -h, --help    show this help

Only sha256 digests of PCR values are stored. With no TPM/tools the result is
an explicit skip with a reason; no values are invented. No network is used.
USAGE
}

diag_dir="${GB_DIAGNOSTICS_DIR:-/tmp/gb-os-diagnostics}"
out="$diag_dir/measured-boot.json"
check_mode=0

while [ "$#" -gt 0 ]; do
    case "$1" in
        --out)      out="${2:-}"; [ "$#" -ge 2 ] || die "--out needs a value"; shift ;;
        --out=*)    out="${1#*=}" ;;
        --check)    check_mode=1 ;;
        -h|--help)  usage; exit 0 ;;
        *)          die "unknown option: $1" ;;
    esac
    shift
done
[ -n "$out" ] || die "--out must not be empty"

timestamp() { date -u +%Y-%m-%dT%H:%M:%SZ; }

prepare_dir() {
    mkdir -p -- "$(dirname -- "$out")" || die "cannot create diagnostics directory for $out"
}

# A skip is real output, not a failure: the reason is explicit and no PCRs are claimed.
write_skip() {
    local reason="$1"
    prepare_dir
    printf '{\n  "schema": "gb-measured-boot/1",\n  "status": "skip",\n  "reason": "%s",\n  "timestamp": "%s",\n  "pcrs": []\n}\n' \
        "$reason" "$(timestamp)" >"$out.tmp.$$"
    mv -f -- "$out.tmp.$$" "$out" || die "cannot write diagnostics file $out"
    printf 'SKIP: %s\n' "$reason"
}

write_ok() {
    local source="$1" pcr_json="$2" count="$3"
    pcr_json="${pcr_json%,$'\n'}"
    prepare_dir
    {
        printf '{\n'
        printf '  "schema": "gb-measured-boot/1",\n'
        printf '  "status": "ok",\n'
        printf '  "timestamp": "%s",\n' "$(timestamp)"
        printf '  "source": "%s",\n' "$source"
        printf '  "algorithm": "sha256",\n'
        printf '  "pcrs": [\n'
        printf '%s\n' "$pcr_json"
        printf '  ],\n'
        printf '  "note": "sha256 of raw PCR values; raw values are never stored or logged"\n'
        printf '}\n'
    } >"$out.tmp.$$"
    mv -f -- "$out.tmp.$$" "$out" || die "cannot write diagnostics file $out"
    printf 'captured %s PCR digests from %s -> %s\n' "$count" "$source" "$out"
}

# tpm2_pcrread prints "  0 : 0x<hex>". Keep only the index and a sha256 of the
# hex string; the raw line stays in a local variable and is never printed.
capture_tpm2() {
    local raw line idx hex sha count=0 pcr_json=""
    raw="$(tpm2_pcrread sha256 2>/dev/null)" || return 1
    [ -n "$raw" ] || return 1
    while IFS= read -r line; do
        case "$line" in
            *0x*)
                idx="$(printf '%s\n' "$line" | sed -n 's/^[[:space:]]*\([0-9][0-9]*\)[[:space:]]*:[[:space:]]*0x.*$/\1/p')"
                hex="$(printf '%s\n' "$line" | sed -n 's/.*0x\([0-9A-Fa-f][0-9A-Fa-f]*\).*$/\1/p')"
                [ -n "$idx" ] && [ -n "$hex" ] || continue
                sha="$(printf '%s' "$hex" | sha256sum | cut -d' ' -f1)"
                pcr_json+="    {\"index\": ${idx}, \"sha256\": \"${sha}\"},"$'\n'
                count=$((count + 1))
                ;;
        esac
    done <<<"$raw"
    [ "$count" -gt 0 ] || return 1
    write_ok "tpm2_pcrread" "$pcr_json" "$count"
}

# The kernel binary event log is a sequence of <pcr,type,count><alg,len,digest>*<eventlen,event>
# little-endian records. The bounded Python reader hashes each PCR's raw digest
# bytes and emits sanitized JSON; it never prints the event data.
capture_binlog() {
    local log="/sys/kernel/security/tpm0/binary_bios_measurements"
    [ -r "$log" ] || return 1
    command -v python3 >/dev/null 2>&1 || return 1
    prepare_dir
    GB_PCR_LOG="$log" GB_PCR_OUT="$out.tmp.$$" python3 - <<'PY' || return 1
import hashlib
import json
import os
import struct
import sys
from datetime import datetime, timezone

log = os.environ["GB_PCR_LOG"]
target = os.environ["GB_PCR_OUT"]
try:
    with open(log, "rb") as stream:
        data = stream.read()
except OSError:
    sys.exit(1)

offset = 0
digests = {}
while offset + 12 <= len(data):
    pcr, _etype, count = struct.unpack_from("<III", data, offset)
    offset += 12
    if pcr > 23 or count > 16:
        break
    for _ in range(count):
        if offset + 4 > len(data):
            break
        _alg, length = struct.unpack_from("<HH", data, offset)
        offset += 4
        if length == 0 or length > 128 or offset + length > len(data):
            break
        digests.setdefault(pcr, hashlib.sha256()).update(data[offset:offset + length])
        offset += length
    if offset + 4 > len(data):
        break
    (event_length,) = struct.unpack_from("<I", data, offset)
    offset += 4
    if offset + event_length > len(data):
        break
    offset += event_length

if not digests:
    sys.exit(1)

doc = {
    "schema": "gb-measured-boot/1",
    "status": "ok",
    "timestamp": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
    "source": "binary_bios_measurements",
    "algorithm": "sha256",
    "pcrs": [{"index": i, "sha256": h.hexdigest()} for i, h in sorted(digests.items())],
    "note": "sha256 of raw PCR values; raw values are never stored or logged",
}
with open(target, "w", encoding="utf-8") as stream:
    json.dump(doc, stream, sort_keys=True, indent=2)
    stream.write("\n")
PY
    mv -f -- "$out.tmp.$$" "$out" || die "cannot write diagnostics file $out"
    printf 'captured %s PCR digests from binary_bios_measurements -> %s\n' \
        "$(grep -c '"index"' "$out")" "$out"
}

# --check validates structure only and prints status/count, never any digest.
check_json() {
    [ -f "$out" ] || { printf 'CHECK: FAIL missing %s\n' "$out"; return 1; }
    GB_PCR_CHECK="$out" python3 - <<'PY'
import json
import os
import sys

path = os.environ["GB_PCR_CHECK"]
try:
    with open(path, encoding="utf-8") as stream:
        doc = json.load(stream)
except (OSError, ValueError) as exc:
    print("CHECK: FAIL invalid JSON (%s)" % type(exc).__name__)
    sys.exit(1)
if doc.get("schema") != "gb-measured-boot/1":
    sys.exit("CHECK: FAIL schema")
status = doc.get("status")
if status not in ("ok", "skip"):
    sys.exit("CHECK: FAIL status")
if not isinstance(doc.get("timestamp"), str) or not doc["timestamp"]:
    sys.exit("CHECK: FAIL timestamp")
pcrs = doc.get("pcrs")
if not isinstance(pcrs, list):
    sys.exit("CHECK: FAIL pcrs")
if status == "ok" and not pcrs:
    sys.exit("CHECK: FAIL status ok with no PCR digests")
for row in pcrs:
    if not isinstance(row, dict) or not isinstance(row.get("index"), int):
        sys.exit("CHECK: FAIL PCR index")
    digest = row.get("sha256")
    if not isinstance(digest, str) or len(digest) != 64:
        sys.exit("CHECK: FAIL PCR digest length")
    if any(char not in "0123456789abcdef" for char in digest.lower()):
        sys.exit("CHECK: FAIL PCR digest is not sha256 hex")
print("CHECK: PASS %s (status=%s, pcrs=%d)" % (path, status, len(pcrs)))
PY
}

if [ "$check_mode" -eq 1 ]; then
    command -v python3 >/dev/null 2>&1 || die "--check requires python3 (stdlib only, no network)"
    check_json
    exit $?
fi

if command -v tpm2_pcrread >/dev/null 2>&1 && capture_tpm2; then
    exit 0
fi
if capture_binlog; then
    exit 0
fi
write_skip "no TPM2 tools responded and /sys/kernel/security/tpm0/binary_bios_measurements is absent, unreadable or yielded no PCR digests"
exit 0
