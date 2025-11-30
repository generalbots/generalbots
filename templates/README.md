# General Bots Templates

The General Bots Templates provide ready-to-use business solutions powered by conversational AI. Each template includes pre-configured dialogs, database schemas, scheduled jobs, webhooks, and tools that can be customized for your specific needs.

## 📁 Template Structure

Each template follows this standard structure:

```
template-name.gbai/
├── template-name.gbdialog/     # BASIC scripts (.bas files)
│   ├── start.bas               # Initial setup, tools, welcome message
│   ├── tables.bas              # Database schema definitions
│   ├── *-jobs.bas              # Scheduled automation jobs
│   └── *.bas                   # Tool implementations
├── template-name.gbot/         # Bot configuration
│   └── config.csv              # Theme, prompts, settings
├── template-name.gbkb/         # Knowledge base content
├── template-name.gbdrive/      # Document templates, assets
└── template-name.gbdata/       # Initial data files
```

## 🏷️ Template Categories

| Category | Icon | Description |
|----------|------|-------------|
| **CRM & Sales** | 💼 | Customer relationships, leads, opportunities, sales pipeline |
| **Operations & ERP** | 🏭 | Inventory, purchasing, supply chain, production |
| **Human Resources** | 👥 | Employees, attendance, leave, recruitment |
| **Finance & Accounting** | 💰 | Invoicing, expenses, budgets, billing |
| **Healthcare & Medical** | 🏥 | Patients, appointments, pharmacy, medical billing |
| **Education & Training** | 🎓 | Students, courses, enrollment, faculty |
| **Real Estate** | 🏠 | Properties, leases, tenants, maintenance |
| **Legal & Compliance** | ⚖️ | Cases, contracts, compliance tracking |
| **Events & Scheduling** | 📅 | Events, room/desk booking, reservations |
| **IT & Support** | 🖥️ | Helpdesk, tickets, assets, bug tracking |
| **Marketing** | 📢 | Campaigns, content, social media, broadcasts |
| **Nonprofit** | 🤝 | Donors, volunteers, memberships, fundraising |
| **AI & Data** | 🤖 | Search, crawling, talk-to-data, LLM tools |

---

## 📋 Available Templates

### 💼 CRM & Sales

| Template | Folder | Description | Key Features |
|----------|--------|-------------|--------------|
| **CRM** | `crm.gbai` | Complete CRM system | Lead management, opportunity tracking, case management, quotes, email campaigns |
| **Store** | `store.gbai` | E-commerce checkout | Product catalog, cart, checkout flow |

### 🏭 Operations & ERP

| Template | Folder | Description | Key Features |
|----------|--------|-------------|--------------|
| **ERP** | `erp.gbai` | Enterprise resource planning | Inventory management, purchasing, warehouse operations |

### 👥 Human Resources

| Template | Folder | Description | Key Features |
|----------|--------|-------------|--------------|
| **Employees** | `hr/employees.gbai` | Employee management system | Directory, onboarding, org chart, emergency contacts, document tracking |

### 🎓 Education & Training

| Template | Folder | Description | Key Features |
|----------|--------|-------------|--------------|
| **Education** | `edu.gbai` | Educational enrollment | Student enrollment, course management, data collection |

### ⚖️ Legal & Compliance

| Template | Folder | Description | Key Features |
|----------|--------|-------------|--------------|
| **Law** | `law.gbai` | Legal case management | Case summaries, document querying, legal research |

### 🖥️ IT & Support

| Template | Folder | Description | Key Features |
|----------|--------|-------------|--------------|
| **Helpdesk** | `it/helpdesk.gbai` | IT support ticketing | Ticket creation, SLA tracking, escalation, webhooks for integration |

### 📢 Marketing & Communications

| Template | Folder | Description | Key Features |
|----------|--------|-------------|--------------|
| **Marketing** | `marketing.gbai` | Marketing automation | Social posting, broadcasts, content ideas |
| **Announcements** | `announcements.gbai` | Company communications | News distribution, scheduled summaries |
| **Broadcast** | `broadcast.gbai` | Message broadcasting | Multi-channel broadcasts |

