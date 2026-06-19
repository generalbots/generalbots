# Compliance API 🟡 BETA

> **ISO 27001 compliance management — security checks, issue tracking, audit trails, training, evidence collection, and reporting.**

---

## Base URL

```
/api/compliance
```

## Authentication

All endpoints require a valid session token via `Authorization: Bearer <token>` header. Audit and evidence endpoints require `compliance_admin` or `admin` role.

---

## Endpoints

### Security Checks

**`GET /api/compliance/checks`**

Lists all compliance checks with optional filters.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `status` | string | No | Filter: `pass`, `fail`, `pending`, `skip` |
| `control` | string | No | ISO 27001 control ID (e.g., `A.8.1.1`) |
| `category` | string | No | Category filter |
| `page` | integer | No | Page number (default: 1) |
| `limit` | integer | No | Items per page (default: 50) |

**Response:**

```json
{
  "checks": [
    {
      "check_id": "chk_001",
      "name": "Asset Inventory",
      "control": "A.8.1.1",
      "category": "asset-management",
      "description": "Verify all information assets are inventoried",
      "status": "pass",
      "last_checked": "2025-06-04T08:00:00Z",
      "next_scheduled": "2025-06-11T08:00:00Z",
      "evidence_count": 3
    }
  ],
  "total": 42,
  "summary": {
    "pass": 35,
    "fail": 4,
    "pending": 2,
    "skip": 1
  }
}
```

---

**`POST /api/compliance/checks`**

Creates a new compliance check.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | Yes | Check name |
| `control` | string | Yes | ISO 27001 control reference |
| `category` | string | Yes | Check category |
| `description` | string | No | What this check verifies |
| `frequency` | string | No | `daily`, `weekly`, `monthly`, `quarterly` |
| `automated` | boolean | No | Whether check runs automatically (default: false) |

**Request Body:**

```json
{
  "name": "Password Policy Enforcement",
  "control": "A.9.4.3",
  "category": "access-control",
  "description": "Verify password complexity and rotation policies are enforced",
  "frequency": "weekly",
  "automated": true
}
```

**Response:**

```json
{
  "check_id": "chk_043",
  "name": "Password Policy Enforcement",
  "control": "A.9.4.3",
  "status": "pending",
  "created_at": "2025-06-04T09:00:00Z"
}
```

---

### Get Check Details

**`GET /api/compliance/checks/:check_id`**

Retrieves full details and history for a specific check.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `check_id` | string | Yes | Check ID |

**Response:**

```json
{
  "check_id": "chk_001",
  "name": "Asset Inventory",
  "control": "A.8.1.1",
  "category": "asset-management",
  "description": "Verify all information assets are inventoried",
  "status": "pass",
  "frequency": "monthly",
  "automated": false,
  "history": [
    {
      "date": "2025-06-04T08:00:00Z",
      "status": "pass",
      "checked_by": "admin@company.com",
      "notes": "All 127 assets accounted for in inventory system"
    },
    {
      "date": "2025-05-04T08:00:00Z",
      "status": "fail",
      "checked_by": "admin@company.com",
      "notes": "3 laptops missing from inventory",
      "issue_id": "iss_012"
    }
  ],
  "evidence_count": 3,
  "created_at": "2025-01-15T10:00:00Z"
}
```

---

### Security Issues

**`GET /api/compliance/issues`**

Lists security issues found during compliance checks.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `status` | string | No | `open`, `in_progress`, `resolved`, `accepted` |
| `severity` | string | No | `critical`, `high`, `medium`, `low` |
| `control` | string | No | ISO 27001 control ID |
| `assigned_to` | string | No | Filter by assignee email |
| `page` | integer | No | Page number |
| `limit` | integer | No | Items per page |

**Response:**

```json
{
  "issues": [
    {
      "issue_id": "iss_014",
      "title": "Unencrypted backup storage",
      "severity": "critical",
      "status": "in_progress",
      "control": "A.10.1.1",
      "check_id": "chk_015",
      "assigned_to": "security@company.com",
      "created_at": "2025-06-01T10:00:00Z",
      "due_date": "2025-06-15T00:00:00Z",
      "description": "Backup files stored without encryption at rest"
    }
  ],
  "total": 7,
  "by_severity": {
    "critical": 1,
    "high": 3,
    "medium": 2,
    "low": 1
  }
}
```

