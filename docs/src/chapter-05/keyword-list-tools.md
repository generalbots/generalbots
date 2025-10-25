# LIST_TOOLS Keyword

**Syntax**

```
LIST_TOOLS
```

**Parameters**

_None_ – This keyword takes no arguments.

**Description**

`LIST_TOOLS` returns a formatted string that lists all tools currently associated with the active conversation session. The list includes each tool’s name and its order of addition. If no tools are active, the keyword returns a message indicating that the tool set is empty.

**Example**

```basic
ADD_TOOL "enrollment.bas"
ADD_TOOL "weather.bas"
SET tools = LIST_TOOLS
TALK tools
```

Possible output:

```
Active tools in this conversation (2):
1. enrollment
2. weather
```

If no tools have been added:

```
No tools are currently active in this conversation.
```

**Implementation Notes**

- The keyword queries the `session_tool_associations` table for the current session ID.
- The result is a plain text string; it can be directly passed to `TALK` or stored in a variable.
- Errors during database access are logged and result in a runtime error.
