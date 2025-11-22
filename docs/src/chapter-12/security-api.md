# Security API

## Overview

The Security API provides comprehensive security management capabilities for BotServer, including vulnerability scanning, threat detection, security monitoring, access control management, and security configuration. It enables automated security operations and real-time threat response.

## Endpoints

### Get Security Dashboard

Retrieves an overview of current security posture and recent security events.

**Endpoint**: `GET /api/security/dashboard`

**Authentication**: Required (Admin/Security Officer)

**Response**:
```json
{
  "success": true,
  "data": {
    "security_score": 92.5,
    "last_updated": "2024-01-15T16:00:00Z",
    "threat_level": "low",
    "metrics": {
      "active_threats": 0,
      "blocked_attempts_24h": 127,
      "vulnerabilities": {
        "critical": 0,
        "high": 2,
        "medium": 8,
        "low": 15
      },
      "patch_compliance": 95.5,
      "failed_login_attempts": 23,
      "suspicious_activities": 5
    },
    "recent_events": [
      {
        "id": "event_123",
        "type": "blocked_access",
        "severity": "medium",
        "timestamp": "2024-01-15T15:45:00Z",
        "description": "Multiple failed login attempts blocked",
        "source_ip": "203.0.113.45",
        "action_taken": "IP temporarily blocked"
      }
    ],
    "system_health": {
      "firewall": "active",
      "ids_ips": "active",
      "antivirus": "active",
      "encryption": "enabled",
      "backup": "healthy",
      "monitoring": "active"
    },
    "alerts": [
      {
        "id": "alert_456",
        "priority": "high",
        "message": "2 high-severity vulnerabilities require attention",
        "action_required": "Review and remediate vulnerabilities",
        "due_date": "2024-01-22T00:00:00Z"
      }
    ]
  }
}
```

**Example**:
```bash
curl -X GET "http://localhost:3000/api/security/dashboard" \
  -H "Authorization: Bearer YOUR_TOKEN"
```

---

### Scan for Vulnerabilities

Initiates a vulnerability scan on specified systems or applications.

**Endpoint**: `POST /api/security/vulnerabilities/scan`

**Authentication**: Required (Admin/Security Officer)

**Request Body**:
```json
{
  "scan_type": "comprehensive",
  "targets": {
    "systems": ["web_server", "database", "api_server"],
    "applications": ["botserver", "web_client"],
    "networks": ["production", "staging"]
  },
  "scan_options": {
    "include_authenticated": true,
    "depth": "thorough",
    "check_exploitability": true,
    "include_cve_details": true
  },
  "schedule": "immediate",
  "notification": {
    "email": ["security@example.com"],
    "webhook": "https://monitoring.example.com/webhook"
  }
}
```

**Parameters**:
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `scan_type` | string | Yes | Type: `quick`, `standard`, `comprehensive`, `compliance` |
| `targets` | object | Yes | Systems, applications, or networks to scan |
| `scan_options` | object | No | Scanning options and depth |
| `schedule` | string | No | Schedule: `immediate`, `scheduled` (default: immediate) |
| `scheduled_time` | string | No | ISO 8601 timestamp if scheduled |
| `notification` | object | No | Notification settings |

**Response**:
```json
{
  "success": true,
  "data": {
    "scan_id": "scan_789",
    "status": "initiated",
    "started_at": "2024-01-15T16:00:00Z",
    "estimated_duration_minutes": 45,
    "targets_count": 5,
    "progress_url": "/api/security/vulnerabilities/scan/scan_789/status"
  },
  "message": "Vulnerability scan initiated"
}
```

**Example**:
```bash
curl -X POST "http://localhost:3000/api/security/vulnerabilities/scan" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "scan_type": "comprehensive",
    "targets": {
      "systems": ["web_server", "database"]
    }
  }'
```

---

### Get Scan Status

Retrieves the status of a running vulnerability scan.

**Endpoint**: `GET /api/security/vulnerabilities/scan/{scan_id}/status`

**Authentication**: Required (Admin/Security Officer)

