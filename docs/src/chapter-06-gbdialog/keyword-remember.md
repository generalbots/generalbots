# REMEMBER / RECALL Keywords

The `REMEMBER` and `RECALL` keywords provide a powerful time-based memory system for storing and retrieving data associated with users. Unlike standard memory operations, `REMEMBER` supports automatic expiration of stored values.

## Syntax

### REMEMBER

```basic
REMEMBER key, value, duration
```

### RECALL

```basic
result = RECALL key
```

## Parameters

### REMEMBER Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `key` | String | Unique identifier for the memory entry |
| `value` | Any | Data to store (string, number, boolean, array, or object) |
| `duration` | String | How long to remember the value |

### Duration Formats

| Format | Example | Description |
|--------|---------|-------------|
| `N seconds` | `"30 seconds"` | Expires after N seconds |
| `N minutes` | `"5 minutes"` | Expires after N minutes |
| `N hours` | `"2 hours"` | Expires after N hours |
| `N days` | `"7 days"` | Expires after N days |
| `N weeks` | `"2 weeks"` | Expires after N weeks |
| `N months` | `"3 months"` | Expires after ~N×30 days |
| `N years` | `"1 year"` | Expires after ~N×365 days |
| `forever` | `"forever"` | Never expires |
| `permanent` | `"permanent"` | Never expires (alias) |
| Plain number | `"30"` | Interpreted as days |

## Examples

### Basic Usage

```basic
' Remember user's preferred language for 30 days
REMEMBER "preferred_language", "Spanish", "30 days"

' Later, recall the preference
language = RECALL "preferred_language"
TALK "Your language preference is: " + language
```

### Session-Based Memory

```basic
' Remember a temporary verification code for 5 minutes
code = RANDOM(100000, 999999)
REMEMBER "verification_code", code, "5 minutes"
TALK "Your verification code is: " + code

' Verify the code later
HEAR user_code
stored_code = RECALL "verification_code"

IF user_code = stored_code THEN
    TALK "Code verified successfully!"
ELSE
    TALK "Invalid or expired code."
END IF
```

### Storing Complex Data

```basic
' Store user preferences as an array
preferences = ["dark_mode", "notifications_on", "english"]
REMEMBER "user_preferences", preferences, "1 year"

' Store a shopping cart temporarily
cart = ["item1", "item2", "item3"]
REMEMBER "shopping_cart", cart, "2 hours"
```

### Permanent Storage

```basic
' Store important user information permanently
REMEMBER "account_created", NOW(), "forever"
REMEMBER "user_tier", "premium", "permanent"
```

### Promotional Campaigns

```basic
' Track if user has seen a promotional message
has_seen = RECALL "promo_summer_2024"

IF has_seen = null THEN
    TALK "🎉 Special summer offer: 20% off all products!"
    REMEMBER "promo_summer_2024", true, "30 days"
END IF
```

### Rate Limiting

```basic
' Simple rate limiting for API calls
call_count = RECALL "api_calls_today"

IF call_count = null THEN
    call_count = 0
END IF

IF call_count >= 100 THEN
    TALK "You've reached your daily API limit. Please try again tomorrow."
ELSE
    call_count = call_count + 1
    REMEMBER "api_calls_today", call_count, "24 hours"
    ' Process the API call
END IF
```

## How It Works

1. **Storage**: Data is stored in the `bot_memories` database table with:
   - User ID and Bot ID association
   - JSON-serialized value
   - Creation timestamp
   - Optional expiration timestamp

2. **Retrieval**: When `RECALL` is called:
   - System checks if the key exists for the user/bot combination
   - Verifies the entry hasn't expired
   - Returns the value or `null` if not found/expired

3. **Automatic Cleanup**: Expired entries are not returned and can be periodically cleaned up by maintenance tasks.

## Database Schema

The `REMEMBER` keyword uses the following database structure:

```sql
CREATE TABLE bot_memories (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    bot_id TEXT NOT NULL,
    session_id TEXT,
    key TEXT NOT NULL,
    value JSONB NOT NULL,
    created_at TEXT NOT NULL,
    expires_at TEXT,
    UNIQUE(user_id, bot_id, key)
);
```

## Comparison with Other Memory Keywords

| Keyword | Scope | Persistence | Expiration |
|---------|-------|-------------|------------|
| `SET USER MEMORY` | User | Permanent | No |
| `SET BOT MEMORY` | Bot (all users) | Permanent | No |
| `REMEMBER` | User | Configurable | Yes |
| `REMEMBER USER FACT` | User | Permanent | No |

## Best Practices

1. **Use descriptive keys**: Choose meaningful key names like `"last_login"` instead of `"ll"`.

2. **Set appropriate durations**: Match the duration to your use case:
   - Session data: minutes to hours
   - Preferences: weeks to months
   - Important data: `forever`

3. **Handle null values**: Always check if `RECALL` returns `null`:
   ```basic
   value = RECALL "some_key"
   IF value = null THEN
       ' Handle missing/expired data
   END IF
   ```

4. **Avoid storing sensitive data**: Don't store passwords, API keys, or other secrets.

## Error Handling

```basic
' REMEMBER returns a confirmation message on success
result = REMEMBER "key", "value", "1 day"
' result = "Remembered 'key' for 1 day"

' RECALL returns null if key doesn't exist or has expired
value = RECALL "nonexistent_key"
' value = null
```

## Related Keywords

- [SET USER MEMORY](./keyword-set-user-memory.md) - Permanent user-scoped storage
- [GET USER MEMORY](./keyword-get-user-memory.md) - Retrieve permanent user data
- [SET BOT MEMORY](./keyword-set-bot-memory.md) - Bot-wide storage
- [GET BOT MEMORY](./keyword-get-bot-memory.md) - Retrieve bot-wide data

## See Also

- [Memory Management](../chapter-10-features/memory-management.md)
- [User Session Handling](../chapter-10-features/user-sessions.md)