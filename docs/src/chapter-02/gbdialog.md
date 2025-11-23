# .gbdialog Dialogs

The `.gbdialog` package contains BASIC scripts that define conversation flows, tool integrations, and bot behavior.

## What is .gbdialog?

`.gbdialog` files are written in a specialized BASIC dialect that controls:
- Tool execution and integrations
- LLM prompting and context
- Knowledge base activation
- Session and memory management
- External API calls

## Modern Approach: Let the LLM Work
### Minimal BASIC Philosophy

Instead of complex logic, use the LLM's natural understanding:

```basic
' Example from announcements.gbai/update-summary.bas
' Generate summaries from documents
let text = GET "announcements.gbkb/news/news.pdf"
let resume = LLM "In a few words, resume this: " + text
SET BOT MEMORY "resume", resume

' Example from law.gbai/case.bas
' Load context and let LLM answer questions
text = GET "case-" + cod + ".pdf"
text = "Based on this document, answer the person's questions:\n\n" + text
SET CONTEXT text
TALK "Case loaded. You can ask me anything about the case."
```

## Key Components

### 1. LLM Integration
```basic
' Direct LLM usage for natural conversation
response = LLM "Help the user with their question"
TALK response

' Context-aware responses
SET CONTEXT "user_type" AS "premium customer"
answer = LLM "Provide personalized recommendations"
TALK answer
```

### 2. Tool Execution
```basic
' Define tools with parameters
PARAM name AS string LIKE "John Smith" DESCRIPTION "Customer name"
PARAM email AS string LIKE "john@example.com" DESCRIPTION "Email"

' LLM automatically knows when to call this
SAVE "customers.csv", name, email
TALK "Registration complete!"
```

### 3. Knowledge Base Usage
```basic
' Activate knowledge base collections
USE KB "products"
USE KB "policies"

' LLM searches these automatically when answering
answer = LLM "Answer based on our product catalog and policies"
TALK answer
```

### 4. Session Management
```basic
' Store session data
SET "user_name", name
SET "preferences", "email notifications"

' Retrieve later
saved_name = GET "user_name"
TALK "Welcome back, " + saved_name
```

## Script Structure

### Entry Point: start.bas
Every bot needs a `start.bas` file:

```basic
' Minimal start script - let LLM handle everything
USE KB "company_docs"
response = LLM "Welcome the user and offer assistance"
TALK response
```

### Tool Definitions
Create separate `.bas` files for each tool:

```basic
' enrollment.bas - The LLM knows when to use this
PARAM student_name AS string
PARAM course AS string
DESCRIPTION "Enrolls a student in a course"

SAVE "enrollments.csv", student_name, course, NOW()
TALK "Enrolled successfully!"
```

## Best Practices

### 1. Minimal Logic
```basic
' Good - Let LLM handle the conversation
answer = LLM "Process the user's request appropriately"
TALK answer

' Avoid - Don't micromanage the flow
' IF user_says_this THEN do_that...
```

### 2. Clear Tool Descriptions
```basic
DESCRIPTION "This tool books appointments for customers"
' The LLM uses this description to know when to call the tool
```

### 3. Context Over Conditions
```basic
' Provide context, not rules
SET CONTEXT "business_hours" AS "9AM-5PM weekdays"
response = LLM "Inform about availability"
' LLM naturally understands to mention hours when relevant
```

### 4. Trust the LLM
```basic
' Simple prompt, sophisticated behavior
answer = LLM "Be a helpful customer service agent"
' LLM handles greetings, questions, complaints naturally
```

## Common Patterns

### Document Summarization (from announcements.gbai)
```basic
' Schedule automatic updates
SET SCHEDULE "59 * * * *"

' Fetch and summarize documents
let text = GET "announcements.gbkb/news/news.pdf"
let resume = LLM "In a few words, resume this: " + text
SET BOT MEMORY "resume", resume
```

### Interactive Case Analysis (from law.gbai)
```basic
' Ask for case number
TALK "What is the case number?"
HEAR cod

' Load case document
text = GET "case-" + cod + ".pdf"

IF text THEN 
    ' Set context for LLM to use
    text = "Based on this document, answer the person's questions:\n\n" + text
    SET CONTEXT text 
    TALK "Case loaded. Ask me anything about it."
ELSE
    TALK "Case not found, please try again."
END IF
```

### Tool Definition Pattern
```basic
' Tool parameters (auto-discovered by LLM)
PARAM name AS string
PARAM email AS string
DESCRIPTION "Enrollment tool"

' Tool logic (called when LLM decides)
SAVE "enrollments.csv", name, email
TALK "Successfully enrolled " + name
```

### Multi-Collection Search
```basic
USE KB "products"
USE KB "reviews"
USE KB "specifications"

answer = LLM "Answer product questions comprehensively"
TALK answer
```

## Advanced Features

### Memory Management
```basic
SET BOT MEMORY "company_policy", policy_text
' Available across all sessions

retrieved = GET BOT MEMORY "company_policy"
```

### External APIs
```basic
result = GET "https://api.example.com/data"
response = LLM "Interpret this data: " + result
TALK response
```

### Suggestions
```basic
ADD SUGGESTION "Schedule Meeting" AS "schedule"
ADD SUGGESTION "View Products" AS "products"
' UI shows these as quick actions
```

## Error Handling

The system handles errors gracefully:
- Syntax errors caught at compile time
- Runtime errors logged but don't crash
- LLM provides fallback responses
- Timeouts prevent infinite operations

## Script Execution

Scripts run in a sandboxed environment with:
- Access to session state
- LLM generation capabilities
- Knowledge base search
- Tool execution rights
- External API access (configured)

## Migration from Traditional Bots

### Old Way (Complex Logic)
```basic
' DON'T DO THIS - 1990s style
' IF INSTR(user_input, "order") > 0 THEN
'   IF INSTR(user_input, "status") > 0 THEN
'     TALK "Checking order status..."
'   ELSE IF INSTR(user_input, "new") > 0 THEN
'     TALK "Creating new order..."
'   END IF
' END IF
```

### New Way (LLM Intelligence)
```basic
' DO THIS - Let LLM understand naturally
response = LLM "Handle the customer's order request"
TALK response
' LLM understands context and intent automatically
```

The key is to **trust the LLM** and write less code for more intelligent behavior.