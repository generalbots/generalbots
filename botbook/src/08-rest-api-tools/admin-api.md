# Admin API 🟡 BETA

The Admin API provides endpoints for system administration, user management, and configuration management.

## Status: Roadmap

This API is on the development roadmap. The endpoints documented below represent the planned interface design.

## Base URL

```
http://localhost:8080/api/admin
```

## Authentication

Uses the standard botserver authentication mechanism with administrator-level permissions required.

## Endpoints

### System Configuration

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/admin/config` | Retrieve system configuration |
| PUT | `/api/admin/config` | Update system configuration |

### User Management

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/admin/users` | List all users |
| POST | `/api/admin/users` | Create a new user |
| GET | `/api/admin/users/{user_id}` | Get user details |
| PUT | `/api/admin/users/{user_id}` | Update user |
| DELETE | `/api/admin/users/{user_id}` | Delete user |

### Bot Management

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/admin/bots` | List all bots |
| GET | `/api/admin/bots/{bot_id}` | Get bot details |
| POST | `/api/admin/bots/{bot_id}/restart` | Restart a bot |
| DELETE | `/api/admin/bots/{bot_id}` | Delete a bot |

### System Health

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/admin/health` | System health check |
| GET | `/api/admin/metrics` | System metrics |

### Audit Logs

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/admin/audit` | Retrieve audit logs |
| GET | `/api/admin/audit/{event_id}` | Get specific audit event |

## Request Examples

### Get System Configuration

```bas
config = GET "/api/admin/config"
TALK "Server port: " + config.server_port
```

### Create User

```bas
user_data = NEW OBJECT
user_data.email = "admin@example.com"
user_data.role = "administrator"

result = POST "/api/admin/users", user_data
TALK "Created user: " + result.id
```

### Restart Bot

```bas
POST "/api/admin/bots/my-bot/restart", {}
TALK "Bot restart initiated"
```

## Response Codes

| Code | Description |
|------|-------------|
| 200 | Success |
| 201 | Created |
| 204 | No Content (successful deletion) |
| 400 | Bad Request |
| 401 | Unauthorized |
| 403 | Forbidden (insufficient permissions) |
| 404 | Not Found |
| 500 | Internal Server Error |

## Required Permissions

| Endpoint Category | Required Role |
|-------------------|---------------|
| System Configuration | `admin` |
| User Management | `admin` |
| Bot Management | `admin` or `bot_manager` |
| System Health | `admin` or `monitor` |
| Audit Logs | `admin` or `auditor` |