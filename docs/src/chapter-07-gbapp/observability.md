# Observability

General Bots uses a comprehensive observability stack for monitoring, logging, and metrics collection. This chapter explains how logging works and how Vector integrates without requiring code changes.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         BotServer Application                               │
│                                                                             │
│   log::trace!() ──┐                                                         │
│   log::debug!() ──┼──▶ Log Files (./botserver-stack/logs/)                 │
│   log::info!()  ──┤                                                         │
│   log::warn!()  ──┤                                                         │
│   log::error!() ──┘                                                         │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                              Vector Agent                                   │
│                         (Collects from log files)                           │
│                                                                             │
│   ┌─────────────┐     ┌─────────────┐     ┌─────────────┐                  │
│   │   Sources   │ ──▶ │ Transforms  │ ──▶ │    Sinks    │                  │
│   │  (Files)    │     │  (Parse)    │     │ (Outputs)   │                  │
│   └─────────────┘     └─────────────┘     └─────────────┘                  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
                    ┌─────────────────┼─────────────────┐
                    │                 │                 │
                    ▼                 ▼                 ▼
             ┌───────────┐     ┌───────────┐     ┌───────────┐
             │ InfluxDB  │     │  Grafana  │     │  Alerts   │
             │ (Metrics) │     │(Dashboard)│     │(Webhook)  │
             └───────────┘     └───────────┘     └───────────┘
```

## No Code Changes Required

**You do NOT need to replace `log::trace!()`, `log::info!()`, `log::error!()` calls.**

Vector works by:

1. **Tailing log files** - Reads from `./botserver-stack/logs/`
2. **Parsing log lines** - Extracts level, timestamp, message
3. **Routing by level** - Sends errors to alerts, metrics to InfluxDB
4. **Enriching data** - Adds hostname, service name, etc.

Log directory structure:
- `logs/system/` - BotServer application logs
- `logs/drive/` - MinIO logs
- `logs/tables/` - PostgreSQL logs
- `logs/cache/` - Redis logs
- `logs/llm/` - LLM server logs
- `logs/email/` - Stalwart logs
- `logs/directory/` - Zitadel logs
- `logs/vectordb/` - Qdrant logs
- `logs/meet/` - LiveKit logs
- `logs/alm/` - Forgejo logs

This approach:
- Requires zero code changes
- Works with existing logging
- Can be added/removed without recompilation
- Scales independently from the application

## Vector Configuration

### Installation

Vector is installed as the **observability** component:

```bash
./botserver install observability
```

### Configuration File

Configuration is at `./botserver-stack/conf/monitoring/vector.toml`:

```toml
# Vector Configuration for General Bots
# Collects logs without requiring code changes
# Component: observability (Vector)
# Config: ./botserver-stack/conf/monitoring/vector.toml

#
# SOURCES - Where logs come from
#

[sources.botserver_logs]
type = "file"
include = ["./botserver-stack/logs/system/*.log"]
read_from = "beginning"

[sources.drive_logs]
type = "file"
include = ["./botserver-stack/logs/drive/*.log"]
read_from = "beginning"

[sources.tables_logs]
type = "file"
include = ["./botserver-stack/logs/tables/*.log"]
read_from = "beginning"

[sources.cache_logs]
type = "file"
include = ["./botserver-stack/logs/cache/*.log"]
read_from = "beginning"

[sources.llm_logs]
type = "file"
include = ["./botserver-stack/logs/llm/*.log"]
read_from = "beginning"

[sources.service_logs]
type = "file"
include = [
  "./botserver-stack/logs/email/*.log",
  "./botserver-stack/logs/directory/*.log",
  "./botserver-stack/logs/vectordb/*.log",
  "./botserver-stack/logs/meet/*.log",
  "./botserver-stack/logs/alm/*.log"
]
read_from = "beginning"

#
# TRANSFORMS - Parse and enrich logs
#

[transforms.parse_botserver]
type = "remap"
inputs = ["botserver_logs"]
source = '''
# Parse standard log format: [TIMESTAMP LEVEL target] message
. = parse_regex!(.message, r'^(?P<timestamp>\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d+Z?)\s+(?P<level>\w+)\s+(?P<target>\S+)\s+(?P<message>.*)$')

# Convert timestamp
.timestamp = parse_timestamp!(.timestamp, "%Y-%m-%dT%H:%M:%S%.fZ")

# Normalize level
.level = downcase!(.level)

# Add service name
.service = "botserver"

# Extract session_id if present
if contains(string!(.message), "session") {
  session_match = parse_regex(.message, r'session[:\s]+(?P<session_id>[a-f0-9-]+)') ?? {}
  if exists(session_match.session_id) {
    .session_id = session_match.session_id
  }
}

# Extract user_id if present
if contains(string!(.message), "user") {
  user_match = parse_regex(.message, r'user[:\s]+(?P<user_id>[a-f0-9-]+)') ?? {}
  if exists(user_match.user_id) {
    .user_id = user_match.user_id
  }
}
'''

