# Compliance API Reference

> **Programmatic access to privacy and compliance features**

---

## Overview

The Compliance API allows you to programmatically manage data subject requests, consent records, and compliance scanning. Use this API to integrate privacy features into your applications or automate compliance workflows.

**Base URL:** `https://your-server.com/api/compliance`

---

## Authentication

All API requests require authentication using a Bearer token:

```/dev/null/auth-header.txt
Authorization: Bearer your-api-key
```

Get your API key from **Settings** → **API Keys** → **Create New Key** with `compliance` scope.

---

## Endpoints

### Data Subject Requests (DSR)

#### List All Requests

```/dev/null/api-request.txt
GET /api/compliance/dsr
```

**Query Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `status` | string | Filter by status: `pending`, `processing`, `completed`, `rejected` |
| `type` | string | Filter by type: `access`, `deletion`, `rectification`, `portability`, `objection` |
| `from` | date | Start date (YYYY-MM-DD) |
| `to` | date | End date (YYYY-MM-DD) |
| `limit` | number | Results per page (default: 20, max: 100) |
| `offset` | number | Pagination offset |

**Example Request:**

```/dev/null/dsr-list-request.txt
GET /api/compliance/dsr?status=pending&limit=10
```

**Example Response:**

```/dev/null/dsr-list-response.json
{
  "total": 7,
  "limit": 10,
  "offset": 0,
  "requests": [
    {
      "id": "DSR-2025-0142",
      "type": "access",
      "status": "pending",
      "userId": "usr_abc123",
      "email": "john.doe@email.com",
      "submittedAt": "2025-05-13T10:30:00Z",
      "dueDate": "2025-05-28T10:30:00Z",
      "assignee": null
    },
    {
      "id": "DSR-2025-0141",
      "type": "deletion",
      "status": "processing",
      "userId": "usr_def456",
      "email": "sarah@company.com",
      "submittedAt": "2025-05-10T14:15:00Z",
      "dueDate": "2025-05-25T14:15:00Z",
      "assignee": "admin@company.com"
    }
  ]
}
```

---

#### Get Single Request

```/dev/null/dsr-get-request.txt
GET /api/compliance/dsr/{id}
```

**Example Response:**

```/dev/null/dsr-get-response.json
{
  "id": "DSR-2025-0142",
  "type": "access",
  "status": "pending",
  "userId": "usr_abc123",
  "email": "john.doe@email.com",
  "name": "John Doe",
  "submittedAt": "2025-05-13T10:30:00Z",
  "dueDate": "2025-05-28T10:30:00Z",
  "assignee": null,
  "message": "I would like a copy of all my data",
  "verifiedAt": "2025-05-13T10:35:00Z",
  "dataFound": {
    "profile": true,
    "conversations": true,
    "consents": true,
    "activityLogs": true
  },
  "history": [
    {
      "action": "created",
      "timestamp": "2025-05-13T10:30:00Z",
      "actor": "system"
    },
    {
      "action": "verified",
      "timestamp": "2025-05-13T10:35:00Z",
      "actor": "system"
    }
  ]
}
```

---

#### Create Request

```/dev/null/dsr-create-request.txt
POST /api/compliance/dsr
```

**Request Body:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `type` | string | Yes | `access`, `deletion`, `rectification`, `portability`, `objection` |
| `email` | string | Yes | User's email address |
| `userId` | string | No | User ID if known |
| `message` | string | No | User's message/reason |
| `skipVerification` | boolean | No | Skip email verification (default: false) |

**Example Request:**

```/dev/null/dsr-create-body.json
POST /api/compliance/dsr
Content-Type: application/json

{
  "type": "access",
  "email": "john.doe@email.com",
  "message": "Please provide all my personal data"
}
```

**Example Response:**

```/dev/null/dsr-create-response.json
{
  "id": "DSR-2025-0143",
  "type": "access",
  "status": "pending_verification",
  "email": "john.doe@email.com",
  "submittedAt": "2025-05-15T14:00:00Z",
  "dueDate": "2025-05-30T14:00:00Z",
  "verificationSent": true
}
```

---

#### Update Request Status

```/dev/null/dsr-update-request.txt
PATCH /api/compliance/dsr/{id}
```

**Request Body:**

| Field | Type | Description |
|-------|------|-------------|
| `status` | string | `processing`, `completed`, `rejected` |
| `assignee` | string | Email of person handling request |
| `notes` | string | Internal notes |
| `rejectionReason` | string | Required if status is `rejected` |

**Example Request:**

```/dev/null/dsr-update-body.json
PATCH /api/compliance/dsr/DSR-2025-0142
Content-Type: application/json

{
  "status": "processing",
  "assignee": "admin@company.com"
}
```

---

#### Complete Request (with data package)

```/dev/null/dsr-complete-request.txt
POST /api/compliance/dsr/{id}/complete
```

**Request Body:**

| Field | Type | Description |
|-------|------|-------------|
| `notifyUser` | boolean | Send completion email (default: true) |
| `dataPackageUrl` | string | URL to downloadable data (for access/portability) |
| `expiresAt` | datetime | When download link expires |

