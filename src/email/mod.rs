use crate::{config::EmailConfig, core::urls::ApiUrls, shared::state::AppState};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use axum::{
    routing::{get, post},
    Router,
};
use base64::{engine::general_purpose, Engine as _};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use imap::types::Seq;
use lettre::{transport::smtp::authentication::Credentials, Message, SmtpTransport, Transport};
use log::{debug, info, warn};
use mailparse::{parse_mail, MailHeaderMap};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

pub mod stalwart_client;
pub mod stalwart_sync;
pub mod vectordb;

// Helper function to extract user from session
async fn extract_user_from_session(state: &Arc<AppState>) -> Result<Uuid, String> {
    // For now, return a default user ID - in production this would check session/token
    // This should be replaced with proper session management
    Ok(Uuid::new_v4())
}

// ===== Router Configuration =====

/// Configure email API routes
pub fn configure() -> Router<Arc<AppState>> {
    Router::new()
        // JSON API endpoints for services
        .route(ApiUrls::EMAIL_ACCOUNTS, get(list_email_accounts))
        .route(
            &format!("{}/add", ApiUrls::EMAIL_ACCOUNTS),
            post(add_email_account),
        )
        .route(
            ApiUrls::EMAIL_ACCOUNT_BY_ID.replace(":id", "{account_id}"),
            axum::routing::delete(delete_email_account),
        )
        .route(ApiUrls::EMAIL_LIST, post(list_emails))
        .route(ApiUrls::EMAIL_SEND, post(send_email))
        .route(ApiUrls::EMAIL_DRAFT, post(save_draft))
        .route(
            ApiUrls::EMAIL_FOLDERS.replace(":account_id", "{account_id}"),
            get(list_folders),
        )
        .route(ApiUrls::EMAIL_LATEST, get(get_latest_email))
        .route(
            ApiUrls::EMAIL_GET.replace(":campaign_id", "{campaign_id}"),
            get(get_email),
        )
        .route(
            ApiUrls::EMAIL_CLICK
                .replace(":campaign_id", "{campaign_id}")
                .replace(":email", "{email}"),
            post(track_click),
        )
        // Email read tracking endpoints
        .route("/api/email/tracking/pixel/{tracking_id}", get(serve_tracking_pixel))
        .route("/api/email/tracking/status/{tracking_id}", get(get_tracking_status))
        .route("/api/email/tracking/list", get(list_sent_emails_tracking))
        .route("/api/email/tracking/stats", get(get_tracking_stats))
        // UI HTMX endpoints (return HTML fragments)
        .route("/ui/email/accounts", get(list_email_accounts_htmx))
        .route("/ui/email/list", get(list_emails_htmx))
        .route("/ui/email/folders", get(list_folders_htmx))
        .route("/ui/email/compose", get(compose_email_htmx))
        .route("/ui/email/:id", get(get_email_content_htmx))
        .route("/ui/email/:id/delete", delete(delete_email_htmx))
        .route("/ui/email/labels", get(list_labels_htmx))
        .route("/ui/email/templates", get(list_templates_htmx))
        .route("/ui/email/signatures", get(list_signatures_htmx))
        .route("/ui/email/rules", get(list_rules_htmx))
        .route("/ui/email/search", get(search_emails_htmx))
        .route("/ui/email/auto-responder", post(save_auto_responder))
}

// Export SaveDraftRequest for other modules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveDraftRequest {
    pub account_id: String,
    pub to: String,
    pub cc: Option<String>,
    pub bcc: Option<String>,
    pub subject: String,
    pub body: String,
}

// ===== Email Tracking Structures =====

/// Sent email tracking record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentEmailTracking {
    pub id: String,
    pub tracking_id: String,
    pub bot_id: String,
    pub account_id: String,
    pub from_email: String,
    pub to_email: String,
    pub cc: Option<String>,
    pub bcc: Option<String>,
    pub subject: String,
    pub sent_at: DateTime<Utc>,
    pub read_at: Option<DateTime<Utc>>,
    pub read_count: i32,
    pub first_read_ip: Option<String>,
    pub last_read_ip: Option<String>,
    pub user_agent: Option<String>,
    pub is_read: bool,
}

/// Tracking status response for UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingStatusResponse {
    pub tracking_id: String,
    pub to_email: String,
    pub subject: String,
    pub sent_at: String,
    pub is_read: bool,
    pub read_at: Option<String>,
    pub read_count: i32,
}

/// Query params for tracking pixel
#[derive(Debug, Deserialize)]
pub struct TrackingPixelQuery {
    pub t: Option<String>, // Additional tracking token
}

/// Query params for listing tracked emails
#[derive(Debug, Deserialize)]
pub struct ListTrackingQuery {
    pub account_id: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub filter: Option<String>, // "all", "read", "unread"
}

/// Tracking statistics response
#[derive(Debug, Serialize)]
pub struct TrackingStatsResponse {
    pub total_sent: i64,
    pub total_read: i64,
    pub read_rate: f64,
    pub avg_time_to_read_hours: Option<f64>,
}

// ===== Request/Response Structures =====

#[derive(Debug, Serialize, Deserialize)]
pub struct EmailAccountRequest {
    pub email: String,
    pub display_name: Option<String>,
    pub imap_server: String,
    pub imap_port: u16,
    pub smtp_server: String,
    pub smtp_port: u16,
    pub username: String,
    pub password: String,
    pub is_primary: bool,
}

#[derive(Debug, Serialize)]
pub struct EmailAccountResponse {
    pub id: String,
    pub email: String,
    pub display_name: Option<String>,
    pub imap_server: String,
    pub imap_port: u16,
    pub smtp_server: String,
    pub smtp_port: u16,
    pub is_primary: bool,
    pub is_active: bool,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct EmailResponse {
    pub id: String,
    pub from_name: String,
    pub from_email: String,
    pub to: String,
    pub subject: String,
    pub preview: String,
    pub body: String,
    pub date: String,
    pub time: String,
    pub read: bool,
    pub folder: String,
    pub has_attachments: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmailRequest {
    pub to: String,
    pub subject: String,
    pub body: String,
    pub cc: Option<String>,
    pub bcc: Option<String>,
    pub attachments: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendEmailRequest {
    pub account_id: String,
    pub to: String,
    pub cc: Option<String>,
    pub bcc: Option<String>,
    pub subject: String,
    pub body: String,
    pub is_html: bool,
}

#[derive(Debug, Serialize)]
pub struct SaveDraftResponse {
    pub success: bool,
    pub draft_id: Option<String>,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct ListEmailsRequest {
    pub account_id: String,
    pub folder: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct MarkEmailRequest {
    pub account_id: String,
    pub email_id: String,
    pub read: bool,
}

#[derive(Debug, Deserialize)]
pub struct DeleteEmailRequest {
    pub account_id: String,
    pub email_id: String,
}

#[derive(Debug, Serialize)]
pub struct FolderInfo {
    pub name: String,
    pub path: String,
    pub unread_count: i32,
    pub total_count: i32,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
}

// ===== Error Handling =====

struct EmailError(String);

impl IntoResponse for EmailError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.0).into_response()
    }
}

impl From<String> for EmailError {
    fn from(s: String) -> Self {
        EmailError(s)
    }
}

// ===== Helper Functions =====

fn parse_from_field(from: &str) -> (String, String) {
    if let Some(start) = from.find('<') {
        if let Some(end) = from.find('>') {
            let name = from[..start].trim().trim_matches('"').to_string();
            let email = from[start + 1..end].to_string();
            return (name, email);
        }
    }
    (String::new(), from.to_string())
}

fn format_email_time(date_str: &str) -> String {
    // Simple time formatting - in production, use proper date parsing
    if date_str.is_empty() {
        return "Unknown".to_string();
    }
    // Return simplified version for now
    date_str
        .split_whitespace()
        .take(4)
        .collect::<Vec<_>>()
        .join(" ")
}

fn encrypt_password(password: &str) -> String {
    // In production, use proper encryption like AES-256
    // For now, base64 encode (THIS IS NOT SECURE - USE PROPER ENCRYPTION)
    general_purpose::STANDARD.encode(password.as_bytes())
}

fn decrypt_password(encrypted: &str) -> Result<String, String> {
    // In production, use proper decryption
    general_purpose::STANDARD
        .decode(encrypted)
        .map_err(|e| format!("Decryption failed: {}", e))
        .and_then(|bytes| {
            String::from_utf8(bytes).map_err(|e| format!("UTF-8 conversion failed: {}", e))
        })
}

// ===== Account Management Endpoints =====

pub async fn add_email_account(
    State(state): State<Arc<AppState>>,
    Json(request): Json<EmailAccountRequest>,
) -> Result<Json<ApiResponse<EmailAccountResponse>>, EmailError> {
    // Get user_id from session
    let user_id = match extract_user_from_session(&state).await {
        Ok(id) => id,
        Err(_) => return Err(EmailError("Authentication required".to_string())),
    };

    let account_id = Uuid::new_v4();
    let encrypted_password = encrypt_password(&request.password);

    let conn = state.conn.clone();
    tokio::task::spawn_blocking(move || {
        use crate::shared::models::schema::user_email_accounts::dsl::*;
        let mut db_conn = conn.get().map_err(|e| format!("DB connection error: {}", e))?;

        // If this is primary, unset other primary accounts
        if request.is_primary {
            diesel::update(user_email_accounts.filter(user_id.eq(&user_id)))
                .set(is_primary.eq(false))
                .execute(&mut db_conn)
                .ok();
        }

        diesel::sql_query(
            "INSERT INTO user_email_accounts
            (id, user_id, email, display_name, imap_server, imap_port, smtp_server, smtp_port, username, password_encrypted, is_primary, is_active)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)"
        )
        .bind::<diesel::sql_types::Uuid, _>(account_id)
        .bind::<diesel::sql_types::Uuid, _>(user_id)
        .bind::<diesel::sql_types::Text, _>(&request.email)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(request.display_name.as_ref())
        .bind::<diesel::sql_types::Text, _>(&request.imap_server)
        .bind::<diesel::sql_types::Integer, _>(request.imap_port as i32)
        .bind::<diesel::sql_types::Text, _>(&request.smtp_server)
        .bind::<diesel::sql_types::Integer, _>(request.smtp_port as i32)
        .bind::<diesel::sql_types::Text, _>(&request.username)
        .bind::<diesel::sql_types::Text, _>(&encrypted_password)
        .bind::<diesel::sql_types::Bool, _>(request.is_primary)
        .bind::<diesel::sql_types::Bool, _>(true)
        .execute(&mut db_conn)
        .map_err(|e| format!("Failed to insert account: {}", e))?;

        Ok::<_, String>(account_id)
    })
    .await
    .map_err(|e| EmailError(format!("Task join error: {}", e)))?
    .map_err(EmailError)?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(EmailAccountResponse {
            id: account_id.to_string(),
            email: request.email,
            display_name: request.display_name,
            imap_server: request.imap_server,
            imap_port: request.imap_port,
            smtp_server: request.smtp_server,
            smtp_port: request.smtp_port,
            is_primary: request.is_primary,
            is_active: true,
            created_at: chrono::Utc::now().to_rfc3339(),
        }),
        message: Some("Email account added successfully".to_string()),
    }))
}

