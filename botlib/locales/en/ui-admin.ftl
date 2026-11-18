# =============================================================================
# General Bots — en — Admin, monitoring and system apps
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# -----------------------------------------------------------------------------
# Admin Panel (additional UI keys)
# -----------------------------------------------------------------------------
admin-panel-title = Admin Panel

# =============================================================================
# MONITORING SCREENS
# =============================================================================
# -----------------------------------------------------------------------------
# Monitoring Dashboard
# -----------------------------------------------------------------------------
monitoring-title = Monitoring Dashboard
monitoring-toggle-view = Toggle View
monitoring-last-updated = Last Updated
monitoring-live-view = Live View
monitoring-grid-view = Grid View

# -----------------------------------------------------------------------------
# Monitoring Panels
# -----------------------------------------------------------------------------
monitoring-sessions = Sessions
monitoring-messages = Messages
monitoring-resources = Resources
monitoring-services = Services
monitoring-active-bots = Active Bots
monitoring-loading = Loading...

# -----------------------------------------------------------------------------
# Service Status
# -----------------------------------------------------------------------------
monitoring-status-running = Running
monitoring-status-warning = Warning
monitoring-status-stopped = Stopped
monitoring-status-healthy = Healthy
monitoring-status-degraded = Degraded
monitoring-status-down = Down

# -----------------------------------------------------------------------------
# Resource Metrics
# -----------------------------------------------------------------------------
monitoring-cpu = CPU
monitoring-memory = Memory
monitoring-disk = Disk
monitoring-network = Network
monitoring-requests-per-sec = Requests/sec
monitoring-active-connections = Active Connections
monitoring-uptime = Uptime

# -----------------------------------------------------------------------------
# Logs
# -----------------------------------------------------------------------------
monitoring-logs-title = System Logs
monitoring-logs-filter = Filter Logs
monitoring-logs-level = Log Level
monitoring-logs-all = All Levels
monitoring-logs-debug = Debug
monitoring-logs-info = Info
monitoring-logs-warning = Warning
monitoring-logs-error = Error
monitoring-logs-critical = Critical
monitoring-logs-search = Search logs...
monitoring-logs-no-results = No logs found

# -----------------------------------------------------------------------------
# Health
# -----------------------------------------------------------------------------
monitoring-health-title = System Health
monitoring-health-status = Health Status
monitoring-health-services = Service Health
monitoring-health-database = Database
monitoring-health-cache = Cache
monitoring-health-queue = Message Queue
monitoring-health-storage = Storage
monitoring-health-external = External Services

# -----------------------------------------------------------------------------
# Metrics
# -----------------------------------------------------------------------------
monitoring-metrics-title = Performance Metrics
monitoring-metrics-response-time = Response Time
monitoring-metrics-throughput = Throughput
monitoring-metrics-error-rate = Error Rate
monitoring-metrics-latency = Latency

# -----------------------------------------------------------------------------
# Alerts
# -----------------------------------------------------------------------------
monitoring-alerts-title = System Alerts
monitoring-alerts-active = Active Alerts
monitoring-alerts-resolved = Resolved
monitoring-alerts-all = All Alerts
monitoring-alert-severity = Severity
monitoring-alert-critical = Critical
monitoring-alert-high = High
monitoring-alert-medium = Medium
monitoring-alert-low = Low
monitoring-alert-info = Info
monitoring-alert-acknowledge = Acknowledge
monitoring-alert-resolve = Resolve
monitoring-no-alerts = No active alerts

# -----------------------------------------------------------------------------
# Security Module
# -----------------------------------------------------------------------------
security-tab-compliance = API Compliance Report
security-tab-protection = Protection
security-export-report = Export Report
security-run-scan = Run Compliance Scan
security-critical = Critical
security-critical-desc = Immediate action required
security-high = High
security-high-desc = Security risk
security-medium = Medium
security-medium-desc = Should be addressed
security-low = Low
security-low-desc = Best practice
security-info = Info
security-info-desc = Informational
security-filter-severity = Severity:
security-filter-all-severities = All Severities
security-filter-type = Type:
security-filter-all-types = All Types
security-type-password = Password in Config
security-type-hardcoded = Hardcoded Secrets
security-type-deprecated = Deprecated Keywords
security-type-fragile = Fragile Code
security-type-config = Configuration Issues
security-results = Compliance Issues
security-col-severity = Severity
security-col-issue = Issue Type
security-col-location = Location
security-col-details = Description
security-col-action = Action

