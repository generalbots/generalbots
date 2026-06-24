# AutoTask LLM Executor - Prompt Guide

**Version:** 6.2.0
**Purpose:** Guide LLM to generate and execute automated tasks using BASIC scripts

---

## System Overview

AutoTask is an AI-driven task execution system that:

1. **Analyzes user intent** - "Send email to all customers", "Create weekly report"
2. **Plans execution steps** - Break down into actionable tasks
3. **Generates BASIC scripts** - Using available keywords to accomplish the task
4. **Executes scripts** - Run immediately or schedule for later

### This is NOT just for app creation!

AutoTask handles ANY automation:
- Send emails to customer lists
- Generate reports from database
- Create documents in .gbdrive
- Schedule recurring tasks
- Process data transformations
- Integrate with external APIs

---

## File Locations

```
.gbdrive/
├── reports/              # Generated reports
├── documents/            # Created documents
├── exports/              # Data exports
└── apps/{appname}/       # HTMX apps (synced to SITES_ROOT)

.gbdialog/
├── schedulers/           # Scheduled jobs (cron-based)
├── tools/                # Voice/chat triggered tools
└── handlers/             # Event handlers
```

---

## Execution Flow

```
User Intent
     │
     ▼
┌─────────────────┐
│ Phase 1: Plan   │  LLM analyzes intent, creates step list
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Phase 2: Generate│  LLM generates BASIC code for each step
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Phase 3: Execute │  BASIC interpreter runs the scripts
└─────────────────┘
```

---

## Complete BASIC Keywords Reference

### Data Operations

| Keyword | Syntax | Description |
|---------|--------|-------------|
| `GET` | `GET FROM {table} WHERE {condition}` | Query database records |
| `SET` | `SET {variable} = {value}` | Set variable value |
| `SAVE` | `SAVE {data} TO {table}` | Insert/update database record |
| `FIND` | `FIND {value} IN {table}` | Search for specific value |
| `FIRST` | `FIRST({array})` | Get first element |
| `LAST` | `LAST({array})` | Get last element |
| `FORMAT` | `FORMAT "{template}", var1, var2` | Format string with variables |

### Communication

| Keyword | Syntax | Description |
|---------|--------|-------------|
| `SEND MAIL` | `SEND MAIL TO "{email}" WITH subject, body` | Send email |
| `SEND TEMPLATE` | `SEND TEMPLATE "{name}" TO "{email}"` | Send email template |
| `SEND SMS` | `SEND SMS TO "{phone}" MESSAGE "{text}"` | Send SMS |
| `TALK` | `TALK "{message}"` | Respond to user |
| `HEAR` | `HEAR "{phrase}" AS {variable}` | Listen for user input |

### File Operations

| Keyword | Syntax | Description |
|---------|--------|-------------|
| `CREATE FILE` | `CREATE FILE "{path}" WITH {content}` | Create file in .gbdrive |
| `READ FILE` | `READ FILE "{path}"` | Read file content |
| `WRITE FILE` | `WRITE FILE "{path}" WITH {content}` | Write to file |
| `DELETE FILE` | `DELETE FILE "{path}"` | Delete file |
| `COPY FILE` | `COPY FILE "{source}" TO "{dest}"` | Copy file |
| `MOVE FILE` | `MOVE FILE "{source}" TO "{dest}"` | Move/rename file |
| `LIST FILES` | `LIST FILES "{path}"` | List directory contents |
| `UPLOAD` | `UPLOAD {data} TO "{path}"` | Upload file |
| `DOWNLOAD` | `DOWNLOAD "{url}" TO "{path}"` | Download file |

### HTTP Operations

| Keyword | Syntax | Description |
|---------|--------|-------------|
| `GET HTTP` | `GET HTTP "{url}"` | HTTP GET request |
| `POST HTTP` | `POST HTTP "{url}" WITH {data}` | HTTP POST request |
| `PUT HTTP` | `PUT HTTP "{url}" WITH {data}` | HTTP PUT request |
| `DELETE HTTP` | `DELETE HTTP "{url}"` | HTTP DELETE request |
| `WEBHOOK` | `WEBHOOK "{url}" WITH {data}` | Send webhook |

### AI/LLM Operations

| Keyword | Syntax | Description |
|---------|--------|-------------|
| `LLM` | `LLM "{prompt}"` | Call LLM with prompt |
| `USE KB` | `USE KB "{knowledge_base}"` | Use knowledge base for context |
| `CLEAR KB` | `CLEAR KB` | Clear knowledge base context |
| `USE TOOL` | `USE TOOL "{tool_name}"` | Enable external tool |
| `CLEAR TOOLS` | `CLEAR TOOLS` | Disable all tools |
| `USE WEBSITE` | `USE WEBSITE "{url}"` | Scrape website for context |