### 🤖 AI & Data

| Template | Folder | Description | Key Features |
|----------|--------|-------------|--------------|
| **AI Search** | `ai-search.gbai` | Document search & QR | PDF search, QR code scanning, AI summaries |
| **Crawler** | `crawler.gbai` | Website data extraction | Web crawling, knowledge updates |
| **Talk to Data** | `talk-to-data.gbai` | Natural language SQL | Query databases in plain English, charts |
| **LLM Server** | `llm-server.gbai` | LLM as REST API | API generation for LLM access |
| **LLM Tools** | `llm-tools.gbai` | Custom LLM integration | Real-time data access, custom logic |
| **BI** | `bi.gbai` | Business intelligence | Dashboards, analytics |

### 🔧 Utility Templates

| Template | Folder | Description | Key Features |
|----------|--------|-------------|--------------|
| **Default** | `default.gbai` | Base template | Starting point for custom bots |
| **Office** | `office.gbai` | Office automation | Document processing, API integration, data sync |
| **Reminder** | `reminder.gbai` | Reminder system | Scheduled reminders |
| **Backup** | `backup.gbai` | Data backup | Automated backups |
| **API Client** | `api-client.gbai` | API consumption | External API integration |
| **Public APIs** | `public-apis.gbai` | Public API access | Common public API integrations |
| **WhatsApp** | `whatsapp.gbai` | WhatsApp integration | WhatsApp-specific features |

---

## 🔧 Key Components

### start.bas - Template Initialization

Every template should have a `start.bas` that:

1. **Registers Tools** - Makes functions available to the AI
2. **Sets Up Knowledge Base** - Loads relevant KB content
3. **Configures Context** - Sets the AI personality/role
4. **Adds Suggestions** - Provides quick-action buttons
5. **Displays Welcome** - Greets users with capabilities

```basic
' Example start.bas structure
ADD TOOL "create-ticket"
ADD TOOL "search-tickets"

USE KB "helpdesk.gbkb"

SET_CONTEXT "helpdesk" AS "You are an IT support assistant..."

CLEAR_SUGGESTIONS
ADD_SUGGESTION "new" AS "Create new ticket"
ADD_SUGGESTION "status" AS "Check ticket status"

BEGIN TALK
    Welcome to IT Helpdesk!
    How can I help you today?
END TALK

BEGIN SYSTEM PROMPT
    You are an IT support assistant...
END SYSTEM PROMPT
```

### tables.bas - Database Schema

Define your data model using the TABLE keyword:

```basic
TABLE employees
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4()
    first_name VARCHAR(100) NOT NULL
    last_name VARCHAR(100) NOT NULL
    email VARCHAR(255) UNIQUE NOT NULL
    department_id UUID REFERENCES departments(id)
    hire_date DATE NOT NULL
    is_active BOOLEAN DEFAULT TRUE
    created_at TIMESTAMP DEFAULT NOW()
END TABLE
```

### *-jobs.bas - Scheduled Automation

Set up recurring tasks with SET SCHEDULE:

```basic
PARAM job_name AS STRING

IF job_name = "daily_report" THEN
    ' Generate and send daily report
    ...
END IF

IF job_name = "setup_schedules" THEN
    SET SCHEDULE "0 8 * * *" "jobs.bas" "daily_report"
    SET SCHEDULE "0 18 * * *" "jobs.bas" "end_of_day"
    TALK "Schedules configured"
END IF
```

### Webhook Scripts

Expose scripts as HTTP endpoints:

```basic
WEBHOOK "ticket-webhook"

PARAM action AS STRING
PARAM ticket_number AS STRING
...

' Validate API key
api_key = GET "webhook.headers.X-API-Key"
IF api_key != expected_key THEN
    RETURN #{ status: 401, error: "Unauthorized" }
END IF

' Process webhook
IF action = "create" THEN
    ...
END IF
```

### Tool Scripts

Functions that AI can call (registered with ADD TOOL):

