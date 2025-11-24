# Users API

The Users API provides endpoints for user management operations. Currently, user management is handled entirely through Zitadel, with minimal direct API endpoints in BotServer.

## Status

**Partially Implemented** - User data is managed by Zitadel. BotServer only maintains session associations.

## Current Implementation

### Get Current User

**GET** `/api/users/me`

Returns current authenticated user information from session.

Headers:
- `Authorization: Bearer {session_token}`

Response:
```json
{
  "user_id": "user-123",
  "username": "john_doe",
  "email": "john@example.com",
  "created_at": "2024-01-01T00:00:00Z"
}
```

Note: This data is cached from Zitadel and may not be real-time.

## Planned Endpoints (Not Implemented)

The following endpoints are planned but not yet implemented:

### List Users

**GET** `/api/users` (Planned)

Would list users in the organization.

### Get User by ID

**GET** `/api/users/:id` (Planned)

Would retrieve specific user details.

### Update User

**PUT** `/api/users/:id` (Planned)

Would update user information.

### Delete User

**DELETE** `/api/users/:id` (Planned)

Would deactivate user account.

## User Management via Zitadel

Currently, all user management operations must be performed through:

1. **Zitadel Admin Console**
   - Create users
   - Update profiles
   - Reset passwords
   - Manage roles
   - Deactivate accounts

2. **Zitadel API**
   - Direct API calls to Zitadel
   - Not proxied through BotServer
   - Requires Zitadel authentication

## Database Schema

Users are minimally stored in BotServer:

```sql
-- users table
CREATE TABLE users (
    id UUID PRIMARY KEY,
    zitadel_id TEXT UNIQUE,
    username TEXT,
    email TEXT,
    created_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ
);
```

This is only for session management and caching.

## Authentication

All user authentication happens through Zitadel:
- No password storage in BotServer
- OAuth2/OIDC flow for login
- Sessions managed locally
- Token validation through Zitadel

## Future Implementation

When fully implemented, the Users API will:
- Proxy Zitadel operations
- Provide unified API interface
- Cache user data for performance
- Handle organization-specific operations
- Integrate with bot permissions

## Current Workarounds

To manage users today:

1. **Use Zitadel Console**
   - Access at Zitadel URL (usually port 8080)
   - Full user management capabilities
   - Role and permission assignment

2. **Direct Zitadel API**
   - Use Zitadel's REST API
   - Authenticate with service account
   - Not integrated with BotServer auth

3. **Session Management**
   - Users tracked via sessions
   - Minimal profile data cached
   - No direct user CRUD operations

## Summary

User management in BotServer is delegated to Zitadel. The Users API provides minimal endpoints for session-related user data. Full user management requires using Zitadel's admin console or API directly.