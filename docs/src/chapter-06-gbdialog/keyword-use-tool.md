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

' Now system AI can use weather functions during conversations
TALK "What weather information would you like?"
' System AI automatically uses the tool when needed
```

### Multiple Tools

```basic
' Load several tools
USE TOOL "calculator"
USE TOOL "translator"
USE TOOL "date-time"

' System AI has access to all loaded tools during conversations
TALK "I can help you with calculations, translations, and date/time information."
' System AI automatically uses the appropriate tools when needed
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

## Related

- [CLEAR TOOLS](./keyword-clear-tools.md)
- [Tool Definition](../chapter-08/tool-definition.md)
- [PARAM Declaration](../chapter-08/param-declaration.md)
