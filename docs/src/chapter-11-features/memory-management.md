# Memory Management

General Bots provides a comprehensive memory system that enables persistent storage, cross-session continuity, and multi-agent data sharing. This chapter covers all memory types, their use cases, and best practices.

## Overview

The memory system supports four distinct scopes:

| Memory Type | Scope | Persistence | Use Case |
|-------------|-------|-------------|----------|
| **User Memory** | Per user, all bots | Permanent | Preferences, profile, facts |
| **Bot Memory** | Per bot, all users | Permanent | Bot state, counters, config |
| **Session Memory** | Per session | Session lifetime | Current conversation context |
| **Episodic Memory** | Per conversation | Permanent | Conversation summaries |

## User Memory

User memory follows users across all bots and sessions, enabling personalization and continuity.

### Keywords

```basic
' Store user data
SET USER MEMORY "key", value

' Retrieve user data
value = GET USER MEMORY("key")

' Store a fact about the user
SET USER FACT "occupation", "software engineer"

' Get all user facts
facts = USER FACTS()
```

### Examples

#### Personalized Greeting

```basic
' Check if returning user
name = GET USER MEMORY("name")

IF name = "" THEN
    TALK "Hello! What's your name?"
    HEAR name
    SET USER MEMORY "name", name
    TALK "Nice to meet you, " + name + "!"
ELSE
    TALK "Welcome back, " + name + "!"
END IF
```

#### Cross-Bot Preferences

```basic
' In any bot - store preference
SET USER MEMORY "language", "pt-BR"
SET USER MEMORY "timezone", "America/Sao_Paulo"

' In any other bot - use preference
language = GET USER MEMORY("language")
IF language = "pt-BR" THEN
    TALK "Olá! Como posso ajudar?"
ELSE
    TALK "Hello! How can I help?"
END IF
```

#### User Facts for AI Context

```basic
' Store facts about the user
SET USER FACT "company", "Acme Corp"
SET USER FACT "role", "Product Manager"
SET USER FACT "interests", "AI, automation, productivity"

' Later, use facts to personalize AI responses
facts = USER FACTS()
SET CONTEXT "user_profile" AS facts

response = LLM "Help me draft a product roadmap"
' AI now knows user's role and interests
```

### Database Schema

User memory is stored in the `user_memory` table:

| Column | Type | Description |
|--------|------|-------------|
| `id` | UUID | Primary key |
| `user_id` | UUID | User identifier |
| `key` | VARCHAR(255) | Memory key |
| `value` | JSONB | Stored value (any type) |
| `memory_type` | VARCHAR(50) | preference, fact, context |
| `ttl` | TIMESTAMP | Optional expiration |
| `created_at` | TIMESTAMP | Creation time |
| `updated_at` | TIMESTAMP | Last update |

### Configuration

```csv
name,value
user-memory-enabled,true
user-memory-max-keys,1000
user-memory-default-ttl,0
```

| Option | Default | Description |
|--------|---------|-------------|
| `user-memory-enabled` | `true` | Enable user memory |
| `user-memory-max-keys` | `1000` | Max keys per user |
| `user-memory-default-ttl` | `0` | Default TTL (0 = no expiry) |

## Bot Memory

Bot memory stores data at the bot level, shared across all users but isolated per bot.

### Keywords

```basic
' Store bot data
SET BOT MEMORY "key", value

' Retrieve bot data
value = GET BOT MEMORY("key")
```

### Examples

#### Bot Statistics

```basic
' Track bot usage
conversations = GET BOT MEMORY("total_conversations")
conversations = conversations + 1
SET BOT MEMORY "total_conversations", conversations

PRINT "This bot has had " + conversations + " conversations"
```

#### Feature Flags

```basic
' Store feature configuration
SET BOT MEMORY "enable_voice", true
SET BOT MEMORY "max_retries", 3
SET BOT MEMORY "welcome_message", "Hello! I'm your assistant."

' Use in logic
enableVoice = GET BOT MEMORY("enable_voice")
IF enableVoice THEN
    ' Enable voice features
END IF
```