# -----------------------------------------------------------------------------
# Learn Module
# -----------------------------------------------------------------------------
learn-title = Learn
learn-my-progress = My Progress
learn-completed = Completed
learn-in-progress = In Progress
learn-certificates = Certificates
learn-time-spent = Time Spent
learn-categories = Categories
learn-all-courses = All Courses
learn-mandatory = Mandatory
learn-compliance = Compliance
learn-security = Security
learn-skills = Skills
learn-onboarding = Onboarding
learn-difficulty = Difficulty
learn-my-certificates = My Certificates
learn-view-all = View All

# -----------------------------------------------------------------------------
# Workspace Module
# -----------------------------------------------------------------------------
workspace-title = Workspace
workspace-search-pages = Search pages...
workspace-recent = Recent
workspace-favorites = Favorites
workspace-pages = Pages
workspace-templates = Templates
workspace-trash = Trash
workspace-settings = Settings

# -----------------------------------------------------------------------------
# Player Module
# -----------------------------------------------------------------------------
player-title = Media Player
player-no-file = No file selected
player-search = Search files...
player-recent = Recent
player-files = Files

# -----------------------------------------------------------------------------
# Goals Module
# -----------------------------------------------------------------------------
goals-title = Goals & OKRs
goals-dashboard = Dashboard
goals-objectives = Objectives
goals-alignment = Alignment
goals-ai-suggestions = AI Suggestions

# semantic i18n key (#2542)
admin-ops-dashboard-title = Operations Dashboard

# semantic i18n key (#2542)
admin-ops-dashboard-subtitle = Real-time system metrics, distributed tracing, and performance monitoring

# semantic i18n key (#2542)
admin-ops-auto-refresh-on = Auto-refresh: ON

# semantic i18n key (#2542)
admin-ops-avg-response = Avg Response

# semantic i18n key (#2542)
admin-ops-requests-min = Requests/min

# semantic i18n key (#2542)
admin-ops-disk-io = Disk I/O

# semantic i18n key (#2542)
admin-ops-iops-read-write = IOPS (read/write)

# semantic i18n key (#2542)
admin-ops-in-95-out-50 = In: 95 / Out: 50

# semantic i18n key (#2542)
admin-ops-stable = Stable

# semantic i18n key (#2542)
admin-ops-request-metrics = Request Metrics

# semantic i18n key (#2542)
admin-ops-errors-label = Errors

# semantic i18n key (#2542)
admin-ops-distributed-tracing = Distributed Tracing

# semantic i18n key (#2542)
admin-ops-all-traces = All Traces

# semantic i18n key (#2542)
admin-ops-errors-only = Errors Only

# semantic i18n key (#2542)
admin-ops-api-requests = API Requests

# semantic i18n key (#2542)
admin-ops-post-api-chat-message = POST /api/chat/message

# semantic i18n key (#2542)
admin-ops-trace-a1b2c3d4e5f6 = trace_a1b2c3d4e5f6

# semantic i18n key (#2542)
admin-ops-get-api-kb-search = GET /api/kb/search

# semantic i18n key (#2542)
admin-ops-trace-b2c3d4e5f6g7 = trace_b2c3d4e5f6g7

# semantic i18n key (#2542)
admin-ops-post-api-billing-checkout = POST /api/billing/checkout

# semantic i18n key (#2542)
admin-ops-trace-c3d4e5f6g7h8 = trace_c3d4e5f6g7h8

# semantic i18n key (#2542)
admin-ops-get-api-user-profile = GET /api/user/profile

# semantic i18n key (#2542)
admin-ops-trace-d4e5f6g7h8i9 = trace_d4e5f6g7h8i9

# semantic i18n key (#2542)
admin-ops-api-gateway = API Gateway

