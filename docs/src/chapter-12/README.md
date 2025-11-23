# REST API Reference

BotServer provides a comprehensive REST API for bot management, user operations, and system administration. This chapter documents all available API endpoints and their usage.

## Overview

The BotServer REST API enables:
- Bot interaction and management
- User authentication and authorization
- Session management
- File operations
- Group and organization management
- System administration
- Analytics and monitoring

## Base URL

All API endpoints are served from:
```
http://localhost:8080/api
```

In production, replace with your domain and use HTTPS:
```
https://your-domain.com/api
```

## Authentication

Most API endpoints require authentication. BotServer supports:

### Session Token

Include the session token in the Authorization header:
```
Authorization: Bearer <session-token>
```

### Getting a Token

Authenticate through Zitadel OAuth flow or use the login endpoint to obtain a session token.

## API Categories

### Core APIs

- **[Files API](./files-api.md)** - File upload, download, and management
- **[Document Processing](./document-processing.md)** - Document parsing and indexing
- **[Users API](./users-api.md)** - User management and profiles
- **[Groups API](./groups-api.md)** - Group and organization management
- **[Admin API](./admin-api.md)** - System administration endpoints

### Communication APIs

- **[Email API](./email-api.md)** - Email integration and management
- **[Notifications API](./notifications-api.md)** - Push notifications and alerts
- **[Conversations API](./conversations-api.md)** - Chat history and messages
- **[Calls API](./calls-api.md)** - Voice and video call management

### Productivity APIs

- **[Calendar API](./calendar-api.md)** - Calendar and scheduling
- **[Tasks API](./tasks-api.md)** - Task management
- **[Whiteboard API](./whiteboard-api.md)** - Collaborative whiteboard

### Data & Analytics APIs

- **[Storage API](./storage-api.md)** - Object storage operations
- **[Analytics API](./analytics-api.md)** - Usage analytics and metrics
- **[Reports API](./reports-api.md)** - Report generation
- **[Backup API](./backup-api.md)** - Backup and restore operations

### AI & ML APIs

- **[AI API](./ai-api.md)** - AI model interactions
- **[ML API](./ml-api.md)** - Machine learning operations

### Security & Compliance APIs

- **[Security API](./security-api.md)** - Security operations
- **[User Security](./user-security.md)** - User security settings
- **[Compliance API](./compliance-api.md)** - Compliance management
- **[Group Membership](./group-membership.md)** - Access control

### Monitoring & Operations

- **[Monitoring API](./monitoring-api.md)** - System monitoring
- **[Example Integrations](./examples.md)** - Integration examples

## Request Format

### Headers

Standard headers for API requests:
```
Content-Type: application/json
Authorization: Bearer <token>
Accept: application/json
```

### Request Body

JSON format for request bodies:
```json
{
  "field1": "value1",
  "field2": "value2"
}
```

## Response Format

### Success Response

```json
{
  "success": true,
  "data": {
    // Response data
  },
  "message": "Operation successful"
}
```

### Error Response

```json
{
  "success": false,
  "error": {
    "code": "ERROR_CODE",
    "message": "Human readable error message",
    "details": {
      // Additional error details
    }
  }
}
```

### HTTP Status Codes

| Code | Meaning |
|------|---------|
| 200 | Success |
| 201 | Created |
| 204 | No Content |
| 400 | Bad Request |
| 401 | Unauthorized |
| 403 | Forbidden |
| 404 | Not Found |
| 409 | Conflict |
| 422 | Unprocessable Entity |
| 429 | Too Many Requests |
| 500 | Internal Server Error |
| 503 | Service Unavailable |

## Common Parameters

### Pagination

Many endpoints support pagination:
```
GET /api/resource?limit=20&offset=0
```

- `limit` - Number of results per page (default: 20, max: 100)
- `offset` - Number of results to skip

### Sorting

Sort results by field:
```
GET /api/resource?sort=created_at&order=desc
```

- `sort` - Field to sort by
- `order` - Sort direction (`asc` or `desc`)

### Filtering

Filter results:
```
GET /api/resource?filter[status]=active&filter[type]=user
```

## Rate Limiting

API endpoints are rate-limited to prevent abuse:

- **Anonymous**: 60 requests per hour
- **Authenticated**: 1000 requests per hour
- **Admin**: 5000 requests per hour

Rate limit headers:
```
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 999
X-RateLimit-Reset: 1516131600
```

## Versioning

The API is versioned through the URL path:
```
/api/v1/endpoint  (current)
/api/v2/endpoint  (future)
```

Currently, all endpoints use v1 implicitly.

## WebSocket API

For real-time communication:
```
ws://localhost:8080/ws
```

See the WebSocket documentation for details.

## Error Codes

Common error codes across all APIs:

| Code | Description |
|------|-------------|
| `UNAUTHORIZED` | Missing or invalid authentication |
| `FORBIDDEN` | Insufficient permissions |
| `NOT_FOUND` | Resource not found |
| `VALIDATION_ERROR` | Invalid input data |
| `RATE_LIMITED` | Too many requests |
| `SERVER_ERROR` | Internal server error |
| `SERVICE_UNAVAILABLE` | Service temporarily unavailable |

## SDK Support

While official SDKs are not yet available, the API is designed to be easily consumed by:
- JavaScript/TypeScript
- Python
- Rust
- Go
- Any HTTP client library

## Testing

### Using cURL

```bash
curl -X GET \
  http://localhost:8080/api/users/me \
  -H 'Authorization: Bearer YOUR_TOKEN'
```

### Using Postman

Import the OpenAPI specification (when available) or manually configure requests.

## Best Practices

1. **Use HTTPS in Production**: Never send tokens over unencrypted connections
2. **Handle Errors Gracefully**: Always check for error responses
3. **Respect Rate Limits**: Implement exponential backoff
4. **Cache When Possible**: Reduce unnecessary API calls
5. **Use Pagination**: Don't request large datasets at once
6. **Validate Input**: Check data before sending
7. **Log Errors**: Track API errors for debugging

## Getting Help

- Check the specific endpoint documentation
- Review error messages and codes
- Enable debug logging
- Contact support or file an issue

## Note on Implementation Status

Some API endpoints documented in this chapter may be:
- Partially implemented
- Planned for future releases
- Subject to change

Always test endpoints in your development environment and check the source code for the most current implementation status.