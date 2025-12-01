# Container Deployment (LXC)

BotServer uses LXC (Linux Containers) for isolated component deployment with system-level containerization.

## What is LXC?

- **System containers** - Full Linux userspace (lightweight VMs)
- **Shared kernel** - More efficient than virtual machines
- **Isolation** - Separate processes, networking, filesystems
- **Resource control** - CPU, memory, I/O limits

## Automatic Setup

```bash
./botserver --container
```

This automatically:
1. Detects LXC/LXD availability
2. Initializes LXD if needed
3. Creates Debian 12 containers per component
4. Mounts directories for persistent data
5. Configures networking and ports
6. Installs and starts services

## Container Architecture

### Container Naming

```
{tenant}-tables      → PostgreSQL
{tenant}-drive       → S3-compatible storage
{tenant}-cache       → Valkey cache
{tenant}-llm         → LLM server (optional)
{tenant}-email       → Mail server (optional)
```

Default tenant: `default` → `default-tables`, `default-drive`, etc.

### Directory Mounting

```
Host: botserver-stack/tables/data/  → Container: /opt/gbo/data/
Host: botserver-stack/tables/conf/  → Container: /opt/gbo/conf/
Host: botserver-stack/tables/logs/  → Container: /opt/gbo/logs/
```

Data persists even if containers are deleted.

### Port Forwarding

| Container Port | Host Port | Service |
|----------------|-----------|---------|
| 5432 | 5432 | PostgreSQL |
| 9000 | 9000 | Drive API |
| 9001 | 9001 | Drive Console |
| 6379 | 6379 | Cache |

## Common Operations

```bash
# List containers
lxc list

# Execute command in container
lxc exec default-tables -- psql -U gbuser botserver

# View logs
lxc exec default-tables -- journalctl -u tables

# Stop/Start
lxc stop default-tables
lxc start default-tables

# Delete (data in mounts persists)
lxc delete default-tables --force
```

## Resource Limits

```bash
lxc config set default-tables limits.cpu 2
lxc config set default-tables limits.memory 4GB
```

## Snapshots

```bash
# Create
lxc snapshot default-tables backup-2024-01-15

# List
lxc info default-tables

# Restore
lxc restore default-tables backup-2024-01-15
```

## Troubleshooting

| Issue | Solution |
|-------|----------|
| LXC not installed | `sudo snap install lxd && sudo lxd init --auto` |
| Permission denied | `sudo usermod -aG lxd $USER && newgrp lxd` |
| Container won't start | `lxc console default-tables --show-log` |
| Port in use | `sudo netstat -tulpn \| grep PORT` |

## Container vs Local

| Use Containers When | Use Local When |
|---------------------|----------------|
| Clean isolation needed | Maximum performance |
| Multiple instances | LXC not available |
| Easy cleanup/reinstall | Simple deployment |
| Security isolation | Direct service access |

## Migration

### Local → Container

```bash
pg_dump botserver > backup.sql
./botserver --container
lxc exec default-tables -- psql -U gbuser botserver < backup.sql
```

### Container → Local

```bash
lxc exec default-tables -- pg_dump -U gbuser botserver > backup.sql
./botserver uninstall tables
./botserver install tables --local
psql -U gbuser botserver < backup.sql
```

## See Also

- [Installation](../chapter-01/installation.md) - Local setup
- [Docker Deployment](./docker-deployment.md) - Docker alternative
- [Architecture](./architecture.md) - System design