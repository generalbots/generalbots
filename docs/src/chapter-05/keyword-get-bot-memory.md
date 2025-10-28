# GET_BOT_MEMORY Keyword

**Syntax**

```
GET_BOT_MEMORY "key"
```

**Parameters**

- `"key"` – The memory key to retrieve.

**Description**

`GET_BOT_MEMORY` reads a value stored in the bot’s persistent memory table (`bot_memories`). It returns the stored string or an empty string if the key does not exist.

**Example (from `auth.bas`)**

```basic
SET attempts = GET_BOT_MEMORY "login_attempts"
IF attempts = "" THEN
    SET attempts = "0"
ENDIF
```

The script fetches the number of previous login attempts, defaulting to zero if the key is missing.
