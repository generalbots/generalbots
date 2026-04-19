# Monitoring Dashboard

The Monitoring Dashboard is the **live operations homepage** for your General Bots deployment. It provides real-time visibility into system health, active sessions, and resource utilization through an animated, interactive SVG visualization.

## Live System Architecture

<img src="../assets/suite/live-monitoring-organism.svg" alt="Live Monitoring Dashboard" style="max-width: 100%; height: auto;">

The dashboard displays botserver at the center orchestrating all interactions, with animated data packets flowing between components:

- **Left Side (Data Layer)**: PostgreSQL, Qdrant vector database, and MinIO storage
- **Right Side (Services)**: BotModels AI, Cache, and Vault security
- **Center**: botserver core with pulsing rings indicating activity
- **Top**: Real-time metrics panels for sessions, messages, and response time
- **Bottom**: Resource utilization bars and activity ticker

---

## Accessing the Dashboard

The monitoring dashboard is the **default homepage** when accessing Suite:

```/dev/null/monitoring-url.txt#L1
http://localhost:9000/monitoring
```

Or from within Suite:
1. Click the apps menu (⋮⋮⋮)
2. Select **Monitoring**

---

## Real-Time Metrics

### Active Sessions Panel

Displays current conversation sessions:

```/dev/null/sessions-example.txt#L1-4
Active Sessions: 12
Peak Today: 47
Avg Duration: 8m 32s
Trend: ↑ +3 in last hour
```

### Messages Panel

Shows message throughput:

```/dev/null/messages-example.txt#L1-4
Today: 1,234 messages
This Hour: 89
Avg Response: 1.2s
Rate: 14.8 msg/min
```

### Resource Utilization

Real-time system resources:

| Resource | Current | Threshold |
|----------|---------|-----------|
| **CPU** | 65% | Warning > 80% |
| **Memory** | 72% | Warning > 85% |
| **GPU** | 45% | Warning > 90% |
| **Disk** | 28% | Warning > 90% |

---

## Service Health Status

Each service has a status indicator:

| Service | Status | Health Check |
|---------|--------|--------------|
| **PostgreSQL** | 🟢 Running | Connection pool, query latency |
| **Qdrant** | 🟢 Running | Vector count, search time |
| **MinIO** | 🟢 Running | Storage usage, object count |
| **BotModels** | 🟢 Running | Token usage, response time |
| **Cache** | 🟢 Running | Hit rate, memory usage |
| **Vault** | 🟢 Running | Seal status, policy count |

### Status Indicators

| Status | Color | Animation |
|--------|-------|-----------|
| **Running** | 🟢 Green | Gentle pulse |
| **Warning** | 🟡 Amber | Fast pulse |
| **Stopped** | 🔴 Red | No animation |

---

## Live Data Endpoints

The dashboard pulls real data from these HTMX endpoints:

| Endpoint | Interval | Data |
|----------|----------|------|
| `/api/monitoring/metric/sessions` | 5s | Session count, trend |
| `/api/monitoring/metric/messages` | 10s | Message count, rate |
| `/api/monitoring/metric/response_time` | 10s | Avg response time |
| `/api/monitoring/resources/bars` | 15s | CPU, memory, GPU, disk |
| `/api/monitoring/services/status` | 30s | Service health JSON |
| `/api/monitoring/activity/latest` | 5s | Activity ticker text |
| `/api/monitoring/bots/active` | 30s | Active bot list |

---

## API Access

### Full Status Endpoint

```/dev/null/api-call.txt#L1
GET /api/monitoring/status
```

Returns complete system status:

```/dev/null/monitoring-response.json#L1-25
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
    "cpu_percent": 65,
    "memory_percent": 72,
    "gpu_percent": 45,
    "disk_percent": 28
  },
  "services": {
    "postgresql": "running",
    "qdrant": "running",
    "cache": "running",
    "drive": "running",
    "botmodels": "running",
    "vault": "running"
  }
}
```

### Active Bots Endpoint

```/dev/null/bots-api.txt#L1
GET /api/monitoring/bots
```

Returns list of deployed bots with metrics:

