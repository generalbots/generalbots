# Organizations API 🟡 BETA

> **Manage organization settings, branding, audit logs, and Office 365 migration.**

---

## Base URL

```
/api/organizations
```

## Authentication

All endpoints require a valid session token via `Authorization: Bearer <token>` header. Some endpoints require administrator-level permissions.

---

## Endpoints

### Get Current Organization

**`GET /api/organizations/current`**

Retrieve the current organization details.

**Response:**
```json
{
  "id": "org_abc123",
  "name": "Acme Corp",
  "slug": "acme-corp",
  "description": "Main organization",
  "created_at": "2024-01-15T10:00:00Z"
}
```

---

### Update Current Organization

**`PUT /api/organizations/current`**

Update organization settings. Supports partial updates — only provided fields are merged.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| name | string | No | Organization display name |
| description | string | No | Organization description |
| slug | string | No | URL-friendly identifier |

**Request Body:**
```json
{
  "name": "Acme Corporation",
  "description": "Updated description"
}
```

**Response:**
```json
{
  "success": true
}
```

---

### Delete Current Organization

**`DELETE /api/organizations/current`**

Delete the current organization. The default organization cannot be deleted.

**Response:**
```json
{
  "success": true,
  "message": "Organization org_abc123 deleted"
}
```

**Error Response (403):**
```json
{
  "error": "Cannot delete the default organization"
}
```

---

### Get Organization Settings

**`GET /api/organizations/current/settings`**

Retrieve organization-level settings from the configuration store.

**Response:**
```json
{
  "theme": "dark",
  "logo_url": "/images/logo.png",
  "primary_color": "#3B82F6",
  "allow_public_registration": false
}
```

---

### Get Organization Stats

**`GET /api/organizations/current/stats`**

Retrieve usage statistics for the current organization.

**Response:**
```json
{
  "users": { "used": 12, "limit": 50 },
  "bots": { "used": 3, "limit": 20 },
  "kb_documents": { "used": 45, "limit": 500 },
  "storage_mb": { "used": 128, "limit": 5120 }
}
```

---

### Update Organization Contact

**`POST /api/organizations/current/contact`**

Update the organization's contact information. Logs an audit event.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| email | string | No | Contact email address |
| phone | string | No | Contact phone number |
| address | string | No | Physical address |
| website | string | No | Organization website |

**Request Body:**
```json
{
  "email": "contact@acme.com",
  "phone": "+1-555-0100",
  "website": "https://acme.com"
}
```

**Response:**
```json
{
  "success": true
}
```

---

### Update Organization Branding

**`POST /api/organizations/current/branding`**

Update the organization's branding configuration. Logs an audit event.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| logo_url | string | No | URL to the organization logo |
| primary_color | string | No | Primary brand color (hex) |
| secondary_color | string | No | Secondary brand color (hex) |
| font_family | string | No | Custom font family |
| login_message | string | No | Custom login page message |

**Request Body:**
```json
{
  "logo_url": "/images/acme-logo.png",
  "primary_color": "#1E40AF",
  "login_message": "Welcome to Acme Corp"
}
```

**Response:**
```json
{
  "success": true
}
```

---

### Get Organization Audit Log

**`GET /api/organizations/current/audit`**

Retrieve recent audit log entries for the organization. Returns the last 50 entries in reverse chronological order.

**Response:**
```json
{
  "entries": [
    {
      "timestamp": "2024-01-15T14:30:00Z",
      "actor": "admin",
      "action": "settings_updated",
      "detail": "name,description"
    },
    {
      "timestamp": "2024-01-15T14:00:00Z",
      "actor": "admin",
      "action": "branding_updated",
      "detail": "primary_color,logo_url"
    }
  ],
  "total": 2
}
```

---

### Export Organization Data

**`GET /api/organizations/current/export`**

Export all organization data including users, bots, and settings. Returns a download URL for the generated JSON file.

**Response:**
```json
{
  "success": true,
  "message": "Export complete",
  "download_url": "/api/files/download?path=/opt/gbo/tmp/org-export-1705312800.json"
}
```

---

## Admin Endpoints

### Office 365 Migration

**`POST /api/admin/migrate/office365`**

Synchronize users and groups from an Azure AD / Office 365 tenant into the local directory. Supports both full and delta sync modes.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| tenant_id | string | Yes | Azure AD tenant ID |
| client_id | string | Yes | Azure AD application client ID |
| client_secret | string | Yes | Azure AD application client secret |
| sync_mode | string | No | `"full"` (default) or `"delta"` |

**Request Body:**
```json
{
  "tenant_id": "abc123-def456",
  "client_id": "app-789-ghi",
  "client_secret": "secret...",
  "sync_mode": "full"
}
```

**Response:**
```json
{
  "success": true,
  "groups_created": 5,
  "groups_updated": 2,
  "users_mapped": 42,
  "users_created": 10,
  "users_updated": 8,
  "errors": [],
  "duration_ms": 3450
}
```

---

## Examples

### Update Organization Name

```bash
curl -X PUT http://localhost:8080/api/organizations/current \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name": "Acme Corporation"}'
```

### Get Usage Stats

```bash
curl -X GET http://localhost:8080/api/organizations/current/stats \
  -H "Authorization: Bearer $TOKEN"
```

### Export All Data

```bash
curl -X GET http://localhost:8080/api/organizations/current/export \
  -H "Authorization: Bearer $TOKEN"
```

### Office 365 Full Sync

```bash
curl -X POST http://localhost:8080/api/admin/migrate/office365 \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "tenant_id": "my-tenant-id",
    "client_id": "my-client-id",
    "client_secret": "my-secret",
    "sync_mode": "full"
  }'
```

---

## Response Codes

| Code | Description |
|------|-------------|
| 200 | Success |
| 201 | Created |
| 400 | Bad Request (invalid parameters) |
| 401 | Unauthorized |
| 403 | Forbidden (e.g., cannot delete default org) |
| 404 | Not Found |
| 500 | Internal Server Error |

---

## See Also

- [Admin API](./admin-api.md) — System administration endpoints
- [SCIM API](./scim-api.md) — SCIM 2.0 user and group provisioning
- [Security API](./security-api.md) — Access control and audit
