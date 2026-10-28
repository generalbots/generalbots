#!/usr/bin/env bash
set -euo pipefail

# Build script for the General Bots Desktop-Local edition (Chat + Drive).
# Produces Windows (MSI/NSIS) and Linux (deb/AppImage) installers via Tauri.
#
# Prerequisites:
#   - Rust toolchain (rustup, cargo)
#   - Node/npm (for Tauri CLI: `cargo install tauri-cli`)
#   - Windows: NSIS, WiX Toolset
#   - Linux: dpkg-deb, libwebkit2gtk-4.1-dev, libssl-dev, AppImage tools
#
# Usage:
#   ./scripts/build-desktop-local.sh [--target windows|linux|all]

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BOTSERVER_DIR="$REPO_ROOT/botserver"
BOTAPP_DIR="$REPO_ROOT/botapp"
INSTALLERS_DIR="$BOTSERVER_DIR/botserver-installers"
BINARIES_DIR="$BOTAPP_DIR/binaries"

TARGET="${1:---target}"
PLATFORM="${2:-all}"

if [[ "$TARGET" == "--target" ]]; then
    PLATFORM="${PLATFORM}"
fi

echo "=== General Bots Desktop-Local Build ==="
echo "Platform: $PLATFORM"
echo ""

# Step 1: Build botserver with desktop-local features
echo "[1/4] Building botserver with desktop-local features..."
cargo build \
    -p botserver \
    --release \
    --no-default-features \
    --features desktop-local

# Step 2: Place botserver binary where Tauri expects it (externalBin)
echo "[2/4] Staging botserver binary for Tauri bundle..."
mkdir -p "$BINARIES_DIR"

if [[ "$OSTYPE" == "msys" || "$OSTYPE" == "win32" || "$OSTYPE" == "cygwin" ]]; then
    BOTSERVER_BIN="$REPO_ROOT/target/release/botserver.exe"
    # Tauri expects binaries named with target triple suffix
    TRIPLE="x86_64-pc-windows-msvc"
    cp "$BOTSERVER_BIN" "$BINARIES_DIR/botserver-${TRIPLE}.exe"
else
    BOTSERVER_BIN="$REPO_ROOT/target/release/botserver"
    TRIPLE="x86_64-unknown-linux-gnu"
    cp "$BOTSERVER_BIN" "$BINARIES_DIR/botserver-${TRIPLE}"
fi

echo "   Staged: $BINARIES_DIR/botserver-${TRIPLE}"

# Step 3: Ensure installer files are in place
echo "[3/4] Checking bundled installer files..."
mkdir -p "$INSTALLERS_DIR"

if [ -z "$(ls -A "$INSTALLERS_DIR" 2>/dev/null)" ]; then
    echo "   WARNING: $INSTALLERS_DIR is empty."
    echo "   For offline mode, place third-party binaries (tables, cache, drive, llm, vector_db)"
    echo "   in this directory before building. Use scripts/compute-desktop-checksums.sh to"
    echo "   download and verify them."
fi

# Step 4: Build Tauri installer
echo "[4/4] Building Tauri installer..."
cd "$BOTAPP_DIR"

TAURI_ARGS="--features desktop-local --config tauri.desktop-local.conf.json"

case "$PLATFORM" in
    windows)
        cargo tauri build $TAURI_ARGS --target x86_64-pc-windows-msvc
        ;;
    linux)
        cargo tauri build $TAURI_ARGS --target x86_64-unknown-linux-gnu
        ;;
    all)
        cargo tauri build $TAURI_ARGS
        ;;
    *)
        echo "Unknown platform: $PLATFORM (use windows, linux, or all)"
        exit 1
        ;;
esac

echo ""
echo "=== Build complete ==="
echo "Installers are in: $BOTAPP_DIR/target/release/bundle/"
