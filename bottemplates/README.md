# General Bots Templates

Pre-built bot packages for common business use cases. Templates are organized by category for easy discovery.

---

## Complete Template List (Flat Reference)

| # | Template | Category | Folder | Key Features |
|---|----------|----------|--------|--------------|
| 1 | Default | Core | `core/default.gbai` | Minimal starter bot |
| 2 | Template | Core | `core/template.gbai` | Reference implementation |
| 3 | AI Search | Core | `core/ai-search.gbai` | QR codes, document search |
| 4 | Analytics Dashboard | Core | `core/analytics-dashboard.gbai` | Metrics, Reports |
| 5 | Analytics | Core | `core/analytics.gbai` | Metrics, Reports |
| 6 | API Client | Core | `core/api-client.gbai` | Generic API client |
| 7 | Backup | Core | `core/backup.gbai` | Server backup scripts |
| 8 | BI | Core | `core/bi.gbai` | Dashboards, role separation |
| 9 | Bling | Core | `core/bling.gbai` | Bling ERP integration |
| 10 | Crawler | Core | `core/crawler.gbai` | Web indexing |
| 11 | Public APIs | Core | `core/public-apis.gbai` | Public API examples |
| 12 | Store Server | Core | `core/store-server.gbai` | Store management |
| 13 | Talk to Data | Core | `core/talk-to-data.gbai` | Natural language SQL |
| 14 | VectorDB Stats | Core | `core/vectordb-statistics.gbai` | VectorDB monitoring |
| 15 | Content Moderation | AI | `ai/content-moderation.gbai` | Moderation workflows |
| 16 | Customer Support | AI | `ai/customer-support-workflow.gbai` | Support multiagent |
| 17 | LLM Server | AI | `ai/llm-server.gbai` | Model hosting |
| 18 | LLM Tools | AI | `ai/llm-tools.gbai` | TOOL-based LLM examples |
| 19 | Marketing Campaign | AI | `ai/marketing-campaign.gbai` | Campaign multiagent |
| 20 | Order Processing | AI | `ai/order-processing.gbai` | Order workflows |
| 21 | Announcements | Communications | `communications/announcements.gbai` | Company news, multiple KB |
| 22 | Broadcast | Communications | `communications/broadcast.gbai` | Mass messaging |
| 23 | Quick Broadcast | Communications | `communications/quick-broadcast.gbai` | Simple messaging |
| 24 | Office | Communications | `communications/office.gbai` | Office automation |
| 25 | Reminder | Communications | `communications/reminder.gbai` | Scheduled alerts |
| 26 | WhatsApp | Communications | `communications/whatsapp.gbai` | WhatsApp Business |
| 27 | Education | Education | `education/edu.gbai` | Course management |
| 28 | Bank | Finance | `finance/bank.gbai` | Banking services |
| 29 | Helpdesk | IT | `it/helpdesk.gbai` | IT helpdesk ticketing |
| 30 | Employee Engage | HR | `hr/employee-engage.gbai` | Engagement surveys |
| 31 | Employees | HR | `hr/employees.gbai` | Employee management |
| 32 | Team Feedback | HR | `hr/team-feedback.gbai` | Feedback collection |
| 33 | HIPAA Medical | Legal | `legal/hipaa-medical.gbai` | HIPAA, HITECH |
| 34 | Law | Legal | `legal/law.gbai` | Document templates |
| 35 | Privacy | Legal | `legal/privacy.gbai` | LGPD, GDPR, CCPA |
| 36 | ERP | Operations | `operations/erp.gbai` | Process automation |
| 37 | Attendance CRM | Sales | `sales/attendance-crm.gbai` | Event attendance tracking |
| 38 | Attendance | Sales | `sales/attendance.gbai` | Simple attendance |
| 39 | Campaign Manager | Sales | `sales/campaign-manager.gbai` | Campaign tools |
| 40 | Contacts | Sales | `sales/contacts.gbai` | Contact management |
| 41 | CRM | Sales | `sales/crm.gbai` | Customer management |
| 42 | Marketing | Sales | `sales/marketing.gbai` | Marketing automation |
| 43 | Sales Deals | Sales | `sales/sales-deals.gbai` | Deals CRM |
| 44 | Sales Pipeline | Sales | `sales/sales-pipeline.gbai` | Sales workflow |
| 45 | Store | Sales | `sales/store.gbai` | Product catalog |

---

## Categories

### `/core`
Platform administration, integrations, and core references.

| Template | Description |
|----------|-------------|
| `default.gbai` | Starter template |
| `template.gbai` | Reference for new templates |
| `ai-search.gbai` | Document search (RAG, Semantic) |
| `analytics-dashboard.gbai` | Platform analytics bot |
| `analytics.gbai` | Platform analytics |
| `api-client.gbai` | Generic API client |
| `backup.gbai` | Server backup scripts |
| `bi.gbai` | Business intelligence |
| `bling.gbai` | Bling ERP integration |
| `crawler.gbai` | Web crawler (Indexing, Scraping) |
| `public-apis.gbai` | Public API examples |
| `store-server.gbai` | Store server logic |
| `talk-to-data.gbai` | Data queries (Natural language SQL) |
| `vectordb-statistics.gbai` | VectorDB metrics |