**Example Request:**

```/dev/null/dsr-complete-body.json
POST /api/compliance/dsr/DSR-2025-0142/complete
Content-Type: application/json

{
  "notifyUser": true,
  "dataPackageUrl": "https://secure.company.com/data/abc123.zip",
  "expiresAt": "2025-06-15T00:00:00Z"
}
```

---

### Consent Management

#### Get User Consent

```/dev/null/consent-get-request.txt
GET /api/compliance/consent/{userId}
```

**Example Response:**

```/dev/null/consent-get-response.json
{
  "userId": "usr_abc123",
  "email": "john.doe@email.com",
  "consents": [
    {
      "type": "terms_of_service",
      "status": "given",
      "version": "2.3",
      "timestamp": "2025-01-15T10:32:00Z",
      "method": "web_form",
      "ip": "192.168.1.100"
    },
    {
      "type": "marketing",
      "status": "given",
      "timestamp": "2025-01-15T10:32:00Z",
      "method": "web_form"
    },
    {
      "type": "analytics",
      "status": "withdrawn",
      "timestamp": "2025-03-22T15:15:00Z",
      "method": "preference_center"
    }
  ]
}
```

---

#### Record Consent

```/dev/null/consent-record-request.txt
POST /api/compliance/consent
```

**Request Body:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `userId` | string | Yes | User identifier |
| `email` | string | Yes | User's email |
| `type` | string | Yes | Consent type (e.g., `marketing`, `analytics`) |
| `status` | string | Yes | `given` or `withdrawn` |
| `method` | string | No | How consent was collected |
| `ip` | string | No | User's IP address |
| `userAgent` | string | No | User's browser |

**Example Request:**

```/dev/null/consent-record-body.json
POST /api/compliance/consent
Content-Type: application/json

{
  "userId": "usr_abc123",
  "email": "john.doe@email.com",
  "type": "marketing",
  "status": "given",
  "method": "chatbot",
  "ip": "192.168.1.100"
}
```

**Example Response:**

```/dev/null/consent-record-response.json
{
  "success": true,
  "consentId": "con_xyz789",
  "userId": "usr_abc123",
  "type": "marketing",
  "status": "given",
  "timestamp": "2025-05-15T14:30:00Z"
}
```

---

#### Withdraw Consent

```/dev/null/consent-withdraw-request.txt
DELETE /api/compliance/consent/{userId}/{type}
```

**Example Request:**

```/dev/null/consent-withdraw-example.txt
DELETE /api/compliance/consent/usr_abc123/marketing
```

**Example Response:**

```/dev/null/consent-withdraw-response.json
{
  "success": true,
  "userId": "usr_abc123",
  "type": "marketing",
  "status": "withdrawn",
  "timestamp": "2025-05-15T14:35:00Z"
}
```

---

#### List Consent Types

```/dev/null/consent-types-request.txt
GET /api/compliance/consent-types
```

**Example Response:**

```/dev/null/consent-types-response.json
{
  "consentTypes": [
    {
      "id": "terms_of_service",
      "name": "Terms of Service",
      "required": true,
      "description": "Agreement to terms and conditions",
      "currentVersion": "2.3"
    },
    {
      "id": "marketing",
      "name": "Marketing Communications",
      "required": false,
      "description": "Receive promotional emails and offers"
    },
    {
      "id": "analytics",
      "name": "Analytics & Improvement",
      "required": false,
      "description": "Help us improve by analyzing usage patterns"
    }
  ]
}
```

---

### Compliance Scanning

#### Start a Scan

```/dev/null/scan-start-request.txt
POST /api/compliance/scan
```

**Request Body:**

| Field | Type | Description |
|-------|------|-------------|
| `type` | string | `quick`, `full`, or `custom` |
| `targets` | array | For custom: `["bots", "kb", "users", "logs"]` |
| `botId` | string | Scan specific bot only |

**Example Request:**

```/dev/null/scan-start-body.json
POST /api/compliance/scan
Content-Type: application/json

{
  "type": "full",
  "targets": ["bots", "kb", "users", "logs"]
}
```

**Example Response:**

```/dev/null/scan-start-response.json
{
  "scanId": "scan_20250515_001",
  "status": "running",
  "type": "full",
  "startedAt": "2025-05-15T14:45:00Z",
  "estimatedDuration": "30 minutes"
}
```

---

#### Get Scan Status

```/dev/null/scan-status-request.txt
GET /api/compliance/scan/{scanId}
```

**Example Response (In Progress):**

```/dev/null/scan-status-progress.json
{
  "scanId": "scan_20250515_001",
  "status": "running",
  "progress": 45,
  "currentStep": "Scanning conversation logs",
  "startedAt": "2025-05-15T14:45:00Z"
}
```

**Example Response (Complete):**

