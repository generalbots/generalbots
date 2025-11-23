# Compliance API

## Overview

The Compliance API provides comprehensive compliance monitoring, audit logging, risk assessment, and regulatory reporting capabilities for BotServer. It supports GDPR, SOC 2, ISO 27001, HIPAA, and other compliance frameworks.

## Endpoints

### Get Compliance Dashboard

Retrieves an overview of current compliance status across all frameworks.

**Endpoint**: `GET /api/compliance/dashboard`

**Authentication**: Required (Admin/Compliance Officer)

**Response**:
```json
{
  "success": true,
  "data": {
    "overall_compliance_score": 94.5,
    "last_updated": "2024-01-15T16:00:00Z",
    "frameworks": {
      "gdpr": {
        "status": "compliant",
        "score": 96.0,
        "last_audit": "2024-01-01T00:00:00Z",
        "next_audit": "2024-04-01T00:00:00Z",
        "issues_count": 2,
        "critical_issues": 0
      },
      "soc2": {
        "status": "compliant",
        "score": 95.5,
        "last_audit": "2023-12-15T00:00:00Z",
        "next_audit": "2024-12-15T00:00:00Z",
        "issues_count": 3,
        "critical_issues": 0
      },
      "iso27001": {
        "status": "in_progress",
        "score": 92.0,
        "last_audit": "2024-01-10T00:00:00Z",
        "next_audit": "2024-07-10T00:00:00Z",
        "issues_count": 8,
        "critical_issues": 1
      },
      "hipaa": {
        "status": "not_applicable",
        "score": null,
        "last_audit": null,
        "next_audit": null,
        "issues_count": 0,
        "critical_issues": 0
      }
    },
    "recent_audits": [
      {
        "id": "audit_001",
        "framework": "gdpr",
        "type": "quarterly_review",
        "date": "2024-01-01T00:00:00Z",
        "auditor": "John Doe",
        "status": "passed",
        "findings_count": 2
      }
    ],
    "upcoming_tasks": [
      {
        "id": "task_001",
        "title": "Quarterly Access Review",
        "due_date": "2024-01-31T00:00:00Z",
        "priority": "high",
        "assigned_to": "Security Team"
      }
    ],
    "metrics": {
      "policies_compliance": 98.5,
      "training_completion": 100,
      "incident_response_time_avg": 2.5,
      "data_breach_count": 0,
      "vulnerability_remediation_rate": 95.0
    }
  }
}
```

**Example**:
```bash
curl -X GET "http://localhost:3000/api/compliance/dashboard" \
  -H "Authorization: Bearer YOUR_TOKEN"
```

---

### List Audit Logs

Retrieves audit logs with filtering and search capabilities.

**Endpoint**: `GET /api/compliance/audit-logs`

**Authentication**: Required (Admin/Compliance Officer)

**Query Parameters**:
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `event_type` | string | No | Filter by event type: `access`, `modification`, `deletion`, `security`, `admin` |
| `user_id` | string | No | Filter by user ID |
| `resource_type` | string | No | Filter by resource type: `user`, `file`, `configuration`, `database` |
| `severity` | string | No | Filter by severity: `low`, `medium`, `high`, `critical` |
| `start_date` | string | No | Start date (ISO 8601) |
| `end_date` | string | No | End date (ISO 8601) |
| `search` | string | No | Search in log messages |
| `page` | integer | No | Page number (default: 1) |
| `per_page` | integer | No | Results per page (default: 50, max: 100) |

**Response**:
```json
{
  "success": true,
  "data": {
    "logs": [
      {
        "id": "log_123456",
        "timestamp": "2024-01-15T14:32:15Z",
        "event_type": "access",
        "event_category": "data_access",
        "severity": "medium",
        "user": {
          "id": "user_789",
          "username": "john.doe",
          "email": "john.doe@example.com",
          "ip_address": "192.168.1.100",
          "user_agent": "Mozilla/5.0..."
        },
        "resource": {
          "type": "file",
          "id": "file_456",
          "name": "confidential_report.pdf",
          "classification": "company_confidential"
        },
        "action": "download",
        "result": "success",
        "details": {
          "file_size": 2048576,
          "access_reason": "quarterly_report_review",
          "approval_id": "approval_321"
        },
        "location": {
          "country": "US",
          "region": "California",
          "city": "San Francisco"
        },
        "session_id": "session_654",
        "correlation_id": "corr_987"
      }
    ],
    "total": 15234,
    "page": 1,
    "per_page": 50,
    "total_pages": 305
  }
}
```

