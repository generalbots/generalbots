# Installation

botserver installs itself automatically through the bootstrap process. Just run the binary.

## Runtime Dependencies

Before running the botserver binary, you must install required system libraries on the **host system**.

### Quick Install (Recommended)

Download and run the dependency installer:

```bash
curl -fsSL https://raw.githubusercontent.com/GeneralBots/botserver/main/scripts/install-dependencies.sh | sudo bash
```

Or if you have the script locally:

```bash
sudo ./scripts/install-dependencies.sh
```

### Manual Install (Debian/Ubuntu)

```bash
sudo apt update
sudo apt install -y \
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

# For container support (LXC)
sudo snap install lxd
sudo lxd init --auto
```

### Manual Install (Fedora/RHEL)

```bash
sudo dnf install -y \
    libpq \
    openssl-libs \
    xz-libs \
    zlib \
    ca-certificates \
    curl \
    wget \
    lxc
```

> ⚠️ **Common Error**: If you see `error while loading shared libraries: libpq.so.5`, install `libpq5` (Debian/Ubuntu) or `libpq` (Fedora/RHEL).

## System Requirements

| Resource | Minimum | Production |
|----------|---------|------------|
| **OS** | Linux, macOS, Windows | Linux (Ubuntu/Debian) |
| **RAM** | 4GB | 16GB+ |
| **Disk** | 10GB | 100GB SSD |
| **CPU** | 1 core | 2+ cores |
| **GPU** | None | RTX 3060+ (12GB VRAM) for local LLM |

## Quick Start

```bash
./botserver
```

The bootstrap process automatically:
1. Detects your system (OS/architecture)
2. Creates `botserver-stack/` directory structure
3. Downloads PostgreSQL, Drive, Cache, LLM server
4. Initializes database and storage
5. Deploys default bot
6. Starts all services

**First run takes 2-5 minutes.**

## Using Existing Services

If you have existing infrastructure, configure it in your bot's `config.csv`:

```csv
name,value
database-url,postgres://myuser:mypass@myhost:5432/mydb
drive-server,http://my-drive:9000
drive-accesskey,my-access-key
drive-secret,my-secret-key
```

## Default Ports

| Service | Port | Config Key |
|---------|------|------------|
| UI Server | 8080 | `server-port` |
| PostgreSQL | 5432 | `DATABASE_URL` |
| Drive API | 9000 | `DRIVE_SERVER` |
| Drive Console | 9001 | - |
| LLM Server | 8081 | `llm-server-port` |
| Embedding | 8082 | `embedding-url` |
| Cache | 6379 | Internal |

## Verify Installation

```bash
# Check services
./botserver status

# Test database
psql $DATABASE_URL -c "SELECT version();"

# Test LLM
curl http://localhost:8081/v1/models

# Open UI
open http://localhost:9000
```

## Bot Deployment

Bots deploy to object storage (not local filesystem):

```bash
mybot.gbai → creates 'mybot' bucket in drive
```

The `work/` folder is for internal use only.

### S3 Sync for Development

Use S3-compatible tools for local editing:
- **Cyberduck** (GUI)
- **rclone** (CLI)
- **WinSCP** (Windows)

```bash
# rclone sync example
rclone sync ./mybot.gbai drive:mybot --watch
```

Edits sync automatically - changes reload without restart.

## Memory Optimization

For limited RAM systems:

```csv
name,value
llm-server-ctx-size,2048
llm-server-parallel,2
```

Use quantized models (Q3_K_M, Q4_K_M) for smaller memory footprint.

## GPU Setup

For GPU acceleration:

```csv
name,value
llm-server-gpu-layers,35
```

Requires CUDA installed and 12GB+ VRAM.

## Deployment Options

| Method | Use Case | Guide |
|--------|----------|-------|
| **Local** | Development, single instance | This page |
| **Docker** | Production, microservices | [Docker Deployment](../02-architecture-packages/docker-deployment.md) |
| **LXC** | Isolated components, Linux | [Container Deployment](../02-architecture-packages/containers.md) |
| **Brother Mode** | Container managing host containers | See below |

### Container-on-Host (Brother Mode)

You can run `botserver` inside a container (Docker/LXC) while letting it manage other containers directly on the host system. This is useful for CI/CD pipelines or managing "host" deployment from a restricted environment.

**Requirements:**
- Mount host's LXD socket to container
- Run container as privileged (if accessing host devices)

**Docker Run Example:**
```bash
docker run -d \
  --name botserver \
  --network host \
  --privileged \
  -v /var/snap/lxd/common/lxd/unix.socket:/var/snap/lxd/common/lxd/unix.socket \
  -e VAULT_ADDR="https://127.0.0.1:8200" \
  -e VAULT_TOKEN="<your-token>" \
  botserver:latest
```

> **Note**: For non-snap LXD, use `-v /var/lib/lxd/unix.socket:/var/lib/lxd/unix.socket` instead.

The installer detects if it is running in a container but needs to manage the host (brother mode) and will configure the host's LXD/LXC environment safely.

> ⚠️ **IMPORTANT**: Container create commands (`botserver install ... --container`) must be run from the **host system**, not inside a container.

### Example: Create Vault and VectorDB

```bash
# Run on HOST system
botserver install vault --container --tenant mycompany
botserver install vector_db --container --tenant mycompany

# Verify containers
lxc list | grep mycompany
```

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Database connection | Check `DATABASE_URL`, verify PostgreSQL running |
| Port conflict | Change port in config or stop conflicting service |
| Memory issues | Reduce `llm-server-ctx-size`, use quantized model |
| GPU not detected | Verify CUDA, set `llm-server-gpu-layers,0` for CPU |

## Next Steps

- [Quick Start Guide](./quick-start.md) - Create your first bot
- [First Conversation](./first-conversation.md) - Test your bot
- [Configuration Reference](../10-configuration-deployment/README.md) - All settings