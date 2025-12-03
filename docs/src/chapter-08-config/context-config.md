# Configuration Management

Configuration in General Bots is designed to be simple and transparent. Each bot uses a `config.csv` file for settings, with additional environment variables for system-level configuration.

## The config.csv File

Located in your bot's `.gbot` package, this file controls all bot-specific settings using simple name-value pairs.

### File Format

```csv
name,value
setting_name,setting_value
another_setting,another_value
```

- **Empty rows** are used for visual grouping
- **No quotes** needed for string values
- **Case-sensitive** names
- **Comments** not supported (keep it simple)

## Core Configuration Sections

### Server Configuration
```csv
server-host,0.0.0.0
server-port,8080
sites-root,/tmp
```

| Name | Description | Default | Example |
|------|-------------|---------|---------|
| `server-host` | Bind address for the web server | `0.0.0.0` | `0.0.0.0` |
| `server-port` | Port for the web interface | `8080` | `8080` |
| `sites-root` | Directory for generated sites | `/tmp` | `/tmp` |

### LLM Configuration - Overview

For detailed LLM configuration, see the tables below. The basic settings are:

```csv
llm-key,none
llm-url,http://localhost:8081
llm-model,../../../../data/llm/DeepSeek-R1-Distill-Qwen-1.5B-Q3_K_M.gguf
```

#### Core LLM Settings

| Name | Description | Default | Example |
|------|-------------|---------|---------|
| `llm-key` | API key for LLM service | `none` | `gsk-...` for Groq |
| `llm-url` | LLM service endpoint | `http://localhost:8081` | `https://api.groq.com/openai/v1` |
| `llm-model` | Model path or name | Required | `mixtral-8x7b-32768` |

#### Model Path Options
- **Local GGUF**: `../../../../data/llm/model.gguf`
- **Absolute path**: `/opt/models/model.gguf`
- **Cloud model name**: `mixtral-8x7b-32768` (for Groq)

#### Supported Formats
- **GGUF**: Quantized models (Q3_K_M, Q4_K_M, Q5_K_M, Q8_0)
- **API Models**: Any Groq or OpenAI-compatible model

### LLM Cache Settings

```csv
llm-cache,false
llm-cache-ttl,3600
llm-cache-semantic,true
llm-cache-threshold,0.95
```

| Name | Description | Default | Type |
|------|-------------|---------|------|
| `llm-cache` | Enable response caching | `false` | Boolean |
| `llm-cache-ttl` | Cache time-to-live in seconds | `3600` | Number |
| `llm-cache-semantic` | Use semantic similarity | `true` | Boolean |
| `llm-cache-threshold` | Similarity threshold for cache hits | `0.95` | Float |

**Cache Strategy Tips:**
- Enable for production to reduce API costs
- Semantic cache finds similar (not just identical) queries
- Lower threshold (0.90) = more hits but less precision
- Higher threshold (0.98) = fewer hits but exact matches

### Context Management

```csv
episodic-memory-threshold,4
episodic-memory-history,2
```

| Name | Description | Default | Range |
|------|-------------|---------|-------|
| `episodic-memory-threshold` | Messages before compaction | `4` | 1-10 |
| `episodic-memory-history` | Messages to keep in history | Not set | 1-20 |

### Embedding Configuration

```csv
embedding-url,http://localhost:8082
embedding-model,../../../../data/llm/bge-small-en-v1.5-f32.gguf
```

| Name | Description | Default | Type |
|------|-------------|---------|------|
| `embedding-url` | Embedding service endpoint | `http://localhost:8082` | URL |
| `embedding-model` | Path to embedding model | Required for KB | Path |

### LLM Server Settings (When Self-Hosting)

