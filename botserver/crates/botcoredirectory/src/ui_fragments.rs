use crate::api::DirectoryApiState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use chrono::{DateTime, Utc};
use diesel::RunQueryDsl;
use serde::Deserialize;
use std::sync::Arc;

pub fn configure_directory_ui_routes() -> Router<Arc<DirectoryApiState>> {
    Router::new()
        .route("/api/directory/users/list", get(list_users_rows))
        .route("/api/directory/users/search", get(search_users_rows))
        .route("/api/directory/users/list/pagination", get(users_pagination))
        .route("/api/directory/users/:id/profile", get(user_profile))
        .route("/api/directory/users/:id/permissions", get(user_permissions))
        .route("/api/directory/users/:id/security", get(user_security))
        .route("/api/directory/users/:id/activity", get(user_activity))
        .route("/api/directory/users/:id/edit", get(user_edit_form))
        .route("/api/directory/users/:id/delete", axum::routing::delete(delete_user))
        .route("/api/directory/groups/list", get(list_groups_rows))
        .route("/api/directory/groups/search", get(search_groups_rows))
        .route("/api/directory/groups/list/pagination", get(groups_pagination))
        .route("/api/directory/groups/create", axum::routing::post(create_group))
        .route("/api/directory/groups/:id", get(group_detail))
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
    pub role: Option<String>,
    pub status: Option<String>,
    pub page: Option<u32>,
}

fn get_conn(
    state: &Arc<DirectoryApiState>,
) -> Option<diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>> {
    state.conn.get().ok()
}

#[derive(Debug, diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct CountResult {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    count: i64,
}

pub async fn list_users_rows(State(state): State<Arc<DirectoryApiState>>) -> Html<String> {
    let query = SearchQuery {
        q: None,
        role: None,
        status: None,
        page: None,
    };
    Html(load_users_rows(&state, &query))
}

pub async fn search_users_rows(
    State(state): State<Arc<DirectoryApiState>>,
    Query(query): Query<SearchQuery>,
) -> Html<String> {
    Html(load_users_rows(&state, &query))
}

fn load_users_rows(state: &Arc<DirectoryApiState>, query: &SearchQuery) -> String {
    let mut conn = match get_conn(state) {
        Some(c) => c,
        None => return empty_rows(),
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

    let mut sql = String::from(
        "SELECT id, username, email, is_active, is_admin, created_at FROM users WHERE 1=1",
    );
    let mut binds: Vec<String> = Vec::new();

    if let Some(q) = query.q.as_ref() {
        if !q.trim().is_empty() {
            let pattern = format!("%{}%", q.trim());
            sql.push_str(" AND (username ILIKE $1 OR email ILIKE $1)");
            binds.push(pattern);
        }
    }
    if let Some(role) = query.role.as_ref() {
        if role == "admin" {
            sql.push_str(" AND is_admin = true");
        } else if role == "user" {
            sql.push_str(" AND is_admin = false");
        }
    }
    if let Some(status) = query.status.as_ref() {
        if status == "active" {
            sql.push_str(" AND is_active = true");
        } else if status == "inactive" || status == "suspended" {
            sql.push_str(" AND is_active = false");
        }
    }
    sql.push_str(" ORDER BY created_at DESC LIMIT 50");

    let rows = if binds.is_empty() {
        diesel::sql_query(&sql).load::<UserRow>(&mut conn)
    } else {
        diesel::sql_query(&sql)
            .bind::<diesel::sql_types::Text, _>(binds[0].clone())
            .load::<UserRow>(&mut conn)
    };

    let rows: Vec<UserRow> = rows.unwrap_or_default();

    if rows.is_empty() {
        return empty_rows();
    }

    let mut html = String::new();
    for user in rows {
        let initials: String = user
            .username
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| !w.is_empty())
            .filter_map(|w| w.chars().next())
            .take(2)
            .collect::<String>()
            .to_uppercase();

        let role_class = if user.is_admin { "admin" } else { "user" };
        let role_label = if user.is_admin { "Admin" } else { "User" };
        let status_class = if user.is_active { "active" } else { "inactive" };
        let status_label = if user.is_active { "Active" } else { "Inactive" };

        html.push_str(&format!(
            r##"<tr>
    <td><input type="checkbox" value="{id}"></td>
    <td>
        <div class="user-cell">
            <div class="user-avatar">{initials}</div>
            <div class="user-info">
                <span class="user-name">{username}</span>
                <span class="user-username">@{username}</span>
            </div>
        </div>
    </td>
    <td>{email}</td>
    <td><span class="role-badge {role_class}">{role_label}</span></td>
    <td><span class="status-badge {status_class}"><span class="status-dot"></span>{status_label}</span></td>
    <td>{created}</td>
    <td>
        <div class="action-buttons">
            <button class="action-btn" onclick="openDetailPanel('{id}')">View</button>
            <button class="action-btn" onclick="editUser('{id}')">Edit</button>
            <button class="action-btn danger" onclick="confirmDeleteUser('{id}', '{username}')">Delete</button>
        </div>
    </td>
</tr>"##,
            id = user.id,
            username = html_escape(&user.username),
            email = html_escape(&user.email),
            created = user.created_at.format("%b %d, %Y"),
        ));
    }

    html
}