**Example**:
```bash
curl -X GET "http://localhost:3000/api/compliance/audit-logs?event_type=access&severity=high&start_date=2024-01-01" \
  -H "Authorization: Bearer YOUR_TOKEN"
```

---

### Export Audit Logs

Exports audit logs in various formats for compliance reporting.

**Endpoint**: `POST /api/compliance/audit-logs/export`

**Authentication**: Required (Admin/Compliance Officer)

**Request Body**:
```json
{
  "format": "csv",
  "filters": {
    "event_type": "security",
    "start_date": "2024-01-01T00:00:00Z",
    "end_date": "2024-01-31T23:59:59Z",
    "severity": ["high", "critical"]
  },
  "fields": [
    "timestamp",
    "event_type",
    "user",
    "action",
    "resource",
    "result",
    "ip_address"
  ],
  "include_metadata": true
}
```

**Parameters**:
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `format` | string | Yes | Export format: `csv`, `json`, `pdf`, `xlsx` |
| `filters` | object | No | Same filters as list endpoint |
| `fields` | array | No | Fields to include (default: all) |
| `include_metadata` | boolean | No | Include export metadata (default: true) |

**Response**:
```json
{
  "success": true,
  "data": {
    "export_id": "export_123",
    "status": "processing",
    "estimated_completion": "2024-01-15T14:35:00Z",
    "download_url": null,
    "expires_at": null
  },
  "message": "Export initiated. You will receive a notification when ready."
}
```

**Status Check**:
```bash
curl -X GET "http://localhost:3000/api/compliance/exports/export_123" \
  -H "Authorization: Bearer YOUR_TOKEN"
```

**Response (when ready)**:
```json
{
  "success": true,
  "data": {
    "export_id": "export_123",
    "status": "completed",
    "format": "csv",
    "file_size_bytes": 5242880,
    "records_count": 1234,
    "download_url": "/api/compliance/exports/export_123/download",
    "expires_at": "2024-01-22T14:33:00Z"
  }
}
```

---

### Run Compliance Check

Executes a compliance check against specified frameworks or controls.

**Endpoint**: `POST /api/compliance/checks/run`

**Authentication**: Required (Admin/Compliance Officer)

**Request Body**:
```json
{
  "framework": "gdpr",
  "scope": "full",
  "controls": ["data_protection", "access_control", "encryption"],
  "notify_on_completion": true,
  "generate_report": true
}
```

**Parameters**:
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `framework` | string | Yes | Framework: `gdpr`, `soc2`, `iso27001`, `hipaa`, `pci_dss` |
| `scope` | string | No | Scope: `full`, `partial`, `critical_only` (default: full) |
| `controls` | array | No | Specific controls to check (default: all) |
| `notify_on_completion` | boolean | No | Send notification when complete (default: true) |
| `generate_report` | boolean | No | Generate detailed report (default: true) |

**Response**:
```json
{
  "success": true,
  "data": {
    "check_id": "check_456",
    "framework": "gdpr",
    "status": "running",
    "started_at": "2024-01-15T14:40:00Z",
    "estimated_duration_minutes": 15,
    "progress_url": "/api/compliance/checks/check_456/status"
  },
  "message": "Compliance check initiated"
}
```

**Example**:
```bash
curl -X POST "http://localhost:3000/api/compliance/checks/run" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "framework": "gdpr",
    "scope": "full"
  }'
```

---

### Get Compliance Check Status

Retrieves the status of a running compliance check.

**Endpoint**: `GET /api/compliance/checks/{check_id}/status`

**Authentication**: Required (Admin/Compliance Officer)

**Response**:
```json
{
  "success": true,
  "data": {
    "check_id": "check_456",
    "framework": "gdpr",
    "status": "completed",
    "started_at": "2024-01-15T14:40:00Z",
    "completed_at": "2024-01-15T14:52:30Z",
    "duration_minutes": 12.5,
    "results": {
      "overall_compliance": 96.0,
      "controls_checked": 24,
      "controls_passed": 22,
      "controls_failed": 2,
      "controls_warning": 3,
      "critical_issues": 0,
      "high_issues": 2,
      "medium_issues": 3,
      "low_issues": 5
    },
    "report_url": "/api/compliance/reports/report_789",
    "findings": [
      {
        "control_id": "gdpr_7.2",
        "control_name": "Data Retention Policy",
        "status": "failed",
        "severity": "high",
        "description": "User data retention exceeds policy limits",
        "evidence": "Found 15 user accounts with data older than 2 years",
        "remediation": "Implement automated data deletion for inactive accounts",
        "due_date": "2024-02-15T00:00:00Z"
      }
    ]
  }
}
```

