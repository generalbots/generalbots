# Conversation Management

BotServer manages conversations through sessions, message history, and context tracking, providing seamless interaction between users and bots.

## Overview

Conversation management handles:
- Session lifecycle
- Message history persistence
- Context maintenance
- Multi-turn interactions
- User state tracking

## Session Management

### Session Creation

Sessions are created when:
- User first interacts with bot
- Authentication successful
- Anonymous user connects

Session contains:
- Unique session ID
- User reference (or anonymous)
- Bot ID
- Creation timestamp
- Expiration time

### Session Persistence

Sessions stored in PostgreSQL:
- `user_sessions` table
- Cached in Redis for performance
- Automatic cleanup on expiry

## Message History

### Storage Structure

Messages stored in `message_history` table:
- Session ID reference
- User ID
- Bot ID
- Message content
- Sender (user/bot)
- Timestamp

### Message Types

- **User Messages** - Input from users
- **Bot Responses** - Generated replies
- **System Messages** - Status updates
- **Tool Outputs** - Results from tool execution

### History Retrieval

```basic
# In BASIC scripts
let history = GET_BOT_MEMORY("conversation_history")
```

## Context Management

### Context Layers

1. **System Context** - Bot configuration and prompts
2. **Conversation Context** - Recent message history
3. **Knowledge Context** - Retrieved documents
4. **User Context** - User preferences and state
5. **Tool Context** - Available tool definitions

### Context Window

Limited by LLM constraints:
- Automatic truncation
- Oldest messages removed first
- Important context preserved
- Summarization for long conversations

### Context Operations

```basic
# Set context in BASIC
SET_CONTEXT "user_info" AS "Name: John, Role: Admin"

# Clear context
CLEAR_CONTEXT "user_info"
```

## Conversation Flow

### Turn Management

1. User sends message
2. Session validated
3. Context assembled
4. LLM processes input
5. Tools invoked if needed
6. Response generated
7. History updated
8. Response sent to user

### State Tracking

Conversation state includes:
- Current topic
- Active tools
- Pending operations
- User preferences
- Session variables

## Multi-Turn Interactions

### Maintaining Continuity

- Previous messages included in context
- Entity tracking across turns
- Reference resolution
- Topic persistence

### Example Flow

```
User: "Book a meeting for tomorrow"
Bot: "What time would you prefer?"
User: "2 PM"
Bot: "Meeting booked for tomorrow at 2 PM"
```

Bot maintains context of "meeting" and "tomorrow" across turns.

## Conversation Patterns

### Question-Answer

Simple single-turn interactions:
```basic
let question = HEAR
let answer = LLM "Answer: " + question
TALK answer
```

### Guided Conversation

Multi-step flows with validation:
```basic
TALK "What's your name?"
let name = HEAR

TALK "What's your email?"
let email = HEAR

# Process collected information
```

### Contextual Dialog

Using conversation history:
```basic
let history = GET_CONVERSATION_HISTORY()
let response = LLM "Continue this conversation: " + history
TALK response
```

## Conversation Persistence

### Database Storage

All conversations permanently stored:
- Full message history
- Timestamps
- User associations
- Bot responses

### Archival

Old conversations:
- Compressed after 30 days
- Archived after 90 days
- Configurable retention
- GDPR compliance

## Anonymous Conversations

### Anonymous Sessions

Users without authentication:
- Temporary session created
- Limited permissions
- No permanent storage
- Session expires quickly

### Upgrading to Authenticated

When anonymous user logs in:
- Session associated with user
- History preserved
- Permissions updated
- Full features enabled

## Conversation Analytics

### Metrics Tracked

- Message count
- Conversation length
- Response time
- User satisfaction
- Tool usage
- Topic distribution

### Analysis Capabilities

- Sentiment analysis
- Topic extraction
- Intent classification
- Entity recognition
- Performance metrics

## WebSocket Communication

### Real-Time Messaging

WebSocket connection for:
- Instant message delivery
- Streaming responses
- Typing indicators
- Connection status

### Protocol

```javascript
// Client sends
{
  "type": "message",
  "content": "User message",
  "session_id": "session-123"
}

// Server responds
{
  "type": "response",
  "content": "Bot response",
  "streaming": false
}
```

## Conversation Recovery

### Session Restoration

After disconnection:
- Session ID validates
- History restored
- Context rebuilt
- Conversation continues

### Error Recovery

On failure:
- Last state saved
- Graceful degradation
- User notified
- Automatic retry

## Best Practices

1. **Keep Context Relevant** - Remove outdated information
2. **Manage History Length** - Truncate old messages
3. **Save Important State** - Use BOT_MEMORY for persistence
4. **Handle Disconnections** - Implement recovery logic
5. **Track Metrics** - Monitor conversation quality
6. **Respect Privacy** - Implement data retention policies

## Configuration

### Session Settings

```
SESSION_TIMEOUT_MINUTES=30
MAX_HISTORY_LENGTH=50
CONTEXT_WINDOW_SIZE=4000
```

### Retention Policy

```
MESSAGE_RETENTION_DAYS=90
ARCHIVE_AFTER_DAYS=30
ANONYMOUS_RETENTION_HOURS=24
```

## Limitations

- Context window size constraints
- History storage costs
- Real-time processing overhead
- Concurrent conversation limits

## Future Enhancements

- Voice conversation support
- Multi-language conversations
- Conversation branching
- Advanced analytics
- Conversation templates

## Summary

Conversation management in BotServer provides robust session handling, message persistence, and context management, enabling sophisticated multi-turn interactions while maintaining performance and reliability.