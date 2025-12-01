# CRM Contacts Template

The CRM Contacts template provides a complete contact management solution with natural language interface. Users can add, search, update, and manage contacts through conversational interactions.

## Overview

| Property | Value |
|----------|-------|
| Template ID | `crm/contacts.gbai` |
| Category | CRM |
| Complexity | Intermediate |

## Features

- Add new contacts with validation
- Search contacts by name, email, or phone
- Update contact information
- Tag and categorize contacts
- Export contacts to CSV

## Installation

Copy the template folder to your work directory or use the Sources app to create from template.

## Configuration

Add these settings to your `config.csv`:

| Key | Value | Description |
|-----|-------|-------------|
| `contacts-table` | `contacts` | Database table name |
| `require-email` | `true` | Require email for new contacts |
| `allow-duplicates` | `false` | Allow duplicate entries |
| `default-tags` | `lead` | Default tags for new contacts |

---

## Sample Conversations

### Adding a New Contact

<div class="wa-chat">
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>I'd be happy to help you add a new contact! 📇</p>
      <p>What's the contact's full name?</p>
      <div class="wa-time">10:32</div>
    </div>
  </div>
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Maria Santos</p>
      <div class="wa-time">10:32</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Got it! Maria Santos.</p>
      <p>What's Maria's email address?</p>
      <div class="wa-time">10:32</div>
    </div>
  </div>
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>maria.santos@company.com</p>
      <div class="wa-time">10:33</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Perfect! And what's the phone number?</p>
      <p>(You can skip this by typing "skip")</p>
      <div class="wa-time">10:33</div>
    </div>
  </div>
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>+55 11 98765-4321</p>
      <div class="wa-time">10:33</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Which company does Maria work for?</p>
      <div class="wa-time">10:33</div>
    </div>
  </div>
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Tech Solutions Ltd</p>
      <div class="wa-time">10:34</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Great! Here's the contact I'm about to save:</p>
      <p>📋 <strong>Contact Details</strong></p>
      <p>━━━━━━━━━━━━━━━━━</p>
      <p>👤 <strong>Name:</strong> Maria Santos</p>
      <p>📧 <strong>Email:</strong> maria.santos@company.com</p>
      <p>📱 <strong>Phone:</strong> +55 11 98765-4321</p>
      <p>🏢 <strong>Company:</strong> Tech Solutions Ltd</p>
      <p>🏷️ <strong>Tags:</strong> lead</p>
      <p>Should I save this contact?</p>
      <div class="wa-time">10:34</div>
    </div>
  </div>
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Yes, save it</p>
      <div class="wa-time">10:34</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>✅ Contact saved successfully!</p>
      <p>Maria Santos has been added to your contacts with ID #1247.</p>
      <p>What would you like to do next?</p>
      <p>• Add another contact</p>
      <p>• Search contacts</p>
      <p>• View recent contacts</p>
      <div class="wa-time">10:34</div>
    </div>
  </div>
</div>

---

### Searching Contacts

<div class="wa-chat">
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Search for contacts at Tech Solutions</p>
      <div class="wa-time">10:35</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>🔍 Found 3 contacts at "Tech Solutions":</p>
      <p></p>
      <p>1. <strong>Maria Santos</strong> - maria.santos@company.com</p>
      <p>   📱 +55 11 98765-4321 | 🏷️ lead</p>
      <p></p>
      <p>2. <strong>João Silva</strong> - joao.silva@techsolutions.com</p>
      <p>   📱 +55 11 91234-5678 | 🏷️ customer</p>
      <p></p>
      <p>3. <strong>Ana Costa</strong> - ana.costa@techsolutions.com</p>
      <p>   📱 +55 11 99876-5432 | 🏷️ prospect</p>
      <p></p>
      <p>Type a number to see full details, or tell me what else you need.</p>
      <div class="wa-time">10:35</div>
    </div>
  </div>
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>2</p>
      <div class="wa-time">10:35</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>📋 <strong>Contact Details</strong></p>
      <p>━━━━━━━━━━━━━━━━━</p>
      <p>👤 <strong>Name:</strong> João Silva</p>
      <p>📧 <strong>Email:</strong> joao.silva@techsolutions.com</p>
      <p>📱 <strong>Phone:</strong> +55 11 91234-5678</p>
      <p>🏢 <strong>Company:</strong> Tech Solutions Ltd</p>
      <p>🏷️ <strong>Tags:</strong> customer</p>
      <p>📅 <strong>Added:</strong> Jan 15, 2025</p>
      <p></p>
      <p>What would you like to do?</p>
      <p>• Edit this contact</p>
      <p>• Delete this contact</p>
      <p>• Send email</p>
      <div class="wa-time">10:35</div>
    </div>
  </div>