[transforms.parse_service_logs]
type = "remap"
inputs = ["service_logs"]
source = '''
# Basic parsing for service logs
.timestamp = now()
.level = "info"

# Detect errors
if contains(string!(.message), "ERROR") || contains(string!(.message), "error") {
  .level = "error"
}
if contains(string!(.message), "WARN") || contains(string!(.message), "warn") {
  .level = "warn"
}

# Extract service name from file path
.service = replace(string!(.file), r'.*/(\w+)\.log$', "$1")
'''

#
# FILTERS - Route by log level
#

[transforms.filter_errors]
type = "filter"
inputs = ["parse_botserver", "parse_service_logs"]
condition = '.level == "error"'

[transforms.filter_warnings]
type = "filter"
inputs = ["parse_botserver", "parse_service_logs"]
condition = '.level == "warn"'

[transforms.filter_info]
type = "filter"
inputs = ["parse_botserver"]
condition = '.level == "info" || .level == "debug"'

#
# METRICS - Convert logs to metrics
#

[transforms.log_to_metrics]
type = "log_to_metric"
inputs = ["parse_botserver"]

[[transforms.log_to_metrics.metrics]]
type = "counter"
field = "level"
name = "log_events_total"
tags.level = "{{level}}"
tags.service = "{{service}}"

[[transforms.log_to_metrics.metrics]]
type = "counter"
field = "message"
name = "errors_total"
tags.service = "{{service}}"
increment_by_value = false

#
# SINKS - Where logs go
#

# All logs to file (backup)
[sinks.file_backup]
type = "file"
inputs = ["parse_botserver", "parse_service_logs"]
path = "./botserver-stack/logs/vector/all-%Y-%m-%d.log"
encoding.codec = "json"

# Metrics to InfluxDB
[sinks.influxdb]
type = "influxdb_metrics"
inputs = ["log_to_metrics"]
endpoint = "http://localhost:8086"
org = "pragmatismo"
bucket = "metrics"
token = "${INFLUXDB_TOKEN}"

# Errors to alerting (webhook)
[sinks.alert_webhook]
type = "http"
inputs = ["filter_errors"]
uri = "http://localhost:8080/api/admin/alerts"
method = "post"
encoding.codec = "json"

# Console output (for debugging)
[sinks.console]
type = "console"
inputs = ["filter_errors"]
encoding.codec = "text"
```

## Log Format

BotServer uses the standard Rust `log` crate format:

```
2024-01-15T10:30:45.123Z INFO botserver::core::bot Processing message for session: abc-123
2024-01-15T10:30:45.456Z DEBUG botserver::llm::cache Cache hit for prompt hash: xyz789
2024-01-15T10:30:45.789Z ERROR botserver::drive::upload Failed to upload file: permission denied
```

Vector parses this automatically without code changes.

## Metrics Collection

### Automatic Metrics

Vector converts log events to metrics:

| Metric | Description |
|--------|-------------|
| `log_events_total` | Total log events by level |
| `errors_total` | Error count by service |
| `warnings_total` | Warning count by service |

### Application Metrics

BotServer also exposes metrics via `/api/metrics` (Prometheus format):

```
# HELP botserver_sessions_active Current active sessions
# TYPE botserver_sessions_active gauge
botserver_sessions_active 42

# HELP botserver_messages_total Total messages processed
# TYPE botserver_messages_total counter
botserver_messages_total{channel="web"} 1234
botserver_messages_total{channel="whatsapp"} 567

# HELP botserver_llm_latency_seconds LLM response latency
# TYPE botserver_llm_latency_seconds histogram
botserver_llm_latency_seconds_bucket{le="0.5"} 100
botserver_llm_latency_seconds_bucket{le="1.0"} 150
botserver_llm_latency_seconds_bucket{le="2.0"} 180
```

Vector can scrape these directly:

```toml
[sources.prometheus_metrics]
type = "prometheus_scrape"
endpoints = ["http://localhost:8080/api/metrics"]
scrape_interval_secs = 15
```

## Alerting

### Error Alerts

Vector sends errors to a webhook for alerting:

```toml
[sinks.alert_webhook]
type = "http"
inputs = ["filter_errors"]
uri = "http://localhost:8080/api/admin/alerts"
method = "post"
encoding.codec = "json"
```

### Slack Integration

```toml
[sinks.slack_alerts]
type = "http"
inputs = ["filter_errors"]
uri = "https://hooks.slack.com/services/YOUR/WEBHOOK/URL"
method = "post"
encoding.codec = "json"

