# Permissions Matrix

This chapter documents the permission system in General Bots, detailing which APIs require authentication, the context they operate under, and how to configure access control.

## Overview

General Bots uses a role-based access control (RBAC) system managed through Zitadel (directory service). Permissions are organized into:

- **Realms**: Top-level permission boundaries (typically per organization)
- **Groups**: Collections of users with shared permissions
- **Permissions**: Specific actions that can be granted to groups

## User Context vs System Context

APIs operate in one of two contexts:

| Context | Description | Authentication |
|---------|-------------|----------------|
| **User Context** | Operations performed on behalf of a logged-in user | User's OAuth token |
| **System Context** | Operations performed by the bot or system | Service account token |

### User Context Operations

These operations use the authenticated user's identity:

- Reading user's own files
- Sending messages as the user
- Accessing user's calendar
- Managing user's tasks
- Viewing user's email

### System Context Operations

These operations use a service account:

- Bot-initiated messages
- Scheduled tasks execution
- System monitoring
- Cross-user analytics
- Backup operations

## API Permission Matrix

### File APIs

| Endpoint | Method | User Context | System Context | Required Permission |
|----------|--------|--------------|----------------|---------------------|
| `/api/drive/list` | GET | User's files | All bot files | `files:read` |
| `/api/drive/upload` | POST | User's folder | Bot storage | `files:write` |
| `/api/drive/delete` | DELETE | User's files | Any file | `files:delete` |
| `/api/drive/share` | POST | Own files | Any file | `files:share` |

### Email APIs

| Endpoint | Method | User Context | System Context | Required Permission |
|----------|--------|--------------|----------------|---------------------|
| `/api/email/inbox` | GET | User's inbox | N/A | `email:read` |
| `/api/email/send` | POST | As user | As bot | `email:send` |
| `/api/email/drafts` | GET | User's drafts | N/A | `email:read` |

### Meet APIs

| Endpoint | Method | User Context | System Context | Required Permission |
|----------|--------|--------------|----------------|---------------------|
| `/api/meet/rooms` | GET | Visible rooms | All rooms | `meet:read` |
| `/api/meet/create` | POST | As organizer | As bot | `meet:create` |
| `/api/meet/join` | POST | As participant | As bot participant | `meet:join` |
| `/api/meet/invite` | POST | Own meetings | Any meeting | `meet:invite` |

### Calendar APIs

| Endpoint | Method | User Context | System Context | Required Permission |
|----------|--------|--------------|----------------|---------------------|
| `/api/calendar/events` | GET | User's events | Bot calendar | `calendar:read` |
| `/api/calendar/create` | POST | User's calendar | Bot calendar | `calendar:write` |
| `/api/calendar/book` | POST | As attendee | As organizer | `calendar:book` |

### Tasks APIs

| Endpoint | Method | User Context | System Context | Required Permission |
|----------|--------|--------------|----------------|---------------------|
| `/api/tasks` | GET | User's tasks | All tasks | `tasks:read` |
| `/api/tasks` | POST | Assigned to user | Any assignment | `tasks:write` |
| `/api/tasks/complete` | POST | Own tasks | Any task | `tasks:complete` |

### Admin APIs

| Endpoint | Method | User Context | System Context | Required Permission |
|----------|--------|--------------|----------------|---------------------|
| `/api/admin/users` | GET | N/A | Full access | `admin:users` |
| `/api/admin/bots` | GET | N/A | Full access | `admin:bots` |
| `/api/admin/config` | PUT | N/A | Full access | `admin:config` |
| `/api/monitoring/status` | GET | N/A | Full access | `admin:monitor` |

## Permission Definitions

### Core Permissions

| Permission | Description |
|------------|-------------|
| `chat:read` | View conversation history |
| `chat:write` | Send messages |
| `files:read` | View and download files |
| `files:write` | Upload and modify files |
| `files:delete` | Delete files |
| `files:share` | Share files with others |

### Communication Permissions

| Permission | Description |
|------------|-------------|
| `email:read` | Read email messages |
| `email:send` | Send email messages |
| `meet:read` | View meeting information |
| `meet:create` | Create new meetings |
| `meet:join` | Join meetings |
| `meet:invite` | Invite others to meetings |

### Productivity Permissions

| Permission | Description |
|------------|-------------|
| `calendar:read` | View calendar events |
| `calendar:write` | Create/modify events |
| `calendar:book` | Book appointments |
| `tasks:read` | View tasks |
| `tasks:write` | Create/modify tasks |
| `tasks:complete` | Mark tasks complete |

### Administrative Permissions

| Permission | Description |
|------------|-------------|
| `admin:users` | Manage users |
| `admin:groups` | Manage groups |
| `admin:bots` | Manage bot configurations |
| `admin:config` | Modify system configuration |
| `admin:monitor` | Access monitoring data |
| `admin:backup` | Perform backup operations |

## Default Groups

General Bots creates these default groups:

### Administrators

```
Permissions:
  - admin:*
  - All other permissions
```

Full system access for system administrators.

### Managers

```
Permissions:
  - chat:read, chat:write
  - files:read, files:write, files:share
  - email:read, email:send
  - meet:*, calendar:*, tasks:*
  - admin:monitor
```

Access to productivity features and basic monitoring.

### Users

```
Permissions:
  - chat:read, chat:write
  - files:read, files:write
  - email:read, email:send
  - meet:read, meet:join
  - calendar:read, calendar:write
  - tasks:read, tasks:write, tasks:complete
```

Standard user access to all productivity features.

### Guests

```
Permissions:
  - chat:read, chat:write
```

Chat-only access for anonymous or guest users.

## Configuring Permissions

### In Zitadel

1. Access Zitadel admin console
2. Navigate to **Organization** → **Roles**
3. Create roles matching permission names
4. Assign roles to groups
5. Add users to groups

### In config.csv

Map Zitadel roles to General Bots permissions:

```csv
key,value
permission-mapping-admin,admin:*
permission-mapping-manager,chat:*|files:*|email:*|meet:*|calendar:*|tasks:*
permission-mapping-user,chat:*|files:read|files:write
permission-default-anonymous,chat:read|chat:write
```

## Anonymous Access

The chat interface supports anonymous users:

| Feature | Anonymous Access | Notes |
|---------|-----------------|-------|
| Chat (default bot) | Yes | Session-based |
| Chat history | No | Requires login |
| Drive access | No | Requires login |
| Mail access | No | Requires login |
| Tasks | No | Requires login |
| Meet | No | Requires login |
| Settings | No | Requires login |

Anonymous users:
- Can chat with the default bot only
- Session stored on server
- No persistent history
- Cannot access other tabs

## Checking Permissions in BASIC

Use role-based logic in your scripts:

```basic
' Get user role from session
role = GET role

' Check role and respond accordingly
IF role = "admin" THEN
    TALK "Welcome, administrator. You have full access."
ELSE IF role = "manager" THEN
    TALK "Welcome, manager. You can view reports."
ELSE
    TALK "Welcome! How can I help you today?"
END IF
```

## Audit Logging

All permission checks are logged. Access audit logs through the admin API:

```
GET /api/admin/audit?filter=permission
```

Log entries include:
- Timestamp
- User ID
- Action attempted
- Resource accessed
- Result (allowed/denied)
- Reason for denial (if applicable)

## See Also

- [User Authentication](./user-auth.md) - Login and session management
- [User Context vs System Context](./user-system-context.md) - Detailed context explanation
- [Security Policy](./security-policy.md) - Security guidelines
- [API Endpoints](./api-endpoints.md) - Full API reference