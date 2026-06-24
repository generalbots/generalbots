# =============================================================================
# General Bots - English UI Translations
# =============================================================================

# -----------------------------------------------------------------------------
# Navigation
# -----------------------------------------------------------------------------
nav-home = Home
nav-chat = Chat
nav-drive = Drive
nav-tasks = Tasks
nav-mail = Mail
nav-calendar = Calendar
nav-meet = Meet
nav-paper = Paper
nav-video = Video
nav-research = Research
nav-analytics = Analytics
nav-settings = Settings
nav-admin = Admin
nav-monitoring = Monitoring
nav-sources = Sources
nav-tools = Tools
nav-attendant = Attendant
nav-learn = Learn
nav-crm = CRM
nav-billing = Billing
nav-products = Products
nav-tickets = Tickets
nav-docs = Docs
nav-sheet = Sheets
nav-slides = Slides
nav-social = Social
nav-all-apps = All Apps
nav-people = People
nav-editor = Editor
nav-dashboards = Dashboards
nav-security = Security
nav-designer = Designer
nav-project = Project
nav-canvas = Canvas
nav-goals = Goals
nav-player = Player
nav-workspace = Workspace

# -----------------------------------------------------------------------------
# Dashboard
# -----------------------------------------------------------------------------
dashboard-title = Dashboard
dashboard-welcome = Welcome back, { $name }!
dashboard-quick-actions = Quick Actions
dashboard-recent-activity = Recent Activity
dashboard-no-activity = No recent activity yet. Start exploring!
dashboard-analytics = Analytics

# -----------------------------------------------------------------------------
# Quick Actions
# -----------------------------------------------------------------------------
quick-start-chat = Start Chat
quick-upload-files = Upload Files
quick-new-task = New Task
quick-compose-email = Compose Email
quick-start-meeting = Start Meeting
quick-new-event = New Event

# -----------------------------------------------------------------------------
# Application Cards
# -----------------------------------------------------------------------------
app-chat-name = Chat
app-chat-desc = AI-powered conversations. Ask questions, get help, and automate tasks.

app-drive-name = Drive
app-drive-desc = Cloud storage for all your files. Upload, organize, and share.

app-tasks-name = Tasks
app-tasks-desc = Stay organized with to-do lists, priorities, and due dates.

app-mail-name = Mail
app-mail-desc = Email client with AI-assisted writing and smart organization.

app-calendar-name = Calendar
app-calendar-desc = Schedule meetings, events, and manage your time effectively.

app-meet-name = Meet
app-meet-desc = Video conferencing with screen sharing and live transcription.

app-paper-name = Paper
app-paper-desc = Write documents with AI assistance. Notes, reports, and more.

app-research-name = Research
app-research-desc = AI-powered search and discovery across all your sources.

app-analytics-name = Analytics
app-analytics-desc = Dashboards and reports to track usage and insights.

# -----------------------------------------------------------------------------
# Suite Header
# -----------------------------------------------------------------------------
suite-title = General Bots Suite
suite-tagline = Your AI-powered productivity workspace. Chat, collaborate, and create.
suite-new-intent = New Intent

# -----------------------------------------------------------------------------
# AI Panel
# -----------------------------------------------------------------------------
ai-developer = AI Developer
ai-developing = Developing: { $project }
ai-quick-actions = Quick Actions
ai-add-field = Add field
ai-change-color = Change color
ai-add-validation = Add validation
ai-export-data = Export data
ai-placeholder = Type your modifications...
ai-thinking = AI is thinking...
ai-status-online = Online
ai-status-offline = Offline

# -----------------------------------------------------------------------------
# Chat
# -----------------------------------------------------------------------------
chat-title = Chat
chat-placeholder = Type your message...
chat-send = Send
chat-new-conversation = New Conversation
chat-history = Chat History
chat-clear = Clear Chat
chat-export = Export Chat
chat-typing = { $name } is typing...
chat-online = Online
chat-offline = Offline
chat-last-seen = Last seen { $time }
chat-mention-title = Reference Entity
chat-mention-placeholder = Message... (type @ to mention)
chat-mention-search = Search entities...
chat-mention-no-results = No results found
chat-mention-type-hint = Type : to search

# -----------------------------------------------------------------------------
# Drive / Files
# -----------------------------------------------------------------------------
drive-title = Drive
drive-upload = Upload
drive-new-folder = New Folder
drive-empty = No files yet. Upload something!
drive-search = Search files...
drive-sort-name = Name
drive-sort-date = Date
drive-sort-size = Size
drive-sort-type = Type
drive-view-grid = Grid View
drive-view-list = List View
drive-selected = { $count ->
    [one] { $count } item selected
   *[other] { $count } items selected
}
drive-file-size = { $size ->
    [bytes] { $value } B
    [kb] { $value } KB
    [mb] { $value } MB
    [gb] { $value } GB
   *[other] { $value } bytes
}
drive-drop-files = Drop files here to upload

# -----------------------------------------------------------------------------
# Tasks
# -----------------------------------------------------------------------------
tasks-title = Tasks
tasks-new = New Task
tasks-due-today = Due Today
tasks-overdue = Overdue
tasks-completed = Completed
tasks-all = All Tasks
tasks-priority-high = High Priority
tasks-priority-medium = Medium Priority
tasks-priority-low = Low Priority
tasks-no-due-date = No due date
tasks-add-subtask = Add subtask
tasks-mark-complete = Mark as complete
tasks-mark-incomplete = Mark as incomplete
tasks-delete-confirm = Are you sure you want to delete this task?
tasks-count = { $count ->
    [zero] No tasks
    [one] { $count } task
   *[other] { $count } tasks
}

