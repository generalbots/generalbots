# Compliance API

The Compliance API provides endpoints for regulatory compliance management, audit trails, and policy enforcement.

## Status: Roadmap

This API is on the development roadmap. The endpoints documented below represent the planned interface design.

## Base URL

```
http://localhost:9000/api/v1/compliance
```

## Authentication

Uses the standard botserver authentication mechanism with appropriate role-based permissions.

## Endpoints

### Compliance Status

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/compliance/status` | Get overall compliance status |
| GET | `/api/v1/compliance/status/{framework}` | Get status for specific framework (GDPR, CCPA, HIPAA) |

### Audit Trails

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/compliance/audit-trails` | List audit trail entries |
| GET | `/api/v1/compliance/audit-trails/{id}` | Get specific audit entry |
| POST | `/api/v1/compliance/audit-trails/export` | Export audit trails to file |

### Policy Management

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/compliance/policies` | List all policies |
| POST | `/api/v1/compliance/policies` | Create a new policy |
| GET | `/api/v1/compliance/policies/{policy_id}` | Get policy details |
| PUT | `/api/v1/compliance/policies/{policy_id}` | Update a policy |
| DELETE | `/api/v1/compliance/policies/{policy_id}` | Delete a policy |

### Compliance Reports

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/v1/compliance/reports/generate` | Generate a compliance report |
| GET | `/api/v1/compliance/reports` | List generated reports |
| GET | `/api/v1/compliance/reports/{report_id}` | Download a report |

### Data Governance

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/compliance/data-governance` | Get data governance status |
| POST | `/api/v1/compliance/data-governance/scan` | Initiate data classification scan |
| GET | `/api/v1/compliance/data-governance/scan/{scan_id}` | Get scan results |

### Privacy Management

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/v1/compliance/privacy/request` | Submit privacy request (DSAR) |
| GET | `/api/v1/compliance/privacy/requests` | List privacy requests |
| GET | `/api/v1/compliance/privacy/status/{request_id}` | Get request status |

### Retention Policies

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/compliance/retention` | Get retention policies |
| PUT | `/api/v1/compliance/retention` | Update retention policies |
| POST | `/api/v1/compliance/retention/apply` | Apply retention policy |

## Request Examples

### Check Compliance Status

```bas
status = GET "/api/v1/compliance/status"
TALK "GDPR Status: " + status.gdpr.status
TALK "Last Audit: " + status.last_audit_date
```

### Create a Policy

```bas
policy = NEW OBJECT
policy.name = "Data Retention Policy"
policy.framework = "GDPR"
policy.rules = ["retain_logs_90_days", "anonymize_pii_on_request"]

result = POST "/api/v1/compliance/policies", policy
TALK "Policy created: " + result.id
```

### Submit Privacy Request

```bas
request = NEW OBJECT
request.type = "data_export"
request.email = "user@example.com"
request.reason = "GDPR Article 20 - Data Portability"

result = POST "/api/v1/compliance/privacy/request", request
TALK "Request ID: " + result.request_id
```

### Generate Compliance Report

```bas
report_config = NEW OBJECT
report_config.framework = "GDPR"
report_config.period = "2024-Q1"
report_config.format = "pdf"

result = POST "/api/v1/compliance/reports/generate", report_config
TALK "Report generation started: " + result.report_id
```

## Response Codes

| Code | Description |
|------|-------------|
| 200 | Success |
| 201 | Created |
| 202 | Accepted (async operation started) |
| 400 | Bad Request |
| 401 | Unauthorized |
| 403 | Forbidden (insufficient permissions) |
| 404 | Not Found |
| 500 | Internal Server Error |

## Supported Compliance Frameworks

| Framework | Description |
|-----------|-------------|
| GDPR | General Data Protection Regulation (EU) |
| CCPA | California Consumer Privacy Act |
| HIPAA | Health Insurance Portability and Accountability Act |
| SOC2 | Service Organization Control 2 |
| ISO27001 | Information Security Management |

## Required Permissions

| Endpoint Category | Required Role |
|-------------------|---------------|
| Compliance Status | `compliance_viewer` or higher |
| Audit Trails | `compliance_auditor` or `admin` |
| Policy Management | `compliance_admin` or `admin` |
| Reports | `compliance_viewer` or higher |
| Data Governance | `compliance_admin` or `admin` |
| Privacy Requests | `privacy_officer` or `admin` |
| Retention Policies | `compliance_admin` or `admin` |