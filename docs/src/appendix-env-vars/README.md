# Appendix C: Environment Variables

General Bots uses a minimal set of environment variables. All configuration is managed through `config.csv` files within each bot's `.gbot` folder, with secrets stored securely in Vault.

## Required Environment Variables

Only Vault-related environment variables are used by General Bots:

### VAULT_* Variables

**Purpose**: HashiCorp Vault integration for secure secrets management.

| Variable | Description | Example |
|----------|-------------|---------|
| `VAULT_ADDR` | Vault server URL | `http://localhost:8200` |
| `VAULT_TOKEN` | Authentication token | Auto-generated during bootstrap |
| `VAULT_NAMESPACE` | Vault namespace (optional) | `admin` |

**Example**:
```bash
VAULT_ADDR=http://localhost:8200
VAULT_TOKEN=hvs.your-vault-token
```

## Auto-Managed Services

The following services are automatically configured through Vault:

| Service | Management |
|---------|------------|
| PostgreSQL | Connection credentials in Vault |
| S3-Compatible Storage | Access keys in Vault |
| Cache | Connection managed via Vault |
| Email (Stalwart) | Credentials in Vault |
| LLM API Keys | Stored in Vault |

You do **not** need to set environment variables for these services. Vault handles credential distribution and rotation automatically.

## What NOT to Use Environment Variables For

**All application configuration belongs in `config.csv`**:

| Configuration | Where to Configure |
|--------------|-------------------|
| Database connection | Managed by Vault |
| Storage credentials | Managed by Vault |
| LLM API keys | Managed by Vault |
| LLM provider | `config.csv`: `llm-url` |
| Email settings | `config.csv`: `email-*` |
| Channel tokens | `config.csv`: `whatsapp-*`, etc. |
| Bot settings | `config.csv`: all bot-specific settings |
| Feature flags | `config.csv`: various keys |

## Configuration Philosophy

General Bots follows these principles:

1. **Vault-First**: All secrets are managed by Vault
2. **Minimal Environment**: Only Vault address and token use environment variables
3. **config.csv for Settings**: All application configuration is in `config.csv`
4. **Per-Bot Configuration**: Each bot has its own `config.csv` in its `.gbot` folder
5. **No Hardcoded Secrets**: Never store secrets in code or config files

## Setting Environment Variables

### Linux/macOS

```bash
export VAULT_ADDR=http://localhost:8200
export VAULT_TOKEN=hvs.your-vault-token
```

### Systemd Service

```ini
[Service]
Environment="VAULT_ADDR=http://localhost:8200"
Environment="VAULT_TOKEN=hvs.your-vault-token"
```

### LXC Container

When using LXC deployment, environment variables are set in the container configuration:

```bash
lxc config set container-name environment.VAULT_ADDR="http://localhost:8200"
lxc config set container-name environment.VAULT_TOKEN="hvs.your-vault-token"
```

## Security Notes

1. **Never commit tokens**: Use `.env` files (gitignored) or secrets management
2. **Rotate regularly**: Vault tokens should be rotated periodically
3. **Limit access**: Only the botserver process needs these variables
4. **Use TLS**: Always use HTTPS for Vault in production

## Bootstrap Process

During bootstrap, General Bots:

1. Connects to Vault using `VAULT_*` variables
2. Retrieves credentials for all managed services
3. Configures database, storage, cache, and other services
4. Stores service endpoints securely

This eliminates the need for manual credential management.

## Troubleshooting

### Vault Connection Failed

```
Error: Failed to connect to Vault
```

Verify:
- `VAULT_ADDR` is set correctly
- Vault server is running and accessible
- `VAULT_TOKEN` is valid and not expired
- Network allows connection to Vault host

### Service Not Available

If a managed service (database, storage, cache) is unavailable:

1. Check Vault is running and unsealed
2. Verify secrets exist in Vault
3. Check service container/process status
4. Review logs for connection errors

## See Also

- [config.csv Format](../chapter-08-config/config-csv.md) - Bot configuration
- [Secrets Management](../chapter-08-config/secrets-management.md) - Vault integration details
- [Drive Integration](../chapter-08-config/drive.md) - Storage setup
- [Authentication](../chapter-12-auth/README.md) - Security features