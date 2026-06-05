# Meet API

> **Video conferencing, webinars, voice channels, and collaborative whiteboard management**

---

## Base URL

```
/api/meet
```

## Authentication

All endpoints require a valid session token via `Authorization: Bearer <token>` header.

---

## Endpoints

### Create Room

**`POST /api/meet/create`**

Creates a new video conference room.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | No | Room display name |
| `max_participants` | integer | No | Maximum allowed participants (default: 50) |
| `recording_enabled` | boolean | No | Enable automatic recording |
| `transcription_enabled` | boolean | No | Enable live transcription |
| `scheduled_at` | string | ISO 8601 | Schedule room for future time |
| `metadata` | object | No | Custom key-value pairs |

**Response:**
```json
{
  "id": "room_abc123",
  "name": "Weekly Standup",
  "url": "https://meet.example.com/room_abc123",
  "created_at": "2026-06-04T10:00:00Z",
  "participants": [],
  "status": "active",
  "recording_enabled": false,
  "transcription_enabled": false,
  "metadata": {}
}
```

---

### List Rooms

**`GET /api/meet/rooms`**

Returns all rooms accessible to the authenticated user.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `status` | string | No | Filter by status: `active`, `scheduled`, `ended` |
| `page` | integer | No | Page number (default: 1) |
| `limit` | integer | No | Results per page (default: 20, max: 100) |

**Response:**
```json
{
  "rooms": [
    {
      "id": "room_abc123",
      "name": "Weekly Standup",
      "status": "active",
      "participants_count": 5,
      "created_at": "2026-06-04T10:00:00Z"
    }
  ],
  "total": 42,
  "page": 1,
  "limit": 20
}
```

---

### Get Room Details

**`GET /api/meet/rooms/:id`**

Returns full details of a specific room.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Room identifier |

**Response:**
```json
{
  "id": "room_abc123",
  "name": "Weekly Standup",
  "url": "https://meet.example.com/room_abc123",
  "status": "active",
  "created_at": "2026-06-04T10:00:00Z",
  "scheduled_at": null,
  "recording_enabled": true,
  "transcription_enabled": true,
  "participants": [
    {
      "id": "user_001",
      "name": "João Silva",
      "joined_at": "2026-06-04T10:01:00Z",
      "is_muted": false
    }
  ],
  "metadata": {
    "project": "GeneralBots"
  }
}
```

---

### Join Room

**`POST /api/meet/rooms/:id/join`**

Joins an active room and returns connection credentials.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Room identifier |
| `display_name` | string | No | Name shown to other participants |

**Response:**
```json
{
  "room_id": "room_abc123",
  "token": "livekit_token_xyz",
  "url": "wss://livekit.example.com",
  "participant_id": "part_005",
  "ice_servers": [
    {
      "urls": ["stun:stun.example.com:3478"],
      "username": "",
      "credential": ""
    }
  ]
}
```

---

### Room Transcription

**`POST /api/meet/rooms/:id/transcription`**

Manages live transcription for a room.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Room identifier |
| `action` | string | Yes | `start` or `stop` |
| `language` | string | No | Transcription language code (default: `pt-BR`) |

**Response:**
```json
{
  "room_id": "room_abc123",
  "transcription_active": true,
  "language": "pt-BR",
  "stream_url": "wss://transcribe.example.com/room_abc123"
}
```

---

### Generate Token

**`POST /api/meet/token`**

Generates an authentication token for room access.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `room_id` | string | Yes | Room identifier |
| `identity` | string | Yes | Participant identity |
| `ttl` | integer | No | Token TTL in seconds (default: 86400) |
| `role` | string | No | `host` or `participant` (default: `participant`) |

**Response:**
```json
{
  "token": "eyJhbGciOiJIUzI1NiIs...",
  "expires_at": "2026-06-05T10:00:00Z"
}
```

---

### Send Invitation

**`POST /api/meet/invite`**

Sends an email or system invitation to join a room.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `room_id` | string | Yes | Room identifier |
| `recipients` | string[] | Yes | List of email addresses or user IDs |
| `message` | string | No | Custom invitation message |
| `scheduled_at` | string | ISO 8601 | Schedule invitation delivery |

**Response:**
```json
{
  "invitations_sent": 3,
  "recipients": ["user1@example.com", "user2@example.com"],
  "room_url": "https://meet.example.com/room_abc123"
}
```

