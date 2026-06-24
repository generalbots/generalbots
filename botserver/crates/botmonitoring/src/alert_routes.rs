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

fn get_alerts() -> Vec<Alert> {
    vec![]
}

fn get_alert_rules() -> Vec<AlertRule> {
    vec![]
}

pub async fn list_alerts<S: MonitoringState>(State(_state): State<Arc<S>>) -> Json<Vec<Alert>> {
    Json(get_alerts())
}

pub async fn get_alert<S: MonitoringState>(Path(id): Path<Uuid>, State(_state): State<Arc<S>>) -> Result<Json<Alert>, StatusCode> {
    let alerts = get_alerts();
    alerts.into_iter().find(|a| a.id == id).map(Json).ok_or(StatusCode::NOT_FOUND)
}

pub async fn acknowledge_alert<S: MonitoringState>(Path(_id): Path<Uuid>, State(_state): State<Arc<S>>) -> StatusCode {
    StatusCode::OK
}

pub async fn silence_alert<S: MonitoringState>(Path(_id): Path<Uuid>, Query(_query): Query<SilenceQuery>, State(_state): State<Arc<S>>) -> StatusCode {
    StatusCode::OK
}

pub async fn acknowledge_all_alerts<S: MonitoringState>(State(_state): State<Arc<S>>) -> StatusCode {
    StatusCode::OK
}

pub async fn list_rules<S: MonitoringState>(State(_state): State<Arc<S>>) -> Json<Vec<AlertRule>> {
    Json(get_alert_rules())
}

pub async fn get_rule<S: MonitoringState>(Path(id): Path<Uuid>, State(_state): State<Arc<S>>) -> Result<Json<AlertRule>, StatusCode> {
    let rules = get_alert_rules();
    rules.into_iter().find(|r| r.id == id).map(Json).ok_or(StatusCode::NOT_FOUND)
}

pub async fn update_rule<S: MonitoringState>(Path(_id): Path<Uuid>, State(_state): State<Arc<S>>, Json(_rule): Json<serde_json::Value>) -> StatusCode {
    StatusCode::OK
}

pub async fn delete_rule<S: MonitoringState>(Path(_id): Path<Uuid>, State(_state): State<Arc<S>>) -> StatusCode {
    StatusCode::OK
}

pub async fn export_history<S: MonitoringState>(Query(_query): Query<ExportQuery>, State(_state): State<Arc<S>>) -> Json<Vec<Alert>> {
    Json(get_alerts())
}

pub async fn export_monitoring_data<S: MonitoringState>(Query(query): Query<ExportQuery>, State(_state): State<Arc<S>>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "exported_at": chrono::Utc::now(),
        "range": query.range,
        "metrics": [],
        "alerts": [],
    }))
}
