# Template Examples

The `templates` section showcases the official BASIC dialog templates shipped with GeneralBots. They are stored under `templates/` and can be referenced directly in dialogs via `ADD_TOOL`.

## start.bas

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

## auth.bas

```basic
REM Simple authentication flow
SET attempts = 0
LABEL auth_loop
HEAR password
IF password = "secret123" THEN
    TALK "Authentication successful."
ELSE
    SET attempts = attempts + 1
    IF attempts >= 3 THEN
        TALK "Too many attempts. Goodbye."
        EXIT
    ENDIF
    TALK "Incorrect password. Try again."
    GOTO auth_loop
ENDIF
```

## generate-summary.bas

```basic
REM Generates a summary using the LLM keyword
SET topic = "GeneralBots platform"
TALK "Generating summary for " + topic + "..."
SET summary = LLM "Summarize the following: " + topic
TALK summary
```

## enrollment.bas (Tool Example)

```basic
REM Demonstrates adding a custom tool to the conversation
ADD_TOOL "enrollment.bas"
TALK "Enrollment tool added. You can now use ENROLL command."
```

These templates illustrate common patterns: greeting, authentication, LLM integration, and tool registration. Users can copy and adapt them for their own bots.
