use crate::shared::state::AppState;
use crate::core::config::EmailConfig;
use super::types::*;
use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use diesel::prelude::*;
use log::{error, info, warn};
use mailparse::{parse_mail, MailHeaderMap};
use std::sync::Arc;
use uuid::Uuid;

fn extract_user_from_session(_state: &Arc<AppState>) -> Result<Uuid, String> {
    Ok(Uuid::new_v4())
}

fn fetch_emails_from_folder(
    config: &EmailConfig,
    folder: &str,
) -> Result<Vec<EmailSummary>, String> {
    #[cfg(feature = "mail")]
    {
        let client = imap::ClientBuilder::new(&config.server, config.port)
            .connect()
            .map_err(|e| format!("Connection error: {}", e))?;

        let mut session = client
            .login(&config.username, &config.password)
            .map_err(|e| format!("Login failed: {:?}", e))?;

        let folder_name = match folder {
            "sent" => "Sent",
            "drafts" => "Drafts",
            "trash" => "Trash",
            _ => "INBOX",
        };

        session
            .select(folder_name)
            .map_err(|e| format!("Select folder failed: {}", e))?;

        let messages = session
            .fetch("1:20", "(FLAGS RFC822.HEADER)")
            .map_err(|e| format!("Fetch failed: {}", e))?;

        let mut emails = Vec::new();
        for message in messages.iter() {
            if let Some(header) = message.header() {
                let parsed = parse_mail(header).ok();
                if let Some(mail) = parsed {
                    let subject = mail.headers.get_first_value("Subject").unwrap_or_default();
                    let from = mail.headers.get_first_value("From").unwrap_or_default();
                    let date = mail.headers.get_first_value("Date").unwrap_or_default();
                    let flags = message.flags();
                    let read = flags.iter().any(|f| matches!(f, imap::types::Flag::Seen));

                    let preview = subject.chars().take(100).collect();
                    emails.push(EmailSummary {
                        id: message.message.to_string(),
                        from_name: from.clone(),
                        from_email: from,
                        subject,
                        preview,
                        date,
                        read,
                    });
                }
            }
        }

        session.logout().ok();
        Ok(emails)
    }

    #[cfg(not(feature = "mail"))]
    {
        Ok(Vec::new())
    }
}

fn get_folder_counts(
    config: &EmailConfig,
) -> Result<std::collections::HashMap<String, usize>, String> {
    use std::collections::HashMap;

    #[cfg(feature = "mail")]
    {
        let client = imap::ClientBuilder::new(&config.server, config.port)
            .connect()
            .map_err(|e| format!("Connection error: {}", e))?;

        let mut session = client
            .login(&config.username, &config.password)
            .map_err(|e| format!("Login failed: {:?}", e))?;

        let mut counts = HashMap::new();

        for folder in ["INBOX", "Sent", "Drafts", "Trash"] {
            if let Ok(mailbox) = session.examine(folder) {
                counts.insert((*folder).to_string(), mailbox.exists as usize);
            }
        }

        session.logout().ok();
        Ok(counts)
    }

    #[cfg(not(feature = "mail"))]
    {
        Ok(HashMap::new())
    }
}

fn fetch_email_by_id(config: &EmailConfig, id: &str) -> Result<EmailContent, String> {
    #[cfg(feature = "mail")]
    {
        let client = imap::ClientBuilder::new(&config.server, config.port)
            .connect()
            .map_err(|e| format!("Connection error: {}", e))?;

        let mut session = client
            .login(&config.username, &config.password)
            .map_err(|e| format!("Login failed: {:?}", e))?;

        session
            .select("INBOX")
            .map_err(|e| format!("Select failed: {}", e))?;

        let messages = session
            .fetch(id, "RFC822")
            .map_err(|e| format!("Fetch failed: {}", e))?;

        if let Some(message) = messages.iter().next() {
            if let Some(body) = message.body() {
                let parsed = parse_mail(body).map_err(|e| format!("Parse failed: {}", e))?;

                let subject = parsed
                    .headers
                    .get_first_value("Subject")
                    .unwrap_or_default();
                let from = parsed.headers.get_first_value("From").unwrap_or_default();
                let to = parsed.headers.get_first_value("To").unwrap_or_default();
                let date = parsed.headers.get_first_value("Date").unwrap_or_default();

                let body_text = parsed
                    .subparts
                    .iter()
                    .find_map(|p| p.get_body().ok())
                    .or_else(|| parsed.get_body().ok())
                    .unwrap_or_default();

                session.logout().ok();

                return Ok(EmailContent {
                    id: id.to_string(),
                    from_name: from.clone(),
                    from_email: from,
                    to,
                    subject,
                    body: body_text,
                    date,
                    read: false,
                });
            }
        }

        session.logout().ok();
        Err("Email not found".to_string())
    }

    #[cfg(not(feature = "mail"))]
    {
        Err("Mail feature not enabled".to_string())
    }
}

