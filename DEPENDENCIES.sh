#!/bin/bash
#
# DEPENDENCIES.sh - runtime dependencies for General Bots (implementation)
#
# Installs or verifies the system packages required to RUN the botserver
# binary, the bundled services and the desktop bundle. Build tooling lives in
# DEV-DEPENDENCIES.sh; PROD-DEPENDENCIES.sh is the supported entry point.
#
# Usage:
#   sudo ./DEPENDENCIES.sh --install   # install the runtime set (default)
#   ./DEPENDENCIES.sh --check          # verify without root; exit 1 when incomplete
#   ./DEPENDENCIES.sh --help
#
# Environment:
#   GB_SERVICE_USER  user added to incus-admin (default: $SUDO_USER)
#

set -u
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

RED=''; GREEN=''; YELLOW=''; NC=''
if [ -t 1 ]; then
    RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; NC='\033[0m'
fi

usage() {
    cat <<EOF
Usage: $0 --check | --install

  --check     Verify commands, libraries, groups and packages without root.
              Prints every missing item and exits non-zero.
  --install   Install the missing runtime set and configure incus-admin.
              Requires root (use sudo). This is the default mode.

Environment:
  GB_SERVICE_USER   user added to incus-admin (default: \$SUDO_USER)
EOF
}

MODE=install
case "${1:-}" in
    --check) MODE=check ;;
    --install) MODE=install ;;
    -h|--help) usage; exit 0 ;;
    "") ;;
    *) echo "Unknown option: $1" >&2; usage >&2; exit 2 ;;
esac

if [ "$MODE" = install ] && [ "${EUID:-$(id -u)}" -ne 0 ]; then
    echo -e "${RED}Error: --install requires root (use sudo)${NC}" >&2
    echo "Run '$0 --check' to verify the host without root." >&2
    exit 1
fi

OS_ID=unknown
OS_LIKE=''
if [ -r /etc/os-release ]; then
    . /etc/os-release
    OS_ID=${ID:-unknown}
    OS_LIKE=${ID_LIKE:-}
else
    echo -e "${RED}Error: cannot read /etc/os-release; unsupported host${NC}" >&2
    exit 1
fi

FAMILY=unsupported
case "$OS_ID" in
    ubuntu|debian|linuxmint|pop|elementary|raspbian|neon) FAMILY=debian ;;
    fedora|rhel|centos|rocky|almalinux|ol) FAMILY=fedora ;;
    arch|manjaro|endeavouros) FAMILY=arch ;;
    alpine) FAMILY=alpine ;;
    *)
        case " $OS_LIKE " in
            *" debian "*|*" ubuntu "*) FAMILY=debian ;;
            *" fedora "*|*" rhel "*) FAMILY=fedora ;;
            *" arch "*) FAMILY=arch ;;
        esac
        ;;
esac

if [ "$FAMILY" = unsupported ]; then
    echo -e "${RED}Unsupported OS: $OS_ID${NC}" >&2
    echo "Supported families: Debian/Ubuntu, Fedora/RHEL, Arch, Alpine." >&2
    echo "Required: libpq, libssl, liblzma, zlib, clang runtime, poppler," >&2
    echo "libzbar, incus, GTK3/WebKitGTK 4.1/libsoup3, fonts." >&2
    exit 1
fi

