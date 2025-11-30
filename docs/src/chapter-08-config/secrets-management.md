# Secrets Management

General Bots uses a layered approach to configuration and secrets management. The goal is to keep `.env` **minimal** - containing only Vault connection info - while all sensitive data is stored securely in Vault.

## Configuration Layers

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         Configuration Hierarchy                              │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────┐     ┌─────────────┐     ┌─────────────┐     ┌───────────┐ │
│  │    .env     │     │   Zitadel   │     │   Vault     │     │config.csv │ │
│  │(Vault ONLY) │     │  (Identity) │     │  (Secrets)  │     │(Bot Config)│ │
│  └──────┬──────┘     └──────┬──────┘     └──────┬──────┘     └─────┬─────┘ │
│         │                   │                   │                   │       │
│         ▼                   ▼                   ▼                   ▼       │
│  • VAULT_ADDR        • User accounts     • Directory URL       • Bot params │
│  • VAULT_TOKEN       • Organizations     • Database creds      • LLM config │
│                      • Projects          • API keys            • Features   │
│                      • Applications      • Drive credentials   • Behavior   │
│                      • MFA settings      • Encryption keys                 │
│                      • SSO/OAuth         • ALL service secrets             │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## What Goes Where?

### .env (Vault Connection ONLY)

The `.env` file should contain **ONLY** Vault connection info:

```bash
# .env - ONLY Vault connection
# Everything else comes from Vault!

VAULT_ADDR=https://localhost:8200
VAULT_TOKEN=hvs.your-root-token
```

That's it. **Two variables only.**

**Why so minimal?**
- `.env` files can be accidentally committed to git
- Environment variables may appear in logs
- Reduces attack surface if server is compromised
- Single point of secret management (Vault)
- Easy rotation - change in Vault, not in files

### Zitadel (Identity & Access)

Zitadel manages **user-facing** identity:

| What | Example |
|------|---------|
| User accounts | john@example.com |
| Organizations | Acme Corp |
| Projects | Production Bot |
| Applications | Web UI, Mobile App |
| MFA settings | TOTP, SMS, WebAuthn |
| SSO providers | Google, Microsoft |
| User metadata | Department, Role |

**Not stored in Zitadel:**
- Service passwords
- API keys
- Encryption keys

### Vault (Service Secrets)

Vault manages **machine-to-machine** secrets:

| Path | Contents |
|------|----------|
| `gbo/drive` | MinIO access key and secret |
| `gbo/tables` | PostgreSQL username and password |
| `gbo/cache` | Redis password |
| `gbo/llm` | OpenAI, Anthropic, Groq API keys |
| `gbo/encryption` | Master encryption key, data keys |
| `gbo/email` | SMTP credentials |
| `gbo/meet` | LiveKit API key and secret |
| `gbo/alm` | Forgejo admin password, runner token |

### config.csv (Bot Configuration)

The bot's `config.csv` contains **non-sensitive** configuration:

```csv
# Bot behavior - NOT secrets
llm-provider,openai
llm-model,gpt-4o
llm-temperature,0.7
llm-max-tokens,4096

# Feature flags
feature-voice-enabled,true
feature-file-upload,true

# Vault references for sensitive values
llm-api-key,vault:gbo/llm/openai_key
```

Note: Most service credentials (database, drive, cache) are fetched automatically from Vault at startup. You only need `vault:` references in config.csv for bot-specific secrets like LLM API keys.

## How Secrets Flow

### At Startup

```
1. BotServer starts
2. Reads .env for VAULT_ADDR and VAULT_TOKEN (only 2 variables)
3. Connects to Vault
4. Fetches ALL service credentials:
   - gbo/directory → Zitadel URL, client_id, client_secret
   - gbo/tables → Database host, port, username, password
   - gbo/drive → MinIO endpoint, accesskey, secret
   - gbo/cache → Redis host, port, password
   - gbo/llm → API keys for all providers
   - gbo/encryption → Master encryption keys
5. Connects to all services using Vault credentials
6. Reads config.csv for bot configuration
7. For keys referencing Vault (vault:path/key):
   - Fetches from Vault automatically
8. System ready
```