fn empty_rows() -> String {
    r##"<tr>
    <td colspan="7" class="loading-cell">No users found</td>
</tr>"##
    .to_string()
}

pub async fn users_pagination(State(state): State<Arc<DirectoryApiState>>) -> Html<String> {
    let mut conn = match get_conn(&state) {
        Some(c) => c,
        None => return Html(String::new()),
    };
    let total = diesel::sql_query("SELECT COUNT(*) as count FROM users")
        .get_result::<CountResult>(&mut conn)
        .map(|r| r.count)
        .unwrap_or(0);

    Html(pagination_html(total, 50))
}

pub async fn list_groups_rows(State(state): State<Arc<DirectoryApiState>>) -> Html<String> {
    Html(load_groups_rows(&state, None))
}

pub async fn search_groups_rows(
    State(state): State<Arc<DirectoryApiState>>,
    Query(query): Query<SearchQuery>,
) -> Html<String> {
    Html(load_groups_rows(&state, query.q.as_deref()))
}

fn load_groups_rows(state: &Arc<DirectoryApiState>, search: Option<&str>) -> String {
    let mut conn = match get_conn(state) {
        Some(c) => c,
        None => return empty_rows(),
    };

    #[derive(Debug, diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct GroupRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: uuid::Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        display_name: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        description: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Bool)]
        is_active: bool,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)]
        created_at: DateTime<Utc>,
    }

    let mut sql = String::from(
        "SELECT id, name, display_name, description, is_active, created_at FROM rbac_groups WHERE 1=1",
    );
    let mut binds: Vec<String> = Vec::new();
    if let Some(q) = search {
        if !q.trim().is_empty() {
            let pattern = format!("%{}%", q.trim());
            sql.push_str(" AND (name ILIKE $1 OR display_name ILIKE $1)");
            binds.push(pattern);
        }
    }
    sql.push_str(" ORDER BY name LIMIT 50");

    let rows = if binds.is_empty() {
        diesel::sql_query(&sql).load::<GroupRow>(&mut conn)
    } else {
        diesel::sql_query(&sql)
            .bind::<diesel::sql_types::Text, _>(binds[0].clone())
            .load::<GroupRow>(&mut conn)
    };

    let rows: Vec<GroupRow> = rows.unwrap_or_default();

    if rows.is_empty() {
        return empty_rows();
    }

    let mut html = String::new();
    for group in rows {
        let status_class = if group.is_active { "active" } else { "inactive" };
        let status_label = if group.is_active { "Active" } else { "Inactive" };
        let desc = group.description.unwrap_or_else(|| "—".to_string());
        html.push_str(&format!(
            r##"<tr>
    <td>
        <div class="group-cell">
            <div class="group-icon">{initial}</div>
            <div class="group-info">
                <span class="group-name">{display_name}</span>
                <span class="group-slug">@{name}</span>
            </div>
        </div>
    </td>
    <td>{desc}</td>
    <td><span class="status-badge {status_class}"><span class="status-dot"></span>{status_label}</span></td>
    <td>{created}</td>
    <td>
        <div class="action-buttons">
            <button class="action-btn" onclick="openGroupDetail('{id}')">View</button>
        </div>
    </td>
</tr>"##,
            initial = group.display_name.chars().next().unwrap_or('G'),
            display_name = html_escape(&group.display_name),
            name = html_escape(&group.name),
            desc = html_escape(&desc),
            created = group.created_at.format("%b %d, %Y"),
            id = group.id,
        ));
    }

    html
}

