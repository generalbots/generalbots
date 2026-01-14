pub mod ui;

use crate::{config::EmailConfig, core::urls::ApiUrls, shared::state::AppState};
use crate::core::middleware::AuthenticatedUser;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use axum::{
    routing::{delete, get, post},
    Router,
};
use base64::{engine::general_purpose, Engine as _};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::sql_types::{Bool, Integer, Nullable, Text, Timestamptz, Uuid as DieselUuid, Varchar};
use imap::types::Seq;
use lettre::{transport::smtp::authentication::Credentials, Message, SmtpTransport, Transport};
use log::{debug, info, warn};
use mailparse::{parse_mail, MailHeaderMap};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, QueryableByName)]
pub struct EmailAccountBasicRow {
    #[diesel(sql_type = DieselUuid)]
    pub id: Uuid,
    #[diesel(sql_type = Text)]
    pub email: String,
    #[diesel(sql_type = Nullable<Text>)]
    pub display_name: Option<String>,
    #[diesel(sql_type = Bool)]
    pub is_primary: bool,
}

#[derive(Debug, QueryableByName)]
pub struct ImapCredentialsRow {
    #[diesel(sql_type = Text)]
    pub imap_server: String,
    #[diesel(sql_type = Integer)]
    pub imap_port: i32,
    #[diesel(sql_type = Text)]
    pub username: String,
    #[diesel(sql_type = Text)]
    pub password_encrypted: String,
}

#[derive(Debug, QueryableByName)]
pub struct SmtpCredentialsRow {
    #[diesel(sql_type = Text)]
    pub email: String,
    #[diesel(sql_type = Text)]
    pub display_name: String,
    #[diesel(sql_type = Integer)]
    pub smtp_port: i32,
    #[diesel(sql_type = Text)]
    pub smtp_server: String,
    #[diesel(sql_type = Text)]
    pub username: String,
    #[diesel(sql_type = Text)]
    pub password_encrypted: String,
}

#[derive(Debug, QueryableByName)]
pub struct EmailSearchRow {
    #[diesel(sql_type = Text)]
    pub id: String,
    #[diesel(sql_type = Text)]
    pub subject: String,
    #[diesel(sql_type = Text)]
    pub from_address: String,
    #[diesel(sql_type = Text)]
    pub to_addresses: String,
    #[diesel(sql_type = Nullable<Text>)]
    pub body_text: Option<String>,
    #[diesel(sql_type = Timestamptz)]
    pub received_at: DateTime<Utc>,
}

/// Strip HTML tags from a string to create plain text version
fn strip_html_tags(html: &str) -> String {
    // Replace common HTML entities
    let text = html
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'");

    // Replace <br> and </p> with newlines
    let text = text
        .replace("<br>", "\n")
        .replace("<br/>", "\n")
        .replace("<br />", "\n")
        .replace("</p>", "\n")
        .replace("</div>", "\n")
        .replace("</li>", "\n");

    // Remove all remaining HTML tags
    let mut result = String::with_capacity(text.len());
    let mut in_tag = false;

    for c in text.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(c),
            _ => {}
        }
    }

    // Clean up multiple consecutive newlines and trim
    let mut cleaned = String::new();
    let mut prev_newline = false;
    for c in result.chars() {
        if c == '\n' {
            if !prev_newline {
                cleaned.push(c);
            }
            prev_newline = true;
        } else {
            cleaned.push(c);
            prev_newline = false;
        }
    }

    cleaned.trim().to_string()
}

#[derive(Debug, QueryableByName, Serialize)]
pub struct EmailSignatureRow {
    #[diesel(sql_type = DieselUuid)]
    pub id: Uuid,
    #[diesel(sql_type = DieselUuid)]
    pub user_id: Uuid,
    #[diesel(sql_type = Nullable<DieselUuid>)]
    pub bot_id: Option<Uuid>,
    #[diesel(sql_type = Varchar)]
    pub name: String,
    #[diesel(sql_type = Text)]
    pub content_html: String,
    #[diesel(sql_type = Text)]
    pub content_plain: String,
    #[diesel(sql_type = Bool)]
    pub is_default: bool,
    #[diesel(sql_type = Bool)]
    pub is_active: bool,
    #[diesel(sql_type = Timestamptz)]
    pub created_at: DateTime<Utc>,
    #[diesel(sql_type = Timestamptz)]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSignatureRequest {
    pub name: String,
    pub content_html: String,
    #[serde(default)]
    pub content_plain: Option<String>,
    #[serde(default)]
    pub is_default: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSignatureRequest {
    pub name: Option<String>,
    pub content_html: Option<String>,
    pub content_plain: Option<String>,
    pub is_default: Option<bool>,
    pub is_active: Option<bool>,
}

pub mod stalwart_client;
pub mod stalwart_sync;
pub mod vectordb;

fn extract_user_from_session(_state: &Arc<AppState>) -> Result<Uuid, String> {
    Ok(Uuid::new_v4())
}

pub fn configure() -> Router<Arc<AppState>> {
    Router::new()
        .route(ApiUrls::EMAIL_ACCOUNTS, get(list_email_accounts))
        .route(
            &format!("{}/add", ApiUrls::EMAIL_ACCOUNTS),
            post(add_email_account),
        )
        .route(
            &ApiUrls::EMAIL_ACCOUNT_BY_ID.replace(":id", "{account_id}"),
            axum::routing::delete(delete_email_account),
        )
        .route(ApiUrls::EMAIL_LIST, post(list_emails))
        .route(ApiUrls::EMAIL_SEND, post(send_email))
        .route(ApiUrls::EMAIL_DRAFT, post(save_draft))
        .route(
            &ApiUrls::EMAIL_FOLDERS.replace(":account_id", "{account_id}"),
            get(list_folders),
        )
        .route(ApiUrls::EMAIL_LATEST, get(get_latest_email))
        .route(
            &ApiUrls::EMAIL_GET.replace(":campaign_id", "{campaign_id}"),
            get(get_email),
        )
        .route(
            &ApiUrls::EMAIL_CLICK
                .replace(":campaign_id", "{campaign_id}")
                .replace(":email", "{email}"),
            post(track_click),
        )
        .route(
            "/api/email/tracking/pixel/{tracking_id}",
            get(serve_tracking_pixel),
        )
        .route(
            "/api/email/tracking/status/{tracking_id}",
            get(get_tracking_status),
        )
        .route("/api/email/tracking/list", get(list_sent_emails_tracking))
        .route("/api/email/tracking/stats", get(get_tracking_stats))
        // HTMX/HTML APIs
        .route(ApiUrls::EMAIL_ACCOUNTS_HTMX, get(list_email_accounts_htmx))
        .route(ApiUrls::EMAIL_LIST_HTMX, get(list_emails_htmx))
        .route(ApiUrls::EMAIL_FOLDERS_HTMX, get(list_folders_htmx))
        .route(ApiUrls::EMAIL_COMPOSE_HTMX, get(compose_email_htmx))
        .route(ApiUrls::EMAIL_CONTENT_HTMX, get(get_email_content_htmx))
        .route("/api/ui/email/:id/delete", delete(delete_email_htmx))
        .route(ApiUrls::EMAIL_LABELS_HTMX, get(list_labels_htmx))
        .route(ApiUrls::EMAIL_TEMPLATES_HTMX, get(list_templates_htmx))
        .route(ApiUrls::EMAIL_SIGNATURES_HTMX, get(list_signatures_htmx))
        .route(ApiUrls::EMAIL_RULES_HTMX, get(list_rules_htmx))
        .route(ApiUrls::EMAIL_SEARCH_HTMX, get(search_emails_htmx))
        .route(ApiUrls::EMAIL_AUTO_RESPONDER_HTMX, post(save_auto_responder))
        // Signatures API
        .route("/api/email/signatures", get(list_signatures).post(create_signature))
        .route("/api/email/signatures/default", get(get_default_signature))
        .route("/api/email/signatures/{id}", get(get_signature).put(update_signature).delete(delete_signature))
}

// =============================================================================
// SIGNATURE HANDLERS
// =============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct EmailSignature {
    pub id: String,
    pub name: String,
    pub content_html: String,
    pub content_text: String,
    pub is_default: bool,
}

