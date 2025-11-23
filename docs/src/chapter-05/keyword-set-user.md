# SET_USER Keyword

The **SET_USER** keyword defines the active user context for the current bot session.  
It associates all subsequent actions, memory entries, and responses with a specific user identity.

---

## Syntax

```basic
SET_USER "user-id"
```

---

## Parameters

- `"user-id"` — A unique identifier (UUID or username) representing the user.  
  This value is used to link session data, memory, and logs to the correct user record.

---

## Description

`SET_USER` updates the bot’s internal session to reflect the specified user identity.  
This is typically used after authentication or when switching between users in multi-user environments.  
Once set, all commands such as `SET_CONTEXT`, `SET_BOT_MEMORY`, and `FIND` operate within the scope of that user.

If the user ID does not exist in the database, the system automatically creates a new user record.  
The command ensures that the session token and memory cache are properly synchronized.

---

## Example

```basic
' Set the active user for the session
SET_USER "john_doe"

' Store personalized memory
SET_BOT_MEMORY "preferred_language" "English"

' Retrieve user-specific data
FIND "recent orders"
```

---

## Implementation Notes

- Implemented in Rust under `src/session/mod.rs` and `src/org/mod.rs`.  
- The keyword interacts with the session manager to update the active user ID.  
- It ensures that all subsequent operations are scoped to the correct user context.  
- If cache component or database caching is enabled, the user ID is stored for persistence across sessions.

---

## Related Keywords

- [`SET_CONTEXT`](keyword-set-context.md) — Defines the operational context for the session.  
- [`SET_BOT_MEMORY`](keyword-set-bot-memory.md) — Stores persistent data for the bot or user.  
- [`GET_BOT_MEMORY`](keyword-get-bot-memory.md) — Retrieves stored memory entries.

---

## Summary

`SET_USER` is essential for maintaining user-specific state and personalization in GeneralBots.  
It ensures that each session operates independently and securely under the correct user identity.
