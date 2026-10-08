use super::*;

#[derive(Debug, Deserialize)]
pub struct UiListQuery {
    pub search: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UiPageListQuery {
    pub parent_id: Option<Uuid>,
}

pub(crate) fn ui_get_bot_context(state: &WorkspacesState) -> Uuid {
    get_bot_context(state)
}

pub(crate) fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

pub(crate) fn render_empty_state(icon: &str, title: &str, description: &str) -> String {
    format!(
        r##"<div class="empty-state">
<div class="empty-icon">{icon}</div>
<h3>{title}</h3>
<p>{description}</p>
</div>"##
    )
}

pub(crate) fn render_workspace_card(workspace: &DbWorkspace, member_count: i64, page_count: i64) -> String {
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
        .map(html_escape)
        .unwrap_or_else(|| "📁".to_string());

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

pub(crate) fn render_workspace_row(workspace: &DbWorkspace, member_count: i64, page_count: i64) -> String {
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

pub(crate) fn render_page_item(page: &DbWorkspacePage, child_count: i64) -> String {
    let title = html_escape(&page.title);
    let icon_escaped = page
        .icon_value
        .as_deref()
        .map(html_escape)
        .unwrap_or_else(|| "📄".to_string());
    let id = page.id;
    let workspace_id = page.workspace_id;
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
<span class="page-icon">{icon_escaped}</span>
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
    State(state): State<Arc<WorkspacesState>>,
    Query(query): Query<UiListQuery>,
) -> Html<String> {
    let Ok(mut conn) = state.pool.get() else {
        return Html(render_empty_state("⚠️", "Database Error", "Could not connect to database"));
    };

    let branch_id = ui_get_bot_context(&state);

    let mut q = aiworkspaces::table
        .filter(aiworkspaces::branch_id.eq(branch_id))
        .into_boxed();

    if let Some(search) = &query.search {
        let pattern = format!("%{search}%");
        q = q.filter(
            aiworkspaces::name
                .ilike(pattern.clone())
                .or(aiworkspaces::description.ilike(pattern)),
        );
    }

    let db_workspaces: Vec<DbWorkspace> = match q
        .order(aiworkspaces::updated_at.desc())
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
        let member_count: i64 = aiworkspace_members::table
            .filter(aiworkspace_members::workspace_id.eq(workspace.id))
            .count()
            .get_result(&mut conn)
            .unwrap_or(0);

        let page_count: i64 = aiworkspace_pages::table
            .filter(aiworkspace_pages::workspace_id.eq(workspace.id))
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
    State(state): State<Arc<WorkspacesState>>,
    Query(query): Query<UiListQuery>,
) -> Html<String> {
    let Ok(mut conn) = state.pool.get() else {
        return Html(render_empty_state("⚠️", "Database Error", "Could not connect to database"));
    };


    let branch_id = ui_get_bot_context(&state);

    let mut q = aiworkspaces::table
        .filter(aiworkspaces::branch_id.eq(branch_id))
        .into_boxed();

    if let Some(search) = &query.search {
        let pattern = format!("%{search}%");
        q = q.filter(
            aiworkspaces::name
                .ilike(pattern.clone())
                .or(aiworkspaces::description.ilike(pattern)),
        );
    }

    let db_workspaces: Vec<DbWorkspace> = match q
        .order(aiworkspaces::updated_at.desc())
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
        let member_count: i64 = aiworkspace_members::table
            .filter(aiworkspace_members::workspace_id.eq(workspace.id))
            .count()
            .get_result(&mut conn)
            .unwrap_or(0);

        let page_count: i64 = aiworkspace_pages::table
            .filter(aiworkspace_pages::workspace_id.eq(workspace.id))
            .count()
            .get_result(&mut conn)
            .unwrap_or(0);

        cards.push_str(&render_workspace_card(workspace, member_count, page_count));
    }

    Html(format!(r##"<div class="workspace-grid">{cards}</div>"##))
}