---

### Create Risk Assessment

Creates a new risk assessment for compliance purposes.

**Endpoint**: `POST /api/compliance/risk-assessments`

**Authentication**: Required (Admin/Compliance Officer)

**Request Body**:
```json
{
  "title": "Q1 2024 Risk Assessment",
  "description": "Quarterly risk assessment for all systems",
  "scope": {
    "systems": ["production", "staging", "backup"],
    "data_types": ["pii", "financial", "confidential"],
    "departments": ["engineering", "operations", "sales"]
  },
  "methodology": "nist_800_30",
  "assessor": {
    "name": "Jane Smith",
    "email": "jane.smith@example.com",
    "role": "Security Analyst"
  },
  "due_date": "2024-02-01T00:00:00Z"
}
```

**Response**:
```json
{
  "success": true,
  "data": {
    "assessment_id": "assessment_321",
    "status": "draft",
    "created_at": "2024-01-15T15:00:00Z",
    "url": "/api/compliance/risk-assessments/assessment_321"
  },
  "message": "Risk assessment created successfully"
}
```

---

### List Risk Assessments

Retrieves all risk assessments with filtering options.

**Endpoint**: `GET /api/compliance/risk-assessments`

**Authentication**: Required (Admin/Compliance Officer)

**Query Parameters**:
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `status` | string | No | Filter by status: `draft`, `in_progress`, `completed`, `approved` |
| `start_date` | string | No | Filter by creation date (ISO 8601) |
| `end_date` | string | No | Filter by creation date (ISO 8601) |
| `assessor` | string | No | Filter by assessor email |
| `page` | integer | No | Page number (default: 1) |
| `per_page` | integer | No | Results per page (default: 20) |

**Response**:
```json
{
  "success": true,
  "data": {
    "assessments": [
      {
        "id": "assessment_321",
        "title": "Q1 2024 Risk Assessment",
        "status": "completed",
        "created_at": "2024-01-15T15:00:00Z",
        "completed_at": "2024-01-20T16:30:00Z",
        "assessor": "Jane Smith",
        "risk_summary": {
          "critical_risks": 1,
          "high_risks": 3,
          "medium_risks": 8,
          "low_risks": 15,
          "total_risks": 27
        },
        "overall_risk_score": 6.2,
        "risk_level": "medium"
      }
    ],
    "total": 12,
    "page": 1,
    "per_page": 20,
    "total_pages": 1
  }
}
```

---

### Get Risk Assessment Details

Retrieves detailed information about a specific risk assessment.

**Endpoint**: `GET /api/compliance/risk-assessments/{assessment_id}`

**Authentication**: Required (Admin/Compliance Officer)

**Response**:
```json
{
  "success": true,
  "data": {
    "id": "assessment_321",
    "title": "Q1 2024 Risk Assessment",
    "description": "Quarterly risk assessment for all systems",
    "status": "completed",
    "created_at": "2024-01-15T15:00:00Z",
    "completed_at": "2024-01-20T16:30:00Z",
    "assessor": {
      "name": "Jane Smith",
      "email": "jane.smith@example.com",
      "role": "Security Analyst"
    },
    "methodology": "nist_800_30",
    "scope": {
      "systems": ["production", "staging", "backup"],
      "data_types": ["pii", "financial", "confidential"],
      "departments": ["engineering", "operations", "sales"]
    },
    "overall_risk_score": 6.2,
    "risk_level": "medium",
    "risks": [
      {
        "id": "risk_001",
        "title": "Unpatched Critical Vulnerability",
        "description": "Critical security vulnerability in web server",
        "category": "technical",
        "asset": "Production Web Server",
        "threat": "External exploitation of known vulnerability",
        "vulnerability": "CVE-2024-12345 - Remote Code Execution",
        "likelihood": {
          "score": 4,
          "level": "high",
          "justification": "Publicly known exploit available"
        },
        "impact": {
          "score": 5,
          "level": "critical",
          "justification": "Could lead to complete system compromise",
          "financial": 500000,
          "reputation": "severe",
          "operational": "severe",
          "compliance": "moderate"
        },
        "risk_score": 20,
        "risk_level": "critical",
        "current_controls": [
          "WAF enabled",
          "Network segmentation"
        ],
        "control_effectiveness": "partial",
        "residual_risk_score": 12,
        "treatment": {
          "strategy": "mitigate",
          "actions": [
            "Apply security patch within 24 hours",
            "Increase monitoring",
            "Conduct vulnerability scan"
          ],
          "owner": "Infrastructure Team",
          "due_date": "2024-01-16T00:00:00Z",
          "status": "in_progress",
          "estimated_cost": 5000
        },
        "status": "open",
        "created_at": "2024-01-15T15:30:00Z",
        "updated_at": "2024-01-15T16:00:00Z"
      }
    ],
    "summary": {
      "total_risks": 27,
      "critical_risks": 1,
      "high_risks": 3,
      "medium_risks": 8,
      "low_risks": 15,
      "accepted_risks": 5,
      "mitigated_risks": 18,
      "transferred_risks": 2,
      "avoided_risks": 2
    },
    "recommendations": [
      "Prioritize remediation of critical and high-risk items",
      "Implement automated patch management",
      "Conduct monthly vulnerability assessments"
    ],
    "approvals": [
      {
        "approver": "John Manager",
        "role": "CTO",
        "approved_at": "2024-01-21T09:00:00Z",
        "comments": "Approved. Ensure critical items addressed by Q2."
      }
    ]
  }
}
```

