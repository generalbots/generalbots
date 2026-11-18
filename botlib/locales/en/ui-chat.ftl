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

# semantic i18n key (#2542)
chat-no-suggestions-available = No suggestions available

# semantic i18n key (#2542)
chat-start-conversation = Start a conversation

# semantic i18n key (#2542)
chat-empty-hint = Type a message below or use voice input

# semantic i18n key (#2542)
chat-no-conversations-yet = No conversations yet
