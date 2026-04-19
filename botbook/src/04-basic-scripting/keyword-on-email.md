# ON EMAIL

Monitors an email address and triggers a script when new emails arrive.

## Syntax

```basic
' Basic - trigger on any email
ON EMAIL "address@example.com"

' With FROM filter - only from specific sender
ON EMAIL "address@example.com" FROM "sender@example.com"

' With SUBJECT filter - only matching subject
ON EMAIL "address@example.com" SUBJECT "pattern"
```

## Description

The `ON EMAIL` keyword registers an email monitor that triggers a script whenever new emails arrive at the specified address. This is part of the event-driven programming model, similar to `SET SCHEDULE` and `ON` (for database triggers).

When an email arrives, the system creates an entry in `email_received_events` which can be processed by your script.

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| address | String | Yes | Email address to monitor |
| FROM | String | No | Filter to only trigger for emails from this sender |
| SUBJECT | String | No | Filter to only trigger for emails matching this subject pattern |

## Examples

### Basic Email Monitoring

```basic
' Monitor support inbox for any incoming email
ON EMAIL "support@company.com"
    email = GET LAST "email_received_events"
    TALK "New support request from " + email.from_address
    TALK "Subject: " + email.subject
END ON
```

### Filter by Sender

```basic
' Only trigger for emails from a specific supplier
ON EMAIL "orders@company.com" FROM "supplier@vendor.com"
    email = GET LAST "email_received_events"
    
    ' Process supplier invoice
    IF email.has_attachments THEN
        attachments = email.attachments
        FOR EACH att IN attachments
            IF att.mime_type = "application/pdf" THEN
                COPY att.filename TO "account://finance@company.com/Invoices/"
            END IF
        NEXT
    END IF
END ON
```

### Filter by Subject

```basic
' Only trigger for urgent alerts
ON EMAIL "alerts@company.com" SUBJECT "URGENT"
    email = GET LAST "email_received_events"
    
    ' Send SMS notification to on-call team
    SMS "+1-555-0100", "URGENT ALERT: " + email.subject
    
    ' Create task for follow-up
    CREATE TASK "Handle urgent alert", email.subject, "high"
END ON
```

### Process Attachments

```basic
ON EMAIL "invoices@company.com"
    email = GET LAST "email_received_events"
    
    IF email.has_attachments THEN
        FOR EACH attachment IN email.attachments
            ' Save to drive
            filename = attachment.filename
            COPY attachment TO "account://user@gmail.com/Invoices/" + filename
            
            ' Extract data if PDF
            IF attachment.mime_type = "application/pdf" THEN
                data = SAVE FROM UNSTRUCTURED attachment, "invoices"
                TALK "Processed invoice: " + data.invoice_number
            END IF
        NEXT
    END IF
END ON
```

### Auto-Reply System

```basic
ON EMAIL "info@company.com"
    email = GET LAST "email_received_events"
    
    ' Check if it's a common question
    response = LLM "Classify this email and suggest a response: " + email.subject
    
    IF response.category = "pricing" THEN
        SEND MAIL email.from_address, "RE: " + email.subject, "
            Thank you for your inquiry about pricing.
            Please visit our pricing page: https://company.com/pricing
            
            Best regards,
            Company Team
        "
    ELSE IF response.category = "support" THEN
        ' Create support ticket
        CREATE TASK "Support: " + email.subject, email.from_address, "normal"
        
        SEND MAIL email.from_address, "RE: " + email.subject, "
            Your support request has been received.
            Ticket ID: " + task.id + "
            We'll respond within 24 hours.
        "
    END IF
END ON
```

### Multi-Account Monitoring

```basic
' Monitor multiple email addresses
ON EMAIL "sales@company.com"
    email = GET LAST "email_received_events"
    SET BOT MEMORY "lead_source", "sales_inbox"
    ' Process sales inquiry
END ON

ON EMAIL "support@company.com"
    email = GET LAST "email_received_events"
    SET BOT MEMORY "lead_source", "support_inbox"
    ' Process support request
END ON

ON EMAIL "billing@company.com" FROM "stripe.com"
    email = GET LAST "email_received_events"
    ' Process Stripe payment notifications
END ON
```

## Email Event Properties

When an email is received, the event object contains:

| Property | Type | Description |
|----------|------|-------------|
| `id` | UUID | Unique event identifier |
| `monitor_id` | UUID | ID of the monitor that triggered |
| `message_uid` | Integer | Email server message UID |
| `message_id` | String | Email Message-ID header |
| `from_address` | String | Sender email address |
| `to_addresses` | Array | List of recipient addresses |
| `subject` | String | Email subject line |
| `has_attachments` | Boolean | Whether email has attachments |
| `attachments` | Array | List of attachment objects |

### Attachment Properties

| Property | Type | Description |
|----------|------|-------------|
| `filename` | String | Original filename |
| `mime_type` | String | MIME type (e.g., "application/pdf") |
| `size` | Integer | Size in bytes |

## Database Tables

### email_monitors

Stores the monitor configuration:

| Column | Type | Description |
|--------|------|-------------|
| `id` | UUID | Monitor ID |
| `bot_id` | UUID | Bot that owns this monitor |
| `email_address` | VARCHAR | Address being monitored |
| `script_path` | VARCHAR | Script to execute |
| `is_active` | BOOLEAN | Whether monitor is active |
| `filter_from` | VARCHAR | Optional FROM filter |
| `filter_subject` | VARCHAR | Optional SUBJECT filter |
| `last_uid` | BIGINT | Last processed message UID |

### email_received_events

Logs received emails:

| Column | Type | Description |
|--------|------|-------------|
| `id` | UUID | Event ID |
| `monitor_id` | UUID | Monitor that triggered |
| `message_uid` | BIGINT | Email server UID |
| `from_address` | VARCHAR | Sender address |
| `subject` | VARCHAR | Email subject |
| `has_attachments` | BOOLEAN | Has attachments flag |
| `processed` | BOOLEAN | Whether event was processed |

## Best Practices

1. **Use filters when possible** - Reduce unnecessary script executions
2. **Handle errors gracefully** - Emails may have unexpected formats
3. **Process attachments safely** - Validate MIME types before processing
4. **Avoid infinite loops** - Don't send emails that trigger your own monitors
5. **Log important actions** - Track what was processed for debugging

## Comparison with Other Event Keywords

| Keyword | Trigger Source | Use Case |
|---------|---------------|----------|
| `ON EMAIL` | Incoming emails | Process messages, auto-reply |
| `ON CHANGE` | File system changes | Sync files, process uploads |
| `ON INSERT` | Database inserts | React to new data |
| `SET SCHEDULE` | Time-based | Periodic tasks |
| `WEBHOOK` | HTTP requests | External integrations |

## Related Keywords

- [ON CHANGE](./keyword-on-change.md) - File/folder monitoring
- [ON](./keyword-on.md) - Database trigger events
- [SET SCHEDULE](./keyword-set-schedule.md) - Time-based scheduling
- [SEND MAIL](./keyword-send-mail.md) - Send emails
- [WEBHOOK](./keyword-webhook.md) - HTTP webhooks

---

**TriggerKind:** `EmailReceived = 5`