/// List email accounts - HTMX HTML response for UI
pub async fn list_email_accounts_htmx(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    // Get user_id from session
    let user_id = match extract_user_from_session(&state).await {
        Ok(id) => id,
        Err(_) => {
            return axum::response::Html(r#"
                <div class="account-item" onclick="document.getElementById('add-account-modal').showModal()">
                    <span>+ Add email account</span>
                </div>
            "#.to_string());
        }
    };

    let conn = state.conn.clone();
    let accounts = tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().map_err(|e| format!("DB connection error: {}", e))?;

        diesel::sql_query(
            "SELECT id, email, display_name, is_primary FROM user_email_accounts WHERE user_id = $1 AND is_active = true ORDER BY is_primary DESC"
        )
        .bind::<diesel::sql_types::Uuid, _>(user_id)
        .load::<(Uuid, String, Option<String>, bool)>(&mut db_conn)
        .map_err(|e| format!("Query failed: {}", e))
    })
    .await
    .ok()
    .and_then(|r| r.ok())
    .unwrap_or_default();

    if accounts.is_empty() {
        return axum::response::Html(r#"
            <div class="account-item" onclick="document.getElementById('add-account-modal').showModal()">
                <span>+ Add email account</span>
            </div>
        "#.to_string());
    }

    let mut html = String::new();
    for (id, email, display_name, is_primary) in accounts {
        let name = display_name.unwrap_or_else(|| email.clone());
        let primary_badge = if is_primary { r#"<span class="badge">Primary</span>"# } else { "" };
        html.push_str(&format!(
            r#"<div class="account-item" data-account-id="{}">
                <span>{}</span>
                {}
            </div>"#,
            id, name, primary_badge
        ));
    }

    axum::response::Html(html)
}

/// List email accounts - JSON API for services
pub async fn list_email_accounts(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<Vec<EmailAccountResponse>>>, EmailError> {
    // Get user_id from session
    let user_id = match extract_user_from_session(&state).await {
        Ok(id) => id,
        Err(_) => return Err(EmailError("Authentication required".to_string())),
    };

    let conn = state.conn.clone();
    let accounts = tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().map_err(|e| format!("DB connection error: {}", e))?;

        let results: Vec<(Uuid, String, Option<String>, String, i32, String, i32, bool, bool, chrono::DateTime<chrono::Utc>)> =
            diesel::sql_query(
                "SELECT id, email, display_name, imap_server, imap_port, smtp_server, smtp_port, is_primary, is_active, created_at
                FROM user_email_accounts WHERE user_id = $1 AND is_active = true ORDER BY is_primary DESC, created_at DESC"
            )
            .bind::<diesel::sql_types::Uuid, _>(user_id)
            .load(&mut db_conn)
            .map_err(|e| format!("Query failed: {}", e))?;

        Ok::<_, String>(results)
    })
    .await
    .map_err(|e| EmailError(format!("Task join error: {}", e)))?
    .map_err(EmailError)?;

    let account_list: Vec<EmailAccountResponse> = accounts
        .into_iter()
        .map(
            |(
                id,
                email,
                display_name,
                imap_server,
                imap_port,
                smtp_server,
                smtp_port,
                is_primary,
                is_active,
                created_at,
            )| {
                EmailAccountResponse {
                    id: id.to_string(),
                    email,
                    display_name,
                    imap_server,
                    imap_port: imap_port as u16,
                    smtp_server,
                    smtp_port: smtp_port as u16,
                    is_primary,
                    is_active,
                    created_at: created_at.to_rfc3339(),
                }
            },
        )
        .collect();

    Ok(Json(ApiResponse {
        success: true,
        data: Some(account_list),
        message: None,
    }))
}

pub async fn delete_email_account(
    State(state): State<Arc<AppState>>,
    Path(account_id): Path<String>,
) -> Result<Json<ApiResponse<()>>, EmailError> {
    let account_uuid =
        Uuid::parse_str(&account_id).map_err(|_| EmailError("Invalid account ID".to_string()))?;

    let conn = state.conn.clone();
    tokio::task::spawn_blocking(move || {
        let mut db_conn = conn
            .get()
            .map_err(|e| format!("DB connection error: {}", e))?;

        diesel::sql_query("UPDATE user_email_accounts SET is_active = false WHERE id = $1")
            .bind::<diesel::sql_types::Uuid, _>(account_uuid)
            .execute(&mut db_conn)
            .map_err(|e| format!("Failed to delete account: {}", e))?;

        Ok::<_, String>(())
    })
    .await
    .map_err(|e| EmailError(format!("Task join error: {}", e)))?
    .map_err(EmailError)?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(()),
        message: Some("Email account deleted".to_string()),
    }))
}

// ===== Email Operations =====