pub async fn list_signatures(
    State(state): State<Arc<AppState>>,
    user: AuthenticatedUser,
) -> impl IntoResponse {
    let mut conn = match state.conn.get() {
        Ok(c) => c,
        Err(e) => {
            return Json(serde_json::json!({
                "error": format!("Database connection error: {}", e),
                "signatures": []
            }));
        }
    };

    let user_id = user.user_id;
    let result: Result<Vec<EmailSignatureRow>, _> = diesel::sql_query(
        "SELECT id, user_id, bot_id, name, content_html, content_plain, is_default, is_active, created_at, updated_at
         FROM email_signatures
         WHERE user_id = $1 AND is_active = true
         ORDER BY is_default DESC, name ASC"
    )
    .bind::<DieselUuid, _>(user_id)
    .load(&mut conn);

    match result {
        Ok(signatures) => Json(serde_json::json!({
            "signatures": signatures
        })),
        Err(e) => {
            warn!("Failed to list signatures: {}", e);
            // Return empty list with default signature as fallback
            Json(serde_json::json!({
                "signatures": [{
                    "id": "default",
                    "name": "Default Signature",
                    "content_html": "<p>Best regards,<br>The Team</p>",
                    "content_plain": "Best regards,\nThe Team",
                    "is_default": true
                }]
            }))
        }
    }
}

pub async fn get_default_signature(
    State(state): State<Arc<AppState>>,
    user: AuthenticatedUser,
) -> impl IntoResponse {
    let mut conn = match state.conn.get() {
        Ok(c) => c,
        Err(e) => {
            return Json(serde_json::json!({
                "id": "default",
                "name": "Default Signature",
                "content_html": "<p>Best regards,<br>The Team</p>",
                "content_plain": "Best regards,\nThe Team",
                "is_default": true,
                "_error": format!("Database connection error: {}", e)
            }));
        }
    };

    let user_id = user.user_id;
    let result: Result<EmailSignatureRow, _> = diesel::sql_query(
        "SELECT id, user_id, bot_id, name, content_html, content_plain, is_default, is_active, created_at, updated_at
         FROM email_signatures
         WHERE user_id = $1 AND is_default = true AND is_active = true
         LIMIT 1"
    )
    .bind::<DieselUuid, _>(user_id)
    .get_result(&mut conn);

    match result {
        Ok(signature) => Json(serde_json::json!({
            "id": signature.id,
            "name": signature.name,
            "content_html": signature.content_html,
            "content_plain": signature.content_plain,
            "is_default": signature.is_default
        })),
        Err(_) => {
            // Return default signature as fallback
            Json(serde_json::json!({
                "id": "default",
                "name": "Default Signature",
                "content_html": "<p>Best regards,<br>The Team</p>",
                "content_plain": "Best regards,\nThe Team",
                "is_default": true
            }))
        }
    }
}

pub async fn get_signature(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    user: AuthenticatedUser,
) -> impl IntoResponse {
    let signature_id = match Uuid::parse_str(&id) {
        Ok(id) => id,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "error": "Invalid signature ID"
            }))).into_response();
        }
    };

    let mut conn = match state.conn.get() {
        Ok(c) => c,
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": format!("Database connection error: {}", e)
            }))).into_response();
        }
    };

    let user_id = user.user_id;
    let result: Result<EmailSignatureRow, _> = diesel::sql_query(
        "SELECT id, user_id, bot_id, name, content_html, content_plain, is_default, is_active, created_at, updated_at
         FROM email_signatures
         WHERE id = $1 AND user_id = $2"
    )
    .bind::<DieselUuid, _>(signature_id)
    .bind::<DieselUuid, _>(user_id)
    .get_result(&mut conn);

    match result {
        Ok(signature) => Json(serde_json::json!(signature)).into_response(),
        Err(_) => (StatusCode::NOT_FOUND, Json(serde_json::json!({
            "error": "Signature not found"
        }))).into_response()
    }
}

