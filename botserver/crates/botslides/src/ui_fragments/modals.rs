use super::html_escape;
use axum::{response::Html, Json};

pub async fn handle_share_modal(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let id = payload.get("presentation_id").and_then(|v| v.as_str()).unwrap_or("");
    let name = payload.get("name").and_then(|v| v.as_str()).unwrap_or("Apresentação");
    Html(format!(
        r##"<div class="sl-modal" id="share-modal" style="position:fixed;top:0;left:0;right:0;bottom:0;background:rgba(0,0,0,0.7);z-index:1000;display:flex;align-items:center;justify-content:center;">
<div class="sl-modal-content" style="background:#1e293b;border:1px solid #334155;border-radius:8px;max-width:480px;width:90%;">
<div class="sl-modal-header" style="padding:16px;border-bottom:1px solid #334155;display:flex;justify-content:space-between;align-items:center;">
<h3 style="margin:0;color:#f8fafc;">Compartilhar "{name}"</h3>
<button onclick="this.closest('.sl-modal').remove()" style="background:none;border:none;color:#94a3b8;cursor:pointer;font-size:20px;">×</button>
</div>
<div class="sl-modal-body" hx-post="/suite/slides/forms/share" hx-vals='{{"presentation_id":"{id}"}}' hx-trigger="load" hx-swap="innerHTML"></div>
</div>
</div>"##,
        name = html_escape(name),
        id = html_escape(id)
    ))
}

pub async fn handle_ai_modal(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let id = payload.get("presentation_id").and_then(|v| v.as_str()).unwrap_or("");
    Html(format!(
        r##"<div class="sl-modal" id="ai-modal" style="position:fixed;top:0;left:0;right:0;bottom:0;background:rgba(0,0,0,0.7);z-index:1000;display:flex;align-items:center;justify-content:center;">
<div class="sl-modal-content" style="background:#1e293b;border:1px solid #4338ca;border-radius:8px;max-width:560px;width:90%;">
<div class="sl-modal-header" style="padding:16px;border-bottom:1px solid #4338ca;display:flex;justify-content:space-between;align-items:center;">
<h3 style="margin:0;color:#c7d2fe;">✨ Criar com IA</h3>
<button onclick="this.closest('.sl-modal').remove()" style="background:none;border:none;color:#94a3b8;cursor:pointer;font-size:20px;">×</button>
</div>
<div class="sl-modal-body" hx-post="/suite/slides/forms/ai" hx-vals='{{"presentation_id":"{id}"}}' hx-trigger="load" hx-swap="innerHTML">
<div id="ai-result" style="margin-top:12px;padding:8px;background:#0f172a;border-radius:4px;min-height:80px;color:#cbd5e1;font-size:13px;"></div>
</div>
</div>
</div>"##,
        id = html_escape(id)
    ))
}

pub async fn handle_new_slide_modal(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let id = payload.get("presentation_id").and_then(|v| v.as_str()).unwrap_or("");
    Html(format!(
        r##"<div class="sl-modal" id="new-slide-modal" style="position:fixed;top:0;left:0;right:0;bottom:0;background:rgba(0,0,0,0.7);z-index:1000;display:flex;align-items:center;justify-content:center;">
<div class="sl-modal-content" style="background:#1e293b;border:1px solid #334155;border-radius:8px;max-width:560px;width:90%;">
<div class="sl-modal-header" style="padding:16px;border-bottom:1px solid #334155;display:flex;justify-content:space-between;align-items:center;">
<h3 style="margin:0;color:#f8fafc;">Adicionar Slide</h3>
<button onclick="this.closest('.sl-modal').remove()" style="background:none;border:none;color:#94a3b8;cursor:pointer;font-size:20px;">×</button>
</div>
<div class="sl-modal-body" hx-post="/suite/slides/forms/new-slide" hx-vals='{{"presentation_id":"{id}"}}' hx-trigger="load" hx-swap="innerHTML"></div>
</div>
</div>"##,
        id = html_escape(id)
    ))
}

