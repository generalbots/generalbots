# Installation

BotServer installs itself automatically through the bootstrap process. No manual setup required - just run the binary and everything gets configured.

## Bootstrap Flow Diagram

![Auto-Bootstrap Process](./assets/bootstrap-process.svg)

## System Requirements

### Minimum Requirements
- **OS**: Linux, macOS, or Windows
- **RAM**: 4GB minimum
- **Disk**: 10GB for installation + data storage
- **CPU**: 1 core (sufficient for development/testing)

### Recommended for Production
- **OS**: Linux server (Ubuntu/Debian preferred)
- **RAM**: 16GB or more
- **Disk**: 100GB SSD storage
- **CPU**: 2+ cores
- **GPU**: RTX 3060 or better (12GB VRAM minimum) for local LLM hosting

## Quick Start

BotServer handles all dependencies automatically:

```bash
# Download and run
./botserver
```

The bootstrap process automatically downloads everything to `botserver-stack/`:
- PostgreSQL database  
- Drive (S3-compatible object storage)  
- Valkey cache
- LLM server and models
- All required dependencies

**No manual installation required!**

## Environment Variables

The `.env` file is **automatically generated** during bootstrap from a blank environment with secure random credentials.

### Automatic Generation (Bootstrap Mode)

When you first run `./botserver`, it creates `.env` with:

```bash
# Auto-generated secure credentials
DATABASE_URL=postgres://gbuser:RANDOM_PASS@localhost:5432/botserver
DRIVE_SERVER=http://localhost:9000
DRIVE_ACCESSKEY=GENERATED_KEY
DRIVE_SECRET=GENERATED_SECRET
```

### Using Existing Services

If you already have PostgreSQL or drive storage running, you can point to them:

```bash
# Point to your existing PostgreSQL
DATABASE_URL=postgres://myuser:mypass@myhost:5432/mydb

# Point to your existing drive/S3
DRIVE_SERVER=http://my-drive:9000
DRIVE_ACCESSKEY=my-access-key
DRIVE_SECRET=my-secret-key
```

## Configuration

### Bot Configuration Parameters

Each bot has a `config.csv` file in its `.gbot/` directory. Available parameters:

#### Server Configuration
```csv
name,value
server-host,0.0.0.0
server-port,8080
sites-root,/tmp
```

#### LLM Configuration
```csv
name,value
llm-key,none
llm-url,http://localhost:8081
llm-model,../../../../data/llm/model.gguf
llm-cache,false            # Semantic cache (needs integration)
llm-cache-ttl,3600         # Cache TTL in seconds (needs integration)
llm-cache-semantic,true    # Enable semantic matching (needs integration)
llm-cache-threshold,0.95   # Similarity threshold (needs integration)
```

#### LLM Server Settings
```csv
name,value
llm-server,false
llm-server-path,botserver-stack/bin/llm/build/bin
llm-server-host,0.0.0.0
llm-server-port,8081
llm-server-gpu-layers,0
llm-server-n-moe,0
llm-server-ctx-size,4096
llm-server-n-predict,1024
llm-server-parallel,6
llm-server-cont-batching,true
```

#### Email Configuration
```csv
name,value
email-from,from@domain.com
email-server,mail.domain.com
email-port,587
email-user,user@domain.com
email-pass,yourpassword
```

#### Theme Configuration
```csv
name,value
theme-color1,#0d2b55
theme-color2,#fff9c2
theme-title,My Bot
theme-logo,https://example.com/logo.svg
```

#### Prompt Configuration
```csv
name,value
prompt-history,2
prompt-compact,4
```

## Bootstrap Process

When you first run BotServer, it:

1. **Detects your system** - Identifies OS and architecture
2. **Creates directories** - Sets up `botserver-stack/` structure
3. **Downloads components** - Gets all required binaries
4. **Configures services** - Sets up database, storage, and cache
5. **Initializes database** - Creates tables and initial data
6. **Deploys default bot** - Creates a working bot instance
7. **Starts services** - Launches all components

This typically takes 2-5 minutes on first run.

## Storage Setup

BotServer uses S3-compatible object storage. Each bot deployment creates a new bucket in the drive:

```bash
# Deploy a new bot = create a new bucket in drive
# Bots are stored in the object storage, not local filesystem
mybot.gbai → creates 'mybot' bucket in drive storage
```

**Note**: The `work/` folder is for internal use only and should not be used for bot deployment. Bot packages should be deployed directly to the object storage (drive).

The storage server runs on:
- API: http://localhost:9000
- Console: http://localhost:9001

### Local Development with S3 Sync Tools

You can edit your bot files locally and have them automatically sync to drive storage using S3-compatible tools:

