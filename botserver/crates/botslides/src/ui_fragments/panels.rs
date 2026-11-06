use super::{empty_fragment, html_escape};
use axum::{extract::State, response::Html, Json};
use std::sync::Arc;
use crate::types::Slide;
use crate::SlidesState;

pub async fn handle_theme_panel(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let name = payload.get("theme_name").and_then(|v| v.as_str()).unwrap_or("Default");
    let primary = payload.get("primary").and_then(|v| v.as_str()).unwrap_or("#3b82f6");
    let secondary = payload.get("secondary").and_then(|v| v.as_str()).unwrap_or("#10b981");
    let accent = payload.get("accent").and_then(|v| v.as_str()).unwrap_or("#f59e0b");
    let bg = payload.get("background").and_then(|v| v.as_str()).unwrap_or("#0f172a");
    let text = payload.get("text").and_then(|v| v.as_str()).unwrap_or("#f8fafc");
    let heading_font = payload.get("heading_font").and_then(|v| v.as_str()).unwrap_or("Arial");
    let body_font = payload.get("body_font").and_then(|v| v.as_str()).unwrap_or("Helvetica");
    Html(format!(
        r##"<div class="sl-panel" id="theme-panel" style="padding:16px;">
<div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:12px;">
<h3 style="margin:0;color:#f8fafc;font-size:16px;">Tema: {name}</h3>
<button hx-post="/suite/slides/fragments/themes-picker" hx-target="#theme-panel" hx-swap="outerHTML" style="background:#3b82f6;color:white;border:none;padding:4px 12px;border-radius:4px;cursor:pointer;font-size:12px;">Trocar Tema</button>
</div>
<div style="display:grid;grid-template-columns:1fr 1fr;gap:12px;">
<div style="padding:12px;background:#0f172a;border:1px solid #334155;border-radius:6px;">
<div style="font-size:11px;color:#94a3b8;margin-bottom:6px;">Cores</div>
<div style="display:flex;gap:4px;align-items:center;margin-bottom:4px;"><div style="width:16px;height:16px;background:{primary};border-radius:3px;"></div><span style="color:#cbd5e1;font-size:11px;">Primária</span></div>
<div style="display:flex;gap:4px;align-items:center;margin-bottom:4px;"><div style="width:16px;height:16px;background:{secondary};border-radius:3px;"></div><span style="color:#cbd5e1;font-size:11px;">Secundária</span></div>
<div style="display:flex;gap:4px;align-items:center;margin-bottom:4px;"><div style="width:16px;height:16px;background:{accent};border-radius:3px;"></div><span style="color:#cbd5e1;font-size:11px;">Acento</span></div>
<div style="display:flex;gap:4px;align-items:center;"><div style="width:16px;height:16px;background:{bg};border-radius:3px;border:1px solid #475569;"></div><span style="color:#cbd5e1;font-size:11px;">Fundo</span></div>
</div>
<div style="padding:12px;background:#0f172a;border:1px solid #334155;border-radius:6px;">
<div style="font-size:11px;color:#94a3b8;margin-bottom:6px;">Fontes</div>
<div style="color:{text};font-family:{heading_font};font-size:18px;font-weight:bold;margin-bottom:4px;">Aa Cabeçalho</div>
<div style="color:{text};font-family:{body_font};font-size:13px;">Aa Texto corpo</div>
</div>
</div>
</div>"##,
        name = html_escape(name),
        primary = html_escape(primary),
        secondary = html_escape(secondary),
        accent = html_escape(accent),
        bg = html_escape(bg),
        text = html_escape(text),
        heading_font = html_escape(heading_font),
        body_font = html_escape(body_font)
    ))
}

