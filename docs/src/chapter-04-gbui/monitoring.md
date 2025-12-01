# Monitoring Dashboard

The Monitoring Dashboard provides real-time visibility into your General Bots deployment through an animated, interactive SVG visualization showing system health, active sessions, and resource utilization.

## Live System Visualization

<img src="../assets/suite/live-monitoring-organism.svg" alt="Live Monitoring Dashboard" style="max-width: 100%; height: auto;">

The dashboard displays BotServer at the center orchestrating all interactions, with animated data packets flowing between components:

- **Left Side (Data Layer)**: PostgreSQL, Qdrant vector database, and MinIO storage
- **Right Side (Services)**: BotModels AI, Cache, and Vault security
- **Center**: BotServer core with pulsing rings indicating activity
- **Top**: Real-time metrics panels for sessions, messages, and response time
- **Bottom**: Resource utilization bars and activity ticker

## Accessing the Dashboard

Access monitoring from the Suite interface:
1. Click the apps menu (⋮⋮⋮)
2. Select **Monitoring**
3. Or navigate directly to `/monitoring`

## Dashboard Features

### Animated System Architecture

The SVG visualization shows real-time data flow:

| Component | Color | Description |
|-----------|-------|-------------|
| **BotServer** | Blue/Purple | Central orchestrator with rotating ring |
| **PostgreSQL** | Blue | Primary database with cylinder icon |
| **Qdrant** | Purple | Vector database with triangle nodes |
| **MinIO** | Amber | Object storage with disk icon |
| **BotModels** | Pink | AI/ML service with neural network icon |
| **Cache** | Cyan | In-memory cache with lightning icon |
| **Vault** | Green | Secrets management with lock icon |

### Status Indicators

Each service has a status dot:

| Status | Color | Animation |
|--------|-------|-----------|
| **Running** | 🟢 Green | Gentle pulse |
| **Warning** | 🟡 Amber | Fast pulse |
| **Stopped** | 🔴 Red | No animation |

### Real-Time Metrics

Three metric panels at the top update automatically:

| Panel | Update Interval | Description |
|-------|-----------------|-------------|
| **Active Sessions** | 5 seconds | Current open conversations with trend |
| **Messages Today** | 10 seconds | Total messages with hourly rate |
| **Avg Response** | 10 seconds | Average response time in milliseconds |

### Resource Utilization

Resource bars show system health:

| Resource | Gradient | Warning Threshold |
|----------|----------|-------------------|
| **CPU** | Blue/Purple | > 80% |
| **Memory** | Green | > 85% |
| **GPU** | Purple | > 90% |
| **Disk** | Amber | > 90% |

### Activity Ticker

A live ticker at the bottom shows the latest system events with a pulsing green indicator.

## View Modes

Toggle between two views using the grid button or press `V`:

### Live View (Default)
The animated SVG visualization showing the complete system topology with flowing data packets.

### Grid View
Traditional panel-based layout with detailed tree views for each metric category:
- Sessions panel with active, peak, and duration
- Messages panel with counts and rates
- Resources panel with progress bars
- Services panel with health status
- Bots panel with active bot list

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `V` | Toggle between Live and Grid view |
| `R` | Refresh all metrics |

## HTMX Integration

The dashboard uses HTMX for real-time updates without full page reloads:

| Endpoint | Interval | Data |
|----------|----------|------|
| `/api/monitoring/metric/sessions` | 5s | Session count |
| `/api/monitoring/metric/messages` | 10s | Message count |
| `/api/monitoring/metric/response_time` | 10s | Avg response |
| `/api/monitoring/resources/bars` | 15s | Resource SVG bars |
| `/api/monitoring/services/status` | 30s | Service health JSON |
| `/api/monitoring/activity/latest` | 5s | Activity text |
| `/api/monitoring/timestamp` | 5s | Last updated time |

## API Access

Access monitoring data programmatically:

### Get Full Status

```/dev/null/monitoring-api.txt
GET /api/monitoring/status
```

**Response:**

```/dev/null/monitoring-response.json
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

### Service-Specific Endpoints

| Endpoint | Returns |
|----------|---------|
| `/api/monitoring/services` | All service details |
| `/api/monitoring/bots` | Active bot list |
| `/api/monitoring/history?period=24h` | Historical metrics |
| `/api/monitoring/prometheus` | Prometheus format export |

## Component Health Details

| Component | Health Check | Warning Signs |
|-----------|--------------|---------------|
| **PostgreSQL** | Connection count, query rate | > 80 connections, slow queries |
| **Qdrant** | Vector count, search latency | > 50ms search time |
| **MinIO** | Storage usage, object count | > 80% storage used |
| **BotModels** | Token usage, response latency | > 2s response time |
| **Vault** | Seal status, policy count | Unsealed without auth |
| **Cache** | Hit rate, memory usage | < 80% hit rate |

## Alerts Configuration

Configure alert thresholds in `config.csv`:

```/dev/null/config-alerts.csv
name,value
alert-cpu-threshold,80
alert-memory-threshold,85
alert-disk-threshold,90
alert-response-time-ms,5000
alert-email,admin@example.com
```

## Console Mode

For terminal-based monitoring:

```/dev/null/console-command.bash
./botserver --console --monitor
```

Output:
```/dev/null/console-output.txt
[MONITOR] 2025-01-15 14:32:00
Sessions: 12 active (peak: 47)
Messages: 1,234 today (89/hour)
CPU: 65% | MEM: 72% | GPU: 45%
Services: 6/6 running
```

## Tips & Best Practices

💡 **Watch the data packets** - Flowing animations indicate active communication between components

💡 **Monitor trends** - The session trend indicator (↑/↓) shows direction of change

💡 **Click services** - Click any service node in Live view to see detailed status

💡 **Set up alerts** - Configure thresholds before issues become critical

💡 **Use keyboard shortcuts** - Press `R` for quick refresh, `V` to toggle views

💡 **Check historical data** - Query `/api/monitoring/history` for trend analysis

## Troubleshooting

### Dashboard not loading

**Possible causes:**
1. WebSocket connection failed
2. API endpoint unreachable
3. Browser blocking HTMX

**Solution:**
1. Check browser console for errors
2. Verify `/api/monitoring/status` returns data
3. Refresh the page

### Metrics showing "--"

**Possible causes:**
1. Initial load in progress
2. API timeout
3. Service unavailable

**Solution:**
1. Wait 5-10 seconds for first update
2. Check network tab for failed requests
3. Verify services are running

### Animations stuttering

**Possible causes:**
1. High CPU usage
2. Many browser tabs open
3. Hardware acceleration disabled

**Solution:**
1. Close unused tabs
2. Enable hardware acceleration in browser
3. Use Grid view for lower resource usage

## See Also

- [Monitoring API Reference](../chapter-10-api/monitoring-api.md)
- [Console Mode](./console-mode.md)
- [Configuration Options](../chapter-08-config/README.md)
- [Analytics App](./apps/analytics.md)