ADD TOOL "add-contact"
ADD TOOL "search-contact"
ADD TOOL "update-contact"
ADD TOOL "list-contacts"
ADD TOOL "add-company"
ADD TOOL "contact-history"

USE KB "contacts.gbkb"

SET CONTEXT "contact directory" AS "You are a contact management assistant helping organize and search contacts. Help with adding new contacts, searching the directory, updating contact information, managing company records, and viewing contact history."

CLEAR SUGGESTIONS

ADD SUGGESTION "add" AS "bot-suggest-add-a-new-contact"
ADD SUGGESTION "search" AS "bot-suggest-search-contacts"
ADD SUGGESTION "companies" AS "bot-suggest-view-companies"
ADD SUGGESTION "recent" AS "bot-suggest-recent-contacts"
ADD SUGGESTION "export" AS "bot-suggest-export-contacts"

BEGIN TALK
**Contact Directory**

I can help you with:
• Add new contacts and companies
• Search by name, email, or company
• Update contact information
• Manage company records
• View contact history
• Export contact lists

Select an option or tell me what you need.
END TALK

BEGIN SYSTEM PROMPT
You are a contact directory assistant.

Contact fields: name, email, phone, company, job title, address, tags, notes.

Confirm before making changes to existing contacts.
Be flexible with partial matches when searching.
Suggest adding missing information when appropriate.
END SYSTEM PROMPT
