# config.csv Format

The `config.csv` file is the central configuration for each bot, located in the `.gbot` package. It uses a simple name-value pair format.

## File Format

```csv
name,value
setting_name,setting_value
another_setting,another_value
```

- **Empty rows** are used for visual grouping
- **No quotes** needed for string values
- **Case-sensitive** names

## Core Server Settings

### Server Configuration
```csv
server_host,0.0.0.0
server_port,8080
sites_root,/tmp
```

| Name | Description | Default | Example |
|------|-------------|---------|---------|
| `server_host` | Bind address for the web server | `0.0.0.0` | `0.0.0.0` |
| `server_port` | Port for the web interface | `8080` | `8080` |
| `sites_root` | Directory for generated sites | `/tmp` | `/tmp` |

## LLM Configuration

### LLM Connection
```csv
llm-key,none
llm-url,http://localhost:8081
llm-model,../../../../data/llm/DeepSeek-R1-Distill-Qwen-1.5B-Q3_K_M.gguf
```

| Name | Description | Default | Example |
|------|-------------|---------|---------|
| `llm-key` | API key for LLM service | `none` | `none` or API key |
| `llm-url` | LLM service endpoint | `http://localhost:8081` | `http://localhost:8081` |
| `llm-model` | Path to GGUF model file | Model path | `../../../../data/llm/model.gguf` |

### LLM Cache Settings
```csv
llm-cache,false
llm-cache-ttl,3600
llm-cache-semantic,true
llm-cache-threshold,0.95
```

| Name | Description | Default | Example |
|------|-------------|---------|---------|
| `llm-cache` | Enable response caching | `false` | `true` or `false` |
| `llm-cache-ttl` | Cache TTL in seconds | `3600` | `3600` |
| `llm-cache-semantic` | Enable semantic similarity caching | `true` | `true` or `false` |
| `llm-cache-threshold` | Similarity threshold (0-1) | `0.95` | `0.95` |

### LLM Server Settings (when running embedded)
```csv
llm-server,false
llm-server-path,botserver-stack/bin/llm/build/bin
llm-server-host,0.0.0.0
llm-server-port,8081
llm-server-gpu-layers,0
llm-server-n-moe,0
llm-server-ctx-size,4096
llm-server-n-predict,1024
llm-server-parallel,6
llm-server-cont-batching,true
llm-server-mlock,false
llm-server-no-mmap,false
```

| Name | Description | Default |
|------|-------------|---------|
| `llm-server` | Run embedded LLM server | `false` |
| `llm-server-path` | Path to LLM server binaries | `botserver-stack/bin/llm/build/bin` |
| `llm-server-host` | LLM server bind address | `0.0.0.0` |
| `llm-server-port` | LLM server port | `8081` |
| `llm-server-gpu-layers` | GPU layers to offload | `0` |
| `llm-server-n-moe` | Number of MoE experts | `0` |
| `llm-server-ctx-size` | Context size in tokens | `4096` |
| `llm-server-n-predict` | Max prediction tokens | `1024` |
| `llm-server-parallel` | Parallel requests | `6` |
| `llm-server-cont-batching` | Continuous batching | `true` |
| `llm-server-mlock` | Lock model in memory | `false` |
| `llm-server-no-mmap` | Disable memory mapping | `false` |

## Prompt Settings

```csv
prompt-compact,4
prompt-history,2
```

| Name | Description | Default | Example |
|------|-------------|---------|---------|
| `prompt-compact` | Context compaction level | `4` | `4` |
| `prompt-history` | Messages to keep in history | Not set | `2` |

## Embedding Configuration

```csv
embedding-url,http://localhost:8082
embedding-model,../../../../data/llm/bge-small-en-v1.5-f32.gguf
```

| Name | Description | Default |
|------|-------------|---------|
| `embedding-url` | Embedding service endpoint | `http://localhost:8082` |
| `embedding-model` | Path to embedding model | Model path |

## Email Configuration

```csv
email-from,from@domain.com
email-server,mail.domain.com
email-port,587
email-user,user@domain.com
email-pass,
```

| Name | Description | Example |
|------|-------------|---------|
| `email-from` | Sender email address | `noreply@example.com` |
| `email-server` | SMTP server hostname | `smtp.gmail.com` |
| `email-port` | SMTP port | `587` |
| `email-user` | SMTP username | `user@example.com` |
| `email-pass` | SMTP password | Password (empty if not set) |

## Theme Configuration

```csv
theme-color1,#0d2b55
theme-color2,#fff9c2
theme-logo,https://pragmatismo.com.br/icons/general-bots.svg
theme-title,Announcements General Bots
```

| Name | Description | Example |
|------|-------------|---------|
| `theme-color1` | Primary theme color | `#0d2b55` |
| `theme-color2` | Secondary theme color | `#fff9c2` |
| `theme-logo` | Logo URL | `https://example.com/logo.svg` |
| `theme-title` | Bot display title | `My Bot` |

## Custom Database

```csv
custom-server,localhost
custom-port,5432
custom-database,mycustomdb
custom-username,
custom-password,
```

| Name | Description | Example |
|------|-------------|---------|
| `custom-server` | Database server | `localhost` |
| `custom-port` | Database port | `5432` |
| `custom-database` | Database name | `mydb` |
| `custom-username` | Database user | Username |
| `custom-password` | Database password | Password |

## MCP Server

```csv
mcp-server,false
```

| Name | Description | Default |
|------|-------------|---------|
| `mcp-server` | Enable MCP server | `false` |

## Complete Example

### Minimal Configuration
```csv
name,value
server_port,8080
llm-url,http://localhost:8081
llm-model,../../../../data/llm/model.gguf
```

### Production Configuration
```csv
name,value
,
server_host,0.0.0.0
server_port,443
sites_root,/var/www/sites
,
llm-key,sk-...
llm-url,https://api.openai.com
llm-model,gpt-4
,
llm-cache,true
llm-cache-ttl,7200
,
email-from,bot@company.com
email-server,smtp.company.com
email-port,587
email-user,bot@company.com
email-pass,secure_password
,
theme-title,Company Assistant
theme-color1,#003366
theme-color2,#ffffff
```

## Configuration Loading

1. Default values are applied first
2. `config.csv` values override defaults
3. Environment variables override config.csv (if implemented)
4. All values are strings - parsed as needed by the application

## Best Practices

✅ **DO:**
- Group related settings with empty rows
- Use descriptive values
- Keep sensitive data in environment variables when possible
- Test configuration changes in development first

❌ **DON'T:**
- Include quotes around values
- Use spaces around commas
- Leave trailing commas
- Include comments with # (use empty name field instead)

## Validation

The system validates:
- Required fields are present
- Port numbers are valid (1-65535)
- URLs are properly formatted
- File paths exist (for model files)
- Email settings are complete if email features are used