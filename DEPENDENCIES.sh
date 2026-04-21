        #!/bin/bash
        #
        # DEPENDENCIES.sh - Runtime Dependencies for General Bots
        # 
        # This script installs all system packages required to RUN botserver binary.
        # These are the minimal dependencies needed for production deployment.
        #
        # Usage: sudo ./DEPENDENCIES.sh
        #

        set -e

        # Colors
        RED='\033[0;31m'
        GREEN='\033[0;32m'
        YELLOW='\033[1;33m'
        NC='\033[0m'

        echo -e "${GREEN}========================================${NC}"
        echo -e "${GREEN}  General Bots Runtime Dependencies${NC}"
        echo -e "${GREEN}========================================${NC}"

        # Check root
        if [ "$EUID" -ne 0 ]; then
            echo -e "${RED}Error: Run as root (use sudo)${NC}"
            exit 1
        fi

        # Detect OS
        if [ -f /etc/os-release ]; then
            . /etc/os-release
            OS=$ID
        else
            echo -e "${RED}Error: Cannot detect OS${NC}"
            exit 1
        fi

        echo -e "${YELLOW}OS: $OS${NC}"

        install_debian_ubuntu() {
                
                apt-get install -y \
                    libpq5 \
                    libssl3 \
                    liblzma5 \
                    zlib1g \
                    ca-certificates \
                    curl \
                    wget \
                    libclang1 \
                    pkg-config \
                    snapd


        }

        install_fedora_rhel() {
            dnf install -y \
                libpq \
                openssl-libs \
                xz-libs \
                zlib \
                ca-certificates \
                curl \
                wget \
                abseil-cpp \
                clang-libs \
                pkgconf-pkg-config \
                lxc \
                lxc-templates
        }

        install_arch() {
            pacman -Sy --noconfirm \
                postgresql-libs \
                openssl \
                xz \
                zlib \
                ca-certificates \
                curl \
                wget \
                abseil-cpp \
                clang \
                pkgconf \
                lxc
        }

        install_alpine() {
            apk add --no-cache \
                libpq \
                openssl \
                xz-libs \
                zlib \
                ca-certificates \
                curl \
                wget \
                abseil-cpp \
                clang \
                pkgconf \
                lxc
        }

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
                echo -e "${RED}Unsupported OS: $OS${NC}"
                echo "Required libraries:"
                echo "  - libpq (PostgreSQL client)"
                echo "  - libssl (OpenSSL)"
                echo "  - liblzma (XZ compression)"
                echo "  - zlib (compression)"
                echo "  - abseil-cpp (Google Abseil)"
                echo "  - clang (LLVM runtime)"
                echo "  - LXC (containers)"
                exit 1
                ;;
        esac

        echo -e "${GREEN}Runtime dependencies installed!${NC}"
        echo ""
        echo "You can now run:"
        echo "  ./botserver"
