# Chapter 09: API and Tooling

Define tools that LLMs can call from your BASIC scripts.

## Overview

Tools are BASIC scripts with PARAM declarations that become callable functions for the LLM. This enables AI-driven automation with structured inputs.

## Tool Structure

```basic
' weather.bas - A tool the LLM can invoke
PARAM city AS STRING LIKE "London" DESCRIPTION "City name"
PARAM units AS STRING LIKE "celsius" DESCRIPTION "Temperature units"

DESCRIPTION "Gets current weather for a city"

data = GET "api.weather.com/current?city=" + city
TALK "Weather in " + city + ": " + data.temperature + "°"
```

## How It Works

1. **PARAM** declarations define inputs
2. **DESCRIPTION** explains the tool's purpose
3. LLM decides when to call the tool
4. Parameters collected through conversation
5. Tool executes with validated inputs

## PARAM Declaration

```basic
PARAM name AS type LIKE "example" DESCRIPTION "explanation"
```

| Component | Purpose |
|-----------|---------|
| `name` | Variable name |
| `type` | STRING, INTEGER, DATE, etc. |
| `LIKE` | Example value for LLM |
| `DESCRIPTION` | What this parameter is for |

## Tool Formats

Tools compile to multiple formats:

| Format | Use Case |
|--------|----------|
| MCP | Model Context Protocol |
| OpenAI | Function calling |
| Internal | BASIC runtime |

## Chapter Contents

- [Tool Definition](./tool-definition.md) - Creating tools
- [PARAM Declaration](./param-declaration.md) - Parameter syntax
- [Tool Compilation](./compilation.md) - Build process
- [MCP Format](./mcp-format.md) - MCP integration
- [OpenAI Format](./openai-format.md) - Function calling
- [GET Integration](./get-integration.md) - API calls
- [External APIs](./external-apis.md) - Third-party services
- [LLM REST Server](./llm-rest-server.md) - Hosting models
- [NVIDIA GPU Setup](./nvidia-gpu-setup.md) - GPU acceleration

## See Also

- [BASIC Dialogs](../chapter-06-gbdialog/README.md) - Scripting reference
- [REST API](../chapter-10-api/README.md) - HTTP endpoints