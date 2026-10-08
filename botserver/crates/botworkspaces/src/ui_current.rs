use super::*;

pub(crate) fn resolve_current_workspace_id(state: &WorkspacesState) -> Uuid {
    let Ok(mut conn) = state.pool.get() else { return Uuid::nil(); };
    let branch_id = ui_get_bot_context(state);
    aiworkspaces::table
        .filter(aiworkspaces::branch_id.eq(branch_id))
        .select(aiworkspaces::id)
        .first(&mut conn)
        .unwrap_or(Uuid::nil())
}

pub(crate) async fn workspace_current_detail(State(state): State<Arc<WorkspacesState>>) -> Html<String> {
    let id = resolve_current_workspace_id(&state);
    workspace_detail(State(state), Path(id)).await
}

pub(crate) async fn workspace_current_pages(State(state): State<Arc<WorkspacesState>>) -> Html<String> {
    let id = resolve_current_workspace_id(&state);
    ui_workspace_pages(State(state), Path(id), Query(UiPageListQuery { parent_id: None })).await
}

pub(crate) async fn workspace_current_members(State(state): State<Arc<WorkspacesState>>) -> Html<String> {
    let id = resolve_current_workspace_id(&state);
    ui_workspace_members(State(state), Path(id)).await
}

pub(crate) async fn workspace_current_settings(State(state): State<Arc<WorkspacesState>>) -> Html<String> {
    let id = resolve_current_workspace_id(&state);
    workspace_settings(State(state), Path(id)).await
}

pub(crate) async fn workspace_current_search(State(state): State<Arc<WorkspacesState>>, Query(query): Query<UiListQuery>) -> Html<String> {
    let id = resolve_current_workspace_id(&state);
    search_results(State(state), Path(id), Query(query)).await
}

pub(crate) async fn workspace_current_page_new(State(state): State<Arc<WorkspacesState>>, Query(query): Query<UiPageListQuery>) -> Html<String> {
    let id = resolve_current_workspace_id(&state);
    new_page_form(State(state), Path(id), Query(query)).await
}

pub(crate) async fn workspace_current_page_create(
    State(state): State<Arc<WorkspacesState>>,
    Json(req): Json<CreatePageRequest>,
) -> axum::response::Html<String> {
    let id = resolve_current_workspace_id(&state);
    match create_page(State(state), Path(id), Json(req)).await {
        Ok(page) => axum::response::Html(format!("<div class='page-item' data-page-id='{}'>{}</div>", page.id, html_escape(&page.title))),
        Err(_) => axum::response::Html("<div class='error'>Failed to create page</div>".to_string()),
    }
}

pub(crate) async fn workspace_commands() -> Html<String> {
    let commands = [
        ("/text", "Text block", "📝"),
        ("/heading1", "Heading 1", "🔠"),
        ("/heading2", "Heading 2", "🔡"),
        ("/todo", "To-do list", "☑️"),
        ("/bullet", "Bulleted list", "•"),
        ("/numbered", "Numbered list", "1."),
        ("/quote", "Quote", "💬"),
        ("/code", "Code block", "💻"),
        ("/divider", "Divider", "—"),
        ("/callout", "Callout", "💡"),
    ];
    let mut html = String::from("<div class=\"command-list\">");
    for (cmd, label, icon) in &commands {
        html.push_str(&format!(
            "<div class=\"command-item\" data-command=\"{cmd}\"><span class=\"command-icon\">{icon}</span><span class=\"command-label\">{label}</span><kbd>{cmd}</kbd></div>",
            cmd = html_escape(cmd),
            icon = html_escape(icon),
            label = html_escape(label),
        ));
    }
    html.push_str("</div>");
    Html(html)
}

pub(crate) async fn workspace_current_invite(State(state): State<Arc<WorkspacesState>>) -> Html<String> {
    let id = resolve_current_workspace_id(&state);
    Html(format!(
        r##"<div class="invite-modal">
        <div class="invite-modal-header"><h3>Invite Members</h3><button class="modal-close" onclick="closeWorkspaceModal()">×</button></div>
        <form hx-post="/api/ui/workspaces/{id}/members/add" hx-target="#members-list" hx-swap="innerHTML">
            <label>Email address</label>
            <input type="email" name="email" placeholder="teammate@example.com" required />
            <label>Role</label>
            <select name="role"><option value="member">Member</option><option value="admin">Admin</option></select>
            <button type="submit" class="btn-primary btn-sm">Send Invite</button>
        </form>
        </div>"##
    ))
}

