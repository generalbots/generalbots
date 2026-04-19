# Service Catalog

This catalog provides detailed information about every external service that General Bots integrates with.

## LLM Providers

### OpenAI

| Property | Value |
|----------|-------|
| **Service URL** | `https://api.openai.com/v1` |
| **Config Key** | `llm-provider=openai` |
| **API Key Config** | `llm-api-key` (stored in Vault) |
| **Documentation** | [platform.openai.com/docs](https://platform.openai.com/docs) |
| **BASIC Keywords** | `LLM` |
| **Supported Models** | `gpt-5`, `gpt-oss-120b`, `gpt-oss-20b` |

### Groq

| Property | Value |
|----------|-------|
| **Service URL** | `https://api.groq.com/openai/v1` |
| **Config Key** | `llm-provider=groq` |
| **API Key Config** | `llm-api-key` (stored in Vault) |
| **Documentation** | [console.groq.com/docs](https://console.groq.com/docs) |
| **BASIC Keywords** | `LLM` |
| **Supported Models** | `llama-4-scout`, `llama-4-maverick`, `qwen3`, `mixtral-8x22b` |

### Anthropic

| Property | Value |
|----------|-------|
| **Service URL** | `https://api.anthropic.com/v1` |
| **Config Key** | `llm-provider=anthropic` |
| **API Key Config** | `llm-api-key` (stored in Vault) |
| **Documentation** | [docs.anthropic.com](https://docs.anthropic.com) |
| **BASIC Keywords** | `LLM` |
| **Supported Models** | `claude-opus-4.5`, `claude-sonnet-4.5` |

### Azure OpenAI

| Property | Value |
|----------|-------|
| **Service URL** | `https://{resource}.openai.azure.com/` |
| **Config Key** | `llm-provider=azure` |
| **API Key Config** | `llm-api-key` (stored in Vault) |
| **Documentation** | [learn.microsoft.com/azure/ai-services/openai](https://learn.microsoft.com/azure/ai-services/openai) |
| **BASIC Keywords** | `LLM` |

### Google (Gemini)

| Property | Value |
|----------|-------|
| **Service URL** | `https://generativelanguage.googleapis.com/v1` |
| **Config Key** | `llm-provider=google` |
| **API Key Config** | `llm-api-key` (stored in Vault) |
| **Documentation** | [ai.google.dev/docs](https://ai.google.dev/docs) |
| **BASIC Keywords** | `LLM` |
| **Supported Models** | `gemini-3-pro`, `gemini-2.5-pro`, `gemini-2.5-flash` |

### xAI (Grok)

| Property | Value |
|----------|-------|
| **Service URL** | `https://api.x.ai/v1` |
| **Config Key** | `llm-provider=xai` |
| **API Key Config** | `llm-api-key` (stored in Vault) |
| **Documentation** | [docs.x.ai](https://docs.x.ai) |
| **BASIC Keywords** | `LLM` |
| **Supported Models** | `grok-4` |

### DeepSeek

| Property | Value |
|----------|-------|
| **Service URL** | `https://api.deepseek.com/v1` |
| **Config Key** | `llm-provider=deepseek` |
| **API Key Config** | `llm-api-key` (stored in Vault) |
| **Documentation** | [platform.deepseek.com/docs](https://platform.deepseek.com/docs) |
| **BASIC Keywords** | `LLM` |
| **Supported Models** | `deepseek-v3.1`, `deepseek-r3` |

### Mistral AI

| Property | Value |
|----------|-------|
| **Service URL** | `https://api.mistral.ai/v1` |
| **Config Key** | `llm-provider=mistral` |
| **API Key Config** | `llm-api-key` (stored in Vault) |
| **Documentation** | [docs.mistral.ai](https://docs.mistral.ai) |
| **BASIC Keywords** | `LLM` |
| **Supported Models** | `mixtral-8x22b` |

---

## Weather Services

### OpenWeatherMap

| Property | Value |
|----------|-------|
| **Service URL** | `https://api.openweathermap.org/data/2.5` |
| **Config Key** | `weather-api-key` |
| **Documentation** | [openweathermap.org/api](https://openweathermap.org/api) |
| **BASIC Keywords** | `WEATHER` |
| **Free Tier** | 1,000 calls/day |
| **Required Plan** | Free or higher |

**Example Usage:**
```basic
weather = WEATHER "Seattle"
TALK weather
```

---

## Messaging Channels

### WhatsApp Business API

| Property | Value |
|----------|-------|
| **Service URL** | `https://graph.facebook.com/v17.0` |
| **Config Keys** | `whatsapp-api-key`, `whatsapp-phone-number-id`, `whatsapp-business-account-id` |
| **Documentation** | [developers.facebook.com/docs/whatsapp](https://developers.facebook.com/docs/whatsapp) |
| **BASIC Keywords** | `SEND WHATSAPP`, `SEND FILE` (WhatsApp) |
| **Webhook URL** | `/api/channels/whatsapp/webhook` |

### Microsoft Teams

| Property | Value |
|----------|-------|
| **Service URL** | `https://smba.trafficmanager.net/apis` |
| **Config Keys** | `teams-app-id`, `teams-app-password`, `teams-tenant-id` |
| **Documentation** | [learn.microsoft.com/microsoftteams/platform](https://learn.microsoft.com/microsoftteams/platform) |
| **BASIC Keywords** | `SEND TEAMS`, `SEND FILE` (Teams) |
| **Webhook URL** | `/api/channels/teams/messages` |

### Instagram Messaging

| Property | Value |
|----------|-------|
| **Service URL** | `https://graph.facebook.com/v17.0` |
| **Config Keys** | `instagram-access-token`, `instagram-page-id`, `instagram-account-id` |
| **Documentation** | [developers.facebook.com/docs/instagram-api](https://developers.facebook.com/docs/instagram-api) |
| **BASIC Keywords** | `SEND INSTAGRAM` |
| **Webhook URL** | `/api/channels/instagram/webhook` |

### Telegram

| Property | Value |
|----------|-------|
| **Service URL** | `https://api.telegram.org/bot{token}` |
| **Config Keys** | `telegram-bot-token` |
| **Documentation** | [core.telegram.org/bots/api](https://core.telegram.org/bots/api) |
| **BASIC Keywords** | `SEND TELEGRAM` |
| **Webhook URL** | `/api/channels/telegram/webhook` |

---

## Storage Services

### S3-Compatible Storage

General Bots uses S3-compatible object storage. Configuration is **automatically managed** by the Directory service (Zitadel).

| Property | Value |
|----------|-------|
| **Local Default** | MinIO on port 9000 |
| **Management** | Directory service (automatic) |
| **Console Port** | 9001 (when using MinIO) |
| **BASIC Keywords** | `GET` (file retrieval) |

**Compatible Services:**
- MinIO (default local installation)
- Backblaze B2
- Wasabi
- DigitalOcean Spaces
- Cloudflare R2
- Any S3-compatible provider

Storage credentials are provisioned and rotated automatically by the Directory service. No manual configuration required.

---

## Directory Services

### Zitadel (Identity Provider)

| Property | Value |
|----------|-------|
| **Local Default** | Port 8080 |
| **Environment Variables** | `DIRECTORY_URL`, `DIRECTORY_CLIENT_ID`, `DIRECTORY_CLIENT_SECRET` |
| **Documentation** | [zitadel.com/docs](https://zitadel.com/docs) |
| **Purpose** | User authentication, SSO, OAuth2/OIDC, service credential management |

The Directory service manages:
- User authentication
- Service credentials (database, storage, cache)
- OAuth applications
- Role-based access control

---

## Email Services

### Stalwart Mail Server

| Property | Value |
|----------|-------|
| **Ports** | 25 (SMTP), 993 (IMAPS), 587 (Submission) |
| **Management** | Directory service (automatic) |
| **Documentation** | [stalw.art/docs](https://stalw.art/docs) |
| **BASIC Keywords** | `SEND MAIL` |

Email accounts are created and managed through the Directory service.

### External IMAP/SMTP

| Property | Value |
|----------|-------|
| **Config Keys** | `smtp-server`, `smtp-port`, `imap-server`, `imap-port`, `email-username`, `email-password` |
| **BASIC Keywords** | `SEND MAIL` |
| **Supported Providers** | Gmail, Outlook, custom SMTP/IMAP |

**Gmail Configuration Example (in config.csv):**
```csv
smtp-server,smtp.gmail.com
smtp-port,587
imap-server,imap.gmail.com
imap-port,993
```

---

## Local Services (BotModels)

### Image Generation

| Property | Value |
|----------|-------|
| **Service URL** | `http://localhost:5000` (default) |
| **Config Keys** | `botmodels-enabled`, `botmodels-url` |
| **BASIC Keywords** | `IMAGE` |
| **Requires** | BotModels service running |

### Video Generation

| Property | Value |
|----------|-------|
| **Service URL** | `http://localhost:5000` (default) |
| **Config Keys** | `botmodels-enabled`, `botmodels-url` |
| **BASIC Keywords** | `VIDEO` |
| **Requires** | BotModels service running, GPU recommended |

### Audio Generation (TTS)

| Property | Value |
|----------|-------|
| **Service URL** | `http://localhost:5000` (default) |
| **Config Keys** | `botmodels-enabled`, `botmodels-url` |
| **BASIC Keywords** | `AUDIO` |
| **Requires** | BotModels service running |

### Vision/Captioning

| Property | Value |
|----------|-------|
| **Service URL** | `http://localhost:5000` (default) |
| **Config Keys** | `botmodels-enabled`, `botmodels-url` |
| **BASIC Keywords** | `SEE` |
| **Requires** | BotModels service running |

---

## Internal Services

These services are deployed locally as part of the General Bots stack. All are managed by the Directory service:

| Service | Default Port | Purpose | Management |
|---------|-------------|---------|------------|
| PostgreSQL | 5432 | Primary database | Vault |
| Qdrant | 6333 | Vector storage for KB | Vault |
| Cache | 6379 | Caching | Vault |
| Stalwart | 25, 993 | Email server (optional) | Vault |
| BotModels | 5000 | AI model inference | config.csv |

---

## Service Health Checks

All services can be checked via the monitoring API:

```
GET /api/monitoring/services
```

Response includes status for all configured external services.

---

## Troubleshooting

### Common Issues

1. **API Key Invalid** - Verify key in `config.csv`, ensure no trailing whitespace
2. **Rate Limited** - Check service quotas, implement caching with `SET BOT MEMORY`
3. **Connection Timeout** - Verify network access to external URLs
4. **Service Unavailable** - Check service status pages

### Debug Logging

Enable trace logging to see external API calls:

```bash
RUST_LOG=trace ./botserver
```