# -----------------------------------------------------------------------------
# Calendar
# -----------------------------------------------------------------------------
calendar-title = Calendar
calendar-today = Today
calendar-new-event = New Event
calendar-all-day = All day
calendar-repeat = Repeat
calendar-reminder = Reminder
calendar-view-day = Day
calendar-view-week = Week
calendar-view-month = Month
calendar-view-year = Year
calendar-no-events = No events scheduled
calendar-event-title = Event title
calendar-event-location = Location
calendar-event-description = Description
calendar-event-attendees = Attendees

# -----------------------------------------------------------------------------
# Meet / Video Conferencing
# -----------------------------------------------------------------------------
meet-title = Meet
meet-join = Join Meeting
meet-start = Start Meeting
meet-mute = Mute
meet-unmute = Unmute
meet-video-on = Camera On
meet-video-off = Camera Off
meet-share-screen = Share Screen
meet-stop-sharing = Stop Sharing
meet-end-call = End Call
meet-leave = Leave Meeting
meet-participants = { $count ->
    [one] { $count } participant
   *[other] { $count } participants
}
meet-waiting-room = Waiting Room
meet-admit = Admit
meet-remove = Remove
meet-chat = Meeting Chat
meet-raise-hand = Raise Hand
meet-lower-hand = Lower Hand
meet-recording = Recording
meet-start-recording = Start Recording
meet-stop-recording = Stop Recording

# -----------------------------------------------------------------------------
# Mail / Email
# -----------------------------------------------------------------------------
mail-title = Mail
mail-compose = Compose
mail-inbox = Inbox
mail-sent = Sent
mail-drafts = Drafts
mail-trash = Trash
mail-spam = Spam
mail-starred = Starred
mail-archive = Archive
mail-to = To
mail-cc = CC
mail-bcc = BCC
mail-subject = Subject
mail-body = Message
mail-reply = Reply
mail-reply-all = Reply All
mail-forward = Forward
mail-send = Send
mail-discard = Discard
mail-save-draft = Save Draft
mail-attach = Attach Files
mail-unread = { $count ->
    [one] { $count } unread
   *[other] { $count } unread
}
mail-empty-inbox = Your inbox is empty
mail-no-subject = (No subject)

# -----------------------------------------------------------------------------
# Settings
# -----------------------------------------------------------------------------
settings-title = Settings
settings-general = General
settings-account = Account
settings-notifications = Notifications
settings-privacy = Privacy
settings-security = Security
settings-language = Language
settings-theme = Theme
settings-theme-light = Light
settings-theme-dark = Dark
settings-theme-system = System
settings-save = Save Changes
settings-saved = Settings saved successfully
settings-timezone = Timezone
settings-date-format = Date Format
settings-time-format = Time Format

# -----------------------------------------------------------------------------
# Auth / Login
# -----------------------------------------------------------------------------
auth-login = Log In
auth-logout = Log Out
auth-signup = Sign Up
auth-forgot-password = Forgot Password?
auth-reset-password = Reset Password
auth-email = Email
auth-password = Password
auth-confirm-password = Confirm Password
auth-remember-me = Remember me
auth-login-success = Logged in successfully
auth-logout-success = Logged out successfully
auth-invalid-credentials = Invalid email or password
auth-session-expired = Your session has expired. Please log in again.

# -----------------------------------------------------------------------------
# Search
# -----------------------------------------------------------------------------
search-placeholder = Search...
search-no-results = No results found
search-results = { $count ->
    [one] { $count } result
   *[other] { $count } results
}
search-in-progress = Searching...
search-advanced = Advanced Search
search-filters = Filters
search-clear-filters = Clear Filters

# -----------------------------------------------------------------------------
# Pagination
# -----------------------------------------------------------------------------
pagination-previous = Previous
pagination-next = Next
pagination-first = First
pagination-last = Last
pagination-page = Page { $current } of { $total }
pagination-showing = Showing { $from } to { $to } of { $total }

# -----------------------------------------------------------------------------
# Tables
# -----------------------------------------------------------------------------
table-no-data = No data available
table-loading = Loading data...
table-actions = Actions
table-select-all = Select All
table-deselect-all = Deselect All
table-export = Export
table-import = Import

# -----------------------------------------------------------------------------
# Forms
# -----------------------------------------------------------------------------
form-required = Required
form-optional = Optional
form-submit = Submit
form-reset = Reset
form-clear = Clear
form-uploading = Uploading...
form-processing = Processing...

# -----------------------------------------------------------------------------
# Modals / Dialogs
# -----------------------------------------------------------------------------
modal-confirm-title = Confirm Action
modal-confirm-message = Are you sure you want to proceed?
modal-delete-title = Delete Confirmation
modal-delete-message = This action cannot be undone. Are you sure?

# -----------------------------------------------------------------------------
# Tooltips
# -----------------------------------------------------------------------------
tooltip-copy = Copy to clipboard
tooltip-copied = Copied!
tooltip-expand = Expand
tooltip-collapse = Collapse
tooltip-refresh = Refresh
tooltip-download = Download
tooltip-upload = Upload
tooltip-print = Print
tooltip-fullscreen = Fullscreen
tooltip-exit-fullscreen = Exit Fullscreen

# -----------------------------------------------------------------------------
# Settings - Language & Localization
# -----------------------------------------------------------------------------
settings-language = Language
settings-language-desc = Choose your preferred language
settings-display-language = Display Language
settings-language-affects = Affects all text in the application
settings-date-format = Date Format
settings-date-format-desc = How dates are displayed
settings-time-format = Time Format
settings-time-format-desc = 12-hour or 24-hour clock
settings-saved = Settings saved successfully
settings-language-changed = Language changed successfully
settings-reload-required = Page reload required to apply changes