# Runtime matrix. A "|" separates alternative package names; the first one
# available is installed. libclang1 is versioned on most releases.
runtime_packages=''
case "$FAMILY" in
    debian)
        runtime_packages="libpq5 libssl3t64|libssl3 liblzma5 zlib1g ca-certificates curl wget libclang1|libclang1-20|libclang1-19|libclang1-18|libclang1-17|libclang1-16|libclang1-15|libclang1-14 poppler-utils libzbar0t64|libzbar0 incus libgtk-3-0t64|libgtk-3-0 libwebkit2gtk-4.1-0 libsoup-3.0-0 librsvg2-2 libayatana-appindicator3-1 fontconfig fonts-dejavu-core fonts-liberation"
        ;;
    fedora)
        runtime_packages="libpq openssl-libs xz-libs zlib ca-certificates curl wget abseil-cpp clang-libs pkgconf-pkg-config poppler-utils zbar incus gtk3 webkit2gtk4.1 libsoup3 librsvg2 libappindicator-gtk3 fontconfig dejavu-sans-fonts liberation-fonts"
        ;;
    arch)
        runtime_packages="postgresql-libs openssl xz zlib ca-certificates curl wget abseil-cpp clang pkgconf poppler zbar incus gtk3 webkit2gtk-4.1 libsoup3 librsvg libappindicator-gtk3 fontconfig ttf-dejavu ttf-liberation"
        ;;
    alpine)
        runtime_packages="libpq openssl xz-libs zlib ca-certificates curl wget abseil-cpp clang pkgconf poppler-utils zbar incus gtk+3.0 webkit2gtk-4.1 libsoup3 librsvg libayatana-appindicator fontconfig font-dejavu font-liberation"
        ;;
esac
read -r -a RUNTIME_PACKAGES <<< "$runtime_packages"
RUNTIME_COMMANDS=(incus pdftotext curl wget)
RUNTIME_LIBS=(libpq.so.5 libssl.so.3 liblzma.so.5 libz.so.1 libclang- libpoppler.so
    libzbar.so.0 libgtk-3.so.0 libwebkit2gtk-4.1.so.0 libsoup-3.0.so.0
    librsvg-2.so.2 libfontconfig.so.1)

FAILED=()
LDCONFIG_CACHE=''
LDCONFIG_READY=0

group_exists() { grep -q "^$1:" /etc/group 2>/dev/null; }

is_member() {
    case " $(id -nG "$1" 2>/dev/null) " in
        *" $2 "*) return 0 ;;
    esac
    return 1
}

spec_display() {
    case "$1" in
        libclang1*) printf 'libclang1 (or a versioned libclang1-NN)' ;;
        *) printf '%s' "${1//|/ or }" ;;
    esac
}

have_lib() {
    if [ "$LDCONFIG_READY" -eq 0 ]; then
        LDCONFIG_READY=1
        if command -v ldconfig >/dev/null 2>&1; then
            LDCONFIG_CACHE=$(ldconfig -p 2>/dev/null || true)
        fi
    fi
    case "$LDCONFIG_CACHE" in
        *"$1"*) return 0 ;;
    esac
    local pattern
    for pattern in "/usr/lib/$1"* "/usr/lib/*/$1"* "/lib/$1"* "/lib/*/$1"* "/usr/lib64/$1"*; do
        compgen -G "$pattern" >/dev/null 2>&1 && return 0
    done
    return 1
}

pkg_installed() {
    case "$FAMILY" in
        debian)
            local status
            status=$(dpkg-query -W -f='${Status}' "$1" 2>/dev/null || true)
            case "$status" in *"install ok installed"*) return 0 ;; esac
            return 1
            ;;
        fedora) rpm -q "$1" >/dev/null 2>&1 ;;
        arch) pacman -Q "$1" >/dev/null 2>&1 ;;
        alpine) apk info -e "$1" >/dev/null 2>&1 ;;
        *) return 1 ;;
    esac
}

pkg_installed_any() {
    local -a names
    local name
    IFS='|' read -r -a names <<< "$1"
    for name in "${names[@]}"; do
        pkg_installed "$name" && return 0
    done
    return 1
}

apt_pick() {
    local -a names
    local name
    IFS='|' read -r -a names <<< "$1"
    for name in "${names[@]}"; do
        if apt-cache show "$name" >/dev/null 2>&1; then
            printf '%s\n' "$name"
            return 0
        fi
    done
    return 1
}

missing_item() { printf '  MISSING  %-10s %s\n' "$1" "$2"; }

