#!/usr/bin/env bash
# Verifies that the documented /releases/latest/download/... links resolve and
# that "latest" is a product release (issue #1804).
#
# Product releases are tagged vMAJOR.MINOR.PATCH and carry the desktop assets.
# Component builds (alm-*, pythonbox-*, drive-bin-*) and platform extras
# (vMAJOR.MINOR.PATCH-windows/-macos) are pre-releases, so they never take
# "latest".
#
# Usage:
#   scripts/verify-latest-release.sh [repository-root]
#
# Environment:
#   REPO / GITHUB_REPOSITORY   owner/name (default generalbots/generalbots)
#   GH_TOKEN / GITHUB_TOKEN    optional; raises the API rate limit
set -euo pipefail

ROOT="${1:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
REPO="${REPO:-${GITHUB_REPOSITORY:-generalbots/generalbots}}"
API="${GITHUB_API_URL:-https://api.github.com}"
TOKEN="${GH_TOKEN:-${GITHUB_TOKEN:-}}"

fail() {
  echo "::error::$*" >&2
  exit 1
}

notice() {
  echo "::warning::$*" >&2
}

api_get() {
  local path="$1"
  if [ -n "$TOKEN" ]; then
    curl -fsSL -H "Authorization: Bearer ${TOKEN}" \
      -H "Accept: application/vnd.github+json" "${API}/${path}"
  else
    curl -fsSL -H "Accept: application/vnd.github+json" "${API}/${path}"
  fi
}

json_field() {
  python3 -c 'import json,sys; value=json.load(sys.stdin); print(value.get(sys.argv[1], ""))' "$1"
}

echo "checking latest release of ${REPO}"

release_json="$(api_get "repos/${REPO}/releases/latest")"
tag="$(printf '%s' "$release_json" | json_field tag_name)"
prerelease="$(printf '%s' "$release_json" | json_field prerelease)"
draft="$(printf '%s' "$release_json" | json_field draft)"

[ -n "$tag" ] || fail "no latest release found for ${REPO}"
[ "$prerelease" = "False" ] || fail "latest release ${tag} is a pre-release"
[ "$draft" = "False" ] || fail "latest release ${tag} is a draft"
[[ "$tag" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]] \
  || fail "latest release ${tag} is not a product tag (vMAJOR.MINOR.PATCH); a component/platform release took 'latest'"

echo "latest product release: ${tag}"

assets="$(printf '%s' "$release_json" | python3 -c '
import json, sys
print("\n".join(asset["name"] for asset in json.load(sys.stdin).get("assets", [])))
')"

has_asset() {
  grep -Fxq "$1" <<<"$assets"
}

for required in generalbots-linux-x86_64.tar.gz SHA256SUMS-linux; do
  has_asset "$required" \
    || fail "latest release ${tag} has no ${required} asset (the documented links will 404)"
  echo "asset present: ${required}"
done

# Every documented latest/download URL must be an asset of the latest release.
mapfile -t urls < <(
  {
    grep -rhoE 'https://github\.com/[^[:space:]"'"'"')]+/releases/latest/download/[A-Za-z0-9._-]+' \
      "$ROOT/README.md" \
      "$ROOT/desktop/linux/README.txt" \
      "$ROOT/desktop/linux/start.sh" \
      "$ROOT/desktop/linux/install.sh" 2>/dev/null || true
    find "$ROOT/botbook/src" -name '*.md' -type f -print0 2>/dev/null \
      | xargs -0 -r grep -hoE 'https://github\.com/[^[:space:]"'"'"')]+/releases/latest/download/[A-Za-z0-9._-]+' 2>/dev/null || true
  } | sort -u
)

[ "${#urls[@]}" -gt 0 ] || fail "no /releases/latest/download/ URLs found in the docs"

for url in "${urls[@]}"; do
  asset="${url##*/}"
  # Documentation templates such as .../generalbots-linux-${ARCH}.tar.gz are
  # not concrete links; skip the truncated asset name they produce.
  if [[ "$asset" == *- ]]; then
    notice "skipping template URL ${url}"
    continue
  fi
  has_asset "$asset" \
    || fail "documented URL points at missing asset ${asset}: ${url}"
  status="$(curl -fsSIL --retry 5 --retry-delay 5 -o /dev/null -w '%{http_code}' "$url" || true)"
  [ "$status" = "200" ] || fail "documented link returned HTTP ${status}: ${url}"
  echo "link ok: ${url}"
done

# The source tree should agree with the released product version. This is a
# warning, not a failure: a build that failed for a newer commit must not make
# the (still valid) stable links look broken.
source_version="$(python3 "$ROOT/scripts/release_version.py" --root "$ROOT" current)"
expected="${tag#v}"
if [ "$source_version" != "$expected" ]; then
  notice "latest release ${tag} does not match the source version ${source_version}"
fi

echo "latest download links are healthy (${tag})"
