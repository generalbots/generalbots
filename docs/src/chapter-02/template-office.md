# Office Automation Template (office.gbai)

A General Bots template for role-based office productivity with department-specific knowledge bases and context-aware assistance.

---

## Overview

The Office template provides a multi-role office assistant that adapts its behavior, knowledge, and suggestions based on the user's role. Whether you're a manager, developer, customer support agent, HR professional, or finance team member, the bot tailors its responses and available resources accordingly.

## Features

- **Role-Based Access** - Different capabilities per user role
- **Dynamic Knowledge Bases** - Automatically loads relevant KB per role
- **Context-Aware Responses** - AI behavior adapts to role requirements
- **Custom Suggestions** - Role-specific quick actions
- **Tool Integration** - Calendar, tasks, documents, meetings, notes
- **Persistent Role Memory** - Remembers user role across sessions

---

## Package Structure

```
office.gbai/
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

---

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

---

## Available Tools

| Tool | Description |
|------|-------------|
| `calendar` | Schedule and manage appointments |
| `tasks` | Create and track to-do items |
| `documents` | Access and manage documents |
| `meetings` | Schedule and join meetings |
| `notes` | Create and search notes |

---

## Sample Conversation

### Role Selection

```
Bot:  Welcome to the Office Assistant!
      Please select your role:

User: manager

Bot:  Welcome, Manager! I can help with reports, team 
      management, and policies.
      
      Quick Actions:
      • Team performance
      • Generate report
      • Schedule meeting
```

### Manager Interaction

```
User: I need to review my team's performance

Bot:  I'll help you review team performance. Here's a summary:

      📊 Team Performance Dashboard
      
      Active Projects: 5
      Completed This Month: 3
      
      Top Performers:
      • Sarah Chen: 12 tasks completed
      • Mike Johnson: 10 tasks completed
      
      Pending Approvals: 2
      
      Would you like a detailed report or to schedule 1:1s?
```

### Developer Interaction

```
User: How do I implement pagination in our API?

Bot:  Based on our API documentation, here's the pagination pattern:

      GET /api/v1/users?page=1&per_page=20
      
      Response includes:
      • data: Array of results
      • meta.total: Total count
      • meta.page: Current page
      • links.next: Next page URL
      
      Our coding standards recommend cursor-based pagination 
      for large datasets.
      
      📄 Source: docs/api/pagination.md
```

### Customer Support Interaction

```
User: Customer asking about order #12345

Bot:  I found order #12345. Here are the details:
      
      📦 Order Status: Shipped
      📅 Shipped: May 14, 2025
      🚚 Carrier: FedEx
      📍 Tracking: 7891234567890
      
      Expected delivery: May 17, 2025
      
      Would you like to:
      • Send tracking info to customer
      • Create a support ticket
      • Escalate to supervisor
```

---

## Configuration

Configure in `office.gbot/config.csv`:

| Parameter | Description | Example |
|-----------|-------------|---------|
| `Default Role` | Role when none selected | `customer` |
| `Remember Role` | Persist role across sessions | `true` |
| `Role Timeout` | Hours before re-asking role | `24` |
| `Theme Color` | UI accent color | `blue` |

---

## How It Works

### Role Detection and Setup

```basic
ADD TOOL "calendar"
ADD TOOL "tasks"
ADD TOOL "documents"
ADD TOOL "meetings"
ADD TOOL "notes"

role = GET role

IF NOT role THEN
    TALK "Welcome to the Office Assistant!"
    TALK "Please select your role:"
    HEAR role AS NAME
    role = LOWER(role)
    SET role, role
END IF

SWITCH role
  CASE "manager"
    SET CONTEXT "You are an executive assistant helping managers..."
    USE KB "management"
    USE KB "reports"
    USE KB "team-policies"
    TALK "Welcome, Manager! I can help with reports and team management."

  CASE "developer"
    SET CONTEXT "You are a technical assistant helping developers..."
    USE KB "documentation"
    USE KB "apis"
    USE KB "coding-standards"
    TALK "Welcome, Developer! I can help with documentation and APIs."

  ' ... more roles
END SWITCH
```

### Dynamic Suggestions

```basic
CLEAR SUGGESTIONS

SWITCH role
  CASE "manager"
    ADD SUGGESTION "performance" AS "Team performance"
    ADD SUGGESTION "report" AS "Generate report"
    ADD SUGGESTION "meeting" AS "Schedule meeting"

  CASE "developer"
    ADD SUGGESTION "docs" AS "Search documentation"
    ADD SUGGESTION "api" AS "API reference"
    ADD SUGGESTION "review" AS "Code review checklist"

  CASE "customer"
    ADD SUGGESTION "order" AS "Track my order"
    ADD SUGGESTION "product" AS "Product information"
    ADD SUGGESTION "support" AS "Contact support"
END SWITCH
```

---

## Customization

### Adding New Roles

Extend the `start.bas` file:

```basic
CASE "sales"
    SET CONTEXT "You are a sales assistant helping with leads and deals."
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

---

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

---

## Best Practices

1. **Clear role definitions** - Define clear boundaries for each role
2. **Relevant suggestions** - Keep quick actions useful for each role
3. **Appropriate KBs** - Only load necessary knowledge bases
4. **Security awareness** - Restrict sensitive data by role
5. **Regular updates** - Keep knowledge bases current
6. **Feedback loops** - Monitor which features each role uses

---

## Troubleshooting

| Issue | Cause | Solution |
|-------|-------|----------|
| Wrong KB loaded | Role not set correctly | Check role detection logic |
| Missing suggestions | Role not in switch | Add role to all switch blocks |
| Context confusion | Multiple roles used | Clear context between role changes |
| Slow responses | Too many KBs loaded | Load only essential KBs per role |

---

## Use Cases

- **Corporate Offices** - Multi-department support
- **Startups** - Flexible role-based assistance
- **Remote Teams** - Unified office assistant
- **Enterprise** - Department-specific knowledge management

---

## Related Templates

- [Contacts](./template-crm-contacts.md) - Contact management
- [Reminder](./template-reminder.md) - Task and reminder management
- [CRM](./template-crm.md) - Full CRM for sales roles
- [Analytics](./template-analytics.md) - Platform analytics for managers

---

## See Also

- [Templates Reference](./templates.md) - Full template list
- [Template Samples](./template-samples.md) - Example conversations
- [gbdialog Reference](../chapter-06-gbdialog/README.md) - BASIC scripting guide