# Settings - Profile
settings-profile = Profile Settings
settings-profile-desc = Manage your personal information and preferences
settings-profile-photo = Profile Photo
settings-profile-photo-desc = Your profile photo is visible to other users
settings-upload-photo = Upload Photo
settings-remove-photo = Remove
settings-basic-info = Basic Information
settings-display-name = Display Name
settings-username = Username
settings-email-address = Email Address
settings-bio = Bio
settings-bio-placeholder = Tell us about yourself...
settings-contact-info = Contact Information
settings-phone-number = Phone Number
settings-location = Location
settings-website = Website

# Settings - Security
settings-security = Security Settings
settings-security-desc = Protect your account with enhanced security
settings-change-password = Change Password
settings-change-password-desc = Update your password regularly for better security
settings-current-password = Current Password
settings-new-password = New Password
settings-confirm-password = Confirm New Password
settings-update-password = Update Password
settings-2fa = Two-Factor Authentication
settings-2fa-desc = Add an extra layer of security to your account
settings-authenticator-app = Authenticator App
settings-authenticator-desc = Use an authenticator app for 2FA codes
settings-enable-2fa = Enable 2FA
settings-disable-2fa = Disable 2FA
settings-active-sessions = Active Sessions
settings-active-sessions-desc = Manage your active login sessions
settings-this-device = This device
settings-terminate-session = Terminate
settings-terminate-all = Terminate All Other Sessions

# Settings - Appearance
settings-appearance = Appearance
settings-appearance-desc = Customize how the application looks
settings-theme-selection = Theme
settings-theme-selection-desc = Choose your preferred color theme
settings-theme-dark = Dark
settings-theme-light = Light
settings-theme-blue = Blue
settings-theme-purple = Purple
settings-theme-green = Green
settings-theme-orange = Orange
settings-layout-preferences = Layout Preferences
settings-compact-mode = Compact Mode
settings-compact-mode-desc = Reduce spacing for more content
settings-show-sidebar = Show Sidebar
settings-show-sidebar-desc = Always show navigation sidebar
settings-animations = Animations
settings-animations-desc = Enable UI animations and transitions

# Settings - Notifications
settings-notifications-title = Notifications
settings-notifications-desc = Control how you receive notifications
settings-email-notifications = Email Notifications
settings-direct-messages = Direct Messages
settings-direct-messages-desc = Receive email for new direct messages
settings-mentions = Mentions
settings-mentions-desc = Receive email when someone mentions you
settings-weekly-digest = Weekly Digest
settings-weekly-digest-desc = Get a weekly summary of activity
settings-marketing = Marketing
settings-marketing-desc = Receive news and product updates
settings-push-notifications = Push Notifications
settings-enable-push = Enable Push Notifications
settings-enable-push-desc = Receive browser push notifications
settings-notification-sound = Sound
settings-notification-sound-desc = Play sound for notifications
settings-in-app-notifications = In-App Notifications

# Settings - Storage
settings-storage = Storage
settings-storage-desc = Manage your storage usage
settings-storage-usage = Storage Usage
settings-storage-used = { $used } of { $total } used
settings-storage-upgrade = Upgrade Storage

# Settings - Privacy
settings-privacy-title = Privacy
settings-privacy-desc = Control your privacy settings
settings-data-collection = Data Collection
settings-analytics = Analytics
settings-analytics-desc = Help us improve by sending anonymous usage data
settings-crash-reports = Crash Reports
settings-crash-reports-desc = Automatically send crash reports
settings-download-data = Download Your Data
settings-download-data-desc = Get a copy of all your data
settings-delete-account = Delete Account
settings-delete-account-desc = Permanently delete your account and all data
settings-delete-account-warning = This action cannot be undone

# Settings - Billing
settings-billing = Billing
settings-billing-desc = Manage your subscription and payment methods
settings-current-plan = Current Plan
settings-free-plan = Free Plan
settings-pro-plan = Pro Plan
settings-enterprise-plan = Enterprise Plan
settings-upgrade-plan = Upgrade Plan
settings-payment-methods = Payment Methods
settings-add-payment = Add Payment Method
settings-billing-history = Billing History

# -----------------------------------------------------------------------------
# Paper (Document Editor)
# -----------------------------------------------------------------------------
paper-title = Paper
paper-new-note = New Note
paper-search-notes = Search notes...
paper-quick-start = Quick Start
paper-template-blank = Blank
paper-template-meeting = Meeting
paper-template-todo = To-Do
paper-template-research = Research
paper-untitled = Untitled
paper-placeholder = Start writing, or type / for commands...
paper-commands = Commands
paper-heading1 = Heading 1
paper-heading1-desc = Large section heading
paper-heading2 = Heading 2
paper-heading2-desc = Medium section heading
paper-heading3 = Heading 3
paper-heading3-desc = Small section heading
paper-paragraph = Paragraph
paper-paragraph-desc = Plain text
paper-bullet-list = Bullet List
paper-bullet-list-desc = Unordered list
paper-numbered-list = Numbered List
paper-numbered-list-desc = Ordered list
paper-todo-list = To-Do List
paper-todo-list-desc = Checkable task list
paper-quote = Quote
paper-quote-desc = Blockquote for citations
paper-divider = Divider
paper-divider-desc = Horizontal line
paper-code-block = Code Block
paper-code-block-desc = Formatted code
paper-table = Table
paper-table-desc = Insert table
paper-image = Image
paper-image-desc = Insert image from URL
paper-callout = Callout
paper-callout-desc = Highlighted information box
paper-ai-write = AI Write
paper-ai-write-desc = Generate text with AI
paper-ai-summarize = AI Summarize
paper-ai-summarize-desc = Summarize selected text
paper-ai-expand = AI Expand
paper-ai-expand-desc = Expand on selected text
paper-ai-improve = AI Improve
paper-ai-improve-desc = Improve writing quality
paper-ai-translate = AI Translate
paper-ai-translate-desc = Translate to another language
paper-ai-assistant = AI Assistant
paper-ai-quick-actions = Quick Actions
paper-ai-rewrite = Rewrite
paper-ai-make-shorter = Make Shorter
paper-ai-make-longer = Make Longer
paper-ai-fix-grammar = Fix Grammar
paper-ai-tone = Tone
paper-ai-tone-professional = Professional
paper-ai-tone-casual = Casual
paper-ai-tone-friendly = Friendly
paper-ai-tone-formal = Formal
paper-ai-translate-to = Translate to
paper-ai-custom-prompt = Custom Prompt
paper-ai-custom-placeholder = Describe what you want...
paper-ai-generate = Generate
paper-ai-response = AI Response
paper-ai-apply = Apply
paper-ai-regenerate = Regenerate
paper-ai-copy = Copy
paper-word-count = { $count } words
paper-char-count = { $count } characters
paper-saved = Saved
paper-saving = Saving...
paper-last-edited = Last edited: { $time }
paper-last-edited-now = Last edited: Just now
paper-export = Export Document
paper-export-pdf = PDF
paper-export-docx = Word (.docx)
paper-export-markdown = Markdown
paper-export-html = HTML
paper-export-txt = Plain Text

