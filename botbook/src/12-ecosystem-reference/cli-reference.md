# CLI Reference

botserver provides a command-line interface for managing components, secrets, and services.

> ⚠️ **IMPORTANT**: All container create commands (`botserver install ... --container`) must be run from the **host system**, not from inside a container. The botserver binary manages LXC containers from the host level.

## General Usage

```bash
botserver <command> [options]
```

## Commands Overview

| Command | Description |
|---------|-------------|
| `install` | Install a component |
| `remove` | Remove a component |
| `list` | List all available components |
| `status` | Check component status |
| `start` | Start all installed components |
| `stop` | Stop all components |
| `restart` | Restart all components |
| `vault` | Manage secrets in HashiCorp Vault |
| `rotate-secret` | Rotate credentials for a component |
| `rotate-secrets` | Rotate ALL credentials |
| `version` | Show version information |

## Global Options

| Option | Description |
|--------|-------------|
| `--container` | Use LXC container mode instead of local installation |
| `--tenant <name>` | Specify tenant name (default: "default") |
| `--help`, `-h` | Show help information |
| `--version`, `-v` | Show version |

---

## Component Management

### Install a Component

```bash
botserver install <component> [--container] [--tenant <name>]
```

> ⚠️ **Run from host**: Container install commands must be executed on the host machine, not inside any container.

**Examples:**

```bash
# Install vault locally
botserver install vault

# Install vault in an LXC container with tenant name (run on HOST)
botserver install vault --container --tenant pragmatismo

# Install vector database (run on HOST)
botserver install vector_db --container --tenant pragmatismo
```

**Example: Create Vault and VectorDB containers**

This example shows how to create both Vault (secrets management) and VectorDB (Qdrant for embeddings) containers from scratch:

```bash
# Run these commands on the HOST system, not inside a container

# Step 1: Install Vault container
botserver install vault --container --tenant mycompany

# Step 2: Install VectorDB (Qdrant) container
botserver install vector_db --container --tenant mycompany

# Step 3: Verify containers are running
lxc list | grep mycompany

# Expected output:
# | mycompany-vault     | RUNNING | 10.x.x.x (eth0) | ... |
# | mycompany-vectordb  | RUNNING | 10.x.x.x (eth0) | ... |

# Step 4: Get container IPs for configuration
lxc list mycompany-vault -c n4 --format csv
lxc list mycompany-vectordb -c n4 --format csv

# Step 5: Test Vault health
curl http://<vault-ip>:8200/v1/sys/health

# Step 6: Test VectorDB health
curl http://<vectordb-ip>:6333/health
```

**Available Components:**

| Component | Description |
|-----------|-------------|
| `vault` | HashiCorp Vault - Secrets management |
| `tables` | PostgreSQL - Primary database |
| `cache` | Valkey - Redis-compatible cache |
| `drive` | MinIO - S3-compatible object storage |
| `llm` | llama.cpp - Local LLM server |
| `email` | Stalwart - Mail server |
| `proxy` | Caddy - HTTPS reverse proxy |
| `dns` | CoreDNS - DNS server |
| `directory` | Zitadel - Identity management |
| `alm` | Forgejo - Git repository |
| `alm_ci` | Forgejo Runner - CI/CD |
| `meeting` | LiveKit - Video conferencing |
| `vector_db` | Qdrant - Vector database |
| `timeseries_db` | InfluxDB - Time series database |
| `observability` | Vector - Log aggregation |

### Remove a Component

```bash
botserver remove <component> [--container] [--tenant <name>]
```

### List Components

```bash
botserver list [--container] [--tenant <name>]
```

Shows all available components and their installation status.

### Check Status

```bash
botserver status <component> [--container] [--tenant <name>]
```

---

## Service Control

### Start Services

```bash
botserver start [--container] [--tenant <name>]
```

Starts all installed components.

### Stop Services

```bash
botserver stop
```

Stops all running components.

### Restart Services

```bash
botserver restart [--container] [--tenant <name>]
```

---

## Vault Commands

The `vault` subcommand manages secrets stored in HashiCorp Vault.

### Prerequisites

> ⚠️ **SECURITY WARNING**: Never expose `VAULT_TOKEN` in shell history or scripts.
> Use a secrets file with restricted permissions (600) or environment injection.

Vault commands require these environment variables:

```bash
# Secure method: use a file with restricted permissions
echo "VAULT_TOKEN=<your-vault-token>" > ~/.vault-token
chmod 600 ~/.vault-token
source ~/.vault-token

export VAULT_ADDR=http://<vault-ip>:8200
```

### Migrate Secrets from .env

Migrates secrets from an existing `.env` file to Vault.

```bash
botserver vault migrate [env_file]
```

**Arguments:**

| Argument | Description | Default |
|----------|-------------|---------|
| `env_file` | Path to .env file | `.env` |

**Example:**

```bash
# Migrate from default .env
botserver vault migrate

# Migrate from specific file
botserver vault migrate /opt/gbo/bin/system/.env
```

**Migrated Secret Paths:**

| .env Variables | Vault Path |
|----------------|------------|
| `TABLES_*` | `gbo/tables` |
| `CUSTOM_*` | `gbo/custom` |
| `DRIVE_*` | `gbo/drive` |
| `EMAIL_*` | `gbo/email` |
| `STRIPE_*` | `gbo/stripe` |
| `AI_*`, `LLM_*` | `gbo/llm` |

After migration, your `.env` file only needs:

```env
RUST_LOG=info
VAULT_ADDR=http://<vault-ip>:8200
VAULT_TOKEN=<vault-token>
SERVER_HOST=0.0.0.0
SERVER_PORT=5858
```

### Store Secrets

Store key-value pairs at a Vault path.

```bash
botserver vault put <path> <key=value> [key=value...]
```

**Examples:**

```bash
# Store database credentials
botserver vault put gbo/tables host=localhost port=5432 username=postgres password=secret

# Store email configuration
botserver vault put gbo/email server=mail.example.com user=admin password=secret

# Store API keys
botserver vault put gbo/llm api_key=sk-xxx endpoint=https://api.openai.com
```

### Retrieve Secrets

Get secrets from a Vault path.

```bash
botserver vault get <path> [key]
```

**Examples:**

```bash
# Get all secrets at a path (values are masked)
botserver vault get gbo/tables

# Get a specific key value
botserver vault get gbo/tables password

# Get drive credentials
botserver vault get gbo/drive
```

**Output:**

```
Secrets at gbo/tables:
  host=localhost
  port=5432
  database=botserver
  username=gbuser
  password=67a6...
```

> **Note:** Sensitive values (password, secret, key, token) are automatically masked in output.

### List Secret Paths

Shows all configured secret paths.

```bash
botserver vault list
```

**Output:**

```
Configured secret paths:
  gbo/tables           - Database credentials
  gbo/drive            - S3/MinIO credentials
  gbo/cache            - Redis credentials
  gbo/email            - SMTP credentials
  gbo/directory        - Zitadel credentials
  gbo/llm              - AI API keys
  gbo/encryption       - Encryption keys
  gbo/meet             - LiveKit credentials
  gbo/alm              - Forgejo credentials
  gbo/vectordb         - Qdrant credentials
  gbo/observability    - InfluxDB credentials
  gbo/stripe           - Payment credentials
  gbo/custom           - Custom database
```

### Health Check

Check Vault connection status.

```bash
botserver vault health
```

**Output (success):**

```
* Vault is healthy
  Address: http://10.16.164.100:8200
```

**Output (failure):**

```
x Vault not configured
  Set VAULT_ADDR and VAULT_TOKEN environment variables
```

---

## Version Information

Show botserver version and component status.

```bash
botserver version [--all]
```

**Options:**

| Option | Description |
|--------|-------------|
| `--all` | Show detailed info: build, components, Vault status |

**Examples:**

```bash
# Simple version
botserver version
# Output: botserver 6.1.0

# Detailed version with all components
botserver version --all
```

**Output with --all:**

```
botserver 6.1.0

Build Information:
  rustc: rustc 1.83.0 (90b35a623 2024-11-26)
  target: x86_64
  os: linux

Installed Components:
  * vault (installed)
  * tables (installed)
  * cache (installed)

Available Components: 15

Secrets:
  Vault: connected
```

---

## Secret Rotation

Rotate credentials for security compliance and breach response.

> 🔒 **SECURITY**: Regular credential rotation is a security best practice.
> Recommended rotation schedule:
> - **Production**: Every 90 days
> - **After employee departure**: Immediately
> - **After security incident**: Immediately

