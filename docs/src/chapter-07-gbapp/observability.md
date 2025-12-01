# Observability

General Bots provides built-in observability for monitoring, logging, and metrics collection.

## Overview

The observability system works automatically without code changes:

- **Log Collection** - Reads from `./botserver-stack/logs/`
- **Log Parsing** - Extracts level, timestamp, message
- **Routing** - Sends errors to alerts, metrics to storage
- **Enrichment** - Adds hostname, service name, etc.

## Log Directory Structure

| Directory | Contents |
|-----------|----------|
| `logs/system/` | BotServer application logs |
| `logs/drive/` | Storage service logs |
| `logs/tables/` | PostgreSQL logs |
| `logs/cache/` | Cache component logs |
| `logs/llm/` | LLM server logs |
| `logs/email/` | Email service logs |
| `logs/directory/` | Identity service logs |
| `logs/vectordb/` | Vector database logs |
| `logs/meet/` | Video meeting logs |

## Installation

The observability component is installed automatically during bootstrap, or manually:

```bash
./botserver install observability
```

Configuration is at `./botserver-stack/conf/monitoring/vector.toml`.

## Log Format

BotServer uses the standard Rust log format:

```
2024-01-15T10:30:45.123Z INFO botserver::core::bot Processing message for session: abc-123
2024-01-15T10:30:45.789Z ERROR botserver::drive::upload Failed to upload file: permission denied
```

Logs are parsed and routed automatically.

## Metrics

### Available Metrics

| Metric | Description |
|--------|-------------|
| `log_events_total` | Total log events by level |
| `errors_total` | Error count by service |
| `botserver_sessions_active` | Current active sessions |
| `botserver_messages_total` | Total messages processed |
| `botserver_llm_latency_seconds` | LLM response latency |

### Metrics Endpoint

BotServer exposes Prometheus-compatible metrics at `/api/metrics`.

## Alerting

Alerts are sent automatically when errors occur:

- **Webhook** - POST to `/api/admin/alerts`
- **Slack** - Configure webhook URL in settings
- **Email** - Configure SMTP in `config.csv`

Configure alert thresholds in `config.csv`:

```csv
name,value
alert-cpu-threshold,80
alert-memory-threshold,85
alert-response-time-ms,5000
```

## Dashboards

A pre-built Grafana dashboard is available at `templates/grafana-dashboard.json`.

Key panels include:
- Active Sessions
- Messages per Minute
- Error Rate
- LLM Latency (P95)

## Log Levels

| Level | When to Use |
|-------|-------------|
| `error` | Something failed, requires attention |
| `warn` | Unexpected but handled, worth noting |
| `info` | Normal operations, key events |
| `debug` | Detailed flow, development |

Set in `config.csv`:

```csv
name,value
log-level,info
```

## Troubleshooting

### Logs Not Collecting

1. Check observability service is running
2. Verify log directory permissions
3. Review service logs for errors

### High Log Volume

1. Increase log level in `config.csv`
2. Set retention policies in metrics storage
3. Filter debug logs in production

## Best Practices

1. **Don't log sensitive data** - Never log passwords or tokens
2. **Use appropriate levels** - Errors for failures, info for operations
3. **Monitor trends** - Watch for gradual increases, not just spikes
4. **Set up alerts early** - Don't wait for problems

## See Also

- [Scaling and Load Balancing](./scaling.md) - Scale with your cluster
- [Infrastructure Design](./infrastructure.md) - Full architecture overview
- [Monitoring Dashboard](../chapter-04-gbui/monitoring.md) - Built-in monitoring UI