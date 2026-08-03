use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::real_time::{Alert, AlertRule};
use crate::MonitoringState;

#[derive(Deserialize)]
pub struct SilenceQuery {
    pub duration: Option<u64>,
}

#[derive(Deserialize)]
pub struct ExportQuery {
    pub range: Option<String>,
}

#[derive(Deserialize)]
pub struct AlertRulePayload {
    pub name: Option<String>,
    pub description: Option<String>,
    pub metric_name: Option<String>,
    pub enabled: Option<bool>,
    pub severity: Option<String>,
    pub threshold: Option<f64>,
}

pub async fn list_alerts<S: MonitoringState>(State(state): State<Arc<S>>) -> Json<Vec<Alert>> {
    let collector = state.metrics_collector();
    Json(collector.get_active_alerts().await)
}

pub async fn get_alert<S: MonitoringState>(Path(id): Path<Uuid>, State(state): State<Arc<S>>) -> Result<Json<Alert>, StatusCode> {
    let collector = state.metrics_collector();
    let recent = collector.get_recent_alerts(500).await;
    recent
        .into_iter()
        .find(|a| a.id == id)
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

pub async fn acknowledge_alert<S: MonitoringState>(Path(id): Path<Uuid>, State(state): State<Arc<S>>) -> StatusCode {
    let collector = state.metrics_collector();
    if collector.acknowledge_alert(id, "admin").await {
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}

pub async fn silence_alert<S: MonitoringState>(Path(id): Path<Uuid>, Query(_query): Query<SilenceQuery>, State(state): State<Arc<S>>) -> StatusCode {
    let collector = state.metrics_collector();
    if collector.acknowledge_alert(id, "silenced").await {
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}

pub async fn acknowledge_all_alerts<S: MonitoringState>(State(state): State<Arc<S>>) -> StatusCode {
    let collector = state.metrics_collector();
    let alerts = collector.get_active_alerts().await;
    for alert in alerts {
        collector.acknowledge_alert(alert.id, "admin").await;
    }
    StatusCode::OK
}

pub async fn list_rules<S: MonitoringState>(State(state): State<Arc<S>>) -> Json<Vec<AlertRule>> {
    let collector = state.metrics_collector();
    Json(collector.get_alert_rules().await)
}

pub async fn get_rule<S: MonitoringState>(Path(id): Path<Uuid>, State(state): State<Arc<S>>) -> Result<Json<AlertRule>, StatusCode> {
    let collector = state.metrics_collector();
    let rules = collector.get_alert_rules().await;
    rules.into_iter().find(|r| r.id == id).map(Json).ok_or(StatusCode::NOT_FOUND)
}

pub async fn update_rule<S: MonitoringState>(Path(id): Path<Uuid>, State(state): State<Arc<S>>, Json(payload): Json<AlertRulePayload>) -> Result<StatusCode, StatusCode> {
    let collector = state.metrics_collector();
    let mut rule = collector
        .get_alert_rules()
        .await
        .into_iter()
        .find(|r| r.id == id)
        .ok_or(StatusCode::NOT_FOUND)?;

    if let Some(name) = payload.name {
        rule.name = name;
    }
    if let Some(description) = payload.description {
        rule.description = description;
    }
    if let Some(metric_name) = payload.metric_name {
        rule.metric_name = metric_name;
    }
    if let Some(enabled) = payload.enabled {
        rule.enabled = enabled;
    }
    if let Some(threshold) = payload.threshold {
        rule.condition = crate::real_time::AlertCondition::GreaterThan(threshold);
    }
    if let Some(severity) = payload.severity {
        rule.severity = match severity.as_str() {
            "critical" => crate::real_time::AlertSeverity::Critical,
            "error" => crate::real_time::AlertSeverity::Error,
            "warning" => crate::real_time::AlertSeverity::Warning,
            _ => crate::real_time::AlertSeverity::Info,
        };
    }

    collector.replace_alert_rule(rule).await;
    Ok(StatusCode::OK)
}

pub async fn delete_rule<S: MonitoringState>(Path(id): Path<Uuid>, State(state): State<Arc<S>>) -> StatusCode {
    let collector = state.metrics_collector();
    if collector.remove_alert_rule(id).await {
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}

pub async fn export_history<S: MonitoringState>(Query(_query): Query<ExportQuery>, State(state): State<Arc<S>>) -> Json<Vec<Alert>> {
    let collector = state.metrics_collector();
    Json(collector.get_recent_alerts(500).await)
}

pub async fn export_monitoring_data<S: MonitoringState>(Query(query): Query<ExportQuery>, State(state): State<Arc<S>>) -> Json<serde_json::Value> {
    let collector = state.metrics_collector();
    let metrics = collector.get_metrics().await;
    let alerts = collector.get_recent_alerts(500).await;
    let health = collector.get_health().await;

    let metrics_json: Vec<serde_json::Value> = metrics
        .into_iter()
        .map(|m| {
            serde_json::json!({
                "name": m.name,
                "value": m.current_value,
                "updated_at": m.updated_at,
            })
        })
        .collect();

    Json(serde_json::json!({
        "exported_at": chrono::Utc::now(),
        "range": query.range,
        "health": {
            "status": format!("{:?}", health.status),
            "cpu_usage_percent": health.cpu_usage_percent,
            "memory_usage_percent": health.memory_usage_percent,
            "disk_usage_percent": health.disk_usage_percent,
            "uptime_seconds": health.uptime_seconds,
        },
        "metrics": metrics_json,
        "alerts": alerts,
    }))
}
