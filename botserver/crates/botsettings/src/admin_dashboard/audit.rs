use axum::{extract::State, response::Html};
use std::sync::Arc;

use botcore::shared::state::AppState;

pub async fn admin_audit(State(_state): State<Arc<AppState>>) -> Html<String> {
    let stack = botcore::shared::utils::get_stack_path();
    let log_path = std::path::PathBuf::from(format!("{}/conf/directory/audit-log.json", stack));

    let entries: Vec<serde_json::Value> = std::fs::read_to_string(&log_path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();

    if entries.is_empty() {
        return Html(
            r#"<div class="empty-state"><p>No audit events recorded yet.</p></div>"#.to_string(),
        );
    }

    let recent: Vec<&serde_json::Value> = entries.iter().rev().take(100).collect();

    let mut rows = String::new();
    for entry in recent {
        let timestamp = entry
            .get("timestamp")
            .and_then(|v| v.as_str())
            .unwrap_or("—")
            .to_string();
        let action = entry
            .get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("—")
            .to_string();
        let detail = entry
            .get("detail")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let actor = entry
            .get("actor")
            .and_then(|v| v.as_str())
            .unwrap_or("—")
            .to_string();

        let detail_display = if detail.is_empty() {
            "—".to_string()
        } else {
            detail
        };

        rows.push_str(&format!(
            r##"<tr>
    <td class="audit-time">{timestamp}</td>
    <td class="audit-actor">{actor}</td>
    <td><span class="audit-action">{action}</span></td>
    <td class="audit-detail">{detail}</td>
</tr>"##,
            timestamp = timestamp,
            actor = actor,
            action = action,
            detail = detail_display,
        ));
    }

    Html(format!(
        r##"<div class="audit-page">
    <div class="page-header">
        <h1>Audit Log</h1>
        <p class="subtitle">Recent organization security and configuration events</p>
    </div>
    <table class="audit-table">
        <thead>
            <tr>
                <th>Timestamp</th>
                <th>Actor</th>
                <th>Action</th>
                <th>Detail</th>
            </tr>
        </thead>
        <tbody>{rows}</tbody>
    </table>
</div>"##
    ))
}
