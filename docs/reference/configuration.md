# Configuration Reference

Complete reference for General Bots configuration options.

## Configuration Files

### Bot Configuration (`config.csv`)

Located in each bot's `.gbot` folder:

```
mybot.gbai/
└── mybot.gbot/
    └── config.csv
```

Format: CSV with `name,value` columns.

```csv
name,value
theme-title,My Bot
theme-color1,#1565C0
theme-color2,#E3F2FD
```

## Theme Settings

| Setting | Description | Default | Example |
|---------|-------------|---------|---------|
| `theme-title` | Bot display name | Bot ID | `My Company Bot` |
| `theme-color1` | Primary color | `#1565C0` | `#2196F3` |
| `theme-color2` | Secondary/background | `#E3F2FD` | `#FFFFFF` |
| `theme-logo` | Logo URL | Default logo | `https://example.com/logo.svg` |
| `theme-favicon` | Favicon URL | Default | `https://example.com/favicon.ico` |

### Color Examples

```csv
name,value
theme-color1,#1565C0
theme-color2,#E3F2FD
```

| Scheme | Primary | Secondary |
|--------|---------|-----------|
| Blue | `#1565C0` | `#E3F2FD` |
| Green | `#2E7D32` | `#E8F5E9` |
| Purple | `#7B1FA2` | `#F3E5F5` |
| Orange | `#EF6C00` | `#FFF3E0` |
| Red | `#C62828` | `#FFEBEE` |
| Dark | `#212121` | `#424242` |

## Prompt Settings

| Setting | Description | Default | Range |
|---------|-------------|---------|-------|
| `prompt-history` | Messages in context | `2` | 1-10 |
| `prompt-compact` | Compact mode threshold | `4` | 2-20 |
| `prompt-max-tokens` | Max response tokens | `2048` | 256-8192 |
| `prompt-temperature` | Response creativity | `0.7` | 0.0-2.0 |

```csv
name,value
prompt-history,2
prompt-compact,4
prompt-max-tokens,2048
prompt-temperature,0.7
```

### History Settings

- `prompt-history=1`: Minimal context, faster responses
- `prompt-history=2`: Balanced (recommended)
- `prompt-history=5`: More context, slower responses

## LLM Settings

| Setting | Description | Default |
|---------|-------------|---------|
| `llm-provider` | LLM provider | `openai` |
| `llm-model` | Model name | `gpt-4` |
| `llm-api-key` | API key (or use env) | - |
| `llm-endpoint` | Custom endpoint | Provider default |

```csv
name,value
llm-provider,openai
llm-model,gpt-4-turbo
```

### Supported Providers

| Provider | Models |
|----------|--------|
| `openai` | `gpt-4`, `gpt-4-turbo`, `gpt-3.5-turbo` |
| `anthropic` | `claude-3-opus`, `claude-3-sonnet` |
| `groq` | `llama-3-70b`, `mixtral-8x7b` |
| `ollama` | Any local model |

## Feature Flags

| Setting | Description | Default |
|---------|-------------|---------|
| `feature-voice` | Enable voice input/output | `false` |
| `feature-file-upload` | Allow file uploads | `true` |
| `feature-suggestions` | Show quick replies | `true` |
| `feature-typing` | Show typing indicator | `true` |
| `feature-history` | Show chat history | `true` |

```csv
name,value
feature-voice,true
feature-file-upload,true
feature-suggestions,true
```

## Security Settings

| Setting | Description | Default |
|---------|-------------|---------|
| `auth-required` | Require authentication | `false` |
| `auth-provider` | Auth provider | `zitadel` |
| `allowed-domains` | Allowed email domains | `*` |
| `rate-limit` | Requests per minute | `60` |

```csv
name,value
auth-required,true
auth-provider,zitadel
allowed-domains,example.com,company.org
rate-limit,30
```

## Environment Variables

### Required

| Variable | Description |
|----------|-------------|
| `DIRECTORY_URL` | Zitadel/Auth instance URL |
| `DIRECTORY_CLIENT_ID` | OAuth client ID |
| `DIRECTORY_CLIENT_SECRET` | OAuth client secret |

### Optional Overrides

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection | Auto-configured |
| `REDIS_URL` | Redis connection | Auto-configured |
| `S3_ENDPOINT` | S3/MinIO endpoint | Auto-configured |
| `S3_ACCESS_KEY` | S3 access key | Auto-configured |
| `S3_SECRET_KEY` | S3 secret key | Auto-configured |
| `QDRANT_URL` | Vector DB URL | Auto-configured |

### LLM API Keys

| Variable | Description |
|----------|-------------|
| `OPENAI_API_KEY` | OpenAI API key |
| `ANTHROPIC_API_KEY` | Anthropic API key |
| `GROQ_API_KEY` | Groq API key |