**Response**:
```json
{
  "success": true,
  "data": {
    "scan_id": "scan_789",
    "status": "running",
    "progress_percent": 65,
    "started_at": "2024-01-15T16:00:00Z",
    "elapsed_minutes": 29,
    "estimated_remaining_minutes": 16,
    "current_target": "database_server",
    "targets_completed": 3,
    "targets_total": 5,
    "vulnerabilities_found": {
      "critical": 0,
      "high": 2,
      "medium": 5,
      "low": 8
    }
  }
}
```

---

### List Vulnerabilities

Lists all detected vulnerabilities with filtering options.

**Endpoint**: `GET /api/security/vulnerabilities`

**Authentication**: Required (Admin/Security Officer)

**Query Parameters**:
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `severity` | string | No | Filter by severity: `critical`, `high`, `medium`, `low` |
| `status` | string | No | Filter by status: `open`, `in_progress`, `resolved`, `false_positive`, `accepted` |
| `system` | string | No | Filter by system name |
| `cve_id` | string | No | Filter by CVE identifier |
| `exploitable` | boolean | No | Filter by exploitability |
| `page` | integer | No | Page number (default: 1) |
| `per_page` | integer | No | Results per page (default: 20, max: 100) |

**Response**:
```json
{
  "success": true,
  "data": {
    "vulnerabilities": [
      {
        "id": "vuln_123",
        "cve_id": "CVE-2024-12345",
        "title": "Remote Code Execution in Web Framework",
        "severity": "critical",
        "cvss_score": 9.8,
        "status": "open",
        "detected_at": "2024-01-15T16:30:00Z",
        "system": "web_server",
        "component": "Web Framework v2.1.0",
        "description": "A critical vulnerability allows remote code execution without authentication",
        "exploitable": true,
        "exploit_available": true,
        "affected_assets": [
          "production-web-01",
          "production-web-02"
        ],
        "remediation": {
          "summary": "Upgrade to Web Framework v2.1.5 or apply security patch",
          "steps": [
            "Download security patch from vendor",
            "Test patch in staging environment",
            "Apply patch during maintenance window",
            "Verify patch installation",
            "Restart web services"
          ],
          "estimated_effort_hours": 4,
          "patch_available": true,
          "patch_url": "https://vendor.example.com/patches/CVE-2024-12345"
        },
        "references": [
          "https://nvd.nist.gov/vuln/detail/CVE-2024-12345",
          "https://vendor.example.com/security-advisory/2024-001"
        ],
        "due_date": "2024-01-16T00:00:00Z",
        "assigned_to": "security_team"
      }
    ],
    "total": 25,
    "page": 1,
    "per_page": 20,
    "total_pages": 2,
    "summary": {
      "critical": 1,
      "high": 3,
      "medium": 8,
      "low": 13
    }
  }
}
```

**Example**:
```bash
curl -X GET "http://localhost:3000/api/security/vulnerabilities?severity=critical&status=open" \
  -H "Authorization: Bearer YOUR_TOKEN"
```

---

### Update Vulnerability Status

Updates the status of a vulnerability (e.g., mark as resolved, false positive).

**Endpoint**: `PATCH /api/security/vulnerabilities/{vuln_id}`

**Authentication**: Required (Admin/Security Officer)

**Request Body**:
```json
{
  "status": "resolved",
  "resolution": {
    "method": "patched",
    "description": "Applied vendor security patch v2.1.5",
    "verified": true,
    "verified_by": "security_team",
    "verification_date": "2024-01-16T14:00:00Z"
  },
  "notes": "Patch applied successfully to all production servers"
}
```

**Response**:
```json
{
  "success": true,
  "data": {
    "id": "vuln_123",
    "status": "resolved",
    "updated_at": "2024-01-16T14:00:00Z",
    "resolution_time_hours": 21.5
  },
  "message": "Vulnerability status updated successfully"
}
```

---

### Detect Threats

Runs threat detection analysis on systems, logs, or network traffic.

**Endpoint**: `POST /api/security/threats/detect`

**Authentication**: Required (Admin/Security Officer)

**Request Body**:
```json
{
  "detection_type": "anomaly",
  "scope": {
    "logs": true,
    "network_traffic": true,
    "user_behavior": true,
    "file_integrity": true
  },
  "time_range": {
    "start": "2024-01-15T00:00:00Z",
    "end": "2024-01-15T23:59:59Z"
  },
  "sensitivity": "high",
  "include_iocs": true
}
```

