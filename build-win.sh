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

    if [ ! -f "$PG_ZIP" ] || [ ! -d "$PG_DIR/bin" ]; then
        info "Downloading PostgreSQL 17.4 Windows binaries..."
        mkdir -p /tmp/pg-windows
        wget -q --show-progress "$PG_URL" -O "$PG_ZIP"
        info "Extracting..."
        unzip -q -o "$PG_ZIP" -d /tmp/pg-windows
    else
        info "PostgreSQL Windows binaries already downloaded and extracted."
    fi

    if [ ! -f "$PG_DIR/lib/libpq.dll.a" ]; then
        if [ -f "$PG_DIR/bin/libpq.dll" ] && [ -f "$PG_DIR/lib/libpq.lib" ]; then
            info "Generating libpq.dll.a import library from MSVC .lib + DLL..."
            python3 - "$PG_DIR/lib/libpq.lib" "$PG_DIR/bin/libpq.dll" "$PG_DIR/lib/libpq.dll.a" <<'PYEOF' || true
import subprocess, sys, os, re

def extract_exports(lib_path):
    result = subprocess.run(["strings", "-n", "3", lib_path],
        capture_output=True, text=True, check=True)
    exports = set()
    for line in result.stdout.splitlines():
        line = line.strip()
        if not re.match(r'^[a-zA-Z_][a-zA-Z0-9_]*$', line):
            continue
        if line.startswith(('__', '@', 'B.')):
            continue
        if line in ('libpq', 'NULL', 'THUNK', 'DESCRIPTOR', 'idata',
                     'Microsoft', 'R', 'LINK', 'O'):
            continue
        if line.startswith(('PQ', 'pgtls', 'pg_', 'lo_', 'fe_',
                            'libpq_', 'append', 'create', 'destroy',
                            'enlarge', 'init', 'printf', 'reset',
                            'term', 'winsock_')):
            exports.add(line)
    return sorted(exports)

def create_def(exports, dll_name):
    lines = [f"LIBRARY {dll_name}", "EXPORTS"]
    lines.extend(exports)
    return "\n".join(lines) + "\n"

lib, dll, out = sys.argv[1], sys.argv[2], sys.argv[3]
exports = extract_exports(lib)
sys.stderr.write(f"Extracted {len(exports)} export symbols\n")
def_path = "/tmp/libpq_gen_" + str(os.getpid()) + ".def"
with open(def_path, "w") as f:
    f.write(create_def(exports, os.path.basename(dll)))
r = subprocess.run(["x86_64-w64-mingw32-dlltool",
    "-d", def_path, "-l", out, "-D", dll],
    capture_output=True, text=True)
os.unlink(def_path)
if r.returncode != 0:
    sys.stderr.write(f"dlltool: {r.stderr.strip()}\n")
sys.exit(0 if os.path.exists(out) and os.path.getsize(out) > 1000 else 1)
PYEOF
        fi
    fi

    if [ ! -f "$PG_DIR/lib/libpq.dll.a" ]; then
        if [ -f "$PG_DIR/lib/libpq.a" ]; then
            warn "Could not generate libpq.dll.a, falling back to static linking"
            info "Creating symlink libpq.dll.a -> libpq.a"
            ln -sf libpq.a "$PG_DIR/lib/libpq.dll.a"
        fi
    fi

    if [ -f "$PG_DIR/lib/libpq.dll.a" ]; then
        info "PostgreSQL libpq installed successfully."
    else
        err "PostgreSQL installation failed."
        err "Expected libpq.dll.a or libpq.a in $PG_DIR/lib/"
        ls -la "$PG_DIR/lib/" 2>/dev/null || true
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
            copied=$((copied + 1))
        else
            warn "DLL not found: $dll (botserver.exe may not run without it)"
        fi
    done

    # Also copy from winpthread location
    if [ ! -f "$DLL_DIR/libwinpthread-1.dll" ]; then
        src=$(find /usr -name "libwinpthread-1.dll" 2>/dev/null | head -1)
        if [ -n "$src" ]; then
            cp -v "$src" "$DLL_DIR/libwinpthread-1.dll"
            copied=$((copied + 1))
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
        -C linker=x86_64-w64-mingw32-gcc \
        -C link-arg=-lwinpthread \
        -C lto=no \
        -C codegen-units=1 \
    "
    export CARGO_BUILD_JOBS=1

    info "Linker: x86_64-w64-mingw32-gcc"
    info "LTO=off, codegen-units=1, jobs=1"

    cargo build -p botserver --target "$TARGET" 2>&1

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
    # Desativa-se temporariamente o encadeamento estrito de erros do pipeline para evitar abortamento prematuro por SIGPIPE
    set +e
    wine "$exe" --help 2>&1 | head -35
    set -e
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
