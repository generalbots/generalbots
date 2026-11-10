# =============================================================================
# General Bots — en — Attendant (support queue)
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# =============================================================================
# ATTENDANT / CRM SCREENS
# =============================================================================
# -----------------------------------------------------------------------------
# CRM Disabled State
# -----------------------------------------------------------------------------
attendant-crm-disabled = CRM Features Not Enabled
attendant-crm-disabled-desc = The Attendant Console requires CRM features to be enabled for this bot. This allows human agents to receive and respond to conversations transferred from the bot.
attendant-crm-enable-instruction = To enable CRM features, add this line to your bot's
attendant-crm-config-file = config.csv
attendant-crm-create-attendant = Then create an
attendant-crm-attendant-file = attendant.csv
attendant-crm-configure-team = file to configure your team

# -----------------------------------------------------------------------------
# Queue Sidebar
# -----------------------------------------------------------------------------
attendant-title = Attendant Console
attendant-status-online = Online
attendant-status-busy = Busy
attendant-status-away = Away
attendant-status-offline = Offline
attendant-status-ready = Online - Ready for conversations
attendant-status-busy-msg = Busy - Handling conversations
attendant-status-away-msg = Away - Will be back soon
attendant-status-offline-msg = Offline - Not available

# -----------------------------------------------------------------------------
# Queue Stats
# -----------------------------------------------------------------------------
attendant-waiting = Waiting
attendant-active = Active
attendant-resolved = Resolved
attendant-mine = Mine

# -----------------------------------------------------------------------------
# Queue Filters
# -----------------------------------------------------------------------------
attendant-filter-all = All
attendant-filter-waiting = Waiting
attendant-filter-mine = Mine
attendant-filter-priority = Priority

# -----------------------------------------------------------------------------
# Conversation List
# -----------------------------------------------------------------------------
attendant-no-conversations = No conversations in queue
attendant-new-conversations-appear = New conversations will appear here
attendant-unread = Unread
attendant-typing = typing...
attendant-select-conversation = Select a conversation
attendant-select-conversation-desc = Choose a conversation from the queue to start responding

# -----------------------------------------------------------------------------
# Channel Tags
# -----------------------------------------------------------------------------
attendant-channel-whatsapp = WhatsApp
attendant-channel-teams = Teams
attendant-channel-instagram = Instagram
attendant-channel-web = Web
attendant-channel-telegram = Telegram
attendant-channel-email = Email

# -----------------------------------------------------------------------------
# Priority Tags
# -----------------------------------------------------------------------------
attendant-priority-urgent = Urgent
attendant-priority-high = High
attendant-priority-normal = Normal

# -----------------------------------------------------------------------------
# Chat Area
# -----------------------------------------------------------------------------
attendant-message-placeholder = Type your message...
attendant-send = Send
attendant-attach-file = Attach file
attendant-insert-emoji = Insert emoji
attendant-quick-responses = Quick Responses
attendant-transfer = Transfer
attendant-resolve = Resolve
attendant-more-actions = More Actions

# -----------------------------------------------------------------------------
# Quick Responses
# -----------------------------------------------------------------------------
attendant-quick-greeting = Hello! How can I help you today?
attendant-quick-thanks = Thank you for your patience.
attendant-quick-checking = Let me check that for you.
attendant-quick-moment = One moment please.

# -----------------------------------------------------------------------------
# Transfer Modal
# -----------------------------------------------------------------------------
attendant-transfer-title = Transfer Conversation
attendant-transfer-to = Transfer to
attendant-transfer-reason = Reason (optional)
attendant-transfer-reason-placeholder = Why are you transferring this conversation?
attendant-transfer-cancel = Cancel
attendant-transfer-confirm = Transfer

# -----------------------------------------------------------------------------
# AI Insights Sidebar
# -----------------------------------------------------------------------------
attendant-ai-insights = AI Insights
attendant-ai-summary = Conversation Summary
attendant-ai-sentiment = Customer Sentiment
attendant-sentiment-positive = Positive
attendant-sentiment-neutral = Neutral
attendant-sentiment-negative = Negative
attendant-smart-replies = Smart Replies
attendant-confidence = Confidence
attendant-source = Source

# -----------------------------------------------------------------------------
# Customer Details
# -----------------------------------------------------------------------------
attendant-customer-details = Customer Details
attendant-customer-name = Name
attendant-customer-email = Email
attendant-customer-phone = Phone
attendant-customer-location = Location
attendant-customer-tags = Tags

# -----------------------------------------------------------------------------
# Conversation History
# -----------------------------------------------------------------------------
attendant-history = History
attendant-history-resolved = Resolved
attendant-history-transferred = Transferred
attendant-history-abandoned = Abandoned
attendant-view-history = View Full History

# -----------------------------------------------------------------------------
# Toast Messages
# -----------------------------------------------------------------------------
attendant-toast-transferred = Conversation transferred successfully
attendant-toast-resolved = Conversation marked as resolved
attendant-toast-assigned = Conversation assigned to you
attendant-toast-error = An error occurred
attendant-toast-connection-lost = Connection lost. Reconnecting...
attendant-toast-connection-restored = Connection restored
