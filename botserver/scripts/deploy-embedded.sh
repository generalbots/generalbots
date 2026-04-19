#!/bin/bash
#
# BotServer Embedded Deployment Script
# For Orange Pi, Raspberry Pi, and other ARM/x86 SBCs
#
# Usage: ./deploy-embedded.sh [target-host] [options]
#
# Examples:
#   ./deploy-embedded.sh orangepi@192.168.1.100
#   ./deploy-embedded.sh pi@raspberrypi.local --with-ui
#   ./deploy-embedded.sh --local --with-ui
#

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
BOTUI_DIR="$(dirname "$PROJECT_DIR")/botui"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Default values
TARGET_HOST=""
WITH_UI=false
WITH_LLAMA=false
LOCAL_INSTALL=false
ARCH=""
SERVICE_NAME="botserver"
LLAMA_MODEL="tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf"
LLAMA_URL="https://huggingface.co/TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF/resolve/main"

print_banner() {
    echo -e "${BLUE}"
    echo "╔════════════════════════════════════════════════════════════╗"
    echo "║     BotServer Embedded Deployment                          ║"
    echo "║     Orange Pi / Raspberry Pi / ARM SBCs                    ║"
    echo "╚════════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
}

detect_arch() {
    local arch=$(uname -m)
    case $arch in
        aarch64|arm64)
            ARCH="aarch64-unknown-linux-gnu"
            echo -e "${GREEN}Detected: ARM64${NC}"
            ;;
        armv7l|armhf)
            ARCH="armv7-unknown-linux-gnueabihf"
            echo -e "${GREEN}Detected: ARMv7 (32-bit)${NC}"
            ;;
        x86_64)
            ARCH="x86_64-unknown-linux-gnu"
            echo -e "${GREEN}Detected: x86_64${NC}"
            ;;
        *)
            echo -e "${RED}Unknown architecture: $arch${NC}"
            exit 1
            ;;
    esac
}

install_rust_target() {
    echo -e "${YELLOW}Installing Rust target: $ARCH${NC}"
    rustup target add $ARCH 2>/dev/null || true
}

install_cross_compiler() {
    echo -e "${YELLOW}Installing cross-compilation tools...${NC}"
    
    case $ARCH in
        aarch64-unknown-linux-gnu)
            if ! command -v aarch64-linux-gnu-gcc &> /dev/null; then
                echo "Installing aarch64 cross-compiler..."
                sudo apt-get update
                sudo apt-get install -y gcc-aarch64-linux-gnu
            fi
            export CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc
            export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
            ;;
        armv7-unknown-linux-gnueabihf)
            if ! command -v arm-linux-gnueabihf-gcc &> /dev/null; then
                echo "Installing armv7 cross-compiler..."
                sudo apt-get update
                sudo apt-get install -y gcc-arm-linux-gnueabihf
            fi
            export CC_armv7_unknown_linux_gnueabihf=arm-linux-gnueabihf-gcc
            export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc
            ;;
    esac
}

build_botserver() {
    echo -e "${YELLOW}Building botserver for $ARCH...${NC}"
    cd "$PROJECT_DIR"
    
    # Build release
    cargo build --release --target $ARCH
    
    BINARY_PATH="$PROJECT_DIR/target/$ARCH/release/botserver"
    
    if [ -f "$BINARY_PATH" ]; then
        echo -e "${GREEN}Build successful: $BINARY_PATH${NC}"
        ls -lh "$BINARY_PATH"
    else
        echo -e "${RED}Build failed!${NC}"
        exit 1
    fi
}

build_local() {
    echo -e "${YELLOW}Building botserver locally...${NC}"
    cd "$PROJECT_DIR"
    cargo build --release
    BINARY_PATH="$PROJECT_DIR/target/release/botserver"
}

create_systemd_service() {
    cat > /tmp/botserver.service << 'EOF'
[Unit]
Description=BotServer - General Bots Server
After=network.target postgresql.service
Wants=network-online.target

[Service]
Type=simple
User=botserver
Group=botserver
WorkingDirectory=/opt/botserver
ExecStart=/opt/botserver/botserver
Restart=always
RestartSec=5
Environment=RUST_LOG=info
Environment=DATABASE_URL=postgres://localhost/botserver

# Security
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/opt/botserver/data

[Install]
WantedBy=multi-user.target
EOF
}

create_kiosk_service() {
    cat > /tmp/botui-kiosk.service << 'EOF'
[Unit]
Description=BotUI Kiosk Mode
After=graphical.target botserver.service
Wants=botserver.service

[Service]
Type=simple
User=pi
Environment=DISPLAY=:0
ExecStartPre=/bin/sleep 5
ExecStart=/usr/bin/chromium-browser --kiosk --noerrdialogs --disable-infobars --disable-session-crashed-bubble --app=http://localhost:9000/embedded/
Restart=always
RestartSec=10

[Install]
WantedBy=graphical.target
EOF
}

