# Chapter 05: BASIC Dialogs

**Write chatbots like it's 1985.** BotServer uses BASIC - yes, that BASIC - for conversation scripts. Simple English-like commands that anyone can write. No frameworks, no callbacks, no async/await complexity.

## Why BASIC?

Because `TALK "Hello"` is clearer than `await ctx.send(Message(content="Hello"))`. We chose simplicity over sophistication. Your grandmother could write these scripts.

## The 5-Minute Tutorial

### Your First Script

Create `start.bas`:
```basic
TALK "Hi! What's your name?"
HEAR name
TALK "Nice to meet you, " + name
```

That's a working chatbot. Three lines.

### Add Knowledge

```basic
USE KB "documentation"
TALK "You can ask me anything about our documentation!"
TALK "What would you like to know?"
HEAR question
```

The bot now has access to your documents and can answer questions about them.

### Add Tools

```basic
USE TOOL "weather"
TALK "I can check the weather for you."
TALK "Which city?"
HEAR location
```

Tools are automatically discovered and can be called by the AI as needed.

## Core Commands

The essential BASIC commands you need:

| Command | Purpose | Example |
|---------|---------|---------|
| TALK | Send message | `TALK "Hello"` |
| HEAR | Get input | `HEAR name` |
| USE KB | Load knowledge | `USE KB "docs"` |
| USE TOOL | Enable function | `USE TOOL "weather"` |

## Real Examples from Templates

### Weather Bot (from default.gbai)
```basic
TALK "I can check the weather for any city."
TALK "Which location?"
HEAR location
USE TOOL "weather"
```

### Email Tool (from default.gbai)
```basic
TALK "I can help you send an email."
TALK "Who would you like to email?"
HEAR recipient
TALK "What's the subject?"
HEAR subject
TALK "What's the message?"
HEAR message
USE TOOL "send-email"
```

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
HEAR answer
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

### Interactive Assistant
```basic
TALK "Hello! I'm your assistant."
TALK "How can I help you today?"
HEAR request

IF INSTR(request, "weather") > 0 THEN
  USE TOOL "weather"
  TALK "Which city?"
  HEAR city
ELSE IF INSTR(request, "email") > 0 THEN
  USE TOOL "send-email"
  TALK "I'll help you send an email."
ELSE
  TALK "I can help with weather or email."
END IF
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

Next: [BASIC Keywords Reference](./keywords.md)