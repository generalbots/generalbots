use super::{empty_fragment, html_escape};
use axum::{response::Html, Json};
use crate::types::{MediaElement, Slide, TransitionConfig};

pub async fn handle_slide_list_panel(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let presentation_id = payload.get("presentation_id").and_then(|v| v.as_str()).unwrap_or("");
    let current = payload.get("current_slide").and_then(|v| v.as_u64()).unwrap_or(0);
    let slides: Vec<Slide> = serde_json::from_value(payload.get("slides").cloned().unwrap_or(serde_json::Value::Null)).unwrap_or_default();
    if slides.is_empty() {
        return Html(empty_fragment("Nenhum slide. Adicione um slide para começar."));
    }
    let mut html = String::from(
        r##"<div class="sl-panel" id="slide-list-panel" style="padding:8px;">
<div class="sl-panel-header" style="display:flex;justify-content:space-between;align-items:center;padding:4px 8px;margin-bottom:8px;">
<h3 style="margin:0;color:#f8fafc;font-size:14px;">Slides</h3>
<button hx-get="/suite/slides/modals/new-slide" hx-vals='{"presentation_id":"{id}"}' hx-target="#modal-container" hx-swap="innerHTML" style="background:#3b82f6;color:white;border:none;padding:4px 8px;border-radius:4px;cursor:pointer;font-size:11px;">+ Slide</button>
</div>"##
    );
    for (i, slide) in slides.iter().enumerate() {
        let active = if i as u64 == current { "border:2px solid #3b82f6;background:#1e3a8a;" } else { "border:1px solid #334155;background:#0f172a;" };
        let bg = slide.background.color.clone().unwrap_or_else(|| "#0f172a".to_string());
        let preview_text = slide.elements.iter()
            .find_map(|e| e.content.text.clone())
            .unwrap_or_default();
        let truncated = preview_text.chars().take(40).collect::<String>();
        let element_count = slide.elements.len();
        html.push_str(&format!(
            r##"<div class="sl-slide-thumb" hx-get="/suite/slides/fragments/thumbnails" hx-vals='{{"presentation_id":"{pid}","index":"{i}"}}' hx-target="#slides-content" hx-swap="innerHTML" style="padding:8px;margin-bottom:6px;cursor:pointer;border-radius:6px;{active}">
<div style="aspect-ratio:16/9;background:{bg};border-radius:4px;display:flex;align-items:center;justify-content:center;color:#94a3b8;font-size:11px;padding:8px;text-align:center;overflow:hidden;">{truncated}</div>
<div style="font-size:11px;color:#cbd5e1;margin-top:4px;">Slide {n} • {ec} elem.</div>
</div>"##,
            pid = html_escape(presentation_id),
            i = i,
            active = active,
            bg = html_escape(&bg),
            truncated = html_escape(&truncated),
            n = i + 1,
            ec = element_count
        ));
    }
    html.push_str("</div>");
    Html(html)
}

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
<button hx-get="/suite/slides/fragments/themes-picker" hx-target="#theme-panel" hx-swap="outerHTML" style="background:#3b82f6;color:white;border:none;padding:4px 12px;border-radius:4px;cursor:pointer;font-size:12px;">Trocar Tema</button>
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

