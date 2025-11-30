ADD TOOL "calendar"
ADD TOOL "tasks"
ADD TOOL "documents"
ADD TOOL "meetings"
ADD TOOL "notes"

CLEAR SUGGESTIONS

ADD SUGGESTION "manager" AS "Manager access"
ADD SUGGESTION "developer" AS "Developer access"
ADD SUGGESTION "customer" AS "Customer support"
ADD SUGGESTION "hr" AS "HR resources"
ADD SUGGESTION "finance" AS "Finance tools"

role = GET role

IF NOT role THEN
    TALK "Welcome to the Office Assistant!"
    TALK "Please select your role:"
    HEAR role AS NAME
    role = LOWER(role)
    SET role, role
END IF

SWITCH role
  CASE "manager"
    SET CONTEXT "You are an executive assistant helping managers with reports, team management, and strategic decisions."
    USE KB "management"
    USE KB "reports"
    USE KB "team-policies"
    TALK "Welcome, Manager! I can help with reports, team management, and policies."

  CASE "developer"
    SET CONTEXT "You are a technical assistant helping developers with documentation, APIs, and coding best practices."
    USE KB "documentation"
    USE KB "apis"
    USE KB "coding-standards"
    TALK "Welcome, Developer! I can help with documentation, APIs, and development guidelines."

  CASE "customer"
    SET CONTEXT "You are a customer service assistant. Be helpful, friendly, and focus on resolving customer issues."
    USE KB "products"
    USE KB "support"
    USE KB "faq"
    TALK "Welcome! How can I assist you today?"

  CASE "hr"
    SET CONTEXT "You are an HR assistant helping with employee matters, policies, and benefits."
    USE KB "hr-policies"
    USE KB "benefits"
    USE KB "onboarding"
    TALK "Welcome, HR! I can help with policies, benefits, and onboarding."

  CASE "finance"
    SET CONTEXT "You are a finance assistant helping with budgets, expenses, and financial reports."
    USE KB "budgets"
    USE KB "expenses"
    USE KB "financial-reports"
    TALK "Welcome, Finance! I can help with budgets, expenses, and reporting."

  DEFAULT
    SET CONTEXT "You are a general office assistant. Help users with common office tasks and direct them to appropriate resources."
    USE KB "general"
    USE KB "faq"
    TALK "Welcome! I'm your office assistant. How can I help?"
END SWITCH

CLEAR SUGGESTIONS

SWITCH role
  CASE "manager"
    ADD SUGGESTION "performance" AS "Team performance"
    ADD SUGGESTION "report" AS "Generate report"
    ADD SUGGESTION "meeting" AS "Schedule meeting"

  CASE "developer"
    ADD SUGGESTION "docs" AS "Search documentation"
    ADD SUGGESTION "api" AS "API reference"
    ADD SUGGESTION "review" AS "Code review checklist"

  CASE "customer"
    ADD SUGGESTION "order" AS "Track my order"
    ADD SUGGESTION "product" AS "Product information"
    ADD SUGGESTION "support" AS "Contact support"

  CASE "hr"
    ADD SUGGESTION "handbook" AS "Employee handbook"
    ADD SUGGESTION "benefits" AS "Benefits overview"
    ADD SUGGESTION "onboard" AS "New hire checklist"

  CASE "finance"
    ADD SUGGESTION "expense" AS "Expense policy"
    ADD SUGGESTION "budget" AS "Budget status"
    ADD SUGGESTION "approval" AS "Approval workflow"

  DEFAULT
    ADD SUGGESTION "help" AS "Help"
    ADD SUGGESTION "directory" AS "Contact directory"
    ADD SUGGESTION "hours" AS "Office hours"
END SWITCH

BEGIN SYSTEM PROMPT
You are a role-based office assistant.

Current user role: ${role}

Adapt your responses and suggestions based on the user's role.
Maintain professional and helpful communication.
Route complex requests to appropriate specialists when needed.
END SYSTEM PROMPT
