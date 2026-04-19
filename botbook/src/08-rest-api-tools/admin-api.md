# Admin API

The Admin API provides endpoints for system administration, user management, and configuration management.

## Status: Roadmap

This API is on the development roadmap. The endpoints documented below represent the planned interface design.

## Base URL

```
http://localhost:9000/api/v1/admin
```

## Authentication

Uses the standard botserver authentication mechanism with administrator-level permissions required.

## Endpoints

### System Configuration

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/admin/config` | Retrieve system configuration |
| PUT | `/api/v1/admin/config` | Update system configuration |

### User Management

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/admin/users` | List all users |
| POST | `/api/v1/admin/users` | Create a new user |
| GET | `/api/v1/admin/users/{user_id}` | Get user details |
| PUT | `/api/v1/admin/users/{user_id}` | Update user |
| DELETE | `/api/v1/admin/users/{user_id}` | Delete user |

### Bot Management

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/admin/bots` | List all bots |
| GET | `/api/v1/admin/bots/{bot_id}` | Get bot details |
| POST | `/api/v1/admin/bots/{bot_id}/restart` | Restart a bot |
| DELETE | `/api/v1/admin/bots/{bot_id}` | Delete a bot |

### System Health

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/admin/health` | System health check |
| GET | `/api/v1/admin/metrics` | System metrics |

### Audit Logs

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/admin/audit` | Retrieve audit logs |
| GET | `/api/v1/admin/audit/{event_id}` | Get specific audit event |

## Request Examples

### Get System Configuration

```bas
config = GET "/api/v1/admin/config"
TALK "Server port: " + config.server_port
```

### Create User

```bas
user_data = NEW OBJECT
user_data.email = "admin@example.com"
user_data.role = "administrator"

result = POST "/api/v1/admin/users", user_data
TALK "Created user: " + result.id
```

### Restart Bot

```bas
POST "/api/v1/admin/bots/my-bot/restart", {}
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