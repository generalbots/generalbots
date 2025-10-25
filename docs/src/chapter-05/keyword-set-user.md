# SET_USER Keyword

**Syntax**

```
SET_USER "user-id"
```

**Parameters**

- `"user-id"` – UUID string identifying the user.

**Description**

`SET_USER` updates the current session’s user identifier. This is useful when a dialog authenticates a user and wants to associate subsequent interactions with that user’s record.

**Example (from `auth.bas`)**

```basic
SET_USER "550e8400-e29b-41d4-a716-446655440000"
TALK "User authenticated."
```

After execution, all future bot messages in this session are linked to the specified user ID.
