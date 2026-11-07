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

> ⚠️ **Common Error**: If you see `error while loading shared libraries: libpq.so.5`, run `./PROD-DEPENDENCIES.sh --check` — it names the exact missing library and the package for this host release. On a minimal server image the second failure is usually `libsystemd.so.0` from the provisioned Valkey.

## System Requirements

| Resource | Minimum | Production |
|----------|---------|------------|
| **OS** | Linux, macOS, Windows | Linux (Ubuntu/Debian) |
| **RAM** | 4GB | 16GB+ |
| **Disk** | 10GB | 100GB SSD |
| **CPU** | 1 core | 2+ cores |
| **GPU** | None | RTX 3060+ (12GB VRAM) for local LLM |

## Distribution Packages

Each release publishes the same desktop edition (Chat + Drive) in four shapes,
built from the same binaries by `.github/workflows/desktop-binaries.yml`:

| Package | Install |
|---------|---------|
| `generalbots-linux-x86_64.deb` | `sudo apt install ./generalbots-linux-x86_64.deb` |
| `generalbots-linux-x86_64.rpm` | `sudo dnf install ./generalbots-linux-x86_64.rpm` |
| `generalbots-linux-x86_64.AppImage` | `chmod +x generalbots-linux-x86_64.AppImage && ./generalbots-linux-x86_64.AppImage` |
| `generalbots-x64.msi` / `generalbots-x64-setup.exe` | Windows: double-click |

The `.deb` declares dependencies on `libssl3` and `libwebkit2gtk-4.1-0`; the
`.rpm` declares `openssl-libs`, `webkit2gtk4.1` and `libpq` (see
`botapp/tauri.desktop-local.conf.json`). The AppImage still uses the host
WebKitGTK — it does not bundle it — and needs FUSE 2, or can be started with
`--appimage-extract-and-run`.

The single self-extracting `generalbots` executable is also published: one file,
no package manager, no unpack step.

## Quick Start

```bash
./botserver
```

The bootstrap process automatically:
1. Detects your system (OS/architecture)
2. Creates the `botserver-stack/` directory structure
3. Downloads the components pinned in `botserver/3rdparty.toml` — PostgreSQL,
   Valkey, MinIO and `mc`, Qdrant, Vault, Zitadel, Caddy, Stalwart, llama.cpp
   (the last one is skipped when the wizard is given a hosted provider)
4. Initializes database, object storage and secrets in Vault
5. Deploys the default bot
6. Starts the services

The desktop edition runs the same sequence headlessly as
`./start.sh --provision`, printing one `GBWIZARD:{...}` line per component.

**First run takes 2-5 minutes.**

## Using Existing Services

The platform components are not configured in `config.csv`. Their endpoints and
credentials live in Vault, one path per service, seeded on first boot:

| Path | Fields |
|------|--------|
| `secret/gbo/tables` | `host`, `port`, `database`, `username`, `password`, `url` |
| `secret/gbo/drive` | `host`, `port`, `accesskey`, `secret`, `bucket`, `backend` |
| `secret/gbo/cache` | `host`, `port`, `password`, `url` |
| `secret/gbo/llm` | `url`, `model`, `openai_key` |

Point those at your own instances with `vault kv put` before the first start.
Per-bot settings (`server-port`, `llm-model`, …) do go in the bot's
`config.csv` — see [config.csv Format](../10-configuration-deployment/config-csv.md).

## Default Ports

| Service | Port |
|---------|------|
| botui (suite) | 3000 |
| botui (cloud) | 4000 |
| botui (login) | 5000 |
| botserver (API + WebSocket) | 8080 |
| PostgreSQL | 5432 |
| Valkey | 6379 |
| Drive API | 9100 |
| Drive console | 9101 |
| llama.cpp (chat) | 8081 |
| llama.cpp (embeddings) | 8082 |
| BotModels | 8085 |
| Vault | 8200 |
| Qdrant | 6333 |
| Zitadel | 8300 |

## Verify Installation

```bash
# Component status (requires the component name — no bare `status`)
./botserver list
./botserver status tables

# API health — the endpoint that answers without a token
curl -s http://localhost:8080/health

# LLM (only when the local runtime was installed)
curl -s http://localhost:8081/v1/models
```

The component CLI is `botserver list | status <component> | start | stop |
restart | install <component> | remove <component>` (and `rotate-secret`,
`vault`, `security <fix|status>`). Component logs land in
`botserver-stack/logs/`.

## Bot Deployment

Bots live in object storage, not on the local filesystem. The bucket is the
tenant (`{org}.gborg`); a workspace prefix (`{workspace}.gbai/`) holds one or
more bots as sibling `{bot}.gbot`, `{bot}.gbdialog`, `{bot}.gbkb` and
`{bot}.gbdrive` directories:

```
cristo.gborg/
└── cristo.gbai/
    ├── cristo.gbot/          # channel prompts (PROMPT-<CHANNEL>.md)
    ├── cristo.gbdialog/      # start.bas, tables.bas, {tool}.bas
    ├── cristo.gbkb/          # documents indexed for retrieval
    └── cristo.gbdrive/       # generated files and media
```

`drive_monitor` discovers buckets and workspace prefixes at runtime and creates
the bot rows, so uploading a tree is enough to register a bot. The `work/`
folder is an internal materialization target, never a source.

For bots backed by an ALM repository, `.gbdialog/` and `.gbot/` come from that
repository (pulled every 15 s) and a copy written straight into drive is
archived on the next pass. `.gbkb/` and `.gbdrive/` stay drive-owned.

### S3 Sync for Development

Use any S3-compatible client against the drive endpoint:
- **Cyberduck** (GUI)
- **rclone** (CLI)
- **`mc`** (bundled by the provisioner at `botserver-stack/bin/mc`)

```bash
# rclone sync example (bucket = {org}.gborg, prefix = workspace)
rclone sync ./cristo.gbai drive:cristo.gborg/cristo.gbai --watch
```

`.gbkb`/`.gbdrive` edits are picked up by `drive_monitor` within seconds;
`.gbdialog`/`.gbot` edits belong in the bot's repository.

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