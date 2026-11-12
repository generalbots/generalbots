# General Bots - Error Messages (English)
# This file contains all error message translations

# =============================================================================
# HTTP Errors
# =============================================================================

error-http-400 = Bad request. Please check your input.
error-http-401 = Authentication required. Please log in.
error-http-403 = You don't have permission to access this resource.
error-http-404 = { $entity } not found.
error-http-409 = Conflict: { $message }
error-http-429 = Too many requests. Please wait { $seconds } seconds.
error-http-500 = Internal server error. Please try again later.
error-http-502 = Bad gateway. The server received an invalid response.
error-http-503 = Service temporarily unavailable. Please try again later.
error-http-504 = Request timed out after { $milliseconds }ms.

# =============================================================================
# Validation Errors
# =============================================================================

error-validation-required = { $field } is required.
error-validation-email = Please enter a valid email address.
error-validation-url = Please enter a valid URL.
error-validation-phone = Please enter a valid phone number.
error-validation-min-length = { $field } must be at least { $min } characters.
error-validation-max-length = { $field } must be no more than { $max } characters.
error-validation-min-value = { $field } must be at least { $min }.
error-validation-max-value = { $field } must be no more than { $max }.
error-validation-pattern = { $field } format is invalid.
error-validation-unique = { $field } already exists.
error-validation-mismatch = { $field } does not match { $other }.
error-validation-date-format = Please enter a valid date in the format { $format }.
error-validation-date-past = { $field } must be in the past.
error-validation-date-future = { $field } must be in the future.

# =============================================================================
# Authentication Errors
# =============================================================================

error-auth-invalid-credentials = Invalid email or password.
error-auth-account-locked = Your account has been locked. Please contact support.
error-auth-account-disabled = Your account has been disabled.
error-auth-session-expired = Your session has expired. Please log in again.
error-auth-token-invalid = Invalid or expired token.
error-auth-token-missing = Authentication token is required.
error-auth-mfa-required = Multi-factor authentication is required.
error-auth-mfa-invalid = Invalid verification code.
error-auth-password-weak = Password is too weak. Please use a stronger password.
error-auth-password-expired = Your password has expired. Please reset it.

# =============================================================================
# Configuration Errors
# =============================================================================

error-config = Configuration error: { $message }
error-config-missing = Missing configuration: { $key }
error-config-invalid = Invalid configuration value for { $key }: { $reason }
error-config-file-not-found = Configuration file not found: { $path }
error-config-parse = Failed to parse configuration: { $message }

# =============================================================================
# Database Errors
# =============================================================================

error-database = Database error: { $message }
error-database-connection = Failed to connect to database.
error-database-timeout = Database operation timed out.
error-database-constraint = Database constraint violation: { $constraint }
error-database-duplicate = A record with this { $field } already exists.
error-database-migration = Database migration failed: { $message }

# =============================================================================
# File & Storage Errors
# =============================================================================

error-file-not-found = File not found: { $filename }
error-file-too-large = File is too large. Maximum size is { $maxSize }.
error-file-type-not-allowed = File type not allowed. Allowed types: { $allowedTypes }.
error-file-empty = The uploaded file is empty.
error-file-name-invalid = The file name is not valid.
error-file-content-mismatch = The file content does not match its type ({ $expected } expected, { $detected } found).
error-file-archive-unsafe = The archive was refused: { $reason }.
error-file-extension-blocked = Files with the .{ $extension } extension are not allowed.
error-file-extension-unknown = The .{ $extension } extension is not recognised.
error-file-upload-failed = File upload failed: { $message }
error-file-read = Failed to read file: { $message }
error-file-write = Failed to write file: { $message }
error-storage-full = Storage quota exceeded.
error-storage-unavailable = Storage service is unavailable.

# =============================================================================
# Network & External Service Errors
# =============================================================================

error-network = Network error: { $message }
error-network-timeout = Connection timed out.
error-network-unreachable = Server is unreachable.
error-service-unavailable = Service unavailable: { $service }
error-external-api = External API error: { $message }
error-rate-limit = Rate limited. Retry after { $seconds }s.

# =============================================================================
# Bot & Dialog Errors
# =============================================================================

error-bot-not-found = Bot not found: { $botId }
error-bot-disabled = This bot is currently disabled.
error-bot-script-error = Script error at line { $line }: { $message }
error-bot-timeout = Bot response timed out.
error-bot-quota-exceeded = Bot usage quota exceeded.
error-dialog-not-found = Dialog not found: { $dialogId }
error-dialog-invalid = Invalid dialog configuration: { $message }

# =============================================================================
# LLM & AI Errors
# =============================================================================

error-llm-unavailable = AI service is currently unavailable.
error-llm-timeout = AI request timed out.
error-llm-rate-limit = AI rate limit exceeded. Please wait before trying again.
error-llm-content-filter = Content was filtered by safety guidelines.
error-llm-context-length = Input is too long. Please shorten your message.
error-llm-invalid-response = Received invalid response from AI service.
error-llm-empty-response = Sorry, I could not process your message right now. Please try again in a few seconds.