# semantic i18n key (#2542)
admin-ops-primary-2-replicas = Primary + 2 Replicas

# semantic i18n key (#2542)
admin-ops-qdrant-vector-db = Qdrant (Vector DB)

# semantic i18n key (#2542)
admin-ops-redis-cache = Redis Cache

# semantic i18n key (#2542)
admin-ops-cluster-mode-3-shards = Cluster mode | 3 shards

# semantic i18n key (#2542)
admin-ops-llm-service = LLM Service

# semantic i18n key (#2542)
admin-ops-openai-gpt-4-high-latency = OpenAI GPT-4 | High latency

# semantic i18n key (#2542)
admin-ops-object-storage = Object Storage

# semantic i18n key (#2542)
admin-ops-s3-compatible = S3 Compatible

# semantic i18n key (#2542)
admin-ops-error-tracking = Error Tracking

# semantic i18n key (#2542)
admin-ops-view-all-errors = View All Errors

# semantic i18n key (#2542)
admin-ops-warnings = Warnings

# semantic i18n key (#2542)
admin-ops-payment-processing-failed-stripe-timeout = Payment processing failed: Stripe API timeout

# semantic i18n key (#2542)
admin-ops-billing-checkout-rs-145 = billing/checkout.rs:145

# semantic i18n key (#2542)
admin-ops-rate-limit-approaching-openai = Rate limit approaching for OpenAI API

# semantic i18n key (#2542)
admin-ops-llm-openai-rs-89 = llm/openai.rs:89

# semantic i18n key (#2542)
admin-ops-core-analytics-rs-234 = core/analytics.rs:234

# semantic i18n key (#2542)
admin-ops-cache-miss-rate-above-threshold = Cache miss rate above threshold

# semantic i18n key (#2542)
admin-ops-cache-redis-rs-67 = cache/redis.rs:67

# semantic i18n key (#2542)
admin-ops-endpoint-performance = Endpoint Performance

# semantic i18n key (#2542)
admin-ops-by-requests = By Requests

# semantic i18n key (#2542)
admin-ops-by-latency = By Latency

# semantic i18n key (#2542)
admin-ops-by-error-rate = By Error Rate

# semantic i18n key (#2542)
admin-ops-post = POST

# semantic i18n key (#2542)
admin-ops-api-chat-message = /api/chat/message

# semantic i18n key (#2542)
admin-ops-req-min = req/min

# semantic i18n key (#2542)
admin-ops-errors-metric = errors

# semantic i18n key (#2542)
admin-ops-api-kb-search = /api/kb/search

# semantic i18n key (#2542)
admin-roles-title = Role Management

# semantic i18n key (#2542)
admin-roles-subtitle = Configure roles and assign permissions to control access across the suite

# semantic i18n key (#2542)
admin-roles-sidebar-title = Roles

# semantic i18n key (#2542)
admin-roles-system-filter = System Roles

# semantic i18n key (#2542)
admin-roles-custom-filter = Custom Roles

# semantic i18n key (#2542)
admin-roles-select-title = Select a Role

# semantic i18n key (#2542)
admin-roles-select-hint = Choose a role from the list to view and manage its permissions

# semantic i18n key (#2542)
admin-roles-name-label = Role Name

# semantic i18n key (#2542)
admin-roles-name-badge-placeholder = role_name

# semantic i18n key (#2542)
admin-roles-description-placeholder = Role description goes here

# semantic i18n key (#2542)
admin-roles-permissions-label = Permissions

# semantic i18n key (#2542)
admin-roles-assigned-users-tab = Assigned Users

# semantic i18n key (#2542)
admin-roles-assigned-groups-tab = Assigned Groups

# semantic i18n key (#2542)
admin-roles-permission-assignment = Permission Assignment

# semantic i18n key (#2542)
admin-roles-permission-assignment-help = Use the dual-list to assign permissions to this role

# semantic i18n key (#2542)
admin-roles-available-permissions = Available Permissions

# semantic i18n key (#2542)
admin-roles-category-documents = Documents

# semantic i18n key (#2542)
admin-roles-category-spreadsheets = Spreadsheets

