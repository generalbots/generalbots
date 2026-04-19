# CLEAR TOOLS Keyword

**Syntax**

```
CLEAR TOOLS
```

**Parameters**

_None_ – This keyword takes no arguments.

**Description**

`CLEAR TOOLS` removes every tool that has been added to the current conversation session. It clears the list of active tools stored in the session‑tool association table, effectively resetting the tool environment for the dialog. After execution, no previously added tools (via `USE TOOL`) remain available.

**Example**

```basic
USE TOOL "enrollment.bas"
TALK "Enrollment tool added."
CLEAR TOOLS
TALK "All tools have been cleared from this conversation."
```

After `CLEAR TOOLS` runs, the `enrollment.bas` tool is no longer accessible in the same session.
