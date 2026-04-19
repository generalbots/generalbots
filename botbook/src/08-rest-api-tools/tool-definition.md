# Tool Definition

In botserver, a **tool** is simply a `.bas` file. That's it!

## How It Works

1. **Create a `.bas` file** in your `.gbdialog/` folder
2. **The LLM automatically discovers it** and can call it when needed
3. **No manual registration required** - it just works!

### Tool Discovery and Execution Flow

<svg width="700" height="600" viewBox="0 0 700 600" xmlns="http://www.w3.org/2000/svg" style="background: transparent;">
  <!-- Title -->
  <text x="350" y="25" text-anchor="middle" font-family="system-ui, -apple-system, sans-serif" font-size="16" font-weight="300" fill="currentColor" opacity="0.9">LLM Tool Discovery and Execution Pipeline</text>
  
  <!-- User Input -->
  <rect x="200" y="50" width="300" height="40" fill="none" stroke="currentColor" stroke-width="1.5" rx="8" opacity="0.8"/>
  <text x="350" y="75" text-anchor="middle" font-family="system-ui" font-size="12" fill="currentColor">"Send an email to John about the meeting"</text>
  
  <!-- Arrow down -->
  <path d="M 350 90 L 350 120" stroke="currentColor" stroke-width="2" fill="none" marker-end="url(#arrow1)" opacity="0.6"/>
  
  <!-- LLM Analysis -->
  <rect x="250" y="120" width="200" height="50" fill="none" stroke="currentColor" stroke-width="1.5" rx="8" opacity="0.8"/>
  <text x="350" y="140" text-anchor="middle" font-family="system-ui" font-size="13" font-weight="500" fill="currentColor">LLM Analyzes</text>
  <text x="350" y="158" text-anchor="middle" font-family="system-ui" font-size="11" fill="currentColor" opacity="0.7">"Need email tool"</text>
  
  <!-- Arrow down -->
  <path d="M 350 170 L 350 200" stroke="currentColor" stroke-width="2" fill="none" marker-end="url(#arrow1)" opacity="0.6"/>
  
  <!-- Tool Discovery -->
  <rect x="150" y="200" width="400" height="80" fill="none" stroke="currentColor" stroke-width="1.5" rx="8" opacity="0.8"/>
  <text x="350" y="220" text-anchor="middle" font-family="system-ui" font-size="13" font-weight="500" fill="currentColor">Tool Discovery</text>
  
  <!-- Inner box for tools list -->
  <rect x="240" y="230" width="220" height="40" fill="none" stroke="currentColor" stroke-width="1" rx="4" opacity="0.5" stroke-dasharray="3,2"/>
  <text x="350" y="246" text-anchor="middle" font-family="monospace" font-size="10" fill="currentColor" opacity="0.8">Scan .gbdialog/</text>
  <text x="350" y="260" text-anchor="middle" font-family="monospace" font-size="9" fill="currentColor" opacity="0.7">• send-email.bas ✓ • create-task.bas • get-weather.bas</text>
  
  <!-- Arrow down -->
  <path d="M 350 280 L 350 310" stroke="currentColor" stroke-width="2" fill="none" marker-end="url(#arrow1)" opacity="0.6"/>
  
  <!-- Parameter Collection -->
  <rect x="200" y="310" width="300" height="70" fill="none" stroke="currentColor" stroke-width="1.5" rx="8" opacity="0.8"/>
  <text x="350" y="330" text-anchor="middle" font-family="system-ui" font-size="13" font-weight="500" fill="currentColor">Parameter Collection</text>
  <text x="350" y="350" text-anchor="middle" font-family="monospace" font-size="10" fill="currentColor" opacity="0.7">to → "John"</text>
  <text x="350" y="364" text-anchor="middle" font-family="monospace" font-size="10" fill="currentColor" opacity="0.7">subject → "Meeting"</text>
  <text x="350" y="378" text-anchor="middle" font-family="monospace" font-size="10" fill="currentColor" opacity="0.7">body → (generated)</text>
  
  <!-- Arrow down -->
  <path d="M 350 380 L 350 410" stroke="currentColor" stroke-width="2" fill="none" marker-end="url(#arrow1)" opacity="0.6"/>
  
  <!-- Execute Tool -->
  <rect x="250" y="410" width="200" height="50" fill="none" stroke="currentColor" stroke-width="1.5" rx="8" opacity="0.8"/>
  <text x="350" y="430" text-anchor="middle" font-family="system-ui" font-size="13" font-weight="500" fill="currentColor">Execute Tool</text>
  <text x="350" y="448" text-anchor="middle" font-family="monospace" font-size="11" fill="currentColor" opacity="0.7">send-email.bas</text>
  
  <!-- Arrow down -->
  <path d="M 350 460 L 350 490" stroke="currentColor" stroke-width="2" fill="none" marker-end="url(#arrow1)" opacity="0.6"/>
  
  <!-- Return Result -->
  <rect x="250" y="490" width="200" height="50" fill="none" stroke="currentColor" stroke-width="1.5" rx="8" opacity="0.8"/>
  <text x="350" y="510" text-anchor="middle" font-family="system-ui" font-size="13" font-weight="500" fill="currentColor">Return Result</text>
  <text x="350" y="528" text-anchor="middle" font-family="system-ui" font-size="11" fill="currentColor" opacity="0.7">"Email sent!"</text>
  
  <!-- Arrow marker -->
  <defs>
    <marker id="arrow1" markerWidth="10" markerHeight="10" refX="9" refY="5" orient="auto">
      <polygon points="0 0, 10 5, 0 10" fill="currentColor" opacity="0.6"/>
    </marker>
  </defs>
