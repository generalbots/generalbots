# Chapter 8: REST API & Tools

HTTP API endpoints for integrating with botserver.

## Overview

botserver exposes REST endpoints organized by functional area. All endpoints follow consistent patterns for authentication, pagination, and error handling.

## Base URL

```
http://localhost:8000/api/v1
```

## Authentication

```bash
Authorization: Bearer <token>
```

## API Categories

| Category | Prefix | Description |
|----------|--------|-------------|
| **User APIs** | `/api/user/*` | Personal settings, profile, preferences |
| **Admin APIs** | `/api/admin/*` | Organization management (requires admin role) |
| **Files** | `/files/*` | Drive operations |
| **Chat** | `/chat/*` | Conversations and messages |

## User vs Admin Endpoints

The API separates user-level and admin-level operations:

**User Endpoints** (`/api/user/*`):
- Personal profile and settings
- User's own files and data
- Individual preferences
- Accessible by all authenticated users

**Admin Endpoints** (`/api/admin/*`):
- Organization-wide settings
- User management
- Group management
- DNS, billing, audit logs
- Requires `admin` role

## Quick Example

```bash
curl -X POST http://localhost:8000/api/v1/chat \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"message": "Hello", "session_id": "abc123"}'
```

## Response Format

```json
{
  "success": true,
  "data": { ... },
  "error": null
}
```

## Chapter Contents

- [Files API](./files-api.md) - Upload/download
- [Document Processing](./document-processing.md) - Text extraction
- [Users API](./users-api.md) - User management
- [User Security API](./user-security.md) - 2FA, sessions
- [Groups API](./groups-api.md) - Group management
- [Conversations API](./conversations-api.md) - Chat sessions
- [Calendar API](./calendar-api.md) - Scheduling
- [Tasks API](./tasks-api.md) - Task management
- [Storage API](./storage-api.md) - Object storage
- [Analytics API](./analytics-api.md) - Metrics
- [Admin API](./admin-api.md) - Administration
- [AI API](./ai-api.md) - LLM endpoints
- [Example Integrations](./examples.md) - Code samples

## See Also

- [LLM Tools](../08-rest-api-tools/README.md) - Tool definitions
- [Authentication](../09-security/README.md) - Security
- [Permissions Matrix](../09-security/permissions-matrix.md) - Access control