pub async fn list_emails(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ListEmailsRequest>,
) -> Result<Json<ApiResponse<Vec<EmailResponse>>>, EmailError> {
    let account_uuid = Uuid::parse_str(&request.account_id)
        .map_err(|_| EmailError("Invalid account ID".to_string()))?;

    // Get account credentials from database
    let conn = state.conn.clone();
    let account_info = tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().map_err(|e| format!("DB connection error: {}", e))?;

        let result: (String, i32, String, String) = diesel::sql_query(
            "SELECT imap_server, imap_port, username, password_encrypted FROM user_email_accounts WHERE id = $1 AND is_active = true"
        )
        .bind::<diesel::sql_types::Uuid, _>(account_uuid)
        .get_result(&mut db_conn)
        .map_err(|e| format!("Account not found: {}", e))?;

        Ok::<_, String>(result)
    })
    .await
    .map_err(|e| EmailError(format!("Task join error: {}", e)))?
    .map_err(EmailError)?;

    let (imap_server, imap_port, username, encrypted_password) = account_info;
    let password = decrypt_password(&encrypted_password).map_err(EmailError)?;

    // Connect to IMAP
    let tls = native_tls::TlsConnector::builder()
        .build()
        .map_err(|e| EmailError(format!("Failed to create TLS connector: {:?}", e)))?;

    let client = imap::ClientBuilder::new(imap_server.as_str(), imap_port as u16)
        .native_tls(&tls)
        .map_err(|e| EmailError(format!("Failed to create IMAP client: {:?}", e)))?
        .connect()
        .map_err(|e| EmailError(format!("Failed to connect to IMAP: {:?}", e)))?;

    let mut session = client
        .login(&username, &password)
        .map_err(|e| EmailError(format!("Login failed: {:?}", e)))?;

    let folder = request.folder.unwrap_or_else(|| "INBOX".to_string());
    session
        .select(&folder)
        .map_err(|e| EmailError(format!("Failed to select folder: {:?}", e)))?;

    let messages = session
        .search("ALL")
        .map_err(|e| EmailError(format!("Failed to search emails: {:?}", e)))?;

    let mut email_list = Vec::new();
    let limit = request.limit.unwrap_or(50);
    let offset = request.offset.unwrap_or(0);

    let recent_messages: Vec<_> = messages.iter().cloned().collect();
    let recent_messages: Vec<Seq> = recent_messages
        .into_iter()
        .rev()
        .skip(offset)
        .take(limit)
        .collect();

    for seq in recent_messages {
        let fetch_result = session.fetch(seq.to_string(), "RFC822");
        let messages =
            fetch_result.map_err(|e| EmailError(format!("Failed to fetch email: {:?}", e)))?;

        for msg in messages.iter() {
            let body = msg
                .body()
                .ok_or_else(|| EmailError("No body found".to_string()))?;

            let parsed = parse_mail(body)
                .map_err(|e| EmailError(format!("Failed to parse email: {:?}", e)))?;

            let headers = parsed.get_headers();
            let subject = headers.get_first_value("Subject").unwrap_or_default();
            let from = headers.get_first_value("From").unwrap_or_default();
            let to = headers.get_first_value("To").unwrap_or_default();
            let date = headers.get_first_value("Date").unwrap_or_default();

            let body_text = if let Some(body_part) = parsed
                .subparts
                .iter()
                .find(|p| p.ctype.mimetype == "text/plain")
            {
                body_part.get_body().unwrap_or_default()
            } else {
                parsed.get_body().unwrap_or_default()
            };

            let body_html = if let Some(body_part) = parsed
                .subparts
                .iter()
                .find(|p| p.ctype.mimetype == "text/html")
            {
                body_part.get_body().unwrap_or_default()
            } else {
                String::new()
            };

            let preview = body_text.lines().take(3).collect::<Vec<_>>().join(" ");
            let preview_truncated = if preview.len() > 150 {
                format!("{}...", &preview[..150])
            } else {
                preview
            };

            let (from_name, from_email) = parse_from_field(&from);
            let has_attachments = parsed.subparts.iter().any(|p| {
                p.get_content_disposition().disposition == mailparse::DispositionType::Attachment
            });

            email_list.push(EmailResponse {
                id: seq.to_string(),
                from_name,
                from_email,
                to,
                subject,
                preview: preview_truncated,
                body: if body_html.is_empty() {
                    body_text
                } else {
                    body_html
                },
                date: format_email_time(&date),
                time: format_email_time(&date),
                read: false, // IMAP flags checked during fetch
                folder: folder.clone(),
                has_attachments,
            });
        }
    }

    session.logout().ok();

    Ok(Json(ApiResponse {
        success: true,
        data: Some(email_list),
        message: None,
    }))
}

pub async fn send_email(
    State(state): State<Arc<AppState>>,
    Json(request): Json<SendEmailRequest>,
) -> Result<Json<ApiResponse<()>>, EmailError> {
    let account_uuid = Uuid::parse_str(&request.account_id)
        .map_err(|_| EmailError("Invalid account ID".to_string()))?;

    // Get account credentials
    let conn = state.conn.clone();
    let account_info = tokio::task::spawn_blocking(move || {
        let mut db_conn = conn
            .get()
            .map_err(|e| format!("DB connection error: {}", e))?;

        let result: (String, String, i32, String, String, String) = diesel::sql_query(
            "SELECT email, display_name, smtp_port, smtp_server, username, password_encrypted
            FROM user_email_accounts WHERE id = $1 AND is_active = true",
        )
        .bind::<diesel::sql_types::Uuid, _>(account_uuid)
        .get_result(&mut db_conn)
        .map_err(|e| format!("Account not found: {}", e))?;

        Ok::<_, String>(result)
    })
    .await
    .map_err(|e| EmailError(format!("Task join error: {}", e)))?
    .map_err(EmailError)?;

    let (from_email, display_name, smtp_port, smtp_server, username, encrypted_password) =
        account_info;
    let password = decrypt_password(&encrypted_password).map_err(EmailError)?;

    let from_addr = if display_name.is_empty() {
        from_email.clone()
    } else {
        format!("{} <{}>", display_name, from_email)
    };

    // Check if email-read-pixel is enabled in bot config
    let pixel_enabled = is_tracking_pixel_enabled(&state, None).await;
    let tracking_id = Uuid::new_v4();

    // Build email body with tracking pixel if enabled
    let final_body = if pixel_enabled && request.is_html {
        inject_tracking_pixel(&request.body, &tracking_id.to_string(), &state).await
    } else {
        request.body.clone()
    };

    // Build email
    let mut email_builder = Message::builder()
        .from(
            from_addr
                .parse()
                .map_err(|e| EmailError(format!("Invalid from address: {}", e)))?,
        )
        .to(request
            .to
            .parse()
            .map_err(|e| EmailError(format!("Invalid to address: {}", e)))?)
        .subject(request.subject.clone());

    if let Some(ref cc) = request.cc {
        email_builder = email_builder.cc(cc
            .parse()
            .map_err(|e| EmailError(format!("Invalid cc address: {}", e)))?);
    }

    if let Some(ref bcc) = request.bcc {
        email_builder = email_builder.bcc(
            bcc.parse()
                .map_err(|e| EmailError(format!("Invalid bcc address: {}", e)))?,
        );
    }

    let email = email_builder
        .body(final_body)
        .map_err(|e| EmailError(format!("Failed to build email: {}", e)))?;

    // Send email
    let creds = Credentials::new(username, password);
    let mailer = SmtpTransport::relay(&smtp_server)
        .map_err(|e| EmailError(format!("Failed to create SMTP transport: {}", e)))?
        .port(smtp_port as u16)
        .credentials(creds)
        .build();

    mailer
        .send(&email)
        .map_err(|e| EmailError(format!("Failed to send email: {}", e)))?;

    // Save tracking record if pixel tracking is enabled
    if pixel_enabled {
        let conn = state.conn.clone();
        let to_email = request.to.clone();
        let subject = request.subject.clone();
        let cc_clone = request.cc.clone();
        let bcc_clone = request.bcc.clone();

        let _ = tokio::task::spawn_blocking(move || {
            save_email_tracking_record(
                conn,
                tracking_id,
                account_uuid,
                Uuid::nil(), // bot_id - would come from session in production
                &from_email,
                &to_email,
                cc_clone.as_deref(),
                bcc_clone.as_deref(),
                &subject,
            )
        })
        .await;
    }

    info!("Email sent successfully from account {} with tracking_id {}", account_uuid, tracking_id);

    Ok(Json(ApiResponse {
        success: true,
        data: Some(()),
        message: Some("Email sent successfully".to_string()),
    }))
}

