# Monitoring Dashboard

The Monitoring Dashboard provides real-time visibility into your General Bots deployment, displaying system health, active sessions, and resource utilization in a clean tree-based interface.

## Overview

Access the Monitoring tab from the Suite interface to view:
- Active sessions and conversations
- Message throughput
- System resources (CPU, GPU, Memory)
- Service health status
- Bot activity metrics

## Dashboard Layout

The monitoring interface uses a hierarchical tree view for organized data display with the following panels:

- **Sessions** - Active connections, peak usage, average duration
- **Messages** - Daily totals, hourly rates, response times
- **Resources** - CPU, Memory, GPU, and Disk utilization with progress bars
- **Services** - Health status of PostgreSQL, Qdrant, Cache, Drive, BotModels
- **Active Bots** - List of running bots with session counts

## Metrics Explained

### Sessions

| Metric | Description |
|--------|-------------|
| **Active** | Current open conversations |
| **Peak Today** | Maximum concurrent sessions today |
| **Avg Duration** | Average session length |
| **Unique Users** | Distinct users today |

### Messages

| Metric | Description |
|--------|-------------|
| **Today** | Total messages processed today |
| **This Hour** | Messages in the current hour |
| **Avg Response** | Average bot response time |
| **Success Rate** | Percentage of successful responses |

### Resources

| Resource | Description | Warning Threshold |
|----------|-------------|-------------------|
| **CPU** | Processor utilization | > 80% |
| **Memory** | RAM usage | > 85% |
| **GPU** | Graphics processor (if available) | > 90% |
| **Disk** | Storage utilization | > 90% |

### Services

| Status | Indicator | Meaning |
|--------|-----------|---------|
| Running | Green dot | Service is healthy |
| Warning | Yellow dot | Service degraded |
| Stopped | Red dot | Service unavailable |

## Real-Time Updates

The dashboard refreshes automatically using HTMX polling:
- Session counts: Every 5 seconds
- Message metrics: Every 10 seconds
- Resource usage: Every 15 seconds
- Service health: Every 30 seconds

## Accessing via API

Monitoring data is available programmatically:

```
GET /api/monitoring/status
```

**Response:**
```json
{
  "sessions": {
    "active": 12,
    "peak_today": 47,
    "avg_duration_seconds": 512
  },
  "messages": {
    "today": 1234,
    "this_hour": 89,
    "avg_response_ms": 1200
  },
  "resources": {
    "cpu_percent": 78,
    "memory_percent": 62,
    "gpu_percent": 45,
    "disk_percent": 28
  },
  "services": {
    "postgresql": "running",
    "qdrant": "running",
    "cache": "running",
    "drive": "running",
    "botmodels": "stopped"
  }
}
```

## Console Mode

In console mode, monitoring displays as text output:

```bash
./botserver --console --monitor
```

Output:
```
[MONITOR] 2024-01-15 14:32:00
Sessions: 12 active (peak: 47)
Messages: 1,234 today (89/hour)
CPU: 78% | MEM: 62% | GPU: 45%
Services: 4/5 running
```

## Alerts Configuration

Configure alerts in `config.csv`:

```csv
key,value
alert-cpu-threshold,80
alert-memory-threshold,85
alert-disk-threshold,90
alert-response-time-ms,5000
alert-email,admin@example.com
```

## Bot-Specific Metrics

View metrics for individual bots:

```
GET /api/monitoring/bots/{bot_id}
```

Returns:
- Message count for this bot
- Active sessions for this bot
- Average response time
- KB query statistics
- Tool execution counts

## Historical Data

Access historical metrics:

```
GET /api/monitoring/history?period=24h
```

Supported periods:
- `1h` - Last hour (minute granularity)
- `24h` - Last 24 hours (hourly granularity)
- `7d` - Last 7 days (daily granularity)
- `30d` - Last 30 days (daily granularity)

## Performance Tips

1. **High CPU Usage**
   - Check for complex BASIC scripts
   - Review LLM call frequency
   - Consider semantic caching

2. **High Memory Usage**
   - Reduce `max-context-messages`
   - Clear unused KB collections
   - Restart services to clear caches

3. **Slow Response Times**
   - Enable semantic caching
   - Optimize KB document sizes
   - Use faster LLM models

## Integration with External Tools

Export metrics for external monitoring:

```
GET /api/monitoring/prometheus
```

Compatible with:
- Prometheus
- Grafana
- Datadog
- New Relic

## See Also

- [Console Mode](./console-mode.md) - Command-line interface
- [Settings](../chapter-08-config/README.md) - Configuration options
- [Monitoring API](../chapter-10-api/monitoring-api.md) - Full API reference