# User Security API

BotServer provides RESTful endpoints for user management, authentication, authorization, and security features.

## Overview

The User Security API enables:
- User authentication and sessions
- Role-based access control
- Security settings management
- Audit logging
- Password policies
- Two-factor authentication

## Base URL

```
http://localhost:8080/api/v1/security
```

## Authentication

Most security endpoints require authentication:

```http
Authorization: Bearer <token>
```

## User Management

### Create User

**POST** `/users`

Create a new user account.

**Request Body:**
```json
{
  "username": "johndoe",
  "email": "john@example.com",
  "full_name": "John Doe",
  "role": "user",
  "groups": ["support_team"],
  "metadata": {
    "department": "Customer Service",
    "employee_id": "EMP001"
  }
}
```

**Response:**
```json
{
  "user_id": "usr_abc123",
  "username": "johndoe",
  "email": "john@example.com",
  "created_at": "2024-01-15T10:00:00Z",
  "status": "pending_activation"
}
```

### Get User

**GET** `/users/{user_id}`

Retrieve user information.

**Response:**
```json
{
  "user_id": "usr_abc123",
  "username": "johndoe",
  "email": "john@example.com",
  "full_name": "John Doe",
  "role": "user",
  "groups": ["support_team"],
  "status": "active",
  "created_at": "2024-01-15T10:00:00Z",
  "last_login": "2024-01-15T14:30:00Z",
  "email_verified": true,
  "two_factor_enabled": false
}
```

### Update User

**PATCH** `/users/{user_id}`

Update user information.

**Request Body:**
```json
{
  "full_name": "John Smith",
  "role": "admin",
  "groups": ["support_team", "admin_team"]
}
```

### Delete User

**DELETE** `/users/{user_id}`

Delete or deactivate a user account.

**Response:**
```json
{
  "user_id": "usr_abc123",
  "status": "deactivated",
  "deactivated_at": "2024-01-15T15:00:00Z"
}
```

### List Users

**GET** `/users`

List all users with pagination and filters.

**Query Parameters:**
- `page` - Page number (default: 1)
- `limit` - Items per page (default: 20)
- `role` - Filter by role
- `group` - Filter by group
- `status` - Filter by status: `active`, `inactive`, `pending`
- `search` - Search in username, email, full name

**Response:**
```json
{
  "users": [
    {
      "user_id": "usr_abc123",
      "username": "johndoe",
      "email": "john@example.com",
      "full_name": "John Doe",
      "role": "user",
      "status": "active"
    }
  ],
  "total": 150,
  "page": 1,
  "limit": 20
}
```

## Authentication

### Login

**POST** `/auth/login`

Authenticate user via directory service.

**Request Body:**
```json
{
  "username": "johndoe",
  "password": "secure_password",
  "two_factor_code": "123456"
}
```

