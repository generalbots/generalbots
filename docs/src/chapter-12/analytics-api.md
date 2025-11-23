# Analytics API

BotServer provides RESTful endpoints for tracking, analyzing, and visualizing user interactions, bot performance, and business metrics.

## Overview

The Analytics API enables:
- User behavior tracking
- Conversation analytics
- Business metrics calculation
- Custom event tracking
- Funnel analysis
- Cohort analysis

## Base URL

```
http://localhost:8080/api/v1/analytics
```

## Authentication

All Analytics API requests require authentication:

```http
Authorization: Bearer <token>
```

## Endpoints

### Track Event

**POST** `/events`

Track a custom analytics event.

**Request Body:**
```json
{
  "event_name": "button_clicked",
  "user_id": "user123",
  "session_id": "sess_abc123",
  "properties": {
    "button_name": "submit",
    "page": "checkout",
    "value": 99.99
  },
  "timestamp": "2024-01-15T10:00:00Z"
}
```

**Response:**
```json
{
  "event_id": "evt_xyz789",
  "tracked": true,
  "timestamp": "2024-01-15T10:00:00Z"
}
```

### Batch Events

**POST** `/events/batch`

Track multiple events at once.

**Request Body:**
```json
{
  "events": [
    {
      "event_name": "page_view",
      "user_id": "user123",
      "properties": {"page": "home"}
    },
    {
      "event_name": "search",
      "user_id": "user123",
      "properties": {"query": "help"}
    }
  ]
}
```

**Response:**
```json
{
  "tracked": 2,
  "failed": 0,
  "event_ids": ["evt_001", "evt_002"]
}
```

### Get User Analytics

**GET** `/users/{user_id}/analytics`

Get analytics data for a specific user.

**Query Parameters:**
- `start_date` - Start of date range
- `end_date` - End of date range
- `metrics` - Comma-separated metrics to include

**Response:**
```json
{
  "user_id": "user123",
  "metrics": {
    "total_sessions": 45,
    "total_messages": 234,
    "average_session_duration": 450,
    "last_active": "2024-01-15T09:30:00Z",
    "satisfaction_score": 4.5,
    "topics_discussed": ["support", "billing", "features"]
  },
  "engagement": {
    "daily_active": true,
    "retention_30d": 0.85,
    "churn_risk": "low"
  }
}
```

### Conversation Analytics

**GET** `/conversations/analytics`

Get conversation-level analytics.

**Query Parameters:**
- `bot_id` - Filter by bot
- `period` - Time period: `today`, `week`, `month`, `custom`
- `group_by` - Grouping: `hour`, `day`, `week`, `month`

**Response:**
```json
{
  "summary": {
    "total_conversations": 1234,
    "completed_conversations": 1100,
    "average_duration_seconds": 360,
    "average_messages_per_conversation": 8,
    "resolution_rate": 0.89
  },
  "timeline": [
    {
      "date": "2024-01-15",
      "conversations": 156,
      "messages": 1248,
      "unique_users": 142
    }
  ],
  "topics": [
    {"topic": "technical_support", "count": 456, "percentage": 37},
    {"topic": "billing", "count": 234, "percentage": 19}
  ]
}
```

### Funnel Analysis

**POST** `/funnels`

Analyze user progression through defined steps.

**Request Body:**
```json
{
  "name": "onboarding_funnel",
  "steps": [
    {"name": "signup", "event": "user_registered"},
    {"name": "profile", "event": "profile_completed"},
    {"name": "first_action", "event": "first_conversation"}
  ],
  "date_range": {
    "start": "2024-01-01",
    "end": "2024-01-31"
  }
}
```

**Response:**
```json
{
  "funnel": {
    "name": "onboarding_funnel",
    "total_users": 1000,
    "steps": [
      {
        "name": "signup",
        "users": 1000,
        "percentage": 100
      },
      {
        "name": "profile",
        "users": 750,
        "percentage": 75,
        "drop_off": 250
      },
      {
        "name": "first_action",
        "users": 600,
        "percentage": 60,
        "drop_off": 150
      }
    ],
    "completion_rate": 0.60
  }
}
```

### Cohort Analysis

**POST** `/cohorts`

Analyze user cohorts over time.

**Request Body:**
```json
{
  "cohort_type": "acquisition",
  "group_by": "week",
  "metric": "retention",
  "date_range": {
    "start": "2024-01-01",
    "end": "2024-01-31"
  }
}
```

**Response:**
```json
{
  "cohorts": [
    {
      "cohort_date": "2024-01-01",
      "users": 250,
      "retention": {
        "week_0": 1.0,
        "week_1": 0.65,
        "week_2": 0.45,
        "week_3": 0.38,
        "week_4": 0.35
      }
    }
  ]
}
```

### Real-time Analytics

**GET** `/realtime`

Get real-time analytics data.

**Response:**
```json
{
  "current": {
    "active_users": 234,
    "active_conversations": 89,
    "messages_per_minute": 45,
    "average_response_time_ms": 250
  },
  "trends": {
    "users_change": "+12%",
    "conversations_change": "+8%",
    "satisfaction_change": "+2%"
  },
  "alerts": [
    {
      "type": "high_traffic",
      "message": "Traffic 50% above normal"
    }
  ],
  "timestamp": "2024-01-15T10:00:00Z"
}
```

