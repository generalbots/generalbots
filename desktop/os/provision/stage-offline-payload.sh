#!/usr/bin/env bash
# Stage the first-boot provisioner payload into the mkosi image tree (#2441).
#
# The desktop launcher copies $HERE/installers/* into BOTSERVER_INSTALLERS_PATH
# (desktop/linux/start.sh), and botserver resolves every component/model from
# that flat cache by the filenames pinned in botserver/3rdparty.toml. This
# script copies the already-downloaded artifacts from a LOCAL cache directory
# into that image location, verifying SHA-256 on the way. It never downloads:
# no CDN URL, network client or credential appears here or in the manifest.
#
# Usage:
#   stage-offline-payload.sh --cache-dir DIR [options]
#     --manifest FILE    payload manifest (default: alongside this script)
#     --thirdparty FILE  checksum source (default: <repo>/botserver/3rdparty.toml)
#     --repo DIR         repository root (default: three levels above this script)
#     --dest DIR         image tree destination (default: manifest payload_dir)
#     --include-model    also stage the ~1.8 GB local-model group (opt-in)
#     --max-mib N        payload budget in MiB (default: 4096); fail closed if exceeded
#     --require-cache    fail instead of SKIP when the cache is absent/empty
#
# Exit: 0 on success or on a documented SKIP; 1 on a bad/missing/tampered
# artifact; 2 on a usage error.
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/../../.." && pwd)"
MANIFEST="$SCRIPT_DIR/offline-payload.toml"
THIRDPARTY="$REPO_ROOT/botserver/3rdparty.toml"
DEST=""
CACHE=""
INCLUDE_MODEL=0
REQUIRE_CACHE=0
MAX_MIB=4096
THIRDPARTY_SET=0

usage() {
  sed -n '2,22p' "${BASH_SOURCE[0]}"
}

skip() {
  printf '#2441 SKIP: %s\n' "$*"
  exit 0
}

die() {
  printf 'error: %s\n' "$*" >&2
  exit 1
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --cache-dir)    CACHE="${2:-}"; shift 2 ;;
    --manifest)     MANIFEST="${2:-}"; shift 2 ;;
    --thirdparty)   THIRDPARTY="${2:-}"; THIRDPARTY_SET=1; shift 2 ;;
    --repo)         REPO_ROOT="${2:-}"; shift 2 ;;
    --dest)         DEST="${2:-}"; shift 2 ;;
    --max-mib)      MAX_MIB="${2:-}"; shift 2 ;;
    --include-model) INCLUDE_MODEL=1; shift ;;
    --require-cache) REQUIRE_CACHE=1; shift ;;
    -h|--help)      usage; exit 0 ;;
    *) printf 'unexpected argument: %s\n' "$1" >&2; usage >&2; exit 2 ;;
  esac
done

# A malformed budget is a usage error, never an implicit "unlimited" pass.
case "$MAX_MIB" in
  ''|*[!0-9]*) printf 'error: --max-mib must be a positive integer (MiB)\n' >&2; exit 2 ;;
esac
[ "$MAX_MIB" -gt 0 ] || { printf 'error: --max-mib must be a positive integer (MiB)\n' >&2; exit 2; }

# --repo moves the whole default tree (checksum source and image destination).
if [ "$THIRDPARTY_SET" -eq 0 ]; then
  THIRDPARTY="$REPO_ROOT/botserver/3rdparty.toml"
fi

[ -n "$CACHE" ] || skip "no --cache-dir given; the image will not carry an offline payload"
if [ ! -d "$CACHE" ]; then
  if [ "$REQUIRE_CACHE" -eq 1 ]; then
    die "payload cache directory is missing: $CACHE (--require-cache)"
  fi
  skip "payload cache directory is absent: $CACHE"
fi
if [ -z "$(find "$CACHE" -maxdepth 1 -type f -print -quit)" ]; then
  if [ "$REQUIRE_CACHE" -eq 1 ]; then
    die "payload cache directory is empty: $CACHE (--require-cache)"
  fi
  skip "payload cache directory is empty: $CACHE"
fi
[ -f "$MANIFEST" ] || die "payload manifest not found: $MANIFEST"
command -v python3 >/dev/null 2>&1 || die "python3 is required to parse the manifest"
command -v sha256sum >/dev/null 2>&1 || die "sha256sum is required to verify payloads"

PLAN="$(mktemp)"
NEW_LIST="$(mktemp)"
NEW_SUMS="$(mktemp)"
trap 'rm -f "$PLAN" "$NEW_LIST" "$NEW_SUMS"' EXIT

# Emit one pipe-separated record per payload: id, filename, sha256, required,
# group, sha256_sidecar. A non-whitespace IFS is required: empty fields (no
# upstream digest) must not be collapsed by `read`. Python's tomllib is in
# the stdlib of the runners and of the Debian tools tree; the Bash side only
# handles files and checksums.
MANIFEST_PATH="$MANIFEST" THIRDPARTY_PATH="$THIRDPARTY" INCLUDE_MODEL="$INCLUDE_MODEL" \
  python3 - <<'PY' > "$PLAN"
import os
import re
import sys
import tomllib

manifest_path = os.environ["MANIFEST_PATH"]
thirdparty_path = os.environ["THIRDPARTY_PATH"]
include_model = os.environ["INCLUDE_MODEL"] == "1"

with open(manifest_path, "rb") as handle:
    manifest = tomllib.load(handle)

if manifest.get("version") != 1:
    sys.exit("error: unsupported payload manifest version")
if manifest.get("arch") != "x86_64" or manifest.get("os") != "linux":
    sys.exit("error: payload manifest does not describe the x86_64 Linux image")

