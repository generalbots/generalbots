#!/usr/bin/env bash
# ==========================================
# General Bots - Windows Cross-Compilation
# ==========================================
# Self-contained: installs all deps and builds botserver.exe
#
# Usage:
#   ./build-win.sh                  # build (installs deps if needed)
#   ./build-win.sh --no-deps        # skip dependency install
#   ./build-win.sh check            # verify prerequisites only
#   ./build-win.sh wine             # test .exe under Wine
# ==========================================

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT_DIR"

TARGET="x86_64-pc-windows-gnu"
PG_DIR="/tmp/pg-windows/pgsql"
PG_URL="https://get.enterprisedb.com/postgresql/postgresql-17.4-1-windows-x64-binaries.zip"
PG_ZIP="/tmp/pg-windows/postgresql-windows.zip"
BUILD_LOG="/tmp/build_win.log"

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; CYAN='\033[0;36m'; NC='\033[0m'
info()  { echo -e "${GREEN}[INFO]${NC} $*"; }
warn()  { echo -e "${YELLOW}[WARN]${NC} $*"; }
err()   { echo -e "${RED}[ERR]${NC}  $*" >&2; }
header(){ echo -e "\n${CYAN}====== $* ======${NC}\n"; }

# ---- 1. System dependencies ----
install_system_deps() {
    header "System Dependencies"

    local pkgs=()
    # MinGW cross-compiler
    if ! command -v x86_64-w64-mingw32-gcc &>/dev/null; then
        pkgs+=(mingw-w64)
    fi
    # lld - low-memory PE/COFF linker
    if ! command -v lld &>/dev/null; then
        pkgs+=(lld)
    fi
    # Wine for testing
    if ! command -v wine &>/dev/null; then
        pkgs+=(wine)
        # Enable 32-bit architecture for wine32 if needed
        if ! dpkg --print-foreign-architectures | grep -q i386; then
            info "Enabling i386 architecture for Wine 32-bit support..."
            sudo dpkg --add-architecture i386
            NEED_APT_UPDATE=true
        fi
    fi
    # Build essentials
    if ! command -v make &>/dev/null; then
        pkgs+=(build-essential)
    fi
    # curl, wget for downloads
    if ! command -v curl &>/dev/null; then
        pkgs+=(curl)
    fi
    if ! command -v wget &>/dev/null; then
        pkgs+=(wget)
    fi
    # unzip for PostgreSQL
    if ! command -v unzip &>/dev/null; then
        pkgs+=(unzip)
    fi
    # pkg-config
    if ! command -v pkg-config &>/dev/null; then
        pkgs+=(pkg-config)
    fi

    if [ ${#pkgs[@]} -gt 0 ]; then
        info "Installing: ${pkgs[*]}"
        if [ "${NEED_APT_UPDATE:-false}" = "true" ] || ! command -v lld &>/dev/null; then
            sudo apt-get update -qq
        fi
        sudo apt-get install -y -qq "${pkgs[@]}"
        info "System deps installed."
    else
        info "All system dependencies already satisfied."
    fi

    # Wine architecture enablement (if not already)
    if command -v wine &>/dev/null; then
        if ! wine --version &>/dev/null 2>&1; then
            warn "Wine not fully configured, running winecfg..."
            wine wineboot -u 2>/dev/null || true
        fi
    fi
}

# ---- 2. Rust target ----
install_rust_target() {
    header "Rust Target"
    if ! rustup target list --installed | grep -q "$TARGET"; then
        info "Adding Rust target $TARGET..."
        rustup target add "$TARGET"
        info "Target installed."
    else
        info "Target $TARGET already installed."
    fi
}

# ---- 3. PostgreSQL Windows libs ----
install_postgres() {
    header "PostgreSQL Windows Libraries (libpq)"

    if [ -f "$PG_DIR/lib/libpq.dll.a" ]; then
        info "PostgreSQL libpq found at $PG_DIR"
        return 0
    fi

    info "Downloading PostgreSQL 17.4 Windows binaries..."
    mkdir -p /tmp/pg-windows
    wget -q --show-progress "$PG_URL" -O "$PG_ZIP"
    info "Extracting..."
    unzip -q -o "$PG_ZIP" -d /tmp/pg-windows

    # The zip contains a 'pgsql' directory at root
    if [ ! -f "$PG_DIR/lib/libpq.dll.a" ]; then
        # Try to find pgsql directory
        PG_EXTRACTED=$(find /tmp/pg-windows -name "libpq.dll.a" 2>/dev/null | head -1)
        if [ -z "$PG_EXTRACTED" ]; then
            err "Could not find libpq.dll.a in extracted archive"
            err "Extracted contents:"
            ls -la /tmp/pg-windows/
            return 1
        fi
        # Create symlink or move
        PG_EXTRACTED_DIR=$(dirname "$(dirname "$PG_EXTRACTED")")
        info "Found PostgreSQL at $PG_EXTRACTED_DIR, linking to $PG_DIR"
        ln -sfn "$PG_EXTRACTED_DIR" "$PG_DIR" 2>/dev/null || \
            cp -r "$PG_EXTRACTED_DIR"/* "$PG_DIR/"
    fi

    if [ -f "$PG_DIR/lib/libpq.dll.a" ]; then
        info "PostgreSQL libpq installed successfully."
    else
        err "PostgreSQL installation failed."
        return 1
    fi
}

# ---- 4. Runtime DLLs ----
install_runtime_dlls() {
    header "Windows Runtime DLLs"

    DLL_DIR="$ROOT_DIR/target/$TARGET/debug"
    mkdir -p "$DLL_DIR"

    local dlls=(
        libpq.dll
        libssl-3-x64.dll libcrypto-3-x64.dll
        libintl-9.dll libiconv-2.dll
        libwinpthread-1.dll
        vcruntime140.dll vcruntime140_1.dll
    )
    local copied=0

    for dll in "${dlls[@]}"; do
        if [ -f "$DLL_DIR/$dll" ]; then
            continue
        fi
        src=$(find "$PG_DIR" /usr/lib/gcc /usr/x86_64-w64-mingw32 \
            -name "$dll" 2>/dev/null | head -1)
        if [ -n "$src" ]; then
            cp -v "$src" "$DLL_DIR/$dll"
            ((copied++))
        else
            warn "DLL not found: $dll (botserver.exe may not run without it)"
        fi
    done

    # Also copy from winpthread location
    if [ ! -f "$DLL_DIR/libwinpthread-1.dll" ]; then
        src=$(find /usr -name "libwinpthread-1.dll" 2>/dev/null | head -1)
        if [ -n "$src" ]; then
            cp -v "$src" "$DLL_DIR/libwinpthread-1.dll"
            ((copied++))
        fi
    fi

    if [ "$copied" -gt 0 ]; then
        info "Copied $copied DLL(s) to $DLL_DIR"
    else
        info "All DLLs already present."
    fi
}

# ---- 5. Build ----
do_build() {
    header "Building botserver.exe"

    export PQ_LIB_DIR_X86_64_PC_WINDOWS_GNU="$PG_DIR/lib"
    info "PQ_LIB_DIR_X86_64_PC_WINDOWS_GNU=$PG_DIR/lib"

    # Memory-minimizing flags for systems with as little as 3GB RAM
    # lld in GNU mode uses ~500MB peak vs GNU ld ~3GB+
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

    info "Linker: lld (GNU emulation for PE/COFF)"
    info "LTO=off, codegen-units=1, jobs=1"

    cargo build -p botserver --target "$TARGET" 2>&1 | tail -30

    local exe="$ROOT_DIR/target/$TARGET/debug/botserver.exe"
    if [ -f "$exe" ]; then
        echo ""
        info "=========================================="
        info "  BUILD COMPLETE"
        info "=========================================="
        ls -lh "$exe"
        file "$exe"
    else
        err "Build failed: $exe not found"
        return 1
    fi
}

# ---- 6. Wine test ----
test_wine() {
    header "Testing with Wine"

    local exe="$ROOT_DIR/target/$TARGET/debug/botserver.exe"
    if [ ! -f "$exe" ]; then
        err "botserver.exe not found. Run ./build-win.sh first."
        return 1
    fi

    if ! command -v wine &>/dev/null; then
        err "Wine not installed. Run sudo apt install wine wine32 wine64"
        return 1
    fi

    info "Testing $exe with Wine..."
    wine "$exe" --help 2>&1 | head -30
    echo ""

    if wine "$exe" --help >/dev/null 2>&1; then
        info "Wine test PASSED"
    else
        warn "Wine test had issues (may be missing DLLs)"
    fi
}

# ---- Main ----
main() {
    echo "=============================================="
    echo "  GB Windows Cross-Compilation"
    echo "  Target: $TARGET"
    echo "=============================================="

    local mode="${1:-build}"

    case "$mode" in
        build)
            install_system_deps
            install_rust_target
            install_postgres
            install_runtime_dlls
            do_build
            ;;
        --no-deps)
            info "Skipping dependency installation..."
            do_build
            ;;
        check)
            install_system_deps
            install_rust_target
            install_postgres
            install_runtime_dlls
            info "All prerequisites satisfied."
            ;;
        wine)
            test_wine
            ;;
        clean)
            cargo clean --target "$TARGET"
            info "Cleaned target/$TARGET"
            ;;
        full)
            # Full rebuild from scratch
            cargo clean --target "$TARGET"
            install_system_deps
            install_rust_target
            install_postgres
            install_runtime_dlls
            do_build
            test_wine
            ;;
        *)
            echo "Usage: $0 [build|--no-deps|check|wine|clean|full]"
            exit 1
            ;;
    esac
}

main "$@"
