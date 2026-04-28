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

install_mold() {
curl -L "https://github.com/rui314/mold/releases/download/v2.4.0/mold-2.4.0-x86_64-linux.tar.gz" -o /tmp/mold.tar.gz
tar -xzf /tmp/mold.tar.gz -C /tmp
cp "/tmp/mold-2.4.0-x86_64-linux/bin/mold" /usr/local/bin/
rm -rf /tmp/mold-2.4.0* /tmp/mold.tar.gz
ldconfig
}

command -v mold &> /dev/null || install_mold

echo "Dev dependencies installed!"
echo ""
echo "✅ Tools installed: clang, lld, mold, sccache"
echo "📦 Project will use .cargo/config.toml from workspace"
echo "⚡ Link time reduced by ~30-40% with mold/lld"
echo ""
echo "Next steps:"
echo "  1. Run: ./DEV-DEPENDENCIES.sh (already done)"
echo "  2. Workspace .cargo/config.toml will be used automatically"
echo "  3. Build: cargo build -p botserver --bin botserver"