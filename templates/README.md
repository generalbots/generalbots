# General Bots Templates

Pre-built bot packages for common business use cases. Templates are organized by category for easy discovery.

## Categories

### `/compliance`
Privacy and regulatory compliance templates.

| Template | Description | Regulations |
|----------|-------------|-------------|
| `privacy.gbai` | Data subject rights portal | LGPD, GDPR, CCPA |
| `hipaa-medical.gbai` | Healthcare privacy management | HIPAA, HITECH |

### `/sales`
Customer relationship and marketing templates.

| Template | Description | Features |
|----------|-------------|----------|
| `crm.gbai` | Full CRM system | Leads, Contacts, Accounts, Opportunities, Activities |
| `marketing.gbai` | Marketing automation | Campaigns, Lead capture, Email sequences |

### `/productivity`
Office and personal productivity templates.

| Template | Description | Features |
|----------|-------------|----------|
| `office.gbai` | Office automation | Document management, Scheduling |
| `reminder.gbai` | Reminder and notification system | Scheduled alerts, Follow-ups |

### `/platform`
Platform administration and analytics templates.

| Template | Description | Features |
|----------|-------------|----------|
| `analytics.gbai` | Platform analytics bot | Metrics, Reports, AI insights |

### `/integration`
External API and service integrations.

| Template | Description | APIs |
|----------|-------------|------|
| `api-client.gbai` | REST API client examples | Various |
| `public-apis.gbai` | Public API integrations | Weather, News, etc. |

### `/hr`
Human resources templates.

| Template | Description | Features |
|----------|-------------|----------|
| `employee-mgmt.gbai` | Employee management | Directory, Onboarding |

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

### `/finance`
Financial services templates.

| Template | Description | Features |
|----------|-------------|----------|
| `bank.gbai` | Banking services | Account management |
| `finance.gbai` | Financial operations | Invoicing, Payments |

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
| `talk-to-data.gbai` | Natural language data queries |
| `template.gbai` | Template for creating templates |
| `whatsapp.gbai` | WhatsApp-specific features |

## Template Structure

Each `.gbai` template follows this structure:

```
template-name.gbai/
├── README.md                 # Template documentation
├── template-name.gbdialog/   # BASIC dialog scripts
│   ├── start.bas            # Entry point
│   └── *.bas                # Additional dialogs
├── template-name.gbot/       # Bot configuration
│   └── config.csv           # Settings
├── template-name.gbkb/       # Knowledge base (optional)
│   └── docs/                # Documents for RAG
├── template-name.gbdrive/    # File storage (optional)
└── template-name.gbui/       # Custom UI (optional)
    └── index.html
```

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

## Creating Custom Templates

1. Copy `template.gbai` as a starting point
2. Rename the folder to `your-template.gbai`
3. Update internal folder names to match
4. Edit `config.csv` with your bot settings
5. Create dialog scripts in the `.gbdialog` folder
6. Add documentation in `README.md`

### Template Best Practices

- Use `HEAR AS` for typed input validation
- Use spaces in keywords (e.g., `SET BOT MEMORY`, not `SET_BOT_MEMORY`)
- Log activities for audit trails
- Include error handling
- Document all configuration options
- Provide example conversations

## Contributing Templates

1. Create your template following the structure above
2. Test thoroughly with different inputs
3. Document all features and configuration
4. Submit a pull request with:
   - Template files
   - Updated category README
   - Entry in this document

## License

All templates are licensed under AGPL-3.0 as part of General Bots.

---

**Pragmatismo** - General Bots Open Source Platform