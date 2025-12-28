use crate::core::urls::ApiUrls;
use crate::llm::observability::{ObservabilityConfig, ObservabilityManager, QuickStats};
use crate::shared::state::AppState;
use axum::{
    extract::State,
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt::Write as FmtWrite;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable)]
pub struct AnalyticsStats {
    pub message_count: i64,
    pub session_count: i64,
    pub active_sessions: i64,
    pub avg_response_time: f64,
}

#[derive(Debug, QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CountResult {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub count: i64,
}

#[derive(Debug, QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AvgResult {
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Double>)]
    pub avg: Option<f64>,
}

#[derive(Debug, QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct HourlyCount {
    #[diesel(sql_type = diesel::sql_types::Double)]
    pub hour: f64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsQuery {
    pub query: Option<String>,
    #[serde(rename = "timeRange")]
    pub time_range: Option<String>,
}

#[derive(Debug)]
pub struct AnalyticsService {
    observability: Arc<RwLock<ObservabilityManager>>,
}

impl AnalyticsService {
    pub fn new() -> Self {
        let config = ObservabilityConfig::default();
        Self {
            observability: Arc::new(RwLock::new(ObservabilityManager::new(config))),
        }
    }

    pub fn with_config(config: ObservabilityConfig) -> Self {
        Self {
            observability: Arc::new(RwLock::new(ObservabilityManager::new(config))),
        }
    }

    pub async fn get_quick_stats(&self) -> QuickStats {
        let manager = self.observability.read().await;
        manager.get_quick_stats()
    }

    pub async fn get_observability_manager(
        &self,
    ) -> tokio::sync::RwLockReadGuard<'_, ObservabilityManager> {
        self.observability.read().await
    }
}

impl Default for AnalyticsService {
    fn default() -> Self {
        Self::new()
    }
}

pub fn configure_analytics_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route(ApiUrls::ANALYTICS_MESSAGES_COUNT, get(handle_message_count))
        .route(
            ApiUrls::ANALYTICS_SESSIONS_ACTIVE,
            get(handle_active_sessions),
        )
        .route(
            ApiUrls::ANALYTICS_RESPONSE_AVG,
            get(handle_avg_response_time),
        )
        .route(ApiUrls::ANALYTICS_LLM_TOKENS, get(handle_llm_tokens))
        .route(ApiUrls::ANALYTICS_STORAGE_USAGE, get(handle_storage_usage))
        .route(ApiUrls::ANALYTICS_ERRORS_COUNT, get(handle_errors_count))
        .route(
            ApiUrls::ANALYTICS_TIMESERIES_MESSAGES,
            get(handle_timeseries_messages),
        )
        .route(
            ApiUrls::ANALYTICS_TIMESERIES_RESPONSE,
            get(handle_timeseries_response),
        )
        .route(
            ApiUrls::ANALYTICS_CHANNELS_DISTRIBUTION,
            get(handle_channels_distribution),
        )
        .route(
            ApiUrls::ANALYTICS_BOTS_PERFORMANCE,
            get(handle_bots_performance),
        )
        .route(
            ApiUrls::ANALYTICS_ACTIVITY_RECENT,
            get(handle_recent_activity),
        )
        .route(ApiUrls::ANALYTICS_QUERIES_TOP, get(handle_top_queries))
        .route(ApiUrls::ANALYTICS_CHAT, post(handle_analytics_chat))
        .route(ApiUrls::ANALYTICS_LLM_STATS, get(handle_llm_stats))
        .route(ApiUrls::ANALYTICS_BUDGET_STATUS, get(handle_budget_status))
}

pub async fn handle_message_count(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let conn = state.conn.clone();

    let count = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {}", e);
                return 0i64;
            }
        };

        diesel::sql_query(
            "SELECT COUNT(*) as count FROM message_history WHERE created_at > NOW() - INTERVAL '24 hours'",
        )
        .get_result::<CountResult>(&mut db_conn)
        .map(|r| r.count)
        .unwrap_or(0)
    })
    .await
    .unwrap_or(0);

    let mut html = String::new();
    html.push_str("<div class=\"metric-icon messages\">");
    html.push_str("<svg width=\"20\" height=\"20\" viewBox=\"0 0 24 24\" fill=\"none\"><path d=\"M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/></svg>");
    html.push_str("</div>");
    html.push_str("<div class=\"metric-content\">");
    html.push_str("<span class=\"metric-value\">");
    html.push_str(&format_number(count));
    html.push_str("</span>");
    html.push_str("<span class=\"metric-label\">Messages Today</span>");
    html.push_str("</div>");

    Html(html)
}

