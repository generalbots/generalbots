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
# Settings → Model — provider picker and per-key models (#2568)
mdl-prov-intro = 选择提供商。地址会自动填入；您只需选择模型并粘贴密钥。
mdl-prov-search = 搜索提供商
mdl-prov-connect = 连接
mdl-prov-none = 没有该名称的提供商。
mdl-prov-org-title = 您的组织服务器
mdl-prov-org-desc = 任何与 OpenAI API 兼容的服务器（vLLM、Ollama、LM Studio、Azure…）
mdl-prov-add = 添加
mdl-prov-back = 所有提供商
mdl-prov-address = 地址
mdl-prov-auto = 自动
mdl-prov-url-ph = https://server.example/v1/chat/completions
mdl-prov-model-list = 模型列表
mdl-prov-other = 其他模型…
mdl-prov-other-ph = 准确的模型名称
mdl-prov-get-key = 获取密钥
mdl-prov-key-ph = 在此粘贴密钥
mdl-prov-key-opt = 如果服务器不需要密钥，请留空。
mdl-prov-key-again = 已存储密钥。请重新粘贴密钥以保存。
mdl-prov-sent = 您编写和附加的内容将发送到 %h（此计算机之外）。
mdl-prov-said = 提供商返回：
mdl-prov-test = 测试连接
mdl-prov-use = 使用此提供商
mdl-prov-url-needed = 请输入服务器地址。
mdl-prov-model-needed = 请选择模型。
mdl-prov-key-needed = 请粘贴 API 密钥。
mdl-err-key-refused = 密钥被拒绝
mdl-err-provider-refused = 提供商拒绝了请求：请检查密钥和模型
mdl-err-not-found = 找不到模型或地址
mdl-err-limit = 已达到使用限额
mdl-err-no-answer = 服务器未响应
mdl-err-timeout = 服务器响应超时
mdl-prov-desc-openai = 模型 GPT
mdl-prov-desc-xai = 模型 Grok
mdl-prov-desc-kimi = 模型 Kimi
mdl-prov-desc-zhipu = 模型 GLM
mdl-prov-desc-hunyuan = 模型 Hunyuan
mdl-prov-desc-minimax = 模型 MiniMax
mdl-prov-desc-anthropic = Claude Opus、Sonnet 和 Haiku
mdl-prov-desc-gemini = 通过 Google AI Studio 使用 Gemini
mdl-prov-desc-meta = 通过 Meta API 使用 Llama
mdl-prov-desc-mistral = Mistral Large 和 Small（欧洲）
mdl-prov-desc-deepseek = DeepSeek Chat 和 Reasoner
mdl-prov-desc-qwen = 通过 Model Studio 使用 Qwen
mdl-prov-desc-openrouter = 一个密钥即可使用数百个模型
mdl-prov-desc-groq = 响应极快的开放模型
mdl-km-title = 此密钥的模型
mdl-km-fetch = 获取模型
mdl-km-hint = 获取此密钥可用的模型，并开启要在聊天菜单中显示的模型。
mdl-km-unavailable = 此服务器不支持模型发现功能。
mdl-km-no-match = 没有该名称的模型。
mdl-km-filter = 筛选模型
mdl-km-count = %t 个中已开启 %n 个
mdl-km-save = 保存模型
mdl-km-fetch-fail = 无法获取模型（%e）。
mdl-km-empty = 提供商未返回任何模型。
mdl-km-fetching = 正在获取模型…
mdl-km-saving = 正在保存…
mdl-km-saved = 模型已保存。已开启的模型将显示在聊天菜单中。
mdl-km-save-fail = 无法保存（%e）。
mdl-km-stored = 密钥字段为空时，搜索将使用已存储的密钥。

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
settings-network-p2p-description = Offline mode keeps every connection on <code>127.0.0.1</code> and shared links work on your local network only.<br />P2P enables future features such as free shared-link relaying and peer discovery. No data leaves this machine until you explicitly share something.
settings-network-p2p-restart-hint = This preference is saved locally and takes effect after the app restarts. The relay endpoint is fixed (<code id="network-relay-host">peer.generalbots.org</code>); peer and home-site resolution always uses it, so no manual configuration is required.

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

# semantic i18n key (#2542)
settings-date-format-mm-dd-yyyy-us = MM/DD/YYYY (US)

# semantic i18n key (#2542)
settings-date-format-dd-mm-yyyy-eu-br = DD/MM/YYYY (EU/BR)

# semantic i18n key (#2542)
settings-date-format-yyyy-mm-dd-iso = YYYY-MM-DD (ISO)