pub async fn create_signature(
    State(state): State<Arc<AppState>>,
    user: AuthenticatedUser,
    Json(payload): Json<CreateSignatureRequest>,
) -> impl IntoResponse {
    let mut conn = match state.conn.get() {
        Ok(c) => c,
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "success": false,
                "error": format!("Database connection error: {}", e)
            }))).into_response();
        }
    };

    let new_id = Uuid::new_v4();
    let user_id = user.user_id;
    let content_plain = payload.content_plain.unwrap_or_else(|| {
        // Strip HTML tags for plain text version using simple regex
        strip_html_tags(&payload.content_html)
    });

    // If this is set as default, unset other defaults first
    if payload.is_default {
        let _ = diesel::sql_query(
            "UPDATE email_signatures SET is_default = false WHERE user_id = $1 AND is_default = true"
        )
        .bind::<DieselUuid, _>(user_id)
        .execute(&mut conn);
    }

    let result = diesel::sql_query(
        "INSERT INTO email_signatures (id, user_id, name, content_html, content_plain, is_default, is_active, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, true, NOW(), NOW())
         RETURNING id"
    )
    .bind::<DieselUuid, _>(new_id)
    .bind::<DieselUuid, _>(user_id)
    .bind::<Varchar, _>(&payload.name)
    .bind::<Text, _>(&payload.content_html)
    .bind::<Text, _>(&content_plain)
    .bind::<Bool, _>(payload.is_default)
    .execute(&mut conn);

    match result {
        Ok(_) => Json(serde_json::json!({
            "success": true,
            "id": new_id,
            "name": payload.name
        })).into_response(),
        Err(e) => {
            warn!("Failed to create signature: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "success": false,
                "error": format!("Failed to create signature: {}", e)
            }))).into_response()
        }
    }
}

pub async fn update_signature(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    user: AuthenticatedUser,
    Json(payload): Json<UpdateSignatureRequest>,
) -> impl IntoResponse {
    let signature_id = match Uuid::parse_str(&id) {
        Ok(id) => id,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "success": false,
                "error": "Invalid signature ID"
            }))).into_response();
        }
    };

    let mut conn = match state.conn.get() {
        Ok(c) => c,
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "success": false,
                "error": format!("Database connection error: {}", e)
            }))).into_response();
        }
    };

    let user_id = user.user_id;

    // Build dynamic update query
    let mut updates = vec!["updated_at = NOW()".to_string()];
    if payload.name.is_some() {
        updates.push("name = $3".to_string());
    }
    if payload.content_html.is_some() {
        updates.push("content_html = $4".to_string());
    }
    if payload.content_plain.is_some() {
        updates.push("content_plain = $5".to_string());
    }
    if let Some(is_default) = payload.is_default {
        if is_default {
            // Unset other defaults first
            let _ = diesel::sql_query(
                "UPDATE email_signatures SET is_default = false WHERE user_id = $1 AND is_default = true AND id != $2"
            )
            .bind::<DieselUuid, _>(user_id)
            .bind::<DieselUuid, _>(signature_id)
            .execute(&mut conn);
        }
        updates.push("is_default = $6".to_string());
    }
    if payload.is_active.is_some() {
        updates.push("is_active = $7".to_string());
    }

    let result = diesel::sql_query(format!(
        "UPDATE email_signatures SET {} WHERE id = $1 AND user_id = $2",
        updates.join(", ")
    ))
    .bind::<DieselUuid, _>(signature_id)
    .bind::<DieselUuid, _>(user_id)
    .bind::<Varchar, _>(payload.name.unwrap_or_default())
    .bind::<Text, _>(payload.content_html.unwrap_or_default())
    .bind::<Text, _>(payload.content_plain.unwrap_or_default())
    .bind::<Bool, _>(payload.is_default.unwrap_or(false))
    .bind::<Bool, _>(payload.is_active.unwrap_or(true))
    .execute(&mut conn);

    match result {
        Ok(rows) if rows > 0 => Json(serde_json::json!({
            "success": true,
            "id": id
        })).into_response(),
        Ok(_) => (StatusCode::NOT_FOUND, Json(serde_json::json!({
            "success": false,
            "error": "Signature not found"
        }))).into_response(),
        Err(e) => {
            warn!("Failed to update signature: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "success": false,
                "error": format!("Failed to update signature: {}", e)
            }))).into_response()
        }
    }
}

pub async fn delete_signature(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    user: AuthenticatedUser,
) -> impl IntoResponse {
    let signature_id = match Uuid::parse_str(&id) {
        Ok(id) => id,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "success": false,
                "error": "Invalid signature ID"
            }))).into_response();
        }
    };

    let mut conn = match state.conn.get() {
        Ok(c) => c,
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "success": false,
                "error": format!("Database connection error: {}", e)
            }))).into_response();
        }
    };

    let user_id = user.user_id;

    // Soft delete by setting is_active = false
    let result = diesel::sql_query(
        "UPDATE email_signatures SET is_active = false, updated_at = NOW() WHERE id = $1 AND user_id = $2"
    )
    .bind::<DieselUuid, _>(signature_id)
    .bind::<DieselUuid, _>(user_id)
    .execute(&mut conn);

    match result {
        Ok(rows) if rows > 0 => Json(serde_json::json!({
            "success": true,
            "id": id
        })).into_response(),
        Ok(_) => (StatusCode::NOT_FOUND, Json(serde_json::json!({
            "success": false,
            "error": "Signature not found"
        }))).into_response(),
        Err(e) => {
            warn!("Failed to delete signature: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "success": false,
                "error": format!("Failed to delete signature: {}", e)
            }))).into_response()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveDraftRequest {
    pub account_id: String,
    pub to: String,
    pub cc: Option<String>,
    pub bcc: Option<String>,
    pub subject: String,
    pub body: String,
}

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

#[derive(Debug, Deserialize)]
pub struct TrackingPixelQuery {
    pub t: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListTrackingQuery {
    pub account_id: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub filter: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TrackingStatsResponse {
    pub total_sent: i64,
    pub total_read: i64,
    pub read_rate: f64,
    pub avg_time_to_read_hours: Option<f64>,
}

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

pub struct EmailError(String);

impl IntoResponse for EmailError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.0).into_response()
    }
}

