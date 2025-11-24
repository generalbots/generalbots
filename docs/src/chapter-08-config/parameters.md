# Configuration Parameters

Complete reference of all available parameters in `config.csv`.

## Server Parameters

### Web Server
| Parameter | Description | Default | Type |
|-----------|-------------|---------|------|
| `server_host` | Server bind address | `0.0.0.0` | IP address |
| `server_port` | Server listen port | `8080` | Number (1-65535) |
| `sites_root` | Generated sites directory | `/tmp` | Path |

### MCP Server
| Parameter | Description | Default | Type |
|-----------|-------------|---------|------|
| `mcp-server` | Enable MCP protocol server | `false` | Boolean |

## LLM Parameters

### Core LLM Settings
| Parameter | Description | Default | Type |
|-----------|-------------|---------|------|
| `llm-key` | API key for LLM service | `none` | String |
| `llm-url` | LLM service endpoint | `http://localhost:8081` | URL |
| `llm-model` | Model path or identifier | Required | Path/String |

### LLM Cache
| Parameter | Description | Default | Type |
|-----------|-------------|---------|------|
| `llm-cache` | Enable response caching | `false` | Boolean |
| `llm-cache-ttl` | Cache time-to-live | `3600` | Seconds |
| `llm-cache-semantic` | Semantic similarity cache | `true` | Boolean |
| `llm-cache-threshold` | Similarity threshold | `0.95` | Float (0-1) |

### Embedded LLM Server
| Parameter | Description | Default | Type |
|-----------|-------------|---------|------|
| `llm-server` | Run embedded server | `false` | Boolean |
| `llm-server-path` | Server binary path | `botserver-stack/bin/llm/build/bin` | Path |
| `llm-server-host` | Server bind address | `0.0.0.0` | IP address |
| `llm-server-port` | Server port | `8081` | Number |
| `llm-server-gpu-layers` | GPU offload layers | `0` | Number |
| `llm-server-n-moe` | MoE experts count | `0` | Number |
| `llm-server-ctx-size` | Context size | `4096` | Tokens |
| `llm-server-n-predict` | Max predictions | `1024` | Tokens |
| `llm-server-parallel` | Parallel requests | `6` | Number |
| `llm-server-cont-batching` | Continuous batching | `true` | Boolean |
| `llm-server-mlock` | Lock in memory | `false` | Boolean |
| `llm-server-no-mmap` | Disable mmap | `false` | Boolean |

## Embedding Parameters

| Parameter | Description | Default | Type |
|-----------|-------------|---------|------|
| `embedding-url` | Embedding service endpoint | `http://localhost:8082` | URL |
| `embedding-model` | Embedding model path | Required for KB | Path |

## Prompt Parameters

| Parameter | Description | Default | Type |
|-----------|-------------|---------|------|
| `prompt-compact` | Context compaction level | `4` | Number |
| `prompt-history` | Messages in history | Not set | Number |

## Email Parameters

| Parameter | Description | Default | Type |
|-----------|-------------|---------|------|
| `email-from` | Sender address | Required for email | Email |
| `email-server` | SMTP hostname | Required for email | Hostname |
| `email-port` | SMTP port | `587` | Number |
| `email-user` | SMTP username | Required for email | String |
| `email-pass` | SMTP password | Required for email | String |

## Theme Parameters

| Parameter | Description | Default | Type |
|-----------|-------------|---------|------|
| `theme-color1` | Primary color | Not set | Hex color |
| `theme-color2` | Secondary color | Not set | Hex color |
| `theme-logo` | Logo URL | Not set | URL |
| `theme-title` | Bot display title | Not set | String |

## Custom Database Parameters

| Parameter | Description | Default | Type |
|-----------|-------------|---------|------|
| `custom-server` | Database server | `localhost` | Hostname |
| `custom-port` | Database port | `5432` | Number |
| `custom-database` | Database name | Not set | String |
| `custom-username` | Database user | Not set | String |
| `custom-password` | Database password | Not set | String |

## Parameter Types

### Boolean
Values: `true` or `false` (case-sensitive)

### Number
Integer values, must be within valid ranges:
- Ports: 1-65535
- Tokens: Positive integers
- Percentages: 0-100

### Float
Decimal values:
- Thresholds: 0.0 to 1.0

### Path
File system paths:
- Relative: `../../../../data/model.gguf`
- Absolute: `/opt/models/model.gguf`

### URL
Valid URLs:
- HTTP: `http://localhost:8081`
- HTTPS: `https://api.example.com`

### String
Any text value (no quotes needed in CSV)

### Email
Valid email format: `user@domain.com`

### Hex Color
HTML color codes: `#RRGGBB` format

## Required vs Optional

### Always Required
- None - all parameters have defaults or are optional

### Required for Features
- **LLM**: `llm-model` must be set
- **Email**: `email-from`, `email-server`, `email-user`
- **Embeddings**: `embedding-model` for knowledge base
- **Custom DB**: `custom-database` if using external database

## Configuration Precedence

1. **Built-in defaults** (hardcoded)
2. **config.csv values** (override defaults)
3. **Environment variables** (if implemented, override config)

## Special Values

- `none` - Explicitly no value (for `llm-key`)
- Empty string - Unset/use default
- `false` - Feature disabled
- `true` - Feature enabled

## Performance Tuning

### For Local Models
```csv
llm-server-ctx-size,8192
llm-server-n-predict,2048
llm-server-parallel,4
llm-cache,true
llm-cache-ttl,7200
```

### For Production
```csv
llm-server-cont-batching,true
llm-cache-semantic,true
llm-cache-threshold,0.90
llm-server-parallel,8
```

### For Low Memory
```csv
llm-server-ctx-size,2048
llm-server-n-predict,512
llm-server-mlock,false
llm-server-no-mmap,false
llm-cache,false
```

## Validation Rules

1. **Paths**: Model files must exist
2. **URLs**: Must be valid format
3. **Ports**: Must be 1-65535
4. **Emails**: Must contain @ and domain
5. **Colors**: Must be valid hex format
6. **Booleans**: Exactly `true` or `false`
