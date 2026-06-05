# Admin API (Full)

> **System administration — configuration, organization management, user lifecycle, groups, permissions, and role management.**

---

## Base URL

```
/api/admin
```

## Authentication

All endpoints require a valid session token via `Authorization: Bearer <token>` header. Most endpoints require `admin` role.

---

## Endpoints

### System Configuration

**`GET /api/admin/config`**

Retrieves the full system configuration.

**Response:**

```json
{
  "server": {
    "host": "0.0.0.0",
    "port": 8080,
    "log_level": "info",
    "max_connections": 1000
  },
  "database": {
    "host": "tables.local",
    "port": 5432,
    "name": "botserver",
    "pool_size": 20
  },
  "cache": {
    "host": "cache.local",
    "port": 6379
  },
  "llm": {
    "default_provider": "groq",
    "default_model": "llama-3.3-70b-versatile",
    "timeout_seconds": 60
  },
  "features": {
    "whatsapp": true,
    "voice": false,
    "analytics": true,
    "compliance": true
  }
}
```

---

**`POST /api/admin/config`**

Updates system configuration. Partial updates are supported.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `server` | object | No | Server settings |
| `llm` | object | No | LLM defaults |
| `features` | object | No | Feature flags |

**Request Body:**

```json
{
  "llm": {
    "default_model": "gpt-4o",
    "timeout_seconds": 120
  },
  "features": {
    "voice": true
  }
}
```

**Response:**

```json
{
  "success": true,
  "updated_at": "2025-06-04T12:00:00Z",
  "changes": ["llm.default_model", "llm.timeout_seconds", "features.voice"]
}
```

---

### Organization Management

**`GET /api/admin/organizations/list`**

Lists all organizations.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `page` | integer | No | Page number (default: 1) |
| `limit` | integer | No | Items per page (default: 20) |

**Response:**

```json
{
  "organizations": [
    {
      "org_id": "org_001",
      "name": "Acme Corp",
      "plan": "enterprise",
      "user_count": 25,
      "bot_count": 5,
      "created_at": "2025-01-15T10:00:00Z",
      "status": "active"
    }
  ],
  "total": 12
}
```

---

**`POST /api/admin/organizations/create`**

Creates a new organization.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | Yes | Organization name |
| `plan` | string | No | `free`, `pro`, `enterprise` (default: `free`) |
| `admin_email` | string | Yes | Initial admin email |
| `settings` | object | No | Org-specific settings |

**Request Body:**

```json
{
  "name": "TechStart Ltda",
  "plan": "pro",
  "admin_email": "admin@techstart.com.br",
  "settings": {
    "max_bots": 10,
    "max_users": 50,
    "retention_days": 90
  }
}
```

**Response:**

```json
{
  "org_id": "org_002",
  "name": "TechStart Ltda",
  "plan": "pro",
  "status": "active",
  "admin_email": "admin@techstart.com.br",
  "created_at": "2025-06-04T12:00:00Z"
}
```

---

**`GET /api/admin/organizations/:org_id`**

Retrieves full details for an organization.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `org_id` | string | Yes | Organization ID |

**Response:**

```json
{
  "org_id": "org_001",
  "name": "Acme Corp",
  "plan": "enterprise",
  "status": "active",
  "user_count": 25,
  "bot_count": 5,
  "storage_used_mb": 2048,
  "storage_limit_mb": 51200,
  "settings": {
    "max_bots": 100,
    "max_users": 500,
    "retention_days": 365
  },
  "created_at": "2025-01-15T10:00:00Z",
  "updated_at": "2025-06-01T08:00:00Z"
}
```

---

### User Management

**`POST /api/admin/users/create`**

Creates a new user account.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `email` | string | Yes | User email |
| `name` | string | Yes | Full name |
| `password` | string | Yes | Initial password (min 8 chars) |
| `org_id` | string | No | Organization to assign |
| `roles` | array | No | Initial roles (default: `["viewer"]`) |

**Request Body:**

```json
{
  "email": "dev@techstart.com.br",
  "name": "Maria Silva",
  "password": "T3mpP@ss!",
  "org_id": "org_002",
  "roles": ["viewer"]
}
```

**Response:**

```json
{
  "user_id": "usr_002",
  "email": "dev@techstart.com.br",
  "name": "Maria Silva",
  "org_id": "org_002",
  "roles": ["viewer"],
  "created_at": "2025-06-04T12:00:00Z",
  "status": "active"
}
```

