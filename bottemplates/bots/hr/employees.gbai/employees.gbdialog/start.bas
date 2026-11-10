ADD TOOL "add-employee"
ADD TOOL "update-employee"
ADD TOOL "search-employee"
ADD TOOL "employee-directory"
ADD TOOL "org-chart"
ADD TOOL "emergency-contacts"

USE KB "employees.gbkb"

SET CONTEXT "employee management" AS "You are an HR assistant helping manage employee information. Help with adding new employees, updating records, searching the directory, viewing org charts, and managing emergency contacts. Maintain confidentiality of employee data."

CLEAR SUGGESTIONS

ADD SUGGESTION "directory" AS "bot-suggest-employee-directory"
ADD SUGGESTION "add" AS "bot-suggest-add-new-employee"
ADD SUGGESTION "search" AS "bot-suggest-search-employee"
ADD SUGGESTION "org" AS "bot-suggest-organization-chart"
ADD SUGGESTION "emergency" AS "bot-suggest-emergency-contacts"

BEGIN TALK
**Employee Management System**

I can help you with:
• View employee directory
• Add new employees
• Search for employees
• View organization chart
• Manage emergency contacts
• Generate employee reports

Select an option or tell me what you need.
END TALK

BEGIN SYSTEM PROMPT
You are an HR assistant for the Employee Management System.

Confirm sensitive operations before executing.
Never expose salaries or personal IDs without authorization.
Use professional and helpful language.
END SYSTEM PROMPT
