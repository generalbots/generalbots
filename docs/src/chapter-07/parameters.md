# Bot Parameters

Comprehensive reference for all bot configuration parameters available in `config.csv`.

## Parameter Categories

Bot parameters are organized into functional groups for easier management and understanding.

## Core Bot Settings

### Identity Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `botId` | UUID | Yes | Generated | Unique bot identifier |
| `title` | String | Yes | None | Bot display name |
| `description` | String | No | Empty | Bot description |
| `version` | String | No | "1.0" | Bot version |
| `author` | String | No | Empty | Bot creator |
| `language` | String | No | "en" | Default language |
| `timezone` | String | No | "UTC" | Bot timezone |

### Behavior Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `welcomeMessage` | String | No | Empty | Initial greeting |
| `fallbackMessage` | String | No | "I don't understand" | Default error response |
| `goodbyeMessage` | String | No | "Goodbye!" | Session end message |
| `typingDelay` | Number | No | 1000 | Typing indicator delay (ms) |
| `responseTimeout` | Number | No | 30000 | Response timeout (ms) |
| `maxRetries` | Number | No | 3 | Maximum retry attempts |
| `debugMode` | Boolean | No | false | Enable debug logging |

## LLM Parameters

### Model Configuration

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `llmProvider` | String | Yes | "openai" | LLM provider (openai, anthropic, google, local) |
| `llmModel` | String | Yes | "gpt-4" | Model name |
| `llmApiKey` | String | Yes* | None | API key (*not required for local) |
| `llmEndpoint` | String | No | Provider default | Custom API endpoint |
| `llmOrganization` | String | No | Empty | Organization ID (OpenAI) |
| `llmProject` | String | No | Empty | Project ID (Google) |

### Response Control

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `llmTemperature` | Float | No | 0.7 | Creativity (0.0-1.0) |
| `llmMaxTokens` | Number | No | 2000 | Max response tokens |
| `llmTopP` | Float | No | 1.0 | Nucleus sampling |
| `llmFrequencyPenalty` | Float | No | 0.0 | Reduce repetition |
| `llmPresencePenalty` | Float | No | 0.0 | Encourage new topics |
| `llmStopSequences` | String | No | Empty | Stop generation sequences |
| `llmSystemPrompt` | String | No | Default | System instruction |

### Cost Management

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `llmCostLimit` | Number | No | 100 | Monthly cost limit ($) |
| `llmTokenLimit` | Number | No | 1000000 | Monthly token limit |
| `llmRequestLimit` | Number | No | 10000 | Daily request limit |
| `llmCacheEnabled` | Boolean | No | true | Enable response caching |
| `llmCacheTTL` | Number | No | 3600 | Cache duration (seconds) |

## Knowledge Base Parameters

### Vector Database

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `vectorDbUrl` | String | No | "http://localhost:6333" | Qdrant URL |
| `vectorDbApiKey` | String | No | Empty | Qdrant API key |
| `vectorDbCollection` | String | No | Bot name | Default collection |
| `embeddingModel` | String | No | "text-embedding-ada-002" | Embedding model |
| `embeddingDimension` | Number | No | 1536 | Vector dimension |

### Search Configuration

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `searchTopK` | Number | No | 5 | Results to return |
| `searchThreshold` | Float | No | 0.7 | Minimum similarity |
| `searchRerank` | Boolean | No | false | Enable reranking |
| `chunkSize` | Number | No | 1000 | Text chunk size |
| `chunkOverlap` | Number | No | 200 | Chunk overlap |

## Storage Parameters

### Object Storage

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `storageProvider` | String | No | "minio" | Storage provider |
| `storageEndpoint` | String | Yes | "localhost:9000" | S3/MinIO endpoint |
| `storageAccessKey` | String | Yes | None | Access key |
| `storageSecretKey` | String | Yes | None | Secret key |
| `storageBucket` | String | No | "botserver" | Default bucket |
| `storageRegion` | String | No | "us-east-1" | AWS region |
| `storageUseSsl` | Boolean | No | false | Use HTTPS |

### File Handling

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `maxFileSize` | Number | No | 10 | Max file size (MB) |
| `allowedFileTypes` | String | No | "pdf,doc,txt,csv" | Allowed extensions |
| `fileRetention` | Number | No | 90 | Days to keep files |
| `autoDeleteTemp` | Boolean | No | true | Auto-delete temp files |

## Communication Parameters

### Email Settings

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `emailEnabled` | Boolean | No | false | Enable email |
| `smtpHost` | String | No* | Empty | SMTP server |
| `smtpPort` | Number | No | 587 | SMTP port |
| `smtpUser` | String | No* | Empty | Email username |
| `smtpPassword` | String | No* | Empty | Email password |
| `smtpFrom` | String | No* | Empty | From address |
| `smtpUseTls` | Boolean | No | true | Use TLS |
| `smtpUseStarttls` | Boolean | No | true | Use STARTTLS |

