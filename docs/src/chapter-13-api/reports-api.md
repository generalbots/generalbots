# Reports API

BotServer provides RESTful endpoints for generating, managing, and retrieving various types of reports and analytics.

## Overview

The Reports API enables:
- Conversation analytics
- Usage statistics
- Performance metrics
- Custom report generation
- Data export capabilities

## Base URL

```
http://localhost:8080/api/v1/reports
```

## Authentication

All Reports API requests require authentication:

```http
Authorization: Bearer <token>
```

## Endpoints

### Generate Report

**POST** `/generate`

Generate a new report based on specified criteria.

**Request Body:**
```json
{
  "type": "conversation_analytics",
  "date_range": {
    "start": "2024-01-01T00:00:00Z",
    "end": "2024-01-31T23:59:59Z"
  },
  "filters": {
    "bot_id": "default-bot",
    "channel": "web"
  },
  "format": "pdf"
}
```

**Response:**
```json
{
  "report_id": "rpt_abc123",
  "status": "processing",
  "estimated_completion": "2024-01-15T10:05:00Z",
  "download_url": null
}
```

### Get Report Status

**GET** `/reports/{report_id}`

Check the status of a report generation.

**Response:**
```json
{
  "report_id": "rpt_abc123",
  "status": "completed",
  "created_at": "2024-01-15T10:00:00Z",
  "completed_at": "2024-01-15T10:03:00Z",
  "download_url": "/api/v1/reports/rpt_abc123/download",
  "expires_at": "2024-01-22T10:03:00Z"
}
```

### Download Report

**GET** `/reports/{report_id}/download`

Download a completed report.

**Response:** Binary file (PDF, CSV, or Excel based on format)

### List Reports

**GET** `/reports`

List all generated reports.

**Query Parameters:**
- `page` - Page number (default: 1)
- `limit` - Items per page (default: 20)
- `status` - Filter by status
- `type` - Filter by report type

**Response:**
```json
{
  "reports": [
    {
      "report_id": "rpt_abc123",
      "type": "conversation_analytics",
      "status": "completed",
      "created_at": "2024-01-15T10:00:00Z",
      "size_bytes": 102400
    }
  ],
  "total": 42,
  "page": 1,
  "limit": 20
}
```

### Delete Report

**DELETE** `/reports/{report_id}`

Delete a generated report.

**Response:**
```json
{
  "deleted": true,
  "report_id": "rpt_abc123"
}
```

## Report Types

### Conversation Analytics

**Type:** `conversation_analytics`

Analyze conversation patterns and metrics.

**Generated Data:**
- Total conversations
- Average duration
- Messages per conversation
- User satisfaction scores
- Topic distribution
- Peak usage times

### Usage Statistics

**Type:** `usage_statistics`

Track system and resource usage.

**Generated Data:**
- Active users
- API calls
- Storage usage
- Token consumption
- Channel distribution
- Geographic distribution

### Performance Metrics

**Type:** `performance_metrics`

Monitor system performance.

**Generated Data:**
- Response times
- Error rates
- Uptime statistics
- Throughput metrics
- Cache hit rates
- Database performance

### Bot Analytics

**Type:** `bot_analytics`

Analyze individual bot performance.

**Generated Data:**
- Conversation count
- Success rate
- Popular intents
- Fallback rate
- User retention
- Engagement metrics

### Custom Reports

**Type:** `custom`

Create custom reports with SQL queries.

**Request Body:**
```json
{
  "type": "custom",
  "query": "SELECT * FROM conversations WHERE created_at > ?",
  "parameters": ["2024-01-01"],
  "format": "csv"
}
```

## Scheduled Reports

### Create Schedule

**POST** `/schedules`

Schedule recurring report generation.

**Request Body:**
```json
{
  "name": "Weekly Analytics",
  "report_config": {
    "type": "conversation_analytics",
    "filters": {"bot_id": "all"}
  },
  "schedule": "0 9 * * MON",
  "recipients": ["admin@example.com"],
  "format": "pdf"
}
```