```csv
llm-server,true
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

#### Performance Parameters

| Parameter | Description | Default | Impact |
|-----------|-------------|---------|---------|
| `llm-server-gpu-layers` | Layers to offload to GPU | `0` | 0=CPU only, higher=more GPU usage |
| `llm-server-n-moe` | MoE experts count | `0` | Enables 120B+ models on consumer GPUs |
| `llm-server-ctx-size` | Context window (tokens) | `4096` | More context = more memory |
| `llm-server-n-predict` | Max output tokens | `1024` | Limits response length |
| `llm-server-parallel` | Concurrent requests | `6` | Higher = more throughput |
| `llm-server-cont-batching` | Continuous batching | `true` | Better multi-user performance |
| `llm-server-mlock` | Lock model in RAM | `false` | Prevents swapping to disk |
| `llm-server-no-mmap` | Disable memory mapping | `false` | Uses more RAM but may be faster |

#### Hardware-Specific Settings

**RTX 3090 (24GB VRAM)**
- Set `llm-server-gpu-layers` to 35-45 for 7B-32B models
- Enable `llm-server-n-moe` 2-4 for 120B+ models
- Can run DeepSeek-V3 with proper MoE settings

**RTX 4070/Ti (12-16GB)**
- Set `llm-server-gpu-layers` to 25-30 for 7B-14B models
- Keep `llm-server-ctx-size` at 4096 to save VRAM

**CPU-Only Setup**
- Keep `llm-server-gpu-layers` at 0
- Enable `llm-server-mlock` to prevent swapping
- Set `llm-server-parallel` to CPU cores -2

### Email Configuration

```csv
email-from,from@domain.com
email-server,mail.domain.com
email-port,587
email-user,user@domain.com
email-pass,password
```

All email parameters are required if you want to send emails from your bot.

### Custom Database (Optional)

```csv
custom-server,localhost
custom-port,5432
custom-database,mycustomdb
custom-username,dbuser
custom-password,dbpass
```

## Configuration Examples

### Minimal Configuration
```csv
name,value
server-port,8080
llm-url,http://localhost:8081
llm-model,../../../../data/llm/model.gguf
```

### Production Configuration (Groq Cloud)
```csv
name,value
,
server-host,0.0.0.0
server-port,443
sites-root,/var/www/sites
,
# Groq is 10x faster than traditional cloud providers
llm-key,gsk-your-groq-api-key
llm-url,https://api.groq.com/openai/v1
llm-model,mixtral-8x7b-32768
,
llm-cache,true
llm-cache-ttl,7200
llm-cache-semantic,true
llm-cache-threshold,0.95
,
episodic-memory-threshold,6
,
email-from,bot@company.com
email-server,smtp.company.com
email-port,587
email-user,bot@company.com
email-pass,secure-password
```

### Local Development (Self-Hosted)
```csv
name,value
,
server-port,3000
,
# Run your own LLM server
llm-server,true
llm-server-gpu-layers,35
llm-server-ctx-size,8192
llm-server-n-predict,2048
llm-model,../../../../data/llm/DeepSeek-R1-Distill-Qwen-7B-Q4_K_M.gguf
,
# Disable cache for development
llm-cache,false
episodic-memory-threshold,2
```

## Environment Variables

System-level configuration uses environment variables:

```bash
# Database
DATABASE_URL=postgres://gbuser:password@localhost:5432/botserver

# Object Storage
DRIVE_SERVER=http://localhost:9000
DRIVE_ACCESSKEY=minioadmin
DRIVE_SECRET=minioadmin

# Optional
LOG_LEVEL=debug
```

## Configuration Priority

Settings are applied in this order (later overrides earlier):
1. Default values in code
2. config.csv settings
3. Environment variables (for system settings only)

## Best Practices

1. **Keep it Simple**: Only configure what you need to change
2. **Use Groups**: Empty rows make the file readable
3. **Test Locally**: Verify settings before production
4. **Secure Secrets**: Use environment variables for passwords in production
5. **Document Changes**: Comment significant changes in version control

## Validation

The system validates configuration on startup:
- Missing required values cause clear error messages
- Invalid URLs or paths are detected early
- Port conflicts are reported
- Model file existence is verified

## Hot Reload

Some settings support hot reload without restart:
- Cache settings
- Context parameters
- Email configuration

Others require restart:
- Server ports
- LLM model changes
- Database connections

## Troubleshooting

### Common Issues

**Port Already in Use**
- Change `server-port` to an available port
- Check for other services on the same port

**Model Not Found**
- Verify the path in `llm-model` is correct
- Ensure the GGUF file exists
- Use absolute paths if relative paths fail

**LLM Server Won't Start**
- Check `llm-server-gpu-layers` doesn't exceed your GPU capability
- Reduce `llm-server-ctx-size` if out of memory
- Set `llm-server-gpu-layers` to 0 for CPU-only
- Verify model file exists at the specified path
- Check available VRAM with `nvidia-smi` (if using GPU)

**Cache Not Working**
- Ensure `llm-cache` is set to `true`
- Check `llm-cache-threshold` isn't too high (0.95 is usually good)
- Verify Valkey/Redis is running

## Quick Model Recommendations

### Best Models by Hardware

**24GB+ VRAM (RTX 3090, 4090)**
- DeepSeek-V3 (with MoE enabled)
- Qwen2.5-32B-Instruct-Q4_K_M
- DeepSeek-R1-Distill-Qwen-14B (runs fast with room to spare)

**12-16GB VRAM (RTX 4070, 4070Ti)**  
- DeepSeek-R1-Distill-Llama-8B
- Qwen2.5-14B-Q4_K_M
- Mistral-7B-Instruct-Q5_K_M

**8GB VRAM or CPU-Only**
- DeepSeek-R1-Distill-Qwen-1.5B
- Phi-3-mini-4k-instruct
- Qwen2.5-3B-Instruct-Q5_K_M

**Cloud API (Fastest)**
- Groq: mixtral-8x7b-32768
- Groq: llama-3.1-70b-versatile

## Summary

General Bots configuration is intentionally simple - a CSV file with name-value pairs. No complex YAML, no nested JSON, just straightforward settings that anyone can edit. Start with minimal configuration and add settings as needed.

For LLM configuration, the key decision is local vs cloud:
- **Local**: Full control, no recurring costs, complete privacy
- **Cloud (Groq)**: 10x faster inference, pay-per-use, no hardware needed