# =============================================================================
# Email Errors
# =============================================================================

error-email-send-failed = Failed to send email: { $message }
error-email-invalid-recipient = Invalid recipient email address: { $email }
error-email-attachment-failed = Failed to attach file: { $filename }
error-email-template-not-found = Email template not found: { $template }

# =============================================================================
# Calendar & Scheduling Errors
# =============================================================================

error-calendar-conflict = Time slot conflicts with existing event.
error-calendar-past-date = Cannot schedule events in the past.
error-calendar-invalid-recurrence = Invalid recurrence pattern.
error-calendar-event-not-found = Event not found: { $eventId }

# =============================================================================
# Task Errors
# =============================================================================

error-task-not-found = Task not found: { $taskId }
error-task-already-completed = Task has already been completed.
error-task-circular-dependency = Circular dependency detected in tasks.
error-task-invalid-status = Invalid task status transition.

# =============================================================================
# Permission Errors
# =============================================================================

error-permission-denied = You don't have permission to perform this action.
error-permission-resource = You don't have access to this { $resource }.
error-permission-action = You cannot { $action } this { $resource }.
error-permission-owner-only = Only the owner can perform this action.

# =============================================================================
# Generic Errors
# =============================================================================

error-internal = Internal error: { $message }
error-unexpected = An unexpected error occurred. Please try again.
error-not-implemented = This feature is not yet implemented.
error-maintenance = System is under maintenance. Please try again later.
error-unknown = An unknown error occurred.

# =============================================================================
# Localized API contract (#2094) — stable codes, localized messages
# =============================================================================

error-locale-invalid = The requested language is not valid.
error-locale-unsupported = This language is not available.
error-locale-persist-failed = Could not save your language preference.
error-bot-id-required = The bot id is required.
error-bot-id-invalid = The bot id is not valid.
error-bot-name-required = The bot name is required.
error-token-invalid-format = The token format is not valid.
error-token-signing-failed = Could not sign the token.
error-token-signature-invalid = The token signature is not valid.
error-token-payload-invalid = The token payload is not valid.
error-token-payload-json-invalid = The token payload is not valid JSON.
error-cloud-token-required = Provide a cloud_token.
error-llm-subscription-token-missing = Missing subscription token.
error-llm-token-store-unavailable = The token store is unavailable.
error-llm-subscription-token-expired = The subscription token expired; send the message again.
error-llm-prompt-missing = There is no prompt in the request.
error-llm-account-unlinked = Connect your Claude or ChatGPT account in Settings → AI accounts first.

# =============================================================================
# Localized API errors — routes, drive sync, peers, computer use (#2094)
# =============================================================================

error-agent-drive-database-unavailable = The file store is temporarily unavailable.
error-agent-drive-lookup-failed = Could not read the conversation. Please try again.
error-agent-drive-conversation-not-found = Conversation not found.
error-agent-drive-bad-owner = The conversation owner record is not valid.
error-agent-drive-forbidden = This conversation belongs to another user.
error-agent-drive-bad-bot = The bot record for this conversation is not valid.
error-agent-drive-workspace-stopped = The agent workspace is not running.
error-agent-drive-no-files = There are no files in the workspace output folder.
error-agent-drive-named-files-missing = The requested files are not in the workspace output folder.
error-agent-drive-save-failed = Could not save any file to Drive.
error-agent-drive-file-missing = The file is not in the workspace output folder.
error-agent-drive-save-one-failed = Could not save the file to Drive.
error-agent-drive-bad-path = The Drive path returned by the store is not valid.
error-agent-drive-link-failed = Could not create the share link.
error-subagent-no-llm = No LLM provider is configured for sub-agents.
error-goal-run-not-running = This goal run is not running.
error-goal-run-not-paused = This goal run is not paused.
error-autotask-not-found = Autotask not found.
error-autotask-intent-required = Describe what the task should do.
error-peer-network-disabled = The peer network is disabled on this node.
error-peer-network-unauthorized = A valid peer bearer token is required.
error-peer-network-unconfigured = This node has no peer token configured, so peer endpoints are closed.
error-peer-network-forbidden = An administrator session is required for this peer control.
error-peer-network-invalid = The peer request is not valid: { $message }
error-peer-network-unknown-node = Unknown peer node: { $message }
error-peer-network-registry-full = The peer registry is full.
error-peer-network-payload-store-full = The peer payload store is full: { $message }
error-peer-network-recipient-required = The recipient query parameter is required.
error-drive-sync-field-required = { $field } is required.
error-drive-sync-field-absolute = { $field } must be an absolute path.
error-drive-sync-field-parent = { $field } must not contain '..'.
error-drive-sync-field-not-folder = { $field } is not a folder on this machine: { $path }
error-drive-sync-field-local-folder = The local folder
error-drive-sync-field-drive-folder = The Drive folder
error-drive-sync-job-not-found = Sync job { $id } was not found.
error-drive-sync-job-name-required = A sync job name is required.
error-drive-sync-job-exists = A sync job named { $name } already exists.
error-drive-sync-conflict-path-required = A conflict path is required.
error-drive-sync-conflict-path-escape = The conflict path escapes its sync folder.
error-drive-sync-task-failed = The { $task } task failed: { $error }
error-org-auth-service-unavailable = The directory service is not available.
error-org-default-delete = The default organization cannot be deleted.
error-org-settings-save-failed = Could not save the organization settings.
error-org-name-required = The organization name is required.
error-org-tenant-missing = No tenant exists yet — run the migrations or the bootstrap first.
error-computer-use-session-required = A session id is required.
error-computer-use-kind-invalid = The kind must be 'browser' or 'terminal'.
error-computer-use-action-not-found = Action not found.
error-computer-use-no-consent = There is no consent record for this action; approve it first.
error-computer-use-browser-only = This action must run in the browser panel.
error-computer-use-url-missing = The action has no URL.
error-terminal-approval-required = Terminal execution requires explicit user approval.
error-agent-vm-disabled = Agent/VM is disabled for this conversation. Turn it on in the chat controls to use the workspace.
error-terminal-action-id-required = Terminal execution requires an approved action id.
error-terminal-command-required = A command is required.
error-terminal-command-too-long = The command is limited to 16000 characters.
error-terminal-blocked-pattern = The command matches a blocked destructive pattern.
error-terminal-workspace-stopped = The agent workspace is not running. Finish provisioning (or restart the desktop app), then retry.
error-terminal-timeout = The command timed out.

