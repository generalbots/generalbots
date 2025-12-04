# Using Templates

Templates are pre-built bot configurations that accelerate development. General Bots includes templates for common use cases like CRM, HR, IT support, and more.

## Template Structure

Each template follows the `.gbai` package format:

```
template-name.gbai/
├── template-name.gbot/       # Configuration
│   └── config.csv            # Bot settings
├── template-name.gbkb/       # Knowledge base
│   └── *.md                  # Documentation files
├── template-name.gbdialog/   # Dialog scripts
│   ├── start.bas             # Entry point
│   └── *.bas                 # Tool scripts
└── template-name.gbdrive/    # File storage
    └── templates/            # Document templates
```

## Available Templates

### Business Operations

| Template | Description | Path |
|----------|-------------|------|
| CRM Contacts | Contact management | `crm/contacts.gbai` |
| Sales Pipeline | Deal tracking | `crm/sales-pipeline.gbai` |
| HR Employees | Employee management | `hr/employees.gbai` |
| IT Helpdesk | Ticket system | `it/helpdesk.gbai` |

### Productivity

| Template | Description | Path |
|----------|-------------|------|
| Office | Document automation | `productivity/office.gbai` |
| Reminder | Task reminders | `productivity/reminder.gbai` |
| Analytics | Data dashboards | `platform/analytics.gbai` |

### Integration

| Template | Description | Path |
|----------|-------------|------|
| API Client | External API calls | `integration/api-client.gbai` |
| Public APIs | 70+ free APIs | `integration/public-apis.gbai` |

### Compliance

| Template | Description | Path |
|----------|-------------|------|
| HIPAA Medical | Healthcare compliance | `compliance/hipaa-medical.gbai` |
| Privacy | GDPR/LGPD | `compliance/privacy.gbai` |

## Using a Template

### 1. Copy the Template

```bash
cp -r templates/crm/contacts.gbai templates/mycrm.gbai
```

### 2. Rename Internal Folders

```bash
cd templates/mycrm.gbai
mv contacts.gbot mycrm.gbot
mv contacts.gbkb mycrm.gbkb
mv contacts.gbdialog mycrm.gbdialog
mv contacts.gbdrive mycrm.gbdrive
```

### 3. Update Configuration

Edit `mycrm.gbot/config.csv`:

```csv
name,value
theme-title,My CRM Bot
theme-color1,#2196F3
theme-color2,#E3F2FD
episodic-memory-history,2
```

### 4. Customize Knowledge Base

Edit files in `mycrm.gbkb/`:

```markdown
# My Company CRM Guide

## Adding Contacts
To add a new contact, say "add contact" and provide:
- Name
- Email
- Phone number
- Company

## Searching Contacts
Say "find contact [name]" to search.
```

### 5. Modify Dialogs

Edit `mycrm.gbdialog/start.bas`:

```basic
' My CRM Bot - Start Script

USE KB "mycrm.gbkb"
USE TOOL "add-contact"
USE TOOL "search-contact"

SET CONTEXT "crm" AS "You are a CRM assistant for My Company."

TALK "Welcome to My Company CRM!"
TALK "I can help you manage contacts and deals."

ADD SUGGESTION "add" AS "Add new contact"
ADD SUGGESTION "search" AS "Find a contact"
ADD SUGGESTION "list" AS "Show all contacts"
```

### 6. Restart Server

```bash
cargo run
```

## Creating Custom Templates

### Step 1: Create Package Structure

```bash
mkdir -p templates/mytemplate.gbai/{mytemplate.gbot,mytemplate.gbkb,mytemplate.gbdialog,mytemplate.gbdrive}
```

### Step 2: Create Configuration

```csv
# mytemplate.gbot/config.csv
name,value
theme-title,My Template
theme-color1,#1565C0
theme-color2,#E3F2FD
theme-logo,https://example.com/logo.svg
episodic-memory-history,2
episodic-memory-threshold,4
```

### Step 3: Create Start Dialog

```basic
' mytemplate.gbdialog/start.bas

' Load knowledge base
USE KB "mytemplate.gbkb"

' Register tools
USE TOOL "my-action"

' Set AI context
SET CONTEXT "assistant" AS "You are a helpful assistant."

' Welcome message
BEGIN TALK
    **Welcome to My Template!**
    
    I can help you with:
    • Feature 1
    • Feature 2
    • Feature 3
END TALK

' Add quick suggestions
CLEAR SUGGESTIONS
ADD SUGGESTION "help" AS "Show help"
ADD SUGGESTION "action" AS "Do something"
```

### Step 4: Create Tools

```basic
' mytemplate.gbdialog/my-action.bas

PARAM item_name AS STRING LIKE "Example" DESCRIPTION "Name of the item"
PARAM quantity AS INTEGER LIKE 1 DESCRIPTION "How many items"

DESCRIPTION "Performs an action with the specified item."

' Validate input
IF item_name = "" THEN
    TALK "Please provide an item name."
    item_name = HEAR
END IF

' Process
let result = item_name + " x " + quantity

' Save record
SAVE "items.csv", item_name, quantity, result

' Respond
TALK "✅ Created: " + result

RETURN result
```

### Step 5: Add Knowledge Base

```markdown
# mytemplate.gbkb/guide.md

# My Template Guide

## Overview
This template helps you accomplish specific tasks.

## Features

### Feature 1
Description of feature 1.

### Feature 2
Description of feature 2.

## FAQ

### How do I get started?
Just say "help" to see available commands.

### How do I contact support?
Email support@example.com.
```

## Template Best Practices

### Configuration

- Use clear, descriptive `theme-title`
- Choose accessible color combinations
- Set appropriate `episodic-memory-history` (2-4 recommended)

### Knowledge Base

- Write clear, concise documentation
- Use headers for organization
- Include FAQ section
- Keep files under 50KB each

### Dialogs

- Always include `start.bas`
- Use `DESCRIPTION` for all tools
- Validate user input
- Provide helpful error messages
- Add suggestions for common actions

### File Organization

```
mytemplate.gbai/
├── mytemplate.gbot/
│   └── config.csv
├── mytemplate.gbkb/
│   ├── guide.md           # Main documentation
│   ├── faq.md             # Frequently asked questions
│   └── troubleshooting.md # Common issues
├── mytemplate.gbdialog/
│   ├── start.bas          # Entry point (required)
│   ├── tool-1.bas         # First tool
│   ├── tool-2.bas         # Second tool
│   └── jobs.bas           # Scheduled tasks
└── mytemplate.gbdrive/
    └── templates/
        └── report.docx    # Document templates
```

## Sharing Templates

### Export

```bash
tar -czf mytemplate.tar.gz templates/mytemplate.gbai
```

### Import

```bash
tar -xzf mytemplate.tar.gz -C templates/
```

### Version Control

Templates work well with Git:

```bash
cd templates/mytemplate.gbai
git init
git add .
git commit -m "Initial template"
```

## Next Steps

- [BASIC Language Reference](../reference/basic-language.md) - Complete keyword list
- [API Reference](../api/README.md) - Integrate with external systems
- [Deployment Guide](deployment.md) - Production setup