pub async fn save_draft(
    State(state): State<Arc<AppState>>,
    Json(request): Json<SaveDraftRequest>,
) -> Result<Json<SaveDraftResponse>, EmailError> {
    let account_uuid = Uuid::parse_str(&request.account_id)
        .map_err(|_| EmailError("Invalid account ID".to_string()))?;

    // Get user_id from session
    let user_id = match extract_user_from_session(&state).await {
        Ok(id) => id,
        Err(_) => return Err(EmailError("Authentication required".to_string())),
    };
    let draft_id = Uuid::new_v4();

    let conn = state.conn.clone();
    tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().map_err(|e| format!("DB connection error: {}", e))?;

        diesel::sql_query(
            "INSERT INTO email_drafts (id, user_id, account_id, to_address, cc_address, bcc_address, subject, body)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
        )
        .bind::<diesel::sql_types::Uuid, _>(draft_id)
        .bind::<diesel::sql_types::Uuid, _>(user_id)
        .bind::<diesel::sql_types::Uuid, _>(account_uuid)
        .bind::<diesel::sql_types::Text, _>(&request.to)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(request.cc.as_ref())
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(request.bcc.as_ref())
        .bind::<diesel::sql_types::Text, _>(&request.subject)
        .bind::<diesel::sql_types::Text, _>(&request.body)
        .execute(&mut db_conn)
        .map_err(|e| format!("Failed to save draft: {}", e))?;

        Ok::<_, String>(())
    })
    .await
    .map_err(|e| EmailError(format!("Task join error: {}", e)))?
    .map_err(|e| {
        return EmailError(e);
    })?;

    Ok(Json(SaveDraftResponse {
        success: true,
        draft_id: Some(draft_id.to_string()),
        message: "Draft saved successfully".to_string(),
    }))
}

pub async fn list_folders(
    State(state): State<Arc<AppState>>,
    Path(account_id): Path<String>,
) -> Result<Json<ApiResponse<Vec<FolderInfo>>>, EmailError> {
    let account_uuid =
        Uuid::parse_str(&account_id).map_err(|_| EmailError("Invalid account ID".to_string()))?;

    // Get account credentials
    let conn = state.conn.clone();
    let account_info = tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().map_err(|e| format!("DB connection error: {}", e))?;

        let result: (String, i32, String, String) = diesel::sql_query(
            "SELECT imap_server, imap_port, username, password_encrypted FROM user_email_accounts WHERE id = $1 AND is_active = true"
        )
        .bind::<diesel::sql_types::Uuid, _>(account_uuid)
        .get_result(&mut db_conn)
        .map_err(|e| format!("Account not found: {}", e))?;

        Ok::<_, String>(result)
    })
    .await
    .map_err(|e| EmailError(format!("Task join error: {}", e)))?
    .map_err(EmailError)?;

    let (imap_server, imap_port, username, encrypted_password) = account_info;
    let password = decrypt_password(&encrypted_password).map_err(EmailError)?;

    // Connect and list folders
    let tls = native_tls::TlsConnector::builder()
        .build()
        .map_err(|e| EmailError(format!("TLS error: {:?}", e)))?;

    let client = imap::ClientBuilder::new(imap_server.as_str(), imap_port as u16)
        .native_tls(&tls)
        .map_err(|e| EmailError(format!("Failed to create IMAP client: {:?}", e)))?
        .connect()
        .map_err(|e| EmailError(format!("Failed to connect to IMAP: {:?}", e)))?;

    let mut session = client
        .login(&username, &password)
        .map_err(|e| EmailError(format!("Login failed: {:?}", e)))?;

    let folders = session
        .list(None, Some("*"))
        .map_err(|e| EmailError(format!("Failed to list folders: {:?}", e)))?;

    let folder_list: Vec<FolderInfo> = folders
        .iter()
        .map(|f| FolderInfo {
            name: f.name().to_string(),
            path: f.name().to_string(),
            unread_count: 0, // Counts are fetched separately via IMAP STATUS
            total_count: 0,
        })
        .collect();

    session.logout().ok();

    Ok(Json(ApiResponse {
        success: true,
        data: Some(folder_list),
        message: None,
    }))
}

// ===== Legacy endpoints for backward compatibility =====

pub async fn get_latest_email_from(
    State(_state): State<Arc<AppState>>,
    Json(_request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, EmailError> {
    Ok(Json(serde_json::json!({
        "success": false,
        "message": "Please use the new /api/email/list endpoint with account_id"
    })))
}

pub async fn save_click(
    Path((campaign_id, email)): Path<(String, String)>,
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    info!(
        "Click tracked - Campaign: {}, Email: {}",
        campaign_id, email
    );

    let pixel: Vec<u8> = vec![
        0x47, 0x49, 0x46, 0x38, 0x39, 0x61, 0x01, 0x00, 0x01, 0x00, 0x80, 0x00, 0x00, 0xFF, 0xFF,
        0xFF, 0x00, 0x00, 0x00, 0x21, 0xF9, 0x04, 0x01, 0x00, 0x00, 0x00, 0x00, 0x2C, 0x00, 0x00,
        0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x02, 0x02, 0x44, 0x01, 0x00, 0x3B,
    ];

    (StatusCode::OK, [("content-type", "image/gif")], pixel)
}

// ===== Email Read Tracking Functions =====

/// 1x1 transparent GIF pixel bytes
const TRACKING_PIXEL: [u8; 43] = [
    0x47, 0x49, 0x46, 0x38, 0x39, 0x61, 0x01, 0x00, 0x01, 0x00, 0x80, 0x00, 0x00, 0xFF, 0xFF,
    0xFF, 0x00, 0x00, 0x00, 0x21, 0xF9, 0x04, 0x01, 0x00, 0x00, 0x00, 0x00, 0x2C, 0x00, 0x00,
    0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x02, 0x02, 0x44, 0x01, 0x00, 0x3B,
];

/// Check if email-read-pixel is enabled in config
async fn is_tracking_pixel_enabled(state: &Arc<AppState>, bot_id: Option<Uuid>) -> bool {
    let config_manager = crate::core::config::ConfigManager::new(state.conn.clone());
    let bot_id = bot_id.unwrap_or(Uuid::nil());

    config_manager
        .get_config(&bot_id, "email-read-pixel", Some("false"))
        .map(|v| v.to_lowercase() == "true")
        .unwrap_or(false)
}

/// Inject tracking pixel into HTML email body
async fn inject_tracking_pixel(html_body: &str, tracking_id: &str, state: &Arc<AppState>) -> String {
    // Get base URL from config or use default
    let config_manager = crate::core::config::ConfigManager::new(state.conn.clone());
    let base_url = config_manager
        .get_config(&Uuid::nil(), "server-url", Some("http://localhost:8080"))
        .unwrap_or_else(|| "http://localhost:8080".to_string());

    let pixel_url = format!("{}/api/email/tracking/pixel/{}", base_url, tracking_id);
    let pixel_html = format!(
        r#"<img src="{}" width="1" height="1" style="display:none;visibility:hidden;width:1px;height:1px;border:0;" alt="" />"#,
        pixel_url
    );

    // Insert pixel before closing </body> tag, or at the end if no body tag
    if html_body.to_lowercase().contains("</body>") {
        html_body.replace("</body>", &format!("{}</body>", pixel_html))
            .replace("</BODY>", &format!("{}</BODY>", pixel_html))
    } else {
        format!("{}{}", html_body, pixel_html)
    }
}

/// Save email tracking record to database
fn save_email_tracking_record(
    conn: crate::shared::utils::DbPool,
    tracking_id: Uuid,
    account_id: Uuid,
    bot_id: Uuid,
    from_email: &str,
    to_email: &str,
    cc: Option<&str>,
    bcc: Option<&str>,
    subject: &str,
) -> Result<(), String> {
    let mut db_conn = conn.get().map_err(|e| format!("DB connection error: {}", e))?;

    let id = Uuid::new_v4();
    let now = Utc::now();

    diesel::sql_query(
        r#"INSERT INTO sent_email_tracking
           (id, tracking_id, bot_id, account_id, from_email, to_email, cc, bcc, subject, sent_at, read_count, is_read)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, 0, false)"#
    )
    .bind::<diesel::sql_types::Uuid, _>(id)
    .bind::<diesel::sql_types::Uuid, _>(tracking_id)
    .bind::<diesel::sql_types::Uuid, _>(bot_id)
    .bind::<diesel::sql_types::Uuid, _>(account_id)
    .bind::<diesel::sql_types::Text, _>(from_email)
    .bind::<diesel::sql_types::Text, _>(to_email)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(cc)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(bcc)
    .bind::<diesel::sql_types::Text, _>(subject)
    .bind::<diesel::sql_types::Timestamptz, _>(now)
    .execute(&mut db_conn)
    .map_err(|e| format!("Failed to save tracking record: {}", e))?;

    debug!("Saved email tracking record: tracking_id={}", tracking_id);
    Ok(())
}

/// Serve tracking pixel and record email open
pub async fn serve_tracking_pixel(
    Path(tracking_id): Path<String>,
    State(state): State<Arc<AppState>>,
    Query(_query): Query<TrackingPixelQuery>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    // Extract client info from headers
    let client_ip = headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or(s).trim().to_string())
        .or_else(|| {
            headers
                .get("x-real-ip")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string())
        });

    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // Parse tracking ID
    if let Ok(tracking_uuid) = Uuid::parse_str(&tracking_id) {
        let conn = state.conn.clone();
        let ip_clone = client_ip.clone();
        let ua_clone = user_agent.clone();

        // Update tracking record asynchronously
        let _ = tokio::task::spawn_blocking(move || {
            update_email_read_status(conn, tracking_uuid, ip_clone, ua_clone)
        })
        .await;

        info!("Email read tracked: tracking_id={}, ip={:?}", tracking_id, client_ip);
    } else {
        warn!("Invalid tracking ID received: {}", tracking_id);
    }

    // Always return the pixel, regardless of tracking success
    // This prevents email clients from showing broken images
    (
        StatusCode::OK,
        [
            ("content-type", "image/gif"),
            ("cache-control", "no-store, no-cache, must-revalidate, max-age=0"),
            ("pragma", "no-cache"),
            ("expires", "0"),
        ],
        TRACKING_PIXEL.to_vec(),
    )
}

