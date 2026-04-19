# Monitoring API

The Monitoring API provides endpoints for real-time system monitoring, performance metrics, and health checks.

## Status: Roadmap

This API is on the development roadmap. The endpoints documented below represent the planned interface design.

## Base URL

```
http://localhost:9000/api/v1/monitoring
```

## Authentication

Uses the standard botserver authentication mechanism with appropriate role-based permissions.

## Endpoints

### System Health

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/monitoring/health` | Get overall system health |
| GET | `/api/v1/monitoring/health/live` | Kubernetes liveness probe |
| GET | `/api/v1/monitoring/health/ready` | Kubernetes readiness probe |

### Performance Metrics

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/monitoring/metrics` | Get all metrics (Prometheus format) |
| GET | `/api/v1/monitoring/metrics/summary` | Get metrics summary |
| GET | `/api/v1/monitoring/metrics/{metric_name}` | Get specific metric |

### Service Status

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/monitoring/services` | List all services status |
| GET | `/api/v1/monitoring/services/{service_id}` | Get specific service status |
| POST | `/api/v1/monitoring/services/{service_id}/restart` | Restart a service |

### Resource Usage

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/monitoring/resources` | Get resource usage overview |
| GET | `/api/v1/monitoring/resources/cpu` | Get CPU usage |
| GET | `/api/v1/monitoring/resources/memory` | Get memory usage |
| GET | `/api/v1/monitoring/resources/disk` | Get disk usage |
| GET | `/api/v1/monitoring/resources/network` | Get network statistics |

### Alert Configuration

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/monitoring/alerts` | List all alerts |
| POST | `/api/v1/monitoring/alerts` | Create a new alert rule |
| GET | `/api/v1/monitoring/alerts/{alert_id}` | Get alert details |
| PUT | `/api/v1/monitoring/alerts/{alert_id}` | Update alert rule |
| DELETE | `/api/v1/monitoring/alerts/{alert_id}` | Delete alert rule |
| GET | `/api/v1/monitoring/alerts/active` | Get currently firing alerts |

### Log Stream

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/monitoring/logs` | Get recent logs |
| GET | `/api/v1/monitoring/logs/stream` | Stream logs via WebSocket |
| GET | `/api/v1/monitoring/logs/search` | Search logs with query |

## Request Examples

### Check System Health

```bas
health = GET "/api/v1/monitoring/health"
TALK "System Status: " + health.status
TALK "Uptime: " + health.uptime
FOR EACH component IN health.components
    TALK component.name + ": " + component.status
NEXT
```

### Get Performance Metrics

```bas
metrics = GET "/api/v1/monitoring/metrics/summary"
TALK "Request Rate: " + metrics.requests_per_second + "/s"
TALK "Average Latency: " + metrics.avg_latency_ms + "ms"
TALK "Error Rate: " + metrics.error_rate + "%"
```

### Check Resource Usage

```bas
resources = GET "/api/v1/monitoring/resources"
TALK "CPU: " + resources.cpu.usage_percent + "%"
TALK "Memory: " + resources.memory.used_mb + "/" + resources.memory.total_mb + " MB"
TALK "Disk: " + resources.disk.used_gb + "/" + resources.disk.total_gb + " GB"
```

### Create Alert Rule

```bas
alert = NEW OBJECT
alert.name = "High CPU Alert"
alert.metric = "cpu_usage_percent"
alert.condition = ">"
alert.threshold = 80
alert.duration = "5m"
alert.severity = "warning"
alert.notify = ["ops@example.com"]

result = POST "/api/v1/monitoring/alerts", alert
TALK "Alert created: " + result.alert_id
```

### Get Active Alerts

```bas
alerts = GET "/api/v1/monitoring/alerts/active"
IF alerts.count > 0 THEN
    TALK "Active alerts: " + alerts.count
    FOR EACH alert IN alerts.items
        TALK alert.severity + ": " + alert.message
    NEXT
ELSE
    TALK "No active alerts"
END IF
```

### Search Logs

```bas
params = NEW OBJECT
params.query = "error"
params.level = "error"
params.start_time = "2025-01-01T00:00:00Z"
params.limit = 100

logs = GET "/api/v1/monitoring/logs/search?" + ENCODE_PARAMS(params)
FOR EACH log IN logs.entries
    TALK log.timestamp + " [" + log.level + "] " + log.message
NEXT
```

## Health Response Format

```json
{
  "status": "healthy",
  "uptime": "5d 12h 30m",
  "version": "6.1.0",
  "components": [
    {"name": "database", "status": "healthy", "latency_ms": 2},
    {"name": "cache", "status": "healthy", "latency_ms": 1},
    {"name": "storage", "status": "healthy", "latency_ms": 5},
    {"name": "llm", "status": "healthy", "latency_ms": 150}
  ]
}
```

## Metrics Format

Metrics are exposed in Prometheus format:

```
# HELP http_requests_total Total HTTP requests
# TYPE http_requests_total counter
http_requests_total{method="GET",status="200"} 12345

# HELP http_request_duration_seconds HTTP request latency
# TYPE http_request_duration_seconds histogram
http_request_duration_seconds_bucket{le="0.1"} 8000
http_request_duration_seconds_bucket{le="0.5"} 11000
http_request_duration_seconds_bucket{le="1"} 12000
```

## Alert Severity Levels

| Level | Description |
|-------|-------------|
| `info` | Informational, no action required |
| `warning` | Attention needed, not critical |
| `error` | Error condition, requires attention |
| `critical` | Critical, immediate action required |

## Response Codes

| Code | Description |
|------|-------------|
| 200 | Success |
| 201 | Alert created |
| 400 | Bad Request (invalid parameters) |
| 401 | Unauthorized |
| 403 | Forbidden (insufficient permissions) |
| 404 | Resource not found |
| 500 | Internal Server Error |
| 503 | Service Unavailable |

## Required Permissions

| Endpoint Category | Required Role |
|-------------------|---------------|
| Health Checks | Public (no auth for basic health) |
| Metrics | `monitor` or `admin` |
| Service Status | `monitor` or `admin` |
| Resource Usage | `monitor` or `admin` |
| Alert Configuration | `admin` |
| Logs | `admin` or `log_viewer` |