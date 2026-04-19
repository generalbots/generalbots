#!/bin/bash
set -e

if [ "$EUID" -ne 0 ]; then
    echo "Run as root (use sudo)"
    exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
echo "Installing runtime dependencies first..."
bash "$SCRIPT_DIR/DEPENDENCIES.sh"

echo "Installing dev/build dependencies..."
OS=$(grep -oP '(?<=^ID=).+' /etc/os-release 2>/dev/null | tr -d '"' || echo "unknown")

install_debian() {
    apt-get install -y -qq \
        clang lld build-essential pkg-config libssl-dev libpq-dev cmake git \
        libglib2.0-dev libgtk-3-dev libwebkit2gtk-4.1-dev libjavascriptcoregtk-4.1-dev \
        libayatana-appindicator3-dev librsvg2-dev libsoup-3.0-dev
}

install_fedora() {
    dnf install -y -q \
        clang lld gcc gcc-c++ make pkg-config openssl-devel postgresql-devel cmake git \
        glib2-devel gobject-introspection-devel gtk3-devel webkit2gtk3-devel \
        javascriptcoregtk-devel libappindicator-gtk3-devel librsvg2-devel libsoup3-devel
}

install_arch() {
    pacman -Sy --noconfirm \
        clang lld gcc make pkg-config openssl libpq cmake git \
        glib2 gtk3 webkit2gtk4 javascriptcoregtk libappindicator librsvg libsoup
}

case $OS in
    ubuntu|debian|linuxmint|pop) install_debian ;;
    fedora|rhel|centos|rocky|almalinux) install_fedora ;;
    arch|manjaro) install_arch ;;
    *) echo "Unsupported OS: $OS"; exit 1 ;;
esac

install_cargo_tools() {
    CARGO_BIN="${HOME}/.cargo/bin"
    if [ -f "$CARGO_BIN/cargo" ]; then
        export PATH="$CARGO_BIN:$PATH"
        . "$CARGO_BIN/env" 2>/dev/null
        cargo install mold --locked 2>/dev/null || true
    fi
}

command -v mold &> /dev/null || install_cargo_tools

echo "Dev dependencies installed!"