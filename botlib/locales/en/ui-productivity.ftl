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

# ── #2307 Tasks goal/monitor detail panel ──────────────────────────
tasks-detail-untitled-goal = Untitled Goal
tasks-detail-goal-progress = Goal progress
tasks-detail-current-value = Current value
tasks-detail-target-value = Target value
tasks-detail-percent = Progress
tasks-detail-no-actions = No actions yet
tasks-detail-pending-info = Missing information
tasks-detail-scheduler-info = Scheduler information
tasks-detail-monitor-info = Monitor information
tasks-detail-unavailable = Unavailable
tasks-detail-load-failed = Unable to load goal details


# ── #2307 Database insert/confirm dialogs ──────────────────────────
db-insert-row = Insert row
db-insert-into = Insert into
db-provide-value = Provide at least one value
db-insert-failed = Insert failed
db-confirm-title = Confirm
db-confirm-accept = Confirm
db-confirm-delete = Delete this row?
db-confirm-drop = Drop table
db-delete-failed = Delete failed
db-drop-failed = Drop failed

# Tasks detail view (#2307)
tasks-detail-type-goal = Goal
tasks-detail-status-pending = Pending
tasks-detail-pending-title = Pending items
tasks-detail-required-reason = A reason is required.
tasks-detail-type-scheduler = Scheduler
tasks-detail-type-monitor = Monitor
tasks-detail-last-action = Last action
tasks-detail-next-run = Next run
tasks-detail-file = File
tasks-detail-interval = Interval
tasks-detail-last-check = Last check
tasks-detail-last-value = Last value
tasks-detail-progress = Progress
tasks-detail-app = App
tasks-detail-field = Field
tasks-detail-reason = Reason
tasks-detail-cron = Cron

# Database dialog copy (#2307)
db-error = Error
db-database = Database
db-empty-query = Execute a query to see results
db-insert = Insert
db-select-table = Select a table
tasks-load-error = Could not load tasks.
tasks-sign-in-to-load = Sign in to load tasks.
calendar-collapse-sidebar = Collapse sidebar
calendar-prev-month = Previous month
calendar-next-month = Next month
calendar-prev-period = Previous period
calendar-next-period = Next period
calendar-add-calendar = Add calendar
paper-toggle-sidebar = Toggle sidebar
paper-insert-ai-response = Insert AI response
paper-replace-selection = Replace selection
paper-highlight = Highlight
paper-align-left = Align left
paper-align-center = Align center
paper-align-right = Align right
paper-insert-link = Insert link
paper-insert-image = Insert image
paper-insert-table = Insert table
paper-bold = Bold
paper-italic = Italic
paper-underline = Underline
paper-strikethrough = Strikethrough
products-view-grid = Grid view
products-view-list = List view
sheet-freeze-panes = Freeze panes

# semantic i18n key (#2542)
db-schema-title = 🗄️ Database Schema

# semantic i18n key (#2542)
db-loading-tables = Loading tables...

# semantic i18n key (#2542)
db-query-builder = Query Builder

# semantic i18n key (#2542)
db-run-query = Run Query

# semantic i18n key (#2542)
db-select-table-hint = Select a table to view data

# semantic i18n key (#2542)
db-prev-page = ◀ Prev

# semantic i18n key (#2542)
db-page-info-default = Page 1 of 1

# semantic i18n key (#2542)
db-next-page = Next ▶

# semantic i18n key (#2542)
research-searching-sources = Searching sources...

# semantic i18n key (#2542)
research-suggestion-latest-ai = Latest AI developments

# semantic i18n key (#2542)
research-suggestion-quantum = Quantum computing explained

# semantic i18n key (#2542)
research-suggestion-software-architecture = Software architecture

# semantic i18n key (#2542)
research-suggestion-ai-productivity = AI productivity tools

# semantic i18n key (#2542)
research-related-heading = Related

# semantic i18n key (#2542)
research-loading = Loading Research…

# semantic i18n key (#2542)
research-service-unavailable = Research service unavailable

# semantic i18n key (#2542)
calendar-mini-month-placeholder = January 2025

# semantic i18n key (#2542)
calendar-upcoming = Upcoming

# semantic i18n key (#2542)
calendar-import-export = Import / Export

# semantic i18n key (#2542)
calendar-popup-event-title = Event Title

# semantic i18n key (#2542)
calendar-popup-location-placeholder = Conference Room A

# semantic i18n key (#2542)
calendar-popup-event-description = Event description

# semantic i18n key (#2542)
calendar-personal = Personal

# semantic i18n key (#2542)
calendar-import-ical-title = Import iCal File

# semantic i18n key (#2542)
calendar-select-ics-file = Select .ics file

# semantic i18n key (#2542)
calendar-import-to-calendar-label = Import to calendar

# semantic i18n key (#2542)
calendar-default-calendar = Default Calendar

# semantic i18n key (#2542)
calendar-loading = Loading calendar…

# semantic i18n key (#2542)
calendar-unavailable = Calendar unavailable

# semantic i18n key (#2542)
tasks-progress-status = STATUS

# semantic i18n key (#2542)
tasks-progress-sample-title = Implement User Authentication System

# semantic i18n key (#2542)
tasks-progress-runtime = Runtime:

# semantic i18n key (#2542)
tasks-progress-sample-action = Choose Token Expiration Strategy

# semantic i18n key (#2542)
tasks-progress-estimated = Estimated:

# semantic i18n key (#2542)
tasks-progress-sample-decision = Will need approval for security configuration

# semantic i18n key (#2542)
tasks-progress-log = PROGRESS LOG

# semantic i18n key (#2542)
tasks-progress-terminal-live = TERMINAL (LIVE AGENT ACTIVITY)

# semantic i18n key (#2542)
tasks-progress-processed = Processed:

# semantic i18n key (#2542)
tasks-progress-data-points = data points

# semantic i18n key (#2542)
tasks-progress-sample-speed = ~8 sources/min

# semantic i18n key (#2542)
tasks-progress-sample-insight = Synthesizing competitive insights...

# semantic i18n key (#2542)
tasks-progress-sample-pricing = Cross-referencing pricing data...

# semantic i18n key (#2542)
tasks-progress-view-details = View Details ▸

# semantic i18n key (#2542)
tasks-create-run = Create & Run

# semantic i18n key (#2542)
tasks-loading-tasks = Loading tasks...

# semantic i18n key (#2542)
tasks-select-task = Select a task

# semantic i18n key (#2542)
tasks-select-task-desc = Click on a task from the list to view details

# semantic i18n key (#2542)
tasks-bot-database-label = Bot Database:

# semantic i18n key (#2542)
tasks-shared-resources-label = Shared Resources:

# semantic i18n key (#2542)
tasks-starting = Starting...

# semantic i18n key (#2542)
tasks-create-new-intent = Create New Intent

# semantic i18n key (#2542)
tasks-intent-accomplish-label = What would you like to accomplish?

# semantic i18n key (#2542)
tasks-execution-mode = Execution Mode

# semantic i18n key (#2542)
tasks-mode-automatic = Automatic

# semantic i18n key (#2542)
tasks-mode-supervised = Supervised

# semantic i18n key (#2542)
tasks-mode-manual-approval = Manual Approval

# semantic i18n key (#2542)
tasks-make-decision = Make Decision

# semantic i18n key (#2542)
tasks-decision-required = Decision Required

# semantic i18n key (#2542)
tasks-loading-decision-details = Loading decision details...

# semantic i18n key (#2542)
tasks-skip-for-now = Skip for Now

# semantic i18n key (#2542)
tasks-confirm-decision = Confirm Decision

# semantic i18n key (#2542)
tasks-due-soon = Due soon

# semantic i18n key (#2542)
tasks-autotask-items = // AUTOTASK ITEMS

# semantic i18n key (#2542)
tasks-edit-task = Edit Task

# semantic i18n key (#2542)
tasks-assignee-user-id = Assignee (user ID)

# semantic i18n key (#2542)
tasks-parent-task-subtask-of = Parent task (subtask of…)

# semantic i18n key (#2542)
tasks-reopen-action = ↩ Reopen

# semantic i18n key (#2542)
tasks-complete-action = ✓ Complete

# semantic i18n key (#2542)
tasks-delete-action = 🗑 Delete

# semantic i18n key (#2542)
db-schema-browser = Schema Browser

# semantic i18n key (#2542)
db-loading-schema = Loading schema...

# semantic i18n key (#2542)
db-new-table = + New Table

# semantic i18n key (#2542)
db-previous = ← Previous

# semantic i18n key (#2542)
db-next = Next →

# semantic i18n key (#2542)
db-sql-console = SQL Console

# semantic i18n key (#2542)
db-execute-query = Execute (Ctrl+Enter)

# semantic i18n key (#2542)
db-explain = EXPLAIN

# semantic i18n key (#2542)
db-create-new-table = Create New Table

# semantic i18n key (#2542)
db-table-name = Table Name

# semantic i18n key (#2542)
db-columns = Columns

# semantic i18n key (#2542)
db-not-null = NOT NULL

# semantic i18n key (#2542)
db-type-varchar-255 = VARCHAR(255)

# semantic i18n key (#2542)
db-type-text = TEXT

# semantic i18n key (#2542)
db-type-integer = INTEGER

# semantic i18n key (#2542)
db-type-bigint = BIGINT

# semantic i18n key (#2542)
db-type-decimal = DECIMAL

# semantic i18n key (#2542)
db-type-boolean = BOOLEAN

# semantic i18n key (#2542)
db-type-date = DATE

# semantic i18n key (#2542)
db-type-timestamptz = TIMESTAMPTZ

# semantic i18n key (#2542)
db-type-jsonb = JSONB

# semantic i18n key (#2542)
db-add-column = + Add Column

# semantic i18n key (#2542)
db-create-table = Create Table

# semantic i18n key (#2542)
db-edit-row = Edit Row

# semantic i18n key (#2542)
db-import-csv = Import CSV

# semantic i18n key (#2542)
db-select-csv-file = Select CSV file

# semantic i18n key (#2542)
analytics-ask-placeholder = Ask about your analytics data...
tasks-intent-description-placeholder = Describe what you want to create or automate...
tasks-assignee-placeholder = UUID of the assignee (optional)
tasks-parent-placeholder = UUID of parent task (optional)

# semantic i18n key (#2542)
db-placeholder-search-tables = Search tables...
db-aria-close-sql-console = Close SQL console
db-placeholder-sql-query = SELECT * FROM table_name WHERE condition...
db-aria-sql-query = SQL query
db-placeholder-table-name-example = my_table
db-placeholder-column-id = id
db-placeholder-column-name-example = column_name
