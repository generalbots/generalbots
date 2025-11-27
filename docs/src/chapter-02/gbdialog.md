# .gbdialog Dialogs

The [`.gbdialog`](../chapter-02/gbdialog.md) package contains BASIC scripts that define conversation flows, tool integrations, and bot behavior.

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
text = GET "announcements.gbkb/news/news.pdf"
resume = LLM "In a few words, resume this: " + text
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
' LLM is for background processing only - generates content once for all users
' Example: Generate a summary that all users will see
text = GET "document.pdf"
summary = LLM "Summarize this document: " + text
SET BOT MEMORY "daily_summary", summary

' For interactive conversations, use SET CONTEXT and TALK
SET CONTEXT "user_type" AS "premium customer"
TALK "How can I help you today?"
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
See [Knowledge Base documentation](../chapter-03/knowledge-base.md) for details.
```basic
' Activate knowledge base collections
USE KB "products"
USE KB "policies"

' The system AI searches these automatically during conversations
' No LLM command needed - just TALK to the user
TALK "What product information can I help you with?"
```



## Script Structure

### Entry Point: start.bas (Optional)
The `start.bas` file in the [`.gbdialog`](../chapter-02/gbdialog.md) folder is **optional**, but required if you want to activate tools or knowledge bases:

```basic
' Optional start script - needed only to activate tools/KB
USE KB "company_docs"
USE TOOL "book-meeting"
USE TOOL "check-status"
TALK "Welcome! How can I assist you today?"
```

**When you need start.bas:**
- To activate knowledge bases with `USE KB`
- To activate tools with `USE TOOL`
- To set initial context or configuration

**When you don't need start.bas:**
- For simple conversational bots
- When the LLM can handle everything without tools/KB
- For basic Q&A without document search

### Tool Definitions
Create separate `.bas` files for each tool. See [KB and Tools](../chapter-03/kb-and-tools.md) for more information:

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
' Good - Let system AI handle the conversation naturally
TALK "How can I help you?"
' System AI understands context and responds appropriately

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
TALK "When would you like to schedule?"
' System AI naturally understands to mention hours when relevant
```

### 4. Trust the System AI
```basic
' The system AI handles conversations naturally
TALK "Hello! I'm here to help."
' System handles greetings, questions, complaints naturally
```

## Common Patterns

### Document Summarization - Background Processing (from announcements.gbai)
```basic
' Schedule automatic updates - runs in background
SET SCHEDULE "59 * * * *"

' Fetch and summarize documents ONCE for all users
text = GET "announcements.gbkb/news/news.pdf"
resume = LLM "In a few words, resume this: " + text
SET BOT MEMORY "resume", resume  ' Stored for all users
```

### Interactive Case Analysis - User Conversations (from law.gbai)
```basic
' Ask for case number - interactive with user
TALK "What is the case number?"
HEAR cod

' Load case document
text = GET "case-" + cod + ".pdf"

IF text THEN 
    ' Set context for system AI to use in conversation
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

' System AI searches these collections automatically during conversation
TALK "What would you like to know about our products?"
```

## Advanced Features

### Memory Management
See [Storage documentation](../chapter-09/storage.md) for persistent data options.
```basic
SET BOT MEMORY "company_policy", policy_text
' Available across all sessions

retrieved = GET BOT MEMORY "company_policy"
```

### External APIs
See [External APIs chapter](../chapter-08/external-apis.md) for integration patterns.
```basic
result = GET "https://api.example.com/data"
' For background processing only
summary = LLM "Summarize this data: " + result
SET BOT MEMORY "api_summary", summary
```

### Suggestions
See [UI Interface](../chapter-04/ui-interface.md) for UI integration.
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

### New Way (System AI Intelligence)
```basic
' DO THIS - Let system AI handle conversation naturally
TALK "How can I help you with your order?"
' System AI understands context and intent automatically
```

The key is to **trust the system AI** and write less code for more intelligent behavior.

## Important Distinction

- **[LLM Command](../chapter-09/ai-llm.md)**: For background/batch processing, generates content ONCE, stored in BOT MEMORY for all users
- **[Interactive Conversations](../chapter-09/conversation.md)**: Use HEAR/TALK/SET CONTEXT, system AI handles the natural conversation flow

## See Also

- [Chapter 1: Quick Start](../chapter-01/quick-start.md) - Getting started with your first bot
- [Chapter 2: Bot Architecture](../chapter-02/README.md) - Understanding all components
- [Chapter 3: Knowledge Base](../chapter-03/knowledge-base.md) - Working with KB collections
- [Chapter 5: Keywords Reference](../chapter-05/README.md) - Complete BASIC command reference
- [Chapter 9: Conversation Flow](../chapter-09/conversation.md) - Advanced dialog patterns