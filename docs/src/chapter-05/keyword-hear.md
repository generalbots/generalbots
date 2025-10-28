# HEAR Keyword

**Syntax**

```
HEAR variable_name
```

**Parameters**

- `variable_name` – Identifier where the user’s next message will be stored.

**Description**

`HEAR` pauses script execution and waits for the next user input. The received text is assigned to the specified variable, which can then be used in subsequent commands.

**Example (from `start.bas`)**

```basic
HEAR user_input
IF user_input = "help" THEN
    TALK "Sure, I can assist with account info, orders, or support."
ENDIF
```

The script waits for the user to type a message, stores it in `user_input`, and then evaluates the condition.
