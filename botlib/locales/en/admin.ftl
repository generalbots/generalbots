# =============================================================================
# General Bots - Admin Translations (English)
# =============================================================================
# Administrative interface translations for the GB Admin Panel
# =============================================================================

# -----------------------------------------------------------------------------
# Admin Navigation & Dashboard
# -----------------------------------------------------------------------------
admin-title = Administration
admin-dashboard = Admin Dashboard
admin-overview = Overview
admin-welcome = Welcome to the Admin Panel

admin-nav-dashboard = Dashboard
admin-nav-users = Users
admin-nav-bots = Bots
admin-nav-tenants = Tenants
admin-nav-settings = Settings
admin-nav-logs = Logs
admin-nav-analytics = Analytics
admin-nav-security = Security
admin-nav-integrations = Integrations
admin-nav-billing = Billing
admin-nav-support = Support
admin-nav-groups = Groups
admin-nav-dns = DNS
admin-nav-system = System

# -----------------------------------------------------------------------------
# Admin Quick Actions
# -----------------------------------------------------------------------------
admin-quick-actions = Quick Actions
admin-create-user = Create User
admin-create-group = Create Group
admin-register-dns = Register DNS
admin-recent-activity = Recent Activity
admin-system-health = System Health

# -----------------------------------------------------------------------------
# User Management
# -----------------------------------------------------------------------------
admin-users-title = User Management
admin-users-list = User List
admin-users-add = Add User
admin-users-edit = Edit User
admin-users-delete = Delete User
admin-users-search = Search users...
admin-users-filter = Filter Users
admin-users-export = Export Users
admin-users-import = Import Users
admin-users-total = Total Users
admin-users-active = Active Users
admin-users-inactive = Inactive Users
admin-users-suspended = Suspended Users
admin-users-pending = Pending Verification
admin-users-last-login = Last Login
admin-users-created = Created
admin-users-role = Role
admin-users-status = Status
admin-users-actions = Actions
admin-users-no-users = No users found
admin-users-confirm-delete = Are you sure you want to delete this user?
admin-users-deleted = User deleted successfully
admin-users-saved = User saved successfully
admin-users-invite = Invite User
admin-users-invite-sent = Invitation sent successfully
admin-users-bulk-actions = Bulk Actions
admin-users-select-all = Select All
admin-users-deselect-all = Deselect All

# User Details
admin-user-details = User Details
admin-user-profile = Profile
admin-user-email = Email
admin-user-name = Name
admin-user-phone = Phone
admin-user-avatar = Avatar
admin-user-timezone = Timezone
admin-user-language = Language
admin-user-role-admin = Administrator
admin-user-role-manager = Manager
admin-user-role-user = User
admin-user-role-viewer = Viewer
admin-user-status-active = Active
admin-user-status-inactive = Inactive
admin-user-status-suspended = Suspended
admin-user-status-pending = Pending
admin-user-permissions = Permissions
admin-user-activity = Activity Log
admin-user-sessions = Active Sessions
admin-user-terminate-session = Terminate Session
admin-user-terminate-all = Terminate All Sessions
admin-user-reset-password = Reset Password
admin-user-force-logout = Force Logout
admin-user-enable-2fa = Enable 2FA
admin-user-disable-2fa = Disable 2FA

# -----------------------------------------------------------------------------
# Group Management
# -----------------------------------------------------------------------------
admin-groups-title = Group Management
admin-groups-subtitle = Manage groups, members, and permissions
admin-groups-list = Group List
admin-groups-add = Add Group
admin-groups-create = Create Group
admin-groups-edit = Edit Group
admin-groups-delete = Delete Group
admin-groups-search = Search groups...
admin-groups-filter = Filter Groups
admin-groups-total = Total Groups
admin-groups-active = Active Groups
admin-groups-no-groups = No groups found
admin-groups-confirm-delete = Are you sure you want to delete this group?
admin-groups-deleted = Group deleted successfully
admin-groups-saved = Group saved successfully
admin-groups-created = Group created successfully
admin-groups-loading = Loading groups...

