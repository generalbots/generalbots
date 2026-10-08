use super::*;

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

pub(crate) fn render_canvas_card(canvas: &DbCanvas, element_count: i64) -> String {
    let name = html_escape(&canvas.name);
    let description = canvas
        .description
        .as_deref()
        .map(html_escape)
        .unwrap_or_default();
    let bg_color = canvas
        .background_color
        .as_deref()
        .unwrap_or("#ffffff");
    let updated = canvas.updated_at.format("%Y-%m-%d %H:%M").to_string();
    let id = canvas.id;
    let template_badge = if canvas.is_template {
        r##"<span class="badge badge-info">Template</span>"##
    } else {
        ""
    };
    let public_badge = if canvas.is_public {
        r##"<span class="badge badge-success">Public</span>"##
    } else {
        ""
    };

    format!(
        r##"<div class="canvas-card" data-id="{id}">
<div class="canvas-preview" style="background-color: {bg_color};">
<div class="canvas-element-count">{element_count} elements</div>
</div>
<div class="canvas-info">
<h4 class="canvas-name">{name}</h4>
<p class="canvas-description">{description}</p>
<div class="canvas-meta">
<span class="canvas-updated">{updated}</span>
{template_badge}
{public_badge}
</div>
</div>
<div class="canvas-actions">
<button class="btn btn-sm btn-primary" hx-get="/api/ui/canvas/{id}/editor" hx-target="#canvas-editor" hx-swap="innerHTML">
Open
</button>
<button class="btn btn-sm btn-secondary" hx-get="/api/ui/canvas/{id}/settings" hx-target="#modal-content" hx-swap="innerHTML">
Settings
</button>
<button class="btn btn-sm btn-danger" hx-delete="/api/canvas/{id}" hx-confirm="Delete this canvas?" hx-swap="none">
Delete
</button>
</div>
</div>"##
    )
}

pub(crate) fn render_canvas_row(canvas: &DbCanvas, element_count: i64) -> String {
    let name = html_escape(&canvas.name);
    let description = canvas
        .description
        .as_deref()
        .map(html_escape)
        .unwrap_or_else(|| "-".to_string());
    let updated = canvas.updated_at.format("%Y-%m-%d %H:%M").to_string();
    let id = canvas.id;
    let status = if canvas.is_public { "Public" } else { "Private" };

    format!(
        r##"<tr class="canvas-row" data-id="{id}">
<td class="canvas-name">
<a href="#" hx-get="/api/ui/canvas/{id}/editor" hx-target="#canvas-editor" hx-swap="innerHTML">{name}</a>
</td>
<td class="canvas-description">{description}</td>
<td class="canvas-elements">{element_count}</td>
<td class="canvas-status">{status}</td>
<td class="canvas-updated">{updated}</td>
<td class="canvas-actions">
<button class="btn btn-xs btn-primary" hx-get="/api/ui/canvas/{id}/editor" hx-target="#canvas-editor">Open</button>
<button class="btn btn-xs btn-danger" hx-delete="/api/canvas/{id}" hx-confirm="Delete?" hx-swap="none">Delete</button>
</td>
</tr>"##
    )
}