# semantic i18n key (#2542)
admin-roles-category-presentations = Presentations

# semantic i18n key (#2542)
admin-roles-category-meetings = Meetings

# semantic i18n key (#2542)
admin-roles-category-ai-bots = AI & Bots

# semantic i18n key (#2542)
admin-roles-category-automation = Automation

# semantic i18n key (#2542)
admin-roles-assigned-permissions = Assigned Permissions

# semantic i18n key (#2542)
admin-roles-no-permissions-assigned = No permissions assigned

# semantic i18n key (#2542)
admin-roles-reset-to-default = Reset to Default

# semantic i18n key (#2542)
admin-roles-users-with-role = Users with this Role

# semantic i18n key (#2542)
admin-roles-groups-with-role = Groups with this Role

# semantic i18n key (#2542)
admin-roles-change-history = Role Change History

# semantic i18n key (#2542)
admin-roles-create-new-role = Create New Role

# semantic i18n key (#2542)
admin-roles-name-help = Lowercase, no spaces (use underscores)

# semantic i18n key (#2542)
admin-roles-copy-permissions-from = Copy Permissions From

# semantic i18n key (#2542)
admin-roles-start-with-no-permissions = Start with no permissions

# semantic i18n key (#2542)
admin-roles-standard-user = Standard User

# semantic i18n key (#2542)
admin-roles-power-user = Power User

# semantic i18n key (#2542)
admin-roles-viewer = Viewer

# semantic i18n key (#2542)
admin-roles-guest-user = Guest User

# semantic i18n key (#2542)
admin-roles-create-role-btn = Create Role

# semantic i18n key (#2542)
admin-roles-delete-role-title = Delete Role

# semantic i18n key (#2542)
admin-roles-delete-warning = This will remove the role from all users and groups. This action cannot be undone.

# semantic i18n key (#2542)
admin-roles-assign-to-users = Assign Role to Users

# semantic i18n key (#2542)
admin-roles-search-users-label = Search Users

# semantic i18n key (#2542)
admin-roles-type-to-search-users = Type to search for users

# semantic i18n key (#2542)
admin-roles-expiration-optional = Expiration (Optional)

# semantic i18n key (#2542)
admin-roles-expiration-help = Leave empty for permanent assignment

# semantic i18n key (#2542)
admin-roles-assign-to-groups = Assign Role to Groups

# semantic i18n key (#2542)
admin-roles-search-groups-label = Search Groups

# semantic i18n key (#2542)
admin-roles-type-to-search-groups = Type to search for groups

# semantic i18n key (#2542)
admin-roles-assign-role-btn = Assign Role

# semantic i18n key (#2542)
admin-onboarding-step-welcome = Welcome

# semantic i18n key (#2542)
admin-onboarding-step-welcome-desc = Tell us about yourself

# semantic i18n key (#2542)
admin-onboarding-step-org-desc = Create your workspace

# semantic i18n key (#2542)
admin-onboarding-step-plan = Choose Plan

# semantic i18n key (#2542)
admin-onboarding-step-plan-desc = Select your subscription

# semantic i18n key (#2542)
admin-onboarding-step-payment-desc = Add payment method

# semantic i18n key (#2542)
admin-onboarding-step-start = Get Started

# semantic i18n key (#2542)
admin-onboarding-step-start-desc = Create your first bot

# semantic i18n key (#2542)
admin-onboarding-view-guide = View guide

# semantic i18n key (#2542)
admin-onboarding-welcome-title = Welcome to General Bots!

# semantic i18n key (#2542)
admin-onboarding-welcome-subtitle = Let's get you set up in just a few minutes. First, tell us a bit about yourself.

# semantic i18n key (#2542)
admin-onboarding-full-name = Full Name

# semantic i18n key (#2542)
admin-onboarding-your-role = Your Role

# semantic i18n key (#2542)
admin-onboarding-role-placeholder = Select your role...

# semantic i18n key (#2542)
admin-onboarding-role-developer = Developer / Engineer

# semantic i18n key (#2542)
admin-onboarding-role-product-manager = Product Manager

# semantic i18n key (#2542)
admin-onboarding-role-customer-support = Customer Support