**Response:**
```json
{
  "schedule_id": "sch_xyz789",
  "name": "Weekly Analytics",
  "next_run": "2024-01-22T09:00:00Z",
  "active": true
}
```

### List Schedules

**GET** `/schedules`

List all report schedules.

**Response:**
```json
{
  "schedules": [
    {
      "schedule_id": "sch_xyz789",
      "name": "Weekly Analytics",
      "cron": "0 9 * * MON",
      "active": true,
      "last_run": "2024-01-15T09:00:00Z",
      "next_run": "2024-01-22T09:00:00Z"
    }
  ]
}
```

### Update Schedule

**PATCH** `/schedules/{schedule_id}`

Update a report schedule.

**Request Body:**
```json
{
  "active": false
}
```

### Delete Schedule

**DELETE** `/schedules/{schedule_id}`

Delete a report schedule.

## Dashboard Data

### Get Dashboard Metrics

**GET** `/dashboard`

Get real-time dashboard metrics.

**Response:**
```json
{
  "metrics": {
    "active_sessions": 42,
    "messages_today": 1234,
    "average_response_time_ms": 250,
    "error_rate": 0.01
  },
  "charts": {
    "hourly_messages": [
      {"hour": 0, "count": 45},
      {"hour": 1, "count": 32}
    ],
    "channel_distribution": {
      "web": 60,
      "whatsapp": 30,
      "teams": 10
    }
  },
  "timestamp": "2024-01-15T10:30:00Z"
}
```

## Export Formats

### PDF Export

Professional reports with charts and branding.

```json
{
  "format": "pdf",
  "options": {
    "include_charts": true,
    "include_logo": true,
    "paper_size": "A4"
  }
}
```

### CSV Export

Raw data for further analysis.

```json
{
  "format": "csv",
  "options": {
    "delimiter": ",",
    "include_headers": true,
    "encoding": "UTF-8"
  }
}
```

### Excel Export

Formatted spreadsheets with multiple sheets.

```json
{
  "format": "excel",
  "options": {
    "include_charts": true,
    "separate_sheets": true
  }
}
```

### JSON Export

Structured data for programmatic access.

```json
{
  "format": "json",
  "options": {
    "pretty_print": true,
    "include_metadata": true
  }
}
```

## Error Responses

### 400 Bad Request
```json
{
  "error": "invalid_date_range",
  "message": "End date must be after start date"
}
```

### 404 Not Found
```json
{
  "error": "report_not_found",
  "message": "Report rpt_abc123 not found"
}
```

### 429 Too Many Requests
```json
{
  "error": "rate_limit_exceeded",
  "message": "Report generation limit exceeded",
  "retry_after": 3600
}
```

## Usage Examples

### Generate Monthly Report

```bash
curl -X POST \
  -H "Authorization: Bearer token123" \
  -H "Content-Type: application/json" \
  -d '{
    "type": "conversation_analytics",
    "date_range": {
      "start": "2024-01-01T00:00:00Z",
      "end": "2024-01-31T23:59:59Z"
    },
    "format": "pdf"
  }' \
  http://localhost:8080/api/v1/reports/generate
```

### Download Report

```bash
curl -X GET \
  -H "Authorization: Bearer token123" \
  http://localhost:8080/api/v1/reports/rpt_abc123/download \
  -o report.pdf
```

## Performance Tips

1. **Schedule Off-Peak**: Generate large reports during low-usage periods
2. **Use Filters**: Narrow scope to reduce processing time
3. **Cache Results**: Frequently accessed reports are cached
4. **Async Generation**: Large reports process in background
5. **Pagination**: Use pagination for list endpoints

## Rate Limits

| Operation | Limit | Window |
|-----------|-------|--------|
| Generate Report | 10/hour | Per user |
| Download Report | 100/hour | Per user |
| Dashboard Access | 60/minute | Per user |

## Related APIs

- [Analytics API](./analytics-api.md) - Real-time analytics
- [Monitoring API](./monitoring-api.md) - System monitoring
- [Storage API](./storage-api.md) - Report storage