# Group Details
admin-group-details = Group Details
admin-group-name = Group Name
admin-group-description = Description
admin-group-visibility = Visibility
admin-group-visibility-public = Public
admin-group-visibility-private = Private
admin-group-visibility-hidden = Hidden
admin-group-join-policy = Join Policy
admin-group-join-invite = Invite Only
admin-group-join-request = Request to Join
admin-group-join-open = Open
admin-group-members = Members
admin-group-member-count = { $count ->
    [one] { $count } member
   *[other] { $count } members
}
admin-group-add-member = Add Member
admin-group-remove-member = Remove Member
admin-group-permissions = Permissions
admin-group-settings = Settings
admin-group-analytics = Analytics
admin-group-overview = Overview

# Group View Modes
admin-groups-view-grid = Grid View
admin-groups-view-list = List View
admin-groups-all-visibility = All Visibility

# -----------------------------------------------------------------------------
# DNS Management
# -----------------------------------------------------------------------------
admin-dns-title = DNS Management
admin-dns-subtitle = Register and manage DNS hostnames for your bots
admin-dns-register = Register Hostname
admin-dns-registered = Registered Hostnames
admin-dns-search = Search hostnames...
admin-dns-refresh = Refresh
admin-dns-loading = Loading DNS records...
admin-dns-no-records = No DNS records found
admin-dns-confirm-delete = Are you sure you want to remove this hostname?
admin-dns-deleted = Hostname removed successfully
admin-dns-saved = DNS record saved successfully
admin-dns-created = Hostname registered successfully

# DNS Form Fields
admin-dns-hostname = Hostname
admin-dns-hostname-placeholder = mybot.example.com
admin-dns-hostname-help = Enter the full domain name you want to register
admin-dns-record-type = Record Type
admin-dns-record-type-a = A (IPv4)
admin-dns-record-type-aaaa = AAAA (IPv6)
admin-dns-record-type-cname = CNAME
admin-dns-ttl = TTL (seconds)
admin-dns-ttl-5min = 5 minutes (300)
admin-dns-ttl-1hour = 1 hour (3600)
admin-dns-ttl-1day = 1 day (86400)
admin-dns-target = Target/IP Address
admin-dns-target-placeholder-ipv4 = 192.168.1.1
admin-dns-target-placeholder-ipv6 = 2001:db8::1
admin-dns-target-placeholder-cname = target.example.com
admin-dns-target-help-a = Enter the IPv4 address to point to
admin-dns-target-help-aaaa = Enter the IPv6 address to point to
admin-dns-target-help-cname = Enter the target domain name
admin-dns-auto-ssl = Automatically provision SSL certificate

# DNS Table Headers
admin-dns-col-hostname = Hostname
admin-dns-col-type = Type
admin-dns-col-target = Target
admin-dns-col-ttl = TTL
admin-dns-col-ssl = SSL
admin-dns-col-status = Status
admin-dns-col-actions = Actions

# DNS Status
admin-dns-status-active = Active
admin-dns-status-pending = Pending
admin-dns-status-error = Error
admin-dns-ssl-enabled = SSL Enabled
admin-dns-ssl-disabled = No SSL
admin-dns-ssl-pending = SSL Pending

# DNS Info Cards
admin-dns-help-title = DNS Configuration Help
admin-dns-help-a-record = A Record
admin-dns-help-a-record-desc = Maps a domain name to an IPv4 address. Use this to point your hostname directly to a server IP.
admin-dns-help-aaaa-record = AAAA Record
admin-dns-help-aaaa-record-desc = Maps a domain name to an IPv6 address. Similar to A record but for IPv6 connectivity.
admin-dns-help-cname-record = CNAME Record
admin-dns-help-cname-record-desc = Creates an alias from one domain to another. Useful for pointing subdomains to your main domain.
admin-dns-help-ssl = SSL/TLS
admin-dns-help-ssl-desc = Automatically provisions Let's Encrypt certificates for secure HTTPS connections.

# DNS Edit/Remove Modals
admin-dns-edit-title = Edit DNS Record
admin-dns-remove-title = Remove Hostname
admin-dns-remove-warning = This will delete the DNS record and any associated SSL certificates. The hostname will no longer resolve.

