# Installation 🟡 BETA

botserver installs itself automatically through the bootstrap process. Just run the binary.

## Runtime Dependencies

Before running the botserver binary, you must install required system libraries on the **host system**.

### Verify (recommended)

`--check` does not require root and does not install anything. It prints every
missing command, shared library, group and package, then exits non-zero:

```bash
./PROD-DEPENDENCIES.sh --check
```

### Install

```bash
sudo ./PROD-DEPENDENCIES.sh --install
```

The install path is idempotent and resolves release-specific package renames
(Debian/Ubuntu `libssl3t64`, `libgtk-3-0t64`, `libzbar0t64`, versioned
`libclang1-NN`). `DEPENDENCIES.sh` holds the package matrix;
`PROD-DEPENDENCIES.sh` is the supported entry point, and
`DEV-DEPENDENCIES.sh` adds the build toolchain on top of the same runtime set.

### Runtime matrix

| Category | What is required | Required by |
|----------|------------------|-------------|
| PostgreSQL client | libpq (`libpq.so.5`) | database access |
| TLS | OpenSSL (`libssl.so.3`) | HTTPS, JWT |
| Compression | liblzma, zlib | archives, HTTP |
| Clang runtime | libclang | native service components |
| PDF | poppler (`pdftotext`, `libpoppler.so`) | document extraction |
| Media | zbar (`libzbar.so.0`) | BotModels barcode/QR decoding |
| Containers | `incus` and the `incus-admin` group | agent VMs, terminals, browser actions |
| Desktop shell | GTK3, WebKitGTK 4.1, libsoup3, librsvg, appindicator | desktop/tauri bundle |
| Fonts | fontconfig, DejaVu and Liberation fonts | UI rendering |
| Base tools | `ca-certificates`, `curl`, `wget` | bootstrap downloads |

The matrix is implemented for Debian/Ubuntu (apt), Fedora/RHEL (dnf), Arch
(pacman) and Alpine (apk). Package names differ by release; `--check` prints
the exact names for the host instead of duplicating a list here.

### Windows and macOS

The Linux matrix above does not apply to the desktop bundles:

- **Windows:** `.\PROD-DEPENDENCIES.ps1 -Check` reports WebView2, the VC++
  2015-2022 x64 runtime, `libpq.dll` and the WSL2/Incus agent backend, then
  exits non-zero when the desktop shell or agent mode cannot run. `-Install`
  prints the remediation commands and does not download or install anything.
- **macOS:** `./PROD-DEPENDENCIES.sh --check` reports that no desktop bundle is
  built for macOS (`desktop/macos/README.md`).

See [First-Run Wizard](../07-user-interface/first-run-wizard.md) for the exit
codes and the per-platform agent backend details.

### Incus

The Agent Workspace runs each session in an Incus container. The Incus socket
is `root:incus-admin 0660` and the daemon checks the peer credentials, so the
account that runs `botserver` must carry the `incus-admin` group in the
**running** process — adding it to `/etc/group` alone is not enough:

```bash
sudo usermod -aG incus-admin botserver   # or the account that runs botserver
sudo systemctl restart botserver         # a fresh login session also works
```

`DEPENDENCIES.sh` installs Incus where the distribution packages it, adds the
service user to `incus-admin`, starts the daemon, runs `incus admin init
--minimal` when no storage pool exists, and prints the re-login/restart
reminder.

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
| LLM Server | 8081 | internal |
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