# n8n Migration Guide

Migrating workflows and automations from n8n to General Bots.

<img src="../assets/gb-decorative-header.svg" alt="General Bots" style="max-height: 100px; width: 100%; object-fit: contain;">

## Overview

n8n is a workflow automation platform with a visual node-based editor. General Bots provides equivalent automation capabilities through BASIC scripting, offering more flexibility and integrated features without execution limits.

## Why Migrate

| Aspect | n8n | General Bots |
|--------|-----|--------------|
| Automation | Visual workflows | BASIC scripts (more powerful) |
| Pricing | Per-execution limits | Unlimited executions |
| AI Integration | Via API nodes | Native LLM keywords |
| Chat/Bot | Not included | Full multi-channel |
| Productivity Suite | Not included | Email, calendar, files, tasks |
| Knowledge Base | Not included | Built-in RAG |
| Self-hosting | Available | Available |

## Workflow Mapping

### Triggers

| n8n Trigger | General Bots Equivalent |
|-------------|------------------------|
| Schedule Trigger | `SET SCHEDULE` |
| Webhook | `WEBHOOK` |
| Email Trigger (IMAP) | `ON "email:received"` |
| Database Trigger | `ON "table:tablename:insert"` |
| Manual Trigger | Direct script execution |
| Cron | `SET SCHEDULE "cron expression"` |

### Common Nodes

| n8n Node | General Bots Equivalent |
|----------|------------------------|
| HTTP Request | `GET`, `POST`, `PUT`, `DELETE` |
| Set | Variable assignment |
| IF | `IF/THEN/ELSE/END IF` |
| Switch | `SWITCH/CASE/END SWITCH` |
| Code (JavaScript) | BASIC script |
| Function | BASIC subroutines |
| Merge | Array operations |
| Split In Batches | `FOR EACH` |
| Wait | `WAIT` |
| Send Email | `SEND MAIL` |
| Slack | `POST` to Slack webhook |
| Discord | `POST` to Discord webhook |
| Google Sheets | `GET`/`POST` to Sheets API |
| Airtable | `GET`/`POST` to Airtable API |
| MySQL/PostgreSQL | `FIND`, `INSERT`, `UPDATE`, `DELETE` |
| MongoDB | `GET`/`POST` to MongoDB API |

## Migration Examples

### Scheduled Data Sync

**n8n workflow:**
```
Schedule Trigger → HTTP Request → IF → Google Sheets
```

**General Bots equivalent:**

```basic
SET SCHEDULE "every hour"

data = GET "https://api.example.com/data"

IF data.status = "active" THEN
    FOR EACH item IN data.items
        INSERT "synced_data", #{
            id: item.id,
            name: item.name,
            value: item.value,
            synced_at: NOW()
        }
    NEXT item
END IF

TALK "Synced " + LEN(data.items) + " items"
```

### Webhook Processing

**n8n workflow:**
```
Webhook → Set → IF → Send Email + Slack
```

**General Bots equivalent:**

```basic
WEBHOOK "order-received"

order = body
customer_name = order.customer.name
order_total = order.total

IF order_total > 1000 THEN
    SEND MAIL TO "sales@company.com" SUBJECT "Large Order" BODY "Order from " + customer_name + ": $" + order_total
    
    POST "https://hooks.slack.com/services/xxx", #{
        text: "Large order received: $" + order_total
    }
END IF
```

### Multi-Step API Orchestration

**n8n workflow:**
```
Webhook → HTTP Request (API 1) → Code → HTTP Request (API 2) → IF → Multiple outputs
```

**General Bots equivalent:**

```basic
WEBHOOK "process-lead"

lead = body

' Step 1: Enrich lead data
enriched = POST "https://api.clearbit.com/enrich", #{email: lead.email}

' Step 2: Score the lead
WITH lead_data
    .email = lead.email
    .company = enriched.company.name
    .industry = enriched.company.industry
    .size = enriched.company.employees
END WITH

score = SCORE LEAD lead_data

' Step 3: Route based on score
IF score.status = "hot" THEN
    POST "https://api.salesforce.com/leads", lead_data
    SEND MAIL TO "sales@company.com" SUBJECT "Hot Lead" BODY lead_data
ELSEIF score.status = "warm" THEN
    POST "https://api.hubspot.com/contacts", lead_data
ELSE
    INSERT "cold_leads", lead_data
END IF
```

### Error Handling

**n8n approach:** Error Trigger node

**General Bots equivalent:**

```basic
SET SCHEDULE "every 5 minutes"

TRY
    result = GET "https://api.example.com/health"
    IF result.status <> "healthy" THEN
        THROW "Service unhealthy: " + result.message
    END IF
CATCH
    SEND MAIL TO "ops@company.com" SUBJECT "Alert: Service Down" BODY ERROR_MESSAGE
    POST "https://hooks.slack.com/services/xxx", #{text: "Service alert: " + ERROR_MESSAGE}
END TRY
```

