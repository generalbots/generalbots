#!/usr/bin/env bash
# PROD-DEPENDENCIES.sh - runtime prerequisites entry point for General Bots
#
# Linux: verifies or installs everything needed to RUN the binaries through
#        DEPENDENCIES.sh (the package matrix). Build tooling lives in
#        DEV-DEPENDENCIES.sh.
# macOS: reports whether the packaged desktop bundle can run here. The bundle
#        has no dmg target and no macOS CI job, and agent mode has no native
#        Incus daemon, so the script reports the platform as unsupported
#        instead of pretending a download exists.
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
workflow="$root/.github/workflows/desktop-binaries.yml"

ok() { printf '  [ok]      %s\n' "$1"; }
missing() { printf '  [missing] %s\n' "$1"; }
info() { printf '  [info]    %s\n' "$1"; }

show_install_guidance() {
    cat <<'MSG'
No installation is performed by this script on macOS.

macOS desktop bundle: NOT PUBLISHED
  - botapp/tauri.desktop-local.conf.json has no dmg bundle target.
  - .github/workflows/desktop-binaries.yml has no macOS job.
  - Use the Linux x86_64 or Windows x64 desktop bundle.

If you need a local development server on macOS:
  xcode-select --install
  cargo build -p botserver
  cargo build -p botui
This is a source-tree development setup, not the packaged desktop edition.

Agent mode: Incus via WSL2 is unavailable on macOS. Run agent actions on a
Linux host or on Windows with WSL2.
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

if [ -f "$workflow" ] && grep -Eq 'runs-on:[[:space:]]*macos|macos-latest|macos-[0-9]+' "$workflow"; then
    ok "macOS job present in .github/workflows/desktop-binaries.yml"
else
    missing "no macOS job in .github/workflows/desktop-binaries.yml"
    bundle_unsupported=1
fi

if [ "$bundle_unsupported" -eq 1 ]; then
    info "supported desktop platforms are Linux x86_64 and Windows x64."
    info "the download page and the first-run wizard must not offer macOS."
fi

echo
echo "--- Agent mode (Incus) ---"
missing "agent mode is not supported on macOS (no WSL2, no native Incus daemon)."
info "run agent actions on a Linux host or on Windows with WSL2."

echo
if [ "$bundle_unsupported" -eq 1 ]; then
    echo "Result: UNSUPPORTED - the General Bots desktop bundle is not built for macOS."
    exit 2
fi
echo "Result: AGENT UNAVAILABLE - the desktop bundle could run, but agent mode cannot."
exit 2