---

**`POST /api/compliance/issues`**

Creates a new security issue.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `title` | string | Yes | Issue title |
| `severity` | string | Yes | `critical`, `high`, `medium`, `low` |
| `control` | string | Yes | Related ISO 27001 control |
| `check_id` | string | No | Related check ID |
| `assigned_to` | string | No | Assignee email |
| `due_date` | string | No | ISO 8601 deadline |
| `description` | string | No | Detailed description |

**Request Body:**

```json
{
  "title": "Missing MFA on admin panel",
  "severity": "high",
  "control": "A.9.4.2",
  "check_id": "chk_022",
  "assigned_to": "devops@company.com",
  "due_date": "2025-06-20T00:00:00Z",
  "description": "Administrative panel does not enforce multi-factor authentication"
}
```

**Response:**

```json
{
  "issue_id": "iss_015",
  "title": "Missing MFA on admin panel",
  "severity": "high",
  "status": "open",
  "created_at": "2025-06-04T09:30:00Z"
}
```

---

**`PUT /api/compliance/issues/:issue_id`**

Updates an existing issue (status, assignment, notes).

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `issue_id` | string | Yes | Issue ID |
| `status` | string | No | New status |
| `assigned_to` | string | No | Reassign |
| `severity` | string | No | Escalate/de-escalate |
| `resolution_notes` | string | No | Notes when resolving |

**Request Body:**

```json
{
  "status": "resolved",
  "resolution_notes": "MFA enabled on all admin accounts via TOTP. Verified on 2025-06-04."
}
```

**Response:**

```json
{
  "issue_id": "iss_015",
  "status": "resolved",
  "resolved_at": "2025-06-04T14:00:00Z",
  "resolved_by": "admin@company.com"
}
```

---

### Audit Trail

**`GET /api/compliance/audit`**

Retrieves audit log entries.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `start_date` | string | No | ISO 8601 start (default: 30 days ago) |
| `end_date` | string | No | ISO 8601 end (default: now) |
| `action` | string | No | Filter by action type |
| `actor` | string | No | Filter by user email |
| `page` | integer | No | Page number |
| `limit` | integer | No | Items per page |

**Response:**

```json
{
  "entries": [
    {
      "audit_id": "aud_001",
      "timestamp": "2025-06-04T14:00:00Z",
      "actor": "admin@company.com",
      "action": "issue.resolved",
      "resource": "iss_015",
      "details": {
        "issue_title": "Missing MFA on admin panel",
        "resolution": "MFA enabled on all admin accounts"
      },
      "ip_address": "192.168.1.100"
    },
    {
      "audit_id": "aud_002",
      "timestamp": "2025-06-04T13:45:00Z",
      "actor": "security@company.com",
      "action": "check.completed",
      "resource": "chk_015",
      "details": {
        "result": "fail",
        "notes": "Backup encryption not configured"
      },
      "ip_address": "192.168.1.105"
    }
  ],
  "total": 256
}
```

---

**`POST /api/compliance/audit`**

Creates a manual audit log entry.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `action` | string | Yes | Action type |
| `resource` | string | No | Resource ID |
| `details` | object | No | Action details |

**Request Body:**

```json
{
  "action": "policy.reviewed",
  "resource": "pol_003",
  "details": {
    "policy_name": "Data Retention Policy",
    "review_result": "approved",
    "next_review_date": "2025-12-01"
  }
}
```

**Response:**

```json
{
  "audit_id": "aud_003",
  "timestamp": "2025-06-04T15:00:00Z",
  "actor": "admin@company.com",
  "action": "policy.reviewed"
}
```

---

### Training

**`POST /api/compliance/training`**

Records a completed training session or enrolls a user.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `user_email` | string | Yes | Trainee email |
| `training_name` | string | Yes | Training module name |
| `completed_at` | string | No | ISO 8601 completion time (default: now) |
| `score` | integer | No | Quiz score (0-100) |
| `expires_at` | string | No | Certification expiry |

**Request Body:**

```json
{
  "user_email": "dev@company.com",
  "training_name": "Security Awareness 2025",
  "score": 92,
  "expires_at": "2026-06-04T00:00:00Z"
}
```

