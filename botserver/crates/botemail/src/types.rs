use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Deserialize)]
pub struct FlagRequest {
    pub email_ids: Vec<Uuid>,
    pub follow_up: String,
}

#[derive(Debug, Serialize)]
pub struct FlagResponse {
    pub flagged_count: usize,
}

#[derive(Debug, Deserialize)]
pub struct SnoozeRequest {
    pub email_ids: Vec<Uuid>,
    pub preset: String,
}

#[derive(Debug, Serialize)]
pub struct SnoozeResponse {
    pub snoozed_count: usize,
    pub snooze_until: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct NudgeCheckRequest {
    pub user_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct Nudge {
    pub email_id: Uuid,
    pub from: String,
    pub subject: String,
    pub days_ago: i64,
}

#[derive(Debug, Serialize)]
pub struct NudgesResponse {
    pub nudges: Vec<Nudge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlags {
    pub crm_enabled: bool,
    pub campaigns_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailCrmLink {
    pub email_id: Uuid,
    pub contact_id: Option<Uuid>,
    pub opportunity_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeadExtractionRequest {
    pub from: String,
    pub subject: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeadExtractionResponse {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: String,
    pub company: Option<String>,
    pub phone: Option<String>,
    pub value: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartReplyRequest {
    pub email_id: Uuid,
    pub context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartReplyResponse {
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailCategoryResponse {
    pub category: String,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    pub username: String,
    pub password: String,
    pub server: String,
    pub port: u16,
    pub from: String,
    pub smtp_server: String,
    pub smtp_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticatedUser {
    pub user_id: Uuid,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub roles: Vec<String>,
    pub groups: Vec<String>,
    pub organization_id: Option<Uuid>,
    pub is_admin: bool,
    pub is_super_admin: bool,
}

pub struct EmailTrackingParams<'a> {
    pub tracking_id: Uuid,
    pub account_id: Uuid,
    pub bot_id: Uuid,
    pub from_email: &'a str,
    pub to_email: &'a str,
    pub cc: Option<&'a str>,
    pub bcc: Option<&'a str>,
    pub subject: &'a str,
}
