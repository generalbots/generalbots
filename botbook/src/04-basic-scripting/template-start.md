# start.bas Template

The `start.bas` template is the entry point dialog that runs when a user begins a conversation with the bot. It sets up the initial context, loads bot memories, and presents the user with options.

## Overview

The start dialog typically performs these tasks:
1. Loads bot memories containing important context
2. Sets up context collections for the LLM
3. Clears and configures suggestion buttons
4. Presents an initial greeting to the user

## Example: Announcements Bot

Here's the `start.bas` template from the announcements bot:

```basic
resume1 = GET BOT MEMORY "resume"
resume2 = GET BOT MEMORY "auxiliom"
resume3 = GET BOT MEMORY "toolbix"

SET CONTEXT "general" AS resume1
SET CONTEXT "auxiliom" AS resume2
SET CONTEXT "toolbix" AS resume3

CLEAR SUGGESTIONS

ADD_SUGGESTION "general" AS "Show me the weekly announcements"
ADD_SUGGESTION "auxiliom" AS "Explain Auxiliom to me"
ADD_SUGGESTION "auxiliom" AS "What does Auxiliom provide?"
ADD_SUGGESTION "toolbix" AS "Show me Toolbix features"
ADD_SUGGESTION "toolbix" AS "How can Toolbix help my business?"

TALK resume1
TALK "You can ask me about any of the announcements or circulars."
```

## Breaking Down the Script

### Loading Bot Memories

```basic
resume1 = GET BOT MEMORY "resume"
resume2 = GET BOT MEMORY "auxiliom"
resume3 = GET BOT MEMORY "toolbix"
```

Bot memories are persistent key-value pairs stored in the database. They typically contain:
- Introduction text
- System prompts
- Important context that should always be available

### Setting Context

```basic
SET CONTEXT "general" AS resume1
SET CONTEXT "auxiliom" AS resume2
SET CONTEXT "toolbix" AS resume3
```

The `SET_CONTEXT` keyword adds text to named context collections that the LLM can access. This ensures the bot has relevant information when answering questions.

### Configuring Suggestions

```basic
CLEAR_SUGGESTIONS;

ADD_SUGGESTION "general" AS "Show me the weekly announcements"
ADD_SUGGESTION "auxiliom" AS "Explain Auxiliom to me"
```

Suggestions appear as clickable buttons in the chat interface, helping users understand what the bot can do.

### Initial Greeting

```basic
TALK resume1
TALK "You can ask me about any of the announcements or circulars."
```

The bot speaks the introduction text and provides guidance on how to interact.

## Simple Start Template

For a minimal bot, a start template might be:

```basic
intro = GET BOT MEMORY "introduction"
SET CONTEXT "main" AS intro

TALK "Hello! I'm your assistant."
TALK "How can I help you today?"
```

## Start Template with Authentication

For bots requiring login:

```basic
user_id = GET BOT MEMORY "current_user"

IF user_id = "" THEN
    TALK "Welcome! Please log in to continue."
    RUN "auth.bas"
ELSE
    welcome = "Welcome back, " + user_id + "!"
    TALK welcome
    TALK "What would you like to do today?"
END IF
```

## Start Template with Knowledge Base

For knowledge-focused bots:

```basic
USE KB "policies"
USE KB "procedures"
USE KB "faqs"

intro = GET BOT MEMORY "assistant_role"
SET CONTEXT "role" AS intro

TALK "I have access to company policies, procedures, and FAQs."
TALK "Ask me anything about our documentation!"
```

## Best Practices

1. **Load Essential Context**: Always load critical bot memories at startup
2. **Set Clear Expectations**: Tell users what the bot can do
3. **Provide Suggestions**: Help users get started with example queries
4. **Keep It Brief**: Don't overwhelm users with too much text initially
5. **Handle Errors**: Check if memories exist before using them

## Configuration

The start dialog is specified in the bot's `config.csv`:

```csv
name,value
Start Dialog,start
```

This tells botserver to run `start.bas` when a conversation begins.

## Common Patterns

### Multi-Purpose Bot

```basic
CLEAR_SUGGESTIONS;
ADD_SUGGESTION "support" AS "I need help with a problem"
ADD_SUGGESTION "sales" AS "Tell me about your products"
ADD_SUGGESTION "docs" AS "Show me the documentation"

TALK "I can help with support, sales, or documentation."
TALK "What brings you here today?"
```

### Personalized Greeting

```basic
hour = HOUR(NOW())
greeting = ""

IF hour < 12 THEN
    greeting = "Good morning!"
ELSE IF hour < 17 THEN
    greeting = "Good afternoon!"
ELSE
    greeting = "Good evening!"
END IF

TALK greeting
TALK "How may I assist you?"
```

### Context-Aware Start

```basic
last_topic = GET BOT MEMORY "last_topic"

IF last_topic <> "" THEN
    msg = "Last time we discussed " + last_topic + "."
    TALK msg
    TALK "Would you like to continue that conversation?"
ELSE
    TALK "Welcome! What would you like to know?"
END IF
```

## Summary

The `start.bas` template sets the tone for the entire conversation. It should be welcoming, informative, and guide users toward productive interactions. By loading context, setting suggestions, and providing clear instructions, you create a smooth onboarding experience for users engaging with your bot.