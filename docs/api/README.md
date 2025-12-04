# API Reference

General Bots exposes a REST API and WebSocket interface for integration with external systems.

## Base URL

```
http://localhost:8080/api
```

## Authentication

All API requests require authentication via Bearer token:

```bash
curl -H "Authorization: Bearer <token>" \
     http://localhost:8080/api/sessions
```

## Endpoints Overview

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/sessions` | GET | List active sessions |
| `/api/sessions` | POST | Create new session |
| `/api/sessions/:id` | GET | Get session details |
| `/api/sessions/:id/messages` | POST | Send message |
| `/api/drive/*` | * | File storage operations |
| `/api/tasks/*` | * | Task management |
| `/api/email/*` | * | Email operations |
| `/api/calendar/*` | * | Calendar/CalDAV |
| `/api/meet/*` | * | Video meetings |
| `/api/kb/*` | * | Knowledge base search |
| `/api/analytics/*` | * | Analytics dashboard |

## WebSocket

Real-time communication via WebSocket:

```
ws://localhost:8080/ws
```

### Connection

```javascript
const ws = new WebSocket('ws://localhost:8080/ws');
ws.onmessage = (event) => {
    const data = JSON.parse(event.data);
    console.log('Received:', data);
};
```

### Message Format

```json
{
    "type": "message",
    "session_id": "uuid",
    "content": "Hello bot",
    "timestamp": "2024-01-01T00:00:00Z"
}
```

## Response Format

All responses follow a consistent structure:

### Success

```json
{
    "success": true,
    "data": { ... }
}
```

### Error

```json
{
    "success": false,
    "error": {
        "code": "NOT_FOUND",
        "message": "Resource not found"
    }
}
```

## Rate Limiting

API requests are rate limited per IP:

| Endpoint Type | Requests/Second | Burst |
|--------------|-----------------|-------|
| Standard API | 100 | 200 |
| Auth endpoints | 10 | 20 |
| LLM endpoints | 5 | 10 |

## Detailed Documentation

- [REST Endpoints](rest-endpoints.md) - Complete endpoint reference
- [WebSocket API](websocket.md) - Real-time communication
- [HTMX Integration](htmx.md) - Frontend patterns