pub async fn groups_pagination(State(state): State<Arc<DirectoryApiState>>) -> Html<String> {
    let mut conn = match get_conn(&state) {
        Some(c) => c,
        None => return Html(String::new()),
    };
    let total = diesel::sql_query("SELECT COUNT(*) as count FROM rbac_groups")
        .get_result::<CountResult>(&mut conn)
        .map(|r| r.count)
        .unwrap_or(0);

    Html(pagination_html(total, 50))
}

fn pagination_html(total: i64, per_page: i64) -> String {
    if total <= per_page {
        return String::new();
    }
    let pages = (total + per_page - 1) / per_page;
    let mut html = String::from(
        r##"<button class="page-btn" disabled>‹</button>
<button class="page-btn active">1</button>"##,
    );
    for page in 2..=pages.min(10) {
        html.push_str(&format!(
            r##"<button class="page-btn">{page}</button>"##
        ));
    }
    html.push_str(r##"<button class="page-btn">›</button>"##);
    html
}

pub async fn create_group(
    State(state): State<Arc<DirectoryApiState>>,
    axum::extract::Form(form): axum::extract::Form<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let name = form.get("name").cloned().unwrap_or_default();
    let display_name = form.get("display_name").cloned().unwrap_or_else(|| name.clone());
    if name.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, Html("Group name is required".to_string()));
    }

    let mut conn = match get_conn(&state) {
        Some(c) => c,
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html("Database unavailable".to_string()),
            );
        }
    };

    let group_id = uuid::Uuid::new_v4();
    let result = diesel::sql_query(
        "INSERT INTO rbac_groups (id, name, display_name, description, is_active, created_by, created_at, updated_at)
         VALUES ($1, $2, $3, $4, true, NULL, NOW(), NOW())",
    )
    .bind::<diesel::sql_types::Uuid, _>(group_id)
    .bind::<diesel::sql_types::Text, _>(&name)
    .bind::<diesel::sql_types::Text, _>(&display_name)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(
        form.get("description").map(|d| d.clone()),
    )
    .execute(&mut conn);

    match result {
        Ok(_) => (StatusCode::OK, Html(load_groups_rows(&state, None))),
        Err(e) => {
            log::error!("Failed to create group: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Html("Failed to create group".to_string()))
        }
    }
}

pub async fn user_profile(
    State(state): State<Arc<DirectoryApiState>>,
    Path(id): Path<uuid::Uuid>,
) -> Html<String> {
    let mut conn = match get_conn(&state) {
        Some(c) => c,
        None => return Html(String::new()),
    };

    #[derive(Debug, diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct UserRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        username: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        email: String,
        #[diesel(sql_type = diesel::sql_types::Bool)]
        is_admin: bool,
        #[diesel(sql_type = diesel::sql_types::Bool)]
        is_active: bool,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)]
        created_at: DateTime<Utc>,
    }

    let user: Option<UserRow> = diesel::sql_query(
        "SELECT username, email, is_admin, is_active, created_at FROM users WHERE id = $1",
    )
    .bind::<diesel::sql_types::Uuid, _>(id)
    .get_result(&mut conn)
    .ok();

    match user {
        Some(u) => Html(format!(
            r##"<div class="detail-section">
    <div class="detail-label">Username</div>
    <div class="detail-value">{username}</div>
</div>
<div class="detail-section">
    <div class="detail-label">Email</div>
    <div class="detail-value">{email}</div>
</div>
<div class="detail-section">
    <div class="detail-label">Role</div>
    <div class="detail-value">{role}</div>
</div>
<div class="detail-section">
    <div class="detail-label">Status</div>
    <div class="detail-value">{status}</div>
</div>
<div class="detail-section">
    <div class="detail-label">Member Since</div>
    <div class="detail-value">{created}</div>
</div>"##,
            username = html_escape(&u.username),
            email = html_escape(&u.email),
            role = if u.is_admin { "Admin" } else { "User" },
            status = if u.is_active { "Active" } else { "Inactive" },
            created = u.created_at.format("%b %d, %Y"),
        )),
        None => Html(r#"<p class="text-muted">User not found</p>"#.to_string()),
    }
}