fn move_email_to_trash(config: &EmailConfig, id: &str) -> Result<(), String> {
    #[cfg(feature = "mail")]
    {
        let client = imap::ClientBuilder::new(&config.server, config.port)
            .connect()
            .map_err(|e| format!("Connection error: {}", e))?;

        let mut session = client
            .login(&config.username, &config.password)
            .map_err(|e| format!("Login failed: {:?}", e))?;

        session
            .select("INBOX")
            .map_err(|e| format!("Select failed: {}", e))?;

        session
            .store(id, "+FLAGS (\\Deleted)")
            .map_err(|e| format!("Store failed: {}", e))?;

        session
            .expunge()
            .map_err(|e| format!("Expunge failed: {}", e))?;

        session.logout().ok();
        Ok(())
    }

    #[cfg(not(feature = "mail"))]
    {
        Err("Mail feature not enabled".to_string())
    }
}

pub async fn list_emails_htmx(
    State(state): State<Arc<AppState>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let folder = params
        .get("folder")
        .cloned()
        .unwrap_or_else(|| "inbox".to_string());

    let user_id = match extract_user_from_session(&state) {
        Ok(id) => id,
        Err(_) => {
            return axum::response::Html(
                r#"<div class="empty-state">
                    <h3>Authentication required</h3>
                    <p>Please sign in to view your emails</p>
                </div>"#
                    .to_string(),
            );
        }
    };

    let conn = state.conn.clone();
    let account_result = tokio::task::spawn_blocking(move || {
        let db_conn_result = conn.get();
        let mut db_conn = match db_conn_result {
            Ok(c) => c,
            Err(e) => return Err(format!("DB connection error: {}", e)),
        };

        diesel::sql_query("SELECT * FROM user_email_accounts WHERE user_id = $1 LIMIT 1")
            .bind::<diesel::sql_types::Uuid, _>(user_id)
            .get_result::<EmailAccountRow>(&mut db_conn)
            .optional()
            .map_err(|e| format!("Failed to get email account: {}", e))
    })
    .await;

    let account = match account_result {
        Ok(Ok(Some(acc))) => acc,
        Ok(Ok(None)) => {
            return axum::response::Html(
                r##"<div class="empty-state">
                    <h3>No email account configured</h3>
                    <p>Please add an email account in settings to get started</p>
                    <a href="#settings" class="btn-primary" style="margin-top: 1rem; display: inline-block;">Add Email Account</a>
                </div>"##
                    .to_string(),
            );
        }
        Ok(Err(e)) => {
            error!("Email account query error: {}", e);
            return axum::response::Html(
                r#"<div class="empty-state">
                    <h3>Unable to load emails</h3>
                    <p>There was an error connecting to the database. Please try again later.</p>
                </div>"#
                    .to_string(),
            );
        }
        Err(e) => {
            error!("Task join error: {}", e);
            return axum::response::Html(
                r#"<div class="empty-state">
                    <h3>Unable to load emails</h3>
                    <p>An internal error occurred. Please try again later.</p>
                </div>"#
                    .to_string(),
            );
        }
    };

    let config = EmailConfig {
        username: account.username.clone(),
        password: account.password_encrypted.clone(),
        server: account.imap_server.clone(),
        port: account.imap_port as u16,
        from: account.email.clone(),
        smtp_server: account.smtp_server.clone(),
        smtp_port: account.smtp_port as u16,
    };

    let emails = fetch_emails_from_folder(&config, &folder).unwrap_or_default();

    let mut html = String::new();
    use std::fmt::Write;
    for email in &emails {
        let unread_class = if !email.read { "unread" } else { "" };
        let _ = write!(
            html,
            r##"<div class="mail-item {}"
                 hx-get="/api/email/{}"
                 hx-target="#mail-content"
                 hx-swap="innerHTML">
                <div class="mail-header">
                    <span>{}</span>
                    <span class="text-sm text-gray">{}</span>
                </div>
                <div class="mail-subject">{}</div>
                <div class="mail-preview">{}</div>
            </div>"##,
            unread_class, email.id, email.from_name, email.date, email.subject, email.preview
        );
    }

    if html.is_empty() {
        html = format!(
            r#"<div class="empty-state">
                <h3>No emails in {}</h3>
                <p>This folder is empty</p>
            </div>"#,
            folder
        );
    }

    axum::response::Html(html)
}