### At Runtime

```
1. User sends message
2. Bot processes, needs LLM
3. Reads config.csv: llm-api-key = vault:gbo/llm/openai_key
4. Fetches from Vault (cached for performance)
5. Calls OpenAI API
6. Returns response
```

## Setting Up Vault

### Initial Setup

When you run `./botserver install secrets`, it:

1. Downloads and installs Vault
2. Initializes with a single unseal key
3. Creates initial secret paths
4. Outputs root token to `conf/vault/init.json`

```bash
# Check Vault status
./botserver status secrets

# View init credentials (protect this file!)
cat botserver-stack/conf/vault/init.json
```

### Storing Secrets

Use the Vault CLI or API:

```bash
# Directory (Zitadel) - includes URL, no longer in .env
vault kv put gbo/directory \
  url=https://localhost:8080 \
  project_id=your-project-id \
  client_id=your-client-id \
  client_secret=your-client-secret

# Database - includes host/port, no longer in .env
vault kv put gbo/tables \
  host=localhost \
  port=5432 \
  database=botserver \
  username=gbuser \
  password=secure-password

# Drive (MinIO)
vault kv put gbo/drive \
  endpoint=https://localhost:9000 \
  accesskey=minioadmin \
  secret=minioadmin123

# Cache (Redis)
vault kv put gbo/cache \
  host=localhost \
  port=6379 \
  password=redis-secret

# LLM API keys
vault kv put gbo/llm \
  openai_key=sk-xxxxx \
  anthropic_key=sk-ant-xxxxx \
  groq_key=gsk_xxxxx \
  deepseek_key=sk-xxxxx

# Encryption keys
vault kv put gbo/encryption \
  master_key=your-32-byte-key

# Vector database (Qdrant)
vault kv put gbo/vectordb \
  url=https://localhost:6334 \
  api_key=optional-api-key

# Observability (InfluxDB)
vault kv put gbo/observability \
  url=http://localhost:8086 \
  org=pragmatismo \
  bucket=metrics \
  token=your-influx-token
```

### Automatic Management

**Secrets are managed automatically** - you don't need a UI for day-to-day operations:

| Action | How It Works |
|--------|--------------|
| Service startup | Fetches credentials from Vault |
| Key rotation | Update in Vault, services reload |
| New bot deployment | Inherits organization secrets |
| LLM provider change | Update config.csv, key fetched automatically |

### Emergency Access

For emergency situations (lost credentials, key rotation), admins can:

1. **Access Vault UI**: `https://localhost:8200/ui`
2. **Use Vault CLI**: `vault kv get gbo/llm`
3. **Check init.json**: Contains unseal key and root token

```bash
# Emergency: unseal Vault after restart
UNSEAL_KEY=$(cat botserver-stack/conf/vault/init.json | jq -r '.unseal_keys_b64[0]')
vault operator unseal $UNSEAL_KEY
```

## Migrating from Environment Variables

If you're currently using environment variables:

### Before (Old Way)

```bash
# .env - TOO MANY SECRETS!
DATABASE_URL=postgres://user:password@localhost/db
DIRECTORY_URL=https://localhost:8080
DIRECTORY_PROJECT_ID=12345
REDIS_PASSWORD=redis-secret
OPENAI_API_KEY=sk-xxxxx
ANTHROPIC_API_KEY=sk-ant-xxxxx
DRIVE_ACCESSKEY=minio
DRIVE_SECRET=minio123
ENCRYPTION_KEY=super-secret-key
```

### After (With Vault)

```bash
# .env - ONLY VAULT CONNECTION
VAULT_ADDR=https://localhost:8200
VAULT_TOKEN=hvs.xxxxx
```