### Task & Scheduling

| Keyword | Syntax | Description |
|---------|--------|-------------|
| `CREATE_TASK` | `CREATE_TASK "{title}", "{assignee}", "{due}", {project}` | Create task |
| `WAIT` | `WAIT {seconds}` | Pause execution |
| `ON` | `ON "{event}" DO {action}` | Event handler |
| `ON EMAIL` | `ON EMAIL FROM "{filter}" DO {action}` | Email trigger |
| `ON CHANGE` | `ON CHANGE {table} DO {action}` | Database change trigger |

### Bot & Memory

| Keyword | Syntax | Description |
|---------|--------|-------------|
| `SET BOT MEMORY` | `SET BOT MEMORY "{key}" = {value}` | Store bot-level data |
| `GET BOT MEMORY` | `GET BOT MEMORY "{key}"` | Retrieve bot-level data |
| `REMEMBER` | `REMEMBER "{key}" = {value}` | Store session data |
| `SET CONTEXT` | `SET CONTEXT "{key}" = {value}` | Set conversation context |
| `ADD SUGGESTION` | `ADD SUGGESTION "{text}"` | Add response suggestion |
| `CLEAR SUGGESTIONS` | `CLEAR SUGGESTIONS` | Clear suggestions |

### User & Session

| Keyword | Syntax | Description |
|---------|--------|-------------|
| `SET USER` | `SET USER "{property}" = {value}` | Update user property |
| `TRANSFER TO HUMAN` | `TRANSFER TO HUMAN` | Escalate to human agent |
| `ADD_MEMBER` | `ADD_MEMBER "{group}", "{email}", "{role}"` | Add user to group |

### Documents & Content

| Keyword | Syntax | Description |
|---------|--------|-------------|
| `CREATE DRAFT` | `CREATE DRAFT "{title}" WITH {content}` | Create document draft |
| `CREATE SITE` | `CREATE SITE "{name}" WITH {config}` | Create website |
| `SAVE FROM UNSTRUCTURED` | `SAVE FROM UNSTRUCTURED {data} TO {table}` | Parse and save data |

### Multi-Bot Operations

| Keyword | Syntax | Description |
|---------|--------|-------------|
| `ADD BOT` | `ADD BOT "{name}" WITH TRIGGER "{phrase}"` | Add sub-bot |
| `REMOVE BOT` | `REMOVE BOT "{name}"` | Remove sub-bot |
| `LIST BOTS` | `LIST BOTS` | List active bots |
| `DELEGATE TO` | `DELEGATE TO "{bot}"` | Delegate to another bot |
| `SEND TO BOT` | `SEND TO BOT "{name}" MESSAGE "{msg}"` | Inter-bot message |
| `BROADCAST MESSAGE` | `BROADCAST MESSAGE "{msg}"` | Broadcast to all bots |

### Social Media

| Keyword | Syntax | Description |
|---------|--------|-------------|
| `POST TO SOCIAL` | `POST TO SOCIAL "{platform}" MESSAGE "{text}"` | Social media post |
| `GET SOCIAL FEED` | `GET SOCIAL FEED "{platform}"` | Get social feed |

### Control Flow

| Keyword | Syntax | Description |
|---------|--------|-------------|
| `IF/THEN/ELSE/END IF` | `IF condition THEN ... ELSE ... END IF` | Conditional |
| `FOR EACH/NEXT` | `FOR EACH item IN collection ... NEXT` | Loop |
| `SWITCH/CASE/END SWITCH` | `SWITCH var CASE val ... END SWITCH` | Switch statement |
| `PRINT` | `PRINT {value}` | Debug output |

### Built-in Variables

| Variable | Description |
|----------|-------------|
| `TODAY` | Current date |
| `NOW` | Current datetime |
| `USER` | Current user object |
| `SESSION` | Current session object |
| `BOT` | Current bot object |

---

## Planning Prompt Template

Use this prompt for Phase 1 (intent analysis):

```
You are a task automation planner. Analyze the user's request and create an execution plan.

USER REQUEST: {user_intent}

Available capabilities:
- Query/modify database tables
- Send emails (individual or bulk)
- Create/read/write files in .gbdrive
- Make HTTP requests to external APIs
- Generate documents and reports
- Schedule tasks for later execution
- Process data with LLM

Create a JSON response:
{
  "intent": "original request",
  "task_type": "immediate|scheduled|triggered",
  "description": "what this task accomplishes",
  "requires_confirmation": true/false,
  "steps": [
    {
      "order": 1,
      "action": "description of step",
      "type": "query|email|file|http|llm|schedule",
      "details": { step-specific configuration }
    }
  ],
  "schedule": "cron expression if scheduled",
  "trigger": "trigger phrase if voice/chat activated"
}
```

---

## BASIC Generation Prompt Template

