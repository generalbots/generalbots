use super::html_escape;
use axum::{response::Html, Json};

pub async fn handle_theme_picker(Json(_payload): Json<serde_json::Value>) -> Html<String> {
    let themes = vec![
        ("default", "Padrão", "#3b82f6", "#10b981", "#f8fafc"),
        ("corporate", "Corporativo", "#1e40af", "#0891b2", "#f1f5f9"),
        ("creative", "Criativo", "#ec4899", "#8b5cf6", "#fff7ed"),
        ("minimal", "Minimalista", "#475569", "#94a3b8", "#fafafa"),
        ("dark", "Escuro", "#0f172a", "#3b82f6", "#f8fafc"),
        ("warm", "Quente", "#dc2626", "#f59e0b", "#fef3c7"),
        ("nature", "Natureza", "#16a34a", "#65a30d", "#f0fdf4"),
        ("ocean", "Oceano", "#0369a1", "#0ea5e9", "#e0f2fe"),
        ("sunset", "Pôr do Sol", "#ea580c", "#facc15", "#fffbeb"),
        ("midnight", "Meia-noite", "#1e1b4b", "#6366f1", "#e0e7ff"),
    ];
    let mut html = String::from(
        r##"<div class="sl-panel" id="themes-picker" style="padding:16px;">
<h3 style="margin:0 0 12px 0;color:#f8fafc;font-size:16px;">Escolher Tema</h3>
<div style="display:grid;grid-template-columns:repeat(auto-fill,minmax(180px,1fr));gap:12px;">"##
    );
    for (id, name, primary, secondary, text) in themes {
        let preview = format!(
            "background:linear-gradient(135deg,{} 0%,{} 100%);color:{};",
            primary, secondary, text
        );
        html.push_str(&format!(
            r##"<button hx-post="/api/slides/theme" hx-vals='{{"theme_name":"{id}"}}' hx-target="#theme-panel" hx-swap="outerHTML" style="padding:0;background:#0f172a;border:1px solid #334155;border-radius:8px;cursor:pointer;overflow:hidden;text-align:left;">
<div style="{preview}padding:24px;text-align:center;font-weight:600;font-size:14px;">{name}</div>
<div style="padding:8px;font-size:11px;color:#94a3b8;">{name}</div>
</button>"##,
            id = id,
            name = html_escape(name),
            preview = preview
        ));
    }
    html.push_str("</div></div>");
    Html(html)
}
