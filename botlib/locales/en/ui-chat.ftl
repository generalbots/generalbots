# =============================================================================
# General Bots — en — Chat
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

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

# Additional Chat translations
chat-camera = Camera
chat-camera-title = Take a photo
chat-camera-capture = Take photo
chat-camera-cancel = Cancel
chat-camera-waiting = Waiting for the camera to start…
chat-camera-waiting-frames = Permission granted; waiting for video frames
chat-camera-ready = Camera ready. Take the photo or cancel.
chat-camera-started = Camera started — tap Take photo.
chat-camera-sent = Photo sent.
chat-camera-denied = Camera permission was denied. Allow camera access for the desktop app in your system settings to take photos.
chat-camera-missing = No camera was found on this device.
chat-camera-no-device = no video device
chat-camera-device = video device
chat-camera-no-frames = The camera preview did not start. Choose a photo from the system picker instead.
chat-camera-timeout = The camera did not answer. Choose a photo from the system picker instead.
chat-camera-error = Could not start the camera.
chat-camera-unsupported = This desktop app does not provide camera access.
voice-mode-title = Voice conversation
voice-mode-listening = Listening… speak naturally
voice-mode-thinking = Thinking…
voice-mode-speaking = Speaking…
voice-mode-idle = Voice conversation
voice-mode-end = End voice conversation
chat-message-placeholder = Message...

# Chat message controls and notices
chat-copy-message = Copy message
chat-copied = Copied
chat-message-copied = Message copied
chat-copy-failed = Copy failed
chat-listen-tts = Listen (TTS)
chat-listen-message = Listen to this message
chat-image-unavailable = image unavailable
chat-open = Open
chat-expand = Expand
chat-collapse = Collapse
chat-preview = Preview
chat-consent-affected-count = Affected records
chat-close-preview = Close preview
chat-preview-load-failed = Could not load the preview.
chat-show-more = Show more
chat-show-less = Show less
chat-tool = tool
chat-param = param
chat-params = params
chat-tool-result = Tool result
chat-retry = Retry
chat-open-as-document = Open as document
chat-show-more-history = show more…
chat-disconnected-reconnecting = Chat disconnected. Reconnecting...
chat-connect-retrying = Could not reach the chat server. Retrying...
chat-connection-lost = Connection lost. Trying to reconnect...
chat-attachment-sent = Attachment sent: 
chat-attachments-sent = Attachments sent: 
chat-message-not-delivered = Message not delivered — reconnecting
chat-attachment-read-failed = Could not read an attachment. Attach it again.
chat-attachment-queued = Connection is back shortly — attachment queued
chat-reconnecting-queued = Reconnecting — your message will be sent when the connection is back.
chat-bot-private-signin = This bot is private — sign in to continue
chat-approved-plan = Approved plan: 
chat-retrying-attachment = Retrying attachment
chat-expanded-preview = Expanded preview

# #2362 — chat code/artifact control labels
chat-code-show-all = Show all
chat-code-lines = lines
chat-artifact-save-drive = Save to Drive
chat-artifact-download-svg = Download SVG
chat-artifact-download-png = Download PNG
chat-artifact-open-canvas = Open in Canvas
chat-artifact-folder = Folder (optional)
chat-artifact-kind-markdown = Markdown document
chat-artifact-kind-html = HTML page
chat-artifact-kind-code = Code
chat-artifact-words = words

# #2362 — attachment action label
chat-open-in-app = Open in app
# Assistant-look chat: transcript export and answer feedback
chat-export-actions = Conversation actions
chat-export-md = Export conversation as Markdown
chat-export-print = Print or save as PDF
chat-export-done = Conversation exported
chat-export-empty = Nothing to export yet
chat-export-untitled = Conversation
chat-export-user = You
chat-export-assistant = Assistant
chat-feedback-label = Answer feedback
chat-feedback-good = Good answer
chat-feedback-bad = Bad answer
chat-feedback-comment = What could be better?
chat-feedback-send = Send
chat-staged-files = Attached files
chat-staged-close = Close preview

# Chat engineer status strip (#2342)
chat-status-bot = Bot
chat-status-session = Session
chat-status-model = Model
chat-status-state = State
chat-status-no-session = —
chat-status-copy-session = Copy session id
chat-status-copied = Copied
chat-status-idle = Idle
chat-status-streaming = Streaming
chat-status-thinking = Thinking
chat-status-error = Error
chat-status-stopped = Stopped

chat-response-ready = Response ready
chat-response-started = Assistant is responding
chat-tool-result-ready = Tool result available

