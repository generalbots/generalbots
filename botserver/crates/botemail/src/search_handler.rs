use axum::{
    extract::State,
    response::IntoResponse,
};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use std::sync::Arc;

use crate::models::{AppState, EmailAccountBasicRow, EmailMessageRow};
use crate::models::extract_user_from_session;

/// Well-known mail folders accepted by the search filter. User-defined
/// IMAP/Gmail folders are also accepted when they pass `is_safe_custom_folder`.
const KNOWN_FOLDERS: &[&str] = &[
    "inbox", "sent", "sent items", "sent mail", "drafts", "draft", "archive", "archived",
    "trash", "deleted", "deleted items", "spam", "junk", "junk email", "all mail", "important",
    "starred", "flagged", "outbox", "notes", "templates",
];

/// Validates the `folder` query parameter. `None` means "no folder filter",
/// which is also the result for absent/empty/`all` input and for any value that
/// fails the allowlist. The accepted value is always bound as a SQL parameter;
/// it is never interpolated into the query text.
fn validate_folder(raw: Option<&str>) -> Option<String> {
    let value = match raw {
        Some(v) => v.trim(),
        None => return None,
    };
    if value.is_empty() || value.eq_ignore_ascii_case("all") {
        return None;
    }
    if KNOWN_FOLDERS.contains(&value.to_ascii_lowercase().as_str()) {
        return Some(value.to_string());
    }
    if is_safe_custom_folder(value) {
        return Some(value.to_string());
    }
    None
}

/// Conservative character allowlist for user-defined folder names. Rejects
/// quotes, statement separators, comment markers and comparison operators so
/// that a value can never alter SQL structure, even if a bind were bypassed.
fn is_safe_custom_folder(value: &str) -> bool {
    if value.len() > 128 || value.contains("--") || value.contains("/*") || value.contains("*/") {
        return false;
    }
    value.chars().all(|c| {
        c.is_alphanumeric()
            || matches!(
                c,
                ' ' | '.' | '_' | '-' | '/' | '&' | '+' | '[' | ']' | '#' | '@' | ','
            )
    })
}

/// Parses a `date_from`/`date_to` query parameter with chrono. RFC3339
/// timestamps and date-only values (UTC midnight, for backward compatibility)
/// are accepted. `None` means "no date filter", including for malformed or
/// hostile input. Parsed values are bound as `Timestamptz` SQL parameters.
fn parse_date_filter(raw: Option<&str>) -> Option<DateTime<Utc>> {
    let value = match raw {
        Some(v) => v.trim(),
        None => return None,
    };
    if value.is_empty() {
        return None;
    }
    match chrono::DateTime::parse_from_rfc3339(value) {
        Ok(parsed) => Some(parsed.with_timezone(&Utc)),
        Err(_) => {
            let as_midnight = format!("{value}T00:00:00Z");
            match chrono::DateTime::parse_from_rfc3339(&as_midnight) {
                Ok(parsed) => Some(parsed.with_timezone(&Utc)),
                Err(_) => None,
            }
        }
    }
}