# Additional Chat translations
chat-voice = Voice input
chat-message-placeholder = Message...

# Drive translations
drive-my-drive = My Drive
drive-shared = Shared with me
drive-recent = Recent
drive-starred = Starred
drive-trash = Trash
drive-loading-storage = Loading storage...
drive-storage-used = { $used } of { $total } used
drive-empty-folder = This folder is empty

# Tasks translations
tasks-active = Active Intents
tasks-awaiting = Awaiting Decision
tasks-paused = Paused
tasks-blocked = Blocked/Issues
tasks-time-saved = Active Time Saved:
tasks-input-placeholder = What would you like to do? e.g., 'create a CRM app' or 'remind me to call John tomorrow'

# Calendar additional translations
calendar-my-calendars = My Calendars

# Email additional translations
email-scheduled = Scheduled
email-tracking = Tracking

# Email folder translations
email-inbox = Inbox
email-starred = Starred
email-sent = Sent
email-drafts = Drafts
email-spam = Spam
email-trash = Trash
email-compose = Compose

# -----------------------------------------------------------------------------
# Research
# -----------------------------------------------------------------------------
research-title = Research
research-search-placeholder = Ask anything...
research-collections = Collections
research-new-collection = New Collection
research-recent = Recent
research-academic = Academic
research-code = Code
research-internal = Internal
research-search-all = Search everything
research-academic-papers = Academic papers
research-code-docs = Code & documentation
research-internal-kb = Internal knowledge base
research-sources = Sources
research-trending = Trending
research-pro-search = Pro Search
research-include-images = Include Images
research-try-asking = Try asking about
research-related = Related Questions
research-view-all-sources = View All Sources
research-export-citations = Export Citations
research-save-to-collection = Save to Collection

# -----------------------------------------------------------------------------
# Admin Panel (additional UI keys)
# -----------------------------------------------------------------------------
admin-panel-title = Admin Panel
admin-quick-actions = Quick Actions
admin-create-user = Create User
admin-create-group = Create Group
admin-register-dns = Register DNS
admin-recent-activity = Recent Activity
admin-system-health = System Health

# -----------------------------------------------------------------------------
# Meet (additional keys)
# -----------------------------------------------------------------------------
meet-new-meeting = New Meeting
meet-join-meeting = Join Meeting
meet-active-rooms = Active Rooms
meet-room-title = Meeting Room
meet-record = Record
meet-camera = Camera
meet-share = Share
meet-info = Info
meet-more = More
meet-share-meeting = Share Meeting
meet-meeting-title = Meeting Title
meet-meeting-code = Meeting Code
meet-meeting-link = Meeting Link
meet-send-invite = Send Invite

# -----------------------------------------------------------------------------
# Common Labels (additional)
# -----------------------------------------------------------------------------
label-username = Username
label-email = Email
label-display-name = Display Name
label-password = Password
label-role = Role
label-group-name = Group Name
label-hostname = Hostname
label-record-type = Record Type
label-target = Target
label-your-name = Your Name

# -----------------------------------------------------------------------------
# Actions (additional)
# -----------------------------------------------------------------------------
action-register = Register

# -----------------------------------------------------------------------------
# Analytics (additional UI keys)
# -----------------------------------------------------------------------------
analytics-dashboard-title = Analytics Dashboard
analytics-last-hour = Last Hour
analytics-last-6h = Last 6 Hours
analytics-last-24h = Last 24 Hours
analytics-last-7d = Last 7 Days
analytics-last-30d = Last 30 Days

# -----------------------------------------------------------------------------
# Notifications
# -----------------------------------------------------------------------------
notifications-title = Notifications
notifications-clear = Clear all
notifications-empty = No notifications

# -----------------------------------------------------------------------------
# All Applications
# -----------------------------------------------------------------------------
nav-all-apps = All Applications

# =============================================================================
# AUTH SCREENS - Complete translations for login, register, forgot/reset password
# =============================================================================

# -----------------------------------------------------------------------------
# Login Screen
# -----------------------------------------------------------------------------
auth-welcome-back = Welcome Back
auth-sign-in-to-account = Sign in to your General Bots account
auth-email-address = Email Address
auth-email-placeholder = you@example.com
auth-password-placeholder = ••••••••
auth-sign-in = Sign In
auth-or-continue-with = or continue with
auth-dont-have-account = Don't have an account?
auth-create-account = Create account
auth-google = Google
auth-microsoft = Microsoft
auth-github = GitHub
auth-apple = Apple

# -----------------------------------------------------------------------------
# Two-Factor Authentication
# -----------------------------------------------------------------------------
auth-2fa-title = Two-Factor Authentication
auth-2fa-subtitle = Enter the 6-digit code from your authenticator app
auth-2fa-verify = Verify Code
auth-2fa-didnt-receive = Didn't receive a code?
auth-2fa-resend = Resend code
auth-2fa-back-to-login = Back to login
auth-2fa-trust-device = Trust this device
auth-2fa-trust-desc = Don't ask for 2FA on this device for 30 days