</svg>

## Simple Example

Create `get-weather.bas`:

```basic
' This tool gets weather information
' The LLM will call this when users ask about weather

TALK "Let me check the weather for you..."
weather = GET "/api/weather/San Francisco"
TALK "The weather is: " + weather
```

That's a tool! The LLM now knows it can call this when users ask about weather.

## Tool with Parameters

Create `send-email.bas`:

```basic
' Send an email to someone
PARAM to AS STRING
PARAM subject AS STRING
PARAM body AS STRING

GET "/email/send" WITH to, subject, body
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

```basic
' Register tools for this conversation
USE TOOL "get-weather"
USE TOOL "send-email"
USE TOOL "create-task"

TALK "Hello! I can help with weather, email, and tasks."
```

### Method 3: LLM-Driven Tool Selection

Let the LLM decide which tools to use naturally:

```basic
' In start.bas
' Load all available tools - LLM decides when to use them
USE TOOL "weather"
USE TOOL "email"
USE TOOL "enrollment"

TALK "I can help with various tasks. What do you need?"
' The LLM will automatically call the right tool based on user intent
```

## Tool Format Conversion

botserver automatically converts your `.bas` tools to:

- **MCP (Model Context Protocol)** format
- **Groq/OpenAI-compatible function calling** format
- Other LLM provider formats

You never write these formats manually - just write `.bas` files!

### Conversion Pipeline

<svg width="600" height="400" viewBox="0 0 600 400" xmlns="http://www.w3.org/2000/svg" style="background: transparent;">
  <!-- Title -->
  <text x="300" y="25" text-anchor="middle" font-family="system-ui, -apple-system, sans-serif" font-size="14" font-weight="300" fill="currentColor" opacity="0.9">Tool Format Conversion Pipeline</text>
  
  <!-- Source file -->
  <rect x="220" y="50" width="160" height="35" fill="none" stroke="currentColor" stroke-width="1.5" rx="6" opacity="0.8"/>
  <text x="300" y="72" text-anchor="middle" font-family="monospace" font-size="12" fill="currentColor">send-email.bas</text>
  
  <!-- Arrow down -->
  <path d="M 300 85 L 300 110" stroke="currentColor" stroke-width="2" fill="none" marker-end="url(#arrow2)" opacity="0.6"/>
  
  <!-- BASIC Parser -->
  <rect x="200" y="110" width="200" height="70" fill="none" stroke="currentColor" stroke-width="1.5" rx="8" opacity="0.8"/>
  <text x="300" y="130" text-anchor="middle" font-family="system-ui" font-size="12" font-weight="500" fill="currentColor">BASIC Parser</text>
  <text x="300" y="148" text-anchor="middle" font-family="system-ui" font-size="10" fill="currentColor" opacity="0.7">• Extract PARAM</text>
  <text x="300" y="162" text-anchor="middle" font-family="system-ui" font-size="10" fill="currentColor" opacity="0.7">• Parse DESCRIPTION</text>
  <text x="300" y="176" text-anchor="middle" font-family="system-ui" font-size="10" fill="currentColor" opacity="0.7">• Analyze code</text>
  
  <!-- Branching arrows -->
  <path d="M 300 180 L 300 200 L 120 200 L 120 230" stroke="currentColor" stroke-width="1.5" fill="none" marker-end="url(#arrow2)" opacity="0.5"/>
  <path d="M 300 180 L 300 200 L 240 200 L 240 230" stroke="currentColor" stroke-width="1.5" fill="none" marker-end="url(#arrow2)" opacity="0.5"/>
  <path d="M 300 180 L 300 200 L 360 200 L 360 230" stroke="currentColor" stroke-width="1.5" fill="none" marker-end="url(#arrow2)" opacity="0.5"/>
  <path d="M 300 180 L 300 200 L 480 200 L 480 230" stroke="currentColor" stroke-width="1.5" fill="none" marker-end="url(#arrow2)" opacity="0.5"/>
  
  <!-- Format boxes -->
  <rect x="70" y="230" width="100" height="40" fill="none" stroke="currentColor" stroke-width="1" rx="6" opacity="0.7"/>
  <text x="120" y="254" text-anchor="middle" font-family="system-ui" font-size="11" fill="currentColor">MCP Format</text>
  
  <rect x="190" y="230" width="100" height="40" fill="none" stroke="currentColor" stroke-width="1" rx="6" opacity="0.7"/>
  <text x="240" y="254" text-anchor="middle" font-family="system-ui" font-size="11" fill="currentColor">OpenAI Function</text>
  
  <rect x="310" y="230" width="100" height="40" fill="none" stroke="currentColor" stroke-width="1" rx="6" opacity="0.7"/>
  <text x="360" y="254" text-anchor="middle" font-family="system-ui" font-size="11" fill="currentColor">Claude Tool</text>
  
  <rect x="430" y="230" width="100" height="40" fill="none" stroke="currentColor" stroke-width="1" rx="6" opacity="0.7"/>
  <text x="480" y="254" text-anchor="middle" font-family="system-ui" font-size="11" fill="currentColor">Local Model</text>
  
  <!-- Converging arrows -->
  <path d="M 120 270 L 120 290 L 300 290 L 300 310" stroke="currentColor" stroke-width="1.5" fill="none" opacity="0.5"/>
  <path d="M 240 270 L 240 290 L 300 290" stroke="currentColor" stroke-width="1.5" fill="none" opacity="0.5"/>
  <path d="M 360 270 L 360 290 L 300 290" stroke="currentColor" stroke-width="1.5" fill="none" opacity="0.5"/>
  <path d="M 480 270 L 480 290 L 300 290 L 300 310" stroke="currentColor" stroke-width="1.5" fill="none" marker-end="url(#arrow2)" opacity="0.5"/>
  
  <!-- LLM Provider -->
  <rect x="200" y="310" width="200" height="50" fill="none" stroke="currentColor" stroke-width="1.5" rx="8" opacity="0.8"/>
  <text x="300" y="330" text-anchor="middle" font-family="system-ui" font-size="12" font-weight="500" fill="currentColor">LLM Provider</text>
  <text x="300" y="348" text-anchor="middle" font-family="system-ui" font-size="10" fill="currentColor" opacity="0.7">Receives Native Format</text>
  
  <!-- Arrow marker -->
  <defs>
    <marker id="arrow2" markerWidth="8" markerHeight="8" refX="7" refY="4" orient="auto">
      <polygon points="0 0, 8 4, 0 8" fill="currentColor" opacity="0.6"/>
    </marker>
  </defs>
</svg>

## Complete Example

Here's a real tool from the codebase - `enrollment.bas`:

```basic
PARAM name AS string          LIKE "Abreu Silva"                DESCRIPTION "Required full name of the individual."
PARAM birthday AS date        LIKE "23/09/2001"                 DESCRIPTION "Required birth date of the individual in DD/MM/YYYY format."
PARAM email AS string         LIKE "abreu.silva@example.com"    DESCRIPTION "Required email address for contact purposes."
PARAM personalid AS integer   LIKE "12345678900"                DESCRIPTION "Required Personal ID number of the individual (only numbers)."
PARAM address AS string       LIKE "Rua das Flores, 123 - SP"   DESCRIPTION "Required full address of the individual."

