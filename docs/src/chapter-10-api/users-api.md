# Users API

The Users API provides endpoints for user management operations. User authentication is handled through Zitadel, with BotServer maintaining session associations and user preferences.

## Overview

User management in General Bots follows a federated model:

- **Zitadel**: Primary identity provider (authentication, SSO, user creation)
- **BotServer**: Session management, preferences, bot-specific user data

## Endpoints

### Get Current User

**GET** `/api/users/me`

Returns current authenticated user information.

**Headers:**
```
Authorization: Bearer {session_token}
```

**Response:**
```json
{
  "user_id": "user-123",
  "username": "john_doe",
  "email": "john@example.com",
  "display_name": "John Doe",
  "avatar_url": "/api/users/user-123/avatar",
  "roles": ["user", "manager"],
  "created_at": "2024-01-01T00:00:00Z",
  "last_login": "2024-01-15T10:30:00Z"
}
```

### Get User by ID

**GET** `/api/users/:id`

Retrieve specific user details.

**Required Permission:** `admin:users` or same user

**Response:**
```json
{
  "user_id": "user-123",
  "username": "john_doe",
  "email": "john@example.com",
  "display_name": "John Doe",
  "status": "active",
  "created_at": "2024-01-01T00:00:00Z"
}
```

### List Users

**GET** `/api/users`

List users in the organization.

**Required Permission:** `admin:users`

**Query Parameters:**
- `limit` - Number of results (default: 50, max: 100)
- `offset` - Pagination offset
- `status` - Filter by status (active/suspended/inactive)
- `role` - Filter by role
- `search` - Search by name or email

**Response:**
```json
{
  "users": [
    {
      "user_id": "user-123",
      "username": "john_doe",
      "email": "john@example.com",
      "display_name": "John Doe",
      "status": "active",
      "roles": ["user", "manager"]
    },
    {
      "user_id": "user-456",
      "username": "jane_smith",
      "email": "jane@example.com",
      "display_name": "Jane Smith",
      "status": "active",
      "roles": ["user"]
    }
  ],
  "total": 47,
  "limit": 50,
  "offset": 0
}
```

### Update User

**PUT** `/api/users/:id`

Update user information.

**Required Permission:** `admin:users` or same user (limited fields)

**Request:**
```json
{
  "display_name": "John D. Doe",
  "avatar_url": "https://example.com/avatar.jpg"
}
```

**Admin-only fields:**
```json
{
  "status": "suspended",
  "roles": ["user"]
}
```

**Response:**
```json
{
  "user_id": "user-123",
  "status": "updated",
  "updated_fields": ["display_name"]
}
```

### Update User Settings

**PUT** `/api/users/:id/settings`

Update user preferences.

**Request:**
```json
{
  "theme": "dark",
  "language": "en",
  "notifications": {
    "email": true,
    "push": false,
    "digest": "daily"
  },
  "default_bot": "support-bot"
}
```

**Response:**
```json
{
  "status": "updated",
  "settings": {
    "theme": "dark",
    "language": "en"
  }
}
```

### Get User Settings

**GET** `/api/users/:id/settings`

Retrieve user preferences.

**Response:**
```json
{
  "theme": "dark",
  "language": "en",
  "timezone": "America/New_York",
  "notifications": {
    "email": true,
    "push": false,
    "digest": "daily"
  },
  "default_bot": "support-bot"
}
```

### Suspend User

**POST** `/api/users/:id/suspend`

Suspend a user account.

**Required Permission:** `admin:users`

**Request:**
```json
{
  "reason": "Policy violation"
}
```

**Response:**
```json
{
  "user_id": "user-123",
  "status": "suspended",
  "suspended_at": "2024-01-15T10:30:00Z"
}
```

### Activate User

**POST** `/api/users/:id/activate`

Reactivate a suspended user.

**Required Permission:** `admin:users`

**Response:**
```json
{
  "user_id": "user-123",
  "status": "active",
  "activated_at": "2024-01-15T10:30:00Z"
}
```

### Delete User

**DELETE** `/api/users/:id`

Deactivate/delete user account.

**Required Permission:** `admin:users`

**Response:**
```json
{
  "user_id": "user-123",
  "status": "deleted",
  "deleted_at": "2024-01-15T10:30:00Z"
}
```

