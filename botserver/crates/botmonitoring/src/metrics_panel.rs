use crate::MonitoringState;
use crate::MonitoringUrls;
use axum::extract::State;
use axum::response::Html;
use std::sync::Arc;

pub async fn system_status<S: MonitoringState, U: MonitoringUrls>(State(state): State<Arc<S>>) -> Html<String> {
    let collector = state.metrics_collector();
    let health = collector.get_health().await;
    let db_ok = state.is_db_healthy();

    let text = match (health.status, db_ok) {
        (crate::HealthStatus::Healthy, true) => "All Systems Operational",
        (crate::HealthStatus::Unhealthy, _) | (_, false) => "System Unavailable",
        _ => "System Degraded",
    };

    Html(text.to_string())
}

pub async fn last_sync<S: MonitoringState, U: MonitoringUrls>(State(state): State<Arc<S>>) -> Html<String> {
    let collector = state.metrics_collector();
    let health = collector.get_health().await;
    Html(format!("Last sync: {}", health.last_check.format("%H:%M:%S UTC")))
}

pub async fn metrics_table<S: MonitoringState, U: MonitoringUrls>(State(state): State<Arc<S>>) -> Html<String> {
    let collector = state.metrics_collector();
    let metrics = collector.get_metrics().await;

    if metrics.is_empty() {
        return Html(
            r##"<tbody>
    <tr>
        <td colspan="6" class="empty-cell">No metrics collected yet. System metrics are gathered every 60 seconds.</td>
    </tr>
</tbody>"##.to_string(),
        );
    }

    let mut rows = String::new();
    for metric in metrics.iter().take(100) {
        let metric_type = format!("{:?}", metric.metric_type);
        let category = metric
            .labels
            .get("category")
            .cloned()
            .unwrap_or_else(|| default_category(&metric.name).to_string());

        rows.push_str(&format!(
            r##"<tr>
    <td class="metric-name">{name}</td>
    <td>{metric_type}</td>
    <td>{value}</td>
    <td><span class="category-badge">{category}</span></td>
    <td class="metric-desc">{desc}</td>
    <td><span class="updated-at">{updated}</span></td>
</tr>"##,
            name = html_escape(&metric.name),
            value = format_metric_value(metric.current_value, &metric.unit),
            desc = if metric.description.is_empty() { "—" } else { &metric.description },
            updated = metric.updated_at.format("%H:%M:%S UTC"),
        ));
    }

    Html(format!(r##"<tbody>{rows}</tbody>"##))
}

pub async fn logs_services<S: MonitoringState, U: MonitoringUrls>(_state: State<Arc<S>>) -> Html<String> {
    let services = [
        "botserver",
        "postgresql",
        "valkey",
        "minio",
        "qdrant",
        "llm",
        "drive-monitor",
        "websocket",
        "monitoring",
    ];

    let mut options = String::new();
    for service in services {
        options.push_str(&format!(
            r##"<option value="{service}">{service}</option>"##
        ));
    }

    Html(options)
}

fn default_category(name: &str) -> &'static str {
    if name.starts_with("system_") {
        "system"
    } else if name.starts_with("governance_") {
        "security"
    } else if name.contains("db") || name.contains("postgres") {
        "database"
    } else if name.contains("cache") || name.contains("redis") {
        "cache"
    } else {
        "application"
    }
}

fn format_metric_value(value: f64, unit: &Option<String>) -> String {
    match unit.as_deref() {
        Some(u) => format!("{value:.2} {u}"),
        None if value.fract() == 0.0 => format!("{value:.0}"),
        None => format!("{value:.2}"),
    }
}

fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
