use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use diesel::prelude::*;
use std::sync::Arc;

use crate::models::{AppState, EmailAccountBasicRow, EmailMessageRow};
use crate::models::extract_user_from_session;

const ACCOUNT_COLORS: [&str; 8] = [
    "#3b82f6", "#ef4444", "#22c55e", "#f59e0b",
    "#8b5cf6", "#ec4899", "#06b6d4", "#84cc16",
];

fn color_for_index(idx: usize) -> &'static str {
    ACCOUNT_COLORS[idx % ACCOUNT_COLORS.len()]
}

pub async fn get_thread(
    State(state): State<Arc<AppState>>,
    Path(thread_subject): Path<String>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<axum::response::Html<String>, ThreadError> {
    let user_id = extract_user_from_session()
        .map_err(|_| ThreadError("Authentication required".into()))?;

    let limit: i64 = params.get("limit").and_then(|v| v.parse().ok()).unwrap_or(50);
    let offset: i64 = params.get("offset").and_then(|v| v.parse().ok()).unwrap_or(0);

    let pool = state.pool.clone();
    let subject_clone = thread_subject.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut db = pool.get().map_err(|e| format!("DB pool error: {e}"))?;

        let accounts: Vec<EmailAccountBasicRow> = diesel::sql_query(
            "SELECT id, email, display_name, is_primary \
             FROM user_email_accounts WHERE user_id = $1 AND is_active = true",
        )
        .bind::<diesel::sql_types::Uuid, _>(user_id)
        .load(&mut db)
        .map_err(|e| format!("Accounts query failed: {e}"))?;

        let account_map: std::collections::HashMap<uuid::Uuid, (String, String)> = accounts
            .iter()
            .enumerate()
            .map(|(i, a)| {
                let name = a.display_name.clone().unwrap_or_else(|| a.email.clone());
                (a.id, (name, color_for_index(i).to_string()))
            })
            .collect();

        let acc_ids: Vec<uuid::Uuid> = accounts.iter().map(|a| a.id).collect();

        let messages: Vec<EmailMessageRow> = diesel::sql_query(
            "SELECT id, account_id, message_id_header, in_reply_to, subject, normalized_subject, \
             from_address, to_addresses, body_text, body_html, has_attachments, folder, uid, \
             flags, is_read, is_flagged, received_at, synced_at \
             FROM email_messages \
             WHERE account_id = ANY($1) AND normalized_subject = $2 \
             ORDER BY received_at ASC LIMIT $3 OFFSET $4",
        )
        .bind::<diesel::sql_types::Array<diesel::sql_types::Uuid>, _>(&acc_ids)
        .bind::<diesel::sql_types::Text, _>(&subject_clone)
        .bind::<diesel::sql_types::BigInt, _>(limit)
        .bind::<diesel::sql_types::BigInt, _>(offset)
        .load(&mut db)
        .map_err(|e| format!("Thread query failed: {e}"))?;

        Ok::<_, String>((messages, account_map))
    })
    .await;

    let (messages, account_map) = result
        .map_err(|e| ThreadError(format!("Task error: {e}")))?
        .map_err(ThreadError)?;

    if messages.is_empty() {
        return Ok(axum::response::Html(
            r#"<div class="empty-state"><h3>No messages in this thread</h3></div>"#.into(),
        ));
    }

    let default_sender = ("Unknown".to_string(), "#6b7280".to_string());
    let msg_count = messages.len();
    let mut html = String::from(r#"<div class="thread-view">"#);
    use std::fmt::Write;
    let _ = write!(
        html,
        r#"<div class="thread-header"><h3>{} messages</h3>\
           <p class="text-sm text-gray">Subject: {}</p></div>"#,
        msg_count, thread_subject
    );

    for (i, msg) in messages.iter().enumerate() {
        let (sender, color) = account_map
            .get(&msg.account_id)
            .unwrap_or(&default_sender);
        let date = msg.received_at.format("%b %d, %Y %H:%M").to_string();
        let body = msg
            .body_html
            .as_deref()
            .or(msg.body_text.as_deref())
            .unwrap_or("(no content)");
        let expanded = if i == msg_count - 1 { "expanded" } else { "collapsed" };
        let preview: String = msg.body_text.as_deref().unwrap_or("").chars().take(100).collect();
        let toggle = if i == msg_count - 1 { "&#9660;" } else { "&#9654;" };

        let _ = write!(
            html,
            r##"<div class="thread-message {expanded}" data-id="{id}">\
                <div class="thread-msg-header" onclick="this.parentElement.classList.toggle('expanded')">\
                <span class="account-dot" style="background:{color};"></span>\
                <span class="thread-sender">{sender}</span>\
                <span class="thread-from text-sm text-gray">&lt;{from}&gt;</span>\
                <span class="thread-date text-sm text-gray">{date}</span>\
                <span class="thread-toggle">{toggle}</span></div>\
                <div class="thread-msg-preview">{preview}</div>\
                <div class="thread-msg-body">{body}</div></div>"##,
            id = msg.id,
            color = color,
            sender = sender,
            from = msg.from_address,
            date = date,
            toggle = toggle,
            preview = preview,
            body = body,
        );
    }

    html.push_str("</div>");
    Ok(axum::response::Html(html))
}

pub struct ThreadError(pub String);

impl IntoResponse for ThreadError {
    fn into_response(self) -> axum::response::Response {
        (axum::http::StatusCode::INTERNAL_SERVER_ERROR, self.0).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_consistency() {
        assert_eq!(color_for_index(0), color_for_index(8));
        assert_eq!(color_for_index(3), "#f59e0b");
    }
}
