# Tickets - AI-Assisted Support Cases

> **Manage customer support with AI-powered resolution suggestions**

---

## Overview

Tickets is your AI-assisted support system following Microsoft Dynamics nomenclature. Create and manage support cases, track resolution times, and leverage AI to suggest solutions and automate common responses.

---

## Key Concepts (Dynamics Nomenclature)

| Entity | Description |
|--------|-------------|
| **Case** | Support ticket/request from a customer |
| **Resolution** | AI-suggested or manual solution to a case |
| **Activity** | Actions taken on a case (responses, calls, etc.) |

---

## Features

### Case Management

Track and resolve customer support requests:

- **Case Number** - Unique identifier
- **Subject** - Brief description of the issue
- **Account** - Customer reporting the issue
- **Contact** - Person who reported the issue
- **Priority** - Urgency level
- **Status** - Current case state
- **Category** - Issue classification
- **Description** - Full issue details
- **Assigned To** - Support agent handling the case

### Case Statuses

| Status | Description |
|--------|-------------|
| **Open** | New case awaiting attention |
| **Pending** | Waiting for customer response or external input |
| **In Progress** | Being actively worked on |
| **Resolved** | Solution provided, awaiting confirmation |
| **Closed** | Case completed and closed |

### Priority Levels

| Priority | Description |
|----------|-------------|
| **Critical** | System down, immediate attention required |
| **High** | Major issue affecting business operations |
| **Medium** | Standard issue with workaround available |
| **Low** | Minor issue or general inquiry |

### AI Assistance

The AI assistant helps with:

- **Auto-categorization** - Automatically classify incoming cases
- **Solution Suggestions** - Recommend resolutions based on similar cases
- **Response Templates** - Generate contextual reply drafts
- **Priority Detection** - Identify urgent cases from description
- **Knowledge Search** - Find relevant KB articles automatically

---

## Summary Dashboard

Real-time support metrics:

| Metric | Description |
|--------|-------------|
| **Open Cases** | Number of unresolved cases |
| **Urgent** | Cases with critical/high priority |
| **Resolved Today** | Cases closed today |
| **AI Resolved** | Percentage of cases resolved by AI |

---

## Navigation Tabs

| Tab | View |
|-----|------|
| **All Cases** | Complete case list |
| **Open** | Only open cases |
| **Pending** | Cases awaiting response |
| **Resolved** | Completed cases |

---

## API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/tickets` | GET | List cases with filters |
| `/api/tickets` | POST | Create new case |
| `/api/tickets/:id` | GET | Get case details |
| `/api/tickets/:id` | PUT | Update case |
| `/api/tickets/:id/resolve` | POST | Mark case as resolved |
| `/api/tickets/:id/close` | POST | Close case |
| `/api/tickets/:id/reopen` | POST | Reopen closed case |
| `/api/tickets/:id/assign` | POST | Assign to agent |
| `/api/tickets/:id/activities` | GET | Get case activities |
| `/api/tickets/:id/activities` | POST | Add activity to case |
| `/api/tickets/:id/ai-suggest` | GET | Get AI resolution suggestions |
| `/api/tickets/search` | GET | Search cases |
| `/api/tickets/stats/*` | GET | Get support statistics |

---

## @ Mentions in Chat

Reference cases directly in chat conversations:

| Mention | Example |
|---------|---------|
| `@case:` | @case:CS-2024-001 |

Hover over a mention to see case details. Click to navigate to the record.

---

## Filtering Options

### Status Filters

| Filter | Options |
|--------|---------|
| **Status** | All, Open, Pending, In Progress, Resolved, Closed |

### Priority Filters

| Filter | Options |
|--------|---------|
| **Priority** | All, Critical, High, Medium, Low |

### Category Filters

| Filter | Options |
|--------|---------|
| **Category** | All, Technical, Billing, General, Feature Request |

---

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `N` | New case (when in Tickets) |
| `Escape` | Close modal |
| `/` | Focus search |
| `R` | Reply to selected case |

---

## AI Resolution Flow

1. **Case Created** - Customer submits support request
2. **AI Analysis** - System analyzes case description
3. **Auto-Categorize** - Priority and category assigned
4. **Suggest Solutions** - AI finds similar resolved cases
5. **Generate Response** - Draft reply created for review
6. **Agent Review** - Support agent approves or modifies
7. **Resolution** - Customer receives response

---

## Integration with CRM

Tickets integrates with your CRM data:

1. **Account Linking** - Cases linked to customer accounts
2. **Contact Association** - Track who reported each issue
3. **History Access** - View customer's previous cases
4. **Activity Sync** - Support activities appear in CRM timeline

---

## Best Practices

### Case Management

1. **Respond quickly** - Acknowledge cases within SLA
2. **Set accurate priority** - Ensure urgent issues get attention
3. **Document thoroughly** - Record all resolution steps
4. **Update status** - Keep case status current

### Using AI Assistance

1. **Review suggestions** - Always verify AI recommendations
2. **Train the model** - Mark good suggestions to improve accuracy
3. **Personalize responses** - Edit AI drafts for customer context
4. **Escalate when needed** - Don't rely on AI for complex issues

### SLA Management

1. **Define SLAs** - Set response and resolution time targets
2. **Monitor compliance** - Track SLA performance
3. **Escalate proactively** - Flag cases approaching SLA breach

---

## Reports

Available in Analytics:

| Report | Description |
|--------|-------------|
| **Open Cases by Priority** | Distribution of active cases |
| **Resolution Time** | Average time to resolve by category |
| **Cases by Category** | Volume breakdown by issue type |
| **AI Resolution Rate** | Percentage resolved with AI assistance |
| **Agent Performance** | Cases handled per agent |
| **SLA Compliance** | Percentage meeting SLA targets |

---

## Case Categories

| Category | Description |
|----------|-------------|
| **Technical** | Product bugs, errors, technical issues |
| **Billing** | Invoice questions, payment issues |
| **General** | General inquiries, how-to questions |
| **Feature Request** | Suggestions for new features |

---

## Activity Types

Activities logged on cases:

| Activity | Description |
|----------|-------------|
| **Email** | Email sent to/from customer |
| **Phone Call** | Phone conversation logged |
| **Note** | Internal note added |
| **Status Change** | Case status updated |
| **Assignment** | Case reassigned |
| **Resolution** | Solution provided |

---

## See Also

- [CRM](./crm.md) — Link cases to accounts and contacts
- [Chat](./chat.md) — AI assistant for support queries
- [Analytics](./analytics.md) — Support reports and dashboards
- [Tasks](./tasks.md) — Create follow-up tasks from cases