---

### Record Training Completion

Records security or compliance training completion for users.

**Endpoint**: `POST /api/compliance/training/record`

**Authentication**: Required (Admin/HR)

**Request Body**:
```json
{
  "user_id": "user_789",
  "training_type": "security_awareness",
  "training_name": "Annual Security Awareness Training 2024",
  "completion_date": "2024-01-15T14:00:00Z",
  "score": 95,
  "certificate_url": "https://training.example.com/cert/123456",
  "valid_until": "2025-01-15T00:00:00Z",
  "provider": "KnowBe4"
}
```

**Response**:
```json
{
  "success": true,
  "data": {
    "record_id": "training_record_456",
    "user_id": "user_789",
    "training_type": "security_awareness",
    "completion_date": "2024-01-15T14:00:00Z",
    "valid_until": "2025-01-15T00:00:00Z",
    "status": "valid"
  },
  "message": "Training completion recorded successfully"
}
```

---

### Get Training Records

Retrieves training records for compliance reporting.

**Endpoint**: `GET /api/compliance/training/records`

**Authentication**: Required (Admin/HR/Compliance Officer)

**Query Parameters**:
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `user_id` | string | No | Filter by user ID |
| `training_type` | string | No | Filter by training type |
| `status` | string | No | Filter by status: `valid`, `expired`, `pending` |
| `department` | string | No | Filter by department |
| `start_date` | string | No | Completed after date (ISO 8601) |
| `end_date` | string | No | Completed before date (ISO 8601) |
| `page` | integer | No | Page number (default: 1) |
| `per_page` | integer | No | Results per page (default: 50) |

**Response**:
```json
{
  "success": true,
  "data": {
    "records": [
      {
        "record_id": "training_record_456",
        "user": {
          "id": "user_789",
          "name": "John Doe",
          "email": "john.doe@example.com",
          "department": "Engineering"
        },
        "training_type": "security_awareness",
        "training_name": "Annual Security Awareness Training 2024",
        "completion_date": "2024-01-15T14:00:00Z",
        "score": 95,
        "status": "valid",
        "valid_until": "2025-01-15T00:00:00Z",
        "certificate_url": "https://training.example.com/cert/123456"
      }
    ],
    "summary": {
      "total_records": 234,
      "completed": 230,
      "pending": 4,
      "expired": 12,
      "compliance_rate": 98.3
    },
    "total": 234,
    "page": 1,
    "per_page": 50,
    "total_pages": 5
  }
}
```

---

### Generate Compliance Report

Generates a comprehensive compliance report.

**Endpoint**: `POST /api/compliance/reports/generate`

**Authentication**: Required (Admin/Compliance Officer)

**Request Body**:
```json
{
  "report_type": "quarterly_compliance",
  "framework": "gdpr",
  "period": {
    "start_date": "2024-01-01T00:00:00Z",
    "end_date": "2024-03-31T23:59:59Z"
  },
  "sections": [
    "executive_summary",
    "compliance_status",
    "audit_findings",
    "risk_assessment",
    "incidents",
    "training_compliance",
    "recommendations"
  ],
  "format": "pdf",
  "include_evidence": true,
  "confidentiality": "internal"
}
```

