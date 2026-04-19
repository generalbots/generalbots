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
llm-provider,anthropic
llm-model,claude-sonnet-4.5
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
1. botserver starts
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
  url=https://localhost:9000 \
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

## Vault Auto-Unseal Options

When Vault restarts (server reboot, container restart), it starts in a **sealed** state and cannot serve secrets until unsealed. This section covers 4 local options for auto-unseal **without depending on big tech cloud providers**.

### Comparison Table

| Option | Security | Cost | Complexity | Best For |
|--------|----------|------|------------|----------|
| **Secrets File** | ⭐⭐⭐ Medium | Free | Low | Development, Small Production |
| **TPM** | ⭐⭐⭐⭐ High | Free (if hardware has TPM) | Medium | Servers with TPM 2.0 |
| **HSM** | ⭐⭐⭐⭐⭐ Highest | $500-$2000+ | High | Enterprise, Compliance |
| **Transit (2nd Vault)** | ⭐⭐⭐⭐ High | Free | Medium | Multi-server setups |

---

### Option 1: Secrets File (Default)

Store unseal keys in a separate file with restricted permissions. This is the **default** for botserver.

**How it works:**
- Unseal keys stored in `/opt/gbo/secrets/vault-unseal-keys`
- File has `chmod 600` (root only)
- botserver reads this file at startup to auto-unseal
- Keys are never logged or exposed

**Setup:**

```bash
# Create secrets directory
mkdir -p /opt/gbo/secrets
chmod 700 /opt/gbo/secrets

# After vault init, save unseal keys (replace with your actual keys)
cat > /opt/gbo/secrets/vault-unseal-keys << 'EOF'
VAULT_UNSEAL_KEY_1=xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
VAULT_UNSEAL_KEY_2=xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
VAULT_UNSEAL_KEY_3=xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
EOF

chmod 600 /opt/gbo/secrets/vault-unseal-keys
chown root:root /opt/gbo/secrets/vault-unseal-keys
```

**In your .env:**

```env
VAULT_ADDR=http://10.16.164.168:8200
VAULT_TOKEN=<root-token>
VAULT_UNSEAL_KEYS_FILE=/opt/gbo/secrets/vault-unseal-keys
```

**Security considerations:**
- ✅ Separate from `.env` (which might be in git, logs)
- ✅ Only root can read
- ⚠️ Anyone with root access can unseal
- ⚠️ Backup this file securely (encrypted)

---

### Option 2: TPM (Trusted Platform Module)

Use server's TPM hardware chip to store unseal keys. Keys **never leave the hardware**.

**Requirements:**
- TPM 2.0 chip (most modern servers have this)
- Linux with `tpm2-tools` installed

**Check if your server has TPM:**

```bash
# Check for TPM device
ls -la /dev/tpm*

# Install TPM tools
apt install tpm2-tools

# Check TPM status
tpm2_getcap properties-fixed
```

**Setup Vault with TPM seal:**

```hcl
# /opt/gbo/conf/vault/config.hcl
seal "pkcs11" {
  lib            = "/usr/lib/x86_64-linux-gnu/libtpm2_pkcs11.so"
  slot           = "0"
  pin            = "userpin"
  key_label      = "vault-unseal"
  hmac_key_label = "vault-hmac"
}

storage "file" {
  path = "/opt/gbo/data/vault"
}

listener "tcp" {
  address     = "0.0.0.0:8200"
  tls_disable = 1
}
```

**Cost:** Free (hardware already in server)

**Security considerations:**
- ✅ Keys never leave TPM hardware
- ✅ Cannot be extracted even with root access
- ✅ Tied to physical server
- ⚠️ If server dies, keys are lost (need backup strategy)

---

### Option 3: HSM (Hardware Security Module)

Dedicated hardware device for cryptographic operations. **Highest security** for enterprise/compliance.

**Popular HSM Options:**

| Device | Price | Form Factor | Best For |
|--------|-------|-------------|----------|
| **YubiHSM 2** | ~$650 | USB stick | Small business, startups |
| **Nitrokey HSM 2** | ~$109 | USB stick | Budget-conscious |
| **Thales Luna** | $5,000-$20,000 | PCIe/Network | Enterprise |
| **AWS CloudHSM** | ~$1.50/hr | Cloud | Hybrid setups |
| **SoftHSM** | Free | Software | Testing only |

**Setup with YubiHSM 2:**

```bash
# Install YubiHSM connector
apt install yubihsm-connector yubihsm-shell

# Start connector
systemctl enable yubihsm-connector
systemctl start yubihsm-connector
```

