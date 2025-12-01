# Compliance - Security Scanner

> **Your privacy and security guardian**

<img src="../../assets/suite/compliance-screen.svg" alt="Compliance Interface Screen" style="max-width: 100%; height: auto;">

---

## Overview

Compliance is the security and privacy management app in General Bots Suite. Monitor data handling, manage consent, respond to data subject requests, and ensure your bots comply with regulations like LGPD, GDPR, and CCPA. Compliance helps you protect user data and maintain trust.

---

## Features

### Compliance Dashboard

The dashboard gives you an at-a-glance view of your compliance status:

| Metric | Description |
|--------|-------------|
| **Overall Score** | Percentage score with color indicator |
| **Open Requests** | Pending data subject requests |
| **Data Breaches** | Count in last 90 days |
| **Consent Rate** | Percentage of users with active consent |

**Score Breakdown by Area:**
- Data Protection
- Consent Management
- Access Controls
- Data Retention
- Breach Response
- Documentation

**Score Meanings:**

| Score | Status | Action Needed |
|-------|--------|---------------|
| 90-100% | ✓ Excellent | Maintain current practices |
| 70-89% | ⚠ Good | Address minor issues |
| 50-69% | ⚠ Fair | Prioritize improvements |
| Below 50% | ✗ Poor | Immediate action required |

---

### Security Scanner

Automatically scan your bots and data for compliance issues.

#### Running a Scan

1. Click **Scan Now** in the top right
2. Select scan type:
   - **Quick** - Basic checks (5 minutes)
   - **Full** - Complete audit (30 minutes)
   - **Custom** - Select specific areas
3. Choose scan targets:
   - All bots
   - Knowledge bases
   - User data
   - Conversation logs
   - External integrations
4. Click **Start Scan**

#### Scan Results

Results are categorized by severity:

| Severity | Icon | Description |
|----------|------|-------------|
| **Critical** | ✗ | Requires immediate attention |
| **Warning** | ⚠ | Should be addressed soon |
| **Passed** | ✓ | No issues found |

**Common Issues Found:**
- Unencrypted PII in logs
- Consent records needing renewal
- Missing retention policies
- Missing privacy policy links

---

### Data Subject Requests (DSR)

Handle user requests for their data rights.

#### Request Types

| Type | Icon | Description | Deadline |
|------|------|-------------|----------|
| **Data Access** | 📥 | User wants copy of their data | 15-30 days |
| **Data Deletion** | 🗑️ | User wants data erased | 15-30 days |
| **Data Portability** | 📤 | User wants data in machine format | 15-30 days |
| **Rectification** | ✏️ | User wants to correct data | 15-30 days |
| **Processing Objection** | ✋ | User objects to data processing | Immediate |
| **Consent Withdrawal** | 🚫 | User withdraws consent | Immediate |

#### Processing a Request

1. Verify user identity
2. Review data found:
   - User Profile
   - Conversation History
   - Consent Records
   - Activity Logs
3. Generate data package (for access requests)
4. Send to user or complete deletion
5. Mark request as complete

---

### Consent Management

Track and manage user consent.

**Consent Types:**

| Type | Required | Description |
|------|----------|-------------|
| **Terms of Service** | Yes | Agreement to terms and conditions |
| **Marketing** | No | Promotional communications |
| **Analytics** | No | Usage data collection |
| **Third-Party Sharing** | No | Sharing with partners |

**Consent Record Information:**
- User ID and email
- Consent status (given/denied/withdrawn)
- Timestamp
- Collection method (web, chat, email)
- IP address and browser info

---

### Data Mapping

See where personal data is stored:

| Category | Data Types | Storage Locations | Retention |
|----------|------------|-------------------|-----------|
| **Personal Identifiers** | Names, emails, phones | Users table, conversation logs | 3 years |
| **Communication Data** | Messages, attachments | Conversation logs, MinIO, Qdrant | 1 year |
| **Behavioral Data** | Page views, clicks | Analytics events, preferences | 90 days |

---

### Policy Management

Manage your compliance policies:

**Policy Types:**
- Privacy Policy
- Data Retention Policy
- Cookie Policy

**Data Retention Rules:**

| Data Type | Retention | Action |
|-----------|-----------|--------|
| Conversation logs | 1 year | Auto-delete |
| User profiles | 3 years | Anonymize |
| Analytics data | 90 days | Auto-delete |
| Consent records | 5 years | Archive |
| Audit logs | 7 years | Archive |

---

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `S` | Start scan |
| `R` | View reports |
| `D` | Open data map |
| `P` | View policies |
| `N` | New request |
| `/` | Search |
| `Ctrl+E` | Export report |
| `Escape` | Close dialog |

---

## Tips & Tricks

### Staying Compliant

💡 **Schedule regular scans** - Weekly scans catch issues early

💡 **Set up alerts** - Get notified of critical issues immediately

💡 **Document everything** - Keep records of all compliance decisions

💡 **Train your team** - Everyone should understand data handling rules

### Handling Requests

💡 **Respond quickly** - Start processing within 24 hours

