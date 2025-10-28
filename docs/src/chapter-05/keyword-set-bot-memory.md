# SET_BOT_MEMORY Keyword

**Syntax**

```
SET_BOT_MEMORY "key", "value"
```

**Parameters**

- `"key"` – Identifier for the memory entry to store.
- `"value"` – The string value to associate with the key.

**Description**

`SET_BOT_MEMORY` stores a key‑value pair in the persistent bot memory table (`bot_memories`). The entry is scoped to the bot instance, not to a specific user. If the key already exists, its value is updated; otherwise a new row is inserted. The operation is performed asynchronously in a background task, so the keyword returns immediately.

**Example**

```basic
SET_BOT_MEMORY "last_greeting", "Hello, world!"
TALK "Bot memory updated."
```

After execution, the key `last_greeting` will contain the value `"Hello, world!"` and can be retrieved later with `GET_BOT_MEMORY`.

**Implementation Notes**

- The keyword spawns a Tokio task that acquires a database connection, checks for an existing entry, and either updates or inserts the record.
- Errors are logged but do not interrupt script execution; the keyword always returns `UNIT`.
- Values are stored as plain strings; binary data should be encoded (e.g., Base64) before storage.