# semantic i18n key (#2542)
settings-timezone-america-new-york = New York (GMT-5)

# semantic i18n key (#2542)
settings-timezone-america-los-angeles = Los Angeles (GMT-8)

# semantic i18n key (#2542)
settings-timezone-europe-london = London (GMT+0)

# semantic i18n key (#2542)
settings-timezone-europe-paris = Paris (GMT+1)

# semantic i18n key (#2542)
settings-timezone-asia-tokyo = Tokyo (GMT+9)

# semantic i18n key (#2542)
settings-desktop-wallpaper = Desktop Wallpaper

# semantic i18n key (#2542)
settings-desktop-wallpaper-desc = Choose the background displayed on your desktop

# semantic i18n key (#2542)
settings-organizations-bots = Organizations & Bots

# semantic i18n key (#2542)
settings-organizations-bots-desc = Manage your organizations and bot hierarchy

# semantic i18n key (#2542)
settings-current-organization = Current Organization

# semantic i18n key (#2542)
settings-current-organization-desc = Select the organization you want to manage

# semantic i18n key (#2542)
settings-loading-organizations = Loading organizations...

# semantic i18n key (#2542)
settings-bot-tree = Bot Tree

# semantic i18n key (#2542)
settings-bot-tree-desc = Bots belonging to the selected organization

# semantic i18n key (#2542)
settings-loading-bots = Loading bots...

# semantic i18n key (#2542)
settings-storage-sync-desc = Manage your storage and sync settings

# semantic i18n key (#2542)
settings-knowledge = Knowledge

# semantic i18n key (#2542)
settings-drive-files-as-context = Use all Drive files as knowledge context

# semantic i18n key (#2542)
settings-drive-files-as-context-desc = When enabled (the default), the names of your recent Drive files travel with each chat turn so the assistant can use them as context. Turn it off to keep Drive out of the prompt.

# semantic i18n key (#2542)
settings-loading-storage-information = Loading storage information...

# semantic i18n key (#2542)
settings-connected-cloud-storage = Connected Cloud Storage

# semantic i18n key (#2542)
settings-loading-storage-connections = Loading storage connections...

# semantic i18n key (#2542)
settings-os-subtitle = Low-level host settings and tools for this shell installation

# semantic i18n key (#2542)
settings-os-host-identity-desc = Host identity, kernel and timezone

# semantic i18n key (#2542)
settings-os-kernel = Kernel

# semantic i18n key (#2542)
settings-os-architecture = Architecture

# semantic i18n key (#2542)
settings-os-host-readonly-hint = Hostname and timezone are read-only in this phase. Use the Terminal app or the host tools in the kiosk image to change them.

# semantic i18n key (#2542)
settings-os-services-desc = Start, stop and restart host services

# semantic i18n key (#2542)
settings-os-load-services-hint = Open the panel to load services.

# semantic i18n key (#2542)
settings-os-disk-usage-desc = Disk usage for mounted filesystems

# semantic i18n key (#2542)
settings-os-load-storage-hint = Open the panel to load storage.

# semantic i18n key (#2542)
settings-os-log-files = Log files

# semantic i18n key (#2542)
settings-os-log-files-desc = Files this installation writes, with location and size

# semantic i18n key (#2542)
settings-os-load-log-files-hint = Open the panel to load log files.

# semantic i18n key (#2542)
settings-os-tail-log-desc = Tail the last 200 lines of a service log

# semantic i18n key (#2542)
settings-os-reload = Reload

# semantic i18n key (#2542)
settings-os-restart-service-title = Restart service

# semantic i18n key (#2542)
settings-os-the-service = the service

# semantic i18n key (#2542)
settings-os-service-name = Service name

# semantic i18n key (#2542)
settings-os-restart-action = Restart

# semantic i18n key (#2542)
settings-os-restart-confirm-message = Restart <strong id="os-restart-service-name" data-i18n="settings-os-the-service">the service</strong>? Type the service name to confirm; the restart is written to the audit log.

# semantic i18n key (#2542)
settings-profile-directory-managed = Managed by the Zitadel directory

# semantic i18n key (#2542)
settings-profile-open-directory-profile = Open directory profile

# semantic i18n key (#2542)
settings-profile-public-title = Public Profile

# semantic i18n key (#2542)
settings-profile-public-desc = How you appear on your General Bots home page

# semantic i18n key (#2542)
settings-profile-public-display-name = Public Display Name

# semantic i18n key (#2542)
settings-profile-public-url-label = Your profile URL will be

# semantic i18n key (#2542)
settings-profile-coming-soon = Coming soon

