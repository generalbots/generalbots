use crate::real_time::{Alert, AlertSeverity, AlertStatus};
use crate::MonitoringState;
use axum::extract::State;
use axum::response::Html;
use std::sync::Arc;

pub async fn alert_count<S: MonitoringState>(State(state): State<Arc<S>>) -> Html<String> {
    let collector = state.metrics_collector();
    let count = collector.get_active_alerts().await.len();
    Html(count.to_string())
}

pub async fn alert_summary<S: MonitoringState>(State(state): State<Arc<S>>) -> Html<String> {
    let collector = state.metrics_collector();
    let alerts = collector.get_active_alerts().await;

    let critical = alerts.iter().filter(|a| a.severity == AlertSeverity::Critical).count();
    let warning = alerts.iter().filter(|a| a.severity == AlertSeverity::Warning).count();
    let info = alerts.iter().filter(|a| a.severity == AlertSeverity::Info).count();

    Html(format!(
        r##"<span class="summary-item critical">{critical} Critical</span>
<span class="summary-item warning">{warning} Warning</span>
<span class="summary-item info">{info} Info</span>"##
    ))
}

pub async fn active_alerts<S: MonitoringState>(State(state): State<Arc<S>>) -> Html<String> {
    let collector = state.metrics_collector();
    let alerts = collector.get_active_alerts().await;

    if alerts.is_empty() {
        return Html(
            r##"<div class="alert-placeholder">
    <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path>
        <polyline points="22 4 12 14.01 9 11.01"></polyline>
    </svg>
    <p>No active alerts</p>
    <span>All systems are operating normally</span>
</div>"##.to_string(),
        );
    }

    let items: String = alerts.iter().map(alert_item_html).collect();
    Html(items)
}

pub async fn alert_history<S: MonitoringState>(State(state): State<Arc<S>>) -> Html<String> {
    let collector = state.metrics_collector();
    let alerts = collector.get_recent_alerts(50).await;

    if alerts.is_empty() {
        return Html(
            r##"<div class="history-empty">No alert history recorded yet.</div>"##.to_string(),
        );
    }

    let items: String = alerts.iter().map(history_item_html).collect();
    Html(items)
}

fn severity_class(severity: &AlertSeverity) -> &'static str {
    match severity {
        AlertSeverity::Critical => "critical",
        AlertSeverity::Error => "critical",
        AlertSeverity::Warning => "warning",
        AlertSeverity::Info => "info",
    }
}

fn status_label(status: &AlertStatus) -> &'static str {
    match status {
        AlertStatus::Firing => "firing",
        AlertStatus::Resolved => "resolved",
        AlertStatus::Acknowledged => "acknowledged",
        AlertStatus::Silenced => "silenced",
    }
}

fn alert_item_html(alert: &Alert) -> String {
    let severity = severity_class(&alert.severity);
    let status = status_label(&alert.status);

    format!(
        r##"<div class="alert-item {severity}" data-status="{status}">
    <div class="alert-severity">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9"></path>
            <path d="M13.73 21a2 2 0 0 1-3.46 0"></path>
        </svg>
    </div>
    <div class="alert-content">
        <div class="alert-title">{title}</div>
        <div class="alert-message">{message}</div>
        <div class="alert-meta">
            <span>
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <circle cx="12" cy="12" r="10"></circle>
                    <polyline points="12 6 12 12 16 14"></polyline>
                </svg>
                {started}
            </span>
            <span>
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path>
                    <polyline points="14 2 14 8 20 8"></polyline>
                </svg>
                {metric}
            </span>
        </div>
    </div>
    <div class="alert-actions">
        <button class="alert-action-btn" onclick="acknowledgeAlert('{id}')" title="Acknowledge">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="9 11 12 14 22 4"></polyline>
                <path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11"></path>
            </svg>
        </button>
        <button class="alert-action-btn" onclick="openAlertDetailModal('{id}')" title="Details">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="12" cy="12" r="3"></circle>
                <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 1 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"></path>
            </svg>
        </button>
    </div>
</div>"##,
        title = html_escape(&alert.rule_name),
        message = html_escape(&alert.message),
        started = alert.started_at.format("%Y-%m-%d %H:%M UTC"),
        metric = html_escape(&alert.metric_name),
        id = alert.id,
    )
}

fn history_item_html(alert: &Alert) -> String {
    let severity = severity_class(&alert.severity);
    let outcome = if alert.resolved_at.is_some() {
        "resolved"
    } else if alert.acknowledged_at.is_some() {
        "acknowledged"
    } else {
        "expired"
    };

    format!(
        r##"<div class="history-item">
    <span class="history-time">{started}</span>
    <span class="history-severity {severity}">{severity}</span>
    <div class="history-content">
        <span class="history-name">{title}</span>
        <span class="history-meta">{metric} · {message}</span>
    </div>
    <span class="history-outcome {outcome}">{outcome}</span>
</div>"##,
        started = alert.started_at.format("%b %d, %H:%M"),
        title = html_escape(&alert.rule_name),
        metric = html_escape(&alert.metric_name),
        message = html_escape(&alert.message),
    )
}

fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
