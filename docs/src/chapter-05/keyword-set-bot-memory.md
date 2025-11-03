# SET_BOT_MEMORY Keyword

The **SET_BOT_MEMORY** keyword stores a key-value pair in the bot’s persistent memory.  
This allows the bot to remember information across sessions, enabling long-term context and personalization.

---

## Syntax

```basic
SET_BOT_MEMORY "key" "value"
```

---

## Parameters

- `"key"` — The identifier for the memory entry to store.  
- `"value"` — The string value associated with the key.

---

## Description

`SET_BOT_MEMORY` saves data in the bot’s memory table, which is shared across all sessions for that bot instance.  
This memory is persistent and can be retrieved later using the `GET_BOT_MEMORY` keyword.  
It is useful for storing configuration values, user preferences, or other reusable data.

If the key already exists, its value is updated.  
If the key does not exist, a new entry is created automatically.

The operation is asynchronous — the command returns immediately while the memory update is processed in the background.

---

## Example

```basic
' Store a greeting message
SET_BOT_MEMORY "last_greeting" "Hello, world!"

' Retrieve it later
GET_BOT_MEMORY "last_greeting" INTO GREETING
TALK GREETING
```

---

## Implementation Notes

- Implemented in Rust under `src/shared/state.rs` and `src/shared/models.rs`.  
- Uses asynchronous database operations to update or insert memory entries.  
- Memory entries are scoped to the bot instance, not to individual users.  
- The system ensures thread-safe access using Tokio tasks and connection pooling.

---

## Related Keywords

- [`GET_BOT_MEMORY`](keyword-get-bot-memory.md) — Retrieves stored memory entries.  
- [`SET_CONTEXT`](keyword-set-context.md) — Defines the operational context for the session.  
- [`SET_USER`](keyword-set-user.md) — Associates the session with a specific user.  
- [`SET_SCHEDULE`](keyword-set-schedule.md) — Defines scheduled tasks that may depend on memory values.

---

## Summary

`SET_BOT_MEMORY` enables persistent data storage for bots, allowing them to maintain long-term state and recall information across sessions.  
It is a fundamental command for building intelligent, context-aware automation in GeneralBots.