pub async fn handle_insert_element_modal(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let id = payload.get("presentation_id").and_then(|v| v.as_str()).unwrap_or("");
    let idx = payload.get("slide_index").and_then(|v| v.as_u64()).unwrap_or(0);
    Html(format!(
        r##"<div class="sl-modal" id="insert-element-modal" style="position:fixed;top:0;left:0;right:0;bottom:0;background:rgba(0,0,0,0.7);z-index:1000;display:flex;align-items:center;justify-content:center;">
<div class="sl-modal-content" style="background:#1e293b;border:1px solid #334155;border-radius:8px;max-width:560px;width:90%;">
<div class="sl-modal-header" style="padding:16px;border-bottom:1px solid #334155;display:flex;justify-content:space-between;align-items:center;">
<h3 style="margin:0;color:#f8fafc;">Inserir Elemento</h3>
<button onclick="this.closest('.sl-modal').remove()" style="background:none;border:none;color:#94a3b8;cursor:pointer;font-size:20px;">×</button>
</div>
<div class="sl-modal-body" hx-post="/suite/slides/forms/insert-element" hx-vals='{{"presentation_id":"{id}","slide_index":"{idx}"}}' hx-trigger="load" hx-swap="innerHTML"></div>
</div>
</div>"##,
        id = html_escape(id),
        idx = idx
    ))
}

pub async fn handle_add_text_modal(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let id = payload.get("presentation_id").and_then(|v| v.as_str()).unwrap_or("");
    let idx = payload.get("slide_index").and_then(|v| v.as_u64()).unwrap_or(0);
    Html(format!(
        r##"<div class="sl-modal" id="add-text-modal" style="position:fixed;top:0;left:0;right:0;bottom:0;background:rgba(0,0,0,0.7);z-index:1000;display:flex;align-items:center;justify-content:center;">
<div class="sl-modal-content" style="background:#1e293b;border:1px solid #334155;border-radius:8px;max-width:480px;width:90%;">
<div class="sl-modal-header" style="padding:16px;border-bottom:1px solid #334155;display:flex;justify-content:space-between;align-items:center;">
<h3 style="margin:0;color:#f8fafc;">Adicionar Texto</h3>
<button onclick="this.closest('.sl-modal').remove()" style="background:none;border:none;color:#94a3b8;cursor:pointer;font-size:20px;">×</button>
</div>
<div class="sl-modal-body" hx-post="/suite/slides/forms/add-text" hx-vals='{{"presentation_id":"{id}","slide_index":"{idx}"}}' hx-trigger="load" hx-swap="innerHTML"></div>
</div>
</div>"##,
        id = html_escape(id),
        idx = idx
    ))
}

pub async fn handle_add_image_modal(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let id = payload.get("presentation_id").and_then(|v| v.as_str()).unwrap_or("");
    Html(format!(
        r##"<div class="sl-modal" id="add-image-modal" style="position:fixed;top:0;left:0;right:0;bottom:0;background:rgba(0,0,0,0.7);z-index:1000;display:flex;align-items:center;justify-content:center;">
<div class="sl-modal-content" style="background:#1e293b;border:1px solid #334155;border-radius:8px;max-width:480px;width:90%;">
<div class="sl-modal-header" style="padding:16px;border-bottom:1px solid #334155;display:flex;justify-content:space-between;align-items:center;">
<h3 style="margin:0;color:#f8fafc;">Adicionar Imagem</h3>
<button onclick="this.closest('.sl-modal').remove()" style="background:none;border:none;color:#94a3b8;cursor:pointer;font-size:20px;">×</button>
</div>
<div class="sl-modal-body" hx-post="/suite/slides/forms/add-image" hx-vals='{{"presentation_id":"{id}"}}' hx-trigger="load" hx-swap="innerHTML"></div>
</div>
</div>"##,
        id = html_escape(id)
    ))
}

