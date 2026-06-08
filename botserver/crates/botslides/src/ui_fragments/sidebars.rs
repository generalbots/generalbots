use super::{empty_fragment, html_escape, render_metadata_card};
use axum::{response::Html, Json};
use crate::types::{Presentation, PresentationMetadata, Slide};

pub async fn handle_presentation_list_sidebar(Json(items): Json<Vec<PresentationMetadata>>) -> Html<String> {
    if items.is_empty() {
        return Html(empty_fragment("Nenhuma apresentação. Crie uma nova para começar."));
    }
    let mut html = String::from(
        r##"<div class="sl-sidebar" id="presentation-list-sidebar">
<div class="sl-sidebar-header" style="display:flex;justify-content:space-between;align-items:center;padding:12px;border-bottom:1px solid #334155;">
<h3 style="margin:0;color:#f8fafc;font-size:14px;">Apresentações</h3>
<button hx-get="/api/slides/new" hx-target="#slides-content" hx-swap="innerHTML" style="background:#10b981;color:white;border:none;padding:4px 10px;border-radius:4px;cursor:pointer;font-size:12px;">+ Nova</button>
</div>
<div class="sl-sidebar-body" style="padding:8px;display:flex;flex-direction:column;gap:8px;max-height:calc(100vh - 200px);overflow-y:auto;">"##
    );
    for m in &items {
        html.push_str(&render_metadata_card(m));
    }
    html.push_str("</div></div>");
    Html(html)
}

pub async fn handle_search_sidebar(Json(items): Json<Vec<PresentationMetadata>>) -> Html<String> {
    if items.is_empty() {
        return Html(empty_fragment("Nenhum resultado para esta busca."));
    }
    let mut html = String::from(
        r##"<div class="sl-sidebar" id="search-sidebar">
<div class="sl-sidebar-header" style="padding:12px;border-bottom:1px solid #334155;">
<h3 style="margin:0;color:#f8fafc;font-size:14px;">Resultados da Busca</h3>
</div>
<div class="sl-sidebar-body" style="padding:8px;display:flex;flex-direction:column;gap:8px;">"##
    );
    for m in &items {
        html.push_str(&render_metadata_card(m));
    }
    html.push_str(&format!(
        r##"<div style="padding:8px;color:#94a3b8;font-size:12px;text-align:center;">{} apresentação(ões) encontrada(s)</div>"##,
        items.len()
    ));
    html.push_str("</div></div>");
    Html(html)
}

pub async fn handle_recent_sidebar(Json(items): Json<Vec<PresentationMetadata>>) -> Html<String> {
    let recent: Vec<&PresentationMetadata> = items.iter().take(5).collect();
    if recent.is_empty() {
        return Html(empty_fragment("Nenhuma apresentação recente."));
    }
    let mut html = String::from(
        r##"<div class="sl-sidebar" id="recent-sidebar">
<div class="sl-sidebar-header" style="padding:12px;border-bottom:1px solid #334155;">
<h3 style="margin:0;color:#f8fafc;font-size:14px;">Recentes</h3>
</div>
<div class="sl-sidebar-body" style="padding:8px;display:flex;flex-direction:column;gap:8px;">"##
    );
    for m in recent {
        html.push_str(&render_metadata_card(m));
    }
    html.push_str("</div></div>");
    Html(html)
}

