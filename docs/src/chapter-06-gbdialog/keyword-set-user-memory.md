# SET USER MEMORY

Persists data at the user level, accessible across sessions and bots. Unlike `SET BOT MEMORY` which stores data per-bot, user memory follows the user wherever they go.

## Syntax

```basic
SET USER MEMORY "key", value
```

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `key` | String | Unique identifier for the stored value |
| `value` | Any | The value to store (string, number, object) |

## Description

`SET USER MEMORY` stores persistent data associated with a specific user. This data:

- **Persists across sessions** - Available when user returns days/weeks later
- **Persists across bots** - Accessible from any bot the user interacts with
- **Survives restarts** - Stored in the database, not just memory
- **Supports TTL** - Optional time-to-live for automatic expiration

This is ideal for user preferences, profile data, and cross-bot personalization.

## Examples

### Basic Usage

```basic
' Store user preferences
SET USER MEMORY "language", "pt-BR"
SET USER MEMORY "timezone", "America/Sao_Paulo"
SET USER MEMORY "theme", "dark"

TALK "Preferences saved!"
```

### Store Complex Objects

```basic
' Store user profile
profile = #{ 
    name: username,
    email: useremail,
    plan: "premium",
    signupDate: NOW()
}
SET USER MEMORY "profile", profile

TALK "Profile updated successfully!"
```

### Cross-Bot Data Sharing

```basic
' In sales-bot: Store purchase history
purchase = #{
    orderId: orderid,
    amount: total,
    date: NOW()
}
SET USER MEMORY "lastPurchase", purchase

' In support-bot: Access the same data
lastPurchase = GET USER MEMORY("lastPurchase")
TALK "I see your last order was #" + lastPurchase.orderId
```

### User Preferences for Personalization

```basic
' Check if returning user
name = GET USER MEMORY("name")

IF name = "" THEN
    TALK "Welcome! What's your name?"
    HEAR name
    SET USER MEMORY "name", name
    TALK "Nice to meet you, " + name + "!"
ELSE
    TALK "Welcome back, " + name + "!"
END IF
```

### Store User Facts

```basic
' Store facts about the user for AI context
SET USER MEMORY "fact_occupation", "software engineer"
SET USER MEMORY "fact_interests", "AI, automation, productivity"
SET USER MEMORY "fact_company", "Acme Corp"

' These can be used to personalize AI responses
```

## Related Keywords

| Keyword | Description |
|---------|-------------|
| [`GET USER MEMORY`](./keyword-get-user-memory.md) | Retrieve user-level persisted data |
| [`SET BOT MEMORY`](./keyword-set-bot-memory.md) | Store data at bot level |
| [`GET BOT MEMORY`](./keyword-get-bot-memory.md) | Retrieve bot-level data |
| [`USER FACTS`](./keyword-user-facts.md) | Get all stored user facts |

## Database Storage

User memory is stored in the `user_memory` table with the following structure:

| Column | Description |
|--------|-------------|
| `user_id` | The user's unique identifier |
| `key` | The memory key |
| `value` | JSON-encoded value |
| `memory_type` | Type classification (preference, fact, context) |
| `ttl` | Optional expiration timestamp |
| `created_at` | When the memory was created |
| `updated_at` | Last modification time |

## Config.csv Options

```csv
name,value
user-memory-enabled,true
user-memory-max-keys,1000
user-memory-default-ttl,0
```

| Option | Default | Description |
|--------|---------|-------------|
| `user-memory-enabled` | `true` | Enable/disable user memory |
| `user-memory-max-keys` | `1000` | Maximum keys per user |
| `user-memory-default-ttl` | `0` | Default TTL in seconds (0 = no expiry) |

## Best Practices

1. **Use descriptive keys** - `user_language` not `lang`
2. **Prefix related keys** - `pref_theme`, `pref_language`, `fact_name`
3. **Don't store sensitive data** - No passwords or tokens
4. **Consider TTL for temporary data** - Session-specific data should expire
5. **Keep values reasonable size** - Don't store large files or blobs

## See Also

- [Memory Management](../chapter-11-features/memory-management.md) - Complete memory architecture
- [Multi-Agent Orchestration](../chapter-11-features/multi-agent-orchestration.md) - Cross-bot data sharing
- [User Context](../chapter-12-auth/user-system-context.md) - User vs system context