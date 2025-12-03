# Time-Series Database Module

InfluxDB 3 integration for metrics, analytics, and operational data.

## Overview

High-performance time-series storage supporting 2.5M+ points/sec ingestion with async batching.

## Configuration

Add to `config.csv`:

```csv
influxdb-url,http://localhost:8086
influxdb-token,your-token
influxdb-org,pragmatismo
influxdb-bucket,metrics
```

Or environment variables:

```bash
INFLUXDB_URL=http://localhost:8086
INFLUXDB_TOKEN=your-token
INFLUXDB_ORG=pragmatismo
INFLUXDB_BUCKET=metrics
```

## Metric Points

Structure:

| Field | Description |
|-------|-------------|
| `measurement` | Metric name (e.g., "messages", "response_time") |
| `tags` | Indexed key-value pairs for filtering |
| `fields` | Actual metric values |
| `timestamp` | When the metric was recorded |

## Built-in Metrics

| Measurement | Tags | Fields |
|-------------|------|--------|
| `messages` | bot, channel, user | count |
| `response_time` | bot, endpoint | duration_ms |
| `llm_tokens` | bot, model, type | input, output, total |
| `kb_queries` | bot, collection | count, latency_ms |
| `errors` | bot, type, severity | count |

## Usage in Rust

```rust
let client = TimeSeriesClient::new(config).await?;

client.write_point(
    MetricPoint::new("messages")
        .tag("bot", "sales-bot")
        .tag("channel", "whatsapp")
        .field_i64("count", 1)
).await?;
```

## Querying

REST endpoint for analytics:

```
GET /api/analytics/timeseries/messages?range=24h
GET /api/analytics/timeseries/response_time?range=7d
```

## Installation

The timeseries_db component is installed via package manager:

```bash
gb install timeseries_db
```

Ports: 8086 (HTTP API), 8083 (RPC)

## See Also

- [Analytics Module](../chapter-04-gbui/apps/analytics.md)
- [Observability Setup](./observability.md)