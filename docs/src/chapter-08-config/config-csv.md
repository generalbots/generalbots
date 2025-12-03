# config.csv Format

The `config.csv` file is the heart of bot configuration in General Bots. Located in each bot's `.gbot` package, it uses a simple, human-readable format that anyone can edit.

## Why CSV?

We chose CSV because:
- **No syntax errors** - Just name,value pairs
- **Spreadsheet compatible** - Edit in Excel, Google Sheets, or any text editor
- **Human readable** - No brackets, no indentation wars
- **Git friendly** - Clean diffs, easy merges

## Basic Format

```csv
name,value
server-port,8080
llm-model,../../../../data/llm/model.gguf
```

That's it. No quotes, no special characters, just names and values.

## Visual Organization

Use empty rows to group related settings:

```csv
name,value

# Server settings
server-host,0.0.0.0
server-port,8080

# LLM settings (see Configuration Management for details)
llm-url,http://localhost:8081
llm-model,model.gguf

# Email settings
email-from,bot@example.com
email-server,smtp.example.com
```

## Key Points

- **Case matters**: `server-port` not `Server-Port`
- **No spaces**: Around commas or in names
- **Paths**: Can be relative or absolute
- **Booleans**: Use `true` or `false`
- **Numbers**: Just write them directly

## Quick Example

A complete working configuration:

```csv
name,value
server-port,8080
llm-url,http://localhost:8081
llm-model,../../../../data/llm/DeepSeek-R1-Distill-Qwen-1.5B-Q3_K_M.gguf
episodic-memory-threshold,4
```

Four lines. Bot configured. That's the General Bots way.

## LLM Configuration

Basic LLM settings in config.csv:
- `llm-url` - Where your LLM server is (local or cloud)
- `llm-model` - Which model to use
- `llm-key` - API key if using cloud services like Groq

For detailed LLM configuration including GPU settings, cache, performance tuning, and hardware-specific recommendations, see [Configuration Management](./context-config.md#llm-configuration---overview).

## Where to Find Settings

For the complete list of available settings and detailed explanations, see [Configuration Management](./context-config.md).

## Philosophy

Configuration should be boring. You should spend time on your bot's personality and capabilities, not fighting with config files. CSV keeps it simple so you can focus on what matters.