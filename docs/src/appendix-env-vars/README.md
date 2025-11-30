# Appendix C: Environment Variables

General Bots uses a minimal set of environment variables. All service configuration is managed through the Directory service (Zitadel), and application settings are stored in `config.csv` files within each bot's `.gbot` folder.

## Required Environment Variables

Only one set of environment variables is used by General Bots:

### DIRECTORY_* Variables

**Purpose**: Directory service (Zitadel) configuration for identity and service management.

| Variable | Description | Example |
|----------|-------------|---------|
| `DIRECTORY_URL` | Zitadel instance URL | `http://localhost:8080` |
| `DIRECTORY_CLIENT_ID` | OAuth client ID | Auto-generated during bootstrap |
| `DIRECTORY_CLIENT_SECRET` | OAuth client secret | Auto-generated during bootstrap |

**Example**:
```bash
DIRECTORY_URL=http://localhost:8080
DIRECTORY_CLIENT_ID=your-client-id
DIRECTORY_CLIENT_SECRET=your-client-secret
```

## Auto-Managed Services

The following services are automatically configured through the Directory service:

| Service | Management |
|---------|------------|
| PostgreSQL | Connection managed via Directory |
| S3-Compatible Storage | Credentials managed via Directory |
| Cache (Valkey) | Connection managed via Directory |
| Email (Stalwart) | Accounts managed via Directory |

You do **not** need to set environment variables for these services. The Directory service handles credential distribution and rotation automatically.

## What NOT to Use Environment Variables For

**Do NOT use environment variables for**:

| Configuration | Where to Configure |
|--------------|-------------------|
| Database connection | Managed by Directory service |
| Storage credentials | Managed by Directory service |
| LLM API keys | `config.csv`: `llm-api-key` |
| LLM provider | `config.csv`: `llm-provider` |
| Email settings | Managed by Directory service |
| Channel tokens | `config.csv`: `whatsapp-api-key`, etc. |
| Bot settings | `config.csv`: all bot-specific settings |
| Weather API | `config.csv`: `weather-api-key` |
| Feature flags | `config.csv`: `enable-*` keys |

## Configuration Philosophy

General Bots follows these principles:

1. **Directory-First**: Infrastructure credentials are managed by the Directory service
2. **Minimal Environment**: Only identity provider settings use environment variables
3. **Database-Stored**: All application configuration is stored in the database via `config.csv` sync
4. **Per-Bot Configuration**: Each bot has its own `config.csv` in its `.gbot` folder
5. **No Hardcoded Defaults**: Configuration must be explicitly provided

## Setting Environment Variables

### Linux/macOS

```bash
export DIRECTORY_URL=http://localhost:8080
export DIRECTORY_CLIENT_ID=your-client-id
export DIRECTORY_CLIENT_SECRET=your-client-secret
```

### Systemd Service

```ini
[Service]
Environment="DIRECTORY_URL=http://localhost:8080"
Environment="DIRECTORY_CLIENT_ID=your-client-id"
Environment="DIRECTORY_CLIENT_SECRET=your-client-secret"
```

### LXC Container

When using LXC deployment, environment variables are set in the container configuration:

```bash
lxc config set container-name environment.DIRECTORY_URL="http://localhost:8080"
```

## Security Notes

1. **Never commit credentials**: Use `.env` files (gitignored) or secrets management
2. **Rotate regularly**: The Directory service can rotate credentials automatically
3. **Limit access**: Only the botserver process needs these variables
4. **Use TLS**: Always use HTTPS for the Directory URL in production

## Troubleshooting

### Directory Connection Failed

```
Error: Failed to connect to Directory service
```

Verify:
- `DIRECTORY_URL` is set correctly
- Zitadel is running and accessible
- Network allows connection to Directory host
- Client credentials are valid

### Service Not Available

If a managed service (database, storage, cache) is unavailable:

1. Check the Directory service is running
2. Verify service registration in Zitadel
3. Check service container/process status
4. Review logs for connection errors

## Bootstrap Process

During bootstrap, General Bots:

1. Connects to the Directory service using `DIRECTORY_*` variables
2. Registers itself as an application
3. Retrieves credentials for managed services
4. Starts services with provided credentials
5. Stores service endpoints in the database

This eliminates the need for manual credential management.

## See Also

- [config.csv Format](../chapter-08-config/config-csv.md) - Bot configuration
- [External Services](../appendix-external-services/README.md) - Service configuration
- [Drive Integration](../chapter-08-config/drive.md) - Storage setup
- [Authentication](../chapter-12-auth/README.md) - Directory service integration