pub(crate) async fn pages_current_get(State(state): State<Arc<WorkspacesState>>) -> Html<String> {
    let workspace_id = resolve_current_workspace_id(&state);
    let Ok(mut conn) = state.pool.get() else {
        return Html("<p class='text-muted'>Unable to load page</p>".to_string());
    };

    let page: Option<DbWorkspacePage> = aiworkspace_pages::table
        .filter(aiworkspace_pages::workspace_id.eq(workspace_id))
        .order(aiworkspace_pages::position.asc())
        .first(&mut conn)
        .ok();

    match page {
        Some(page) => {
            let content_preview = if page.content.is_null() || page.content == serde_json::json!([]) {
                "<p class=\"text-muted\">This page is empty.</p>".to_string()
            } else {
                let blocks = page
                    .content
                    .as_array()
                    .cloned()
                    .unwrap_or_default();
                let mut html = String::new();
                for block in blocks.iter().take(5) {
                    let block_type = block
                        .get("type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("text");
                    let text = block
                        .get("content")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    html.push_str(&format!(
                        "<div class=\"block-preview block-{block_type}\">{text}</div>",
                        block_type = html_escape(block_type),
                        text = html_escape(text),
                    ));
                }
                html
            };
            Html(format!(
                "<div class='page-content-inner'><h2 class='page-title-read'>{}</h2>{}</div>",
                html_escape(&page.title),
                content_preview,
            ))
        }
        None => Html("<div class='empty-page'><p>No page selected</p></div>".to_string()),
    }
}

#[derive(Deserialize)]
pub struct PageTitleRequest {
    pub title: Option<String>,
}

pub(crate) async fn pages_current_put(
    State(state): State<Arc<WorkspacesState>>,
    Json(req): Json<PageTitleRequest>,
) -> Html<String> {
    let workspace_id = resolve_current_workspace_id(&state);
    let title = req.title.unwrap_or_else(|| "Untitled".to_string());
    let Ok(mut conn) = state.pool.get() else {
        return Html("".to_string());
    };

    let _ = diesel::update(aiworkspace_pages::table.filter(aiworkspace_pages::workspace_id.eq(workspace_id)))
        .set((
            aiworkspace_pages::title.eq(&title),
            aiworkspace_pages::updated_at.eq(Utc::now()),
        ))
        .execute(&mut conn);

    Html("".to_string())
}

pub(crate) async fn pages_current_blocks(State(state): State<Arc<WorkspacesState>>) -> Html<String> {
    let workspace_id = resolve_current_workspace_id(&state);
    let Ok(mut conn) = state.pool.get() else {
        return Html("<p class='text-muted'>Unable to load blocks</p>".to_string());
    };

    let page: Option<DbWorkspacePage> = aiworkspace_pages::table
        .filter(aiworkspace_pages::workspace_id.eq(workspace_id))
        .order(aiworkspace_pages::position.asc())
        .first(&mut conn)
        .ok();

    match page {
        Some(page) if !page.content.is_null() => {
            let blocks = page
                .content
                .as_array()
                .cloned()
                .unwrap_or_default();
            if blocks.is_empty() {
                return Html("<div class='empty-page'><p>Press <kbd>/</kbd> for commands or start typing...</p></div>".to_string());
            }
            let mut html = String::from("<div class=\"blocks\">");
            for block in &blocks {
                let block_type = block.get("type").and_then(|v| v.as_str()).unwrap_or("text");
                let text = block.get("content").and_then(|v| v.as_str()).unwrap_or("");
                html.push_str(&format!(
                    "<div class=\"block block-{block_type}\" contenteditable=\"true\">{text}</div>",
                    block_type = html_escape(block_type),
                    text = html_escape(text),
                ));
            }
            html.push_str("</div>");
            Html(html)
        }
        _ => Html("<div class='empty-page'><p>Press <kbd>/</kbd> for commands or start typing...</p></div>".to_string()),
    }
}

pub fn configure_workspaces_ui_routes() -> Router<Arc<WorkspacesState>> {
    Router::new()
        .route("/api/ui/workspaces", get(workspace_list))
        .route("/api/ui/workspaces/cards", get(workspace_cards))
        .route("/api/ui/workspaces/count", get(workspace_count))
        .route("/api/ui/workspaces/new", get(new_workspace_form))
        .route("/api/ui/workspaces/:workspace_id", get(workspace_detail))
        .route("/api/ui/workspaces/:workspace_id/pages", get(ui_workspace_pages).post(workspace_ui_create_page))
        .route("/api/ui/workspaces/:workspace_id/pages/new", get(new_page_form))
        .route("/api/ui/workspaces/:workspace_id/members", get(ui_workspace_members))
        .route("/api/ui/workspaces/:workspace_id/members/add", get(add_member_form))
        .route("/api/ui/workspaces/:workspace_id/settings", get(workspace_settings))
        .route("/api/ui/workspaces/:workspace_id/search", get(search_results))
        .route("/api/ui/pages/:page_id", get(page_detail))
        .route("/api/ui/workspaces/current", get(workspace_current_detail))
        .route("/api/ui/workspaces/current/pages", get(workspace_current_pages).post(workspace_current_page_create))
        .route("/api/ui/workspaces/current/pages/new", get(workspace_current_page_new))
        .route("/api/ui/workspaces/current/members", get(workspace_current_members))
        .route("/api/ui/workspaces/current/settings", get(workspace_current_settings))
        .route("/api/ui/workspaces/current/search", get(workspace_current_search))
        .route("/api/ui/workspaces/commands", get(workspace_commands))
        .route("/api/ui/workspaces/current/invite", get(workspace_current_invite))
        .route("/api/pages/current", get(pages_current_get).put(pages_current_put))
        .route("/api/ui/pages/current/blocks", get(pages_current_blocks))
}

pub(crate) async fn workspace_ui_create_page(
    State(state): State<Arc<WorkspacesState>>,
    Path(workspace_id): Path<Uuid>,
    Json(req): Json<CreatePageRequest>,
) -> axum::response::Html<String> {
    match create_page(State(state), Path(workspace_id), Json(req)).await {
        Ok(page) => axum::response::Html(format!("<div class='page-item' data-page-id='{}'>{}</div>", page.id, html_escape(&page.title))),
        Err(_) => axum::response::Html("<div class='error'>Failed to create page</div>".to_string()),
    }
}