thirdparty = {}
if os.path.isfile(thirdparty_path):
    with open(thirdparty_path, "rb") as handle:
        thirdparty = tomllib.load(handle)
else:
    print(f"#2441 NOTE: {thirdparty_path} not found; manifest pins are not cross-checked",
          file=sys.stderr)

safe_name = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._-]*$")
sha_pattern = re.compile(r"^[0-9a-fA-F]{64}$")
seen = set()
records = []

for item in manifest.get("payload", []):
    ident = str(item.get("id", ""))
    key = str(item.get("key", ""))
    section = str(item.get("section", ""))
    filename = str(item.get("filename", ""))
    sha = str(item.get("sha256", "")).strip().lower()
    sidecar = str(item.get("sha256_sidecar", ""))
    group = str(item.get("group", ""))
    if not ident or ident in seen:
        sys.exit(f"error: missing or duplicate payload id {ident!r}")
    seen.add(ident)
    if section not in ("components", "models") or group not in ("core", "local_model", "optional"):
        sys.exit(f"error: {ident}: unknown section/group")
    for label, name in (("filename", filename), ("sidecar", sidecar)):
        if name and (not safe_name.fullmatch(name) or name in (".", "..")):
            sys.exit(f"error: {ident}: unsafe {label} {name!r}")
    if sha and not sha_pattern.fullmatch(sha):
        sys.exit(f"error: {ident}: malformed sha256 {sha!r}")

    if thirdparty:
        entry = thirdparty.get(section, {}).get(key)
        if entry is None:
            sys.exit(f"error: {ident}: {section}.{key} missing from {thirdparty_path}")
        pinned = str(entry.get("sha256", "")).strip().lower()
        if pinned != sha:
            sys.exit(f"error: {ident}: manifest sha256 {sha or '<empty>'} != 3rdparty.toml {pinned or '<empty>'}")

    if group == "local_model" and not include_model:
        print(f"#2441 SKIP {ident}: local-model artifact ({filename}); use --include-model",
              file=sys.stderr)
        continue
    required = "0" if group == "optional" else "1"
    records.append("|".join((ident, filename, sha, required, group, sidecar)))

for record in records:
    print(record)
PY

if [ -z "$DEST" ]; then
  DEST="$REPO_ROOT/$(awk -F= '/^payload_dir[[:space:]]*=/ {gsub(/["[:space:]]/, "", $2); print $2; exit}' "$MANIFEST")"
fi
[ -n "$DEST" ] || die "no payload destination could be resolved"
mkdir -p -- "$DEST"

# Remove only files a previous run of this script staged, so switching
# --include-model off cannot leave stale multi-gigabyte artifacts behind.
LIST_FILE="$DEST/.gb-offline-payload-files"
if [ -f "$LIST_FILE" ]; then
  while IFS= read -r old; do
    case "$old" in
      ''|.|..|*/*|*\\*) continue ;;
    esac
    rm -f -- "$DEST/$old"
  done < "$LIST_FILE"
fi

count=0
total=0
while IFS='|' read -r id filename sha required group sidecar; do
  [ -n "$id" ] || continue
  src="$CACHE/$filename"
  if [ ! -f "$src" ]; then
    if [ "$required" -eq 1 ]; then
      die "required payload '$id' is missing from cache: $src"
    fi
    printf '#2441 SKIP %s: optional artifact %s is not in the cache\n' "$id" "$filename" >&2
    continue
  fi

  actual="$(sha256sum -- "$src" | awk '{print $1}')"
  if [ -n "$sha" ]; then
    [ "$actual" = "$sha" ] || die "sha256 mismatch for $filename (manifest $sha, cache $actual)"
    provenance="sha256=$sha"
  elif [ -n "$sidecar" ]; then
    side="$CACHE/$sidecar"
    [ -f "$side" ] || die "$id: checksum sidecar missing: $side"
    wanted="$(awk 'NR == 1 {print $1}' "$side")"
    [[ "$wanted" =~ ^[0-9a-fA-F]{64}$ ]] || die "$id: malformed digest in $sidecar"
    [ "$actual" = "$wanted" ] || die "$id: $filename does not match $sidecar"
    provenance="sha256=$actual (via $sidecar)"
  else
    printf '#2441 NOTE %s: no upstream digest is published for %s; artifact copied UNPINNED\n' \
      "$id" "$filename" >&2
    provenance="UNPINNED"
  fi

  install -m 0644 -- "$src" "$DEST/$filename"
  printf '%s\n' "$filename" >> "$NEW_LIST"
  printf '%s  %s\n' "$actual" "$filename" >> "$NEW_SUMS"
  bytes="$(stat -c %s -- "$src" 2>/dev/null || printf '0')"
  count=$((count + 1))
  total=$((total + bytes))
  printf 'STAGED %-14s %-58s %12s bytes %s\n' "$id" "$filename" "$bytes" "$provenance"
done < "$PLAN"

sort -u -- "$NEW_LIST" > "$LIST_FILE"
sort -u -- "$NEW_SUMS" > "$DEST/SHA256SUMS.offline"
mib=$(( total / 1048576 ))
printf '#2441 staged %s artifact(s), %s bytes (%s MiB), budget %s MiB into %s\n' \
  "$count" "$total" "$mib" "$MAX_MIB" "$DEST"
max_bytes=$(( MAX_MIB * 1048576 ))
if [ "$total" -gt "$max_bytes" ]; then
  die "staged payload $total bytes ($mib MiB) exceeds --max-mib $MAX_MIB MiB"
fi
if [ "$INCLUDE_MODEL" -ne 1 ]; then
  printf '#2441 NOTE the ~1.8 GB local model group was NOT staged (use --include-model)\n'
fi