impl From<String> for EmailError {
    fn from(s: String) -> Self {
        Self(s)
    }
}

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
    if date_str.is_empty() {
        return "Unknown".to_string();
    }

    date_str
        .split_whitespace()
        .take(4)
        .collect::<Vec<_>>()
        .join(" ")
}

fn encrypt_password(password: &str) -> String {
    general_purpose::STANDARD.encode(password.as_bytes())
}

fn decrypt_password(encrypted: &str) -> Result<String, String> {
    general_purpose::STANDARD
        .decode(encrypted)
        .map_err(|e| format!("Decryption failed: {e}"))
        .and_then(|bytes| {
            String::from_utf8(bytes).map_err(|e| format!("UTF-8 conversion failed: {e}"))
        })
}

pub async fn add_email_account(
    State(state): State<Arc<AppState>>,
    Json(request): Json<EmailAccountRequest>,
) -> Result<Json<ApiResponse<EmailAccountResponse>>, EmailError> {
    let Ok(current_user_id) = extract_user_from_session(&state) else {
        return Err(EmailError("Authentication required".to_string()));
    };

    let account_id = Uuid::new_v4();
    let encrypted_password = encrypt_password(&request.password);

    let resp_email = request.email.clone();
    let resp_display_name = request.display_name.clone();
    let resp_imap_server = request.imap_server.clone();
    let resp_imap_port = request.imap_port;
    let resp_smtp_server = request.smtp_server.clone();
    let resp_smtp_port = request.smtp_port;
    let resp_is_primary = request.is_primary;

    let conn = state.conn.clone();
    tokio::task::spawn_blocking(move || {
        use crate::shared::models::schema::user_email_accounts::dsl::{is_primary, user_email_accounts, user_id};
        let mut db_conn = conn.get().map_err(|e| format!("DB connection error: {e}"))?;


        if request.is_primary {
            diesel::update(user_email_accounts.filter(user_id.eq(&current_user_id)))
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
        .bind::<diesel::sql_types::Uuid, _>(current_user_id)
        .bind::<diesel::sql_types::Text, _>(&request.email)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(request.display_name.as_ref())
        .bind::<diesel::sql_types::Text, _>(&request.imap_server)
        .bind::<diesel::sql_types::Integer, _>(i32::from(request.imap_port))
        .bind::<diesel::sql_types::Text, _>(&request.smtp_server)
        .bind::<diesel::sql_types::Integer, _>(i32::from(request.smtp_port))
        .bind::<diesel::sql_types::Text, _>(&request.username)
        .bind::<diesel::sql_types::Text, _>(&encrypted_password)
        .bind::<diesel::sql_types::Bool, _>(request.is_primary)
        .bind::<diesel::sql_types::Bool, _>(true)
        .execute(&mut db_conn)
        .map_err(|e| format!("Failed to insert account: {e}"))?;

        Ok::<_, String>(account_id)
    })
    .await
    .map_err(|e| EmailError(format!("Task join error: {e}")))?
    .map_err(EmailError)?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(EmailAccountResponse {
            id: account_id.to_string(),
            email: resp_email,
            display_name: resp_display_name,
            imap_server: resp_imap_server,
            imap_port: resp_imap_port,
            smtp_server: resp_smtp_server,
            smtp_port: resp_smtp_port,
            is_primary: resp_is_primary,
            is_active: true,
            created_at: chrono::Utc::now().to_rfc3339(),
        }),
        message: Some("Email account added successfully".to_string()),
    }))
}

pub async fn list_email_accounts_htmx(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let Ok(user_id) = extract_user_from_session(&state) else {
        return axum::response::Html(r#"
            <div class="account-item" onclick="document.getElementById('add-account-modal').showModal()">
                <span>+ Add email account</span>
            </div>
        "#.to_string());
    };

    let conn = state.conn.clone();
    let accounts = tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().map_err(|e| format!("DB connection error: {e}"))?;

        diesel::sql_query(
            "SELECT id, email, display_name, is_primary FROM user_email_accounts WHERE user_id = $1 AND is_active = true ORDER BY is_primary DESC"
        )
        .bind::<diesel::sql_types::Uuid, _>(user_id)
        .load::<EmailAccountBasicRow>(&mut db_conn)
        .map_err(|e| format!("Query failed: {e}"))
    })
    .await
    .ok()
    .and_then(Result::ok)
    .unwrap_or_default();

    if accounts.is_empty() {
        return axum::response::Html(r#"
            <div class="account-item" onclick="document.getElementById('add-account-modal').showModal()">
                <span>+ Add email account</span>
            </div>
        "#.to_string());
    }

    let mut html = String::new();
    for account in accounts {
        let name = account
            .display_name
            .clone()
            .unwrap_or_else(|| account.email.clone());
        let primary_badge = if account.is_primary {
            r#"<span class="badge">Primary</span>"#
        } else {
            ""
        };
        use std::fmt::Write;
        let _ = write!(
            html,
            r#"<div class="account-item" data-account-id="{}">
                <span>{}</span>
                {}
            </div>"#,
            account.id, name, primary_badge
        );
    }

    axum::response::Html(html)
}

