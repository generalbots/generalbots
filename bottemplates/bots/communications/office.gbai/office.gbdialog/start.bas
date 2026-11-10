ADD TOOL "calendar"
ADD TOOL "tasks"
ADD TOOL "documents"
ADD TOOL "meetings"
ADD TOOL "notes"

CLEAR SUGGESTIONS

ADD SUGGESTION "manager" AS "bot-suggest-manager-access"
ADD SUGGESTION "developer" AS "bot-suggest-developer-access"
ADD SUGGESTION "customer" AS "bot-suggest-customer-support"
ADD SUGGESTION "hr" AS "bot-suggest-hr-resources"
ADD SUGGESTION "finance" AS "bot-suggest-finance-tools"

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
    ADD SUGGESTION "performance" AS "bot-suggest-team-performance"
    ADD SUGGESTION "report" AS "bot-suggest-generate-report"
    ADD SUGGESTION "meeting" AS "bot-suggest-schedule-meeting"

  CASE "developer"
    ADD SUGGESTION "docs" AS "bot-suggest-search-documentation"
    ADD SUGGESTION "api" AS "bot-suggest-api-reference"
    ADD SUGGESTION "review" AS "bot-suggest-code-review-checklist"

  CASE "customer"
    ADD SUGGESTION "order" AS "bot-suggest-track-my-order"
    ADD SUGGESTION "product" AS "bot-suggest-product-information"
    ADD SUGGESTION "support" AS "bot-suggest-contact-support"

  CASE "hr"
    ADD SUGGESTION "handbook" AS "bot-suggest-employee-handbook"
    ADD SUGGESTION "benefits" AS "bot-suggest-benefits-overview"
    ADD SUGGESTION "onboard" AS "bot-suggest-new-hire-checklist"

  CASE "finance"
    ADD SUGGESTION "expense" AS "bot-suggest-expense-policy"
    ADD SUGGESTION "budget" AS "bot-suggest-budget-status"
    ADD SUGGESTION "approval" AS "bot-suggest-approval-workflow"

  DEFAULT
    ADD SUGGESTION "help" AS "bot-suggest-help"
    ADD SUGGESTION "directory" AS "bot-suggest-contact-directory"
    ADD SUGGESTION "hours" AS "bot-suggest-office-hours"
END SWITCH

BEGIN SYSTEM PROMPT
You are a role-based office assistant.

Current user role: ${role}

Adapt your responses and suggestions based on the user's role.
Maintain professional and helpful communication.
Route complex requests to appropriate specialists when needed.
END SYSTEM PROMPT
