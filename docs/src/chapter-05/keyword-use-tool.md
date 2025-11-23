# USE TOOL

## Syntax

```basic
USE TOOL tool-name
```

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| tool-name | String | Name of the tool to load (without .bas extension) |

## Description

Loads a tool definition and makes it available to the LLM for the current session. Tools extend the bot's capabilities with specific functions like calculations, API calls, or data processing.

## Examples

### Basic Usage

```basic
' Load weather tool
USE TOOL "weather"

' Now LLM can use weather functions
answer = LLM "What's the weather in Tokyo?"
TALK answer
```

### Multiple Tools

```basic
' Load several tools
USE TOOL "calculator"
USE TOOL "translator"
USE TOOL "date-time"

' LLM has access to all loaded tools
response = LLM "Calculate 15% tip on $45.80 and translate to Spanish"
TALK response
```

### Conditional Loading

```basic
user_type = GET "user_type"

IF user_type = "admin" THEN
    USE TOOL "admin-functions"
    USE TOOL "database-query"
ELSE
    USE TOOL "basic-search"
END IF
```

### With Error Handling

```basic
tool_needed = "advanced-analytics"

IF FILE EXISTS tool_needed + ".bas" THEN
    USE TOOL tool_needed
    TALK "Analytics tool loaded"
ELSE
    TALK "Advanced features not available"
END IF
```

## Tool Definition Format

Tools are defined as BASIC scripts with PARAM declarations:

```basic
' weather.bas
PARAM location AS string LIKE "Tokyo" DESCRIPTION "City name"
DESCRIPTION "Get current weather for a location"

' Tool logic here
temp = GET_TEMPERATURE(location)
conditions = GET_CONDITIONS(location)
result = location + ": " + temp + "°, " + conditions
RETURN result
```

## Notes

- Tools remain active for the entire session
- Use CLEAR TOOLS to remove all loaded tools
- Tool names should be descriptive
- Tools are loaded from the .gbdialog/tools/ directory
- Maximum 10 tools can be active simultaneously

## Related

- [CLEAR TOOLS](./keyword-clear-tools.md)
- [Tool Definition](../chapter-08/tool-definition.md)
- [PARAM Declaration](../chapter-08/param-declaration.md)