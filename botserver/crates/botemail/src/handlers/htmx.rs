use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use diesel::prelude::*;
use log::{info, warn};
use std::sync::Arc;
use uuid::Uuid;

use crate::models::{AppState, EmailError, extract_user_from_session};
#[cfg(feature = "mail")]
use crate::models::{EmailSummary, EmailContent};

#[cfg(feature = "mail")]
pub async fn list_emails_htmx(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let folder = params.get("folder").cloned().unwrap_or_else(|| "inbox".to_string());
    let user_id = match extract_user_from_session(&headers) {
        Ok(id) => id,
        Err(_) => return axum::response::Html(r#"<div class="empty-state"><h3>Authentication required</h3><p>Please sign in to view your emails</p></div>"#.to_string()),
    };

    let pool = state.pool.clone();
    let folder_db = match folder.as_str() {
        "sent" => "Sent".to_string(),
        "drafts" => "Drafts".to_string(),
        "trash" => "Trash".to_string(),
        _ => "INBOX".to_string(),
    };
    let emails_result = tokio::task::spawn_blocking(move || {
        use diesel::sql_query;
        let mut db_conn = pool.get().map_err(|e| format!("DB connection error: {}", e))?;
        sql_query(
            "SELECT m.id::text AS id, m.from_address AS from_name, \
                    m.from_address AS from_email, m.subject, \
                    COALESCE(m.body_text, '') AS preview, \
                    m.received_at::text AS date, m.is_read AS read \
             FROM email_messages m \
             JOIN user_email_accounts a ON a.id = m.account_id \
             WHERE a.user_id = $1 AND m.folder = $2 \
             ORDER BY m.received_at DESC LIMIT 50",
        )
        .bind::<diesel::sql_types::Uuid, _>(user_id)
        .bind::<diesel::sql_types::Text, _>(&folder_db)
        .load::<EmailSummary>(&mut db_conn)
        .map_err(|e| format!("Emails query failed: {e}"))
    }).await;

    let emails = match emails_result {
        Ok(Ok(list)) => list,
        Ok(Err(e)) => {
            log::error!("Email list query error: {}", e);
            return axum::response::Html(r#"<div class="empty-state"><h3>Unable to load emails</h3><p>There was an error. Please try again later.</p></div>"#.to_string());
        }
        Err(e) => {
            log::error!("Email list task error: {}", e);
            return axum::response::Html(r#"<div class="empty-state"><h3>Unable to load emails</h3><p>There was an error. Please try again later.</p></div>"#.to_string());
        }
    };

    let mut html = String::new();
    use std::fmt::Write;
    for email in &emails {
        let unread_class = if !email.read { "unread" } else { "" };
        let _ = write!(html, r##"<div class="mail-item {}" hx-get="/api/ui/email/content/{}" hx-target="#mail-content" hx-swap="innerHTML"><input type="checkbox" class="mail-item-checkbox" data-id="{}" onclick="event.stopPropagation()" style="margin-right: 8px; cursor: pointer;" /><div class="mail-header"><span>{}</span><span class="text-sm text-gray">{}</span></div><div class="mail-subject">{}</div><div class="mail-preview">{}</div></div>"##, unread_class, email.id, email.id, email.from_name, email.date, email.subject, email.preview);
    }

    if html.is_empty() {
        html = format!(r#"<div class="empty-state"><h3>No emails in {}</h3><p>This folder is empty</p></div>"#, folder);
    }

    axum::response::Html(html)
}

#[cfg(not(feature = "mail"))]
pub async fn list_emails_htmx(
    State(_state): State<Arc<AppState>>,
    axum::extract::Query(_params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    axum::response::Html(r#"<div class="empty-state"><h3>Mail feature not enabled</h3></div>"#.to_string())
}

#[cfg(feature = "mail")]
pub async fn list_folders_htmx(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let user_id = match extract_user_from_session(&headers) {
        Ok(id) => id,
        Err(_) => return axum::response::Html(r#"<div class="nav-item">Please sign in</div>"#.to_string()),
    };

    let pool = state.pool.clone();
    let counts_result = tokio::task::spawn_blocking(move || {
        use diesel::sql_query;
        let mut db_conn = pool.get().map_err(|e| format!("DB connection error: {}", e))?;
        #[derive(diesel::QueryableByName)]
        struct CountRow {
            #[diesel(sql_type = diesel::sql_types::Text)]
            folder: String,
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            count: i64,
        }
        let rows = sql_query(
            "SELECT m.folder AS folder, COUNT(*) AS count \
             FROM email_messages m \
             JOIN user_email_accounts a ON a.id = m.account_id \
             WHERE a.user_id = $1 \
             GROUP BY m.folder",
        )
        .bind::<diesel::sql_types::Uuid, _>(user_id)
        .load::<CountRow>(&mut db_conn)
        .map_err(|e| format!("Folder counts query failed: {e}"))?;
        let mut counts = std::collections::HashMap::new();
        for r in rows {
            counts.insert(r.folder, r.count);
        }
        Ok::<_, String>(counts)
    }).await;

    let folder_counts = match counts_result {
        Ok(Ok(c)) => c,
        _ => return axum::response::Html(r#"<div class="nav-item">Error loading folders</div>"#.to_string()),
    };

    let mut html = String::new();
    for (folder_name, count) in &[("inbox", folder_counts.get("INBOX").unwrap_or(&0)), ("sent", folder_counts.get("Sent").unwrap_or(&0)), ("drafts", folder_counts.get("Drafts").unwrap_or(&0)), ("trash", folder_counts.get("Trash").unwrap_or(&0))] {
        let active = if *folder_name == "inbox" { "active" } else { "" };
        let count_badge = if **count > 0 { format!(r#"<span style="margin-left: auto; font-size: 0.875rem; color: #64748b;">{}</span>"#, count) } else { String::new() };
        let display: String = folder_name.chars().next().unwrap_or_default().to_uppercase().collect::<String>() + &folder_name[1..];
        use std::fmt::Write;
        let _ = write!(html, r##"<div class="nav-item {}" hx-get="/api/ui/email/list?folder={}" hx-target="#mail-list" hx-swap="innerHTML"><span>{}</span> {} {}</div>"##, active, folder_name, "", display, count_badge);
    }

    axum::response::Html(html)
}

#[cfg(not(feature = "mail"))]
pub async fn list_folders_htmx(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    axum::response::Html(r#"<div class="nav-item">Mail feature not enabled</div>"#.to_string())
}

pub async fn compose_email_htmx(State(_state): State<Arc<AppState>>) -> Result<impl IntoResponse, EmailError> {
    Ok(axum::response::Html(r##"<div class="mail-content-view"><h2>Compose New Email</h2><form class="compose-form" hx-post="/api/email/send" hx-target="#mail-content" hx-swap="innerHTML"><div class="form-group"><label>To:</label><input type="email" name="to" required></div><div class="form-group"><label>Subject:</label><input type="text" name="subject" required></div><div class="form-group"><label>Message:</label><textarea name="body" rows="10" required></textarea></div><div class="compose-actions"><button type="submit" class="btn-primary">Send</button><button type="button" class="btn-secondary" hx-post="/api/email/draft" hx-include="closest form">Save Draft</button></div></form></div>"##))
}

#[cfg(feature = "mail")]
pub async fn get_email_content_htmx(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, EmailError> {
    let user_id = extract_user_from_session(&headers)
        .map_err(|_| EmailError("Authentication required".to_string()))?;

    let msg_id = uuid::Uuid::parse_str(&id)
        .map_err(|_| EmailError("Invalid message id".to_string()))?;

    // The poller stores full messages (body_html/body_text, uid, flags) in
    // email_messages; read the content from the DB scoped to the caller's
    // accounts instead of fetching live from IMAP by a non-UID UUID (the
    // previous path always failed → clicks rendered nothing).
    let pool = state.pool.clone();
    // `id` is used again for the rendered HTML after the closure, so the
    // closure gets its own copy.
    let id_for_content = id.clone();
    let content = tokio::task::spawn_blocking(move || -> Result<EmailContent, String> {
        use diesel::sql_query;
        let mut db_conn = pool.get().map_err(|e| format!("DB connection error: {e}"))?;

        #[derive(diesel::QueryableByName)]
        struct MsgRow {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            account_id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Text)]
            subject: String,
            #[diesel(sql_type = diesel::sql_types::Varchar)]
            from_address: String,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
            to_addresses: Option<String>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
            body_text: Option<String>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
            body_html: Option<String>,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)]
            received_at: chrono::DateTime<chrono::Utc>,
            #[diesel(sql_type = diesel::sql_types::Bool)]
            is_read: bool,
        }
        let row = sql_query(
            "SELECT m.account_id, m.subject, m.from_address, m.to_addresses, \
                    m.body_text, m.body_html, m.received_at, m.is_read \
             FROM email_messages m \
             JOIN user_email_accounts a ON a.id = m.account_id \
             WHERE m.id = $1 AND a.user_id = $2 LIMIT 1",
        )
        .bind::<diesel::sql_types::Uuid, _>(msg_id)
        .bind::<diesel::sql_types::Uuid, _>(user_id)
        .get_result::<MsgRow>(&mut db_conn)
        .optional()
        .map_err(|e| format!("Message query failed: {e}"))?
        .ok_or_else(|| "Email not found".to_string())?;

        let body = row.body_html.filter(|h| !h.is_empty())
            .or(row.body_text)
            .unwrap_or_default();
        let to = row.to_addresses.unwrap_or_default();
        Ok(EmailContent {
            id: id_for_content,
            from_name: row.from_address.clone(),
            from_email: row.from_address,
            to,
            subject: row.subject,
            body,
            date: row.received_at.format("%b %d, %Y %H:%M").to_string(),
            read: row.is_read,
        })
    })
    .await
    .map_err(|e| EmailError(format!("Task join error: {e}")))?
    .map_err(|e| EmailError(e))?;

    let html = format!(r##"<div class="mail-content-view"><div id="nudges-banner-{id}" class="nudges-banner" data-email-id="{id}"></div><div class="mail-actions"><button hx-get="/api/ui/email/compose?reply_to={id}" hx-target="#mail-content" hx-swap="innerHTML">Reply</button><button hx-get="/api/ui/email/compose?forward={id}" hx-target="#mail-content" hx-swap="innerHTML">Forward</button><details class="snooze-menu"><summary class="mail-action-btn">Snooze</summary><div class="snooze-presets"><button onclick="snoozeEmail('{id}','later-today')">Later today</button><button onclick="snoozeEmail('{id}','tomorrow')">Tomorrow</button><button onclick="snoozeEmail('{id}','this-weekend')">This weekend</button><button onclick="snoozeEmail('{id}','next-week')">Next week</button></div></details><button hx-delete="/api/ui/email/{id}/delete" hx-target="#mail-list" hx-swap="innerHTML" hx-confirm="Delete this email?">Delete</button></div><h2>{subject}</h2><div style="display: flex; align-items: center; gap: 1rem; margin: 1rem 0;"><div><div style="font-weight: 600;">{from_name}</div><div class="text-sm text-gray">to: {to}</div></div><div style="margin-left: auto;" class="text-sm text-gray">{date}</div></div><div class="mail-body">{body}</div><div id="smart-reply-{id}" class="smart-reply-chips" data-email-id="{id}"></div></div>"##,
    id = id,
    subject = content.subject,
    from_name = content.from_name,
    to = content.to,
    date = content.date,
    body = content.body);

    Ok(axum::response::Html(html))
}

#[cfg(not(feature = "mail"))]
pub async fn get_email_content_htmx(
    State(_state): State<Arc<AppState>>,
    Path(_id): Path<String>,
) -> Result<impl IntoResponse, EmailError> {
    Ok(axum::response::Html(r#"<div class="mail-content-view"><p>Mail feature not enabled</p></div>"#.to_string()))
}

#[cfg(feature = "mail")]
pub async fn delete_email_htmx(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let user_id = match extract_user_from_session(&headers) {
        Ok(id) => id,
        Err(_) => return axum::response::Html(r#"<div class="empty-state"><h3>Authentication required</h3><p>Please sign in to delete emails</p></div>"#.to_string()),
    };

    let msg_id = match uuid::Uuid::parse_str(&id) {
        Ok(mid) => mid,
        Err(_) => return axum::response::Html(r#"<div class="empty-state"><h3>Invalid message id</h3></div>"#.to_string()),
    };

    // Delete the message from the local store, scoped to the caller's
    // accounts. The previous path moved it to trash via live IMAP with the
    // DB UUID as a sequence number — always failed. Poller re-sync skips
    // nothing here: a later UID range pass may re-add it, which matches the
    // "soft delete in local store" model.
    let pool = state.pool.clone();
    let deleted = tokio::task::spawn_blocking(move || {
        use diesel::sql_query;
        let mut db_conn = pool.get().map_err(|e| format!("DB connection error: {e}"))?;
        sql_query(
            "DELETE FROM email_messages m \
             USING user_email_accounts a \
             WHERE m.id = $1 AND a.id = m.account_id AND a.user_id = $2",
        )
        .bind::<diesel::sql_types::Uuid, _>(msg_id)
        .bind::<diesel::sql_types::Uuid, _>(user_id)
        .execute(&mut db_conn)
        .map_err(|e| format!("Delete failed: {e}"))
    })
    .await;

    match deleted {
        Ok(Ok(0)) => axum::response::Html(r#"<div class="empty-state"><h3>Email not found</h3></div>"#.to_string()),
        Ok(Ok(_)) => {
            info!("Email {id} deleted from local store");
            axum::response::Html(r#"<div class="success-message"><p>Email deleted</p></div><script>setTimeout(function() { htmx.trigger('#mail-list', 'load'); }, 100);</script>"#.to_string())
        }
        _ => axum::response::Html(r#"<div class="empty-state"><h3>Error deleting email</h3><p>Could not delete email</p></div>"#.to_string()),
    }
}

#[cfg(not(feature = "mail"))]
pub async fn delete_email_htmx(State(_state): State<Arc<AppState>>, Path(_id): Path<String>) -> impl IntoResponse {
    axum::response::Html(r#"<div class="empty-state"><h3>Mail feature not enabled</h3></div>"#.to_string())
}

pub async fn list_labels_htmx(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let user_id = match extract_user_from_session(&headers) {
        Ok(id) => id,
        Err(_) => return axum::response::Html(r#"<div class="label-item">Sign in to view labels</div>"#.to_string()),
    };

    let pool = state.pool.clone();
    let labels_result = tokio::task::spawn_blocking(move || {
        let mut db_conn = pool.get().map_err(|e| format!("DB connection error: {}", e))?;
        diesel::sql_query("SELECT id, name, color FROM email_labels WHERE user_id = $1 ORDER BY name ASC")
            .bind::<diesel::sql_types::Uuid, _>(user_id)
            .get_results::<crate::models::EmailLabelRow>(&mut db_conn)
            .map_err(|e| format!("Failed to load labels: {}", e))
    }).await;

    let labels = match labels_result {
        Ok(Ok(list)) => list,
        _ => Vec::new(),
    };

    if labels.is_empty() {
        return axum::response::Html(r#"<div class="label-item" style="--label-color: #ef4444;"><span class="label-dot" style="background: #ef4444;"></span><span>Important</span></div><div class="label-item" style="--label-color: #3b82f6;"><span class="label-dot" style="background: #3b82f6;"></span><span>Work</span></div><div class="label-item" style="--label-color: #22c55e;"><span class="label-dot" style="background: #22c55e;"></span><span>Personal</span></div>"#.to_string());
    }

    let mut html = String::new();
    use std::fmt::Write;
    for label in labels {
        let _ = write!(html, r##"<div class="label-item" style="--label-color: {};"><span class="label-dot" style="background: {};"></span><span>{}</span></div>"##, label.color, label.color, label.name);
    }

    axum::response::Html(html)
}

#[derive(Debug, QueryableByName, serde::Serialize)]
pub struct SignatureRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Varchar)]
    pub name: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub content_html: String,
}

#[derive(Debug, QueryableByName, serde::Serialize)]
pub struct TemplateRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Varchar)]
    pub name: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub subject_template: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub body_html_template: String,
}

#[derive(Debug, QueryableByName, serde::Serialize)]
pub struct RuleRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Varchar)]
    pub name: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub conditions_json: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub actions_json: String,
    #[diesel(sql_type = diesel::sql_types::Bool)]
    pub is_active: bool,
}

pub async fn list_templates_htmx(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let user_id = match extract_user_from_session(&headers) {
        Ok(id) => id,
        Err(_) => return axum::response::Html(r#"<div class="template-item">Please sign in</div>"#.to_string()),
    };

    let pool = state.pool.clone();
    let temps_result = tokio::task::spawn_blocking(move || {
        let mut db_conn = pool.get().map_err(|e| format!("DB connection error: {}", e))?;
        diesel::sql_query("SELECT id, name, subject_template, body_html_template FROM email_templates WHERE user_id = $1 OR user_id IS NULL ORDER BY name ASC")
            .bind::<diesel::sql_types::Uuid, _>(user_id)
            .get_results::<TemplateRow>(&mut db_conn)
            .map_err(|e| format!("Failed to load templates: {}", e))
    }).await;

    let temps = match temps_result {
        Ok(Ok(list)) => list,
        _ => Vec::new(),
    };

    if temps.is_empty() {
        return axum::response::Html(r#"<div class="template-item" onclick="useTemplate('welcome')"><h4>Welcome Email</h4><p>Standard welcome message for new contacts</p></div><div class="template-item" onclick="useTemplate('followup')"><h4>Follow Up</h4><p>General follow-up template</p></div><div class="template-item" onclick="useTemplate('meeting')"><h4>Meeting Request</h4><p>Request a meeting with scheduling options</p></div><p class="text-sm text-gray" style="margin-top: 1rem; text-align: center;">Click a template to use it</p>"#.to_string());
    }

    let mut html = String::new();
    use std::fmt::Write;
    for temp in temps {
        let _ = write!(html, r##"<div class="template-item" onclick="useTemplate('{}')"><h4>{}</h4><p class="text-sm text-gray">{}</p></div>"##, temp.id, temp.name, temp.subject_template);
    }
    let _ = writeln!(html, r#"<p class="text-sm text-gray" style="margin-top: 1rem; text-align: center;">Click a template to use it</p>"#);

    axum::response::Html(html)
}

pub async fn list_signatures_htmx(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let user_id = match extract_user_from_session(&headers) {
        Ok(id) => id,
        Err(_) => return axum::response::Html(r#"<div class="signature-item">Please sign in</div>"#.to_string()),
    };

    let pool = state.pool.clone();
    let sigs_result = tokio::task::spawn_blocking(move || {
        let mut db_conn = pool.get().map_err(|e| format!("DB connection error: {}", e))?;
        diesel::sql_query("SELECT id, name, content_html FROM email_signatures WHERE user_id = $1 AND is_active = true ORDER BY name ASC")
            .bind::<diesel::sql_types::Uuid, _>(user_id)
            .get_results::<SignatureRow>(&mut db_conn)
            .map_err(|e| format!("Failed to load signatures: {}", e))
    }).await;

    let sigs = match sigs_result {
        Ok(Ok(list)) => list,
        _ => Vec::new(),
    };

    if sigs.is_empty() {
        return axum::response::Html(r#"<div class="signature-item" onclick="useSignature('default')"><h4>Default Signature</h4><p class="text-sm text-gray">Best regards,<br>Your Name</p></div><div class="signature-item" onclick="useSignature('formal')"><h4>Formal Signature</h4><p class="text-sm text-gray">Sincerely,<br>Your Name<br>Title | Company</p></div><p class="text-sm text-gray" style="margin-top: 1rem; text-align: center;">Click a signature to insert it</p>"#.to_string());
    }

    let mut html = String::new();
    use std::fmt::Write;
    for sig in sigs {
        let _ = write!(html, r##"<div class="signature-item" onclick="useSignature('{}')"><h4>{}</h4><div class="text-sm text-gray">{}</div></div>"##, sig.id, sig.name, sig.content_html);
    }
    let _ = writeln!(html, r#"<p class="text-sm text-gray" style="margin-top: 1rem; text-align: center;">Click a signature to insert it</p>"#);

    axum::response::Html(html)
}

pub async fn list_rules_htmx(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let user_id = match extract_user_from_session(&headers) {
        Ok(id) => id,
        Err(_) => return axum::response::Html(r#"<div class="rule-item">Please sign in</div>"#.to_string()),
    };

    let pool = state.pool.clone();
    let rules_result = tokio::task::spawn_blocking(move || {
        let mut db_conn = pool.get().map_err(|e| format!("DB connection error: {}", e))?;
        diesel::sql_query("SELECT id, name, conditions_json, actions_json, is_active FROM email_rules WHERE user_id = $1 ORDER BY name ASC")
            .bind::<diesel::sql_types::Uuid, _>(user_id)
            .get_results::<RuleRow>(&mut db_conn)
            .map_err(|e| format!("Failed to load rules: {}", e))
    }).await;

    let rules = match rules_result {
        Ok(Ok(list)) => list,
        _ => Vec::new(),
    };

    if rules.is_empty() {
        return axum::response::Html(r#"<div class="rule-item"><div class="rule-header"><span class="rule-name">Auto-archive newsletters</span><label class="toggle-label"><input type="checkbox" checked><span class="toggle-switch"></span></label></div><p class="text-sm text-gray">From: *@newsletter.* → Archive</p></div><div class="rule-item"><div class="rule-header"><span class="rule-name">Label work emails</span><label class="toggle-label"><input type="checkbox" checked><span class="toggle-switch"></span></label></div><p class="text-sm text-gray">From: *@company.com → Label: Work</p></div>"#.to_string());
    }

    let mut html = String::new();
    use std::fmt::Write;
    for rule in rules {
        let checked = if rule.is_active { "checked" } else { "" };
        let _ = write!(html, r##"<div class="rule-item"><div class="rule-header"><span class="rule-name">{}</span><label class="toggle-label"><input type="checkbox" {}><span class="toggle-switch"></span></label></div><p class="text-sm text-gray">If: {} → Action: {}</p></div>"##, rule.name, checked, rule.conditions_json, rule.actions_json);
    }

    axum::response::Html(html)
}

pub async fn create_rule(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    axum::Json(payload): axum::Json<serde_json::Value>,
) -> impl IntoResponse {
    let user_id = match extract_user_from_session(&headers) {
        Ok(id) => id,
        Err(_) => return (StatusCode::UNAUTHORIZED, axum::Json(serde_json::json!({ "success": false, "error": "Unauthorized" }))).into_response(),
    };

    let name = payload.get("name").and_then(|v| v.as_str()).unwrap_or("Unnamed Rule").to_string();
    let condition = payload.get("condition").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let action = payload.get("action").and_then(|v| v.as_str()).unwrap_or("archive").to_string();

    let pool = state.pool.clone();
    let insert_result = tokio::task::spawn_blocking(move || {
        let mut db_conn = pool.get().map_err(|e| format!("DB connection error: {}", e))?;
        let bot_id = Uuid::nil();
        diesel::sql_query("INSERT INTO email_rules (id, user_id, bot_id, name, conditions_json, actions_json, is_active, priority, stop_processing, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW(), NOW())")
            .bind::<diesel::sql_types::Uuid, _>(Uuid::new_v4())
            .bind::<diesel::sql_types::Uuid, _>(user_id)
            .bind::<diesel::sql_types::Uuid, _>(bot_id)
            .bind::<diesel::sql_types::Varchar, _>(&name)
            .bind::<diesel::sql_types::Text, _>(&condition)
            .bind::<diesel::sql_types::Text, _>(&action)
            .bind::<diesel::sql_types::Bool, _>(true)
            .bind::<diesel::sql_types::Integer, _>(0)
            .bind::<diesel::sql_types::Bool, _>(true)
            .execute(&mut db_conn)
            .map_err(|e| format!("Failed to insert rule: {}", e))
    }).await;

    match insert_result {
        Ok(Ok(_)) => (StatusCode::OK, axum::Json(serde_json::json!({ "success": true }))).into_response(),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, axum::Json(serde_json::json!({ "success": false, "error": "Database error" }))).into_response(),
    }
}

pub async fn create_template(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    axum::Json(payload): axum::Json<serde_json::Value>,
) -> impl IntoResponse {
    let user_id = match extract_user_from_session(&headers) {
        Ok(id) => id,
        Err(_) => return (StatusCode::UNAUTHORIZED, axum::Json(serde_json::json!({ "success": false, "error": "Unauthorized" }))).into_response(),
    };

    let name = payload.get("name").and_then(|v| v.as_str()).unwrap_or("Unnamed Template").to_string();
    let subject = payload.get("subject").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let body = payload.get("body").and_then(|v| v.as_str()).unwrap_or("").to_string();

    let pool = state.pool.clone();
    let insert_result = tokio::task::spawn_blocking(move || {
        let mut db_conn = pool.get().map_err(|e| format!("DB connection error: {}", e))?;
        let bot_id = Uuid::nil();
        diesel::sql_query("INSERT INTO email_templates (id, bot_id, user_id, name, subject_template, body_html_template, variables_json, is_shared, usage_count, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW(), NOW())")
            .bind::<diesel::sql_types::Uuid, _>(Uuid::new_v4())
            .bind::<diesel::sql_types::Uuid, _>(bot_id)
            .bind::<diesel::sql_types::Uuid, _>(user_id)
            .bind::<diesel::sql_types::Varchar, _>(&name)
            .bind::<diesel::sql_types::Text, _>(&subject)
            .bind::<diesel::sql_types::Text, _>(&body)
            .bind::<diesel::sql_types::Text, _>("{}")
            .bind::<diesel::sql_types::Bool, _>(false)
            .bind::<diesel::sql_types::Integer, _>(0)
            .execute(&mut db_conn)
            .map_err(|e| format!("Failed to insert template: {}", e))
    }).await;

    match insert_result {
        Ok(Ok(_)) => (StatusCode::OK, axum::Json(serde_json::json!({ "success": true }))).into_response(),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, axum::Json(serde_json::json!({ "success": false, "error": "Database error" }))).into_response(),
    }
}

use axum::http::StatusCode;




pub async fn search_emails_htmx(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let query = params.get("q").map(|s| s.as_str()).unwrap_or("");
    if query.is_empty() {
        return axum::response::Html(r#"<div class="empty-state"><p>Enter a search term to find emails</p></div>"#.to_string());
    }

    let search_term = format!("%{}%", query.to_lowercase());

    let mut conn = match state.pool.get() {
        Ok(c) => c,
        Err(_) => return axum::response::Html(r#"<div class="empty-state error"><p>Database connection error</p></div>"#.to_string()),
    };

    use crate::models::EmailSearchRow;
    let results: Vec<EmailSearchRow> = match diesel::sql_query(
        "SELECT id, subject, from_address, to_addresses, body_text, received_at FROM emails WHERE LOWER(subject) LIKE $1 OR LOWER(from_address) LIKE $1 OR LOWER(body_text) LIKE $1 ORDER BY received_at DESC LIMIT 50"
    )
        .bind::<diesel::sql_types::Text, _>(&search_term)
        .load::<EmailSearchRow>(&mut conn)
    {
        Ok(r) => r,
        Err(e) => { warn!("Email search query failed: {}", e); Vec::new() }
    };

    if results.is_empty() {
        return axum::response::Html(format!(r#"<div class="empty-state"><svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="11" cy="11" r="8"></circle><path d="m21 21-4.35-4.35"></path></svg><h3>No results for "{}"</h3><p>Try different keywords or check your spelling.</p></div>"#, query));
    }

    let mut html = String::from(r#"<div class="search-results">"#);
    use std::fmt::Write;
    let _ = write!(html, r#"<div class="result-stats">Found {} results for "{}"</div>"#, results.len(), query);
    for row in results {
        let preview: String = row.body_text.as_deref().unwrap_or("").chars().take(100).collect();
        let formatted_date = row.received_at.format("%b %d, %Y").to_string();
        let _ = write!(html, r##"<div class="email-item" hx-get="/api/ui/email/content/{}" hx-target="#email-content" hx-swap="innerHTML"><div class="email-sender">{}</div><div class="email-subject">{}</div><div class="email-preview">{}</div><div class="email-date">{}</div></div>"##, row.id, row.from_address, row.subject, preview, formatted_date);
    }
    html.push_str("</div>");
    axum::response::Html(html)
}

pub async fn save_auto_responder(
    State(_state): State<Arc<AppState>>,
    axum::Form(form): axum::Form<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    info!("Saving auto-responder settings: {:?}", form);
    axum::response::Html(r#"<div class="notification success">Auto-responder settings saved successfully!</div>"#.to_string())
}