pub async fn handle_add_shape_modal(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let id = payload.get("presentation_id").and_then(|v| v.as_str()).unwrap_or("");
    let idx = payload.get("slide_index").and_then(|v| v.as_u64()).unwrap_or(0);
    Html(format!(
        r##"<div class="sl-modal" id="add-shape-modal" style="position:fixed;top:0;left:0;right:0;bottom:0;background:rgba(0,0,0,0.7);z-index:1000;display:flex;align-items:center;justify-content:center;">
<div class="sl-modal-content" style="background:#1e293b;border:1px solid #334155;border-radius:8px;max-width:480px;width:90%;">
<div class="sl-modal-header" style="padding:16px;border-bottom:1px solid #334155;display:flex;justify-content:space-between;align-items:center;">
<h3 style="margin:0;color:#f8fafc;">Adicionar Forma</h3>
<button onclick="this.closest('.sl-modal').remove()" style="background:none;border:none;color:#94a3b8;cursor:pointer;font-size:20px;">×</button>
</div>
<div class="sl-modal-body" hx-post="/suite/slides/forms/add-shape" hx-vals='{{"presentation_id":"{id}","slide_index":"{idx}"}}' hx-trigger="load" hx-swap="innerHTML"></div>
</div>
</div>"##,
        id = html_escape(id),
        idx = idx
    ))
}

pub async fn handle_add_chart_modal(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let id = payload.get("presentation_id").and_then(|v| v.as_str()).unwrap_or("");
    let idx = payload.get("slide_index").and_then(|v| v.as_u64()).unwrap_or(0);
    Html(format!(
        r##"<div class="sl-modal" id="add-chart-modal" style="position:fixed;top:0;left:0;right:0;bottom:0;background:rgba(0,0,0,0.7);z-index:1000;display:flex;align-items:center;justify-content:center;">
<div class="sl-modal-content" style="background:#1e293b;border:1px solid #334155;border-radius:8px;max-width:480px;width:90%;">
<div class="sl-modal-header" style="padding:16px;border-bottom:1px solid #334155;display:flex;justify-content:space-between;align-items:center;">
<h3 style="margin:0;color:#f8fafc;">Adicionar Gráfico</h3>
<button onclick="this.closest('.sl-modal').remove()" style="background:none;border:none;color:#94a3b8;cursor:pointer;font-size:20px;">×</button>
</div>
<div class="sl-modal-body" hx-post="/suite/slides/forms/add-chart" hx-vals='{{"presentation_id":"{id}","slide_index":"{idx}"}}' hx-trigger="load" hx-swap="innerHTML"></div>
</div>
</div>"##,
        id = html_escape(id),
        idx = idx
    ))
}

pub async fn handle_add_table_modal(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let id = payload.get("presentation_id").and_then(|v| v.as_str()).unwrap_or("");
    let idx = payload.get("slide_index").and_then(|v| v.as_u64()).unwrap_or(0);
    Html(format!(
        r##"<div class="sl-modal" id="add-table-modal" style="position:fixed;top:0;left:0;right:0;bottom:0;background:rgba(0,0,0,0.7);z-index:1000;display:flex;align-items:center;justify-content:center;">
<div class="sl-modal-content" style="background:#1e293b;border:1px solid #334155;border-radius:8px;max-width:480px;width:90%;">
<div class="sl-modal-header" style="padding:16px;border-bottom:1px solid #334155;display:flex;justify-content:space-between;align-items:center;">
<h3 style="margin:0;color:#f8fafc;">Adicionar Tabela</h3>
<button onclick="this.closest('.sl-modal').remove()" style="background:none;border:none;color:#94a3b8;cursor:pointer;font-size:20px;">×</button>
</div>
<div class="sl-modal-body" hx-post="/suite/slides/forms/add-table" hx-vals='{{"presentation_id":"{id}","slide_index":"{idx}"}}' hx-trigger="load" hx-swap="innerHTML"></div>
</div>
</div>"##,
        id = html_escape(id),
        idx = idx
    ))
}

