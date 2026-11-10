# =============================================================================
# General Bots — en — Productivity apps (tasks, calendar, meet, mail, research, paper)
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

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
