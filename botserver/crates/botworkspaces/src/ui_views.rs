use super::*;

pub async fn workspace_count(State(state): State<Arc<WorkspacesState>>) -> Html<String> {
    let Ok(mut conn) = state.pool.get() else {
        return Html("0".to_string());
    };

    let branch_id = ui_get_bot_context(&state);


    let count: i64 = aiworkspaces::table
        .filter(aiworkspaces::branch_id.eq(branch_id))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    Html(count.to_string())
}

pub async fn workspace_detail(
    State(state): State<Arc<WorkspacesState>>,
    Path(workspace_id): Path<Uuid>,
) -> Html<String> {
    let Ok(mut conn) = state.pool.get() else {
        return Html(render_empty_state("⚠️", "Database Error", "Could not connect to database"));
    };

    let workspace: DbWorkspace = match aiworkspaces::table
        .filter(aiworkspaces::id.eq(workspace_id))
        .first(&mut conn)
    {
        Ok(w) => w,
        Err(_) => {
            return Html(render_empty_state("❌", "Not Found", "Workspace not found"));
        }
    };

    let member_count: i64 = aiworkspace_members::table
        .filter(aiworkspace_members::workspace_id.eq(workspace_id))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    let page_count: i64 = aiworkspace_pages::table
        .filter(aiworkspace_pages::workspace_id.eq(workspace_id))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    let name = html_escape(&workspace.name);
    let description = workspace
        .description
        .as_deref()
        .map(html_escape)
        .unwrap_or_else(|| "No description".to_string());
    let icon = workspace.icon_value.as_deref().map(html_escape).unwrap_or_else(|| "📁".to_string());
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

pub async fn ui_workspace_pages(
    State(state): State<Arc<WorkspacesState>>,
    Path(workspace_id): Path<Uuid>,
    Query(query): Query<UiPageListQuery>,
) -> Html<String> {
    let Ok(mut conn) = state.pool.get() else {
        return Html(render_empty_state("⚠️", "Database Error", "Could not connect to database"));
    };

    let pages: Vec<DbWorkspacePage> = match query.parent_id {
        Some(parent_id) => aiworkspace_pages::table
            .filter(aiworkspace_pages::workspace_id.eq(workspace_id))
            .filter(aiworkspace_pages::parent_id.eq(parent_id))
            .order(aiworkspace_pages::position.asc())
            .load(&mut conn)
            .unwrap_or_default(),
        None => aiworkspace_pages::table
            .filter(aiworkspace_pages::workspace_id.eq(workspace_id))
            .filter(aiworkspace_pages::parent_id.is_null())
            .order(aiworkspace_pages::position.asc())
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
        let child_count: i64 = aiworkspace_pages::table
            .filter(aiworkspace_pages::parent_id.eq(page.id))
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

pub async fn ui_workspace_members(
    State(state): State<Arc<WorkspacesState>>,
    Path(workspace_id): Path<Uuid>,
) -> Html<String> {
    let Ok(mut conn) = state.pool.get() else {
        return Html(render_empty_state("⚠️", "Database Error", "Could not connect to database"));
    };

    let members: Vec<DbWorkspaceMember> = aiworkspace_members::table
        .filter(aiworkspace_members::workspace_id.eq(workspace_id))
        .order(aiworkspace_members::joined_at.asc())
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
    State(state): State<Arc<WorkspacesState>>,
    Path(page_id): Path<Uuid>,
) -> Html<String> {
    let Ok(mut conn) = state.pool.get() else {
        return Html(render_empty_state("⚠️", "Database Error", "Could not connect to database"));
    };

    let page: DbWorkspacePage = match aiworkspace_pages::table
        .filter(aiworkspace_pages::id.eq(page_id))
        .first(&mut conn)
    {
        Ok(p) => p,
        Err(_) => {
            return Html(render_empty_state("❌", "Not Found", "Page not found"));
        }
    };

    let title = html_escape(&page.title);
    let icon = page.icon_value.as_deref().map(html_escape).unwrap_or_else(|| "📄".to_string());
    let created = page.created_at.format("%Y-%m-%d %H:%M").to_string();
    let updated = page.updated_at.format("%Y-%m-%d %H:%M").to_string();
    let workspace_id = page.workspace_id;

    let content_preview = if page.content.is_null() || page.content == serde_json::json!([]) {
        r##"<p class="text-muted">This page is empty. Click Edit to add content.</p>"##.to_string()
    } else {
        r##"<div class="page-blocks" id="page-blocks" hx-get="/api/ui/pages/{page_id}/blocks" hx-trigger="load" hx-swap="innerHTML"></div>"##
            .to_string()
            .replace("{page_id}", &page_id.to_string())
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