# -----------------------------------------------------------------------------
# Register Screen
# -----------------------------------------------------------------------------
auth-create-your-account = Create Your Account
auth-join-general-bots = Join General Bots and start building
auth-first-name = First Name
auth-last-name = Last Name
auth-create-password = Create Password
auth-confirm-your-password = Confirm Password
auth-password-strength = Password Strength
auth-password-weak = Weak
auth-password-fair = Fair
auth-password-good = Good
auth-password-strong = Strong
auth-password-req-length = At least 8 characters
auth-password-req-uppercase = One uppercase letter
auth-password-req-lowercase = One lowercase letter
auth-password-req-number = One number
auth-password-req-special = One special character
auth-passwords-match = Passwords match
auth-passwords-dont-match = Passwords don't match
auth-agree-terms = I agree to the
auth-terms-of-service = Terms of Service
auth-and = and
auth-privacy-policy = Privacy Policy
auth-sign-up = Sign Up
auth-already-have-account = Already have an account?
auth-sign-in-link = Sign in
auth-registration-success = Account Created Successfully!
auth-check-email = Please check your email to verify your account
auth-email-sent-to = We've sent a verification link to
auth-resend-verification = Resend Verification Email
auth-go-to-login = Go to Login

# -----------------------------------------------------------------------------
# Forgot Password Screen
# -----------------------------------------------------------------------------
auth-forgot-password-title = Forgot Password?
auth-forgot-password-subtitle = No worries! Enter your email and we'll send you reset instructions.
auth-send-reset-link = Send Reset Link
auth-back-to-login = Back to login
auth-reset-email-sent = Reset Email Sent!
auth-reset-instructions = We've sent password reset instructions to
auth-check-inbox = Check your inbox
auth-check-spam = Check your spam folder if you don't see it
auth-link-expires = The link expires in 1 hour
auth-resend-email = Resend Email
auth-didnt-receive-email = Didn't receive the email?

# -----------------------------------------------------------------------------
# Reset Password Screen
# -----------------------------------------------------------------------------
auth-reset-password-title = Reset Password
auth-reset-password-subtitle = Create a new secure password for your account
auth-new-password = New Password
auth-confirm-new-password = Confirm New Password
auth-reset-password-btn = Reset Password
auth-password-reset-success = Password Reset Successfully!
auth-password-updated = Your password has been updated. You can now sign in with your new password.
auth-invalid-token = Invalid or Expired Link
auth-invalid-token-desc = This password reset link is invalid or has expired. Please request a new one.
auth-request-new-link = Request New Link

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

# =============================================================================
# SOURCES SCREENS
# =============================================================================

# -----------------------------------------------------------------------------
# Sources Main
# -----------------------------------------------------------------------------
sources-title = Sources
sources-subtitle = Repositories, Apps, Prompts, Templates & MCP Servers
sources-search = Search sources...

# -----------------------------------------------------------------------------
# Sources Tabs
# -----------------------------------------------------------------------------
sources-repositories = Repositories
sources-apps = Apps
sources-prompts = Prompts
sources-templates = Templates
sources-servers = MCP Servers
sources-models = AI Models
sources-news = News

# -----------------------------------------------------------------------------
# Repository Cards
# -----------------------------------------------------------------------------
sources-repo-connect = Connect
sources-repo-disconnect = Disconnect
sources-repo-browse = Browse
sources-repo-connected = Connected
sources-repo-disconnected = Disconnected
sources-repo-stars = Stars
sources-repo-forks = Forks
sources-repo-last-updated = Last updated

# -----------------------------------------------------------------------------
# Prompt Cards
# -----------------------------------------------------------------------------
sources-prompt-use = Use
sources-prompt-copy = Copy
sources-prompt-edit = Edit
sources-prompt-rating = Rating
sources-prompt-uses = Uses

# -----------------------------------------------------------------------------
# Server Cards
# -----------------------------------------------------------------------------
sources-server-active = Active
sources-server-inactive = Inactive
sources-server-connect = Connect
sources-server-configure = Configure

# -----------------------------------------------------------------------------
# Model Cards
# -----------------------------------------------------------------------------
sources-model-active = Active
sources-model-coming-soon = Coming Soon
sources-model-provider = Provider
sources-model-context = Context
sources-model-tokens = tokens

# -----------------------------------------------------------------------------
# App Cards
# -----------------------------------------------------------------------------
sources-app-open = Open
sources-app-edit = Edit
sources-app-installed = Installed
sources-app-install = Install

# -----------------------------------------------------------------------------
# Template Cards
# -----------------------------------------------------------------------------
sources-template-preview = Preview
sources-template-use = Use Template
sources-template-components = components

# -----------------------------------------------------------------------------
# Categories
# -----------------------------------------------------------------------------
sources-category-all = All
sources-category-development = Development
sources-category-productivity = Productivity
sources-category-communication = Communication
sources-category-analytics = Analytics
sources-category-security = Security
sources-category-other = Other

# -----------------------------------------------------------------------------
# Empty States
# -----------------------------------------------------------------------------
sources-empty-repos = No repositories connected
sources-empty-apps = No apps available
sources-empty-prompts = No prompts found
sources-empty-templates = No templates available
sources-empty-servers = No MCP servers configured
sources-empty-models = No models available
sources-empty-results = No results found
sources-empty-results-desc = Try adjusting your search or filters

# =============================================================================
# TOOLS / COMPLIANCE SCREENS
# =============================================================================

# -----------------------------------------------------------------------------
# Compliance Main
# -----------------------------------------------------------------------------
compliance-title = API Compliance Report
compliance-subtitle = Security scan for all bots - Check for passwords, fragile code, and misconfigurations
compliance-export-report = Export Report
compliance-run-scan = Run Compliance Scan
compliance-scanning = Scanning...