### Rotate Single Component

```bash
botserver rotate-secret <component>
```

**Available Components:**

| Component | What Gets Rotated |
|-----------|-------------------|
| `tables` | PostgreSQL password |
| `drive` | MinIO access key and secret |
| `cache` | Valkey/Redis password |
| `email` | SMTP password |
| `directory` | Zitadel client secret |
| `encryption` | Master encryption key (⚠️ dangerous) |
| `jwt` | JWT signing secret (⚠️ invalidates refresh tokens) |

**Examples:**

```bash
# Rotate database password
botserver rotate-secret tables

# Output:
# ⚠️  WARNING: You must update PostgreSQL with the new password!
#
# Run this SQL command:
#   ALTER USER postgres WITH PASSWORD 'NewP@ssw0rd...';
#
# Old password: 67a6...
# New password: Xk9m...
# Save to Vault? [y/N]: y
# ✓ Credentials saved to Vault

# Rotate S3/MinIO credentials
botserver rotate-secret drive

# Rotate Redis password
botserver rotate-secret cache
```

> ⚠️ **WARNING**: After rotating, you MUST manually update the service with the new credentials before restarting botserver.

### Rotate All Secrets

Rotate all credentials at once. Use for security incidents or compliance requirements.

```bash
botserver rotate-secrets --all
```

**Output:**

```
🔐 ROTATING ALL SECRETS
========================

⚠️  CRITICAL WARNING!
This will generate new credentials for ALL components.
You MUST update each service manually after rotation.

Type 'ROTATE ALL' to continue: ROTATE ALL

Generating new credentials...

✓ tables: ALTER USER postgres WITH PASSWORD 'Xk9mP@ss...';
✓ drive: mc admin user add myminio AKIAEXAMPLE... secretkey...
✓ cache: redis-cli CONFIG SET requirepass 'NewRedisP@ss...'
✓ email: new password = SmtpP@ss...
✓ directory: new client_secret = ZitadelSecret...

========================
✓ All secrets rotated and saved to Vault

⚠️  IMPORTANT: Run the commands above to update each service!
⚠️  Then restart botserver: botserver restart
```

### Post-Rotation Checklist

After rotating secrets, follow this checklist:

```bash
# 1. Update PostgreSQL
lxc exec pragmatismo-tables -- psql -U postgres -c "ALTER USER postgres WITH PASSWORD 'new-password';"

# 2. Update MinIO (create new user, migrate data, delete old)
lxc exec pragmatismo-drive -- mc admin user add local newkey newsecret
lxc exec pragmatismo-drive -- mc admin policy attach local readwrite --user newkey

# 3. Update Valkey/Redis
lxc exec pragmatismo-cache -- redis-cli CONFIG SET requirepass 'new-password'
lxc exec pragmatismo-cache -- redis-cli CONFIG REWRITE

# 4. Update Zitadel (via admin console)
# Navigate to: Settings > OAuth > Applications > Update Secret

# 5. Restart botserver
botserver restart

# 6. Verify all services
botserver version --all
```

> 🔒 **ENCRYPTION KEY WARNING**: Rotating the encryption key (`botserver rotate-secret encryption`) will make ALL existing encrypted data unreadable. Only do this if you have re-encryption procedures in place.

---

## Security Considerations

### Current Limitations

⚠️ **Manual Service Updates Required**
After rotating credentials, you MUST manually update each service:

- **Database (tables):** Run the provided SQL command to update PostgreSQL user password
- **Drive (MinIO):** Run the provided `mc admin` commands to update S3 credentials
- **Cache (Redis):** Run the provided `redis-cli` command to update password
- **Directory (Zitadel):** Update client secret via admin console

⚠️ **Service Restart Required**
After rotating **JWT secret**, you MUST restart botserver:
```bash
botserver restart
```

All users will need to re-login (refresh tokens invalidated). Access tokens (15-minute expiry) will expire naturally.

⚠️ **No Automatic Rollback**
If verification fails, you must manually restore from backups:
```bash
# Database: Re-run SQL with old password
# JWT: Restore .env.backup.<timestamp>
# Other: Use backup values shown in rotation output
```

### Available Components for Rotation

