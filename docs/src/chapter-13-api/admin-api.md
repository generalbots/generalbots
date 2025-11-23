# Admin API

The Admin API provides system administration endpoints for monitoring, configuration, backup, and user management.

## Base URL

All admin endpoints are prefixed with `/admin` and require administrator authentication.

## System Status

### Get System Status

**GET** `/admin/system/status`

Returns overall system health and component status.

**Response:**
```json
{
  "status": "healthy",
  "components": {
    "database": "connected",
    "cache": "connected", 
    "storage": "connected",
    "vector_db": "connected"
  },
  "uptime_seconds": 86400,
  "version": "6.0.8",
  "last_check": "2024-01-20T10:30:00Z"
}
```

### Get System Metrics

**GET** `/admin/system/metrics`

Returns system performance metrics.

**Response:**
```json
{
  "cpu": {
    "usage_percent": 45.2,
    "cores": 8
  },
  "memory": {
    "used_mb": 2048,
    "total_mb": 8192,
    "percent": 25.0
  },
  "disk": {
    "used_gb": 50,
    "total_gb": 500,
    "percent": 10.0
  },
  "network": {
    "bytes_sent": 1073741824,
    "bytes_received": 2147483648
  },
  "timestamp": "2024-01-20T10:30:00Z"
}
```

## Logs Management

### View Logs

**GET** `/admin/logs/view`

Retrieves system logs with optional filtering.

**Query Parameters:**
- `level` - Log level filter (debug, info, warn, error)
- `start_time` - Start timestamp (ISO 8601)
- `end_time` - End timestamp (ISO 8601)
- `service` - Service name filter
- `limit` - Maximum entries (default: 1000)
- `offset` - Pagination offset

**Response:**
```json
{
  "entries": [
    {
      "timestamp": "2024-01-20T10:30:00Z",
      "level": "info",
      "service": "web_server",
      "message": "Request processed successfully",
      "metadata": {}
    }
  ],
  "total": 5000,
  "filtered": 100
}
```

### Export Logs

**POST** `/admin/logs/export`

Exports system logs to a downloadable file.

**Request:**
```json
{
  "format": "json",
  "start_time": "2024-01-20T00:00:00Z",
  "end_time": "2024-01-20T23:59:59Z",
  "include_metadata": true
}
```

**Response:**
```json
{
  "export_id": "550e8400-e29b-41d4-a716-446655440000",
  "file_size": 1048576,
  "download_url": "/admin/logs/export/550e8400-e29b-41d4-a716-446655440000",
  "expires_at": "2024-01-21T10:30:00Z"
}
```

## Configuration

### Get Configuration

**GET** `/admin/config`

Returns current system configuration (sensitive values redacted).

**Response:**
```json
{
  "server": {
    "host": "127.0.0.1",
    "port": 8080,
    "workers": 4
  },
  "database": {
    "host": "localhost",
    "port": 5432,
    "pool_size": 10
  },
  "cache": {
    "host": "localhost",
    "port": 6379
  },
  "storage": {
    "endpoint": "http://localhost:9000",
    "bucket": "botserver"
  }
}
```

### Update Configuration

**PUT** `/admin/config/update`

Updates system configuration settings.

**Request:**
```json
{
  "section": "server",
  "key": "workers",
  "value": "8"
}
```

**Response:**
```json
{
  "success": true,
  "message": "Configuration updated",
  "restart_required": true
}
```

## Maintenance

### Schedule Maintenance

**POST** `/admin/maintenance/schedule`

Schedules a maintenance window.

**Request:**
```json
{
  "start_time": "2024-01-21T02:00:00Z",
  "duration_minutes": 60,
  "description": "System update and database maintenance",
  "notify_users": true
}
```

**Response:**
```json
{
  "maintenance_id": "maint-123",
  "scheduled": true,
  "notifications_sent": 150
}
```

## Backup Management

### Create Backup

**POST** `/admin/backup/create`

Creates a system backup.

**Request:**
```json
{
  "backup_type": "full",
  "include_data": true,
  "include_config": true,
  "compression": "gzip"
}
```

