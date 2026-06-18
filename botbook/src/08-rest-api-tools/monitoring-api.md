# Monitoring API

> **Real-time system health, performance metrics, logs, analytics dashboards, and activity tracking.**

---

## Base URL

```
/api/monitoring
```

## Authentication

All endpoints require a valid session token via `Authorization: Bearer <token>` header. Public health check endpoints (`/health`) do not require authentication.

---

## Monitoring Endpoints

### Health Check

**`GET /api/monitoring/health`**

Returns overall system health status.

**Response:**

```json
{
  "status": "healthy",
  "uptime_seconds": 432100,
  "version": "6.1.0",
  "timestamp": "2025-06-04T12:00:00Z",
  "components": [
    {
      "name": "database",
      "status": "healthy",
      "latency_ms": 2,
      "details": "PostgreSQL 15.3 on tables.local:5432"
    },
    {
      "name": "cache",
      "status": "healthy",
      "latency_ms": 1,
      "details": "Valkey 7.2 on cache.local:6379"
    },
    {
      "name": "drive",
      "status": "healthy",
      "latency_ms": 5,
      "details": "MinIO on drive.local:9000"
    },
    {
      "name": "vectordb",
      "status": "healthy",
      "latency_ms": 3,
      "details": "Qdrant on vectordb.local:6333"
    },
    {
      "name": "llm",
      "status": "degraded",
      "latency_ms": 1520,
      "details": "Groq API responding slowly"
    }
  ]
}
```

---

### Performance Metrics

**`GET /api/monitoring/metrics`**

Returns system performance metrics.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `format` | string | No | `json` (default), `prometheus` |

**Response (json):**

```json
{
  "requests": {
    "total": 125430,
    "per_second": 42.5,
    "error_rate": 0.8,
    "latency": {
      "p50": 25,
      "p90": 85,
      "p95": 150,
      "p99": 320,
      "avg": 45
    }
  },
  "websockets": {
    "active": 28,
    "total_connections": 15420,
    "messages_per_second": 125.3
  },
  "llm": {
    "requests": 1250,
    "avg_response_time_ms": 2100,
    "tokens_processed": 3450000,
    "error_rate": 1.2
  },
  "database": {
    "connections_active": 8,
    "connections_idle": 12,
    "queries_per_second": 180,
    "slow_queries": 3
  },
  "memory": {
    "used_mb": 512,
    "total_mb": 2048,
    "usage_percent": 25.0
  },
  "collected_at": "2025-06-04T12:00:00Z"
}
```

---

### System Logs

**`GET /api/monitoring/logs`**

Retrieves recent log entries.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `level` | string | No | `error`, `warn`, `info`, `debug` |
| `component` | string | No | Filter by component name |
| `search` | string | No | Text search in log messages |
| `since` | string | No | ISO 8601 start time |
| `until` | string | No | ISO 8601 end time |
| `limit` | integer | No | Max entries (default: 100, max: 1000) |

**Response:**

```json
{
  "entries": [
    {
      "timestamp": "2025-06-04T12:00:00Z",
      "level": "info",
      "component": "drive_monitor",
      "message": "Bot 'default' synced from Drive",
      "metadata": {
        "bot": "default",
        "files_synced": 5,
        "duration_ms": 1200
      }
    },
    {
      "timestamp": "2025-06-04T11:59:45Z",
      "level": "warn",
      "component": "websocket",
      "message": "WebSocket message rate limit approached",
      "metadata": {
        "session_id": "sess_abc123",
        "rate": "8.5 msgs/s",
        "limit": "10 msgs/s"
      }
    },
    {
      "timestamp": "2025-06-04T11:59:30Z",
      "level": "error",
      "component": "llm",
      "message": "Groq API timeout after 30s",
      "metadata": {
        "model": "llama-3.3-70b-versatile",
        "timeout_ms": 30000
      }
    }
  ],
  "total": 1247,
  "has_more": true
}
```

---

## Analytics Endpoints

### Analytics Dashboard

**`GET /api/analytics/dashboard`**

Returns aggregated analytics data for the dashboard view.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `period` | string | No | `today`, `7d`, `30d`, `90d` (default: 7d) |

