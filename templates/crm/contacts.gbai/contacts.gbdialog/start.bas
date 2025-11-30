' Contact Directory Template - Start Script
' Manages contacts, companies, and contact information

' ============================================================================
' SETUP TOOLS - Register available contact management tools
' ============================================================================

ADD TOOL "add-contact"
ADD TOOL "search-contact"
ADD TOOL "update-contact"
ADD TOOL "list-contacts"
ADD TOOL "add-company"
ADD TOOL "contact-history"

' ============================================================================
' SETUP KNOWLEDGE BASE
' ============================================================================

USE KB "contacts.gbkb"

' ============================================================================
' SET CONTEXT FOR AI
' ============================================================================

SET CONTEXT "contact directory" AS "You are a contact management assistant helping organize and search contacts. You can help with: adding new contacts, searching the directory, updating contact information, managing company records, and viewing contact history. Always maintain data accuracy and help users find contacts quickly."

' ============================================================================
' SETUP SUGGESTIONS
' ============================================================================

CLEAR SUGGESTIONS

ADD SUGGESTION "add" AS "Add a new contact"
ADD SUGGESTION "search" AS "Search contacts"
ADD SUGGESTION "companies" AS "View companies"
ADD SUGGESTION "recent" AS "Recent contacts"
ADD SUGGESTION "export" AS "Export contacts"

' ============================================================================
' WELCOME MESSAGE
' ============================================================================

BEGIN TALK
    📇 **Contact Directory**

    Welcome! I'm your contact management assistant.

    **What I can help you with:**
    • ➕ Add new contacts and companies
    • 🔍 Search contacts by name, email, or company
    • ✏️ Update contact information
    • 🏢 Manage company records
    • 📋 View contact history and notes
    • 📤 Export contact lists

    Just tell me what you need or select an option below!
END TALK

' ============================================================================
' SYSTEM PROMPT FOR AI INTERACTIONS
' ============================================================================

BEGIN SYSTEM PROMPT
    You are a contact directory assistant. Your responsibilities include:

    1. Helping users add and manage contact records
    2. Searching contacts efficiently by any field
    3. Maintaining accurate contact information
    4. Organizing contacts by company and tags
    5. Tracking interaction history with contacts

    Contact fields include:
    - Name (first and last)
    - Email address
    - Phone number
    - Company name
    - Job title
    - Address
    - Tags/categories
    - Notes

    Always confirm before making changes to existing contacts.
    When searching, be flexible with partial matches.
    Suggest adding missing information when appropriate.
END SYSTEM PROMPT
