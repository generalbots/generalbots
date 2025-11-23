# Container Deployment (LXC)

BotServer uses **LXC** (Linux Containers) for isolated component deployment. This provides system-level containerization without the overhead of full virtualization.

## What is LXC?

LXC is a lightweight container technology that runs on Linux:

- **System containers** - Full Linux userspace (like lightweight VMs)
- **Shared kernel** - More efficient than virtual machines
- **Isolation** - Separate process trees, networking, filesystems
- **Resource control** - CPU, memory, and I/O limits

BotServer uses LXC to run PostgreSQL, drive, and cache in isolated containers.

## Automatic Container Setup

When you run BotServer with `--container` flag:

```bash
./botserver --container
```

The bootstrap process automatically:

1. **Detects LXC/LXD** - Checks if `lxc` command is available
2. **Initializes LXD** - Runs `lxd init --auto` if needed
3. **Creates containers** - Launches Debian 12 containers for each component
4. **Mounts directories** - Binds host paths for persistent data
5. **Configures networking** - Maps container ports to localhost
6. **Installs components** - Runs installation inside containers
7. **Creates services** - Sets up systemd inside containers
8. **Starts containers** - Everything starts automatically

## Container Architecture

### Container Naming

Each component runs in a dedicated container:

```
{tenant}-tables      → PostgreSQL database
{tenant}-drive       → Drive (S3-compatible object storage)
{tenant}-cache       → Cache (Valkey)
{tenant}-llm         → LLM server (optional)
{tenant}-email       → Stalwart mail (optional)
```

Default tenant is `default`, so containers are named:
- `default-tables`
- `default-drive`
- `default-cache`

### Directory Mounting

Host directories are mounted into containers for persistence:

```
Host: botserver-stack/tables/data/
  → Container: /opt/gbo/data/

Host: botserver-stack/tables/conf/
  → Container: /opt/gbo/conf/

Host: botserver-stack/tables/logs/
  → Container: /opt/gbo/logs/
```

Data persists even if containers are deleted!

### Port Forwarding

Container ports are mapped to localhost:

```
Container: 5432 → Host: 5432   (PostgreSQL)
Container: 9000 → Host: 9000   (Drive API)
Container: 9001 → Host: 9001   (Drive Console)
Container: 6379 → Host: 6379   (Cache)
```

Access services on localhost as if they were running natively!

## Manual Container Operations

### List Containers

```bash
lxc list
```

Output:
```
+----------------+---------+----------------------+
|      NAME      |  STATE  |       IPV4           |
+----------------+---------+----------------------+
| default-tables | RUNNING | 10.x.x.x (eth0)      |
| default-drive  | RUNNING | 10.x.x.x (eth0)      |
| default-cache  | RUNNING | 10.x.x.x (eth0)      |
+----------------+---------+----------------------+
```

### Execute Commands in Container

```bash
# PostgreSQL container
lxc exec default-tables -- psql -U gbuser botserver

# Drive container
lxc exec default-drive -- mc admin info local

# Cache container
lxc exec default-cache -- valkey-cli ping
```

### View Container Logs

```bash
# Container system logs
lxc exec default-tables -- journalctl -u tables

# Service logs (mounted from host)
tail -f botserver-stack/tables/logs/postgresql.log
```

### Stop/Start Containers

```bash
# Stop a container
lxc stop default-tables

# Start a container
lxc start default-tables

# Restart a container
lxc restart default-drive
```

### Delete Containers

```bash
# Stop and delete
lxc stop default-tables
lxc delete default-tables

# Or force delete
lxc delete default-tables --force
```

**Note**: Data in mounted directories persists!

## Container Configuration

### Resource Limits

Limit CPU and memory per container:

```bash
# Limit to 2 CPU cores
lxc config set default-tables limits.cpu 2

# Limit to 4GB RAM
lxc config set default-tables limits.memory 4GB

# View configuration
lxc config show default-tables
```

### Privileged vs Unprivileged

By default, BotServer creates **privileged containers** for compatibility:

```bash
lxc launch images:debian/12 default-tables \
  -c security.privileged=true
```

For better security, use **unprivileged containers** (may require additional setup):

```bash
lxc launch images:debian/12 default-tables \
  -c security.privileged=false
```

### Storage Backends

LXC supports multiple storage backends:

- **dir** - Simple directory (default)
- **btrfs** - Copy-on-write filesystem
- **zfs** - Advanced filesystem with snapshots
- **lvm** - Logical volume management

Check current backend:

```bash
lxc storage list
```

## Advanced Usage

