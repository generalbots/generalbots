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