deploy_remote() {
    local host=$1
    
    echo -e "${YELLOW}Deploying to $host...${NC}"
    
    # Create remote directory
    ssh $host "sudo mkdir -p /opt/botserver/data && sudo chown -R \$(whoami):\$(whoami) /opt/botserver"
    
    # Copy binary
    echo "Copying botserver binary..."
    scp "$BINARY_PATH" "$host:/opt/botserver/botserver"
    ssh $host "chmod +x /opt/botserver/botserver"
    
    # Copy config
    if [ -f "$PROJECT_DIR/.env.example" ]; then
        scp "$PROJECT_DIR/.env.example" "$host:/opt/botserver/.env"
    fi
    
    # Copy systemd service
    create_systemd_service
    scp /tmp/botserver.service "$host:/tmp/"
    ssh $host "sudo mv /tmp/botserver.service /etc/systemd/system/"
    
    # Create user and setup
    ssh $host "sudo useradd -r -s /bin/false botserver 2>/dev/null || true"
    ssh $host "sudo chown -R botserver:botserver /opt/botserver"
    
    # Enable and start service
    ssh $host "sudo systemctl daemon-reload"
    ssh $host "sudo systemctl enable botserver"
    ssh $host "sudo systemctl start botserver"
    
    echo -e "${GREEN}BotServer deployed and running on $host${NC}"
    
    # Deploy UI if requested
    if [ "$WITH_UI" = true ]; then
        deploy_ui_remote $host
    fi
}

deploy_ui_remote() {
    local host=$1
    
    echo -e "${YELLOW}Deploying embedded UI to $host...${NC}"
    
    # Copy embedded UI files
    if [ -d "$BOTUI_DIR/ui/embedded" ]; then
        ssh $host "mkdir -p /opt/botserver/ui/embedded"
        scp -r "$BOTUI_DIR/ui/embedded/"* "$host:/opt/botserver/ui/embedded/"
    fi
    
    # Setup kiosk mode
    create_kiosk_service
    scp /tmp/botui-kiosk.service "$host:/tmp/"
    ssh $host "sudo mv /tmp/botui-kiosk.service /etc/systemd/system/"
    ssh $host "sudo systemctl daemon-reload"
    ssh $host "sudo systemctl enable botui-kiosk"
    
    echo -e "${GREEN}Kiosk mode configured. Reboot to start.${NC}"
}

deploy_local() {
    echo -e "${YELLOW}Installing locally...${NC}"
    
    # Build
    build_local
    
    # Install
    sudo mkdir -p /opt/botserver/data
    sudo cp "$BINARY_PATH" /opt/botserver/
    sudo chmod +x /opt/botserver/botserver
    
    if [ -f "$PROJECT_DIR/.env.example" ]; then
        sudo cp "$PROJECT_DIR/.env.example" /opt/botserver/.env
    fi
    
    # Create user
    sudo useradd -r -s /bin/false botserver 2>/dev/null || true
    sudo chown -R botserver:botserver /opt/botserver
    
    # Setup systemd
    create_systemd_service
    sudo mv /tmp/botserver.service /etc/systemd/system/
    sudo systemctl daemon-reload
    sudo systemctl enable botserver
    sudo systemctl start botserver
    
    echo -e "${GREEN}BotServer installed and running locally${NC}"
    
    if [ "$WITH_UI" = true ]; then
        # Copy UI files
        sudo mkdir -p /opt/botserver/ui/embedded
        sudo cp -r "$BOTUI_DIR/ui/embedded/"* /opt/botserver/ui/embedded/
        
        # Setup kiosk
        create_kiosk_service
        sudo mv /tmp/botui-kiosk.service /etc/systemd/system/
        sudo systemctl daemon-reload
        sudo systemctl enable botui-kiosk
        
        echo -e "${GREEN}Kiosk mode configured. Reboot to start.${NC}"
    fi
}

install_llama_cpp() {
    local host=$1
    local is_local=$2
    
    echo -e "${YELLOW}Installing llama.cpp...${NC}"
    
    local commands='
        # Install dependencies
        sudo apt-get update
        sudo apt-get install -y build-essential cmake git
        
        # Clone and build llama.cpp
        cd /opt
        if [ ! -d "llama.cpp" ]; then
            sudo git clone https://github.com/ggerganov/llama.cpp.git
            sudo chown -R $(whoami):$(whoami) llama.cpp
        fi
        cd llama.cpp
        
        # Build with optimizations for ARM
        mkdir -p build && cd build
        cmake .. -DLLAMA_NATIVE=ON -DCMAKE_BUILD_TYPE=Release
        make -j$(nproc)
        
        # Create models directory
        mkdir -p /opt/llama.cpp/models
    '
    
    if [ "$is_local" = true ]; then
        eval "$commands"
    else
        ssh $host "$commands"
    fi
}

download_model() {
    local host=$1
    local is_local=$2
    
    echo -e "${YELLOW}Downloading model: $LLAMA_MODEL...${NC}"
    
    local commands="
        cd /opt/llama.cpp/models
        if [ ! -f '$LLAMA_MODEL' ]; then
            wget -c '$LLAMA_URL/$LLAMA_MODEL'
        fi
        ls -lh /opt/llama.cpp/models/
    "
    
    if [ "$is_local" = true ]; then
        eval "$commands"
    else
        ssh $host "$commands"
    fi
}