check_requirements() {
    local missing=0
    local cmd lib spec version
    local check_user="${SUDO_USER:-$(id -un)}"

    echo -e "${YELLOW}Verifying runtime prerequisites: $OS_ID (family: $FAMILY)${NC}"
    echo

    for cmd in "${RUNTIME_COMMANDS[@]}"; do
        command -v "$cmd" >/dev/null 2>&1 || { missing_item "command" "$cmd"; missing=$((missing + 1)); }
    done
    for lib in "${RUNTIME_LIBS[@]}"; do
        have_lib "$lib" || { missing_item "library" "${lib}*"; missing=$((missing + 1)); }
    done
    for spec in "${RUNTIME_PACKAGES[@]}"; do
        # The libclang soname above is authoritative; the package is versioned.
        case "$spec" in libclang1*) continue ;; esac
        pkg_installed_any "$spec" || { missing_item "package" "$(spec_display "$spec")"; missing=$((missing + 1)); }
    done

    if ! group_exists incus-admin; then
        missing_item "group" "incus-admin"
        missing=$((missing + 1))
    elif ! is_member "$check_user" incus-admin; then
        missing_item "membership" "$check_user is not in incus-admin"
        missing=$((missing + 1))
    fi

    if command -v incus >/dev/null 2>&1 &&
        { [ "${EUID:-$(id -u)}" -eq 0 ] || is_member "$check_user" incus-admin; }; then
        if ! incus version >/dev/null 2>&1; then
            missing_item "service" "incus daemon not reachable (sudo systemctl enable --now incus)"
            missing=$((missing + 1))
        elif ! incus storage list --format csv 2>/dev/null | grep -q .; then
            missing_item "storage" "no incus storage pool (sudo incus admin init --minimal)"
            missing=$((missing + 1))
        fi
    fi

    echo
    if [ "$missing" -eq 0 ]; then
        echo -e "${GREEN}All runtime prerequisites are satisfied.${NC}"
        if command -v incus >/dev/null 2>&1; then
            version=$(incus version 2>&1 || true)
            [ -n "$version" ] && printf '%s\n' "$version" | sed 's/^/  /'
        fi
        return 0
    fi
    echo -e "${RED}$missing runtime prerequisite(s) missing.${NC}"
    echo "Install with: sudo $SCRIPT_DIR/PROD-DEPENDENCIES.sh --install"
    return 1
}

incus_guidance() {
    echo
    echo -e "${YELLOW}incus is not installed and is not in the configured repositories.${NC}"
    case "$FAMILY" in
        debian)
            echo "  Debian: enable trixie or bookworm-backports; Ubuntu 22.04: add the"
            echo "  Zabbly repository (https://github.com/zabbly/incus), then re-run."
            ;;
        fedora) echo "  Fedora: sudo dnf copr enable ganto/incus"; echo "  Then re-run: sudo ./PROD-DEPENDENCIES.sh --install" ;;
        arch) echo "  Arch: incus is in [extra]; refresh with sudo pacman -Syu and re-run." ;;
        alpine) echo "  Alpine: incus is in [community]; enable it (or use edge) and re-run." ;;
    esac
}

install_debian() {
    export DEBIAN_FRONTEND=noninteractive
    local spec pkg
    apt-get update -qq || FAILED+=("apt-get update")
    for spec in "${RUNTIME_PACKAGES[@]}"; do
        if pkg=$(apt_pick "$spec"); then
            apt-get install -y "$pkg" || FAILED+=("$pkg")
        else
            FAILED+=("$(spec_display "$spec") (not in configured apt repositories)")
        fi
    done
}

install_batch() {
    local pkg
    for pkg in "${RUNTIME_PACKAGES[@]}"; do
        "$@" "$pkg" || FAILED+=("$pkg")
    done
}

install_fedora() {
    dnf install -y "${RUNTIME_PACKAGES[@]}" || install_batch dnf install -y
}

install_arch() {
    pacman -Sy --noconfirm || FAILED+=("pacman -Sy")
    pacman -S --needed --noconfirm "${RUNTIME_PACKAGES[@]}" ||
        install_batch pacman -S --needed --noconfirm
}

