# CRM Contacts Template (contacts.gbai)

A General Bots template for managing contact directories with search, add, update, and company management capabilities.

## Overview

The Contacts template provides a complete contact management system with natural language interaction. Users can add new contacts, search the directory, manage company records, and track contact history through conversational AI.

## Features

- **Contact Management** - Add, update, search, and delete contacts
- **Company Records** - Automatic company creation and association
- **Flexible Search** - Search by name, email, company, or phone
- **Activity Tracking** - Log all contact interactions
- **Tag System** - Organize contacts with custom tags
- **Export Capabilities** - Export contact lists in various formats

## Package Structure

```
contacts.gbai/
├── README.md
├── contacts.gbdialog/
│   ├── start.bas           # Main entry point and tool registration
│   ├── add-contact.bas     # Add new contacts
│   └── search-contact.bas  # Search contact directory
├── contacts.gbkb/          # Knowledge base for contact help
└── contacts.gbot/
    └── config.csv          # Bot configuration
```

## Scripts

| File | Description |
|------|-------------|
| `start.bas` | Initializes tools, sets context, and displays welcome menu |
| `add-contact.bas` | Creates new contact records with validation |
| `search-contact.bas` | Searches directory by multiple fields |

## Available Tools

The template registers these tools for LLM access:

| Tool | Description |
|------|-------------|
| `add-contact` | Add a new contact to the directory |
| `search-contact` | Search contacts by any field |
| `update-contact` | Modify existing contact information |
| `list-contacts` | List all contacts with optional filters |
| `add-company` | Create a new company record |
| `contact-history` | View interaction history for a contact |

## Data Schema

### Contacts Table

| Field | Type | Description |
|-------|------|-------------|
| `contactid` | String | Unique identifier (CON-YYYYMMDD-XXXX) |
| `firstname` | String | Contact's first name |
| `lastname` | String | Contact's last name |
| `fullname` | String | Combined full name |
| `email` | Email | Email address |
| `phone` | Phone | Phone number |
| `companyname` | String | Associated company |
| `jobtitle` | String | Job title or role |
| `tags` | String | Comma-separated tags |
| `notes` | String | Additional notes |
| `createdby` | String | User who created the record |
| `createdat` | DateTime | Creation timestamp |

### Companies Table

| Field | Type | Description |
|-------|------|-------------|
| `companyid` | String | Unique identifier |
| `name` | String | Company name |
| `createdat` | DateTime | Creation timestamp |

### Activities Table

| Field | Type | Description |
|-------|------|-------------|
| `contactid` | String | Related contact ID |
| `action` | String | Action description |
| `createdby` | String | User who performed action |
| `createdat` | DateTime | Activity timestamp |

## Usage

### Adding a Contact

Users can add contacts naturally:

- "Add John Smith from Acme Corp"
- "Create a new contact for jane@company.com"
- "Add contact: Mike Johnson, Sales Manager at TechCo"

Or provide structured input:

```
First Name: John
Last Name: Smith
Email: john.smith@acme.com
Phone: +1-555-123-4567
Company: Acme Corporation
Job Title: VP of Sales
Tags: customer, vip
Notes: Met at trade show
```

### Searching Contacts

Search using natural language:

- "Find contacts at Acme"
- "Search for John"
- "Look up john.smith@acme.com"
- "Find all VIP contacts"

Search filters:

| Filter | Example |
|--------|---------|
| By name | "search John Smith" |
| By email | "search john@company.com" |
| By company | "find contacts at Microsoft" |
| By phone | "lookup +1-555-1234" |
| By tag | "show all VIP contacts" |

### Managing Companies

Companies are auto-created when adding contacts:

```basic
' When adding a contact with a new company
IF companyname THEN
    existingcompany = FIND "companies.csv", "name=" + companyname
    IF COUNT(existingcompany) = 0 THEN
        ' Auto-create company record
        SAVE "companies.csv", companyid, companyname, createdat
    END IF
END IF
```

## Configuration

Configure in `contacts.gbot/config.csv`:

| Parameter | Description | Example |
|-----------|-------------|---------|
| `Theme Color` | UI accent color | `blue` |
| `Default Tags` | Auto-applied tags | `new,prospect` |
| `Require Email` | Email required? | `true` |
| `Duplicate Check` | Check for duplicates | `true` |

## Customization

### Adding Custom Fields

Extend the contact schema in `add-contact.bas`:

```basic
PARAM department AS STRING LIKE "Engineering" DESCRIPTION "Department name" OPTIONAL
PARAM linkedin AS STRING LIKE "linkedin.com/in/john" DESCRIPTION "LinkedIn profile" OPTIONAL

' Include in save
SAVE "contacts.csv", contactid, firstname, lastname, fullname, email, phone, 
     companyname, jobtitle, department, linkedin, tags, notes, createdby, createdat
```

### Custom Search Filters

Add specialized search in `search-contact.bas`:

```basic
IF searchby = "department" THEN
    results = FIND "contacts.csv", "department LIKE " + searchterm
END IF

IF searchby = "recent" THEN
    cutoff = DATEADD(NOW(), -7, "day")
    results = FIND "contacts.csv", "createdat >= " + cutoff
END IF
```

### Integration with Email

Send welcome emails to new contacts:

```basic
' In add-contact.bas after SAVE
IF email THEN
    subject = "Thank you for connecting!"
    body = "Hi " + firstname + ", we look forward to working with you."
    SEND EMAIL email, subject, body
END IF
```

### Integration with CRM Pipeline

Connect contacts to leads:

```basic
' Convert contact to lead
PARAM convert_to_lead AS BOOLEAN DESCRIPTION "Create lead from contact" OPTIONAL

IF convert_to_lead THEN
    WITH new_lead
        id = FORMAT(GUID())
        name = fullname
        email = email
        phone = phone
        company = companyname
        source = "contact_conversion"
        status = "new"
        created_at = NOW()
    END WITH
    
    SAVE "leads.csv", new_lead
    TALK "Lead created from contact: " + fullname
END IF
```

## Example Conversations

### Adding a Contact
```
User: Add a new contact
Bot: I'll help you add a new contact. What is their first name?
User: Sarah
Bot: And their last name?
User: Johnson
Bot: What is Sarah's email address?
User: sarah.johnson@techstartup.io
Bot: Phone number? (optional)
User: 555-987-6543
Bot: What company does Sarah work for?
User: TechStartup Inc
Bot: What is her job title?
User: CTO
Bot: ✅ Contact added: Sarah Johnson
     ID: CON-20240115-4821
     Email: sarah.johnson@techstartup.io
     Phone: 555-987-6543
     Company: TechStartup Inc
     Title: CTO
```

### Searching Contacts
```
User: Find contacts at TechStartup
Bot: Found 2 contact(s):
     ---
     **Sarah Johnson**
     sarah.johnson@techstartup.io
     555-987-6543
     TechStartup Inc
     CTO
     ID: CON-20240115-4821
     ---
     **Mike Chen**
     mike.chen@techstartup.io
     TechStartup Inc
     Developer
     ID: CON-20240110-3392
```

## Best Practices

1. **Validate input** - Use `PARAM` type validation for email and phone
2. **Check duplicates** - Search before creating to avoid duplicates
3. **Log activities** - Track all changes for audit trail
4. **Use tags** - Organize contacts with meaningful tags
5. **Keep notes** - Add context about how/where you met contacts
6. **Regular cleanup** - Archive inactive contacts periodically

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Duplicate contacts | Enable duplicate checking in config |
| Search returns nothing | Try broader search terms |
| Company not linked | Ensure company name matches exactly |
| Missing activities | Check activity logging is enabled |

## Related Templates

- `crm.gbai` - Full CRM with leads, opportunities, and pipeline
- `marketing.gbai` - Marketing automation with contact segmentation
- `office.gbai` - Office productivity with contact directory

## Use Cases

- **Sales Teams** - Manage prospect and customer contacts
- **HR Departments** - Employee and candidate directories
- **Event Management** - Attendee and speaker contacts
- **Networking** - Professional contact management
- **Customer Support** - Customer contact lookup

## License

AGPL-3.0 - Part of General Bots Open Source Platform.

---

**Pragmatismo** - General Bots