[sinks.slack_alerts.request]
headers.content-type = "application/json"

[sinks.slack_alerts.encoding]
codec = "json"
```

### Email Alerts

Use with an SMTP relay or webhook-to-email service:

```toml
[sinks.email_alerts]
type = "http"
inputs = ["filter_errors"]
uri = "http://localhost:8025/api/send"
method = "post"
encoding.codec = "json"
```

## Grafana Dashboards

### Pre-built Dashboard

Import the General Bots dashboard from `templates/grafana-dashboard.json`:

1. Open Grafana at `http://localhost:3000`
2. Go to Dashboards → Import
3. Upload `grafana-dashboard.json`
4. Select InfluxDB data source

### Key Panels

| Panel | Query |
|-------|-------|
| Active Sessions | `from(bucket:"metrics") \|> filter(fn: (r) => r._measurement == "sessions_active")` |
| Messages/Minute | `from(bucket:"metrics") \|> filter(fn: (r) => r._measurement == "messages_total") \|> derivative()` |
| Error Rate | `from(bucket:"metrics") \|> filter(fn: (r) => r.level == "error") \|> count()` |
| LLM Latency P95 | `from(bucket:"metrics") \|> filter(fn: (r) => r._measurement == "llm_latency") \|> quantile(q: 0.95)` |

## Configuration Options

### config.csv Settings

```csv
# Observability settings
observability-enabled,true
observability-log-level,info
observability-metrics-endpoint,/api/metrics
observability-vector-enabled,true
```

### Log Levels

| Level | When to Use |
|-------|-------------|
| `error` | Something failed, requires attention |
| `warn` | Unexpected but handled, worth noting |
| `info` | Normal operations, key events |
| `debug` | Detailed flow, development |
| `trace` | Very detailed, performance impact |

Set in config.csv:

```csv
log-level,info
```

Or environment:

```bash
RUST_LOG=info ./botserver
```

## Troubleshooting

### Vector Not Collecting Logs

```bash
# Check Vector status
systemctl status gbo-observability

# View Vector logs
journalctl -u gbo-observability -f

# Test configuration
vector validate ./botserver-stack/conf/monitoring/vector.toml
```

### Missing Metrics in InfluxDB

```bash
# Check InfluxDB connection
curl http://localhost:8086/health

# Verify bucket exists
influx bucket list

# Check Vector sink status
vector top
```

### High Log Volume

If logs are too verbose:

1. Increase log level in config.csv
2. Add filters in Vector to drop debug logs
3. Set retention policies in InfluxDB

```toml
# Drop debug logs before sending to InfluxDB
[transforms.drop_debug]
type = "filter"
inputs = ["parse_botserver"]
condition = '.level != "debug" && .level != "trace"'
```

## Best Practices

### 1. Don't Log Sensitive Data

```rust
// Bad
log::info!("User password: {}", password);

// Good
log::info!("User {} authenticated successfully", user_id);
```

### 2. Use Structured Context

```rust
// Better for parsing
log::info!("session={} user={} action=message_sent", session_id, user_id);
```

### 3. Set Appropriate Levels

```rust
// Errors: things that failed
log::error!("Database connection failed: {}", err);

// Warnings: unusual but handled
log::warn!("Retrying LLM request after timeout");

// Info: normal operations
log::info!("Session {} started", session_id);

// Debug: development details
log::debug!("Cache lookup for key: {}", key);

// Trace: very detailed
log::trace!("Entering function process_message");
```

### 4. Keep Vector Config Simple

Start with basic collection, add transforms as needed.

## Summary

- **No code changes needed** - Vector collects from log files
- **Keep using log macros** - `log::info!()`, `log::error!()`, etc.
- **Vector handles routing** - Errors to alerts, all to storage
- **InfluxDB for metrics** - Time-series storage and queries
- **Grafana for dashboards** - Visualize everything

## Next Steps

- [Scaling and Load Balancing](./scaling.md) - Scale observability with your cluster
- [Infrastructure Design](./infrastructure.md) - Full architecture overview
- [Monitoring Dashboard](../chapter-04-gbui/monitoring.md) - Built-in monitoring UI