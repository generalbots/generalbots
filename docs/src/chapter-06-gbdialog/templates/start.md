# start.bas (Template)

A minimal greeting and help flow to get users started.

```basic
REM Basic greeting and help flow
SET user_name = "Guest"
TALK "Hello, " + user_name + "! How can I help you today?"
HEAR user_input
IF user_input = "help" THEN
    TALK "Sure, I can assist with account info, orders, or support."
ELSE
    TALK "Sorry, I didn't understand."
ENDIF
```

**Purpose**

- Shows how to set a variable with `SET`.
- Uses `TALK` to send a message and `HEAR` to receive user input.
- Demonstrates simple branching with `IF/ELSE`.

**Keywords used:** `SET`, `TALK`, `HEAR`, `IF`, `ELSE`.

---
