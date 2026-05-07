use std::fmt::Write as FmtWrite;
use std::sync::Arc;

use axum::{extract::State, response::Html, response::IntoResponse};
use diesel::RunQueryDsl;

use crate::analytics_types::*;
use crate::DbPool;

pub async fn handle_message_count(State(pool): State<Arc<DbPool>>) -> impl IntoResponse {
    let conn = pool.clone();

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

pub async fn handle_active_sessions(State(pool): State<Arc<DbPool>>) -> impl IntoResponse {
    let conn = pool.clone();

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

pub async fn handle_avg_response_time(State(pool): State<Arc<DbPool>>) -> impl IntoResponse {
    let conn = pool.clone();

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

pub async fn handle_llm_tokens(State(pool): State<Arc<DbPool>>) -> impl IntoResponse {
    let conn = pool.clone();

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

pub async fn handle_storage_usage(State(_pool): State<Arc<DbPool>>) -> impl IntoResponse {
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

pub async fn handle_errors_count(State(pool): State<Arc<DbPool>>) -> impl IntoResponse {
    let conn = pool.clone();

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