pub async fn handle_transitions_panel(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let pid = payload.get("presentation_id").and_then(|v| v.as_str()).unwrap_or("");
    let slide_idx = payload.get("slide_index").and_then(|v| v.as_u64()).unwrap_or(0);
    let transitions: Vec<TransitionConfig> = serde_json::from_value(payload.get("transitions").cloned().unwrap_or(serde_json::Value::Null)).unwrap_or_default();
    let mut html = String::from(
        r##"<div class="sl-panel" id="transitions-panel" style="padding:16px;">
<h3 style="margin:0 0 12px 0;color:#f8fafc;font-size:16px;">Transições</h3>
<div style="display:grid;grid-template-columns:1fr 1fr;gap:8px;">"##
    );
    let types = vec![
        ("none", "Nenhuma"),
        ("fade", "Fade"),
        ("slide_left", "Slide Esq"),
        ("slide_right", "Slide Dir"),
        ("slide_up", "Slide Cima"),
        ("slide_down", "Slide Baixo"),
        ("zoom_in", "Zoom In"),
        ("zoom_out", "Zoom Out"),
        ("flip_horizontal", "Flip H"),
        ("flip_vertical", "Flip V"),
        ("cube", "Cubo"),
        ("cover", "Cobrir"),
    ];
    for (ttype, tname) in types {
        let is_current = transitions.iter().any(|t| t.transition_type == ttype);
        let bg = if is_current { "#1e3a8a" } else { "#0f172a" };
        let border = if is_current { "border:2px solid #3b82f6;" } else { "border:1px solid #334155;" };
        html.push_str(&format!(
            r##"<button hx-post="/api/slides/transition" hx-vals='{{"presentation_id":"{pid}","slide_index":"{idx}","transition_type":"{ttype}","duration":"500"}}' hx-target="#transitions-panel" hx-swap="outerHTML" style="padding:12px;background:{bg};{border}border-radius:6px;color:#f8fafc;cursor:pointer;text-align:center;font-size:12px;">{tname}</button>"##,
            pid = html_escape(pid),
            idx = slide_idx,
            ttype = ttype,
            tname = tname,
            bg = bg,
            border = border
        ));
    }
    html.push_str("</div></div>");
    Html(html)
}

pub async fn handle_animations_panel(Json(_payload): Json<serde_json::Value>) -> Html<String> {
    Html(format!(
        r##"<div class="sl-panel" id="animations-panel" style="padding:16px;">
<h3 style="margin:0 0 12px 0;color:#f8fafc;font-size:16px;">Animações</h3>
<div style="display:grid;grid-template-columns:1fr 1fr;gap:8px;">
<button hx-get="/suite/slides/modals/animation" hx-vals='{{"action":"add","type":"fade_in"}}' hx-target="#modal-container" hx-swap="innerHTML" style="padding:12px;background:#0f172a;border:1px solid #334155;border-radius:6px;color:#f8fafc;cursor:pointer;font-size:12px;text-align:center;">Fade In</button>
<button hx-get="/suite/slides/modals/animation" hx-vals='{{"action":"add","type":"fade_out"}}' hx-target="#modal-container" hx-swap="innerHTML" style="padding:12px;background:#0f172a;border:1px solid #334155;border-radius:6px;color:#f8fafc;cursor:pointer;font-size:12px;text-align:center;">Fade Out</button>
<button hx-get="/suite/slides/modals/animation" hx-vals='{{"action":"add","type":"slide_in_left"}}' hx-target="#modal-container" hx-swap="innerHTML" style="padding:12px;background:#0f172a;border:1px solid #334155;border-radius:6px;color:#f8fafc;cursor:pointer;font-size:12px;text-align:center;">Slide In</button>
<button hx-get="/suite/slides/modals/animation" hx-vals='{{"action":"add","type":"zoom_in"}}' hx-target="#modal-container" hx-swap="innerHTML" style="padding:12px;background:#0f172a;border:1px solid #334155;border-radius:6px;color:#f8fafc;cursor:pointer;font-size:12px;text-align:center;">Zoom In</button>
<button hx-get="/suite/slides/modals/animation" hx-vals='{{"action":"add","type":"bounce"}}' hx-target="#modal-container" hx-swap="innerHTML" style="padding:12px;background:#0f172a;border:1px solid #334155;border-radius:6px;color:#f8fafc;cursor:pointer;font-size:12px;text-align:center;">Bounce</button>
<button hx-get="/suite/slides/modals/animation" hx-vals='{{"action":"add","type":"spin"}}' hx-target="#modal-container" hx-swap="innerHTML" style="padding:12px;background:#0f172a;border:1px solid #334155;border-radius:6px;color:#f8fafc;cursor:pointer;font-size:12px;text-align:center;">Spin</button>
</div>
</div>"##
    ))
}

