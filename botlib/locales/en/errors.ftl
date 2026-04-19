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
