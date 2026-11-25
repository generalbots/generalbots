use super::*;

pub async fn get_metric(
    State(state): State<Arc<AppState>>,
    Query(query): Query<MetricQuery>,
) -> Json<serde_json::Value> {
    let collector = &state.metrics_collector;

    let result = match query.aggregation.as_deref() {
        Some("p50") => collector.get_percentile(&query.name, 50.0).await,
        Some("p95") => collector.get_percentile(&query.name, 95.0).await,
        Some("p99") => collector.get_percentile(&query.name, 99.0).await,
        Some("rate") => {
            let window = Duration::minutes(query.window_minutes.unwrap_or(1));
            Some(collector.get_rate(&query.name, window).await)
        }
        Some("sum" | _) | None => collector.get_aggregate(&query.name).await,
    };

    Json(match result {
        Some(value) => serde_json::json!({
            "metric": query.name,
            "value": value,
            "timestamp": Utc::now(),
        }),
        None => serde_json::json!({
            "error": "Metric not found",
            "metric": query.name,
        }),
    })
}

/// Prometheus `# TYPE` and `# HELP` metadata for a metric family. Office and
/// latency duration families follow the `_count`/`_sum`/`_max` convention.
fn describe_metric(name: &str) -> (&'static str, &'static str) {
    match name {
        "office_recalc_duration_ms_count" => {
            ("counter", "Count of spreadsheet recalculations observed.")
        }
        "office_recalc_duration_ms_sum" => (
            "counter",
            "Sum of spreadsheet recalculation durations in milliseconds.",
        ),
        "office_recalc_duration_ms_max" => (
            "gauge",
            "Maximum observed spreadsheet recalculation duration in milliseconds.",
        ),
        "office_workbook_memory_bytes" => (
            "gauge",
            "Resident memory per workbook in bytes (label is an opaque hash).",
        ),
        "office_export_duration_ms_count" => {
            ("counter", "Count of office export operations observed.")
        }
        "office_export_duration_ms_sum" => (
            "counter",
            "Sum of office export durations in milliseconds.",
        ),
        "office_export_duration_ms_max" => (
            "gauge",
            "Maximum observed office export duration in milliseconds.",
        ),
        "http_request_duration_ms_count" => {
            ("counter", "Count of HTTP requests observed per route.")
        }
        "http_request_duration_ms_sum" => (
            "counter",
            "Sum of HTTP request latencies in milliseconds.",
        ),
        "http_request_duration_ms_max" => (
            "gauge",
            "Maximum observed HTTP request latency in milliseconds.",
        ),
        "users.total" => ("gauge", "Total registered users."),
        "users.active" => ("gauge", "Users active in the last seven days."),
        "sessions.total" => ("gauge", "Total user sessions."),
        "storage.gb" => ("gauge", "Knowledge-base storage in gigabytes."),
        _ if name.ends_with("_count") => ("counter", "Accumulated event count."),
        _ if name.ends_with("_sum") => ("counter", "Accumulated sum of observed values."),
        _ => ("gauge", "General Bots metric."),
    }
}

pub async fn export_metrics(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let collector = &state.metrics_collector;
    let metrics = collector.get_metrics().await;

    use std::fmt::Write;
    let mut prometheus_format = String::new();
    let mut seen_metrics = std::collections::HashSet::new();

    for metric in &metrics {
        if seen_metrics.insert(metric.name.as_str()) {
            let (metric_type, help) = describe_metric(&metric.name);
            if let Err(e) = writeln!(prometheus_format, "# HELP {} {}", metric.name, help) {
                log::warn!("Failed to write Prometheus HELP line: {e}");
            }
            if let Err(e) = writeln!(prometheus_format, "# TYPE {} {}", metric.name, metric_type) {
                log::warn!("Failed to write Prometheus TYPE line: {e}");
            }
        }

        let labels = metric
            .labels
            .iter()
            .map(|(k, v)| format!("{}=\"{}\"", k, v))
            .collect::<Vec<_>>()
            .join(",");

        if labels.is_empty() {
            if let Err(e) = writeln!(
                prometheus_format,
                "{} {} {}",
                metric.name,
                metric.value,
                metric.timestamp.timestamp_millis()
            ) {
                log::warn!("Failed to write Prometheus metric line: {e}");
            }
        } else {
            if let Err(e) = writeln!(
                prometheus_format,
                "{}{{{}}} {} {}",
                metric.name,
                labels,
                metric.value,
                metric.timestamp.timestamp_millis()
            ) {
                log::warn!("Failed to write labelled Prometheus metric line: {e}");
            }
        }
    }

    let mut response = prometheus_format.into_response();
    response.headers_mut().insert(
        axum::http::header::CONTENT_TYPE,
        axum::http::HeaderValue::from_static("text/plain; version=0.0.4; charset=utf-8"),
    );
    response
}

pub fn configure() -> axum::routing::Router<Arc<AppState>> {
    use axum::routing::{get, Router};

    Router::new()
        .route(ApiUrls::ANALYTICS_DASHBOARD, get(get_dashboard))
        .route(ApiUrls::ANALYTICS_METRIC, get(get_metric))
        .route(ApiUrls::METRICS, get(export_metrics))
        .route("/api/activity/recent", get(get_recent_activity))
}

/// Canonical Prometheus scrape router. Self-contained: merge it into the sub
/// router to expose `GET /metrics` with the same handler as `/api/metrics`.
/// See the driver report for the exact registration site (unowned file).
pub fn configure_canonical() -> axum::routing::Router<Arc<AppState>> {
    use axum::routing::{get, Router};

    Router::new().route(ApiUrls::METRICS_CANONICAL, get(export_metrics))
}
