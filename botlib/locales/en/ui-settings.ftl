# =============================================================================
# General Bots — en — Settings
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

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
settings-create-api-key = Create API Key
settings-api-key-name = Key Name
settings-api-key-scopes = Scopes (comma-separated)
settings-api-key-scopes-hint = Leave empty for the default "user" scope.
settings-api-key-expiry = Expires in (days)
settings-api-key-expiry-hint = Leave empty for no expiration.
settings-add-webhook = Add Webhook
settings-webhook-url = Webhook URL
settings-webhook-events = Events (comma-separated)
settings-webhook-events-hint = Leave empty to receive all events.
settings-timezone = Timezone
settings-date-format = Date Format
settings-time-format = Time Format

# -----------------------------------------------------------------------------
# Settings - Language & Localization
# -----------------------------------------------------------------------------
settings-language-desc = Choose your preferred language
settings-display-language = Display Language
settings-language-affects = Affects all text in the application
settings-date-format-desc = How dates are displayed
settings-time-format-desc = 12-hour or 24-hour clock
settings-language-changed = Language changed successfully
settings-reload-required = Page reload required to apply changes

# Settings - Profile
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
# OAuth Account Linking (Settings)
# -----------------------------------------------------------------------------
oauth-connected-accounts = Connected Accounts
oauth-connect = Connect
oauth-unlink = Unlink
oauth-not-connected = Not connected
oauth-linked = Linked
oauth-no-accounts = No accounts linked yet.
oauth-loading = Loading linked accounts…

## Payment cards (Stripe SetupIntent)
cards-title = Payment & Cards
cards-saved = Saved cards
cards-hint = Cards are stored securely by our payment provider. Card numbers never reach our servers.
cards-add = Add card
cards-add-first = Add your first card
cards-none = No cards saved yet
cards-empty-hint = Add a card to enable automatic billing and faster checkouts. You will be redirected to our secure payment provider to enter your card details.
cards-default = Default
cards-set-default = Set default
cards-default-btn = Default card
cards-remove = Remove
cards-remove-confirm = Remove this card?
cards-expires = Expires
cards-load-error = Could not load saved cards.
cards-add-error = Could not add a card
cards-default-error = Could not update default
cards-remove-error = Could not remove card
cards-default-updated = Default card updated
cards-removed = Card removed

# Settings → Model (#2568)
mdl-title = Model
mdl-subtitle = Where the bot thinks. Choose a model that runs on this machine, your own provider endpoint, or the free hosted model.
mdl-current = Current model
mdl-where = Where the bot thinks
mdl-local = On this computer
mdl-own = Your server or provider key
mdl-free = Free hosted model
mdl-active = In use
mdl-endpoint = Endpoint URL
mdl-model = Model
mdl-key = API key
mdl-key-stored = key stored
mdl-test = Test
mdl-save = Save
mdl-test-ok = Reachable, latency %ms ms
mdl-test-fail = Failed: %e
mdl-testing = Testing…
mdl-saving = Saving…
mdl-saved = Saved. Takes effect from the next message.
mdl-swapped = Ready. The new model is now active.
mdl-restart = Saved. Restart the app to use the new model.
mdl-admin-only = Only the administrator can change the model.
mdl-loading = Loading…
mdl-local-active = Active local model: %m
mdl-local-hint = Pick a model on the On-device AI page, then it can be used here.
mdl-local-open = Open On-device AI
mdl-free-note = Everything you write and attach is sent to %h.
mdl-inconsistent = The model configuration is inconsistent.
mdl-restore-free = Restore free model
mdl-confirm-external = What you write and attach will be sent to %h, outside this computer. Continue?
mdl-scope-machine = On this computer
mdl-scope-network = Your organization's server (%h)
mdl-scope-external = An external host (%h)
mdl-unavailable = This server does not expose the model API.