```/dev/null/bots-response.json#L1-18
{
  "bots": [
    {
      "name": "default",
      "status": "active",
      "sessions_today": 34,
      "messages_today": 567,
      "avg_response_ms": 980
    },
    {
      "name": "support",
      "status": "active",
      "sessions_today": 12,
      "messages_today": 234,
      "avg_response_ms": 1100
    }
  ]
}
```

### Historical Data

```/dev/null/history-api.txt#L1
GET /api/monitoring/history?period=24h
```

Returns time-series data for charting.

### Prometheus Export

```/dev/null/prometheus-api.txt#L1
GET /api/monitoring/prometheus
```

Returns metrics in Prometheus format for external monitoring systems.

---

## View Modes

Toggle between two views using the grid button or press `V`:

### Live View (Default)

The animated SVG visualization showing the complete system topology with flowing data packets. This is the recommended view for operations dashboards.

### Grid View

Traditional panel-based layout with detailed metrics:

- **Sessions Panel**: Active, peak, average duration
- **Messages Panel**: Counts, rates, response times
- **Resources Panel**: Progress bars with thresholds
- **Services Panel**: Health status for each component
- **Bots Panel**: List of active bots with metrics

---

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `V` | Toggle between Live and Grid view |
| `R` | Refresh all metrics immediately |
| `F` | Toggle fullscreen mode |
| `?` | Show keyboard shortcuts |

---

## Alert Configuration

Configure alert thresholds in `config.csv`:

```/dev/null/config-alerts.csv#L1-6
name,value
alert-cpu-threshold,80
alert-memory-threshold,85
alert-disk-threshold,90
alert-response-time-ms,5000
alert-email,admin@example.com
```

When thresholds are exceeded:
1. Service status turns amber/red
2. Alert notification sent to configured email
3. Activity ticker shows alert message

---

## Console Mode Monitoring

For terminal-based monitoring or headless servers:

```/dev/null/console-command.sh#L1
./botserver --console --monitor
```

Output:

```/dev/null/console-output.txt#L1-6
[MONITOR] 2025-01-15 14:32:00
Sessions: 12 active (peak: 47)
Messages: 1,234 today (89/hour)
CPU: 65% | MEM: 72% | GPU: 45%
Services: 6/6 running
Latest: User enrolled in Computer Science course
```

---

## Component Health Details

| Component | Metrics Monitored | Warning Signs |
|-----------|-------------------|---------------|
| **PostgreSQL** | Connection count, query rate, replication lag | > 80 connections, queries > 100ms |
| **Qdrant** | Vector count, search latency, memory | > 50ms search, > 80% memory |
| **MinIO** | Storage usage, object count, bandwidth | > 80% storage, high error rate |
| **BotModels** | Token usage, response latency, queue depth | > 2s response, queue > 10 |
| **Vault** | Seal status, policy count, auth failures | Sealed, repeated auth failures |
| **Cache** | Hit rate, memory usage, evictions | < 80% hit rate, frequent evictions |

---

## Best Practices

1. **Keep monitoring visible** — Use a dedicated screen or dashboard monitor for operations
2. **Set appropriate thresholds** — Configure alerts before issues become critical
3. **Watch data flow** — Animated packets indicate active communication between components
4. **Monitor trends** — The session trend indicator (↑/↓) shows direction of change
5. **Use historical data** — Query `/api/monitoring/history` for trend analysis
6. **Enable Prometheus export** — Integrate with existing monitoring infrastructure

---

## Troubleshooting

### Dashboard Not Loading

1. Check browser console for errors
2. Verify `/api/monitoring/status` returns data
3. Ensure WebSocket connection is established
4. Refresh the page

### Metrics Showing "--"

1. Wait 5-10 seconds for initial data load
2. Check network tab for failed API requests
3. Verify all services are running
4. Check botserver logs for errors

### Animations Stuttering

1. Close unused browser tabs
2. Enable hardware acceleration in browser settings
3. Use Grid view for lower resource usage
4. Check if system CPU is overloaded

### Service Showing Red

1. Check service-specific logs in `botserver-stack/logs/`
2. Verify Vault is unsealed
3. Check database connection limits
4. Restart the affected service

---

## See Also

- [Console Mode](./console-mode.md) — Terminal-based interface
- [HTMX Architecture](./htmx-architecture.md) — How real-time updates work
- [Suite Manual](./suite-manual.md) — Complete user guide
- [Analytics App](./apps/analytics.md) — Business metrics and reporting