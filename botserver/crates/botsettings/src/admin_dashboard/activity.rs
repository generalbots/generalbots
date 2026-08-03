use axum::{
    extract::{Query, State},
    response::Html,
};
use chrono::{DateTime, Utc};
use diesel::RunQueryDsl;
use serde::Deserialize;
use std::sync::Arc;

use botcore::shared::state::AppState;

use super::get_conn;

#[derive(Deserialize)]
pub struct ActivityQuery {
    pub page: Option<u32>,
}

pub async fn dashboard_activity(State(state): State<Arc<AppState>>, Query(query): Query<ActivityQuery>) -> Html<String> {
    let page = query.page.unwrap_or(1).max(1);
    let limit = 8;
    let offset = (page - 1) * limit;

    let mut conn = match get_conn(&state) {
        Some(c) => c,
        None => return Html(String::new()),
    };

    #[derive(Debug, diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct ActivityRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        kind: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        title: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        detail: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)]
        ts: DateTime<Utc>,
    }

    let rows: Vec<ActivityRow> = diesel::sql_query(
        "SELECT 'member' as kind, COALESCE(email, username) as title, 'joined the organization' as detail, created_at as ts FROM users
         UNION ALL
         SELECT 'bot', name, 'bot status updated', updated_at FROM bots
         UNION ALL
         SELECT 'session', 'Conversation', 'new session started', created_at FROM user_sessions
         ORDER BY ts DESC LIMIT $1 OFFSET $2",
    )
    .bind::<diesel::sql_types::Int4, _>(limit as i32)
    .bind::<diesel::sql_types::Int4, _>(offset as i32)
    .load::<ActivityRow>(&mut conn)
    .unwrap_or_default();

    if rows.is_empty() && page == 1 {
        return Html(
            r##"<div class="activity-empty"><p>No recent activity</p></div>"##.to_string(),
        );
    }

    let mut items = String::new();
    for row in rows {
        let (avatar, time) = match row.kind.as_str() {
            "bot" => (
                r##"<div class="activity-avatar bot">
    <svg viewBox="0 0 24 24" width="20" height="20" stroke="currentColor" stroke-width="2" fill="none">
        <rect x="3" y="11" width="18" height="10" rx="2"></rect><circle cx="12" cy="5" r="2"></circle><path d="M12 7v4"></path>
    </svg>
</div>"##,
                relative_time(row.ts),
            ),
            "session" => (
                r##"<div class="activity-avatar">
    <svg viewBox="0 0 24 24" width="20" height="20" stroke="currentColor" stroke-width="2" fill="none">
        <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"></path>
    </svg>
</div>"##,
                relative_time(row.ts),
            ),
            _ => (
                r##"<div class="activity-avatar"><img src="/suite/assets/avatars/default.svg" alt="User avatar"></div>"##,
                relative_time(row.ts),
            ),
        };

        let detail = row.detail.unwrap_or_default();
        items.push_str(&format!(
            r##"<div class="activity-item {kind}">
    {avatar}
    <div class="activity-content">
        <div class="activity-text"><strong>{title}</strong> {detail}</div>
        <span class="activity-time">{time}</span>
    </div>
</div>"##,
            kind = if row.kind == "session" { "" } else { &row.kind },
            title = row.title,
            detail = detail,
        ));
    }

    Html(items)
}

pub async fn activity_recent(State(state): State<Arc<AppState>>) -> axum::Json<serde_json::Value> {
    let mut conn = match get_conn(&state) {
        Some(c) => c,
        None => return axum::Json(serde_json::json!([])),
    };

    #[derive(Debug, diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct ActivityRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        kind: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        title: String,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)]
        ts: DateTime<Utc>,
    }

    let rows: Vec<ActivityRow> = diesel::sql_query(
        "SELECT 'member' as kind, COALESCE(email, username) as title, created_at as ts FROM users
         UNION ALL
         SELECT 'bot', name, updated_at FROM bots
         ORDER BY ts DESC LIMIT 10",
    )
    .load::<ActivityRow>(&mut conn)
    .unwrap_or_default();

    let items: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|r| {
            serde_json::json!({
                "type": r.kind,
                "name": r.title,
                "path": "#",
                "icon": if r.kind == "bot" { "bot" } else { "user" },
                "modified_at": r.ts.to_rfc3339(),
                "app": r.kind,
            })
        })
        .collect();

    axum::Json(serde_json::json!(items))
}

fn relative_time(time: DateTime<Utc>) -> String {
    let diff = Utc::now() - time;
    if diff.num_days() > 0 {
        format!("{}d ago", diff.num_days())
    } else if diff.num_hours() > 0 {
        format!("{}h ago", diff.num_hours())
    } else if diff.num_minutes() > 0 {
        format!("{}m ago", diff.num_minutes())
    } else {
        "just now".to_string()
    }
}
