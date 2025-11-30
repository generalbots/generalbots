# Zapier and Make Migration Guide

Migrating workflows from Zapier or Make (formerly Integromat) to General Bots.

<img src="../assets/gb-decorative-header.svg" alt="General Bots" style="max-height: 100px; width: 100%; object-fit: contain;">

## Overview

Zapier and Make are visual automation platforms connecting apps through triggers and actions. General Bots provides equivalent automation through BASIC scripting, offering more power and flexibility without per-task pricing.

## Why Migrate

| Aspect | Zapier/Make | General Bots |
|--------|-------------|--------------|
| Pricing | Per-task/operation | Unlimited executions |
| Automation | Visual workflows | BASIC scripts |
| AI Integration | Via paid apps | Native LLM keywords |
| Chat/Bot | Not included | Multi-channel |
| Productivity Suite | Not included | Email, calendar, files |
| Custom Logic | Limited | Full programming |
| Self-hosting | Not available | Full control |
| API Creation | Not available | Instant webhooks |

## Cost Comparison

### Zapier Pricing

| Plan | Tasks/Month | Cost |
|------|-------------|------|
| Free | 100 | $0 |
| Starter | 750 | $19.99 |
| Professional | 2,000 | $49 |
| Team | 50,000 | $69/user |
| Company | 100,000+ | Custom |

### Make Pricing

| Plan | Operations/Month | Cost |
|------|------------------|------|
| Free | 1,000 | $0 |
| Core | 10,000 | $9 |
| Pro | 10,000 | $16 |
| Teams | 10,000 | $29/user |
| Enterprise | Custom | Custom |

### General Bots

| Plan | Operations | Cost |
|------|------------|------|
| Self-hosted | Unlimited | Infrastructure only |

## Trigger Mapping

| Zapier/Make Trigger | General Bots Equivalent |
|---------------------|------------------------|
| Schedule | `SET SCHEDULE` |
| Webhook | `WEBHOOK` |
| New Email | `ON "email:received"` |
| New Row (Sheets) | `ON "table:name:insert"` |
| Form Submission | `ON FORM SUBMIT` |
| New File | `ON "file:created"` |
| RSS Feed | Scheduled `GET` |
| App-specific | API polling or webhooks |

## Action Mapping

| Zapier/Make Action | General Bots Equivalent |
|--------------------|------------------------|
| Send Email | `SEND MAIL` |
| HTTP Request | `GET`, `POST`, `PUT`, `DELETE` |
| Create Row | `INSERT` |
| Update Row | `UPDATE` |
| Filter | `IF/THEN/ELSE` |
| Formatter | String/date functions |
| Delay | `WAIT` |
| Paths | `IF` branches |
| Loop | `FOR EACH` |
| Code (JS/Python) | BASIC script |
| Slack Message | `POST` to Slack webhook |
| Create Task | `CREATE TASK` |
| Send SMS | SMS integration |

## Migration Examples

### Simple Zap: Form to Email

**Zapier:**
```
Typeform → Gmail (Send Email)
```

**General Bots:**
```basic
ON FORM SUBMIT "contact-form"
    name = fields.name
    email = fields.email
    message = fields.message
    
    SEND MAIL TO "support@company.com" SUBJECT "New Contact: " + name BODY "From: " + email + "\n\nMessage:\n" + message
END ON
```

### Multi-Step Zap: Lead Processing

**Zapier:**
```
Webhook → Filter → Clearbit Enrich → Salesforce (Create Lead) → Slack (Send Message)
```

**General Bots:**
```basic
WEBHOOK "new-lead"

lead = body

' Filter
IF lead.email = "" OR NOT CONTAINS(lead.email, "@") THEN
    RETURN #{status: "invalid", reason: "Invalid email"}
END IF

' Enrich
SET HEADER "Authorization", "Bearer " + GET CONFIG "clearbit-key"
enriched = GET "https://person.clearbit.com/v2/people/find?email=" + lead.email

' Create in CRM
WITH salesforce_lead
    .Email = lead.email
    .FirstName = enriched.name.givenName
    .LastName = enriched.name.familyName
    .Company = enriched.employment.name
    .Title = enriched.employment.title
END WITH

SET HEADER "Authorization", "Bearer " + GET CONFIG "salesforce-token"
result = POST "https://yourinstance.salesforce.com/services/data/v52.0/sobjects/Lead", salesforce_lead

' Notify Slack
POST GET CONFIG "slack-webhook", #{
    text: "New lead: " + lead.email + " from " + enriched.employment.name
}

RETURN #{status: "success", salesforce_id: result.id}
```