Use this prompt for Phase 2 (code generation):

```
Generate a BASIC script for: {step_description}

Context:
- Bot ID: {bot_id}
- User: {user_info}
- Available tables: {tables}
- Previous step output: {previous_output}

Requirements:
- Use only valid BASIC keywords listed in the reference
- Handle errors gracefully
- Log important actions
- Return meaningful results

Output ONLY the BASIC code, no explanation.
```

---

## Example Tasks

### Example 1: Send Email to All Customers

**User Intent:** "Send a holiday greeting to all my customers"

**Generated BASIC:**
```basic
' Task: Send holiday greeting to all customers
' Generated by AutoTask

customers = GET FROM customers WHERE status = "active"

subject = "Happy Holidays from Our Team!"
body = "Dear valued customer,\n\nWishing you a wonderful holiday season..."

FOR EACH customer IN customers
  SEND MAIL TO customer.email WITH subject, body
  WAIT 1
NEXT

TALK "Sent holiday greetings to " + COUNT(customers) + " customers"
```

### Example 2: Generate Weekly Report

**User Intent:** "Create a weekly sales report every Monday"

**Generated BASIC (saved to .gbdialog/schedulers/weekly-report.bas):**
```basic
' Scheduler: weekly_sales_report
' Schedule: 0 9 * * 1 (Monday 9 AM)

orders = GET FROM orders WHERE created_at > TODAY - 7

total_revenue = 0
total_orders = COUNT(orders)

FOR EACH order IN orders
  total_revenue = total_revenue + order.total
NEXT

report = "# Weekly Sales Report\n"
report = report + "Period: " + (TODAY - 7) + " to " + TODAY + "\n\n"
report = report + "Total Orders: " + total_orders + "\n"
report = report + "Total Revenue: $" + total_revenue + "\n"

CREATE FILE "reports/weekly-sales-" + TODAY + ".md" WITH report

SEND MAIL TO "manager@company.com" WITH "Weekly Sales Report", report
```

### Example 3: Voice-Triggered Tool

**User Intent:** "When I say 'check inventory', show me low stock items"

**Generated BASIC (saved to .gbdialog/tools/check-inventory.bas):**
```basic
' Tool: check_inventory
' Trigger: "check inventory"

HEAR "check inventory" AS request

items = GET FROM inventory WHERE quantity < reorder_level

IF COUNT(items) = 0 THEN
  TALK "All inventory levels are healthy!"
ELSE
  response = "Found " + COUNT(items) + " items low on stock:\n\n"
  FOR EACH item IN items
    response = response + "- " + item.name + ": " + item.quantity + " left\n"
  NEXT
  TALK response
END IF
```

### Example 4: Create Document from Data

**User Intent:** "Create an invoice for order 12345"

**Generated BASIC:**
```basic
' Task: Generate invoice for order

order = GET FROM orders WHERE id = "12345"
customer = GET FROM customers WHERE id = order.customer_id
items = GET FROM order_items WHERE order_id = order.id

invoice = "# INVOICE\n\n"
invoice = invoice + "Invoice #: INV-" + order.id + "\n"
invoice = invoice + "Date: " + TODAY + "\n\n"
invoice = invoice + "Bill To:\n"
invoice = invoice + customer.name + "\n"
invoice = invoice + customer.address + "\n\n"
invoice = invoice + "## Items\n\n"

total = 0
FOR EACH item IN items
  invoice = invoice + "- " + item.name + " x" + item.quantity
  invoice = invoice + " @ $" + item.price + " = $" + (item.quantity * item.price) + "\n"
  total = total + (item.quantity * item.price)
NEXT

invoice = invoice + "\n**Total: $" + total + "**\n"

CREATE FILE "invoices/INV-" + order.id + ".md" WITH invoice

TALK "Invoice created for order " + order.id
```

---

## Decision Points

Some tasks may require user confirmation:

```json
{
  "type": "decision",
  "question": "This will send emails to 1,234 customers. Proceed?",
  "options": [
    {"id": "proceed", "label": "Yes, send to all"},
    {"id": "test", "label": "Send test to me first"},
    {"id": "cancel", "label": "Cancel"}
  ]
}
```

---

## Error Handling

Scripts should handle errors:

```basic
' Good error handling example

result = GET FROM customers WHERE id = customer_id

IF result = NULL THEN
  TALK "Customer not found"
ELSE
  ' Process customer
END IF
```

---

## Remember

- **AutoTask is for ANY automation**, not just app creation
- **Use real BASIC keywords** from the reference above
- **Files go to .gbdrive** (documents, reports, exports)
- **Scripts go to .gbdialog** (schedulers, tools, handlers)
- **Always handle errors** gracefully
- **Confirm destructive actions** (bulk emails, deletes)
- **Log important operations** for audit trail