# =============================================================================
# Collaboration, groups, autotasks and AI accounts (#2094)
# =============================================================================

error-db-unavailable = The database is unavailable right now. Please try again.
error-collab-invalid-resource = The resource type or resource id is invalid.
error-collab-owner-only = Only the owner can do this.
error-collab-invalid-grantee-type = The grantee type must be 'user' or 'domain'.
error-collab-invalid-role = The role is invalid.
error-collab-invalid-grantee-id = The grantee id is invalid.
error-collab-grant-not-found = The grant was not found.
error-collab-invalid-new-owner = The new owner id is invalid.
error-collab-link-not-found = The share link was not found.
error-collab-comment-not-found = The comment was not found.
error-collab-comment-resolve-forbidden = The comment was not found, or it is not yours to resolve.
error-collab-comment-body-required = A comment body is required.
error-collab-comment-too-long = The comment is limited to 8000 characters.
error-collab-invalid-emoji = The emoji is invalid.
error-collab-invalid-cursor = The 'before' cursor is invalid.
error-collab-invalid-action = The action is invalid.
error-collab-invalid-name = The name is invalid.
error-collab-content-required = Content is required.
error-collab-content-too-large = The content is too large.
error-collab-version-not-found = The version was not found.
error-collab-invalid-base-version = The base version is invalid.
error-collab-invalid-parent = The parent id is invalid.
error-collab-invalid-op-type = The operation type is invalid.
error-collab-operation-not-found = The operation was not found.
error-collab-not-a-conflict = The operation is not an unresolved conflict.
error-collab-invalid-resolution = The resolution must be 'accept-server' or 'accept-client'.
error-group-budget-not-found = The group budget was not found.
error-group-budget-invalid-id = The budget id is invalid.
error-group-budget-amount-invalid = The amount must be zero or positive.
error-group-budget-caps-invalid = The caps must be zero or positive (zero means no limit).
error-autotask-fields-required = Title, schedule and command are required.
error-autotask-kind-invalid = The kind must be 'cron' or 'systemd'.
error-conversation-not-found = The conversation was not found.
error-conversation-session-invalid = The conversation session is invalid.
error-subagent-invalid-tasks = The sub-agent tasks are invalid.
error-ai-account-unknown-provider = The AI account provider is unknown.
error-ai-account-workspace-unavailable = The agent workspace is not available in this build.
error-ai-account-agent-invalid = The agent must be reasoning, agentic or fast.
error-ai-account-not-connected-vibe = Connect the account before assigning it to Vibe.
error-ai-account-not-connected-chat = Connect the account before selecting it for chat.
error-computer-use-action-id-mismatch = The action id does not match the path.
error-http-no-route = No route matches this path.
error-goal-run-advance-refused = The run cannot advance in its current state.

# =============================================================================
# People, attendance and products routes (#2094)
# =============================================================================

error-person-not-found = Person not found.
error-team-not-found = Team not found.
error-time-off-not-found = Time-off request not found.
error-people-invalid-start-date = The start date is invalid. Use the YYYY-MM-DD format.
error-people-invalid-end-date = The end date is invalid. Use the YYYY-MM-DD format.
error-attendance-webhook-not-found = Webhook not found.
error-attendance-session-id-invalid = The session id is invalid.
error-attendance-session-not-found = Session not found.
error-attendance-attendant-id-required = An attendant id is required.
attendance-no-attendant-matching-skills = No attendant was found with the required skills.
attendance-response-sent = Response sent.
error-product-not-found = Product not found.
error-service-not-found = Service not found.
error-service-invalid-value = The service value is invalid.
error-service-price-required = This service has no price; provide a value.

# Service-tax value validation (#2094)
error-invalid-value = The supplied value could not be parsed.
error-value-required = A value is required for this service.
