use super::*;

mod elements;
mod forms;

pub use elements::canvas_elements_svg;
pub(crate) use elements::render_element_svg;
pub use forms::{canvas_settings, new_canvas_form};

pub async fn canvas_detail(
    State(state): State<Arc<CanvasState>>,
    Path(canvas_id): Path<Uuid>,
) -> Html<String> {
    let Ok(mut conn) = state.pool.get() else {
        return Html(render_empty_state(
            "⚠️",
            "Database Error",
            "Could not connect to database",
        ));
    };

    let canvas: DbCanvas = match canvases::table
        .filter(canvases::id.eq(canvas_id))
        .first(&mut conn)
    {
        Ok(c) => c,
        Err(_) => {
            return Html(render_empty_state("❌", "Not Found", "Canvas not found"));
        }
    };

    let elements: Vec<DbCanvasElement> = canvas_elements::table
        .filter(canvas_elements::canvas_id.eq(canvas_id))
        .order(canvas_elements::z_index.asc())
        .load(&mut conn)
        .unwrap_or_default();

    let name = html_escape(&canvas.name);
    let description = canvas
        .description
        .as_deref()
        .map(html_escape)
        .unwrap_or_else(|| "No description".to_string());
    let bg_color = canvas.background_color.as_deref().unwrap_or("#ffffff");
    let created = canvas.created_at.format("%Y-%m-%d %H:%M").to_string();
    let updated = canvas.updated_at.format("%Y-%m-%d %H:%M").to_string();
    let element_count = elements.len();
    let status = if canvas.is_public {
        "Public"
    } else {
        "Private"
    };
    let template_status = if canvas.is_template { "Yes" } else { "No" };

    Html(format!(
        r##"<div class="canvas-detail">
<div class="canvas-header">
<h2>{name}</h2>
<p class="canvas-description">{description}</p>
</div>
<div class="canvas-stats">
<div class="stat">
<span class="stat-label">Elements</span>
<span class="stat-value">{element_count}</span>
</div>
<div class="stat">
<span class="stat-label">Size</span>
<span class="stat-value">{width}x{height}</span>
</div>
<div class="stat">
<span class="stat-label">Background</span>
<span class="stat-value" style="background-color: {bg_color}; padding: 2px 8px;">{bg_color}</span>
</div>
<div class="stat">
<span class="stat-label">Status</span>
<span class="stat-value">{status}</span>
</div>
<div class="stat">
<span class="stat-label">Template</span>
<span class="stat-value">{template_status}</span>
</div>
</div>
<div class="canvas-dates">
<span>Created: {created}</span>
<span>Updated: {updated}</span>
</div>
<div class="canvas-actions">
<button class="btn btn-primary" hx-get="/api/ui/canvas/{canvas_id}/editor" hx-target="#canvas-editor" hx-swap="innerHTML">
Open Editor
</button>
<button class="btn btn-secondary" hx-get="/api/canvas/{canvas_id}/export" hx-vals='{{"format":"svg"}}' hx-target="#export-result">
Export SVG
</button>
<button class="btn btn-secondary" hx-get="/api/canvas/{canvas_id}/export" hx-vals='{{"format":"json"}}' hx-target="#export-result">
Export JSON
</button>
</div>
<div id="export-result"></div>
</div>"##,
        width = canvas.width,
        height = canvas.height
    ))
}

pub async fn canvas_editor(
    State(state): State<Arc<CanvasState>>,
    Path(canvas_id): Path<Uuid>,
) -> Html<String> {
    let Ok(mut conn) = state.pool.get() else {
        return Html(render_empty_state(
            "⚠️",
            "Database Error",
            "Could not connect to database",
        ));
    };

    let canvas: DbCanvas = match canvases::table
        .filter(canvases::id.eq(canvas_id))
        .first(&mut conn)
    {
        Ok(c) => c,
        Err(_) => {
            return Html(render_empty_state("❌", "Not Found", "Canvas not found"));
        }
    };

    let name = html_escape(&canvas.name);
    let bg_color = canvas.background_color.as_deref().unwrap_or("#ffffff");

    Html(format!(
        r##"<div class="canvas-editor" data-canvas-id="{canvas_id}">
<div class="editor-toolbar">
<div class="toolbar-left">
<span class="canvas-title">{name}</span>
</div>
<div class="toolbar-center">
<button class="tool-btn" data-tool="select" title="Select">
<span>🔲</span>
</button>
<button class="tool-btn" data-tool="rectangle" title="Rectangle">
<span>⬜</span>
</button>
<button class="tool-btn" data-tool="ellipse" title="Ellipse">
<span>⭕</span>
</button>
<button class="tool-btn" data-tool="line" title="Line">
<span>📏</span>
</button>
<button class="tool-btn" data-tool="text" title="Text">
<span>📝</span>
</button>
<button class="tool-btn" data-tool="freehand" title="Freehand">
<span>✏️</span>
</button>
<button class="tool-btn" data-tool="sticky" title="Sticky Note">
<span>📌</span>
</button>
</div>
<div class="toolbar-right">
<button class="btn btn-sm" hx-post="/api/canvas/{canvas_id}/versions" hx-swap="none">
Save Version
</button>
<button class="btn btn-sm btn-secondary" hx-get="/api/ui/canvas/{canvas_id}" hx-target="#canvas-detail">
Close
</button>
</div>
</div>
<div class="editor-workspace">
<div class="canvas-container" style="background-color: {bg_color};">
<svg id="canvas-svg"
width="{width}"
height="{height}"
viewBox="0 0 {width} {height}"
hx-get="/api/ui/canvas/{canvas_id}/elements"
hx-trigger="load"
hx-swap="innerHTML">
</svg>
</div>
<div class="properties-panel" id="properties-panel">
<h4>Properties</h4>
<p class="text-muted">Select an element to edit its properties</p>
</div>
</div>
</div>"##,
        width = canvas.width,
        height = canvas.height
    ))
}