#### Cache API Results

```basic
' Cache expensive API calls
cachedRates = GET BOT MEMORY("exchange_rates")
cachedTime = GET BOT MEMORY("exchange_rates_time")

IF cachedRates = "" OR (NOW() - cachedTime) > 3600 THEN
    ' Refresh cache
    rates = GET "https://api.exchangerate.host/latest"
    SET BOT MEMORY "exchange_rates", rates
    SET BOT MEMORY "exchange_rates_time", NOW()
ELSE
    rates = cachedRates
END IF
```

### Use Cases

| Use Case | Example Key | Description |
|----------|-------------|-------------|
| Counters | `total_orders` | Track bot-wide metrics |
| Config | `max_items` | Runtime configuration |
| Cache | `api_cache_products` | Cached API responses |
| State | `last_sync_time` | Operational state |

## Session Memory

Session memory is temporary storage for the current conversation session.

### Keywords

```basic
' Store in session
SET "key", value

' Retrieve from session
value = GET "key"

' Set context for AI
SET CONTEXT "topic" AS "billing inquiry"
```

### Examples

#### Conversation State

```basic
' Track conversation flow
SET "current_step", "collecting_info"
SET "collected_name", username
SET "collected_email", useremail

' Later in conversation
step = GET "current_step"
IF step = "collecting_info" THEN
    ' Continue collecting
END IF
```

#### Multi-Turn Context

```basic
' Build context through conversation
SET CONTEXT "customer_id" AS customerid
SET CONTEXT "issue_type" AS "refund"
SET CONTEXT "order_id" AS orderid

' AI has full context for responses
response = LLM "Help resolve this customer issue"
```

### Session Lifetime

- Created when user starts conversation
- Persists across messages in same conversation
- Cleared when conversation ends or times out
- Default timeout: 30 minutes of inactivity

## Episodic Memory

Episodic memory stores summaries of past conversations for long-term context.

### How It Works

1. **Conversation Ends** - System detects conversation completion
2. **Summary Generated** - LLM creates concise summary
3. **Stored** - Summary saved with metadata
4. **Retrieved** - Used in future conversations for context

### Example

```basic
' System automatically creates episode summaries
' Example summary stored:
' {
'   "conversation_id": "abc123",
'   "summary": "User asked about refund policy, was satisfied with explanation",
'   "topics": ["refunds", "policy"],
'   "sentiment": "positive",
'   "resolution": "resolved",
'   "created_at": "2025-01-15T10:30:00Z"
' }

' In future conversations, retrieve relevant episodes
episodes = GET USER MEMORY("recent_episodes")
SET CONTEXT "previous_interactions" AS episodes
```

### Configuration

```csv
name,value
episodic-memory-enabled,true
episodic-summary-model,fast
episodic-max-episodes,100
episodic-retention-days,365
```

## Memory Patterns

### Profile Builder Pattern

Build user profile progressively through conversations.

```basic
' Check what we know
profile = GET USER MEMORY("profile")
IF profile = "" THEN
    profile = #{ }
END IF

' Fill in missing information naturally
IF profile.name = "" THEN
    ' Ask for name when appropriate
END IF

IF profile.preferences = "" THEN
    ' Learn preferences from behavior
END IF

' Update profile
SET USER MEMORY "profile", profile
```

### Preference Learning Pattern

Learn preferences from user behavior.

```basic
' Track user choices
choice = HEAR selection
choices = GET USER MEMORY("choices_history")
IF choices = "" THEN choices = []

' Add new choice
choices = APPEND(choices, #{
    choice: choice,
    context: currentContext,
    timestamp: NOW()
})
SET USER MEMORY "choices_history", choices

' Analyze patterns periodically
IF LEN(choices) >= 10 THEN
    preferences = LLM "Analyze these choices and identify preferences: " + JSON(choices)
    SET USER MEMORY "learned_preferences", preferences
END IF
```

