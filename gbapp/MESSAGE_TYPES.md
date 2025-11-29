# Message Types Documentation

## Overview

The botserver uses a simple enum-based system for categorizing different types of messages flowing through the system. This document describes each message type and its usage.

## Message Type Enum

The `MessageType` enum is defined in both Rust (backend) and JavaScript (frontend) to ensure consistency across the entire application.

### Backend (Rust)
Location: `src/core/shared/message_types.rs`

### Frontend (JavaScript)
Location: `ui/shared/messageTypes.js`

## Message Types

| Value | Name | Description | Usage |
|-------|------|-------------|-------|
| 0 | `EXTERNAL` | Messages from external systems | WhatsApp, Instagram, Teams, and other external channel integrations |
| 1 | `USER` | User messages from web interface | Regular user input from the web chat interface |
| 2 | `BOT_RESPONSE` | Bot responses | Can contain either regular text content or JSON-encoded events (theme changes, thinking indicators, etc.) |
| 3 | `CONTINUE` | Continue interrupted response | Used when resuming a bot response that was interrupted |
| 4 | `SUGGESTION` | Suggestion or command message | Used for contextual suggestions and command messages |
| 5 | `CONTEXT_CHANGE` | Context change notification | Signals when the conversation context has changed |

## Special Handling for BOT_RESPONSE (Type 2)

The `BOT_RESPONSE` type requires special handling in the frontend because it can contain two different types of content:

### 1. Regular Text Content
Standard bot responses containing plain text or markdown that should be displayed directly to the user.

### 2. Event Messages
JSON-encoded objects with the following structure:
```json
{
  "event": "event_type",
  "data": {
    // Event-specific data
  }
}
```

#### Supported Events:
- `thinking_start` - Bot is processing/thinking
- `thinking_end` - Bot finished processing
- `warn` - Warning message to display
- `context_usage` - Context usage update
- `change_theme` - Theme customization data

## Frontend Detection Logic

The frontend uses the following logic to differentiate between regular content and event messages:

1. Check if `message_type === 2` (BOT_RESPONSE)
2. Check if content starts with `{` or `[` (potential JSON)
3. Attempt to parse as JSON
4. If successful and has `event` and `data` properties, handle as event
5. Otherwise, process as regular message content

## Usage Examples

### Rust Backend
```rust
use crate::shared::message_types::MessageType;

let response = BotResponse {
    // ... other fields
    message_type: MessageType::BOT_RESPONSE,
    // ...
};
```

### JavaScript Frontend
```javascript
if (message.message_type === MessageType.BOT_RESPONSE) {
    // Handle bot response
}

if (isUserMessage(message)) {
    // Handle user message
}
```

## Migration Notes

When migrating from magic numbers to the MessageType enum:

1. Replace all hardcoded message type numbers with the appropriate constant
2. Import the MessageType module/script where needed
3. Use the helper functions for type checking when available

## Benefits

1. **Type Safety**: Reduces errors from using wrong message type numbers
2. **Readability**: Code is self-documenting with named constants
3. **Maintainability**: Easy to add new message types or modify existing ones
4. **Consistency**: Same values used across frontend and backend