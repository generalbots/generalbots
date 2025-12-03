# Privacy Template

The privacy template provides a complete LGPD/GDPR/CCPA-compliant Privacy Rights Center, enabling users to exercise their data protection rights through a conversational interface.

## Topic: Data Privacy & Compliance

This template is perfect for:
- LGPD compliance (Brazil)
- GDPR compliance (EU)
- CCPA compliance (California)
- Data subject rights management
- Consent management portals

## The Code

```basic
ADD TOOL "request-data"
ADD TOOL "export-data"
ADD TOOL "delete-data"
ADD TOOL "manage-consents"
ADD TOOL "rectify-data"
ADD TOOL "object-processing"

USE KB "privacy.gbkb"

CLEAR SUGGESTIONS

ADD SUGGESTION "access" AS "View my data"
ADD SUGGESTION "export" AS "Export my data"
ADD SUGGESTION "delete" AS "Delete my data"
ADD SUGGESTION "consents" AS "Manage consents"
ADD SUGGESTION "correct" AS "Correct my data"
ADD SUGGESTION "object" AS "Object to processing"

SET CONTEXT "privacy rights" AS "You are a Privacy Rights Center assistant helping users exercise their data protection rights under LGPD, GDPR, and CCPA. Help with data access, rectification, erasure, portability, and consent management."

BEGIN TALK
**Privacy Rights Center**

As a data subject, you have the following rights:

1. **Access** - View all data we hold about you
2. **Rectification** - Correct inaccurate data
3. **Erasure** - Request deletion of your data
4. **Portability** - Export your data
5. **Object** - Opt-out of certain processing
6. **Consent** - Review and update your consents

Select an option or describe your request.
END TALK

BEGIN SYSTEM PROMPT
You are a Privacy Rights Center assistant for LGPD/GDPR/CCPA compliance.

Data subject rights:
- Right of Access: View all personal data
- Right to Rectification: Correct inaccurate data
- Right to Erasure: Delete personal data (right to be forgotten)
- Right to Portability: Export data in machine-readable format
- Right to Object: Opt-out of marketing, profiling, etc.
- Consent Management: Review and withdraw consents

Always verify identity before processing sensitive requests.
Log all privacy requests for compliance audit.
Provide clear timelines for request fulfillment.
Escalate complex requests to the Data Protection Officer.
END SYSTEM PROMPT
```

## Sample Dialogs

These conversations show how the privacy template works in real-world scenarios.

### Dialog 1: Data Access Request

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">🔒</div>
    <div class="wa-header-info">
      <div class="wa-header-name">Privacy Center</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-date"><span>Today</span></div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <div class="wa-sender">Privacy Center</div>
      <p><strong>Privacy Rights Center</strong></p>
      <p>As a data subject, you have the following rights:</p>
      <p>1. <strong>Access</strong> - View all data we hold about you</p>
      <p>2. <strong>Rectification</strong> - Correct inaccurate data</p>
      <p>3. <strong>Erasure</strong> - Request deletion of your data</p>
      <div class="wa-time">10:00</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>I want to see all the data you have about me</p>
      <div class="wa-time">10:01 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>🔐 <strong>Identity Verification Required</strong></p>
      <p>To protect your privacy, I need to verify your identity before providing access to your personal data.</p>
      <p>I've sent a verification code to your registered email (j***@email.com).</p>
      <p>Please enter the 6-digit code:</p>
      <div class="wa-time">10:01</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>482917</p>
      <div class="wa-time">10:02 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>✅ Identity verified. Here's a summary of your data:</p>
      <p><strong>📋 Personal Information</strong></p>
      <p>• Name: John Smith</p>
      <p>• Email: j***@email.com</p>
      <p>• Phone: ***-***-4567</p>
      <p>• Account created: Jan 15, 2023</p>
      <p><strong>📊 Activity Data</strong></p>
      <p>• Orders: 12</p>
      <p>• Support tickets: 3</p>
      <p>• Last login: Today</p>
      <p><strong>🔔 Marketing Preferences</strong></p>
      <p>• Email marketing: Opted in</p>
      <p>• SMS notifications: Opted out</p>
      <p>Would you like a full export of your data?</p>
      <div class="wa-time">10:02</div>
    </div>
  </div>
</div>