**Response:**

```json
{
  "period": "7d",
  "users": {
    "active": 156,
    "new": 23,
    "returning": 133
  },
  "conversations": {
    "total": 1240,
    "avg_per_user": 7.9,
    "avg_duration_seconds": 340,
    "completion_rate": 82.5
  },
  "messages": {
    "total": 8900,
    "user_messages": 4200,
    "bot_messages": 4700,
    "avg_response_time_ms": 1850
  },
  "tools": {
    "invocations": 320,
    "success_rate": 94.7,
    "most_used": [
      { "name": "check_inventory", "count": 85 },
      { "name": "create_report", "count": 62 },
      { "name": "send_email", "count": 41 }
    ]
  },
  "top_bots": [
    { "name": "default", "conversations": 680, "messages": 4800 },
    { "name": "sales", "conversations": 320, "messages": 2100 },
    { "name": "support", "conversations": 240, "messages": 2000 }
  ]
}
```

---

### Single Metric

**`GET /api/analytics/metric`**

Returns a specific metric with time-series data.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | Yes | Metric name: `conversations`, `messages`, `response_time`, `error_rate`, `tool_usage` |
| `period` | string | No | `1h`, `24h`, `7d`, `30d` (default: 24h) |
| `granularity` | string | No | `minute`, `hour`, `day` (default: auto) |

**Response:**

```json
{
  "metric": "conversations",
  "period": "24h",
  "granularity": "hour",
  "data": [
    { "timestamp": "2025-06-03T12:00:00Z", "value": 12 },
    { "timestamp": "2025-06-03T13:00:00Z", "value": 18 },
    { "timestamp": "2025-06-03T14:00:00Z", "value": 25 },
    { "timestamp": "2025-06-03T15:00:00Z", "value": 31 },
    { "timestamp": "2025-06-03T16:00:00Z", "value": 22 }
  ],
  "summary": {
    "total": 108,
    "avg": 18.0,
    "peak": 31,
    "peak_at": "2025-06-03T15:00:00Z"
  }
}
```

---

### Metrics List

**`GET /api/metrics`**

Lists all available system metrics with descriptions.

**Response:**

```json
{
  "metrics": [
    {
      "name": "http_requests_total",
      "type": "counter",
      "description": "Total HTTP requests processed",
      "labels": ["method", "status", "endpoint"]
    },
    {
      "name": "http_request_duration_ms",
      "type": "histogram",
      "description": "HTTP request latency in milliseconds",
      "labels": ["method", "endpoint"]
    },
    {
      "name": "websocket_connections_active",
      "type": "gauge",
      "description": "Currently active WebSocket connections"
    },
    {
      "name": "llm_tokens_total",
      "type": "counter",
      "description": "Total tokens processed by LLM",
      "labels": ["model", "bot"]
    },
    {
      "name": "database_query_duration_ms",
      "type": "histogram",
      "description": "Database query execution time",
      "labels": ["query_type"]
    },
    {
      "name": "drive_sync_total",
      "type": "counter",
      "description": "Total Drive sync operations",
      "labels": ["bot", "status"]
    },
    {
      "name": "memory_usage_bytes",
      "type": "gauge",
      "description": "Current memory usage in bytes"
    },
    {
      "name": "cpu_usage_percent",
      "type": "gauge",
      "description": "Current CPU usage percentage"
    }
  ]
}
```

---

### Recent Activity

**`GET /api/activity/recent`**

Returns recent system activity across all components.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `limit` | integer | No | Max entries (default: 20, max: 100) |
| `type` | string | No | Filter: `message`, `tool_call`, `login`, `config_change`, `deploy` |

**Response:**

