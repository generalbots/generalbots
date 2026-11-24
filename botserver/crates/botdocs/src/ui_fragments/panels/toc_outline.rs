use super::*;

pub async fn handle_toc_panel(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let doc_id = payload
        .get("id")
        .or_else(|| payload.get("doc_id"))
        .and_then(|value| value.as_str())
        .unwrap_or("")
        .to_string();
    let toc: Option<TableOfContents> = serde_json::from_value(
        payload
            .get("toc")
            .cloned()
            .unwrap_or(serde_json::Value::Null),
    )
    .ok();
    let toc = match toc {
        Some(toc) if !toc.entries.is_empty() => Some(toc),
        _ => {
            let items = payload
                .get("content")
                .and_then(|value| value.as_str())
                .map(heading_items)
                .unwrap_or_else(Vec::new);
            if items.is_empty() {
                None
            } else {
                Some(TableOfContents {
                    id: uuid::Uuid::new_v4().to_string(),
                    title: "Sumário".to_string(),
                    entries: items
                        .iter()
                        .map(|item| TocEntry {
                            id: item.id.clone(),
                            text: item.text.clone(),
                            level: item.level,
                            page_number: None,
                            position: item.position,
                        })
                        .collect(),
                    max_level: 6,
                    show_page_numbers: false,
                    use_hyperlinks: false,
                })
            }
        }
    };
    match toc {
        Some(t) if !t.entries.is_empty() => {
            let mut html = String::from(
                r##"<div class="dc-panel" id="toc-panel">
<div class="dc-panel-header" style="display:flex;justify-content:space-between;align-items:center;padding:12px;border-bottom:1px solid #334155;">
<h3 style="margin:0;color:#f8fafc;font-size:16px;">Sumário</h3>
<button class="btn-icon" hx-post="/suite/docs/fragments/toc" hx-vals='{"id":"{id}"}' hx-target="#docs-panel-body" hx-swap="innerHTML" style="background:#3b82f6;color:white;border:none;padding:4px 10px;border-radius:4px;cursor:pointer;font-size:12px;">⟳ Atualizar</button>
</div>
<div class="dc-panel-body" style="padding:8px;">"##,
            );
            for entry in &t.entries {
                let indent = (entry.level as usize) * 16;
                let page = entry
                    .page_number
                    .map(|p| {
                        format!(
                            "<span style=\"color:#94a3b8;font-size:11px;float:right;\">p. {p}</span>"
                        )
                    })
                    .unwrap_or_else(String::new);
                let marker = if t.use_hyperlinks { r##"🔗 "## } else { "" };
                html.push_str(&format!(
                    r##"<div class="dc-toc-entry" style="padding:4px 4px 4px {indent}px;border-bottom:1px solid #33415520;">
<a href="#pos-{pos}" style="color:#3b82f6;text-decoration:none;font-size:13px;display:block;">{marker}{text}{page}</a>
</div>"##,
                    indent = indent,
                    pos = entry.position,
                    marker = marker,
                    text = html_escape(&entry.text),
                    page = page
                ));
            }
            html.push_str("</div></div>");
            Html(html.replace("{id}", &html_escape(&doc_id)))
        }
        _ => Html(empty_fragment(
            "No table of contents generated. Use the Generate TOC button to create one automatically from headings.",
        )),
    }
}

pub async fn handle_outline_panel(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let mut outline: Vec<OutlineItem> = serde_json::from_value(
        payload
            .get("outline")
            .cloned()
            .unwrap_or(serde_json::Value::Null),
    )
    .unwrap_or_else(|error| {
        log::warn!("docs: outline panel payload unreadable: {error}");
        Vec::new()
    });
    if outline.is_empty() {
        outline = payload
            .get("content")
            .and_then(|value| value.as_str())
            .map(heading_items)
            .unwrap_or_else(Vec::new);
    }
    if outline.is_empty() {
        return Html(empty_fragment(
            "No outline items detected. Use headings (H1, H2, H3) to create a hierarchical structure.",
        ));
    }
    let mut body = String::new();
    for o in &outline {
        let indent = (o.level.max(1) as usize - 1) * 16;
        body.push_str(&format!(
            r##"<div class="dc-outline-item" style="padding:4px 4px 4px {indent}px;border-bottom:1px solid #33415520;">
<a href="#pos-{pos}" style="color:#3b82f6;text-decoration:none;font-size:13px;">{text}</a>
</div>"##,
            indent = indent,
            pos = o.position,
            text = html_escape(&o.text)
        ));
    }
    Html(panel_shell("dc", "outline-panel", "Estrutura", &body))
}