# Settings → Instructions, Shortcuts, Text size (#2568)
instr-title = Custom instructions
instr-subtitle = Tell the assistant about yourself and how you want it to respond.
instr-enabled = Enable custom instructions
instr-enabled-desc = When off, the assistant answers without these instructions.
instr-about = What should the assistant know about you?
instr-about-placeholder = Role, language, context, preferences…
instr-response = How should the assistant respond?
instr-response-placeholder = Tone, level of detail, format…
instr-save = Save
instr-saving = Saving…
instr-saved = Saved.
instr-save-failed = Could not save: %e
instr-loading = Loading…
instr-load-failed = Could not load: %e
instr-unavailable = This server does not expose custom instructions.
sc-subtitle = Press Shift + ? anywhere in the suite to open this list.
sc-group-general = General
sc-group-drive = Drive
sc-note = Shortcuts are ignored while you are typing in a field, so they never interrupt a message or a form.
ts-title = Text size
ts-desc = Choose how large text appears across the suite
ts-small = Small
ts-default = Default
ts-large = Large
ts-xlarge = Extra large
# Settings → Appearance: chat background pattern
chatbg-title = Chat background
chatbg-desc = Choose the pattern behind your conversations
chatbg-none = None
chatbg-image = Paper
chatbg-dots = Dots
chatbg-microdots = Micro dots
chatbg-grid = Grid
chatbg-blueprint = Blueprint
chatbg-weave = Weave
chatbg-mosaic = Mosaic

# semantic i18n key (#2542)
settings-network-participation-desc = Control whether this installation participates in the peer network

# semantic i18n key (#2542)
settings-p2p-network = P2P Network

# semantic i18n key (#2542)
settings-network-mode-label = Network mode

# semantic i18n key (#2542)
settings-network-mode-p2p = P2P (recommended)

# semantic i18n key (#2542)
settings-role-viewer = Viewer

# semantic i18n key (#2542)
settings-register-dns-hostname = Register DNS Hostname

# semantic i18n key (#2542)
settings-dns-aaaa-ipv6 = AAAA (IPv6)

# semantic i18n key (#2542)
settings-dns-cname = CNAME

# semantic i18n key (#2542)
settings-create-token = Create token

# semantic i18n key (#2542)
settings-system-preview-desc = Preview mode and version information for this deployment

# semantic i18n key (#2542)
settings-system-preview-surface-desc = Controls how much surface is revealed, not what is installed.

# semantic i18n key (#2542)
settings-system-preview-toggle-desc = Reveals apps in preview and the full settings tree. Off keeps only the essential settings for this edition.

# semantic i18n key (#2542)
settings-component-versions = Component versions

# semantic i18n key (#2542)
settings-component-versions-desc = BotServer, BotUI and service versions for this installation

# semantic i18n key (#2542)
settings-open-versions = Open Versions

# semantic i18n key (#2542)
settings-on-device-ai = On-device AI

# semantic i18n key (#2542)
settings-operating-system = Operating System

# semantic i18n key (#2542)
settings-ledger = Ledger

# semantic i18n key (#2542)
settings-updates = Updates

# semantic i18n key (#2542)
settings-this-workspace = This workspace

# semantic i18n key (#2542)
settings-loading-identity = Loading identity…

# semantic i18n key (#2542)
settings-plan-placeholder = Plan: —

# semantic i18n key (#2542)
settings-preview-placeholder = Preview: —

# semantic i18n key (#2542)
settings-oda-subtitle = Models that run on this machine. Download a model that fits the hardware, activate it, and the local engine uses it on its next start. No API keys are used here.

# semantic i18n key (#2542)
settings-oda-detecting-hardware = Detecting hardware…

# semantic i18n key (#2542)
settings-oda-recommended-for-machine = Recommended for this machine

# semantic i18n key (#2542)
settings-oda-activate = Activate

# semantic i18n key (#2542)
settings-oda-models = Models

# semantic i18n key (#2542)
settings-oda-model = Model

# semantic i18n key (#2542)
settings-oda-params = Params

# semantic i18n key (#2542)
settings-oda-quant = Quant

# semantic i18n key (#2542)
settings-oda-min-vram = Min VRAM

# semantic i18n key (#2542)
settings-oda-loading-models = Loading models…
