# GET BOT MEMORY

Retrieve persistent key-value data stored at the bot level.

## Syntax

```basic
GET BOT MEMORY key
```

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `key` | String | The identifier of the memory item to retrieve |

## Description

The `GET BOT MEMORY` keyword retrieves values previously stored with `SET BOT MEMORY`. These values are:
- Persistent across all user sessions
- Shared between all users of the same bot
- Stored in the database permanently
- Available until explicitly updated or cleared

If the key doesn't exist, returns an empty string.

## Examples

### Retrieve Simple Values
```basic
welcome = GET BOT MEMORY "welcome_message"
IF welcome = "" THEN
    welcome = "Welcome to our bot!"
END IF
TALK welcome
```

### Read Configuration
```basic
max_retries = GET BOT MEMORY "max_retries"
IF max_retries = "" THEN
    max_retries = "3"
END IF

timeout = GET BOT MEMORY "timeout_seconds"
IF timeout = "" THEN
    timeout = "30"
END IF
```

### Retrieve and Parse JSON
```basic
user_data = GET BOT MEMORY "user_profile"
IF user_data <> "" THEN
    ' Parse JSON data
    name = JSON_GET(user_data, "name")
    level = JSON_GET(user_data, "level")
    TALK "Welcome back, " + name + "! You are level " + level
END IF
```

### Counter Management
```basic
' Get current visitor count
count = GET BOT MEMORY "visitor_count"
IF count = "" THEN
    count = "0"
END IF
count = VAL(count) + 1
SET BOT MEMORY "visitor_count", STR(count)
TALK "Visitor #" + STR(count)
```

### Dynamic Keys
```basic
today = FORMAT(NOW(), "YYYY-MM-DD")
daily_stats = GET BOT MEMORY "stats_" + today
IF daily_stats = "" THEN
    TALK "No statistics for today yet"
ELSE
    TALK "Today's count: " + daily_stats
END IF
```

### Configuration with Defaults
```basic
' Function to get config with default
FUNCTION GetConfig(key, default_value)
    value = GET BOT MEMORY key
    IF value = "" THEN
        value = default_value
        SET BOT MEMORY key, default_value
    END IF
    RETURN value
END FUNCTION

' Use the function
email_server = GetConfig("email_server", "mail.example.com")
email_port = GetConfig("email_port", "587")
```

## Return Value

Returns a string containing:
- The stored value if the key exists
- Empty string ("") if the key doesn't exist
- Empty string if database error occurs

## Performance

- Direct database lookup with indexed keys
- Single query execution
- Synchronous operation (blocks until complete)
- Cached at database level for repeated access

## Best Practices

1. **Always Check for Empty Values**
   ```basic
   value = GET BOT MEMORY "some_key"
   IF value = "" THEN
       ' Handle missing value
       value = "default"
   END IF
   ```

2. **Use Consistent Key Naming**
   ```basic
   ' Good: hierarchical keys
   server = GET BOT MEMORY "config:email:server"
   port = GET BOT MEMORY "config:email:port"
   
   ' Bad: inconsistent naming
   ' srv = GET BOT MEMORY "emailSrv"
   ' p = GET BOT MEMORY "mail_port"
   ```

3. **Cache Frequently Used Values**
   ```basic
   ' At start of conversation
   config_timeout = GET BOT MEMORY "timeout"
   config_retries = GET BOT MEMORY "retries"
   
   ' Use cached values throughout
   IF elapsed > VAL(config_timeout) THEN
       TALK "Request timed out"
   END IF
   ```

4. **Validate Retrieved Data**
   ```basic
   max_items = GET BOT MEMORY "max_items"
   IF max_items = "" OR NOT IS_NUMERIC(max_items) THEN
       max_items = "10"
   END IF
   ```

## Error Handling

- Database connection failures return empty string
- Invalid bot IDs return empty string
- Non-existent keys return empty string
- All errors are logged for debugging

## Use Cases

### Global Configuration
```basic
api_key = GET BOT MEMORY "api_key"
api_url = GET BOT MEMORY "api_url"
```

### Feature Flags
```basic
feature_enabled = GET BOT MEMORY "feature:new_ui"
IF feature_enabled = "true" THEN
    ' Show new interface
ELSE
    ' Show old interface
END IF
```

### Shared Counters
```basic
total_processed = GET BOT MEMORY "total_processed"
daily_limit = GET BOT MEMORY "daily_limit"
IF VAL(total_processed) >= VAL(daily_limit) THEN
    TALK "Daily limit reached"
END IF
```

### Bot State
```basic
maintenance_mode = GET BOT MEMORY "maintenance_mode"
IF maintenance_mode = "true" THEN
    TALK "System is under maintenance. Please try again later."
    EXIT
END IF
```

## Related Keywords

- [SET BOT MEMORY](./keyword-set-bot-memory.md) - Store bot-level memory
- [SET](./keyword-set.md) - Set user-scoped variables
- [GET](./keyword-get.md) - Get user variables
- [REMEMBER](./keyword-remember.md) - Store user-specific memories

## Implementation

Located in `src/basic/keywords/bot_memory.rs`

The implementation:
- Performs synchronous database query
- Uses connection pooling for efficiency
- Returns empty string on any error
- Validates bot ID before querying