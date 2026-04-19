# General Bots Templates

Pre-built bot packages for common business use cases. Templates are organized by category for easy discovery.

---

## Complete Template List (Flat Reference)

| # | Template | Category | Folder | Key Features |
|---|----------|----------|--------|--------------|
| 1 | Default | Core | `default.gbai` | Minimal starter bot |
| 2 | Template | Core | `template.gbai` | Reference implementation |
| 3 | AI Search | Search | `ai-search.gbai` | QR codes, document search |
| 4 | Announcements | Communications | `announcements.gbai` | Company news, multiple KB |
| 5 | Analytics Dashboard | Platform | `analytics-dashboard.gbai` | Metrics, Reports |
| 6 | Backup | Platform | `backup.gbai` | Server backup scripts |
| 7 | Bank | Finance | `bank.gbai` | Banking services |
| 8 | BI | Platform | `bi.gbai` | Dashboards, role separation |
| 9 | Bling | Integration | `bling.gbai` | Bling ERP integration |
| 10 | Broadcast | Communications | `broadcast.gbai` | Mass messaging |
| 11 | Crawler | Search | `crawler.gbai` | Web indexing |
| 12 | CRM | Sales | `sales/crm.gbai` | Customer management |
| 13 | Attendance CRM | Sales | `sales/attendance-crm.gbai` | Event attendance tracking |
| 14 | Marketing | Sales | `sales/marketing.gbai` | Campaign tools |
| 15 | Education | Education | `edu.gbai` | Course management |
| 16 | ERP | Operations | `erp.gbai` | Process automation |
| 17 | HIPAA Medical | Compliance | `compliance/hipaa-medical.gbai` | HIPAA, HITECH |
| 18 | Privacy | Compliance | `compliance/privacy.gbai` | LGPD, GDPR, CCPA |
| 19 | Law | Legal | `law.gbai` | Document templates |
| 20 | LLM Server | AI | `llm-server.gbai` | Model hosting |
| 21 | LLM Tools | AI | `llm-tools.gbai` | TOOL-based LLM examples |
| 22 | Store | E-commerce | `store.gbai` | Product catalog |
| 23 | Talk to Data | Platform | `talk-to-data.gbai` | Natural language SQL |
| 24 | WhatsApp | Messaging | `whatsapp.gbai` | WhatsApp Business |

---

## Categories

### `/sales`
Customer relationship and marketing templates.

| Template | Description | Features |
|----------|-------------|----------|
| `crm.gbai` | Full CRM system | Leads, Contacts, Accounts, Opportunities |
| `marketing.gbai` | Marketing automation | Campaigns, Lead capture, Email sequences |
| `attendance-crm.gbai` | Event attendance | Check-ins, Tracking |

### `/compliance`
Privacy and regulatory compliance templates.

| Template | Description | Regulations |
|----------|-------------|-------------|
| `privacy.gbai` | Data subject rights portal | LGPD, GDPR, CCPA |
| `hipaa-medical.gbai` | Healthcare privacy management | HIPAA, HITECH |

### `/platform`
Platform administration and analytics templates.

| Template | Description | Features |
|----------|-------------|----------|
| `analytics-dashboard.gbai` | Platform analytics bot | Metrics, Reports, AI insights |
| `bi.gbai` | Business intelligence | Dashboards, role separation |
| `backup.gbai` | Backup automation | Server backup scripts |
| `talk-to-data.gbai` | Data queries | Natural language SQL |

### `/finance`
Financial services templates.

| Template | Description | Features |
|----------|-------------|----------|
| `bank.gbai` | Banking services | Account management |

### `/integration`
External API and service integrations.

| Template | Description | APIs |
|----------|-------------|------|
| `bling.gbai` | Bling ERP | Brazilian ERP integration |

### `/productivity`
Office and personal productivity templates.

| Template | Description | Features |
|----------|-------------|----------|
| `office.gbai` | Office automation | Document management, Scheduling |
| `reminder.gbai` | Reminder system | Scheduled alerts, Follow-ups |

### `/hr`
Human resources templates.

| Template | Description | Features |
|----------|-------------|----------|
| `employees.gbai` | Employee management | Directory, Onboarding |

### `/it`
IT service management templates.

| Template | Description | Features |
|----------|-------------|----------|
| `helpdesk.gbai` | IT helpdesk ticketing | Tickets, Knowledge base |

### `/healthcare`
Healthcare-specific templates.

| Template | Description | Features |
|----------|-------------|----------|
| `patient-comm.gbai` | Patient communication | Appointments, Reminders |

### `/nonprofit`
Nonprofit organization templates.

| Template | Description | Features |
|----------|-------------|----------|
| `donor-mgmt.gbai` | Donor management | Donations, Communications |

### Root Level
Core and utility templates.

| Template | Description |
|----------|-------------|
| `default.gbai` | Starter template |
| `template.gbai` | Template for creating templates |
| `ai-search.gbai` | AI-powered document search |
| `announcements.gbai` | Company announcements |
| `backup.gbai` | Backup automation |
| `broadcast.gbai` | Message broadcasting |
| `crawler.gbai` | Web crawling |
| `edu.gbai` | Education/training |
| `erp.gbai` | ERP integration |
| `law.gbai` | Legal document processing |
| `llm-server.gbai` | LLM server management |
| `llm-tools.gbai` | LLM tool definitions |
| `store.gbai` | E-commerce |
| `whatsapp.gbai` | WhatsApp-specific features |

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
botserver --install-template crm
```

### From BASIC

```basic
INSTALL TEMPLATE "crm"
```

### Manual

Copy the template folder to your bot's packages directory:

```bash
cp -r templates/sales/crm.gbai /path/to/your/bot/packages/
```

---

## Creating Custom Templates

1. Copy `template.gbai` as a starting point
2. Rename the folder to `your-template.gbai`
3. Update internal folder names to match
4. Edit `config.csv` with your bot settings
5. Create dialog scripts in the `.gbdialog` folder
6. Use **ON** triggers instead of polling loops
7. Add `PARAM` and `DESCRIPTION` to make scripts LLM-invokable
8. Add documentation in `README.md`

### Template Best Practices

- Use `ON` for event-driven automation
- Use `TALK TO` for multi-channel notifications
- Use `LLM` for intelligent decision-making
- Use `SCORE LEAD` / `AI SCORE LEAD` for lead qualification
- Create `.bas` files with `DESCRIPTION` for LLM tool discovery
- Log activities for audit trails
- Include error handling
- Document all configuration options

---

## Contributing Templates

1. Create your template following the structure above
2. Test thoroughly with different inputs
3. Document all features and configuration
4. Submit a pull request with:
   - Template files
   - Updated category README
   - Entry in this document

---

## License

All templates are licensed under AGPL-3.0 as part of General Bots.

---

**Pragmatismo** - General Bots Open Source Platform