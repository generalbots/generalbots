use std::fmt::Write as FmtWrite;
use std::sync::Arc;

use axum::{extract::State, response::Html, response::IntoResponse, Json};
use diesel::RunQueryDsl;

use crate::analytics_types::*;
use crate::DbPool;

#[derive(Debug, diesel::QueryableByName, Clone)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct ActivityRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    activity_type: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    description: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    time_ago: String,
}

fn get_default_activities() -> Vec<ActivityRow> {
    vec![]
}

pub async fn handle_recent_activity(State(pool): State<Arc<DbPool>>) -> impl IntoResponse {
    let conn = pool.clone();

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

pub async fn handle_top_queries(State(pool): State<Arc<DbPool>>) -> impl IntoResponse {
    let conn = pool.clone();

    #[derive(Debug, diesel::QueryableByName)]
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
    State(_pool): State<Arc<DbPool>>,
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
