# Notifications API

BotServer provides RESTful endpoints for managing notifications across multiple channels including push notifications, in-app alerts, and message broadcasting.

## Overview

The Notifications API enables:
- Push notifications to users
- Broadcast messages to groups
- Alert management
- Notification preferences
- Delivery tracking

## Base URL

```
http://localhost:8080/api/v1/notifications
```

## Authentication

All Notifications API requests require authentication:

```http
Authorization: Bearer <token>
```

## Endpoints

### Send Notification

**POST** `/send`

Send a notification to one or more recipients.

**Request Body:**
```json
{
  "recipients": ["user123", "user456"],
  "title": "System Update",
  "message": "Maintenance scheduled for tonight",
  "priority": "normal",
  "channels": ["web", "email"],
  "data": {
    "action": "view_details",
    "url": "/maintenance"
  }
}
```

**Response:**
```json
{
  "notification_id": "ntf_abc123",
  "recipients_count": 2,
  "status": "queued",
  "delivery": {
    "web": "pending",
    "email": "pending"
  }
}
```

### Broadcast Message

**POST** `/broadcast`

Send a message to all users or a specific group.

**Request Body:**
```json
{
  "target": "all",
  "filters": {
    "channel": "web",
    "last_active": "7d"
  },
  "message": {
    "title": "New Feature Available",
    "body": "Check out our latest update!",
    "image_url": "https://example.com/feature.png"
  },
  "schedule": "2024-01-15T14:00:00Z"
}
```

**Response:**
```json
{
  "broadcast_id": "brd_xyz789",
  "target_count": 1250,
  "scheduled_for": "2024-01-15T14:00:00Z",
  "status": "scheduled"
}
```

### Get Notification Status

**GET** `/notifications/{notification_id}`

Get the status of a sent notification.

**Response:**
```json
{
  "notification_id": "ntf_abc123",
  "created_at": "2024-01-15T10:00:00Z",
  "status": "delivered",
  "delivery_details": [
    {
      "recipient": "user123",
      "channel": "web",
      "status": "delivered",
      "delivered_at": "2024-01-15T10:00:05Z"
    },
    {
      "recipient": "user123",
      "channel": "email",
      "status": "delivered",
      "delivered_at": "2024-01-15T10:00:10Z"
    }
  ]
}
```

### List Notifications

**GET** `/notifications`

List sent notifications with optional filters.

**Query Parameters:**
- `page` - Page number (default: 1)
- `limit` - Items per page (default: 20)
- `status` - Filter by status
- `channel` - Filter by channel
- `start_date` - Start date filter
- `end_date` - End date filter

**Response:**
```json
{
  "notifications": [
    {
      "notification_id": "ntf_abc123",
      "title": "System Update",
      "status": "delivered",
      "created_at": "2024-01-15T10:00:00Z",
      "recipients_count": 2
    }
  ],
  "total": 150,
  "page": 1,
  "limit": 20
}
```

### Mark as Read

**PATCH** `/notifications/{notification_id}/read`

Mark a notification as read by the current user.

**Response:**
```json
{
  "notification_id": "ntf_abc123",
  "marked_as_read": true,
  "read_at": "2024-01-15T10:05:00Z"
}
```

### Delete Notification

**DELETE** `/notifications/{notification_id}`

Delete a notification from the system.

**Response:**
```json
{
  "deleted": true,
  "notification_id": "ntf_abc123"
}
```

## User Preferences

### Get Preferences

**GET** `/users/{user_id}/preferences`

Get notification preferences for a user.

**Response:**
```json
{
  "user_id": "user123",
  "preferences": {
    "email": {
      "enabled": true,
      "frequency": "immediate"
    },
    "push": {
      "enabled": true,
      "quiet_hours": {
        "enabled": true,
        "start": "22:00",
        "end": "08:00"
      }
    },
    "sms": {
      "enabled": false
    },
    "categories": {
      "system": true,
      "marketing": false,
      "updates": true
    }
  }
}
```

### Update Preferences

**PATCH** `/users/{user_id}/preferences`

Update notification preferences.

**Request Body:**
```json
{
  "email": {
    "enabled": false
  },
  "push": {
    "quiet_hours": {
      "enabled": true,
      "start": "23:00",
      "end": "07:00"
    }
  }
}
```

## Notification Templates

### Create Template

**POST** `/templates`

Create a reusable notification template.

