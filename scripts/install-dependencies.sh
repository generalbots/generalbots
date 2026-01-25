#!/bin/bash
#
# install-dependencies.sh
# Installs all runtime dependencies required to run the botserver binary
#
# Usage: sudo ./install-dependencies.sh
#
# This script must be run on the HOST system (not inside a container)
# before running botserver for the first time.
#

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}  botserver Dependency Installer${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo -e "${RED}Error: This script must be run as root (use sudo)${NC}"
    exit 1
fi

# Detect OS
if [ -f /etc/os-release ]; then
    . /etc/os-release
    OS=$ID
    VERSION=$VERSION_ID
else
    echo -e "${RED}Error: Cannot detect operating system${NC}"
    exit 1
fi

echo -e "${YELLOW}Detected OS: $OS $VERSION${NC}"
echo ""

install_debian_ubuntu() {
    echo -e "${GREEN}Installing dependencies for Debian/Ubuntu...${NC}"

    apt-get update

    # Runtime libraries for botserver binary
    apt-get install -y \
        libpq5 \
        libssl3 \
        liblzma5 \
        zlib1g \
        ca-certificates \
        curl \
        wget \
        libabseil-dev \
        libclang-dev \
        pkg-config

    # LXC/LXD for container management (optional but recommended)
    echo ""
    echo -e "${YELLOW}Installing LXD for container support...${NC}"
    apt-get install -y snapd || true
    snap install lxd || apt-get install -y lxd || true

    # Initialize LXD if not already done
    if command -v lxd &> /dev/null; then
        if ! lxc list &> /dev/null 2>&1; then
            echo -e "${YELLOW}Initializing LXD...${NC}"
            lxd init --auto || true
        fi
    fi

    echo -e "${GREEN}Debian/Ubuntu dependencies installed successfully!${NC}"
}

install_fedora_rhel() {
    echo -e "${GREEN}Installing dependencies for Fedora/RHEL...${NC}"

    dnf install -y \
        libpq \
        openssl-libs \
        xz-libs \
        zlib \
        ca-certificates \
        curl \
        wget

    # LXC for container management
    dnf install -y lxc lxc-templates || true

    echo -e "${GREEN}Fedora/RHEL dependencies installed successfully!${NC}"
}

install_arch() {
    echo -e "${GREEN}Installing dependencies for Arch Linux...${NC}"

    pacman -Sy --noconfirm \
        postgresql-libs \
        openssl \
        xz \
        zlib \
        ca-certificates \
        curl \
        wget \
        lxc

    echo -e "${GREEN}Arch Linux dependencies installed successfully!${NC}"
}

install_alpine() {
    echo -e "${GREEN}Installing dependencies for Alpine Linux...${NC}"

    apk add --no-cache \
        libpq \
        openssl \
        xz-libs \
        zlib \
        ca-certificates \
        curl \
        wget \
        lxc

    echo -e "${GREEN}Alpine Linux dependencies installed successfully!${NC}"
}

# Install based on detected OS
case $OS in
    ubuntu|debian|linuxmint|pop)
        install_debian_ubuntu
        ;;
    fedora|rhel|centos|rocky|almalinux)
        install_fedora_rhel
        ;;
    arch|manjaro)
        install_arch
        ;;
    alpine)
        install_alpine
        ;;
    *)
        echo -e "${RED}Unsupported operating system: $OS${NC}"
        echo ""
        echo "Please manually install the following libraries:"
        echo "  - libpq (PostgreSQL client library)"
        echo "  - libssl (OpenSSL)"
        echo "  - liblzma (XZ compression)"
        echo "  - zlib (compression)"
        echo "  - LXC/LXD (for container support)"
        exit 1
        ;;
esac

echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}  Dependencies installed successfully!${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo "You can now run botserver:"
echo ""
echo "  ./botserver"
echo ""
echo "Or install components in containers:"
echo ""
echo "  ./botserver install vault --container --tenant mycompany"
echo "  ./botserver install vector_db --container --tenant mycompany"
echo ""
echo -e "${YELLOW}Note: Container commands must be run from the HOST system.${NC}"
echo ""
