# SOC 2 Type II Compliance

This guide covers General Bots' SOC 2 Type II compliance implementation, including security controls, audit logging, evidence collection, and compliance reporting.

## Overview

SOC 2 Type II is a security framework developed by the American Institute of CPAs (AICPA) that evaluates how organizations manage customer data based on five Trust Service Criteria:

1. **Security** - Protection against unauthorized access
2. **Availability** - System accessibility as agreed
3. **Processing Integrity** - Accurate and timely processing
4. **Confidentiality** - Protection of confidential information
5. **Privacy** - Personal information handling

General Bots implements controls across all five criteria to ensure enterprise-grade security.

## Trust Service Criteria Implementation

### Security (Common Criteria)

| Control | Implementation | Status |
|---------|---------------|--------|
| CC1.1 - Integrity & Ethics | Code of conduct, security policies | ✅ |
| CC2.1 - Communication | Security awareness training | ✅ |
| CC3.1 - Risk Assessment | Quarterly risk assessments | ✅ |
| CC4.1 - Monitoring | Continuous security monitoring | ✅ |
| CC5.1 - Control Activities | Access controls, encryption | ✅ |
| CC6.1 - Logical Access | RBAC, MFA, session management | ✅ |
| CC7.1 - System Operations | Change management, incident response | ✅ |
| CC8.1 - Change Management | Documented change procedures | ✅ |
| CC9.1 - Risk Mitigation | Vendor management, BCP | ✅ |

### Availability

| Control | Implementation |
|---------|---------------|
| A1.1 - Capacity Management | Auto-scaling, resource monitoring |
| A1.2 - Recovery Operations | Automated backups, disaster recovery |
| A1.3 - Recovery Testing | Quarterly DR tests |

### Processing Integrity

| Control | Implementation |
|---------|---------------|
| PI1.1 - Processing Accuracy | Input validation, data integrity checks |
| PI1.2 - Processing Completeness | Transaction logging, audit trails |
| PI1.3 - Processing Timeliness | SLA monitoring, performance metrics |

### Confidentiality

| Control | Implementation |
|---------|---------------|
| C1.1 - Confidential Information | Data classification, encryption at rest |
| C1.2 - Disposal | Secure deletion, data retention policies |

### Privacy

| Control | Implementation |
|---------|---------------|
| P1.1 - Notice | Privacy policy, cookie consent |
| P2.1 - Choice and Consent | Opt-in/opt-out mechanisms |
| P3.1 - Collection | Data minimization |
| P4.1 - Use and Retention | Purpose limitation, retention schedules |
| P5.1 - Access | Data export (GDPR Article 15) |
| P6.1 - Disclosure | Third-party data sharing controls |
| P7.1 - Quality | Data accuracy verification |
| P8.1 - Monitoring | Privacy impact assessments |

## Audit Logging

### Event Categories

General Bots logs the following security-relevant events:

| Category | Events Logged |
|----------|--------------|
| Authentication | Login, logout, MFA events, password changes |
| Authorization | Permission grants, role assignments, access denials |
| Data Access | Read operations on sensitive data |
| Data Modification | Create, update, delete operations |
| Administrative | Configuration changes, user management |
| Security | Failed auth attempts, suspicious activity |

### Log Structure

```json
{
  "id": "uuid",
  "timestamp": "2025-01-21T10:30:00Z",
  "organization_id": "org-uuid",
  "actor_id": "user-uuid",
  "actor_email": "user@company.com",
  "actor_ip": "192.168.1.100",
  "action": "role_assign",
  "resource_type": "role",
  "resource_id": "role-uuid",
  "resource_name": "admin",
  "details": {
    "description": "Assigned role 'admin' to user",
    "before_state": null,
    "after_state": {"role": "admin"},
    "changes": [{"field": "role", "old_value": null, "new_value": "admin"}]
  },
  "result": "success",
  "metadata": {}
}
```

### Log Retention

| Log Type | Retention Period | Storage |
|----------|-----------------|---------|
| Security Events | 7 years | Immutable storage |
| Access Logs | 2 years | Standard storage |
| Application Logs | 90 days | Standard storage |
| Debug Logs | 30 days | Ephemeral storage |

