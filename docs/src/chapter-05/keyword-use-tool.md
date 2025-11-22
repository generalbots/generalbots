# USE_TOOL Keyword

**Syntax**

```
USE_TOOL "tool-path.bas"
```

**Parameters**

- `"tool-path.bas"` – Relative path to a `.bas` file inside the `.gbdialog` package (e.g., `enrollment.bas`).

**Description**

`USE_TOOL` compiles the specified BASIC script and registers it as a tool for the current session. The compiled tool becomes available for use in the same conversation, allowing its keywords to be invoked.

The keyword performs the following steps:

1. Extracts the tool name from the provided path (removing the `.bas` extension and any leading `.gbdialog/` prefix).
2. Validates that the tool name is not empty.
3. Spawns an asynchronous task that:
   - Checks that the tool exists and is active for the bot in the `basic_tools` table.
   - Inserts a row into `session_tool_associations` linking the tool to the current session (or does nothing if the association already exists).
4. Returns a success message indicating the tool is now available, or an error if the tool cannot be found or the database operation fails.

**Example**

```basic
USE_TOOL "enrollment.bas"
TALK "Enrollment tool added. You can now use ENROLL command."
```

After execution, the `enrollment.bas` script is compiled and its keywords become callable in the current dialog.

**Implementation Notes**

- The operation runs in a separate thread with its own Tokio runtime to avoid blocking the main engine.
- Errors are logged and propagated as runtime errors in the BASIC script.