# semantic i18n key (#2542)
chat-no-suggestions-available = No suggestions available

# semantic i18n key (#2542)
chat-start-conversation = Start a conversation

# semantic i18n key (#2542)
chat-empty-hint = Type a message below or use voice input

# semantic i18n key (#2542)
chat-no-conversations-yet = No conversations yet

# semantic i18n key (#2542)
chat-context-label = Context:

# semantic i18n key (#2542)
chat-select-context = Select context...

# semantic i18n key (#2542)
chat-no-contexts-configured = No contexts configured

# semantic i18n key (#2542)
projector-title = Content Viewer

# semantic i18n key (#2542)
projector-loading-content = Loading content...

# semantic i18n key (#2542)
projector-slide-info = Slide 1 of 1

# semantic i18n key (#2542)
projector-code-info = code.rs

# Projector viewer control (#2542)
chat-projector-rotate = Rotate

# Chat header / app chrome (#2542)
chat-conversation-clear = Clear context
chat-conversation-cleared = Conversation context cleared
chat-conversation-copy-link = Copy link
chat-conversation-delete = Delete
chat-conversation-delete-confirm = Delete this conversation?
chat-conversation-delete-failed = Failed to delete conversation
chat-conversation-delete-title = Delete conversation
chat-conversation-delete-unavailable = Confirmation dialog unavailable
chat-conversation-deleted = Conversation deleted
chat-conversation-export-json = Export transcript (.json)
chat-conversation-export-md = Export transcript (.md)
chat-conversation-exported = Conversation exported
chat-conversation-link-copied = Conversation link copied
chat-conversation-link-copy-failed = Failed to copy link
chat-conversation-menu = Conversation menu
chat-conversation-rename = Rename
chat-conversation-rename-failed = Failed to rename conversation
chat-conversation-renamed = Conversation renamed
chat-conversation-title = Conversation title
chat-conversation-untitled = Untitled conversation

# -----------------------------------------------------------------------------
# #2328 — loading / empty / error state copy
# -----------------------------------------------------------------------------
chat-empty-title = Start the conversation
chat-empty-desc = Type a message below, or pick a suggestion when your bot offers one.
chat-empty-action = Write the first message

# -----------------------------------------------------------------------------
# #2342 — chat engineer UX: slash commands, help, inspect/copy-markdown
# -----------------------------------------------------------------------------
chat-cmd-menu-title = Commands
chat-cmd-help = Show keyboard shortcuts
chat-cmd-new = Start a new conversation
chat-cmd-clear = Clear this conversation
chat-cmd-stop = Stop generating
chat-stop = Stop
chat-cmd-failed = Command failed
chat-help = Keyboard shortcuts
chat-inspect = Inspect
chat-inspect-title = Message details
chat-inspect-sender = Sender
chat-inspect-id = Message id
chat-inspect-no-id = none
chat-inspect-length = Characters
chat-copy-markdown = Copy as Markdown
chat-copied-markdown = Copied as Markdown

# Model attribution on each bot answer
chat-model-written-by = Answer written by

