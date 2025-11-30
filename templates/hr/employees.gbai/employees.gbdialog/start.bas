' Employee Management Template - Start Script
' This script runs when a user first connects to the bot
' Sets up tools, knowledge base, and provides welcome message

' ============================================================================
' SETUP TOOLS - Register available employee management tools
' ============================================================================

ADD TOOL "add-employee"
ADD TOOL "update-employee"
ADD TOOL "search-employee"
ADD TOOL "employee-directory"
ADD TOOL "org-chart"
ADD TOOL "emergency-contacts"

' ============================================================================
' SETUP KNOWLEDGE BASE
' ============================================================================

USE KB "employees.gbkb"

' ============================================================================
' SET CONTEXT FOR AI
' ============================================================================

SET CONTEXT "employee management" AS "You are an HR assistant helping manage employee information. You can help with: adding new employees, updating employee records, searching the employee directory, viewing org charts, and managing emergency contacts. Always maintain confidentiality of employee data."

' ============================================================================
' SETUP SUGGESTIONS
' ============================================================================

CLEAR SUGGESTIONS

ADD SUGGESTION "directory" AS "Show me the employee directory"
ADD SUGGESTION "add" AS "Add a new employee"
ADD SUGGESTION "search" AS "Search for an employee"
ADD SUGGESTION "org" AS "Show organization chart"
ADD SUGGESTION "emergency" AS "View emergency contacts"

' ============================================================================
' WELCOME MESSAGE
' ============================================================================

BEGIN TALK
    👥 **Employee Management System**

    Welcome! I'm your HR assistant for managing employee information.

    **What I can help you with:**
    • 📋 View the employee directory
    • ➕ Add new employees to the system
    • 🔍 Search for specific employees
    • 🏢 View the organization chart
    • 🆘 Manage emergency contacts
    • 📊 Generate employee reports

    Just tell me what you need or select one of the options below!
END TALK

' ============================================================================
' SYSTEM PROMPT FOR AI INTERACTIONS
' ============================================================================

BEGIN SYSTEM PROMPT
    You are an HR assistant for the Employee Management System. Your responsibilities include:

    1. Helping users add, update, and search employee records
    2. Maintaining data privacy and confidentiality
    3. Providing accurate employee information when requested
    4. Assisting with organizational structure queries
    5. Managing emergency contact information

    Always confirm sensitive operations before executing them.
    Never expose sensitive data like salaries or personal IDs without proper authorization.
    Use professional and helpful language.
END SYSTEM PROMPT