### Context Handoff Pattern

Pass context between bots in multi-agent scenarios.

```basic
' Sending bot: Store context for receiving bot
handoffContext = #{
    topic: currentTopic,
    userIntent: detectedIntent,
    conversationSummary: summary,
    relevantFacts: facts
}
SET USER MEMORY "handoff_context", handoffContext

' Transfer to specialist
TRANSFER CONVERSATION TO "specialist-bot"

' Receiving bot: Retrieve context
context = GET USER MEMORY("handoff_context")
SET CONTEXT "background" AS context.conversationSummary
SET CONTEXT "intent" AS context.userIntent

' Clear handoff context after use
SET USER MEMORY "handoff_context", ""
```

### TTL Pattern

Use time-to-live for temporary data.

```basic
' Store with expiration (implementation depends on memory type)
' For session-like data in user memory:
SET USER MEMORY "temp_auth_code", #{
    code: authCode,
    expires: NOW() + 300  ' 5 minutes
}

' Check expiration
stored = GET USER MEMORY("temp_auth_code")
IF stored <> "" AND stored.expires > NOW() THEN
    ' Valid
ELSE
    ' Expired or not found
    SET USER MEMORY "temp_auth_code", ""
END IF
```

## Best Practices

### Key Naming Conventions

```basic
' Use consistent prefixes
SET USER MEMORY "pref_language", "en"      ' Preferences
SET USER MEMORY "pref_timezone", "UTC"
SET USER MEMORY "fact_name", "John"        ' Facts
SET USER MEMORY "fact_company", "Acme"
SET USER MEMORY "ctx_last_topic", "sales"  ' Context
SET USER MEMORY "cache_products", data     ' Cached data
```

### Don't Store Sensitive Data

```basic
' ❌ DON'T: Store sensitive data
SET USER MEMORY "password", userPassword
SET USER MEMORY "ssn", socialSecurityNumber
SET USER MEMORY "credit_card", cardNumber

' ✅ DO: Store references only
SET USER MEMORY "payment_method_id", paymentId
SET USER MEMORY "verified", true
```

### Handle Missing Data Gracefully

```basic
' Always check for empty/missing
name = GET USER MEMORY("name")
IF name = "" THEN
    name = "there"  ' Default value
END IF
TALK "Hello, " + name + "!"
```

### Clean Up Old Data

```basic
' Periodic cleanup of old data
lastCleanup = GET BOT MEMORY("last_memory_cleanup")
IF lastCleanup = "" OR (NOW() - lastCleanup) > 86400 THEN
    ' Run cleanup logic
    ' Remove expired entries, old cache, etc.
    SET BOT MEMORY "last_memory_cleanup", NOW()
END IF
```

## Troubleshooting

### Memory Not Persisting

1. Check memory type - session memory doesn't persist
2. Verify database connection
3. Check for key name typos (keys are case-sensitive)
4. Review memory limits

### Cross-Bot Memory Not Sharing

1. Ensure using `USER MEMORY` not `BOT MEMORY`
2. Verify same user identity
3. Check `user-memory-enabled` config

### Memory Full Errors

1. Clean up old/unused keys
2. Increase `user-memory-max-keys`
3. Use TTL for temporary data
4. Consolidate related keys into objects

## See Also

- [SET USER MEMORY](../chapter-06-gbdialog/keyword-set-user-memory.md) - Store user memory
- [GET USER MEMORY](../chapter-06-gbdialog/keyword-get-user-memory.md) - Retrieve user memory
- [SET BOT MEMORY](../chapter-06-gbdialog/keyword-set-bot-memory.md) - Store bot memory
- [GET BOT MEMORY](../chapter-06-gbdialog/keyword-get-bot-memory.md) - Retrieve bot memory
- [Multi-Agent Orchestration](./multi-agent-orchestration.md) - Cross-bot data sharing