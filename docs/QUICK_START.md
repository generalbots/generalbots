# Quick Start Guide

## Prerequisites

- Rust 1.75+ and Cargo
- PostgreSQL 14+ (or Docker)
- Optional: MinIO for S3-compatible storage
- Optional: Redis/Valkey for caching

## Installation

### 1. Clone the Repository

```bash
git clone https://github.com/GeneralBots/BotServer.git
cd BotServer
```

### 2. Build the Project

```bash
# Build with default features (includes console UI)
cargo build

# Build with all features
cargo build --all-features

# Build for release
cargo build --release
```

## Running BotServer

### Default Mode (with Console UI)

```bash
# Run with console UI showing real-time status, logs, and file browser
cargo run

# The console UI provides:
# - System metrics (CPU, Memory, GPU if available)
# - Service status monitoring
# - Real-time logs
# - File browser for drive storage
# - Database status
```

### Background Service Mode

```bash
# Run without console UI (background service)
cargo run -- --noconsole

# Run without any UI
cargo run -- --noui
```

### Desktop Mode

```bash
# Run with Tauri desktop application
cargo run -- --desktop
```

### Advanced Options

```bash
# Specify a tenant
cargo run -- --tenant my-organization

# Container deployment mode
cargo run -- --container

# Combine options
cargo run -- --noconsole --tenant production
```

## First-Time Setup

When you run BotServer for the first time, it will:

1. **Automatically start required services:**
   - PostgreSQL database (if not running)
   - MinIO S3-compatible storage (if configured)
   - Redis cache (if configured)

2. **Run database migrations automatically:**
   - Creates all required tables and indexes
   - Sets up initial schema

3. **Bootstrap initial configuration:**
   - Creates `.env` file with defaults
   - Sets up bot templates
   - Configures service endpoints

## Configuration

### Environment Variables

Copy the example environment file and customize:

```bash
cp .env.example .env
```

Key configuration options in `.env`:

```bash
# Database
DATABASE_URL=postgres://postgres:postgres@localhost:5432/botserver

# Server
SERVER_HOST=127.0.0.1
SERVER_PORT=8080

# Drive (MinIO)
DRIVE_SERVER=http://localhost:9000
DRIVE_ACCESSKEY=minioadmin
DRIVE_SECRET=minioadmin

# LLM Configuration
LLM_SERVER=http://localhost:8081
LLM_MODEL=llama2

# Logging (automatically configured)
# All external library traces are suppressed by default
# Use RUST_LOG=botserver=trace for detailed debugging
RUST_LOG=info
```

## Accessing the Application

### Web Interface

Once running, access the web interface at:
```
http://localhost:8080
```

### API Endpoints

- Health Check: `GET http://localhost:8080/api/health`
- Chat: `POST http://localhost:8080/api/chat`
- Tasks: `GET/POST http://localhost:8080/api/tasks`
- Drive: `GET http://localhost:8080/api/drive/files`

## Console UI Controls

When running with the console UI (default):

### Keyboard Shortcuts

- `Tab` - Switch between panels
- `↑/↓` - Navigate lists
- `Enter` - Select/Open
- `Esc` - Go back/Cancel
- `q` - Quit application
- `l` - View logs
- `f` - File browser
- `s` - System status
- `h` - Help

### Panels

1. **Status Panel** - System metrics and service health
2. **Logs Panel** - Real-time application logs
3. **File Browser** - Navigate drive storage
4. **Database Panel** - Connection status and stats

## Troubleshooting

### Database Connection Issues

```bash
# Check if PostgreSQL is running
ps aux | grep postgres

# Start PostgreSQL manually if needed
sudo systemctl start postgresql

# Verify connection
psql -U postgres -h localhost
```

### Drive/MinIO Issues

```bash
# Check if MinIO is running
ps aux | grep minio

# Start MinIO manually
./botserver-stack/bin/drive/minio server ./botserver-stack/data/drive
```

### Console UI Not Showing

```bash
# Ensure console feature is compiled
cargo build --features console

# Check terminal compatibility
echo $TERM  # Should be xterm-256color or similar
```

### High CPU/Memory Usage

```bash
# Run without console for lower resource usage
cargo run -- --noconsole

# Check running services
htop
```

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run with specific features
cargo test --features console

# Run integration tests
cargo test --test '*'
```

### Enable Detailed Logging

```bash
# Trace level for botserver only
RUST_LOG=botserver=trace cargo run

# Debug level
RUST_LOG=botserver=debug cargo run

# Info level (default)
RUST_LOG=info cargo run
```

### Building Documentation

```bash
# Build and open Rust documentation
cargo doc --open

# Build book documentation
cd docs && mdbook build
```

## Next Steps

1. **Configure your first bot** - See [Bot Configuration Guide](./BOT_CONFIGURATION.md)
2. **Set up integrations** - See [Integration Guide](./05-INTEGRATION_STATUS.md)
3. **Deploy to production** - See [Deployment Guide](./DEPLOYMENT.md)
4. **Explore the API** - See [API Documentation](./API.md)

## Getting Help

- **Documentation**: [Complete Docs](./INDEX.md)
- **Issues**: [GitHub Issues](https://github.com/GeneralBots/BotServer/issues)
- **Community**: [Discussions](https://github.com/GeneralBots/BotServer/discussions)