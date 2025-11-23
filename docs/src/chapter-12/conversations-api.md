# Conversations API

The Conversations API provides endpoints for managing chat conversations, message history, and real-time communication.

## Overview

**Note**: These endpoints are planned but not yet implemented. They represent the intended API design for conversation management.

## Planned Endpoints

### Start Conversation

**POST** `/conversations/start`

Initiates a new conversation with a bot.

**Planned Request:**
```json
{
  "bot_id": "bot-123",
  "initial_message": "Hello"
}
```

**Planned Response:**
```json
{
  "conversation_id": "conv-456",
  "session_id": "session-789",
  "status": "active"
}
```

### Send Message

**POST** `/conversations/:id/messages`

Sends a message in an existing conversation.

**Planned Request:**
```json
{
  "content": "User message",
  "attachments": []
}
```

### Get Conversation History

**GET** `/conversations/:id/history`

Retrieves message history for a conversation.

**Planned Query Parameters:**
- `limit` - Number of messages
- `before` - Messages before timestamp
- `after` - Messages after timestamp

### List Conversations

**GET** `/conversations`

Lists user's conversations.

**Planned Query Parameters:**
- `bot_id` - Filter by bot
- `status` - Filter by status (active/archived)

## Current Implementation

Currently, conversations are handled through:
- WebSocket connections at `/ws`
- Session management in database
- Message history stored in `message_history` table

Real-time messaging is functional but REST endpoints for conversation management are not yet implemented.

## WebSocket Protocol

The current implementation uses WebSocket for real-time conversations:

```javascript
// Connect
ws = new WebSocket('ws://localhost:8080/ws');

// Send message
ws.send(JSON.stringify({
  type: 'message',
  content: 'Hello',
  session_id: 'session-123'
}));

// Receive response
ws.onmessage = (event) => {
  const response = JSON.parse(event.data);
  console.log(response.content);
};
```

## Future Implementation

These REST endpoints will be added to provide:
- Conversation management
- History retrieval
- Batch operations
- Analytics integration

## Status

**Not Implemented** - Use WebSocket connection for conversations.