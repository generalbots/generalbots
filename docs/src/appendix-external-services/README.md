# Appendix B: External Services

This appendix catalogs all external services that General Bots integrates with, including their configuration requirements, associated BASIC keywords, and API endpoints.

## Overview

General Bots connects to external services for extended functionality. All service credentials should be stored in `config.csv` within the bot's `.gbot` folder - never hardcoded in scripts.

Infrastructure services (database, storage, cache) are automatically managed by the Directory service (Zitadel).

## Service Categories

| Category | Services | Configuration Location |
|----------|----------|----------------------|
| LLM Providers | OpenAI, Groq, Anthropic, Azure OpenAI | `config.csv` |
| Weather | OpenWeatherMap | `config.csv` |
| Messaging Channels | WhatsApp, Teams, Instagram, Telegram | `config.csv` |
| Storage | S3-Compatible (MinIO, etc.) | Vault (automatic) |
| Directory | Zitadel | `VAULT_*` environment variables |
| Email | Stalwart / IMAP/SMTP | Vault (automatic) |
| Calendar | CalDAV servers | `config.csv` |
| Database | PostgreSQL | Vault (automatic) |
| Cache | Redis-compatible | Vault (automatic) |

## Quick Reference

### BASIC Keywords That Call External Services

| Keyword | Service | Config Key |
|---------|---------|-----------|
| `LLM` | LLM Provider | `llm-provider`, `llm-api-key` |
| `WEATHER` | OpenWeatherMap | `weather-api-key` |
| `SEND MAIL` | SMTP Server | Managed by Directory service |
| `SEND WHATSAPP` | WhatsApp Business API | `whatsapp-api-key`, `whatsapp-phone-number-id` |
| `SEND TEAMS` | Microsoft Teams | `teams-app-id`, `teams-app-password` |
| `SEND INSTAGRAM` | Instagram Graph API | `instagram-access-token`, `instagram-page-id` |
| `GET` (with http/https URL) | Any HTTP endpoint | N/A |
| `IMAGE` | BotModels (local) | `botmodels-enabled`, `botmodels-url` |
| `VIDEO` | BotModels (local) | `botmodels-enabled`, `botmodels-url` |
| `AUDIO` | BotModels (local) | `botmodels-enabled`, `botmodels-url` |
| `SEE` | BotModels (local) | `botmodels-enabled`, `botmodels-url` |
| `FIND` | Qdrant (local) | Internal service |
| `USE WEBSITE` | Web crawling | N/A |

## Service Configuration Template

Add these to your `config.csv`:

```csv
key,value
llm-provider,openai
llm-api-key,YOUR_API_KEY
llm-model,gpt-4o
weather-api-key,YOUR_OPENWEATHERMAP_KEY
whatsapp-api-key,YOUR_WHATSAPP_KEY
whatsapp-phone-number-id,YOUR_PHONE_ID
teams-app-id,YOUR_TEAMS_APP_ID
teams-app-password,YOUR_TEAMS_PASSWORD
instagram-access-token,YOUR_INSTAGRAM_TOKEN
instagram-page-id,YOUR_PAGE_ID
botmodels-enabled,true
botmodels-url,http://localhost:5000
```

## Auto-Managed Services

The following services are automatically configured by the Directory service (Zitadel):

| Service | What's Managed |
|---------|----------------|
| PostgreSQL | Connection credentials, database creation |
| S3-Compatible Storage | Access keys, bucket policies |
| Cache | Connection credentials |
| Stalwart Email | User accounts, SMTP/IMAP access |

You do **not** need to configure these services manually. The Directory service handles credential provisioning and rotation.

## Security Notes

1. **Never hardcode credentials** - Always use `config.csv` or `GET BOT MEMORY`
2. **Rotate keys regularly** - Update `config.csv` and restart the bot
3. **Use least privilege** - Only grant permissions needed by the bot
4. **Audit access** - Monitor external API usage through logs
5. **Infrastructure credentials** - Managed automatically by Directory service

## See Also

- [Service Catalog](./catalog.md) - Detailed service documentation
- [LLM Providers](./llm-providers.md) - AI model configuration
- [Weather API](./weather.md) - Weather service setup
- [Channel Integrations](./channels.md) - Messaging platform setup
- [Storage Services](./storage.md) - S3-compatible storage
- [Directory Services](./directory.md) - User authentication
- [Environment Variables](../appendix-env-vars/README.md) - DIRECTORY_* configuration