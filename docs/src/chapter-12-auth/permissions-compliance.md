# API Permissions Compliance Matrix

This document provides a detailed compliance matrix showing which documented permissions are actually enforced in the API code, and identifies gaps between documentation and implementation.

## Overview

General Bots uses a role-based access control (RBAC) system where:
- **Permissions** are assigned to **Groups**
- **Groups** contain **Users**
- **Users** can belong to multiple **Groups**
- Effective permissions = union of all group permissions

## Permission Model

```
┌─────────────────────────────────────────────────────────────┐
│                      PERMISSION MODEL                        │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│   ┌──────────┐     ┌──────────┐     ┌──────────┐          │
│   │Permission│────▶│  Group   │────▶│   User   │          │
│   │files:read│     │ managers │     │  john    │          │
│   └──────────┘     └──────────┘     └──────────┘          │
│                          │                │                │
│   ┌──────────┐           │                │                │
│   │Permission│───────────┘                │                │
│   │email:send│                            ▼                │
│   └──────────┘     ┌──────────┐     ┌──────────┐          │
│                    │  Group   │────▶│   User   │          │
│   ┌──────────┐     │  admins  │     │  mary    │          │
│   │Permission│────▶└──────────┘     └──────────┘          │
│   │ admin:*  │           │                                 │
│   └──────────┘           └─────────────────┘               │
│                                                             │
│   User can belong to multiple groups                       │
│   Effective permissions = union of all group permissions   │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## File APIs Compliance

| Endpoint | Method | Documented Permission | Code Location | Status | Notes |
|----------|--------|----------------------|---------------|--------|-------|
| `/api/drive/list` | GET | `files:read` | `file_operations.rs` | ⚠️ Partial | User context validated, permission check implicit |
| `/api/drive/upload` | POST | `files:write` | `file_operations.rs` | ⚠️ Partial | Bot storage validated, no explicit permission check |
| `/api/drive/delete` | DELETE | `files:delete` | `file_operations.rs` | ⚠️ Partial | User session required, no role validation |
| `/api/drive/share` | POST | `files:share` | Not implemented | ❌ Missing | Endpoint not yet implemented |
| `/api/drive/download` | GET | `files:read` | `file_operations.rs` | ⚠️ Partial | Path validation only |

## Email APIs Compliance

| Endpoint | Method | Documented Permission | Code Location | Status | Notes |
|----------|--------|----------------------|---------------|--------|-------|
| `/api/email/inbox` | GET | `email:read` | Not implemented | ❌ Missing | REST endpoint pending |
| `/api/email/send` | POST | `email:send` | `send_mail.rs` | ⚠️ Partial | Via BASIC keyword only |
| `/api/email/drafts` | GET | `email:read` | `create_draft.rs` | ⚠️ Partial | Via BASIC keyword only |

## Meet APIs Compliance

| Endpoint | Method | Documented Permission | Code Location | Status | Notes |
|----------|--------|----------------------|---------------|--------|-------|
| `/api/meet/rooms` | GET | `meet:read` | Not implemented | ❌ Missing | - |
| `/api/meet/create` | POST | `meet:create` | Not implemented | ❌ Missing | - |
| `/api/meet/join` | POST | `meet:join` | Not implemented | ❌ Missing | - |
| `/api/meet/invite` | POST | `meet:invite` | Not implemented | ❌ Missing | - |

## Calendar APIs Compliance

| Endpoint | Method | Documented Permission | Code Location | Status | Notes |
|----------|--------|----------------------|---------------|--------|-------|
| `/api/calendar/events` | GET | `calendar:read` | Not implemented | ❌ Missing | - |
| `/api/calendar/create` | POST | `calendar:write` | Not implemented | ❌ Missing | - |
| `/api/calendar/book` | POST | `calendar:book` | `book.rs` | ⚠️ Partial | Via BASIC keyword |

## Tasks APIs Compliance

| Endpoint | Method | Documented Permission | Code Location | Status | Notes |
|----------|--------|----------------------|---------------|--------|-------|
| `/api/tasks` | GET | `tasks:read` | `create_task.rs` | ⚠️ Partial | Read via BASIC |
| `/api/tasks` | POST | `tasks:write` | `create_task.rs` | ⚠️ Partial | Via BASIC keyword |
| `/api/tasks/complete` | POST | `tasks:complete` | Not implemented | ❌ Missing | - |

## Admin APIs Compliance

| Endpoint | Method | Documented Permission | Code Location | Status | Notes |
|----------|--------|----------------------|---------------|--------|-------|
| `/api/admin/users` | GET | `admin:users` | `directory/api.rs` | ✅ Implemented | `is_admin` check |
| `/api/admin/bots` | GET | `admin:bots` | `api.rs` | ⚠️ Partial | Basic validation |
| `/api/admin/config` | PUT | `admin:config` | Not implemented | ❌ Missing | - |
| `/api/monitoring/status` | GET | `admin:monitor` | `monitoring.rs` | ✅ Implemented | Health endpoints |

## User Management APIs Compliance

| Endpoint | Method | Documented Permission | Code Location | Status | Notes |
|----------|--------|----------------------|---------------|--------|-------|
| `/api/users/me` | GET | authenticated | `directory/api.rs` | ✅ Implemented | Session validated |
| `/api/users/me` | PUT | authenticated | `directory/api.rs` | ⚠️ Partial | Limited fields |
| `/api/users/:id` | GET | `admin:users` | `directory/api.rs` | ✅ Implemented | Admin check |
| `/api/users` | GET | `admin:users` | `directory/api.rs` | ✅ Implemented | List users |

## Group Management APIs Compliance

| Endpoint | Method | Documented Permission | Code Location | Status | Notes |
|----------|--------|----------------------|---------------|--------|-------|
| `/api/groups/create` | POST | `admin:groups` | `add_member.rs` | ✅ Implemented | Via CREATE TEAM |
| `/api/groups/list` | GET | `admin:groups` | `add_member.rs` | ⚠️ Partial | Basic implementation |
| `/api/groups/:id/members` | GET | `admin:groups` | `add_member.rs` | ✅ Implemented | - |
| `/api/groups/:id/members` | POST | `admin:groups` | `add_member.rs` | ✅ Implemented | ADD MEMBER |

## BASIC Keywords Permission Enforcement

| Keyword | Permission Required | Enforcement | Status |
|---------|-------------------|-------------|--------|
| `GET` (file) | `files:read` | User session + path validation | ⚠️ Partial |
| `GET` (HTTP) | None | URL validation only | ✅ OK |
| `SAVE` | `files:write` | User session only | ⚠️ Partial |
| `DELETE` (data) | `data:write` | Table validation | ⚠️ Partial |
| `DELETE FILE` | `files:delete` | User session + path | ⚠️ Partial |
| `DELETE HTTP` | None | URL validation | ✅ OK |
| `SEND MAIL` | `email:send` | User session only | ⚠️ Partial |
| `WEBHOOK` | `admin:webhooks` | Bot ownership only | ⚠️ Partial |
| `SET SCHEDULE` | `admin:schedules` | Bot ownership only | ⚠️ Partial |
| `ADD MEMBER` | `admin:groups` | Role validation | ✅ Implemented |
| `GET BOT MEMORY` | `bot:read` | Bot session only | ⚠️ Partial |
| `SET BOT MEMORY` | `bot:write` | Bot session only | ⚠️ Partial |

## Permission Check Implementation Status

### Fully Implemented ✅

These have explicit permission checks in code:

| Location | Check Type | Code Reference |
|----------|-----------|----------------|
| User listing | `is_admin` flag | `directory/api.rs:229` |
| User creation | `is_admin` flag | `provisioning.rs:88` |
| Group management | Role-based | `add_member.rs:172` |
| Member permissions | Role hierarchy | `add_member.rs:304` |

### Partially Implemented ⚠️

These validate user session but not specific permissions:

| Location | Current Check | Missing |
|----------|--------------|---------|
| File operations | User session exists | Permission-specific validation |
| Data operations | Bot ownership | User permission check |
| Email operations | User session | `email:send` permission |
| HTTP operations | None (external) | Rate limiting per role |

### Not Implemented ❌

These need implementation:

| Feature | Priority | Recommendation |
|---------|----------|----------------|
| Fine-grained file permissions | High | Check `files:read/write/delete` |
| Calendar API permissions | Medium | Implement RBAC |
| Meet API permissions | Medium | Implement RBAC |
| Task completion tracking | Low | Add `tasks:complete` |

## Compliance Gaps Summary

### Critical Gaps (Security Risk)

1. **File Operations**: No permission-level check beyond session
   - Risk: Any authenticated user can access any bot's files
   - Fix: Add `files:read/write/delete` checks in `file_operations.rs`

2. **Data Operations**: SQL injection protection only, no authorization
   - Risk: Users can modify data they shouldn't access
   - Fix: Add row-level security or permission checks

### Medium Gaps (Functionality)

1. **Email API**: BASIC keywords work, REST endpoints missing
2. **Calendar/Meet APIs**: Documented but not implemented
3. **Role Inheritance**: Groups don't cascade permissions properly

### Low Gaps (Documentation)

1. **Anonymous Access**: Well documented, correctly implemented
2. **Admin APIs**: Working but documentation could be clearer

## Recommendations

### Immediate Actions

1. Add middleware for permission checking:

```rust
// Proposed middleware pattern
async fn require_permission(
    permission: &str,
    user: &UserSession,
    state: &AppState,
) -> Result<(), AuthError> {
    let user_permissions = get_user_permissions(user, state).await?;
    if !user_permissions.contains(permission) {
        return Err(AuthError::InsufficientPermissions);
    }
    Ok(())
}
```

2. Update BASIC keyword registration to include permission requirements:

```rust
// Add permission metadata to keyword registration
pub struct KeywordDefinition {
    pub name: &'static str,
    pub required_permission: Option<&'static str>,
    pub handler: fn(...),
}
```

### Short-term Actions

1. Implement missing REST API endpoints for documented features
2. Add permission checks to all file operations
3. Create audit logging for permission denials

### Long-term Actions

1. Implement full RBAC with permission inheritance
2. Add row-level security for data operations
3. Create admin UI for permission management

## Testing Permissions

To verify permission enforcement:

```basic
' Test script: check-permissions.bas
role = GET role

IF role = "admin" THEN
    ' Test admin operations
    users = GET "/api/admin/users"
    TALK "Admin access confirmed: " + users.count + " users"
ELSE
    TALK "You have role: " + role
    TALK "Admin operations not available"
END IF
```

## See Also

- [Permissions Matrix](./permissions-matrix.md) - Full permission definitions
- [User Authentication](./user-auth.md) - Authentication flow
- [Security Policy](./security-policy.md) - Security guidelines
- [API Endpoints](./api-endpoints.md) - Full API reference