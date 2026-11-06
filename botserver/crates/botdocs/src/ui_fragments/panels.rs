use super::{empty_fragment, html_escape};
use crate::state::DocState;
use axum::extract::State;
use axum::{response::Html, Json};
use std::sync::Arc;
use crate::types_core::{DocumentComment, DocumentStyle, Endnote, Footnote, OutlineItem, TableOfContents, TocEntry, TrackChange};

pub async fn handle_toc_panel(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let toc: Option<TableOfContents> = serde_json::from_value(payload.get("toc").cloned().unwrap_or(serde_json::Value::Null)).ok();
    // The toolbar/sidebar buttons post only the live editor HTML, not a
    // persisted TOC: derive one from the headings so the panel has content.
    let toc = match toc {
        Some(toc) if !toc.entries.is_empty() => Some(toc),
        _ => {
            let items = payload
                .get("content")
                .and_then(|value| value.as_str())
                .map(heading_items)
                .unwrap_or_default();
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
<button class="btn-icon" hx-post="/api/docs/toc/update" hx-vals='{"id":"{id}"}' hx-target="#toc-panel" hx-swap="outerHTML" style="background:#3b82f6;color:white;border:none;padding:4px 10px;border-radius:4px;cursor:pointer;font-size:12px;">⟳ Atualizar</button>
</div>
<div class="dc-panel-body" style="padding:8px;">"##
            );
            for entry in &t.entries {
                let indent = (entry.level as usize) * 16;
                let page = entry.page_number.map(|p| format!("<span style=\"color:#94a3b8;font-size:11px;float:right;\">p. {p}</span>")).unwrap_or_default();
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
            Html(html)
        }
        _ => Html(empty_fragment("Nenhum sumário gerado. Use o botão Gerar Sumário para criar automaticamente a partir dos cabeçalhos.")),
    }
}

