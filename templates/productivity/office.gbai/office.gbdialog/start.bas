' Office Bot - Role-based Knowledge Base Routing
' This template demonstrates SWITCH keyword for multi-tenant office environments

' Get user role from session or directory
role = GET role

' If no role set, ask the user
IF role = "" THEN
    TALK "Welcome to the Office Assistant!"
    TALK "Please select your role:"
    ADD SUGGESTION "Manager"
    ADD SUGGESTION "Developer"
    ADD SUGGESTION "Customer"
    ADD SUGGESTION "HR"
    ADD SUGGESTION "Finance"

    role = HEAR "What is your role?"
    role = LOWER(role)
    SET role, role
END IF

' Route to appropriate knowledge bases based on role
SWITCH role
  CASE "manager"
    SET CONTEXT "You are an executive assistant helping managers with reports, team management, and strategic decisions."
    USE KB "management"
    USE KB "reports"
    USE KB "team-policies"
    TALK "Welcome, Manager! I can help you with reports, team management, and company policies."

  CASE "developer"
    SET CONTEXT "You are a technical assistant helping developers with documentation, APIs, and coding best practices."
    USE KB "documentation"
    USE KB "apis"
    USE KB "coding-standards"
    TALK "Welcome, Developer! I can help you with technical documentation, APIs, and development guidelines."

  CASE "customer"
    SET CONTEXT "You are a customer service assistant. Be helpful, friendly, and focus on resolving customer issues."
    USE KB "products"
    USE KB "support"
    USE KB "faq"
    TALK "Welcome! I'm here to help you with our products and services. How can I assist you today?"

  CASE "hr"
    SET CONTEXT "You are an HR assistant helping with employee matters, policies, and benefits."
    USE KB "hr-policies"
    USE KB "benefits"
    USE KB "onboarding"
    TALK "Welcome, HR! I can help you with employee policies, benefits information, and onboarding procedures."

  CASE "finance"
    SET CONTEXT "You are a finance assistant helping with budgets, expenses, and financial reports."
    USE KB "budgets"
    USE KB "expenses"
    USE KB "financial-reports"
    TALK "Welcome, Finance! I can help you with budget queries, expense policies, and financial reporting."

  DEFAULT
    SET CONTEXT "You are a general office assistant. Help users with common office tasks and direct them to appropriate resources."
    USE KB "general"
    USE KB "faq"
    TALK "Welcome! I'm your general office assistant. How can I help you today?"
END SWITCH

' Load common tools available to all roles
USE TOOL "calendar"
USE TOOL "tasks"
USE TOOL "documents"

' Set up suggestions based on role
CLEAR SUGGESTIONS

SWITCH role
  CASE "manager"
    ADD SUGGESTION "Show team performance"
    ADD SUGGESTION "Generate report"
    ADD SUGGESTION "Schedule meeting"

  CASE "developer"
    ADD SUGGESTION "Search documentation"
    ADD SUGGESTION "API reference"
    ADD SUGGESTION "Code review checklist"

  CASE "customer"
    ADD SUGGESTION "Track my order"
    ADD SUGGESTION "Product information"
    ADD SUGGESTION "Contact support"

  CASE "hr"
    ADD SUGGESTION "Employee handbook"
    ADD SUGGESTION "Benefits overview"
    ADD SUGGESTION "New hire checklist"

  CASE "finance"
    ADD SUGGESTION "Expense policy"
    ADD SUGGESTION "Budget status"
    ADD SUGGESTION "Approval workflow"

  DEFAULT
    ADD SUGGESTION "Help"
    ADD SUGGESTION "Contact directory"
    ADD SUGGESTION "Office hours"
END SWITCH