**Response**:
```json
{
  "success": true,
  "data": {
    "report_id": "report_789",
    "status": "generating",
    "estimated_completion": "2024-01-15T15:30:00Z",
    "progress_url": "/api/compliance/reports/report_789/status"
  },
  "message": "Report generation initiated"
}
```

---

### Get Incident Reports

Retrieves security incident reports for compliance purposes.

**Endpoint**: `GET /api/compliance/incidents`

**Authentication**: Required (Admin/Compliance Officer)

**Query Parameters**:
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `severity` | string | No | Filter by severity: `low`, `medium`, `high`, `critical` |
| `status` | string | No | Filter by status: `open`, `investigating`, `contained`, `resolved`, `closed` |
| `category` | string | No | Filter by category: `data_breach`, `malware`, `unauthorized_access`, `dos`, `other` |
| `start_date` | string | No | Incidents after date (ISO 8601) |
| `end_date` | string | No | Incidents before date (ISO 8601) |
| `reported_by` | string | No | Filter by reporter user ID |
| `page` | integer | No | Page number (default: 1) |
| `per_page` | integer | No | Results per page (default: 20) |

**Response**:
```json
{
  "success": true,
  "data": {
    "incidents": [
      {
        "id": "incident_001",
        "title": "Suspicious Login Attempts",
        "category": "unauthorized_access",
        "severity": "medium",
        "status": "resolved",
        "detected_at": "2024-01-15T08:23:15Z",
        "reported_at": "2024-01-15T08:30:00Z",
        "resolved_at": "2024-01-15T10:45:00Z",
        "reported_by": {
          "id": "user_123",
          "name": "Security Monitor",
          "email": "security@example.com"
        },
        "affected_systems": ["authentication_service", "user_database"],
        "affected_users_count": 1,
        "data_compromised": false,
        "notification_required": false,
        "root_cause": "Automated bot attempting to brute force user account",
        "response_actions": [
          "Blocked IP address",
          "Reset user password",
          "Enabled additional monitoring"
        ],
        "lessons_learned": "Implement rate limiting on login endpoint",
        "follow_up_actions": [
          {
            "action": "Deploy rate limiting",
            "owner": "Engineering Team",
            "due_date": "2024-01-20T00:00:00Z",
            "status": "completed"
          }
        ]
      }
    ],
    "total": 15,
    "page": 1,
    "per_page": 20,
    "total_pages": 1
  }
}
```

---

### Get Data Subject Access Request (DSAR)

Handles GDPR data subject access requests.

**Endpoint**: `POST /api/compliance/dsar/create`

**Authentication**: Required (Admin/Privacy Officer)

**Request Body**:
```json
{
  "subject_email": "user@example.com",
  "request_type": "access",
  "requested_data": ["personal_info", "activity_logs", "communications"],
  "requester_verification": "email_verified",
  "due_date": "2024-02-14T00:00:00Z"
}
```

**Parameters**:
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `subject_email` | string | Yes | Email of data subject |
| `request_type` | string | Yes | Type: `access`, `deletion`, `rectification`, `portability`, `restriction` |
| `requested_data` | array | No | Specific data categories requested |
| `requester_verification` | string | Yes | Verification method used |
| `due_date` | string | No | Due date (default: 30 days from request) |

**Response**:
```json
{
  "success": true,
  "data": {
    "dsar_id": "dsar_123",
    "request_type": "access",
    "status": "processing",
    "created_at": "2024-01-15T16:00:00Z",
    "due_date": "2024-02-14T00:00:00Z",
    "estimated_completion": "2024-01-20T00:00:00Z"
  },
  "message": "DSAR created and processing initiated"
}
```

---

### Get Policy Compliance Status

Retrieves compliance status for security policies.

**Endpoint**: `GET /api/compliance/policies/status`

**Authentication**: Required (Admin/Compliance Officer)

