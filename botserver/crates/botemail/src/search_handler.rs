use axum::{
    extract::State,
    response::IntoResponse,
};
use diesel::prelude::*;
use std::sync::Arc;

use crate::models::{AppState, EmailAccountBasicRow, EmailMessageRow};
use crate::models::extract_user_from_session;

pub async fn search_emails(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<axum::response::Html<String>, SearchError> {
    let query = params.get("q").cloned().unwrap_or_default();
    if query.trim().is_empty() {
        return Ok(axum::response::Html(
            r#"<div class="empty-state"><p>Enter a search term to find emails</p></div>"#.into(),
        ));
    }

    let user_id = extract_user_from_session(&headers)
        .map_err(|_| SearchError("Authentication required".into()))?;

    let account_filter = params.get("account").cloned();
    let folder_filter = params.get("folder").cloned();
    let date_from = params.get("date_from").cloned();
    let date_to = params.get("date_to").cloned();
    let has_attachment = params.get("has_attachment").and_then(|v| v.parse().ok());
    let unread_only = params.get("unread_only").and_then(|v| v.parse().ok());

    let pool = state.pool.clone();
    let query_clone = query.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut db = pool.get().map_err(|e| format!("DB pool error: {e}"))?;

        let accounts: Vec<EmailAccountBasicRow> = diesel::sql_query(
            "SELECT id, email, display_name, is_primary \
             FROM user_email_accounts WHERE user_id = $1 AND is_active = true",
        )
        .bind::<diesel::sql_types::Uuid, _>(user_id)
        .load(&mut db)
        .map_err(|e| format!("Accounts query failed: {e}"))?;

        let account_map: std::collections::HashMap<uuid::Uuid, String> = accounts
            .iter()
            .map(|a| (a.id, a.email.clone()))
            .collect();

        let mut acc_ids: Vec<uuid::Uuid> = accounts.iter().map(|a| a.id).collect();

        if let Some(ref acc_str) = account_filter {
            if let Ok(acc_id) = uuid::Uuid::parse_str(acc_str) {
                acc_ids.retain(|id| *id == acc_id);
            }
        }

        let search_pat = format!("%{}%", query_clone.to_lowercase());
        let mut sql = String::from(
            "SELECT id, account_id, message_id_header, in_reply_to, subject, normalized_subject, \
             from_address, to_addresses, body_text, body_html, has_attachments, folder, uid, \
             flags, is_read, is_flagged, received_at, synced_at \
             FROM email_messages \
             WHERE account_id = ANY($1) \
             AND (LOWER(subject) LIKE $2 OR LOWER(from_address) LIKE $2 \
                  OR LOWER(to_addresses) LIKE $2 OR LOWER(body_text) LIKE $2)",
        );

        if let Some(ref f) = folder_filter {
            if !f.is_empty() && f != "all" {
                sql.push_str(&format!(" AND folder = '{f}'"));
            }
        }
        if has_attachment.unwrap_or(false) {
            sql.push_str(" AND has_attachments = true");
        }
        if unread_only.unwrap_or(false) {
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

        sql.push_str(" ORDER BY received_at DESC LIMIT 50");

        let messages: Vec<EmailMessageRow> = diesel::sql_query(&sql)
            .bind::<diesel::sql_types::Array<diesel::sql_types::Uuid>, _>(&acc_ids)
            .bind::<diesel::sql_types::Text, _>(&search_pat)
            .load(&mut db)
            .map_err(|e| format!("Search query failed: {e}"))?;

        Ok::<_, String>((messages, account_map))
    })
    .await;

    let (messages, account_map) = result
        .map_err(|e| SearchError(format!("Task error: {e}")))?
        .map_err(SearchError)?;

    if messages.is_empty() {
        return Ok(axum::response::Html(format!(
            r#"<div class="empty-state"><svg width="48" height="48" viewBox="0 0 24 24" \
                fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="11" cy="11" r="8"/>\
                <path d="m21 21-4.35-4.35"/></svg>\
                <h3>No results for &quot;{query}&quot;</h3>\
                <p>Try different keywords or adjust filters.</p></div>"#,
        )));
    }

    let mut html = String::from(r#"<div class="search-results">"#);
    use std::fmt::Write;
    let _ = write!(
        html,
        r#"<div class="result-stats">Found {} results for &quot;{}&quot;</div>"#,
        messages.len(),
        query
    );

    for msg in &messages {
        let acct_email = account_map
            .get(&msg.account_id)
            .map(|s| s.as_str())
            .unwrap_or("?");
        let preview: String = msg.body_text.as_deref().unwrap_or("").chars().take(120).collect();
        let date = msg.received_at.format("%b %d, %Y %H:%M").to_string();
        let attach = if msg.has_attachments {
            r#" <span title="Has attachments">&#128206;</span>"#
        } else {
            ""
        };

        let _ = write!(
            html,
            r##"<div class="mail-item" data-account-id="{aid}" \
                hx-get="/api/ui/email/content/{id}?account_id={aid}" \
                hx-target="#mail-content" hx-swap="innerHTML">\
                <div class="mail-header"><span class="mail-from">{from}</span>\
                <span class="text-sm text-gray">{acct}</span>\
                <span class="text-sm text-gray">{date}</span>{attach}</div>\
                <div class="mail-subject">{subj}</div>\
                <div class="mail-preview">{preview}</div></div>"##,
            aid = msg.account_id,
            id = msg.id,
            from = msg.from_address,
            acct = acct_email,
            date = date,
            subj = msg.subject,
            preview = preview,
        );
    }

    html.push_str("</div>");
    Ok(axum::response::Html(html))
}

pub struct SearchError(pub String);

impl IntoResponse for SearchError {
    fn into_response(self) -> axum::response::Response {
        (axum::http::StatusCode::INTERNAL_SERVER_ERROR, self.0).into_response()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_search_pattern_generation() {
        let q = "hello world";
        let pat = format!("%{}%", q.to_lowercase());
        assert_eq!(pat, "%hello world%");
    }
}