```basic
PARAM name AS STRING LIKE "John" DESCRIPTION "Employee name"
PARAM email AS STRING LIKE "john@co.com" DESCRIPTION "Email address"

DESCRIPTION "Creates a new employee record in the system"

' Implementation
employee = CREATE OBJECT
SET employee.name = name
SET employee.email = email
...
SAVE "employees", employee.id, employee

RETURN #{ success: true, employee_id: employee.id }
```

---

## 🚀 Getting Started

### 1. Choose a Template

Browse the categories above and select a template that matches your use case.

### 2. Copy to Your Bot

```bash
cp -r templates/hr/employees.gbai your-bot.gbai
```

### 3. Customize

- Edit `config.csv` for branding (colors, logo, title)
- Modify `tables.bas` for your data model
- Update tools in `.gbdialog/` for your workflow
- Add content to `.gbkb/` for AI knowledge

### 4. Initialize

Run the template's setup job to configure schedules:

```basic
' In conversation or via API
RUN "jobs.bas" WITH job_name = "setup_schedules"
```

---

## 📖 Available Keywords

Templates use these BASIC keywords:

### Data Operations
- `SAVE`, `INSERT`, `UPDATE`, `DELETE`, `MERGE`
- `FIND`, `FILTER`, `MAP`, `JOIN`, `GROUP_BY`
- `IMPORT`, `EXPORT` (CSV, JSON, Excel)
- `FILL` (template filling)

### Communication
- `TALK`, `HEAR` - Conversation
- `SEND MAIL` - Email
- `SEND_SMS` - Text messages
- `BROADCAST` - Multi-recipient

### Scheduling & Tasks
- `SET SCHEDULE` - Cron jobs
- `CREATE_TASK` - Task management
- `BOOK` - Calendar booking

### AI & LLM
- `LLM` - Call language model
- `CALCULATE` - LLM-based calculations
- `VALIDATE` - LLM-based validation
- `TRANSLATE` - Translation
- `SUMMARIZE` - Text summarization
- `EXTRACT_DATA` - Data extraction

### Files & Documents
- `READ`, `WRITE`, `COPY`, `MOVE`
- `GENERATE_PDF`, `MERGE_PDF`
- `COMPRESS`, `EXTRACT`
- `UPLOAD`, `DOWNLOAD`

### Integrations
- `POST`, `PUT`, `PATCH`, `DELETE` - HTTP
- `GRAPHQL`, `SOAP` - API protocols
- `WEBHOOK` - Expose endpoints
- `QR_CODE` - Generate QR codes

### Multimedia
- `IMAGE`, `VIDEO`, `AUDIO` - Generation
- `SEE` - Vision/captioning

---

## 📊 Template Feature Matrix

| Template | Tools | Schedules | Webhooks | KB | Drive |
|----------|:-----:|:---------:|:--------:|:--:|:-----:|
| CRM | ✅ | ✅ | ✅ | ⬜ | ⬜ |
| ERP | ✅ | ✅ | ⬜ | ⬜ | ⬜ |
| Employees | ✅ | ✅ | ⬜ | ⬜ | ✅ |
| Helpdesk | ✅ | ✅ | ✅ | ✅ | ✅ |
| Education | ✅ | ⬜ | ⬜ | ⬜ | ⬜ |
| AI Search | ✅ | ⬜ | ⬜ | ✅ | ✅ |
| Announcements | ✅ | ✅ | ⬜ | ✅ | ⬜ |
| Marketing | ✅ | ⬜ | ⬜ | ⬜ | ⬜ |

---

## 🔗 Resources

- [Full Documentation](https://docs.pragmatismo.com.br)
- [BASIC Language Reference](https://docs.pragmatismo.com.br/basic)
- [API Reference](https://docs.pragmatismo.com.br/api)

---

## 📝 Creating Custom Templates

1. Start from `default.gbai` or copy an existing template
2. Define your data model in `tables.bas`
3. Create tools for your business logic
4. Set up scheduled jobs for automation
5. Add webhooks for external integrations
6. Configure `start.bas` for initialization
7. Add knowledge base content for AI context

---

*General Bots Templates - Conversational AI for Business*