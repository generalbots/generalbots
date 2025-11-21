# Installation

## Quick Start

BotServer automatically installs and configures everything you need. Just run it!

```bash
# Download and run
./botserver

# Or build from source
cargo run
```

That's it! The bootstrap process handles everything automatically.

## What Happens Automatically

When you first run BotServer, the **bootstrap process** automatically:

1. **Detects your system** - Linux, macOS, or Windows
2. **Creates installation directory** - `/opt/gbo` (Linux/macOS) or `botserver-stack` (Windows)
3. **Installs core components**:
   - **PostgreSQL** (database) - Generates secure credentials, creates schema
   - **MinIO** (file storage) - Creates buckets, sets up access keys
   - **Valkey/Redis** (cache) - Configures and starts service
4. **Writes `.env` file** - All credentials stored securely
5. **Creates default bots** - Scans `templates/` for `.gbai` packages
6. **Starts web server** - Ready at `http://localhost:8080`

### First Run Output

You'll see messages like:

```
🚀 BotServer starting...
📦 Bootstrap: Detecting system...
📦 Installing PostgreSQL...
   ✓ Database created
   ✓ Schema initialized
   ✓ Credentials saved to .env
📦 Installing MinIO...
   ✓ Object storage ready
   ✓ Buckets created
📦 Installing Valkey...
   ✓ Cache server running
🤖 Creating bots from templates...
   ✓ default.gbai → Default bot
   ✓ announcements.gbai → Announcements bot
✅ BotServer ready at http://localhost:8080
```

## System Requirements

- **Memory**: 8GB RAM minimum, 16GB recommended
- **Storage**: 10GB free space
- **OS**: Linux, macOS, or Windows

## Installation Modes

BotServer automatically detects and chooses the best installation mode for your system.

### Local Mode (Default)

Components install directly on your system:
- PostgreSQL at `localhost:5432`
- MinIO at `localhost:9000`
- Valkey at `localhost:6379`

**When used**: No container runtime detected, or you prefer native installation.

### Container Mode (LXC)

Components run in isolated **LXC** (Linux Containers):
- System-level container isolation
- Lightweight virtualization
- Automatic setup with `lxd init --auto`

```bash
# Force container mode
./botserver --container

# Force local mode
./botserver --local
```

**Benefits**:
- ✅ Isolated environments - no system pollution
- ✅ Easy cleanup - just remove containers
- ✅ Version control - run multiple BotServer instances
- ✅ Security - containers provide isolation

**What happens**:
1. Bootstrap detects LXC/LXD availability
2. Creates Debian 12 containers for PostgreSQL, MinIO, Valkey
3. Mounts host directories for persistent data
4. Maps container ports to localhost
5. Creates systemd services inside containers
6. Manages container lifecycle automatically

**Container names**: `{tenant}-tables`, `{tenant}-drive`, `{tenant}-cache`

### Hybrid Mode

Mix local and containerized components:

```bash
# Install PostgreSQL locally, MinIO in container
./botserver install tables --local
./botserver install drive --container

# Or mix any combination
./botserver install cache --local
```

**Note**: Container mode requires LXC/LXD installed on your system.

## Post-Installation

### Access the Web Interface

Open your browser to `http://localhost:8080` and start chatting!

### Check Component Status

```bash
./botserver status tables    # PostgreSQL
./botserver status drive     # MinIO
./botserver status cache     # Valkey
```

### View Configuration

The bootstrap process creates `.env` with all credentials:

```bash
cat .env
```

You'll see auto-generated values like:

```env
DATABASE_URL=postgres://gbuser:SECURE_RANDOM_PASS@localhost:5432/botserver
DRIVE_SERVER=http://localhost:9000
DRIVE_ACCESSKEY=GENERATED_KEY
DRIVE_SECRET=GENERATED_SECRET
```

## Optional Components

After installation, you can add more components:

```bash
./botserver install email      # Stalwart email server
./botserver install directory  # Zitadel identity provider
./botserver install llm        # Local LLM server
./botserver install meeting    # LiveKit video conferencing
```

## Adding Your Own Bot

1. Create a `.gbai` folder in `templates/`:
   ```
   templates/mybot.gbai/
   ├── mybot.gbdialog/
   │   └── start.bas
   ├── mybot.gbot/
   │   └── config.csv
   └── mybot.gbkb/
       └── docs/
   ```

2. Restart BotServer:
   ```bash
   ./botserver
   ```

3. Bootstrap automatically detects and creates your bot!

## Troubleshooting

### Port Already in Use

If port 8080 is taken, edit `templates/default.gbai/default.gbot/config.csv`:

```csv
name,value
server_port,3000
```

### Database Connection Failed

The bootstrap will retry automatically. If it persists:

```bash
./botserver install tables --force
```

### LXC Not Available

If you see "LXC not found" and want container mode:

```bash
# Install LXC/LXD
sudo snap install lxd
sudo lxd init --auto

# Then run bootstrap
./botserver --container
```

### Clean Install

Remove installation directory and restart:

```bash
# Linux/macOS
rm -rf /opt/gbo
./botserver

# Windows
rmdir /s botserver-stack
botserver.exe
```

## Advanced: Manual Component Control

While bootstrap handles everything, you can manually control components:

```bash
# Install specific component
./botserver install <component>

# Start/stop components
./botserver start tables
./botserver stop drive

# Uninstall component
./botserver uninstall cache
```

Available components: `tables`, `cache`, `drive`, `llm`, `email`, `directory`, `proxy`, `dns`, `meeting`, `vector_db`, and more.

## Next Steps

- [First Conversation](./first-conversation.md) - Start chatting with your bot
- [Understanding Sessions](./sessions.md) - Learn how conversations work
- [About Packages](../chapter-02/README.md) - Create custom bots