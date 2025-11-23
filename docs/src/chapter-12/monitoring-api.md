# Monitoring API

BotServer provides RESTful endpoints for system monitoring, health checks, performance metrics, and operational insights.

## Overview

The Monitoring API enables:
- System health monitoring
- Performance metrics collection
- Resource usage tracking
- Service status checks
- Alert management
- Log aggregation

## Base URL

```
http://localhost:8080/api/v1/monitoring
```

## Authentication

Most monitoring endpoints require authentication:

```http
Authorization: Bearer <token>
```

Note: Health check endpoints may be accessible without authentication for load balancer integration.

## Endpoints

### Health Check

**GET** `/health`

Basic health check for load balancers and monitoring systems.

**Response:**
```json
{
  "status": "healthy",
  "timestamp": "2024-01-15T10:00:00Z",
  "version": "1.0.0"
}
```

### Detailed Health

**GET** `/health/detailed`

Comprehensive health status of all components.

**Response:**
```json
{
  "status": "healthy",
  "components": {
    "database": {
      "status": "healthy",
      "latency_ms": 2,
      "connection_pool": {
        "active": 5,
        "idle": 15,
        "max": 20
      }
    },
    "cache": {
      "status": "healthy",
      "latency_ms": 1,
      "memory_used_mb": 256,
      "hit_rate": 0.92
    },
    "storage": {
      "status": "healthy",
      "latency_ms": 15,
      "available_gb": 450,
      "used_gb": 50
    },
    "llm": {
      "status": "healthy",
      "model_loaded": true,
      "requests_per_second": 12
    }
  },
  "timestamp": "2024-01-15T10:00:00Z"
}
```

### System Metrics

**GET** `/metrics`

Get current system metrics.

**Response:**
```json
{
  "system": {
    "cpu": {
      "usage_percent": 45.2,
      "cores": 8,
      "load_average": [2.1, 2.5, 2.3]
    },
    "memory": {
      "total_gb": 32,
      "used_gb": 18.5,
      "free_gb": 13.5,
      "usage_percent": 57.8
    },
    "disk": {
      "total_gb": 500,
      "used_gb": 120,
      "free_gb": 380,
      "usage_percent": 24
    },
    "network": {
      "bytes_sent": 1048576000,
      "bytes_received": 524288000,
      "packets_sent": 1000000,
      "packets_received": 500000
    }
  },
  "timestamp": "2024-01-15T10:00:00Z"
}
```

### Application Metrics

**GET** `/metrics/application`

Get application-specific metrics.

**Response:**
```json
{
  "application": {
    "uptime_seconds": 86400,
    "requests": {
      "total": 150000,
      "rate_per_second": 25,
      "average_latency_ms": 150
    },
    "sessions": {
      "active": 234,
      "total_today": 1500,
      "average_duration_minutes": 15
    },
    "conversations": {
      "active": 89,
      "completed_today": 456,
      "average_messages": 12
    },
    "errors": {
      "total": 23,
      "rate_per_hour": 2,
      "last_error": "2024-01-15T09:45:00Z"
    }
  }
}
```

### Performance Metrics

**GET** `/metrics/performance`

Get detailed performance metrics.

**Query Parameters:**
- `period` - Time period: `1h`, `24h`, `7d`, `30d`
- `resolution` - Data resolution: `minute`, `hour`, `day`

**Response:**
```json
{
  "performance": {
    "response_times": {
      "p50": 120,
      "p95": 450,
      "p99": 800,
      "mean": 150,
      "min": 10,
      "max": 2000
    },
    "throughput": {
      "requests_per_second": 25,
      "messages_per_second": 50,
      "bytes_per_second": 102400
    },
    "error_rates": {
      "client_errors": 0.02,
      "server_errors": 0.001,
      "timeout_errors": 0.005
    }
  },
  "period": "24h",
  "resolution": "hour"
}
```

### Service Status

**GET** `/status`

Get status of all services.

**Response:**
```json
{
  "services": [
    {
      "name": "web_server",
      "status": "running",
      "uptime_seconds": 86400,
      "port": 8080,
      "pid": 1234
    },
    {
      "name": "database",
      "status": "running",
      "uptime_seconds": 172800,
      "port": 5432,
      "version": "14.5"
    },
    {
      "name": "cache",
      "status": "running",
      "uptime_seconds": 86400,
      "port": 6379,
      "memory_used_mb": 256
    },
    {
      "name": "llm_server",
      "status": "running",
      "uptime_seconds": 43200,
      "port": 8081,
      "model": "local-model"
    }
  ]
}
```

### Logs

**GET** `/logs`

Retrieve application logs.

**Query Parameters:**
- `level` - Log level filter: `error`, `warn`, `info`, `debug`
- `start_time` - Start timestamp
- `end_time` - End timestamp
- `service` - Service name filter
- `limit` - Maximum entries (default: 100)

**Response:**
```json
{
  "logs": [
    {
      "timestamp": "2024-01-15T10:00:00Z",
      "level": "info",
      "service": "web_server",
      "message": "Request processed successfully",
      "metadata": {
        "request_id": "req_123",
        "user_id": "user_456",
        "duration_ms": 150
      }
    },
    {
      "timestamp": "2024-01-15T09:59:55Z",
      "level": "error",
      "service": "database",
      "message": "Connection timeout",
      "metadata": {
        "query": "SELECT * FROM users",
        "timeout_ms": 5000
      }
    }
  ],
  "total": 2,
  "query": {
    "level": "all",
    "limit": 100
  }
}
```

### Alerts