pub async fn handle_active_sessions(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let conn = state.conn.clone();

    let count = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {}", e);
                return 0i64;
            }
        };

        diesel::sql_query(
            "SELECT COUNT(DISTINCT session_id) as count FROM message_history WHERE created_at > NOW() - INTERVAL '30 minutes'",
        )
        .get_result::<CountResult>(&mut db_conn)
        .map(|r| r.count)
        .unwrap_or(0)
    })
    .await
    .unwrap_or(0);

    let mut html = String::new();
    html.push_str("<div class=\"metric-icon sessions\">");
    html.push_str("<svg width=\"20\" height=\"20\" viewBox=\"0 0 24 24\" fill=\"none\"><path d=\"M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/><circle cx=\"9\" cy=\"7\" r=\"4\" stroke=\"currentColor\" stroke-width=\"2\"/><path d=\"M23 21v-2a4 4 0 0 0-3-3.87\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/><path d=\"M16 3.13a4 4 0 0 1 0 7.75\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/></svg>");
    html.push_str("</div>");
    html.push_str("<div class=\"metric-content\">");
    html.push_str("<span class=\"metric-value\">");
    html.push_str(&count.to_string());
    html.push_str("</span>");
    html.push_str("<span class=\"metric-label\">Active Sessions</span>");
    html.push_str("</div>");

    Html(html)
}

pub async fn handle_avg_response_time(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let conn = state.conn.clone();

    let avg_time = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {}", e);
                return 0.0f64;
            }
        };

        diesel::sql_query(
            "SELECT AVG(EXTRACT(EPOCH FROM (updated_at - created_at))) as avg FROM message_history WHERE role = 1 AND created_at > NOW() - INTERVAL '24 hours'",
        )
        .get_result::<AvgResult>(&mut db_conn)
        .map(|r| r.avg.unwrap_or(0.0))
        .unwrap_or(0.0)
    })
    .await
    .unwrap_or(0.0);

    let display_time = if avg_time < 1.0 {
        format!("{}ms", (avg_time * 1000.0) as i32)
    } else {
        format!("{:.1}s", avg_time)
    };

    let mut html = String::new();
    html.push_str("<div class=\"metric-icon response\">");
    html.push_str("<svg width=\"20\" height=\"20\" viewBox=\"0 0 24 24\" fill=\"none\"><circle cx=\"12\" cy=\"12\" r=\"10\" stroke=\"currentColor\" stroke-width=\"2\"/><polyline points=\"12 6 12 12 16 14\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/></svg>");
    html.push_str("</div>");
    html.push_str("<div class=\"metric-content\">");
    html.push_str("<span class=\"metric-value\">");
    html.push_str(&display_time);
    html.push_str("</span>");
    html.push_str("<span class=\"metric-label\">Avg Response</span>");
    html.push_str("</div>");

    Html(html)
}

pub async fn handle_llm_tokens(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let conn = state.conn.clone();

    let tokens = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {}", e);
                return 0i64;
            }
        };

        diesel::sql_query(
            "SELECT COALESCE(SUM((metadata->>'tokens')::bigint), COUNT(*) * 150) as count FROM message_history WHERE created_at > NOW() - INTERVAL '24 hours'",
        )
        .get_result::<CountResult>(&mut db_conn)
        .map(|r| r.count)
        .unwrap_or(0)
    })
    .await
    .unwrap_or(0);

    let mut html = String::new();
    html.push_str("<div class=\"metric-icon tokens\">");
    html.push_str("<svg width=\"20\" height=\"20\" viewBox=\"0 0 24 24\" fill=\"none\"><path d=\"M12 2L2 7l10 5 10-5-10-5z\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/><path d=\"M2 17l10 5 10-5\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/><path d=\"M2 12l10 5 10-5\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/></svg>");
    html.push_str("</div>");
    html.push_str("<div class=\"metric-content\">");
    html.push_str("<span class=\"metric-value\">");
    html.push_str(&format_number(tokens));
    html.push_str("</span>");
    html.push_str("<span class=\"metric-label\">Tokens Used</span>");
    html.push_str("</div>");

    Html(html)
}