pub async fn canvas_list(
    State(state): State<Arc<CanvasState>>,
    Query(query): Query<UiListQuery>,
) -> Html<String> {
    let Ok(mut conn) = state.pool.get() else {
        return Html(render_empty_state("⚠️", "Database Error", "Could not connect to database"));
    };

    let (org_id, bot_id) = (state.get_bot_context)(&state.pool);

    let mut q = canvases::table
        .filter(canvases::org_id.eq(org_id))
        .filter(canvases::bot_id.eq(bot_id))
        .into_boxed();

    if let Some(is_template) = query.is_template {
        q = q.filter(canvases::is_template.eq(is_template));
    }

    if let Some(search) = &query.search {
        let pattern = format!("%{search}%");
        q = q.filter(
            canvases::name
                .ilike(pattern.clone())
                .or(canvases::description.ilike(pattern)),
        );
    }

    let db_canvases: Vec<DbCanvas> = match q
        .order(canvases::updated_at.desc())
        .limit(50)
        .load(&mut conn)
    {
        Ok(c) => c,
        Err(_) => {
            return Html(render_empty_state("⚠️", "Error", "Failed to load canvases"));
        }
    };

    if db_canvases.is_empty() {
        return Html(render_empty_state(
            "🎨",
            "No Canvases",
            "Create your first canvas to get started",
        ));
    }

    let mut rows = String::new();
    for canvas in &db_canvases {
        let element_count: i64 = canvas_elements::table
            .filter(canvas_elements::canvas_id.eq(canvas.id))
            .count()
            .get_result(&mut conn)
            .unwrap_or(0);

        rows.push_str(&render_canvas_row(canvas, element_count));
    }

    Html(format!(
        r##"<table class="table canvas-table">
<thead>
<tr>
<th>Name</th>
<th>Description</th>
<th>Elements</th>
<th>Status</th>
<th>Updated</th>
<th>Actions</th>
</tr>
</thead>
<tbody>{rows}</tbody>
</table>"##
    ))
}

pub async fn canvas_cards(
    State(state): State<Arc<CanvasState>>,
    Query(query): Query<UiListQuery>,
) -> Html<String> {
    let Ok(mut conn) = state.pool.get() else {
        return Html(render_empty_state("⚠️", "Database Error", "Could not connect to database"));
    };

    let (org_id, bot_id) = (state.get_bot_context)(&state.pool);

    let mut q = canvases::table
        .filter(canvases::org_id.eq(org_id))
        .filter(canvases::bot_id.eq(bot_id))
        .into_boxed();

    if let Some(is_template) = query.is_template {
        q = q.filter(canvases::is_template.eq(is_template));
    }

    if let Some(search) = &query.search {
        let pattern = format!("%{search}%");
        q = q.filter(
            canvases::name
                .ilike(pattern.clone())
                .or(canvases::description.ilike(pattern)),
        );
    }

    let db_canvases: Vec<DbCanvas> = match q
        .order(canvases::updated_at.desc())
        .limit(50)
        .load(&mut conn)
    {
        Ok(c) => c,
        Err(_) => {
            return Html(render_empty_state("⚠️", "Error", "Failed to load canvases"));
        }
    };

    if db_canvases.is_empty() {
        return Html(render_empty_state(
            "🎨",
            "No Canvases",
            "Create your first canvas to get started",
        ));
    }

    let mut cards = String::new();
    for canvas in &db_canvases {
        let element_count: i64 = canvas_elements::table
            .filter(canvas_elements::canvas_id.eq(canvas.id))
            .count()
            .get_result(&mut conn)
            .unwrap_or(0);

        cards.push_str(&render_canvas_card(canvas, element_count));
    }

    Html(format!(r##"<div class="canvas-grid">{cards}</div>"##))
}

pub async fn canvas_count(State(state): State<Arc<CanvasState>>) -> Html<String> {
    let Ok(mut conn) = state.pool.get() else {
        return Html("0".to_string());
    };

    let (org_id, bot_id) = (state.get_bot_context)(&state.pool);

    let count: i64 = canvases::table
        .filter(canvases::org_id.eq(org_id))
        .filter(canvases::bot_id.eq(bot_id))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    Html(count.to_string())
}

pub async fn canvas_templates_count(State(state): State<Arc<CanvasState>>) -> Html<String> {
    let Ok(mut conn) = state.pool.get() else {
        return Html("0".to_string());
    };

    let (org_id, bot_id) = (state.get_bot_context)(&state.pool);

    let count: i64 = canvases::table
        .filter(canvases::org_id.eq(org_id))
        .filter(canvases::bot_id.eq(bot_id))
        .filter(canvases::is_template.eq(true))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    Html(count.to_string())
}