💡 **Verify identity** - Confirm requestor is the data subject

💡 **Be thorough** - Check all data sources before responding

💡 **Keep records** - Document how each request was handled

### Data Protection

💡 **Minimize data collection** - Only collect what you need

💡 **Enable encryption** - Protect data at rest and in transit

💡 **Use anonymization** - Remove PII when possible

💡 **Regular audits** - Review who has access to what data

---

## Troubleshooting

### Scan finds false positives

**Possible causes:**
1. Pattern matching too aggressive
2. Test data flagged as real PII
3. Encrypted data misidentified

**Solution:**
1. Review and dismiss false positives
2. Add test data locations to exclusion list
3. Configure scan sensitivity in settings
4. Report issues to improve detection

---

### DSR deadline approaching

**Possible causes:**
1. Complex request requiring manual review
2. Data spread across multiple systems
3. Identity verification pending

**Solution:**
1. Prioritize the request immediately
2. Use automated data collection tools
3. Contact user if verification needed
4. Document reason if extension required

---

### Consent not recording

**Possible causes:**
1. Consent widget not configured
2. JavaScript error on page
3. Database connection issue

**Solution:**
1. Check consent configuration in settings
2. Test consent flow in preview mode
3. Check error logs for issues
4. Verify database connectivity

---

### Data not deleting automatically

**Possible causes:**
1. Retention policy not applied
2. Scheduled job not running
3. Data referenced by other records

**Solution:**
1. Verify policy is active and applied to bot
2. Check scheduled job status in settings
3. Review dependencies that prevent deletion
4. Manually delete if needed

---

## BASIC Integration

Use Compliance features in your dialogs:

### Check Consent

```botserver/docs/src/chapter-04-gbui/apps/compliance-consent.basic
hasConsent = CHECK CONSENT user.id FOR "marketing"

IF hasConsent THEN
    TALK "I can send you our newsletter!"
ELSE
    TALK "Would you like to receive our newsletter?"
    HEAR response AS BOOLEAN
    IF response THEN
        RECORD CONSENT user.id FOR "marketing"
        TALK "Great! You're now subscribed."
    END IF
END IF
```

### Request Data Access

```botserver/docs/src/chapter-04-gbui/apps/compliance-access.basic
TALK "I can help you access your personal data."
HEAR email AS EMAIL "Please confirm your email address"

IF email = user.email THEN
    request = CREATE DSR REQUEST
        TYPE "access"
        USER user.id
        EMAIL email
    
    TALK "Your request #" + request.id + " has been submitted."
    TALK "You'll receive your data within 15 days."
ELSE
    TALK "Email doesn't match. Please contact support."
END IF
```

### Delete User Data

```botserver/docs/src/chapter-04-gbui/apps/compliance-delete.basic
TALK "Are you sure you want to delete all your data?"
TALK "This action cannot be undone."
HEAR confirm AS BOOLEAN

IF confirm THEN
    request = CREATE DSR REQUEST
        TYPE "deletion"
        USER user.id
    
    TALK "Deletion request submitted: #" + request.id
    TALK "Your data will be deleted within 30 days."
ELSE
    TALK "No problem. Your data remains safe."
END IF
```

### Log Compliance Event

```botserver/docs/src/chapter-04-gbui/apps/compliance-log.basic
' Log when sensitive data is accessed
LOG COMPLIANCE EVENT
    TYPE "data_access"
    USER user.id
    DATA_TYPE "order_history"
    REASON "User requested order status"
    BOT "support"

TALK "Here's your order history..."
```

---

## API Endpoint: /api/compliance

The Compliance API allows programmatic access to compliance features.

### Endpoints Summary

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/compliance/scan` | POST | Start a compliance scan |
| `/api/compliance/scan/{id}` | GET | Get scan results |
| `/api/compliance/dsr` | POST | Create DSR request |
| `/api/compliance/dsr/{id}` | GET | Get DSR status |
| `/api/compliance/consent` | POST | Record consent |
| `/api/compliance/consent/{userId}` | GET | Get user consent |
| `/api/compliance/report` | GET | Generate compliance report |

### Authentication

All endpoints require API key authentication:

```botserver/docs/src/chapter-04-gbui/apps/compliance-auth.txt
Authorization: Bearer your-api-key
```

### Example: Check User Consent

```botserver/docs/src/chapter-04-gbui/apps/compliance-api-example.json
GET /api/compliance/consent/usr_abc123

Response:
{
  "userId": "usr_abc123",
  "consents": [
    {
      "type": "terms_of_service",
      "status": "given",
      "timestamp": "2025-01-15T10:32:00Z"
    },
    {
      "type": "marketing",
      "status": "withdrawn",
      "timestamp": "2025-03-22T15:15:00Z"
    }
  ]
}
```

---

## See Also

- [Compliance API Reference](./compliance-api.md) - Full API documentation
- [Analytics App](./analytics.md) - Monitor compliance metrics
- [Sources App](./sources.md) - Configure bot policies
- [How To: Monitor Your Bot](../how-to/monitor-sessions.md)