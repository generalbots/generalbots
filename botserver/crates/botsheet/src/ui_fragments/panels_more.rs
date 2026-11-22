use super::*;

pub async fn handle_comments_panel(
    headers: HeaderMap,
    Json(sheet): Json<Spreadsheet>,
) -> Html<String> {
    let lang = Lang::from_headers(&headers);
    let comments: Vec<(&str, &CellComment)> = sheet
        .worksheets
        .iter()
        .flat_map(|w| w.comments.as_ref().into_iter().flatten())
        .map(|(k, v)| (k.as_str(), v))
        .collect();
    if comments.is_empty() {
        return Html(empty_fragment(&t(lang, "panel.comments.empty")));
    }
    let mut body = String::new();
    for (cell, c) in comments {
        let resolved = if c.resolved {
            format!(
                r##"<span style="color:#10b981;font-size:11px;background:#064e3b;padding:2px 6px;border-radius:3px;">{}</span>"##,
                t(lang, "panel.comments.resolved")
            )
        } else {
            String::new()
        };
        body.push_str(&format!(
            r##"<div class="ss-comment-item" style="padding:8px;border-bottom:1px solid #334155;border-left:3px solid #f59e0b;background:#1e293b50;margin-bottom:4px;">
<div style="display:flex;justify-content:space-between;align-items:center;">
<div style="font-weight:600;color:#f8fafc;font-size:13px;">{cell} — {author}</div>
{resolved}
</div>
<div style="font-size:12px;color:#cbd5e1;margin-top:4px;">{content}</div>
<div style="font-size:10px;color:#64748b;margin-top:4px;">{created}</div>
</div>"##,
            cell = html_escape(cell),
            author = html_escape(&c.author_name),
            content = html_escape(&c.content),
            created = c.created_at.format("%d/%m/%Y %H:%M"),
            resolved = resolved
        ));
    }
    Html(panel_shell_with_body_style(
        "ss",
        "comments-panel",
        &t(lang, "panel.comments.title"),
        "padding:8px;max-height:400px;overflow-y:auto;",
        &body,
    ))
}