---

**`PUT /api/admin/users/:user_id/update`**

Updates an existing user's details.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `user_id` | string | Yes | User ID |
| `name` | string | No | Updated name |
| `email` | string | No | Updated email |
| `org_id` | string | No | Reassign organization |
| `roles` | array | No | Replace roles |
| `status` | string | No | `active`, `suspended`, `inactive` |

**Request Body:**

```json
{
  "name": "Maria Santos",
  "roles": ["viewer", "editor"]
}
```

**Response:**

```json
{
  "user_id": "usr_002",
  "name": "Maria Santos",
  "roles": ["viewer", "editor"],
  "updated_at": "2025-06-04T13:00:00Z"
}
```

---

**`DELETE /api/admin/users/:user_id/delete`**

Deletes a user account (soft delete — data retained for compliance).

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `user_id` | string | Yes | User ID |
| `confirm` | boolean | Yes | Must be `true` to proceed |
| `reason` | string | No | Deletion reason |

**Request Body:**

```json
{
  "confirm": true,
  "reason": "User left the organization"
}
```

**Response:**

```json
{
  "user_id": "usr_002",
  "status": "deleted",
  "deleted_at": "2025-06-04T14:00:00Z",
  "message": "User soft-deleted. Data retained for 90 days."
}
```

---

**`GET /api/admin/users/list`**

Lists all users with optional filters.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `org_id` | string | No | Filter by organization |
| `status` | string | No | `active`, `suspended`, `inactive` |
| `role` | string | No | Filter by role |
| `page` | integer | No | Page number |
| `limit` | integer | No | Items per page |

**Response:**

```json
{
  "users": [
    {
      "user_id": "usr_001",
      "email": "admin@acme.com",
      "name": "João Souza",
      "org_id": "org_001",
      "roles": ["admin"],
      "status": "active",
      "last_login": "2025-06-04T08:00:00Z",
      "created_at": "2025-01-15T10:00:00Z"
    },
    {
      "user_id": "usr_003",
      "email": "dev@techstart.com.br",
      "name": "Carlos Lima",
      "org_id": "org_002",
      "roles": ["viewer", "editor"],
      "status": "active",
      "last_login": "2025-06-03T16:00:00Z",
      "created_at": "2025-03-10T09:00:00Z"
    }
  ],
  "total": 47
}
```

---

**`GET /api/admin/users/search`**

Searches users by name, email, or ID.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `q` | string | Yes | Search query |
| `limit` | integer | No | Max results (default: 20) |

**Response:**

```json
{
  "query": "maria",
  "results": [
    {
      "user_id": "usr_002",
      "email": "dev@techstart.com.br",
      "name": "Maria Santos",
      "org_id": "org_002",
      "roles": ["viewer", "editor"],
      "score": 0.95
    }
  ],
  "total": 1
}
```

---

### User Profile & Status

**`GET /api/admin/users/:user_id/profile`**

Retrieves the full profile for a user.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `user_id` | string | Yes | User ID |

**Response:**

```json
{
  "user_id": "usr_001",
  "email": "admin@acme.com",
  "name": "João Souza",
  "org_id": "org_001",
  "org_name": "Acme Corp",
  "avatar_url": "/avatars/usr_001.jpg",
  "timezone": "America/Sao_Paulo",
  "language": "pt-br",
  "created_at": "2025-01-15T10:00:00Z",
  "last_login": "2025-06-04T08:00:00Z",
  "login_count": 145,
  "mfa_enabled": true
}
```

---

**`GET /api/admin/users/:user_id/permissions`**

Returns all permissions assigned to a user (direct + inherited from roles).

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `user_id` | string | Yes | User ID |

**Response:**

```json
{
  "user_id": "usr_001",
  "permissions": [
    "bots.create",
    "bots.read",
    "bots.update",
    "bots.delete",
    "users.create",
    "users.read",
    "users.update",
    "users.delete",
    "config.read",
    "config.update",
    "billing.read",
    "billing.manage",
    "compliance.read",
    "compliance.manage",
    "admin.full"
  ],
  "sources": {
    "direct": [],
    "roles": ["admin"],
    "inherited": ["admin.full"]
  }
}
```

---

**`GET /api/admin/users/:user_id/roles`**

