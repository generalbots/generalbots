# Notion Migration Guide

Migrating content and workflows from Notion to General Bots.

<img src="../assets/gb-decorative-header.svg" alt="General Bots" style="max-height: 100px; width: 100%; object-fit: contain;">

## Overview

Notion is a collaborative workspace combining notes, databases, and project management. General Bots provides equivalent functionality through its knowledge base, task management, and AI-powered conversation features—with the added benefit of self-hosting and native automation.

## Why Migrate

| Aspect | Notion | General Bots |
|--------|--------|--------------|
| Hosting | Cloud only | Self-hosted |
| Pricing | $10-15/user/month | No per-user fees |
| AI Features | $10/user/month add-on | Native (any LLM) |
| Automation | Limited | Full BASIC scripting |
| Chat/Bot | Not included | Multi-channel |
| API Creation | Not available | Instant webhooks |
| Data Sovereignty | Their servers | Your infrastructure |

## Content Mapping

### Notion to General Bots

| Notion Feature | General Bots Equivalent |
|----------------|------------------------|
| Pages | Knowledge base documents (.gbkb) |
| Databases | Tables (CSV/database) |
| Kanban boards | Task boards |
| Calendar | Calendar API |
| Comments | Conversation history |
| Templates | Bot templates |
| Integrations | BASIC scripts + webhooks |
| Notion AI | LLM keyword |

## Migration Process

### Step 1: Export Notion Content

Navigate to Settings & Members, then Settings, and scroll to Export content. Choose the Markdown & CSV export format and download the ZIP file.

The export includes pages as Markdown files, databases as CSV files, and attachments in folders.

### Step 2: Prepare Knowledge Base

Organize exported content for General Bots:

```
my-bot.gbkb/
├── company-info/
│   ├── about.md
│   ├── policies.md
│   └── procedures.md
├── products/
│   ├── catalog.md
│   └── pricing.md
└── support/
    ├── faq.md
    └── troubleshooting.md
```

### Step 3: Import Documents

Place Markdown files in your `.gbkb` folder. General Bots automatically indexes them for RAG:

```basic
USE KB "company-info"
USE KB "products"
USE KB "support"

TALK "How can I help you?"
HEAR question
answer = LLM question
TALK answer
```

### Step 4: Convert Databases

Transform Notion CSV exports to General Bots tables:

```basic
' Import Notion database export
data = READ "notion-export/Projects.csv"

FOR EACH row IN data
    INSERT "projects", #{
        name: row["Name"],
        status: row["Status"],
        due_date: row["Due Date"],
        assignee: row["Assignee"],
        imported_at: NOW()
    }
NEXT row

TALK "Imported " + LEN(data) + " projects"
```

### Step 5: Recreate Task Boards

Convert Notion Kanban boards to General Bots task boards:

```basic
' Create project for Notion board
project_id = CREATE PROJECT "Product Roadmap" WITH DESCRIPTION "Migrated from Notion"

' Import tasks
tasks = READ "notion-export/Roadmap.csv"

FOR EACH task IN tasks
    status = SWITCH task["Status"]
        CASE "Not Started" : "todo"
        CASE "In Progress" : "in_progress"
        CASE "Done" : "done"
        DEFAULT : "todo"
    END SWITCH
    
    CREATE TASK task["Name"] IN PROJECT project_id WITH STATUS status
NEXT task
```

## Notion AI to General Bots

### Document Summarization

Notion AI allows highlighting text and selecting "Summarize" but is limited to Notion content. General Bots provides broader capability:

```basic
USE KB "documents"
summary = LLM "Summarize the key points from our Q3 report"
TALK summary
```

### Content Generation

Where Notion AI uses the `/ai` command for basic prompting, General Bots offers full control:

```basic
SET CONTEXT "You are a technical writer. Write clear, concise documentation."

TALK "What would you like me to write?"
HEAR topic

content = LLM "Write comprehensive documentation about: " + topic
WRITE "/docs/" + SLUGIFY(topic) + ".md", content
TALK "Documentation created!"
```

### Q&A on Documents

Notion AI asks questions about single page content. General Bots searches across your entire knowledge base:

```basic
' Load entire knowledge base
USE KB "all-docs"
USE KB "wiki"
USE KB "procedures"

' Answer questions across all content
TALK "Ask me anything about our documentation"
HEAR question
answer = LLM question
TALK answer
```

## Automation Migration

### Notion Automations (Limited)

Notion has basic automations for status changes, due date reminders, and Slack notifications.

### General Bots Equivalent

Status change automation:

```basic
ON "table:projects:update"
    IF params.old_status <> params.new_status THEN
        IF params.new_status = "complete" THEN
            SEND MAIL TO params.owner_email SUBJECT "Project Completed" BODY "Your project " + params.name + " is now complete!"
        END IF
    END IF
END ON
```

Due date reminders:

```basic
SET SCHEDULE "every day at 9am"

upcoming = FIND "tasks", "due_date = DATEADD(NOW(), 1, 'day') AND status <> 'done'"

FOR EACH task IN upcoming
    SEND MAIL TO task.assignee_email SUBJECT "Task Due Tomorrow" BODY "Reminder: " + task.name + " is due tomorrow"
NEXT task
```

Slack notifications:

```basic
ON "table:tasks:insert"
    POST "https://hooks.slack.com/services/xxx", #{
        text: "New task created: " + params.name,
        channel: "#tasks"
    }
END ON
```

## Database Migration

### Notion Database Properties