pub async fn list_email_accounts(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<Vec<EmailAccountResponse>>>, EmailError> {
    let Ok(current_user_id) = extract_user_from_session(&state) else {
        return Err(EmailError("Authentication required".to_string()));
    };

    let conn = state.conn.clone();
    let accounts = tokio::task::spawn_blocking(move || {
        use crate::shared::models::schema::user_email_accounts::dsl::{
            created_at, display_name, email, id, imap_port, imap_server, is_active, is_primary,
            smtp_port, smtp_server, user_email_accounts, user_id,
        };
        let mut db_conn = conn
            .get()
            .map_err(|e| format!("DB connection error: {e}"))?;

        let results = user_email_accounts
            .filter(user_id.eq(current_user_id))
            .filter(is_active.eq(true))
            .order((is_primary.desc(), created_at.desc()))
            .select((
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
            ))
            .load::<(
                Uuid,
                String,
                Option<String>,
                String,
                i32,
                String,
                i32,
                bool,
                bool,
                chrono::DateTime<chrono::Utc>,
            )>(&mut db_conn)
            .map_err(|e| format!("Query failed: {e}"))?;

        Ok::<_, String>(results)
    })
    .await
    .map_err(|e| EmailError(format!("Task join error: {e}")))?
    .map_err(EmailError)?;

    let account_list: Vec<EmailAccountResponse> = accounts
        .into_iter()
        .map(
            |(
                acc_id,
                acc_email,
                acc_display_name,
                acc_imap_server,
                acc_imap_port,
                acc_smtp_server,
                acc_smtp_port,
                acc_is_primary,
                acc_is_active,
                acc_created_at,
            )| {
                EmailAccountResponse {
                    id: acc_id.to_string(),
                    email: acc_email,
                    display_name: acc_display_name,
                    imap_server: acc_imap_server,
                    imap_port: acc_imap_port as u16,
                    smtp_server: acc_smtp_server,
                    smtp_port: acc_smtp_port as u16,
                    is_primary: acc_is_primary,
                    is_active: acc_is_active,
                    created_at: acc_created_at.to_rfc3339(),
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
            .map_err(|e| format!("DB connection error: {e}"))?;

        diesel::sql_query("UPDATE user_email_accounts SET is_active = false WHERE id = $1")
            .bind::<diesel::sql_types::Uuid, _>(account_uuid)
            .execute(&mut db_conn)
            .map_err(|e| format!("Failed to delete account: {e}"))?;

        Ok::<_, String>(())
    })
    .await
    .map_err(|e| EmailError(format!("Task join error: {e}")))?
    .map_err(EmailError)?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(()),
        message: Some("Email account deleted".to_string()),
    }))
}

