# Bot Templates

botserver includes pre-built bot templates for various use cases. Each template is a complete `.gbai` package ready to deploy.

---

## Complete Template List (Flat Reference)

| # | Template | Category | Folder | Key Features |
|---|----------|----------|--------|--------------|
| 1 | Default | Core | `default.gbai` | Minimal starter bot |
| 2 | Template | Core | `template.gbai` | Reference implementation |
| 3 | Announcements | Communications | `announcements.gbai` | Company news, multiple KB |
| 4 | AI Search | Search | `ai-search.gbai` | QR codes, document search |
| 5 | API Client | Integration | `api-client.gbai` | REST API patterns |
| 6 | Backup | Administration | `backup.gbai` | Server backup scripts |
| 7 | BI | Analytics | `bi.gbai` | Dashboards, role separation |
| 8 | Broadcast | Communications | `broadcast.gbai` | Mass messaging |
| 9 | Crawler | Search | `crawler.gbai` | Web indexing |
| 10 | CRM | Sales | `crm.gbai` | Customer management |
| 11 | Education | Education | `edu.gbai` | Course management |
| 12 | ERP | Operations | `erp.gbai` | Process automation |
| 13 | Law | Legal | `law.gbai` | Document templates |
| 14 | LLM Server | AI | `llm-server.gbai` | Model hosting |
| 15 | LLM Tools | AI | `llm-tools.gbai` | Prompt engineering |
| 16 | Marketing | Marketing | `marketing.gbai` | Campaign tools |
| 17 | Public APIs | Integration | `public-apis.gbai` | Weather, news APIs |
| 18 | Reminder | Productivity | `reminder.gbai` | Task reminders |
| 19 | Store | E-commerce | `store.gbai` | Product catalog |
| 20 | Talk to Data | Analytics | `talk-to-data.gbai` | Natural language SQL |
| 21 | WhatsApp | Messaging | `whatsapp.gbai` | WhatsApp Business |
| 22 | Office | Productivity | `office.gbai` | Document processing |
| 23 | Employee Management | HR | `hr/employees.gbai` | Employee CRUD |
| 24 | IT Helpdesk | IT | `it/helpdesk.gbai` | Ticket management |
| 25 | Sales Pipeline | CRM | `crm/sales-pipeline.gbai` | Deal tracking |
| 26 | Contact Directory | CRM | `crm/contacts.gbai` | Contact management |

---

## Templates by Category

### Core Templates

| Template | Folder | Purpose |
|----------|--------|---------|
| Default | `default.gbai` | Minimal starter bot for learning |
| Template | `template.gbai` | Complete example structure |

### HR & People

| Template | Folder | Key Files |
|----------|--------|-----------|
| Employee Management | `hr/employees.gbai` | `start.bas`, `add-employee.bas`, `search-employee.bas` |
| Leave Management | `hr/leave.gbai` | `start.bas`, `request-leave.bas`, `approve-leave.bas` |
| Recruitment | `hr/recruitment.gbai` | `start.bas`, `post-job.bas`, `add-applicant.bas` |

### IT & Support

| Template | Folder | Key Files |
|----------|--------|-----------|
| IT Helpdesk | `it/helpdesk.gbai` | `start.bas`, `create-ticket.bas`, `update-ticket.bas` |
| Asset Tracking | `it/assets.gbai` | `start.bas`, `add-asset.bas`, `checkout-asset.bas` |

### CRM & Sales

| Template | Folder | Key Files |
|----------|--------|-----------|
| CRM | `crm.gbai` | `lead-management.bas`, `opportunity-management.bas` |
| Sales Pipeline | `crm/sales-pipeline.gbai` | `start.bas`, `create-deal.bas`, `update-stage.bas` |
| Contact Directory | `crm/contacts.gbai` | `start.bas`, `add-contact.bas`, `search-contact.bas` |

### Finance & Accounting

| Template | Folder | Key Files |
|----------|--------|-----------|
| Invoicing | `finance/invoicing.gbai` | `start.bas`, `create-invoice.bas`, `send-reminder.bas` |
| Expense Tracker | `finance/expenses.gbai` | `start.bas`, `submit-expense.bas`, `approve-expense.bas` |

### Operations

| Template | Folder | Key Files |
|----------|--------|-----------|
| ERP | `erp.gbai` | Process automation, integrations |
| Warehouse | `operations/warehouse.gbai` | `start.bas`, `receive-stock.bas`, `ship-order.bas` |

---

## Template Structure

All templates follow this standard directory layout:

```
template-name.gbai/
  template-name.gbdialog/    # BASIC dialog scripts
    start.bas                # Entry point (required)
    *.bas                    # Tool scripts (auto-discovered)
    *-jobs.bas               # Scheduled jobs
  template-name.gbkb/        # Knowledge base collections
    collection1/             # Documents for USE KB "collection1"
  template-name.gbdrive/     # File storage (not KB)
    uploads/                 # User uploaded files
    exports/                 # Generated files
  template-name.gbot/        # Configuration
    config.csv               # Bot parameters
  template-name.gbtheme/     # UI theme (optional)
    default.css              # Theme CSS
```

