# HEAR Keyword

Waits for and captures user input, storing it in a variable.

## Syntax
```
HEAR variable_name
```

## Parameters
- `variable_name` - The name of the variable to store the user's input

## Description
The `HEAR` keyword pauses script execution and waits for the user to provide input. When the user sends a message, it is stored in the specified variable and script execution continues.

## Examples

### Basic Usage
```basic
TALK "What is your name?"
HEAR user_name
TALK "Hello, " + user_name + "!"
```

### With Validation
```basic
TALK "Please enter your email address:"
HEAR user_email

IF user_email CONTAINS "@" THEN
    TALK "Thank you!"
ELSE
    TALK "That doesn't look like a valid email. Please try again."
    HEAR user_email
END IF
```

## Usage Notes

- Script execution pauses at HEAR until user provides input
- The variable is created if it doesn't exist
- User input is stored as a string
- Multiple HEAR commands can be used in sequence
- Timeouts may occur if user doesn't respond within configured limits

## Session State

While waiting for HEAR input:
- The session is marked as "waiting for input"
- Other messages from the user go to the HEAR variable
- Tool execution is paused
- Context is preserved

## Error Handling

- If the user doesn't respond within the timeout, the variable may be empty
- The script continues execution with whatever input was received
- No runtime error occurs for missing input

## Related Keywords
- `TALK` - Send message to user
- `WAIT` - Pause without expecting input
- `SET` - Assign values without user input
