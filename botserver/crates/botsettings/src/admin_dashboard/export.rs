use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse},
};
use chrono::{DateTime, Utc};
use diesel::RunQueryDsl;
use std::sync::Arc;

use botcore::shared::state::AppState;

use super::{count_query, get_conn};

pub async fn export_report(
    State(state): State<Arc<AppState>>,
) -> Result<axum::response::Response, StatusCode> {
    let mut conn = match get_conn(&state) {
        Some(c) => c,
        None => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    let members = count_query(&mut conn, "SELECT COUNT(*) as count FROM users");
    let bots = count_query(&mut conn, "SELECT COUNT(*) as count FROM bots");
    let conversations = count_query(
        &mut conn,
        "SELECT COUNT(DISTINCT session_id) as count FROM message_history WHERE created_at > NOW() - INTERVAL '7 days'",
    );
    let messages = count_query(
        &mut conn,
        "SELECT COUNT(*) as count FROM message_history WHERE created_at > NOW() - INTERVAL '24 hours'",
    );

    let body = format!(
        "General Bots Organization Report\nGenerated: {}\n\nMembers: {}\nActive Bots: {}\nConversations (7d): {}\nMessages (24h): {}\n",
        Utc::now().to_rfc3339(),
        members,
        bots,
        conversations,
        messages,
    );

    Ok(axum::response::Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/plain")
        .header(
            "Content-Disposition",
            "attachment; filename=\"org-report.txt\"",
        )
        .body(axum::body::Body::from(body))
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response()))
}

pub async fn stats_users(State(state): State<Arc<AppState>>) -> axum::Json<serde_json::Value> {
    let mut conn = match get_conn(&state) {
        Some(c) => c,
        None => return axum::Json(serde_json::json!({ "error": "db unavailable" })),
    };
    let total = count_query(&mut conn, "SELECT COUNT(*) as count FROM users");
    axum::Json(serde_json::json!({ "total": total }))
}

pub async fn stats_bots(State(state): State<Arc<AppState>>) -> axum::Json<serde_json::Value> {
    let mut conn = match get_conn(&state) {
        Some(c) => c,
        None => return axum::Json(serde_json::json!({ "error": "db unavailable" })),
    };
    let total = count_query(&mut conn, "SELECT COUNT(*) as count FROM bots");
    axum::Json(serde_json::json!({ "total": total }))
}

pub async fn stats_groups(State(state): State<Arc<AppState>>) -> axum::Json<serde_json::Value> {
    let mut conn = match get_conn(&state) {
        Some(c) => c,
        None => return axum::Json(serde_json::json!({ "error": "db unavailable" })),
    };
    let total = count_query(&mut conn, "SELECT COUNT(*) as count FROM rbac_groups");
    axum::Json(serde_json::json!({ "total": total }))
}

pub async fn stats_storage(State(state): State<Arc<AppState>>) -> axum::Json<serde_json::Value> {
    let mut conn = match get_conn(&state) {
        Some(c) => c,
        None => return axum::Json(serde_json::json!({ "error": "db unavailable" })),
    };
    let used_bytes = count_query(
        &mut conn,
        "SELECT COALESCE(SUM(file_size), 0) as count FROM kb_documents",
    );
    axum::Json(serde_json::json!({
        "used_bytes": used_bytes,
        "used_gb": used_bytes as f64 / 1_073_741_824.0,
    }))
}

pub async fn list_users(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> axum::response::Response {
    let mut conn = match get_conn(&state) {
        Some(c) => c,
        None => return axum::Json(serde_json::json!([])).into_response(),
    };

    #[derive(Debug, diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct UserRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: uuid::Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)]
        username: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        email: String,
        #[diesel(sql_type = diesel::sql_types::Bool)]
        is_active: bool,
        #[diesel(sql_type = diesel::sql_types::Bool)]
        is_admin: bool,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)]
        created_at: DateTime<Utc>,
    }

    let rows: Vec<UserRow> = diesel::sql_query(
        "SELECT id, username, email, is_active, is_admin, created_at FROM users ORDER BY created_at DESC LIMIT 50",
    )
    .load::<UserRow>(&mut conn)
    .unwrap_or_default();

    // Settings → Users swaps this endpoint into the page with HTMX (#1839).
    // Serve the same rows as an HTML table for that request, while keeping the
    // JSON contract for API/automation callers.
    let wants_html = headers
        .get("HX-Request")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    if wants_html {
        let mut html = String::from(
            "<table class=\"data-table users-table\"><thead><tr>\
             <th>User</th><th>Email</th><th>Role</th><th>Status</th><th>Created</th>\
             </tr></thead><tbody>",
        );
        if rows.is_empty() {
            html.push_str(
                "<tr><td colspan=\"5\"><div class=\"empty-state\"><p>No users found</p></div></td></tr>",
            );
        }
        for user in &rows {
            let role = if user.is_admin {
                "<span class=\"role-badge admin\">Admin</span>"
            } else {
                "<span class=\"role-badge\">User</span>"
            };
            let status = if user.is_active {
                "<span class=\"status-badge active\">Active</span>"
            } else {
                "<span class=\"status-badge inactive\">Inactive</span>"
            };
            html.push_str(&format!(
                "<tr><td>{username}</td><td>{email}</td><td>{role}</td>\
                 <td>{status}</td><td>{created}</td></tr>",
                username = crate::webhook_delivery::escape_html(&user.username),
                email = crate::webhook_delivery::escape_html(&user.email),
                role = role,
                status = status,
                created = user.created_at.format("%Y-%m-%d"),
            ));
        }
        html.push_str("</tbody></table>");
        return Html(html).into_response();
    }

    let users: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|u| {
            serde_json::json!({
                "id": u.id,
                "username": u.username,
                "email": u.email,
                "is_active": u.is_active,
                "is_admin": u.is_admin,
                "created_at": u.created_at,
            })
        })
        .collect();

    axum::Json(serde_json::json!(users)).into_response()
}

pub async fn list_groups(State(state): State<Arc<AppState>>) -> axum::Json<serde_json::Value> {
    let mut conn = match get_conn(&state) {
        Some(c) => c,
        None => return axum::Json(serde_json::json!([])),
    };

    #[derive(Debug, diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct GroupRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: uuid::Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
    }

    // `rbac_user_groups` is the membership table (user_id, group_id) — it has
    // no `name` column (#1942). The group itself lives in `rbac_groups`.
    let rows: Vec<GroupRow> =
        diesel::sql_query("SELECT id, name FROM rbac_groups ORDER BY name")
            .load::<GroupRow>(&mut conn)
            .unwrap_or_default();

    let groups: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|g| serde_json::json!({ "id": g.id, "name": g.name }))
        .collect();

    axum::Json(serde_json::json!(groups))
}

pub async fn list_dns(State(state): State<Arc<AppState>>) -> axum::Json<serde_json::Value> {
    let mut conn = match get_conn(&state) {
        Some(c) => c,
        None => return axum::Json(serde_json::json!([])),
    };

    #[derive(Debug, diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct DnsRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: uuid::Uuid,
        #[diesel(sql_type = diesel::sql_types::Varchar)]
        domain: String,
    }

    let rows: Vec<DnsRow> = diesel::sql_query("SELECT id, domain FROM bot_domains ORDER BY domain")
        .load::<DnsRow>(&mut conn)
        .unwrap_or_default();

    let domains: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|d| serde_json::json!({ "id": d.id, "domain": d.domain }))
        .collect();

    axum::Json(serde_json::json!(domains))
}
