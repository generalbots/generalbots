#!/usr/bin/env bash
# Build the MinIO server binary from the official source tag (#2612).
#
# Why this exists: the release that fixes CVE-2025-62506
# (RELEASE.2025-10-15T17-29-55Z) publishes **no binary assets**, dl.min.io
# returns 410 for every path and the container images are not anonymously
# pullable. Building the official tag is the only way to get the patch.
#
# Integrity: the upstream source archive is content-pinned in
# botserver/3rdparty.toml (`[components.drive].source_sha256`). The digest was
# recorded by fetching the GitHub tag archive twice; the build refuses a
# different archive. There is no upstream binary checksum, so the digest of
# the compiled binary is written to <output>.sha256 for the installer and the
# `check_component_checksums.py --verify-minio` gate to verify.
#
# Usage:
#   ./scripts/build-minio-fixed.sh [output-path]
#
# Requirements: Go 1.24.x (go.mod requires go 1.24.0, toolchain go1.24.8).
# The binary is static (CGO_ENABLED=0) and reports the release tag and commit.
set -euo pipefail

TAG="RELEASE.2025-10-15T17-29-55Z"
SHA="9e49d5e7a648f00e26f2246f4dc28e6b07f8c84a"
SOURCE_SHA="be6d0bd3696c3a13a35f02d3a0280b64319c67918b4501c5c3d87f96d000085c"
OUT="${1:-$(pwd)/minio-${TAG}}"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

command -v go >/dev/null 2>&1 || {
  echo "error: go not found in PATH (need 1.24.x)" >&2
  exit 1
}

echo "downloading ${TAG} source"
curl -fsSL --retry 3 --max-time 180 \
  -o "$WORK/minio.tar.gz" \
  "https://github.com/minio/minio/archive/refs/tags/${TAG}.tar.gz"

echo "${SOURCE_SHA}  ${WORK}/minio.tar.gz" | sha256sum -c - || {
  echo "error: source archive does not match the pinned source_sha256 in botserver/3rdparty.toml" >&2
  exit 1
}

tar xzf "$WORK/minio.tar.gz" -C "$WORK"

LDFLAGS="-X github.com/minio/minio/cmd.Version=${TAG}"
LDFLAGS+=" -X github.com/minio/minio/cmd.ReleaseTag=${TAG}"
LDFLAGS+=" -X github.com/minio/minio/cmd.CommitID=${SHA}"
LDFLAGS+=" -X github.com/minio/minio/cmd.ShortCommitID=${SHA:0:12}"

echo "building ${OUT}"
(
  cd "$WORK/minio-${TAG}"
  CGO_ENABLED=0 go build -tags kqueue -trimpath \
    --ldflags "$LDFLAGS" -o "$OUT" .
)

VERSION="$("$OUT" --version 2>&1 | head -2)"
echo "$VERSION"
case "$VERSION" in
  *"$TAG"*"${SHA:0:12}"*)
    echo "verified: ${TAG} includes the CVE-2025-62506 fix"
    ;;
  *)
    echo "error: built binary does not report ${TAG} / ${SHA:0:12}; refusing it" >&2
    exit 1
    ;;
esac

BINARY_SHA="$(sha256sum "$OUT" | awk '{print $1}')"
printf '%s\n' "$BINARY_SHA" > "${OUT}.sha256"
echo "binary sha256: ${BINARY_SHA} (written to ${OUT}.sha256)"