pub async fn handle_notes_modal(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let id = payload.get("presentation_id").and_then(|v| v.as_str()).unwrap_or("");
    let idx = payload.get("slide_index").and_then(|v| v.as_u64()).unwrap_or(0);
    let notes = payload.get("notes").and_then(|v| v.as_str()).unwrap_or("");
    Html(format!(
        r##"<div class="sl-modal" id="notes-modal" style="position:fixed;top:0;left:0;right:0;bottom:0;background:rgba(0,0,0,0.7);z-index:1000;display:flex;align-items:center;justify-content:center;">
<div class="sl-modal-content" style="background:#1e293b;border:1px solid #334155;border-radius:8px;max-width:560px;width:90%;">
<div class="sl-modal-header" style="padding:16px;border-bottom:1px solid #334155;display:flex;justify-content:space-between;align-items:center;">
<h3 style="margin:0;color:#f8fafc;">📝 Notas do Apresentador</h3>
<button onclick="this.closest('.sl-modal').remove()" style="background:none;border:none;color:#94a3b8;cursor:pointer;font-size:20px;">×</button>
</div>
<div class="sl-modal-body" hx-post="/suite/slides/forms/notes" hx-vals='{{"presentation_id":"{id}","slide_index":"{idx}","notes":"{notes}"}}' hx-trigger="load" hx-swap="innerHTML"></div>
</div>
</div>"##,
        id = html_escape(id),
        idx = idx,
        notes = html_escape(notes)
    ))
}

pub async fn handle_background_modal(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let id = payload.get("presentation_id").and_then(|v| v.as_str()).unwrap_or("");
    Html(format!(
        r##"<div class="sl-modal" id="background-modal" style="position:fixed;top:0;left:0;right:0;bottom:0;background:rgba(0,0,0,0.7);z-index:1000;display:flex;align-items:center;justify-content:center;">
<div class="sl-modal-content" style="background:#1e293b;border:1px solid #334155;border-radius:8px;max-width:480px;width:90%;">
<div class="sl-modal-header" style="padding:16px;border-bottom:1px solid #334155;display:flex;justify-content:space-between;align-items:center;">
<h3 style="margin:0;color:#f8fafc;">🎨 Fundo do Slide</h3>
<button onclick="this.closest('.sl-modal').remove()" style="background:none;border:none;color:#94a3b8;cursor:pointer;font-size:20px;">×</button>
</div>
<div class="sl-modal-body" hx-post="/suite/slides/forms/background" hx-vals='{{"presentation_id":"{id}"}}' hx-trigger="load" hx-swap="innerHTML"></div>
</div>
</div>"##,
        id = html_escape(id)
    ))
}

pub async fn handle_transition_modal(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let id = payload.get("presentation_id").and_then(|v| v.as_str()).unwrap_or("");
    let idx = payload.get("slide_index").and_then(|v| v.as_u64()).unwrap_or(0);
    let ttype = payload.get("transition_type").and_then(|v| v.as_str()).unwrap_or("fade");
    Html(format!(
        r##"<div class="sl-modal" id="transition-modal" style="position:fixed;top:0;left:0;right:0;bottom:0;background:rgba(0,0,0,0.7);z-index:1000;display:flex;align-items:center;justify-content:center;">
<div class="sl-modal-content" style="background:#1e293b;border:1px solid #334155;border-radius:8px;max-width:480px;width:90%;">
<div class="sl-modal-header" style="padding:16px;border-bottom:1px solid #334155;display:flex;justify-content:space-between;align-items:center;">
<h3 style="margin:0;color:#f8fafc;">✨ Transição: {ttype}</h3>
<button onclick="this.closest('.sl-modal').remove()" style="background:none;border:none;color:#94a3b8;cursor:pointer;font-size:20px;">×</button>
</div>
<div class="sl-modal-body" hx-post="/suite/slides/forms/transition" hx-vals='{{"presentation_id":"{id}","slide_index":"{idx}","transition_type":"{ttype}"}}' hx-trigger="load" hx-swap="innerHTML"></div>
</div>
</div>"##,
        id = html_escape(id),
        idx = idx,
        ttype = html_escape(ttype)
    ))
}