</div>

---

### Updating a Contact

<div class="wa-chat">
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Update Maria's phone number</p>
      <div class="wa-time">10:40</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>I found Maria Santos in your contacts.</p>
      <p>Current phone: +55 11 98765-4321</p>
      <p>What's the new phone number?</p>
      <div class="wa-time">10:40</div>
    </div>
  </div>
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>+55 11 99999-8888</p>
      <div class="wa-time">10:40</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>✅ Updated!</p>
      <p>Maria Santos's phone is now +55 11 99999-8888</p>
      <div class="wa-time">10:40</div>
    </div>
  </div>
</div>

---

### Exporting Contacts

<div class="wa-chat">
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Export all contacts to CSV</p>
      <div class="wa-time">11:00</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>📊 Exporting contacts...</p>
      <p></p>
      <p>✅ Export complete!</p>
      <p>📁 <strong>contacts-export-2025-05-15.csv</strong></p>
      <p>• 47 contacts exported</p>
      <p>• File saved to your Drive</p>
      <p></p>
      <p>Would you like me to email this file to you?</p>
      <div class="wa-time">11:00</div>
    </div>
  </div>
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Yes please</p>
      <div class="wa-time">11:00</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>📧 Sent! Check your inbox for the export file.</p>
      <div class="wa-time">11:00</div>
    </div>
  </div>
</div>

---

## What It Can Do

| Capability | Description |
|------------|-------------|
| Add contacts | Collect name, email, phone, company through conversation |
| Search | Find by any field - name, email, company, phone |
| Update | Modify any contact field naturally |
| Delete | Remove contacts with confirmation |
| Tags | Categorize contacts (lead, customer, prospect) |
| Export | Generate CSV files for external use |
| Bulk import | Upload CSV to add multiple contacts |

---

## Customization Ideas

### Add Custom Fields

Configure additional fields like LinkedIn profile, job title, or lead source in your bot's `config.csv`.

### Add Validation

The bot validates email formats and phone numbers automatically. Configure stricter rules as needed.

### Connect to External CRM

Use the `POST` and `GET` keywords to sync contacts with Salesforce, HubSpot, or other CRM systems.

---

## Related Templates

- [Sales Pipeline](./templates.md) - Track deals and opportunities
- [Customer Support](./templates.md) - Support ticket management
- [Template Samples](./template-samples.md) - More conversation examples

---

<style>
.wa-chat{background-color:#e5ddd5;border-radius:8px;padding:20px 15px;margin:20px 0;max-width:500px;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,Helvetica,Arial,sans-serif;font-size:14px}
.wa-message{margin-bottom:10px}
.wa-message.user{text-align:right}
.wa-message.user .wa-bubble{background-color:#dcf8c6;display:inline-block;text-align:left}
.wa-message.bot .wa-bubble{background-color:#fff;display:inline-block}
.wa-bubble{padding:8px 12px;border-radius:8px;box-shadow:0 1px .5px rgba(0,0,0,.13);max-width:85%}
.wa-bubble p{margin:0 0 4px 0;line-height:1.4;color:#303030}
.wa-bubble p:last-child{margin-bottom:0}
.wa-time{font-size:11px;color:#8696a0;text-align:right;margin-top:4px}
</style>