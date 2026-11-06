#!/usr/bin/env bash
# Build the MinIO server binary from the official source tag.
#
# Why this exists: the release that fixes CVE-2025-62506
# (RELEASE.2025-10-15T17-29-55Z) publishes **no binary assets**, dl.min.io
# returns 410 for every path and the container images are not anonymously
# pullable. Building the official tag is the only way to get the patch.
#
# Usage:
#   ./scripts/build-minio-fixed.sh [output-path]
#
# Requirements: Go 1.24.x (go.mod requires go 1.24.0, toolchain go1.24.8).
# The binary is static (CGO_ENABLED=0) and reports the release tag and commit.
set -euo pipefail

TAG="RELEASE.2025-10-15T17-29-55Z"
SHA="9e49d5e7a648f00e26f2246f4dc28e6b07f8c84a"
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

"$OUT" --version | head -2
sha256sum "$OUT"