pub async fn handle_comments_panel(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let comments: Vec<DocumentComment> = serde_json::from_value(payload.get("comments").cloned().unwrap_or(serde_json::Value::Null)).unwrap_or_default();
    if comments.is_empty() {
        return Html(empty_fragment("Nenhum comentário. Adicione anotações ao texto selecionado para começar a revisar."));
    }
    let mut html = String::from(
        r##"<div class="dc-panel" id="comments-panel">
<div class="dc-panel-header" style="padding:12px;border-bottom:1px solid #334155;">
<h3 style="margin:0;color:#f8fafc;font-size:16px;">Comentários</h3>
</div>
<div class="dc-panel-body" style="padding:8px;max-height:400px;overflow-y:auto;">"##
    );
    for c in &comments {
        let resolved = if c.resolved { r##"<span style="color:#10b981;font-size:11px;background:#064e3b;padding:2px 6px;border-radius:3px;">Resolvido</span>"## } else { r##"<span style="color:#f59e0b;font-size:11px;background:#78350f;padding:2px 6px;border-radius:3px;">Aberto</span>"## };
        html.push_str(&format!(
            r##"<div class="dc-comment-item" style="padding:10px;border-bottom:1px solid #334155;border-left:3px solid #f59e0b;background:#1e293b50;margin-bottom:4px;border-radius:4px;">
<div style="display:flex;justify-content:space-between;align-items:center;">
<div style="font-weight:600;color:#f8fafc;font-size:13px;">{author}</div>
{resolved}
</div>
<div style="font-size:12px;color:#cbd5e1;margin-top:4px;">{content}</div>
<div style="font-size:10px;color:#64748b;margin-top:4px;">Posição {pos} • {created}</div>
</div>"##,
            author = html_escape(&c.author_name),
            content = html_escape(&c.content),
            pos = c.position,
            created = c.created_at.format("%d/%m %H:%M"),
            resolved = resolved
        ));
    }
    html.push_str("</div></div>");
    Html(html)
}

pub async fn handle_track_changes_panel(
    State(state): State<Arc<DocState>>,
    Json(payload): Json<serde_json::Value>,
) -> Html<String> {
    let mut changes: Vec<TrackChange> = serde_json::from_value(payload.get("track_changes").cloned().unwrap_or(serde_json::Value::Null)).unwrap_or_default();
    if changes.is_empty() {
        if let Some(doc) = panel_document(&state, &payload).await {
            changes = doc.track_changes.unwrap_or_default();
        }
    }
    let doc_id = payload
        .get("id")
        .or_else(|| payload.get("doc_id"))
        .and_then(|value| value.as_str())
        .unwrap_or("");
    if changes.is_empty() {
        return Html(empty_fragment("Nenhuma alteração rastreada. Ative o controle de alterações no menu Revisão."));
    }
    let mut html = String::from(
        r##"<div class="dc-panel" id="tc-panel">
<div class="dc-panel-header" style="padding:12px;border-bottom:1px solid #334155;display:flex;justify-content:space-between;align-items:center;">
<h3 style="margin:0;color:#f8fafc;font-size:16px;">Controle de Alterações</h3>
<div>
<button hx-post="/api/docs/track-changes/accept-reject-all" hx-vals='{"doc_id":"{id}","accept":true}' hx-swap="none" style="background:#10b981;color:white;border:none;padding:4px 8px;border-radius:4px;cursor:pointer;font-size:11px;margin-right:4px;">✓ Aceitar Tudo</button>
<button hx-post="/api/docs/track-changes/accept-reject-all" hx-vals='{"doc_id":"{id}","accept":false}' hx-swap="none" style="background:#dc2626;color:white;border:none;padding:4px 8px;border-radius:4px;cursor:pointer;font-size:11px;">✗ Rejeitar Tudo</button>
</div>
</div>
<div class="dc-panel-body" style="padding:8px;">"##
    )
    .replace("{id}", &html_escape(doc_id));
    for c in &changes {
        let icon = match c.change_type.as_str() {
            "insertion" => r##"<span style="color:#10b981;">+</span>"##,
            "deletion" => r##"<span style="color:#dc2626;">−</span>"##,
            _ => r##"<span style="color:#f59e0b;">~</span>"##,
        };
        let original = c.original_text.as_deref().unwrap_or("");
        let new = c.new_text.as_deref().unwrap_or("");
        html.push_str(&format!(
            r##"<div class="dc-tc-item" style="padding:8px;border-bottom:1px solid #334155;background:#0f172a;margin-bottom:4px;border-radius:4px;">
<div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:4px;">
<div style="font-size:12px;color:#94a3b8;">{icon} <strong>{author}</strong> • {change_type} • pos {pos}</div>
<div>
<button hx-post="/api/docs/track-changes/accept-reject" hx-vals='{{"doc_id":"{doc_id}","change_id":"{cid}","accept":true}}' hx-swap="none" style="background:#10b981;color:white;border:none;padding:2px 6px;border-radius:3px;cursor:pointer;font-size:11px;margin-right:2px;">✓</button>
<button hx-post="/api/docs/track-changes/accept-reject" hx-vals='{{"doc_id":"{doc_id}","change_id":"{cid}","accept":false}}' hx-swap="none" style="background:#dc2626;color:white;border:none;padding:2px 6px;border-radius:3px;cursor:pointer;font-size:11px;">✗</button>
</div>
</div>
<div style="font-size:12px;">
{original_label} <span style="color:#dc2626;text-decoration:line-through;">{original}</span><br/>
{new_label} <span style="color:#10b981;">{new}</span>
</div>
</div>"##,
            icon = icon,
            author = html_escape(&c.author_name),
            change_type = html_escape(&c.change_type),
            pos = c.position,
            cid = c.id,
            doc_id = html_escape(doc_id),
            original_label = if original.is_empty() { "" } else { "Original:" },
            new_label = if new.is_empty() { "" } else { "Novo:" },
            original = html_escape(original),
            new = html_escape(new)
        ));
    }
    html.push_str("</div></div>");
    Html(html)
}

pub async fn handle_footnotes_panel(
    State(state): State<Arc<DocState>>,
    Json(payload): Json<serde_json::Value>,
) -> Html<String> {
    let mut notes: Vec<Footnote> = serde_json::from_value(payload.get("footnotes").cloned().unwrap_or(serde_json::Value::Null)).unwrap_or_default();
    if notes.is_empty() {
        if let Some(doc) = panel_document(&state, &payload).await {
            notes = doc.footnotes.unwrap_or_default();
        }
    }
    if notes.is_empty() {
        return Html(empty_fragment("Nenhuma nota de rodapé. Use Inserir > Nota de Rodapé para adicionar referências."));
    }
    let mut html = String::from(
        r##"<div class="dc-panel" id="footnotes-panel">
<div class="dc-panel-header" style="padding:12px;border-bottom:1px solid #334155;">
<h3 style="margin:0;color:#f8fafc;font-size:16px;">Notas de Rodapé</h3>
</div>
<div class="dc-panel-body" style="padding:8px;">"##
    );
    for n in &notes {
        html.push_str(&format!(
            r##"<div class="dc-footnote-item" style="padding:8px;border-bottom:1px solid #334155;">
<div style="font-size:12px;color:#94a3b8;">[{mark}] pos {pos}</div>
<div style="font-size:13px;color:#cbd5e1;margin-top:2px;">{content}</div>
</div>"##,
            mark = html_escape(&n.reference_mark),
            pos = n.position,
            content = html_escape(&n.content)
        ));
    }
    html.push_str("</div></div>");
    Html(html)
}

pub async fn handle_endnotes_panel(
    State(state): State<Arc<DocState>>,
    Json(payload): Json<serde_json::Value>,
) -> Html<String> {
    let mut notes: Vec<Endnote> = serde_json::from_value(payload.get("endnotes").cloned().unwrap_or(serde_json::Value::Null)).unwrap_or_default();
    if notes.is_empty() {
        if let Some(doc) = panel_document(&state, &payload).await {
            notes = doc.endnotes.unwrap_or_default();
        }
    }
    if notes.is_empty() {
        return Html(empty_fragment("Nenhuma nota final. Use Inserir > Nota Final para adicionar referências no fim do documento."));
    }
    let mut html = String::from(
        r##"<div class="dc-panel" id="endnotes-panel">
<div class="dc-panel-header" style="padding:12px;border-bottom:1px solid #334155;">
<h3 style="margin:0;color:#f8fafc;font-size:16px;">Notas Finais</h3>
</div>
<div class="dc-panel-body" style="padding:8px;">"##
    );
    for n in &notes {
        html.push_str(&format!(
            r##"<div class="dc-endnote-item" style="padding:8px;border-bottom:1px solid #334155;">
<div style="font-size:12px;color:#94a3b8;">[{mark}] pos {pos}</div>
<div style="font-size:13px;color:#cbd5e1;margin-top:2px;">{content}</div>
</div>"##,
            mark = html_escape(&n.reference_mark),
            pos = n.position,
            content = html_escape(&n.content)
        ));
    }
    html.push_str("</div></div>");
    Html(html)
}

pub async fn handle_styles_panel(
    State(state): State<Arc<DocState>>,
    Json(payload): Json<serde_json::Value>,
) -> Html<String> {
    let mut styles: Vec<DocumentStyle> = serde_json::from_value(payload.get("styles").cloned().unwrap_or(serde_json::Value::Null)).unwrap_or_default();
    if styles.is_empty() {
        if let Some(doc) = panel_document(&state, &payload).await {
            styles = doc.styles.unwrap_or_default();
        }
    }
    if styles.is_empty() {
        return Html(empty_fragment("Nenhum estilo personalizado. Crie estilos baseados em Normal/Título 1/Título 2 para acelerar a formatação."));
    }
    let mut html = String::from(
        r##"<div class="dc-panel" id="styles-panel">
<div class="dc-panel-header" style="padding:12px;border-bottom:1px solid #334155;">
<h3 style="margin:0;color:#f8fafc;font-size:16px;">Estilos</h3>
</div>
<div class="dc-panel-body" style="padding:8px;">"##
    );
    for s in &styles {
        let font_preview = format!("{} {}pt", s.font_family.as_deref().unwrap_or("Arial"), s.font_size.unwrap_or(12.0) as i32);
        html.push_str(&format!(
            r##"<div class="dc-style-item" style="padding:8px;border-bottom:1px solid #334155;">
<div style="font-weight:600;color:#f8fafc;">{name}</div>
<div style="font-size:11px;color:#94a3b8;">Tipo: {kind} • Fonte: {font}</div>
</div>"##,
            name = html_escape(&s.name),
            kind = html_escape(&s.style_type),
            font = html_escape(&font_preview)
        ));
    }
    html.push_str("</div></div>");
    Html(html)
}

pub async fn handle_outline_panel(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let mut outline: Vec<OutlineItem> = serde_json::from_value(payload.get("outline").cloned().unwrap_or(serde_json::Value::Null)).unwrap_or_default();
    if outline.is_empty() {
        outline = payload
            .get("content")
            .and_then(|value| value.as_str())
            .map(heading_items)
            .unwrap_or_default();
    }
    if outline.is_empty() {
        return Html(empty_fragment("Nenhum item de estrutura detectado. Use títulos (H1, H2, H3) para criar uma estrutura hierárquica."));
    }
    let mut html = String::from(
        r##"<div class="dc-panel" id="outline-panel">
<div class="dc-panel-header" style="padding:12px;border-bottom:1px solid #334155;">
<h3 style="margin:0;color:#f8fafc;font-size:16px;">Estrutura</h3>
</div>
<div class="dc-panel-body" style="padding:8px;">"##
    );
    for o in &outline {
        let indent = (o.level.max(1) as usize - 1) * 16;
        html.push_str(&format!(
            r##"<div class="dc-outline-item" style="padding:4px 4px 4px {indent}px;border-bottom:1px solid #33415520;">
<a href="#pos-{pos}" style="color:#3b82f6;text-decoration:none;font-size:13px;">{text}</a>
</div>"##,
            indent = indent,
            pos = o.position,
            text = html_escape(&o.text)
        ));
    }
    html.push_str("</div></div>");
    Html(html)
}

pub async fn handle_ai_panel(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let doc_id = payload
        .get("document_id")
        .or_else(|| payload.get("id"))
        .and_then(|value| value.as_str())
        .unwrap_or("");
    let action = payload.get("action").and_then(|v| v.as_str()).unwrap_or("improve");
    let prompt = payload.get("prompt").and_then(|v| v.as_str()).unwrap_or("");
    Html(format!(
        r##"<div class="dc-panel" id="ai-panel" style="padding:16px;background:#1e1b4b;border:1px solid #4338ca;border-radius:8px;">
<div style="display:flex;align-items:center;gap:8px;margin-bottom:12px;">
<span style="font-size:24px;">✨</span>
<h3 style="margin:0;color:#c7d2fe;font-size:16px;">Assistente IA — {action_label}</h3>
</div>
<form hx-post="/api/docs/ai/custom" hx-vals='{{"document_id":"{id}","action":"{action}","prompt":"{action}"}}' hx-target="#ai-result" hx-swap="innerHTML" style="display:flex;flex-direction:column;gap:8px;">
<textarea name="selected_text" placeholder="Cole o texto a processar aqui" rows="5" style="padding:8px;background:#0f172a;border:1px solid #4338ca;border-radius:4px;color:#f8fafc;font-family:inherit;resize:vertical;">{prompt}</textarea>
<button type="submit" style="background:#6366f1;color:white;border:none;padding:10px;border-radius:4px;cursor:pointer;font-weight:600;">{action_label}</button>
</form>
<div id="ai-result" style="margin-top:12px;padding:8px;background:#0f172a;border-radius:4px;min-height:80px;color:#cbd5e1;font-size:13px;"></div>
</div>"##,
        id = html_escape(doc_id),
        action = html_escape(action),
        action_label = match action {
            "summarize" => "Resumir",
            "expand" => "Expandir",
            "improve" => "Melhorar",
            "simplify" => "Simplificar",
            "translate" => "Traduzir",
            "custom" => "Comando personalizado",
            _ => "Processar",
        },
        prompt = html_escape(prompt)
    ))
}

/// Loads the stored document for a panel. Drive-opened ids (`drive-…`) and
/// the unsaved `current` session are not stored yet, so they stay empty.
async fn panel_document(
    state: &Arc<DocState>,
    payload: &serde_json::Value,
) -> Option<crate::types::Document> {
    let id = payload
        .get("id")
        .or_else(|| payload.get("doc_id"))
        .and_then(|value| value.as_str())?;
    if id.is_empty() || id == "current" || id.starts_with("drive-") {
        return None;
    }
    let user_id = crate::storage::get_current_user_id();
    crate::storage::load_document_from_drive(state, &user_id, id)
        .await
        .ok()
        .flatten()
}

/// Extracts H1–H6 headings from editor HTML (attributes allowed). The stored
/// outline API only matches exact `<hN>` tags and cannot see unsaved edits.
fn heading_items(content: &str) -> Vec<OutlineItem> {
    let mut items = Vec::new();
    for level in 1..=6u32 {
        let open = format!("<h{level}");
        let close = format!("</h{level}>");
        let mut position = 0usize;
        while let Some(start) = content[position..].find(&open) {
            let absolute = position + start;
            let Some(tag_end) = content[absolute..].find('>') else { break };
            let text_start = absolute + tag_end + 1;
            let Some(text_end) = content[text_start..].find(&close) else { break };
            let text = plain_text(&content[text_start..text_start + text_end]);
            if !text.is_empty() {
                items.push(OutlineItem {
                    id: uuid::Uuid::new_v4().to_string(),
                    text,
                    level,
                    position: absolute,
                    length: text_end,
                    style_name: format!("Heading {level}"),
                });
            }
            position = text_start + text_end + close.len();
        }
    }
    items.sort_by_key(|item| item.position);
    items
}

fn plain_text(html: &str) -> String {
    let mut output = String::with_capacity(html.len());
    let mut inside_tag = false;
    for character in html.chars() {
        match character {
            '<' => inside_tag = true,
            '>' => inside_tag = false,
            other if !inside_tag => output.push(other),
            _ => {}
        }
    }
    output.trim().to_string()
}