### Custom Queries

**POST** `/query`

Execute custom analytics queries.

**Request Body:**
```json
{
  "query": "SELECT COUNT(*) as total, AVG(duration) as avg_duration FROM conversations WHERE bot_id = ? AND date >= ?",
  "parameters": ["bot_123", "2024-01-01"],
  "timeout_seconds": 30
}
```

**Response:**
```json
{
  "results": [
    {
      "total": 1234,
      "avg_duration": 360.5
    }
  ],
  "execution_time_ms": 125,
  "rows_returned": 1
}
```

### Export Analytics

**POST** `/export`

Export analytics data in various formats.

**Request Body:**
```json
{
  "type": "user_analytics",
  "format": "csv",
  "date_range": {
    "start": "2024-01-01",
    "end": "2024-01-31"
  },
  "filters": {
    "bot_id": "bot_123"
  }
}
```

**Response:**
```json
{
  "export_id": "exp_abc123",
  "status": "processing",
  "format": "csv",
  "download_url": null,
  "expires_at": null
}
```

## Metrics

### Standard Metrics

| Metric | Description | Type |
|--------|-------------|------|
| `sessions` | Number of user sessions | Counter |
| `messages` | Total messages exchanged | Counter |
| `duration` | Conversation duration | Histogram |
| `satisfaction` | User satisfaction score | Gauge |
| `resolution_rate` | Percentage of resolved issues | Percentage |
| `response_time` | Bot response time | Histogram |
| `fallback_rate` | Unhandled message rate | Percentage |

### Custom Metrics

Define custom metrics for specific business needs:

```json
{
  "metric_name": "conversion_rate",
  "type": "percentage",
  "calculation": "completed_purchases / total_conversations",
  "dimensions": ["bot_id", "channel", "user_segment"]
}
```

## Segmentation

### User Segments

Create and analyze user segments:

```json
{
  "segment_name": "power_users",
  "conditions": [
    {"field": "total_sessions", "operator": ">", "value": 10},
    {"field": "last_active", "operator": ">", "value": "-7d"}
  ]
}
```

### Behavioral Segments

Track behavior-based segments:

```json
{
  "segment_name": "at_risk",
  "conditions": [
    {"field": "days_since_last_active", "operator": ">", "value": 30},
    {"field": "previous_activity", "operator": "=", "value": "high"}
  ]
}
```

## Dashboards

### Create Dashboard

**POST** `/dashboards`

Create a custom analytics dashboard.

**Request Body:**
```json
{
  "name": "Executive Dashboard",
  "widgets": [
    {
      "type": "metric",
      "title": "Active Users",
      "metric": "active_users",
      "period": "today"
    },
    {
      "type": "chart",
      "title": "Conversations Over Time",
      "metric": "conversations",
      "visualization": "line",
      "group_by": "day"
    }
  ]
}
```

### Get Dashboard

**GET** `/dashboards/{dashboard_id}`

Retrieve dashboard with current data.

**Response:**
```json
{
  "dashboard_id": "dash_123",
  "name": "Executive Dashboard",
  "widgets": [
    {
      "widget_id": "w_001",
      "type": "metric",
      "title": "Active Users",
      "value": 234,
      "change": "+12%"
    }
  ],
  "last_updated": "2024-01-15T10:00:00Z"
}
```

## Webhooks

Configure webhooks for analytics events:

```json
{
  "webhook_url": "https://example.com/analytics",
  "events": ["milestone_reached", "anomaly_detected"],
  "filters": {
    "metric": "conversion_rate",
    "threshold": 0.1
  }
}
```

## Rate Limits

| Operation | Limit | Window |
|-----------|-------|--------|
| Track Event | 1000/minute | Per user |
| Batch Events | 100/minute | Per user |
| Query | 60/minute | Per user |
| Export | 10/hour | Per account |

## Best Practices

1. **Use Batch Tracking**: Send events in batches for efficiency
2. **Include Context**: Add relevant properties to events
3. **Consistent Naming**: Use consistent event naming conventions
4. **Sample High-Volume**: Sample events for high-traffic scenarios
5. **Set Up Alerts**: Configure alerts for important metrics
6. **Regular Exports**: Schedule regular data exports for backup

## Integration Examples

### JavaScript SDK

```javascript
// Track event
analytics.track('button_clicked', {
  button_name: 'subscribe',
  plan: 'premium',
  value: 99.99
});

// Identify user
analytics.identify('user123', {
  name: 'John Doe',
  email: 'john@example.com',
  plan: 'premium'
});
```

### Python Example

```python
import requests

# Track event
response = requests.post(
    'http://localhost:8080/api/v1/analytics/events',
    headers={'Authorization': 'Bearer token123'},
    json={
        'event_name': 'purchase_completed',
        'user_id': 'user123',
        'properties': {
            'amount': 99.99,
            'currency': 'USD',
            'items': 3
        }
    }
)
```

## Related APIs

- [Monitoring API](./monitoring-api.md) - System monitoring
- [Reports API](./reports-api.md) - Report generation
- [User API](./user-security.md) - User management