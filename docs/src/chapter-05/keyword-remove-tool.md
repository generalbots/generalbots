# REMOVE_TOOL Keyword

**Syntax**

```
REMOVE_TOOL "tool-path.bas"
```

**Parameters**

- `"tool-path.bas"` – Relative path to a `.bas` file that was previously added with `ADD_TOOL`.

**Description**

`REMOVE_TOOL` disassociates a previously added tool from the current conversation session. After execution, the tool’s keywords are no longer available for invocation in the same dialog.

The keyword performs the following steps:

1. Extracts the tool name from the provided path (removing the `.bas` extension and any leading `.gbdialog/` prefix).
2. Validates that the tool name is not empty.
3. Spawns an asynchronous task that:
   - Deletes the corresponding row from `session_tool_associations` for the current session.
   - Returns a message indicating whether the tool was removed or was not active.

**Example**

```basic
REMOVE_TOOL "enrollment.bas"
TALK "Enrollment tool removed from this conversation."
```

If the `enrollment.bas` tool was active, it will be removed; otherwise the keyword reports that the tool was not active.

**Implementation Notes**

- The operation runs in a separate thread with its own Tokio runtime to avoid blocking the main engine.
- Errors during database deletion are logged and propagated as runtime errors.
