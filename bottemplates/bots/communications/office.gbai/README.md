# Office Automation Template (office.gbai)

A General Bots template for role-based office productivity with department-specific knowledge bases and context-aware assistance.

## Overview

The Office template provides a multi-role office assistant that adapts its behavior, knowledge, and suggestions based on the user's role. Whether you're a manager, developer, customer support agent, HR professional, or finance team member, the bot tailors its responses and available resources accordingly.

## Features

- **Role-Based Access** - Different capabilities per user role
- **Dynamic Knowledge Bases** - Automatically loads relevant KB per role
- **Context-Aware Responses** - AI behavior adapts to role requirements
- **Custom Suggestions** - Role-specific quick actions
- **Tool Integration** - Calendar, tasks, documents, meetings, notes
- **Persistent Role Memory** - Remembers user role across sessions

## Package Structure

```
office.gbai/
├── README.md
├── office.gbdialog/
│   ├── start.bas               # Role selection and configuration
│   ├── api-integration.bas     # External API connections
│   ├── data-sync.bas           # Data synchronization
│   └── document-processor.bas  # Document handling
├── office.gbkb/                # Knowledge bases by role
│   ├── management/
│   ├── documentation/
│   ├── products/
│   ├── hr-policies/
│   └── budgets/
└── office.gbot/
    └── config.csv              # Bot configuration
```

## Scripts

| File | Description |
|------|-------------|
| `start.bas` | Main entry with role selection and context setup |
| `api-integration.bas` | Connect to external office systems |
| `data-sync.bas` | Sync data across office applications |
| `document-processor.bas` | Process and manage documents |

## Supported Roles

### Manager

- **Context**: Executive assistant for reports, team management, strategic decisions
- **Knowledge Bases**: management, reports, team-policies
- **Suggestions**: Team performance, Generate report, Schedule meeting

### Developer

- **Context**: Technical assistant for documentation, APIs, coding best practices
- **Knowledge Bases**: documentation, apis, coding-standards
- **Suggestions**: Search documentation, API reference, Code review checklist

### Customer Support

- **Context**: Customer service assistant focused on resolving issues
- **Knowledge Bases**: products, support, faq
- **Suggestions**: Track my order, Product information, Contact support

### HR

- **Context**: HR assistant for employee matters, policies, benefits
- **Knowledge Bases**: hr-policies, benefits, onboarding
- **Suggestions**: Employee handbook, Benefits overview, New hire checklist

### Finance

- **Context**: Finance assistant for budgets, expenses, financial reports
- **Knowledge Bases**: budgets, expenses, financial-reports
- **Suggestions**: Expense policy, Budget status, Approval workflow

## Available Tools

| Tool | Description |
|------|-------------|
| `calendar` | Schedule and manage appointments |
| `tasks` | Create and track to-do items |
| `documents` | Access and manage documents |
| `meetings` | Schedule and join meetings |
| `notes` | Create and search notes |

## Usage

### First-Time Setup

```
Bot: Welcome to the Office Assistant!
     Please select your role:

User: manager

Bot: Welcome, Manager! I can help with reports, team management, and policies.
```

### Role-Based Interaction

**Manager:**
```
User: Generate a team performance report
Bot: [Uses management KB and context]
     I'll generate the team performance report. What time period?
```

**Developer:**
```
User: How do I authenticate with the payment API?
Bot: [Uses documentation and apis KB]
     According to our API documentation, payment API authentication 
     uses OAuth 2.0. Here's how to implement it...
```

**Customer Support:**
```
User: Customer asking about order #12345
Bot: [Uses products and support KB]
     I found order #12345. It was shipped yesterday and should 
     arrive by Thursday. Would you like the tracking number?
```

## Configuration

Configure in `office.gbot/config.csv`:

| Parameter | Description | Example |
|-----------|-------------|---------|
| `Default Role` | Role when none selected | `customer` |
| `Remember Role` | Persist role across sessions | `true` |
| `Role Timeout` | Hours before re-asking role | `24` |
| `Theme Color` | UI accent color | `blue` |

## Customization

### Adding New Roles

Extend the `start.bas` file:

```basic
CASE "sales"
    SET CONTEXT "You are a sales assistant helping with leads, opportunities, and closing deals."
    USE KB "sales-playbook"
    USE KB "pricing"
    USE KB "competitors"
    TALK "Welcome, Sales! I can help with leads, pricing, and proposals."

    CLEAR SUGGESTIONS
    ADD SUGGESTION "leads" AS "View my leads"
    ADD SUGGESTION "quote" AS "Generate quote"
    ADD SUGGESTION "pipeline" AS "Pipeline status"
```

