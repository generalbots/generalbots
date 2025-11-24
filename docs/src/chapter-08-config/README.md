# Bot Configuration

This chapter covers bot configuration through the `config.csv` file system. Each bot's behavior is controlled by a simple CSV configuration file in its `.gbot` package.

## Configuration System

BotServer uses a straightforward name-value CSV format for configuration:

```csv
name,value
setting_name,setting_value
another_setting,another_value
```

## File Location

```
mybot.gbai/
└── mybot.gbot/
    └── config.csv
```

## Configuration Categories

### Server Settings
- Web server binding and ports
- Site generation paths
- Service endpoints

### LLM Configuration
- Model paths (local GGUF files)
- Service URLs
- Cache settings
- Server parameters (when embedded)

### Prompt Management
- Context compaction levels
- History retention
- Token management

### Email Integration
- SMTP server settings
- Authentication credentials
- Sender configuration

### Theme Customization
- Color schemes
- Logo URLs
- Bot titles

### Custom Database
- External database connections
- Authentication details

## Key Features

### Simple Format
- Plain CSV with name-value pairs
- No complex syntax
- Human-readable

### Flexible Structure
- Empty rows for visual grouping
- Optional settings with defaults
- Extensible for custom needs

### Local-First
- Designed for local LLM models
- Self-hosted services
- No cloud dependency by default

## Example Configurations

### Minimal Setup
Just the essentials to run a bot:
```csv
name,value
llm-url,http://localhost:8081
llm-model,../../../../data/llm/model.gguf
```

### Production Setup
Full configuration with all services:
```csv
name,value
,
server_host,0.0.0.0
server_port,8080
,
llm-url,http://localhost:8081
llm-model,../../../../data/llm/production-model.gguf
llm-cache,true
,
email-server,smtp.company.com
email-from,bot@company.com
,
theme-title,Company Assistant
```

## Configuration Philosophy

1. **Defaults Work**: Most settings have sensible defaults
2. **Local First**: Assumes local services, not cloud APIs
3. **Simple Values**: All values are strings, parsed as needed
4. **No Magic**: What you see is what you get

## See Also

- [config.csv Format](./config-csv.md) - Complete reference
- [LLM Configuration](./llm-config.md) - Language model settings
- [Parameters](./parameters.md) - All available parameters
---

<div align="center">
  <img src="https://pragmatismo.com.br/icons/general-bots-text.svg" alt="General Bots" width="200">
</div>
