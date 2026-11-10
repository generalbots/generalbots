# =============================================================================
# General Bots — en — Desktop shell, navigation, controls
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

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
# Notifications
# -----------------------------------------------------------------------------
notifications-title = Notifications
notifications-clear = Clear all
notifications-empty = No notifications

# #1853 — desktop chrome, launchers and notification settings.
window-ready = Ready
window-loading = Loading app…
window-minimize = Minimize
window-maximize = Maximize
window-fullscreen = Fullscreen
window-exit-fullscreen = Exit fullscreen
window-close = Close
app-loading = Application loading…
start-menu-search-placeholder = Type to find an app…
start-menu-recent = Recent
start-menu-no-apps = No apps found
start-menu-user = User
category-ai = AI
category-business = Business
category-office = Office
category-dev = Developer
category-system = System
category-tools = Tools
sidebar-apps = Apps
sidebar-apps-toggle = Show/hide apps
sidebar-apps-toggle-aria = Toggle apps panel
sidebar-apps-filter = Filter apps…
sidebar-apps-filter-aria = Filter apps
sidebar-preview = Preview mode
sidebar-preview-hint = Show the applications that are still in preview
sidebar-preview-toggle = Toggle preview mode
sidebar-app-preview-title = {app} (preview)
sidebar-apps-count = {count} apps
sidebar-apps-count-preview = {count} (+ {preview} preview)
spotlight-placeholder = Search apps, files… or type a goal for the Concierge
spotlight-hint = ↑↓ navigate · Enter run · Ctrl+Space or Esc close
spotlight-empty = No results
spotlight-goal = Run goal: “{goal}”
spotlight-goal-subtitle = Concierge · plan & execute across apps
palette-placeholder = Search apps and commands. Or type @ <command> to send to the assistant...
palette-empty = No matching apps or commands
palette-kind-app = App
palette-hint = ↑↓ navigate · Enter run · Esc close · Ctrl+K toggle
palette-open-app = Open {app}
palette-value-prompt = {name}: {summary} — value for {key}:
palette-execute = Execute {method} {path} and summarize the result.
palette-use-command = Use the {command} command to {summary}.
toast-dismiss = Dismiss
settings-language-display = Display Language
settings-language-display-desc = Choose the language for the application interface
settings-language-regional = Regional Settings
settings-language-regional-desc = Configure date, time, and number formats
settings-language-date-format = Date Format
settings-language-time-format = Time Format
settings-language-timezone = Timezone
settings-language-first-day = First Day of Week
settings-language-time-12h = 12-hour (1:30 PM)
settings-language-time-24h = 24-hour (13:30)
settings-language-sunday = Sunday
settings-language-monday = Monday
settings-email-notifications-desc = Sent by the workspace notification sender, never by a connected Gmail/Outlook mailbox.
settings-push-enable = Enable Push Notifications
settings-push-enable-desc = Receive browser push notifications
settings-push-sound = Sound
settings-push-sound-desc = Play sound for notifications
settings-desktop-notifications = Desktop Notifications
settings-desktop-notifications-desc = Show notifications on desktop
settings-badge-count = Badge Count
settings-badge-count-desc = Show unread count badge
settings-save-preferences = Save Preferences
settings-notifications-saved = Notification preferences saved
settings-notification-recipient = Delivery mailbox
settings-notification-recipient-desc = Leave blank to use your primary Mail account or profile email.
language-search-placeholder = Search languages…
settings-profile-title = Profile Settings
settings-security-title = Security Settings
