# Chapter 10: REST API Reference

HTTP API endpoints for integrating with BotServer.

## Base URL

```
http://localhost:8000/api/v1
```

## Authentication

```bash
Authorization: Bearer <token>
```

## Core Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/chat` | POST | Send message, get response |
| `/sessions` | GET | List active sessions |
| `/sessions/:id` | GET | Get session details |
| `/files` | POST | Upload file |
| `/files/:id` | GET | Download file |

## Quick Example

```bash
curl -X POST http://localhost:8000/api/v1/chat \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"message": "Hello", "session_id": "abc123"}'
```

## Response Format

```json
{
  "success": true,
  "data": { ... },
  "error": null
}
```

## Chapter Contents

- [Files API](./files-api.md) - Upload/download
- [Document Processing](./document-processing.md) - Text extraction
- [Users API](./users-api.md) - User management
- [Groups API](./groups-api.md) - Group management
- [Conversations API](./conversations-api.md) - Chat sessions
- [Calendar API](./calendar-api.md) - Scheduling
- [Tasks API](./tasks-api.md) - Task management
- [Storage API](./storage-api.md) - Object storage
- [Analytics API](./analytics-api.md) - Metrics
- [Admin API](./admin-api.md) - Administration
- [AI API](./ai-api.md) - LLM endpoints
- [Example Integrations](./examples.md) - Code samples

## See Also

- [API and Tooling](../chapter-09-api/README.md) - Tool definitions
- [Authentication](../chapter-12-auth/README.md) - Security