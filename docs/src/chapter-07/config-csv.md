# config.csv Format

The `config.csv` file is the central configuration for each bot instance. Located in the `.gbot` package directory, it controls all bot behavior, integrations, and system settings.

## File Location

```
mybot.gbai/
└── mybot.gbot/
    └── config.csv
```

## Format

Configuration uses simple CSV format with two columns: `key` and `value`.

```csv
key,value
botId,00000000-0000-0000-0000-000000000000
title,My Bot Name
description,Bot description here
```

## Core Settings

### Bot Identity

| Key | Description | Default | Example |
|-----|-------------|---------|---------|
| `botId` | Unique bot identifier (UUID) | Generated | `00000000-0000-0000-0000-000000000000` |
| `title` | Bot display name | Required | `Customer Support Bot` |
| `description` | Bot description | Empty | `Handles customer inquiries` |
| `logoUrl` | Bot avatar/logo URL | Empty | `https://example.com/logo.png` |
| `welcomeMessage` | Initial greeting | Empty | `Hello! How can I help you today?` |

### LLM Configuration

| Key | Description | Default | Example |
|-----|-------------|---------|---------|
| `llmModel` | Model to use | `gpt-4` | `gpt-4`, `claude-3`, `llama-3` |
| `llmApiKey` | API key for LLM service | Required | `sk-...` |
| `llmEndpoint` | Custom LLM endpoint | Provider default | `https://api.openai.com/v1` |
| `llmTemperature` | Response creativity (0-1) | `0.7` | `0.3` for factual, `0.9` for creative |
| `llmMaxTokens` | Max response length | `2000` | `4000` |
| `answerMode` | Response strategy | `default` | `simple`, `detailed`, `technical` |

### Knowledge Base

| Key | Description | Default | Example |
|-----|-------------|---------|---------|
| `qdrantUrl` | Vector database URL | `http://localhost:6333` | `http://qdrant:6333` |
| `qdrantApiKey` | Qdrant API key | Empty | `your-api-key` |
| `embeddingModel` | Model for embeddings | `text-embedding-ada-002` | `all-MiniLM-L6-v2` |
| `chunkSize` | Text chunk size | `1000` | `500` |
| `chunkOverlap` | Overlap between chunks | `200` | `100` |
| `topK` | Number of search results | `5` | `10` |

### Storage Configuration

| Key | Description | Default | Example |
|-----|-------------|---------|---------|
| `minioEndpoint` | MinIO/S3 endpoint | `localhost:9000` | `minio.example.com` |
| `minioAccessKey` | Storage access key | Required | `minioadmin` |
| `minioSecretKey` | Storage secret key | Required | `minioadmin` |
| `minioBucket` | Default bucket name | `botserver` | `my-bot-files` |
| `minioUseSsl` | Use HTTPS for MinIO | `false` | `true` |

### Database

| Key | Description | Default | Example |
|-----|-------------|---------|---------|
| `databaseUrl` | PostgreSQL connection | Required | `postgresql://user:pass@localhost/botdb` |
| `maxConnections` | Connection pool size | `10` | `25` |
| `connectionTimeout` | Timeout in seconds | `30` | `60` |

### Email Integration

| Key | Description | Default | Example |
|-----|-------------|---------|---------|
| `smtpHost` | SMTP server | Empty | `smtp.gmail.com` |
| `smtpPort` | SMTP port | `587` | `465` |
| `smtpUser` | Email username | Empty | `bot@example.com` |
| `smtpPassword` | Email password | Empty | `app-specific-password` |
| `smtpFrom` | From address | Empty | `noreply@example.com` |
| `smtpUseTls` | Use TLS | `true` | `false` |

### Calendar Integration

| Key | Description | Default | Example |
|-----|-------------|---------|---------|
| `calendarEnabled` | Enable calendar features | `false` | `true` |
| `calendarProvider` | Calendar service | `google` | `microsoft`, `caldav` |
| `calendarApiKey` | Calendar API key | Empty | `your-api-key` |
| `workingHoursStart` | Business hours start | `09:00` | `08:30` |
| `workingHoursEnd` | Business hours end | `17:00` | `18:00` |
| `timezone` | Default timezone | `UTC` | `America/New_York` |

### Authentication

| Key | Description | Default | Example |
|-----|-------------|---------|---------|
| `authEnabled` | Require authentication | `false` | `true` |
| `authProvider` | Auth provider | `local` | `oauth`, `saml`, `ldap` |
| `authClientId` | OAuth client ID | Empty | `client-id` |
| `authClientSecret` | OAuth secret | Empty | `client-secret` |
| `authCallbackUrl` | OAuth callback | Empty | `https://bot.example.com/auth/callback` |
| `jwtSecret` | JWT signing secret | Generated | `your-secret-key` |
| `sessionTimeout` | Session duration (min) | `1440` | `60` |

### Channel Configuration

