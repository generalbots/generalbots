#!/bin/bash
# Auto-create desktop1 container with VNC/GUI support (Issue #530)

set -e

CONTAINER="desktop1"
VNC_PORT="5901"
NOVNC_PORT="6080"
DOMAIN="${DOMAIN:-gb.solutions}"

echo "=== Creating $CONTAINER container ==="

# Create container
sudo incus launch ubuntu:24.04 "$CONTAINER" -c limits.cpu=2 -c limits.memory=4GiB

# Install desktop environment
sudo incus exec "$CONTAINER" -- apt update
sudo incus exec "$CONTAINER" -- apt install -y xfce4 xfce4-goodies

# Install VNC server
sudo incus exec "$CONTAINER" -- apt install -y tightvncserver

# Install noVNC
sudo incus exec "$CONTAINER" -- apt install -y novnc python3-websockify

# Configure VNC
sudo incus exec "$CONTAINER" -- bash -c "mkdir -p ~/.vnc && echo 'password' | vncpasswd -f > ~/.vnc/passwd && chmod 600 ~/.vnc/passwd"

# Setup VNC service
sudo incus exec "$CONTAINER" -- bash -c "cat > /etc/systemd/system/vncserver@.service << 'EOF'
[Unit]
Description=Start TightVNC server at startup
After=syslog.target network.target

[Service]
Type=forking
User=ubuntu
PAMName=login
PIDFile=/home/ubuntu/.vnc/%H:%i.pid
ExecStartPre=-/usr/bin/vncserver -kill :%i > /dev/null 2>&1
ExecStart=/usr/bin/vncserver -depth 24 -geometry 1280x720 :%i
ExecStop=/usr/bin/vncserver -kill :%i

[Install]
WantedBy=multi-user.target
EOF"

sudo incus exec "$CONTAINER" -- systemctl enable vncserver@1
sudo incus exec "$CONTAINER" -- systemctl start vncserver@1

# SSH access
sudo incus exec "$CONTAINER" -- apt install -y openssh-server

echo "=== $CONTAINER created successfully ==="
echo "VNC: vnc://$CONTAINER.$DOMAIN:$VNC_PORT"
echo "noVNC: https://$CONTAINER.$DOMAIN/vnc"
echo "SSH: ssh ubuntu@$CONTAINER.$DOMAIN"