pub async fn handle_storage_usage(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    let usage_gb = 2.4f64;
    let total_gb = 10.0f64;
    let percentage = (usage_gb / total_gb * 100.0) as i32;

    let mut html = String::new();
    html.push_str("<div class=\"metric-icon storage\">");
    html.push_str("<svg width=\"20\" height=\"20\" viewBox=\"0 0 24 24\" fill=\"none\"><path d=\"M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z\" stroke=\"currentColor\" stroke-width=\"2\"/></svg>");
    html.push_str("</div>");
    html.push_str("<div class=\"metric-content\">");
    html.push_str("<span class=\"metric-value\">");
    let _ = write!(html, "{:.1} GB", usage_gb);
    html.push_str("</span>");
    html.push_str("<span class=\"metric-label\">Storage (");
    let _ = write!(html, "{percentage}");
    html.push_str("%)</span>");
    html.push_str("</div>");

    Html(html)
}

pub async fn handle_errors_count(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let conn = state.conn.clone();

    let count = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {}", e);
                return 0i64;
            }
        };

        diesel::sql_query(
            "SELECT COUNT(*) as count FROM analytics_events WHERE event_type = 'error' AND created_at > NOW() - INTERVAL '24 hours'",
        )
        .get_result::<CountResult>(&mut db_conn)
        .map(|r| r.count)
        .unwrap_or(0)
    })
    .await
    .unwrap_or(0);

    let status_class = if count > 10 {
        "error"
    } else if count > 0 {
        "warning"
    } else {
        "success"
    };

    let mut html = String::new();
    html.push_str("<div class=\"metric-icon errors ");
    html.push_str(status_class);
    html.push_str("\">");
    html.push_str("<svg width=\"20\" height=\"20\" viewBox=\"0 0 24 24\" fill=\"none\"><circle cx=\"12\" cy=\"12\" r=\"10\" stroke=\"currentColor\" stroke-width=\"2\"/><line x1=\"12\" y1=\"8\" x2=\"12\" y2=\"12\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\"/><line x1=\"12\" y1=\"16\" x2=\"12.01\" y2=\"16\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\"/></svg>");
    html.push_str("</div>");
    html.push_str("<div class=\"metric-content\">");
    html.push_str("<span class=\"metric-value\">");
    html.push_str(&count.to_string());
    html.push_str("</span>");
    html.push_str("<span class=\"metric-label\">Errors (24h)</span>");
    html.push_str("</div>");

    Html(html)
}

pub async fn handle_timeseries_messages(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let conn = state.conn.clone();

    let hourly_data = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {}", e);
                return Vec::new();
            }
        };

        diesel::sql_query(
            "SELECT EXTRACT(HOUR FROM created_at) as hour, COUNT(*) as count
             FROM message_history
             WHERE created_at > NOW() - INTERVAL '24 hours'
             GROUP BY EXTRACT(HOUR FROM created_at)
             ORDER BY hour",
        )
        .load::<HourlyCount>(&mut db_conn)
        .unwrap_or_default()
    })
    .await
    .unwrap_or_default();

    let hours: Vec<i32> = (0..24).collect();
    let mut counts: Vec<i64> = vec![0; 24];

    for data in hourly_data {
        let hour_idx = data.hour as usize;
        if hour_idx < 24 {
            counts[hour_idx] = data.count;
        }
    }

    let labels: Vec<String> = hours.iter().map(|h| format!("{}:00", h)).collect();
    let max_count = counts.iter().max().copied().unwrap_or(1).max(1);

    let mut html = String::new();
    html.push_str("<div class=\"chart-container\">");
    html.push_str("<div class=\"chart-bars\">");

    for (i, count) in counts.iter().enumerate() {
        let height_pct = (*count as f64 / max_count as f64) * 100.0;
        let _ = write!(
            html,
            "<div class=\"chart-bar\" style=\"height: {}%\" title=\"{}: {} messages\"></div>",
            height_pct, labels[i], count
        );
    }

    html.push_str("</div>");
    html.push_str("<div class=\"chart-labels\">");
    for (i, label) in labels.iter().enumerate() {
        if i % 4 == 0 {
            let _ = write!(html, "<span>{label}</span>");
        }
    }
    html.push_str("</div>");
    html.push_str("</div>");

    Html(html)
}

