# SET USER Keyword

Switch the current user context within a script execution.

## Syntax

```basic
SET USER user_id
```

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `user_id` | String (UUID) | The UUID of the user to switch to |

## Description

The `SET USER` keyword changes the active user context for subsequent operations in the script. This is useful for administrative scripts that need to perform actions on behalf of different users.

## Example

```basic
' Admin script to update user preferences
SET USER "550e8400-e29b-41d4-a716-446655440000"
SET USER MEMORY "theme", "dark"
SET USER MEMORY "language", "pt-BR"

TALK "User preferences updated."
```

## Example: Batch User Operations

```basic
' Process multiple users
users = GET "SELECT id FROM users WHERE needs_update = true"

FOR EACH user IN users
    SET USER user.id
    SET USER MEMORY "migrated", "true"
    SEND MAIL user.email, "Account Updated", "Your account has been migrated."
NEXT
```

## Use Cases

- Administrative batch operations
- Multi-tenant management scripts
- User impersonation for support
- Scheduled maintenance tasks

## Security

- Requires admin privileges to execute
- All actions are logged with original admin identity
- Cannot escalate privileges beyond script permissions

## See Also

- [SET USER MEMORY](./keyword-set-user-memory.md)
- [GET USER MEMORY](./keyword-get-user-memory.md)
- [SET CONTEXT](./keyword-set-context.md)