| Key | Description | Default | Example |
|-----|-------------|---------|---------|
| `webEnabled` | Enable web interface | `true` | `false` |
| `whatsappEnabled` | Enable WhatsApp | `false` | `true` |
| `whatsappToken` | WhatsApp API token | Empty | `EAAI...` |
| `whatsappPhoneId` | WhatsApp phone ID | Empty | `123456789` |
| `teamsEnabled` | Enable MS Teams | `false` | `true` |
| `teamsAppId` | Teams app ID | Empty | `app-id` |
| `teamsAppPassword` | Teams app password | Empty | `app-password` |
| `slackEnabled` | Enable Slack | `false` | `true` |
| `slackToken` | Slack bot token | Empty | `xoxb-...` |

### Security

| Key | Description | Default | Example |
|-----|-------------|---------|---------|
| `corsOrigins` | Allowed CORS origins | `*` | `https://example.com` |
| `rateLimitPerMinute` | API rate limit | `60` | `100` |
| `maxFileSize` | Max upload size (MB) | `10` | `50` |
| `allowedFileTypes` | Permitted file types | `pdf,doc,txt` | `*` |
| `encryptionKey` | Data encryption key | Generated | `base64-key` |
| `requireHttps` | Force HTTPS | `false` | `true` |

### Monitoring

| Key | Description | Default | Example |
|-----|-------------|---------|---------|
| `metricsEnabled` | Enable metrics | `false` | `true` |
| `metricsEndpoint` | Metrics endpoint | `/metrics` | `/admin/metrics` |
| `loggingLevel` | Log level | `info` | `debug`, `warn`, `error` |
| `logToFile` | Log to file | `false` | `true` |
| `logFilePath` | Log file location | `./logs` | `/var/log/botserver` |
| `sentryDsn` | Sentry error tracking | Empty | `https://...@sentry.io/...` |

### Advanced Features

| Key | Description | Default | Example |
|-----|-------------|---------|---------|
| `webAutomationEnabled` | Enable web scraping | `false` | `true` |
| `ocrEnabled` | Enable OCR | `false` | `true` |
| `speechEnabled` | Enable speech | `false` | `true` |
| `translationEnabled` | Enable translation | `false` | `true` |
| `cacheEnabled` | Enable Redis cache | `false` | `true` |
| `cacheUrl` | Redis URL | `redis://localhost:6379` | `redis://cache:6379` |

## Environment Variable Override

Any config value can be overridden using environment variables:

```bash
# Override LLM model
export BOT_LLM_MODEL=gpt-4-turbo

# Override database URL
export BOT_DATABASE_URL=postgresql://prod@db/botserver
```

## Multiple Bots Configuration

Each bot has its own `config.csv`. The system loads all bot configurations on startup:

```
templates/
├── support.gbai/
│   └── support.gbot/
│       └── config.csv    # Support bot config
├── sales.gbai/
│   └── sales.gbot/
│       └── config.csv    # Sales bot config
└── default.gbai/
    └── default.gbot/
        └── config.csv    # Default bot config
```

## Configuration Validation

The system validates configuration on startup:
- Required fields must be present
- UUIDs must be valid format
- URLs must be reachable
- API keys are tested
- File paths must exist

## Hot Reload

Changes to `config.csv` can be reloaded without restart:
1. Edit the file
2. Call `/api/admin/reload-config` endpoint
3. Or use the admin UI reload button

## Security Best Practices

1. **Never commit API keys** - Use environment variables
2. **Encrypt sensitive values** - Use `encryptionKey` setting
3. **Rotate credentials regularly** - Update keys monthly
4. **Use strong JWT secrets** - At least 32 characters
5. **Restrict CORS origins** - Don't use `*` in production
6. **Enable HTTPS** - Set `requireHttps=true`
7. **Set rate limits** - Prevent abuse
8. **Monitor access** - Enable logging and metrics

## Troubleshooting

### Bot Won't Start
- Check required fields are set
- Verify database connection
- Ensure bot ID is unique

### LLM Not Responding
- Verify API key is valid
- Check endpoint URL
- Test rate limits

### Storage Issues
- Verify MinIO is running
- Check access credentials
- Test bucket permissions

### Authentication Problems
- Verify JWT secret matches
- Check session timeout
- Test OAuth callback URL

## Example Configuration

Complete example for a production bot:

```csv
key,value
botId,a1b2c3d4-e5f6-7890-abcd-ef1234567890
title,Customer Support Assistant
description,24/7 automated customer support
welcomeMessage,Hello! I'm here to help with any questions.
llmModel,gpt-4
llmApiKey,${LLM_API_KEY}
llmTemperature,0.3
answerMode,detailed
databaseUrl,${DATABASE_URL}
minioEndpoint,storage.example.com
minioAccessKey,${MINIO_ACCESS}
minioSecretKey,${MINIO_SECRET}
minioBucket,support-bot
minioUseSsl,true
authEnabled,true
authProvider,oauth
authClientId,${OAUTH_CLIENT_ID}
authClientSecret,${OAUTH_CLIENT_SECRET}
corsOrigins,https://app.example.com
requireHttps,true
loggingLevel,info
metricsEnabled,true
cacheEnabled,true
cacheUrl,redis://cache:6379
```