### Dialog 2: Data Deletion Request

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">🔒</div>
    <div class="wa-header-info">
      <div class="wa-header-name">Privacy Center</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>I want to delete all my data</p>
      <div class="wa-time">14:30 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>⚠️ <strong>Data Deletion Request</strong></p>
      <p>You're requesting to exercise your <strong>Right to Erasure</strong> (Right to be Forgotten).</p>
      <p>Please note:</p>
      <p>• This action is <strong>irreversible</strong></p>
      <p>• Your account will be permanently closed</p>
      <p>• Some data may be retained for legal compliance</p>
      <p>Before proceeding, I need to verify your identity. Please confirm your registered email address:</p>
      <div class="wa-time">14:30</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>john.smith@email.com</p>
      <div class="wa-time">14:31 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>📧 A verification link has been sent to john.smith@email.com</p>
      <p>Please click the link to confirm your deletion request.</p>
      <p><strong>What will be deleted:</strong></p>
      <p>• Personal profile information</p>
      <p>• Activity history and preferences</p>
      <p>• Marketing and communication history</p>
      <p><strong>What will be retained (legal requirement):</strong></p>
      <p>• Transaction records (7 years)</p>
      <p>• Tax-related documents (5 years)</p>
      <p>Your request will be processed within <strong>15 business days</strong> (LGPD) / <strong>30 days</strong> (GDPR).</p>
      <p>Request ID: <strong>DEL-2025-001234</strong></p>
      <div class="wa-time">14:31</div>
    </div>
  </div>
</div>

### Dialog 3: Consent Management

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">🔒</div>
    <div class="wa-header-info">
      <div class="wa-header-name">Privacy Center</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>I want to manage my consents</p>
      <div class="wa-time">16:00 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>⚙️ <strong>Your Consent Settings</strong></p>
      <p></p>
      <p><strong>Essential Services</strong> (Required)</p>
      <p>✅ Account management - Required for service</p>
      <p>✅ Security features - Required for protection</p>
      <p></p>
      <p><strong>Optional Consents</strong></p>
      <p>✅ Email marketing - Receive promotions</p>
      <p>❌ SMS notifications - Text messages</p>
      <p>✅ Personalization - Customized experience</p>
      <p>❌ Third-party sharing - Partner offers</p>
      <p>✅ Analytics - Help us improve</p>
      <p></p>
      <p>Which consent would you like to change?</p>
      <div class="wa-time">16:00</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Turn off email marketing</p>
      <div class="wa-time">16:01 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>✅ <strong>Consent Updated</strong></p>
      <p>Email marketing has been <strong>disabled</strong>.</p>
      <p>You will no longer receive promotional emails from us.</p>
      <p>This change takes effect immediately. You can re-enable this at any time.</p>
      <p>📋 Audit log updated for compliance.</p>
      <div class="wa-time">16:01</div>
    </div>
  </div>
</div>

### Dialog 4: Data Export (Portability)

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">🔒</div>
    <div class="wa-header-info">
      <div class="wa-header-name">Privacy Center</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Export my data</p>
      <div class="wa-time">11:00 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>📦 <strong>Data Export Request</strong></p>
      <p>Choose your preferred export format:</p>
      <p>1️⃣ <strong>JSON</strong> - Machine-readable, for transferring to other services</p>
      <p>2️⃣ <strong>CSV</strong> - Spreadsheet format, easy to view</p>
      <p>3️⃣ <strong>PDF</strong> - Human-readable report</p>
      <div class="wa-time">11:00</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>JSON</p>
      <div class="wa-time">11:01 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>⏳ <strong>Preparing Your Data Export</strong></p>
      <p>Your data export is being prepared in JSON format.</p>
      <p>📧 A secure download link will be sent to your email within <strong>24 hours</strong>.</p>
      <p>The link will expire in <strong>7 days</strong> for security.</p>
      <p>Request ID: <strong>EXP-2025-005678</strong></p>
      <div class="wa-time">11:01</div>
    </div>
  </div>
</div>

## Keywords Used

| Keyword | Purpose |
|---------|---------|
| `ADD TOOL` | Register privacy rights tools |
| `USE KB` | Load privacy policy knowledge base |
| `ADD SUGGESTION` | Create quick action buttons for rights |
| `SET CONTEXT` | Define privacy assistant behavior |
| `BEGIN TALK` | Welcome message with rights summary |
| `BEGIN SYSTEM PROMPT` | Compliance rules and procedures |

## Template Structure

```
privacy.gbai/
├── privacy.gbdialog/
│   ├── start.bas           # Main entry point
│   ├── request-data.bas    # Data access requests
│   ├── export-data.bas     # Data portability
│   ├── delete-data.bas     # Right to erasure
│   ├── manage-consents.bas # Consent management
│   └── rectify-data.bas    # Data correction
├── privacy.gbot/
│   └── config.csv          # Configuration
├── privacy.gbkb/
│   └── privacy-policy.md   # Privacy documentation
└── privacy.gbui/
    └── index.html          # Web portal UI
```

## Data Subject Rights by Regulation