### Channel Configuration

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `webEnabled` | Boolean | No | true | Web interface |
| `webPort` | Number | No | 8080 | Web port |
| `whatsappEnabled` | Boolean | No | false | WhatsApp integration |
| `whatsappToken` | String | No* | Empty | WhatsApp token |
| `teamsEnabled` | Boolean | No | false | Teams integration |
| `teamsAppId` | String | No* | Empty | Teams app ID |
| `slackEnabled` | Boolean | No | false | Slack integration |
| `slackToken` | String | No* | Empty | Slack token |

## Security Parameters

### Authentication

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `authRequired` | Boolean | No | false | Require authentication |
| `authProvider` | String | No | "local" | Auth provider |
| `jwtSecret` | String | Yes* | Generated | JWT secret |
| `jwtExpiration` | Number | No | 86400 | Token expiration (s) |
| `sessionTimeout` | Number | No | 3600 | Session timeout (s) |
| `maxSessions` | Number | No | 100 | Max concurrent sessions |

### Access Control

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `corsOrigins` | String | No | "*" | Allowed origins |
| `ipWhitelist` | String | No | Empty | Allowed IPs |
| `ipBlacklist` | String | No | Empty | Blocked IPs |
| `rateLimitPerMinute` | Number | No | 60 | Requests per minute |
| `rateLimitPerHour` | Number | No | 1000 | Requests per hour |
| `requireHttps` | Boolean | No | false | Force HTTPS |

### Data Protection

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `encryptData` | Boolean | No | true | Encrypt stored data |
| `encryptionKey` | String | Yes* | Generated | Encryption key |
| `maskPii` | Boolean | No | true | Mask personal data |
| `auditLogging` | Boolean | No | true | Enable audit logs |
| `dataRetention` | Number | No | 365 | Data retention (days) |

## Performance Parameters

### Caching

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `cacheEnabled` | Boolean | No | true | Enable caching |
| `cacheProvider` | String | No | "redis" | Cache provider |
| `cacheUrl` | String | No | "redis://localhost:6379" | Cache URL |
| `cacheTtl` | Number | No | 3600 | Default TTL (s) |
| `cacheMaxSize` | Number | No | 100 | Max cache size (MB) |

### Resource Limits

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `maxCpu` | Number | No | 2 | CPU cores limit |
| `maxMemory` | Number | No | 2048 | Memory limit (MB) |
| `maxConnections` | Number | No | 100 | DB connections |
| `maxWorkers` | Number | No | 4 | Worker threads |
| `queueSize` | Number | No | 1000 | Task queue size |

## Monitoring Parameters

### Logging

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `logLevel` | String | No | "info" | Log level |
| `logToFile` | Boolean | No | true | Log to file |
| `logFilePath` | String | No | "./logs" | Log directory |
| `logRotation` | String | No | "daily" | Rotation schedule |
| `logRetention` | Number | No | 30 | Keep logs (days) |

### Metrics

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `metricsEnabled` | Boolean | No | false | Enable metrics |
| `metricsEndpoint` | String | No | "/metrics" | Metrics endpoint |
| `sentryDsn` | String | No | Empty | Sentry DSN |
| `datadogApiKey` | String | No | Empty | Datadog API key |
| `prometheusPort` | Number | No | 9090 | Prometheus port |

## Feature Flags

### Experimental Features

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `betaFeatures` | Boolean | No | false | Enable beta features |
| `webAutomation` | Boolean | No | false | Web scraping |
| `ocrEnabled` | Boolean | No | false | OCR support |
| `speechEnabled` | Boolean | No | false | Speech I/O |
| `visionEnabled` | Boolean | No | false | Image analysis |
| `codeExecution` | Boolean | No | false | Code running |

## Environment-Specific Parameters

### Development

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `devMode` | Boolean | No | false | Development mode |
| `hotReload` | Boolean | No | false | Hot reload |
| `mockServices` | Boolean | No | false | Use mock services |
| `verboseErrors` | Boolean | No | false | Detailed errors |

### Production

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `prodMode` | Boolean | No | true | Production mode |
| `clustering` | Boolean | No | false | Enable clustering |
| `loadBalancing` | Boolean | No | false | Load balancing |
| `autoScale` | Boolean | No | false | Auto-scaling |

## Parameter Validation

Parameters are validated on startup:
1. Required parameters must be present
2. Types are checked and coerced
3. Ranges are enforced
4. Dependencies verified
5. Conflicts detected

## Environment Variable Override

Any parameter can be overridden via environment:
```bash
BOT_TITLE="My Bot" BOT_LLM_MODEL="gpt-4-turbo" botserver
```

## Dynamic Parameter Updates

Some parameters can be updated at runtime:
- Log level
- Rate limits
- Cache settings
- Feature flags

Use the admin API to update:
```
POST /api/admin/config
{
  "logLevel": "debug",
  "rateLimitPerMinute": 120
}
```

## Best Practices

1. **Start with defaults**: Most parameters have sensible defaults
2. **Override only what's needed**: Don't set everything
3. **Use environment variables**: For sensitive values
4. **Document custom values**: Explain why changed
5. **Test configuration**: Validate before production
6. **Monitor performance**: Adjust based on metrics
7. **Version control**: Track configuration changes