**Response**:
```json
{
  "success": true,
  "data": {
    "detection_id": "detect_456",
    "status": "completed",
    "started_at": "2024-01-15T16:00:00Z",
    "completed_at": "2024-01-15T16:15:00Z",
    "threats_detected": 3,
    "threats": [
      {
        "id": "threat_001",
        "type": "brute_force_attack",
        "severity": "high",
        "confidence": 95,
        "detected_at": "2024-01-15T14:23:00Z",
        "source": {
          "ip": "198.51.100.42",
          "country": "Unknown",
          "reputation_score": 15
        },
        "target": {
          "system": "authentication_service",
          "user": "admin"
        },
        "indicators": [
          "150 failed login attempts in 5 minutes",
          "Multiple user accounts targeted",
          "Known malicious IP address"
        ],
        "action_taken": "IP blocked automatically",
        "status": "mitigated",
        "recommended_actions": [
          "Review authentication logs",
          "Verify account security",
          "Consider additional rate limiting"
        ]
      }
    ]
  }
}
```

---

### List Active Threats

Retrieves currently active security threats.

**Endpoint**: `GET /api/security/threats/active`

**Authentication**: Required (Admin/Security Officer)

**Response**:
```json
{
  "success": true,
  "data": {
    "active_threats_count": 2,
    "threat_level": "medium",
    "threats": [
      {
        "id": "threat_002",
        "type": "suspicious_activity",
        "severity": "medium",
        "status": "active",
        "first_detected": "2024-01-15T15:30:00Z",
        "last_seen": "2024-01-15T16:20:00Z",
        "occurrences": 12,
        "description": "Unusual data access pattern detected",
        "affected_systems": ["file_server"],
        "affected_users": ["user_789"],
        "indicators": [
          "User accessing files outside normal hours",
          "Large volume of file downloads",
          "Access to sensitive directories"
        ],
        "risk_score": 65,
        "recommended_actions": [
          "Investigate user activity",
          "Review access permissions",
          "Contact user for verification"
        ]
      }
    ]
  }
}
```

---

### Block IP Address

Blocks an IP address from accessing the system.

**Endpoint**: `POST /api/security/firewall/block-ip`

**Authentication**: Required (Admin/Security Officer)

**Request Body**:
```json
{
  "ip_address": "203.0.113.45",
  "duration": "permanent",
  "reason": "Multiple failed login attempts",
  "scope": "all_services",
  "notify_admin": true
}
```

**Parameters**:
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `ip_address` | string | Yes | IP address or CIDR block to block |
| `duration` | string | No | Duration: `1h`, `24h`, `7d`, `permanent` (default: 24h) |
| `reason` | string | Yes | Reason for blocking |
| `scope` | string | No | Scope: `all_services`, `web`, `api`, `ssh` (default: all_services) |
| `notify_admin` | boolean | No | Send notification (default: true) |

**Response**:
```json
{
  "success": true,
  "data": {
    "rule_id": "fw_rule_789",
    "ip_address": "203.0.113.45",
    "status": "blocked",
    "created_at": "2024-01-15T16:30:00Z",
    "expires_at": "2024-01-16T16:30:00Z",
    "scope": "all_services"
  },
  "message": "IP address blocked successfully"
}
```

---

### Unblock IP Address

Removes an IP address from the block list.

**Endpoint**: `DELETE /api/security/firewall/block-ip/{ip_address}`

**Authentication**: Required (Admin/Security Officer)

**Response**:
```json
{
  "success": true,
  "data": {
    "ip_address": "203.0.113.45",
    "status": "unblocked",
    "unblocked_at": "2024-01-15T17:00:00Z"
  },
  "message": "IP address unblocked successfully"
}
```

---

### List Blocked IPs

Retrieves list of blocked IP addresses.

**Endpoint**: `GET /api/security/firewall/blocked-ips`

**Authentication**: Required (Admin/Security Officer)

**Response**:
```json
{
  "success": true,
  "data": {
    "blocked_ips": [
      {
        "ip_address": "203.0.113.45",
        "reason": "Multiple failed login attempts",
        "blocked_at": "2024-01-15T16:30:00Z",
        "expires_at": "2024-01-16T16:30:00Z",
        "scope": "all_services",
        "blocked_attempts": 156
      }
    ],
    "total": 15
  }
}
```