# -----------------------------------------------------------------------------
# Bot Selector
# -----------------------------------------------------------------------------
compliance-all-bots = All Bots
compliance-select-bots = Select Bots

# -----------------------------------------------------------------------------
# Stats Cards
# -----------------------------------------------------------------------------
compliance-critical = Critical
compliance-critical-desc = Requires immediate action
compliance-high = High
compliance-high-desc = Security risk
compliance-medium = Medium
compliance-medium-desc = Should be addressed
compliance-low = Low
compliance-low-desc = Best practice
compliance-info = Info
compliance-info-desc = Informational

# -----------------------------------------------------------------------------
# Filters
# -----------------------------------------------------------------------------
compliance-filter-severity = Severity
compliance-filter-type = Type
compliance-filter-all-severities = All Severities
compliance-filter-all-types = All Types
compliance-search-issues = Search issues...

# -----------------------------------------------------------------------------
# Issue Types
# -----------------------------------------------------------------------------
compliance-type-password = Password in Config
compliance-type-hardcoded = Hardcoded Secrets
compliance-type-deprecated = Deprecated Keywords
compliance-type-fragile = Fragile Code
compliance-type-config = Configuration Issues

# -----------------------------------------------------------------------------
# Results Table
# -----------------------------------------------------------------------------
compliance-results = Results
compliance-results-count = { $count ->
    [one] { $count } issue found
   *[other] { $count } issues found
}
compliance-col-severity = Severity
compliance-col-issue = Issue
compliance-col-location = Location
compliance-col-details = Details
compliance-col-action = Action
compliance-view-details = View Details
compliance-fix-issue = Fix Issue
compliance-ignore = Ignore
compliance-no-issues = No issues found
compliance-no-issues-desc = Great! Your bots are compliant.

# -----------------------------------------------------------------------------
# Scan Progress
# -----------------------------------------------------------------------------
compliance-scan-in-progress = Scan in progress...
compliance-scan-checking = Checking { $item }...
compliance-scan-complete = Scan complete
compliance-scan-failed = Scan failed

# =============================================================================
# ATTENDANT / CRM SCREENS
# =============================================================================

# -----------------------------------------------------------------------------
# CRM Disabled State
# -----------------------------------------------------------------------------
attendant-crm-disabled = CRM Features Not Enabled
attendant-crm-disabled-desc = The Attendant Console requires CRM features to be enabled for this bot. This allows human agents to receive and respond to conversations transferred from the bot.
attendant-crm-enable-instruction = To enable CRM features, add this line to your bot's
attendant-crm-config-file = config.csv
attendant-crm-create-attendant = Then create an
attendant-crm-attendant-file = attendant.csv
attendant-crm-configure-team = file to configure your team

# -----------------------------------------------------------------------------
# Queue Sidebar
# -----------------------------------------------------------------------------
attendant-title = Attendant Console
attendant-status-online = Online
attendant-status-busy = Busy
attendant-status-away = Away
attendant-status-offline = Offline
attendant-status-ready = Online - Ready for conversations
attendant-status-busy-msg = Busy - Handling conversations
attendant-status-away-msg = Away - Will be back soon
attendant-status-offline-msg = Offline - Not available

# -----------------------------------------------------------------------------
# Queue Stats
# -----------------------------------------------------------------------------
attendant-waiting = Waiting
attendant-active = Active
attendant-resolved = Resolved
attendant-mine = Mine

# -----------------------------------------------------------------------------
# Queue Filters
# -----------------------------------------------------------------------------
attendant-filter-all = All
attendant-filter-waiting = Waiting
attendant-filter-mine = Mine
attendant-filter-priority = Priority

# -----------------------------------------------------------------------------
# Conversation List
# -----------------------------------------------------------------------------
attendant-no-conversations = No conversations in queue
attendant-new-conversations-appear = New conversations will appear here
attendant-unread = Unread
attendant-typing = typing...
attendant-select-conversation = Select a conversation
attendant-select-conversation-desc = Choose a conversation from the queue to start responding

# -----------------------------------------------------------------------------
# Channel Tags
# -----------------------------------------------------------------------------
attendant-channel-whatsapp = WhatsApp
attendant-channel-teams = Teams
attendant-channel-instagram = Instagram
attendant-channel-web = Web
attendant-channel-telegram = Telegram
attendant-channel-email = Email

# -----------------------------------------------------------------------------
# Priority Tags
# -----------------------------------------------------------------------------
attendant-priority-urgent = Urgent
attendant-priority-high = High
attendant-priority-normal = Normal

# -----------------------------------------------------------------------------
# Chat Area
# -----------------------------------------------------------------------------
attendant-message-placeholder = Type your message...
attendant-send = Send
attendant-attach-file = Attach file
attendant-insert-emoji = Insert emoji
attendant-quick-responses = Quick Responses
attendant-transfer = Transfer
attendant-resolve = Resolve
attendant-more-actions = More Actions

# -----------------------------------------------------------------------------
# Quick Responses
# -----------------------------------------------------------------------------
attendant-quick-greeting = Hello! How can I help you today?
attendant-quick-thanks = Thank you for your patience.
attendant-quick-checking = Let me check that for you.
attendant-quick-moment = One moment please.

# -----------------------------------------------------------------------------
# Transfer Modal
# -----------------------------------------------------------------------------
attendant-transfer-title = Transfer Conversation
attendant-transfer-to = Transfer to
attendant-transfer-reason = Reason (optional)
attendant-transfer-reason-placeholder = Why are you transferring this conversation?
attendant-transfer-cancel = Cancel
attendant-transfer-confirm = Transfer