# semantic i18n key (#2542)
admin-onboarding-role-executive = Executive / Founder

# semantic i18n key (#2542)
admin-onboarding-use-case-label = What do you want to build?

# semantic i18n key (#2542)
admin-onboarding-use-case-placeholder = Select primary use case...

# semantic i18n key (#2542)
admin-onboarding-usecase-support-bot = Customer Support Bot

# semantic i18n key (#2542)
admin-onboarding-usecase-sales-assistant = Sales Assistant

# semantic i18n key (#2542)
admin-onboarding-usecase-internal-kb = Internal Knowledge Base

# semantic i18n key (#2542)
admin-onboarding-usecase-lead-generation = Lead Generation

# semantic i18n key (#2542)
admin-onboarding-usecase-employee-onboarding = Employee Onboarding

# semantic i18n key (#2542)
admin-onboarding-usecase-ecommerce = E-commerce Assistant

# semantic i18n key (#2542)
admin-onboarding-usecase-other = Something else

# semantic i18n key (#2542)
admin-onboarding-hear-about-label = How did you hear about us?

# semantic i18n key (#2542)
admin-onboarding-source-search = Search Engine

# semantic i18n key (#2542)
admin-onboarding-source-social = Social Media

# semantic i18n key (#2542)
admin-onboarding-source-friend = Friend / Colleague

# semantic i18n key (#2542)
admin-onboarding-create-org-title = Create Your Organization

# semantic i18n key (#2542)
admin-onboarding-create-org-subtitle = Your organization is where you'll manage bots, team members, and knowledge bases.

# semantic i18n key (#2542)
admin-onboarding-org-name = Organization Name

# semantic i18n key (#2542)
admin-onboarding-org-name-hint = This will be visible to your team members

# semantic i18n key (#2542)
admin-onboarding-org-url = Organization URL

# semantic i18n key (#2542)
admin-onboarding-org-url-prefix = app.generalbots.com/

# semantic i18n key (#2542)
admin-onboarding-org-url-hint = Only lowercase letters, numbers, and hyphens

# semantic i18n key (#2542)
admin-onboarding-team-size = Team Size

# semantic i18n key (#2542)
admin-onboarding-team-size-placeholder = How many people will use this?

# semantic i18n key (#2542)
admin-onboarding-team-just-me = Just me

# semantic i18n key (#2542)
admin-onboarding-industry-placeholder = Select your industry (optional)

# semantic i18n key (#2542)
admin-onboarding-industry-technology = Technology

# semantic i18n key (#2542)
admin-onboarding-industry-healthcare = Healthcare

# semantic i18n key (#2542)
admin-onboarding-industry-finance = Finance / Banking

# semantic i18n key (#2542)
admin-onboarding-industry-retail = Retail / E-commerce

# semantic i18n key (#2542)
admin-onboarding-industry-education = Education

# semantic i18n key (#2542)
admin-onboarding-industry-manufacturing = Manufacturing

# semantic i18n key (#2542)
admin-onboarding-industry-consulting = Consulting / Services

# semantic i18n key (#2542)
admin-onboarding-industry-government = Government

# semantic i18n key (#2542)
admin-onboarding-industry-nonprofit = Non-profit

# semantic i18n key (#2542)
admin-onboarding-choose-plan-title = Choose Your Plan

# semantic i18n key (#2542)
admin-onboarding-choose-plan-subtitle = Select the plan that best fits your needs. You can upgrade or downgrade anytime.

# semantic i18n key (#2542)
admin-onboarding-plan-free = Free

# semantic i18n key (#2542)
admin-onboarding-plan-free-desc = For individuals and small projects

# semantic i18n key (#2542)
admin-onboarding-per-month = /month

# semantic i18n key (#2542)
admin-onboarding-select-free = Select Free

# semantic i18n key (#2542)
admin-onboarding-most-popular = Most Popular

# semantic i18n key (#2542)
admin-onboarding-plan-pro-desc = For growing teams

# semantic i18n key (#2542)
admin-onboarding-select-pro = Select Pro

