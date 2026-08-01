# USE TOOL 🟡 BETA

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

## Admin-Only Tools (Role Gate)

Tools can be restricted to administrators declaratively through the bot script itself — the `USE TOOL` keyword is the authorization point:

- `USE TOOL` associates the tool with the current session (`session_tool_associations`)
- The server only executes a tool when it was associated with that session — `run_llm_tool_call` and `run_tool_exec` (TOOL_EXEC / message_type 6) both verify the association before running
- There is no `admin_only` flag and no hardcoded tool list on the server; the script is the single source of truth

### Runtime `role` Variable

Each BASIC script has access to the runtime variable `role`, which is resolved from the user's RBAC groups (`rbac_user_groups` → `rbac_groups`). It is `"admin"` when the user belongs to a group whose name contains `admin`, otherwise `"user"`.

```basic
' Only register admin tools for administrators
IF role = "admin" THEN
    USE TOOL "chart-batizados"
    USE TOOL "pendencias"
    USE TOOL "revisar-pendencias"
END IF
```

Suggestions for private tools can also be gated the same way:

```basic
IF role = "admin" THEN
    ADD SUGGESTION TOOL "chart-batizados" as "Grafico Batizados"
END IF
```

### Why this is secure

Even if a non-admin session somehow sends a `TOOL_EXEC` message with an admin tool name, the server checks `session_tool_associations` first: since `start.bas` only ran `USE TOOL` for admins, the tool is not associated with the non-admin session and the execution is skipped (logged as `tool '...' not associated with session ...`).

## Related

- [CLEAR TOOLS](./keyword-clear-tools.md)
- [Tool Definition](../08-rest-api-tools/tool-definition.md)
- [PARAM Declaration](../08-rest-api-tools/param-declaration.md)
