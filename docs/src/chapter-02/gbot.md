# .gbot Bot Configuration

The `.gbot` package contains configuration files that define bot behavior, parameters, and operational settings.

## What is .gbot?

`.gbot` files configure:
- Bot identity and description
- LLM provider settings
- Context management
- Bot behavior settings
- Integration parameters

## Configuration Structure

The primary configuration file is `config.csv`:

```csv
key,value
bot_name,Customer Support Assistant
bot_description,AI-powered support agent
llm_provider,openai
llm_model,gpt-4
temperature,0.7
max_tokens,1000
system_prompt,You are a helpful customer support agent...
```

## Key Configuration Parameters

### Bot Identity
- `bot_name`: Display name for the bot
- `bot_description`: Purpose and capabilities
- `version`: Bot version for tracking

### LLM Configuration
- `llm_provider`: openai, azure, local
- `llm_model`: Model name (gpt-4, claude-3, etc.)
- `temperature`: Creativity control (0.0-1.0)
- `max_tokens`: Response length limit

### Answer Modes
- `0`: Direct LLM responses only
- `1`: LLM with tool calling
- `2`: Knowledge base documents only
- `3`: Include web search results
- `4`: Mixed mode with tools and KB

### Context Management
- `context_window`: Number of messages to retain
- `context_provider`: How context is managed
- `memory_enabled`: Whether to use bot memory

## Configuration Loading

The system loads configuration from:
1. `config.csv` in the .gbot package
2. Environment variables (override)
3. Database settings (persistent)
4. Runtime API calls (temporary)

## Configuration Precedence

Settings are applied in this order (later overrides earlier):
1. Default values
2. .gbot/config.csv  
3. Environment variables
4. Database configuration
5. Runtime API updates

## Dynamic Configuration

Some settings can be changed at runtime:
```basic
REM Store configuration dynamically
SET BOT MEMORY "preferred_style", "detailed"
```

## Bot Memory

The `SET BOT MEMORY` and `GET BOT MEMORY` keywords allow storing and retrieving bot-specific data that persists across sessions.