---

### Check Access Permissions

Checks if a user has specific permissions for a resource.

**Endpoint**: `POST /api/security/access/check`

**Authentication**: Required (Admin)

**Request Body**:
```json
{
  "user_id": "user_789",
  "resource_type": "file",
  "resource_id": "file_123",
  "action": "read",
  "context": {
    "ip_address": "192.168.1.100",
    "time": "2024-01-15T16:30:00Z"
  }
}
```

**Response**:
```json
{
  "success": true,
  "data": {
    "allowed": true,
    "reason": "User has direct read permission",
    "permissions": ["read", "write"],
    "granted_by": "group_membership",
    "group": "engineering_team",
    "conditions": [],
    "audit_logged": true
  }
}
```

---

### Review User Access

Retrieves comprehensive access review for a user.

**Endpoint**: `GET /api/security/access/review/{user_id}`

**Authentication**: Required (Admin/Manager)

**Response**:
```json
{
  "success": true,
  "data": {
    "user_id": "user_789",
    "username": "john.doe",
    "email": "john.doe@example.com",
    "department": "Engineering",
    "last_access_review": "2024-01-01T00:00:00Z",
    "next_access_review": "2024-04-01T00:00:00Z",
    "account_status": "active",
    "privileged_access": true,
    "mfa_enabled": true,
    "last_login": "2024-01-15T09:00:00Z",
    "permissions": {
      "direct": [
        {
          "resource_type": "system",
          "resource_name": "production_database",
          "permissions": ["read", "write"],
          "granted_date": "2023-06-01T00:00:00Z",
          "granted_by": "admin_user",
          "justification": "Database administrator role"
        }
      ],
      "group_inherited": [
        {
          "group": "engineering_team",
          "resource_type": "repository",
          "resource_name": "botserver",
          "permissions": ["read", "write", "deploy"],
          "granted_date": "2023-01-15T00:00:00Z"
        }
      ],
      "role_based": [
        {
          "role": "developer",
          "permissions": ["code_review", "deploy_staging"],
          "granted_date": "2023-01-15T00:00:00Z"
        }
      ]
    },
    "recent_activity": {
      "login_count_30d": 62,
      "failed_login_count_30d": 2,
      "files_accessed_30d": 145,
      "privileged_actions_30d": 23
    },
    "anomalies": [],
    "recommendations": [
      "Access appears appropriate for role",
      "Consider removing write access to production database if no longer needed"
    ]
  }
}
```

---

### Revoke User Access

Revokes all or specific access for a user.

**Endpoint**: `POST /api/security/access/revoke`

**Authentication**: Required (Admin)

**Request Body**:
```json
{
  "user_id": "user_789",
  "revoke_type": "specific",
  "resources": [
    {
      "resource_type": "system",
      "resource_id": "production_database"
    }
  ],
  "reason": "User changed roles",
  "effective_date": "2024-01-16T00:00:00Z",
  "notify_user": true
}
```

**Response**:
```json
{
  "success": true,
  "data": {
    "revocation_id": "revoke_456",
    "user_id": "user_789",
    "resources_affected": 1,
    "effective_date": "2024-01-16T00:00:00Z",
    "status": "completed"
  },
  "message": "Access revoked successfully"
}
```

---

### Get Security Logs

Retrieves security-specific logs with advanced filtering.

**Endpoint**: `GET /api/security/logs`

**Authentication**: Required (Admin/Security Officer)

**Query Parameters**:
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `log_type` | string | No | Type: `authentication`, `authorization`, `firewall`, `intrusion`, `data_access` |
| `severity` | string | No | Filter by severity: `info`, `warning`, `error`, `critical` |
| `user_id` | string | No | Filter by user ID |
| `ip_address` | string | No | Filter by IP address |
| `start_time` | string | No | Start timestamp (ISO 8601) |
| `end_time` | string | No | End timestamp (ISO 8601) |
| `search` | string | No | Search in log messages |
| `page` | integer | No | Page number (default: 1) |
| `per_page` | integer | No | Results per page (default: 100, max: 1000) |

