use axum::{
    extract::{Query, State},
    response::Html,
};
use diesel::RunQueryDsl;
use serde::Deserialize;
use std::sync::Arc;

use botcore::shared::state::AppState;

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[derive(Debug, diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct RoleRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: uuid::Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    name: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    display_name: String,
}

#[derive(Debug, diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct GroupRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: uuid::Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    name: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    display_name: String,
}

#[derive(Debug, diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct UserRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: uuid::Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    username: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    email: String,
}

pub async fn search_roles(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SearchQuery>,
) -> Html<String> {
    let Ok(mut conn) = state.conn.get() else {
        return Html(String::new());
    };

    let pattern = query
        .q
        .clone()
        .unwrap_or_default()
        .trim()
        .to_string();

    let rows: Vec<RoleRow> = if pattern.is_empty() {
        diesel::sql_query(
            "SELECT id, name, display_name FROM rbac_roles WHERE is_active = true ORDER BY display_name LIMIT 20",
        )
        .load(&mut conn)
        .unwrap_or_default()
    } else {
        let p = format!("%{pattern}%");
        diesel::sql_query(
            "SELECT id, name, display_name FROM rbac_roles WHERE is_active = true AND (name ILIKE $1 OR display_name ILIKE $1) ORDER BY display_name LIMIT 20",
        )
        .bind::<diesel::sql_types::Text, _>(p)
        .load(&mut conn)
        .unwrap_or_default()
    };

    if rows.is_empty() {
        return Html(r#"<p class="empty-text">No roles found</p>"#.to_string());
    }

    let mut html = String::new();
    for r in rows {
        html.push_str(&format!(
            r#"<div class="search-result-item" data-id="{id}" data-name="{name}"><span class="result-title">{display}</span><span class="result-slug">@{name}</span></div>"#,
            id = r.id,
            name = html_escape(&r.name),
            display = html_escape(&r.display_name),
        ));
    }
    Html(html)
}

pub async fn search_groups(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SearchQuery>,
) -> Html<String> {
    let Ok(mut conn) = state.conn.get() else {
        return Html(String::new());
    };

    let pattern = query.q.clone().unwrap_or_default().trim().to_string();

    let rows: Vec<GroupRow> = if pattern.is_empty() {
        diesel::sql_query(
            "SELECT id, name, display_name FROM rbac_groups WHERE is_active = true ORDER BY display_name LIMIT 20",
        )
        .load(&mut conn)
        .unwrap_or_default()
    } else {
        let p = format!("%{pattern}%");
        diesel::sql_query(
            "SELECT id, name, display_name FROM rbac_groups WHERE is_active = true AND (name ILIKE $1 OR display_name ILIKE $1) ORDER BY display_name LIMIT 20",
        )
        .bind::<diesel::sql_types::Text, _>(p)
        .load(&mut conn)
        .unwrap_or_default()
    };

    if rows.is_empty() {
        return Html(r#"<p class="empty-text">No groups found</p>"#.to_string());
    }

    let mut html = String::new();
    for r in rows {
        html.push_str(&format!(
            r#"<div class="search-result-item" data-id="{id}" data-name="{name}"><span class="result-title">{display}</span><span class="result-slug">@{name}</span></div>"#,
            id = r.id,
            name = html_escape(&r.name),
            display = html_escape(&r.display_name),
        ));
    }
    Html(html)
}

pub async fn search_users(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SearchQuery>,
) -> Html<String> {
    let Ok(mut conn) = state.conn.get() else {
        return Html(String::new());
    };

    let pattern = query.q.clone().unwrap_or_default().trim().to_string();

    let rows: Vec<UserRow> = if pattern.is_empty() {
        diesel::sql_query(
            "SELECT id, username, email FROM users ORDER BY username LIMIT 20",
        )
        .load(&mut conn)
        .unwrap_or_default()
    } else {
        let p = format!("%{pattern}%");
        diesel::sql_query(
            "SELECT id, username, email FROM users WHERE username ILIKE $1 OR email ILIKE $1 ORDER BY username LIMIT 20",
        )
        .bind::<diesel::sql_types::Text, _>(p)
        .load(&mut conn)
        .unwrap_or_default()
    };

    if rows.is_empty() {
        return Html(r#"<p class="empty-text">No users found</p>"#.to_string());
    }

    let mut html = String::new();
    for r in rows {
        html.push_str(&format!(
            r#"<div class="search-result-item" data-id="{id}" data-username="{username}"><span class="result-title">{username}</span><span class="result-slug">{email}</span></div>"#,
            id = r.id,
            username = html_escape(&r.username),
            email = html_escape(&r.email),
        ));
    }
    Html(html)
}
