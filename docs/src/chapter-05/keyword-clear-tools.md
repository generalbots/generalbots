# CLEAR_TOOLS Keyword

**Syntax**

```
CLEAR_TOOLS
```

**Parameters**

_None_ – This keyword takes no arguments.

**Description**

`CLEAR_TOOLS` removes every tool that has been added to the current conversation session. It clears the list of active tools stored in the session‑tool association table, effectively resetting the tool environment for the dialog. After execution, no previously added tools (via `ADD_TOOL`) remain available.

**Example**

```basic
ADD_TOOL "enrollment.bas"
TALK "Enrollment tool added."
CLEAR_TOOLS
TALK "All tools have been cleared from this conversation."
```

After `CLEAR_TOOLS` runs, the `enrollment.bas` tool is no longer accessible in the same session.