create_llama_service() {
    cat > /tmp/llama-server.service << 'EOF'
[Unit]
Description=llama.cpp Server - Local LLM Inference
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/opt/llama.cpp
ExecStart=/opt/llama.cpp/build/bin/llama-server \
    -m /opt/llama.cpp/models/tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf \
    --host 0.0.0.0 \
    --port 8080 \
    -c 2048 \
    -ngl 0 \
    --threads 4
Restart=always
RestartSec=5
Environment=LLAMA_LOG_LEVEL=info

[Install]
WantedBy=multi-user.target
EOF
}

setup_llama_service() {
    local host=$1
    local is_local=$2
    
    echo -e "${YELLOW}Setting up llama.cpp systemd service...${NC}"
    
    create_llama_service
    
    if [ "$is_local" = true ]; then
        sudo mv /tmp/llama-server.service /etc/systemd/system/
        sudo systemctl daemon-reload
        sudo systemctl enable llama-server
        sudo systemctl start llama-server
    else
        scp /tmp/llama-server.service "$host:/tmp/"
        ssh $host "sudo mv /tmp/llama-server.service /etc/systemd/system/"
        ssh $host "sudo systemctl daemon-reload"
        ssh $host "sudo systemctl enable llama-server"
        ssh $host "sudo systemctl start llama-server"
    fi
    
    echo -e "${GREEN}llama.cpp server configured on port 8080${NC}"
}

deploy_llama() {
    local host=$1
    local is_local=${2:-false}
    
    install_llama_cpp "$host" "$is_local"
    download_model "$host" "$is_local"
    setup_llama_service "$host" "$is_local"
}

show_help() {
    echo "Usage: $0 [target-host] [options]"
    echo ""
    echo "Options:"
    echo "  --local       Install on this machine"
    echo "  --with-ui     Also deploy embedded UI with kiosk mode"
    echo "  --with-llama  Install llama.cpp for local LLM inference"
    echo "  --model NAME  Specify GGUF model (default: TinyLlama 1.1B Q4)"
    echo "  --arch ARCH   Force target architecture"
    echo "  -h, --help    Show this help"
    echo ""
    echo "Examples:"
    echo "  $0 orangepi@192.168.1.100"
    echo "  $0 pi@raspberrypi.local --with-ui"
    echo "  $0 --local --with-ui"
    echo ""
    echo "Supported boards:"
    echo "  - Raspberry Pi (Zero, 3, 4, 5)"
    echo "  - Orange Pi (Zero, One, PC, etc)"
    echo "  - Banana Pi"
    echo "  - Rock Pi"
    echo "  - ODROID"
    echo "  - Any ARM64/ARMv7/x86_64 Linux SBC"
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --local)
            LOCAL_INSTALL=true
            shift
            ;;
        --with-ui)
            WITH_UI=true
            shift
            ;;
        --with-llama)
            WITH_LLAMA=true
            shift
            ;;
        --model)
            LLAMA_MODEL="$2"
            shift 2
            ;;
        --arch)
            ARCH="$2"
            shift 2
            ;;
        -h|--help)
            show_help
            exit 0
            ;;
        *)
            if [ -z "$TARGET_HOST" ]; then
                TARGET_HOST="$1"
            fi
            shift
            ;;
    esac
done

# Main
print_banner

if [ "$LOCAL_INSTALL" = true ]; then
    detect_arch
    deploy_local
    if [ "$WITH_LLAMA" = true ]; then
        deploy_llama "" true
    fi
elif [ -n "$TARGET_HOST" ]; then
    # Get remote arch
    echo "Detecting remote architecture..."
    REMOTE_ARCH=$(ssh $TARGET_HOST "uname -m")
    case $REMOTE_ARCH in
        aarch64|arm64)
            ARCH="aarch64-unknown-linux-gnu"
            ;;
        armv7l|armhf)
            ARCH="armv7-unknown-linux-gnueabihf"
            ;;
        x86_64)
            ARCH="x86_64-unknown-linux-gnu"
            ;;
    esac
    echo -e "${GREEN}Remote arch: $ARCH${NC}"
    
    install_rust_target
    install_cross_compiler
    build_botserver
    deploy_remote $TARGET_HOST
    if [ "$WITH_LLAMA" = true ]; then
        deploy_llama $TARGET_HOST false
    fi
else
    show_help
    exit 1
fi

echo ""
echo -e "${GREEN}Deployment complete!${NC}"
echo ""
echo "Check status:"
echo "  ssh $TARGET_HOST 'sudo systemctl status botserver'"
echo ""
echo "View logs:"
echo "  ssh $TARGET_HOST 'sudo journalctl -u botserver -f'"
echo ""
if [ "$WITH_UI" = true ]; then
    echo "Access UI at: http://$TARGET_HOST:9000/embedded/"
fi
if [ "$WITH_LLAMA" = true ]; then
    echo ""
    echo "llama.cpp server running at: http://$TARGET_HOST:9000"
    echo "Test: curl http://$TARGET_HOST:9000/v1/models"
fi
