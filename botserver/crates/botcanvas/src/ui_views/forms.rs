use super::*;

pub async fn canvas_settings(
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
    let description = canvas
        .description
        .as_deref()
        .map(html_escape)
        .unwrap_or_else(String::new);
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

pub async fn new_canvas_form(State(_state): State<Arc<CanvasState>>) -> Html<String> {
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
