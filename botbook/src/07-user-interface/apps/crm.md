# CRM - Customer Relationship Management

> **Manage your sales pipeline from lead to close**

---

## Overview

CRM is your complete sales management solution following Microsoft Dynamics nomenclature. Track leads through qualification, manage opportunities in a visual pipeline, and convert them to accounts and contacts when deals close.

---

## Key Concepts (Dynamics Nomenclature)

| Entity | Description |
|--------|-------------|
| **Lead** | Unqualified prospect - someone who might become a customer |
| **Opportunity** | Qualified lead in the active sales process |
| **Account** | Company/organization (converted customer) |
| **Contact** | Person at an Account |
| **Activity** | Tasks, calls, emails linked to any entity |

### Entity Flow

```
Lead ──(qualify)──► Opportunity ──(convert)──► Account + Contact
```

---

## Features

### Pipeline View (Kanban)

The default view shows your sales pipeline as a Kanban board with drag-and-drop functionality:

| Stage | Description |
|-------|-------------|
| **Lead** | New unqualified prospects |
| **Qualified** | Leads that meet your criteria |
| **Proposal** | Opportunities with sent proposals |
| **Negotiation** | Active deal discussions |
| **Won** | Successfully closed deals |
| **Lost** | Deals that didn't close |

**Drag cards between columns** to update opportunity stages instantly.

### Leads Management

Track and qualify incoming prospects:

- **Name** - Contact name
- **Company** - Organization name
- **Email** - Primary contact email
- **Phone** - Contact phone number
- **Source** - Where the lead came from (web, referral, event, etc.)
- **Status** - New, Contacted, Qualified

### Opportunities

Manage active sales deals:

- **Opportunity Name** - Deal identifier
- **Account** - Associated company
- **Value** - Expected deal amount
- **Stage** - Current pipeline position
- **Probability** - Win likelihood percentage
- **Expected Close** - Target close date
- **Owner** - Sales representative

### Accounts

Company records for your customers:

- **Account Name** - Company name
- **Industry** - Business sector
- **Phone** - Main phone number
- **City** - Location
- **Annual Revenue** - Company size indicator
- **Contacts** - Number of associated contacts

### Contacts

People at your customer accounts:

- **Name** - Full name
- **Account** - Associated company
- **Title** - Job title/role
- **Email** - Contact email
- **Phone** - Direct phone

---

## Pipeline Summary Metrics

Real-time dashboard showing:

| Metric | Description |
|--------|-------------|
| **Total Pipeline Value** | Sum of all active opportunity values |
| **Conversion Rate** | Percentage of leads that convert to wins |
| **Avg Deal Size** | Average value of won opportunities |
| **Won This Month** | Total value closed this month |

---

## Navigation Tabs

| Tab | View |
|-----|------|
| **Pipeline** | Kanban board of opportunities |
| **Leads** | Table of all leads |
| **Opportunities** | Table of all opportunities |
| **Accounts** | Table of all accounts |
| **Contacts** | Table of all contacts |

---

## Email Tab

The **Email** tab in CRM shows all emails linked to the selected contact. Click any contact row to load their emails via `GET /api/ui/email/list?contact_email=`.

Click **Compose Email** to open the email modal which posts to `POST /api/crm/email/send`. The email is automatically linked to the contact via `email_crm_links`.

## Enabling CRM

Add `crm` to `apps=` in `botserver/.product`:

```
apps=...,crm
```

## API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/crm/leads` | GET | List leads with filters |
| `/api/crm/leads` | POST | Create new lead |
| `/api/crm/leads/:id` | GET | Get lead details |
| `/api/crm/leads/:id` | PUT | Update lead |
| `/api/crm/leads/:id/qualify` | POST | Qualify lead to opportunity |
| `/api/crm/opportunities` | GET | List opportunities |
| `/api/crm/opportunities` | POST | Create opportunity |
| `/api/crm/opportunity/:id/stage` | POST | Update opportunity stage |
| `/api/crm/accounts` | GET | List accounts |
| `/api/crm/accounts` | POST | Create account |
| `/api/crm/contacts` | GET | List contacts |
| `/api/crm/contacts` | POST | Create contact |
| `/api/crm/pipeline` | GET | Get pipeline data by stage |
| `/api/crm/count` | GET | Get counts by stage |
| `/api/crm/stats/*` | GET | Get various statistics |
| `/api/crm/search` | GET | Search across all CRM entities |

---

## @ Mentions in Chat

Reference CRM entities directly in chat conversations:

| Mention | Example |
|---------|---------|
| `@lead:` | @lead:John Smith |
| `@opportunity:` | @opportunity:Enterprise Deal |
| `@account:` | @account:Acme Corp |
| `@contact:` | @contact:Jane Doe |

Hover over a mention to see entity details. Click to navigate to the record.

---

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `N` | New lead (when in CRM) |
| `Escape` | Close modal |
| `/` | Focus search |

---

## Filtering Options

### Leads Filter

- All Leads
- New
- Contacted
- Qualified

---

## Best Practices

### Lead Management

1. **Respond quickly** - Follow up on new leads within 24 hours
2. **Qualify early** - Move quality leads to Opportunities promptly
3. **Track source** - Know where your best leads come from

### Pipeline Health

1. **Update stages daily** - Keep pipeline accurate
2. **Set realistic close dates** - Update Expected Close as needed
3. **Review weekly** - Identify stuck opportunities

### Data Quality

1. **Complete profiles** - Fill in all available information
2. **Link contacts to accounts** - Maintain relationships
3. **Log activities** - Track all customer interactions

---

## See Also

- [Billing](./billing.md) — Create invoices from won opportunities
- [Products](./products.md) — Add products to quotes and invoices
- [Analytics](./analytics.md) — CRM reports and dashboards
- [Tasks](./tasks.md) — Create follow-up tasks from CRM