pub async fn handle_presentation_view(Json(pres): Json<Presentation>) -> Html<String> {
    let total = pres.slides.len();
    let current = 0;
    let bg = pres.theme.colors.background.clone();
    let text = pres.theme.colors.text.clone();
    let primary = pres.theme.colors.primary.clone();
    let current_slide = pres.slides.first();
    let slide_html = current_slide.map(render_slide).unwrap_or_default();
    Html(format!(
        r##"<div class="sl-view" id="slides-view" data-presentation-id="{id}" data-current-slide="0">
<div class="sl-view-header" style="padding:12px;border-bottom:1px solid #334155;display:flex;justify-content:space-between;align-items:center;background:#0f172a;">
<div>
<h2 style="margin:0;color:#f8fafc;">{name}</h2>
<div style="font-size:12px;color:#94a3b8;margin-top:2px;">{total} slides • Tema: {theme}</div>
</div>
<div style="display:flex;gap:6px;">
<button hx-get="/suite/slides/modals/insert-element" hx-vals='{{"presentation_id":"{id}","slide_index":"0"}}' hx-target="#modal-container" hx-swap="innerHTML" style="background:#3b82f6;color:white;border:none;padding:6px 12px;border-radius:4px;cursor:pointer;font-size:12px;">+ Elemento</button>
<button hx-get="/suite/slides/modals/ai" hx-vals='{{"presentation_id":"{id}"}}' hx-target="#modal-container" hx-swap="innerHTML" style="background:#6366f1;color:white;border:none;padding:6px 12px;border-radius:4px;cursor:pointer;font-size:12px;">✨ IA</button>
<button hx-get="/suite/slides/modals/share" hx-vals='{{"presentation_id":"{id}"}}' hx-target="#modal-container" hx-swap="innerHTML" style="background:#10b981;color:white;border:none;padding:6px 12px;border-radius:4px;cursor:pointer;font-size:12px;">🔗 Compartilhar</button>
<button hx-post="/api/slides/presenter/start" hx-vals='{{"presentation_id":"{id}"}}' style="background:#f59e0b;color:white;border:none;padding:6px 12px;border-radius:4px;cursor:pointer;font-size:12px;">▶ Apresentar</button>
</div>
</div>
<div class="sl-canvas-container" id="slides-content" style="background:#1e293b;padding:24px;display:flex;align-items:center;justify-content:center;min-height:60vh;">
<div class="sl-canvas" id="slide-canvas" data-bg="{bg}" style="aspect-ratio:16/9;width:100%;max-width:960px;background:{bg};border:1px solid #334155;border-radius:8px;position:relative;overflow:hidden;box-shadow:0 8px 24px rgba(0,0,0,0.4);">{slide_html}</div>
</div>
<div class="sl-status-bar" style="padding:8px 12px;border-top:1px solid #334155;display:flex;justify-content:space-between;align-items:center;background:#0f172a;font-size:12px;color:#94a3b8;">
<span>Slide {cur} de {total}</span>
<span>Cor primária: <span style="color:{primary};">●</span> {primary}</span>
</div>
</div>"##,
        id = html_escape(&pres.id),
        name = html_escape(&pres.name),
        total = total,
        theme = html_escape(&pres.theme.name),
        cur = current + 1,
        bg = html_escape(&bg),
        primary = html_escape(&primary),
        slide_html = slide_html
    ))
}

pub async fn handle_slide_thumbnails(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let slide: Option<Slide> = serde_json::from_value(payload.get("slide").cloned().unwrap_or(serde_json::Value::Null)).ok();
    let pid = payload.get("presentation_id").and_then(|v| v.as_str()).unwrap_or("");
    let idx = payload.get("index").and_then(|v| v.as_u64()).unwrap_or(0);
    let slide_html = slide.as_ref().map(render_slide).unwrap_or_default();
    let bg = slide.as_ref()
        .and_then(|s| s.background.color.clone())
        .unwrap_or_else(|| "#0f172a".to_string());
    Html(format!(
        r##"<div class="sl-canvas-container" id="slides-content" data-current-slide="{idx}" style="background:#1e293b;padding:24px;display:flex;align-items:center;justify-content:center;min-height:60vh;">
<div class="sl-canvas" id="slide-canvas" data-bg="{bg}" style="aspect-ratio:16/9;width:100%;max-width:960px;background:{bg};border:1px solid #334155;border-radius:8px;position:relative;overflow:hidden;box-shadow:0 8px 24px rgba(0,0,0,0.4);">{slide_html}</div>
</div>
<input type="hidden" id="current-presentation" value="{pid}" />
<input type="hidden" id="current-slide-index" value="{idx}" />"##,
        idx = idx,
        bg = html_escape(&bg),
        slide_html = slide_html,
        pid = html_escape(pid)
    ))
}

fn render_slide(slide: &Slide) -> String {
    let mut html = String::new();
    for e in &slide.elements {
        let style = format!(
            "position:absolute;left:{x}px;top:{y}px;width:{w}px;height:{h}px;transform:rotate({r}deg);z-index:{z};",
            x = e.x as i32, y = e.y as i32, w = e.width as i32, h = e.height as i32,
            r = e.rotation as i32, z = e.z_index
        );
        let fill = e.style.fill.clone().unwrap_or_else(|| "transparent".to_string());
        let color = e.style.color.clone().unwrap_or_else(|| "#f8fafc".to_string());
        let font_size = e.style.font_size.unwrap_or(16.0) as i32;
        let font_family = e.style.font_family.clone().unwrap_or_else(|| "Arial".to_string());
        let text = e.content.text.clone().unwrap_or_default();
        let text_align = e.style.text_align.clone().unwrap_or_else(|| "left".to_string());
        html.push_str(&format!(
            r##"<div class="sl-element" data-id="{id}" data-type="{kind}" style="{style}background:{fill};color:{color};font-size:{fs}px;font-family:{ff};text-align:{ta};display:flex;align-items:center;padding:4px;">{text}</div>"##,
            id = html_escape(&e.id),
            kind = html_escape(&e.element_type),
            style = style,
            fill = html_escape(&fill),
            color = html_escape(&color),
            fs = font_size,
            ff = html_escape(&font_family),
            ta = html_escape(&text_align),
            text = html_escape(&text)
        ));
    }
    html
}
