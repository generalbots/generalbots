# CRM Contacts Template

The CRM Contacts template provides a complete contact management solution with natural language interface. Users can add, search, update, and manage contacts through conversational interactions.

## Overview

| Property | Value |
|----------|-------|
| Template ID | `crm/contacts.gbai` |
| Category | CRM |
| Complexity | Intermediate |
| Dependencies | Database, Email (optional) |

## Features

- Add new contacts with validation
- Search contacts by name, email, or phone
- Update contact information
- Tag and categorize contacts
- Export contacts to CSV
- Integration with email campaigns

## Installation

```basic
DEPLOY TEMPLATE "crm/contacts.gbai"
```

Or copy the template folder to your work directory:

```bash
cp -r templates/crm/contacts.gbai work/
```

## Configuration

Add these settings to your `config.csv`:

| Key | Value | Description |
|-----|-------|-------------|
| `contacts-table` | `contacts` | Database table name |
| `require-email` | `true` | Require email for new contacts |
| `allow-duplicates` | `false` | Allow duplicate entries |
| `default-tags` | `lead` | Default tags for new contacts |

## Sample Conversation

<div class="chat-container">

<div class="chat-message user">
<div class="chat-bubble">
<div class="chat-sender">You</div>
<div class="chat-text">Hi, I need to add a new contact</div>
<div class="chat-time">10:32 AM</div>
</div>
</div>

<div class="chat-message bot">
<div class="chat-bubble">
<div class="chat-sender">CRM Bot</div>
<div class="chat-text">I'd be happy to help you add a new contact! 📇

What's the contact's full name?</div>
<div class="chat-time">10:32 AM</div>
</div>
</div>

<div class="chat-message user">
<div class="chat-bubble">
<div class="chat-sender">You</div>
<div class="chat-text">Maria Santos</div>
<div class="chat-time">10:32 AM</div>
</div>
</div>

<div class="chat-message bot">
<div class="chat-bubble">
<div class="chat-sender">CRM Bot</div>
<div class="chat-text">Got it! Maria Santos.

What's Maria's email address?</div>
<div class="chat-time">10:32 AM</div>
</div>
</div>

<div class="chat-message user">
<div class="chat-bubble">
<div class="chat-sender">You</div>
<div class="chat-text">maria.santos@company.com</div>
<div class="chat-time">10:33 AM</div>
</div>
</div>

<div class="chat-message bot">
<div class="chat-bubble">
<div class="chat-sender">CRM Bot</div>
<div class="chat-text">Perfect! And what's the phone number? (You can skip this by typing "skip")</div>
<div class="chat-time">10:33 AM</div>
</div>
</div>

<div class="chat-message user">
<div class="chat-bubble">
<div class="chat-sender">You</div>
<div class="chat-text">+55 11 98765-4321</div>
<div class="chat-time">10:33 AM</div>
</div>
</div>

<div class="chat-message bot">
<div class="chat-bubble">
<div class="chat-sender">CRM Bot</div>
<div class="chat-text">Which company does Maria work for?</div>
<div class="chat-time">10:33 AM</div>
</div>
</div>

<div class="chat-message user">
<div class="chat-bubble">
<div class="chat-sender">You</div>
<div class="chat-text">Tech Solutions Ltd</div>
<div class="chat-time">10:34 AM</div>
</div>
</div>

<div class="chat-message bot">
<div class="chat-bubble">
<div class="chat-sender">CRM Bot</div>
<div class="chat-text">Great! Here's the contact I'm about to save:

📋 **Contact Details**
━━━━━━━━━━━━━━━━━
👤 **Name:** Maria Santos
📧 **Email:** maria.santos@company.com
📱 **Phone:** +55 11 98765-4321
🏢 **Company:** Tech Solutions Ltd
🏷️ **Tags:** lead

Should I save this contact?</div>
<div class="chat-time">10:34 AM</div>
</div>
</div>

<div class="chat-message user">
<div class="chat-bubble">
<div class="chat-sender">You</div>
<div class="chat-text">Yes, save it</div>
<div class="chat-time">10:34 AM</div>
</div>
</div>