**GET** `/alerts`

Get active alerts.

**Response:**
```json
{
  "alerts": [
    {
      "alert_id": "alt_123",
      "severity": "warning",
      "type": "high_cpu",
      "message": "CPU usage above 80% for 5 minutes",
      "triggered_at": "2024-01-15T09:55:00Z",
      "current_value": 85,
      "threshold": 80
    },
    {
      "alert_id": "alt_124",
      "severity": "critical",
      "type": "disk_space",
      "message": "Disk space below 10%",
      "triggered_at": "2024-01-15T09:30:00Z",
      "current_value": 8,
      "threshold": 10
    }
  ],
  "active_count": 2,
  "critical_count": 1,
  "warning_count": 1
}
```

### Configure Alert

**POST** `/alerts/configure`

Configure monitoring alerts.

**Request Body:**
```json
{
  "name": "high_memory_usage",
  "type": "threshold",
  "metric": "memory.usage_percent",
  "condition": "greater_than",
  "threshold": 90,
  "duration_seconds": 300,
  "severity": "warning",
  "notify": {
    "email": ["admin@example.com"],
    "webhook": "https://example.com/alerts"
  }
}
```

### Acknowledge Alert

**POST** `/alerts/{alert_id}/acknowledge`

Acknowledge an active alert.

**Request Body:**
```json
{
  "acknowledged_by": "admin",
  "notes": "Investigating the issue"
}
```

## Metrics Export

### Prometheus Format

**GET** `/metrics/prometheus`

Export metrics in Prometheus format.

**Response:**
```
# HELP botserver_requests_total Total number of requests
# TYPE botserver_requests_total counter
botserver_requests_total{method="GET",status="200"} 45678

# HELP botserver_request_duration_seconds Request duration
# TYPE botserver_request_duration_seconds histogram
botserver_request_duration_seconds_bucket{le="0.1"} 35678
botserver_request_duration_seconds_bucket{le="0.5"} 44678
botserver_request_duration_seconds_bucket{le="1.0"} 45678
```

### Grafana Dashboard

**GET** `/metrics/grafana`

Get Grafana dashboard configuration.

**Response:**
```json
{
  "dashboard": {
    "title": "BotServer Monitoring",
    "panels": [
      {
        "title": "Request Rate",
        "type": "graph",
        "datasource": "prometheus",
        "query": "rate(botserver_requests_total[5m])"
      }
    ]
  }
}
```

## WebSocket Monitoring

### Real-time Metrics Stream

**WebSocket** `/ws/metrics`

Connect to receive real-time metrics updates.

**Message Format:**
```json
{
  "type": "metric_update",
  "metric": "cpu_usage",
  "value": 45.2,
  "timestamp": "2024-01-15T10:00:00Z"
}
```

## Resource Tracking

### Database Connections

**GET** `/resources/database`

Monitor database connection pool.

**Response:**
```json
{
  "pool": {
    "max_connections": 20,
    "active_connections": 5,
    "idle_connections": 10,
    "waiting_requests": 0,
    "total_created": 100,
    "total_closed": 85
  },
  "queries": {
    "active": 2,
    "slow_queries": 1,
    "average_duration_ms": 15
  }
}
```

### Cache Statistics

**GET** `/resources/cache`

Monitor cache performance.

**Response:**
```json
{
  "cache": {
    "hits": 45678,
    "misses": 5678,
    "hit_rate": 0.89,
    "evictions": 234,
    "memory_used_mb": 256,
    "memory_limit_mb": 512,
    "keys_count": 1234
  }
}
```

## Error Tracking

### Error Summary

**GET** `/errors/summary`

Get error summary statistics.

**Response:**
```json
{
  "summary": {
    "total_errors": 234,
    "errors_by_type": {
      "database_error": 45,
      "validation_error": 89,
      "timeout_error": 23,
      "unknown_error": 77
    },
    "errors_by_service": {
      "web_server": 123,
      "background_jobs": 56,
      "webhooks": 55
    },
    "error_rate": 0.02,
    "period": "24h"
  }
}
```

## Debugging

### Debug Information

**GET** `/debug/info`

Get debug information (requires admin privileges).

**Response:**
```json
{
  "debug": {
    "goroutines": 42,
    "memory_allocations": 123456,
    "gc_runs": 234,
    "open_files": 45,
    "environment": "production",
    "config": {
      "debug_mode": false,
      "log_level": "info"
    }
  }
}
```

## Best Practices

1. **Set Up Alerts**: Configure alerts for critical metrics
2. **Monitor Trends**: Track metrics over time, not just current values
3. **Use Dashboards**: Create visual dashboards for quick insights
4. **Log Aggregation**: Centralize logs for easier troubleshooting
5. **Regular Health Checks**: Implement automated health monitoring
6. **Capacity Planning**: Use metrics for resource planning

## Integration

### Monitoring Tools

BotServer integrates with:
- Prometheus
- Grafana
- DataDog
- New Relic
- CloudWatch
- ELK Stack

### Alert Channels

Notifications can be sent to:
- Email
- Slack
- PagerDuty
- Webhooks
- SMS
- Microsoft Teams

## Rate Limits

| Endpoint | Limit | Window |
|----------|-------|--------|
| Health Check | Unlimited | - |
| Metrics | 60/minute | Per IP |
| Logs | 30/minute | Per user |
| Alerts | 10/minute | Per user |

## Related APIs

- [Analytics API](./analytics-api.md) - Business analytics
- [Reports API](./reports-api.md) - Report generation
- [System Status](../chapter-06/architecture.md) - Architecture details