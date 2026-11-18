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