#### Free S3 Sync Tools
- **Cyberduck** (Windows/Mac/Linux) - GUI file browser with S3 support
- **WinSCP** (Windows) - File manager with S3 protocol support  
- **Mountain Duck** (Windows/Mac) - Mount S3 as local drive
- **S3 Browser** (Windows) - Freeware S3 client
- **CloudBerry Explorer** (Windows/Mac) - Free version available
- **rclone** (All platforms) - Command-line sync tool

#### Setup Example with rclone
```bash
# Configure rclone for drive storage
rclone config
# Choose: n) New remote
# Name: drive
# Storage: s3
# Provider: Other
# Access Key: (from .env DRIVE_ACCESSKEY)
# Secret Key: (from .env DRIVE_SECRET)
# Endpoint: http://localhost:9000

# Sync local folder to bucket (watches for changes)
rclone sync ./mybot.gbai drive:mybot --watch

# Now edit files locally:
# - Edit mybot.gbai/mybot.gbot/config.csv → Bot reloads automatically
# - Edit mybot.gbai/mybot.gbdialog/*.bas → Scripts compile automatically
# - Add docs to mybot.gbai/mybot.gbkb/ → Knowledge base updates automatically
```

With this setup:
- ✅ Edit `.csv` files → Bot configuration updates instantly
- ✅ Edit `.bas` files → BASIC scripts compile automatically
- ✅ Add documents → Knowledge base reindexes automatically
- ✅ No manual uploads needed
- ✅ Works like local development but uses object storage

## Database Setup

PostgreSQL is automatically configured with:
- Database: `botserver`
- User: `gbuser`
- Tables created via migrations
- Connection pooling enabled

## Authentication Setup

BotServer uses an external directory service for user management:
- Handles user authentication
- Manages OAuth2/OIDC flows
- Controls access permissions
- Integrates with existing identity providers

## LLM Setup

### Local LLM (Recommended)

The bootstrap downloads a default model to `botserver-stack/data/llm/`. Configure in `config.csv`:

```csv
name,value
llm-url,http://localhost:8081
llm-model,../../../../data/llm/model.gguf
llm-server-gpu-layers,0
```

For GPU acceleration (RTX 3060 or better):
```csv
name,value
llm-server-gpu-layers,35
```

### External LLM Provider

To use external APIs, configure in `config.csv`:

```csv
name,value
llm-url,https://api.provider.com/v1
llm-key,your-api-key
llm-model,model-name
```

## Container Deployment (LXC)

For production isolation using Linux containers:

```bash
# Create container
lxc-create -n botserver -t download -- -d ubuntu -r jammy -a amd64

# Start container
lxc-start -n botserver

# Attach to container
lxc-attach -n botserver

# Install BotServer inside container
./botserver
```

## Verifying Installation

### Check Component Status

```bash
# Check all services
botserver status

# Test database connection
psql $DATABASE_URL -c "SELECT version();"

# Test storage
curl http://localhost:9000/health/live

# Test LLM
curl http://localhost:8081/v1/models
```

### Run Test Bot

```bash
# The default bot is automatically deployed to the drive during bootstrap
# Access UI interface
open http://localhost:8080
```

To deploy additional bots, upload them to the object storage, not the local filesystem. The `work/` folder is reserved for internal operations.

## Troubleshooting

### Database Connection Issues

```bash
# Check if PostgreSQL is running
ps aux | grep postgres

# Test connection
psql -h localhost -U gbuser -d botserver

# Verify DATABASE_URL
echo $DATABASE_URL
```

### Storage Connection Issues

```bash
# Check drive process
ps aux | grep minio

# Test storage access
curl -I $DRIVE_SERVER/health/live
```

### Port Conflicts

Default ports used:

| Service | Port | Configure in |
|---------|------|--------------|
| UI Server | 8080 | config.csv: `server-port` |
| PostgreSQL | 5432 | DATABASE_URL |
| Drive API | 9000 | DRIVE_SERVER |
| Drive Console | 9001 | N/A |
| LLM Server | 8081 | config.csv: `llm-server-port` |
| Embedding Server | 8082 | config.csv: `embedding-url` |
| Valkey Cache | 6379 | Internal |

### Memory Issues

For systems with limited RAM:

1. Reduce LLM context size:
   ```csv
   llm-server-ctx-size,2048
   ```

2. Limit parallel processing:
   ```csv
   llm-server-parallel,2
   ```

3. Use quantized models (Q3_K_M or Q4_K_M)

4. For Mixture of Experts models, adjust CPU MoE threads:
   ```csv
   llm-server-n-moe,4
   ```

### GPU Issues

If GPU is not detected:

1. Check CUDA installation (NVIDIA)
2. Verify GPU memory (12GB minimum)
3. Set `llm-server-gpu-layers` to 0 for CPU-only mode

## Next Steps

- [Quick Start Guide](./quick-start.md) - Create your first bot
- [First Conversation](./first-conversation.md) - Test your bot
- [Configuration Reference](../chapter-02/gbot.md) - All configuration options
- [BASIC Programming](../chapter-05/basics.md) - Learn the scripting language