# -----------------------------------------------------------------------------
# Bot Management
# -----------------------------------------------------------------------------
admin-bots-title = Bot Management
admin-bots-list = Bot List
admin-bots-add = Add Bot
admin-bots-edit = Edit Bot
admin-bots-delete = Delete Bot
admin-bots-search = Search bots...
admin-bots-filter = Filter Bots
admin-bots-total = Total Bots
admin-bots-active = Active Bots
admin-bots-inactive = Inactive Bots
admin-bots-draft = Draft Bots
admin-bots-published = Published Bots
admin-bots-no-bots = No bots found
admin-bots-confirm-delete = Are you sure you want to delete this bot?
admin-bots-deleted = Bot deleted successfully
admin-bots-saved = Bot saved successfully
admin-bots-duplicate = Duplicate Bot
admin-bots-export = Export Bot
admin-bots-import = Import Bot
admin-bots-publish = Publish
admin-bots-unpublish = Unpublish
admin-bots-test = Test Bot
admin-bots-logs = Bot Logs
admin-bots-analytics = Bot Analytics
admin-bots-conversations = Conversations
admin-bots-templates = Templates
admin-bots-dialogs = Dialogs
admin-bots-knowledge-base = Knowledge Base

# Bot Details
admin-bot-details = Bot Details
admin-bot-name = Bot Name
admin-bot-description = Description
admin-bot-avatar = Bot Avatar
admin-bot-language = Language
admin-bot-timezone = Timezone
admin-bot-greeting = Greeting Message
admin-bot-fallback = Fallback Message
admin-bot-channels = Channels
admin-bot-channel-web = Web Chat
admin-bot-channel-whatsapp = WhatsApp
admin-bot-channel-telegram = Telegram
admin-bot-channel-slack = Slack
admin-bot-channel-teams = Microsoft Teams
admin-bot-channel-email = Email
admin-bot-model = AI Model
admin-bot-temperature = Temperature
admin-bot-max-tokens = Max Tokens
admin-bot-system-prompt = System Prompt

# -----------------------------------------------------------------------------
# Tenant Management
# -----------------------------------------------------------------------------
admin-tenants-title = Tenant Management
admin-tenants-list = Tenant List
admin-tenants-add = Add Tenant
admin-tenants-edit = Edit Tenant
admin-tenants-delete = Delete Tenant
admin-tenants-search = Search tenants...
admin-tenants-total = Total Tenants
admin-tenants-active = Active Tenants
admin-tenants-suspended = Suspended Tenants
admin-tenants-trial = Trial Tenants
admin-tenants-no-tenants = No tenants found
admin-tenants-confirm-delete = Are you sure you want to delete this tenant?
admin-tenants-deleted = Tenant deleted successfully
admin-tenants-saved = Tenant saved successfully

# Tenant Details
admin-tenant-details = Tenant Details
admin-tenant-name = Tenant Name
admin-tenant-domain = Domain
admin-tenant-plan = Plan
admin-tenant-plan-free = Free
admin-tenant-plan-starter = Starter
admin-tenant-plan-professional = Professional
admin-tenant-plan-enterprise = Enterprise
admin-tenant-users = Users
admin-tenant-bots = Bots
admin-tenant-storage = Storage Used
admin-tenant-api-calls = API Calls
admin-tenant-limits = Usage Limits
admin-tenant-billing = Billing Info

# -----------------------------------------------------------------------------
# System Settings
# -----------------------------------------------------------------------------
admin-settings-title = System Settings
admin-settings-general = General Settings
admin-settings-security = Security Settings
admin-settings-email = Email Settings
admin-settings-storage = Storage Settings
admin-settings-integrations = Integrations
admin-settings-api = API Settings
admin-settings-appearance = Appearance
admin-settings-localization = Localization
admin-settings-notifications = Notifications
admin-settings-backup = Backup & Restore
admin-settings-maintenance = Maintenance Mode
admin-settings-saved = Settings saved successfully
admin-settings-reset = Reset to Defaults
admin-settings-confirm-reset = Are you sure you want to reset all settings to defaults?

# General Settings
admin-settings-site-name = Site Name
admin-settings-site-url = Site URL
admin-settings-admin-email = Admin Email
admin-settings-support-email = Support Email
admin-settings-default-language = Default Language
admin-settings-default-timezone = Default Timezone
admin-settings-date-format = Date Format
admin-settings-time-format = Time Format
admin-settings-currency = Currency