### Accessing Audit Logs

```http
GET /api/compliance/audit-logs
Authorization: Bearer <token>
```

Query parameters:

| Parameter | Description |
|-----------|-------------|
| `organization_id` | Filter by organization |
| `actor_id` | Filter by user |
| `action` | Filter by action type |
| `resource_type` | Filter by resource type |
| `start_date` | Start of date range |
| `end_date` | End of date range |
| `page` | Page number |
| `per_page` | Results per page |

## Security Controls

### Access Control

**Multi-Factor Authentication (MFA)**

- TOTP-based authentication
- Hardware security key support (FIDO2/WebAuthn)
- SMS backup codes (optional)

**Session Management**

- Configurable session timeout (default: 8 hours)
- Concurrent session limits
- Session invalidation on password change
- IP-based session binding (optional)

**Password Policy**

- Minimum 12 characters
- Complexity requirements
- Password history (last 10)
- Account lockout after 5 failed attempts

### Encryption

**Data at Rest**

- AES-256 encryption for all stored data
- Encrypted database columns for PII
- Encrypted file storage (MinIO with server-side encryption)

**Data in Transit**

- TLS 1.3 for all connections
- Perfect Forward Secrecy
- HSTS with preloading
- Certificate pinning (mobile apps)

### Network Security

- Web Application Firewall (WAF)
- DDoS protection
- Rate limiting per endpoint
- IP allowlisting (enterprise)

## Compliance Reporting

### Generating Compliance Reports

```http
POST /api/compliance/reports
Authorization: Bearer <token>
Content-Type: application/json

{
  "report_type": "soc2",
  "period_start": "2025-01-01",
  "period_end": "2025-03-31",
  "criteria": ["security", "availability", "confidentiality"]
}
```

### Report Types

| Type | Description | Frequency |
|------|-------------|-----------|
| `soc2` | Full SOC 2 compliance report | Quarterly |
| `access_review` | User access review | Monthly |
| `vulnerability` | Vulnerability assessment | Weekly |
| `incident` | Security incident report | As needed |

### Evidence Collection

The compliance module automatically collects evidence for audit:

**User Access Evidence**

- Current user list with roles
- Permission assignment history
- Access review sign-offs

**Change Management Evidence**

- Deployment logs
- Configuration change records
- Approval workflows

**Security Evidence**

- Vulnerability scan results
- Penetration test reports
- Security training completion

### Exporting Evidence

```http
GET /api/compliance/evidence/export
Authorization: Bearer <token>
```

Query parameters:

| Parameter | Description |
|-----------|-------------|
| `criteria` | SOC 2 criteria (CC6.1, A1.1, etc.) |
| `period_start` | Evidence period start |
| `period_end` | Evidence period end |
| `format` | Export format (json, csv, pdf) |

## Incident Response

### Incident Classification

| Severity | Description | Response Time |
|----------|-------------|---------------|
| Critical | Data breach, system compromise | 15 minutes |
| High | Service outage, failed controls | 1 hour |
| Medium | Suspicious activity, minor issues | 4 hours |
| Low | Informational, potential risk | 24 hours |

### Incident Response Process

1. **Detection** - Automated monitoring or manual report
2. **Triage** - Classify severity, assign responder
3. **Containment** - Isolate affected systems
4. **Eradication** - Remove threat
5. **Recovery** - Restore services
6. **Lessons Learned** - Post-incident review

### Incident Logging

```http
POST /api/compliance/incidents
Authorization: Bearer <token>
Content-Type: application/json

{
  "title": "Suspicious login activity detected",
  "severity": "medium",
  "description": "Multiple failed login attempts from unusual location",
  "affected_systems": ["authentication"],
  "detected_at": "2025-01-21T10:00:00Z",
  "detected_by": "automated_monitoring"
}
```

## Vendor Management

### Third-Party Risk Assessment

All vendors handling customer data undergo:

- Security questionnaire
- SOC 2 report review (if available)
- Contract review for security requirements
- Annual reassessment

### Key Vendors

| Vendor | Service | SOC 2 | Data Access |
|--------|---------|-------|-------------|
| PostgreSQL | Database | N/A (self-hosted) | Full |
| MinIO | Object Storage | N/A (self-hosted) | Full |
| Qdrant | Vector DB | N/A (self-hosted) | Full |
| Redis | Caching | N/A (self-hosted) | Session data |

## Business Continuity

### Recovery Objectives

| Metric | Target | Current |
|--------|--------|---------|
| RTO (Recovery Time Objective) | 4 hours | 2 hours |
| RPO (Recovery Point Objective) | 1 hour | 15 minutes |
| MTTR (Mean Time to Recovery) | 2 hours | 45 minutes |

### Backup Strategy

| Data Type | Frequency | Retention | Location |
|-----------|-----------|-----------|----------|
| Database | Every 15 minutes | 30 days | Off-site |
| Files | Hourly | 90 days | Off-site |
| Configuration | On change | Forever | Git |
| Logs | Daily | Per retention policy | Off-site |

### Disaster Recovery

- Multi-region deployment capability
- Automated failover
- Quarterly DR testing
- Documented recovery procedures

## Configuration

### Enabling SOC 2 Features

Add to your `.env`:

```bash
SOC2_COMPLIANCE_ENABLED=true
SOC2_AUDIT_LOG_RETENTION_DAYS=2555
SOC2_EVIDENCE_COLLECTION=true
SOC2_INCIDENT_AUTO_CREATE=true
```

### Compliance Dashboard

Access the compliance dashboard at:

```
/admin/compliance
```

Features:

- Real-time compliance status
- Control effectiveness metrics
- Open findings and remediation
- Upcoming audit timeline

## API Reference

### Get Compliance Status

```http
GET /api/compliance/status
Authorization: Bearer <token>
```

Response:

```json
{
  "overall_status": "compliant",
  "last_assessment": "2025-01-15T00:00:00Z",
  "criteria": {
    "security": {"status": "compliant", "controls_passed": 45, "controls_total": 45},
    "availability": {"status": "compliant", "controls_passed": 12, "controls_total": 12},
    "confidentiality": {"status": "compliant", "controls_passed": 8, "controls_total": 8}
  },
  "open_findings": 0,
  "next_audit": "2025-04-01"
}
```

### List Control Evidence

```http
GET /api/compliance/controls/{control_id}/evidence
Authorization: Bearer <token>
```

### Create Finding

```http
POST /api/compliance/findings
Authorization: Bearer <token>
Content-Type: application/json

{
  "control_id": "CC6.1",
  "title": "MFA not enforced for admin accounts",
  "severity": "high",
  "description": "Admin accounts can bypass MFA requirement",
  "remediation_plan": "Update policy to require MFA for all admin roles",
  "due_date": "2025-02-01"
}
```

## Best Practices

### For Administrators

1. **Enable all logging** - Ensure comprehensive audit trails
2. **Regular access reviews** - Monthly review of user permissions
3. **Monitor dashboards** - Daily check of compliance status
4. **Document exceptions** - Record all policy exceptions with justification
5. **Test controls** - Quarterly verification of control effectiveness

### For Developers

1. **Follow secure coding standards** - No hardcoded secrets, input validation
2. **Use security modules** - SafeCommand, sql_guard, error_sanitizer
3. **Log security events** - Use audit logging for sensitive operations
4. **Handle errors properly** - Never expose internal details

### For Organizations

1. **Assign compliance owner** - Dedicated person for SOC 2
2. **Schedule regular audits** - Annual Type II assessment
3. **Train employees** - Security awareness program
4. **Maintain documentation** - Keep policies current
5. **Plan for incidents** - Test incident response procedures

## Related Topics

- [RBAC Configuration](./rbac-configuration.md)
- [Audit Logging](./audit-logging.md)
- [Security Matrix](./security-matrix.md)
- [Privacy & GDPR](../09-security/compliance-requirements.md)