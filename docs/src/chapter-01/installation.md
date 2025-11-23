# Installation

This guide covers the installation and setup of BotServer on various platforms.

## Prerequisites

- **Rust** 1.70+ (for building from source)
- **PostgreSQL** 14+ (for database)
- **Docker** (optional, for containerized deployment)
- **Git** (for cloning the repository)

## System Requirements

- **OS**: Linux, macOS, or Windows
- **RAM**: Minimum 4GB, recommended 8GB+
- **Disk**: 10GB for installation + data storage
- **CPU**: 2+ cores recommended

## Installation Methods

### 1. Quick Start with Docker

```bash
# Clone the repository
git clone https://github.com/yourusername/botserver
cd botserver

# Start all services
docker-compose up -d
```

### 2. Build from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/botserver
cd botserver

# Build the project
cargo build --release

# Run the server
./target/release/botserver
```

### 3. Package Manager Installation

```bash
# Initialize package manager
botserver init

# Install required components
botserver install tables
botserver install cache
botserver install drive
botserver install llm

# Start services
botserver start all
```

## Environment Variables

BotServer uses only two environment variables:

### Required Variables

```bash
# Database connection string
DATABASE_URL=postgres://gbuser:password@localhost:5432/botserver

# Object storage configuration
DRIVE_SERVER=http://localhost:9000
DRIVE_ACCESSKEY=gbdriveuser
DRIVE_SECRET=your_secret_key
```

**Important**: These are the ONLY environment variables used by BotServer. All other configuration is managed through:
- `config.csv` files in bot packages
- Database configuration tables
- Command-line arguments

## Configuration

### Bot Configuration

Each bot has its own `config.csv` file with parameters like:

```csv
name,value
server_host,0.0.0.0
server_port,8080
llm-url,http://localhost:8081
llm-model,path/to/model.gguf
email-from,from@domain.com
email-server,mail.domain.com
```

See the [Configuration Guide](../chapter-02/gbot.md) for complete parameter reference.

### Theme Configuration

Themes are configured through simple parameters in `config.csv`:

```csv
name,value
theme-color1,#0d2b55
theme-color2,#fff9c2
theme-title,My Bot
theme-logo,https://example.com/logo.svg
```

## Database Setup

### Automatic Setup

```bash
# Bootstrap command creates database and tables
botserver bootstrap
```

### Manual Setup

```sql
-- Create database
CREATE DATABASE botserver;

-- Create user
CREATE USER gbuser WITH PASSWORD 'your_password';

-- Grant permissions
GRANT ALL PRIVILEGES ON DATABASE botserver TO gbuser;
```

Then run migrations:

```bash
diesel migration run
```

## Storage Setup

BotServer uses S3-compatible object storage (MinIO by default):

```bash
# Install MinIO
botserver install drive

# Start MinIO
botserver start drive
```

Default MinIO console: http://localhost:9001
- Username: `minioadmin`
- Password: `minioadmin`

## Authentication Setup

BotServer uses an external directory service for authentication:

```bash
# Install directory service
botserver install directory

# Start directory
botserver start directory
```

The directory service handles:
- User authentication
- OAuth2/OIDC flows
- User management
- Access control

## LLM Setup

### Local LLM Server

```bash
# Install LLM server
botserver install llm

# Download a model
wget https://huggingface.co/models/your-model.gguf -O data/llm/model.gguf

# Configure in config.csv
llm-url,http://localhost:8081
llm-model,data/llm/model.gguf
```

### External LLM Provider

Configure in `config.csv`:

```csv
name,value
llm-url,https://api.openai.com/v1
llm-key,your-api-key
llm-model,gpt-4
```

## Verifying Installation

### Check Component Status

```bash
# Check all services
botserver status

# Test database connection
psql $DATABASE_URL -c "SELECT version();"

# Test storage
curl http://localhost:9000/minio/health/live

# Test LLM
curl http://localhost:8081/v1/models
```

### Run Test Bot

```bash
# Create a test bot
cp -r templates/default.gbai work/test.gbai

# Start the server
botserver run

# Access web interface
open http://localhost:8080
```

## Troubleshooting

### Database Connection Issues

```bash
# Check PostgreSQL is running
systemctl status postgresql

# Test connection
psql -h localhost -U gbuser -d botserver

# Check DATABASE_URL format
echo $DATABASE_URL
```

### Storage Connection Issues

```bash
# Check MinIO is running
docker ps | grep minio

# Test credentials
aws s3 ls --endpoint-url=$DRIVE_SERVER
```

### Port Conflicts

Default ports used by BotServer:

| Service | Port | Configure in |
|---------|------|--------------|
| Web Server | 8080 | config.csv: `server_port` |
| PostgreSQL | 5432 | DATABASE_URL |
| MinIO | 9000/9001 | DRIVE_SERVER |
| LLM Server | 8081 | config.csv: `llm-server-port` |
| Cache (Valkey) | 6379 | Internal |

### Memory Issues

For systems with limited RAM:

1. Reduce LLM context size in `config.csv`:
   ```csv
   llm-server-ctx-size,2048
   ```

2. Limit parallel processing:
   ```csv
   llm-server-parallel,2
   ```

3. Use smaller models

## Next Steps

- [Quick Start Guide](./quick-start.md) - Create your first bot
- [Configuration Reference](../chapter-02/gbot.md) - All configuration options
- [BASIC Programming](../chapter-05/basics.md) - Learn the scripting language
- [Deployment Guide](../chapter-06/containers.md) - Production deployment