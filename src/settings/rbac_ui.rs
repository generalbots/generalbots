use crate::security::error_sanitizer::SafeErrorResponse;
use crate::shared::models::{RbacGroup, RbacRole, User};
use crate::shared::state::AppState;
use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse},
};
use diesel::prelude::*;
use std::sync::Arc;
use uuid::Uuid;

pub async fn rbac_settings_page() -> Html<String> {
    Html(String::from(r##"<div class="rbac-settings">
    <h2>Access Control</h2>
    <p class="text-muted">Manage users, roles, and groups using Role-Based Access Control (RBAC)</p>

    <div class="rbac-tabs">
        <button class="tab-btn active" hx-get="/settings/rbac/users" hx-target="#rbac-content" hx-swap="innerHTML">
            <span class="tab-icon">👥</span> Users
        </button>
        <button class="tab-btn" hx-get="/settings/rbac/roles" hx-target="#rbac-content" hx-swap="innerHTML">
            <span class="tab-icon">🔐</span> Roles
        </button>
        <button class="tab-btn" hx-get="/settings/rbac/groups" hx-target="#rbac-content" hx-swap="innerHTML">
            <span class="tab-icon">📁</span> Groups
        </button>
    </div>

    <div id="rbac-content" hx-get="/settings/rbac/users" hx-trigger="load" hx-swap="innerHTML">
        <div class="loading">Loading...</div>
    </div>
</div>

<style>
.rbac-settings { padding: 1rem; }
.rbac-tabs { display: flex; gap: 0.5rem; margin-bottom: 1rem; border-bottom: 1px solid #e0e0e0; padding-bottom: 0.5rem; }
.tab-btn { background: none; border: none; padding: 0.75rem 1rem; cursor: pointer; border-radius: 4px 4px 0 0; transition: all 0.2s; }
.tab-btn:hover { background: #f5f5f5; }
.tab-btn.active { background: #e3f2fd; color: #1976d2; font-weight: 500; }
.tab-icon { margin-right: 0.5rem; }

.user-list, .role-list, .group-list { display: flex; flex-direction: column; gap: 0.5rem; }
.user-item, .role-item, .group-item { display: flex; align-items: center; justify-content: space-between; padding: 0.75rem 1rem; background: #f9f9f9; border-radius: 8px; cursor: pointer; transition: all 0.2s; }
.user-item:hover, .role-item:hover, .group-item:hover { background: #e3f2fd; }
.user-item.selected { background: #bbdefb; border-left: 3px solid #1976d2; }

.assignment-panel { display: grid; grid-template-columns: 1fr auto 1fr; gap: 1rem; margin-top: 1rem; padding: 1rem; background: #fafafa; border-radius: 8px; }
.list-box { background: white; border: 1px solid #e0e0e0; border-radius: 8px; min-height: 300px; max-height: 400px; overflow-y: auto; }
.list-box-header { padding: 0.75rem 1rem; background: #f5f5f5; border-bottom: 1px solid #e0e0e0; font-weight: 500; }
.list-box-content { padding: 0.5rem; }
.list-item { padding: 0.5rem 0.75rem; border-radius: 4px; cursor: pointer; display: flex; align-items: center; justify-content: space-between; }
.list-item:hover { background: #e3f2fd; }
.list-item.selected { background: #bbdefb; }

.transfer-buttons { display: flex; flex-direction: column; justify-content: center; gap: 0.5rem; }
.transfer-btn { padding: 0.5rem 1rem; border: 1px solid #1976d2; background: white; color: #1976d2; border-radius: 4px; cursor: pointer; font-size: 1.2rem; }
.transfer-btn:hover { background: #1976d2; color: white; }
.transfer-btn:disabled { opacity: 0.5; cursor: not-allowed; }

.search-box { padding: 0.5rem; border-bottom: 1px solid #e0e0e0; }
.search-box input { width: 100%; padding: 0.5rem; border: 1px solid #ddd; border-radius: 4px; }

.badge { padding: 0.25rem 0.5rem; border-radius: 12px; font-size: 0.75rem; }
.badge-system { background: #e8f5e9; color: #2e7d32; }
.badge-custom { background: #fff3e0; color: #ef6c00; }
.badge-role { background: #e3f2fd; color: #1565c0; }
.badge-group { background: #f3e5f5; color: #7b1fa2; }

.empty-state { text-align: center; padding: 2rem; color: #666; }
.loading { text-align: center; padding: 2rem; color: #666; }

.btn-primary { background: #1976d2; color: white; border: none; padding: 0.5rem 1rem; border-radius: 4px; cursor: pointer; }
.btn-primary:hover { background: #1565c0; }
.btn-secondary { background: white; color: #1976d2; border: 1px solid #1976d2; padding: 0.5rem 1rem; border-radius: 4px; cursor: pointer; }
.btn-secondary:hover { background: #e3f2fd; }
.btn-danger { background: #d32f2f; color: white; border: none; padding: 0.25rem 0.5rem; border-radius: 4px; cursor: pointer; font-size: 0.8rem; }
.btn-danger:hover { background: #c62828; }

.create-form { display: flex; gap: 0.5rem; margin-bottom: 1rem; padding: 1rem; background: #f5f5f5; border-radius: 8px; }
.create-form input { flex: 1; padding: 0.5rem; border: 1px solid #ddd; border-radius: 4px; }
</style>"##))
}

fn render_error(context: &str) -> String {
    tracing::warn!(context = %context, "Error occurred in RBAC UI");
    let response = SafeErrorResponse::internal_error();
    format!(r##"<div class="error">{}</div>"##, response.message)
}

pub async fn rbac_users_list(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let conn = state.conn.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().map_err(|e| format!("DB error: {e}"))?;
        use crate::shared::models::schema::users;
        users::table
            .filter(users::is_active.eq(true))
            .order(users::username.asc())
            .load::<User>(&mut db_conn)
            .map_err(|e| format!("Query error: {e}"))
    })
    .await;

    match result {
        Ok(Ok(users)) => {
            let mut html = String::from(r##"<div class="users-section">
                <div class="section-header">
                    <h3>Users</h3>
                    <p class="text-muted">Select a user to manage their roles and groups</p>
                </div>
                <div class="search-box">
                    <input type="text" placeholder="Search users..."
                           hx-get="/settings/rbac/users"
                           hx-trigger="keyup changed delay:300ms"
                           hx-target=".user-list"
                           hx-include="this"
                           name="search" />
                </div>
                <div class="user-list">"##);

            for user in &users {
                let admin_badge = if user.is_admin {
                    r##"<span class="badge badge-system">Admin</span>"##
                } else {
                    ""
                };
                html.push_str(&format!(
                    r##"<div class="user-item" hx-get="/settings/rbac/users/{user_id}/assignment" hx-target="#assignment-panel" hx-swap="innerHTML">
                        <div class="user-info">
                            <span class="user-name">{username}</span>
                            <span class="user-email text-muted">{email}</span>
                        </div>
                        <div class="user-badges">{badges}</div>
                    </div>"##,
                    user_id = user.id,
                    username = user.username,
                    email = user.email,
                    badges = admin_badge
                ));
            }

            if users.is_empty() {
                html.push_str(r##"<div class="empty-state">No users found</div>"##);
            }

            html.push_str(r##"</div><div id="assignment-panel"></div></div>"##);
            Html(html).into_response()
        }
        Ok(Err(e)) => {
            tracing::warn!(error = %e, "rbac_users_list query failed");
            Html(render_error("rbac_users_list")).into_response()
        }
        Err(e) => {
            tracing::warn!(error = %e, "rbac_users_list task failed");
            Html(render_error("rbac_users_list")).into_response()
        }
    }
}

pub async fn rbac_roles_list(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let conn = state.conn.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().map_err(|e| format!("DB error: {e}"))?;
        use crate::shared::models::schema::rbac_roles;
        rbac_roles::table
            .filter(rbac_roles::is_active.eq(true))
            .order(rbac_roles::display_name.asc())
            .load::<RbacRole>(&mut db_conn)
            .map_err(|e| format!("Query error: {e}"))
    })
    .await;

    match result {
        Ok(Ok(roles)) => {
            let mut html = String::from(r##"<div class="roles-section">
                <div class="section-header">
                    <h3>Roles</h3>
                    <p class="text-muted">Manage roles and their permissions</p>
                </div>
                <form class="create-form" hx-post="/api/rbac/roles" hx-target=".role-list" hx-swap="beforeend" hx-on::after-request="this.reset()">
                    <input type="text" name="name" placeholder="Role name (e.g., editor)" required />
                    <input type="text" name="display_name" placeholder="Display name (e.g., Content Editor)" required />
                    <input type="text" name="description" placeholder="Description (optional)" />
                    <button type="submit" class="btn-primary">+ Add Role</button>
                </form>
                <div class="role-list">"##);

            for role in &roles {
                let badge_class = if role.is_system { "badge-system" } else { "badge-custom" };
                let badge_text = if role.is_system { "System" } else { "Custom" };
                let delete_btn = if role.is_system {
                    String::new()
                } else {
                    format!(
                        r##"<button class="btn-danger" hx-delete="/api/rbac/roles/{role_id}" hx-target="closest .role-item" hx-swap="outerHTML" hx-confirm="Delete this role?">Delete</button>"##,
                        role_id = role.id
                    )
                };
                let description = role.description.as_deref().unwrap_or("");
                html.push_str(&format!(
                    r##"<div class="role-item">
                        <div class="role-info">
                            <span class="role-name">{display_name}</span>
                            <span class="role-desc text-muted">{description}</span>
                        </div>
                        <div class="role-actions">
                            <span class="badge {badge_class}">{badge_text}</span>
                            {delete_btn}
                        </div>
                    </div>"##,
                    display_name = role.display_name,
                    description = description,
                    badge_class = badge_class,
                    badge_text = badge_text,
                    delete_btn = delete_btn
                ));
            }

            if roles.is_empty() {
                html.push_str(r##"<div class="empty-state">No roles defined. Create your first role above.</div>"##);
            }

            html.push_str(r##"</div></div>"##);
            Html(html).into_response()
        }
        Ok(Err(e)) => {
            tracing::warn!(error = %e, "rbac_roles_list query failed");
            Html(render_error("rbac_roles_list")).into_response()
        }
        Err(e) => {
            tracing::warn!(error = %e, "rbac_roles_list task failed");
            Html(render_error("rbac_roles_list")).into_response()
        }
    }
}

pub async fn rbac_groups_list(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let conn = state.conn.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().map_err(|e| format!("DB error: {e}"))?;
        use crate::shared::models::schema::rbac_groups;
        rbac_groups::table
            .filter(rbac_groups::is_active.eq(true))
            .order(rbac_groups::display_name.asc())
            .load::<RbacGroup>(&mut db_conn)
            .map_err(|e| format!("Query error: {e}"))
    })
    .await;

    match result {
        Ok(Ok(groups)) => {
            let mut html = String::from(r##"<div class="groups-section">
                <div class="section-header">
                    <h3>Groups</h3>
                    <p class="text-muted">Organize users into groups for easier role assignment</p>
                </div>
                <form class="create-form" hx-post="/api/rbac/groups" hx-target=".group-list" hx-swap="beforeend" hx-on::after-request="this.reset()">
                    <input type="text" name="name" placeholder="Group name (e.g., sales_team)" required />
                    <input type="text" name="display_name" placeholder="Display name (e.g., Sales Team)" required />
                    <input type="text" name="description" placeholder="Description (optional)" />
                    <button type="submit" class="btn-primary">+ Add Group</button>
                </form>
                <div class="group-list">"##);

            for group in &groups {
                let description = group.description.as_deref().unwrap_or("");
                html.push_str(&format!(
                    r##"<div class="group-item">
                        <div class="group-info">
                            <span class="group-name">{display_name}</span>
                            <span class="group-desc text-muted">{description}</span>
                        </div>
                        <div class="group-actions">
                            <button class="btn-secondary" hx-get="/settings/rbac/groups/{group_id}/roles" hx-target="#group-roles-panel" hx-swap="innerHTML">Manage Roles</button>
                            <button class="btn-danger" hx-delete="/api/rbac/groups/{group_id}" hx-target="closest .group-item" hx-swap="outerHTML" hx-confirm="Delete this group?">Delete</button>
                        </div>
                    </div>"##,
                    display_name = group.display_name,
                    description = description,
                    group_id = group.id
                ));
            }

            if groups.is_empty() {
                html.push_str(r##"<div class="empty-state">No groups defined. Create your first group above.</div>"##);
            }

            html.push_str(r##"</div><div id="group-roles-panel"></div></div>"##);
            Html(html).into_response()
        }
        Ok(Err(e)) => {
            tracing::warn!(error = %e, "rbac_groups_list query failed");
            Html(render_error("rbac_groups_list")).into_response()
        }
        Err(e) => {
            tracing::warn!(error = %e, "rbac_groups_list task failed");
            Html(render_error("rbac_groups_list")).into_response()
        }
    }
}

pub async fn user_assignment_panel(State(state): State<Arc<AppState>>, Path(user_id): Path<Uuid>) -> impl IntoResponse {
    let conn = state.conn.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().map_err(|e| format!("DB error: {e}"))?;
        use crate::shared::models::schema::users;
        users::table
            .find(user_id)
            .first::<User>(&mut db_conn)
            .map_err(|e| format!("User not found: {e}"))
    })
    .await;

    match result {
        Ok(Ok(user)) => {
            let html = format!(
                r##"<div class="assignment-container">
                <h4>Manage Access for: {username}</h4>
                <p class="text-muted">{email}</p>

                <div class="assignment-tabs">
                    <button class="tab-btn active" onclick="showTab('roles-tab', this)">Roles</button>
                    <button class="tab-btn" onclick="showTab('groups-tab', this)">Groups</button>
                </div>

                <div id="roles-tab" class="tab-content active">
                    <div class="assignment-panel">
                        <div class="list-box">
                            <div class="list-box-header">Available Roles</div>
                            <div class="list-box-content" id="available-roles"
                                 hx-get="/settings/rbac/users/{uid}/available-roles"
                                 hx-trigger="load, roleAssigned from:body, roleRemoved from:body">
                                <div class="loading">Loading...</div>
                            </div>
                        </div>

                        <div class="transfer-buttons">
                            <button class="transfer-btn" id="assign-role-btn"
                                    hx-post="/api/rbac/users/{uid}/roles/__ROLE_ID__"
                                    hx-trigger="click"
                                    hx-swap="none"
                                    hx-on::after-request="htmx.trigger(document.body, 'roleAssigned')"
                                    disabled>→</button>
                            <button class="transfer-btn" id="remove-role-btn"
                                    hx-delete="/api/rbac/users/{uid}/roles/__ROLE_ID__"
                                    hx-trigger="click"
                                    hx-swap="none"
                                    hx-on::after-request="htmx.trigger(document.body, 'roleRemoved')"
                                    disabled>←</button>
                        </div>

                        <div class="list-box">
                            <div class="list-box-header">Assigned Roles</div>
                            <div class="list-box-content" id="assigned-roles"
                                 hx-get="/settings/rbac/users/{uid}/assigned-roles"
                                 hx-trigger="load, roleAssigned from:body, roleRemoved from:body">
                                <div class="loading">Loading...</div>
                            </div>
                        </div>
                    </div>
                </div>

                <div id="groups-tab" class="tab-content" style="display:none;">
                    <div class="assignment-panel">
                        <div class="list-box">
                            <div class="list-box-header">Available Groups</div>
                            <div class="list-box-content" id="available-groups"
                                 hx-get="/settings/rbac/users/{uid}/available-groups"
                                 hx-trigger="load, groupAdded from:body, groupRemoved from:body">
                                <div class="loading">Loading...</div>
                            </div>
                        </div>

                        <div class="transfer-buttons">
                            <button class="transfer-btn" id="add-group-btn"
                                    hx-post="/api/rbac/users/{uid}/groups/__GROUP_ID__"
                                    hx-trigger="click"
                                    hx-swap="none"
                                    hx-on::after-request="htmx.trigger(document.body, 'groupAdded')"
                                    disabled>→</button>
                            <button class="transfer-btn" id="remove-group-btn"
                                    hx-delete="/api/rbac/users/{uid}/groups/__GROUP_ID__"
                                    hx-trigger="click"
                                    hx-swap="none"
                                    hx-on::after-request="htmx.trigger(document.body, 'groupRemoved')"
                                    disabled>←</button>
                        </div>

                        <div class="list-box">
                            <div class="list-box-header">Member Of</div>
                            <div class="list-box-content" id="assigned-groups"
                                 hx-get="/settings/rbac/users/{uid}/assigned-groups"
                                 hx-trigger="load, groupAdded from:body, groupRemoved from:body">
                                <div class="loading">Loading...</div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            <script>
            function showTab(tabId, btn) {{
                document.querySelectorAll('.tab-content').forEach(t => t.style.display = 'none');
                document.querySelectorAll('.assignment-tabs .tab-btn').forEach(b => b.classList.remove('active'));
                document.getElementById(tabId).style.display = 'block';
                btn.classList.add('active');
            }}

            function selectAvailableRole(el, roleId) {{
                document.querySelectorAll('#available-roles .list-item').forEach(i => i.classList.remove('selected'));
                el.classList.add('selected');
                const btn = document.getElementById('assign-role-btn');
                btn.disabled = false;
                btn.setAttribute('hx-post', '/api/rbac/users/{uid}/roles/' + roleId);
                htmx.process(btn);
            }}

            function selectAssignedRole(el, roleId) {{
                document.querySelectorAll('#assigned-roles .list-item').forEach(i => i.classList.remove('selected'));
                el.classList.add('selected');
                const btn = document.getElementById('remove-role-btn');
                btn.disabled = false;
                btn.setAttribute('hx-delete', '/api/rbac/users/{uid}/roles/' + roleId);
                htmx.process(btn);
            }}

            function selectAvailableGroup(el, groupId) {{
                document.querySelectorAll('#available-groups .list-item').forEach(i => i.classList.remove('selected'));
                el.classList.add('selected');
                const btn = document.getElementById('add-group-btn');
                btn.disabled = false;
                btn.setAttribute('hx-post', '/api/rbac/users/{uid}/groups/' + groupId);
                htmx.process(btn);
            }}

            function selectAssignedGroup(el, groupId) {{
                document.querySelectorAll('#assigned-groups .list-item').forEach(i => i.classList.remove('selected'));
                el.classList.add('selected');
                const btn = document.getElementById('remove-group-btn');
                btn.disabled = false;
                btn.setAttribute('hx-delete', '/api/rbac/users/{uid}/groups/' + groupId);
                htmx.process(btn);
            }}
            </script>"##,
                username = user.username,
                email = user.email,
                uid = user_id
            );
            Html(html).into_response()
        }
        Ok(Err(e)) => {
            tracing::warn!(error = %e, "user_assignment_panel query failed");
            Html(render_error("user_assignment_panel")).into_response()
        }
        Err(e) => {
            tracing::warn!(error = %e, "user_assignment_panel task failed");
            Html(render_error("user_assignment_panel")).into_response()
        }
    }
}

pub async fn available_roles_for_user(State(state): State<Arc<AppState>>, Path(user_id): Path<Uuid>) -> impl IntoResponse {
    let conn = state.conn.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().map_err(|e| format!("DB error: {e}"))?;
        use crate::shared::models::schema::{rbac_roles, rbac_user_roles};

        let assigned_role_ids: Vec<Uuid> = rbac_user_roles::table
            .filter(rbac_user_roles::user_id.eq(user_id))
            .select(rbac_user_roles::role_id)
            .load(&mut db_conn)
            .map_err(|e| format!("Query error: {e}"))?;

        rbac_roles::table
            .filter(rbac_roles::is_active.eq(true))
            .filter(rbac_roles::id.ne_all(assigned_role_ids))
            .order(rbac_roles::display_name.asc())
            .load::<RbacRole>(&mut db_conn)
            .map_err(|e| format!("Query error: {e}"))
    })
    .await;

    match result {
        Ok(Ok(roles)) => {
            let mut html = String::new();
            for role in &roles {
                html.push_str(&format!(
                    r##"<div class="list-item" onclick="selectAvailableRole(this, '{id}')">{name}</div>"##,
                    id = role.id,
                    name = role.display_name
                ));
            }
            if roles.is_empty() {
                html.push_str(r##"<div class="empty-state">All roles assigned</div>"##);
            }
            Html(html).into_response()
        }
        Ok(Err(e)) => {
            tracing::warn!(error = %e, "available_roles_for_user query failed");
            Html(render_error("available_roles_for_user")).into_response()
        }
        Err(e) => {
            tracing::warn!(error = %e, "available_roles_for_user task failed");
            Html(render_error("available_roles_for_user")).into_response()
        }
    }
}

pub async fn assigned_roles_for_user(State(state): State<Arc<AppState>>, Path(user_id): Path<Uuid>) -> impl IntoResponse {
    let conn = state.conn.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().map_err(|e| format!("DB error: {e}"))?;
        use crate::shared::models::schema::{rbac_roles, rbac_user_roles};

        rbac_user_roles::table
            .inner_join(rbac_roles::table)
            .filter(rbac_user_roles::user_id.eq(user_id))
            .filter(rbac_roles::is_active.eq(true))
            .select(RbacRole::as_select())
            .load::<RbacRole>(&mut db_conn)
            .map_err(|e| format!("Query error: {e}"))
    })
    .await;

    match result {
        Ok(Ok(roles)) => {
            let mut html = String::new();
            for role in &roles {
                html.push_str(&format!(
                    r##"<div class="list-item" onclick="selectAssignedRole(this, '{id}')">{name}</div>"##,
                    id = role.id,
                    name = role.display_name
                ));
            }
            if roles.is_empty() {
                html.push_str(r##"<div class="empty-state">No roles assigned</div>"##);
            }
            Html(html).into_response()
        }
        Ok(Err(e)) => {
            tracing::warn!(error = %e, "assigned_roles_for_user query failed");
            Html(render_error("assigned_roles_for_user")).into_response()
        }
        Err(e) => {
            tracing::warn!(error = %e, "assigned_roles_for_user task failed");
            Html(render_error("assigned_roles_for_user")).into_response()
        }
    }
}

pub async fn available_groups_for_user(State(state): State<Arc<AppState>>, Path(user_id): Path<Uuid>) -> impl IntoResponse {
    let conn = state.conn.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().map_err(|e| format!("DB error: {e}"))?;
        use crate::shared::models::schema::{rbac_groups, rbac_user_groups};

        let assigned_group_ids: Vec<Uuid> = rbac_user_groups::table
            .filter(rbac_user_groups::user_id.eq(user_id))
            .select(rbac_user_groups::group_id)
            .load(&mut db_conn)
            .map_err(|e| format!("Query error: {e}"))?;

        rbac_groups::table
            .filter(rbac_groups::is_active.eq(true))
            .filter(rbac_groups::id.ne_all(assigned_group_ids))
            .order(rbac_groups::display_name.asc())
            .load::<RbacGroup>(&mut db_conn)
            .map_err(|e| format!("Query error: {e}"))
    })
    .await;

    match result {
        Ok(Ok(groups)) => {
            let mut html = String::new();
            for group in &groups {
                html.push_str(&format!(
                    r##"<div class="list-item" onclick="selectAvailableGroup(this, '{id}')">{name}</div>"##,
                    id = group.id,
                    name = group.display_name
                ));
            }
            if groups.is_empty() {
                html.push_str(r##"<div class="empty-state">All groups joined</div>"##);
            }
            Html(html).into_response()
        }
        Ok(Err(e)) => {
            tracing::warn!(error = %e, "available_groups_for_user query failed");
            Html(render_error("available_groups_for_user")).into_response()
        }
        Err(e) => {
            tracing::warn!(error = %e, "available_groups_for_user task failed");
            Html(render_error("available_groups_for_user")).into_response()
        }
    }
}

pub async fn assigned_groups_for_user(State(state): State<Arc<AppState>>, Path(user_id): Path<Uuid>) -> impl IntoResponse {
    let conn = state.conn.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().map_err(|e| format!("DB error: {e}"))?;
        use crate::shared::models::schema::{rbac_groups, rbac_user_groups};

        rbac_user_groups::table
            .inner_join(rbac_groups::table)
            .filter(rbac_user_groups::user_id.eq(user_id))
            .filter(rbac_groups::is_active.eq(true))
            .select(RbacGroup::as_select())
            .load::<RbacGroup>(&mut db_conn)
            .map_err(|e| format!("Query error: {e}"))
    })
    .await;

    match result {
        Ok(Ok(groups)) => {
            let mut html = String::new();
            for group in &groups {
                html.push_str(&format!(
                    r##"<div class="list-item" onclick="selectAssignedGroup(this, '{id}')">{name}</div>"##,
                    id = group.id,
                    name = group.display_name
                ));
            }
            if groups.is_empty() {
                html.push_str(r##"<div class="empty-state">Not a member of any group</div>"##);
            }
            Html(html).into_response()
        }
        Ok(Err(e)) => {
            tracing::warn!(error = %e, "assigned_groups_for_user query failed");
            Html(render_error("assigned_groups_for_user")).into_response()
        }
        Err(e) => {
            tracing::warn!(error = %e, "assigned_groups_for_user task failed");
            Html(render_error("assigned_groups_for_user")).into_response()
        }
    }
}
