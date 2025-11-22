# REMEMBER

Store information in bot's long-term memory for future conversations.

## Syntax

```basic
REMEMBER key, value
```

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `key` | String | Memory key/identifier |
| `value` | String | Information to remember |

## Description

The `REMEMBER` keyword stores information persistently across conversations. Unlike session variables that expire, remembered data persists:

- Across multiple conversations
- Between user sessions
- After bot restarts
- Indefinitely until explicitly forgotten

This enables bots to build user profiles, track preferences, and maintain context over time.

## Examples

### Store User Preferences
```basic
name = HEAR "What's your name?"
REMEMBER "user_name", name
TALK "Nice to meet you, " + name + ". I'll remember that!"
```

### Remember User Choices
```basic
language = HEAR "Preferred language? (English/Spanish/French)"
REMEMBER "preferred_language", language
TALK "I'll communicate in " + language + " from now on"
```

### Build User Profile
```basic
REMEMBER "signup_date", TODAY()
REMEMBER "user_tier", "premium"
REMEMBER "last_contact", NOW()
REMEMBER "interaction_count", interaction_count + 1
```

### Store Complex Information
```basic
' Store JSON-like data
preferences = "{theme: 'dark', notifications: true, timezone: 'EST'}"
REMEMBER "user_preferences", preferences

' Store lists
interests = "technology, science, sports"
REMEMBER "interests", interests
```

## Retrieval

Use `RECALL` to retrieve remembered information:

```basic
' In a future conversation
name = RECALL("user_name")
IF name != "" THEN
    TALK "Welcome back, " + name + "!"
ELSE
    TALK "Hello! I don't think we've met before."
END IF
```

## Memory Scope

Memories are scoped by:
- User ID (each user has separate memory)
- Bot ID (memories don't cross bots)
- Optional namespace (for organizing memories)

## Memory Management

### List All Memories
```basic
memories = GET_ALL_MEMORIES()
FOR EACH memory IN memories
    PRINT memory.key + ": " + memory.value
NEXT
```

### Forget Specific Memory
```basic
FORGET "temporary_data"
```

### Clear All Memories
```basic
CLEAR_ALL_MEMORIES()
TALK "Memory wiped clean!"
```

### Memory with Expiration
```basic
' Remember for 30 days
REMEMBER_TEMP "trial_status", "active", 30
```

## Use Cases

### Customer Service
```basic
' Remember customer issues
issue = HEAR "What problem are you experiencing?"
REMEMBER "last_issue", issue
REMEMBER "issue_date", TODAY()

' In follow-up conversation
last_issue = RECALL("last_issue")
IF last_issue != "" THEN
    TALK "Following up on your issue: " + last_issue
END IF
```

### Personal Assistant
```basic
' Remember important dates
birthday = HEAR "When is your birthday?"
REMEMBER "birthday", birthday

' Check on birthday
IF TODAY() = RECALL("birthday") THEN
    TALK "Happy Birthday! 🎉"
END IF
```

### Learning System
```basic
' Track user corrections
correction = HEAR "Actually, that's not correct..."
REMEMBER "correction_" + topic, correction
corrections_count = RECALL("total_corrections") + 1
REMEMBER "total_corrections", corrections_count
```

### Preferences Tracking
```basic
' Remember communication preferences
IF time_of_day = "morning" AND response_time < 5 THEN
    REMEMBER "active_time", "morning"
END IF

preferred_channel = GET_CHANNEL()
REMEMBER "preferred_channel", preferred_channel
```

## Performance Considerations

- Memories are indexed for fast retrieval
- Large values (>1MB) should be stored as files
- Frequently accessed memories are cached
- Memory operations are asynchronous

## Privacy and Security

### Data Protection
- Memories are encrypted at rest
- PII should be marked as sensitive
- Comply with data retention policies
- Support user data deletion requests

### GDPR Compliance
```basic
' Allow users to export their data
IF request = "export_my_data" THEN
    data = EXPORT_USER_MEMORIES()
    SEND_MAIL user_email, "Your Data", data
END IF

' Allow users to delete their data
IF request = "forget_me" THEN
    DELETE_USER_MEMORIES()
    TALK "Your data has been deleted"
END IF
```

## Best Practices

1. **Use descriptive keys**: Make memory keys self-documenting
2. **Validate before storing**: Check data quality
3. **Handle missing memories**: Always check if memory exists
4. **Organize with namespaces**: Group related memories
5. **Clean up old data**: Remove outdated memories
6. **Respect privacy**: Ask permission for sensitive data
7. **Document memories**: Keep track of what's stored

## Advanced Features

### Structured Memory
```basic
' Store structured data
user_profile = CREATE_MAP()
user_profile["name"] = name
user_profile["age"] = age
user_profile["interests"] = interests
REMEMBER "profile", JSON_STRINGIFY(user_profile)

' Retrieve and parse
profile_json = RECALL("profile")
profile = JSON_PARSE(profile_json)
```

### Memory Search
```basic
' Search memories by pattern
matching_memories = SEARCH_MEMORIES("pref_*")
FOR EACH mem IN matching_memories
    PRINT mem.key + ": " + mem.value
NEXT
```

### Memory Analytics
```basic
' Track memory usage
stats = GET_MEMORY_STATS()
PRINT "Total memories: " + stats.count
PRINT "Storage used: " + stats.size_mb + "MB"
PRINT "Oldest memory: " + stats.oldest_date
```

## Error Handling

```basic
TRY
    REMEMBER "important_data", data
CATCH "storage_full"
    ' Clean up old memories
    DELETE_OLD_MEMORIES(30)  ' Delete memories older than 30 days
    RETRY
CATCH "invalid_data"
    LOG "Cannot store invalid data"
END TRY
```

## Related Keywords

- [GET_BOT_MEMORY](./keyword-get-bot-memory.md) - Session-scoped memory
- [SET_BOT_MEMORY](./keyword-set-bot-memory.md) - Temporary memory
- [SET_USER](./keyword-set-user.md) - Set user context
- [SET_CONTEXT](./keyword-set-context.md) - Set conversation context

## Implementation

Located in `src/basic/keywords/remember.rs`

Uses persistent storage (PostgreSQL) with caching layer (Redis) for performance.