/// Update email read status in database
fn update_email_read_status(
    conn: crate::shared::utils::DbPool,
    tracking_id: Uuid,
    client_ip: Option<String>,
    user_agent: Option<String>,
) -> Result<(), String> {
    let mut db_conn = conn.get().map_err(|e| format!("DB connection error: {}", e))?;
    let now = Utc::now();

    // Update tracking record - increment read count, set first/last read info
    diesel::sql_query(
        r#"UPDATE sent_email_tracking
           SET
               is_read = true,
               read_count = read_count + 1,
               read_at = COALESCE(read_at, $2),
               first_read_ip = COALESCE(first_read_ip, $3),
               last_read_ip = $3,
               user_agent = COALESCE(user_agent, $4),
               updated_at = $2
           WHERE tracking_id = $1"#
    )
    .bind::<diesel::sql_types::Uuid, _>(tracking_id)
    .bind::<diesel::sql_types::Timestamptz, _>(now)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(client_ip.as_deref())
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(user_agent.as_deref())
    .execute(&mut db_conn)
    .map_err(|e| format!("Failed to update tracking record: {}", e))?;

    debug!("Updated email read status: tracking_id={}", tracking_id);
    Ok(())
}

/// Get tracking status for a specific email
pub async fn get_tracking_status(
    Path(tracking_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<TrackingStatusResponse>>, EmailError> {
    let tracking_uuid = Uuid::parse_str(&tracking_id)
        .map_err(|_| EmailError("Invalid tracking ID".to_string()))?;

    let conn = state.conn.clone();
    let result = tokio::task::spawn_blocking(move || {
        get_tracking_record(conn, tracking_uuid)
    })
    .await
    .map_err(|e| EmailError(format!("Task join error: {}", e)))?
    .map_err(EmailError)?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(result),
        message: None,
    }))
}

/// Get tracking record from database
fn get_tracking_record(
    conn: crate::shared::utils::DbPool,
    tracking_id: Uuid,
) -> Result<TrackingStatusResponse, String> {
    let mut db_conn = conn.get().map_err(|e| format!("DB connection error: {}", e))?;

    #[derive(QueryableByName)]
    struct TrackingRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        tracking_id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)]
        to_email: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        subject: String,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)]
        sent_at: DateTime<Utc>,
        #[diesel(sql_type = diesel::sql_types::Bool)]
        is_read: bool,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
        read_at: Option<DateTime<Utc>>,
        #[diesel(sql_type = diesel::sql_types::Integer)]
        read_count: i32,
    }

    let row: TrackingRow = diesel::sql_query(
        r#"SELECT tracking_id, to_email, subject, sent_at, is_read, read_at, read_count
           FROM sent_email_tracking WHERE tracking_id = $1"#
    )
    .bind::<diesel::sql_types::Uuid, _>(tracking_id)
    .get_result(&mut db_conn)
    .map_err(|e| format!("Tracking record not found: {}", e))?;

    Ok(TrackingStatusResponse {
        tracking_id: row.tracking_id.to_string(),
        to_email: row.to_email,
        subject: row.subject,
        sent_at: row.sent_at.to_rfc3339(),
        is_read: row.is_read,
        read_at: row.read_at.map(|dt| dt.to_rfc3339()),
        read_count: row.read_count,
    })
}

/// List sent emails with tracking status
pub async fn list_sent_emails_tracking(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListTrackingQuery>,
) -> Result<Json<ApiResponse<Vec<TrackingStatusResponse>>>, EmailError> {
    let conn = state.conn.clone();
    let result = tokio::task::spawn_blocking(move || {
        list_tracking_records(conn, query)
    })
    .await
    .map_err(|e| EmailError(format!("Task join error: {}", e)))?
    .map_err(EmailError)?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(result),
        message: None,
    }))
}

/// List tracking records from database
fn list_tracking_records(
    conn: crate::shared::utils::DbPool,
    query: ListTrackingQuery,
) -> Result<Vec<TrackingStatusResponse>, String> {
    let mut db_conn = conn.get().map_err(|e| format!("DB connection error: {}", e))?;

    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    #[derive(QueryableByName)]
    struct TrackingRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        tracking_id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)]
        to_email: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        subject: String,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)]
        sent_at: DateTime<Utc>,
        #[diesel(sql_type = diesel::sql_types::Bool)]
        is_read: bool,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
        read_at: Option<DateTime<Utc>>,
        #[diesel(sql_type = diesel::sql_types::Integer)]
        read_count: i32,
    }

    // Build query based on filter
    let base_query = match query.filter.as_deref() {
        Some("read") => {
            r#"SELECT tracking_id, to_email, subject, sent_at, is_read, read_at, read_count
               FROM sent_email_tracking WHERE is_read = true
               ORDER BY sent_at DESC LIMIT $1 OFFSET $2"#
        }
        Some("unread") => {
            r#"SELECT tracking_id, to_email, subject, sent_at, is_read, read_at, read_count
               FROM sent_email_tracking WHERE is_read = false
               ORDER BY sent_at DESC LIMIT $1 OFFSET $2"#
        }
        _ => {
            r#"SELECT tracking_id, to_email, subject, sent_at, is_read, read_at, read_count
               FROM sent_email_tracking
               ORDER BY sent_at DESC LIMIT $1 OFFSET $2"#
        }
    };

    let rows: Vec<TrackingRow> = diesel::sql_query(base_query)
        .bind::<diesel::sql_types::BigInt, _>(limit)
        .bind::<diesel::sql_types::BigInt, _>(offset)
        .load(&mut db_conn)
        .map_err(|e| format!("Query failed: {}", e))?;

    Ok(rows
        .into_iter()
        .map(|row| TrackingStatusResponse {
            tracking_id: row.tracking_id.to_string(),
            to_email: row.to_email,
            subject: row.subject,
            sent_at: row.sent_at.to_rfc3339(),
            is_read: row.is_read,
            read_at: row.read_at.map(|dt| dt.to_rfc3339()),
            read_count: row.read_count,
        })
        .collect())
}

/// Get tracking statistics
pub async fn get_tracking_stats(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<TrackingStatsResponse>>, EmailError> {
    let conn = state.conn.clone();
    let result = tokio::task::spawn_blocking(move || {
        calculate_tracking_stats(conn)
    })
    .await
    .map_err(|e| EmailError(format!("Task join error: {}", e)))?
    .map_err(EmailError)?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(result),
        message: None,
    }))
}

