use super::*;

pub async fn handle_comments_panel(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let comments: Vec<DocumentComment> = serde_json::from_value(
        payload
            .get("comments")
            .cloned()
            .unwrap_or(serde_json::Value::Null),
    )
    .unwrap_or_default();
    if comments.is_empty() {
        return Html(format!(
            r##"<div class="dc-panel" id="comments-panel">{}</div>"##,
            empty_fragment("No comments. Add annotations to the selected text to start reviewing.")
        ));
    }
    let mut body = String::new();
    for c in &comments {
        let resolved = if c.resolved {
            r##"<span style="color:#10b981;font-size:11px;background:#064e3b;padding:2px 6px;border-radius:3px;">Resolvido</span>"##
        } else {
            r##"<span style="color:#f59e0b;font-size:11px;background:#78350f;padding:2px 6px;border-radius:3px;">Aberto</span>"##
        };
        body.push_str(&format!(
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
    Html(panel_shell_with_body_style(
        "dc",
        "comments-panel",
        "Comentários",
        "padding:8px;max-height:400px;overflow-y:auto;",
        &body,
    ))
}

pub async fn handle_track_changes_panel(
    State(state): State<Arc<DocState>>,
    user: DocUser,
    Json(payload): Json<serde_json::Value>,
) -> Html<String> {
    let mut changes: Vec<TrackChange> = serde_json::from_value(
        payload
            .get("track_changes")
            .cloned()
            .unwrap_or(serde_json::Value::Null),
    )
    .unwrap_or_default();
    if changes.is_empty() {
        if let Some(doc) = panel_document(&state, user.id(), &payload).await {
            changes = doc.track_changes.unwrap_or_default();
        }
    }
    let doc_id = payload
        .get("id")
        .or_else(|| payload.get("doc_id"))
        .and_then(|value| value.as_str())
        .unwrap_or("");
    if changes.is_empty() {
        return Html(empty_fragment(
            "No tracked changes. Enable track changes in the Review menu.",
        ));
    }
    let mut html = String::from(
        r##"<div class="dc-panel" id="tc-panel">
<div class="dc-panel-header" style="padding:12px;border-bottom:1px solid #334155;display:flex;justify-content:space-between;align-items:center;">
<h3 style="margin:0;color:#f8fafc;font-size:16px;">Controle de Alterações</h3>
<div>
<button hx-post="/api/docs/track-changes/accept-reject-all" hx-vals='{"doc_id":"{id}","accept":true}' hx-swap="none" hx-on::after-request="if(event.detail.successful) GBDocsRefreshTrackChanges()" style="background:#10b981;color:white;border:none;padding:4px 8px;border-radius:4px;cursor:pointer;font-size:11px;margin-right:4px;">✓ Aceitar Tudo</button>
<button hx-post="/api/docs/track-changes/accept-reject-all" hx-vals='{"doc_id":"{id}","accept":false}' hx-swap="none" hx-on::after-request="if(event.detail.successful) GBDocsRefreshTrackChanges()" style="background:#dc2626;color:white;border:none;padding:4px 8px;border-radius:4px;cursor:pointer;font-size:11px;">✗ Rejeitar Tudo</button>
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
<button hx-post="/api/docs/track-changes/accept-reject" hx-vals='{{"doc_id":"{doc_id}","change_id":"{cid}","accept":true}}' hx-swap="none" hx-on::after-request="if(event.detail.successful) GBDocsRefreshTrackChanges()" style="background:#10b981;color:white;border:none;padding:2px 6px;border-radius:3px;cursor:pointer;font-size:11px;margin-right:2px;">✓</button>
<button hx-post="/api/docs/track-changes/accept-reject" hx-vals='{{"doc_id":"{doc_id}","change_id":"{cid}","accept":false}}' hx-swap="none" hx-on::after-request="if(event.detail.successful) GBDocsRefreshTrackChanges()" style="background:#dc2626;color:white;border:none;padding:2px 6px;border-radius:3px;cursor:pointer;font-size:11px;">✗</button>
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
