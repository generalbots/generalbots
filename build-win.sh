#!/usr/bin/env bash
# ==========================================
# General Bots - Windows Cross-Compilation
# ==========================================
# Builds botserver.exe for x86_64-pc-windows-gnu
#
# Environment: 3GB RAM target — minimal memory footprint
#
# Prerequisites (Linux host):
#   sudo apt install mingw-w64 lld
#   rustup target add x86_64-pc-windows-gnu
#
# Requires pre-built PostgreSQL Windows libs at /tmp/pg-windows/pgsql/
#
# Usage:
#   ./build-win.sh            # build
#   ./build-win.sh check      # verify prerequisites only
# ==========================================

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT_DIR"

TARGET="x86_64-pc-windows-gnu"
PG_DIR="/tmp/pg-windows/pgsql"

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; NC='\033[0m'
info()  { echo -e "${GREEN}[INFO]${NC} $*"; }
warn()  { echo -e "${YELLOW}[WARN]${NC} $*"; }
err()   { echo -e "${RED}[ERR]${NC}  $*" >&2; }

check_prereqs() {
    local ok=true
    if ! rustup target list --installed | grep -q "$TARGET"; then
        err "Rust target $TARGET not installed. Run: rustup target add $TARGET"
        ok=false
    fi
    if ! command -v x86_64-w64-mingw32-gcc &>/dev/null; then
        err "MinGW-w64 not found. Install: sudo apt install mingw-w64"
        ok=false
    fi
    if ! command -v lld &>/dev/null; then
        err "lld not found. Install: sudo apt install lld"
        ok=false
    fi
    if [ ! -f "$PG_DIR/lib/libpq.dll.a" ]; then
        err "PostgreSQL libpq not found at $PG_DIR/lib/"
        err "Download: https://get.enterprisedb.com/postgresql/postgresql-17.4-1-windows-x64-binaries.zip"
        err "Extract to /tmp/pg-windows/"
        ok=false
    fi
    $ok
}

setup_env() {
    export PQ_LIB_DIR_X86_64_PC_WINDOWS_GNU="$PG_DIR/lib"
    info "PQ_LIB_DIR_X86_64_PC_WINDOWS_GNU=$PG_DIR/lib"

    # Copy required DLLs alongside .exe
    DLL_DIR="$ROOT_DIR/target/$TARGET/debug"
    mkdir -p "$DLL_DIR"
    for dll in libpq.dll libssl-3-x64.dll libcrypto-3-x64.dll \
               libintl-9.dll libiconv-2.dll libwinpthread-1.dll \
               vcruntime140.dll vcruntime140_1.dll; do
        src="$(find "$PG_DIR" -name "$dll" 2>/dev/null | head -1)"
        if [ -n "$src" ] && [ ! -f "$DLL_DIR/$dll" ]; then
            cp -v "$src" "$DLL_DIR/$dll"
        fi
    done
}

do_build() {
    check_prereqs || return 1
    setup_env

    # Memory-minimizing flags for 3GB RAM systems
    # - lld in GNU mode (PE/COFF for MinGW) uses ~500MB vs GNU ld ~3GB+
    # - No LTO, single codegen unit, single job
    export RUSTFLAGS="\
        -C linker=lld \
        -C link-arg=-flavor \
        -C link-arg=gnu \
        -C link-arg=-target \
        -C link-arg=x86_64-windows-gnu \
        -C link-arg=-lwinpthread \
        -C lto=no \
        -C codegen-units=1 \
        -C link-arg=-Wl,--no-keep-memory \
        -C link-arg=-Wl,--reduce-memory-overheads \
    "
    export CARGO_BUILD_JOBS=1

    info "Building botserver.exe for $TARGET (low-memory mode)..."
    info "Linker: lld (GNU emulation), LTO=off, jobs=1"

    cargo build -p botserver --target "$TARGET"

    local exe="$ROOT_DIR/target/$TARGET/debug/botserver.exe"
    if [ -f "$exe" ]; then
        info "Build complete: $exe"
        ls -lh "$exe"
    else
        err "Build failed: $exe not found"
        return 1
    fi
}

main() {
    echo "=============================================="
    echo "  GB Windows Cross-Compilation (low-mem)"
    echo "  Target: $TARGET"
    echo "=============================================="
    echo ""

    case "${1:-build}" in
        build) do_build ;;
        check) check_prereqs ;;
        clean) cargo clean --target "$TARGET"; info "Cleaned" ;;
        *) echo "Usage: $0 [build|check|clean]"; exit 1 ;;
    esac
}

main "$@"