---

### WebSocket Connection

**`GET /api/meet/ws`**

Establishes a WebSocket connection for real-time room events and signaling.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `room_id` | query | Yes | Room to connect to |
| `token` | query | Yes | Authentication token |

**WebSocket Messages (Server → Client):**
```json
{
  "event": "participant_joined",
  "data": {
    "participant_id": "part_005",
    "name": "João Silva"
  }
}
```

```json
{
  "event": "room_ended",
  "data": {
    "room_id": "room_abc123",
    "reason": "host_ended"
  }
}
```

---

### Turn Credentials

**`GET /api/meet/turn-credentials`**

Returns TURN/STUN server credentials for NAT traversal.

**Response:**
```json
{
  "ice_servers": [
    {
      "urls": ["stun:stun.example.com:3478"]
    },
    {
      "urls": ["turn:turn.example.com:3478"],
      "username": "user_001",
      "credential": "temporary_secret",
      "ttl": 86400
    }
  ]
}
```

---

### Schedule Meeting

**`POST /api/meet/schedule`**

Schedules a meeting for a future time.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | Yes | Meeting title |
| `scheduled_at` | string | Yes | ISO 8601 datetime |
| `duration_minutes` | integer | No | Expected duration |
| `recurrence` | string | No | `daily`, `weekly`, `monthly`, `none` |
| `recurrence_end` | string | ISO 8601 | Recurrence end date |
| `participants` | string[] | No | List of user IDs to invite |
| `agenda` | string | No | Meeting agenda text |

**Response:**
```json
{
  "id": "room_sch_001",
  "name": "Project Review",
  "scheduled_at": "2026-06-10T14:00:00Z",
  "duration_minutes": 60,
  "recurrence": "weekly",
  "recurrence_end": "2026-07-10T14:00:00Z",
  "participants": ["user_001", "user_002"],
  "calendar_links": {
    "google": "https://calendar.google.com/calendar/event?...",
    "outlook": "https://outlook.live.com/calendar/0/deeplink/compose?..."
  }
}
```

---

### Dashboard Statistics

**`GET /api/meet/dashboard/stats`**

Returns aggregated meeting analytics.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `period` | string | No | `day`, `week`, `month` (default: `week`) |
| `start_date` | string | ISO 8601 | Custom period start |
| `end_date` | string | ISO 8601 | Custom period end |

**Response:**
```json
{
  "total_meetings": 142,
  "total_duration_minutes": 8520,
  "average_participants": 6.3,
  "total_participants": 895,
  "peak_hour": "14:00",
  "busiest_day": "Tuesday",
  "top_hosts": [
    {
      "user_id": "user_001",
      "name": "João Silva",
      "meetings_hosted": 28
    }
  ]
}
```

---

### Leave Room

**`POST /api/meet/rooms/:id/leave`**

Removes a participant from a room or ends the room if host.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Room identifier |
| `participant_id` | string | No | Specific participant to remove (host only) |
| `end_room` | boolean | No | End room for all participants (host only) |

**Response:**
```json
{
  "room_id": "room_abc123",
  "action": "left",
  "remaining_participants": 4
}
```

---

## Voice Channel API

### Start Voice

**`POST /api/voice/start`**

Starts a voice-only channel for audio conferencing.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | No | Channel name |
| `max_callers` | integer | No | Maximum callers (default: 20) |
| `recording` | boolean | No | Record the voice session |

**Response:**
```json
{
  "channel_id": "voice_001",
  "dial_in_number": "+5511999999999",
  "pin": "1234",
  "created_at": "2026-06-04T10:00:00Z"
}
```

---

### Stop Voice

**`POST /api/voice/stop`**

Terminates an active voice channel.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `channel_id` | string | Yes | Voice channel identifier |

**Response:**
```json
{
  "channel_id": "voice_001",
  "status": "terminated",
  "duration_seconds": 3600,
  "recording_url": "https://storage.example.com/recordings/voice_001.wav"
}
```

---

## Webinar API

### Create Webinar

**`POST /api/webinar/`**

Creates a new webinar session.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `title` | string | Yes | Webinar title |
| `description` | string | No | Webinar description |
| `scheduled_at` | string | Yes | ISO 8601 start time |
| `duration_minutes` | integer | Yes | Expected duration |
| `max_attendees` | integer | No | Maximum attendees (default: 100) |
| `registration_required` | boolean | No | Require registration (default: true) |
| `recording_enabled` | boolean | No | Record webinar |
| `panelists` | string[] | No | List of panelist user IDs |