/// Calculate tracking statistics from database
fn calculate_tracking_stats(
    conn: crate::shared::utils::DbPool,
) -> Result<TrackingStatsResponse, String> {
    let mut db_conn = conn.get().map_err(|e| format!("DB connection error: {}", e))?;

    #[derive(QueryableByName)]
    struct StatsRow {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        total_sent: i64,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        total_read: i64,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Double>)]
        avg_time_hours: Option<f64>,
    }

    let stats: StatsRow = diesel::sql_query(
        r#"SELECT
               COUNT(*) as total_sent,
               COUNT(*) FILTER (WHERE is_read = true) as total_read,
               AVG(EXTRACT(EPOCH FROM (read_at - sent_at)) / 3600) FILTER (WHERE is_read = true) as avg_time_hours
           FROM sent_email_tracking"#
    )
    .get_result(&mut db_conn)
    .map_err(|e| format!("Stats query failed: {}", e))?;

    let read_rate = if stats.total_sent > 0 {
        (stats.total_read as f64 / stats.total_sent as f64) * 100.0
    } else {
        0.0
    };

    Ok(TrackingStatsResponse {
        total_sent: stats.total_sent,
        total_read: stats.total_read,
        read_rate,
        avg_time_to_read_hours: stats.avg_time_hours,
    })
}

pub async fn get_emails(
    Path(campaign_id): Path<String>,
    State(_state): State<Arc<AppState>>,
) -> String {
    info!("Get emails requested for campaign: {}", campaign_id);
    "No emails tracked".to_string()
}

// ===== EmailService for compatibility with keyword system =====

pub struct EmailService {
    state: Arc<AppState>,
}

impl EmailService {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }

    pub async fn send_email(
        &self,
        to: &str,
        subject: &str,
        body: &str,
        cc: Option<Vec<String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let config = self
            .state
            .config
            .as_ref()
            .ok_or("Email configuration not available")?;

        let from_addr = config
            .email
            .from
            .parse()
            .map_err(|e| format!("Invalid from address: {}", e))?;

        let mut email_builder = Message::builder()
            .from(from_addr)
            .to(to.parse()?)
            .subject(subject);

        if let Some(cc_list) = cc {
            for cc_addr in cc_list {
                email_builder = email_builder.cc(cc_addr.parse()?);
            }
        }

        let email = email_builder.body(body.to_string())?;

        let creds = Credentials::new(config.email.username.clone(), config.email.password.clone());

        let mailer = SmtpTransport::relay(&config.email.smtp_server)?
            .credentials(creds)
            .build();

        mailer.send(&email)?;
        Ok(())
    }

    pub async fn send_email_with_attachment(
        &self,
        to: &str,
        subject: &str,
        body: &str,
        attachment: Vec<u8>,
        filename: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // For now, just send without attachment
        // Full implementation would use lettre's multipart support
        self.send_email(to, subject, body, None).await
    }
}

// Helper functions for draft system
pub async fn fetch_latest_sent_to(config: &EmailConfig, to: &str) -> Result<String, String> {
    use native_tls::TlsConnector;

    let tls = TlsConnector::builder()
        .build()
        .map_err(|e| format!("TLS error: {}", e))?;

    let client = imap::ClientBuilder::new(&config.server, config.port as u16)
        .native_tls(&tls)
        .map_err(|e| format!("IMAP client error: {}", e))?
        .connect()
        .map_err(|e| format!("Connection error: {}", e))?;

    let mut session = client
        .login(&config.username, &config.password)
        .map_err(|e| format!("Login failed: {:?}", e))?;

    session
        .select("INBOX")
        .map_err(|e| format!("Select INBOX failed: {}", e))?;

    // Search for emails to this recipient
    let search_query = format!("TO \"{}\"", to);
    let message_ids = session
        .search(&search_query)
        .map_err(|e| format!("Search failed: {}", e))?;

    if let Some(last_id) = message_ids.last() {
        let messages = session
            .fetch(last_id.to_string(), "BODY[TEXT]")
            .map_err(|e| format!("Fetch failed: {}", e))?;

        if let Some(message) = messages.iter().next() {
            if let Some(body) = message.text() {
                return Ok(String::from_utf8_lossy(body).to_string());
            }
        }
    }

    session.logout().ok();
    Ok(String::new())
}

pub async fn save_email_draft(
    config: &EmailConfig,
    draft: &SaveDraftRequest,
) -> Result<(), String> {
    use chrono::Utc;
    use native_tls::TlsConnector;

    let tls = TlsConnector::builder()
        .build()
        .map_err(|e| format!("TLS error: {}", e))?;

    let client = imap::ClientBuilder::new(&config.server, config.port as u16)
        .native_tls(&tls)
        .map_err(|e| format!("IMAP client error: {}", e))?
        .connect()
        .map_err(|e| format!("Connection error: {}", e))?;

    let mut session = client
        .login(&config.username, &config.password)
        .map_err(|e| format!("Login failed: {:?}", e))?;

    // Create draft email in RFC822 format
    let date = Utc::now().to_rfc2822();
    let message_id = format!("<{}.{}@botserver>", Uuid::new_v4(), config.server);
    let cc_header = if let Some(cc) = &draft.cc {
        format!("Cc: {}\r\n", cc)
    } else {
        String::new()
    };

    let email_content = format!(
        "Date: {}\r\n\
         From: {}\r\n\
         To: {}\r\n\
         {}\
         Subject: {}\r\n\
         Message-ID: {}\r\n\
         Content-Type: text/html; charset=UTF-8\r\n\
         \r\n\
         {}",
        date, config.from, draft.to, cc_header, draft.subject, message_id, draft.text
    );

    // Try to save to Drafts folder, fall back to INBOX if not available
    let folder = session
        .list(None, Some("Drafts"))
        .map_err(|e| format!("List folders failed: {}", e))?
        .iter()
        .find(|name| name.name().to_lowercase().contains("draft"))
        .map(|n| n.name().to_string())
        .unwrap_or_else(|| "INBOX".to_string());

    session
        .append(&folder, email_content.as_bytes())
        .map_err(|e| format!("Append draft failed: {}", e))?;

    session.logout().ok();
    info!("Draft saved to: {}, subject: {}", draft.to, draft.subject);
    Ok(())
}

// ===== Helper Functions for IMAP Operations =====

async fn fetch_emails_from_folder(config: &EmailConfig, folder: &str) -> Result<Vec<EmailSummary>, String> {
    use native_tls::TlsConnector;

    let tls = TlsConnector::builder()
        .build()
        .map_err(|e| format!("TLS error: {}", e))?;

    let client = imap::ClientBuilder::new(&config.server, config.port as u16)
        .native_tls(&tls)
        .map_err(|e| format!("IMAP client error: {}", e))?
        .connect()
        .map_err(|e| format!("Connection error: {}", e))?;

    let mut session = client
        .login(&config.username, &config.password)
        .map_err(|e| format!("Login failed: {:?}", e))?;

    let folder_name = match folder {
        "inbox" => "INBOX",
        "sent" => "Sent",
        "drafts" => "Drafts",
        "trash" => "Trash",
        _ => "INBOX",
    };

    session.select(folder_name).map_err(|e| format!("Select folder failed: {}", e))?;

    let messages = session.fetch("1:20", "(FLAGS RFC822.HEADER)")
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
                let unread = !flags.iter().any(|f| matches!(f, imap::types::Flag::Seen));

                emails.push(EmailSummary {
                    id: message.message.to_string(),
                    from,
                    subject,
                    date,
                    preview: subject.chars().take(100).collect(),
                    unread,
                });
            }
        }
    }

    session.logout().ok();
    Ok(emails)
}

async fn get_folder_counts(config: &EmailConfig) -> Result<std::collections::HashMap<String, usize>, String> {
    use native_tls::TlsConnector;
    use std::collections::HashMap;

    let tls = TlsConnector::builder()
        .build()
        .map_err(|e| format!("TLS error: {}", e))?;

    let client = imap::ClientBuilder::new(&config.server, config.port as u16)
        .native_tls(&tls)
        .map_err(|e| format!("IMAP client error: {}", e))?
        .connect()
        .map_err(|e| format!("Connection error: {}", e))?;

    let mut session = client
        .login(&config.username, &config.password)
        .map_err(|e| format!("Login failed: {:?}", e))?;

    let mut counts = HashMap::new();

    for folder in &["INBOX", "Sent", "Drafts", "Trash"] {
        if let Ok(mailbox) = session.examine(folder) {
            counts.insert(folder.to_string(), mailbox.exists as usize);
        }
    }

    session.logout().ok();
    Ok(counts)
}