# Assistant parity in chat
assistant-parity-block-allow = Allow
assistant-parity-block-allowed = Allowed
assistant-parity-block-api-command = API command
assistant-parity-block-attachment-read = Attachment read
assistant-parity-block-chars = characters
assistant-parity-block-command = Command
assistant-parity-block-decision-failed = Could not record the decision
assistant-parity-block-denied = Denied
assistant-parity-block-deny = Deny
assistant-parity-block-detail = Detail
assistant-parity-block-drive-list = Drive listing
assistant-parity-block-drive-read = Drive file read
assistant-parity-block-drive-search = Drive search
assistant-parity-block-elapsed = Elapsed
assistant-parity-block-error = Error
assistant-parity-block-file = File
assistant-parity-block-kb-search = Knowledge search
assistant-parity-block-notice = Notice
assistant-parity-block-path = Path
assistant-parity-block-permission = Permission requested
assistant-parity-block-query = Query
assistant-parity-block-read = Read
assistant-parity-block-reason = Reason
assistant-parity-block-result = Result
assistant-parity-block-retry = Retry
assistant-parity-block-seconds = s
assistant-parity-block-size = Size
assistant-parity-block-status-done = Done
assistant-parity-block-status-failed = Failed
assistant-parity-block-status-needs-you = Needs your input
assistant-parity-block-status-running = Running
assistant-parity-block-step = Step
assistant-parity-block-thinking = Thinking
assistant-parity-block-tool = Tool
assistant-parity-block-tool-run = Tool run
assistant-parity-block-total = Total
assistant-parity-block-truncated = truncated
assistant-parity-block-type = Type
assistant-parity-block-yes = Yes
chat-parity-attach-title = Attachment preview
chat-parity-attach-unavailable = Content unavailable - use Download to open this file.
chat-parity-attach-download-failed = Could not download the attachment.
chat-parity-model-tiers = Auto & tiers
chat-parity-model-accounts = Provider accounts
chat-parity-model-local = Local & free
chat-drive-parity-attaching = Attaching...
chat-drive-parity-cancel = Cancel
chat-drive-parity-delete = Delete
chat-drive-parity-deleted = Deleted
chat-drive-parity-delete-failed = Could not delete the file.
chat-drive-parity-delete-question = Delete this file?
chat-drive-parity-delete-title = Delete file
chat-drive-parity-deleting = Deleting...
chat-drive-parity-download = Download
chat-drive-parity-download-failed = Could not download the file.
chat-drive-parity-downloading = Downloading...
chat-drive-parity-more = More actions
chat-drive-parity-new-name = New name
chat-drive-parity-preview = Preview
chat-drive-parity-preview-failed = Could not preview the file.
chat-drive-parity-previewing = Opening preview...
chat-drive-parity-rename = Rename
chat-drive-parity-renamed = Renamed
chat-drive-parity-rename-failed = Could not rename the file.
chat-drive-parity-rename-title = Rename file
chat-drive-parity-renaming = Renaming...
chat-drive-parity-unavailable = This action is unavailable in this deployment.
chat-drive-parity-use-in-chat = Use in conversation
chat-parity-agent-does = The agent does it
chat-parity-back = Back
chat-parity-checked-now = Checked just now
chat-parity-close = Close
chat-parity-coming-soon = Coming soon
chat-parity-cost-contract = Per your contract
chat-parity-cost-free = Free
chat-parity-cost-free-limit = Free, with usage limits
chat-parity-cost-sub = Your subscription
chat-parity-cuda-no = No suitable GPU detected
chat-parity-cuda-yes = Graphics card detected
chat-parity-depends-agent = Depends on the agent
chat-parity-driver = Driver
chat-parity-engine = Engine in use
chat-parity-feature-docs = Documents
chat-parity-feature-ocr = Text in images
chat-parity-feature-photos = Photos and charts
chat-parity-feature-web = Web search
chat-parity-features = Features on
chat-parity-gains = Your gains
chat-parity-go-agent = Goes to the agent's provider
chat-parity-go-ext = Goes to an external provider
chat-parity-go-net = Stays in your organization's network
chat-parity-go-server = Goes to the server you set
chat-parity-guide = Not sure which to pick?
chat-parity-guide-sub = Answer two questions.
chat-parity-in-use = In use
chat-parity-load-failed = Could not load this information.
chat-parity-loading = Loading...
chat-parity-machine = This computer
chat-parity-machine-ok = This computer has: %d, %v GB
chat-parity-machine-slow = This computer has no suitable graphics card: it would be slow
chat-parity-need-acp = Install the agent and sign in to your account
chat-parity-need-free = Nothing
chat-parity-need-local = A graphics card with 6 GB or more and a 4 to 6 GB download
chat-parity-need-own = The server address and a key
chat-parity-no-read = Does not read
chat-parity-no-work = Does not work
chat-parity-none-found = None found
chat-parity-not-yet = Not yet
chat-parity-only-net = Only with a server on your network
chat-parity-optional = Optional, you turn it on
chat-parity-pick = Pick an option
chat-parity-priv-ext = Your conversations are sent to an external host (%h).
chat-parity-priv-local = Your conversations stay on this computer.
chat-parity-priv-net = Your conversations are sent to your organization's server (%h).
chat-parity-privacy = Privacy
chat-parity-q1 = Can what you write leave this computer?
chat-parity-q1-help = Think about confidential or personal information.
chat-parity-q1-no = No, it must stay here
chat-parity-q1-no-desc = Only local processing is acceptable.
chat-parity-q1-yes = Yes, it can leave
chat-parity-q1-yes-desc = An external service may process it.
chat-parity-q2-no = No, or I don't know
chat-parity-q2-no-desc = No company server or key available.
chat-parity-q2-open = Does your organization offer an AI server or key?
chat-parity-q2-open-help = Check with your IT area if unsure.
chat-parity-q2-secret = Does your organization allow external AI?
chat-parity-q2-secret-help = Some data policies forbid external services.
chat-parity-q2-secret-no-desc = External AI is not allowed.
chat-parity-q2-secret-yes-desc = External AI is allowed.
chat-parity-q2-yes = Yes
chat-parity-q2-yes-desc = A company server or key is available.
chat-parity-reads = Reads
chat-parity-ready = Ready
chat-parity-rec-free = Start with the free model: no setup needed. For confidential matters, prefer on this computer.
chat-parity-rec-local = Use on this computer: nothing leaves here and this machine has a graphics card.
chat-parity-rec-local-slow = This machine has no suitable graphics card for a local model. Use your organization's server inside its network.
chat-parity-rec-local-tight = A local model will work but may be slow with this graphics card.
chat-parity-rec-own = Use your organization's server or key: the contract defines cost and limits.
chat-parity-rec-own-net = Use your organization's server inside its network to keep the data private.
chat-parity-restart = Start over
chat-parity-retry = Retry
chat-parity-row-cost = Cost
chat-parity-row-docs = Documents
chat-parity-row-needs = You need
chat-parity-row-ocr = Text in images
chat-parity-row-offline = Without internet
chat-parity-row-photos = Photos and charts
chat-parity-row-privacy = Your text
chat-parity-row-web = Web search
chat-parity-rule = Rule
chat-parity-show-me = See how to turn it on
chat-parity-stay-here = Stays on this computer
chat-parity-step = Question %n of 2
chat-parity-sub-acp = Your agent subscription
chat-parity-sub-free = Ready to use
chat-parity-sub-local = Private and offline
chat-parity-sub-own = Your organization's
chat-parity-suggested = Suggested
chat-parity-system-memory = System memory
chat-parity-verdict-bad-gpu = No suitable graphics card
chat-parity-verdict-bad-gpu-long = No NVIDIA graphics card with CUDA was found. Local models would run on the processor, much slower: one answer can take minutes. To keep your text inside your network, prefer your organization's server.
chat-parity-verdict-capacity = This card runs models that need up to %v of video memory.
chat-parity-verdict-good = Comfortable
chat-parity-verdict-low-vram = Card with too little memory (%v)
chat-parity-verdict-low-vram-long = The card has %v of video memory; the smallest local model needs %m.
chat-parity-verdict-mid = Tight but usable
chat-parity-verdict-none = This computer could not be examined
chat-parity-video-memory = Video memory
chat-parity-where = Choose the engine
chat-parity-where-acp = Agent (ACP)
chat-parity-where-free = Free model
chat-parity-where-local = On this computer
chat-parity-where-own = Own server or key
chat-parity-where-sub = Compare the four ways to use the assistant. Click one to see details and turn it on.
chat-parity-works = Works
chat-parity-workshop = Assistant workshop
chat-parity-workshop-lead = Tune where and how your assistant thinks.
chat-parity-workshop-title = Assistant workshop
chat-parity-graphics-card = Graphics card

