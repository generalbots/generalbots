use axum::{
    extract::{Path, Query, State},
    response::Html,
    routing::get,
    Router,
};
use diesel::prelude::*;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::bot::get_default_bot;
use crate::core::shared::schema::{workspace_members, workspace_pages, workspaces};
use crate::shared::state::AppState;

use super::{DbWorkspace, DbWorkspaceMember, DbWorkspacePage};

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub search: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PageListQuery {
    pub parent_id: Option<Uuid>,
}

fn get_bot_context(state: &AppState) -> (Uuid, Uuid) {
    let Ok(mut conn) = state.conn.get() else {
        return (Uuid::nil(), Uuid::nil());
    };
    let (bot_id, _bot_name) = get_default_bot(&mut conn);
    let org_id = Uuid::nil();
    (org_id, bot_id)
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

fn render_empty_state(icon: &str, title: &str, description: &str) -> String {
    format!(
        r##"<div class="empty-state">
            <div class="empty-icon">{icon}</div>
            <h3>{title}</h3>
            <p>{description}</p>
        </div>"##
    )
}

fn render_workspace_card(workspace: &DbWorkspace, member_count: i64, page_count: i64) -> String {
    let name = html_escape(&workspace.name);
    let description = workspace
        .description
        .as_deref()
        .map(html_escape)
        .unwrap_or_else(|| "No description".to_string());
    let updated = workspace.updated_at.format("%Y-%m-%d %H:%M").to_string();
    let id = workspace.id;
    let icon = workspace
        .icon_value
        .as_deref()
        .unwrap_or("📁");

    format!(
        r##"<div class="workspace-card" data-id="{id}">
            <div class="workspace-icon">{icon}</div>
            <div class="workspace-info">
                <h4 class="workspace-name">{name}</h4>
                <p class="workspace-description">{description}</p>
                <div class="workspace-meta">
                    <span class="workspace-members">{member_count} members</span>
                    <span class="workspace-pages">{page_count} pages</span>
                    <span class="workspace-updated">{updated}</span>
                </div>
            </div>
            <div class="workspace-actions">
                <button class="btn btn-sm btn-primary" hx-get="/api/ui/workspaces/{id}/pages" hx-target="#workspace-content" hx-swap="innerHTML">
                    Open
                </button>
                <button class="btn btn-sm btn-secondary" hx-get="/api/ui/workspaces/{id}/settings" hx-target="#modal-content" hx-swap="innerHTML">
                    Settings
                </button>
                <button class="btn btn-sm btn-danger" hx-delete="/api/workspaces/{id}" hx-confirm="Delete this workspace?" hx-swap="none">
                    Delete
                </button>
            </div>
        </div>"##
    )
}

fn render_workspace_row(workspace: &DbWorkspace, member_count: i64, page_count: i64) -> String {
    let name = html_escape(&workspace.name);
    let description = workspace
        .description
        .as_deref()
        .map(html_escape)
        .unwrap_or_else(|| "-".to_string());
    let updated = workspace.updated_at.format("%Y-%m-%d %H:%M").to_string();
    let id = workspace.id;
    let icon = workspace.icon_value.as_deref().unwrap_or("📁");

    format!(
        r##"<tr class="workspace-row" data-id="{id}">
            <td class="workspace-icon">{icon}</td>
            <td class="workspace-name">
                <a href="#" hx-get="/api/ui/workspaces/{id}/pages" hx-target="#workspace-content" hx-swap="innerHTML">{name}</a>
            </td>
            <td class="workspace-description">{description}</td>
            <td class="workspace-members">{member_count}</td>
            <td class="workspace-pages">{page_count}</td>
            <td class="workspace-updated">{updated}</td>
            <td class="workspace-actions">
                <button class="btn btn-xs btn-primary" hx-get="/api/ui/workspaces/{id}/pages" hx-target="#workspace-content">Open</button>
                <button class="btn btn-xs btn-danger" hx-delete="/api/workspaces/{id}" hx-confirm="Delete?" hx-swap="none">Delete</button>
            </td>
        </tr>"##
    )
}

fn render_page_item(page: &DbWorkspacePage, child_count: i64) -> String {
    let title = html_escape(&page.title);
    let id = page.id;
    let workspace_id = page.workspace_id;
    let icon = page.icon_value.as_deref().unwrap_or("📄");
    let updated = page.updated_at.format("%Y-%m-%d %H:%M").to_string();
    let has_children = if child_count > 0 {
        format!(
            r##"<button class="btn-expand" hx-get="/api/ui/workspaces/{workspace_id}/pages?parent_id={id}" hx-target="#children-{id}" hx-swap="innerHTML">
                <span class="expand-icon">▶</span>
            </button>"##
        )
    } else {
        r##"<span class="no-expand"></span>"##.to_string()
    };

    format!(
        r##"<div class="page-item" data-id="{id}">
            <div class="page-row">
                {has_children}
                <span class="page-icon">{icon}</span>
                <a class="page-title" href="#" hx-get="/api/ui/pages/{id}" hx-target="#page-content" hx-swap="innerHTML">{title}</a>
                <span class="page-updated">{updated}</span>
                <div class="page-actions">
                    <button class="btn btn-xs" hx-get="/api/ui/pages/{id}/edit" hx-target="#modal-content">Edit</button>
                    <button class="btn btn-xs btn-danger" hx-delete="/api/pages/{id}" hx-confirm="Delete?" hx-swap="none">Delete</button>
                </div>
            </div>
            <div class="page-children" id="children-{id}"></div>
        </div>"##
    )
}

pub async fn workspace_list(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListQuery>,
) -> Html<String> {
    let Ok(mut conn) = state.conn.get() else {
        return Html(render_empty_state("⚠️", "Database Error", "Could not connect to database"));
    };

    let (org_id, bot_id) = get_bot_context(&state);

    let mut q = workspaces::table
        .filter(workspaces::org_id.eq(org_id))
        .filter(workspaces::bot_id.eq(bot_id))
        .into_boxed();

    if let Some(search) = &query.search {
        let pattern = format!("%{search}%");
        q = q.filter(
            workspaces::name
                .ilike(pattern.clone())
                .or(workspaces::description.ilike(pattern)),
        );
    }

    let db_workspaces: Vec<DbWorkspace> = match q
        .order(workspaces::updated_at.desc())
        .limit(50)
        .load(&mut conn)
    {
        Ok(w) => w,
        Err(_) => {
            return Html(render_empty_state("⚠️", "Error", "Failed to load workspaces"));
        }
    };

    if db_workspaces.is_empty() {
        return Html(render_empty_state(
            "📁",
            "No Workspaces",
            "Create your first workspace to get started",
        ));
    }

    let mut rows = String::new();
    for workspace in &db_workspaces {
        let member_count: i64 = workspace_members::table
            .filter(workspace_members::workspace_id.eq(workspace.id))
            .count()
            .get_result(&mut conn)
            .unwrap_or(0);

        let page_count: i64 = workspace_pages::table
            .filter(workspace_pages::workspace_id.eq(workspace.id))
            .count()
            .get_result(&mut conn)
            .unwrap_or(0);

        rows.push_str(&render_workspace_row(workspace, member_count, page_count));
    }

    Html(format!(
        r##"<table class="table workspace-table">
            <thead>
                <tr>
                    <th></th>
                    <th>Name</th>
                    <th>Description</th>
                    <th>Members</th>
                    <th>Pages</th>
                    <th>Updated</th>
                    <th>Actions</th>
                </tr>
            </thead>
            <tbody>{rows}</tbody>
        </table>"##
    ))
}

pub async fn workspace_cards(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListQuery>,
) -> Html<String> {
    let Ok(mut conn) = state.conn.get() else {
        return Html(render_empty_state("⚠️", "Database Error", "Could not connect to database"));
    };

    let (org_id, bot_id) = get_bot_context(&state);

    let mut q = workspaces::table
        .filter(workspaces::org_id.eq(org_id))
        .filter(workspaces::bot_id.eq(bot_id))
        .into_boxed();

    if let Some(search) = &query.search {
        let pattern = format!("%{search}%");
        q = q.filter(
            workspaces::name
                .ilike(pattern.clone())
                .or(workspaces::description.ilike(pattern)),
        );
    }

    let db_workspaces: Vec<DbWorkspace> = match q
        .order(workspaces::updated_at.desc())
        .limit(50)
        .load(&mut conn)
    {
        Ok(w) => w,
        Err(_) => {
            return Html(render_empty_state("⚠️", "Error", "Failed to load workspaces"));
        }
    };

    if db_workspaces.is_empty() {
        return Html(render_empty_state(
            "📁",
            "No Workspaces",
            "Create your first workspace to get started",
        ));
    }

    let mut cards = String::new();
    for workspace in &db_workspaces {
        let member_count: i64 = workspace_members::table
            .filter(workspace_members::workspace_id.eq(workspace.id))
            .count()
            .get_result(&mut conn)
            .unwrap_or(0);

        let page_count: i64 = workspace_pages::table
            .filter(workspace_pages::workspace_id.eq(workspace.id))
            .count()
            .get_result(&mut conn)
            .unwrap_or(0);

        cards.push_str(&render_workspace_card(workspace, member_count, page_count));
    }

    Html(format!(r##"<div class="workspace-grid">{cards}</div>"##))
}

pub async fn workspace_count(State(state): State<Arc<AppState>>) -> Html<String> {
    let Ok(mut conn) = state.conn.get() else {
        return Html("0".to_string());
    };

    let (org_id, bot_id) = get_bot_context(&state);

    let count: i64 = workspaces::table
        .filter(workspaces::org_id.eq(org_id))
        .filter(workspaces::bot_id.eq(bot_id))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    Html(count.to_string())
}

pub async fn workspace_detail(
    State(state): State<Arc<AppState>>,
    Path(workspace_id): Path<Uuid>,
) -> Html<String> {
    let Ok(mut conn) = state.conn.get() else {
        return Html(render_empty_state("⚠️", "Database Error", "Could not connect to database"));
    };

    let workspace: DbWorkspace = match workspaces::table
        .filter(workspaces::id.eq(workspace_id))
        .first(&mut conn)
    {
        Ok(w) => w,
        Err(_) => {
            return Html(render_empty_state("❌", "Not Found", "Workspace not found"));
        }
    };

    let member_count: i64 = workspace_members::table
        .filter(workspace_members::workspace_id.eq(workspace_id))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    let page_count: i64 = workspace_pages::table
        .filter(workspace_pages::workspace_id.eq(workspace_id))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    let name = html_escape(&workspace.name);
    let description = workspace
        .description
        .as_deref()
        .map(html_escape)
        .unwrap_or_else(|| "No description".to_string());
    let icon = workspace.icon_value.as_deref().unwrap_or("📁");
    let created = workspace.created_at.format("%Y-%m-%d %H:%M").to_string();
    let updated = workspace.updated_at.format("%Y-%m-%d %H:%M").to_string();

    Html(format!(
        r##"<div class="workspace-detail">
            <div class="workspace-header">
                <span class="workspace-icon-large">{icon}</span>
                <div class="workspace-title">
                    <h2>{name}</h2>
                    <p class="workspace-description">{description}</p>
                </div>
            </div>
            <div class="workspace-stats">
                <div class="stat">
                    <span class="stat-label">Members</span>
                    <span class="stat-value">{member_count}</span>
                </div>
                <div class="stat">
                    <span class="stat-label">Pages</span>
                    <span class="stat-value">{page_count}</span>
                </div>
            </div>
            <div class="workspace-dates">
                <span>Created: {created}</span>
                <span>Updated: {updated}</span>
            </div>
            <div class="workspace-actions">
                <button class="btn btn-primary" hx-get="/api/ui/workspaces/{workspace_id}/pages" hx-target="#workspace-content" hx-swap="innerHTML">
                    View Pages
                </button>
                <button class="btn btn-secondary" hx-get="/api/ui/workspaces/{workspace_id}/members" hx-target="#workspace-content" hx-swap="innerHTML">
                    Manage Members
                </button>
                <button class="btn btn-secondary" hx-get="/api/ui/workspaces/{workspace_id}/settings" hx-target="#modal-content" hx-swap="innerHTML">
                    Settings
                </button>
            </div>
        </div>"##
    ))
}

pub async fn workspace_pages(
    State(state): State<Arc<AppState>>,
    Path(workspace_id): Path<Uuid>,
    Query(query): Query<PageListQuery>,
) -> Html<String> {
    let Ok(mut conn) = state.conn.get() else {
        return Html(render_empty_state("⚠️", "Database Error", "Could not connect to database"));
    };

    let pages: Vec<DbWorkspacePage> = match query.parent_id {
        Some(parent_id) => workspace_pages::table
            .filter(workspace_pages::workspace_id.eq(workspace_id))
            .filter(workspace_pages::parent_id.eq(parent_id))
            .order(workspace_pages::position.asc())
            .load(&mut conn)
            .unwrap_or_default(),
        None => workspace_pages::table
            .filter(workspace_pages::workspace_id.eq(workspace_id))
            .filter(workspace_pages::parent_id.is_null())
            .order(workspace_pages::position.asc())
            .load(&mut conn)
            .unwrap_or_default(),
    };

    if pages.is_empty() && query.parent_id.is_none() {
        return Html(render_empty_state(
            "📄",
            "No Pages",
            "Create your first page to get started",
        ));
    }

    let mut items = String::new();
    for page in &pages {
        let child_count: i64 = workspace_pages::table
            .filter(workspace_pages::parent_id.eq(page.id))
            .count()
            .get_result(&mut conn)
            .unwrap_or(0);

        items.push_str(&render_page_item(page, child_count));
    }

    if query.parent_id.is_some() {
        Html(items)
    } else {
        Html(format!(
            r##"<div class="workspace-pages-header">
                <h3>Pages</h3>
                <button class="btn btn-primary" hx-get="/api/ui/workspaces/{workspace_id}/pages/new" hx-target="#modal-content" hx-swap="innerHTML">
                    New Page
                </button>
            </div>
            <div class="page-tree">{items}</div>"##
        ))
    }
}

pub async fn workspace_members(
    State(state): State<Arc<AppState>>,
    Path(workspace_id): Path<Uuid>,
) -> Html<String> {
    let Ok(mut conn) = state.conn.get() else {
        return Html(render_empty_state("⚠️", "Database Error", "Could not connect to database"));
    };

    let members: Vec<DbWorkspaceMember> = workspace_members::table
        .filter(workspace_members::workspace_id.eq(workspace_id))
        .order(workspace_members::joined_at.asc())
        .load(&mut conn)
        .unwrap_or_default();

    if members.is_empty() {
        return Html(render_empty_state(
            "👥",
            "No Members",
            "This workspace has no members",
        ));
    }

    let mut rows = String::new();
    for member in &members {
        let user_id = member.user_id;
        let role = html_escape(&member.role);
        let joined = member.joined_at.format("%Y-%m-%d").to_string();
        let role_class = match role.as_str() {
            "owner" => "badge-primary",
            "admin" => "badge-warning",
            "editor" => "badge-info",
            _ => "badge-secondary",
        };

        rows.push_str(&format!(
            r##"<tr class="member-row" data-user-id="{user_id}">
                <td class="member-user">{user_id}</td>
                <td class="member-role"><span class="badge {role_class}">{role}</span></td>
                <td class="member-joined">{joined}</td>
                <td class="member-actions">
                    <button class="btn btn-xs btn-danger" hx-delete="/api/workspaces/{workspace_id}/members/{user_id}" hx-confirm="Remove member?" hx-swap="none">
                        Remove
                    </button>
                </td>
            </tr>"##
        ));
    }

    Html(format!(
        r##"<div class="workspace-members-header">
            <h3>Members</h3>
            <button class="btn btn-primary" hx-get="/api/ui/workspaces/{workspace_id}/members/add" hx-target="#modal-content" hx-swap="innerHTML">
                Add Member
            </button>
        </div>
        <table class="table members-table">
            <thead>
                <tr>
                    <th>User</th>
                    <th>Role</th>
                    <th>Joined</th>
                    <th>Actions</th>
                </tr>
            </thead>
            <tbody>{rows}</tbody>
        </table>"##
    ))
}

pub async fn page_detail(
    State(state): State<Arc<AppState>>,
    Path(page_id): Path<Uuid>,
) -> Html<String> {
    let Ok(mut conn) = state.conn.get() else {
        return Html(render_empty_state("⚠️", "Database Error", "Could not connect to database"));
    };

    let page: DbWorkspacePage = match workspace_pages::table
        .filter(workspace_pages::id.eq(page_id))
        .first(&mut conn)
    {
        Ok(p) => p,
        Err(_) => {
            return Html(render_empty_state("❌", "Not Found", "Page not found"));
        }
    };

    let title = html_escape(&page.title);
    let icon = page.icon_value.as_deref().unwrap_or("📄");
    let created = page.created_at.format("%Y-%m-%d %H:%M").to_string();
    let updated = page.updated_at.format("%Y-%m-%d %H:%M").to_string();
    let workspace_id = page.workspace_id;

    let content_preview = if page.content.is_null() || page.content == serde_json::json!([]) {
        r##"<p class="text-muted">This page is empty. Click Edit to add content.</p>"##.to_string()
    } else {
        r##"<div class="page-blocks" id="page-blocks" hx-get="/api/ui/pages/{page_id}/blocks" hx-trigger="load" hx-swap="innerHTML"></div>"##.to_string().replace("{page_id}", &page_id.to_string())
    };

    Html(format!(
        r##"<div class="page-detail">
            <div class="page-header">
                <div class="page-breadcrumb" hx-get="/api/ui/pages/{page_id}/breadcrumb" hx-trigger="load" hx-swap="innerHTML"></div>
                <div class="page-title-row">
                    <span class="page-icon-large">{icon}</span>
                    <h2 class="page-title">{title}</h2>
                </div>
            </div>
            <div class="page-meta">
                <span>Created: {created}</span>
                <span>Updated: {updated}</span>
            </div>
            <div class="page-actions">
                <button class="btn btn-primary" hx-get="/api/ui/pages/{page_id}/edit" hx-target="#modal-content" hx-swap="innerHTML">
                    Edit
                </button>
                <button class="btn btn-secondary" hx-get="/api/ui/workspaces/{workspace_id}/pages/new?parent_id={page_id}" hx-target="#modal-content" hx-swap="innerHTML">
                    Add Subpage
                </button>
                <button class="btn btn-danger" hx-delete="/api/pages/{page_id}" hx-confirm="Delete this page?" hx-swap="none">
                    Delete
                </button>
            </div>
            <div class="page-content">
                {content_preview}
            </div>
        </div>"##
    ))
}

pub async fn new_workspace_form(State(_state): State<Arc<AppState>>) -> Html<String> {
    Html(
        r##"<div class="modal-header">
            <h3>New Workspace</h3>
            <button class="btn-close" onclick="closeModal()">&times;</button>
        </div>
        <form class="workspace-form" hx-post="/api/workspaces" hx-swap="none" hx-on::after-request="closeModal(); htmx.trigger('#workspace-list', 'refresh');">
            <div class="form-group">
                <label>Name</label>
                <input type="text" name="name" placeholder="My Workspace" required />
            </div>
            <div class="form-group">
                <label>Description</label>
                <textarea name="description" rows="3" placeholder="Describe your workspace..."></textarea>
            </div>
            <div class="form-actions">
                <button type="button" class="btn btn-secondary" onclick="closeModal()">Cancel</button>
                <button type="submit" class="btn btn-primary">Create Workspace</button>
            </div>
        </form>"##
            .to_string(),
    )
}

pub async fn new_page_form(
    State(_state): State<Arc<AppState>>,
    Path(workspace_id): Path<Uuid>,
    Query(query): Query<PageListQuery>,
) -> Html<String> {
    let parent_input = match query.parent_id {
        Some(parent_id) => format!(r##"<input type="hidden" name="parent_id" value="{parent_id}" />"##),
        None => String::new(),
    };

    Html(format!(
        r##"<div class="modal-header">
            <h3>New Page</h3>
            <button class="btn-close" onclick="closeModal()">&times;</button>
        </div>
        <form class="page-form" hx-post="/api/workspaces/{workspace_id}/pages" hx-swap="none" hx-on::after-request="closeModal(); htmx.trigger('#page-tree', 'refresh');">
            {parent_input}
            <div class="form-group">
                <label>Title</label>
                <input type="text" name="title" placeholder="Page Title" required />
            </div>
            <div class="form-actions">
                <button type="button" class="btn btn-secondary" onclick="closeModal()">Cancel</button>
                <button type="submit" class="btn btn-primary">Create Page</button>
            </div>
        </form>"##
    ))
}

pub async fn workspace_settings(
    State(state): State<Arc<AppState>>,
    Path(workspace_id): Path<Uuid>,
) -> Html<String> {
    let Ok(mut conn) = state.conn.get() else {
        return Html(render_empty_state("⚠️", "Database Error", "Could not connect to database"));
    };

    let workspace: DbWorkspace = match workspaces::table
        .filter(workspaces::id.eq(workspace_id))
        .first(&mut conn)
    {
        Ok(w) => w,
        Err(_) => {
            return Html(render_empty_state("❌", "Not Found", "Workspace not found"));
        }
    };

    let name = html_escape(&workspace.name);
    let description = workspace.description.as_deref().map(html_escape).unwrap_or_default();

    Html(format!(
        r##"<div class="modal-header">
            <h3>Workspace Settings</h3>
            <button class="btn-close" onclick="closeModal()">&times;</button>
        </div>
        <form class="workspace-settings-form" hx-put="/api/workspaces/{workspace_id}" hx-swap="none" hx-on::after-request="closeModal()">
            <div class="form-group">
                <label>Name</label>
                <input type="text" name="name" value="{name}" required />
            </div>
            <div class="form-group">
                <label>Description</label>
                <textarea name="description" rows="3">{description}</textarea>
            </div>
            <div class="form-actions">
                <button type="button" class="btn btn-secondary" onclick="closeModal()">Cancel</button>
                <button type="submit" class="btn btn-primary">Save Changes</button>
            </div>
        </form>"##
    ))
}

pub async fn add_member_form(
    State(_state): State<Arc<AppState>>,
    Path(workspace_id): Path<Uuid>,
) -> Html<String> {
    Html(format!(
        r##"<div class="modal-header">
            <h3>Add Member</h3>
            <button class="btn-close" onclick="closeModal()">&times;</button>
        </div>
        <form class="add-member-form" hx-post="/api/workspaces/{workspace_id}/members" hx-swap="none" hx-on::after-request="closeModal(); htmx.trigger('#members-table', 'refresh');">
            <div class="form-group">
                <label>User ID</label>
                <input type="text" name="user_id" placeholder="User UUID" required />
            </div>
            <div class="form-group">
                <label>Role</label>
                <select name="role" required>
                    <option value="viewer">Viewer</option>
                    <option value="commenter">Commenter</option>
                    <option value="editor">Editor</option>
                    <option value="admin">Admin</option>
                </select>
            </div>
            <div class="form-actions">
                <button type="button" class="btn btn-secondary" onclick="closeModal()">Cancel</button>
                <button type="submit" class="btn btn-primary">Add Member</button>
            </div>
        </form>"##
    ))
}

pub async fn search_results(
    State(state): State<Arc<AppState>>,
    Path(workspace_id): Path<Uuid>,
    Query(query): Query<ListQuery>,
) -> Html<String> {
    let Ok(mut conn) = state.conn.get() else {
        return Html(render_empty_state("⚠️", "Database Error", "Could not connect to database"));
    };

    let search_term = match &query.search {
        Some(s) if !s.is_empty() => s,
        _ => {
            return Html(render_empty_state("🔍", "Search", "Enter a search term"));
        }
    };

    let pattern = format!("%{search_term}%");
    let pages: Vec<DbWorkspacePage> = workspace_pages::table
        .filter(workspace_pages::workspace_id.eq(workspace_id))
        .filter(workspace_pages::title.ilike(&pattern))
        .order(workspace_pages::updated_at.desc())
        .limit(20)
        .load(&mut conn)
        .unwrap_or_default();

    if pages.is_empty() {
        return Html(render_empty_state(
            "🔍",
            "No Results",
            "No pages match your search",
        ));
    }

    let mut items = String::new();
    for page in &pages {
        let title = html_escape(&page.title);
        let id = page.id;
        let icon = page.icon_value.as_deref().unwrap_or("📄");
        let updated = page.updated_at.format("%Y-%m-%d %H:%M").to_string();

        items.push_str(&format!(
            r##"<div class="search-result" data-id="{id}">
                <span class="result-icon">{icon}</span>
                <a class="result-title" href="#" hx-get="/api/ui/pages/{id}" hx-target="#page-content" hx-swap="innerHTML">{title}</a>
                <span class="result-updated">{updated}</span>
            </div>"##
        ));
    }

    Html(format!(
        r##"<div class="search-results">
            <h4>Search Results ({count})</h4>
            {items}
        </div>"##,
        count = pages.len()
    ))
}

pub fn configure_workspaces_ui_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/ui/workspaces", get(workspace_list))
        .route("/api/ui/workspaces/cards", get(workspace_cards))
        .route("/api/ui/workspaces/count", get(workspace_count))
        .route("/api/ui/workspaces/new", get(new_workspace_form))
        .route("/api/ui/workspaces/{workspace_id}", get(workspace_detail))
        .route("/api/ui/workspaces/{workspace_id}/pages", get(workspace_pages))
        .route("/api/ui/workspaces/{workspace_id}/pages/new", get(new_page_form))
        .route("/api/ui/workspaces/{workspace_id}/members", get(workspace_members))
        .route("/api/ui/workspaces/{workspace_id}/members/add", get(add_member_form))
        .route("/api/ui/workspaces/{workspace_id}/settings", get(workspace_settings))
        .route("/api/ui/workspaces/{workspace_id}/search", get(search_results))
        .route("/api/ui/pages/{page_id}", get(page_detail))
}