### Server Settings

| Variable | Description | Default |
|----------|-------------|---------|
| `PORT` | HTTP server port | `8080` |
| `HOST` | Bind address | `0.0.0.0` |
| `RUST_LOG` | Log level | `info` |
| `WORKERS` | Thread pool size | CPU cores |

### Example `.env` File

```bash
# Authentication (required)
DIRECTORY_URL=https://auth.example.com
DIRECTORY_CLIENT_ID=abc123
DIRECTORY_CLIENT_SECRET=secret

# LLM Provider
OPENAI_API_KEY=sk-...

# Optional overrides
DATABASE_URL=postgres://user:pass@localhost/botserver
REDIS_URL=redis://localhost:6379

# Server
PORT=8080
RUST_LOG=info,botserver=debug
```

## Rate Limiting

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `RATE_LIMIT_ENABLED` | Enable rate limiting | `true` |
| `RATE_LIMIT_API_RPS` | API requests/second | `100` |
| `RATE_LIMIT_API_BURST` | API burst limit | `200` |
| `RATE_LIMIT_AUTH_RPS` | Auth requests/second | `10` |
| `RATE_LIMIT_AUTH_BURST` | Auth burst limit | `20` |
| `RATE_LIMIT_LLM_RPS` | LLM requests/second | `5` |
| `RATE_LIMIT_LLM_BURST` | LLM burst limit | `10` |

```bash
RATE_LIMIT_ENABLED=true
RATE_LIMIT_API_RPS=100
RATE_LIMIT_LLM_RPS=5
```

## Logging

### Log Levels

| Level | Description |
|-------|-------------|
| `error` | Errors only |
| `warn` | Warnings and errors |
| `info` | General information (default) |
| `debug` | Detailed debugging |
| `trace` | Very verbose |

### Module-Specific Logging

```bash
# General info, debug for botserver
RUST_LOG=info,botserver=debug

# Quiet except errors, debug for specific module
RUST_LOG=error,botserver::llm=debug

# Full trace for development
RUST_LOG=trace
```

## Database Configuration

### Connection Pool

| Setting | Description | Default |
|---------|-------------|---------|
| `DB_POOL_MIN` | Minimum connections | `2` |
| `DB_POOL_MAX` | Maximum connections | `10` |
| `DB_TIMEOUT` | Connection timeout (sec) | `30` |

### PostgreSQL Tuning

```sql
-- Recommended settings for production
ALTER SYSTEM SET shared_buffers = '256MB';
ALTER SYSTEM SET effective_cache_size = '1GB';
ALTER SYSTEM SET max_connections = 200;
```

## Storage Configuration

### S3/MinIO Settings

| Variable | Description | Default |
|----------|-------------|---------|
| `S3_ENDPOINT` | Endpoint URL | Auto |
| `S3_REGION` | AWS region | `us-east-1` |
| `S3_BUCKET` | Default bucket | `botserver` |
| `S3_ACCESS_KEY` | Access key | Auto |
| `S3_SECRET_KEY` | Secret key | Auto |

## Cache Configuration

### Redis Settings

| Variable | Description | Default |
|----------|-------------|---------|
| `REDIS_URL` | Connection URL | Auto |
| `CACHE_TTL` | Default TTL (seconds) | `3600` |
| `SEMANTIC_CACHE_ENABLED` | Enable LLM caching | `true` |
| `SEMANTIC_CACHE_THRESHOLD` | Similarity threshold | `0.95` |

## Complete Example

### config.csv

```csv
name,value
theme-title,Acme Support Bot
theme-color1,#1565C0
theme-color2,#E3F2FD
theme-logo,https://acme.com/logo.svg
prompt-history,2
prompt-compact,4
prompt-temperature,0.7
llm-provider,openai
llm-model,gpt-4-turbo
feature-voice,false
feature-file-upload,true
feature-suggestions,true
auth-required,true
rate-limit,30
```

### .env

```bash
# Auth
DIRECTORY_URL=https://auth.acme.com
DIRECTORY_CLIENT_ID=bot-client
DIRECTORY_CLIENT_SECRET=supersecret

# LLM
OPENAI_API_KEY=sk-...

# Server
PORT=8080
RUST_LOG=info

# Rate limiting
RATE_LIMIT_ENABLED=true
RATE_LIMIT_API_RPS=100
```

## Configuration Precedence

1. **Environment variables** (highest priority)
2. **Bot config.csv**
3. **Default values** (lowest priority)

Environment variables always override config.csv settings.

## Validation

On startup, General Bots validates configuration and logs warnings for:

- Missing required settings
- Invalid values
- Deprecated options
- Security concerns (e.g., weak rate limits)

Check logs for configuration issues:

```bash
RUST_LOG=info cargo run 2>&1 | grep -i config
```