pub async fn user_permissions(Path(_id): Path<uuid::Uuid>) -> Html<String> {
    Html(
        r##"<div class="permission-list">
    <div class="permission-item"><span>Admin access</span><span class="permission-value">Not granted</span></div>
    <div class="permission-item"><span>Bot management</span><span class="permission-value">Not granted</span></div>
    <div class="permission-item"><span>Group management</span><span class="permission-value">Not granted</span></div>
</div>"##
        .to_string(),
    )
}

pub async fn user_security(Path(_id): Path<uuid::Uuid>) -> Html<String> {
    Html(
        r##"<div class="security-list">
    <div class="security-item"><span>Two-factor authentication</span><span class="security-value">Disabled</span></div>
    <div class="security-item"><span>Password last changed</span><span class="security-value">—</span></div>
    <div class="security-item"><span>Active sessions</span><span class="security-value">—</span></div>
</div>"##
        .to_string(),
    )
}

pub async fn user_activity(
    State(state): State<Arc<DirectoryApiState>>,
    Path(id): Path<uuid::Uuid>,
) -> Html<String> {
    let mut conn = match get_conn(&state) {
        Some(c) => c,
        None => return Html(String::new()),
    };

    #[derive(Debug, diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct SessionRow {
        #[diesel(sql_type = diesel::sql_types::Timestamptz)]
        created_at: DateTime<Utc>,
    }

    let sessions: Vec<SessionRow> = diesel::sql_query(
        "SELECT created_at FROM user_sessions WHERE user_id = $1::text ORDER BY created_at DESC LIMIT 10",
    )
    .bind::<diesel::sql_types::Uuid, _>(id)
    .load::<SessionRow>(&mut conn)
    .unwrap_or_default();

    if sessions.is_empty() {
        return Html(r#"<p class="text-muted">No recent activity</p>"#.to_string());
    }

    let mut html = String::new();
    for s in sessions {
        html.push_str(&format!(
            r##"<div class="activity-entry"><span class="activity-dot"></span><span>Session started</span><span class="activity-time">{}</span></div>"##,
            s.created_at.format("%b %d, %H:%M")
        ));
    }
    Html(html)
}

