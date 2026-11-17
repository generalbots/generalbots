use super::*;

pub async fn handle_footnotes_panel(
    State(state): State<Arc<DocState>>,
    user: DocUser,
    Json(payload): Json<serde_json::Value>,
) -> Html<String> {
    let mut notes: Vec<Footnote> = serde_json::from_value(
        payload
            .get("footnotes")
            .cloned()
            .unwrap_or(serde_json::Value::Null),
    )
    .unwrap_or_default();
    if notes.is_empty() {
        if let Some(doc) = panel_document(&state, user.id(), &payload).await {
            notes = doc.footnotes.unwrap_or_default();
        }
    }
    if notes.is_empty() {
        return Html(empty_fragment(
            "No footnotes. Use Insert > Footnote to add references.",
        ));
    }
    let mut body = String::new();
    for n in &notes {
        body.push_str(&format!(
            r##"<div class="dc-footnote-item" style="padding:8px;border-bottom:1px solid #334155;">
<div style="font-size:12px;color:#94a3b8;">[{mark}] pos {pos}</div>
<div style="font-size:13px;color:#cbd5e1;margin-top:2px;">{content}</div>
</div>"##,
            mark = html_escape(&n.reference_mark),
            pos = n.position,
            content = html_escape(&n.content)
        ));
    }
    Html(panel_shell(
        "dc",
        "footnotes-panel",
        "Notas de Rodapé",
        &body,
    ))
}

pub async fn handle_endnotes_panel(
    State(state): State<Arc<DocState>>,
    user: DocUser,
    Json(payload): Json<serde_json::Value>,
) -> Html<String> {
    let mut notes: Vec<Endnote> = serde_json::from_value(
        payload
            .get("endnotes")
            .cloned()
            .unwrap_or(serde_json::Value::Null),
    )
    .unwrap_or_default();
    if notes.is_empty() {
        if let Some(doc) = panel_document(&state, user.id(), &payload).await {
            notes = doc.endnotes.unwrap_or_default();
        }
    }
    if notes.is_empty() {
        return Html(empty_fragment(
            "No endnotes. Use Insert > Endnote to add references at the end of the document.",
        ));
    }
    let mut body = String::new();
    for n in &notes {
        body.push_str(&format!(
            r##"<div class="dc-endnote-item" style="padding:8px;border-bottom:1px solid #334155;">
<div style="font-size:12px;color:#94a3b8;">[{mark}] pos {pos}</div>
<div style="font-size:13px;color:#cbd5e1;margin-top:2px;">{content}</div>
</div>"##,
            mark = html_escape(&n.reference_mark),
            pos = n.position,
            content = html_escape(&n.content)
        ));
    }
    Html(panel_shell("dc", "endnotes-panel", "Notas Finais", &body))
}

pub async fn handle_styles_panel(
    State(state): State<Arc<DocState>>,
    user: DocUser,
    Json(payload): Json<serde_json::Value>,
) -> Html<String> {
    let mut styles: Vec<DocumentStyle> = serde_json::from_value(
        payload
            .get("styles")
            .cloned()
            .unwrap_or(serde_json::Value::Null),
    )
    .unwrap_or_default();
    if styles.is_empty() {
        if let Some(doc) = panel_document(&state, user.id(), &payload).await {
            styles = doc.styles.unwrap_or_default();
        }
    }
    if styles.is_empty() {
        return Html(format!(
            r##"<div class="dc-panel" id="styles-panel">{}</div>"##,
            empty_fragment(
                "No custom styles. Create styles based on Normal/Heading 1/Heading 2 to speed up formatting."
            )
        ));
    }
    let mut body = String::new();
    for s in &styles {
        let font_preview = format!(
            "{} {}pt",
            s.font_family.as_deref().unwrap_or("Arial"),
            s.font_size.unwrap_or(12.0) as i32
        );
        body.push_str(&format!(
            r##"<div class="dc-style-item" style="padding:8px;border-bottom:1px solid #334155;">
<div style="font-weight:600;color:#f8fafc;">{name}</div>
<div style="font-size:11px;color:#94a3b8;">Tipo: {kind} • Fonte: {font}</div>
</div>"##,
            name = html_escape(&s.name),
            kind = html_escape(&s.style_type),
            font = html_escape(&font_preview)
        ));
    }
    Html(panel_shell("dc", "styles-panel", "Estilos", &body))
}
