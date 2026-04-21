#!/bin/bash
set -e

if [ "$EUID" -ne 0 ]; then
echo "Run as root (use sudo)"
exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
SUDO_USER_HOME="$(eval echo "~${SUDO_USER:-$USER}")"

echo "Installing runtime dependencies first..."
bash "$SCRIPT_DIR/DEPENDENCIES.sh"

echo "Installing dev/build dependencies..."
OS=$(grep -oP '(?<=^ID=).+' /etc/os-release 2>/dev/null | tr -d '"' || echo "unknown")

install_debian() {
apt-get install -y -qq \
    clang lld mold build-essential pkg-config libssl-dev libpq-dev cmake git \
    libglib2.0-dev libgtk-3-dev libwebkit2gtk-4.1-dev libjavascriptcoregtk-4.1-dev \
    libayatana-appindicator3-dev librsvg2-dev libsoup-3.0-dev
}

install_fedora() {
dnf install -y -q \
    clang lld mold gcc gcc-c++ make pkg-config openssl-devel postgresql-devel cmake git \
glib2-devel gobject-introspection-devel gtk3-devel webkit2gtk3-devel \
javascriptcoregtk-devel libappindicator-gtk3-devel librsvg2-devel libsoup3-devel
}

install_arch() {
pacman -Sy --noconfirm \
    clang lld mold gcc make pkg-config openssl libpq cmake git \
glib2 gtk3 webkit2gtk4 javascriptcoregtk libappindicator librsvg libsoup
}

case $OS in
ubuntu|debian|linuxmint|pop) install_debian ;;
fedora|rhel|centos|rocky|almalinux) install_fedora ;;
arch|manjaro) install_arch ;;
*) echo "Unsupported OS: $OS"; exit 1 ;;
esac

run_as_user() {
su - "${SUDO_USER:-$USER}" -c ". '${SUDO_USER_HOME}/.cargo/env' 2>/dev/null; export PATH=\"${SUDO_USER_HOME}/.cargo/bin:\$PATH\"; $*"
}

install_rust() {
if ! run_as_user "rustc --version" &>/dev/null; then
echo "Installing Rust via rustup for ${SUDO_USER:-$USER}..."
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
su - "${SUDO_USER:-$USER}" -c "sh -s -- -y --default-toolchain stable"
run_as_user "rustup default stable"
echo "Rust installed: $(run_as_user 'rustc --version') / $(run_as_user 'cargo --version')"
else
echo "Rust already installed: $(run_as_user 'rustc --version')"
fi
}

install_cargo_tools() {
if run_as_user "cargo --version" &>/dev/null; then
if ! run_as_user "sccache --version" &>/dev/null; then
echo "Installing sccache..."
run_as_user "cargo install sccache --locked"
else
echo "sccache already installed: $(run_as_user 'sccache --version 2>&1 | head -1')"
fi
fi
}

install_rust "$SUDO_USER_HOME"
install_cargo_tools "$SUDO_USER_HOME"

echo "Dev dependencies installed!"
