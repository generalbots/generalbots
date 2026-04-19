# Container Deployment (LXC)

botserver uses LXC (Linux Containers) for isolated component deployment with system-level containerization.

> ⚠️ **IMPORTANT**: All container create and management commands must be run from the **host system**, not from inside a container. The botserver binary manages LXC containers from the host level.

## What is LXC?

- **System containers** - Full Linux userspace (lightweight VMs)
- **Shared kernel** - More efficient than virtual machines
- **Isolation** - Separate processes, networking, filesystems
- **Resource control** - CPU, memory, I/O limits

## Automatic Setup

Run on the **host system**:

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

Run these commands on the **host system**:

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

## Example: Create Vault and VectorDB Containers

Run on the **host system**:

```bash
# Install Vault for secrets management
botserver install vault --container --tenant mycompany

# Install VectorDB (Qdrant) for embeddings
botserver install vector_db --container --tenant mycompany

# Verify containers are running
lxc list | grep mycompany

# Get container IPs
lxc list mycompany-vault -c n4 --format csv
lxc list mycompany-vectordb -c n4 --format csv

# Test services
curl http://<vault-ip>:8200/v1/sys/health
curl http://<vectordb-ip>:6333/health
```

## Migration

### Local → Container

Run on the **host system**:

```bash
pg_dump botserver > backup.sql
./botserver --container
lxc exec default-tables -- psql -U gbuser botserver < backup.sql
```

### Container → Local

Run on the **host system**:

```bash
lxc exec default-tables -- pg_dump -U gbuser botserver > backup.sql
./botserver uninstall tables
./botserver install tables --local
psql -U gbuser botserver < backup.sql
```

## Brother Mode Configuration

If you are running `botserver` itself inside a container (e.g., LXC or Docker) but want it to manage other LXC containers on the host ("Brother Mode"), you must expose the host's LXD socket.

### Required LXD Profile

To allow child containers to communicate with the host LXD daemon, add the `lxd-sock` proxy device to the default profile. This maps the host's socket to `/tmp/lxd.sock` inside the container, avoiding conflicts with missing `/var/lib/lxd` directories in standard images.

LXD installed via snap uses `/var/snap/lxd/common/lxd/unix.socket`:

```bash
lxc profile device add default lxd-sock proxy \
  connect=unix:/var/snap/lxd/common/lxd/unix.socket \
  listen=unix:/tmp/lxd.sock \
  bind=container \
  uid=0 gid=0 mode=0660
```

For LXD installed via packages (non-snap), use:

```bash
lxc profile device add default lxd-sock proxy \
  connect=unix:/var/lib/lxd/unix.socket \
  listen=unix:/tmp/lxd.sock \
  bind=container \
  uid=0 gid=0 mode=0660
```

> **Note**: The `botserver` installer attempts to configure this automatically. If you encounter "socket not found" errors, verify this proxy device exists.

## See Also

- [Installation](../01-getting-started/installation.md) - Local setup
- [Docker Deployment](./docker-deployment.md) - Docker alternative
- [Architecture](./architecture.md) - System design