pub async fn handle_media_panel(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let pid = payload.get("presentation_id").and_then(|v| v.as_str()).unwrap_or("");
    let media: Vec<MediaElement> = serde_json::from_value(payload.get("media").cloned().unwrap_or(serde_json::Value::Null)).unwrap_or_default();
    if media.is_empty() {
        return Html(empty_fragment("Nenhuma mídia. Use o menu Inserir para adicionar imagens, áudio ou vídeo."));
    }
    let mut html = String::from(format!(
        r##"<div class="sl-panel" id="media-panel" style="padding:16px;">
<div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:12px;">
<h3 style="margin:0;color:#f8fafc;font-size:16px;">Mídia ({count})</h3>
<button hx-get="/suite/slides/modals/add-image" hx-vals='{{"presentation_id":"{pid}"}}' hx-target="#modal-container" hx-swap="innerHTML" style="background:#3b82f6;color:white;border:none;padding:4px 10px;border-radius:4px;cursor:pointer;font-size:12px;">+ Adicionar</button>
</div>
<div style="display:grid;grid-template-columns:repeat(auto-fill,minmax(120px,1fr));gap:8px;">"##,
        count = media.len(),
        pid = html_escape(pid)
    ));
    for m in &media {
        let icon = match m.media_type.as_str() {
            "image" => "🖼️",
            "video" => "🎬",
            "audio" => "🔊",
            _ => "📁",
        };
        html.push_str(&format!(
            r##"<div class="sl-media-item" style="padding:8px;background:#0f172a;border:1px solid #334155;border-radius:6px;text-align:center;">
<div style="font-size:32px;">{icon}</div>
<div style="font-size:11px;color:#cbd5e1;margin-top:4px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;">{name}</div>
<div style="font-size:10px;color:#94a3b8;">{w}×{h}</div>
</div>"##,
            icon = icon,
            name = html_escape(m.src.split('/').last().unwrap_or("media")),
            w = m.width as i32,
            h = m.height as i32
        ));
    }
    html.push_str("</div></div>");
    Html(html)
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
<span style="color:#94a3b8;font-size:12px;">Slide {cur}/{total}</span>
</div>
<div style="display:flex;gap:8px;margin-bottom:12px;">
<button hx-post="/api/slides/presenter/start" hx-vals='{{"presentation_id":"{pid}"}}' style="flex:1;background:#10b981;color:white;border:none;padding:10px;border-radius:4px;cursor:pointer;font-weight:600;">▶ Iniciar</button>
<button hx-post="/api/slides/presenter/end" hx-vals='{{"presentation_id":"{pid}"}}' style="flex:1;background:#dc2626;color:white;border:none;padding:10px;border-radius:4px;cursor:pointer;font-weight:600;">⏹ Encerrar</button>
</div>
<div style="padding:8px;background:#1e293b;border-radius:4px;">
<div style="font-size:11px;color:#94a3b8;margin-bottom:4px;">Notas do apresentador</div>
<textarea hx-post="/api/slides/presenter/update" hx-trigger="blur changed" hx-vals='{{"presentation_id":"{pid}","slide_index":"{cur}"}}' hx-swap="none" style="width:100%;min-height:80px;padding:6px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;font-family:inherit;resize:vertical;">{notes}</textarea>
</div>
</div>"##,
        pid = html_escape(pid),
        cur = current + 1,
        total = total,
        notes = html_escape(notes)
    ))
}

