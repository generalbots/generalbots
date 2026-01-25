use axum::{http::StatusCode, response::{IntoResponse, Response}};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::sql_types::{Bool, Integer, Nullable, Text, Timestamptz, Uuid as DieselUuid, Varchar};
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Serialize, Deserialize)]
pub struct EmailSignature {
    pub id: String,
    pub name: String,
    pub content_html: String,
    pub content_text: String,
    pub is_default: bool,
}

pub struct EmailError(pub String);

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

pub struct EmailService {
    pub state: std::sync::Arc<crate::shared::state::AppState>,
}

pub struct EmailData {
    pub id: String,
    pub from_name: String,
    pub from_email: String,
    pub to: String,
    pub subject: String,
    pub body: String,
    pub date: String,
    pub read: bool,
}

#[derive(Debug, QueryableByName)]
pub struct EmailAccountRow {
    #[diesel(sql_type = DieselUuid)]
    pub id: Uuid,
    #[diesel(sql_type = Text)]
    pub email: String,
    #[diesel(sql_type = Nullable<Text>)]
    pub display_name: Option<String>,
    #[diesel(sql_type = Text)]
    pub imap_server: String,
    #[diesel(sql_type = Integer)]
    pub imap_port: i32,
    #[diesel(sql_type = Text)]
    pub smtp_server: String,
    #[diesel(sql_type = Integer)]
    pub smtp_port: i32,
    #[diesel(sql_type = Text)]
    pub username: String,
    #[diesel(sql_type = Text)]
    pub password_encrypted: String,
    #[diesel(sql_type = Bool)]
    pub is_primary: bool,
    #[diesel(sql_type = Bool)]
    pub is_active: bool,
    #[diesel(sql_type = Timestamptz)]
    pub created_at: DateTime<Utc>,
    #[diesel(sql_type = Timestamptz)]
    pub updated_at: DateTime<Utc>,
}

pub struct EmailSummary {
    pub id: String,
    pub from_name: String,
    pub from_email: String,
    pub subject: String,
    pub preview: String,
    pub date: String,
    pub read: bool,
}

pub struct EmailContent {
    pub id: String,
    pub from_name: String,
    pub from_email: String,
    pub to: String,
    pub subject: String,
    pub body: String,
    pub date: String,
    pub read: bool,
}
