# REST API Endpoints

Complete reference for all General Bots REST API endpoints.

## Sessions

### List Sessions

```http
GET /api/sessions
```

**Query Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `limit` | integer | Max results (default: 50) |
| `offset` | integer | Pagination offset |
| `status` | string | Filter by status: `active`, `closed` |

**Response:**

```json
{
    "sessions": [
        {
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "user_id": "user@example.com",
            "bot_id": "default",
            "status": "active",
            "created_at": "2024-01-01T10:00:00Z",
            "last_activity": "2024-01-01T10:30:00Z"
        }
    ],
    "total": 1,
    "limit": 50,
    "offset": 0
}
```

### Create Session

```http
POST /api/sessions
```

**Request Body:**

```json
{
    "bot_id": "default",
    "user_id": "user@example.com",
    "metadata": {
        "channel": "web",
        "language": "en"
    }
}
```

### Get Session

```http
GET /api/sessions/:id
```

### Send Message

```http
POST /api/sessions/:id/messages
```

**Request Body:**

```json
{
    "content": "Hello, I need help",
    "type": "text"
}
```

**Response:**

```json
{
    "id": "msg-uuid",
    "session_id": "session-uuid",
    "role": "assistant",
    "content": "Hello! How can I assist you today?",
    "timestamp": "2024-01-01T10:31:00Z"
}
```

---

## Drive (File Storage)

### List Files

```http
GET /api/drive
GET /api/drive/:path
```

**Query Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `recursive` | boolean | Include subdirectories |
| `type` | string | Filter: `file`, `folder` |

### Upload File

```http
POST /api/drive/:path
Content-Type: multipart/form-data
```

### Download File

```http
GET /api/drive/:path/download
```

### Delete File

```http
DELETE /api/drive/:path
```

---

## Tasks

### List Tasks

```http
GET /api/tasks
```

**Query Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `status` | string | `pending`, `completed`, `cancelled` |
| `priority` | string | `low`, `medium`, `high` |
| `due_before` | datetime | Filter by due date |

### Create Task

```http
POST /api/tasks
```

**Request Body:**

```json
{
    "title": "Follow up with client",
    "description": "Send proposal document",
    "due_date": "2024-01-15T17:00:00Z",
    "priority": "high",
    "assignee": "user@example.com"
}
```

### Update Task

```http
PUT /api/tasks/:id
```

### Delete Task

```http
DELETE /api/tasks/:id
```

---

## Email

### List Emails

```http
GET /api/email
GET /api/email/:folder
```

**Folders:** `inbox`, `sent`, `drafts`, `trash`

### Send Email

```http
POST /api/email/send
```

**Request Body:**

```json
{
    "to": ["recipient@example.com"],
    "cc": [],
    "bcc": [],
    "subject": "Meeting Tomorrow",
    "body": "Hi, let's meet at 3pm.",
    "html": false,
    "attachments": []
}
```

### Get Email

```http
GET /api/email/:id
```

### Delete Email

```http
DELETE /api/email/:id
```

---

## Calendar

### List Events

```http
GET /api/calendar/events
```

**Query Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `start` | datetime | Range start |
| `end` | datetime | Range end |
| `calendar_id` | string | Specific calendar |

### Create Event

```http
POST /api/calendar/events
```

**Request Body:**

```json
{
    "title": "Team Meeting",
    "start_time": "2024-01-15T14:00:00Z",
    "end_time": "2024-01-15T15:00:00Z",
    "location": "Conference Room A",
    "attendees": ["team@example.com"],
    "recurrence": null
}
```

### Export Calendar (iCal)

```http
GET /api/calendar/export.ics
```

### Import Calendar

```http
POST /api/calendar/import
Content-Type: text/calendar
```

---

## Meet (Video Conferencing)

### Create Room

```http
POST /api/meet/rooms
```

**Request Body:**

```json
{
    "name": "Team Standup",
    "scheduled_start": "2024-01-15T09:00:00Z",
    "max_participants": 10
}
```

**Response:**