## Exporting n8n Workflows

### Export Process

1. In n8n, select the workflow
2. Click the three-dot menu → Download
3. Save the JSON file
4. Analyze nodes and connections
5. Translate to BASIC script

### JSON Structure Analysis

n8n exports workflows as JSON:

```json
{
  "nodes": [
    {"type": "n8n-nodes-base.httpRequest", "parameters": {...}},
    {"type": "n8n-nodes-base.if", "parameters": {...}}
  ],
  "connections": {...}
}
```

Map each node type to the equivalent BASIC keyword.

## Feature Comparison

### What You Gain

**Native AI integration:**
```basic
USE KB "company-docs"
response = LLM "Analyze this data and provide insights: " + data
```

**Multi-channel chat:**
```basic
TALK "How can I help you?"
HEAR question
answer = LLM question
TALK answer
```

**Built-in productivity:**
```basic
CREATE TASK "Follow up with " + customer_name DUE DATEADD(NOW(), 3, "day")
BOOK "Meeting with " + customer_name AT meeting_time
SEND MAIL TO customer_email SUBJECT "Confirmation" BODY message
```

**Knowledge base:**
```basic
USE KB "product-docs"
USE KB "pricing-info"
answer = LLM customer_question
```

### What Changes

| n8n Approach | General Bots Approach |
|--------------|----------------------|
| Visual drag-and-drop | Text-based BASIC scripts |
| Node connections | Sequential code flow |
| Credentials UI | config.csv settings |
| Execution history UI | Log files + monitoring |
| Community nodes | HTTP keywords + custom code |

## Credentials Migration

### n8n Credentials

n8n stores credentials separately. Export and configure in General Bots:

**config.csv:**
```csv
key,value
slack-webhook-url,https://hooks.slack.com/services/xxx
api-key-clearbit,your-api-key
salesforce-token,your-token
```

**Usage in BASIC:**
```basic
slack_url = GET CONFIG "slack-webhook-url"
POST slack_url, #{text: "Message"}
```

## Migration Checklist

### Pre-Migration

- [ ] Export all n8n workflows as JSON
- [ ] Document active schedules and triggers
- [ ] List all credentials and API keys
- [ ] Identify critical workflows for priority migration
- [ ] Set up General Bots environment

### Migration

- [ ] Translate workflows to BASIC scripts
- [ ] Configure credentials in config.csv
- [ ] Set up webhooks with same endpoints
- [ ] Configure schedules
- [ ] Test each workflow individually

### Post-Migration

- [ ] Run parallel execution for verification
- [ ] Compare outputs between systems
- [ ] Monitor for errors
- [ ] Decommission n8n workflows
- [ ] Document new BASIC scripts

## Common Patterns

### Batch Processing

**n8n:** Split In Batches node

**General Bots:**
```basic
items = GET "https://api.example.com/items"
batch_size = 10
total = LEN(items)

FOR i = 0 TO total - 1 STEP batch_size
    batch = SLICE(items, i, i + batch_size)
    FOR EACH item IN batch
        PROCESS_ITEM(item)
    NEXT item
    WAIT 1000  ' Rate limiting
NEXT i
```

### Conditional Branching

**n8n:** IF node with multiple branches

**General Bots:**
```basic
SWITCH status
    CASE "new"
        HANDLE_NEW()
    CASE "pending"
        HANDLE_PENDING()
    CASE "complete"
        HANDLE_COMPLETE()
    DEFAULT
        HANDLE_UNKNOWN()
END SWITCH
```

### Data Transformation

**n8n:** Set node or Code node

**General Bots:**
```basic
' Transform data
WITH transformed
    .full_name = data.first_name + " " + data.last_name
    .email = LOWER(data.email)
    .created = NOW()
    .source = "api"
END WITH
```

## Best Practices

**Start with simple workflows.** Migrate straightforward automations first to build familiarity with BASIC syntax.

**Use descriptive variable names.** BASIC scripts are more readable than node graphs when well-written.

**Add comments.** Document your scripts for future maintenance:

```basic
' Daily sales report - sends summary to management
' Runs at 6 PM on weekdays
SET SCHEDULE "0 18 * * 1-5"
```

**Leverage native features.** Don't just replicate n8n workflows—take advantage of General Bots' integrated AI, chat, and productivity features.

**Test incrementally.** Verify each migrated workflow before moving to the next.

## See Also

- [SET SCHEDULE](../chapter-06-gbdialog/keyword-set-schedule.md) - Scheduling reference
- [WEBHOOK](../chapter-06-gbdialog/keyword-webhook.md) - Webhook creation
- [HTTP Keywords](../chapter-06-gbdialog/keywords-http.md) - API integration
- [Platform Comparison](./comparison-matrix.md) - Full feature comparison