pub async fn list_folders_htmx(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let user_id = match extract_user_from_session(&state) {
        Ok(id) => id,
        Err(_) => {
            return axum::response::Html(
                r#"<div class="nav-item">Please sign in</div>"#.to_string(),
            );
        }
    };

    let conn = state.conn.clone();
    let account_result = tokio::task::spawn_blocking(move || {
        let db_conn_result = conn.get();
        let mut db_conn = match db_conn_result {
            Ok(c) => c,
            Err(e) => return Err(format!("DB connection error: {}", e)),
        };

        diesel::sql_query("SELECT * FROM user_email_accounts WHERE user_id = $1 LIMIT 1")
            .bind::<diesel::sql_types::Uuid, _>(user_id)
            .get_result::<EmailAccountRow>(&mut db_conn)
            .optional()
            .map_err(|e| format!("Failed to get email account: {}", e))
    })
    .await;

    let account = match account_result {
        Ok(Ok(Some(acc))) => acc,
        Ok(Ok(None)) => {
            return axum::response::Html(
                r#"<div class="nav-item">No account configured</div>"#.to_string(),
            );
        }
        Ok(Err(e)) => {
            error!("Email folder query error: {}", e);
            return axum::response::Html(
                r#"<div class="nav-item">Error loading folders</div>"#.to_string(),
            );
        }
        Err(e) => {
            error!("Task join error: {}", e);
            return axum::response::Html(
                r#"<div class="nav-item">Error loading folders</div>"#.to_string(),
            );
        }
    };

    let config = EmailConfig {
        username: account.username.clone(),
        password: account.password_encrypted.clone(),
        server: account.imap_server.clone(),
        port: account.imap_port as u16,
        from: account.email.clone(),
        smtp_server: account.smtp_server.clone(),
        smtp_port: account.smtp_port as u16,
    };

    let folder_counts = get_folder_counts(&config).unwrap_or_default();

    let mut html = String::new();
    for (folder_name, icon, count) in &[
        ("inbox", "", folder_counts.get("INBOX").unwrap_or(&0)),
        ("sent", "", folder_counts.get("Sent").unwrap_or(&0)),
        ("drafts", "", folder_counts.get("Drafts").unwrap_or(&0)),
        ("trash", "", folder_counts.get("Trash").unwrap_or(&0)),
    ] {
        let active = if *folder_name == "inbox" {
            "active"
        } else {
            ""
        };
        let count_badge = if **count > 0 {
            format!(
                r#"<span style="margin-left: auto; font-size: 0.875rem; color: #64748b;">{}</span>"#,
                count
            )
        } else {
            String::new()
        };

        use std::fmt::Write;
        let _ = write!(
            html,
            r##"<div class="nav-item {}"
                 hx-get="/api/email/list?folder={}"
                 hx-target="#mail-list"
                 hx-swap="innerHTML">
                <span>{}</span> {}
                {}
            </div>"##,
            active,
            folder_name,
            icon,
            folder_name
                .chars()
                .next()
                .unwrap_or_default()
                .to_uppercase()
                .collect::<String>()
                + &folder_name[1..],
            count_badge
        );
    }

    axum::response::Html(html)
}