# semantic i18n key (#2542)
settings-profile-public-bio = Public Bio

# semantic i18n key (#2542)
settings-profile-public-url-placeholder = your-name.generalbots.org

# semantic i18n key (#2542)
settings-profile-language-region = Language & region

# semantic i18n key (#2542)
settings-profile-public-url-hint-html = The page will be live at <strong><span id="public-profile-url-hint" data-i18n="settings-profile-public-url-placeholder">your-name.generalbots.org</span></strong> once the General Bots home-site relay ships (#1867).

# semantic i18n key (#2542)
settings-security-directory-note = Identity, sessions and 2FA live in the Zitadel directory

# semantic i18n key (#2542)
settings-security-open-directory-security = Open directory security

# semantic i18n key (#2542)
settings-security-connected-devices = Connected Devices

# semantic i18n key (#2542)
settings-security-connected-devices-desc = Devices that have accessed your account

# semantic i18n key (#2542)
settings-security-local-data = Local data

# semantic i18n key (#2542)
settings-security-local-data-desc = This installation keeps everything on this device.

# semantic i18n key (#2542)
settings-security-export-local-data = Export local data

# semantic i18n key (#2542)
settings-security-export-local-data-desc = Download the data stored on this device

# semantic i18n key (#2542)
settings-security-clear-local-cache = Clear local cache

# semantic i18n key (#2542)
settings-security-clear-local-cache-desc = Removes cached settings and drafts stored in this browser profile

# semantic i18n key (#2542)
settings-security-clear-cache-action = Clear cache

# semantic i18n key (#2542)
settings-shared-links-subtitle = Choose how a public file link reaches the person you send it to

# semantic i18n key (#2542)
settings-shared-links-loopback-warning-title = Only this machine is reachable right now.

# semantic i18n key (#2542)
settings-shared-links-loopback-warning-body-html = A copied loopback link (<code>127.0.0.1</code>) opens on the recipient's own computer, not yours. Use a LAN address, set a domain, or wait for the free General Bots address before sharing a link outside this machine.

# semantic i18n key (#2542)
settings-shared-links-lan-title = This network

# semantic i18n key (#2542)
settings-shared-links-lan-desc = Anyone on your Wi-Fi or wired network

# semantic i18n key (#2542)
settings-shared-links-on-this-computer = On this computer

# semantic i18n key (#2542)
settings-shared-links-on-this-network = On this network

# semantic i18n key (#2542)
settings-shared-links-lan-detail = Works for anyone on the same network. Nothing leaves this machine.

# semantic i18n key (#2542)
settings-shared-links-lan-config-title = Configuration

# semantic i18n key (#2542)
settings-shared-links-lan-config-body-html = This tier is on by default: the local server listens on all interfaces. Open the network address from another device on the same network. If device-to-device traffic is blocked, allow inbound TCP port <code id="shared-links-port">8080</code> on this computer.

# semantic i18n key (#2542)
settings-shared-links-lan-config-local-html = If the <code>.local</code> name does not resolve, use this computer's IPv4 address instead (for example <code>http://192.168.1.20:8080</code>).

# semantic i18n key (#2542)
settings-shared-links-domain-title = My own domain

# semantic i18n key (#2542)
settings-shared-links-domain-desc = A name you control, reachable from the internet

# semantic i18n key (#2542)
settings-shared-links-public-link = Public link

# semantic i18n key (#2542)
settings-shared-links-domain-not-configured = Not configured — new links fall back to the local address above.

# semantic i18n key (#2542)
settings-shared-links-domain-config-title = How to configure

# semantic i18n key (#2542)
settings-shared-links-domain-config-dns-html = Point a DNS record (for example <code>files.example.com</code>) at this computer's public address.

# semantic i18n key (#2542)
settings-shared-links-domain-config-url-html = Set the public base URL for this installation: <pre><code>PUBLIC_BASE_URL=https://files.example.com</code></pre> or, in the product file: <pre><code>public_base_url=https://files.example.com</code></pre>

# semantic i18n key (#2542)
settings-shared-links-domain-config-restart = Restart the desktop app so new share links are composed against the domain; existing links keep their tokens.

# semantic i18n key (#2542)
settings-shared-links-domain-config-hint = The registrar token stays in the local vault and is never written to logs. Automatic DNS refresh and TLS certificates are part of the domain-provider work tracked in this issue; until that lands, keep the record updated at your registrar.

# semantic i18n key (#2542)
settings-shared-links-free-title = Free General Bots address

# semantic i18n key (#2542)
settings-shared-links-free-desc = A name provided by General Bots, no domain needed

# semantic i18n key (#2542)
settings-shared-links-free-unavailable-html = Not available yet. The desktop works offline, so this tier needs a free DNS zone and a relay for machines that cannot be reached directly. It ships disabled with the <code>botpublic</code> work tracked in #1864.

# semantic i18n key (#2542)
settings-shared-links-free-what-title = What it will do

# semantic i18n key (#2542)
settings-shared-links-free-what-body-html = The free account will hold a name — not your files. When a machine cannot be reached directly, the managed relay (<code id="shared-links-relay-host">peer.generalbots.org</code>) keeps only an encrypted copy of the shared file and that copy expires. No manual relay configuration is required.

# semantic i18n key (#2542)
settings-shared-links-peer-title = Master peer switch

# semantic i18n key (#2542)
settings-shared-links-peer-desc-html = Stop all P2P traffic for this installation. Offline mode keeps every connection on <code>127.0.0.1</code>.

# semantic i18n key (#2542)
settings-shared-links-offline-default = Offline (default)

# semantic i18n key (#2542)
settings-shared-links-p2p-opt-in = P2P (opt-in)

# semantic i18n key (#2542)
settings-shared-links-relay-hint-html = The relay endpoint is fixed (<code>peer.generalbots.org</code>). Changing this setting may require restarting the app to take full effect.

# semantic i18n key (#2542)
settings-shared-links-check-again = Check again

# semantic i18n key (#2542)
settings-admin-bots-loading = Loading bots...

# semantic i18n key (#2542)
settings-admin-domain-configuration = Domain Configuration

# semantic i18n key (#2542)
settings-admin-audit-loading = Loading audit log...

# semantic i18n key (#2542)
settings-updates-subtitle = Check the latest GitHub release and jump to the official download

# semantic i18n key (#2542)
settings-updates-desktop-title = Desktop updates

# semantic i18n key (#2542)
settings-updates-desktop-checks-html = The app checks <code>github.com/generalbots/generalbots/releases/latest</code>. Installing is always opt-in.

# semantic i18n key (#2542)
settings-updates-check-startup = Check for updates at startup

# semantic i18n key (#2542)
settings-updates-check-startup-desc = Runs at most once a day and only shows a notification. You can still use “Check now” when this is off.

# semantic i18n key (#2542)
settings-updates-current-version = Current version

# semantic i18n key (#2542)
settings-updates-never = Never

# semantic i18n key (#2542)
settings-updates-check-now = Check now

# semantic i18n key (#2542)
settings-updates-view-release-notes = View release notes

# semantic i18n key (#2542)
settings-updates-not-checked-yet = Not checked yet.

# semantic i18n key (#2542)
settings-versions-subtitle = Component versions and update availability

# semantic i18n key (#2542)
settings-versions-loading = Loading version information...

# semantic i18n key (#2542)
settings-ledger-description = Every terminal and browser action, with the consent decision, outcome and screenshot path.

# semantic i18n key (#2542)
settings-ledger-actions-title = Actions ledger

# semantic i18n key (#2542)
settings-ledger-tab-terminal = Terminal

# semantic i18n key (#2542)
settings-ledger-tab-browser = Browser

# semantic i18n key (#2542)
settings-ledger-tab-budgets = Budgets

# semantic i18n key (#2542)
settings-ledger-tab-ttft = TTFT

# semantic i18n key (#2542)
settings-integrations-subtitle = Connect external services and manage API access

# semantic i18n key (#2542)
settings-open-integrations = Open Integrations

# semantic i18n key (#2542)
settings-personal-access-tokens = Personal access tokens

# semantic i18n key (#2542)
settings-personal-tokens-desc = Tokens you mint for calling the API as yourself (<code>gb_…</code>). Model-provider keys live in the Integrations app.

# semantic i18n key (#2542)
settings-no-api-keys = No API keys created yet

# semantic i18n key (#2542)
settings-webhooks-title = Webhooks

# semantic i18n key (#2542)
settings-no-webhooks = No webhooks configured

# semantic i18n key (#2542)
settings-full-catalog = Full catalog

# semantic i18n key (#2542)
settings-loading-mcp-servers = Loading MCP servers

# semantic i18n key (#2542)
settings-agent-skills = Agent Skills

# semantic i18n key (#2542)
settings-import-skill = Import skill

# semantic i18n key (#2542)
settings-loading-skills = Loading skills

# semantic i18n key (#2542)
settings-ai-accounts-subtitle = Claude Code and ChatGPT plan subscriptions are per user. Credentials stay in your agent VM or the local vault; only the status and display metadata are stored here.

# semantic i18n key (#2542)
settings-subscription-accounts = Subscription accounts

# semantic i18n key (#2542)
settings-ai-accounts-desc = Connect an account, then assign it to a Vibe agent type

# semantic i18n key (#2542)
settings-loading-ai-accounts = Loading AI accounts…

# semantic i18n key (#2542)
settings-vibe-subtitle = Assign an LLM provider to each Vibe agent type. Providers are defined in Integrations → Model providers (BYOK) or the Cloud store.

# semantic i18n key (#2542)
settings-agent-llm-providers = Agent LLM Providers

# semantic i18n key (#2542)
settings-agent-llm-providers-desc = Each agent type routes its runs to the provider selected here for this branch

# semantic i18n key (#2542)
settings-loading-agent-providers = Loading agent providers…

# semantic i18n key (#2542)
settings-privacy-subtitle = Manage your data and privacy settings

# semantic i18n key (#2542)
settings-privacy-preferences = Privacy Preferences

# semantic i18n key (#2542)
settings-profile-visibility = Profile Visibility

# semantic i18n key (#2542)
settings-profile-visibility-desc = Who can see your profile

# semantic i18n key (#2542)
settings-contacts-only = Contacts Only

# semantic i18n key (#2542)
settings-online-status = Online Status

# semantic i18n key (#2542)
settings-online-status-desc = Show when you're online

# semantic i18n key (#2542)
settings-read-receipts = Read Receipts

# semantic i18n key (#2542)
settings-read-receipts-desc = Let others know when you've read messages

# semantic i18n key (#2542)
settings-your-data = Your Data

# semantic i18n key (#2542)
settings-export-data = Export Data

# semantic i18n key (#2542)
settings-export-data-desc = Download all your data in a portable format

# semantic i18n key (#2542)
settings-delete-account-title = Permanently delete your account and all associated data

# semantic i18n key (#2542)
settings-delete-my-account = Delete My Account

# semantic i18n key (#2542)
settings-memory-subtitle = What the assistant remembers about you across conversations

# semantic i18n key (#2542)
settings-remembered-facts = Remembered facts

# semantic i18n key (#2542)
settings-memory-facts-desc = These facts are sent with every message so answers stay consistent between conversations. Remove anything you no longer want the assistant to use.

# semantic i18n key (#2542)
settings-memory-all-scopes = All scopes

# semantic i18n key (#2542)
settings-memory-global-all-bots = Global (all bots)

# semantic i18n key (#2542)
settings-memory-bot-specific = Bot-specific

# semantic i18n key (#2542)
settings-default-scope = default

# semantic i18n key (#2542)
settings-manage-memories = Manage memories

# semantic i18n key (#2542)
settings-manage-memories-desc = Memories are created by the assistant. Edit or remove them here.

# semantic i18n key (#2542)
settings-no-plan-info = No plan information is managed by this deployment.

# semantic i18n key (#2542)
settings-manage-plan-on-cloud = Manage plan on Cloud

# semantic i18n key (#2542)
settings-add-method = Add Method

# semantic i18n key (#2542)
settings-no-payment-method = No payment method on file.

# semantic i18n key (#2542)
settings-add-method-on-cloud = Add Method on Cloud

# semantic i18n key (#2542)
settings-no-invoices = No invoices for this account.

# semantic i18n key (#2542)
settings-view-invoices-on-cloud = View invoices on Cloud

# semantic i18n key (#2542)
settings-skills-placeholder =
    Paste a SKILL.md package:
    ---
    name: deploy-check
    description: Verify a deploy before announcing it
    ---
    1. …
settings-memory-search-placeholder = Search memories…
settings-notification-recipient-placeholder = you@example.com

# semantic i18n key (#2542)
settings-profile-placeholder-display-name = John Doe
settings-profile-placeholder-username = johndoe
settings-profile-placeholder-email = john@example.com
settings-profile-placeholder-public-display-name = Your Name
settings-profile-placeholder-public-bio = A short description visible on your public profile...
settings-profile-placeholder-phone = +1 (555) 000-0000
settings-profile-placeholder-location = City, Country
settings-profile-placeholder-website = https://example.com

# semantic i18n key (#2542)
settings-placeholder-dns-hostname = mybot.example.com
settings-placeholder-dns-target = 192.168.1.1 or target.domain.com
settings-placeholder-api-key-name = Production key
settings-placeholder-api-key-scopes = user, admin
settings-placeholder-api-key-expiry = 90
settings-placeholder-webhook-url = https://example.com/hooks/gb
settings-placeholder-webhook-events = user.created, order.completed
