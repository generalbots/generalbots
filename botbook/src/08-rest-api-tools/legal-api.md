# Legal / LGPD API

> **LGPD (Lei Geral de Proteção de Dados) compliance — consent management, cookie policy, legal document hosting, and GDPR data subject rights.**

---

## Base URL

```
/api/legal
```

## Authentication

All endpoints require a valid session token via `Authorization: Bearer <token>` header. GDPR deletion/export endpoints require `admin` or `data_officer` role.

---

## Endpoints

### Consent Management

**`POST /api/legal/consent`**

Records a user's consent for a specific purpose.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `user_id` | string | Yes | User ID |
| `purpose` | string | Yes | Consent purpose: `analytics`, `marketing`, `third_party_sharing`, `profiling` |
| `granted` | boolean | Yes | Whether consent is granted |
| `consent_version` | string | No | Version of consent text shown |
| `ip_address` | string | No | User's IP (auto-detected if omitted) |
| `user_agent` | string | No | Browser user agent (auto-detected if omitted) |

**Request Body:**

```json
{
  "user_id": "usr_001",
  "purpose": "marketing",
  "granted": true,
  "consent_version": "2025-v2"
}
```

**Response:**

```json
{
  "consent_id": "cns_abc123",
  "user_id": "usr_001",
  "purpose": "marketing",
  "granted": true,
  "consent_version": "2025-v2",
  "recorded_at": "2025-06-04T10:00:00Z",
  "ip_address": "192.168.1.50"
}
```

---

**`GET /api/legal/consent/:consent_id`**

Retrieves details of a specific consent record.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `consent_id` | string | Yes | Consent record ID |

**Response:**

```json
{
  "consent_id": "cns_abc123",
  "user_id": "usr_001",
  "purpose": "marketing",
  "granted": true,
  "consent_version": "2025-v2",
  "recorded_at": "2025-06-04T10:00:00Z",
  "ip_address": "192.168.1.50",
  "user_agent": "Mozilla/5.0 ...",
  "revoked_at": null
}
```

---

**`PUT /api/legal/consent/:consent_id`**

Updates (revokes or modifies) an existing consent record.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `consent_id` | string | Yes | Consent record ID |
| `granted` | boolean | Yes | New consent state |
| `revocation_reason` | string | No | Reason if revoking |

**Request Body:**

```json
{
  "granted": false,
  "revocation_reason": "User no longer wishes to receive marketing"
}
```

**Response:**

```json
{
  "consent_id": "cns_abc123",
  "granted": false,
  "revoked_at": "2025-06-04T12:00:00Z",
  "revocation_reason": "User no longer wishes to receive marketing"
}
```

---

### Get Consent by Session

**`GET /api/legal/consent/session`**

Returns all consent records for the current session's user.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `purpose` | string | No | Filter by purpose |

**Response:**

```json
{
  "user_id": "usr_001",
  "consents": [
    {
      "consent_id": "cns_abc123",
      "purpose": "analytics",
      "granted": true,
      "consent_version": "2025-v2",
      "recorded_at": "2025-06-01T08:00:00Z"
    },
    {
      "consent_id": "cns_def456",
      "purpose": "marketing",
      "granted": false,
      "consent_version": "2025-v2",
      "recorded_at": "2025-06-03T14:00:00Z",
      "revoked_at": "2025-06-04T10:00:00Z"
    }
  ]
}
```

---

### Cookie Policy

**`GET /api/legal/cookies/policy`**

Returns the cookie policy configuration for the site.

**Response:**

```json
{
  "policy_version": "2.1",
  "last_updated": "2025-05-15T00:00:00Z",
  "categories": [
    {
      "name": "Strictly Necessary",
      "required": true,
      "description": "Essential cookies for site functionality",
      "cookies": [
        {
          "name": "session_id",
          "purpose": "User session management",
          "duration": "Session",
          "type": "First-party"
        },
        {
          "name": "csrf_token",
          "purpose": "Cross-site request forgery protection",
          "duration": "Session",
          "type": "First-party"
        }
      ]
    },
    {
      "name": "Analytics",
      "required": false,
      "description": "Usage analytics and performance monitoring",
      "cookies": [
        {
          "name": "_ga",
          "purpose": "Google Analytics visitor tracking",
          "duration": "2 years",
          "type": "Third-party"
        }
      ]
    }
  ],
  "consent_required": ["Analytics", "Marketing", "Third-party Sharing"],
  "privacy_policy_url": "/legal/privacy",
  "contact_email": "dpo@company.com"
}
```

---

### Legal Documents

**`GET /api/legal/documents`**

Lists available legal documents (privacy policy, terms of service, etc.).

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `lang` | string | No | Language code (default: `pt-br`) |
| `active_only` | boolean | No | Only published versions (default: true) |

**Response:**

```json
{
  "documents": [
    {
      "slug": "privacy-policy",
      "title": "Política de Privacidade",
      "language": "pt-br",
      "version": "3.1",
      "effective_date": "2025-05-01",
      "published": true
    },
    {
      "slug": "terms-of-service",
      "title": "Termos de Serviço",
      "language": "pt-br",
      "version": "2.0",
      "effective_date": "2025-01-15",
      "published": true
    },
    {
      "slug": "cookie-policy",
      "title": "Política de Cookies",
      "language": "pt-br",
      "version": "2.1",
      "effective_date": "2025-05-15",
      "published": true
    }
  ]
}
```