| Component | Credential Type | Manual Update Required | Service Restart |
|-----------|----------------|------------------------|-----------------|
| `tables` | PostgreSQL password | ✅ Run SQL command | ❌ No |
| `drive` | MinIO S3 credentials | ✅ Run mc commands | ❌ No |
| `cache` | Redis/Valkey password | ✅ Run redis-cli | ❌ No |
| `email` | SMTP password | ✅ Update mail server | ❌ No |
| `directory` | Zitadel client secret | ✅ Update via console | ❌ No |
| `encryption` | Master encryption key | ⚠️ Re-encrypt all data | ❌ No |
| `jwt` | JWT signing secret | ❌ No | ✅ **Yes** |

### Best Practices

1. **Test in staging first** - Never rotate in production without testing
2. **Schedule during low traffic** - Rotate JWT outside peak hours
3. **Have rollback plan ready** - Save backup paths shown during rotation
4. **Monitor logs** - Check for authentication failures after rotation:
   ```bash
   tail -f /var/log/botserver/app.log | grep -i "authentication\\|jwt\\|token"
   ```
5. **Rotate regularly** - Every 90 days for production, per security compliance
6. **After JWT rotation** - Verify all services are healthy before declaring success

### Verification

The `rotate-secret` command includes automatic verification where possible:

- **Database:** Tests PostgreSQL connection with new credentials
- **JWT:** Checks health endpoint (requires service to be running)
- **Other:** Displays manual verification instructions

If verification fails:
1. Check the error message for specific failure details
2. Restore from backup if needed
3. Re-run rotation after fixing the issue

---

## Complete Setup Example

Here's a complete workflow to set up Vault and migrate secrets.

> ⚠️ **Run all commands on the HOST system**, not inside any container.

```bash
# 1. Install Vault in a container (run on HOST)
botserver install vault --container --tenant pragmatismo

# 2. Install Vector DB for embeddings (run on HOST)
botserver install vector_db --container --tenant pragmatismo

# 3. Get Vault container IP
lxc list pragmatismo-vault

# 4. Set environment variables
export VAULT_ADDR=http://<vault-ip>:8200
export VAULT_TOKEN=<root-token-from-init>

# 5. Migrate existing secrets
botserver vault migrate /opt/gbo/bin/system/.env

# 6. Verify migration
botserver vault health
botserver vault get gbo/tables
botserver vault get gbo/drive
botserver vault get gbo/email

# 7. Update .env to use Vault only (SECURE METHOD)
cat > /opt/gbo/bin/system/.env << EOF
RUST_LOG=info
VAULT_ADDR=http://<vault-ip>:8200
SERVER_HOST=0.0.0.0
SERVER_PORT=5858
EOF

# Store token separately with restricted permissions
echo "VAULT_TOKEN=<root-token>" > /opt/gbo/secrets/vault-token
chmod 600 /opt/gbo/secrets/vault-token
chown root:root /opt/gbo/secrets/vault-token

# 8. Restart botserver
botserver restart
```

---

## Secret Paths Reference

### gbo/tables

Database connection credentials.

| Key | Description |
|-----|-------------|
| `host` | Database server hostname |
| `port` | Database port |
| `database` | Database name |
| `username` | Database user |
| `password` | Database password |

### gbo/drive

S3/MinIO storage credentials.

| Key | Description |
|-----|-------------|
| `server` | Storage server hostname |
| `port` | Storage port |
| `use_ssl` | Enable SSL (`true`/`false`) |
| `accesskey` | Access key ID |
| `secret` | Secret access key |
| `org_prefix` | Organization prefix for buckets |

### gbo/email

SMTP email configuration.

| Key | Description |
|-----|-------------|
| `from` | Sender email address |
| `server` | SMTP server hostname |
| `port` | SMTP port |
| `username` | SMTP username |
| `password` | SMTP password |
| `reject_unauthorized` | Reject invalid certs |

### gbo/llm

AI/LLM configuration.

| Key | Description |
|-----|-------------|
| `api_key` | API key for cloud LLM |
| `model` | Model identifier |
| `endpoint` | API endpoint URL |
| `local` | Use local LLM (`true`/`false`) |
| `url` | Local LLM server URL |
| `model_path` | Path to local model file |
| `embedding_model_path` | Path to embedding model |
| `embedding_url` | Embedding server URL |

### gbo/stripe

Payment processing credentials.

