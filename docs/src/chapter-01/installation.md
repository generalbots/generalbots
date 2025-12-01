# Installation

BotServer installs itself automatically through the bootstrap process. Just run the binary.

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

## Environment Configuration

The `.env` file is **auto-generated** with secure random credentials:

```bash
DATABASE_URL=postgres://gbuser:RANDOM@localhost:5432/botserver
DRIVE_SERVER=http://localhost:9000
DRIVE_ACCESSKEY=GENERATED_KEY
DRIVE_SECRET=GENERATED_SECRET
```

### Using Existing Services

Point to your own infrastructure in `.env`:

```bash
DATABASE_URL=postgres://myuser:mypass@myhost:5432/mydb
DRIVE_SERVER=http://my-drive:9000
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
open http://localhost:8080
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
| **Docker** | Production, microservices | [Docker Deployment](../chapter-07-gbapp/docker-deployment.md) |
| **LXC** | Isolated components, Linux | [Container Deployment](../chapter-07-gbapp/containers.md) |

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
- [Configuration Reference](../chapter-08-config/README.md) - All settings