```json
{
    "room_id": "room-uuid",
    "join_url": "https://meet.example.com/room-uuid",
    "token": "participant-token"
}
```

### Join Room

```http
POST /api/meet/rooms/:id/join
```

### List Participants

```http
GET /api/meet/rooms/:id/participants
```

---

## Knowledge Base

### Search

```http
GET /api/kb/search
```

**Query Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `q` | string | Search query (required) |
| `limit` | integer | Max results (default: 10) |
| `threshold` | float | Similarity threshold (0-1) |
| `collection` | string | Specific KB collection |

**Response:**

```json
{
    "results": [
        {
            "id": "doc-uuid",
            "content": "Relevant document content...",
            "score": 0.95,
            "metadata": {
                "source": "company-docs",
                "title": "Employee Handbook"
            }
        }
    ],
    "query": "vacation policy",
    "total": 5
}
```

### List Collections

```http
GET /api/kb/collections
```

### Reindex Collection

```http
POST /api/kb/reindex
```

---

## Analytics

### Dashboard Stats

```http
GET /api/analytics/stats
```

**Query Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `time_range` | string | `day`, `week`, `month`, `year` |

**Response:**

```json
{
    "total_messages": 15420,
    "active_sessions": 45,
    "avg_response_time_ms": 230,
    "error_rate": 0.02,
    "top_queries": [
        {"query": "password reset", "count": 120}
    ]
}
```

### Message Trends

```http
GET /api/analytics/messages/trend
```

---

## Paper (Documents)

### List Documents

```http
GET /api/paper
```

### Create Document

```http
POST /api/paper
```

**Request Body:**

```json
{
    "title": "Meeting Notes",
    "content": "# Meeting Notes\n\n...",
    "type": "note"
}
```

### Get Document

```http
GET /api/paper/:id
```

### Update Document

```http
PUT /api/paper/:id
```

### Delete Document

```http
DELETE /api/paper/:id
```

---

## Designer (Bot Builder)

### List Dialogs

```http
GET /api/designer/dialogs
```

### Create Dialog

```http
POST /api/designer/dialogs
```

**Request Body:**

```json
{
    "name": "greeting",
    "content": "TALK \"Hello!\"\nanswer = HEAR"
}
```

### Validate Dialog

```http
POST /api/designer/dialogs/:id/validate
```

**Response:**

```json
{
    "valid": true,
    "errors": [],
    "warnings": ["Line 15: Consider using END IF"]
}
```

### Deploy Dialog

```http
POST /api/designer/dialogs/:id/deploy
```

---

## Sources (Templates)

### List Templates

```http
GET /api/sources/templates
```

**Query Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `category` | string | Filter by category |

### Get Template

```http
GET /api/sources/templates/:id
```

### Use Template

```http
POST /api/sources/templates/:id/use
```

---

## Admin

### System Stats

```http
GET /api/admin/stats
```

**Response:**

```json
{
    "uptime_seconds": 86400,
    "memory_used_mb": 512,
    "active_connections": 23,
    "database_size_mb": 1024,
    "cache_hit_rate": 0.85
}
```

### Health Check

```http
GET /api/health
```

**Response:**

```json
{
    "status": "healthy",
    "components": {
        "database": "ok",
        "cache": "ok",
        "storage": "ok",
        "llm": "ok"
    }
}
```

---

## Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `BAD_REQUEST` | 400 | Invalid request parameters |
| `UNAUTHORIZED` | 401 | Missing or invalid auth token |
| `FORBIDDEN` | 403 | Insufficient permissions |
| `NOT_FOUND` | 404 | Resource not found |
| `CONFLICT` | 409 | Resource already exists |
| `RATE_LIMITED` | 429 | Too many requests |
| `INTERNAL_ERROR` | 500 | Server error |

---

## Pagination

List endpoints support pagination:

```http
GET /api/tasks?limit=20&offset=40
```

Response includes pagination info:

```json
{
    "data": [...],
    "pagination": {
        "total": 150,
        "limit": 20,
        "offset": 40,
        "has_more": true
    }
}
```
