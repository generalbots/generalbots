# USE_TOOL

Add and activate a tool (custom dialog script) for the current conversation.

## Syntax

```basic
USE_TOOL "tool-name.bas"
```

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `tool-name.bas` | String | Path to the tool's BASIC script file |

## Description

The `USE_TOOL` keyword dynamically loads a tool definition from a `.bas` file and makes its functionality available in the current conversation. Tools are reusable dialog scripts that extend the bot's capabilities with custom functions, API integrations, or specialized workflows.

Once loaded, the tool's keywords and functions become available for use in the conversation until the session ends or the tool is explicitly cleared.

## Examples

### Load a Simple Tool
```basic
USE_TOOL "weather.bas"
' Now weather functions are available
result = GET_WEATHER("New York")
TALK result
```

### Load Multiple Tools
```basic
USE_TOOL "calculator.bas"
USE_TOOL "translator.bas"
USE_TOOL "scheduler.bas"

' All three tools are now active
sum = CALCULATE("150 + 250")
translated = TRANSLATE(sum, "Spanish")
SCHEDULE_REMINDER(translated, "tomorrow")
```

### Conditional Tool Loading
```basic
task = HEAR "What would you like to do?"
IF task CONTAINS "email" THEN
    USE_TOOL "email-composer.bas"
ELSE IF task CONTAINS "calendar" THEN
    USE_TOOL "calendar-manager.bas"
ELSE IF task CONTAINS "document" THEN
    USE_TOOL "document-processor.bas"
END IF
```

### Tool with Parameters
```basic
' Some tools accept configuration
USE_TOOL "api-client.bas"
CONFIGURE_API("https://api.example.com", api_key)
response = CALL_API("GET", "/users")
```

## Tool Structure

Tools are BASIC scripts that define:
- **Functions**: Reusable operations
- **Keywords**: Custom commands
- **Integrations**: API connections
- **Workflows**: Multi-step processes

Example tool file (`calculator.bas`):
```basic
FUNCTION CALCULATE(expression)
    result = EVAL(expression)
    RETURN result
END FUNCTION

FUNCTION PERCENTAGE(value, percent)
    RETURN value * percent / 100
END FUNCTION
```

## Tool Discovery

Tools are discovered from:
1. `.gbdialog/tools/` directory in bot package
2. System tools directory (`/opt/tools/`)
3. User tools directory (`~/.gbtools/`)
4. Inline tool definitions

## Return Value

Returns a status object:
- `success`: Boolean indicating if tool loaded
- `tool_name`: Name of the loaded tool
- `functions_added`: List of new functions available
- `error`: Error message if loading failed

## Tool Compilation

When a tool is loaded:
1. Script is parsed and validated
2. Functions are compiled to MCP format
3. OpenAI function format generated
4. Tool registered in session context
5. Functions become callable

## Session Scope

- Tools are session-specific
- Don't affect other conversations
- Automatically unloaded on session end
- Can be manually removed with `CLEAR_TOOLS`

## Error Handling

```basic
TRY
    USE_TOOL "advanced-tool.bas"
    TALK "Tool loaded successfully"
CATCH "tool_not_found"
    TALK "Tool file doesn't exist"
CATCH "compilation_error"
    TALK "Tool has syntax errors"
CATCH "permission_denied"
    TALK "Not authorized to use this tool"
END TRY
```

## Best Practices

1. **Load Tools Early**: Load at conversation start when possible
2. **Check Dependencies**: Ensure required services are available
3. **Handle Failures**: Always have fallback behavior
4. **Document Tools**: Include usage comments in tool files
5. **Version Tools**: Use version numbers in tool names
6. **Test Thoroughly**: Validate tools before deployment
7. **Limit Tool Count**: Don't load too many tools at once

## Tool Management

### List Active Tools
```basic
tools = GET_ACTIVE_TOOLS()
FOR EACH tool IN tools
    TALK "Active tool: " + tool.name
NEXT
```

### Check Tool Status
```basic
IF IS_TOOL_ACTIVE("calculator.bas") THEN
    result = CALCULATE("2+2")
ELSE
    USE_TOOL "calculator.bas"
END IF
```

### Tool Versioning
```basic
' Load specific version
USE_TOOL "reporter-v2.bas"

' Check version
version = GET_TOOL_VERSION("reporter")
IF version < 2 THEN
    CLEAR_TOOLS()
    USE_TOOL "reporter-v2.bas"
END IF
```

## Advanced Features

### Tool Chaining
```basic
USE_TOOL "data-fetcher.bas"
data = FETCH_DATA(source)

USE_TOOL "data-processor.bas"
processed = PROCESS_DATA(data)

USE_TOOL "report-generator.bas"
report = GENERATE_REPORT(processed)
```

### Dynamic Tool Creation
```basic
' Create tool from template
CREATE_TOOL_FROM_TEMPLATE("custom-api", api_config)
USE_TOOL "custom-api.bas"
```

### Tool Permissions
```basic
IF HAS_PERMISSION("admin-tools") THEN
    USE_TOOL "admin-console.bas"
ELSE
    TALK "Admin tools require elevated permissions"
END IF
```

## Performance Considerations

- Tool compilation happens once per session
- Compiled tools are cached
- Large tools may increase memory usage
- Consider lazy loading for complex tools

## Troubleshooting

### Tool Not Loading
- Check file path is correct
- Verify `.bas` extension
- Ensure file has read permissions
- Check for syntax errors in tool

### Functions Not Available
- Confirm tool loaded successfully
- Check function names match exactly
- Verify no naming conflicts
- Review tool compilation logs

### Performance Issues
- Limit number of active tools
- Use lighter tool versions
- Consider tool optimization
- Check for infinite loops in tools

## Related Keywords

- [CLEAR_TOOLS](./keyword-clear-tools.md) - Remove all tools
- [GET](./keyword-get.md) - Often used within tools
- [LLM](./keyword-llm.md) - Tools can enhance LLM capabilities
- [FORMAT](./keyword-format.md) - Format tool outputs

## Implementation

Located in `src/basic/keywords/use_tool.rs`

Integrates with the tool compiler to dynamically load and compile BASIC tool scripts into callable functions.