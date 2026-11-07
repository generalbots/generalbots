#!/usr/bin/env bash
# PROD-DEPENDENCIES.sh - runtime prerequisites entry point for General Bots
#
# Linux: verifies or installs everything needed to RUN the binaries through
#        DEPENDENCIES.sh (the package matrix). Build tooling lives in
#        DEV-DEPENDENCIES.sh.
# macOS: reports whether the packaged desktop bundle can run here. The dmg
#        target and the GitHub-only macOS workflow exist, but the Linux service
#        stack and the Incus agent have no native darwin builds, so the script
#        reports exactly what is unavailable instead of pretending a complete
#        local stack exists.
# Windows uses PROD-DEPENDENCIES.ps1.
#
# Usage:
#   ./PROD-DEPENDENCIES.sh --check     verify; no root, installs nothing
#   sudo ./PROD-DEPENDENCIES.sh --install
#
# Environment (Linux):
#   GB_SERVICE_USER   user added to incus-admin (default: $SUDO_USER)
#
set -u

root="$(cd "$(dirname "$0")" && pwd)"

usage() {
    cat <<'USAGE'
Usage: ./PROD-DEPENDENCIES.sh [--check|--install]

  Linux:
    --check    verify runtime commands, libraries, groups and packages (no root)
    --install  install the runtime set and configure incus-admin (sudo)

  macOS:
    --check    report whether the packaged desktop bundle can run here
    --install  print remediation guidance; installs nothing

  Windows: .\PROD-DEPENDENCIES.ps1 -Check | -Install
USAGE
}

mode="check"
case "${1:-}" in
    --check) mode="check" ;;
    --install) mode="install" ;;
    -h|--help) usage; exit 0 ;;
    "") ;;
    *)
        echo "unknown option: $1" >&2
        usage >&2
        exit 64
        ;;
esac

os="$(uname -s)"

if [ "$os" = "Linux" ]; then
    case "${1:-}" in
        --check|--install)
            exec bash "$root/DEPENDENCIES.sh" "$1"
            ;;
        "")
            usage
            exit 2
            ;;
    esac
fi

if [ "$os" != "Darwin" ]; then
    cat >&2 <<'MSG'
PROD-DEPENDENCIES.sh supports Linux (runtime set) and macOS (desktop
support report).
  Linux:   ./PROD-DEPENDENCIES.sh --check
  Windows: .\PROD-DEPENDENCIES.ps1 -Check
MSG
    exit 3
fi

config="$root/botapp/tauri.desktop-local.conf.json"
workflows=(
    "$root/.github/workflows/desktop-binaries.yml"
    "$root/.github/workflows/desktop-macos.yml"
)

ok() { printf '  [ok]      %s\n' "$1"; }
missing() { printf '  [missing] %s\n' "$1"; }
info() { printf '  [info]    %s\n' "$1"; }

show_install_guidance() {
    cat <<'MSG'
No installation is performed by this script on macOS.

macOS desktop bundle: BEST EFFORT (UNSIGNED)
  - botapp/tauri.desktop-local.conf.json configures the dmg/app targets.
  - GitHub Actions builds it in .github/workflows/desktop-macos.yml.
  - The Linux service stack (PostgreSQL, Valkey, MinIO/mc, Qdrant, Vault,
    Zitadel, llama.cpp), the BotModels pythonbox and the Incus agent have no
    native darwin builds. Use a Linux VM or a remote Incus host.
  - Gatekeeper bypass for the unsigned app:
    xattr -dr com.apple.quarantine "/Applications/General Bots.app"

If you need a local development server on macOS:
  xcode-select --install
  cargo build -p botserver
  cargo build -p botui
This is a source-tree development setup, not a fully provisioned desktop
edition.

Agent mode: Incus has no native daemon on macOS. Run agent actions on a Linux
host, or configure a remote Incus endpoint (`incus remote add`).
MSG
}

if [ "$mode" = "install" ]; then
    show_install_guidance
    exit 0
fi

echo "========================================"
echo "  General Bots runtime prerequisites"
echo "  (macOS)"
echo "========================================"
echo
info "macOS $(sw_vers -productVersion 2>/dev/null || echo unknown) on $(uname -m)"

echo
echo "--- Desktop bundle ---"
bundle_unsupported=0

if [ -f "$config" ] && grep -q '"dmg"' "$config"; then
    ok "dmg bundle target configured in botapp/tauri.desktop-local.conf.json"
else
    missing "no dmg bundle target in botapp/tauri.desktop-local.conf.json"
    bundle_unsupported=1
fi

macos_job=""
for candidate in "${workflows[@]}"; do
    if [ -f "$candidate" ] && grep -Eq 'runs-on:[[:space:]]*macos|macos-latest|macos-[0-9]+' "$candidate"; then
        macos_job="${candidate#"$root"/}"
        break
    fi
done
if [ -n "$macos_job" ]; then
    ok "macOS job present in $macos_job"
else
    missing "no macOS job in desktop-binaries.yml or desktop-macos.yml"
    bundle_unsupported=1
fi

if [ "$bundle_unsupported" -eq 1 ]; then
    info "supported desktop targets are Linux x86_64, Windows x64 and macOS arm64 (best effort, unsigned)."
    info "the download page and the first-run wizard must report unavailable macOS pieces."
fi

echo
echo "--- Agent mode (Incus) ---"
missing "agent mode needs remote Incus or a Linux VM on macOS (no native Incus daemon)."
info "run agent actions on a Linux host, or configure a remote Incus endpoint."

echo
if [ "$bundle_unsupported" -eq 1 ]; then
    echo "Result: UNSUPPORTED - the macOS desktop bundle configuration is incomplete."
    exit 2
fi
echo "Result: PARTIAL SUPPORT - the macOS bundle is configured, but the Linux service stack and agent mode cannot run natively."
exit 2