```hcl
# /opt/gbo/conf/vault/config.hcl
seal "pkcs11" {
  lib         = "/usr/lib/x86_64-linux-gnu/libyubihsm_pkcs11.so"
  slot        = "0"
  pin         = "0001password"
  key_label   = "vault-unseal-key"
  mechanism   = "0x1085"  # CKM_SHA256_HMAC
}
```

**Setup with Nitrokey HSM 2 (Budget Option):**

```bash
# Install OpenSC
apt install opensc

# Initialize Nitrokey
sc-hsm-tool --initialize --so-pin 3537363231383830 --pin 648219

# Create key for Vault
pkcs11-tool --module /usr/lib/opensc-pkcs11.so --login --pin 648219 \
  --keypairgen --key-type EC:secp256k1 --label vault-key
```

```hcl
# /opt/gbo/conf/vault/config.hcl
seal "pkcs11" {
  lib       = "/usr/lib/opensc-pkcs11.so"
  slot      = "0"
  pin       = "648219"
  key_label = "vault-key"
}
```

**Security considerations:**
- ✅ FIPS 140-2 certified (compliance)
- ✅ Tamper-resistant hardware
- ✅ Keys cannot be extracted
- ✅ Audit logging built-in
- ⚠️ Higher cost
- ⚠️ Physical device management

---

### Option 4: Transit Auto-Unseal (Second Vault)

Use a second "unsealer" Vault instance to unseal the main one. Both can be local.

**Architecture:**

```
┌─────────────────┐      unseals      ┌─────────────────┐
│  Unsealer Vault │ ───────────────► │   Main Vault    │
│  (minimal data) │                   │ (all secrets)   │
│  manual unseal  │                   │  auto-unseal    │
└─────────────────┘                   └─────────────────┘
```

**Setup Unsealer Vault:**

```bash
# Create separate container for unsealer
botserver install vault --container --tenant unsealer

# Initialize unsealer (manual unseal - use Shamir)
lxc exec unsealer-vault -- /opt/gbo/bin/vault operator init \
  -key-shares=5 -key-threshold=3

# Enable transit secrets engine
lxc exec unsealer-vault -- /opt/gbo/bin/vault secrets enable transit

# Create auto-unseal key
lxc exec unsealer-vault -- /opt/gbo/bin/vault write -f transit/keys/autounseal
```

**Configure Main Vault to use Transit:**

```hcl
# /opt/gbo/conf/vault/config.hcl (main vault)
seal "transit" {
  address         = "http://unsealer-vault-ip:8200"
  token           = "unsealer-vault-token"
  disable_renewal = "false"

  key_name   = "autounseal"
  mount_path = "transit/"
}
```

**Security considerations:**
- ✅ No cloud dependency
- ✅ Separation of concerns
- ✅ Unsealer can be on separate network
- ⚠️ Still need to unseal the unsealer manually (or use TPM/HSM for it)
- ⚠️ More infrastructure to manage

---

### Recommendation by Use Case

| Scenario | Recommended Option |
|----------|-------------------|
| **Development/Testing** | Secrets File |
| **Single Server Production** | TPM (if available) or Secrets File |
| **Compliance Required (PCI, HIPAA)** | HSM (YubiHSM 2 or Nitrokey) |
| **Multi-Server Cluster** | Transit Auto-Unseal |
| **Enterprise (budget available)** | Thales Luna HSM |
| **Budget-Conscious Production** | Nitrokey HSM 2 (~$109) |

### Quick Cost Summary

| Solution | One-Time Cost | Monthly Cost |
|----------|---------------|--------------|
| Secrets File | $0 | $0 |
| TPM | $0 (built-in) | $0 |
| Nitrokey HSM 2 | ~$109 | $0 |
| YubiHSM 2 | ~$650 | $0 |
| Thales Luna (Network) | $15,000+ | Support contract |
| AWS CloudHSM | $0 | ~$1,100/month |
| Azure Dedicated HSM | $0 | ~$4,500/month |

**Note:** All 4 options work completely **locally without internet** and **without depending on AWS, Azure, or Google Cloud**. You maintain full control of your keys.

## Migrating from Environment Variables

If you're currently using environment variables:

### Before (Old Way)

```bash
# .env - TOO MANY SECRETS!
DATABASE_URL=postgres://user:password@localhost/db
DIRECTORY_URL=https://localhost:9000
DIRECTORY_CLIENT_ID=your-client-id
DIRECTORY_CLIENT_SECRET=your-client-secret
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
  url=https://localhost:9000 \
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
  url="${DIRECTORY_URL:-https://localhost:9000}" \
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
llm-provider,anthropic
llm-model,claude-sonnet-4.5
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

# botserver will pick up new keys automatically (cache TTL)
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
│                       botserver                                 │
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
- [Infrastructure Design](../02-architecture-packages/infrastructure.md) - Full architecture