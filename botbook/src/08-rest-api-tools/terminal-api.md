# Terminal API 🟡 BETA

> **WebSocket-based terminal for interactive shell sessions**

---

## Base URL

```
/api/terminal
```

## Authentication

All endpoints require a valid session token via `Authorization: Bearer <token>` header.

---

## Endpoints

### WebSocket Connection

**`GET /api/terminal/ws`**

Opens a WebSocket connection for an interactive terminal session.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `session_id` | string | No | Optional session ID to resume a previous terminal session |

**Protocol:**
- Connect via WebSocket to `ws://host/api/terminal/ws?token=<session_token>`
- Send JSON messages to execute commands
- Receive JSON messages with command output

**Message Format (Client → Server):**
```json
{
  "type": "input",
  "data": "ls -la\n"
}
```

**Message Format (Server → Client):**
```json
{
  "type": "output",
  "data": "total 48\ndrwxr-xr-x  6 user user 4096 Jan 15 10:30 .\n"
}
```

**Message Types:**

| Type | Direction | Description |
|------|-----------|-------------|
| `input` | Client → Server | Command input or keystroke |
| `output` | Server → Client | Command output or error |
| `resize` | Client → Server | Terminal resize event |
| `exit` | Both | Terminal session ended |

**Resize Message:**
```json
{
  "type": "resize",
  "cols": 120,
  "rows": 40
}
```

---

### List Sessions

**`GET /api/terminal/list`**

Returns all active terminal sessions.

**Response:**
```json
{
  "success": true,
  "sessions": [
    {
      "id": "term-a1b2c3",
      "user_id": "u123",
      "pid": 45678,
      "started_at": "2026-01-15T10:30:00Z",
      "last_active": "2026-01-15T10:35:00Z",
      "status": "running"
    }
  ]
}
```

---

### Create Session

**`POST /api/terminal/create`**

Creates a new terminal session without opening a WebSocket connection.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `shell` | string | No | Shell to use (default: system default, e.g., `/bin/bash`) |
| `workdir` | string | No | Initial working directory |
| `env` | object | No | Additional environment variables |

**Request Body:**
```json
{
  "shell": "/bin/bash",
  "workdir": "/opt/gbo",
  "env": {
    "TERM": "xterm-256color"
  }
}
```

**Response:**
```json
{
  "success": true,
  "session": {
    "id": "term-d4e5f6",
    "pid": 45679,
    "shell": "/bin/bash",
    "created_at": "2026-01-15T10:40:00Z"
  }
}
```

---

### Kill Session

**`POST /api/terminal/kill`**

Terminates an active terminal session.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `session_id` | string | Yes | ID of the terminal session to kill |

**Request Body:**
```json
{
  "session_id": "term-a1b2c3"
}
```

**Response:**
```json
{
  "success": true,
  "message": "Terminal session term-a1b2c3 terminated"
}
```

---

## Error Responses

| Status | Description |
|--------|-------------|
| `400` | Invalid request (missing required parameters) |
| `401` | Unauthorized (missing or invalid token) |
| `404` | Session not found |
| `429` | Rate limit exceeded (too many concurrent sessions) |
| `500` | Internal server error |

---

## Usage Example

```javascript
// Connect to terminal via WebSocket
const ws = new WebSocket('ws://localhost:8080/api/terminal/ws?token=mytoken');

ws.onopen = () => {
  // Send a command
  ws.send(JSON.stringify({ type: 'input', data: 'whoami\n' }));
};

ws.onmessage = (event) => {
  const msg = JSON.parse(event.data);
  if (msg.type === 'output') {
    process.stdout.write(msg.data);
  }
};

// Resize terminal
ws.send(JSON.stringify({ type: 'resize', cols: 120, rows: 40 }));
```

---

## See Also

- [Security API](security-api.md) — Command execution security and SafeCommand
- [System API](system-api.md) — System status and version checks
