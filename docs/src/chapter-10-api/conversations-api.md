# Conversations API

The Conversations API provides endpoints for managing chat conversations, message history, and real-time communication.

## Overview

Conversations in General Bots are handled primarily through WebSocket connections for real-time messaging, with REST endpoints for history retrieval and session management.

## Endpoints

### Start Conversation

**POST** `/api/conversations/start`

Initiates a new conversation with a bot.

**Request:**
```json
{
  "bot_id": "bot-123",
  "initial_message": "Hello"
}
```

**Response:**
```json
{
  "conversation_id": "conv-456",
  "session_id": "session-789",
  "status": "active"
}
```

### Send Message

**POST** `/api/conversations/:id/messages`

Sends a message in an existing conversation.

**Request:**
```json
{
  "content": "User message",
  "attachments": []
}
```

**Response:**
```json
{
  "message_id": "msg-123",
  "timestamp": "2024-01-15T10:30:00Z",
  "status": "delivered"
}
```

### Get Conversation History

**GET** `/api/conversations/:id/history`

Retrieves message history for a conversation.

**Query Parameters:**
- `limit` - Number of messages (default: 50, max: 100)
- `before` - Messages before timestamp
- `after` - Messages after timestamp

**Response:**
```json
{
  "messages": [
    {
      "id": "msg-001",
      "sender": "user",
      "content": "Hello",
      "timestamp": "2024-01-15T10:00:00Z"
    },
    {
      "id": "msg-002",
      "sender": "bot",
      "content": "Hi! How can I help you?",
      "timestamp": "2024-01-15T10:00:01Z"
    }
  ],
  "has_more": false
}
```

### List Conversations

**GET** `/api/conversations`

Lists user's conversations.

**Query Parameters:**
- `bot_id` - Filter by bot
- `status` - Filter by status (active/archived)
- `limit` - Number of results
- `offset` - Pagination offset

**Response:**
```json
{
  "conversations": [
    {
      "id": "conv-456",
      "bot_id": "bot-123",
      "bot_name": "Support Bot",
      "last_message": "Thank you!",
      "last_activity": "2024-01-15T10:30:00Z",
      "status": "active"
    }
  ],
  "total": 1
}
```

## WebSocket Protocol

Real-time messaging uses WebSocket connections at `/ws`.

### Message Types

| Type | Direction | Description |
|------|-----------|-------------|
| `message` | Both | Chat message |
| `typing` | Server→Client | Bot is typing |
| `suggestion` | Server→Client | Quick reply suggestions |
| `status` | Server→Client | Connection status |
| `error` | Server→Client | Error notification |

### Send Message Format

```json
{
  "type": "message",
  "content": "Hello",
  "session_id": "session-123"
}
```

### Receive Message Format

```json
{
  "type": "message",
  "sender": "bot",
  "content": "Hi! How can I help you?",
  "timestamp": "2024-01-15T10:00:01Z"
}
```

## Anonymous Conversations

Anonymous users can chat without authentication:

- Session created automatically on WebSocket connect
- Limited to default bot only
- No history persistence
- Session expires after inactivity

## Authenticated Conversations

Logged-in users get additional features:

- Full conversation history
- Multiple bot access
- Cross-device sync
- Persistent sessions

## Database Schema

Conversations are stored in:

```sql
-- sessions table
CREATE TABLE sessions (
    id UUID PRIMARY KEY,
    user_id UUID,
    bot_id UUID,
    status TEXT,
    created_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ
);

-- message_history table  
CREATE TABLE message_history (
    id UUID PRIMARY KEY,
    session_id UUID REFERENCES sessions(id),
    sender TEXT,
    content TEXT,
    metadata JSONB,
    created_at TIMESTAMPTZ
);
```

## Error Handling

| Status Code | Error | Description |
|-------------|-------|-------------|
| 400 | `invalid_message` | Malformed message content |
| 401 | `unauthorized` | Authentication required |
| 403 | `forbidden` | No access to conversation |
| 404 | `not_found` | Conversation doesn't exist |
| 429 | `rate_limited` | Too many messages |

## Rate Limits

| Endpoint | Limit |
|----------|-------|
| Messages | 60/minute per user |
| History | 100/minute per user |
| List | 30/minute per user |

## See Also

- [Sessions and Channels](../chapter-01/sessions.md) - Session management
- [TALK Keyword](../chapter-06-gbdialog/keyword-talk.md) - Sending messages from BASIC
- [HEAR Keyword](../chapter-06-gbdialog/keyword-hear.md) - Receiving user input