pub async fn user_edit_form(
    State(state): State<Arc<DirectoryApiState>>,
    Path(id): Path<uuid::Uuid>,
) -> Html<String> {
    let mut conn = match get_conn(&state) {
        Some(c) => c,
        None => return Html(String::new()),
    };

    #[derive(Debug, diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct UserRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        username: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        email: String,
        #[diesel(sql_type = diesel::sql_types::Bool)]
        is_admin: bool,
        #[diesel(sql_type = diesel::sql_types::Bool)]
        is_active: bool,
    }

    let user: Option<UserRow> = diesel::sql_query(
        "SELECT username, email, is_admin, is_active FROM users WHERE id = $1",
    )
    .bind::<diesel::sql_types::Uuid, _>(id)
    .get_result(&mut conn)
    .ok();

    match user {
        Some(u) => Html(format!(
            r##"<form hx-post="/api/directory/users/{id}" hx-target="#users-table-body" hx-swap="innerHTML">
    <div class="modal-header">
        <h2>Edit User</h2>
        <button type="button" class="close-btn" onclick="document.getElementById('edit-user-modal').close()">×</button>
    </div>
    <div class="form-group">
        <label>Username</label>
        <input type="text" name="username" value="{username}">
    </div>
    <div class="form-group">
        <label>Email</label>
        <input type="email" name="email" value="{email}">
    </div>
    <div class="form-row">
        <div class="form-group">
            <label>Role</label>
            <select name="role">
                <option value="user" {user_sel}>User</option>
                <option value="admin" {admin_sel}>Admin</option>
            </select>
        </div>
        <div class="form-group">
            <label>Status</label>
            <select name="status">
                <option value="active" {active_sel}>Active</option>
                <option value="inactive" {inactive_sel}>Inactive</option>
            </select>
        </div>
    </div>
    <div class="modal-footer">
        <button type="button" class="btn-secondary" onclick="document.getElementById('edit-user-modal').close()">Cancel</button>
        <button type="submit" class="btn-primary">Save Changes</button>
    </div>
</form>"##,
            id = id,
            username = html_escape(&u.username),
            email = html_escape(&u.email),
            user_sel = if u.is_admin { "" } else { "selected" },
            admin_sel = if u.is_admin { "selected" } else { "" },
            active_sel = if u.is_active { "selected" } else { "" },
            inactive_sel = if u.is_active { "" } else { "selected" },
        )),
        None => Html(r#"<p>User not found</p>"#.to_string()),
    }
}

pub async fn delete_user(
    State(state): State<Arc<DirectoryApiState>>,
    Path(id): Path<uuid::Uuid>,
) -> impl IntoResponse {
    let mut conn = match get_conn(&state) {
        Some(c) => c,
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html("Database unavailable".to_string()),
            );
        }
    };

    let result = diesel::sql_query("DELETE FROM users WHERE id = $1")
        .bind::<diesel::sql_types::Uuid, _>(id)
        .execute(&mut conn);

    match result {
        Ok(_) => (
            StatusCode::OK,
            Html(load_users_rows(&state, &SearchQuery { q: None, role: None, status: None, page: None })),
        ),
        Err(e) => {
            log::error!("Failed to delete user {}: {}", id, e);
            (StatusCode::INTERNAL_SERVER_ERROR, Html("Failed to delete user".to_string()))
        }
    }
}

pub async fn group_detail(
    State(state): State<Arc<DirectoryApiState>>,
    Path(id): Path<uuid::Uuid>,
) -> Html<String> {
    let mut conn = match get_conn(&state) {
        Some(c) => c,
        None => return Html(String::new()),
    };

    #[derive(Debug, diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct GroupRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        display_name: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        description: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Bool)]
        is_active: bool,
    }

    let group: Option<GroupRow> = diesel::sql_query(
        "SELECT name, display_name, description, is_active FROM rbac_groups WHERE id = $1",
    )
    .bind::<diesel::sql_types::Uuid, _>(id)
    .get_result(&mut conn)
    .ok();

    match group {
        Some(g) => Html(format!(
            r##"<div class="detail-section">
    <div class="detail-label">Group Name</div>
    <div class="detail-value">{display_name}</div>
</div>
<div class="detail-section">
    <div class="detail-label">Slug</div>
    <div class="detail-value">@{name}</div>
</div>
<div class="detail-section">
    <div class="detail-label">Description</div>
    <div class="detail-value">{desc}</div>
</div>
<div class="detail-section">
    <div class="detail-label">Status</div>
    <div class="detail-value">{status}</div>
</div>"##,
            display_name = html_escape(&g.display_name),
            name = html_escape(&g.name),
            desc = html_escape(g.description.as_deref().unwrap_or("—")),
            status = if g.is_active { "Active" } else { "Inactive" },
        )),
        None => Html(r#"<p class="text-muted">Group not found</p>"#.to_string()),
    }
}