```bash
# EVERYTHING in Vault
vault kv put gbo/directory \
  url=https://localhost:8080 \
  project_id=12345 \
  client_id=xxx \
  client_secret=xxx

vault kv put gbo/tables \
  host=localhost \
  port=5432 \
  database=botserver \
  username=user \
  password=password

vault kv put gbo/cache \
  host=localhost \
  port=6379 \
  password=redis-secret

vault kv put gbo/llm \
  openai_key=sk-xxxxx \
  anthropic_key=sk-ant-xxxxx

vault kv put gbo/drive \
  endpoint=https://localhost:9000 \
  accesskey=minio \
  secret=minio123

vault kv put gbo/encryption \
  master_key=super-secret-key
```

### Migration Script

```bash
#!/bin/bash
# migrate-to-vault.sh

# Read existing .env
source .env

# Parse DATABASE_URL if present
if [ -n "$DATABASE_URL" ]; then
  # postgres://user:pass@host:port/db
  DB_USER=$(echo $DATABASE_URL | sed -n 's|postgres://\([^:]*\):.*|\1|p')
  DB_PASS=$(echo $DATABASE_URL | sed -n 's|postgres://[^:]*:\([^@]*\)@.*|\1|p')
  DB_HOST=$(echo $DATABASE_URL | sed -n 's|.*@\([^:]*\):.*|\1|p')
  DB_PORT=$(echo $DATABASE_URL | sed -n 's|.*:\([0-9]*\)/.*|\1|p')
  DB_NAME=$(echo $DATABASE_URL | sed -n 's|.*/\(.*\)|\1|p')
fi

# Store everything in Vault
vault kv put gbo/directory \
  url="${DIRECTORY_URL:-https://localhost:8080}" \
  project_id="${DIRECTORY_PROJECT_ID:-}" \
  client_id="${ZITADEL_CLIENT_ID:-}" \
  client_secret="${ZITADEL_CLIENT_SECRET:-}"

vault kv put gbo/tables \
  host="${DB_HOST:-localhost}" \
  port="${DB_PORT:-5432}" \
  database="${DB_NAME:-botserver}" \
  username="${DB_USER:-gbuser}" \
  password="${DB_PASS:-}"

vault kv put gbo/cache \
  host="${REDIS_HOST:-localhost}" \
  port="${REDIS_PORT:-6379}" \
  password="${REDIS_PASSWORD:-}"

vault kv put gbo/llm \
  openai_key="${OPENAI_API_KEY:-}" \
  anthropic_key="${ANTHROPIC_API_KEY:-}" \
  groq_key="${GROQ_API_KEY:-}" \
  deepseek_key="${DEEPSEEK_API_KEY:-}"

vault kv put gbo/drive \
  endpoint="${DRIVE_ENDPOINT:-https://localhost:9000}" \
  accesskey="${DRIVE_ACCESSKEY:-}" \
  secret="${DRIVE_SECRET:-}"

vault kv put gbo/encryption \
  master_key="${ENCRYPTION_KEY:-}"

# Clean up .env - ONLY Vault connection
cat > .env << EOF
# General Bots - Vault Connection Only
# All other secrets are stored in Vault

VAULT_ADDR=https://localhost:8200
VAULT_TOKEN=$VAULT_TOKEN
EOF

echo "Migration complete!"
echo ".env now contains only Vault connection."
echo "All secrets moved to Vault."
```

## Using Vault References in config.csv

Reference Vault secrets in your bot's config.csv:

```csv
# Direct value (non-sensitive)
llm-provider,openai
llm-model,gpt-4o
llm-temperature,0.7

# Vault reference (sensitive)
llm-api-key,vault:gbo/llm/openai_key

# Multiple keys from same path
drive-accesskey,vault:gbo/drive/accesskey
drive-secret,vault:gbo/drive/secret

# Per-bot secrets (for multi-tenant)
custom-api-key,vault:gbo/bots/mybot/api_key
```

### Syntax

```
vault:<path>/<key>
```

- `path`: Vault KV path (e.g., `gbo/llm`)
- `key`: Specific key within the secret (e.g., `openai_key`)

## Security Best Practices

### 1. Protect init.json

