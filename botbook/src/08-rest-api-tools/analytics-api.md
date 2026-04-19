# Analytics API

The Analytics API provides endpoints for tracking, analyzing, and reporting on bot usage and performance metrics.

## Status: Roadmap

This API is on the development roadmap. The endpoints documented below represent the planned interface design.

## Base URL

```
http://localhost:9000/api/v1/analytics
```

## Authentication

Uses the standard botserver authentication mechanism with appropriate role-based permissions.

## Endpoints

### Usage Statistics

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/analytics/usage` | Get overall usage statistics |
| GET | `/api/v1/analytics/usage/daily` | Get daily usage breakdown |
| GET | `/api/v1/analytics/usage/monthly` | Get monthly usage summary |

### Conversation Metrics

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/analytics/conversations` | Get conversation metrics |
| GET | `/api/v1/analytics/conversations/volume` | Get conversation volume over time |
| GET | `/api/v1/analytics/conversations/duration` | Get average conversation duration |
| GET | `/api/v1/analytics/conversations/resolution` | Get resolution rate metrics |

### User Engagement

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/analytics/engagement` | Get user engagement metrics |
| GET | `/api/v1/analytics/engagement/retention` | Get user retention data |
| GET | `/api/v1/analytics/engagement/satisfaction` | Get satisfaction scores |

### Reports

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/v1/analytics/reports` | Generate a custom report |
| GET | `/api/v1/analytics/reports/{report_id}` | Get report by ID |
| GET | `/api/v1/analytics/reports` | List all reports |

### Real-time Metrics

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/analytics/realtime` | Get real-time metrics |
| GET | `/api/v1/analytics/realtime/active` | Get active sessions count |

## Request Examples

### Get Usage Statistics

```bas
stats = GET "/api/v1/analytics/usage"
TALK "Total conversations: " + stats.total_conversations
TALK "Active users: " + stats.active_users
```

### Get Daily Usage

```bas
daily = GET "/api/v1/analytics/usage/daily?days=7"
FOR EACH day IN daily.data
    TALK day.date + ": " + day.conversations + " conversations"
NEXT
```

### Generate Custom Report

```bas
report_config = NEW OBJECT
report_config.type = "engagement"
report_config.start_date = "2025-01-01"
report_config.end_date = "2025-01-31"
report_config.format = "pdf"

report = POST "/api/v1/analytics/reports", report_config
TALK "Report ID: " + report.id
```

### Get Real-time Metrics

```bas
realtime = GET "/api/v1/analytics/realtime"
TALK "Active sessions: " + realtime.active_sessions
TALK "Messages per minute: " + realtime.messages_per_minute
```

## Response Codes

| Code | Description |
|------|-------------|
| 200 | Success |
| 400 | Bad Request (invalid parameters) |
| 401 | Unauthorized |
| 403 | Forbidden (insufficient permissions) |
| 404 | Not Found |
| 500 | Internal Server Error |

## Query Parameters

### Time Range Filters

| Parameter | Type | Description |
|-----------|------|-------------|
| `start_date` | String | Start date (ISO 8601 format) |
| `end_date` | String | End date (ISO 8601 format) |
| `days` | Integer | Number of days to include |
| `period` | String | Predefined period (today, week, month, year) |

### Grouping Options

| Parameter | Type | Description |
|-----------|------|-------------|
| `group_by` | String | Group results by (hour, day, week, month) |
| `bot_id` | UUID | Filter by specific bot |
| `user_id` | UUID | Filter by specific user |

## Required Permissions

| Endpoint Category | Required Role |
|-------------------|---------------|
| Usage Statistics | `analytics_viewer` or `admin` |
| Conversation Metrics | `analytics_viewer` or `admin` |
| User Engagement | `analytics_viewer` or `admin` |
| Reports | `analytics_admin` or `admin` |
| Real-time Metrics | `analytics_viewer` or `admin` |