# Email Settings
admin-settings-smtp-host = SMTP Host
admin-settings-smtp-port = SMTP Port
admin-settings-smtp-user = SMTP Username
admin-settings-smtp-password = SMTP Password
admin-settings-smtp-encryption = Encryption
admin-settings-smtp-from-name = From Name
admin-settings-smtp-from-email = From Email
admin-settings-smtp-test = Send Test Email
admin-settings-smtp-test-success = Test email sent successfully
admin-settings-smtp-test-failed = Failed to send test email

# Storage Settings
admin-settings-storage-provider = Storage Provider
admin-settings-storage-local = Local Storage
admin-settings-storage-s3 = Amazon S3
admin-settings-storage-minio = MinIO
admin-settings-storage-gcs = Google Cloud Storage
admin-settings-storage-azure = Azure Blob Storage
admin-settings-storage-bucket = Bucket Name
admin-settings-storage-region = Region
admin-settings-storage-access-key = Access Key
admin-settings-storage-secret-key = Secret Key
admin-settings-storage-endpoint = Endpoint URL

# -----------------------------------------------------------------------------
# System Logs
# -----------------------------------------------------------------------------
admin-logs-title = System Logs
admin-logs-search = Search logs...
admin-logs-filter-level = Filter by Level
admin-logs-filter-source = Filter by Source
admin-logs-filter-date = Filter by Date
admin-logs-level-all = All Levels
admin-logs-level-debug = Debug
admin-logs-level-info = Info
admin-logs-level-warning = Warning
admin-logs-level-error = Error
admin-logs-level-critical = Critical
admin-logs-export = Export Logs
admin-logs-clear = Clear Logs
admin-logs-confirm-clear = Are you sure you want to clear all logs?
admin-logs-cleared = Logs cleared successfully
admin-logs-no-logs = No logs found
admin-logs-refresh = Refresh
admin-logs-auto-refresh = Auto Refresh
admin-logs-timestamp = Timestamp
admin-logs-level = Level
admin-logs-source = Source
admin-logs-message = Message
admin-logs-details = Details

# -----------------------------------------------------------------------------
# Analytics
# -----------------------------------------------------------------------------
admin-analytics-title = Analytics
admin-analytics-overview = Overview
admin-analytics-users = User Analytics
admin-analytics-bots = Bot Analytics
admin-analytics-conversations = Conversation Analytics
admin-analytics-performance = Performance
admin-analytics-period = Time Period
admin-analytics-period-today = Today
admin-analytics-period-week = This Week
admin-analytics-period-month = This Month
admin-analytics-period-quarter = This Quarter
admin-analytics-period-year = This Year
admin-analytics-period-custom = Custom Range
admin-analytics-export = Export Report
admin-analytics-total-users = Total Users
admin-analytics-new-users = New Users
admin-analytics-active-users = Active Users
admin-analytics-total-bots = Total Bots
admin-analytics-active-bots = Active Bots
admin-analytics-total-conversations = Total Conversations
admin-analytics-avg-response-time = Avg Response Time
admin-analytics-satisfaction-rate = Satisfaction Rate
admin-analytics-resolution-rate = Resolution Rate

# -----------------------------------------------------------------------------
# Security
# -----------------------------------------------------------------------------
admin-security-title = Security
admin-security-overview = Security Overview
admin-security-audit-log = Audit Log
admin-security-login-attempts = Login Attempts
admin-security-blocked-ips = Blocked IPs
admin-security-api-keys = API Keys
admin-security-webhooks = Webhooks
admin-security-cors = CORS Settings
admin-security-rate-limiting = Rate Limiting
admin-security-encryption = Encryption
admin-security-2fa = Two-Factor Authentication
admin-security-sso = Single Sign-On
admin-security-password-policy = Password Policy

# API Keys
admin-api-keys-title = API Keys
admin-api-keys-add = Create API Key
admin-api-keys-name = Key Name
admin-api-keys-key = API Key
admin-api-keys-secret = Secret Key
admin-api-keys-created = Created
admin-api-keys-last-used = Last Used
admin-api-keys-expires = Expires
admin-api-keys-never = Never
admin-api-keys-revoke = Revoke
admin-api-keys-confirm-revoke = Are you sure you want to revoke this API key?
admin-api-keys-revoked = API key revoked successfully
admin-api-keys-created-success = API key created successfully
admin-api-keys-copy = Copy to Clipboard
admin-api-keys-copied = Copied!
admin-api-keys-warning = Make sure to copy your API key now. You won't be able to see it again!

