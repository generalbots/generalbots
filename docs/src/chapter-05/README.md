# BASIC Dialogs

BotServer uses a simplified version of BASIC (Beginner's All-purpose Symbolic Instruction Code) as its primary scripting language for creating conversational flows. This chapter covers everything you need to know about writing BASIC dialogs for your bots.

## Why BASIC?

BASIC was chosen for BotServer because:
- **Simplicity**: Easy to learn, even for non-programmers
- **Readability**: Clear, English-like syntax
- **Proven**: Decades of use in automation and scripting
- **Extensible**: Easy to add custom keywords
- **Accessible**: No complex programming concepts required

## Core Concepts

### Scripts vs Tools

**Scripts** (`.bas` files in `.gbdialog/`):
- Define conversation flows
- Handle user interactions
- Manage bot behavior
- Run sequentially

**Tools** (`.bas` files in `.gbdialog/tools/`):
- Callable functions for the LLM
- Reusable logic modules
- Parameter support
- Described for AI understanding

### Execution Model

BASIC scripts in BotServer:
1. Start with `start.bas` for each new session
2. Execute line by line
3. Can call other scripts with `RUN`
4. Can load tools with `USE TOOL`
5. Support event handlers with `ON`

## Language Features

### Variables
```basic
' No declaration needed
name = "John"
age = 25
is_valid = TRUE

' String concatenation
message = "Hello, " + name

' Arrays
items = ["apple", "banana", "orange"]
```

### Control Flow
```basic
' IF statements
IF age >= 18 THEN
    TALK "You are an adult"
ELSE
    TALK "You are a minor"
END IF

' FOR loops
FOR i = 1 TO 10
    TALK "Count: " + STR(i)
NEXT

' FOR EACH loops
FOR EACH item IN items
    TALK "Item: " + item
NEXT

' WHILE loops
WHILE counter < 10
    counter = counter + 1
WEND
```

### Functions
```basic
FUNCTION CalculateTotal(price, quantity)
    total = price * quantity
    RETURN total
END FUNCTION

' Call the function
result = CalculateTotal(10.50, 3)
```

## Conversation Keywords

### Basic Interaction
- **TALK**: Send message to user
- **HEAR**: Wait for user input
- **WAIT**: Pause execution

### Suggestions
- **ADD SUGGESTION**: Add quick reply button
- **CLEAR SUGGESTIONS**: Remove all buttons

### Knowledge Base
- **USE KB**: Load knowledge base
- **CLEAR KB**: Unload knowledge base
- **FIND**: Search in loaded KBs

### Tools
- **USE TOOL**: Make tool available to LLM
- **CLEAR TOOLS**: Remove all tools

### Memory
- **SET**: Store session variable
- **GET**: Retrieve session variable
- **SET BOT MEMORY**: Store persistent data
- **GET BOT MEMORY**: Retrieve persistent data

### Context
- **SET CONTEXT**: Add context for LLM
- **SET USER**: Set user information

### Scheduling
- **SET SCHEDULE**: Schedule recurring task

### Communication
- **SEND MAIL**: Send email
- **SEND FILE**: Send file attachment

### AI Integration
- **LLM**: Query language model

## Writing Effective Dialogs

### 1. Start Simple
```basic
' start.bas - Minimal bot
TALK "Hello! How can I help you?"
answer = HEAR
response = LLM "Answer this question: " + answer
TALK response
```

### 2. Add Structure
```basic
' start.bas - Structured conversation
CLEAR SUGGESTIONS
ADD SUGGESTION "Sales" AS "sales"
ADD SUGGESTION "Support" AS "support"
ADD SUGGESTION "Other" AS "other"

choice = HEAR "What brings you here today?"

IF choice = "sales" THEN
    RUN "sales_flow.bas"
ELSE IF choice = "support" THEN
    RUN "support_flow.bas"
ELSE
    RUN "general_help.bas"
END IF
```

### 3. Use Context
```basic
' Load relevant information
USE KB "product_catalog"
USE KB "pricing"

' Set context for better responses
user_type = GET "user_type"
SET CONTEXT "customer_type" AS user_type

' Now LLM has access to KBs and context
answer = LLM "What products match this need: " + user_request
```

### 4. Handle Errors
```basic
TRY
    result = CALL_API(endpoint, data)
    TALK "Success: " + result
CATCH
    TALK "Sorry, something went wrong. Please try again."
    LOG ERROR_MESSAGE
END TRY
```

## Best Practices

### 1. Keep Scripts Focused
Each script should handle one specific flow or feature.

### 2. Use Meaningful Names
```basic
' Good
customer_name = HEAR "What's your name?"

' Bad
x = HEAR "What's your name?"
```

### 3. Add Comments
```basic
' Check if user is authenticated
auth_status = GET SESSION "authenticated"
IF auth_status <> "true" THEN
    ' Redirect to login flow
    RUN "auth.bas"
END IF
```

### 4. Validate Input
```basic
age = HEAR "Please enter your age:"
IF NOT IS_NUMERIC(age) THEN
    TALK "Please enter a valid number"
    ' Ask again
ELSE IF VAL(age) < 0 OR VAL(age) > 120 THEN
    TALK "Please enter a realistic age"
END IF
```

### 5. Provide Feedback
```basic
TALK "Processing your request..."
' Long operation
result = PROCESS_DATA()
TALK "Complete! Here's your result: " + result
```

## Common Patterns

### Menu System
```basic
FUNCTION ShowMenu()
    CLEAR SUGGESTIONS
    ADD SUGGESTION "Option 1" AS "1"
    ADD SUGGESTION "Option 2" AS "2"
    ADD SUGGESTION "Back" AS "back"
    
    choice = HEAR "Select an option:"
    RETURN choice
END FUNCTION
```

### Data Collection
```basic
' Collect user information
name = HEAR "What's your name?"
SET "name", name

email = HEAR "What's your email?"
WHILE NOT IS_VALID_EMAIL(email)
    email = HEAR "Please enter a valid email:"
WEND
SET "email", email
```

### Confirmation
```basic
FUNCTION Confirm(message)
    CLEAR SUGGESTIONS
    ADD SUGGESTION "Yes" AS "yes"
    ADD SUGGESTION "No" AS "no"
    
    answer = HEAR message
    RETURN answer = "yes"
END FUNCTION

IF Confirm("Do you want to proceed?") THEN
    ' Continue
ELSE
    ' Cancel
END IF
```

## Debugging

### Debug Output
```basic
DEBUG "Variable value: " + variable
DEBUG SHOW SESSION
DEBUG SHOW CONTEXT
```

### Logging
```basic
LOG "User selected: " + choice
LOG ERROR "Failed to process: " + error_message
```

### Testing
```basic
' Test mode flag
test_mode = GET BOT MEMORY "test_mode"
IF test_mode = "true" THEN
    ' Use test data
    api_url = "https://test-api.example.com"
ELSE
    ' Use production data
    api_url = "https://api.example.com"
END IF
```

## Next Steps

- [BASIC Syntax Reference](./basics.md) - Complete language reference
- [Keywords Documentation](./keywords.md) - All available keywords
- [Real Examples](./real-basic-examples.md) - Production-ready scripts
- [Templates](./templates.md) - Common dialog templates

## Summary

BASIC in BotServer provides a powerful yet simple way to create conversational AI applications. Its English-like syntax, combined with powerful keywords for AI integration, makes it accessible to both programmers and non-programmers alike. Start simple, iterate quickly, and build sophisticated bots without complex code.