# semantic i18n key (#2542)
admin-dashboard-organization-title = Organization Dashboard

# semantic i18n key (#2542)
admin-dashboard-organization-subtitle = Overview of your organization's activity, members, and system health

# semantic i18n key (#2542)
admin-dashboard-team-members = Team Members

# semantic i18n key (#2542)
admin-dashboard-system-uptime = System Uptime

# semantic i18n key (#2542)
admin-dashboard-api-server = API Server

# semantic i18n key (#2542)
admin-dashboard-operational = Operational

# semantic i18n key (#2542)
admin-dashboard-latency-45ms = Latency: 45ms

# semantic i18n key (#2542)
admin-dashboard-requests-1-2k-min = Requests: 1.2K/min

# semantic i18n key (#2542)
admin-dashboard-connections-45-100 = Connections: 45/100

# semantic i18n key (#2542)
admin-dashboard-query-time-12ms = Query Time: 12ms

# semantic i18n key (#2542)
admin-dashboard-vector-database = Vector Database

# semantic i18n key (#2542)
admin-dashboard-vectors-2-4m = Vectors: 2.4M

# semantic i18n key (#2542)
admin-dashboard-search-8ms = Search: 8ms

# semantic i18n key (#2542)
admin-dashboard-tokens-125k-min = Tokens: 125K/min

# semantic i18n key (#2542)
admin-dashboard-latency-850ms = Latency: 850ms

# semantic i18n key (#2542)
admin-dashboard-high-usage = High Usage

# semantic i18n key (#2542)
admin-dashboard-used-34-5gb = Used: 34.5GB

# semantic i18n key (#2542)
admin-dashboard-iops-2-1k = IOPS: 2.1K

# semantic i18n key (#2542)
admin-dashboard-members-filter = Members

# semantic i18n key (#2542)
admin-dashboard-customer-support-bot = Customer Support Bot

# semantic i18n key (#2542)
admin-dashboard-security-alert = Security Alert:

# semantic i18n key (#2542)
admin-dashboard-customer-feedback-survey = Customer Feedback Survey

# semantic i18n key (#2542)
admin-dashboard-sales-assistant = Sales Assistant

# semantic i18n key (#2542)
admin-dashboard-load-more-activity = Load More Activity

# semantic i18n key (#2542)
admin-dashboard-john-smith = John Smith

# semantic i18n key (#2542)
admin-dashboard-role-distribution = Role Distribution

# semantic i18n key (#2542)
admin-dashboard-manage-roles = Manage Roles

# semantic i18n key (#2542)
admin-dashboard-bot-status = Bot Status

# semantic i18n key (#2542)
admin-dashboard-manage-bots = Manage Bots

# semantic i18n key (#2542)
admin-dashboard-handles-customer-inquiries = Handles customer inquiries

# semantic i18n key (#2542)
admin-dashboard-product-recommendations = Product recommendations

# semantic i18n key (#2542)
admin-dashboard-hr-assistant = HR Assistant

# semantic i18n key (#2542)
admin-dashboard-employee-questions = Employee questions

# semantic i18n key (#2542)
admin-dashboard-it-helpdesk = IT Helpdesk

# semantic i18n key (#2542)
admin-dashboard-technical-support = Technical support

# semantic i18n key (#2542)
admin-dashboard-pending-invitations = Pending Invitations

# semantic i18n key (#2542)
admin-dashboard-loading-invitations = Loading invitations…

# semantic i18n key (#2542)
admin-dashboard-invite-member = Invite Member

# semantic i18n key (#2542)
admin-dashboard-bulk-invite = Bulk Invite

# semantic i18n key (#2542)
admin-dashboard-emails-label = Emails (one per line, comma or semicolon separated)

# semantic i18n key (#2542)
admin-dashboard-manage-users = Manage Users

# semantic i18n key (#2542)
admin-dashboard-roles-permissions = Roles & Permissions

# semantic i18n key (#2542)
workspace-placeholder-untitled = Untitled

# semantic i18n key (#2542)
admin-dashboard-placeholder-invite-email = teammate@example.com
admin-dashboard-placeholder-bulk-emails = a@example.com
b@example.com