**Response**:
```json
{
  "success": true,
  "data": {
    "logs": [
      {
        "id": "log_12345",
        "timestamp": "2024-01-15T16:30:15.123Z",
        "log_type": "authentication",
        "severity": "warning",
        "event": "failed_login",
        "user_id": "user_789",
        "username": "john.doe",
        "ip_address": "192.168.1.100",
        "user_agent": "Mozilla/5.0...",
        "message": "Failed login attempt - incorrect password",
        "metadata": {
          "attempt_count": 3,
          "lockout_remaining": 2,
          "geolocation": "US, California, San Francisco"
        }
      }
    ],
    "total": 5234,
    "page": 1,
    "per_page": 100,
    "total_pages": 53
  }
}
```

---

### Configure Security Settings

Updates security configuration settings.

**Endpoint**: `PUT /api/security/settings`

**Authentication**: Required (Admin)

**Request Body**:
```json
{
  "password_policy": {
    "min_length": 12,
    "require_uppercase": true,
    "require_lowercase": true,
    "require_numbers": true,
    "require_special_chars": true,
    "max_age_days": 90,
    "history_count": 12,
    "lockout_threshold": 5,
    "lockout_duration_minutes": 30
  },
  "session_settings": {
    "max_session_duration_hours": 8,
    "idle_timeout_minutes": 30,
    "require_mfa": true,
    "allow_concurrent_sessions": false
  },
  "access_control": {
    "require_approval_for_privileged_access": true,
    "auto_revoke_inactive_days": 90,
    "enforce_least_privilege": true
  },
  "threat_detection": {
    "enabled": true,
    "sensitivity": "high",
    "auto_block_suspicious_ips": true,
    "alert_on_anomalies": true
  },
  "encryption": {
    "enforce_tls_1_3": true,
    "require_encryption_at_rest": true,
    "encryption_algorithm": "AES-256-GCM"
  }
}
```

**Response**:
```json
{
  "success": true,
  "data": {
    "settings_updated": 15,
    "effective_date": "2024-01-15T16:45:00Z",
    "restart_required": false
  },
  "message": "Security settings updated successfully"
}
```

---

### Get Security Settings

Retrieves current security configuration.

**Endpoint**: `GET /api/security/settings`

**Authentication**: Required (Admin/Security Officer)

**Response**: Same structure as configure endpoint

---

### Perform Security Audit

Initiates a comprehensive security audit.

**Endpoint**: `POST /api/security/audit/run`

**Authentication**: Required (Admin)

**Request Body**:
```json
{
  "audit_type": "comprehensive",
  "scope": [
    "access_controls",
    "configurations",
    "vulnerabilities",
    "compliance",
    "policies"
  ],
  "generate_report": true,
  "remediation_plan": true
}
```

**Response**:
```json
{
  "success": true,
  "data": {
    "audit_id": "audit_789",
    "status": "running",
    "started_at": "2024-01-15T17:00:00Z",
    "estimated_duration_minutes": 60,
    "progress_url": "/api/security/audit/audit_789/status"
  },
  "message": "Security audit initiated"
}
```

---

### Get Audit Results

Retrieves results from a completed security audit.

**Endpoint**: `GET /api/security/audit/{audit_id}/results`

**Authentication**: Required (Admin/Security Officer)

**Response**:
```json
{
  "success": true,
  "data": {
    "audit_id": "audit_789",
    "audit_type": "comprehensive",
    "status": "completed",
    "started_at": "2024-01-15T17:00:00Z",
    "completed_at": "2024-01-15T18:00:00Z",
    "overall_score": 87.5,
    "results": {
      "access_controls": {
        "score": 92,
        "status": "good",
        "findings": [
          {
            "severity": "medium",
            "title": "3 users with excessive permissions",
            "description": "3 users have admin access but don't require it for their roles",
            "recommendation": "Review and reduce privileges"
          }
        ]
      },
      "configurations": {
        "score": 88,
        "status": "acceptable",
        "findings": [
          {
            "severity": "low",
            "title": "TLS 1.2 still enabled",
            "description": "TLS 1.2 is enabled alongside TLS 1.3",
            "recommendation": "Consider disabling TLS 1.2 for enhanced security"
          }
        ]
      },
      "vulnerabilities": {
        "score": 85,
        "status": "acceptable",
        "critical": 0,
        "high": 2,
        "medium": 5,
        "low": 12
      }
    },
    "summary": {
      "total_findings": 23,
      "critical": 0,
      "high": 2,
      "medium": 8,
      "low": 13
    },
    "recommendations": [
      "Address high-severity vulnerabilities within 7 days",
      "Review user access permissions",
      "Update security configurations",
      "Enhance monitoring for suspicious activities"
    ],
    "report_url": "/api/security/reports/audit_report_789.pdf"
  }
}
```

