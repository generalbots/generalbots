#!/bin/bash
set -e
[ "$EUID" -ne 0 ] && { echo "Run as root (use sudo)"; exit 1; }
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
bash "$SCRIPT_DIR/DEPENDENCIES.sh"
OS=$(grep -oP '(?<=^ID=).+' /etc/os-release 2>/dev/null | tr -d '"' || echo "unknown")
install_debian() { apt-get install -y -qq clang lld build-essential pkg-config libssl-dev libpq-dev cmake git libglib2.0-dev libgtk-3-dev libwebkit2gtk-4.1-dev libjavascriptcoregtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev libsoup-3.0-dev curl wget file unzip jq protobuf-compiler libprotobuf-dev libsqlite3-dev libfontconfig1-dev libfreetype6-dev libexpat1-dev libonig-dev; }
install_fedora() { dnf install -y -q clang lld gcc gcc-c++ make pkg-config openssl-devel postgresql-devel cmake git glib2-devel gobject-introspection-devel gtk3-devel webkit2gtk3-devel javascriptcoregtk-devel libappindicator-gtk3-devel librsvg2-devel libsoup3-devel curl wget file unzip jq protobuf-compiler protobuf-devel sqlite-devel fontconfig-devel freetype-devel expat-devel oniguruma-devel; }
install_arch() { pacman -Sy --noconfirm clang lld gcc make pkg-config openssl libpq cmake git glib2 gtk3 webkit2gtk4 javascriptcoregtk libappindicator librsvg libsoup curl wget file unzip jq protobuf sqlite fontconfig freetype2 expat oniguruma; }
case $OS in
ubuntu|debian|linuxmint|pop) install_debian ;;
fedora|rhel|centos|rocky|almalinux) install_fedora ;;
arch|manjaro) install_arch ;;
*) echo "Unsupported OS: $OS"; exit 1 ;;
esac

install_rust() {
  if command -v cargo &> /dev/null; then
    echo "Rust/Cargo already installed: $(cargo --version)"
  else
    echo "Installing Rust via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
    . "$HOME/.cargo/env"
    echo "Rust installed: $(cargo --version), $(rustc --version)"
  fi
}

install_sccache() {
  if command -v sccache &> /dev/null; then
    echo "sccache already installed: $(sccache --version)"
  else
    echo "Installing sccache..."
    SCCACHE_VER="v0.8.1"
    ARCH=$(uname -m)
    curl -L "https://github.com/mozilla/sccache/releases/download/${SCCACHE_VER}/sccache-${SCCACHE_VER}-${ARCH}-unknown-linux-musl.tar.gz" -o /tmp/sccache.tar.gz
    tar -xzf /tmp/sccache.tar.gz -C /tmp
    cp "/tmp/sccache-${SCCACHE_VER}-${ARCH}-unknown-linux-musl/sccache" /usr/local/bin/
    chmod +x /usr/local/bin/sccache
    rm -rf /tmp/sccache*
    echo "sccache installed: $(sccache --version)"
  fi
}

install_mold() {
  if command -v mold &> /dev/null; then
    echo "mold already installed: $(mold --version)"
  else
    echo "Installing mold..."
    curl -L "https://github.com/rui314/mold/releases/download/v2.4.0/mold-2.4.0-x86_64-linux.tar.gz" -o /tmp/mold.tar.gz
    tar -xzf /tmp/mold.tar.gz -C /tmp
    cp "/tmp/mold-2.4.0-x86_64-linux/bin/mold" /usr/local/bin/
    rm -rf /tmp/mold-2.4.0* /tmp/mold.tar.gz
    ldconfig
    echo "mold installed: $(mold --version)"
  fi
}

install_cargo_tools() {
  echo "Installing cargo dev tools..."
  cargo install cargo-audit cargo-machete cargo-tree --quiet 2>/dev/null || true
  echo "cargo-audit, cargo-machete, cargo-tree installed"
}

setup_cargo_config() {
  CARGO_DIR="$SCRIPT_DIR/.cargo"
  mkdir -p "$CARGO_DIR"
  if [ ! -f "$CARGO_DIR/config.toml" ]; then
    cat > "$CARGO_DIR/config.toml" <<'CARGOCONF'
[build]
jobs = -1

[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=mold"]

[env]
RUSTC_WRAPPER = "sccache"
CARGOCONF
    echo "Created .cargo/config.toml with mold + sccache"
  else
    echo ".cargo/config.toml already exists, skipping"
  fi
}

install_rust
install_sccache
install_mold
install_cargo_tools
setup_cargo_config

echo ""
echo "✅ Dev environment ready:"
echo "   Rust:       $(rustc --version)"
echo "   Linker:     clang + lld + mold"
echo "   Cache:      sccache"
echo "   Audit:      cargo-audit, cargo-machete, cargo-tree"
echo "📦 .cargo/config.toml configured"
echo "⚡ Build: cargo build -p botserver --bin botserver"