**Response:**

```json
{
  "training_id": "trn_001",
  "user_email": "dev@company.com",
  "training_name": "Security Awareness 2025",
  "status": "completed",
  "score": 92,
  "completed_at": "2025-06-04T15:00:00Z",
  "expires_at": "2026-06-04T00:00:00Z"
}
```

---

### Compliance Report

**`GET /api/compliance/report`**

Generates an ISO 27001 compliance report.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `format` | string | No | `json`, `pdf`, `csv` (default: json) |
| `period` | string | No | `week`, `month`, `quarter`, `year` (default: month) |
| `include_evidence` | boolean | No | Include evidence links (default: false) |

**Response (json):**

```json
{
  "report_id": "rpt_001",
  "generated_at": "2025-06-04T16:00:00Z",
  "period": "2025-05-01T00:00:00Z/2025-06-01T00:00:00Z",
  "summary": {
    "total_checks": 42,
    "passed": 35,
    "failed": 4,
    "pending": 2,
    "skipped": 1,
    "compliance_score": 83.3,
    "open_issues": 7,
    "critical_issues": 1
  },
  "by_domain": [
    {
      "domain": "Access Control (A.9)",
      "checks": 8,
      "passed": 7,
      "score": 87.5
    },
    {
      "domain": "Cryptography (A.10)",
      "checks": 4,
      "passed": 2,
      "score": 50.0
    }
  ],
  "training_status": {
    "total_employees": 25,
    "trained": 22,
    "pending": 3,
    "coverage_percent": 88.0
  }
}
```

---

### Evidence Collection

**`POST /api/compliance/evidence`**

Uploads evidence for a compliance check or issue.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `check_id` | string | Yes | Related check ID |
| `title` | string | Yes | Evidence title |
| `type` | string | Yes | `screenshot`, `document`, `log_export`, `config_export`, `certificate` |
| `file_url` | string | No | URL to uploaded file |
| `notes` | string | No | Description of evidence |

**Request Body (multipart/form-data):**

```
check_id: chk_001
title: Asset inventory export from 2025-06-04
type: log_export
notes: Full export from asset management system
```

**Response:**

```json
{
  "evidence_id": "evd_001",
  "check_id": "chk_001",
  "title": "Asset inventory export from 2025-06-04",
  "type": "log_export",
  "file_url": "/compliance/evidence/evd_001_asset_export.csv",
  "uploaded_by": "admin@company.com",
  "uploaded_at": "2025-06-04T16:30:00Z"
}
```

---

## ISO 27001 Domains

| Domain | Controls | Description |
|--------|----------|-------------|
| A.5 | A.5.1 – A.5.23 | Information Security Policies |
| A.6 | A.6.1 – A.6.3 | Organization of Information Security |
| A.7 | A.7.1 – A.7.12 | Human Resource Security |
| A.8 | A.8.1 – A.8.34 | Asset Management |
| A.9 | A.9.1 – A.9.4 | Access Control |
| A.10 | A.10.1 – A.10.2 | Cryptography |
| A.11 | A.11.1 – A.11.2 | Physical and Environmental Security |
| A.12 | A.12.1 – A.12.4 | Operations Security |
| A.13 | A.13.1 – A.13.2 | Communications Security |
| A.14 | A.14.1 – A.14.3 | System Acquisition, Development and Maintenance |
| A.15 | A.15.1 – A.15.3 | Supplier Relationships |
| A.16 | A.16.1 – A.16.4 | Information Security Incident Management |
| A.17 | A.17.1 – A.17.3 | Business Continuity |
| A.18 | A.18.1 – A.18.2 | Compliance |

---

## Response Codes

| Code | Description |
|------|-------------|
| 200 | Success |
| 201 | Created |
| 204 | No Content |
| 400 | Bad Request (invalid parameters) |
| 401 | Unauthorized |
| 403 | Forbidden (insufficient permissions) |
| 404 | Resource not found |
| 500 | Internal Server Error |

---

## See Also

- [Legal / LGPD API](./legal-api.md) — data protection and consent
- [Security API](./security-api.md) — application security endpoints
- [Audit Logs](./admin-api-full.md) — system-wide audit trails
