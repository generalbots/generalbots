# Installation

## System Requirements

- **Operating System**: Linux, macOS, or Windows
- **Memory**: 8GB RAM minimum, 16GB recommended
- **Storage**: 10GB free space
- **Dependencies**: Docker (optional), PostgreSQL, Redis

## Installation Methods

### Method 1: Package Manager (Recommended)

```bash
# Install using the built‑in package manager
botserver install tables
botserver install drive  
botserver install cache
botserver install llm
```

### Method 2: Manual Installation

1. Download the botserver binary
2. Set environment variables:
```bash
export DATABASE_URL="postgres://gbuser:password@localhost:5432/botserver"
export DRIVE_SERVER="http://localhost:9000"
export DRIVE_ACCESSKEY="minioadmin"
export DRIVE_SECRET="minioadmin"
```
3. Run the server: `./botserver`

## Configuration

Create a `.env` file in your working directory:

```env
BOT_GUID=your-bot-id
DATABASE_URL=postgres://gbuser:password@localhost:5432/botserver
DRIVE_SERVER=http://localhost:9000
DRIVE_ACCESSKEY=minioadmin
DRIVE_SECRET=minioadmin
```

## Verification

After installation, verify everything is working:

1. Access the web interface at `http://localhost:8080`
2. Check that all services are running:
```bash
botserver status tables
botserver status drive
botserver status cache
botserver status llm
```

The system will automatically create necessary database tables and storage buckets on first run.