### Scheduled Sync

**Make Scenario:**
```
Schedule → HTTP Request → Iterator → Google Sheets (Add Row)
```

**General Bots:**
```basic
SET SCHEDULE "every hour"

data = GET "https://api.example.com/new-orders"

FOR EACH order IN data.orders
    INSERT "orders", #{
        order_id: order.id,
        customer: order.customer_name,
        total: order.total,
        status: order.status,
        synced_at: NOW()
    }
NEXT order

TALK "Synced " + LEN(data.orders) + " orders"
```

### Error Handling

**Zapier:** Error handling path or retry

**General Bots:**
```basic
SET SCHEDULE "every 5 minutes"

TRY
    result = POST "https://api.example.com/sync", data
    IF result.status <> 200 THEN
        THROW "API returned " + result.status
    END IF
CATCH
    ' Log error
    INSERT "error_log", #{
        error: ERROR_MESSAGE,
        timestamp: NOW(),
        data: data
    }
    
    ' Alert
    SEND MAIL TO "ops@company.com" SUBJECT "Sync Error" BODY ERROR_MESSAGE
    POST GET CONFIG "slack-alerts", #{text: "Sync failed: " + ERROR_MESSAGE}
END TRY
```

### Conditional Paths

**Zapier Paths:**
```
Trigger → Path A (if condition) → Actions
       → Path B (else) → Actions
```

**General Bots:**
```basic
WEBHOOK "order-status"

order = body

IF order.total > 1000 THEN
    ' High-value order path
    SEND MAIL TO "vip-team@company.com" SUBJECT "High-Value Order" BODY order
    POST GET CONFIG "slack-vip", #{text: "VIP Order: $" + order.total}
    priority = "high"
    
ELSEIF order.is_rush = true THEN
    ' Rush order path
    SEND MAIL TO "fulfillment@company.com" SUBJECT "RUSH Order" BODY order
    priority = "rush"
    
ELSE
    ' Standard order path
    priority = "normal"
END IF

INSERT "orders", #{
    id: order.id,
    total: order.total,
    priority: priority,
    created: NOW()
}
```

### Data Transformation

**Make/Zapier Formatter:**
- Split text
- Format dates
- Math operations
- Lookup tables

**General Bots:**
```basic
' String operations
full_name = first_name + " " + last_name
email_domain = SPLIT(email, "@")[1]
slug = LOWER(REPLACE(title, " ", "-"))

' Date formatting
formatted_date = FORMAT(created_at, "MMMM d, yyyy")
due_date = DATEADD(NOW(), 7, "day")
days_ago = DATEDIFF("day", created_at, NOW())

' Math
subtotal = price * quantity
tax = subtotal * 0.08
total = subtotal + tax
discount = IIF(total > 100, total * 0.1, 0)

' Lookup
status_label = SWITCH status
    CASE "new" : "New Order"
    CASE "processing" : "In Progress"
    CASE "shipped" : "On the Way"
    CASE "delivered" : "Completed"
    DEFAULT : "Unknown"
END SWITCH
```

## App-Specific Migrations

### Gmail/Email

**Zapier:** Gmail trigger/action

**General Bots:**
```basic
' Send email
SEND MAIL TO recipient SUBJECT subject BODY body

' With attachments
SEND MAIL TO recipient SUBJECT subject BODY body ATTACH "/files/report.pdf"

' Process incoming (via Stalwart webhook)
ON "email:received"
    IF CONTAINS(params.subject, "Order") THEN
        PROCESS_ORDER(params)
    END IF
END ON
```

### Slack

**Zapier:** Slack app

**General Bots:**
```basic
' Simple message
POST "https://hooks.slack.com/services/xxx", #{text: "Hello!"}

' Rich message
WITH slack_message
    .channel = "#general"
    .blocks = [
        #{type: "header", text: #{type: "plain_text", text: "New Order"}},
        #{type: "section", text: #{type: "mrkdwn", text: "*Customer:* " + customer_name}},
        #{type: "section", text: #{type: "mrkdwn", text: "*Total:* $" + total}}
    ]
END WITH
POST GET CONFIG "slack-webhook", slack_message
```

### Google Sheets

**Zapier:** Google Sheets app

