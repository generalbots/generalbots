#!/bin/bash
# setup-stage-gbo.sh
# Run this on the Incus host (administrator@63.141.255.9)
#
# This script sets up a STAGE-GBO project that completely isolates the stage
# environment from PROD, clones the essential containers, changes their IPs
# to 10.0.3.x, restricts disk size to 10GB max, and wipes data where requested.

set -e

PROJECT="STAGE-GBO"
NETWORK="stagebr0"

echo "=== 1. Creating Isolated Project: $PROJECT ==="
# features.networks and features.profiles isolate the network and profiles from default
sudo incus project create $PROJECT \
  -c features.networks=true \
  -c features.profiles=true \
  -c features.storage.volumes=true || echo "Project might already exist."

sudo incus project switch $PROJECT

echo "=== 2. Creating Stage Network (10.0.3.x) ==="
sudo incus network create $NETWORK ipv4.address=10.0.3.1/24 ipv4.nat=true ipv6.address=none || echo "Network might already exist."

echo "=== 3. Configuring Stage Default Profile (10GB Limit) ==="
# Configure the default profile for the STAGE-GBO project to use the new network
sudo incus profile device add default eth0 nic network=$NETWORK name=eth0 || \
sudo incus profile device set default eth0 network $NETWORK || true

# Limit root disk size to 10GB
sudo incus profile device add default root disk path=/ pool=default size=10GB || \
sudo incus profile device set default root size=10GB || true

# Containers to clone
CONTAINERS=("system" "tables" "vault" "cache" "drive" "llm")

# Target IPs for stage environment
declare -A IPS=(
  ["system"]="10.0.3.10"
  ["tables"]="10.0.3.11"
  ["vault"]="10.0.3.12"
  ["cache"]="10.0.3.13"
  ["drive"]="10.0.3.14"
  ["llm"]="10.0.3.15"
)

echo "=== 4. Cloning Containers from PROD (default project) ==="
sudo incus project switch PROD-GBO1

for c in "${CONTAINERS[@]}"; do
  echo "Copying $c to $PROJECT..."
  sudo incus copy PROD-GBO1:$c $PROJECT:$c || echo "  Warning: Failed to copy $c. It might already exist."
done

echo "=== 5. Reconfiguring and Cleaning Data in STAGE-GBO ==="
sudo incus project switch $PROJECT

for c in "${CONTAINERS[@]}"; do
  IP="${IPS[$c]}"
  echo "--> Starting $c for reconfiguration..."
  sudo incus start $c || true
  sleep 3 # Wait for container to initialize
  
  echo "    Setting static IP $IP in /etc/network/interfaces..."
  sudo incus exec $c -- bash -c "cat > /etc/network/interfaces << 'EOF'
auto lo
iface lo inet loopback

auto eth0
iface eth0 inet static
address $IP
netmask 255.255.255.0
gateway 10.0.3.1
dns-nameservers 8.8.8.8 8.8.4.4
EOF"

  echo "    Cleaning logs..."
  sudo incus exec $c -- bash -c 'rm -rf /opt/gbo/logs/* || true'

  # Apply specific data wipe rules
  if [ "$c" == "drive" ]; then
    echo "    Wiping MinIO data (starting from scratch)..."
    sudo incus exec $c -- bash -c 'rm -rf /opt/gbo/data/minio/* || true'
  elif [ "$c" == "tables" ]; then
    echo "    Keeping tables data (database botserver intact as requested)."
  elif [ "$c" == "cache" ]; then
    echo "    Wiping Valkey cache..."
    sudo incus exec $c -- bash -c 'rm -rf /opt/gbo/data/valkey/*.rdb /opt/gbo/data/valkey/*.aof || true'
  elif [ "$c" == "system" ]; then
    echo "    Wiping work directory and compiled ASTs..."
    sudo incus exec $c -- bash -c 'rm -rf /opt/gbo/work/* || true'
  fi

  echo "    Restarting $c to apply new IP..."
  sudo incus restart $c || true
done

echo "=== STAGE-GBO Setup Complete ==="
echo "You are currently in the default project."
sudo incus project switch PROD-GBO1
