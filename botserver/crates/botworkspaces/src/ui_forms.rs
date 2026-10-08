use super::*;

pub async fn new_workspace_form(State(_state): State<Arc<WorkspacesState>>) -> Html<String> {
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
    State(_state): State<Arc<WorkspacesState>>,
    Path(workspace_id): Path<Uuid>,
    Query(query): Query<UiPageListQuery>,
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
    State(_state): State<Arc<WorkspacesState>>,
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
    State(state): State<Arc<WorkspacesState>>,
    Path(workspace_id): Path<Uuid>,
    Query(query): Query<UiListQuery>,
) -> Html<String> {
    let Ok(mut conn) = state.pool.get() else {
        return Html(render_empty_state("⚠️", "Database Error", "Could not connect to database"));
    };

    let search_term = match &query.search {
        Some(s) if !s.is_empty() => s,
        _ => {
            return Html(render_empty_state("🔍", "Search", "Enter a search term"));
        }
    };

    let pattern = format!("%{search_term}%");
    let pages: Vec<DbWorkspacePage> = aiworkspace_pages::table
        .filter(aiworkspace_pages::workspace_id.eq(workspace_id))
        .filter(aiworkspace_pages::title.ilike(&pattern))
        .order(aiworkspace_pages::updated_at.desc())
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

