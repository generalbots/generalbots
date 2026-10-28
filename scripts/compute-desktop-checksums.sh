#!/usr/bin/env bash
# compute-desktop-checksums.sh
#
# Downloads desktop-local component binaries and prints SHA256 values
# ready to paste into 3rdparty.toml.
#
# Usage:
#   ./scripts/compute-desktop-checksums.sh [linux|windows]
#
# Requires: curl, sha256sum (or shasum -a 256 on macOS)

set -euo pipefail

PLATFORM="${1:-windows}"
TMPDIR="$(mktemp -d)"
trap 'rm -rf "$TMPDIR"' EXIT

sha() {
  if command -v sha256sum &>/dev/null; then
    sha256sum "$1" | awk '{print $1}'
  else
    shasum -a 256 "$1" | awk '{print $1}'
  fi
}

# Desktop-local components (must match install.rs core_components + vault)
declare -A WIN_URLS=(
  [vault]="https://releases.hashicorp.com/vault/1.18.2/vault_1.18.2_windows_amd64.zip"
  [tables]="https://get.enterprisedb.com/postgresql/postgresql-17.2-1-windows-x64-binaries.zip"
  [cache]="https://github.com/microsoftarchive/redis/releases/download/win-3.0.504/Redis-x64-3.0.504.zip"
  [drive]="https://dl.min.io/server/minio/release/windows-amd64/minio.exe"
  [mc]="https://dl.min.io/client/mc/release/windows-amd64/mc.exe"
  [vector_db]="https://github.com/qdrant/qdrant/releases/download/v1.12.0/qdrant-x86_64-pc-windows-msvc.zip"
  [llm_win_cuda12]="https://github.com/ggml-org/llama.cpp/releases/download/b7345/llama-b7345-bin-win-cuda-12.4-x64.zip"
  [cudart_win_cuda12]="https://github.com/ggml-org/llama.cpp/releases/download/b7345/cudart-llama-bin-win-cuda-12.4-x64.zip"
)

declare -A LINUX_URLS=(
  [vault]="https://releases.hashicorp.com/vault/1.18.2/vault_1.18.2_linux_amd64.zip"
  [tables]="https://github.com/theseus-rs/postgresql-binaries/releases/download/17.2.0/postgresql-17.2.0-x86_64-unknown-linux-gnu.tar.gz"
  [cache]="https://download.valkey.io/releases/valkey-8.1.5-jammy-x86_64.tar.gz"
  [drive]="https://dl.min.io/server/minio/release/linux-amd64/minio"
  [mc]="https://dl.min.io/client/mc/release/linux-amd64/mc"
  [vector_db]="https://github.com/qdrant/qdrant/releases/download/v1.12.0/qdrant-x86_64-unknown-linux-gnu.tar.gz"
  [llm_linux_vulkan]="https://github.com/ggml-org/llama.cpp/releases/download/b7345/llama-b7345-bin-ubuntu-vulkan-x64.zip"
)

declare -A MODEL_URLS=(
  [deepseek_small]="https://huggingface.co/bartowski/DeepSeek-R1-Distill-Qwen-1.5B-GGUF/resolve/main/DeepSeek-R1-Distill-Qwen-1.5B-Q3_K_M.gguf"
  [bge_embedding]="https://huggingface.co/CompendiumLabs/bge-small-en-v1.5-gguf/resolve/main/bge-small-en-v1.5-f32.gguf"
)

if [ "$PLATFORM" = "windows" ]; then
  declare -n URLS=WIN_URLS
else
  declare -n URLS=LINUX_URLS
fi

echo "# SHA256 checksums for desktop-local ($PLATFORM)"
echo "# Generated on $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo ""

echo "## Components"
for comp in "${!URLS[@]}"; do
  url="${URLS[$comp]}"
  filename="${url##*/}"
  echo -n "Downloading $comp ($filename)... " >&2
  curl -fsSL -o "$TMPDIR/$filename" "$url"
  hash=$(sha "$TMPDIR/$filename")
  echo "done" >&2
  printf '%-25s sha256 = "%s"   # %s\n' "[$comp]" "$hash" "$filename"
  rm -f "$TMPDIR/$filename"
done

echo ""
echo "## Models"
for model in "${!MODEL_URLS[@]}"; do
  url="${MODEL_URLS[$model]}"
  filename="${url##*/}"
  echo -n "Downloading $model ($filename)... " >&2
  curl -fsSL -o "$TMPDIR/$filename" "$url"
  hash=$(sha "$TMPDIR/$filename")
  echo "done" >&2
  printf '%-25s sha256 = "%s"   # %s\n' "[$model]" "$hash" "$filename"
  rm -f "$TMPDIR/$filename"
done
