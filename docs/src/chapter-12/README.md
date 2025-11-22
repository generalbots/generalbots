# Chapter 12: REST API Reference

This chapter provides comprehensive documentation for all REST API endpoints available in BotServer. The API is organized into specialized modules, each handling specific functionality.

## API Overview

BotServer exposes a comprehensive REST API that enables integration with external systems, automation, and management of all platform features. All endpoints follow RESTful principles and return JSON responses.

### Base URL

```
http://localhost:3000/api
```

### Authentication

Most endpoints require authentication via session tokens or API keys. See [Chapter 11: Authentication](../chapter-11/README.md) for details.

### Response Format

All API responses follow a consistent format:

```json
{
  "success": true,
  "data": { ... },
  "message": "Optional message"
}
```

Error responses:

```json
{
  "success": false,
  "error": "Error description",
  "code": "ERROR_CODE"
}
```

## API Categories

### File & Document Management
Complete file operations including upload, download, copy, move, search, and document processing.
- [Files API](./files-api.md) - Basic file operations
- [Document Processing API](./document-processing.md) - Document conversion and manipulation

### User Management
Comprehensive user account management, profiles, security, and preferences.
- [Users API](./users-api.md) - User CRUD operations
- [User Security API](./user-security.md) - 2FA, sessions, devices

### Groups & Organizations
Group creation, membership management, permissions, and analytics.
- [Groups API](./groups-api.md) - Group operations
- [Group Membership API](./group-membership.md) - Member management

### Conversations & Communication
Real-time messaging, calls, screen sharing, and collaboration.
- [Conversations API](./conversations-api.md) - Chat and messaging
- [Calls API](./calls-api.md) - Voice and video calls
- [Whiteboard API](./whiteboard-api.md) - Collaborative whiteboard

### Email & Notifications
Email management, sending, and notification preferences.
- [Email API](./email-api.md) - Email operations
- [Notifications API](./notifications-api.md) - Push notifications

### Calendar & Tasks
Event scheduling, reminders, task management, and dependencies.
- [Calendar API](./calendar-api.md) - Event management
- [Tasks API](./tasks-api.md) - Task operations

### Storage & Data
Data persistence, backup, archival, and quota management.
- [Storage API](./storage-api.md) - Data operations
- [Backup API](./backup-api.md) - Backup and restore

### Analytics & Reporting
Dashboards, metrics collection, insights, and trend analysis.
- [Analytics API](./analytics-api.md) - Analytics operations
- [Reports API](./reports-api.md) - Report generation

### System Administration
System management, configuration, monitoring, and maintenance.
- [Admin API](./admin-api.md) - System administration
- [Monitoring API](./monitoring-api.md) - Health and metrics

### AI & Machine Learning
Text analysis, image processing, translation, and predictions.
- [AI API](./ai-api.md) - AI operations
- [ML API](./ml-api.md) - Machine learning

### Security & Compliance
Audit logs, compliance checking, threat scanning, and encryption.
- [Security API](./security-api.md) - Security operations
- [Compliance API](./compliance-api.md) - Compliance checking

## Quick Reference

### Common Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/health` | GET | Health check |
| `/files/list` | GET | List files |
| `/files/upload` | POST | Upload file |
| `/users/create` | POST | Create user |
| `/groups/create` | POST | Create group |
| `/conversations/create` | POST | Create conversation |
| `/analytics/dashboard` | GET | Get dashboard |
| `/admin/system/status` | GET | System status |

## Rate Limiting

API endpoints are rate-limited to prevent abuse:

- **Standard endpoints**: 1000 requests per hour per user
- **Heavy operations**: 100 requests per hour per user
- **Public endpoints**: 100 requests per hour per IP

Rate limit headers are included in responses:

```
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 999
X-RateLimit-Reset: 1640995200
```

## Error Codes

| Code | Description |
|------|-------------|
| 400 | Bad Request - Invalid input |
| 401 | Unauthorized - Authentication required |
| 403 | Forbidden - Insufficient permissions |
| 404 | Not Found - Resource doesn't exist |
| 429 | Too Many Requests - Rate limit exceeded |
| 500 | Internal Server Error - Server error |
| 503 | Service Unavailable - Service down |

## Pagination

List endpoints support pagination:

```
GET /users/list?page=1&per_page=20
```

Response includes pagination metadata:

```json
{
  "data": [...],
  "total": 1000,
  "page": 1,
  "per_page": 20,
  "total_pages": 50
}
```

## Filtering and Searching

Many endpoints support filtering:

```
GET /files/list?bucket=my-bucket&path=/documents
GET /users/search?query=john&role=admin
GET /events/list?start_date=2024-01-01&end_date=2024-12-31
```

## Webhooks

Subscribe to events via webhooks:

```
POST /webhooks/subscribe
{
  "url": "https://your-server.com/webhook",
  "events": ["user.created", "file.uploaded"]
}
```

## WebSocket API

Real-time communication via WebSocket:

```
ws://localhost:3000/ws
```

Events:
- `message` - New message received
- `status` - User status changed
- `typing` - User is typing
- `call` - Incoming call

## SDK Support

Official SDKs available:
- JavaScript/TypeScript
- Python
- Rust
- Go

## Next Steps

- [Files API Reference](./files-api.md) - Detailed file operations
- [Users API Reference](./users-api.md) - User management
- [Analytics API Reference](./analytics-api.md) - Analytics and reporting
- [Example Integrations](./examples.md) - Code examples

## API Versioning

Current API version: `v1`

Version is specified in the URL:
```
/api/v1/users/list
```

Legacy endpoints without version prefix default to v1 for backward compatibility.