```json
{
  "activities": [
    {
      "id": "act_001",
      "type": "message",
      "timestamp": "2025-06-04T12:00:00Z",
      "description": "User message in bot 'default'",
      "metadata": {
        "bot": "default",
        "session_id": "sess_abc123",
        "user_id": "usr_001"
      }
    },
    {
      "id": "act_002",
      "type": "tool_call",
      "timestamp": "2025-06-04T11:59:50Z",
      "description": "Tool 'check_inventory' executed",
      "metadata": {
        "bot": "default",
        "tool": "check_inventory",
        "duration_ms": 250,
        "success": true
      }
    },
    {
      "id": "act_003",
      "type": "config_change",
      "timestamp": "2025-06-04T11:45:00Z",
      "description": "Bot 'sales' config updated",
      "metadata": {
        "bot": "sales",
        "changed_by": "admin@company.com",
        "changes": ["llm-model"]
      }
    },
    {
      "id": "act_004",
      "type": "deploy",
      "timestamp": "2025-06-04T10:00:00Z",
      "description": "CI/CD deployment completed",
      "metadata": {
        "version": "6.1.0",
        "commit": "abc123f",
        "deployed_by": "alm-ci"
      }
    }
  ]
}
```

---

## Metrics Format

All metrics can also be returned in Prometheus exposition format:

```http
GET /api/monitoring/metrics?format=prometheus
```

```
# HELP http_requests_total Total HTTP requests processed
# TYPE http_requests_total counter
http_requests_total{method="GET",status="200",endpoint="/api/bots"} 12345

# HELP http_request_duration_ms HTTP request latency
# TYPE http_request_duration_ms histogram
http_request_duration_ms_bucket{le="10"} 5000
http_request_duration_ms_bucket{le="50"} 11000
http_request_duration_ms_bucket{le="100"} 12000
http_request_duration_ms_bucket{le="500"} 12400

# HELP websocket_connections_active Active WebSocket connections
# TYPE websocket_connections_active gauge
websocket_connections_active 28

# HELP llm_tokens_total Total LLM tokens processed
# TYPE llm_tokens_total counter
llm_tokens_total{model="llama-3.3-70b",bot="default"} 3450000
```

---

## Response Codes

| Code | Description |
|------|-------------|
| 200 | Success |
| 400 | Bad Request (invalid parameters) |
| 401 | Unauthorized |
| 403 | Forbidden |
| 404 | Metric not found |
| 500 | Internal Server Error |
| 503 | Service Unavailable |

---

## Alert Management

### List Alerts

**`GET /api/monitoring/alerts`**

Lists all active monitoring alerts.

**Response:**
```json
[
  {
    "id": "uuid",
    "rule_id": "uuid",
    "rule_name": "High CPU Usage",
    "severity": "Warning",
    "status": "Firing",
    "metric_name": "system_cpu_usage_percent",
    "metric_value": 92.5,
    "threshold": 90.0,
    "message": "High CPU Usage: system_cpu_usage_percent is 92.5 (threshold: 90.0)",
    "started_at": "2025-06-04T12:00:00Z",
    "resolved_at": null,
    "acknowledged_at": null
  }
]
```

### Get Alert

**`GET /api/monitoring/alerts/{id}`**

Returns a single alert by ID.

### Acknowledge Alert

**`POST /api/monitoring/alerts/{id}/acknowledge`**

Marks an alert as acknowledged.

### Silence Alert

**`POST /api/monitoring/alerts/{id}/silence?duration=3600`**

Silences an alert for the specified duration in seconds.

### Acknowledge All

**`POST /api/monitoring/alerts/acknowledge-all`**

Acknowledges all active alerts at once.

---

## Alert Rules

### List Rules

**`GET /api/monitoring/alerts/rules`**

Lists all configured alert rules.

### Get Rule

**`GET /api/monitoring/alerts/rules/{id}`**

Returns a single alert rule.

### Update Rule

**`PATCH /api/monitoring/alerts/rules/{id}`**

Updates an existing alert rule.

### Delete Rule

**`DELETE /api/monitoring/alerts/rules/{id}`**

Deletes an alert rule.

---

## Data Export

### Export Alerts History

**`GET /api/monitoring/alerts/history/export?range=24h`**

Exports alert history within the specified time range.

### Export Monitoring Data

**`GET /api/monitoring/export?range=24h`**

Exports all monitoring data (metrics and alerts) within the specified time range.

---

## See Also

- [Dashboards API](./dashboards-api.md) — custom dashboard creation
- [Compliance API](./compliance-api.md) — compliance audit trails
- [Admin API](./admin-api-full.md) — system administration
