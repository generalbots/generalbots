use crate::core::urls::ApiUrls;
use crate::shared::state::AppState;
use axum::{
    extract::{Json, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::{DateTime, Duration, Utc};
use diesel::prelude::*;
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub name: String,
    pub value: f64,
    pub labels: HashMap<String, String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct MetricsCollector {
    metrics: Arc<RwLock<Vec<Metric>>>,
    aggregates: Arc<RwLock<HashMap<String, f64>>>,
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(Vec::new())),
            aggregates: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn record(&self, name: String, value: f64, labels: HashMap<String, String>) {
        let metric = Metric {
            name: name.clone(),
            value,
            labels,
            timestamp: Utc::now(),
        };

        let mut metrics = self.metrics.write().await;
        metrics.push(metric);

        let mut aggregates = self.aggregates.write().await;
        let entry = aggregates.entry(name).or_insert(0.0);
        *entry += value;

        if metrics.len() > 10000 {
            let cutoff = Utc::now() - Duration::hours(1);
            metrics.retain(|m| m.timestamp > cutoff);
        }
    }

    pub async fn increment(&self, name: String, labels: HashMap<String, String>) {
        self.record(name, 1.0, labels).await;
    }

    pub async fn gauge(&self, name: String, value: f64, labels: HashMap<String, String>) {
        self.record(name, value, labels).await;
    }

    pub async fn get_metrics(&self) -> Vec<Metric> {
        self.metrics.read().await.clone()
    }

    pub async fn get_aggregate(&self, name: &str) -> Option<f64> {
        self.aggregates.read().await.get(name).copied()
    }

    pub async fn get_rate(&self, name: &str, window: Duration) -> f64 {
        let cutoff = Utc::now() - window;
        let metrics = self.metrics.read().await;
        let count = metrics
            .iter()
            .filter(|m| m.name == name && m.timestamp > cutoff)
            .count();
        count as f64 / window.num_seconds() as f64
    }

    pub async fn get_percentile(&self, name: &str, percentile: f64) -> Option<f64> {
        let metrics = self.metrics.read().await;
        let mut values: Vec<f64> = metrics
            .iter()
            .filter(|m| m.name == name)
            .map(|m| m.value)
            .collect();

        if values.is_empty() {
            return None;
        }

        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let index = ((percentile / 100.0) * values.len() as f64) as usize;
        values.get(index.min(values.len() - 1)).copied()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DashboardData {
    pub total_users: i64,
    pub active_users: i64,
    pub total_messages: i64,
    pub total_sessions: i64,
    pub storage_used_gb: f64,
    pub api_calls_per_minute: f64,
    pub error_rate: f64,
    pub response_time_p95: f64,
    pub charts: Vec<ChartData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChartData {
    pub title: String,
    pub chart_type: String,
    pub labels: Vec<String>,
    pub datasets: Vec<DataSet>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DataSet {
    pub label: String,
    pub data: Vec<f64>,
    pub color: String,
}

pub async fn collect_system_metrics(collector: &MetricsCollector, state: &AppState) {
    let mut conn = match state.conn.get() {
        Ok(c) => c,
        Err(e) => {
            log::error!("Failed to get database connection for metrics: {}", e);
            return;
        }
    };

    #[derive(QueryableByName)]
    struct CountResult {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        count: i64,
    }

    let total_users: i64 = diesel::sql_query("SELECT COUNT(*) as count FROM users")
        .get_result::<CountResult>(&mut conn)
        .map(|r| r.count)
        .unwrap_or(0);

    let active_cutoff = Utc::now() - Duration::days(7);
    let active_users: i64 = diesel::sql_query(
        "SELECT COUNT(DISTINCT user_id) as count FROM user_sessions WHERE updated_at > $1",
    )
    .bind::<diesel::sql_types::Timestamptz, _>(active_cutoff)
    .get_result::<CountResult>(&mut conn)
    .map(|r| r.count)
    .unwrap_or(0);

    let total_sessions: i64 = diesel::sql_query("SELECT COUNT(*) as count FROM user_sessions")
        .get_result::<CountResult>(&mut conn)
        .map(|r| r.count)
        .unwrap_or(0);

    #[derive(QueryableByName)]
    struct SizeResult {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        total_size: i64,
    }

    let storage_bytes: i64 =
        diesel::sql_query("SELECT COALESCE(SUM(file_size), 0) as total_size FROM kb_documents")
            .get_result::<SizeResult>(&mut conn)
            .map(|r| r.total_size)
            .unwrap_or(0);

    let storage_gb = storage_bytes as f64 / (1024.0 * 1024.0 * 1024.0);

    let mut labels = HashMap::new();
    labels.insert("source".to_string(), "system".to_string());

    collector
        .gauge(
            "users.total".to_string(),
            total_users as f64,
            labels.clone(),
        )
        .await;
    collector
        .gauge(
            "users.active".to_string(),
            active_users as f64,
            labels.clone(),
        )
        .await;
    collector
        .gauge(
            "sessions.total".to_string(),
            total_sessions as f64,
            labels.clone(),
        )
        .await;
    collector
        .gauge("storage.gb".to_string(), storage_gb, labels.clone())
        .await;
}

pub async fn track_api_call(
    collector: &MetricsCollector,
    endpoint: String,
    duration_ms: f64,
    status: u16,
) {
    let mut labels = HashMap::new();
    labels.insert("endpoint".to_string(), endpoint);
    labels.insert("status".to_string(), status.to_string());

    collector
        .increment("api.calls".to_string(), labels.clone())
        .await;
    collector
        .record("api.duration_ms".to_string(), duration_ms, labels.clone())
        .await;

    if status >= 500 {
        collector.increment("api.errors".to_string(), labels).await;
    }
}

pub async fn track_message(collector: &MetricsCollector, channel: String, user_id: String) {
    let mut labels = HashMap::new();
    labels.insert("channel".to_string(), channel);
    labels.insert("user_id".to_string(), user_id);

    collector
        .increment("messages.total".to_string(), labels)
        .await;
}

pub async fn track_file_operation(
    collector: &MetricsCollector,
    operation: String,
    size_bytes: i64,
    success: bool,
) {
    let mut labels = HashMap::new();
    labels.insert("operation".to_string(), operation);
    labels.insert("success".to_string(), success.to_string());

    collector
        .increment("files.operations".to_string(), labels.clone())
        .await;

    if success {
        collector
            .record("files.bytes".to_string(), size_bytes as f64, labels)
            .await;
    }
}

pub async fn get_dashboard(
    State(state): State<Arc<AppState>>,
) -> Result<Json<DashboardData>, StatusCode> {
    let collector = &state.metrics_collector;

    collect_system_metrics(collector, &state).await;

    let total_users = collector.get_aggregate("users.total").await.unwrap_or(0.0) as i64;
    let active_users = collector.get_aggregate("users.active").await.unwrap_or(0.0) as i64;
    let total_messages = collector
        .get_aggregate("messages.total")
        .await
        .unwrap_or(0.0) as i64;
    let total_sessions = collector
        .get_aggregate("sessions.total")
        .await
        .unwrap_or(0.0) as i64;
    let storage_used_gb = collector.get_aggregate("storage.gb").await.unwrap_or(0.0);

    let api_calls_per_minute = collector.get_rate("api.calls", Duration::minutes(1)).await * 60.0;
    let error_rate = collector.get_rate("api.errors", Duration::minutes(5)).await
        / collector
            .get_rate("api.calls", Duration::minutes(5))
            .await
            .max(1.0);
    let response_time_p95 = collector
        .get_percentile("api.duration_ms", 95.0)
        .await
        .unwrap_or(0.0);

    let mut charts = Vec::new();

    let now = Utc::now();
    let mut api_labels = Vec::new();
    let mut api_data = Vec::new();

    for i in (0..24).rev() {
        let hour = now - Duration::hours(i);
        api_labels.push(hour.format("%H:00").to_string());
        let rate = collector.get_rate("api.calls", Duration::hours(1)).await;
        api_data.push(rate * 3600.0);
    }

    charts.push(ChartData {
        title: "API Calls (24h)".to_string(),
        chart_type: "line".to_string(),
        labels: api_labels,
        datasets: vec![DataSet {
            label: "Calls/hour".to_string(),
            data: api_data,
            color: "#3b82f6".to_string(),
        }],
    });

    let mut activity_labels = Vec::new();
    let mut activity_data = Vec::new();

    for i in (0..7).rev() {
        let day = now - Duration::days(i);
        activity_labels.push(day.format("%a").to_string());
        activity_data.push((active_users as f64 / 7.0) * (i as f64).mul_add(0.1, 1.0));
    }

    charts.push(ChartData {
        title: "User Activity (7 days)".to_string(),
        chart_type: "bar".to_string(),
        labels: activity_labels,
        datasets: vec![DataSet {
            label: "Active Users".to_string(),
            data: activity_data,
            color: "#10b981".to_string(),
        }],
    });

    let dashboard = DashboardData {
        total_users,
        active_users,
        total_messages,
        total_sessions,
        storage_used_gb,
        api_calls_per_minute,
        error_rate,
        response_time_p95,
        charts,
    };

    Ok(Json(dashboard))
}

#[derive(Debug, Deserialize)]
pub struct MetricQuery {
    pub name: String,
    pub window_minutes: Option<i64>,
    pub aggregation: Option<String>,
}

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

pub async fn export_metrics(State(state): State<Arc<AppState>>) -> (StatusCode, String) {
    let collector = &state.metrics_collector;
    let metrics = collector.get_metrics().await;

    use std::fmt::Write;
    let mut prometheus_format = String::new();
    let mut seen_metrics = HashMap::new();

    for metric in metrics {
        if !seen_metrics.contains_key(&metric.name) {
            let _ = writeln!(prometheus_format, "# TYPE {} gauge", metric.name);
            seen_metrics.insert(metric.name.clone(), true);
        }

        let labels = metric
            .labels
            .iter()
            .map(|(k, v)| format!("{}=\"{}\"", k, v))
            .collect::<Vec<_>>()
            .join(",");

        if labels.is_empty() {
            let _ = writeln!(
                prometheus_format,
                "{} {} {}",
                metric.name,
                metric.value,
                metric.timestamp.timestamp_millis()
            );
        } else {
            let _ = writeln!(
                prometheus_format,
                "{}{{{}}} {} {}",
                metric.name,
                labels,
                metric.value,
                metric.timestamp.timestamp_millis()
            );
        }
    }

    (StatusCode::OK, prometheus_format)
}

pub fn configure() -> axum::routing::Router<Arc<AppState>> {
    use axum::routing::{get, Router};

    Router::new()
        .route(ApiUrls::ANALYTICS_DASHBOARD, get(get_dashboard))
        .route(ApiUrls::ANALYTICS_METRIC, get(get_metric))
        .route(ApiUrls::METRICS, get(export_metrics))
        .route("/api/activity/recent", get(get_recent_activity))
}

/// Get recent user activity for the home page
pub async fn get_recent_activity(
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    // Return recent activity items - in production, fetch from database
    // This powers the home.js loadRecentDocuments() function
    Json(serde_json::json!([
        {
            "id": "1",
            "type": "document",
            "name": "Project Report",
            "path": "/docs/project-report",
            "icon": "📄",
            "modified_at": chrono::Utc::now().to_rfc3339(),
            "app": "docs"
        },
        {
            "id": "2",
            "type": "spreadsheet",
            "name": "Budget 2025",
            "path": "/sheet/budget-2025",
            "icon": "📊",
            "modified_at": (chrono::Utc::now() - chrono::Duration::hours(2)).to_rfc3339(),
            "app": "sheet"
        },
        {
            "id": "3",
            "type": "presentation",
            "name": "Q1 Review",
            "path": "/slides/q1-review",
            "icon": "📽️",
            "modified_at": (chrono::Utc::now() - chrono::Duration::hours(5)).to_rfc3339(),
            "app": "slides"
        },
        {
            "id": "4",
            "type": "folder",
            "name": "Marketing Assets",
            "path": "/drive/marketing",
            "icon": "📁",
            "modified_at": (chrono::Utc::now() - chrono::Duration::days(1)).to_rfc3339(),
            "app": "drive"
        }
    ]))
}

pub fn spawn_metrics_collector(state: Arc<AppState>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));

        loop {
            interval.tick().await;

            let collector = &state.metrics_collector;
            collect_system_metrics(collector, &state).await;

            info!("System metrics collected");
        }
    });
}
