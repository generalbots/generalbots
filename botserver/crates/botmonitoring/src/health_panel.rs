use crate::{health_status_class, format_uptime_duration, MonitoringState, MonitoringUrls};
use axum::{extract::State, response::Html};
use chrono::{Duration as ChronoDuration, Utc};
use std::sync::Arc;

pub async fn health_status<S: MonitoringState, U: MonitoringUrls>(State(state): State<Arc<S>>) -> Html<String> {
    let collector = state.metrics_collector();
    let health = collector.get_health().await;
    let status = health_status_class(&health.status);
    Html(format!(
        r##"<span class="health-indicator {status}" title="{status}"></span>"##
    ))
}

pub async fn overview<S: MonitoringState, U: MonitoringUrls>(State(state): State<Arc<S>>) -> Html<String> {
    let collector = state.metrics_collector();
    let health = collector.get_health().await;
    let db_ok = state.is_db_healthy();
    let status_class = if health.status == crate::HealthStatus::Healthy && db_ok {
        "healthy"
    } else if health.status == crate::HealthStatus::Unhealthy || !db_ok {
        "unhealthy"
    } else {
        "degraded"
    };

    let (title, subtitle) = match status_class {
        "unhealthy" => ("System Unavailable", "One or more health checks are failing"),
        "degraded" => ("System Degraded", "Some components are under stress or unavailable"),
        _ => ("All Systems Operational", "All health checks are passing"),
    };

    Html(format!(
        r##"<div class="health-status {status_class}">
    <div class="status-icon">
        <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path>
            <polyline points="22 4 12 14.01 9 11.01"></polyline>
        </svg>
    </div>
    <div class="status-info">
        <h2>{title}</h2>
        <p>{subtitle}</p>
    </div>
    <div class="status-badge {status_class}">{label}</div>
</div>"##,
        label = status_class,
    ))
}

pub async fn uptime<S: MonitoringState, U: MonitoringUrls>(State(state): State<Arc<S>>) -> Html<String> {
    let collector = state.metrics_collector();
    let health = collector.get_health().await;
    Html(format!(
        r##"<span class="stat-value">{uptime}</span>
<span class="stat-label">Uptime</span>"##,
        uptime = format_uptime_duration(health.uptime_seconds),
    ))
}

pub async fn uptime_percent<S: MonitoringState, U: MonitoringUrls>(State(state): State<Arc<S>>) -> Html<String> {
    let collector = state.metrics_collector();
    let health = collector.get_health().await;
    let window = 30 * 86400u64;
    let percent = if window > 0 {
        ((health.uptime_seconds as f64 / window as f64) * 100.0).min(100.0)
    } else {
        0.0
    };
    Html(format!(
        r##"<span class="stat-value">{percent:.1}%</span>
<span class="stat-label">Uptime (30 days)</span>"##
    ))
}

pub async fn last_incident<S: MonitoringState, U: MonitoringUrls>(State(state): State<Arc<S>>) -> Html<String> {
    let collector = state.metrics_collector();
    let alerts = collector.get_recent_alerts(20).await;
    if let Some(alert) = alerts.iter().find(|a| a.resolved_at.is_some()) {
        let resolved = alert.resolved_at.unwrap_or(alert.started_at);
        Html(format!(
            r##"<span class="stat-value">{age}</span>
<span class="stat-label">{name}</span>"##,
            age = relative_time(resolved),
            name = alert.rule_name,
        ))
    } else {
        Html(
            r##"<span class="stat-value">None</span>
<span class="stat-label">Last Incident</span>"##.to_string(),
        )
    }
}

pub async fn response_time<S: MonitoringState, U: MonitoringUrls>(State(state): State<Arc<S>>) -> Html<String> {
    let collector = state.metrics_collector();
    let health = collector.get_health().await;
    Html(format!(
        r##"<span class="stat-value">{ms:.0} ms</span>
<span class="stat-label">Avg Response Time</span>"##,
        ms = health.average_latency_ms,
    ))
}