pub async fn handle_collaborators_panel(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let collabs: Vec<serde_json::Value> = serde_json::from_value(payload.get("collaborators").cloned().unwrap_or(serde_json::Value::Null)).unwrap_or_default();
    if collabs.is_empty() {
        return Html(empty_fragment("Nenhum colaborador conectado. Compartilhe a apresentação para começar a colaborar em tempo real."));
    }
    let mut html = String::from(
        r##"<div class="sl-panel" id="collab-panel" style="padding:12px;">
<h3 style="margin:0 0 8px 0;color:#f8fafc;font-size:14px;">Colaboradores ({count})</h3>"##
    );
    for c in &collabs {
        let name = c.get("name").and_then(|v| v.as_str()).unwrap_or("Anônimo");
        let color = c.get("color").and_then(|v| v.as_str()).unwrap_or("#3b82f6");
        let slide = c.get("current_slide").and_then(|v| v.as_u64()).unwrap_or(0);
        html.push_str(&format!(
            r##"<div class="sl-collab-item" style="display:flex;align-items:center;gap:8px;padding:6px;border-bottom:1px solid #334155;">
<div style="width:24px;height:24px;border-radius:50%;background:{color};display:flex;align-items:center;justify-content:center;font-size:12px;color:white;font-weight:600;">{initial}</div>
<div style="flex:1;font-size:12px;color:#cbd5e1;">{name}</div>
<div style="font-size:10px;color:#94a3b8;">Slide {slide}</div>
</div>"##,
            color = html_escape(color),
            initial = name.chars().next().unwrap_or('?'),
            name = html_escape(name),
            slide = slide + 1
        ));
    }
    html.push_str("</div>");
    Html(html.replace("{count}", &collabs.len().to_string()))
}

pub async fn handle_slide_sorter(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let pid = payload.get("presentation_id").and_then(|v| v.as_str()).unwrap_or("");
    let slides: Vec<Slide> = serde_json::from_value(payload.get("slides").cloned().unwrap_or(serde_json::Value::Null)).unwrap_or_default();
    if slides.is_empty() {
        return Html(empty_fragment("Nenhum slide para ordenar."));
    }
    let mut html = String::from(format!(
        r##"<div class="sl-panel" id="slide-sorter-panel" style="padding:16px;">
<h3 style="margin:0 0 12px 0;color:#f8fafc;font-size:16px;">Organizar Slides</h3>
<div style="display:grid;grid-template-columns:repeat(auto-fill,minmax(160px,1fr));gap:8px;">"##
    ));
    for (i, slide) in slides.iter().enumerate() {
        let bg = slide.background.color.clone().unwrap_or_else(|| "#0f172a".to_string());
        let text = slide.elements.iter()
            .find_map(|e| e.content.text.clone())
            .unwrap_or_default();
        let truncated = text.chars().take(60).collect::<String>();
        html.push_str(&format!(
            r##"<div class="sl-sorter-item" draggable="true" data-index="{i}" style="padding:8px;background:{bg};border:1px solid #334155;border-radius:6px;cursor:grab;">
<div style="aspect-ratio:16/9;display:flex;align-items:center;justify-content:center;color:#94a3b8;font-size:10px;padding:6px;text-align:center;overflow:hidden;border-radius:4px;background:rgba(0,0,0,0.3);">{truncated}</div>
<div style="font-size:11px;color:#cbd5e1;margin-top:4px;display:flex;justify-content:space-between;">
<span>Slide {n}</span>
<span>↕</span>
</div>
</div>"##,
            i = i,
            bg = html_escape(&bg),
            truncated = html_escape(&truncated),
            n = i + 1
        ));
    }
    html.push_str(&format!(
        r##"</div>
<button hx-post="/api/slides/slide/reorder" hx-vals='{{"presentation_id":"{pid}","order":"0,1,2,3"}}' hx-target="#slide-sorter-panel" hx-swap="outerHTML" style="margin-top:12px;background:#10b981;color:white;border:none;padding:8px 16px;border-radius:4px;cursor:pointer;font-weight:600;">Salvar Ordem</button>
</div>"##,
        pid = html_escape(pid)
    ));
    Html(html)
}