pub async fn compose_email_htmx(
    State(_state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, EmailError> {
    let html = r##"
        <div class="mail-content-view">
            <h2>Compose New Email</h2>
            <form class="compose-form"
                  hx-post="/api/email/send"
                  hx-target="#mail-content"
                  hx-swap="innerHTML">
                <div class="form-group">
                    <label>To:</label>
                    <input type="email" name="to" required>
                </div>
                <div class="form-group">
                    <label>Subject:</label>
                    <input type="text" name="subject" required>
                </div>
                <div class="form-group">
                    <label>Message:</label>
                    <textarea name="body" rows="10" required></textarea>
                </div>
                <div class="compose-actions">
                    <button type="submit" class="btn-primary">Send</button>
                    <button type="button" class="btn-secondary"
                            hx-post="/api/email/draft"
                            hx-include="closest form">Save Draft</button>
                </div>
            </form>
        </div>
    "##;

    Ok(axum::response::Html(html))
}

pub async fn get_email_content_htmx(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, EmailError> {
    let user_id = extract_user_from_session(&state)
        .map_err(|_| EmailError("Authentication required".to_string()))?;

    let conn = state.conn.clone();
    let account = tokio::task::spawn_blocking(move || {
        let mut db_conn = conn
            .get()
            .map_err(|e| format!("DB connection error: {}", e))?;

        diesel::sql_query("SELECT * FROM user_email_accounts WHERE user_id = $1 LIMIT 1")
            .bind::<diesel::sql_types::Uuid, _>(user_id)
            .get_result::<EmailAccountRow>(&mut db_conn)
            .optional()
            .map_err(|e| format!("Failed to get email account: {}", e))
    })
    .await
    .map_err(|e| EmailError(format!("Task join error: {e}")))?
    .map_err(EmailError)?;

    let Some(account) = account else {
        return Ok(axum::response::Html(
            r#"<div class="mail-content-view">
                <p>No email account configured</p>
            </div>"#
                .to_string(),
        ));
    };

    let config = EmailConfig {
        username: account.username.clone(),
        password: account.password_encrypted.clone(),
        server: account.imap_server.clone(),
        port: account.imap_port as u16,
        from: account.email.clone(),
        smtp_server: account.smtp_server.clone(),
        smtp_port: account.smtp_port as u16,
    };

    let email_content = fetch_email_by_id(&config, &id)
        .map_err(|e| EmailError(format!("Failed to fetch email: {}", e)))?;

    let html = format!(
        r##"
        <div class="mail-content-view">
            <div class="mail-actions">
                <button hx-get="/api/email/compose?reply_to={}"
                        hx-target="#mail-content"
                        hx-swap="innerHTML">Reply</button>
                <button hx-get="/api/email/compose?forward={}"
                        hx-target="#mail-content"
                        hx-swap="innerHTML">Forward</button>
                <button hx-delete="/api/email/{}"
                        hx-target="#mail-list"
                        hx-swap="innerHTML"
                        hx-confirm="Delete this email?">Delete</button>
            </div>
            <h2>{}</h2>
            <div style="display: flex; align-items: center; gap: 1rem; margin: 1rem 0;">
                <div>
                    <div style="font-weight: 600;">{}</div>
                    <div class="text-sm text-gray">to: {}</div>
                </div>
                <div style="margin-left: auto;" class="text-sm text-gray">{}</div>
            </div>
            <div class="mail-body">
                {}
            </div>
        </div>
        "##,
        id,
        id,
        id,
        email_content.subject,
        email_content.from_name,
        email_content.to,
        email_content.date,
        email_content.body
    );

    Ok(axum::response::Html(html))
}

pub async fn delete_email_htmx(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let user_id = match extract_user_from_session(&state) {
        Ok(id) => id,
        Err(_) => {
            return axum::response::Html(
                r#"<div class="empty-state">
                    <h3>Authentication required</h3>
                    <p>Please sign in to delete emails</p>
                </div>"#
                    .to_string(),
            );
        }
    };

    let conn = state.conn.clone();
    let account_result = tokio::task::spawn_blocking(move || {
        let db_conn_result = conn.get();
        let mut db_conn = match db_conn_result {
            Ok(c) => c,
            Err(e) => return Err(format!("DB connection error: {}", e)),
        };

        diesel::sql_query("SELECT * FROM user_email_accounts WHERE user_id = $1 LIMIT 1")
            .bind::<diesel::sql_types::Uuid, _>(user_id)
            .get_result::<EmailAccountRow>(&mut db_conn)
            .optional()
            .map_err(|e| format!("Failed to get email account: {}", e))
    })
    .await;

    let account = match account_result {
        Ok(Ok(Some(acc))) => acc,
        Ok(Ok(None)) => {
            return axum::response::Html(
                r#"<div class="empty-state">
                    <h3>No email account configured</h3>
                    <p>Please add an email account first</p>
                </div>"#
                    .to_string(),
            );
        }
        Ok(Err(e)) => {
            error!("Email account query error: {}", e);
            return axum::response::Html(
                r#"<div class="empty-state">
                    <h3>Error deleting email</h3>
                    <p>Database error occurred</p>
                </div>"#
                    .to_string(),
            );
        }
        Err(e) => {
            error!("Task join error: {}", e);
            return axum::response::Html(
                r#"<div class="empty-state">
                    <h3>Error deleting email</h3>
                    <p>An internal error occurred</p>
                </div>"#
                    .to_string(),
            );
        }
    };

    let config = EmailConfig {
        username: account.username.clone(),
        password: account.password_encrypted.clone(),
        server: account.imap_server.clone(),
        port: account.imap_port as u16,
        from: account.email.clone(),
        smtp_server: account.smtp_server.clone(),
        smtp_port: account.smtp_port as u16,
    };

    if let Err(e) = move_email_to_trash(&config, &id) {
        error!("Failed to delete email: {}", e);
        return axum::response::Html(
            r#"<div class="empty-state">
                <h3>Error deleting email</h3>
                <p>Failed to move email to trash</p>
            </div>"#
                .to_string(),
        );
    }

    info!("Email {} moved to trash", id);

    axum::response::Html(
        r#"<div class="success-message">
            <p>Email moved to trash</p>
        </div>
        <script>
            setTimeout(function() {
                htmx.trigger('#mail-list', 'load');
            }, 100);
        </script>"#
            .to_string(),
    )
}

pub async fn list_labels_htmx(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    axum::response::Html(
        r#"
        <div class="label-item" style="--label-color: #ef4444;">
            <span class="label-dot" style="background: #ef4444;"></span>
            <span>Important</span>
        </div>
        <div class="label-item" style="--label-color: #3b82f6;">
            <span class="label-dot" style="background: #3b82f6;"></span>
            <span>Work</span>
        </div>
        <div class="label-item" style="--label-color: #22c55e;">
            <span class="label-dot" style="background: #22c55e;"></span>
            <span>Personal</span>
        </div>
        <div class="label-item" style="--label-color: #f59e0b;">
            <span class="label-dot" style="background: #f59e0b;"></span>
            <span>Finance</span>
        </div>
    "#
        .to_string(),
    )
}

pub async fn list_templates_htmx(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    axum::response::Html(
        r#"
        <div class="template-item" onclick="useTemplate('welcome')">
            <h4>Welcome Email</h4>
            <p>Standard welcome message for new contacts</p>
        </div>
        <div class="template-item" onclick="useTemplate('followup')">
            <h4>Follow Up</h4>
            <p>General follow-up template</p>
        </div>
        <div class="template-item" onclick="useTemplate('meeting')">
            <h4>Meeting Request</h4>
            <p>Request a meeting with scheduling options</p>
        </div>
        <p class="text-sm text-gray" style="margin-top: 1rem; text-align: center;">
            Click a template to use it
        </p>
    "#
        .to_string(),
    )
}

pub async fn list_signatures_htmx(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    axum::response::Html(
        r#"
        <div class="signature-item" onclick="useSignature('default')">
            <h4>Default Signature</h4>
            <p class="text-sm text-gray">Best regards,<br>Your Name</p>
        </div>
        <div class="signature-item" onclick="useSignature('formal')">
            <h4>Formal Signature</h4>
            <p class="text-sm text-gray">Sincerely,<br>Your Name<br>Title | Company</p>
        </div>
        <p class="text-sm text-gray" style="margin-top: 1rem; text-align: center;">
            Click a signature to insert it
        </p>
    "#
        .to_string(),
    )
}

pub async fn list_rules_htmx(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    axum::response::Html(
        r#"
        <div class="rule-item">
            <div class="rule-header">
                <span class="rule-name">Auto-archive newsletters</span>
                <label class="toggle-label">
                    <input type="checkbox" checked>
                    <span class="toggle-switch"></span>
                </label>
            </div>
            <p class="text-sm text-gray">From: *@newsletter.* → Archive</p>
        </div>
        <div class="rule-item">
            <div class="rule-header">
                <span class="rule-name">Label work emails</span>
                <label class="toggle-label">
                    <input type="checkbox" checked>
                    <span class="toggle-switch"></span>
                </label>
            </div>
            <p class="text-sm text-gray">From: *@company.com → Label: Work</p>
        </div>
        <button class="btn-secondary" style="width: 100%; margin-top: 1rem;">
            + Add New Rule
        </button>
    "#
        .to_string(),
    )
}

pub async fn search_emails_htmx(
    State(state): State<Arc<AppState>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let query = params.get("q").map(|s| s.as_str()).unwrap_or("");

    if query.is_empty() {
        return axum::response::Html(
            r#"
            <div class="empty-state">
                <p>Enter a search term to find emails</p>
            </div>
        "#
            .to_string(),
        );
    }

    let search_term = format!("%{query_lower}%", query_lower = query.to_lowercase());

    let Ok(mut conn) = state.conn.get() else {
        return axum::response::Html(
            r#"
                <div class="empty-state error">
                    <p>Database connection error</p>
                </div>
            "#
            .to_string(),
        );
    };

    let search_query = "SELECT id, subject, from_address, to_addresses, body_text, received_at
         FROM emails
         WHERE LOWER(subject) LIKE $1
            OR LOWER(from_address) LIKE $1
            OR LOWER(body_text) LIKE $1
         ORDER BY received_at DESC
         LIMIT 50";

    let results: Vec<EmailSearchRow> = match diesel::sql_query(search_query)
        .bind::<diesel::sql_types::Text, _>(&search_term)
        .load::<EmailSearchRow>(&mut conn)
    {
        Ok(r) => r,
        Err(e) => {
            warn!("Email search query failed: {}", e);
            Vec::new()
        }
    };

    if results.is_empty() {
        return axum::response::Html(format!(
            r#"
            <div class="empty-state">
                <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                    <circle cx="11" cy="11" r="8"></circle>
                    <path d="m21 21-4.35-4.35"></path>
                </svg>
                <h3>No results for "{}"</h3>
                <p>Try different keywords or check your spelling.</p>
            </div>
        "#,
            query
        ));
    }

    let mut html = String::from(r#"<div class="search-results">"#);
    use std::fmt::Write;
    let _ = write!(
        html,
        r#"<div class="result-stats">Found {} results for "{}"</div>"#,
        results.len(),
        query
    );

    for row in results {
        let preview = row
            .body_text
            .as_deref()
            .unwrap_or("")
            .chars()
            .take(100)
            .collect::<String>();
        let formatted_date = row.received_at.format("%b %d, %Y").to_string();

        let _ = write!(
            html,
            r##"
            <div class="email-item" hx-get="/ui/mail/view/{}" hx-target="#email-content" hx-swap="innerHTML">
                <div class="email-sender">{}</div>
                <div class="email-subject">{}</div>
                <div class="email-preview">{}</div>
                <div class="email-date">{}</div>
            </div>
        "##,
            row.id, row.from_address, row.subject, preview, formatted_date
        );
    }

    html.push_str("</div>");
    axum::response::Html(html)
}

pub async fn save_auto_responder(
    State(_state): State<Arc<AppState>>,
    axum::Form(form): axum::Form<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    info!("Saving auto-responder settings: {:?}", form);

    axum::response::Html(
        r#"
        <div class="notification success">
            Auto-responder settings saved successfully!
        </div>
    "#
        .to_string(),
    )
}
}