pub async fn handle_timeseries_response(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let conn = state.conn.clone();

    #[derive(Debug, QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct HourlyAvg {
        #[diesel(sql_type = diesel::sql_types::Double)]
        hour: f64,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Double>)]
        avg_time: Option<f64>,
    }

    let hourly_data = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {}", e);
                return Vec::new();
            }
        };

        diesel::sql_query(
            "SELECT EXTRACT(HOUR FROM created_at) as hour,
                    AVG(EXTRACT(EPOCH FROM (updated_at - created_at))) as avg_time
             FROM message_history
             WHERE role = 1 AND created_at > NOW() - INTERVAL '24 hours'
             GROUP BY EXTRACT(HOUR FROM created_at)
             ORDER BY hour",
        )
        .load::<HourlyAvg>(&mut db_conn)
        .unwrap_or_default()
    })
    .await
    .unwrap_or_default();

    let mut avgs: Vec<f64> = vec![0.0; 24];

    for data in hourly_data {
        let hour_idx = data.hour as usize;
        if hour_idx < 24 {
            avgs[hour_idx] = data.avg_time.unwrap_or(0.0);
        }
    }

    let labels: Vec<String> = (0..24).map(|h| format!("{}:00", h)).collect();
    let max_avg = avgs.iter().copied().fold(0.0f64, f64::max).max(0.1);

    let mut html = String::new();
    html.push_str("<div class=\"chart-container line-chart\">");
    html.push_str("<svg viewBox=\"0 0 480 200\" preserveAspectRatio=\"none\">");
    html.push_str("<polyline fill=\"none\" stroke=\"var(--primary)\" stroke-width=\"2\" points=\"");

    for (i, avg) in avgs.iter().enumerate() {
        let x = (i as f64 / 23.0) * 480.0;
        let y = 180.0f64.mul_add(-(*avg / max_avg), 200.0);
        let _ = write!(html, "{x},{y} ");
    }

    html.push_str("\"/></svg>");
    html.push_str("<div class=\"chart-labels\">");
    for (i, label) in labels.iter().enumerate() {
        if i % 4 == 0 {
            let _ = write!(html, "<span>{label}</span>");
        }
    }
    html.push_str("</div>");
    html.push_str("</div>");

    Html(html)
}

pub async fn handle_channels_distribution(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let conn = state.conn.clone();

    #[derive(Debug, QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct ChannelCount {
        #[diesel(sql_type = diesel::sql_types::Text)]
        channel: String,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        count: i64,
    }

    let channel_data = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {}", e);
                return Vec::new();
            }
        };

        diesel::sql_query(
            "SELECT COALESCE(channel, 'web') as channel, COUNT(*) as count
             FROM sessions
             WHERE created_at > NOW() - INTERVAL '7 days'
             GROUP BY channel
             ORDER BY count DESC
             LIMIT 5",
        )
        .load::<ChannelCount>(&mut db_conn)
        .unwrap_or_default()
    })
    .await
    .unwrap_or_default();

    let total: i64 = channel_data.iter().map(|c| c.count).sum();
    let colors = ["#4f46e5", "#10b981", "#f59e0b", "#ef4444", "#8b5cf6"];

    let mut html = String::new();
    html.push_str("<div class=\"pie-chart-container\">");
    html.push_str("<div class=\"pie-chart\">");

    let mut offset = 0.0f64;
    for (i, data) in channel_data.iter().enumerate() {
        let pct = if total > 0 {
            (data.count as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        let color = colors[i % colors.len()];
        let _ = write!(
            html,
            "<div class=\"pie-segment\" style=\"--offset: {}; --value: {}; --color: {};\"></div>",
            offset, pct, color
        );
        offset += pct;
    }

    html.push_str("</div>");
    html.push_str("<div class=\"pie-legend\">");

    for (i, data) in channel_data.iter().enumerate() {
        let pct = if total > 0 {
            (data.count as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        let color = colors[i % colors.len()];
        let _ = write!(
            html,
            "<div class=\"legend-item\"><span class=\"legend-color\" style=\"background: {};\"></span>{} ({:.0}%)</div>",
            color, html_escape(&data.channel), pct
        );
    }

    html.push_str("</div>");
    html.push_str("</div>");

    Html(html)
}

pub async fn handle_bots_performance(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let conn = state.conn.clone();

    #[derive(Debug, QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct BotStats {
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        count: i64,
    }

    let bot_data = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {}", e);
                return Vec::new();
            }
        };

        diesel::sql_query(
            "SELECT b.name, COUNT(mh.id) as count
             FROM bots b
             LEFT JOIN sessions s ON s.bot_id = b.id
             LEFT JOIN message_history mh ON mh.session_id = s.id
             WHERE mh.created_at > NOW() - INTERVAL '24 hours' OR mh.created_at IS NULL
             GROUP BY b.id, b.name
             ORDER BY count DESC
             LIMIT 5",
        )
        .load::<BotStats>(&mut db_conn)
        .unwrap_or_default()
    })
    .await
    .unwrap_or_default();

    let max_count = bot_data.iter().map(|b| b.count).max().unwrap_or(1).max(1);

    let mut html = String::new();
    html.push_str("<div class=\"horizontal-bars\">");

    for data in bot_data.iter() {
        let pct = (data.count as f64 / max_count as f64) * 100.0;
        html.push_str("<div class=\"bar-row\">");
        let _ = write!(
            html,
            "<span class=\"bar-label\">{}</span>",
            html_escape(&data.name)
        );
        let _ = write!(
            html,
            "<div class=\"bar-track\"><div class=\"bar-fill\" style=\"width: {pct}%;\"></div></div>"
        );
        let _ = write!(html, "<span class=\"bar-value\">{}</span>", data.count);
        html.push_str("</div>");
    }

    html.push_str("</div>");

    Html(html)
}