DESCRIPTION  "This is the enrollment process, called when the user wants to enroll. Once all information is collected, confirm the details and inform them that their enrollment request has been successfully submitted."

' The actual tool logic is simple
SAVE "enrollments.csv", id, name, birthday, email, personalid, address
TALK "Successfully enrolled " + name + "!"

' That's it! The LLM handles:
' - Natural conversation to collect parameters
' - Validation and error handling  
' - Confirming details with the user
' - All the complex interaction flow
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

```basic
' This tool books a meeting room
' It checks availability and sends calendar invites
PARAM room_name AS STRING
PARAM date AS STRING
PARAM attendees AS ARRAY
```

### 2. Validate Parameters

Always validate input:

```basic
IF room_name IS NULL THEN
    TALK "Please specify which room you want to book."
    RETURN
ENDIF
```

### 3. Provide Feedback

Let users know what's happening:

```basic
TALK "Checking room availability..."
available = GET "/calendar/check" WITH room_name, date

IF available THEN
    TALK "Great! Booking the room now..."
    GET "/calendar/book" WITH room_name, date, attendees
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
### Dynamic Tool Management

```basic
' Remove a specific tool
REMOVE TOOL "send-email"

' Clear all tools
CLEAR TOOLS

' List active tools
tools = LIST TOOLS
TALK "Available tools: " + tools
```

## Next Steps

- [PARAM Declaration](./param-declaration.md) - Parameter types and validation
- [GET Keyword Integration](./get-integration.md) - Using GET to call tools
- [External APIs](./external-apis.md) - Calling external services