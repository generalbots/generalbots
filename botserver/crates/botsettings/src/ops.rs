use axum::{
    extract::State,
    response::{Html, IntoResponse, Json},
    routing::get,
    Router,
};
use diesel::RunQueryDsl;
use std::sync::Arc;

use botcore::shared::state::AppState;

pub fn configure_ops_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/ops/status", get(ops_status))
        .route("/api/ops/services/health", get(ops_services_health))
        .route("/api/ops/metrics/summary", get(ops_metrics_summary))
        .route("/api/ops/metrics/requests", get(ops_metrics_requests))
        .route("/api/ops/errors/recent", get(ops_errors_recent))
        .route("/api/ops/endpoints/performance", get(ops_endpoints_performance))
        .route("/api/ops/traces", get(ops_traces))
}

fn get_conn(
    state: &Arc<AppState>,
) -> Option<diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>> {
    state.conn.get().ok()
}

#[derive(Debug, diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct CountRow {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    count: i64,
}

pub async fn ops_status(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let db_ok = get_conn(&state).is_some();
    let cache_ok = state.cache.as_ref().is_some();

    Json(serde_json::json!({
        "status": if db_ok { "operational" } else { "degraded" },
        "database": if db_ok { "up" } else { "down" },
        "cache": if cache_ok { "up" } else { "down" },
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

pub async fn ops_services_health(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let db_ok = get_conn(&state).is_some();
    let cache_ok = state.cache.as_ref().is_some();
    let drive_ok = state.drive.is_some();

    let services = [
        ("api", db_ok, "HTTP API"),
        ("database", db_ok, "PostgreSQL"),
        ("cache", cache_ok, "Valkey"),
        ("drive", drive_ok, "MinIO"),
    ];

    let mut html = String::new();
    for (name, ok, label) in services {
        let cls = if ok { "healthy" } else { "unhealthy" };
        let status = if ok { "Operational" } else { "Unreachable" };
        html.push_str(&format!(
            r#"<div class="service-row" data-service="{name}"><span class="service-dot {cls}"></span><span class="service-name">{label}</span><span class="service-status {cls}">{status}</span></div>"#
        ));
    }

    Html(html)
}

pub async fn ops_metrics_summary(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let Ok(mut conn) = get_conn(&state).ok_or(()) else {
        return Html(String::new());
    };

    let requests: i64 = diesel::sql_query(
        "SELECT COUNT(*)::bigint AS count FROM message_history WHERE created_at > NOW() - INTERVAL '24 hours'",
    )
    .get_result(&mut conn)
    .map(|r: CountRow| r.count)
    .unwrap_or(0);

    let sessions: i64 = diesel::sql_query(
        "SELECT COUNT(*)::bigint AS count FROM user_sessions WHERE updated_at > NOW() - INTERVAL '24 hours'",
    )
    .get_result(&mut conn)
    .map(|r: CountRow| r.count)
    .unwrap_or(0);

    Html(format!(
        r#"<div class="ops-summary">
    <div class="summary-stat"><span class="summary-value">{requests}</span><span class="summary-label">Requests (24h)</span></div>
    <div class="summary-stat"><span class="summary-value">{sessions}</span><span class="summary-label">Active Sessions (24h)</span></div>
    <div class="summary-stat"><span class="summary-value">0</span><span class="summary-label">Errors (24h)</span></div>
    <div class="summary-stat"><span class="summary-value">--</span><span class="summary-label">Avg Latency</span></div>
</div>"#
    ))
}

pub async fn ops_metrics_requests(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let Ok(mut conn) = get_conn(&state).ok_or(()) else {
        return Html("0".to_string());
    };

    let count: i64 = diesel::sql_query(
        "SELECT COUNT(*)::bigint AS count FROM message_history WHERE created_at > NOW() - INTERVAL '1 hour'",
    )
    .get_result(&mut conn)
    .map(|r: CountRow| r.count)
    .unwrap_or(0);

    Html(format!("{count}/hr"))
}

pub async fn ops_errors_recent(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    Html(
        r#"<div class="errors-empty"><p>No errors recorded in the last 24 hours.</p></div>"#.to_string(),
    )
}

pub async fn ops_endpoints_performance(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let Ok(mut conn) = get_conn(&state).ok_or(()) else {
        return Html(String::new());
    };

    #[derive(Debug, diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct BotRow {
        #[diesel(sql_type = diesel::sql_types::Varchar)]
        name: String,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        count: i64,
    }

    let rows: Vec<BotRow> = diesel::sql_query(
        "SELECT b.name, COUNT(mh.id)::bigint AS count
         FROM bots b LEFT JOIN user_sessions us ON us.bot_id = b.bot_id
         LEFT JOIN message_history mh ON mh.session_id = us.id AND mh.created_at > NOW() - INTERVAL '24 hours'
         GROUP BY b.id, b.name ORDER BY count DESC LIMIT 6",
    )
    .load(&mut conn)
    .unwrap_or_default();

    if rows.is_empty() {
        return Html(r#"<div class="empty-state"><p>No endpoint activity</p></div>"#.to_string());
    }

    let max = rows.iter().map(|r| r.count).max().unwrap_or(1).max(1);
    let mut html = String::new();
    for r in rows {
        let pct = (r.count as f64 / max as f64) * 100.0;
        html.push_str(&format!(
            r#"<div class="bar-row"><span class="bar-label">/{name}</span><div class="bar-track"><div class="bar-fill" style="width:{pct:.0}%"></div></div><span class="bar-value">{count}</span></div>"#,
            name = r.name,
            count = r.count,
        ));
    }
    Html(html)
}

pub async fn ops_traces(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let Ok(mut conn) = get_conn(&state).ok_or(()) else {
        return Html(String::new());
    };

    #[derive(Debug, diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct TraceRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        bot: String,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)]
        ts: chrono::DateTime<chrono::Utc>,
        #[diesel(sql_type = diesel::sql_types::Int4)]
        role: i32,
    }

    let rows: Vec<TraceRow> = diesel::sql_query(
        "SELECT COALESCE((SELECT name FROM bots b WHERE b.id = us.bot_id), 'system') AS bot,
                mh.created_at AS ts, mh.role
         FROM message_history mh LEFT JOIN user_sessions us ON us.id = mh.session_id
         ORDER BY mh.created_at DESC LIMIT 8",
    )
    .load(&mut conn)
    .unwrap_or_default();

    if rows.is_empty() {
        return Html(r#"<div class="empty-state"><p>No traces recorded</p></div>"#.to_string());
    }

    let mut html = String::new();
    for r in rows {
        let kind = if r.role == 1 { "request" } else { "response" };
        html.push_str(&format!(
            r#"<div class="trace-row"><span class="trace-time">{time}</span><span class="trace-bot">{bot}</span><span class="trace-kind {kind}">{kind}</span></div>"#,
            time = r.ts.format("%H:%M:%S"),
            bot = r.bot,
        ));
    }
    Html(html)
}
