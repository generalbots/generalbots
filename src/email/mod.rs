use crate::{config::EmailConfig, shared::state::AppState};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use base64::{engine::general_purpose, Engine as _};
use diesel::prelude::*;
use imap::types::Seq;
use lettre::{transport::smtp::authentication::Credentials, Message, SmtpTransport, Transport};
use log::{error, info};
use mailparse::{parse_mail, MailHeaderMap};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

// Export SaveDraftRequest for other modules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveDraftRequest {
    pub to: String,
    pub subject: String,
    pub cc: Option<String>,
    pub text: String,
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

#[derive(Debug, Deserialize)]
pub struct SendEmailRequest {
    pub account_id: String,
    pub to: String,
    pub cc: Option<String>,
    pub bcc: Option<String>,
    pub subject: String,
    pub body: String,
    pub is_html: bool,
}

#[derive(Debug, Deserialize)]
pub struct SaveDraftRequest {
    pub account_id: String,
    pub to: String,
    pub cc: Option<String>,
    pub bcc: Option<String>,
    pub subject: String,
    pub body: String,
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
    // TODO: Get user_id from session/token authentication
    let user_id = Uuid::nil(); // Placeholder - implement proper auth

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

pub async fn list_email_accounts(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<Vec<EmailAccountResponse>>>, EmailError> {
    // TODO: Get user_id from session/token authentication
    let user_id = Uuid::nil(); // Placeholder

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

    let client = imap::connect(
        (imap_server.as_str(), imap_port as u16),
        imap_server.as_str(),
        &tls,
    )
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
                read: false, // TODO: Check IMAP flags
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
        .subject(request.subject);

    if let Some(cc) = request.cc {
        email_builder = email_builder.cc(cc
            .parse()
            .map_err(|e| EmailError(format!("Invalid cc address: {}", e)))?);
    }

    if let Some(bcc) = request.bcc {
        email_builder = email_builder.bcc(
            bcc.parse()
                .map_err(|e| EmailError(format!("Invalid bcc address: {}", e)))?,
        );
    }

    let email = email_builder
        .body(request.body)
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

    info!("Email sent successfully from account {}", account_uuid);

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

    // TODO: Get user_id from session
    let user_id = Uuid::nil();
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

    let client = imap::connect(
        (imap_server.as_str(), imap_port as u16),
        imap_server.as_str(),
        &tls,
    )
    .map_err(|e| EmailError(format!("IMAP connection error: {:?}", e)))?;

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
            unread_count: 0, // TODO: Query actual counts
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
    // This would fetch the latest email sent to the recipient
    // For threading/reply purposes
    // For now, return empty string
    Ok(String::new())
}

pub async fn save_email_draft(
    config: &EmailConfig,
    draft: &SaveDraftRequest,
) -> Result<(), String> {
    // This would save the draft to the email server or local storage
    // For now, just log and return success
    info!("Saving draft to: {}, subject: {}", draft.to, draft.subject);
    Ok(())
}