pub async fn checks<S: MonitoringState, U: MonitoringUrls>(State(state): State<Arc<S>>) -> Html<String> {
    let collector = state.metrics_collector();
    let health = collector.get_health().await;
    let db_ok = state.is_db_healthy();

    let mut cards = String::new();
    for component in health.components.iter() {
        let status = health_status_class(&component.status);
        let latency = component.latency_ms.unwrap_or(0.0);
        cards.push_str(&format!(
            r##"<div class="health-check-card">
    <div class="check-header">
        <span class="check-status {status}"></span>
        <span class="check-name">{name}</span>
        <span class="check-badge {status}">{label}</span>
    </div>
    <div class="check-details">
        <div class="check-row">
            <span class="check-label">Response Time</span>
            <span class="check-value">{latency:.0} ms</span>
        </div>
        <div class="check-row">
            <span class="check-label">Last Check</span>
            <span class="check-value">{last_check}</span>
        </div>
    </div>
</div>"##,
            name = display_name(&component.name),
            label = status,
            last_check = component.last_check.format("%H:%M:%S UTC"),
        ));
    }

    cards.push_str(&format!(
        r##"<div class="health-check-card">
    <div class="check-header">
        <span class="check-status {db_status}"></span>
        <span class="check-name">Database</span>
        <span class="check-badge {db_status}">{db_label}</span>
    </div>
    <div class="check-details">
        <div class="check-row">
            <span class="check-label">Response Time</span>
            <span class="check-value">{db_latency:.0} ms</span>
        </div>
        <div class="check-row">
            <span class="check-label">Last Check</span>
            <span class="check-value">{db_time}</span>
        </div>
    </div>
</div>"##,
        db_status = if db_ok { "healthy" } else { "unhealthy" },
        db_label = if db_ok { "Healthy" } else { "Unhealthy" },
        db_latency = health.average_latency_ms,
        db_time = Utc::now().format("%H:%M:%S UTC"),
    ));

    Html(format!(r##"<div class="health-checks-grid">{cards}</div>"##))
}

pub async fn dependencies<S: MonitoringState, U: MonitoringUrls>(State(state): State<Arc<S>>) -> Html<String> {
    let deps = state.dependencies();
    if deps.is_empty() {
        return Html(
            r##"<div class="dependencies-list"><div class="dependency-row"><span>No external dependencies configured</span></div></div>"##
                .to_string(),
        );
    }

    let mut rows = String::new();
    for dep in deps {
        let status = if dep.healthy { "healthy" } else { "unhealthy" };
        let badge = if dep.healthy { "Online" } else { "Offline" };
        rows.push_str(&format!(
            r##"<div class="dependency-row">
    <div class="dependency-info">
        <span class="dependency-status {status}"></span>
        <span class="dependency-name">{name}</span>
        <span class="dependency-url">{host}</span>
    </div>
    <div class="dependency-stats">
        <span class="stat">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="12" cy="12" r="10"></circle>
                <polyline points="12 6 12 12 16 14"></polyline>
            </svg>
            {latency:.0} ms
        </span>
        <span class="dependency-badge {status}">{badge}</span>
    </div>
</div>"##,
            name = dep.name,
            host = dep.host,
            latency = dep.latency_ms,
        ));
    }

    Html(format!(r##"<div class="dependencies-list">{rows}</div>"##))
}

pub async fn uptime_history<S: MonitoringState, U: MonitoringUrls>(State(state): State<Arc<S>>) -> Html<String> {
    let collector = state.metrics_collector();
    let health = collector.get_health().await;
    let now = Utc::now();

    let mut bars = String::new();
    let uptime_seconds = health.uptime_seconds.max(1);
    let today_percent = ((uptime_seconds.min(86400) as f64 / 86400.0) * 100.0).clamp(0.0, 100.0);

    for i in (0..90).rev() {
        let day = now - ChronoDuration::days(i);
        let age = day.date_naive();
        let uptime_days = uptime_seconds / 86400;

        let (class, label, percent) = if uptime_days >= i as u64 {
            ("healthy", format!("{age} - 100%"), 100.0)
        } else if uptime_days + 1 == i as u64 {
            ("healthy", format!("{age} - {today_percent:.1}%"), today_percent)
        } else {
            ("outage", format!("{age} - no data"), 0.0)
        };

        bars.push_str(&format!(
            r##"<div class="uptime-bar {class}" title="{label}" style="height: {percent:.0}%"></div>"##
        ));
    }

    Html(format!(
        r##"<div class="uptime-bars">{bars}</div>
<div class="uptime-labels">
    <span>90 days ago</span>
    <span>Today</span>
</div>"##
    ))
}

pub async fn incidents<S: MonitoringState, U: MonitoringUrls>(State(state): State<Arc<S>>) -> Html<String> {
    let collector = state.metrics_collector();
    let alerts = collector.get_recent_alerts(10).await;

    if alerts.is_empty() {
        return Html(
            r##"<div class="incident-placeholder">
    <svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path>
        <polyline points="22 4 12 14.01 9 11.01"></polyline>
    </svg>
    <p>No recent incidents</p>
    <span>System has been stable</span>
</div>"##.to_string(),
        );
    }

    let mut items = String::new();
    for alert in alerts {
        let resolved = alert.resolved_at.is_some();
        let class = if resolved { "resolved" } else { "ongoing" };
        items.push_str(&format!(
            r##"<div class="incident-item {class}">
    <div class="incident-icon">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"></path>
            <line x1="12" y1="9" x2="12" y2="13"></line>
            <line x1="12" y1="17" x2="12.01" y2="17"></line>
        </svg>
    </div>
    <div class="incident-content">
        <div class="incident-title">{name}</div>
        <div class="incident-description">{message}</div>
        <div class="incident-meta">{started} · {outcome}</div>
    </div>
</div>"##,
            name = alert.rule_name,
            message = alert.message,
            started = alert.started_at.format("%b %d, %H:%M UTC"),
            outcome = if resolved { "Resolved" } else { "Ongoing" },
        ));
    }

    Html(items)
}

fn display_name(key: &str) -> &str {
    match key {
        "database" => "Database",
        "cache" => "Cache",
        "vector_db" => "Vector DB",
        "llm" => "LLM",
        "minio" => "Storage",
        _ => key,
    }
}

fn relative_time(time: chrono::DateTime<Utc>) -> String {
    let now = Utc::now();
    let diff = now - time;
    if diff.num_days() > 0 {
        format!("{}d ago", diff.num_days())
    } else if diff.num_hours() > 0 {
        format!("{}h ago", diff.num_hours())
    } else {
        format!("{}m ago", diff.num_minutes().max(1))
    }
}