**Response:**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "backup_type": "full",
  "size_bytes": 524288000,
  "created_at": "2024-01-20T10:30:00Z",
  "status": "completed",
  "download_url": "/admin/backups/550e8400-e29b-41d4-a716-446655440000/download",
  "expires_at": "2024-02-19T10:30:00Z"
}
```

### Restore Backup

**POST** `/admin/backup/restore`

Restores system from a backup.

**Request:**
```json
{
  "backup_id": "550e8400-e29b-41d4-a716-446655440000",
  "verify_only": false,
  "restore_options": {
    "data": true,
    "config": false
  }
}
```

**Response:**
```json
{
  "success": true,
  "items_restored": 1500,
  "warnings": [],
  "restart_required": true
}
```

### List Backups

**GET** `/admin/backups`

Lists available backups.

**Response:**
```json
{
  "backups": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "backup_type": "full",
      "size_bytes": 524288000,
      "created_at": "2024-01-20T10:30:00Z",
      "status": "completed",
      "download_url": "/admin/backups/550e8400-e29b-41d4-a716-446655440000/download",
      "expires_at": "2024-02-19T10:30:00Z"
    },
    {
      "id": "660e8400-e29b-41d4-a716-446655440001",
      "backup_type": "incremental",
      "size_bytes": 52428800,
      "created_at": "2024-01-19T22:30:00Z",
      "status": "completed",
      "download_url": "/admin/backups/660e8400-e29b-41d4-a716-446655440001/download",
      "expires_at": "2024-02-18T22:30:00Z"
    }
  ]
}
```

## User Management

### Manage Users

**POST** `/admin/users/manage`

Performs user management operations.

**Request:**
```json
{
  "action": "create",
  "user_data": {
    "username": "newuser",
    "email": "user@example.com",
    "role": "user",
    "active": true
  }
}
```

**Response:**
```json
{
  "success": true,
  "user_id": "user-123",
  "message": "User created successfully"
}
```

## Roles and Permissions

### Get Roles

**GET** `/admin/roles`

Returns all defined roles.

**Response:**
```json
{
  "roles": [
    {
      "id": "admin",
      "name": "Administrator",
      "permissions": ["all"],
      "user_count": 2
    },
    {
      "id": "user",
      "name": "Standard User",
      "permissions": ["read", "write"],
      "user_count": 150
    }
  ]
}
```

### Manage Roles

**POST** `/admin/roles/manage`

Creates or updates roles.

**Request:**
```json
{
  "action": "create",
  "role": {
    "id": "moderator",
    "name": "Moderator",
    "permissions": ["read", "write", "moderate"]
  }
}
```

**Response:**
```json
{
  "success": true,
  "role_id": "moderator",
  "message": "Role created successfully"
}
```

## Quotas

### Get Quotas

**GET** `/admin/quotas`

Returns system quotas and usage.

**Response:**
```json
{
  "quotas": {
    "storage": {
      "limit_gb": 1000,
      "used_gb": 250,
      "percent": 25
    },
    "users": {
      "limit": 1000,
      "current": 152,
      "percent": 15.2
    },
    "api_calls": {
      "daily_limit": 1000000,
      "today_count": 125000,
      "percent": 12.5
    }
  }
}
```

### Manage Quotas

**POST** `/admin/quotas/manage`

Updates quota limits.

**Request:**
```json
{
  "quota_type": "storage",
  "new_limit": 2000,
  "unit": "GB"
}
```

**Response:**
```json
{
  "success": true,
  "quota_type": "storage",
  "previous_limit": 1000,
  "new_limit": 2000,
  "unit": "GB"
}
```

## Licenses

### Get Licenses

**GET** `/admin/licenses`

Returns license information.

**Response:**
```json
{
  "licenses": [
    {
      "id": "license-001",
      "type": "enterprise",
      "status": "active",
      "expires_at": "2025-01-20T00:00:00Z",
      "features": ["unlimited_users", "priority_support"],
      "usage": {
        "users": 152,
        "bots": 5
      }
    }
  ]
}
```

### Manage Licenses

**POST** `/admin/licenses/manage`

Activates or updates licenses.

**Request:**
```json
{
  "action": "activate",
  "license_key": "XXXX-XXXX-XXXX-XXXX"
}
```

**Response:**
```json
{
  "success": true,
  "license_id": "license-002",
  "type": "enterprise",
  "expires_at": "2026-01-20T00:00:00Z",
  "features": ["unlimited_users", "priority_support"]
}
```

## Error Responses

All endpoints may return error responses:

```json
{
  "error": {
    "code": "UNAUTHORIZED",
    "message": "Admin privileges required",
    "details": {}
  }
}
```

Common error codes:
- `UNAUTHORIZED` - Missing or invalid admin credentials
- `FORBIDDEN` - Insufficient permissions
- `NOT_FOUND` - Resource not found
- `BAD_REQUEST` - Invalid request parameters
- `INTERNAL_ERROR` - Server error

## Authentication

Admin API endpoints require administrator authentication via:
- Bearer token in Authorization header
- Session cookie with admin privileges

## Rate Limiting

Admin endpoints are rate-limited to prevent abuse:
- 100 requests per minute for read operations
- 10 requests per minute for write operations
- 5 requests per minute for backup/restore operations