# -----------------------------------------------------------------------------
# Billing
# -----------------------------------------------------------------------------
admin-billing-title = Billing
admin-billing-overview = Billing Overview
admin-billing-current-plan = Current Plan
admin-billing-usage = Usage
admin-billing-invoices = Invoices
admin-billing-payment-methods = Payment Methods
admin-billing-upgrade = Upgrade Plan
admin-billing-downgrade = Downgrade Plan
admin-billing-cancel = Cancel Subscription
admin-billing-invoice-date = Invoice Date
admin-billing-invoice-amount = Amount
admin-billing-invoice-status = Status
admin-billing-invoice-paid = Paid
admin-billing-invoice-pending = Pending
admin-billing-invoice-overdue = Overdue
admin-billing-invoice-download = Download Invoice

# -----------------------------------------------------------------------------
# Backup & Restore
# -----------------------------------------------------------------------------
admin-backup-title = Backup & Restore
admin-backup-create = Create Backup
admin-backup-restore = Restore Backup
admin-backup-schedule = Schedule Backups
admin-backup-list = Backup History
admin-backup-name = Backup Name
admin-backup-size = Size
admin-backup-created = Created
admin-backup-download = Download
admin-backup-delete = Delete
admin-backup-confirm-restore = Are you sure you want to restore this backup? This will overwrite current data.
admin-backup-confirm-delete = Are you sure you want to delete this backup?
admin-backup-in-progress = Backup in progress...
admin-backup-completed = Backup completed successfully
admin-backup-failed = Backup failed
admin-backup-restore-in-progress = Restore in progress...
admin-backup-restore-completed = Restore completed successfully
admin-backup-restore-failed = Restore failed

# -----------------------------------------------------------------------------
# Maintenance Mode
# -----------------------------------------------------------------------------
admin-maintenance-title = Maintenance Mode
admin-maintenance-enable = Enable Maintenance Mode
admin-maintenance-disable = Disable Maintenance Mode
admin-maintenance-status = Current Status
admin-maintenance-active = Maintenance mode is active
admin-maintenance-inactive = Maintenance mode is inactive
admin-maintenance-message = Maintenance Message
admin-maintenance-default-message = We are currently performing scheduled maintenance. Please check back soon.
admin-maintenance-allowed-ips = Allowed IP Addresses
admin-maintenance-confirm-enable = Are you sure you want to enable maintenance mode? Users will not be able to access the system.

# -----------------------------------------------------------------------------
# Common Admin UI Elements
# -----------------------------------------------------------------------------
admin-required = Required
admin-optional = Optional
admin-loading = Loading...
admin-saving = Saving...
admin-deleting = Deleting...
admin-confirm = Confirm
admin-cancel = Cancel
admin-save = Save
admin-create = Create
admin-update = Update
admin-delete = Delete
admin-edit = Edit
admin-view = View
admin-close = Close
admin-back = Back
admin-next = Next
admin-previous = Previous
admin-refresh = Refresh
admin-export = Export
admin-import = Import
admin-search = Search
admin-filter = Filter
admin-clear = Clear
admin-select = Select
admin-select-all = Select All
admin-deselect-all = Deselect All
admin-actions = Actions
admin-more-actions = More Actions
admin-no-data = No data available
admin-error = An error occurred
admin-success = Success
admin-warning = Warning
admin-info = Information

# Table Pagination
admin-showing = Showing { $from } to { $to } of { $total } results
admin-page = Page { $current } of { $total }
admin-items-per-page = Items per page
admin-go-to-page = Go to page

# Bulk Actions
admin-bulk-delete = Delete Selected
admin-bulk-export = Export Selected
admin-bulk-activate = Activate Selected
admin-bulk-deactivate = Deactivate Selected
admin-selected-count = { $count ->
    [one] { $count } item selected
   *[other] { $count } items selected
}
