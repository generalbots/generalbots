# Security API 🟡 BETA

The Security API provides endpoints for security management, access control, and threat monitoring.

## Status: Roadmap

This API is on the development roadmap. The endpoints documented below represent the planned interface design.

## Base URL

```
http://localhost:8080/api/security
```

## Authentication

Uses the standard botserver authentication mechanism with elevated security permissions required.

## Endpoints

### Authentication

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/security/auth/login` | Authenticate user |
| POST | `/api/security/auth/logout` | End session |
| POST | `/api/security/auth/refresh` | Refresh access token |
| POST | `/api/security/auth/mfa/setup` | Setup MFA |
| POST | `/api/security/auth/mfa/verify` | Verify MFA code |

### API Keys

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/security/keys/generate` | Generate new API key |
| GET | `/api/security/keys` | List API keys |
| GET | `/api/security/keys/{key_id}` | Get key details |
| PUT | `/api/security/keys/{key_id}` | Update key permissions |
| DELETE | `/api/security/keys/{key_id}` | Revoke API key |

### Access Control

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/security/roles` | List all roles |
| POST | `/api/security/roles` | Create a role |
| GET | `/api/security/roles/{role_id}` | Get role details |
| PUT | `/api/security/roles/{role_id}` | Update role |
| DELETE | `/api/security/roles/{role_id}` | Delete role |
| GET | `/api/security/permissions` | List all permissions |
| PUT | `/api/security/permissions` | Update permissions |

### Audit Logs

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/security/audit` | List audit log entries |
| GET | `/api/security/audit/{id}` | Get specific audit entry |
| POST | `/api/security/audit/export` | Export audit logs |
| GET | `/api/security/audit/summary` | Get audit summary |

### Session Management

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/security/sessions` | List active sessions |
| GET | `/api/security/sessions/{session_id}` | Get session details |
| DELETE | `/api/security/sessions/{session_id}` | Terminate session |
| DELETE | `/api/security/sessions/user/{user_id}` | Terminate all user sessions |

### Security Monitoring

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/security/threats` | List detected threats |
| GET | `/api/security/vulnerabilities` | List vulnerabilities |
| POST | `/api/security/scan` | Initiate security scan |
| GET | `/api/security/scan/{scan_id}` | Get scan results |

## Request Examples

### Login

```bas
credentials = NEW OBJECT
credentials.email = "admin@example.com"
credentials.password = "secure_password"

result = POST "/api/security/auth/login", credentials
token = result.access_token
TALK "Logged in successfully"
```

### Generate API Key

```bas
key_config = NEW OBJECT
key_config.name = "Integration Key"
key_config.scopes = ["read:bots", "write:messages"]
key_config.expires_in_days = 90

result = POST "/api/security/keys/generate", key_config
TALK "API Key: " + result.key
TALK "Expires: " + result.expires_at
```

### Create Role

```bas
role = NEW OBJECT
role.name = "bot_manager"
role.description = "Can manage bots and configurations"
role.permissions = ["bot:read", "bot:write", "bot:delete", "config:read", "config:write"]

result = POST "/api/security/roles", role
TALK "Role created: " + result.id
```

### List Active Sessions

```bas
sessions = GET "/api/security/sessions"
FOR EACH session IN sessions
    TALK session.user_email + " - " + session.ip_address + " (" + session.last_activity + ")"
NEXT
```

### Query Audit Logs

```bas
audit = GET "/api/security/audit?action=login&days=7"
FOR EACH entry IN audit
    TALK entry.timestamp + " | " + entry.user + " | " + entry.action + " | " + entry.result
NEXT
```

### Terminate Session

```bas
DELETE "/api/security/sessions/session-123"
TALK "Session terminated"
```

## Response Codes

| Code | Description |
|------|-------------|
| 200 | Success |
| 201 | Created |
| 204 | No Content (successful deletion) |
| 400 | Bad Request |
| 401 | Unauthorized (invalid credentials) |
| 403 | Forbidden (insufficient permissions) |
| 404 | Not Found |
| 429 | Too Many Requests (rate limited) |
| 500 | Internal Server Error |

## Security Event Types

| Event Type | Description |
|------------|-------------|
| `login_success` | Successful authentication |
| `login_failed` | Failed authentication attempt |
| `logout` | User logged out |
| `password_changed` | Password was changed |
| `mfa_enabled` | MFA was enabled |
| `api_key_created` | New API key generated |
| `api_key_revoked` | API key was revoked |
| `role_changed` | User role was modified |
| `permission_denied` | Access denied to resource |
| `suspicious_activity` | Potential security threat detected |

## Rate Limiting

| Endpoint | Limit |
|----------|-------|
| `/auth/login` | 5 requests per minute |
| `/auth/mfa/verify` | 3 requests per minute |
| `/keys/generate` | 10 requests per hour |
| Other endpoints | 100 requests per minute |

## Required Permissions

| Endpoint Category | Required Role |
|-------------------|---------------|
| Authentication | Public (login) / Authenticated (others) |
| API Keys | `admin` or `key_manager` |
| Access Control | `admin` |
| Audit Logs | `admin` or `auditor` |
| Session Management | `admin` or `session_manager` |
| Security Monitoring | `admin` or `security_analyst` |

## Best Practices

1. **Use strong passwords** - Minimum 12 characters with mixed case, numbers, and symbols
2. **Enable MFA** - Two-factor authentication for all admin accounts
3. **Rotate API keys** - Set expiration dates and rotate keys regularly
4. **Monitor audit logs** - Review security events daily
5. **Principle of least privilege** - Grant minimum necessary permissions
6. **Terminate inactive sessions** - Auto-expire sessions after inactivity