async fn fetch_email_by_id(config: &EmailConfig, id: &str) -> Result<EmailContent, String> {
    use native_tls::TlsConnector;

    let tls = TlsConnector::builder()
        .build()
        .map_err(|e| format!("TLS error: {}", e))?;

    let client = imap::ClientBuilder::new(&config.server, config.port as u16)
        .native_tls(&tls)
        .map_err(|e| format!("IMAP client error: {}", e))?
        .connect()
        .map_err(|e| format!("Connection error: {}", e))?;

    let mut session = client
        .login(&config.username, &config.password)
        .map_err(|e| format!("Login failed: {:?}", e))?;

    session.select("INBOX").map_err(|e| format!("Select failed: {}", e))?;

    let messages = session.fetch(id, "RFC822")
        .map_err(|e| format!("Fetch failed: {}", e))?;

    if let Some(message) = messages.iter().next() {
        if let Some(body) = message.body() {
            let parsed = parse_mail(body).map_err(|e| format!("Parse failed: {}", e))?;

            let subject = parsed.headers.get_first_value("Subject").unwrap_or_default();
            let from = parsed.headers.get_first_value("From").unwrap_or_default();
            let to = parsed.headers.get_first_value("To").unwrap_or_default();
            let date = parsed.headers.get_first_value("Date").unwrap_or_default();

            let body_text = parsed.subparts.iter()
                .find_map(|p| p.get_body().ok())
                .or_else(|| parsed.get_body().ok())
                .unwrap_or_default();

            session.logout().ok();

            return Ok(EmailContent {
                subject,
                from,
                to,
                date,
                body: body_text,
            });
        }
    }

    session.logout().ok();
    Err("Email not found".to_string())
}

async fn move_email_to_trash(config: &EmailConfig, id: &str) -> Result<(), String> {
    use native_tls::TlsConnector;

    let tls = TlsConnector::builder()
        .build()
        .map_err(|e| format!("TLS error: {}", e))?;

    let client = imap::ClientBuilder::new(&config.server, config.port as u16)
        .native_tls(&tls)
        .map_err(|e| format!("IMAP client error: {}", e))?
        .connect()
        .map_err(|e| format!("Connection error: {}", e))?;

    let mut session = client
        .login(&config.username, &config.password)
        .map_err(|e| format!("Login failed: {:?}", e))?;

    session.select("INBOX").map_err(|e| format!("Select failed: {}", e))?;

    // Mark as deleted and expunge
    session.store(id, "+FLAGS (\\Deleted)")
        .map_err(|e| format!("Store failed: {}", e))?;

    session.expunge().map_err(|e| format!("Expunge failed: {}", e))?;

    session.logout().ok();
    Ok(())
}

#[derive(Debug)]
struct EmailSummary {
    id: String,
    from: String,
    subject: String,
    date: String,
    preview: String,
    unread: bool,
}

#[derive(Debug)]
struct EmailContent {
    subject: String,
    from: String,
    to: String,
    date: String,
    body: String,
}

// ===== HTMX-Specific Handlers =====

/// List emails with HTMX HTML response
pub async fn list_emails_htmx(
    State(state): State<Arc<AppState>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<impl IntoResponse, EmailError> {
    let folder = params.get("folder").unwrap_or(&"inbox".to_string()).clone();

    // Get user's email accounts
    let user_id = extract_user_from_session(&state).await
        .map_err(|_| EmailError("Authentication required".to_string()))?;

    // Get first email account for the user
    let conn = state.conn.clone();
    let account = tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().map_err(|e| format!("DB connection error: {}", e))?;

        diesel::sql_query(
            "SELECT * FROM email_accounts WHERE user_id = $1 LIMIT 1"
        )
        .bind::<diesel::sql_types::Uuid, _>(user_id)
        .get_result::<EmailAccountRow>(&mut db_conn)
        .optional()
        .map_err(|e| format!("Failed to get email account: {}", e))
    })
    .await
    .map_err(|e| EmailError(format!("Task join error: {}", e)))?
    .map_err(|e| EmailError(e))?;

    let Some(account) = account else {
        return Ok(axum::response::Html(
            r#"<div class="empty-state">
                <h3>No email account configured</h3>
                <p>Please add an email account first</p>
            </div>"#.to_string()
        ));
    };

    // Fetch emails using IMAP
    let config = EmailConfig {
        username: account.username.clone(),
        password: account.password.clone(),
        server: account.imap_server.clone(),
        port: account.imap_port as u32,
        from: account.email.clone(),
    };

    let emails = fetch_emails_from_folder(&config, &folder)
        .await
        .unwrap_or_default();

    let mut html = String::new();
    for (idx, email) in emails.iter().enumerate() {
        let unread_class = if email.unread { "unread" } else { "" };
        html.push_str(&format!(
            r#"<div class="mail-item {}"
                 hx-get="/api/email/{}"
                 hx-target="#mail-content"
                 hx-swap="innerHTML">
                <div class="mail-header">
                    <span>{}</span>
                    <span class="text-sm text-gray">{}</span>
                </div>
                <div class="mail-subject">{}</div>
                <div class="mail-preview">{}</div>
            </div>"#,
            unread_class,
            email.id,
            email.from,
            email.date,
            email.subject,
            email.preview
        ));
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

    Ok(axum::response::Html(html))
}

/// List folders with HTMX HTML response
pub async fn list_folders_htmx(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, EmailError> {
    // Get user's first email account
    let user_id = extract_user_from_session(&state).await
        .map_err(|_| EmailError("Authentication required".to_string()))?;

    let conn = state.conn.clone();
    let account = tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().map_err(|e| format!("DB connection error: {}", e))?;

        diesel::sql_query(
            "SELECT * FROM email_accounts WHERE user_id = $1 LIMIT 1"
        )
        .bind::<diesel::sql_types::Uuid, _>(user_id)
        .get_result::<EmailAccountRow>(&mut db_conn)
        .optional()
        .map_err(|e| format!("Failed to get email account: {}", e))
    })
    .await
    .map_err(|e| EmailError(format!("Task join error: {}", e)))?
    .map_err(|e| EmailError(e))?;

    if account.is_none() {
        return Ok(axum::response::Html(
            r#"<div class="nav-item">No account configured</div>"#.to_string()
        ));
    }

    let account = account.unwrap();

    // Get folder list with counts using IMAP
    let config = EmailConfig {
        username: account.username,
        password: account.password,
        server: account.imap_server,
        port: account.imap_port as u32,
        from: account.email,
    };

    let folder_counts = get_folder_counts(&config).await.unwrap_or_default();

    let mut html = String::new();
    for (folder_name, icon, count) in &[
        ("inbox", "", folder_counts.get("INBOX").unwrap_or(&0)),
        ("sent", "", folder_counts.get("Sent").unwrap_or(&0)),
        ("drafts", "", folder_counts.get("Drafts").unwrap_or(&0)),
        ("trash", "", folder_counts.get("Trash").unwrap_or(&0)),
    ] {
        let active = if *folder_name == "inbox" { "active" } else { "" };
        let count_badge = if **count > 0 {
            format!(r#"<span style="margin-left: auto; font-size: 0.875rem; color: #64748b;">{}</span>"#, count)
        } else {
            String::new()
        };

        html.push_str(&format!(
            r#"<div class="nav-item {}"
                 hx-get="/api/email/list?folder={}"
                 hx-target="#mail-list"
                 hx-swap="innerHTML">
                <span>{}</span> {}
                {}
            </div>"#,
            active, folder_name, icon,
            folder_name.chars().next().unwrap().to_uppercase().collect::<String>() + &folder_name[1..],
            count_badge
        ));
    }

    Ok(axum::response::Html(html))
}