pub async fn handle_presenter_panel(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let pid = payload.get("presentation_id").and_then(|v| v.as_str()).unwrap_or("");
    let current = payload.get("current_slide").and_then(|v| v.as_u64()).unwrap_or(0);
    let total = payload.get("total_slides").and_then(|v| v.as_u64()).unwrap_or(0);
    let notes = payload.get("notes").and_then(|v| v.as_str()).unwrap_or("");
    Html(format!(
        r##"<div class="sl-panel" id="presenter-panel" style="padding:16px;background:#0f172a;border:1px solid #4338ca;border-radius:8px;">
<div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:12px;">
<h3 style="margin:0;color:#c7d2fe;font-size:16px;">🎤 Modo Apresentador</h3>
<div style="display:flex;align-items:center;gap:8px;">
<span style="color:#94a3b8;font-size:12px;">{position}</span>
<button onclick="this.closest('#slides-modal-container').innerHTML=''" style="background:none;border:none;color:#94a3b8;cursor:pointer;font-size:18px;">×</button>
</div>
</div>
<div style="display:flex;gap:8px;margin-bottom:12px;">
<button hx-post="/api/slides/presenter/start" hx-vals='{{"presentation_id":"{pid}"}}' hx-swap="none" hx-on::after-request="if(event.detail.successful && window.GBSlidesStartPresent) GBSlidesStartPresent()" style="flex:1;background:#10b981;color:white;border:none;padding:10px;border-radius:4px;cursor:pointer;font-weight:600;">▶ Iniciar</button>
<button hx-post="/api/slides/presenter/end" hx-vals='{{"presentation_id":"{pid}"}}' hx-swap="none" hx-on::after-request="if(event.detail.successful && window.GBSlidesEndPresent) GBSlidesEndPresent()" style="flex:1;background:#dc2626;color:white;border:none;padding:10px;border-radius:4px;cursor:pointer;font-weight:600;">⏹ Encerrar</button>
</div>
<div style="padding:8px;background:#1e293b;border-radius:4px;">
<div style="font-size:11px;color:#94a3b8;margin-bottom:4px;">Notas do apresentador</div>
<textarea name="notes" hx-post="/api/slides/slide/notes" hx-trigger="blur changed" hx-vals='{{"presentation_id":"{pid}","slide_index":"{cur0}"}}' hx-swap="none" style="width:100%;min-height:80px;padding:6px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;font-family:inherit;resize:vertical;">{notes}</textarea>
</div>
</div>"##,
        pid = html_escape(pid),
        position = if total > 0 {
            format!("Slide {}/{}", current + 1, total)
        } else {
            format!("Slide {}", current + 1)
        },
        cur0 = current,
        notes = html_escape(notes)
    ))
}

pub async fn handle_slide_sorter<D: crate::storage::DriveOps>(
    State(state): State<Arc<SlidesState<D>>>,
    Json(payload): Json<serde_json::Value>,
) -> Html<String> {
    let pid = payload
        .get("presentation_id")
        .or_else(|| payload.get("id"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let mut slides: Vec<Slide> = serde_json::from_value(
        payload.get("slides").cloned().unwrap_or(serde_json::Value::Null),
    )
    .unwrap_or_default();
    if slides.is_empty() {
        let pres = super::get_bridge::load_or_blank(&state, pid).await;
        slides = pres.slides;
    }
    if slides.is_empty() {
        return Html(empty_fragment("Nenhum slide para ordenar."));
    }
    let mut html = format!(
        r##"<div class="sl-panel" id="slide-sorter-panel" data-presentation-id="{pid}" style="padding:16px;">
<div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:12px;">
<h3 style="margin:0;color:#f8fafc;font-size:16px;">Organizar Slides</h3>
<button type="button" onclick="this.closest('#slides-modal-container').innerHTML=''" style="background:none;border:none;color:#94a3b8;cursor:pointer;font-size:18px;">×</button>
</div>
<div style="display:grid;grid-template-columns:repeat(auto-fill,minmax(160px,1fr));gap:8px;">"##,
        pid = html_escape(pid)
    );
    for (i, slide) in slides.iter().enumerate() {
        let bg = slide.background.color.clone().unwrap_or_else(|| "#0f172a".to_string());
        let text = slide.elements.iter()
            .find_map(|e| e.content.text.clone())
            .unwrap_or_default();
        let truncated = text.chars().take(60).collect::<String>();
        html.push_str(&format!(
            r##"<div class="sl-sorter-item" draggable="true" data-index="{i}" style="padding:8px;background:{bg};border:1px solid #334155;border-radius:6px;cursor:grab;">
<div style="aspect-ratio:16/9;display:flex;align-items:center;justify-content:center;color:#94a3b8;font-size:10px;padding:6px;text-align:center;overflow:hidden;border-radius:4px;background:rgba(0,0,0,0.3);">{truncated}</div>
<div style="font-size:11px;color:#cbd5e1;margin-top:4px;display:flex;justify-content:space-between;align-items:center;gap:4px;">
<span>Slide <span class="sl-sorter-order">{n}</span></span>
<span style="display:flex;gap:2px;">
<button type="button" class="sl-sorter-up" title="Mover para cima" style="background:#1e293b;color:#cbd5e1;border:1px solid #334155;border-radius:3px;cursor:pointer;font-size:11px;line-height:1;padding:2px 5px;">↑</button>
<button type="button" class="sl-sorter-down" title="Mover para baixo" style="background:#1e293b;color:#cbd5e1;border:1px solid #334155;border-radius:3px;cursor:pointer;font-size:11px;line-height:1;padding:2px 5px;">↓</button>
</span>
</div>
</div>"##,
            i = i,
            bg = html_escape(&bg),
            truncated = html_escape(&truncated),
            n = i + 1
        ));
    }
    html.push_str(
        r##"</div>
<div style="display:flex;align-items:center;gap:10px;margin-top:12px;">
<button type="button" id="slide-sorter-save" style="background:#10b981;color:white;border:none;padding:8px 16px;border-radius:4px;cursor:pointer;font-weight:600;">Salvar Ordem</button>
<span id="slide-sorter-status" role="status" style="font-size:12px;color:#94a3b8;"></span>
</div>
</div>"##,
    );
    Html(html)
}