<div class="chat-message bot">
<div class="chat-bubble">
<div class="chat-sender">CRM Bot</div>
<div class="chat-text">✅ Contact saved successfully!

Maria Santos has been added to your contacts with ID #1247.

What would you like to do next?
• Add another contact
• Search contacts
• View recent contacts</div>
<div class="chat-time">10:34 AM</div>
</div>
</div>

<div class="chat-message user">
<div class="chat-bubble">
<div class="chat-sender">You</div>
<div class="chat-text">Search for contacts at Tech Solutions</div>
<div class="chat-time">10:35 AM</div>
</div>
</div>

<div class="chat-message bot">
<div class="chat-bubble">
<div class="chat-sender">CRM Bot</div>
<div class="chat-text">🔍 Found 3 contacts at "Tech Solutions":

1. **Maria Santos** - maria.santos@company.com
   📱 +55 11 98765-4321 | 🏷️ lead

2. **João Silva** - joao.silva@techsolutions.com
   📱 +55 11 91234-5678 | 🏷️ customer

3. **Ana Costa** - ana.costa@techsolutions.com
   📱 +55 11 99876-5432 | 🏷️ prospect

Type a number to see full details, or tell me what else you need.</div>
<div class="chat-time">10:35 AM</div>
</div>
</div>

</div>

## Dialog Files

### start.bas

The main entry point for the CRM contacts bot:

```basic
' CRM Contacts - Main Dialog
' Handles contact management through conversation

SET CONTEXT "You are a helpful CRM assistant that manages contacts. 
Be professional and efficient. Always confirm before saving data."

ADD SUGGESTION "add" AS "Add new contact"
ADD SUGGESTION "search" AS "Search contacts"
ADD SUGGESTION "recent" AS "View recent"
ADD SUGGESTION "export" AS "Export to CSV"

BEGIN TALK
Welcome to Contact Manager! 📇

I can help you:
• Add new contacts
• Search existing contacts  
• Update contact information
• Export your contact list

What would you like to do?
END TALK
```

### add-contact.bas

Handles adding new contacts:

```basic
' Add Contact Dialog

TALK "I'd be happy to help you add a new contact! 📇"
TALK "What's the contact's full name?"

HEAR contact_name AS STRING

TALK "Got it! " + contact_name + "."
TALK "What's " + contact_name + "'s email address?"

HEAR contact_email AS EMAIL

TALK "Perfect! And what's the phone number? (You can skip this by typing \"skip\")"

HEAR contact_phone AS STRING

IF contact_phone = "skip" THEN
    contact_phone = ""
END IF

TALK "Which company does " + contact_name + " work for?"

HEAR contact_company AS STRING

' Build confirmation message
WITH contact_data
    name = contact_name
    email = contact_email
    phone = contact_phone
    company = contact_company
    tags = "lead"
    created_at = NOW()
END WITH

TALK "Great! Here's the contact I'm about to save:"
TALK ""
TALK "📋 **Contact Details**"
TALK "━━━━━━━━━━━━━━━━━"
TALK "👤 **Name:** " + contact_name
TALK "📧 **Email:** " + contact_email
TALK "📱 **Phone:** " + contact_phone
TALK "🏢 **Company:** " + contact_company
TALK "🏷️ **Tags:** lead"
TALK ""
TALK "Should I save this contact?"

HEAR confirmation AS STRING

IF INSTR(LOWER(confirmation), "yes") > 0 OR INSTR(LOWER(confirmation), "save") > 0 THEN
    SAVE "contacts", contact_data
    
    contact_id = LAST("contacts", "id")
    
    TALK "✅ Contact saved successfully!"
    TALK ""
    TALK contact_name + " has been added to your contacts with ID #" + contact_id + "."
ELSE
    TALK "No problem! The contact was not saved."
END IF

TALK ""
TALK "What would you like to do next?"
TALK "• Add another contact"
TALK "• Search contacts"
TALK "• View recent contacts"
```