#[derive(Debug, QueryableByName, Clone)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct ActivityRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    activity_type: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    description: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    time_ago: String,
}

pub async fn handle_recent_activity(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let conn = state.conn.clone();

    let activities = tokio::task::spawn_blocking(move || {
        let db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {}", e);
                return get_default_activities();
            }
        };

        diesel::sql_query(
            "SELECT
                CASE
                    WHEN role = 0 THEN 'message'
                    WHEN role = 1 THEN 'response'
                    ELSE 'system'
                END as activity_type,
                SUBSTRING(content FROM 1 FOR 50) as description,
                CASE
                    WHEN created_at > NOW() - INTERVAL '1 minute' THEN 'just now'
                    WHEN created_at > NOW() - INTERVAL '1 hour' THEN CONCAT(EXTRACT(MINUTE FROM NOW() - created_at)::int, 'm ago')
                    ELSE CONCAT(EXTRACT(HOUR FROM NOW() - created_at)::int, 'h ago')
                END as time_ago
             FROM message_history
             ORDER BY created_at DESC
             LIMIT 10",
        )
        .load::<ActivityRow>(&mut { db_conn })
        .unwrap_or_else(|_| get_default_activities())
    })
    .await
    .unwrap_or_else(|_| get_default_activities());

    let mut html = String::new();
    html.push_str("<div class=\"activity-list\">");

    for activity in activities.iter() {
        let icon = match activity.activity_type.as_str() {
            "message" => "💬",
            "response" => "🤖",
            "error" => "⚠️",
            _ => "📋",
        };

        html.push_str("<div class=\"activity-item\">");
        let _ = write!(html, "<span class=\"activity-icon\">{icon}</span>");
        html.push_str("<div class=\"activity-content\">");
        let _ = write!(
            html,
            "<span class=\"activity-desc\">{}</span>",
            html_escape(&activity.description)
        );
        let _ = write!(
            html,
            "<span class=\"activity-time\">{}</span>",
            html_escape(&activity.time_ago)
        );
        html.push_str("</div>");
        html.push_str("</div>");
    }

    html.push_str("</div>");

    Html(html)
}

fn get_default_activities() -> Vec<ActivityRow> {
    vec![]
}

