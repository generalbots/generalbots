# .gbdialog Dialogs

The `.gbdialog` package contains BASIC scripts that define conversation flows, tool integrations, and bot behavior.

## What is .gbdialog?

`.gbdialog` files are written in a specialized BASIC dialect that controls:
- Conversation flow and logic
- Tool calls and integrations
- User input processing
- Context management
- Response generation

## Basic Structure

A typical `.gbdialog` script contains:

```basic
REM This is a comment
TALK "Hello! How can I help you today?"

HEAR user_input

IF user_input = "help" THEN
    TALK "I can help you with various tasks..."
ELSE
    LLM user_input
END IF
```

## Key Components

### 1. Control Flow
- `IF/THEN/ELSE/END IF` for conditional logic
- `FOR EACH/IN/NEXT` for loops
- `EXIT FOR` to break loops

### 2. User Interaction
- `HEAR variable` to get user input
- `TALK message` to send responses
- `WAIT seconds` to pause execution

### 3. Data Manipulation
- `SET variable = value` for assignment
- `GET url` to fetch external data
- `FIND table, filter` to query databases

### 4. AI Integration
- `LLM prompt` for AI-generated responses
- `USE_TOOL tool_name` to enable functionality
- `USE_KB collection` to use knowledge bases

## Script Execution

Dialog scripts run in a sandboxed environment with:
- Access to session context and variables
- Ability to call external tools and APIs
- Integration with knowledge bases
- LLM generation capabilities

## Error Handling

The system provides built-in error handling:
- Syntax errors are caught during compilation
- Runtime errors log details but don't crash the bot
- Timeouts prevent infinite loops
- Resource limits prevent abuse