## User Sessions

### List User Sessions

**GET** `/api/users/:id/sessions`

List active sessions for a user.

**Response:**
```json
{
  "sessions": [
    {
      "session_id": "sess-001",
      "bot_id": "support-bot",
      "started_at": "2024-01-15T09:00:00Z",
      "last_activity": "2024-01-15T10:30:00Z",
      "device": "Chrome on Windows"
    }
  ]
}
```

### Terminate Session

**DELETE** `/api/users/:id/sessions/:session_id`

End a specific user session.

**Response:**
```json
{
  "session_id": "sess-001",
  "status": "terminated"
}
```

### Terminate All Sessions

**DELETE** `/api/users/:id/sessions`

End all user sessions (logout everywhere).

**Response:**
```json
{
  "terminated_count": 3,
  "status": "all_sessions_terminated"
}
```

## User Authentication Flow

### Login

**POST** `/api/users/login`

Authenticate user (redirects to Zitadel).

**Request:**
```json
{
  "email": "user@example.com",
  "password": "password",
  "remember_me": true
}
```

**Response:**
```json
{
  "redirect_url": "https://auth.yourdomain.com/oauth/authorize?..."
}
```

### Logout

**POST** `/api/users/logout`

End current session.

**Response:**
```json
{
  "status": "logged_out",
  "redirect_url": "/"
}
```

### Register

**POST** `/api/users/register`

Register new user (if self-registration enabled).

**Request:**
```json
{
  "email": "newuser@example.com",
  "username": "newuser",
  "password": "SecurePassword123!",
  "display_name": "New User"
}
```

**Response:**
```json
{
  "user_id": "user-789",
  "status": "pending_verification",
  "message": "Check your email to verify your account"
}
```

## User Management via Zitadel

For full user management, access Zitadel admin console:

1. **Access Console**: `http://localhost:8080` (or your Zitadel URL)
2. **Create Users**: Organization → Users → Add
3. **Manage Roles**: Users → Select User → Authorizations
4. **Reset Passwords**: Users → Select User → Actions → Reset Password
5. **Configure SSO**: Settings → Identity Providers

## Database Schema

BotServer maintains minimal user data:

```sql
-- users table (synced from Zitadel)
CREATE TABLE users (
    id UUID PRIMARY KEY,
    zitadel_id TEXT UNIQUE,
    username TEXT,
    email TEXT,
    display_name TEXT,
    avatar_url TEXT,
    status TEXT DEFAULT 'active',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- user_settings table
CREATE TABLE user_settings (
    id UUID PRIMARY KEY,
    user_id UUID REFERENCES users(id),
    setting_key TEXT NOT NULL,
    setting_value TEXT,
    UNIQUE(user_id, setting_key)
);

-- user_sessions table
CREATE TABLE sessions (
    id UUID PRIMARY KEY,
    user_id UUID REFERENCES users(id),
    bot_id UUID,
    status TEXT DEFAULT 'active',
    device_info TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    last_activity TIMESTAMPTZ DEFAULT NOW()
);
```

## Error Handling

| Status Code | Error | Description |
|-------------|-------|-------------|
| 400 | `invalid_request` | Malformed request |
| 401 | `unauthorized` | Not authenticated |
| 403 | `forbidden` | Insufficient permissions |
| 404 | `user_not_found` | User doesn't exist |
| 409 | `conflict` | Username/email already exists |
| 422 | `validation_error` | Invalid field values |

## Rate Limits

| Endpoint | Limit |
|----------|-------|
| Login | 10/minute per IP |
| Register | 5/hour per IP |
| User List | 60/minute per user |
| User Update | 30/minute per user |

## BASIC Integration

Access user information in scripts:

```basic
' Get current user info
user_name = GET user_name
user_email = GET user_email

' Greet by name
TALK "Hello, " + user_name + "!"

' Check user role
role = GET role
IF role = "admin" THEN
    TALK "Welcome, administrator!"
END IF
```

## See Also

- [User Authentication](../chapter-12-auth/user-auth.md) - Auth details
- [Permissions Matrix](../chapter-12-auth/permissions-matrix.md) - Access control
- [Groups API](./groups-api.md) - Group management
- [SET USER Keyword](../chapter-06-gbdialog/keyword-set-user.md) - BASIC user context