/// Builds the search query from fixed fragments only. Every user-supplied
/// filter is a bind placeholder ($3 folder, $4 date_from, $5 date_to), so no
/// caller input can ever change the SQL structure.
fn build_search_sql(has_attachment: bool, unread_only: bool) -> String {
    let mut sql = String::from(
        "SELECT id, account_id, message_id_header, in_reply_to, subject, normalized_subject, \
         from_address, to_addresses, body_text, body_html, has_attachments, folder, uid, \
         flags, is_read, is_flagged, received_at, synced_at \
         FROM email_messages \
         WHERE account_id = ANY($1) \
         AND (LOWER(subject) LIKE $2 OR LOWER(from_address) LIKE $2 \
              OR LOWER(to_addresses) LIKE $2 OR LOWER(body_text) LIKE $2) \
         AND ($3 IS NULL OR folder = $3) \
         AND ($4 IS NULL OR received_at >= $4) \
         AND ($5 IS NULL OR received_at <= $5)",
    );
    if has_attachment {
        sql.push_str(" AND has_attachments = true");
    }
    if unread_only {
        sql.push_str(" AND is_read = false");
    }
    sql.push_str(" ORDER BY received_at DESC LIMIT 50");
    sql
}

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
    let folder_filter = validate_folder(params.get("folder").map(String::as_str));
    let date_from_filter = parse_date_filter(params.get("date_from").map(String::as_str));
    let date_to_filter = parse_date_filter(params.get("date_to").map(String::as_str));
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
        let sql = build_search_sql(has_attachment.unwrap_or(false), unread_only.unwrap_or(false));

        let messages: Vec<EmailMessageRow> = diesel::sql_query(&sql)
            .bind::<diesel::sql_types::Array<diesel::sql_types::Uuid>, _>(&acc_ids)
            .bind::<diesel::sql_types::Text, _>(&search_pat)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(
                folder_filter.as_deref(),
            )
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>, _>(date_from_filter)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>, _>(date_to_filter)
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
    use super::*;

    #[test]
    fn test_search_pattern_generation() {
        let q = "hello world";
        let pat = format!("%{}%", q.to_lowercase());
        assert_eq!(pat, "%hello world%");
    }

    #[test]
    fn test_validate_folder_accepts_safe_values() {
        assert_eq!(validate_folder(Some("INBOX")), Some("INBOX".to_string()));
        assert_eq!(
            validate_folder(Some(" sent items ")),
            Some("sent items".to_string())
        );
        assert_eq!(
            validate_folder(Some("[Gmail]/Sent Mail")),
            Some("[Gmail]/Sent Mail".to_string())
        );
        assert_eq!(
            validate_folder(Some("My-Folder_2024")),
            Some("My-Folder_2024".to_string())
        );
        assert_eq!(validate_folder(Some("all")), None);
        assert_eq!(validate_folder(Some("  ")), None);
        assert_eq!(validate_folder(None), None);
    }

    #[test]
    fn test_validate_folder_rejects_injection_payloads() {
        let long = "A".repeat(200);
        for payload in [
            "' OR 1=1 --",
            "INBOX' UNION SELECT id, email FROM user_email_accounts --",
            "'; DROP TABLE email_messages; --",
            "INBOX' OR '1'='1",
            "INBOX--",
            "INBOX/*",
            "INBOX;SELECT 1",
            "INBOX\\'; --",
            "INBOX\nOR 1=1",
            long.as_str(),
        ] {
            assert_eq!(validate_folder(Some(payload)), None, "payload: {payload}");
        }
    }

    #[test]
    fn test_parse_date_filter_accepts_valid_values() {
        let from = parse_date_filter(Some("2024-01-15T10:30:00Z")).expect("rfc3339");
        assert_eq!(from.to_rfc3339(), "2024-01-15T10:30:00+00:00");
        let offset = parse_date_filter(Some("2024-01-15T10:30:00-03:00")).expect("offset");
        assert_eq!(offset.to_rfc3339(), "2024-01-15T13:30:00+00:00");
        let day = parse_date_filter(Some("2024-01-15")).expect("date only");
        assert_eq!(day.to_rfc3339(), "2024-01-15T00:00:00+00:00");
        assert!(parse_date_filter(None).is_none());
        assert!(parse_date_filter(Some("  ")).is_none());
    }

    #[test]
    fn test_parse_date_filter_rejects_injection_payloads() {
        for payload in [
            "' OR 1=1 --",
            "2024-01-01' OR '1'='1",
            "2024-01-01; DROP TABLE email_messages; --",
            "2024-01-01T00:00:00Z' UNION SELECT 1 --",
            "2024-01-01'/*",
            "not-a-date",
        ] {
            assert!(
                parse_date_filter(Some(payload)).is_none(),
                "payload: {payload}"
            );
        }
    }

    #[test]
    fn test_search_sql_has_no_embedded_filter_values() {
        // The query is assembled from fixed fragments; filter values are only
        // ever bound as $3/$4/$5, so no caller input can appear in the SQL.
        let sql = build_search_sql(false, false);
        assert!(sql.contains("account_id = ANY($1)"));
        assert!(sql.contains("LIKE $2"));
        assert!(sql.contains("folder = $3"));
        assert!(sql.contains("received_at >= $4"));
        assert!(sql.contains("received_at <= $5"));
        assert!(
            !sql.contains('\''),
            "SQL must not embed literal values: {sql}"
        );

        let flagged = build_search_sql(true, true);
        assert!(flagged.contains("has_attachments = true"));
        assert!(flagged.contains("is_read = false"));
    }
}