pub async fn handle_animation_modal(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let id = payload.get("presentation_id").and_then(|v| v.as_str()).unwrap_or("");
    let atype = payload.get("type").and_then(|v| v.as_str()).unwrap_or("fade_in");
    Html(format!(
        r##"<div class="sl-modal" id="animation-modal" style="position:fixed;top:0;left:0;right:0;bottom:0;background:rgba(0,0,0,0.7);z-index:1000;display:flex;align-items:center;justify-content:center;">
<div class="sl-modal-content" style="background:#1e293b;border:1px solid #334155;border-radius:8px;max-width:480px;width:90%;">
<div class="sl-modal-header" style="padding:16px;border-bottom:1px solid #334155;display:flex;justify-content:space-between;align-items:center;">
<h3 style="margin:0;color:#f8fafc;">🎬 Animação: {atype}</h3>
<button onclick="this.closest('.sl-modal').remove()" style="background:none;border:none;color:#94a3b8;cursor:pointer;font-size:20px;">×</button>
</div>
<div class="sl-modal-body" hx-post="/suite/slides/forms/animation" hx-vals='{{"presentation_id":"{id}","type":"{atype}"}}' hx-trigger="load" hx-swap="innerHTML"></div>
</div>
</div>"##,
        id = html_escape(id),
        atype = html_escape(atype)
    ))
}

pub async fn handle_export_modal(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let id = payload.get("presentation_id").and_then(|v| v.as_str()).unwrap_or("");
    Html(format!(
        r##"<div class="sl-modal" id="export-modal" style="position:fixed;top:0;left:0;right:0;bottom:0;background:rgba(0,0,0,0.7);z-index:1000;display:flex;align-items:center;justify-content:center;">
<div class="sl-modal-content" style="background:#1e293b;border:1px solid #334155;border-radius:8px;max-width:480px;width:90%;">
<div class="sl-modal-header" style="padding:16px;border-bottom:1px solid #334155;display:flex;justify-content:space-between;align-items:center;">
<h3 style="margin:0;color:#f8fafc;">📥 Exportar Apresentação</h3>
<button onclick="this.closest('.sl-modal').remove()" style="background:none;border:none;color:#94a3b8;cursor:pointer;font-size:20px;">×</button>
</div>
<div class="sl-modal-body" hx-post="/suite/slides/forms/export" hx-vals='{{"presentation_id":"{id}"}}' hx-trigger="load" hx-swap="innerHTML"></div>
</div>
</div>"##,
        id = html_escape(id)
    ))
}

pub async fn handle_master_modal(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let id = payload.get("presentation_id").and_then(|v| v.as_str()).unwrap_or("");
    Html(format!(
        r##"<div class="sl-modal" id="master-modal" style="position:fixed;top:0;left:0;right:0;bottom:0;background:rgba(0,0,0,0.7);z-index:1000;display:flex;align-items:center;justify-content:center;">
<div class="sl-modal-content" style="background:#1e293b;border:1px solid #334155;border-radius:8px;max-width:480px;width:90%;">
<div class="sl-modal-header" style="padding:16px;border-bottom:1px solid #334155;display:flex;justify-content:space-between;align-items:center;">
<h3 style="margin:0;color:#f8fafc;">📐 Slide Mestre</h3>
<button onclick="this.closest('.sl-modal').remove()" style="background:none;border:none;color:#94a3b8;cursor:pointer;font-size:20px;">×</button>
</div>
<div class="sl-modal-body" hx-post="/suite/slides/forms/master" hx-vals='{{"presentation_id":"{id}"}}' hx-trigger="load" hx-swap="innerHTML"></div>
</div>
</div>"##,
        id = html_escape(id)
    ))
}
