# Chapter 06: BASIC Dialogs

BASIC is back, and it's powering the AI revolution. In an age of complex programming languages, General Bots chose BASIC for a simple reason: everyone can learn it in minutes, yet it's powerful enough to orchestrate sophisticated AI workflows.

## Why BASIC in 2025?

We believe AI development shouldn't require a computer science degree. BASIC's English-like syntax means:
- Business analysts can write automation
- Teachers can create educational bots  
- Doctors can build medical assistants
- No programming background needed

Because `TALK "Hello"` is clearer than `await ctx.send(Message(content="Hello"))`. We chose simplicity over sophistication. Your grandmother could write these scripts.

## Beyond Simple Scripts

Modern BASIC isn't your grandfather's language. BASIC scripts can:
- Orchestrate multiple AI models
- Process complex data
- Integrate with any API
- Handle enterprise workflows
- Scale to millions of users

## The 5-Minute Tutorial

### Your First Script

Create `start.bas`:
```basic
TALK "Hi! What's your name?"
HEAR name
TALK "Nice to meet you, " + name
```

That's a working chatbot. Three lines.

### Add [Knowledge](../chapter-03/knowledge-base.md)

```basic
USE KB "documentation"
TALK "You can ask me anything about our documentation!"
```

The bot now has access to your documents and can answer questions about them.

### Add [Tools](../chapter-03/kb-and-tools.md)

```basic
USE TOOL "weather"
TALK "I can check the weather for you."
```

Tools are automatically discovered and can be called by the AI as needed.

## Core Commands

The essential BASIC commands you need:

| Command | Purpose | Example |
|---------|---------|---------|
| [TALK](./keyword-talk.md) | Send message | `TALK "Hello"` |
| HEAR | Get input | `HEAR name` |
| USE KB | Load knowledge | `USE KB "docs"` |
| USE TOOL | Enable function | `USE TOOL "weather"` |


## Variables

Simple variable assignment:
```basic
name = "John"
age = 25
message = "Hello, " + name
```

## Conditionals

Basic IF/THEN logic:
```basic
HEAR answer AS BOOLEAN
IF answer = "yes" THEN
  TALK "Great!"
ELSE
  TALK "No problem!"
END IF
```

## Loops

Simple FOR loops:
```basic
FOR i = 1 TO 3
  TALK "Count: " + i
NEXT
```

## Session Memory

Store and retrieve session data:
```basic
SET BOT MEMORY "user_name" = name
preference = GET BOT MEMORY "user_name"
```

## Knowledge Base

Work with document collections:
```basic
USE KB "policies"
USE KB "procedures"
TALK "I now have access to company policies and procedures."
TALK "What would you like to know?"
```

## Tools from Templates

BotServer includes these ready-to-use tools:

### From default.gbai:
- **weather.vbs** - Get weather for any city
- **send-email.vbs** - Send emails
- **send-sms.vbs** - Send SMS messages
- **translate.vbs** - Translate text
- **calculate.vbs** - Perform calculations

### Usage:
```basic
USE TOOL "weather"
USE TOOL "send-email"
' Tools are now available for the AI to call
```

## Bot Examples

### Simple Q&A Bot
```basic
USE KB "faq"
TALK "Welcome! I can answer questions from our FAQ."
TALK "What would you like to know?"
loop:
  HEAR question
  TALK "Let me find that for you..."
  GOTO loop
```

## File Structure

Your bot's dialog scripts go in the `.gbdialog` folder:

```
mybot.gbai/
  mybot.gbdialog/
    start.bas         # Entry point
    tools/            # Optional tools folder
      custom.vbs      # Custom tools
```

## Writing Tools

Tools are BASIC scripts with parameters:
```basic
PARAM name AS STRING
PARAM email AS STRING
DESCRIPTION "Save contact information"

SAVE "contacts.csv", name, email
TALK "Contact saved!"
```

## Important Notes

- BASIC is case-insensitive
- Comments start with `'` or `REM`
- Strings use double quotes
- Line continuation with `_`
- No semicolons needed
- No async/await complexity

## Quick Reference

### Essential Keywords
- `TALK` - Output message
- `HEAR` - Get input
- `USE KB` - Load knowledge base
- `USE TOOL` - Load tool
- `SET BOT MEMORY` - Store data
- `GET BOT MEMORY` - Retrieve data
- `IF/THEN/ELSE` - Conditionals
- `FOR/NEXT` - Loops
- `GOTO` - Jump to label

### Data Types
- Strings: `"text"`
- Numbers: `42`
- Boolean: True/False
- Variables: Simple assignment with `=`

## Summary

BASIC in BotServer brings conversational AI back to simplicity. No complex frameworks, just straightforward commands that read like English. Focus on the conversation, not the code.

## See Also

### Documentation
- [Dialog Basics](./basics.md) - Core concepts and patterns
- [Universal Messaging](./universal-messaging.md) - Multi-channel support
- [Template Examples](./templates.md) - Ready-to-use scripts
- [Keyword Reference](./keywords.md) - Complete command list
- [Chapter 3: Knowledge Base](../chapter-03/README.md) - Integrate documents
- [Chapter 9: API and Tools](../chapter-09-api/README.md) - External integrations

### Further Reading - Blog Posts
- [BASIC for Everyone](https://pragmatismo.cloud/blog/basic-for-everyone) - Why we chose BASIC for AI development
- [BASIC LLM Tools](https://pragmatismo.cloud/blog/basic-llm-tools) - Extending LLMs with BASIC scripts
- [MCP is the new API](https://pragmatismo.cloud/blog/mcp-is-the-new-api) - How BASIC scripts become universal tools
- [No Forms](https://pragmatismo.cloud/blog/no-forms) - The conversational UI philosophy
- [Beyond Chatbots](https://pragmatismo.cloud/blog/beyond-chatbots) - Real business applications

### Next Chapter
Continue to [Chapter 7: Architecture](../chapter-07-gbapp/README.md) to understand how General Bots works under the hood.

Next: [BASIC Keywords Reference](./keywords.md)