| Notion Property | General Bots Equivalent |
|-----------------|------------------------|
| Title | TEXT column |
| Text | TEXT column |
| Number | NUMERIC column |
| Select | TEXT with validation |
| Multi-select | JSONB array |
| Date | DATE/TIMESTAMP column |
| Person | User reference |
| Files | File path references |
| Checkbox | BOOLEAN column |
| URL | TEXT column |
| Email | TEXT column |
| Phone | TEXT column |
| Formula | Computed in BASIC |
| Relation | Foreign key |
| Rollup | AGGREGATE queries |

### Formula Migration

Notion formulas like `prop("Price") * prop("Quantity")` translate to BASIC calculations:

```basic
' Calculate on insert/update
total = price * quantity
INSERT "orders", #{item: item, price: price, quantity: quantity, total: total}

' Or query with calculation
SELECT "*, price * quantity as total FROM orders"
```

### Relation Migration

Notion relations link databases together. General Bots uses foreign keys:

```basic
' Create related tables
CREATE TABLE "projects" (id, name, status)
CREATE TABLE "tasks" (id, project_id, name, assignee)

' Query with join
tasks = FIND "tasks", "project_id = '" + project_id + "'"

' Or use JOIN keyword
result = JOIN "projects", "tasks", "projects.id = tasks.project_id"
```

## Template Migration

### Notion Templates

Notion templates are pre-filled pages. Convert to General Bots templates as BASIC scripts.

Meeting notes template:

```basic
' meeting-notes.bas
PARAM meeting_title AS string
PARAM attendees AS string
PARAM date AS date

DESCRIPTION "Create meeting notes document"

template = "# " + meeting_title + "

**Date:** " + FORMAT(date, "MMMM d, yyyy") + "
**Attendees:** " + attendees + "

## Agenda
1. 
2. 
3. 

## Discussion Notes


## Action Items
- [ ] 
- [ ] 

## Next Meeting
"

WRITE "/meetings/" + FORMAT(date, "yyyy-MM-dd") + "-" + SLUGIFY(meeting_title) + ".md", template
TALK "Meeting notes created: " + meeting_title
```

### Project Template

```basic
' new-project.bas
PARAM project_name AS string
PARAM owner AS string

DESCRIPTION "Create new project with standard structure"

project_id = CREATE PROJECT project_name WITH DESCRIPTION "Created by template"
ADD USER TO PROJECT project_id, owner, "owner"

' Create standard tasks
CREATE TASK "Define requirements" IN PROJECT project_id
CREATE TASK "Create timeline" IN PROJECT project_id
CREATE TASK "Assign resources" IN PROJECT project_id
CREATE TASK "Kickoff meeting" IN PROJECT project_id
CREATE TASK "First milestone review" IN PROJECT project_id

TALK "Project '" + project_name + "' created with 5 starter tasks"
```

## What You Gain

### Self-Hosting

Your data stays on your infrastructure. No concerns about Notion's data practices or service availability.

### Native AI Without Extra Cost

Notion charges $10/user/month for AI features. General Bots includes AI at no additional cost—use any LLM provider.

### Full Automation

Go beyond Notion's limited automations with complete BASIC scripting:

```basic
SET SCHEDULE "every monday at 9am"

' Generate weekly report
projects = FIND "projects", "status = 'active'"
tasks_completed = AGGREGATE "tasks", "COUNT", "id", "completed_at > DATEADD(NOW(), -7, 'day')"

SET CONTEXT "You are a project manager. Create a concise weekly summary."
report = LLM "Summarize: " + LEN(projects) + " active projects, " + tasks_completed + " tasks completed this week"

SEND MAIL TO "team@company.com" SUBJECT "Weekly Project Summary" BODY report
```

### Multi-Channel Access

Access your knowledge base through any channel:

```basic
' Same bot works on web, WhatsApp, Teams, Slack
TALK "How can I help you today?"
HEAR question

USE KB "company-wiki"
answer = LLM question
TALK answer
```

### Custom APIs

Create APIs instantly—something not possible in Notion:

```basic
WEBHOOK "project-status"

project = FIND "projects", "id = '" + params.id + "'"
tasks = FIND "tasks", "project_id = '" + params.id + "'"

WITH response
    .project = project
    .task_count = LEN(tasks)
    .completed = LEN(FILTER(tasks, "status = 'done'"))
END WITH
```

## Migration Checklist

### Pre-Migration

Before starting, export all Notion content in Markdown & CSV format. Inventory your databases and their properties. Document active integrations. Identify critical templates that need recreation. Set up your General Bots environment.

### Migration

During the migration, organize Markdown files into the .gbkb structure. Import database CSVs to tables. Convert formulas to BASIC calculations. Recreate task boards as projects. Migrate templates to BASIC scripts. Set up automations to replace Notion workflows.

### Post-Migration

After migration, verify all content is searchable in the knowledge base. Test database queries. Confirm automations work correctly. Train your team on the new interface. Redirect any Notion integrations to General Bots.

## Best Practices

Organize your knowledge base thoughtfully by grouping related documents in collections for better RAG results.

Simplify database structures because Notion encourages complex relations while General Bots works best with cleaner schemas.

Leverage AI for migration by using General Bots' LLM to help transform and summarize Notion content:

```basic
content = READ "notion-export/long-document.md"
summary = LLM "Create a concise summary of this document: " + content
WRITE "/summaries/document-summary.md", summary
```

Keep templates as scripts since BASIC templates are more powerful than Notion's static templates.

## See Also

- [Knowledge Base](../chapter-03/README.md) - KB configuration
- [Projects](../chapter-11-features/projects.md) - Project management
- [Template Variables](../chapter-06-gbdialog/template-variables.md) - Dynamic content
- [Platform Comparison](./comparison-matrix.md) - Full feature comparison