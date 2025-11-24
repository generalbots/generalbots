# API Endpoints

General Bots exposes various API endpoints for authentication, session management, and bot interactions. All endpoints require proper authentication except public endpoints.

## Authentication Endpoints

Authentication is handled through Directory Service OAuth2/OIDC flow.

### OAuth Login

**GET** `/auth/login`

Initiates OAuth2 authentication flow with Zitadel.

- Redirects to Zitadel login page
- No body required
- Returns redirect response

### OAuth Callback

**GET** `/auth/callback`

Handles OAuth2 callback from Directory Service.

- Query parameters:
  - `code` - Authorization code from Directory Service
- `state` - State parameter for CSRF protection

Response:
- Sets session cookie
- Redirects to application

### Logout

**POST** `/auth/logout`

Terminates current session.

Headers:
- `Authorization: Bearer {session_token}`

Response:
```json
{
  "success": true,
  "message": "Logged out successfully"
}
```

### Session Validation

**GET** `/auth/validate`

Validates current session token.

Headers:
- `Authorization: Bearer {session_token}`

Response:
```json
{
  "valid": true,
  "user_id": "user-123",
  "expires_at": "2024-01-21T10:00:00Z"
}
```

## Session Management Endpoints

### Get Current Session

**GET** `/api/session`

Returns current session information.

Headers:
- `Authorization: Bearer {session_token}`

Response:
```json
{
  "session_id": "session-123",
  "user_id": "user-456",
  "bot_id": "bot-789",
  "created_at": "2024-01-20T10:00:00Z",
  "expires_at": "2024-01-21T10:00:00Z"
}
```

### Create Bot Session

**POST** `/api/session/create`

Creates a new bot session for authenticated user.

Headers:
- `Authorization: Bearer {session_token}`

Request:
```json
{
  "bot_id": "bot-123"
}
```

Response:
```json
{
  "session_id": "session-456",
  "bot_id": "bot-123",
  "status": "active"
}
```

### End Session

**DELETE** `/api/session/:id`

Terminates a specific session.

Headers:
- `Authorization: Bearer {session_token}`

Response:
```json
{
  "success": true,
  "message": "Session terminated"
}
```

## User Endpoints

### Get Current User

**GET** `/api/users/me`

Returns current user information.

Headers:
- `Authorization: Bearer {session_token}`

Response:
```json
{
  "user_id": "user-123",
  "username": "john_doe",
  "email": "john@example.com",
  "created_at": "2024-01-01T00:00:00Z"
}
```

### Update Profile

**PUT** `/api/users/me`

Updates current user profile.

Headers:
- `Authorization: Bearer {session_token}`

Request:
```json
{
  "email": "newemail@example.com",
  "first_name": "John",
  "last_name": "Doe"
}
```

Note: Actual update happens in Directory Service.

## Bot Interaction Endpoints

### Send Message (WebSocket)

**WebSocket** `/ws`

Real-time bot communication.

Protocol:
```javascript
// Connect
ws = new WebSocket('ws://localhost:8080/ws');

// Send
ws.send(JSON.stringify({
  type: 'message',
  content: 'Hello bot',
  session_id: 'session-123'
}));

// Receive
ws.onmessage = (event) => {
  const msg = JSON.parse(event.data);
  console.log(msg.content);
};
```

### List Available Bots

**GET** `/api/bots`

Lists bots available to current user.

Headers:
- `Authorization: Bearer {session_token}`

Response:
```json
{
  "bots": [
    {
      "id": "bot-123",
      "name": "Customer Service",
      "description": "Help with customer inquiries",
      "status": "online"
    }
  ]
}
```

## Admin Endpoints

See [Admin API](../chapter-12/admin-api.md) for detailed admin endpoints.

### System Status

**GET** `/api/admin/system/status`

Requires admin privileges.

### System Metrics

**GET** `/api/admin/system/metrics`

Requires admin privileges.

## Group Management Endpoints

See [Groups API](../chapter-12/groups-api.md) for detailed group endpoints.

### Create Group

**POST** `/api/groups/create`

### List Groups

**GET** `/api/groups/list`

### Get Group Members

**GET** `/api/groups/:id/members`

## Rate Limiting

All endpoints are rate-limited:

- Public endpoints: 60 requests/hour
- Authenticated: 1000 requests/hour  
- Admin: 5000 requests/hour

Rate limit headers:
- `X-RateLimit-Limit`
- `X-RateLimit-Remaining`
- `X-RateLimit-Reset`

## Error Responses

Standard error format:
```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Human readable message",
    "details": {}
  }
}
```

Common error codes:
- `UNAUTHORIZED` - Missing/invalid authentication
- `FORBIDDEN` - Insufficient permissions
- `NOT_FOUND` - Resource not found
- `RATE_LIMITED` - Too many requests
- `SERVER_ERROR` - Internal error

## CORS Configuration

CORS headers for browser access:
- `Access-Control-Allow-Origin: *` (development)
- `Access-Control-Allow-Methods: GET, POST, PUT, DELETE, OPTIONS`
- `Access-Control-Allow-Headers: Content-Type, Authorization`

Production should restrict origins.

## Health Check

**GET** `/health`

No authentication required.

Response:
```json
{
  "status": "healthy",
  "timestamp": "2024-01-20T10:00:00Z"
}
```

## Implementation Status

Implemented:
- WebSocket communication (`/ws`)
- Admin endpoints (see Admin API)
- Group management endpoints
- Health check

Partially Implemented:
- OAuth flow (via Directory Service)
- Session management

Not Yet Implemented:
- Some user profile endpoints
- Direct REST messaging endpoints
- Batch operations

## Security Notes

1. All endpoints except `/health` require authentication
2. Admin endpoints require admin role in Directory Service
3. Session tokens should be kept secure
4. Use HTTPS in production
5. Implement CSRF protection for state-changing operations

## Best Practices

1. Always include session token in Authorization header
2. Handle token expiration gracefully
3. Implement retry logic with exponential backoff
4. Cache responses when appropriate
5. Use WebSocket for real-time communication
6. Monitor rate limit headers