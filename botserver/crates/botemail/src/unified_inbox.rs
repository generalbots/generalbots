use axum::{
    extract::State,
    response::IntoResponse,
    Json,
};
use diesel::prelude::*;
use log::warn;
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

fn normalize_subject(subj: &str) -> String {
    let s = subj.trim();
    for prefix in &["Re: ", "Re:", "Fwd: ", "Fwd:", "FW: ", "FW:"] {
        if let Some(rest) = s.strip_prefix(prefix) {
            return rest.trim().to_string();
        }
    }
    s.to_string()
}

#[derive(QueryableByName)]
struct InboxStatsRow {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    total: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    unread: i64,
}

pub async fn list_unified_inbox(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<axum::response::Html<String>, UnifiedInboxError> {
    let user_id = extract_user_from_session()
        .map_err(|_| UnifiedInboxError("Authentication required".into()))?;

    let folder = params.get("folder").cloned().unwrap_or_else(|| "INBOX".into());
    let page: i64 = params.get("page").and_then(|v| v.parse().ok()).unwrap_or(1);
    let per_page: i64 = params.get("per_page").and_then(|v| v.parse().ok()).unwrap_or(50);
    let offset = (page - 1).max(0) * per_page;

    let pool = state.pool.clone();
    let accounts_result = tokio::task::spawn_blocking(move || {
        let mut db = pool.get().map_err(|e| format!("DB pool error: {e}"))?;
        diesel::sql_query(
            "SELECT id, email, display_name, is_primary FROM user_email_accounts \
             WHERE user_id = $1 AND is_active = true ORDER BY is_primary DESC",
        )
        .bind::<diesel::sql_types::Uuid, _>(user_id)
        .load::<EmailAccountBasicRow>(&mut db)
        .map_err(|e| format!("Account query failed: {e}"))
    })
    .await;

    let accounts = match accounts_result {
        Ok(Ok(accs)) if !accs.is_empty() => accs,
        _ => {
            return Ok(axum::response::Html(
                r#"<div class="empty-state"><h3>No email accounts</h3></div>"#.into(),
            ))
        }
    };

    let account_map: std::collections::HashMap<uuid::Uuid, (String, String)> = accounts
        .iter()
        .enumerate()
        .map(|(i, a)| {
            let label = a.display_name.clone().unwrap_or_else(|| a.email.clone());
            (a.id, (label, color_for_index(i).to_string()))
        })
        .collect();

    let account_ids: Vec<uuid::Uuid> = accounts.iter().map(|a| a.id).collect();

    let filter_account = params.get("account").and_then(|v| {
        if v == "all" {
            None
        } else {
            uuid::Uuid::parse_str(v).ok()
        }
    });

    let pool2 = state.pool.clone();
    let folder2 = folder.clone();
    let acc_ids2 = account_ids.clone();
    let messages_result = tokio::task::spawn_blocking(move || {
        let mut db = pool2.get().map_err(|e| format!("DB pool error: {e}"))?;
        let mut sql = String::from(
            "SELECT id, account_id, message_id_header, in_reply_to, subject, normalized_subject, \
             from_address, to_addresses, body_text, body_html, has_attachments, folder, uid, \
             flags, is_read, is_flagged, received_at, synced_at \
             FROM email_messages WHERE account_id = ANY($1) AND folder = $2",
        );
        if let Some(ref acc_id) = filter_account {
            sql.push_str(&format!(" AND account_id = '{acc_id}'"));
        }
        sql.push_str(" ORDER BY received_at DESC LIMIT $3 OFFSET $4");

        diesel::sql_query(&sql)
            .bind::<diesel::sql_types::Array<diesel::sql_types::Uuid>, _>(&acc_ids2)
            .bind::<diesel::sql_types::Text, _>(&folder2)
            .bind::<diesel::sql_types::BigInt, _>(per_page)
            .bind::<diesel::sql_types::BigInt, _>(offset)
            .load::<EmailMessageRow>(&mut db)
            .map_err(|e| format!("Query failed: {e}"))
    })
    .await;

    let messages = match messages_result {
        Ok(Ok(msgs)) => msgs,
        Ok(Err(e)) => {
            warn!("Unified inbox query error: {e}");
            return Err(UnifiedInboxError(format!("Query failed: {e}")));
        }
        Err(e) => return Err(UnifiedInboxError(format!("Task error: {e}"))),
    };

    let default_color = ("unknown".to_string(), "#6b7280".to_string());
    let mut seen = std::collections::HashSet::new();
    let mut html = String::from(r#"<div class="unified-inbox">"#);
    use std::fmt::Write;

    for msg in &messages {
        let norm = normalize_subject(&msg.subject);
        if !seen.insert(norm) {
            continue;
        }
        let (_, color) = account_map.get(&msg.account_id).unwrap_or(&default_color);
        let unread = if !msg.is_read { " unread" } else { "" };
        let flag = if msg.is_flagged {
            r#"<span class="flag-icon" title="Flagged">&#9733;</span>"#
        } else {
            ""
        };
        let attach = if msg.has_attachments {
            r#"<span class="attach-icon" title="Attachments">&#128206;</span>"#
        } else {
            ""
        };
        let preview: String = msg.body_text.as_deref().unwrap_or("").chars().take(120).collect();
        let date = msg.received_at.format("%b %d %H:%M").to_string();

        let _ = write!(
            html,
            r##"<div class="mail-item{unread}" data-account-id="{aid}" \
                hx-get="/api/ui/email/content/{id}?account_id={aid}" \
                hx-target="#mail-content" hx-swap="innerHTML">\
                <input type="checkbox" class="mail-item-checkbox" data-id="{id}" onclick="event.stopPropagation()" style="margin-right: 8px; cursor: pointer;" />\
                <div class="mail-header"><span class="account-dot" style="background:{color};"></span>\
                <span class="mail-from">{from}</span>\
                <span class="text-sm text-gray">{date}</span>{flag}{attach}</div>\
                <div class="mail-subject">{subj}</div>\
                <div class="mail-preview">{preview}</div></div>"##,
            unread = unread,
            aid = msg.account_id,
            id = msg.id,
            color = color,
            from = msg.from_address,
            date = date,
            subj = msg.subject,
            preview = preview,
        );
    }

    html.push_str("</div>");
    Ok(axum::response::Html(html))
}

pub async fn unified_stats(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, UnifiedInboxError> {
    let user_id = extract_user_from_session()
        .map_err(|_| UnifiedInboxError("Authentication required".into()))?;

    let pool = state.pool.clone();
    let stats_result = tokio::task::spawn_blocking(move || {
        let mut db = pool.get().map_err(|e| format!("DB pool error: {e}"))?;
        let row: InboxStatsRow = diesel::sql_query(
            "SELECT COUNT(*) as total, \
             COUNT(*) FILTER (WHERE is_read = false) as unread \
             FROM email_messages \
             WHERE account_id IN ( \
               SELECT id FROM user_email_accounts WHERE user_id = $1 AND is_active = true \
             )",
        )
        .bind::<diesel::sql_types::Uuid, _>(user_id)
        .get_result(&mut db)
        .map_err(|e| format!("Stats query failed: {e}"))?;
        Ok::<_, String>(row)
    })
    .await;

    let row = stats_result
        .map_err(|e| UnifiedInboxError(format!("Task error: {e}")))?
        .map_err(UnifiedInboxError)?;

    Ok(Json(serde_json::json!({
        "total": row.total,
        "unread": row.unread,
    })))
}

pub struct UnifiedInboxError(pub String);

impl IntoResponse for UnifiedInboxError {
    fn into_response(self) -> axum::response::Response {
        (axum::http::StatusCode::INTERNAL_SERVER_ERROR, self.0).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_subject_plain() {
        assert_eq!(normalize_subject("Hello World"), "Hello World");
    }

    #[test]
    fn test_normalize_subject_re_prefix() {
        assert_eq!(normalize_subject("Re: Hello World"), "Hello World");
        assert_eq!(normalize_subject("Re:Hello World"), "Hello World");
    }

    #[test]
    fn test_normalize_subject_fwd_prefix() {
        assert_eq!(normalize_subject("Fwd: Meeting Notes"), "Meeting Notes");
        assert_eq!(normalize_subject("FW: Meeting Notes"), "Meeting Notes");
    }

    #[test]
    fn test_normalize_subject_strips_whitespace() {
        assert_eq!(normalize_subject("  Re:  Hello  "), "Hello");
    }

    #[test]
    fn test_color_index_wraps() {
        assert_eq!(ACCOUNT_COLORS.len(), 8);
        assert_eq!(color_for_index(0), "#3b82f6");
        assert_eq!(color_for_index(8), "#3b82f6");
        assert_eq!(color_for_index(15), "#84cc16");
    }
}