| Key | Description |
|-----|-------------|
| `secret_key` | Stripe secret key |
| `professional_plan_price_id` | Professional plan price ID |
| `personal_plan_price_id` | Personal plan price ID |

### gbo/cache

Redis/Valkey credentials.

| Key | Description |
|-----|-------------|
| `password` | Cache password |

### gbo/directory

Zitadel identity provider.

| Key | Description |
|-----|-------------|
| `url` | Zitadel server URL |
| `project_id` | Project ID |
| `client_id` | OAuth client ID |
| `client_secret` | OAuth client secret |
| `masterkey` | Master encryption key |

### gbo/encryption

Encryption keys.

| Key | Description |
|-----|-------------|
| `master_key` | Master encryption key |

---

## Troubleshooting

### Vault Connection Issues

```bash
# Check if Vault is running
lxc exec pragmatismo-vault -- systemctl status vault

# Check Vault seal status
lxc exec pragmatismo-vault -- vault status

# Unseal Vault if sealed
lxc exec pragmatismo-vault -- vault operator unseal <unseal-key>
```

### Component Installation Fails

```bash
# Check logs
tail -f botserver-stack/logs/<component>.log

# Verify container exists
lxc list | grep <tenant>-<component>

# Check container logs
lxc exec <tenant>-<component> -- journalctl -xe
```

### Missing Dependencies

If you see errors like `error while loading shared libraries: libpq.so.5`, install the runtime dependencies:

```bash
# Quick install (recommended) - run on HOST system
curl -fsSL https://raw.githubusercontent.com/GeneralBots/botserver/main/scripts/install-dependencies.sh | sudo bash

# Or manual install (Debian/Ubuntu)
sudo apt-get install -y libpq5 libssl3 liblzma5 zlib1g ca-certificates curl wget

# Or manual install (Fedora/RHEL)
sudo dnf install -y libpq openssl-libs xz-libs zlib ca-certificates curl wget
```

For development/building from source:

```bash
# Install development dependencies
sudo apt-get install -y libpq-dev libssl-dev liblzma-dev
```

---

## Security Best Practices

> 🛡️ **SECURITY HARDENING GUIDE**

> 🔒 **SECURITY NOTES**

### Token Management

- **NEVER** commit tokens or secrets to version control
- **NEVER** pass tokens as command-line arguments (visible in `ps`)
- **NEVER** store tokens in shell history (use `HISTCONTROL=ignorespace`)
- **ALWAYS** use environment variables or secure files with `chmod 600`
- **ROTATE** Vault tokens regularly (recommended: every 30 days)
- **ROTATE** service credentials regularly (recommended: every 90 days)

```bash
# Prevent command from being saved in history (note the leading space)
 export VAULT_TOKEN=s.xxxx
```

### File Permissions

```bash
# Secure your secrets directory
chmod 700 /opt/gbo/secrets
chmod 600 /opt/gbo/secrets/*
chown -R root:root /opt/gbo/secrets
```

### Vault Hardening

```bash
# Enable audit logging
botserver vault put gbo/audit enabled=true

# Use short-lived tokens in production
# Configure token TTL in Vault policies
```

### Network Security

- Run Vault behind a firewall
- Use TLS for Vault connections in production
- Restrict Vault access to specific container IPs

```bash
# Example: Only allow botserver container to reach Vault
iptables -A INPUT -p tcp --dport 8200 -s 10.16.164.33 -j ACCEPT
iptables -A INPUT -p tcp --dport 8200 -j DROP
```

### Credential Rotation Schedule

| Component | Rotation Frequency | Command |
|-----------|-------------------|---------|
| Vault Token | 30 days | Vault UI or API |
| Database | 90 days | `botserver rotate-secret tables` |
| S3/MinIO | 90 days | `botserver rotate-secret drive` |
| Redis | 90 days | `botserver rotate-secret cache` |
| Email | 90 days | `botserver rotate-secret email` |
| All at once | After incident | `botserver rotate-secrets --all` |

### Incident Response

If you suspect a credential breach:

```bash
# 1. Immediately rotate ALL secrets
botserver rotate-secrets --all

# 2. Update all services with new credentials (see output)

# 3. Restart all services
botserver restart

# 4. Check for unauthorized access in logs
grep -r "authentication failed" /opt/gbo/logs/

# 5. Review Vault audit logs
vault audit list
```