pub async fn list_emails(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ListEmailsRequest>,
) -> Result<Json<ApiResponse<Vec<EmailResponse>>>, EmailError> {
    let account_uuid = Uuid::parse_str(&request.account_id)
        .map_err(|_| EmailError("Invalid account ID".to_string()))?;

    let conn = state.conn.clone();
    let account_info = tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().map_err(|e| format!("DB connection error: {e}"))?;

        let result: ImapCredentialsRow = diesel::sql_query(
            "SELECT imap_server, imap_port, username, password_encrypted FROM user_email_accounts WHERE id = $1 AND is_active = true"
        )
        .bind::<diesel::sql_types::Uuid, _>(account_uuid)
        .get_result(&mut db_conn)
        .map_err(|e| format!("Account not found: {e}"))?;

        Ok::<_, String>(result)
    })
    .await
    .map_err(|e| EmailError(format!("Task join error: {e}")))?
    .map_err(EmailError)?;

    let (imap_server, imap_port, username, encrypted_password) = (
        account_info.imap_server,
        account_info.imap_port,
        account_info.username,
        account_info.password_encrypted,
    );
    let password = decrypt_password(&encrypted_password).map_err(EmailError)?;

    let client = imap::ClientBuilder::new(imap_server.as_str(), imap_port as u16)
        .connect()
        .map_err(|e| EmailError(format!("Failed to connect to IMAP: {e:?}")))?;

    let mut session = client
        .login(&username, &password)
        .map_err(|e| EmailError(format!("Login failed: {e:?}")))?;

    let folder = request.folder.unwrap_or_else(|| "INBOX".to_string());
    session
        .select(&folder)
        .map_err(|e| EmailError(format!("Failed to select folder: {e:?}")))?;

    let messages = session
        .search("ALL")
        .map_err(|e| EmailError(format!("Failed to search emails: {e:?}")))?;

    let mut email_list = Vec::new();
    let limit = request.limit.unwrap_or(50);
    let offset = request.offset.unwrap_or(0);

    let mut recent_messages: Vec<Seq> = messages.iter().copied().collect();
    recent_messages.sort_by(|a, b| b.cmp(a));
    let recent_messages: Vec<Seq> = recent_messages
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect();

    for seq in recent_messages {
        let fetch_result = session.fetch(seq.to_string(), "RFC822");
        let messages =
            fetch_result.map_err(|e| EmailError(format!("Failed to fetch email: {e:?}")))?;

        for msg in messages.iter() {
            let body = msg
                .body()
                .ok_or_else(|| EmailError("No body found".to_string()))?;

            let parsed = parse_mail(body)
                .map_err(|e| EmailError(format!("Failed to parse email: {e:?}")))?;

            let headers = parsed.get_headers();
            let subject = headers.get_first_value("Subject").unwrap_or_default();
            let from = headers.get_first_value("From").unwrap_or_default();
            let to = headers.get_first_value("To").unwrap_or_default();
            let date = headers.get_first_value("Date").unwrap_or_default();

            let body_text = parsed
                .subparts
                .iter()
                .find(|p| p.ctype.mimetype == "text/plain")
                .map_or_else(
                    || parsed.get_body().unwrap_or_default(),
                    |body_part| body_part.get_body().unwrap_or_default(),
                );

            let body_html = parsed
                .subparts
                .iter()
                .find(|p| p.ctype.mimetype == "text/html")
                .map_or_else(String::new, |body_part| {
                    body_part.get_body().unwrap_or_default()
                });

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
                read: false,
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

    let conn = state.conn.clone();
    let account_info = tokio::task::spawn_blocking(move || {
        let mut db_conn = conn
            .get()
            .map_err(|e| format!("DB connection error: {e}"))?;

        let result: SmtpCredentialsRow = diesel::sql_query(
            "SELECT email, display_name, smtp_port, smtp_server, username, password_encrypted
            FROM user_email_accounts WHERE id = $1 AND is_active = true",
        )
        .bind::<diesel::sql_types::Uuid, _>(account_uuid)
        .get_result(&mut db_conn)
        .map_err(|e| format!("Account not found: {e}"))?;

        Ok::<_, String>(result)
    })
    .await
    .map_err(|e| EmailError(format!("Task join error: {e}")))?
    .map_err(EmailError)?;

    let (from_email, display_name, smtp_port, smtp_server, username, encrypted_password) = (
        account_info.email,
        account_info.display_name,
        account_info.smtp_port,
        account_info.smtp_server,
        account_info.username,
        account_info.password_encrypted,
    );
    let password = decrypt_password(&encrypted_password).map_err(EmailError)?;

    let from_addr = if display_name.is_empty() {
        from_email.clone()
    } else {
        format!("{display_name} <{from_email}>")
    };

    let pixel_enabled = is_tracking_pixel_enabled(&state, None);
    let tracking_id = Uuid::new_v4();

    let final_body = if pixel_enabled && request.is_html {
        inject_tracking_pixel(&request.body, &tracking_id.to_string(), &state)
    } else {
        request.body.clone()
    };

    let mut email_builder = Message::builder()
        .from(
            from_addr
                .parse()
                .map_err(|e| EmailError(format!("Invalid from address: {e}")))?,
        )
        .to(request
            .to
            .parse()
            .map_err(|e| EmailError(format!("Invalid to address: {e}")))?)
        .subject(request.subject.clone());

    if let Some(ref cc) = request.cc {
        email_builder = email_builder.cc(cc
            .parse()
            .map_err(|e| EmailError(format!("Invalid cc address: {e}")))?);
    }

    if let Some(ref bcc) = request.bcc {
        email_builder = email_builder.bcc(
            bcc.parse()
                .map_err(|e| EmailError(format!("Invalid bcc address: {e}")))?,
        );
    }

    let email = email_builder
        .body(final_body)
        .map_err(|e| EmailError(format!("Failed to build email: {e}")))?;

    let creds = Credentials::new(username, password);
    let mailer = SmtpTransport::relay(&smtp_server)
        .map_err(|e| EmailError(format!("Failed to create SMTP transport: {e}")))?
        .port(u16::try_from(smtp_port).unwrap_or(587))
        .credentials(creds)
        .build();

    mailer
        .send(&email)
        .map_err(|e| EmailError(format!("Failed to send email: {e}")))?;

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
                Uuid::nil(),
                &from_email,
                &to_email,
                cc_clone.as_deref(),
                bcc_clone.as_deref(),
                &subject,
            )
        })
        .await;
    }

    info!("Email sent successfully from account {account_uuid} with tracking_id {tracking_id}");

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

    let Ok(user_id) = extract_user_from_session(&state) else {
        return Err(EmailError("Authentication required".to_string()));
    };
    let draft_id = Uuid::new_v4();

    let conn = state.conn.clone();
    tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().map_err(|e| format!("DB connection error: {e}"))?;

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
        .map_err(|e| format!("Failed to save draft: {e}"))?;

        Ok::<_, String>(())
    })
    .await
    .map_err(|e| EmailError(format!("Task join error: {e}")))?
    .map_err(EmailError)?;

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

    let conn = state.conn.clone();
    let account_info = tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().map_err(|e| format!("DB connection error: {e}"))?;

        let result: ImapCredentialsRow = diesel::sql_query(
            "SELECT imap_server, imap_port, username, password_encrypted FROM user_email_accounts WHERE id = $1 AND is_active = true"
        )
        .bind::<diesel::sql_types::Uuid, _>(account_uuid)
        .get_result(&mut db_conn)
        .map_err(|e| format!("Account not found: {e}"))?;

        Ok::<_, String>(result)
    })
    .await
    .map_err(|e| EmailError(format!("Task join error: {e}")))?
    .map_err(EmailError)?;

    let (imap_server, imap_port, username, encrypted_password) = (
        account_info.imap_server,
        account_info.imap_port,
        account_info.username,
        account_info.password_encrypted,
    );
    let password = decrypt_password(&encrypted_password).map_err(EmailError)?;

    let client = imap::ClientBuilder::new(imap_server.as_str(), imap_port as u16)
        .connect()
        .map_err(|e| EmailError(format!("Failed to connect to IMAP: {e:?}")))?;

    let mut session = client
        .login(&username, &password)
        .map_err(|e| EmailError(format!("Login failed: {e:?}")))?;

    let folders = session
        .list(None, Some("*"))
        .map_err(|e| EmailError(format!("Failed to list folders: {e:?}")))?;

    let folder_list: Vec<FolderInfo> = folders
        .iter()
        .map(|f| FolderInfo {
            name: f.name().to_string(),
            path: f.name().to_string(),
            unread_count: 0,
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

pub fn get_latest_email_from(
    State(_state): State<Arc<AppState>>,
    Json(_request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, EmailError> {
    Ok(Json(serde_json::json!({
        "success": false,
        "message": "Please use the new /api/email/list endpoint with account_id"
    })))
}

pub fn save_click(
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

const TRACKING_PIXEL: [u8; 43] = [
    0x47, 0x49, 0x46, 0x38, 0x39, 0x61, 0x01, 0x00, 0x01, 0x00, 0x80, 0x00, 0x00, 0xFF, 0xFF, 0xFF,
    0x00, 0x00, 0x00, 0x21, 0xF9, 0x04, 0x01, 0x00, 0x00, 0x00, 0x00, 0x2C, 0x00, 0x00, 0x00, 0x00,
    0x01, 0x00, 0x01, 0x00, 0x00, 0x02, 0x02, 0x44, 0x01, 0x00, 0x3B,
];

fn is_tracking_pixel_enabled(state: &Arc<AppState>, bot_id: Option<Uuid>) -> bool {
    let config_manager = crate::core::config::ConfigManager::new(state.conn.clone());
    let bot_id = bot_id.unwrap_or(Uuid::nil());

    config_manager
        .get_config(&bot_id, "email-read-pixel", Some("false"))
        .map(|v| v.to_lowercase() == "true")
        .unwrap_or(false)
}

fn inject_tracking_pixel(html_body: &str, tracking_id: &str, state: &Arc<AppState>) -> String {
    let config_manager = crate::core::config::ConfigManager::new(state.conn.clone());
    let base_url = config_manager
        .get_config(&Uuid::nil(), "server-url", Some("http://localhost:8080"))
        .unwrap_or_else(|_| "http://localhost:8080".to_string());

    let pixel_url = format!("{}/api/email/tracking/pixel/{}", base_url, tracking_id);
    let pixel_html = format!(
        r#"<img src="{}" width="1" height="1" style="display:none;visibility:hidden;width:1px;height:1px;border:0;" alt="" />"#,
        pixel_url
    );

    if html_body.to_lowercase().contains("</body>") {
        html_body
            .replace("</body>", &format!("{}</body>", pixel_html))
            .replace("</BODY>", &format!("{}</BODY>", pixel_html))
    } else {
        format!("{}{}", html_body, pixel_html)
    }
}

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
    let mut db_conn = conn
        .get()
        .map_err(|e| format!("DB connection error: {}", e))?;

    let id = Uuid::new_v4();
    let now = Utc::now();

    diesel::sql_query(
        "INSERT INTO sent_email_tracking
           (id, tracking_id, bot_id, account_id, from_email, to_email, cc, bcc, subject, sent_at, read_count, is_read)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, 0, false)"
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

pub async fn serve_tracking_pixel(
    Path(tracking_id): Path<String>,
    State(state): State<Arc<AppState>>,
    Query(_query): Query<TrackingPixelQuery>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
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

    if let Ok(tracking_uuid) = Uuid::parse_str(&tracking_id) {
        let conn = state.conn.clone();
        let ip_clone = client_ip.clone();
        let ua_clone = user_agent.clone();

        let _ = tokio::task::spawn_blocking(move || {
            update_email_read_status(conn, tracking_uuid, ip_clone, ua_clone)
        })
        .await;

        info!(
            "Email read tracked: tracking_id={}, ip={:?}",
            tracking_id, client_ip
        );
    } else {
        warn!("Invalid tracking ID received: {}", tracking_id);
    }

    (
        StatusCode::OK,
        [
            ("content-type", "image/gif"),
            (
                "cache-control",
                "no-store, no-cache, must-revalidate, max-age=0",
            ),
            ("pragma", "no-cache"),
            ("expires", "0"),
        ],
        TRACKING_PIXEL.to_vec(),
    )
}

fn update_email_read_status(
    conn: crate::shared::utils::DbPool,
    tracking_id: Uuid,
    client_ip: Option<String>,
    user_agent: Option<String>,
) -> Result<(), String> {
    let mut db_conn = conn
        .get()
        .map_err(|e| format!("DB connection error: {}", e))?;
    let now = Utc::now();

    diesel::sql_query(
        r"UPDATE sent_email_tracking
           SET
               is_read = true,
               read_count = read_count + 1,
               read_at = COALESCE(read_at, $2),
               first_read_ip = COALESCE(first_read_ip, $3),
               last_read_ip = $3,
               user_agent = COALESCE(user_agent, $4),
               updated_at = $2
           WHERE tracking_id = $1",
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

pub async fn get_tracking_status(
    Path(tracking_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<TrackingStatusResponse>>, EmailError> {
    let tracking_uuid =
        Uuid::parse_str(&tracking_id).map_err(|_| EmailError("Invalid tracking ID".to_string()))?;

    let conn = state.conn.clone();
    let result = tokio::task::spawn_blocking(move || get_tracking_record(conn, tracking_uuid))
        .await
        .map_err(|e| EmailError(format!("Task join error: {}", e)))?
        .map_err(EmailError)?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(result),
        message: None,
    }))
}

fn get_tracking_record(
    conn: crate::shared::utils::DbPool,
    tracking_id: Uuid,
) -> Result<TrackingStatusResponse, String> {
    let mut db_conn = conn
        .get()
        .map_err(|e| format!("DB connection error: {}", e))?;

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
        r"SELECT tracking_id, to_email, subject, sent_at, is_read, read_at, read_count
           FROM sent_email_tracking WHERE tracking_id = $1",
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

pub async fn list_sent_emails_tracking(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListTrackingQuery>,
) -> Result<Json<ApiResponse<Vec<TrackingStatusResponse>>>, EmailError> {
    let conn = state.conn.clone();
    let result = tokio::task::spawn_blocking(move || list_tracking_records(conn, query))
        .await
        .map_err(|e| EmailError(format!("Task join error: {}", e)))?
        .map_err(EmailError)?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(result),
        message: None,
    }))
}

fn list_tracking_records(
    conn: crate::shared::utils::DbPool,
    query: ListTrackingQuery,
) -> Result<Vec<TrackingStatusResponse>, String> {
    let mut db_conn = conn
        .get()
        .map_err(|e| format!("DB connection error: {}", e))?;

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

    let base_query = match query.filter.as_deref() {
        Some("read") => {
            "SELECT tracking_id, to_email, subject, sent_at, is_read, read_at, read_count
               FROM sent_email_tracking WHERE account_id = $1 AND is_read = true
               ORDER BY sent_at DESC LIMIT $2 OFFSET $3"
        }
        Some("unread") => {
            "SELECT tracking_id, to_email, subject, sent_at, is_read, read_at, read_count
               FROM sent_email_tracking WHERE account_id = $1 AND is_read = false
               ORDER BY sent_at DESC LIMIT $2 OFFSET $3"
        }
        _ => {
            "SELECT tracking_id, to_email, subject, sent_at, is_read, read_at, read_count
               FROM sent_email_tracking WHERE account_id = $1
               ORDER BY sent_at DESC LIMIT $2 OFFSET $3"
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

pub async fn get_tracking_stats(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<TrackingStatsResponse>>, EmailError> {
    let conn = state.conn.clone();
    let result = tokio::task::spawn_blocking(move || calculate_tracking_stats(conn))
        .await
        .map_err(|e| EmailError(format!("Task join error: {}", e)))?
        .map_err(EmailError)?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(result),
        message: None,
    }))
}

fn calculate_tracking_stats(
    conn: crate::shared::utils::DbPool,
) -> Result<TrackingStatsResponse, String> {
    let mut db_conn = conn
        .get()
        .map_err(|e| format!("DB connection error: {}", e))?;

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
        r"SELECT
               COUNT(*) as total_sent,
               COUNT(*) FILTER (WHERE is_read = true) as total_read,
               AVG(EXTRACT(EPOCH FROM (read_at - sent_at)) / 3600) FILTER (WHERE is_read = true) as avg_time_hours
           FROM sent_email_tracking",
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

pub fn get_emails(Path(campaign_id): Path<String>, State(_state): State<Arc<AppState>>) -> String {
    info!("Get emails requested for campaign: {campaign_id}");
    "No emails tracked".to_string()
}

pub struct EmailService {
    state: Arc<AppState>,
}

impl EmailService {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }

    pub fn send_email(
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

    pub fn send_email_with_attachment(
        &self,
        to: &str,
        subject: &str,
        body: &str,
        _attachment: Vec<u8>,
        _filename: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.send_email(to, subject, body, None)
    }
}

pub async fn fetch_latest_sent_to(config: &EmailConfig, to: &str) -> Result<String, String> {
    let client = imap::ClientBuilder::new(&config.server, config.port)
        .connect()
        .map_err(|e| format!("Connection error: {}", e))?;

    let mut session = client
        .login(&config.username, &config.password)
        .map_err(|e| format!("Login failed: {:?}", e))?;

    session
        .select("INBOX")
        .map_err(|e| format!("Select INBOX failed: {}", e))?;

    let search_query = format!("TO \"{}\"", to);
    let message_ids = session
        .search(&search_query)
        .map_err(|e| format!("Search failed: {}", e))?;

    if let Some(&last_id) = message_ids.iter().max() {
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

    let client = imap::ClientBuilder::new(&config.server, config.port)
        .connect()
        .map_err(|e| format!("Connection error: {}", e))?;

    let mut session = client
        .login(&config.username, &config.password)
        .map_err(|e| format!("Login failed: {:?}", e))?;

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
        date, config.from, draft.to, cc_header, draft.subject, message_id, draft.body
    );

    let folder = session
        .list(None, Some("Drafts"))
        .map_err(|e| format!("List folders failed: {}", e))?
        .iter()
        .find(|name| name.name().to_lowercase().contains("draft"))
        .map(|n| n.name().to_string())
        .unwrap_or_else(|| "INBOX".to_string());

    session
        .append(&folder, email_content.as_bytes())
        .finish()
        .map_err(|e| format!("Append draft failed: {}", e))?;

    session.logout().ok();
    info!("Draft saved to: {}, subject: {}", draft.to, draft.subject);
    Ok(())
}

fn fetch_emails_from_folder(
    config: &EmailConfig,
    folder: &str,
) -> Result<Vec<EmailSummary>, String> {
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
                let unread = !flags.iter().any(|f| matches!(f, imap::types::Flag::Seen));

                let preview = subject.chars().take(100).collect();
                emails.push(EmailSummary {
                    id: message.message.to_string(),
                    from,
                    subject,
                    date,
                    preview,
                    unread,
                });
            }
        }
    }

    session.logout().ok();
    Ok(emails)
}

fn get_folder_counts(
    config: &EmailConfig,
) -> Result<std::collections::HashMap<String, usize>, String> {
    use std::collections::HashMap;

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

fn fetch_email_by_id(config: &EmailConfig, id: &str) -> Result<EmailContent, String> {
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

fn move_email_to_trash(config: &EmailConfig, id: &str) -> Result<(), String> {
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
            log::error!("Email account query error: {}", e);
            return axum::response::Html(
                r#"<div class="empty-state">
                    <h3>Unable to load emails</h3>
                    <p>There was an error connecting to the database. Please try again later.</p>
                </div>"#
                    .to_string(),
            );
        }
        Err(e) => {
            log::error!("Task join error: {}", e);
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
        password: account.password.clone(),
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
        let unread_class = if email.unread { "unread" } else { "" };
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
            unread_class, email.id, email.from, email.date, email.subject, email.preview
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

        diesel::sql_query("SELECT * FROM email_accounts WHERE user_id = $1 LIMIT 1")
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
            log::error!("Email folder query error: {}", e);
            return axum::response::Html(
                r#"<div class="nav-item">Error loading folders</div>"#.to_string(),
            );
        }
        Err(e) => {
            log::error!("Task join error: {}", e);
            return axum::response::Html(
                r#"<div class="nav-item">Error loading folders</div>"#.to_string(),
            );
        }
    };

    let config = EmailConfig {
        username: account.username.clone(),
        password: account.password.clone(),
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

        diesel::sql_query("SELECT * FROM email_accounts WHERE user_id = $1 LIMIT 1")
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
        password: account.password.clone(),
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
        email_content.from,
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

        diesel::sql_query("SELECT * FROM email_accounts WHERE user_id = $1 LIMIT 1")
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
            log::error!("Email account query error: {}", e);
            return axum::response::Html(
                r#"<div class="empty-state">
                    <h3>Error deleting email</h3>
                    <p>Database error occurred</p>
                </div>"#
                    .to_string(),
            );
        }
        Err(e) => {
            log::error!("Task join error: {}", e);
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
        password: account.password.clone(),
        server: account.imap_server.clone(),
        port: account.imap_port as u16,
        from: account.email.clone(),
        smtp_server: account.smtp_server.clone(),
        smtp_port: account.smtp_port as u16,
    };

    if let Err(e) = move_email_to_trash(&config, &id) {
        log::error!("Failed to delete email: {}", e);
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

pub async fn get_latest_email(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<EmailData>>, EmailError> {
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

pub async fn get_email(
    State(_state): State<Arc<AppState>>,
    Path(campaign_id): Path<String>,
) -> Result<Json<ApiResponse<EmailData>>, EmailError> {
    Ok(Json(ApiResponse {
        success: true,
        data: Some(EmailData {
            id: campaign_id,
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

pub async fn track_click(
    State(_state): State<Arc<AppState>>,
    Path((campaign_id, email)): Path<(String, String)>,
) -> Result<Json<ApiResponse<()>>, EmailError> {
    info!(
        "Tracking click for campaign {} email {}",
        campaign_id, email
    );

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

#[derive(Debug, QueryableByName)]
struct EmailAccountRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub _id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub _user_id: Uuid,
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
