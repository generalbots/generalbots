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
use crate::core::shared::schema::{canvas_elements, canvases};
use crate::shared::state::AppState;

use super::{DbCanvas, DbCanvasElement};

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub search: Option<String>,
    pub is_template: Option<bool>,
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

fn render_canvas_card(canvas: &DbCanvas, element_count: i64) -> String {
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

fn render_canvas_row(canvas: &DbCanvas, element_count: i64) -> String {
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
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListQuery>,
) -> Html<String> {
    let Ok(mut conn) = state.conn.get() else {
        return Html(render_empty_state("⚠️", "Database Error", "Could not connect to database"));
    };

    let (org_id, bot_id) = get_bot_context(&state);

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
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListQuery>,
) -> Html<String> {
    let Ok(mut conn) = state.conn.get() else {
        return Html(render_empty_state("⚠️", "Database Error", "Could not connect to database"));
    };

    let (org_id, bot_id) = get_bot_context(&state);

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

pub async fn canvas_count(State(state): State<Arc<AppState>>) -> Html<String> {
    let Ok(mut conn) = state.conn.get() else {
        return Html("0".to_string());
    };

    let (org_id, bot_id) = get_bot_context(&state);

    let count: i64 = canvases::table
        .filter(canvases::org_id.eq(org_id))
        .filter(canvases::bot_id.eq(bot_id))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    Html(count.to_string())
}

pub async fn canvas_templates_count(State(state): State<Arc<AppState>>) -> Html<String> {
    let Ok(mut conn) = state.conn.get() else {
        return Html("0".to_string());
    };

    let (org_id, bot_id) = get_bot_context(&state);

    let count: i64 = canvases::table
        .filter(canvases::org_id.eq(org_id))
        .filter(canvases::bot_id.eq(bot_id))
        .filter(canvases::is_template.eq(true))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    Html(count.to_string())
}

pub async fn canvas_detail(
    State(state): State<Arc<AppState>>,
    Path(canvas_id): Path<Uuid>,
) -> Html<String> {
    let Ok(mut conn) = state.conn.get() else {
        return Html(render_empty_state("⚠️", "Database Error", "Could not connect to database"));
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
    let status = if canvas.is_public { "Public" } else { "Private" };
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
    State(state): State<Arc<AppState>>,
    Path(canvas_id): Path<Uuid>,
) -> Html<String> {
    let Ok(mut conn) = state.conn.get() else {
        return Html(render_empty_state("⚠️", "Database Error", "Could not connect to database"));
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

pub async fn canvas_elements_svg(
    State(state): State<Arc<AppState>>,
    Path(canvas_id): Path<Uuid>,
) -> Html<String> {
    let Ok(mut conn) = state.conn.get() else {
        return Html(String::new());
    };

    let elements: Vec<DbCanvasElement> = canvas_elements::table
        .filter(canvas_elements::canvas_id.eq(canvas_id))
        .order(canvas_elements::z_index.asc())
        .load(&mut conn)
        .unwrap_or_default();

    let mut svg_elements = String::new();
    for el in &elements {
        let svg = render_element_svg(el);
        svg_elements.push_str(&svg);
    }

    Html(svg_elements)
}

fn render_element_svg(element: &DbCanvasElement) -> String {
    let id = element.id;
    let x = element.x;
    let y = element.y;
    let width = element.width;
    let height = element.height;
    let rotation = element.rotation;

    let transform = if rotation != 0.0 {
        format!(
            r##" transform="rotate({rotation} {} {})""##,
            x + width / 2.0,
            y + height / 2.0
        )
    } else {
        String::new()
    };

    let fill = element
        .properties
        .get("fill_color")
        .and_then(|v| v.as_str())
        .unwrap_or("transparent");
    let stroke = element
        .properties
        .get("stroke_color")
        .and_then(|v| v.as_str())
        .unwrap_or("#000000");
    let stroke_width = element
        .properties
        .get("stroke_width")
        .and_then(|v| v.as_f64())
        .unwrap_or(2.0);
    let opacity = element
        .properties
        .get("opacity")
        .and_then(|v| v.as_f64())
        .unwrap_or(1.0);

    match element.element_type.as_str() {
        "rectangle" => {
            let corner_radius = element
                .properties
                .get("corner_radius")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            format!(
                r##"<rect data-element-id="{id}" x="{x}" y="{y}" width="{width}" height="{height}" rx="{corner_radius}" fill="{fill}" stroke="{stroke}" stroke-width="{stroke_width}" opacity="{opacity}"{transform} class="canvas-element"/>"##
            )
        }
        "ellipse" => {
            let cx = x + width / 2.0;
            let cy = y + height / 2.0;
            let rx = width / 2.0;
            let ry = height / 2.0;
            format!(
                r##"<ellipse data-element-id="{id}" cx="{cx}" cy="{cy}" rx="{rx}" ry="{ry}" fill="{fill}" stroke="{stroke}" stroke-width="{stroke_width}" opacity="{opacity}"{transform} class="canvas-element"/>"##
            )
        }
        "line" | "arrow" => {
            let x2 = x + width;
            let y2 = y + height;
            let marker = if element.element_type == "arrow" {
                r##" marker-end="url(#arrowhead)""##
            } else {
                ""
            };
            format!(
                r##"<line data-element-id="{id}" x1="{x}" y1="{y}" x2="{x2}" y2="{y2}" stroke="{stroke}" stroke-width="{stroke_width}" opacity="{opacity}"{marker}{transform} class="canvas-element"/>"##
            )
        }
        "text" => {
            let text = element
                .properties
                .get("text")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let font_size = element
                .properties
                .get("font_size")
                .and_then(|v| v.as_f64())
                .unwrap_or(16.0);
            let font_family = element
                .properties
                .get("font_family")
                .and_then(|v| v.as_str())
                .unwrap_or("sans-serif");
            let text_y = y + font_size;
            format!(
                r##"<text data-element-id="{id}" x="{x}" y="{text_y}" font-size="{font_size}" font-family="{font_family}" fill="{fill}" opacity="{opacity}"{transform} class="canvas-element">{text}</text>"##,
                text = html_escape(text)
            )
        }
        "freehand_path" => {
            let path_data = element
                .properties
                .get("path_data")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            format!(
                r##"<path data-element-id="{id}" d="{path_data}" fill="none" stroke="{stroke}" stroke-width="{stroke_width}" opacity="{opacity}"{transform} class="canvas-element"/>"##
            )
        }
        "sticky" => {
            let text = element
                .properties
                .get("text")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let bg = element
                .properties
                .get("fill_color")
                .and_then(|v| v.as_str())
                .unwrap_or("#ffeb3b");
            format!(
                r##"<g data-element-id="{id}" class="canvas-element sticky-note"{transform}>
                    <rect x="{x}" y="{y}" width="{width}" height="{height}" fill="{bg}" stroke="#000" stroke-width="1"/>
                    <text x="{text_x}" y="{text_y}" font-size="14" fill="#000">{text}</text>
                </g>"##,
                text_x = x + 8.0,
                text_y = y + 24.0,
                text = html_escape(text)
            )
        }
        _ => format!(
            r##"<rect data-element-id="{id}" x="{x}" y="{y}" width="{width}" height="{height}" fill="{fill}" stroke="{stroke}" stroke-width="{stroke_width}" opacity="{opacity}"{transform} class="canvas-element"/>"##
        ),
    }
}

pub async fn canvas_settings(
    State(state): State<Arc<AppState>>,
    Path(canvas_id): Path<Uuid>,
) -> Html<String> {
    let Ok(mut conn) = state.conn.get() else {
        return Html(render_empty_state("⚠️", "Database Error", "Could not connect to database"));
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
    let description = canvas.description.as_deref().map(html_escape).unwrap_or_default();
    let bg_color = canvas.background_color.as_deref().unwrap_or("#ffffff");
    let is_public_checked = if canvas.is_public { "checked" } else { "" };
    let is_template_checked = if canvas.is_template { "checked" } else { "" };

    Html(format!(
        r##"<div class="modal-header">
            <h3>Canvas Settings</h3>
            <button class="btn-close" onclick="closeModal()">&times;</button>
        </div>
        <form class="canvas-settings-form" hx-put="/api/canvas/{canvas_id}" hx-swap="none" hx-on::after-request="closeModal()">
            <div class="form-group">
                <label>Name</label>
                <input type="text" name="name" value="{name}" required />
            </div>
            <div class="form-group">
                <label>Description</label>
                <textarea name="description" rows="3">{description}</textarea>
            </div>
            <div class="form-row">
                <div class="form-group">
                    <label>Width</label>
                    <input type="number" name="width" value="{width}" min="100" max="10000" />
                </div>
                <div class="form-group">
                    <label>Height</label>
                    <input type="number" name="height" value="{height}" min="100" max="10000" />
                </div>
            </div>
            <div class="form-group">
                <label>Background Color</label>
                <input type="color" name="background_color" value="{bg_color}" />
            </div>
            <div class="form-group">
                <label class="checkbox-label">
                    <input type="checkbox" name="is_public" value="true" {is_public_checked} />
                    Public (anyone can view)
                </label>
            </div>
            <div class="form-group">
                <label class="checkbox-label">
                    <input type="checkbox" name="is_template" value="true" {is_template_checked} />
                    Save as template
                </label>
            </div>
            <div class="form-actions">
                <button type="button" class="btn btn-secondary" onclick="closeModal()">Cancel</button>
                <button type="submit" class="btn btn-primary">Save Changes</button>
            </div>
        </form>"##,
        width = canvas.width,
        height = canvas.height
    ))
}

pub async fn new_canvas_form(State(_state): State<Arc<AppState>>) -> Html<String> {
    Html(
        r##"<div class="modal-header">
            <h3>New Canvas</h3>
            <button class="btn-close" onclick="closeModal()">&times;</button>
        </div>
        <form class="canvas-form" hx-post="/api/canvas" hx-swap="none" hx-on::after-request="closeModal(); htmx.trigger('#canvas-list', 'refresh');">
            <div class="form-group">
                <label>Name</label>
                <input type="text" name="name" placeholder="My Canvas" required />
            </div>
            <div class="form-group">
                <label>Description</label>
                <textarea name="description" rows="3" placeholder="Describe your canvas..."></textarea>
            </div>
            <div class="form-row">
                <div class="form-group">
                    <label>Width</label>
                    <input type="number" name="width" value="1920" min="100" max="10000" />
                </div>
                <div class="form-group">
                    <label>Height</label>
                    <input type="number" name="height" value="1080" min="100" max="10000" />
                </div>
            </div>
            <div class="form-group">
                <label>Background Color</label>
                <input type="color" name="background_color" value="#ffffff" />
            </div>
            <div class="form-group">
                <label class="checkbox-label">
                    <input type="checkbox" name="is_template" value="true" />
                    Create as template
                </label>
            </div>
            <div class="form-actions">
                <button type="button" class="btn btn-secondary" onclick="closeModal()">Cancel</button>
                <button type="submit" class="btn btn-primary">Create Canvas</button>
            </div>
        </form>"##
            .to_string(),
    )
}

pub fn configure_canvas_ui_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/ui/canvas", get(canvas_list))
        .route("/api/ui/canvas/cards", get(canvas_cards))
        .route("/api/ui/canvas/count", get(canvas_count))
        .route("/api/ui/canvas/templates/count", get(canvas_templates_count))
        .route("/api/ui/canvas/new", get(new_canvas_form))
        .route("/api/ui/canvas/{canvas_id}", get(canvas_detail))
        .route("/api/ui/canvas/{canvas_id}/editor", get(canvas_editor))
        .route("/api/ui/canvas/{canvas_id}/elements", get(canvas_elements_svg))
        .route("/api/ui/canvas/{canvas_id}/settings", get(canvas_settings))
}