Returns roles assigned to a user.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `user_id` | string | Yes | User ID |

**Response:**

```json
{
  "user_id": "usr_001",
  "roles": [
    {
      "role": "admin",
      "assigned_at": "2025-01-15T10:00:00Z",
      "assigned_by": "system"
    },
    {
      "role": "compliance_admin",
      "assigned_at": "2025-03-01T14:00:00Z",
      "assigned_by": "usr_001"
    }
  ]
}
```

---

**`GET /api/admin/users/:user_id/status`**

Returns account status and health information.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `user_id` | string | Yes | User ID |

**Response:**

```json
{
  "user_id": "usr_001",
  "status": "active",
  "email_verified": true,
  "mfa_enabled": true,
  "password_last_changed": "2025-04-01T10:00:00Z",
  "failed_login_attempts": 0,
  "locked_until": null,
  "api_keys_count": 2,
  "active_sessions": 3
}
```

---

**`GET /api/admin/users/:user_id/presence`**

Returns real-time presence information.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `user_id` | string | Yes | User ID |

**Response:**

```json
{
  "user_id": "usr_001",
  "online": true,
  "last_seen": "2025-06-04T12:00:00Z",
  "current_bot": "default",
  "active_session": "sess_xyz789",
  "ip_address": "192.168.1.50"
}
```

---

**`GET /api/admin/users/:user_id/activity`**

Returns activity history for a user.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `user_id` | string | Yes | User ID |
| `limit` | integer | No | Max entries (default: 20) |
| `type` | string | No | Filter: `login`, `message`, `tool_call`, `config_change` |

**Response:**

```json
{
  "user_id": "usr_001",
  "activities": [
    {
      "id": "act_001",
      "type": "login",
      "timestamp": "2025-06-04T08:00:00Z",
      "details": "Login from 192.168.1.50",
      "ip_address": "192.168.1.50"
    },
    {
      "id": "act_002",
      "type": "config_change",
      "timestamp": "2025-06-04T08:15:00Z",
      "details": "Updated bot 'default' LLM model",
      "resource": "bot:default"
    },
    {
      "id": "act_003",
      "type": "message",
      "timestamp": "2025-06-04T09:00:00Z",
      "details": "Sent message in bot 'sales'",
      "resource": "session:sess_abc123"
    }
  ],
  "total": 234
}
```

---

### Group Management

**`POST /api/admin/groups/create`**

Creates a new user group.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | Yes | Group name |
| `description` | string | No | Group description |
| `org_id` | string | Yes | Organization ID |
| `roles` | array | No | Roles assigned to group members |
| `members` | array | No | Initial member user IDs |

**Request Body:**

```json
{
  "name": "DevOps Team",
  "description": "Infrastructure and deployment management",
  "org_id": "org_001",
  "roles": ["editor", "deployer"],
  "members": ["usr_001", "usr_004"]
}
```

**Response:**

```json
{
  "group_id": "grp_001",
  "name": "DevOps Team",
  "org_id": "org_001",
  "roles": ["editor", "deployer"],
  "member_count": 2,
  "created_at": "2025-06-04T12:00:00Z"
}
```

---

## Roles Reference

| Role | Description | Permissions |
|------|-------------|-------------|
| `admin` | Full system access | All permissions |
| `org_admin` | Organization administrator | Manage org users, bots, config |
| `editor` | Can edit bots and content | bots.*, tools.*, documents.* |
| `viewer` | Read-only access | bots.read, users.read |
| `deployer` | Can deploy and restart | deploy.*, bots.restart |
| `compliance_admin` | Compliance management | compliance.*, audit.* |
| `billing_admin` | Billing management | billing.* |
| `bot_manager` | Bot lifecycle management | bots.* |

---

## Response Codes

| Code | Description |
|------|-------------|
| 200 | Success |
| 201 | Created |
| 204 | No Content (deletion) |
| 400 | Bad Request |
| 401 | Unauthorized |
| 403 | Forbidden (insufficient permissions) |
| 404 | Resource not found |
| 409 | Conflict (duplicate email, etc.) |
| 500 | Internal Server Error |

---

## See Also

- [User Security](./user-security.md) — authentication, MFA, API keys
- [Compliance API](./compliance-api.md) — ISO 27001 compliance
- [Monitoring API](./monitoring-api.md) — system health and metrics
- [Billing API](./billing-api.md) — invoice and payment management
