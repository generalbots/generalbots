# Tool Definition

In BotServer, a **tool** is simply a `.bas` file. That's it!

## How It Works

1. **Create a `.bas` file** in your `.gbdialog/` folder
2. **The LLM automatically discovers it** and can call it when needed
3. **No manual registration required** - it just works!

## Simple Example

Create `get-weather.bas`:

```bas
' This tool gets weather information
' The LLM will call this when users ask about weather

TALK "Let me check the weather for you..."
weather = CALL "/api/weather", "San Francisco"
TALK "The weather is: " + weather
```

That's a tool! The LLM now knows it can call this when users ask about weather.

## Tool with Parameters

Create `send-email.bas`:

```bas
' Send an email to someone
PARAM to AS STRING
PARAM subject AS STRING
PARAM body AS STRING

CALL "/email/send", to, subject, body
TALK "Email sent to " + to
```

The `PARAM` declarations tell the LLM what parameters this tool accepts.

## Making Tools Available

### Method 1: Automatic Discovery (Default)

All `.bas` files in your `.gbdialog/` folder are automatically available.

```
mybot.gbai/
  mybot.gbdialog/
    start.bas           ← Entry point
    get-weather.bas     ← Tool (auto-discovered)
    send-email.bas      ← Tool (auto-discovered)
    create-task.bas     ← Tool (auto-discovered)
```

### Method 2: Manual Registration

In your `start.bas`, explicitly add tools:

```bas
' Register tools for this conversation
ADD_TOOL "get-weather"
ADD_TOOL "send-email"
ADD_TOOL "create-task"

TALK "Hello! I can help with weather, email, and tasks."
```

### Method 3: Dynamic Registration

Add tools based on context:

```bas
' In start.bas
TALK "What do you need help with?"
HEAR user_input

IF user_input CONTAINS "weather" THEN
    ADD_TOOL "get-weather"
    TALK "I've loaded the weather tool."
ENDIF

IF user_input CONTAINS "email" THEN
    ADD_TOOL "send-email"
    TALK "I can help with email now."
ENDIF
```

## Tool Format Conversion

BotServer automatically converts your `.bas` tools to:

- **MCP (Model Context Protocol)** format
- **OpenAI function calling** format
- Other LLM provider formats

You never write these formats manually - just write `.bas` files!

## Complete Example

Here's a real tool from the codebase - `enrollment.bas`:

```bas
' Student enrollment tool
PARAM student_name AS STRING
PARAM course AS STRING
PARAM email AS STRING

' Validate email
IF NOT email CONTAINS "@" THEN
    TALK "Please provide a valid email address."
    RETURN
ENDIF

' Create enrollment record
enrollment_id = CALL "/database/insert", "enrollments", {
    "student_name": student_name,
    "course": course,
    "email": email,
    "enrolled_at": NOW()
}

' Send confirmation email
CALL "/email/send", email, "Enrollment Confirmed", 
    "Welcome " + student_name + "! You're enrolled in " + course

TALK "Enrollment complete! Confirmation sent to " + email
```

## That's It!

To create a tool:
1. ✅ Create a `.bas` file
2. ✅ Add `PARAM` declarations if you need parameters
3. ✅ Write your logic using `TALK`, `HEAR`, `CALL`, etc.
4. ✅ Done!

The LLM will automatically:
- Discover your tool
- Understand what it does (from comments and code)
- Know when to call it
- Pass the right parameters

No JSON schemas, no manual registration, no complex configuration. Just write BASIC!

## Best Practices

### 1. Add Comments

The LLM reads your comments to understand the tool:

```bas
' This tool books a meeting room
' It checks availability and sends calendar invites
PARAM room_name AS STRING
PARAM date AS STRING
PARAM attendees AS ARRAY
```

### 2. Validate Parameters

Always validate input:

```bas
IF room_name IS NULL THEN
    TALK "Please specify which room you want to book."
    RETURN
ENDIF
```

### 3. Provide Feedback

Let users know what's happening:

```bas
TALK "Checking room availability..."
available = CALL "/calendar/check", room_name, date

IF available THEN
    TALK "Great! Booking the room now..."
    CALL "/calendar/book", room_name, date, attendees
    TALK "Meeting room booked successfully!"
ELSE
    TALK "Sorry, that room is not available on " + date
ENDIF
```

## Tool Discovery

The LLM discovers tools by:
1. **Reading `.bas` files** in your `.gbdialog/` folder
2. **Extracting comments** to understand purpose
3. **Parsing PARAM declarations** to understand parameters
4. **Building a function signature** automatically

Example tool discovery from `send-email.bas`:

```
Function: send-email
Description: Send an email to someone
Parameters:
  - to: STRING (required)
  - subject: STRING (required)  
  - body: STRING (required)
```

This is generated automatically from your `.bas` file!

## Removing Tools

```bas
' Remove a specific tool
REMOVE_TOOL "send-email"

' Clear all tools
CLEAR_TOOLS

' List active tools
tools = LIST_TOOLS
TALK "Available tools: " + tools
```

## Next Steps

- [PARAM Declaration](./param-declaration.md) - Parameter types and validation
- [GET Keyword Integration](./get-integration.md) - Using GET to call tools
- [External APIs](./external-apis.md) - Calling external services