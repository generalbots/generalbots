# General Bots — desktop shell and setup wizard (#2089)

desktop-document-title = General Bots Desktop
sidebar-toggle = Toggle sidebar
sidebar-collapsed-chat = Chat
sidebar-collapsed-terminal = Terminal
sidebar-collapsed-drive = Drive
sidebar-collapsed-settings = Settings
sidebar-collapsed-account = Account
setup-welcome-title = Welcome to General Bots
setup-welcome-subtitle = Let's get you set up in a few quick steps
setup-welcome-quick-subtitle = Two quick steps to get started
setup-step-llm = LLM Provider
setup-step-profile = User Profile
setup-step-bot = Bot Config
setup-step-training = Training
setup-step-ai-model = AI Model
setup-step-your-name = Your Name
setup-choose-provider-title = Choose your LLM Provider
setup-choose-provider-subtitle = Select the AI model provider for your bot
setup-provider-openai-desc = GPT-4, GPT-3.5 models
setup-provider-nvidia-desc = NVIDIA NIM inference
setup-provider-anthropic-desc = Claude 3, Claude 3.5 models
setup-provider-local-desc = llama.cpp, Ollama, etc.
setup-profile-title = What describes you best?
setup-profile-subtitle = Select your profile type for personalized defaults
setup-profile-developer = Developer
setup-profile-developer-desc = Build and deploy bots, write BASIC scripts
setup-profile-business = Business
setup-profile-business-desc = Automate workflows, manage CRM
setup-profile-support = Support
setup-profile-support-desc = Customer service, ticket management
setup-bot-title = Configure your Bot
setup-bot-subtitle = Give your bot a name, purpose, and optional template
setup-bot-name = Bot Name
setup-bot-name-placeholder = My Assistant
setup-bot-purpose = Bot Purpose
setup-bot-purpose-placeholder = Describe what your bot should do...
setup-bot-template = Bot Template
setup-optional = (optional)
setup-template-scratch = — Start from scratch —
setup-name-title = What should we call you?
setup-name-subtitle = This name appears in your conversations
setup-name-label = Your Name
setup-name-placeholder = e.g. Maria
setup-training-title = Train your Bot
setup-training-subtitle = Drop .pdf, .xls, .doc files for the initial training. You will have access to Drive and can change it later.
setup-drop-text = Drop files here or click to select
setup-drop-hint = .pdf, .xls, .doc — up to 10MB each
setup-select-files = Select Files
setup-file-status-pending = Pending
setup-file-status-uploading = Uploading
setup-file-status-done = Done
setup-file-status-error = Error
setup-saving = Saving...
setup-complete-message = Setup complete! Welcome to General Bots.
setup-failed = Setup failed: {error}
setup-unknown-error = Unknown error
setup-back = Back
setup-next = Next
setup-finish = Finish
setup-skip = Skip Setup
agent-all-apps = All apps

# --- Desktop shell panels: versions, accessibility, sidebar, collaboration,
# --- updates, modals and usage (suite/js/*.js) — #2095

versions-component = Component
versions-version = Version
versions-status = Status
versions-action = Action
versions-up-to-date = Up to date
versions-unknown = Unknown
versions-check-update = Check Update
versions-checking = Checking...
versions-checked = Checked
versions-error = Error
versions-update-available = Update: { $version }
versions-could-not-load = Could not load version information.
versions-ensure-backend = Make sure the backend server is running.
versions-retry = Retry

a11y-chat-conversation = Chat conversation
a11y-chat-messages = Chat messages
a11y-send-message = Send message
a11y-type-message = Type your message
a11y-keyboard-shortcuts = Keyboard Shortcuts
a11y-shortcut-focus-chat = Focus chat input
a11y-shortcut-show-help = Show this help
a11y-shortcut-escape = Close modal / clear selection
a11y-shortcut-command-palette = Command palette
a11y-shortcut-privacy = Privacy settings
a11y-shortcut-scroll-history = Scroll chat history
a11y-shortcut-navigate = Navigate between elements
a11y-shortcut-activate = Activate focused element
a11y-shortcut-delete-files = Delete selected files (Drive)
a11y-shortcut-select-all = Select all files (Drive)
a11y-shortcut-clipboard = Copy / Cut / Paste files
a11y-shortcut-navigate-up = Navigate up in Drive
a11y-shortcut-rename = Rename selected file