# -----------------------------------------------------------------------------
# AI Insights Sidebar
# -----------------------------------------------------------------------------
attendant-ai-insights = AI Insights
attendant-ai-summary = Conversation Summary
attendant-ai-sentiment = Customer Sentiment
attendant-sentiment-positive = Positive
attendant-sentiment-neutral = Neutral
attendant-sentiment-negative = Negative
attendant-smart-replies = Smart Replies
attendant-confidence = Confidence
attendant-source = Source

# -----------------------------------------------------------------------------
# Customer Details
# -----------------------------------------------------------------------------
attendant-customer-details = Customer Details
attendant-customer-name = Name
attendant-customer-email = Email
attendant-customer-phone = Phone
attendant-customer-location = Location
attendant-customer-tags = Tags

# -----------------------------------------------------------------------------
# Conversation History
# -----------------------------------------------------------------------------
attendant-history = History
attendant-history-resolved = Resolved
attendant-history-transferred = Transferred
attendant-history-abandoned = Abandoned
attendant-view-history = View Full History

# -----------------------------------------------------------------------------
# Toast Messages
# -----------------------------------------------------------------------------
attendant-toast-transferred = Conversation transferred successfully
attendant-toast-resolved = Conversation marked as resolved
attendant-toast-assigned = Conversation assigned to you
attendant-toast-error = An error occurred
attendant-toast-connection-lost = Connection lost. Reconnecting...
attendant-toast-connection-restored = Connection restored

# =============================================================================
# CRM
# =============================================================================

# -----------------------------------------------------------------------------
# CRM Navigation & General
# -----------------------------------------------------------------------------
crm-title = CRM
crm-pipeline = Pipeline
crm-leads = Leads
crm-opportunities = Opportunities
crm-accounts = Accounts
crm-contacts = Contacts
crm-activities = Activities

# -----------------------------------------------------------------------------
# CRM Entities
# -----------------------------------------------------------------------------
crm-lead = Lead
crm-lead-desc = Unqualified prospect
crm-opportunity = Opportunity
crm-opportunity-desc = Qualified sales opportunity
crm-account = Account
crm-account-desc = Company or organization
crm-contact = Contact
crm-contact-desc = Person at an account
crm-activity = Activity
crm-activity-desc = Task, call, or email

# -----------------------------------------------------------------------------
# CRM Actions
# -----------------------------------------------------------------------------
crm-qualify = Qualify
crm-convert = Convert
crm-won = Won
crm-lost = Lost
crm-new-lead = New Lead
crm-new-opportunity = New Opportunity
crm-new-account = New Account
crm-new-contact = New Contact

# -----------------------------------------------------------------------------
# CRM Fields
# -----------------------------------------------------------------------------
crm-stage = Stage
crm-value = Value
crm-probability = Probability
crm-close-date = Close Date
crm-company = Company
crm-phone = Phone
crm-email = Email
crm-source = Source
crm-owner = Owner

# -----------------------------------------------------------------------------
# CRM Pipeline Stages
# -----------------------------------------------------------------------------
crm-pipeline-new = New
crm-pipeline-contacted = Contacted
crm-pipeline-qualified = Qualified
crm-pipeline-proposal = Proposal
crm-pipeline-negotiation = Negotiation
crm-pipeline-closed-won = Closed Won
crm-pipeline-closed-lost = Closed Lost

# -----------------------------------------------------------------------------
# CRM Stats & Metrics
# -----------------------------------------------------------------------------
crm-subtitle = Manage leads, opportunities, and customers
crm-stage-lead = Lead
crm-stage-qualified = Qualified
crm-stage-proposal = Proposal
crm-stage-negotiation = Negotiation
crm-stage-won = Won
crm-stage-lost = Lost
crm-conversion-rate = Conversion Rate
crm-pipeline-value = Pipeline Value
crm-avg-deal = Avg Deal Size
crm-won-month = Won This Month

# -----------------------------------------------------------------------------
# CRM Empty States
# -----------------------------------------------------------------------------
crm-no-leads = No leads found
crm-no-opportunities = No opportunities found
crm-no-accounts = No accounts found
crm-no-contacts = No contacts found
crm-drag-hint = Drag cards to change stage

# =============================================================================
# Billing
# =============================================================================

# -----------------------------------------------------------------------------
# Billing Navigation & General
# -----------------------------------------------------------------------------
billing-title = Billing
billing-invoices = Invoices
billing-payments = Payments
billing-quotes = Quotes
billing-dashboard = Dashboard

# -----------------------------------------------------------------------------
# Billing Entities
# -----------------------------------------------------------------------------
billing-invoice = Invoice
billing-invoice-desc = Bill to customer
billing-payment = Payment
billing-payment-desc = Payment received
billing-quote = Quote
billing-quote-desc = Price quotation

# -----------------------------------------------------------------------------
# Billing Status
# -----------------------------------------------------------------------------
billing-due-date = Due Date
billing-overdue = Overdue
billing-paid = Paid
billing-pending = Pending
billing-draft = Draft
billing-sent = Sent
billing-partial = Partial
billing-cancelled = Cancelled

# -----------------------------------------------------------------------------
# Billing Actions
# -----------------------------------------------------------------------------
billing-new-invoice = New Invoice
billing-new-quote = New Quote
billing-new-payment = New Payment
billing-send-invoice = Send Invoice
billing-record-payment = Record Payment
billing-mark-paid = Mark as Paid
billing-void = Void

# -----------------------------------------------------------------------------
# Billing Fields
# -----------------------------------------------------------------------------
billing-amount = Amount
billing-tax = Tax
billing-subtotal = Subtotal
billing-total = Total
billing-discount = Discount
billing-line-items = Line Items
billing-add-item = Add Item
billing-remove-item = Remove Item
billing-customer = Customer
billing-issue-date = Issue Date
billing-payment-terms = Payment Terms
billing-notes = Notes
billing-invoice-number = Invoice Number
billing-quote-number = Quote Number