| Right | LGPD (Brazil) | GDPR (EU) | CCPA (California) |
|-------|---------------|-----------|-------------------|
| Access | Art. 18 | Art. 15 | §1798.100 |
| Rectification | Art. 18 III | Art. 16 | - |
| Erasure | Art. 18 VI | Art. 17 | §1798.105 |
| Portability | Art. 18 V | Art. 20 | §1798.100 |
| Object | Art. 18 IV | Art. 21 | §1798.120 |
| Consent | Art. 8 | Art. 7 | §1798.135 |

## Response Deadlines

| Regulation | Standard | Extended |
|------------|----------|----------|
| LGPD | 15 days | - |
| GDPR | 30 days | 90 days (complex) |
| CCPA | 45 days | 90 days |

## Request Data Tool: request-data.bas

```basic
PARAM request_type AS STRING LIKE "full" DESCRIPTION "Type of data request: full, summary, specific"

DESCRIPTION "Process a data access request (Right of Access)"

' Verify identity first
TALK "🔐 To protect your privacy, I need to verify your identity."
TALK "I'll send a verification code to your registered email."

code = FORMAT(RANDOM(100000, 999999))
SET BOT MEMORY "verification_code_" + user_id, code
SET BOT MEMORY "verification_expiry_" + user_id, DATEADD(NOW(), 10, "minutes")

SEND MAIL user_email, "Privacy Request Verification", "Your verification code is: " + code

TALK "Please enter the 6-digit code sent to your email:"
HEAR entered_code

stored_code = GET BOT MEMORY("verification_code_" + user_id)
expiry = GET BOT MEMORY("verification_expiry_" + user_id)

IF entered_code <> stored_code OR NOW() > expiry THEN
    TALK "❌ Invalid or expired code. Please try again."
    RETURN NULL
END IF

' Log the request for compliance
WITH request
    id = "ACC-" + FORMAT(NOW(), "YYYY") + "-" + FORMAT(RANDOM(100000, 999999))
    user_id = user_id
    type = "access"
    status = "processing"
    created_at = NOW()
    deadline = DATEADD(NOW(), 15, "days")
END WITH

SAVE "privacy_requests.csv", request

' Retrieve user data
userData = FIND "users.csv", "id = '" + user_id + "'"
activityData = FIND "activity_log.csv", "user_id = '" + user_id + "'"
consents = FIND "consents.csv", "user_id = '" + user_id + "'"

TALK "✅ Identity verified. Here's your data:"
TALK ""
TALK "**📋 Personal Information**"
TALK "• Name: " + userData.name
TALK "• Email: " + MASK_EMAIL(userData.email)
TALK "• Account created: " + FORMAT(userData.created_at, "MMM DD, YYYY")
TALK ""
TALK "**📊 Activity Summary**"
TALK "• Total activities: " + UBOUND(activityData)
TALK "• Last activity: " + FORMAT(activityData[1].timestamp, "MMM DD, YYYY")
TALK ""
TALK "**🔔 Consent Status**"
FOR EACH consent IN consents
    status_icon = IIF(consent.granted, "✅", "❌")
    TALK "• " + consent.purpose + ": " + status_icon
NEXT

TALK ""
TALK "Request ID: **" + request.id + "**"
TALK "Would you like a full export of your data?"

RETURN request.id
```

## Delete Data Tool: delete-data.bas

```basic
PARAM confirm AS STRING LIKE "yes" DESCRIPTION "Confirmation to proceed with deletion"

DESCRIPTION "Process a data erasure request (Right to be Forgotten)"

' Warn about consequences
TALK "⚠️ **Data Deletion Request**"
TALK ""
TALK "This will permanently delete:"
TALK "• Your profile and personal information"
TALK "• Activity history and preferences"
TALK "• Communication history"
TALK ""
TALK "**Note:** Some data may be retained for legal compliance:"
TALK "• Financial records (tax requirements)"
TALK "• Fraud prevention data"
TALK "• Legal dispute documentation"
TALK ""
TALK "Type **DELETE MY DATA** to confirm this irreversible action:"

HEAR confirmation

IF UPPER(confirmation) <> "DELETE MY DATA" THEN
    TALK "Deletion cancelled. Your data remains unchanged."
    RETURN NULL
END IF

' Create deletion request
WITH request
    id = "DEL-" + FORMAT(NOW(), "YYYY") + "-" + FORMAT(RANDOM(100000, 999999))
    user_id = user_id
    type = "erasure"
    status = "pending_verification"
    created_at = NOW()
    deadline = DATEADD(NOW(), 15, "days")
END WITH

SAVE "privacy_requests.csv", request

' Send verification email
verification_link = "https://privacy.company.com/verify/" + request.id
SEND MAIL user_email, "Confirm Data Deletion Request", 
    "Click to confirm your data deletion request:\n\n" + verification_link + 
    "\n\nThis link expires in 24 hours.\n\nRequest ID: " + request.id

TALK "📧 A verification email has been sent."
TALK "Please click the link to confirm your deletion request."
TALK ""
TALK "**Timeline:**"
TALK "• Verification: 24 hours"
TALK "• Processing: 15 business days (LGPD) / 30 days (GDPR)"
TALK ""
TALK "Request ID: **" + request.id + "**"

RETURN request.id
```