**Request Body:**
```json
{
  "name": "welcome_message",
  "title": "Welcome to {{app_name}}",
  "body": "Hi {{user_name}}, welcome to our platform!",
  "channels": ["email", "push"],
  "variables": ["app_name", "user_name"]
}
```

### Use Template

**POST** `/send/template`

Send notification using a template.

**Request Body:**
```json
{
  "template": "welcome_message",
  "recipients": ["user789"],
  "variables": {
    "app_name": "BotServer",
    "user_name": "John"
  }
}
```

## Notification Types

### System Notifications

Critical system messages and alerts.

```json
{
  "type": "system",
  "priority": "high",
  "persistent": true,
  "require_acknowledgment": true
}
```

### User Notifications

Personal messages and updates.

```json
{
  "type": "user",
  "priority": "normal",
  "expires_at": "2024-01-22T10:00:00Z"
}
```

### Broadcast Notifications

Mass communications to multiple users.

```json
{
  "type": "broadcast",
  "target": "segment",
  "segment_id": "active_users"
}
```

## Delivery Channels

### Web Push

Browser push notifications.

```json
{
  "channel": "web",
  "options": {
    "icon": "https://example.com/icon.png",
    "badge": "https://example.com/badge.png",
    "vibrate": [200, 100, 200],
    "require_interaction": false
  }
}
```

### Email

Email notifications with rich content.

```json
{
  "channel": "email",
  "options": {
    "from": "noreply@example.com",
    "reply_to": "support@example.com",
    "attachments": [],
    "html": true
  }
}
```

### SMS

Text message notifications.

```json
{
  "channel": "sms",
  "options": {
    "sender_id": "BOTSERV",
    "unicode": true
  }
}
```

### In-App

Notifications within the application.

```json
{
  "channel": "in_app",
  "options": {
    "persist": true,
    "category": "updates"
  }
}
```

## Webhook Events

### Delivery Events

Configure webhooks to receive delivery updates.

```json
{
  "event": "notification.delivered",
  "notification_id": "ntf_abc123",
  "recipient": "user123",
  "channel": "email",
  "delivered_at": "2024-01-15T10:00:10Z"
}
```

### Interaction Events

Track user interactions with notifications.

```json
{
  "event": "notification.clicked",
  "notification_id": "ntf_abc123",
  "user_id": "user123",
  "clicked_at": "2024-01-15T10:05:00Z",
  "action": "view_details"
}
```

## Error Responses

### 400 Bad Request
```json
{
  "error": "invalid_recipients",
  "message": "One or more recipients are invalid",
  "invalid_recipients": ["unknown_user"]
}
```

### 429 Rate Limit
```json
{
  "error": "rate_limit_exceeded",
  "message": "Notification send limit exceeded",
  "limit": 100,
  "window": "1h",
  "retry_after": 3600
}
```

## Usage Examples

### Send Simple Notification

```bash
curl -X POST \
  -H "Authorization: Bearer token123" \
  -H "Content-Type: application/json" \
  -d '{
    "recipients": ["user123"],
    "title": "Hello",
    "message": "This is a test notification"
  }' \
  http://localhost:8080/api/v1/notifications/send
```

### Schedule Broadcast

```bash
curl -X POST \
  -H "Authorization: Bearer token123" \
  -H "Content-Type: application/json" \
  -d '{
    "target": "all",
    "message": {
      "title": "Scheduled Maintenance",
      "body": "System will be unavailable from 2 AM to 4 AM"
    },
    "schedule": "2024-01-20T02:00:00Z"
  }' \
  http://localhost:8080/api/v1/notifications/broadcast
```

## Best Practices

1. **Batch Notifications**: Send to multiple recipients in one request
2. **Use Templates**: Maintain consistent messaging
3. **Respect Preferences**: Check user settings before sending
4. **Handle Failures**: Implement retry logic
5. **Track Delivery**: Monitor delivery rates
6. **Avoid Spam**: Rate limit and deduplicate messages

## Rate Limits

| Operation | Limit | Window |
|-----------|-------|--------|
| Send Notification | 100/hour | Per user |
| Broadcast | 10/day | Per account |
| Template Creation | 20/day | Per account |

## Related APIs

- [User API](./user-security.md) - User management
- [WebSocket API](../chapter-04/web-interface.md) - Real-time notifications
- [Email API](./keyword-send-mail.md) - Email notifications