**Response:**
```json
{
  "id": "webinar_001",
  "title": "Product Launch 2026",
  "description": "New features overview",
  "scheduled_at": "2026-06-15T10:00:00Z",
  "duration_minutes": 90,
  "max_attendees": 100,
  "status": "scheduled",
  "registration_url": "https://meet.example.com/webinar/001/register",
  "share_url": "https://meet.example.com/webinar/001/live"
}
```

---

### Get Webinar

**`GET /api/webinar/:id`**

Returns webinar details.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Webinar identifier |

**Response:**
```json
{
  "id": "webinar_001",
  "title": "Product Launch 2026",
  "status": "scheduled",
  "registered_count": 67,
  "panelists": ["user_001", "user_002"],
  "stream_url": null,
  "recording_url": null
}
```

---

### Start Webinar

**`POST /api/webinar/:id/start`**

Starts the live broadcast for a webinar.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Webinar identifier |

**Response:**
```json
{
  "id": "webinar_001",
  "status": "live",
  "stream_started_at": "2026-06-15T10:00:05Z",
  "stream_url": "https://live.example.com/webinar_001",
  "panelist_room_url": "https://meet.example.com/room_webinar_panel"
}
```

---

### End Webinar

**`POST /api/webinar/:id/end`**

Ends the live broadcast and optionally processes recording.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Webinar identifier |

**Response:**
```json
{
  "id": "webinar_001",
  "status": "ended",
  "duration_seconds": 5400,
  "recording_url": "https://storage.example.com/recordings/webinar_001.mp4",
  "attendees_total": 89,
  "peak_concurrent": 72
}
```

---

### Register for Webinar

**`POST /api/webinar/:id/register`**

Registers an attendee for a webinar.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Webinar identifier |
| `email` | string | Yes | Attendee email |
| `name` | string | Yes | Attendee name |
| `company` | string | No | Company name |

**Response:**
```json
{
  "registration_id": "reg_001",
  "webinar_id": "webinar_001",
  "email": "joao@example.com",
  "status": "confirmed",
  "join_url": "https://meet.example.com/webinar/001/join?token=abc"
}
```

---

### Join Webinar

**`POST /api/webinar/:id/join`**

Joins a live webinar as attendee or panelist.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Webinar identifier |
| `role` | string | No | `attendee` or `panelist` (default: `attendee`) |

**Response:**
```json
{
  "webinar_id": "webinar_001",
  "token": "livekit_token_xyz",
  "url": "wss://livekit.example.com",
  "role": "attendee",
  "features": {
    "can_speak": false,
    "can_share_screen": false,
    "can_chat": true
  }
}
```

---

## Whiteboard API

### Whiteboard WebSocket

**`GET /whiteboard/:id/ws`**

Establishes a WebSocket connection for collaborative whiteboard real-time synchronization.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | path | Yes | Whiteboard identifier |
| `token` | query | Yes | Authentication token |

**WebSocket Messages:**
```json
{
  "event": "draw",
  "data": {
    "user_id": "user_001",
    "shape": {
      "type": "rectangle",
      "x": 100,
      "y": 200,
      "width": 150,
      "height": 80,
      "color": "#3B82F6"
    }
  }
}
```

```json
{
  "event": "cursor",
  "data": {
    "user_id": "user_001",
    "x": 300,
    "y": 450
  }
}
```

---

### Create Whiteboard

**`GET /whiteboard/create/:conversation_id`**

Creates a new whiteboard session linked to a conversation.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `conversation_id` | path | Yes | Conversation identifier to link |

**Response:**
```json
{
  "whiteboard_id": "wb_001",
  "conversation_id": "conv_abc123",
  "created_at": "2026-06-04T10:00:00Z",
  "url": "https://meet.example.com/whiteboard/wb_001",
  "share_url": "https://meet.example.com/whiteboard/wb_001?share=true"
}
```

---

## See Also

- [Conversations API](conversations-api.md) — session and chat management
- [Attendance API](attendance-api.md) — queue and attendant operations
- [Users API](users-api.md) — user and authentication management
- [WebSocket Format](mcp-format.md) — WebSocket message protocol