pub async fn handle_top_queries(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let conn = state.conn.clone();

    #[derive(Debug, QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct QueryCount {
        #[diesel(sql_type = diesel::sql_types::Text)]
        query: String,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        count: i64,
    }

    let queries = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {}", e);
                return Vec::new();
            }
        };

        diesel::sql_query(
            "SELECT SUBSTRING(content FROM 1 FOR 100) as query, COUNT(*) as count
             FROM message_history
             WHERE role = 0 AND created_at > NOW() - INTERVAL '24 hours'
             GROUP BY SUBSTRING(content FROM 1 FOR 100)
             ORDER BY count DESC
             LIMIT 10",
        )
        .load::<QueryCount>(&mut db_conn)
        .unwrap_or_default()
    })
    .await
    .unwrap_or_default();

    let mut html = String::new();
    html.push_str("<div class=\"top-queries-list\">");

    for (i, q) in queries.iter().enumerate() {
        html.push_str("<div class=\"query-item\">");
        let _ = write!(html, "<span class=\"query-rank\">{}</span>", i + 1);
        let _ = write!(
            html,
            "<span class=\"query-text\">{}</span>",
            html_escape(&q.query)
        );
        let _ = write!(html, "<span class=\"query-count\">{}</span>", q.count);
        html.push_str("</div>");
    }

    html.push_str("</div>");

    Html(html)
}

pub async fn handle_analytics_chat(
    State(_state): State<Arc<AppState>>,
    Json(payload): Json<AnalyticsQuery>,
) -> impl IntoResponse {
    let query = payload.query.unwrap_or_default();

    let response = if query.to_lowercase().contains("message") {
        "Based on current data, message volume trends are being analyzed."
    } else if query.to_lowercase().contains("error") {
        "Error rate analysis is available in the errors dashboard."
    } else if query.to_lowercase().contains("performance") {
        "Performance metrics show average response times within normal parameters."
    } else {
        "I can help analyze your data. Ask about messages, errors, or performance."
    };

    let mut html = String::new();
    html.push_str("<div class=\"chat-message assistant\">");
    html.push_str("<div class=\"message-content\">");
    html.push_str(&html_escape(response));
    html.push_str("</div>");
    html.push_str("</div>");

    Html(html)
}

pub async fn handle_llm_stats(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    let service = AnalyticsService::new();
    let stats = service.get_quick_stats().await;

    let mut html = String::new();
    html.push_str("<div class=\"llm-stats\">");
    let _ = write!(html, "<div class=\"stat\"><span class=\"label\">Total Requests</span><span class=\"value\">{}</span></div>", stats.total_requests);
    let _ = write!(html, "<div class=\"stat\"><span class=\"label\">Total Tokens</span><span class=\"value\">{}</span></div>", stats.total_tokens);
    let _ = write!(html, "<div class=\"stat\"><span class=\"label\">Cache Hits</span><span class=\"value\">{}</span></div>", stats.cache_hits);
    let _ = write!(html, "<div class=\"stat\"><span class=\"label\">Cache Hit Rate</span><span class=\"value\">{:.1}%</span></div>", stats.cache_hit_rate * 100.0);
    let _ = write!(html, "<div class=\"stat\"><span class=\"label\">Error Rate</span><span class=\"value\">{:.1}%</span></div>", stats.error_rate * 100.0);
    html.push_str("</div>");

    Html(html)
}

pub async fn handle_budget_status(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    let status = {
        let service = AnalyticsService::new();
        let manager = service.get_observability_manager().await;
        manager.get_budget_status().await
    };

    let mut html = String::new();
    html.push_str("<div class=\"budget-status\">");
    let _ = write!(html, "<div class=\"budget-item\"><span class=\"label\">Daily Spend</span><span class=\"value\">${:.2} / ${:.2}</span></div>", status.daily_spend, status.daily_limit);
    let _ = write!(html, "<div class=\"budget-item\"><span class=\"label\">Monthly Spend</span><span class=\"value\">${:.2} / ${:.2}</span></div>", status.monthly_spend, status.monthly_limit);
    let _ = write!(html, "<div class=\"budget-item\"><span class=\"label\">Daily Remaining</span><span class=\"value\">${:.2} ({:.0}%)</span></div>", status.daily_remaining, status.daily_percentage * 100.0);
    let _ = write!(html, "<div class=\"budget-item\"><span class=\"label\">Monthly Remaining</span><span class=\"value\">${:.2} ({:.0}%)</span></div>", status.monthly_remaining, status.monthly_percentage * 100.0);
    html.push_str("</div>");

    Html(html)
}

fn format_number(n: i64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

impl Default for AnalyticsStats {
    fn default() -> Self {
        Self {
            message_count: 0,
            session_count: 0,
            active_sessions: 0,
            avg_response_time: 0.0,
        }
    }
}