---

## Quick Start Guide

### 1. Choose a Template

Select based on your needs:
- **Simple chat**: Use `default.gbai`
- **Business app**: Choose `crm.gbai`, `bi.gbai`, or `erp.gbai`
- **AI features**: Pick `ai-search.gbai` or `llm-tools.gbai`
- **Communication**: Select `broadcast.gbai` or `whatsapp.gbai`

### 2. Deploy the Template

```bash
# Templates are auto-deployed during bootstrap
# Access at: http://localhost:9000/template-name
```

### 3. Customize Configuration

Edit `template-name.gbot/config.csv`:

```csv
name,value
bot-name,My Custom Bot
welcome-message,Hello! How can I help?
llm-model,model.gguf
temperature,0.7
```

### 4. Add Knowledge Base

Place documents in `.gbkb` folders:
- Each folder becomes a collection
- Use `USE KB "folder-name"` in scripts
- Documents are automatically indexed

### 5. Create Tools

Add `.bas` files to `.gbdialog`:
- Each file becomes a tool
- Auto-discovered by the system
- Called automatically by LLM when needed

---

## Required Files for Each Template

### start.bas (Required)

```basic
' Template Name - Start Script

' Setup Tools
ADD TOOL "tool-name-1"
ADD TOOL "tool-name-2"

' Setup Knowledge Base
USE KB "template-name.gbkb"

' Set Context
SET CONTEXT "context name" AS "You are a [role]. You help with [tasks]."

' Setup Suggestions
CLEAR SUGGESTIONS
ADD SUGGESTION "action1" AS "Display text 1"
ADD SUGGESTION "action2" AS "Display text 2"

' Welcome Message
BEGIN TALK
    **Template Title**
    
    Welcome message here.
    
    **What I can help with:**
    • Feature 1
    • Feature 2
END TALK

BEGIN SYSTEM PROMPT
    Detailed instructions for the AI...
END SYSTEM PROMPT
```

### Tool File Template

```basic
PARAM paramname AS STRING LIKE "example" DESCRIPTION "What this parameter is"
PARAM optionalparam AS STRING LIKE "default" DESCRIPTION "Optional parameter"

DESCRIPTION "What this tool does. Called when user wants to [action]."

' Business logic
let result = "processed"

' Save data (field names = variable names)
SAVE "table.csv", paramname, optionalparam, result

' Store in memory
SET BOT MEMORY "last_item", result

' Response
TALK "✅ Action completed successfully!"
```

### config.csv Template

```csv
name,value
episodic-memory-history,2
episodic-memory-threshold,4
theme-color1,#1565C0
theme-color2,#E3F2FD
theme-logo,https://pragmatismo.com.br/icons/general-bots.svg
theme-title,Template Name - General Bots
```

---

## Syntax Rules for Templates

### DO ✅

```basic
' Variable names (no underscores in names)
let ticketnumber = "TKT001"
let useremail = "user@example.com"

' SAVE with field names = variable names
SAVE "table.csv", ticketnumber, useremail, status

' Keywords with spaces
SET BOT MEMORY "last_ticket", ticketnumber
SET CONTEXT "name" AS "description"
ADD SUGGESTION "key" AS "Display text"
CLEAR SUGGESTIONS
USE KB "myknowledge"
USE TOOL "mytool"

' GET BOT MEMORY as function
let lastticket = GET BOT MEMORY("last_ticket")
```

### DON'T ❌

```basic
' NO: Complex object operations
SET object.field = value  ' WRONG
SAVE "table", object.id, object  ' WRONG
```

---

## Creating Custom Templates

To create your own template:

1. **Copy `template.gbai`** as starting point
2. **Define clear purpose** - one template, one job
3. **Structure folders** properly:
   - `.gbdialog` for scripts
   - `.gbkb` for knowledge collections
   - `.gbdrive` for general files
   - `.gbot` for configuration
4. **Include examples** - sample data and dialogs
5. **Test thoroughly** - verify all features

---

## Best Practices

### Template Selection

1. **Start small**: Begin with `default.gbai`
2. **Match use case**: Choose aligned templates
3. **Combine features**: Mix templates as needed
4. **Keep originals**: Copy before modifying

### Customization Strategy

#### Minimal BASIC Approach
Instead of complex dialog flows, use simple LLM calls:

```basic
' Let system AI handle conversations naturally
TALK "How can I assist you?"
' System AI understands and responds appropriately
```

#### Tool Creation
Only create `.bas` files for specific actions:
- API calls
- Database operations
- File processing
- Calculations

#### Knowledge Base Organization
- One folder per topic/collection
- Name folders clearly
- Keep documents updated
- Index automatically

### Performance Tips

- Remove unused template files
- Index only necessary documents
- Configure appropriate cache settings
- Monitor resource usage

---

## Support Resources

- README files in each template folder
- Example configurations included
- Sample knowledge bases provided
- Community forums for discussions