sidebar-locked-conversations = Locked conversations
sidebar-no-conversations = No conversations yet
sidebar-sign-in-conversations = Sign in to see your conversations
sidebar-history-unavailable = History unavailable
sidebar-back-to-conversations = ← Back to conversations
sidebar-no-locked-conversations = No locked conversations
sidebar-locked-conversation = Locked conversation
sidebar-sign-in = Sign in
sidebar-add-pin = Add pin

collab-collaborators = Collaborators ({ $count })
collab-no-one-else = No one else is here

update-available-title = Update available — v{ $version }
update-now = Update now
update-later = Later
update-skip-version = Skip this version
update-platform = Platform: { $name }
update-asset = Asset: { $name }
update-check-failed = Update check failed: { $error }
update-view-release-notes = View release notes
update-release-notes-title = Release Notes — v{ $version }
update-no-release-notes = No release notes available.

modal-alert-title = Alert
modal-input-title = Input

external-save-to = Save to { $target }

usage-title = Usage & Cost
usage-last-days = Last { $days } days
usage-loading = Loading usage…
usage-per-model = Per model
usage-per-day = Per day
usage-model = Model
usage-tokens = Tokens
usage-messages = Messages
usage-cost = Cost
usage-day = Day
usage-days-7 = 7 days
usage-days-30 = 30 days
usage-days-90 = 90 days

pin-close = Close
pin-length-hint = 4–8 digits
pin-cancel = Cancel
pin-ok = OK

# --- Collaboration surfaces shared by every suite app: share dialog, activity
# --- trail, conflict banner and version history (suite/js/collab-*.js) — #2095

role-owner = owner
role-editor = Editor
role-commenter = Commenter
role-viewer = Viewer

share-request-failed = Request failed ({ $status })
share-warning-title = Shared links are not reachable yet
share-warning-body = This installation is only reachable on localhost, so a link here opens nowhere. Set up one of these first: enable P2P in Settings → Network, or point Shared Links at your own domain.
share-open-shared-links = Open Shared Links
share-open-network = Open Network
share-invite-people = Invite people
share-anyone-with-link = Anyone with the link
share-create-link = Create link
share-transfer-ownership = Transfer ownership
share-transfer = Transfer
share-new-owner-placeholder = new owner email
share-owner-suffix = · owner
share-no-people = No people added yet.
share-no-links = No public link yet.
share-copied = Copied
share-revoke = Revoke
share-enter-email = Enter an email
share-enter-new-owner = Enter the new owner's email
share-needs-base = Set up P2P or your own domain before creating a shared link.

activity-title = Activity
activity-request-failed = Request failed ({ $status })
activity-load-more = Load more
activity-loading = Loading…
activity-empty = No activity yet
activity-action-create = created
activity-action-edit = edited
activity-action-comment = commented on
activity-action-delete = deleted
activity-action-resolve = resolved a comment on
activity-action-reopen = reopened a comment on
activity-action-reaction = reacted to
activity-action-share = shared
activity-action-restore = restored
activity-action-transfer = transferred

conflict-title = Conflicting edits
conflict-dismiss = Dismiss
conflict-edited-concurrently = edited concurrently ({ $op })
conflict-keep-server = Keep server version
conflict-keep-mine = Keep my change
conflict-not-started = Conflict watch not started
conflict-request-failed = Request failed ({ $status })

versions-title = Version history
versions-loading = Loading…
versions-empty = No versions yet — the first save creates one.
versions-version-n = Version { $n }
versions-current-tag = (current)
versions-name = Name…
versions-restore-failed = Restore failed: { $error }
versions-rename-failed = Rename failed: { $error }
versions-restore-dialog-title = Restore version
versions-restore-confirm = Restore this version? A new current version will be created — nothing is lost.
versions-name-dialog-title = Name version
versions-name-prompt = Name this version (e.g. "v2 — approved"):
versions-request-failed = Request failed ({ $status })