## Customization Ideas

### Add Identity Verification Options

```basic
TALK "How would you like to verify your identity?"
ADD SUGGESTION "email" AS "Email verification"
ADD SUGGESTION "sms" AS "SMS verification"
ADD SUGGESTION "id" AS "Upload ID document"

HEAR method

SWITCH method
    CASE "email"
        ' Send email code
    CASE "sms"
        ' Send SMS code
    CASE "id"
        TALK "Please upload a photo of your government-issued ID."
        HEAR id_upload AS FILE
        ' Process ID verification
END SWITCH
```

### Add DPO Escalation

```basic
' For complex requests
IF request_complexity = "high" THEN
    TALK "This request requires review by our Data Protection Officer."
    TALK "You will be contacted within 5 business days."
    
    SEND MAIL "dpo@company.com", "Privacy Request Escalation",
        "Request ID: " + request.id + "\n" +
        "Type: " + request.type + "\n" +
        "User: " + user_email + "\n" +
        "Reason: Complex request requiring DPO review"
END IF
```

### Add Audit Logging

```basic
' Log all privacy operations
WITH auditLog
    timestamp = NOW()
    request_id = request.id
    user_id = user_id
    action = "data_access"
    ip_address = GET_CLIENT_IP()
    user_agent = GET_USER_AGENT()
    result = "success"
END WITH

SAVE "privacy_audit_log.csv", auditLog
```

## Best Practices

1. **Always Verify Identity**: Never provide data without verification
2. **Log Everything**: Maintain audit trails for compliance
3. **Clear Timelines**: Communicate response deadlines clearly
4. **Explain Retention**: Be transparent about what data is retained and why
5. **Easy Consent Management**: Make it simple to change preferences
6. **Secure Communications**: Use encrypted channels for sensitive data

## Related Templates

- [auth.bas](./auth.md) - Authentication patterns
- [bank.bas](./bank.md) - Secure financial data handling
- [hipaa.bas](./hipaa.md) - Healthcare privacy compliance

---

<style>
.wa-chat{background-color:#e5ddd5;border-radius:8px;padding:20px 15px;margin:20px 0;max-width:600px;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,Helvetica,Arial,sans-serif;font-size:14px}
.wa-chat::after{content:'';display:table;clear:both}
.wa-message{clear:both;margin-bottom:10px;max-width:85%;position:relative}
.wa-message.user{float:right}
.wa-message.user .wa-bubble{background-color:#dcf8c6;border-radius:8px 0 8px 8px;margin-left:40px}
.wa-message.bot{float:left}
.wa-message.bot .wa-bubble{background-color:#fff;border-radius:0 8px 8px 8px;margin-right:40px}
.wa-bubble{padding:8px 12px;box-shadow:0 1px .5px rgba(0,0,0,.13)}
.wa-bubble p{margin:0 0 4px 0;line-height:1.4;color:#303030}
.wa-bubble p:last-child{margin-bottom:0}
.wa-time{font-size:11px;color:#8696a0;text-align:right;margin-top:4px}
.wa-message.user .wa-time{color:#61a05e}
.wa-sender{font-size:12px;font-weight:600;color:#06cf9c;margin-bottom:2px}
.wa-status.read::after{content:'✓✓';color:#53bdeb;margin-left:4px}
.wa-date{text-align:center;margin:15px 0;clear:both}
.wa-date span{background-color:#fff;color:#54656f;padding:5px 12px;border-radius:8px;font-size:12px;box-shadow:0 1px .5px rgba(0,0,0,.13)}
.wa-header{background-color:#075e54;color:#fff;padding:10px 15px;margin:-20px -15px 15px -15px;border-radius:8px 8px 0 0;display:flex;align-items:center;gap:10px}
.wa-header-avatar{width:40px;height:40px;background-color:#25d366;border-radius:50%;display:flex;align-items:center;justify-content:center;font-size:18px}
.wa-header-info{flex:1}
.wa-header-name{font-weight:600;font-size:16px}
.wa-header-status{font-size:12px;opacity:.8}
</style>