```/dev/null/scan-status-complete.json
{
  "scanId": "scan_20250515_001",
  "status": "completed",
  "progress": 100,
  "startedAt": "2025-05-15T14:45:00Z",
  "completedAt": "2025-05-15T15:12:00Z",
  "summary": {
    "totalChecks": 148,
    "passed": 145,
    "warnings": 2,
    "critical": 1
  },
  "issues": [
    {
      "severity": "critical",
      "type": "unencrypted_pii",
      "description": "Unencrypted PII found in conversation logs",
      "location": "support-bot/logs/2025-05-10",
      "affectedRecords": 23,
      "recommendation": "Enable automatic PII redaction"
    },
    {
      "severity": "warning",
      "type": "consent_expiring",
      "description": "Consent records older than 2 years",
      "affectedUsers": 12,
      "recommendation": "Send consent renewal requests"
    }
  ]
}
```

---

#### Get Latest Scan Results

```/dev/null/scan-latest-request.txt
GET /api/compliance/scan/latest
```

Returns the most recent completed scan results.

---

### Reports

#### Generate Compliance Report

```/dev/null/report-generate-request.txt
POST /api/compliance/report
```

**Request Body:**

| Field | Type | Description |
|-------|------|-------------|
| `type` | string | `summary`, `detailed`, `audit` |
| `period` | string | `last_30_days`, `last_90_days`, `year`, `custom` |
| `from` | date | Start date for custom period |
| `to` | date | End date for custom period |
| `format` | string | `json`, `pdf`, `csv` |

**Example Request:**

```/dev/null/report-generate-body.json
POST /api/compliance/report
Content-Type: application/json

{
  "type": "summary",
  "period": "last_30_days",
  "format": "json"
}
```

**Example Response:**

```/dev/null/report-generate-response.json
{
  "reportId": "rpt_20250515_001",
  "generatedAt": "2025-05-15T15:00:00Z",
  "period": {
    "from": "2025-04-15",
    "to": "2025-05-15"
  },
  "summary": {
    "overallScore": 92,
    "dsrRequests": {
      "received": 15,
      "completed": 12,
      "pending": 3,
      "averageResponseDays": 8.5
    },
    "consentRate": 94.2,
    "dataBreaches": 0,
    "scansPerformed": 4,
    "issuesFound": 7,
    "issuesResolved": 5
  }
}
```

---

#### Download Report

```/dev/null/report-download-request.txt
GET /api/compliance/report/{reportId}/download
```

Returns the report file in the requested format.

---

### Data Deletion

#### Delete User Data

```/dev/null/data-delete-request.txt
DELETE /api/compliance/user/{userId}/data
```

**Query Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `scope` | string | `all`, `conversations`, `profile`, `analytics` |
| `confirm` | boolean | Must be `true` to execute |

**Example Request:**

```/dev/null/data-delete-example.txt
DELETE /api/compliance/user/usr_abc123/data?scope=all&confirm=true
```

**Example Response:**

```/dev/null/data-delete-response.json
{
  "success": true,
  "userId": "usr_abc123",
  "deletedAt": "2025-05-15T15:30:00Z",
  "scope": "all",
  "itemsDeleted": {
    "profile": 1,
    "conversations": 45,
    "consents": 3,
    "activityLogs": 234
  },
  "retainedForLegal": {
    "auditLogs": 15
  }
}
```

---

## Error Responses

All errors follow this format:

```/dev/null/error-response.json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Human readable message",
    "details": {}
  }
}
```

**Common Error Codes:**

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `UNAUTHORIZED` | 401 | Invalid or missing API key |
| `FORBIDDEN` | 403 | API key lacks required scope |
| `NOT_FOUND` | 404 | Resource not found |
| `VALIDATION_ERROR` | 400 | Invalid request parameters |
| `RATE_LIMITED` | 429 | Too many requests |
| `INTERNAL_ERROR` | 500 | Server error |

---

## Rate Limits

| Endpoint | Limit |
|----------|-------|
| All endpoints | 100 requests/minute |
| Scan endpoints | 5 requests/hour |
| Report generation | 10 requests/hour |

Rate limit headers are included in responses:

```/dev/null/rate-limit-headers.txt
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1621234567
```

---

## Webhooks

Configure webhooks to receive real-time notifications.

**Available Events:**

| Event | Description |
|-------|-------------|
| `dsr.created` | New DSR submitted |
| `dsr.completed` | DSR marked complete |
| `dsr.due_soon` | DSR due within 3 days |
| `consent.changed` | User consent updated |
| `scan.completed` | Compliance scan finished |
| `issue.critical` | Critical issue detected |

**Webhook Payload Example:**

```/dev/null/webhook-payload.json
POST https://your-server.com/webhook
Content-Type: application/json
X-Signature: sha256=...

{
  "event": "dsr.created",
  "timestamp": "2025-05-15T14:00:00Z",
  "data": {
    "id": "DSR-2025-0143",
    "type": "access",
    "email": "user@example.com"
  }
}
```

---

## See Also

- [Compliance App](./compliance.md) - User interface guide
- [How To: Configure Compliance](../how-to/configure-compliance.md)
- [BASIC Compliance Keywords](../../chapter-06-gbdialog/keywords-reference.md)