# --- #2921 missing keys added to every locale catalogue ---
chat-confirm-title = Confirm action
chat-confirm-unavailable = Confirmation unavailable
chat-copy = Copy
chat-delete = Delete
chat-delete-message-confirm = Delete this message from the conversation?
chat-delete-title = Delete message
chat-edit = Edit
chat-edit-unavailable = Composer unavailable
chat-message-actions = Message actions
chat-regenerate = Regenerate
chat-regenerate-no-source = No previous user message to regenerate
chat-regenerate-unavailable = Cannot regenerate this turn
chat-voice-capture-silent = No audio captured
chat-voice-duplex-audio = Audio
chat-voice-duplex-closed = Voice session closed
chat-voice-duplex-connect = Connect voice
chat-voice-duplex-connected = Voice connected
chat-voice-duplex-disabled = Voice disabled
chat-voice-duplex-error = Voice error
chat-voice-duplex-fallback = Falling back to text
chat-voice-duplex-live = Live voice
chat-voice-duplex-lost = Connection lost
chat-voice-duplex-timeout = Voice session timed out
chat-voice-duplex-unavailable = Voice unavailable
chat-voice-mic-failed = Microphone unavailable
chat-web-title = Web search
voice-mode-connecting = Connecting...
voice-mode-fallback-idle = Idle
voice-mode-fallback-listening = Listening
voice-mode-fallback-speaking = Speaking
voice-mode-fallback-thinking = Thinking
voice-mode-live-listening = Listening
voice-mode-live-ready = Ready
voice-mode-live-speaking = Speaking
