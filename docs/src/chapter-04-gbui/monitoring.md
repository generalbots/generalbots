# Monitoring Dashboard

The Monitoring Dashboard provides real-time visibility into your General Bots deployment, displaying system health, active sessions, and resource utilization in a clean tree-based interface.

## Live System Architecture

Your General Bots deployment is a living ecosystem of interconnected components. The diagram below shows how all services work together in real-time:

![Live Monitoring Organism](../assets/suite/live-monitoring-organism.svg)

This animated diagram illustrates BotServer at the center orchestrating all interactions, with the data layer on the left comprising PostgreSQL, Qdrant, and MinIO for storage. Services on the right include BotModels, Vault, and Cache for AI and security functionality. Analytics at the bottom shows InfluxDB collecting metrics. The animated connection flows represent real-time data packets moving between components.

## Overview

Access the Monitoring tab from the Suite interface to view active sessions and conversations, message throughput statistics, system resources including CPU, GPU, and memory utilization, service health status for all components, and bot activity metrics across your deployment.

## Dashboard Layout

The monitoring interface uses a hierarchical tree view for organized data display. The Sessions panel shows active connections, peak usage, and average duration. The Messages panel displays daily totals, hourly rates, and response times. The Resources panel presents CPU, Memory, GPU, and Disk utilization with visual progress bars. The Services panel indicates health status of PostgreSQL, Qdrant, Cache, Drive, and BotModels. The Active Bots panel lists all running bots with their respective session counts.

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

The dashboard refreshes automatically using HTMX polling at different intervals depending on the metric type. Session counts update every 5 seconds for immediate visibility into user activity. Message metrics refresh every 10 seconds to show current throughput. Resource usage updates every 15 seconds since hardware metrics change more gradually. Service health checks run every 30 seconds to detect component issues without excessive overhead.

## Accessing via API

Monitoring data is available programmatically through the REST API:

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

## Understanding Component Health

Each component in the system has specific health indicators that help identify potential issues before they impact users.

| Component | Health Check | Warning Signs |
|-----------|--------------|---------------|
| **PostgreSQL** | Connection count, query rate | > 80 connections, slow queries |
| **Qdrant** | Vector count, search latency | > 50ms search time |
| **MinIO** | Storage usage, object count | > 80% storage used |
| **BotModels** | Token usage, response latency | > 2s response time |
| **Vault** | Seal status, policy count | Unsealed without auth |
| **Cache** | Hit rate, memory usage | < 80% hit rate |
| **InfluxDB** | Write rate, retention | Write failures |

## Console Mode

In console mode, monitoring displays as text output suitable for terminal environments and SSH sessions:

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

Configure alert thresholds in `config.csv` to receive notifications when metrics exceed acceptable levels:

```csv
name,value
alert-cpu-threshold,80
alert-memory-threshold,85
alert-disk-threshold,90
alert-response-time-ms,5000
alert-email,admin@example.com
```

These are example configuration values that should be adjusted based on your infrastructure capacity and operational requirements.

## Bot-Specific Metrics

View metrics for individual bots by querying the bot-specific endpoint:

```
GET /api/monitoring/bots/{bot_id}
```

This returns message count for the specific bot, active sessions currently connected to it, average response time for that bot's interactions, knowledge base query statistics showing search performance, and tool execution counts indicating which tools are used most frequently.

## Historical Data

Access historical metrics to analyze trends and patterns over time:

```
GET /api/monitoring/history?period=24h
```

Supported periods include `1h` for the last hour with minute granularity, `24h` for the last 24 hours with hourly granularity, `7d` for the last 7 days with daily granularity, and `30d` for the last 30 days with daily granularity. Historical data helps identify patterns and plan capacity improvements.

## Performance Tips

When experiencing high CPU usage, check for complex BASIC scripts that may be computationally expensive, review LLM call frequency to identify unnecessary AI invocations, and consider enabling semantic caching to reduce redundant processing.

For high memory usage, reduce the `max-context-messages` configuration to limit conversation history size, clear unused KB collections that consume memory for vector storage, and restart services periodically to clear accumulated caches.

When response times are slow, enable semantic caching to serve repeated queries instantly, optimize KB document sizes by splitting large documents, and consider using faster LLM models that trade some quality for speed.

## Integration with External Tools

Export metrics in Prometheus format for integration with external monitoring systems:

```
GET /api/monitoring/prometheus
```

This endpoint is compatible with Prometheus for metrics collection, Grafana for visualization dashboards, Datadog for cloud monitoring, and New Relic for application performance management.

## Monitoring Best Practices

Check the live diagram regularly since the animated SVG shows real-time data flow and helps visualize system behavior. Set up alerts early rather than waiting for problems to occur before configuring notifications. Monitor trends in addition to absolute values because a gradual increase in CPU usage can be as significant as a sudden spike. Keep historical data by configuring InfluxDB retention policies to maintain useful history for capacity planning. Correlate metrics when troubleshooting since high response time combined with high CPU usually indicates a need for scaling.

## See Also

The monitoring tutorial provides step-by-step guidance for monitoring your bot effectively. Console mode documentation covers the command-line interface for terminal-based monitoring. Configuration options explain all available settings. The Monitoring API reference provides complete endpoint documentation for programmatic access.