install_alpine() {
    apk add --no-cache "${RUNTIME_PACKAGES[@]}" || install_batch apk add --no-cache
}

install_runtime() {
    echo -e "${YELLOW}Installing runtime prerequisites: $OS_ID (family: $FAMILY)${NC}"
    case "$FAMILY" in
        debian) install_debian ;;
        fedora) install_fedora ;;
        arch) install_arch ;;
        alpine) install_alpine ;;
    esac
    pkg_installed incus || incus_guidance
}

post_install() {
    local target_user="${GB_SERVICE_USER:-${SUDO_USER:-}}"
    local version=''
    local attempt

    echo
    echo -e "${YELLOW}Configuring incus-admin${NC}"
    if ! group_exists incus-admin; then
        if command -v groupadd >/dev/null 2>&1; then
            groupadd --system incus-admin && echo "Created group incus-admin."
        elif command -v addgroup >/dev/null 2>&1; then
            addgroup -S incus-admin && echo "Created group incus-admin."
        else
            echo "Could not create group incus-admin; create it manually." >&2
        fi
    fi

    if [ -n "$target_user" ]; then
        if is_member "$target_user" incus-admin; then
            echo "User $target_user is already in incus-admin."
        elif command -v usermod >/dev/null 2>&1 && usermod -aG incus-admin "$target_user"; then
            echo "Added $target_user to incus-admin."
        elif command -v adduser >/dev/null 2>&1 && adduser "$target_user" incus-admin; then
            echo "Added $target_user to incus-admin."
        else
            echo "Could not add $target_user to incus-admin; run: sudo usermod -aG incus-admin $target_user" >&2
        fi
    else
        echo "No invoking user detected; add the service user with:"
        echo "  sudo usermod -aG incus-admin <user>"
    fi

    if command -v systemctl >/dev/null 2>&1 && [ -d /run/systemd/system ]; then
        systemctl enable --now incus >/dev/null 2>&1 || true
    elif command -v rc-service >/dev/null 2>&1; then
        rc-update add incus default >/dev/null 2>&1 || true
        rc-service incus start >/dev/null 2>&1 || true
    fi

    echo
    echo "Group changes apply to new sessions: log out and back in, or run"
    echo "  newgrp incus-admin"
    echo "Restart the service so it inherits the group:"
    echo "  sudo systemctl restart botserver"
    echo "If the incus daemon is not running: sudo systemctl enable --now incus"
    echo

    if command -v incus >/dev/null 2>&1; then
        for attempt in 1 2 3; do
            version=$(incus version 2>&1 || true)
            [ -n "$version" ] && break
            sleep 1
        done
        if [ -n "$version" ]; then
            echo "incus version:"
            printf '%s\n' "$version" | sed 's/^/  /'
            if ! incus storage list --format csv 2>/dev/null | grep -q .; then
                echo "Initializing Incus with the minimal profile..."
                incus admin init --minimal ||
                    echo "Could not initialize Incus; run: sudo incus admin init --minimal" >&2
            fi
        else
            echo "incus is installed but its daemon is not reachable yet."
        fi
    else
        echo "incus is not on PATH; open a new shell after the install."
    fi
}

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}  General Bots runtime dependencies${NC}"
echo -e "${GREEN}========================================${NC}"

if [ "$MODE" = check ]; then
    check_requirements
    exit $?
fi

install_runtime
post_install

if [ "${#FAILED[@]}" -gt 0 ]; then
    echo
    echo -e "${RED}Some runtime items could not be installed:${NC}"
    for item in "${FAILED[@]}"; do
        echo "  - $item"
    done
    echo
    echo "Install them manually, then verify with: ./PROD-DEPENDENCIES.sh --check"
    exit 1
fi

echo
echo -e "${GREEN}Runtime dependencies installed.${NC}"
echo "Verify with: ./PROD-DEPENDENCIES.sh --check"