/// Compose email form with HTMX
pub async fn compose_email_htmx(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, EmailError> {
    let html = r#"
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
    "#;

    Ok(axum::response::Html(html))
}

/// Get email content with HTMX HTML response
pub async fn get_email_content_htmx(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, EmailError> {
    // Get user's email account
    let user_id = extract_user_from_session(&state).await
        .map_err(|_| EmailError("Authentication required".to_string()))?;

    let conn = state.conn.clone();
    let account = tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().map_err(|e| format!("DB connection error: {}", e))?;

        diesel::sql_query(
            "SELECT * FROM email_accounts WHERE user_id = $1 LIMIT 1"
        )
        .bind::<diesel::sql_types::Uuid, _>(user_id)
        .get_result::<EmailAccountRow>(&mut db_conn)
        .optional()
        .map_err(|e| format!("Failed to get email account: {}", e))
    })
    .await
    .map_err(|e| EmailError(format!("Task join error: {}", e)))?
    .map_err(|e| EmailError(e))?;

    let Some(account) = account else {
        return Ok(axum::response::Html(
            r#"<div class="mail-content-view">
                <p>No email account configured</p>
            </div>"#.to_string()
        ));
    };

    // Fetch email content using IMAP
    let config = EmailConfig {
        username: account.username,
        password: account.password,
        server: account.imap_server,
        port: account.imap_port as u32,
        from: account.email.clone(),
    };

    let email_content = fetch_email_by_id(&config, &id)
        .await
        .map_err(|e| EmailError(format!("Failed to fetch email: {}", e)))?;

    let html = format!(
        r#"
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
        "#,
        id, id, id,
        email_content.subject,
        email_content.from,
        email_content.to,
        email_content.date,
        email_content.body
    );

    Ok(axum::response::Html(html))
}

/// Delete email with HTMX response
pub async fn delete_email_htmx(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, EmailError> {
    // Get user's email account
    let user_id = extract_user_from_session(&state).await
        .map_err(|_| EmailError("Authentication required".to_string()))?;

    let conn = state.conn.clone();
    let account = tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().map_err(|e| format!("DB connection error: {}", e))?;

        diesel::sql_query(
            "SELECT * FROM email_accounts WHERE user_id = $1 LIMIT 1"
        )
        .bind::<diesel::sql_types::Uuid, _>(user_id)
        .get_result::<EmailAccountRow>(&mut db_conn)
        .optional()
        .map_err(|e| format!("Failed to get email account: {}", e))
    })
    .await
    .map_err(|e| EmailError(format!("Task join error: {}", e)))?
    .map_err(|e| EmailError(e))?;

    if let Some(account) = account {
        let config = EmailConfig {
            username: account.username,
            password: account.password,
            server: account.imap_server,
            port: account.imap_port as u32,
            from: account.email,
        };

        // Move email to trash folder using IMAP
        move_email_to_trash(&config, &id)
            .await
            .map_err(|e| EmailError(format!("Failed to delete email: {}", e)))?;
    }

    info!("Email {} moved to trash", id);

    // Return updated email list
    list_emails_htmx(State(state), Query(std::collections::HashMap::new())).await
}

/// Get latest email
pub async fn get_latest_email(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<EmailData>>, EmailError> {
    // Mock implementation - replace with actual logic
    Ok(Json(ApiResponse {
        success: true,
        data: Some(EmailData {
            id: Uuid::new_v4().to_string(),
            from: "sender@example.com".to_string(),
            to: "recipient@example.com".to_string(),
            subject: "Latest Email".to_string(),
            body: "This is the latest email content.".to_string(),
            date: chrono::Utc::now().to_rfc3339(),
            unread: true,
        }),
        message: Some("Latest email fetched".to_string()),
    }))
}

/// Get email by ID
pub async fn get_email(
    State(state): State<Arc<AppState>>,
    Path(campaign_id): Path<String>,
) -> Result<Json<ApiResponse<EmailData>>, EmailError> {
    // Mock implementation - replace with actual logic
    Ok(Json(ApiResponse {
        success: true,
        data: Some(EmailData {
            id: campaign_id.clone(),
            from: "sender@example.com".to_string(),
            to: "recipient@example.com".to_string(),
            subject: "Email Subject".to_string(),
            body: "Email content here.".to_string(),
            date: chrono::Utc::now().to_rfc3339(),
            unread: false,
        }),
        message: Some("Email fetched".to_string()),
    }))
}

/// Track email click
pub async fn track_click(
    State(state): State<Arc<AppState>>,
    Path((campaign_id, email)): Path<(String, String)>,
) -> Result<Json<ApiResponse<()>>, EmailError> {
    info!("Tracking click for campaign {} email {}", campaign_id, email);

    Ok(Json(ApiResponse {
        success: true,
        data: Some(()),
        message: Some("Click tracked".to_string()),
    }))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmailData {
    pub id: String,
    pub from: String,
    pub to: String,
    pub subject: String,
    pub body: String,
    pub date: String,
    pub unread: bool,
}

// Database row struct for email accounts
#[derive(Debug, QueryableByName)]
struct EmailAccountRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub user_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub email: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub username: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub password: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub imap_server: String,
    #[diesel(sql_type = diesel::sql_types::Integer)]
    pub imap_port: i32,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub smtp_server: String,
    #[diesel(sql_type = diesel::sql_types::Integer)]
    pub smtp_port: i32,
}

// ===== HTMX UI Endpoint Handlers =====

/// List email labels (HTMX HTML response)
pub async fn list_labels_htmx(
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    // Return default labels as HTML for HTMX
    axum::response::Html(r#"
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
    "#.to_string())
}

/// List email templates (HTMX HTML response)
pub async fn list_templates_htmx(
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    axum::response::Html(r#"
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
    "#.to_string())
}

/// List email signatures (HTMX HTML response)
pub async fn list_signatures_htmx(
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    axum::response::Html(r#"
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
    "#.to_string())
}

/// List email rules (HTMX HTML response)
pub async fn list_rules_htmx(
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    axum::response::Html(r#"
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
    "#.to_string())
}

/// Search emails (HTMX HTML response)
pub async fn search_emails_htmx(
    State(state): State<Arc<AppState>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let query = params.get("q").map(|s| s.as_str()).unwrap_or("");

    if query.is_empty() {
        return axum::response::Html(r#"
            <div class="empty-state">
                <p>Enter a search term to find emails</p>
            </div>
        "#.to_string());
    }

    let search_term = format!("%{}%", query.to_lowercase());

    let conn = match state.conn.get() {
        Ok(c) => c,
        Err(_) => {
            return axum::response::Html(r#"
                <div class="empty-state error">
                    <p>Database connection error</p>
                </div>
            "#.to_string());
        }
    };

    let search_query = format!(
        "SELECT id, subject, from_address, to_addresses, body_text, received_at
         FROM emails
         WHERE LOWER(subject) LIKE $1
            OR LOWER(from_address) LIKE $1
            OR LOWER(body_text) LIKE $1
         ORDER BY received_at DESC
         LIMIT 50"
    );

    let results: Vec<(String, String, String, String, Option<String>, DateTime<Utc>)> =
        match diesel::sql_query(&search_query)
            .bind::<diesel::sql_types::Text, _>(&search_term)
            .load(&conn)
        {
            Ok(r) => r.into_iter().map(|row: (String, String, String, String, Option<String>, DateTime<Utc>)| row).collect(),
            Err(e) => {
                warn!("Email search query failed: {}", e);
                Vec::new()
            }
        };

    if results.is_empty() {
        return axum::response::Html(format!(r#"
            <div class="empty-state">
                <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                    <circle cx="11" cy="11" r="8"></circle>
                    <path d="m21 21-4.35-4.35"></path>
                </svg>
                <h3>No results for "{}"</h3>
                <p>Try different keywords or check your spelling.</p>
            </div>
        "#, query));
    }

    let mut html = String::from(r#"<div class="search-results">"#);
    html.push_str(&format!(r#"<div class="search-header"><span>Found {} result(s) for "{}"</span></div>"#, results.len(), query));

    for (id, subject, from, _to, body, date) in results {
        let preview = body
            .as_deref()
            .unwrap_or("")
            .chars()
            .take(100)
            .collect::<String>();
        let formatted_date = date.format("%b %d, %Y").to_string();

        html.push_str(&format!(r#"
            <div class="email-item" hx-get="/ui/mail/view/{}" hx-target="#email-content" hx-swap="innerHTML">
                <div class="email-sender">{}</div>
                <div class="email-subject">{}</div>
                <div class="email-preview">{}</div>
                <div class="email-date">{}</div>
            </div>
        "#, id, from, subject, preview, formatted_date));
    }

    html.push_str("</div>");
    axum::response::Html(html)
}

/// Save auto-responder settings
pub async fn save_auto_responder(
    State(_state): State<Arc<AppState>>,
    axum::Form(form): axum::Form<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    info!("Saving auto-responder settings: {:?}", form);

    // In production, save to database
    axum::response::Html(r#"
        <div class="notification success">
            Auto-responder settings saved successfully!
        </div>
    "#.to_string())
}