### Custom Knowledge Bases

Create role-specific knowledge bases in `office.gbkb/`:

```
office.gbkb/
├── sales-playbook/
│   ├── objection-handling.md
│   ├── pricing-guide.md
│   └── competitor-comparison.md
```

### Role-Specific Tools

Register different tools per role:

```basic
CASE "manager"
    ADD TOOL "calendar"
    ADD TOOL "tasks"
    ADD TOOL "team-report"
    ADD TOOL "approve-request"

CASE "developer"
    ADD TOOL "search-docs"
    ADD TOOL "api-tester"
    ADD TOOL "code-review"
```

### Custom System Prompts

Extend the system prompt per role:

```basic
BEGIN SYSTEM PROMPT
You are a role-based office assistant.

Current user role: ${role}

Role-specific guidelines:
- Manager: Focus on strategic decisions, delegation, and KPIs
- Developer: Be technical, reference documentation, suggest best practices
- Customer: Be empathetic, focus on resolution, escalate when needed
- HR: Maintain confidentiality, be policy-aware, supportive
- Finance: Be precise with numbers, follow approval processes

Adapt your responses and suggestions based on the user's role.
END SYSTEM PROMPT
```

## Integration Examples

### With Calendar

```basic
' Schedule meeting for manager
IF role = "manager" THEN
    TALK "I'll schedule the team meeting."
    CREATE CALENDAR EVENT "Team Standup", tomorrow + " 9:00 AM", 30
END IF
```

### With Document System

```basic
' Generate document based on role
SWITCH role
    CASE "hr"
        template = "offer-letter-template.docx"
    CASE "sales"
        template = "proposal-template.docx"
    CASE "finance"
        template = "budget-template.xlsx"
END SWITCH

document = GENERATE FROM TEMPLATE template WITH data
```

### With Task Management

```basic
' Create role-appropriate tasks
IF role = "manager" THEN
    CREATE TASK "Review Q4 budget", "high", manager_email
    CREATE TASK "Approve team PTO requests", "medium", manager_email
END IF
```

## Example Conversations

### Manager Role

```
User: I need to review my team's performance
Bot: I'll help you review team performance. Here's a summary:

     📊 Team Performance Dashboard
     
     Active Projects: 5
     Completed This Month: 3
     
     Top Performers:
     - Sarah Chen: 12 tasks completed
     - Mike Johnson: 10 tasks completed
     
     Pending Approvals: 2
     
     Would you like a detailed report or to schedule 1:1s?
```

### Developer Role

```
User: How do I implement pagination in our API?
Bot: Based on our API documentation, here's the pagination pattern:

     ```javascript
     GET /api/v1/users?page=1&per_page=20
     ```
     
     Response includes:
     - `data`: Array of results
     - `meta.total`: Total count
     - `meta.page`: Current page
     - `links.next`: Next page URL
     
     Our coding standards recommend cursor-based pagination 
     for large datasets. See: docs/api/pagination.md
```

## Best Practices

1. **Clear role definitions** - Define clear boundaries for each role
2. **Relevant suggestions** - Keep quick actions useful for each role
3. **Appropriate KBs** - Only load necessary knowledge bases
4. **Security awareness** - Restrict sensitive data by role
5. **Regular updates** - Keep knowledge bases current
6. **Feedback loops** - Monitor which features each role uses

## Troubleshooting

| Issue | Cause | Solution |
|-------|-------|----------|
| Wrong KB loaded | Role not set correctly | Check role detection logic |
| Missing suggestions | Role not in switch statement | Add role to all switch blocks |
| Context confusion | Multiple roles used | Clear context between role changes |
| Slow responses | Too many KBs loaded | Load only essential KBs per role |

## Related Templates

- `contacts.gbai` - Contact management
- `reminder.gbai` - Task and reminder management
- `crm.gbai` - Full CRM for sales roles
- `analytics.gbai` - Platform analytics for managers

## Use Cases

- **Corporate Offices** - Multi-department support
- **Startups** - Flexible role-based assistance
- **Remote Teams** - Unified office assistant
- **Enterprise** - Department-specific knowledge management

## License

AGPL-3.0 - Part of General Bots Open Source Platform.

---

**Pragmatismo** - General Bots