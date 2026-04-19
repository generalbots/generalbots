# SET BOT MEMORY

Store persistent key-value data at the bot level that persists across all conversations.

## Syntax

```basic
SET BOT MEMORY key, value
```

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `key` | String | Unique identifier for the memory item |
| `value` | String | Value to store (can be any string data) |

## Description

The `SET BOT MEMORY` keyword stores data that is:
- Persistent across all user sessions
- Shared between all users of the same bot
- Stored in the database permanently
- Available until explicitly updated or cleared

Bot memory is useful for:
- Configuration settings
- Global counters and statistics
- Shared state between users
- Bot-wide preferences
- Cached data that applies to all conversations

## Examples

### Store Simple Values
```basic
SET BOT MEMORY "welcome_message", "Hello! Welcome to our service."
SET BOT MEMORY "support_email", "support@example.com"
SET BOT MEMORY "business_hours", "9 AM - 5 PM EST"
```

### Store Counters
```basic
current_count = GET BOT MEMORY "visitor_count"
IF current_count = "" THEN
    current_count = "0"
END IF
new_count = VAL(current_count) + 1
SET BOT MEMORY "visitor_count", STR(new_count)
TALK "You are visitor number " + STR(new_count)
```

### Store JSON Data
```basic
user_data = '{"name": "John", "level": 5, "points": 1200}'
SET BOT MEMORY "user_profile", user_data
```

### Dynamic Keys
```basic
today = FORMAT(NOW(), "YYYY-MM-DD")
daily_key = "stats_" + today
SET BOT MEMORY daily_key, "25"
```

### Configuration Management
```basic
' Store bot configuration
SET BOT MEMORY "max_retries", "3"
SET BOT MEMORY "timeout_seconds", "30"
SET BOT MEMORY "api_version", "v2"

' Later, read configuration
max_retries = GET BOT MEMORY "max_retries"
timeout = GET BOT MEMORY "timeout_seconds"
```

## Database Storage

Bot memories are stored in the `bot_memories` table:
- `id`: UUID primary key
- `bot_id`: Reference to the bot
- `key`: Memory key (indexed for fast lookup)
- `value`: Memory value (text)
- `created_at`: Timestamp of creation
- `updated_at`: Timestamp of last update

## Performance Considerations

- Keys are indexed for fast retrieval
- Values are stored as text (no size limit in PostgreSQL)
- Updates are asynchronous to avoid blocking
- Consider using structured keys for organization

## Best Practices

1. **Use Descriptive Keys**: Make keys self-documenting
   ```basic
   SET BOT MEMORY "config:email:smtp_server", "mail.example.com"
   SET BOT MEMORY "stats:daily:2024-01-15", "150"
   ```

2. **Handle Missing Values**: Always check if memory exists
   ```basic
   value = GET BOT MEMORY "some_key"
   IF value = "" THEN
       ' Initialize with default
       SET BOT MEMORY "some_key", "default_value"
       value = "default_value"
   END IF
   ```

3. **Avoid Sensitive Data**: Don't store passwords or tokens
   ```basic
   ' BAD: Don't do this
   ' SET BOT MEMORY "admin_password", "secret123"
   
   ' GOOD: Store non-sensitive config
   SET BOT MEMORY "admin_email", "admin@example.com"
   ```

4. **Structure Complex Data**: Use JSON for complex structures
   ```basic
   settings = '{"theme": "dark", "language": "en", "notifications": true}'
   SET BOT MEMORY "user_preferences", settings
   ```

5. **Clean Up Old Data**: Remove unused memories periodically
   ```basic
   ' Remove old daily stats
   old_date = FORMAT(DATE_ADD(NOW(), -30, "days"), "YYYY-MM-DD")
   SET BOT MEMORY "stats_" + old_date, ""
   ```

## Differences from User Memory

| Aspect | Bot Memory | User Memory |
|--------|------------|-------------|
| Scope | All users of the bot | Single user |
| Persistence | Permanent | Session or permanent |
| Use Case | Global settings | Personal data |
| Access | Any conversation | User's conversations only |

## Error Handling

- If database connection fails, operation is logged but doesn't crash
- Invalid bot IDs are logged as errors
- Duplicate keys update existing values
- Empty keys are not allowed

## Related Keywords

- [GET BOT MEMORY](./keyword-get-bot-memory.md) - Retrieve stored bot memory
- [SET](./keyword-set.md) - Set user-scoped variables
- [REMEMBER](./keyword-remember.md) - Store user-specific memories

## Implementation

Located in `src/basic/keywords/bot_memory.rs`

The implementation:
- Uses async database operations
- Handles updates atomically with transactions
- Validates bot ID format
- Logs all operations for debugging