---

## Security Monitoring

### Real-time Monitoring

BotServer provides real-time security monitoring through:

1. **Authentication Monitoring**: Track login attempts, failures, and suspicious patterns
2. **Access Monitoring**: Monitor data access and privilege usage
3. **Network Monitoring**: Detect unusual network traffic patterns
4. **File Integrity Monitoring**: Track unauthorized file modifications
5. **Configuration Monitoring**: Detect configuration changes

### Alert Thresholds

Default alert thresholds:
- Failed logins: 5 attempts in 5 minutes
- Privilege escalation: Any attempt
- Data exfiltration: >100MB in 1 hour
- Suspicious IPs: Access from blacklisted IPs
- Configuration changes: Any unauthorized change

## Security Controls

### Preventive Controls
- Strong password policy
- Multi-factor authentication
- Access control lists
- Firewall rules
- Encryption (at rest and in transit)

### Detective Controls
- Intrusion detection system
- Log monitoring and analysis
- Vulnerability scanning
- File integrity monitoring
- Anomaly detection

### Corrective Controls
- Automated IP blocking
- Account lockout
- Incident response procedures
- Backup and recovery
- Patch management

## Best Practices

### Vulnerability Management
1. Run weekly vulnerability scans
2. Prioritize by severity and exploitability
3. Patch critical vulnerabilities within 24 hours
4. Track remediation progress
5. Verify fixes after patching

### Threat Detection
1. Enable real-time monitoring
2. Configure appropriate alert thresholds
3. Investigate all high-severity alerts
4. Maintain threat intelligence feeds
5. Regular review of security logs

### Access Control
1. Implement least privilege principle
2. Review access quarterly
3. Enable MFA for all privileged accounts
4. Monitor privileged access usage
5. Revoke access promptly when no longer needed

### Security Configuration
1. Follow security hardening guides
2. Disable unnecessary services
3. Use strong encryption
4. Regular configuration audits
5. Document all security settings

## Error Codes

| Code | Description |
|------|-------------|
| `SCAN_IN_PROGRESS` | Vulnerability scan already running |
| `INVALID_IP_ADDRESS` | Invalid IP address format |
| `PERMISSION_DENIED` | Insufficient permissions for operation |
| `VULNERABILITY_NOT_FOUND` | Specified vulnerability does not exist |
| `THREAT_DETECTION_FAILED` | Threat detection encountered an error |
| `AUDIT_IN_PROGRESS` | Security audit already running |
| `INVALID_SECURITY_SETTING` | Invalid security configuration value |
| `IP_ALREADY_BLOCKED` | IP address is already blocked |
| `FIREWALL_RULE_CONFLICT` | Firewall rule conflicts with existing rule |

## Webhooks

Subscribe to security events:

### Webhook Events
- `security.vulnerability.detected` - New vulnerability found
- `security.threat.detected` - Security threat detected
- `security.attack.blocked` - Attack attempt blocked
- `security.breach.suspected` - Potential breach detected
- `security.audit.completed` - Security audit finished
- `security.critical.alert` - Critical security alert

### Webhook Payload Example
```json
{
  "event": "security.threat.detected",
  "timestamp": "2024-01-15T16:30:00Z",
  "data": {
    "threat_id": "threat_001",
    "type": "brute_force_attack",
    "severity": "high",
    "source_ip": "198.51.100.42",
    "target": "authentication_service",
    "action_taken": "IP blocked"
  }
}
```

## See Also

- [Compliance API](./compliance-api.md) - Compliance operations
- [Backup API](./backup-api.md) - Backup and recovery
- [Monitoring API](./monitoring-api.md) - System monitoring
- [Chapter 11: Security Features](../chapter-11/security-features.md) - Security configuration