# -----------------------------------------------------------------------------
# Billing Reports
# -----------------------------------------------------------------------------
billing-revenue = Revenue
billing-outstanding = Outstanding
billing-this-month = This Month
billing-last-month = Last Month
billing-total-paid = Total Paid
billing-total-overdue = Total Overdue
billing-subtitle = Invoices, payments, and quotes
billing-revenue-month = Revenue This Month
billing-total-revenue = Total Revenue
billing-paid-month = Paid This Month

# -----------------------------------------------------------------------------
# Billing Empty States
# -----------------------------------------------------------------------------
billing-no-invoices = No invoices found
billing-no-payments = No payments found
billing-no-quotes = No quotes found

# =============================================================================
# Products
# =============================================================================

# -----------------------------------------------------------------------------
# Products Navigation & General
# -----------------------------------------------------------------------------
products-title = Products
products-catalog = Catalog
products-services = Services
products-price-lists = Price Lists
products-inventory = Inventory

# -----------------------------------------------------------------------------
# Products Entities
# -----------------------------------------------------------------------------
products-product = Product
products-product-desc = Physical or digital product
products-service = Service
products-service-desc = Service offering
products-price-list = Price List
products-price-list-desc = Pricing tiers

# -----------------------------------------------------------------------------
# Products Actions
# -----------------------------------------------------------------------------
products-new-product = New Product
products-new-service = New Service
products-new-price-list = New Price List
products-new-pricelist = New Price List
products-edit-product = Edit Product
products-duplicate = Duplicate

# -----------------------------------------------------------------------------
# Products Fields
# -----------------------------------------------------------------------------
products-sku = SKU
products-category = Category
products-price = Price
products-unit = Unit
products-stock = Stock
products-cost = Cost
products-margin = Margin
products-barcode = Barcode

# -----------------------------------------------------------------------------
# Products Status
# -----------------------------------------------------------------------------
products-in-stock = In Stock
products-out-of-stock = Out of Stock
products-low-stock = Low Stock
products-active = Active
products-inactive = Inactive
products-featured = Featured
products-archived = Archived

# -----------------------------------------------------------------------------
# Products Stats & Metrics
# -----------------------------------------------------------------------------
products-subtitle = Manage products, services, and pricing
products-items = Products
products-pricelists = Price Lists
products-total-products = Total Products
products-total-services = Total Services

# -----------------------------------------------------------------------------
# Products Empty States
# -----------------------------------------------------------------------------
products-no-products = No products found
products-no-services = No services found
products-no-price-lists = No price lists found

# =============================================================================
# Tickets (Support Cases)
# =============================================================================

# -----------------------------------------------------------------------------
# Tickets Navigation & General
# -----------------------------------------------------------------------------
tickets-title = Tickets
tickets-cases = Cases
tickets-open = Open
tickets-closed = Closed
tickets-all = All Tickets
tickets-my-tickets = My Tickets

# -----------------------------------------------------------------------------
# Tickets Entities
# -----------------------------------------------------------------------------
tickets-case = Case
tickets-case-desc = Support ticket
tickets-resolution = Resolution
tickets-resolution-desc = AI-suggested solution

# -----------------------------------------------------------------------------
# Tickets Priority
# -----------------------------------------------------------------------------
tickets-priority = Priority
tickets-priority-low = Low
tickets-priority-medium = Medium
tickets-priority-high = High
tickets-priority-urgent = Urgent

# -----------------------------------------------------------------------------
# Tickets Status
# -----------------------------------------------------------------------------
tickets-status = Status
tickets-status-new = New
tickets-status-open = Open
tickets-status-pending = Pending
tickets-status-resolved = Resolved
tickets-status-closed = Closed
tickets-status-on-hold = On Hold

# -----------------------------------------------------------------------------
# Tickets Actions
# -----------------------------------------------------------------------------
tickets-new-ticket = New Ticket
tickets-assign = Assign
tickets-reassign = Reassign
tickets-escalate = Escalate
tickets-resolve = Resolve
tickets-reopen = Reopen
tickets-close = Close
tickets-merge = Merge

# -----------------------------------------------------------------------------
# Tickets Fields
# -----------------------------------------------------------------------------
tickets-subject = Subject
tickets-description = Description
tickets-category = Category
tickets-assigned = Assigned To
tickets-unassigned = Unassigned
tickets-created = Created
tickets-updated = Updated
tickets-response-time = Response Time
tickets-resolution-time = Resolution Time
tickets-customer = Customer
tickets-internal-notes = Internal Notes
tickets-attachments = Attachments

# -----------------------------------------------------------------------------
# Tickets AI Features
# -----------------------------------------------------------------------------
tickets-ai-suggestion = AI Suggestion
tickets-apply-suggestion = Apply Suggestion
tickets-ai-summary = AI Summary
tickets-similar-tickets = Similar Tickets
tickets-suggested-articles = Suggested Articles

# -----------------------------------------------------------------------------
# Tickets Empty States
# -----------------------------------------------------------------------------
tickets-no-tickets = No tickets found
tickets-no-open = No open tickets
tickets-no-closed = No closed tickets

# -----------------------------------------------------------------------------
# Security Module
# -----------------------------------------------------------------------------
security-title = Security
security-subtitle = Security tools, compliance scanning, and server protection
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

# CRM / Mail / Campaigns integration keys
crm-email = Email
crm-compose-email = Compose Email
crm-send-email = Send Email
mail-snooze = Snooze
mail-snooze-later-today = Later today (6:00 PM)
mail-snooze-tomorrow = Tomorrow (8:00 AM)
mail-snooze-next-week = Next week (Mon 8:00 AM)
mail-crm-log = Log to CRM
mail-crm-create-lead = Create Lead
mail-add-to-list = Add to List
campaign-send-email = Send Email