**Response:**
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIs...",
  "refresh_token": "eyJhbGciOiJIUzI1NiIs...",
  "token_type": "Bearer",
  "expires_in": 3600,
  "user": {
    "user_id": "usr_abc123",
    "username": "johndoe",
    "role": "user"
  }
}
```

### Refresh Token

**POST** `/auth/refresh`

Refresh an expired access token.

**Request Body:**
```json
{
  "refresh_token": "eyJhbGciOiJIUzI1NiIs..."
}
```

### Logout

**POST** `/auth/logout`

Invalidate current session.

**Request Body:**
```json
{
  "refresh_token": "eyJhbGciOiJIUzI1NiIs..."
}
```

### Verify Token

**GET** `/auth/verify`

Verify if a token is valid.

**Headers:**
```http
Authorization: Bearer <token>
```

**Response:**
```json
{
  "valid": true,
  "user_id": "usr_abc123",
  "expires_at": "2024-01-15T15:00:00Z"
}
```

## Roles and Permissions

### List Roles

**GET** `/roles`

Get all available roles.

**Response:**
```json
{
  "roles": [
    {
      "role_id": "admin",
      "name": "Administrator",
      "permissions": ["users.manage", "bots.manage", "system.configure"]
    },
    {
      "role_id": "user",
      "name": "User",
      "permissions": ["bots.use", "profile.edit"]
    }
  ]
}
```

### Assign Role

**POST** `/users/{user_id}/roles`

Assign a role to a user.

**Request Body:**
```json
{
  "role_id": "admin"
}
```

### Check Permission

**GET** `/users/{user_id}/permissions/{permission}`

Check if user has a specific permission.

**Response:**
```json
{
  "user_id": "usr_abc123",
  "permission": "bots.manage",
  "granted": true,
  "source": "role:admin"
}
```

## Groups

### Create Group

**POST** `/groups`

Create a user group.

**Request Body:**
```json
{
  "name": "support_team",
  "description": "Customer support team",
  "permissions": ["tickets.manage", "kb.edit"]
}
```

### Add User to Group

**POST** `/groups/{group_id}/members`

Add a user to a group.

**Request Body:**
```json
{
  "user_id": "usr_abc123"
}
```

### Remove User from Group

**DELETE** `/groups/{group_id}/members/{user_id}`

Remove a user from a group.

## Security Settings

### Get Security Settings

**GET** `/settings/security`

Get current security configuration.

**Response:**
```json
{
  "password_policy": {
    "min_length": 12,
    "require_uppercase": true,
    "require_lowercase": true,
    "require_numbers": true,
    "require_special": true,
    "expiry_days": 90,
    "history_count": 5
  },
  "session_policy": {
    "timeout_minutes": 30,
    "max_sessions": 5,
    "remember_me_days": 30
  },
  "two_factor": {
    "enabled": false,
    "required_for_roles": ["admin"],
    "methods": ["totp", "sms"]
  },
  "lockout_policy": {
    "max_attempts": 5,
    "lockout_duration_minutes": 30,
    "reset_window_minutes": 15
  }
}
```

### Update Security Settings

**PATCH** `/settings/security`

Update security configuration.

**Request Body:**
```json
{
  "password_policy": {
    "min_length": 14,
    "expiry_days": 60
  },
  "two_factor": {
    "enabled": true
  }
}
```

## Two-Factor Authentication

### Enable 2FA

**POST** `/users/{user_id}/2fa/enable`

Enable two-factor authentication.

**Response:**
```json
{
  "secret": "JBSWY3DPEHPK3PXP",
  "qr_code": "data:image/png;base64,iVBORw0KGgoAAAA...",
  "backup_codes": [
    "12345678",
    "87654321",
    "11223344"
  ]
}
```

### Verify 2FA

**POST** `/users/{user_id}/2fa/verify`

Verify 2FA setup.

**Request Body:**
```json
{
  "code": "123456"
}
```

### Disable 2FA

**POST** `/users/{user_id}/2fa/disable`

Disable two-factor authentication.

## Audit Logs

### Get Audit Logs

**GET** `/audit/logs`

Retrieve security audit logs.

**Query Parameters:**
- `user_id` - Filter by user
- `action` - Filter by action type
- `start_date` - Start of date range
- `end_date` - End of date range
- `page` - Page number
- `limit` - Items per page

**Response:**
```json
{
  "logs": [
    {
      "log_id": "log_123",
      "timestamp": "2024-01-15T14:30:00Z",
      "user_id": "usr_abc123",
      "action": "login",
      "ip_address": "192.168.1.100",
      "user_agent": "Mozilla/5.0...",
      "success": true,
      "details": {
        "method": "password",
        "session_id": "sess_xyz"
      }
    }
  ],
  "total": 500,
  "page": 1
}
```

### Export Audit Logs

**POST** `/audit/export`

Export audit logs for compliance.

**Request Body:**
```json
{
  "format": "csv",
  "date_range": {
    "start": "2024-01-01",
    "end": "2024-01-31"
  },
  "filters": {
    "actions": ["login", "permission_change", "data_access"]
  }
}
```

## Session Management

### List Sessions

**GET** `/users/{user_id}/sessions`

List active sessions for a user.

**Response:**
```json
{
  "sessions": [
    {
      "session_id": "sess_xyz",
      "created_at": "2024-01-15T10:00:00Z",
      "last_activity": "2024-01-15T14:30:00Z",
      "ip_address": "192.168.1.100",
      "user_agent": "Mozilla/5.0...",
      "location": "New York, US"
    }
  ],
  "total": 2
}
```

### Revoke Session

**DELETE** `/sessions/{session_id}`

Revoke a specific session.

### Revoke All Sessions

**POST** `/users/{user_id}/sessions/revoke-all`

Revoke all sessions for a user.

## Password Management

### Change Password

**POST** `/users/{user_id}/password`

Change user password (handled by directory service).

**Request Body:**
```json
{
  "current_password": "old_password",
  "new_password": "new_secure_password"
}
```

### Reset Password Request

**POST** `/auth/password/reset-request`

Request password reset.

**Request Body:**
```json
{
  "email": "john@example.com"
}
```

### Reset Password

**POST** `/auth/password/reset`

Reset password with token.

**Request Body:**
```json
{
  "token": "reset_token_123",
  "new_password": "new_secure_password"
}
```

## API Keys

### Generate API Key

**POST** `/users/{user_id}/api-keys`

Generate an API key for programmatic access.

**Request Body:**
```json
{
  "name": "Integration Key",
  "permissions": ["bots.use"],
  "expires_at": "2024-12-31T23:59:59Z"
}
```

**Response:**
```json
{
  "key_id": "key_123",
  "api_key": "sk_live_abcdef123456...",
  "created_at": "2024-01-15T10:00:00Z",
  "expires_at": "2024-12-31T23:59:59Z"
}
```

### List API Keys

**GET** `/users/{user_id}/api-keys`

List user's API keys.

### Revoke API Key

**DELETE** `/api-keys/{key_id}`

Revoke an API key.

## Error Responses

### 401 Unauthorized
```json
{
  "error": "unauthorized",
  "message": "Invalid credentials"
}
```

### 403 Forbidden
```json
{
  "error": "forbidden",
  "message": "Insufficient permissions"
}
```

### 423 Locked
```json
{
  "error": "account_locked",
  "message": "Account locked due to too many failed attempts",
  "locked_until": "2024-01-15T15:00:00Z"
}
```

## Security Best Practices

1. **Use Strong Passwords**: Enforce complex password requirements
2. **Enable 2FA**: Require for administrative accounts
3. **Regular Audits**: Review audit logs regularly
4. **Session Limits**: Limit concurrent sessions
5. **API Key Rotation**: Rotate keys periodically
6. **Least Privilege**: Grant minimal necessary permissions
7. **Monitor Failed Logins**: Track and alert on suspicious activity

## Rate Limits

| Operation | Limit | Window |
|-----------|-------|--------|
| Login | 5/minute | Per IP |
| Password Reset | 3/hour | Per email |
| API Key Generation | 10/day | Per user |

## Related APIs

- [Authentication](../chapter-11/authentication.md) - Auth details
- [Audit Logs](./monitoring-api.md) - System monitoring
- [Notifications](./notifications-api.md) - Security alerts