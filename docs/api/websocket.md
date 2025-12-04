# WebSocket API

Real-time bidirectional communication with General Bots.

## Connection

### Endpoint

```
ws://localhost:8080/ws
wss://your-domain.com/ws  (production)
```

### Authentication

Include the auth token as a query parameter or in the first message:

```javascript
// Option 1: Query parameter
const ws = new WebSocket('ws://localhost:8080/ws?token=<auth_token>');

// Option 2: First message
ws.onopen = () => {
    ws.send(JSON.stringify({
        type: 'auth',
        token: '<auth_token>'
    }));
};
```

## Message Format

All messages are JSON objects with a `type` field:

```json
{
    "type": "message_type",
    "payload": { ... },
    "timestamp": "2024-01-01T10:00:00Z"
}
```

## Client Messages

### Send Chat Message

```json
{
    "type": "message",
    "session_id": "550e8400-e29b-41d4-a716-446655440000",
    "content": "Hello, I need help with my order"
}
```

### Start Typing

```json
{
    "type": "typing_start",
    "session_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

### Stop Typing

```json
{
    "type": "typing_stop",
    "session_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

### Subscribe to Session

```json
{
    "type": "subscribe",
    "session_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

### Unsubscribe from Session

```json
{
    "type": "unsubscribe",
    "session_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

### Ping (Keep-Alive)

```json
{
    "type": "ping"
}
```

## Server Messages

### Chat Response

```json
{
    "type": "message",
    "session_id": "550e8400-e29b-41d4-a716-446655440000",
    "message_id": "msg-uuid",
    "role": "assistant",
    "content": "I'd be happy to help! What's your order number?",
    "timestamp": "2024-01-01T10:00:01Z"
}
```

### Streaming Response

For LLM responses, content streams in chunks:

```json
{
    "type": "stream_start",
    "session_id": "550e8400-e29b-41d4-a716-446655440000",
    "message_id": "msg-uuid"
}
```

```json
{
    "type": "stream_chunk",
    "message_id": "msg-uuid",
    "content": "I'd be happy to "
}
```

```json
{
    "type": "stream_chunk",
    "message_id": "msg-uuid",
    "content": "help! What's your "
}
```

```json
{
    "type": "stream_end",
    "message_id": "msg-uuid",
    "content": "I'd be happy to help! What's your order number?"
}
```

### Bot Typing Indicator

```json
{
    "type": "bot_typing",
    "session_id": "550e8400-e29b-41d4-a716-446655440000",
    "is_typing": true
}
```

### Tool Execution

When the bot calls a tool:

```json
{
    "type": "tool_call",
    "session_id": "550e8400-e29b-41d4-a716-446655440000",
    "tool": "search_orders",
    "arguments": {"order_id": "12345"}
}
```

```json
{
    "type": "tool_result",
    "session_id": "550e8400-e29b-41d4-a716-446655440000",
    "tool": "search_orders",
    "result": {"status": "shipped", "tracking": "1Z999..."}
}
```

### Error

```json
{
    "type": "error",
    "code": "SESSION_NOT_FOUND",
    "message": "Session does not exist or has expired"
}
```

### Pong (Keep-Alive Response)

```json
{
    "type": "pong",
    "timestamp": "2024-01-01T10:00:00Z"
}
```

### Session Events

```json
{
    "type": "session_created",
    "session_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

```json
{
    "type": "session_closed",
    "session_id": "550e8400-e29b-41d4-a716-446655440000",
    "reason": "user_disconnect"
}
```

## JavaScript Client Example

```javascript
class BotClient {
    constructor(url, token) {
        this.url = url;
        this.token = token;
        this.ws = null;
        this.handlers = {};
    }

    connect() {
        this.ws = new WebSocket(`${this.url}?token=${this.token}`);
        
        this.ws.onopen = () => {
            console.log('Connected to bot');
            this.emit('connected');
        };
        
        this.ws.onmessage = (event) => {
            const data = JSON.parse(event.data);
            this.emit(data.type, data);
        };
        
        this.ws.onclose = () => {
            console.log('Disconnected');
            this.emit('disconnected');
            // Auto-reconnect after 3 seconds
            setTimeout(() => this.connect(), 3000);
        };
        
        this.ws.onerror = (error) => {
            console.error('WebSocket error:', error);
            this.emit('error', error);
        };
    }

    send(type, payload) {
        if (this.ws?.readyState === WebSocket.OPEN) {
            this.ws.send(JSON.stringify({ type, ...payload }));
        }
    }

    sendMessage(sessionId, content) {
        this.send('message', { session_id: sessionId, content });
    }

    subscribe(sessionId) {
        this.send('subscribe', { session_id: sessionId });
    }

    on(event, handler) {
        this.handlers[event] = this.handlers[event] || [];
        this.handlers[event].push(handler);
    }

    emit(event, data) {
        (this.handlers[event] || []).forEach(h => h(data));
    }

    disconnect() {
        this.ws?.close();
    }
}

// Usage
const client = new BotClient('ws://localhost:8080/ws', 'auth-token');

client.on('connected', () => {
    client.subscribe('session-uuid');
});

client.on('message', (data) => {
    console.log('Bot:', data.content);
});

client.on('stream_chunk', (data) => {
    process.stdout.write(data.content);
});

client.on('error', (data) => {
    console.error('Error:', data.message);
});

client.connect();
client.sendMessage('session-uuid', 'Hello!');
```

## Meet WebSocket

Video conferencing uses a separate WebSocket endpoint:

```
ws://localhost:8080/ws/meet
```

### Join Room

```json
{
    "type": "join",
    "room_id": "room-uuid",
    "participant_name": "John"
}
```

### Leave Room

```json
{
    "type": "leave",
    "room_id": "room-uuid"
}
```

### Signaling (WebRTC)

```json
{
    "type": "signal",
    "room_id": "room-uuid",
    "target_id": "participant-uuid",
    "signal": { /* WebRTC signal data */ }
}
```

## Connection Limits

| Limit | Value |
|-------|-------|
| Max connections per IP | 100 |
| Max message size | 64 KB |
| Idle timeout | 5 minutes |
| Ping interval | 30 seconds |

## Error Codes

| Code | Description |
|------|-------------|
| `AUTH_FAILED` | Invalid or expired token |
| `SESSION_NOT_FOUND` | Session doesn't exist |
| `RATE_LIMITED` | Too many messages |
| `MESSAGE_TOO_LARGE` | Exceeds 64 KB limit |
| `INVALID_FORMAT` | Malformed JSON |
| `SUBSCRIPTION_FAILED` | Cannot subscribe to session |