### Container Snapshots

Create snapshots before updates:

```bash
# Create snapshot
lxc snapshot default-tables backup-2024-01-15

# List snapshots
lxc info default-tables

# Restore snapshot
lxc restore default-tables backup-2024-01-15

# Delete snapshot
lxc delete default-tables/backup-2024-01-15
```

### Container Cloning

Clone containers for testing:

```bash
# Clone a container
lxc copy default-tables test-tables

# Start the clone
lxc start test-tables
```

### Network Configuration

View container network:

```bash
# Show network interfaces
lxc exec default-tables -- ip addr

# Show network devices
lxc config device show default-tables
```

Add additional network interfaces:

```bash
lxc config device add default-tables eth1 nic \
  nictype=bridged parent=lxdbr0
```

## Troubleshooting

### LXC Not Installed

```bash
# Ubuntu/Debian
sudo snap install lxd
sudo lxd init --auto

# Or APT (older)
sudo apt install lxd lxd-client
sudo lxd init --auto
```

### Permission Denied

```bash
# Add your user to lxd group
sudo usermod -aG lxd $USER

# Logout and login again
newgrp lxd
```

### Container Won't Start

```bash
# Check container status
lxc info default-tables

# View container logs
lxc console default-tables --show-log

# Check for errors
lxc exec default-tables -- dmesg | tail
```

### Port Already in Use

```bash
# Check what's using the port
sudo netstat -tulpn | grep 5432

# Change port forwarding
lxc config device remove default-tables port-5432
lxc config device add default-tables port-5433 proxy \
  listen=tcp:0.0.0.0:5433 connect=tcp:127.0.0.1:5432
```

### Container Can't Access Network

```bash
# Restart LXD networking
sudo systemctl restart lxd-agent

# Check firewall rules
sudo iptables -L
```

## Monitoring

### Resource Usage

```bash
# CPU and memory usage
lxc list --columns ns4t

# Detailed info
lxc info default-tables
```

### Disk Usage

```bash
# Check container disk usage
lxc exec default-tables -- df -h

# Check all containers
lxc list --format json | jq -r '.[].name' | while read container; do
  echo -n "$container: "
  lxc exec $container -- df -h / --output=used | tail -n1
done
```

### Process Monitoring

```bash
# Show processes in container
lxc exec default-tables -- ps aux

# Show top processes
lxc exec default-tables -- htop
```

## Security Best Practices

### 1. Use Unprivileged Containers

When possible, avoid privileged containers:

```bash
lxc launch images:debian/12 default-tables \
  -c security.privileged=false
```

### 2. Isolate Networks

Create separate networks for different tenants:

```bash
lxc network create tenant1-net
lxc network attach tenant1-net tenant1-tables eth0
```

### 3. Limit Resources

Prevent resource exhaustion:

```bash
lxc config set default-tables limits.cpu 4
lxc config set default-tables limits.memory 8GB
lxc config set default-tables limits.disk 50GB
```

### 4. Regular Updates

Keep containers updated:

```bash
lxc exec default-tables -- apt update
lxc exec default-tables -- apt upgrade -y
```

### 5. Backup Snapshots

Regular snapshots for disaster recovery:

```bash
# Create daily snapshot
lxc snapshot default-tables daily-$(date +%Y%m%d)

# Automated with cron
0 2 * * * lxc snapshot default-tables daily-$(date +\%Y\%m\%d)
```

## Container vs Local Mode

### When to Use Containers

✅ **Use containers when:**
- You want clean isolation
- Running multiple BotServer instances
- Easy cleanup/reinstall is important
- Testing new versions
- Security isolation is critical

### When to Use Local Mode

✅ **Use local when:**
- Maximum performance needed
- LXC not available
- Simple single-instance deployment
- Direct access to services preferred

## Migration

### Local to Container

1. Export data from local installation
2. Install in container mode
3. Import data

```bash
# Backup local data
pg_dump botserver > backup.sql

# Switch to container mode
./botserver --container

# Restore data
lxc exec default-tables -- psql -U gbuser botserver < backup.sql
```

### Container to Local

```bash
# Backup from container
lxc exec default-tables -- pg_dump -U gbuser botserver > backup.sql

# Uninstall containers
./botserver uninstall tables

# Install locally
./botserver install tables --local

# Restore data
psql -U gbuser botserver < backup.sql
```

## Next Steps

- [Architecture Overview](./architecture.md) - Understand the system design
- [Building from Source](./building.md) - Compile with container support
- [Service Layer](./services.md) - Learn about service management