**Response**:
```json
{
  "success": true,
  "data": {
    "overall_compliance": 97.5,
    "policies": [
      {
        "policy_id": "policy_001",
        "policy_name": "Password Policy",
        "category": "access_control",
        "status": "compliant",
        "compliance_rate": 99.5,
        "last_review": "2024-01-01T00:00:00Z",
        "next_review": "2024-07-01T00:00:00Z",
        "violations": {
          "total": 2,
          "critical": 0,
          "high": 0,
          "medium": 2,
          "low": 0
        },
        "affected_users": 2,
        "enforcement_status": "automated"
      },
      {
        "policy_id": "policy_002",
        "policy_name": "Data Retention Policy",
        "category": "data_protection",
        "status": "partial_compliance",
        "compliance_rate": 92.0,
        "last_review": "2024-01-10T00:00:00Z",
        "next_review": "2024-07-10T00:00:00Z",
        "violations": {
          "total": 15,
          "critical": 0,
          "high": 3,
          "medium": 8,
          "low": 4
        },
        "affected_records": 150,
        "enforcement_status": "manual"
      }
    ],
    "compliance_trends": {
      "last_30_days": 97.5,
      "last_90_days": 96.8,
      "last_year": 95.5
    }
  }
}
```

---

## Compliance Frameworks

### GDPR (General Data Protection Regulation)

**Key Areas**:
- Data protection by design
- User consent management
- Data subject rights (access, deletion, portability)
- Breach notification (72 hours)
- Data processing agreements
- Privacy impact assessments

**Automated Checks**:
- User data retention compliance
- Consent documentation
- Data encryption status
- Access logging
- Breach detection and notification

### SOC 2 (Service Organization Control)

**Trust Service Criteria**:
- Security
- Availability
- Processing integrity
- Confidentiality
- Privacy

**Automated Monitoring**:
- Security control effectiveness
- System availability metrics
- Change management compliance
- Access control reviews
- Incident response procedures

### ISO 27001

**Control Categories**:
- Information security policies
- Organization of information security
- Human resource security
- Asset management
- Access control
- Cryptography
- Physical security
- Operations security
- Communications security
- System acquisition and development
- Supplier relationships
- Incident management
- Business continuity
- Compliance

### HIPAA (Health Insurance Portability and Accountability Act)

**Key Requirements**:
- PHI encryption
- Access controls and audit logs
- Business associate agreements
- Risk assessments
- Breach notification
- Employee training

## Best Practices

### Audit Logging
1. Log all access to sensitive data
2. Include user, timestamp, action, resource
3. Retain logs for minimum 7 years
4. Protect logs from tampering
5. Regular log review and analysis

### Risk Management
1. Conduct risk assessments quarterly
2. Prioritize by likelihood and impact
3. Document risk treatment decisions
4. Review and update regularly
5. Maintain risk register

### Training and Awareness
1. Annual security training for all staff
2. Role-specific compliance training
3. Track completion rates
4. Test knowledge retention
5. Update training content regularly

### Incident Response
1. Document all security incidents
2. Classify by severity
3. Define response procedures
4. Conduct post-incident reviews
5. Implement lessons learned

### Policy Management
1. Review policies annually
2. Update based on regulatory changes
3. Communicate changes to staff
4. Track acknowledgments
5. Monitor compliance

## Error Codes

| Code | Description |
|------|-------------|
| `COMPLIANCE_CHECK_FAILED` | Compliance check encountered an error |
| `FRAMEWORK_NOT_SUPPORTED` | Specified compliance framework not supported |
| `INSUFFICIENT_DATA` | Insufficient data to perform compliance check |
| `AUDIT_LOG_NOT_FOUND` | Specified audit log entry not found |
| `RISK_ASSESSMENT_INVALID` | Invalid risk assessment data |
| `TRAINING_RECORD_EXISTS` | Training record already exists for user/date |
| `DSAR_REQUEST_INVALID` | Invalid data subject access request |
| `REPORT_GENERATION_FAILED` | Compliance report generation failed |
| `UNAUTHORIZED_ACCESS` | User lacks compliance officer permissions |

## Webhooks

Subscribe to compliance events:

### Webhook Events
- `compliance.check.completed` - Compliance check finished
- `compliance.violation.detected` - Policy violation detected
- `incident.created` - New security incident reported
- `risk.critical.identified` - Critical risk identified
- `audit.threshold.exceeded` - Audit threshold exceeded
- `training.overdue` - Training completion overdue

### Webhook Payload Example
```json
{
  "event": "compliance.violation.detected",
  "timestamp": "2024-01-15T16:00:00Z",
  "data": {
    "violation_id": "viol_123",
    "policy": "Password Policy",
    "severity": "medium",
    "user_id": "user_789",
    "description": "Password not rotated within 90 days"
  }
}
```

## See Also

- [Security API](./security-api.md) - Security operations
- [Audit API](./admin-api.md) - System administration
- [Backup API](./backup-api.md) - Backup and recovery
- [Chapter 11: Security Policy](../chapter-11/security-policy.md) - Security policies