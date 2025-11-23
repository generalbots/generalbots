# Multi-Channel Support

BotServer provides a flexible multi-channel architecture that allows bots to interact with users across different communication platforms while maintaining consistent conversation state.

## Overview

The channel system in BotServer abstracts communication methods, allowing the same bot logic to work across:
- Web interface (primary channel)
- WebSocket connections
- Voice interactions (optional)
- Future: WhatsApp, Teams, etc.

## Channel Architecture

### Channel Adapter Pattern

BotServer uses an adapter pattern for channel abstraction:

```
User → Channel Adapter → Bot Session → BASIC Script → Response → Channel Adapter → User
```

Each channel adapter:
- Receives user input in channel-specific format
- Converts to common message format
- Processes bot response
- Converts back to channel format

## Implemented Channels

### Web Channel

The primary channel using HTTP/WebSocket:

**Features:**
- Real-time messaging via WebSocket
- Session persistence
- File uploads
- Suggestion buttons
- Theme customization
- Typing indicators

**Implementation:**
- WebChannelAdapter in `core/bot/channels.rs`
- Handles HTTP requests and WebSocket connections
- Maintains response channels for streaming

### Voice Channel (Optional)

Voice interaction support when enabled:

**Features:**
- Speech-to-text input
- Text-to-speech output  
- Continuous conversation flow

**Implementation:**
- VoiceAdapter for audio processing
- Requires voice feature flag
- Integrates with speech services

## Channel Abstraction Layer

### ChannelAdapter Trait

All channels implement a common interface:
- `send_message()` - Send to user
- `receive_message()` - Get from user
- `get_channel_type()` - Identify channel
- `supports_feature()` - Check capabilities

### Message Format

Common message structure across channels:
- `content` - Text content
- `user_id` - User identifier
- `session_id` - Session reference
- `channel` - Channel type
- `metadata` - Channel-specific data

## Session Management

### Unified Sessions

All channels share the same session system:
- Session created on first interaction
- Maintained across channel switches
- Preserves conversation context
- Stores user preferences

### Cross-Channel Continuity

Users can switch channels and continue conversations:
1. Start on web interface
2. Continue on mobile (when available)
3. Context and history preserved

## Response Handling

### BotResponse Structure

Responses adapted for each channel:
- `content` - Main message text
- `suggestions` - Quick reply options
- `message_type` - Text/card/media indicator
- `stream_token` - For streaming responses
- `is_complete` - End of response flag

### Channel-Specific Features

Different channels support different features:

**Web Channel:**
- Rich formatting (Markdown)
- Clickable suggestions
- File attachments
- Inline images

**Voice Channel:**
- Audio streaming
- Voice commands
- Natural pauses

## Implementation Details

### Web Channel Flow

1. User sends message via WebSocket
2. WebChannelAdapter receives message
3. Creates/retrieves session
4. Executes BASIC script
5. Streams response back
6. Updates UI in real-time

### Channel Registration

Channels register with the system on startup:
- Channel type identifier
- Supported features
- Message handlers
- Response formatters

## Message Types

### Standard Messages

Basic text communication:
- User questions
- Bot responses
- System notifications

### Interactive Elements

Channel-specific interactions:
- Suggestion buttons (web)
- Voice commands (voice)
- Quick replies (future: messaging apps)

### Media Messages

Depending on channel support:
- Images
- Documents  
- Audio clips
- Videos

## Channel Features Matrix

| Feature | Web | Voice | WhatsApp* | Teams* |
|---------|-----|-------|-----------|---------|
| Text | ✓ | ✓ | ✓ | ✓ |
| Rich Format | ✓ | ✗ | Limited | ✓ |
| Suggestions | ✓ | ✗ | ✓ | ✓ |
| Files | ✓ | ✗ | ✓ | ✓ |
| Streaming | ✓ | ✓ | ✗ | ✗ |
| Persistence | ✓ | ✓ | ✓ | ✓ |

*Future channels

## Universal Messaging

BASIC scripts work across all channels using universal keywords:
- `TALK` - Sends to any channel
- `HEAR` - Receives from any channel
- Channel adapter handles format conversion

Example:
```basic
TALK "Hello! How can I help?"
let response = HEAR
# Works on web, voice, or any channel
```

## Channel-Specific Behavior

### Adaptive Responses

Bots can detect and adapt to channels:
```basic
let channel = GET_CHANNEL()
if (channel == "voice") {
    TALK "Say 'yes' or 'no'"
} else {
    TALK "Click yes or no below"
    ADD_SUGGESTION "yes" AS "Yes"
    ADD_SUGGESTION "no" AS "No"
}
```

### Feature Detection

Check channel capabilities:
```basic
if (SUPPORTS_FEATURE("suggestions")) {
    ADD_SUGGESTION "help" AS "Get Help"
}
```

## WebSocket Protocol

### Connection Establishment

1. Client connects to `/ws` endpoint
2. Server creates session
3. Bidirectional channel established
4. Heartbeat maintains connection

### Message Protocol

**Client to Server:**
```json
{
  "type": "message",
  "content": "User message",
  "session_id": "uuid"
}
```

**Server to Client:**
```json
{
  "type": "response",
  "content": "Bot response",
  "suggestions": [],
  "is_complete": true
}
```

## Future Channel Support

### Planned Integrations

Infrastructure exists for:
- WhatsApp Business API
- Microsoft Teams
- Slack
- Telegram
- Discord
- SMS

### Adding New Channels

1. Implement ChannelAdapter trait
2. Handle channel-specific protocol
3. Map to common message format
4. Register with system
5. Test cross-channel scenarios

## Best Practices

1. **Write Channel-Agnostic Scripts**: Use universal keywords
2. **Test Across Channels**: Ensure consistency
3. **Handle Feature Differences**: Check capabilities
4. **Maintain Context**: Preserve session state
5. **Format Appropriately**: Adapt content to channel
6. **Monitor Performance**: Track channel metrics

## Limitations

- Some features channel-specific
- Rich media support varies
- Rate limiting per channel
- Authentication requirements differ
- Network reliability impacts real-time channels

## Summary

BotServer's multi-channel architecture enables bots to communicate consistently across different platforms while adapting to channel-specific capabilities. The channel adapter pattern ensures bot logic remains portable while providing optimal user experience on each platform.