### search-contact.bas

Handles contact search:

```basic
' Search Contact Dialog

TALK "🔍 What would you like to search for?"
TALK "You can search by name, email, company, or phone number."

HEAR search_term AS STRING

' Search across multiple fields
results = FIND "contacts" WHERE 
    name LIKE "%" + search_term + "%" OR
    email LIKE "%" + search_term + "%" OR
    company LIKE "%" + search_term + "%" OR
    phone LIKE "%" + search_term + "%"

result_count = COUNT(results)

IF result_count = 0 THEN
    TALK "No contacts found matching \"" + search_term + "\"."
    TALK ""
    TALK "Would you like to:"
    TALK "• Try a different search"
    TALK "• Add a new contact"
ELSE
    TALK "🔍 Found " + result_count + " contact(s) matching \"" + search_term + "\":"
    TALK ""
    
    counter = 1
    FOR EACH contact IN results
        TALK counter + ". **" + contact.name + "** - " + contact.email
        TALK "   📱 " + contact.phone + " | 🏷️ " + contact.tags
        TALK ""
        counter = counter + 1
    NEXT
    
    TALK "Type a number to see full details, or tell me what else you need."
END IF
```

## Database Schema

The template creates this table structure:

```sql
CREATE TABLE contacts (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) UNIQUE,
    phone VARCHAR(50),
    company VARCHAR(255),
    tags VARCHAR(255) DEFAULT 'lead',
    notes TEXT,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW(),
    created_by UUID REFERENCES users(id)
);

CREATE INDEX idx_contacts_email ON contacts(email);
CREATE INDEX idx_contacts_company ON contacts(company);
CREATE INDEX idx_contacts_tags ON contacts(tags);
```

## API Endpoints

The template exposes these REST endpoints:

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/contacts` | List all contacts |
| GET | `/api/v1/contacts/:id` | Get single contact |
| POST | `/api/v1/contacts` | Create contact |
| PUT | `/api/v1/contacts/:id` | Update contact |
| DELETE | `/api/v1/contacts/:id` | Delete contact |
| GET | `/api/v1/contacts/search?q=` | Search contacts |
| GET | `/api/v1/contacts/export` | Export to CSV |

## Customization

### Adding Custom Fields

Edit `tables.bas` to add custom fields:

```basic
TABLE contacts
    FIELD id AS INTEGER PRIMARY KEY
    FIELD name AS STRING(255) REQUIRED
    FIELD email AS EMAIL UNIQUE
    FIELD phone AS PHONE
    FIELD company AS STRING(255)
    FIELD tags AS STRING(255)
    FIELD notes AS TEXT
    ' Add your custom fields below
    FIELD linkedin AS STRING(255)
    FIELD job_title AS STRING(255)
    FIELD lead_source AS STRING(100)
    FIELD lead_score AS INTEGER DEFAULT 0
END TABLE
```

### Changing Default Tags

Update `config.csv`:

```csv
key,value
default-tags,"prospect,website"
```

### Adding Validation

Edit `add-contact.bas` to add custom validation:

```basic
' Validate email domain
IF NOT INSTR(contact_email, "@company.com") THEN
    TALK "⚠️ Warning: This email is not from your company domain."
END IF

' Check for duplicates
existing = FIND "contacts" WHERE email = contact_email
IF COUNT(existing) > 0 THEN
    TALK "⚠️ A contact with this email already exists!"
    TALK "Would you like to update the existing contact instead?"
    ' Handle duplicate logic
END IF
```

## Related Templates

- [Sales Pipeline](./template-sales-pipeline.md) - Track deals and opportunities
- [Marketing Campaigns](./template-marketing.md) - Email campaigns and automation
- [Customer Support](./template-helpdesk.md) - Support ticket management

## Support

For issues with this template:
- Check the [troubleshooting guide](../chapter-13-community/README.md)
- Open an issue on [GitHub](https://github.com/GeneralBots/BotServer/issues)
- Join the [community chat](https://discord.gg/generalbots)