**General Bots:**
```basic
' Read from sheet
SET HEADER "Authorization", "Bearer " + GET CONFIG "google-token"
data = GET "https://sheets.googleapis.com/v4/spreadsheets/{spreadsheetId}/values/Sheet1!A1:D100"

' Append row
POST "https://sheets.googleapis.com/v4/spreadsheets/{spreadsheetId}/values/Sheet1!A1:append?valueInputOption=USER_ENTERED", #{
    values: [[name, email, phone, NOW()]]
}

' Or use General Bots tables directly
INSERT "contacts", #{name: name, email: email, phone: phone}
```

### Airtable

**Zapier:** Airtable app

**General Bots:**
```basic
SET HEADER "Authorization", "Bearer " + GET CONFIG "airtable-key"

' Read records
records = GET "https://api.airtable.com/v0/{baseId}/{tableName}"

' Create record
POST "https://api.airtable.com/v0/{baseId}/{tableName}", #{
    fields: #{
        Name: name,
        Email: email,
        Status: "New"
    }
}
```

### HubSpot

**Zapier:** HubSpot app

**General Bots:**
```basic
SET HEADER "Authorization", "Bearer " + GET CONFIG "hubspot-token"

' Create contact
POST "https://api.hubapi.com/crm/v3/objects/contacts", #{
    properties: #{
        email: email,
        firstname: first_name,
        lastname: last_name,
        company: company
    }
}

' Create deal
POST "https://api.hubapi.com/crm/v3/objects/deals", #{
    properties: #{
        dealname: deal_name,
        amount: amount,
        pipeline: "default",
        dealstage: "appointmentscheduled"
    }
}
```

## What You Gain

### No Operation Limits

```basic
' Process thousands of records without worrying about limits
SET SCHEDULE "every hour"

records = GET "https://api.example.com/all-records"

FOR EACH record IN records
    PROCESS_RECORD(record)  ' No per-operation cost
NEXT record
```

### Native AI Integration

```basic
' AI-powered automation
USE KB "company-docs"

incoming_email = params.body
category = LLM "Categorize this email as: support, sales, billing, or other: " + incoming_email

IF category = "support" THEN
    response = LLM "Draft a helpful support response to: " + incoming_email
    SEND MAIL TO params.from SUBJECT "Re: " + params.subject BODY response
END IF
```

### Multi-Channel Chat

```basic
' Same automation works across channels
TALK "How can I help you?"
HEAR request

USE KB "help-docs"
answer = LLM request
TALK answer

' Available on Web, WhatsApp, Teams, Slack, Telegram, SMS
```

### Built-in Productivity

```basic
' No need for separate calendar, task, email apps
CREATE TASK "Follow up with " + customer_name DUE DATEADD(NOW(), 3, "day")
BOOK "Call with " + customer_name AT meeting_time
SEND MAIL TO customer_email SUBJECT "Confirmation" BODY message
```

## Migration Checklist

### Pre-Migration

- [ ] Export Zap/Scenario descriptions
- [ ] Document all triggers and schedules
- [ ] List all connected apps and credentials
- [ ] Identify critical automations
- [ ] Set up General Bots environment

### Migration

- [ ] Create BASIC scripts for each workflow
- [ ] Configure credentials in config.csv
- [ ] Set up webhooks with same URLs
- [ ] Configure schedules
- [ ] Test each automation

### Post-Migration

- [ ] Run parallel for verification
- [ ] Compare execution results
- [ ] Monitor for errors
- [ ] Disable Zapier/Make automations
- [ ] Cancel subscriptions

## Best Practices

**Start with simple Zaps.** Migrate basic workflows first to learn BASIC syntax.

**Combine multiple Zaps.** Often several Zaps can become one General Bots script.

**Use native features.** Don't replicate Zapier patterns—leverage AI, chat, and productivity features.

**Add error handling.** BASIC provides better error handling than visual builders.

**Document your scripts.** Add comments explaining what each script does.

```basic
' Daily sales report
' Runs at 6 PM on weekdays
' Aggregates daily orders and sends summary to management
SET SCHEDULE "0 18 * * 1-5"

' ... implementation
```

## See Also

- [SET SCHEDULE](../chapter-06-gbdialog/keyword-set-schedule.md) - Scheduling
- [WEBHOOK](../chapter-06-gbdialog/keyword-webhook.md) - Webhooks
- [HTTP Keywords](../chapter-06-gbdialog/keywords-http.md) - API calls
- [Platform Comparison](./comparison-matrix.md) - Full comparison