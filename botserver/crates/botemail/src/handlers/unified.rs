use axum::{
    extract::{Path, State},
    response::IntoResponse,
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

fn account_color_index(idx: usize) -> &'static str {
    ACCOUNT_COLORS[idx % ACCOUNT_COLORS.len()]
}

#[cfg(feature = "mail")]
pub async fn list_unified_htmx(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let user_id = match extract_user_from_session() {
        Ok(id) => id,
        Err(_) => return axum::response::Html(r#"<div class="empty-state"><h3>Authentication required</h3><p>Please sign in to view your unified inbox</p></div>"#.to_string()),
    };

    let folder = params.get("folder").cloned().unwrap_or_else(|| "INBOX".to_string());
    let limit: i64 = params.get("limit")
        .and_then(|v| v.parse().ok())
        .unwrap_or(50);
    let offset: i64 = params.get("offset")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);

    let pool = state.pool.clone();
    let accounts_result = tokio::task::spawn_blocking(move || {
        let mut db_conn = pool.get().map_err(|e| format!("DB connection error: {e}"))?;
        diesel::sql_query(
            "SELECT id, email, display_name, is_primary FROM user_email_accounts WHERE user_id = $1 AND is_active = true ORDER BY is_primary DESC"
        )
            .bind::<diesel::sql_types::Uuid, _>(user_id)
            .load::<EmailAccountBasicRow>(&mut db_conn)
            .map_err(|e| format!("Failed to get accounts: {e}"))
    }).await;

    let accounts = match accounts_result {
        Ok(Ok(accs)) => accs,
        _ => return axum::response::Html(r#"<div class="empty-state"><h3>No email accounts</h3><p>Add an email account to get started</p></div>"#.to_string()),
    };

    if accounts.is_empty() {
        return axum::response::Html(r#"<div class="empty-state"><h3>No email accounts</h3><p>Add an email account to get started</p></div>"#.to_string());
    }

    let pool2 = state.pool.clone();
    let account_ids: Vec<uuid::Uuid> = accounts.iter().map(|a| a.id).collect();
    let account_map: std::collections::HashMap<uuid::Uuid, (String, String)> = accounts
        .iter()
        .enumerate()
        .map(|(i, a)| {
            let name = a.display_name.clone().unwrap_or_else(|| a.email.clone());
            (a.id, (a.email.clone(), account_color_index(i).to_string()))
        })
        .collect();

    let folder_clone = folder.clone();
    let messages_result = tokio::task::spawn_blocking(move || {
        let mut db_conn = pool2.get().map_err(|e| format!("DB connection error: {e}"))?;
        diesel::sql_query(
            "SELECT id, account_id, message_id_header, in_reply_to, subject, normalized_subject, from_address, to_addresses, body_text, body_html, has_attachments, folder, uid, flags, is_read, is_flagged, received_at, synced_at FROM email_messages WHERE account_id = ANY($1) AND folder = $2 ORDER BY received_at DESC LIMIT $3 OFFSET $4"
        )
            .bind::<diesel::sql_types::Array<diesel::sql_types::Uuid>, _>(&account_ids)
            .bind::<diesel::sql_types::Text, _>(&folder_clone)
            .bind::<diesel::sql_types::BigInt, _>(limit)
            .bind::<diesel::sql_types::BigInt, _>(offset)
            .load::<EmailMessageRow>(&mut db_conn)
            .map_err(|e| format!("Failed to query unified inbox: {e}"))
    }).await;

    let messages = match messages_result {
        Ok(Ok(msgs)) => msgs,
        _ => Vec::new(),
    };

    let mut html = String::new();
    use std::fmt::Write;
    for msg in &messages {
        let (ref email_addr, ref color) = account_map
            .get(&msg.account_id)
            .map(|v| (&v.0, &v.1))
            .unwrap_or(&("unknown".to_string(), "#6b7280".to_string()));
        let unread_class = if !msg.is_read { "unread" } else { "" };
        let flag_icon = if msg.is_flagged {
            r#"<span class="flag-icon" title="Flagged">&#9733;</span>"#
        } else {
            ""
        };
        let attach_icon = if msg.has_attachments {
            r#"<span class="attach-icon" title="Has attachments">&#128206;</span>"#
        } else {
            ""
        };
        let preview: String = msg.body_text.as_deref().unwrap_or("").chars().take(120).collect();
        let formatted_date = msg.received_at.format("%b %d, %Y %H:%M").to_string();
        let _ = write!(
            html,
            r##"<div class="mail-item {}" data-account-id="{}" hx-get="/api/ui/email/content/{}?account_id={}" hx-target="#mail-content" hx-swap="innerHTML"><div class="mail-header"><span class="account-dot" style="background: {};"></span><span class="mail-from">{}</span><span class="text-sm text-gray">{}</span>{}{}</div><div class="mail-subject">{}</div><div class="mail-preview">{}</div></div>"##,
            unread_class, msg.account_id, msg.id, msg.account_id,
            color, msg.from_address, formatted_date, flag_icon, attach_icon,
            msg.subject, preview
        );
    }

    if html.is_empty() {
        html = format!(
            r#"<div class="empty-state"><h3>No messages in {folder}</h3><p>This folder is empty across all your accounts</p></div>"#
        );
    }

    axum::response::Html(html)
}

#[cfg(not(feature = "mail"))]
pub async fn list_unified_htmx(
    State(_state): State<Arc<AppState>>,
    axum::extract::Query(_params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    axum::response::Html(r#"<div class="empty-state"><h3>Mail feature not enabled</h3></div>"#.to_string())
}

#[cfg(feature = "mail")]
pub async fn search_all_accounts(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let user_id = match extract_user_from_session() {
        Ok(id) => id,
        Err(_) => return axum::response::Html(r#"<div class="empty-state"><h3>Authentication required</h3></div>"#.to_string()),
    };

    let query = params.get("q").cloned().unwrap_or_default();
    if query.trim().is_empty() {
        return axum::response::Html(
            r#"<div class="empty-state"><p>Enter a search term to find emails across all accounts</p></div>"#.to_string(),
        );
    }

    let date_from = params.get("date_from").cloned();
    let date_to = params.get("date_to").cloned();
    let attachments_only = params.get("attachments_only")
        .and_then(|v| v.parse::<bool>().ok())
        .unwrap_or(false);
    let unread_only = params.get("unread_only")
        .and_then(|v| v.parse::<bool>().ok())
        .unwrap_or(false);
    let limit: i64 = params.get("limit")
        .and_then(|v| v.parse().ok())
        .unwrap_or(50);
    let offset: i64 = params.get("offset")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);

    let pool = state.pool.clone();
    let search_result = tokio::task::spawn_blocking(move || {
        let mut db_conn = pool.get().map_err(|e| format!("DB connection error: {e}"))?;

        let account_rows: Vec<EmailAccountBasicRow> = diesel::sql_query(
            "SELECT id, email, display_name, is_primary FROM user_email_accounts WHERE user_id = $1 AND is_active = true"
        )
            .bind::<diesel::sql_types::Uuid, _>(user_id)
            .load(&mut db_conn)
            .map_err(|e| format!("Failed to get accounts: {e}"))?;

        let account_ids: Vec<uuid::Uuid> = account_rows.iter().map(|a| a.id).collect();
        let account_map: std::collections::HashMap<uuid::Uuid, String> = account_rows
            .iter()
            .map(|a| (a.id, a.email.clone()))
            .collect();

        let search_pattern = format!("%{}%", query.to_lowercase());

        let mut sql = String::from(
            "SELECT id, account_id, message_id_header, in_reply_to, subject, normalized_subject, from_address, to_addresses, body_text, body_html, has_attachments, folder, uid, flags, is_read, is_flagged, received_at, synced_at FROM email_messages WHERE account_id = ANY($1) AND (LOWER(subject) LIKE $2 OR LOWER(from_address) LIKE $2 OR LOWER(body_text) LIKE $2)"
        );

        if attachments_only {
            sql.push_str(" AND has_attachments = true");
        }
        if unread_only {
            sql.push_str(" AND is_read = false");
        }
        if let Some(ref df) = date_from {
            if !df.is_empty() {
                sql.push_str(&format!(" AND received_at >= '{df}'"));
            }
        }
        if let Some(ref dt) = date_to {
            if !dt.is_empty() {
                sql.push_str(&format!(" AND received_at <= '{dt}'"));
            }
        }

        sql.push_str(" ORDER BY received_at DESC LIMIT $3 OFFSET $4");

        let messages: Vec<EmailMessageRow> = diesel::sql_query(&sql)
            .bind::<diesel::sql_types::Array<diesel::sql_types::Uuid>, _>(&account_ids)
            .bind::<diesel::sql_types::Text, _>(&search_pattern)
            .bind::<diesel::sql_types::BigInt, _>(limit)
            .bind::<diesel::sql_types::BigInt, _>(offset)
            .load(&mut db_conn)
            .map_err(|e| format!("Search query failed: {e}"))?;

        Ok::<_, String>((messages, account_map))
    }).await;

    let (messages, account_map) = match search_result {
        Ok(Ok((msgs, map))) => (msgs, map),
        Err(e) => {
            warn!("Search task error: {e}");
            return axum::response::Html(r#"<div class="empty-state error"><p>Search error occurred</p></div>"#.to_string());
        }
        Ok(Err(e)) => {
            warn!("Search query error: {e}");
            return axum::response::Html(r#"<div class="empty-state error"><p>Search error occurred</p></div>"#.to_string());
        }
    };

    if messages.is_empty() {
        return axum::response::Html(format!(
            r#"<div class="empty-state"><svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="11" cy="11" r="8"></circle><path d="m21 21-4.35-4.35"></path></svg><h3>No results for "{query}"</h3><p>Try different keywords or adjust filters.</p></div>"#,
        ));
    }

    let mut html = String::from(r#"<div class="search-results">"#);
    use std::fmt::Write;
    let _ = write!(
        html,
        r#"<div class="result-stats">Found {} results for "{}" across all accounts</div>"#,
        messages.len(), query
    );
    for msg in &messages {
        let account_email = account_map
            .get(&msg.account_id)
            .map(|s| s.as_str())
            .unwrap_or("unknown");
        let preview: String = msg.body_text.as_deref().unwrap_or("").chars().take(120).collect();
        let formatted_date = msg.received_at.format("%b %d, %Y %H:%M").to_string();
        let attach_icon = if msg.has_attachments {
            r#" <span title="Has attachments">&#128206;</span>"#
        } else {
            ""
        };
        let _ = write!(
            html,
            r##"<div class="mail-item" data-account-id="{}" hx-get="/api/ui/email/content/{}?account_id={}" hx-target="#mail-content" hx-swap="innerHTML"><div class="mail-header"><span class="mail-from">{} ({})</span><span class="text-sm text-gray">{}</span>{}</div><div class="mail-subject">{}</div><div class="mail-preview">{}</div></div>"##,
            msg.account_id, msg.id, msg.account_id,
            msg.from_address, account_email, formatted_date, attach_icon,
            msg.subject, preview
        );
    }
    html.push_str("</div>");
    axum::response::Html(html)
}

#[cfg(not(feature = "mail"))]
pub async fn search_all_accounts(
    State(_state): State<Arc<AppState>>,
    axum::extract::Query(_params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    axum::response::Html(r#"<div class="empty-state"><h3>Mail feature not enabled</h3></div>"#.to_string())
}

#[cfg(feature = "mail")]
pub async fn get_thread_htmx(
    State(state): State<Arc<AppState>>,
    Path(thread_subject): Path<String>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let user_id = match extract_user_from_session() {
        Ok(id) => id,
        Err(_) => return axum::response::Html(r#"<div class="empty-state"><h3>Authentication required</h3></div>"#.to_string()),
    };

    let limit: i64 = params.get("limit")
        .and_then(|v| v.parse().ok())
        .unwrap_or(50);
    let offset: i64 = params.get("offset")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);

    let pool = state.pool.clone();
    let thread_result = tokio::task::spawn_blocking(move || {
        let mut db_conn = pool.get().map_err(|e| format!("DB connection error: {e}"))?;

        let account_rows: Vec<EmailAccountBasicRow> = diesel::sql_query(
            "SELECT id, email, display_name, is_primary FROM user_email_accounts WHERE user_id = $1 AND is_active = true"
        )
            .bind::<diesel::sql_types::Uuid, _>(user_id)
            .load(&mut db_conn)
            .map_err(|e| format!("Failed to get accounts: {e}"))?;

        let account_ids: Vec<uuid::Uuid> = account_rows.iter().map(|a| a.id).collect();
        let account_map: std::collections::HashMap<uuid::Uuid, (String, String)> = account_rows
            .iter()
            .enumerate()
            .map(|(i, a)| {
                let name = a.display_name.clone().unwrap_or_else(|| a.email.clone());
                (a.id, (name, account_color_index(i).to_string()))
            })
            .collect();

        let messages: Vec<EmailMessageRow> = diesel::sql_query(
            "SELECT id, account_id, message_id_header, in_reply_to, subject, normalized_subject, from_address, to_addresses, body_text, body_html, has_attachments, folder, uid, flags, is_read, is_flagged, received_at, synced_at FROM email_messages WHERE account_id = ANY($1) AND normalized_subject = $2 ORDER BY received_at ASC LIMIT $3 OFFSET $4"
        )
            .bind::<diesel::sql_types::Array<diesel::sql_types::Uuid>, _>(&account_ids)
            .bind::<diesel::sql_types::Text, _>(&thread_subject)
            .bind::<diesel::sql_types::BigInt, _>(limit)
            .bind::<diesel::sql_types::BigInt, _>(offset)
            .load(&mut db_conn)
            .map_err(|e| format!("Thread query failed: {e}"))?;

        Ok::<_, String>((messages, account_map))
    }).await;

    let (messages, account_map) = match thread_result {
        Ok(Ok((msgs, map))) => (msgs, map),
        _ => return axum::response::Html(r#"<div class="empty-state"><h3>Thread not found</h3></div>"#.to_string()),
    };

    if messages.is_empty() {
        return axum::response::Html(r#"<div class="empty-state"><h3>No messages in this thread</h3></div>"#.to_string());
    }

    let mut html = String::from(r#"<div class="thread-view">"#);
    use std::fmt::Write;
    let _ = write!(
        html,
        r#"<div class="thread-header"><h3>{} messages in thread</h3><p class="text-sm text-gray">Subject: {}</p></div>"#,
        messages.len(), thread_subject
    );

    for msg in &messages {
        let (ref sender, ref color) = account_map
            .get(&msg.account_id)
            .map(|v| (&v.0, &v.1))
            .unwrap_or(&("Unknown".to_string(), "#6b7280".to_string()));
        let formatted_date = msg.received_at.format("%b %d, %Y at %H:%M").to_string();
        let body = msg.body_html
            .as_deref()
            .or(msg.body_text.as_deref())
            .unwrap_or("(no content)");
        let _ = write!(
            html,
            r##"<div class="thread-message" data-id="{}"><div class="thread-msg-header"><span class="account-dot" style="background: {};"></span><span class="thread-sender">{}</span><span class="thread-from text-sm text-gray">&lt;{}&gt;</span><span class="thread-date text-sm text-gray">{}</span></div><div class="thread-msg-body">{}</div></div>"##,
            msg.id, color, sender, msg.from_address, formatted_date, body
        );
    }
    html.push_str("</div>");
    axum::response::Html(html)
}

#[cfg(not(feature = "mail"))]
pub async fn get_thread_htmx(
    State(_state): State<Arc<AppState>>,
    Path(_thread_subject): Path<String>,
    axum::extract::Query(_params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    axum::response::Html(r#"<div class="empty-state"><h3>Mail feature not enabled</h3></div>"#.to_string())
}