```bash
# Set restrictive permissions
chmod 600 botserver-stack/conf/vault/init.json

# Consider encrypting or moving off-server
gpg -c init.json
scp init.json.gpg secure-backup-server:
rm init.json
```

### 2. Use Token Policies

Create limited tokens for applications:

```hcl
# gbo-readonly.hcl
path "gbo/*" {
  capabilities = ["read", "list"]
}
```

```bash
vault policy write gbo-readonly gbo-readonly.hcl
vault token create -policy=gbo-readonly -ttl=24h
```

### 3. Enable Audit Logging

```bash
vault audit enable file file_path=/opt/gbo/logs/vault-audit.log
```

### 4. Rotate Secrets Regularly

```bash
# Rotate LLM keys
vault kv put gbo/llm \
  openai_key=sk-new-key \
  anthropic_key=sk-ant-new-key

# BotServer will pick up new keys automatically (cache TTL)
```

### 5. Backup Vault Data

```bash
# Snapshot Vault data
vault operator raft snapshot save backup.snap

# Or backup the data directory
tar -czf vault-backup.tar.gz botserver-stack/data/vault/
```

## No UI Needed

**You don't need to expose a UI for secrets management** because:

1. **Automatic at runtime**: Secrets are fetched automatically
2. **config.csv for changes**: Update bot config, not secrets
3. **Vault UI for emergencies**: Available at `https://localhost:8200/ui`
4. **CLI for automation**: Scripts can manage secrets

### When Admins Need Access

| Situation | Solution |
|-----------|----------|
| Add new LLM provider | `vault kv put gbo/llm new_key=xxx` |
| Rotate compromised key | Update in Vault, services auto-reload |
| Check what's stored | `vault kv get gbo/llm` or Vault UI |
| Debug connection issues | Check Vault logs and service logs |
| Disaster recovery | Use init.json to unseal and recover |

## Relationship Summary

```
┌─────────────────────────────────────────────────────────────────┐
│                           .env                                  │
│              VAULT_ADDR + VAULT_TOKEN (only!)                   │
└─────────────────────────────┬───────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                          Vault                                  │
│    "Give me all service credentials and connection info"        │
│                                                                 │
│  gbo/directory → Zitadel URL, credentials                       │
│  gbo/tables    → Database connection + credentials              │
│  gbo/drive     → MinIO endpoint + credentials                   │
│  gbo/cache     → Redis connection + password                    │
│  gbo/llm       → All LLM API keys                               │
└─────────────────────────────┬───────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                       BotServer                                 │
│         Connects to all services using Vault secrets            │
└─────────────────────────────┬───────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                        User Request                             │
└─────────────────────────────┬───────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                        Zitadel                                  │
│              "Who is this user? Are they allowed?"              │
│              (Credentials from Vault at startup)                │
└─────────────────────────────┬───────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                       config.csv                                │
│              "What LLM should I use? What model?"               │
│              (Non-sensitive bot configuration)                   │
└─────────────────────────────┬───────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      LLM Provider                               │
│              (API key from Vault at startup)                    │
└─────────────────────────────────────────────────────────────────┘
```

## Vault Paths Reference

| Path | Contents |
|------|----------|
| `gbo/directory` | url, project_id, client_id, client_secret |
| `gbo/tables` | host, port, database, username, password |
| `gbo/drive` | endpoint, accesskey, secret |
| `gbo/cache` | host, port, password |
| `gbo/llm` | openai_key, anthropic_key, groq_key, deepseek_key, mistral_key |
| `gbo/encryption` | master_key, data_key |
| `gbo/email` | host, username, password |
| `gbo/meet` | url, api_key, api_secret |
| `gbo/alm` | url, admin_password, runner_token |
| `gbo/vectordb` | url, api_key |
| `gbo/observability` | url, org, bucket, token |

## Next Steps

- [config.csv Format](./config-csv.md) - Bot configuration reference
- [LLM Configuration](./llm-config.md) - LLM-specific settings
- [Infrastructure Design](../chapter-07-gbapp/infrastructure.md) - Full architecture