### `/ai`
Artificial Intelligence, multiagent workflows, and LLM servers.

| Template | Description |
|----------|-------------|
| `llm-server.gbai` | Model hosting |
| `llm-tools.gbai` | LLM tools / Function calling examples |
| `content-moderation.gbai` | Multiagent content moderation |
| `customer-support-workflow.gbai` | Support workflow automation |
| `marketing-campaign.gbai` | Automated marketing campaigns |
| `order-processing.gbai` | Order processing workflow |

### `/communications`
Messaging, productivity, and office automation tools.

| Template | Description |
|----------|-------------|
| `whatsapp.gbai` | WhatsApp Business |
| `announcements.gbai` | Company announcements |
| `broadcast.gbai` | Message broadcasting |
| `quick-broadcast.gbai` | Simple broadcasting alternative |
| `office.gbai` | Office automation |
| `reminder.gbai` | Reminder system |

### `/education`
Education and training management.

| Template | Description |
|----------|-------------|
| `edu.gbai` | Course management |

### `/finance`
Banking and financial services.

| Template | Description |
|----------|-------------|
| `bank.gbai` | Banking services |

### `/hr`
Human resources, team feedback, and surveys.

| Template | Description |
|----------|-------------|
| `employees.gbai` | Employee management |
| `employee-engage.gbai` | Engagement surveys |
| `team-feedback.gbai` | Team feedback collection |

### `/it`
IT service management and helpdesk.

| Template | Description |
|----------|-------------|
| `helpdesk.gbai` | IT helpdesk ticketing |

### `/legal`
Privacy, legal processing, and regulatory compliance.

| Template | Description |
|----------|-------------|
| `law.gbai` | Legal document processing |
| `privacy.gbai` | Data subject rights portal (LGPD, GDPR) |
| `hipaa-medical.gbai` | Healthcare privacy management |

### `/operations`
Operational automation and ERP.

| Template | Description |
|----------|-------------|
| `erp.gbai` | ERP integration |

### `/sales`
Customer relationship, marketing, and e-commerce templates.

| Template | Description |
|----------|-------------|
| `crm.gbai` | Full CRM system |
| `sales-deals.gbai` | Deals-focused CRM |
| `marketing.gbai` | Marketing automation |
| `campaign-manager.gbai` | Campaign manager tools |
| `attendance-crm.gbai` | Event attendance tracking |
| `attendance.gbai` | Simple attendance |
| `sales-pipeline.gbai` | Sales workflow |
| `contacts.gbai` | Contact management |
| `store.gbai` | E-commerce product catalog |

---

## Template Structure

Each `.gbai` template follows this structure:

```
template-name.gbai/
├── README.md                 # Template documentation
├── template-name.gbdialog/   # BASIC dialog scripts
│   ├── start.bas            # Entry point
│   └── *.bas                # Additional dialogs (auto-discovered as TOOLs)
├── template-name.gbot/       # Bot configuration
│   └── config.csv           # Settings
├── template-name.gbkb/       # Knowledge base (optional)
│   └── docs/                # Documents for RAG
├── template-name.gbdrive/    # File storage (optional)
└── template-name.gbui/       # Custom UI (optional)
    └── index.html
```

---

## Event-Driven Patterns

Templates should use **ON** triggers instead of polling loops:

```basic
' ❌ OLD WAY - Polling (avoid)
mainLoop:
    leads = FIND "leads", "processed = false"
    WAIT 5
GOTO mainLoop

' ✅ NEW WAY - Event-Driven
ON INSERT OF "leads"
    lead = GET LAST "leads"
    score = SCORE LEAD lead
    TALK TO "whatsapp:" + sales_phone, "New lead: " + lead.name
END ON
```

---

## TOOL-Based LLM Integration

Every `.bas` file with `PARAM` and `DESCRIPTION` becomes an LLM-invokable tool:

```basic
' score-lead.bas
PARAM email AS STRING LIKE "john@company.com" DESCRIPTION "Lead email"
PARAM name AS STRING LIKE "John Smith" DESCRIPTION "Lead name"

DESCRIPTION "Score a new lead. Use when user mentions a prospect."

lead = NEW OBJECT
lead.email = email
lead.name = name

score = AI SCORE LEAD lead

IF score.status = "hot" THEN
    TALK TO "whatsapp:+5511999887766", "🔥 Hot lead: " + name
END IF

TALK "Lead scored: " + score.score + "/100"
```

---

## Installation

### From Console

```bash
botserver --install-template sales/crm
```

### From BASIC

```basic
INSTALL TEMPLATE "sales/crm"
```

### Manual

Copy the template folder to your bot's packages directory:

```bash
cp -r templates/sales/crm.gbai /path/to/your/bot/packages/
```

---

## Creating Custom Templates

1. Copy `core/template.gbai` as a starting point
2. Rename the folder to `your-template.gbai`
3. Update internal folder names to match
4. Edit `config.csv` with your bot settings
5. Create dialog scripts in the `.gbdialog` folder
6. Use **ON** triggers instead of polling loops
7. Add `PARAM` and `DESCRIPTION` to make scripts LLM-invokable
8. Add documentation in `README.md`

---

## License

All templates are licensed under AGPL-3.0 as part of General Bots.

---

**Pragmatismo** - General Bots Open Source Platform