pub async fn create_user(
    State(state): State<Arc<DirectoryApiState>>,
    axum::extract::Form(form): axum::extract::Form<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let username = form.get("username").cloned().unwrap_or_default();
    let email = form.get("email").cloned().unwrap_or_default();
    let password = form.get("password").cloned().unwrap_or_default();
    let role = form.get("role").cloned().unwrap_or_else(|| "user".to_string());
    let status = form.get("status").cloned().unwrap_or_else(|| "active".to_string());

    if username.trim().is_empty() || email.trim().is_empty() || password.len() < 8 {
        return (
            StatusCode::BAD_REQUEST,
            Html("Username, email and a password of at least 8 characters are required".to_string()),
        );
    }

    let mut conn = match get_conn(&state) {
        Some(c) => c,
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html("Database unavailable".to_string()),
            );
        }
    };

    let password_hash = match hash_password(&password) {
        Ok(h) => h,
        Err(e) => {
            log::error!("Password hashing failed: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html("Failed to hash password".to_string()),
            );
        }
    };

    let user_id = uuid::Uuid::new_v4();
    let is_admin = role == "admin";
    let is_active = status != "inactive";

    let result = diesel::sql_query(
        "INSERT INTO users (id, username, email, password_hash, is_active, is_admin, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())",
    )
    .bind::<diesel::sql_types::Uuid, _>(user_id)
    .bind::<diesel::sql_types::Text, _>(&username)
    .bind::<diesel::sql_types::Text, _>(&email)
    .bind::<diesel::sql_types::Text, _>(&password_hash)
    .bind::<diesel::sql_types::Bool, _>(is_active)
    .bind::<diesel::sql_types::Bool, _>(is_admin)
    .execute(&mut conn);

    match result {
        Ok(_) => {
            log::info!("User {username} created via admin UI");
            (
                StatusCode::OK,
                Html(load_users_rows(
                    &state,
                    &SearchQuery { q: None, role: None, status: None, page: None },
                )),
            )
        }
        Err(e) => {
            log::error!("Failed to create user: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Html("Failed to create user".to_string()))
        }
    }
}

pub async fn update_user(
    State(state): State<Arc<DirectoryApiState>>,
    Path(id): Path<uuid::Uuid>,
    axum::extract::Form(form): axum::extract::Form<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let username = form.get("username").cloned().unwrap_or_default();
    let email = form.get("email").cloned().unwrap_or_default();
    let role = form.get("role").cloned().unwrap_or_else(|| "user".to_string());
    let status = form.get("status").cloned().unwrap_or_else(|| "active".to_string());

    let mut conn = match get_conn(&state) {
        Some(c) => c,
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html("Database unavailable".to_string()),
            );
        }
    };

    let is_admin = role == "admin";
    let is_active = status != "inactive";

    let result = diesel::sql_query(
        "UPDATE users SET username = $1, email = $2, is_admin = $3, is_active = $4, updated_at = NOW() WHERE id = $5",
    )
    .bind::<diesel::sql_types::Text, _>(&username)
    .bind::<diesel::sql_types::Text, _>(&email)
    .bind::<diesel::sql_types::Bool, _>(is_admin)
    .bind::<diesel::sql_types::Bool, _>(is_active)
    .bind::<diesel::sql_types::Uuid, _>(id)
    .execute(&mut conn);

    match result {
        Ok(_) => (
            StatusCode::OK,
            Html(load_users_rows(
                &state,
                &SearchQuery { q: None, role: None, status: None, page: None },
            )),
        ),
        Err(e) => {
            log::error!("Failed to update user {}: {}", id, e);
            (StatusCode::INTERNAL_SERVER_ERROR, Html("Failed to update user".to_string()))
        }
    }
}

fn hash_password(password: &str) -> anyhow::Result<String> {
    use argon2::password_hash::{rand_core::OsRng, SaltString};
    use argon2::{Argon2, PasswordHasher};

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("Password hashing failed: {}", e))?
        .to_string();
    Ok(hash)
}

fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