---

**`POST /api/legal/documents`**

Creates a new legal document.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `slug` | string | Yes | URL-friendly identifier |
| `title` | string | Yes | Document title |
| `content` | string | Yes | Markdown or HTML content |
| `language` | string | No | Language code (default: `pt-br`) |
| `effective_date` | string | Yes | ISO 8601 date |
| `published` | boolean | No | Publish immediately (default: false) |

**Request Body:**

```json
{
  "slug": "data-processing-agreement",
  "title": "Acordo de Tratamento de Dados",
  "content": "# Acordo de Tratamento de Dados\n\n## 1. Finalidade\n\n...",
  "language": "pt-br",
  "effective_date": "2025-07-01",
  "published": false
}
```

**Response:**

```json
{
  "slug": "data-processing-agreement",
  "title": "Acordo de Tratamento de Dados",
  "version": "1.0",
  "created_at": "2025-06-04T10:00:00Z",
  "published": false
}
```

---

**`GET /api/legal/documents/:slug`**

Retrieves a specific legal document.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `slug` | string | Yes | Document slug |
| `version` | string | No | Specific version (default: latest) |

**Response:**

```json
{
  "slug": "privacy-policy",
  "title": "Política de Privacidade",
  "language": "pt-br",
  "version": "3.1",
  "content": "# Política de Privacidade\n\n...",
  "effective_date": "2025-05-01",
  "published": true,
  "created_at": "2025-04-20T10:00:00Z",
  "updated_at": "2025-04-30T14:00:00Z"
}
```

---

**`PUT /api/legal/documents/:slug`**

Updates a legal document (creates a new version).

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `slug` | string | Yes | Document slug |
| `content` | string | No | Updated content |
| `title` | string | No | Updated title |
| `effective_date` | string | No | New effective date |
| `published` | boolean | No | Publish/unpublish |

**Request Body:**

```json
{
  "content": "# Política de Privacidade\n\nAtualizada em 2025...",
  "effective_date": "2025-08-01",
  "published": true
}
```

**Response:**

```json
{
  "slug": "privacy-policy",
  "version": "3.2",
  "updated_at": "2025-06-04T11:00:00Z",
  "published": true
}
```

---

### GDPR / Data Subject Rights

**`POST /api/legal/gdpr/delete/:user_id`**

Initiates a data deletion request (right to be forgotten).

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `user_id` | string | Yes | User ID to delete |
| `reason` | string | No | Deletion reason |
| `confirm` | boolean | Yes | Must be `true` to proceed |

**Request Body:**

```json
{
  "reason": "User requested account deletion via email",
  "confirm": true
}
```

**Response:**

```json
{
  "request_id": "gdpr_del_001",
  "user_id": "usr_001",
  "status": "pending",
  "created_at": "2025-06-04T12:00:00Z",
  "estimated_completion": "2025-07-04T12:00:00Z",
  "message": "Data deletion request submitted. Personal data will be anonymized within 30 days."
}
```

---

**`POST /api/legal/gdpr/export/:user_id`**

Exports all data for a user (right to data portability).

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `user_id` | string | Yes | User ID to export |
| `format` | string | No | `json`, `csv` (default: json) |

**Request Body:**

```json
{
  "format": "json"
}
```

**Response:**

```json
{
  "request_id": "gdpr_exp_001",
  "user_id": "usr_001",
  "status": "processing",
  "created_at": "2025-06-04T12:30:00Z",
  "download_url": null,
  "message": "Data export initiated. You will receive a download link when ready."
}
```

After processing:

```json
{
  "request_id": "gdpr_exp_001",
  "user_id": "usr_001",
  "status": "completed",
  "download_url": "/api/legal/gdpr/export/gdpr_exp_001/download",
  "expires_at": "2025-06-11T12:30:00Z",
  "size_bytes": 245760
}
```

---

## LGPD Legal Bases

| Base | Description | When Used |
|------|-------------|-----------|
| Consent | Explicit user consent | Marketing, analytics, profiling |
| Contract | Necessary for contract performance | Service delivery, payments |
| Legal Obligation | Required by law | Tax records, court orders |
| Vital Interests | Protects life | Emergency situations |
| Public Interest | Task carried in public interest | Government mandates |
| Legitimate Interest | Controller's legitimate interest | Fraud prevention, security |

---

## Response Codes

| Code | Description |
|------|-------------|
| 200 | Success |
| 201 | Consent recorded / document created |
| 202 | Accepted (async GDPR request) |
| 400 | Bad Request |
| 401 | Unauthorized |
| 403 | Forbidden |
| 404 | Resource not found |
| 409 | Conflict (duplicate consent) |
| 500 | Internal Server Error |

---

## See Also

- [Compliance API](./compliance-api.md) — ISO 27001 compliance checks
- [Security API](./security-api.md) — encryption and access controls
- [Admin API](./admin-api-full.md) — user management and audit trails