# --- Desktop shell: snap picker, workspace tabs, virtual desktops, context
# --- menu, git panel, browser panel, widgets and data export (suite/js/*.js)

snap-title = Snap "{ $app }"
snap-layout-top-left = Top left
snap-layout-top = Top half
snap-layout-top-right = Top right
snap-layout-left = Left half
snap-layout-full = Maximize
snap-layout-right = Right half
snap-layout-bottom-left = Bottom left
snap-layout-bottom = Bottom half
snap-layout-bottom-right = Bottom right
snap-layout-third-left = Left third
snap-layout-two-thirds = Two thirds
snap-layout-third-right = Right third

wstabs-close-tab = Close tab
wstabs-new-tab = New workspace tab
wstabs-default-name = Workspace { $n }
wstabs-default-main = Main
wstabs-rename-title = Rename workspace
wstabs-name-prompt = Workspace name:

vdesk-default-name = Desktop { $n }
vdesk-title = Virtual Desktops
vdesk-new = New desktop
vdesk-rename-title = Rename desktop
vdesk-name-prompt = Desktop name:
vdesk-restore-title = Restore session
vdesk-restore-body = Reopen the { $count } window(s) from your last session on this desktop?
vdesk-restore-body-short = Restore your last session ({ $count } window(s))?

ctxmenu-search = Search DuckDuckGo
ctxmenu-search-prompt = Search DuckDuckGo:
ctxmenu-wikipedia = Wikipedia definition
ctxmenu-wikipedia-prompt = Wikipedia definition:

git-loading-status = Loading status...
git-no-changes = No changes
git-new-branch-prompt = New branch name:
git-loading-diff = Loading diff...
git-diff-failed = Failed to load diff
git-commit-message-required = Please enter a commit message
git-pushing = Pushing...
git-committed = Committed and pushed successfully
git-failed = Git operation failed

browser-session-active = Browser session { $id } active.
browser-init-failed = Failed to initialize browser
browser-init-first = Please initialize browser first
browser-capture = Capture
browser-no-session = No active session
browser-export-failed = Export failed
browser-no-actions = No actions recorded

widget-load-failed = Widget failed to load
widget-no-source = No widget source configured
widget-open-windowed = Open windowed
widget-pin-launcher = Pin to launcher

dataexport-title = Data Export & Backup
dataexport-desc = Export all your data including chat history, memory, drive metadata, and settings as a JSON backup file.
dataexport-export-all = Export All Data
dataexport-import = Import Backup
dataexport-exported = Backup exported successfully
dataexport-too-large = Backup rejected: the file is larger than 4 MB
dataexport-invalid = Invalid backup file: { $error }
dataexport-nothing = This backup contains no restorable settings
dataexport-unknown-date = unknown date
dataexport-restore-confirm = This will restore { $count } setting(s) from the backup of { $date }. Credentials and session tokens are never imported. Continue?
dataexport-restored = Backup restored ({ $count } items).
dataexport-rejected = { $count } entry(ies) rejected by the allowlist.
dataexport-reloading = Reloading...

comments-title = Comments
comments-request-failed = Request failed ({ $status })
comments-loading = Loading comments…
comments-load-failed = Failed to load comments: { $error }
comments-empty-open = No open comments. Use @name to mention someone.
comments-empty-resolved = No resolved comments. Use @name to mention someone.
comments-resolved-chip = Resolved
comments-resolve = Resolve
comments-reopen = Reopen
comments-resolved-announce = Comment resolved
comments-reopened-announce = Comment reopened
comments-resolve-failed = Resolve failed: { $error }
comments-added = Comment added
comments-post = Post
comments-post-failed = Comment failed: { $error }
comments-reaction-failed = Reaction failed: { $error }
comments-delete-failed = Delete failed: { $error }
comments-reply-failed = Reply failed: { $error }
comments-delete-title = Delete comment
comments-delete-confirm = Delete this comment?
comments-reply-title = Reply
comments-reply-prompt = Reply to comment:
comments-filter-open = Open
comments-filter-all = All
comments-filter-resolved = Resolved
comments-input-placeholder = Add a comment… use @name to mention
comments-someone = someone
comments-mention-announce = You were mentioned in a comment by { $author }
