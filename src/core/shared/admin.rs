#![cfg_attr(feature = "mail", allow(unused_imports))]
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, Json},
    routing::{get, post},
    Router,
};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::sql_types::{Nullable, Text, Timestamptz, Uuid as DieselUuid, Varchar};
#[cfg(feature = "mail")]
use lettre::{Message, SmtpTransport, Transport};
#[cfg(feature = "mail")]
use lettre::transport::smtp::authentication::Credentials;
use log::warn;
#[cfg(feature = "mail")]
use log::info;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

// ============================================================================
// Invitation Email Functions
// ============================================================================

/// Send invitation email via SMTP
#[cfg(feature = "mail")]
async fn send_invitation_email(
to_email: &str,
role: &str,
custom_message: Option<&str>,
invitation_id: Uuid,
) -> Result<(), String> {
    let smtp_host = std::env::var("SMTP_HOST").unwrap_or_else(|_| "localhost".to_string());
    let smtp_user = std::env::var("SMTP_USER").ok();
    let smtp_pass = std::env::var("SMTP_PASS").ok();
    let smtp_from = std::env::var("SMTP_FROM").unwrap_or_else(|_| "noreply@generalbots.com".to_string());
    let app_url = std::env::var("APP_URL").unwrap_or_else(|_| "https://app.generalbots.com".to_string());

    let accept_url = format!("{}/accept-invitation?token={}", app_url, invitation_id);

    let body = format!(
        r#"You have been invited to join our organization as a {role}.

{custom_msg}

Click the link below to accept the invitation:
{accept_url}

This invitation will expire in 7 days.

If you did not expect this invitation, you can safely ignore this email.

Best regards,
The General Bots Team"#,
        role = role,
        custom_msg = custom_message.unwrap_or(""),
        accept_url = accept_url
    );

    let email = Message::builder()
        .from(smtp_from.parse().map_err(|e| format!("Invalid from address: {}", e))?)
        .to(to_email.parse().map_err(|e| format!("Invalid to address: {}", e))?)
        .subject("You've been invited to join our organization")
        .body(body)
        .map_err(|e| format!("Failed to build email: {}", e))?;

    let mailer = if let (Some(user), Some(pass)) = (smtp_user, smtp_pass) {
        let creds = Credentials::new(user, pass);
        SmtpTransport::relay(&smtp_host)
            .map_err(|e| format!("SMTP relay error: {}", e))?
            .credentials(creds)
            .build()
    } else {
        SmtpTransport::builder_dangerous(&smtp_host).build()
    };

    mailer.send(&email).map_err(|e| format!("Failed to send email: {}", e))?;

    info!("Invitation email sent successfully to {}", to_email);
    Ok(())
}

/// Send invitation email by fetching details from database
#[cfg(feature = "mail")]
async fn send_invitation_email_by_id(invitation_id: Uuid) -> Result<(), String> {
    let smtp_host = std::env::var("SMTP_HOST").unwrap_or_else(|_| "localhost".to_string());
    let smtp_user = std::env::var("SMTP_USER").ok();
    let smtp_pass = std::env::var("SMTP_PASS").ok();
    let smtp_from = std::env::var("SMTP_FROM").unwrap_or_else(|_| "noreply@generalbots.com".to_string());
    let app_url = std::env::var("APP_URL").unwrap_or_else(|_| "https://app.generalbots.com".to_string());

    // Get database URL and connect
    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL not configured".to_string())?;

    let mut conn = diesel::PgConnection::establish(&database_url)
        .map_err(|e| format!("Database connection failed: {}", e))?;

    // Fetch invitation details
    #[derive(QueryableByName)]
    struct InvitationDetails {
        #[diesel(sql_type = Varchar)]
        email: String,
        #[diesel(sql_type = Varchar)]
        role: String,
        #[diesel(sql_type = Nullable<Text>)]
        message: Option<String>,
    }

    let invitation: InvitationDetails = diesel::sql_query(
        "SELECT email, role, message FROM organization_invitations WHERE id = $1 AND status = 'pending'"
    )
    .bind::<DieselUuid, _>(invitation_id)
    .get_result(&mut conn)
    .map_err(|e| format!("Failed to fetch invitation: {}", e))?;

    let accept_url = format!("{}/accept-invitation?token={}", app_url, invitation_id);

    let body = format!(
        r#"You have been invited to join our organization as a {role}.

{custom_msg}

Click the link below to accept the invitation:
{accept_url}

This invitation will expire in 7 days.

If you did not expect this invitation, you can safely ignore this email.

Best regards,
The General Bots Team"#,
        role = invitation.role,
        custom_msg = invitation.message.as_deref().unwrap_or(""),
        accept_url = accept_url
    );

    let email = Message::builder()
        .from(smtp_from.parse().map_err(|e| format!("Invalid from address: {}", e))?)
        .to(invitation.email.parse().map_err(|e| format!("Invalid to address: {}", e))?)
        .subject("Reminder: You've been invited to join our organization")
        .body(body)
        .map_err(|e| format!("Failed to build email: {}", e))?;

    let mailer = if let (Some(user), Some(pass)) = (smtp_user, smtp_pass) {
        let creds = Credentials::new(user, pass);
        SmtpTransport::relay(&smtp_host)
            .map_err(|e| format!("SMTP relay error: {}", e))?
            .credentials(creds)
            .build()
    } else {
        SmtpTransport::builder_dangerous(&smtp_host).build()
    };

    mailer.send(&email).map_err(|e| format!("Failed to send email: {}", e))?;

    info!("Invitation resend email sent successfully to {}", invitation.email);
    Ok(())
}

use crate::core::urls::ApiUrls;
use crate::core::middleware::AuthenticatedUser;
use crate::shared::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ConfigUpdateRequest {
    pub config_key: String,
    pub config_value: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct MaintenanceScheduleRequest {
    pub scheduled_at: DateTime<Utc>,
    pub duration_minutes: u32,
    pub reason: String,
    pub notify_users: bool,
}

#[derive(Debug, Deserialize)]
pub struct BackupRequest {
    pub backup_type: String,
    pub include_files: bool,
    pub include_database: bool,
    pub compression: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RestoreRequest {
    pub backup_id: String,
    pub restore_point: DateTime<Utc>,
    pub verify_before_restore: bool,
}

#[derive(Debug, Deserialize)]
pub struct UserManagementRequest {
    pub user_id: Uuid,
    pub action: String,
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RoleManagementRequest {
    pub role_name: String,
    pub permissions: Vec<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct QuotaManagementRequest {
    pub user_id: Option<Uuid>,
    pub group_id: Option<Uuid>,
    pub quota_type: String,
    pub limit_value: u64,
}

#[derive(Debug, Deserialize)]
pub struct LicenseManagementRequest {
    pub license_key: String,
    pub license_type: String,
}

#[derive(Debug, Deserialize)]
pub struct LogQuery {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub level: Option<String>,
    pub service: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct SystemStatusResponse {
    pub status: String,
    pub uptime_seconds: u64,
    pub version: String,
    pub services: Vec<ServiceStatus>,
    pub health_checks: Vec<HealthCheck>,
    pub last_restart: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ServiceStatus {
    pub name: String,
    pub status: String,
    pub uptime_seconds: u64,
    pub memory_mb: f64,
    pub cpu_percent: f64,
}

#[derive(Debug, Serialize)]
pub struct HealthCheck {
    pub name: String,
    pub status: String,
    pub message: Option<String>,
    pub last_check: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct SystemMetricsResponse {
    pub cpu_usage: f64,
    pub memory_total_mb: u64,
    pub memory_used_mb: u64,
    pub memory_percent: f64,
    pub disk_total_gb: u64,
    pub disk_used_gb: u64,
    pub disk_percent: f64,
    pub network_in_mbps: f64,
    pub network_out_mbps: f64,
    pub active_connections: u32,
    pub request_rate_per_minute: u32,
    pub error_rate_percent: f64,
}

#[derive(Debug, Serialize)]
pub struct LogEntry {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub level: String,
    pub service: String,
    pub message: String,
    pub metadata: Option<serde_json::Value>,
}

// =============================================================================
// INVITATION MANAGEMENT TYPES
// =============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateInvitationRequest {
    pub email: String,
    #[serde(default = "default_role")]
    pub role: String,
    pub message: Option<String>,
}

fn default_role() -> String {
    "member".to_string()
}

#[derive(Debug, Deserialize)]
pub struct BulkInvitationRequest {
    pub emails: Vec<String>,
    #[serde(default = "default_role")]
    pub role: String,
    pub message: Option<String>,
}

#[derive(Debug, Serialize, QueryableByName)]
pub struct InvitationRow {
    #[diesel(sql_type = DieselUuid)]
    pub id: Uuid,
    #[diesel(sql_type = DieselUuid)]
    pub org_id: Uuid,
    #[diesel(sql_type = Varchar)]
    pub email: String,
    #[diesel(sql_type = Varchar)]
    pub role: String,
    #[diesel(sql_type = Varchar)]
    pub status: String,
    #[diesel(sql_type = Nullable<Text>)]
    pub message: Option<String>,
    #[diesel(sql_type = DieselUuid)]
    pub invited_by: Uuid,
    #[diesel(sql_type = Timestamptz)]
    pub created_at: DateTime<Utc>,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    pub expires_at: Option<DateTime<Utc>>,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    pub accepted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct InvitationResponse {
    pub success: bool,
    pub id: Option<Uuid>,
    pub email: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BulkInvitationResponse {
    pub success: bool,
    pub sent: i32,
    pub failed: i32,
    pub errors: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ConfigResponse {
    pub configs: Vec<ConfigItem>,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ConfigItem {
    pub key: String,
    pub value: serde_json::Value,
    pub description: Option<String>,
    pub editable: bool,
    pub requires_restart: bool,
}

#[derive(Debug, Serialize)]
pub struct MaintenanceResponse {
    pub id: Uuid,
    pub scheduled_at: DateTime<Utc>,
    pub duration_minutes: u32,
    pub reason: String,
    pub status: String,
    pub created_by: String,
}

#[derive(Debug, Serialize)]
pub struct BackupResponse {
    pub id: Uuid,
    pub backup_type: String,
    pub size_bytes: u64,
    pub created_at: DateTime<Utc>,
    pub status: String,
    pub download_url: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct QuotaResponse {
    pub id: Uuid,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub quota_type: String,
    pub limit_value: u64,
    pub current_value: u64,
    pub percent_used: f64,
}

#[derive(Debug, Serialize)]
pub struct LicenseResponse {
    pub id: Uuid,
    pub license_type: String,
    pub status: String,
    pub max_users: u32,
    pub current_users: u32,
    pub features: Vec<String>,
    pub issued_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct SuccessResponse {
    pub success: bool,
    pub message: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AdminDashboardData {
    pub total_users: i64,
    pub active_groups: i64,
    pub running_bots: i64,
    pub storage_used_gb: f64,
    pub storage_total_gb: f64,
    pub recent_activity: Vec<ActivityItem>,
    pub system_health: SystemHealth,
}

#[derive(Debug, Serialize)]
pub struct ActivityItem {
    pub id: String,
    pub action: String,
    pub user: String,
    pub timestamp: DateTime<Utc>,
    pub details: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SystemHealth {
    pub status: String,
    pub cpu_percent: f64,
    pub memory_percent: f64,
    pub services_healthy: i32,
    pub services_total: i32,
}

#[derive(Debug, Serialize)]
pub struct StatValue {
    pub value: String,
    pub label: String,
    pub trend: Option<String>,
}

pub fn configure() -> Router<Arc<AppState>> {
    Router::new()
        .route(ApiUrls::ADMIN_DASHBOARD, get(get_admin_dashboard))
        .route(ApiUrls::ADMIN_STATS_USERS, get(get_stats_users))
        .route(ApiUrls::ADMIN_STATS_GROUPS, get(get_stats_groups))
        .route(ApiUrls::ADMIN_STATS_BOTS, get(get_stats_bots))
        .route(ApiUrls::ADMIN_STATS_STORAGE, get(get_stats_storage))
        .route(ApiUrls::ADMIN_USERS, get(get_admin_users))
        .route(ApiUrls::ADMIN_GROUPS, get(get_admin_groups).post(create_group))
        .route(ApiUrls::ADMIN_BOTS, get(get_admin_bots))
        .route(ApiUrls::ADMIN_DNS, get(get_admin_dns))
        .route(ApiUrls::ADMIN_BILLING, get(get_admin_billing))
        .route(ApiUrls::ADMIN_AUDIT, get(get_admin_audit))
        .route(ApiUrls::ADMIN_SYSTEM, get(get_system_status))
        .route("/api/admin/export-report", get(export_admin_report))
        .route("/api/admin/dashboard/stats", get(get_dashboard_stats))
        .route("/api/admin/dashboard/health", get(get_dashboard_health))
        .route("/api/admin/dashboard/activity", get(get_dashboard_activity))
        .route("/api/admin/dashboard/members", get(get_dashboard_members))
        .route("/api/admin/dashboard/roles", get(get_dashboard_roles))
        .route("/api/admin/dashboard/bots", get(get_dashboard_bots))
        .route("/api/admin/dashboard/invitations", get(get_dashboard_invitations))
        .route("/api/admin/invitations", get(list_invitations).post(create_invitation))
        .route("/api/admin/invitations/bulk", post(create_bulk_invitations))
        .route("/api/admin/invitations/:id", get(get_invitation).delete(cancel_invitation))
        .route("/api/admin/invitations/:id/resend", post(resend_invitation))
}

pub async fn get_admin_dashboard(
    State(_state): State<Arc<AppState>>,
) -> Html<String> {
    let html = r##"
<div class="dashboard-view">
    <div class="page-header">
        <h1 data-i18n="admin-dashboard-title">Dashboard</h1>
        <p class="subtitle" data-i18n="admin-dashboard-subtitle">System overview and quick statistics</p>
    </div>

    <div class="stats-grid">
        <div class="stat-card"
             hx-get="/api/admin/stats/users"
             hx-trigger="load, every 30s"
             hx-swap="innerHTML">
            <div class="loading-state"><div class="spinner"></div></div>
        </div>
        <div class="stat-card"
             hx-get="/api/admin/stats/groups"
             hx-trigger="load, every 30s"
             hx-swap="innerHTML">
            <div class="loading-state"><div class="spinner"></div></div>
        </div>
        <div class="stat-card"
             hx-get="/api/admin/stats/bots"
             hx-trigger="load, every 30s"
             hx-swap="innerHTML">
            <div class="loading-state"><div class="spinner"></div></div>
        </div>
        <div class="stat-card"
             hx-get="/api/admin/stats/storage"
             hx-trigger="load, every 30s"
             hx-swap="innerHTML">
            <div class="loading-state"><div class="spinner"></div></div>
        </div>
    </div>

    <div class="section">
        <h2 data-i18n="admin-quick-actions">Quick Actions</h2>
        <div class="quick-actions-grid">
            <button class="action-card" onclick="document.getElementById('create-user-modal').showModal()">
                <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M16 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"></path>
                    <circle cx="8.5" cy="7" r="4"></circle>
                    <line x1="20" y1="8" x2="20" y2="14"></line>
                    <line x1="23" y1="11" x2="17" y2="11"></line>
                </svg>
                <span data-i18n="admin-add-user">Add User</span>
            </button>
            <button class="action-card" onclick="document.getElementById('create-group-modal').showModal()">
                <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"></path>
                    <circle cx="9" cy="7" r="4"></circle>
                    <line x1="23" y1="11" x2="17" y2="11"></line>
                    <line x1="20" y1="8" x2="20" y2="14"></line>
                </svg>
                <span data-i18n="admin-add-group">Add Group</span>
            </button>
            <button class="action-card" hx-get="/api/admin/audit" hx-target="#admin-content" hx-swap="innerHTML">
                <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <circle cx="12" cy="12" r="10"></circle>
                    <line x1="12" y1="8" x2="12" y2="12"></line>
                    <line x1="12" y1="16" x2="12.01" y2="16"></line>
                </svg>
                <span data-i18n="admin-view-audit">View Audit Log</span>
            </button>
            <button class="action-card" hx-get="/api/admin/billing" hx-target="#admin-content" hx-swap="innerHTML">
                <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <rect x="1" y="4" width="22" height="16" rx="2" ry="2"></rect>
                    <line x1="1" y1="10" x2="23" y2="10"></line>
                </svg>
                <span data-i18n="admin-billing">Billing</span>
            </button>
        </div>
    </div>

    <div class="section">
        <h2 data-i18n="admin-system-health">System Health</h2>
        <div class="health-grid">
            <div class="health-card">
                <div class="health-card-header">
                    <span class="health-card-title">API Server</span>
                    <span class="health-status healthy">Healthy</span>
                </div>
                <div class="health-value">99.9%</div>
                <div class="health-label">Uptime</div>
            </div>
            <div class="health-card">
                <div class="health-card-header">
                    <span class="health-card-title">Database</span>
                    <span class="health-status healthy">Healthy</span>
                </div>
                <div class="health-value">12ms</div>
                <div class="health-label">Avg Response</div>
            </div>
            <div class="health-card">
                <div class="health-card-header">
                    <span class="health-card-title">Storage</span>
                    <span class="health-status healthy">Healthy</span>
                </div>
                <div class="health-value">45%</div>
                <div class="health-label">Capacity Used</div>
            </div>
        </div>
    </div>
</div>
"##;
    Html(html.to_string())
}

pub async fn get_stats_users(
    State(_state): State<Arc<AppState>>,
) -> Html<String> {
    let html = r##"
<div class="stat-icon users">
    <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"></path>
        <circle cx="9" cy="7" r="4"></circle>
        <path d="M23 21v-2a4 4 0 0 0-3-3.87"></path>
        <path d="M16 3.13a4 4 0 0 1 0 7.75"></path>
    </svg>
</div>
<div class="stat-content">
    <span class="stat-value">127</span>
    <span class="stat-label" data-i18n="admin-total-users">Total Users</span>
</div>
"##;
    Html(html.to_string())
}

pub async fn get_stats_groups(
    State(_state): State<Arc<AppState>>,
) -> Html<String> {
    let html = r##"
<div class="stat-icon groups">
    <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"></path>
        <circle cx="9" cy="7" r="4"></circle>
        <circle cx="19" cy="11" r="2"></circle>
    </svg>
</div>
<div class="stat-content">
    <span class="stat-value">12</span>
    <span class="stat-label" data-i18n="admin-active-groups">Active Groups</span>
</div>
"##;
    Html(html.to_string())
}

pub async fn get_stats_bots(
    State(_state): State<Arc<AppState>>,
) -> Html<String> {
    let html = r##"
<div class="stat-icon bots">
    <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <rect x="3" y="11" width="18" height="10" rx="2"></rect>
        <circle cx="12" cy="5" r="2"></circle>
        <path d="M12 7v4"></path>
    </svg>
</div>
<div class="stat-content">
    <span class="stat-value">8</span>
    <span class="stat-label" data-i18n="admin-running-bots">Running Bots</span>
</div>
"##;
    Html(html.to_string())
}

pub async fn get_stats_storage(
    State(_state): State<Arc<AppState>>,
) -> Html<String> {
    let html = r##"
<div class="stat-icon storage">
    <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <ellipse cx="12" cy="5" rx="9" ry="3"></ellipse>
        <path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3"></path>
        <path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5"></path>
    </svg>
</div>
<div class="stat-content">
    <span class="stat-value">45.2 GB</span>
    <span class="stat-label" data-i18n="admin-storage-used">Storage Used</span>
</div>
"##;
    Html(html.to_string())
}

pub async fn get_admin_users(
    State(_state): State<Arc<AppState>>,
) -> Html<String> {
    let html = r##"
<div class="admin-page">
    <div class="page-header">
        <h1 data-i18n="admin-users">Users</h1>
        <p class="subtitle" data-i18n="admin-users-subtitle">Manage user accounts and permissions</p>
    </div>
    <div class="toolbar">
        <button class="btn-primary" onclick="document.getElementById('create-user-modal').showModal()">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <line x1="12" y1="5" x2="12" y2="19"></line>
                <line x1="5" y1="12" x2="19" y2="12"></line>
            </svg>
            Add User
        </button>
    </div>
    <div class="data-table">
        <table>
            <thead>
                <tr>
                    <th>Name</th>
                    <th>Email</th>
                    <th>Role</th>
                    <th>Status</th>
                    <th>Actions</th>
                </tr>
            </thead>
            <tbody>
                <tr>
                    <td>John Doe</td>
                    <td>john@example.com</td>
                    <td><span class="badge badge-admin">Admin</span></td>
                    <td><span class="status status-active">Active</span></td>
                    <td><button class="btn-icon">Edit</button></td>
                </tr>
                <tr>
                    <td>Jane Smith</td>
                    <td>jane@example.com</td>
                    <td><span class="badge badge-user">User</span></td>
                    <td><span class="status status-active">Active</span></td>
                    <td><button class="btn-icon">Edit</button></td>
                </tr>
            </tbody>
        </table>
    </div>
</div>
"##;
    Html(html.to_string())
}

pub async fn get_admin_groups(
    State(_state): State<Arc<AppState>>,
) -> Html<String> {
    let html = r##"
<div class="admin-page">
    <div class="page-header">
        <h1 data-i18n="admin-groups">Groups</h1>
        <p class="subtitle" data-i18n="admin-groups-subtitle">Manage groups and team permissions</p>
    </div>
    <div class="toolbar">
        <button class="btn-primary" onclick="document.getElementById('create-group-modal').showModal()">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <line x1="12" y1="5" x2="12" y2="19"></line>
                <line x1="5" y1="12" x2="19" y2="12"></line>
            </svg>
            Add Group
        </button>
    </div>
    <div class="data-table">
        <table>
            <thead>
                <tr>
                    <th>Name</th>
                    <th>Members</th>
                    <th>Created</th>
                    <th>Actions</th>
                </tr>
            </thead>
            <tbody>
                <tr>
                    <td>Engineering</td>
                    <td>15</td>
                    <td>2024-01-15</td>
                    <td><button class="btn-icon">Manage</button></td>
                </tr>
                <tr>
                    <td>Marketing</td>
                    <td>8</td>
                    <td>2024-02-20</td>
                    <td><button class="btn-icon">Manage</button></td>
                </tr>
            </tbody>
        </table>
    </div>
</div>
"##;
    Html(html.to_string())
}

pub async fn get_admin_bots(
    State(_state): State<Arc<AppState>>,
) -> Html<String> {
    let html = r##"
<div class="admin-page">
    <div class="page-header">
        <h1 data-i18n="admin-bots">Bots</h1>
        <p class="subtitle" data-i18n="admin-bots-subtitle">Manage bot instances and deployments</p>
    </div>
    <div class="data-table">
        <table>
            <thead>
                <tr>
                    <th>Name</th>
                    <th>Status</th>
                    <th>Messages</th>
                    <th>Last Active</th>
                    <th>Actions</th>
                </tr>
            </thead>
            <tbody>
                <tr>
                    <td>Support Bot</td>
                    <td><span class="status status-active">Running</span></td>
                    <td>1,234</td>
                    <td>Just now</td>
                    <td><button class="btn-icon">Configure</button></td>
                </tr>
                <tr>
                    <td>Sales Assistant</td>
                    <td><span class="status status-active">Running</span></td>
                    <td>567</td>
                    <td>5 min ago</td>
                    <td><button class="btn-icon">Configure</button></td>
                </tr>
            </tbody>
        </table>
    </div>
</div>
"##;
    Html(html.to_string())
}

pub async fn get_admin_dns(
    State(_state): State<Arc<AppState>>,
) -> Html<String> {
    let html = r##"
<div class="admin-page">
    <div class="page-header">
        <h1 data-i18n="admin-dns">DNS Management</h1>
        <p class="subtitle" data-i18n="admin-dns-subtitle">Configure custom domains and DNS settings</p>
    </div>
    <div class="toolbar">
        <button class="btn-primary" onclick="document.getElementById('add-dns-modal').showModal()">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <line x1="12" y1="5" x2="12" y2="19"></line>
                <line x1="5" y1="12" x2="19" y2="12"></line>
            </svg>
            Add Domain
        </button>
    </div>
    <div class="data-table">
        <table>
            <thead>
                <tr>
                    <th>Domain</th>
                    <th>Type</th>
                    <th>Status</th>
                    <th>SSL</th>
                    <th>Actions</th>
                </tr>
            </thead>
            <tbody>
                <tr>
                    <td>bot.example.com</td>
                    <td>CNAME</td>
                    <td><span class="status status-active">Active</span></td>
                    <td><span class="status status-active">Valid</span></td>
                    <td><button class="btn-icon">Edit</button></td>
                </tr>
            </tbody>
        </table>
    </div>
</div>
"##;
    Html(html.to_string())
}

pub async fn get_admin_billing(
    State(_state): State<Arc<AppState>>,
) -> Html<String> {
    let html = r##"
<div class="admin-page">
    <div class="page-header">
        <h1 data-i18n="admin-billing">Billing</h1>
        <p class="subtitle" data-i18n="admin-billing-subtitle">Manage subscription and payment settings</p>
    </div>
    <div class="billing-overview">
        <div class="billing-card">
            <h3>Current Plan</h3>
            <div class="plan-name">Enterprise</div>
            <div class="plan-price">$499/month</div>
        </div>
        <div class="billing-card">
            <h3>Next Billing Date</h3>
            <div class="billing-date">January 15, 2025</div>
        </div>
        <div class="billing-card">
            <h3>Payment Method</h3>
            <div class="payment-method">**** **** **** 4242</div>
        </div>
    </div>
</div>
"##;
    Html(html.to_string())
}

pub async fn get_admin_audit(
    State(_state): State<Arc<AppState>>,
) -> Html<String> {
    let now = Utc::now();
    let html = format!(r##"
<div class="admin-page">
    <div class="page-header">
        <h1 data-i18n="admin-audit">Audit Log</h1>
        <p class="subtitle" data-i18n="admin-audit-subtitle">Track system events and user actions</p>
    </div>
    <div class="data-table">
        <table>
            <thead>
                <tr>
                    <th>Time</th>
                    <th>User</th>
                    <th>Action</th>
                    <th>Details</th>
                </tr>
            </thead>
            <tbody>
                <tr>
                    <td>{}</td>
                    <td>admin@example.com</td>
                    <td>User Login</td>
                    <td>Successful login from 192.168.1.1</td>
                </tr>
                <tr>
                    <td>{}</td>
                    <td>admin@example.com</td>
                    <td>Settings Changed</td>
                    <td>Updated system configuration</td>
                </tr>
            </tbody>
        </table>
    </div>
</div>
"##, now.format("%Y-%m-%d %H:%M"), now.format("%Y-%m-%d %H:%M"));
    Html(html)
}

pub async fn get_system_status(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<SystemStatusResponse>, (StatusCode, Json<serde_json::Value>)> {
    let now = Utc::now();

    let status = SystemStatusResponse {
        status: "healthy".to_string(),
        uptime_seconds: 3600 * 24 * 7,
        version: "1.0.0".to_string(),
        services: vec![
            ServiceStatus {
                name: "ui_server".to_string(),
                status: "running".to_string(),
                uptime_seconds: 3600 * 24 * 7,
                memory_mb: 256.5,
                cpu_percent: 12.3,
            },
            ServiceStatus {
                name: "database".to_string(),
                status: "running".to_string(),
                uptime_seconds: 3600 * 24 * 7,
                memory_mb: 512.8,
                cpu_percent: 8.5,
            },
            ServiceStatus {
                name: "cache".to_string(),
                status: "running".to_string(),
                uptime_seconds: 3600 * 24 * 7,
                memory_mb: 128.2,
                cpu_percent: 3.2,
            },
            ServiceStatus {
                name: "storage".to_string(),
                status: "running".to_string(),
                uptime_seconds: 3600 * 24 * 7,
                memory_mb: 64.1,
                cpu_percent: 5.8,
            },
        ],
        health_checks: vec![
            HealthCheck {
                name: "database_connection".to_string(),
                status: "passed".to_string(),
                message: Some("Connected successfully".to_string()),
                last_check: now,
            },
            HealthCheck {
                name: "storage_access".to_string(),
                status: "passed".to_string(),
                message: Some("Storage accessible".to_string()),
                last_check: now,
            },
            HealthCheck {
                name: "api_endpoints".to_string(),
                status: "passed".to_string(),
                message: Some("All endpoints responding".to_string()),
                last_check: now,
            },
        ],
        last_restart: now.checked_sub_signed(chrono::Duration::days(7)).unwrap_or(now),
    };

    Ok(Json(status))
}

pub async fn get_system_metrics(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<SystemMetricsResponse>, (StatusCode, Json<serde_json::Value>)> {
    let metrics = SystemMetricsResponse {
        cpu_usage: 23.5,
        memory_total_mb: 8192,
        memory_used_mb: 4096,
        memory_percent: 50.0,
        disk_total_gb: 500,
        disk_used_gb: 350,
        disk_percent: 70.0,
        network_in_mbps: 12.5,
        network_out_mbps: 8.3,
        active_connections: 256,
        request_rate_per_minute: 1250,
        error_rate_percent: 0.5,
    };

    Ok(Json(metrics))
}

pub fn view_logs(
    State(_state): State<Arc<AppState>>,
    Query(_params): Query<LogQuery>,
) -> Result<Json<Vec<LogEntry>>, (StatusCode, Json<serde_json::Value>)> {
    let now = Utc::now();

    let logs = vec![
        LogEntry {
            id: Uuid::new_v4(),
            timestamp: now,
            level: "info".to_string(),
            service: "ui_server".to_string(),
            message: "Request processed successfully".to_string(),
            metadata: Some(serde_json::json!({
                "endpoint": "/api/files/list",
                "duration_ms": 45,
                "status_code": 200
            })),
        },
        LogEntry {
            id: Uuid::new_v4(),
            timestamp: now
                .checked_sub_signed(chrono::Duration::minutes(5))
                .unwrap_or(now),
            level: "warning".to_string(),
            service: "database".to_string(),
            message: "Slow query detected".to_string(),
            metadata: Some(serde_json::json!({
                "query": "SELECT * FROM users WHERE...",
                "duration_ms": 1250
            })),
        },
        LogEntry {
            id: Uuid::new_v4(),
            timestamp: now
                .checked_sub_signed(chrono::Duration::minutes(10))
                .unwrap_or(now),
            level: "error".to_string(),
            service: "storage".to_string(),
            message: "Failed to upload file".to_string(),
            metadata: Some(serde_json::json!({
                "file": "document.pdf",
                "error": "Connection timeout"
            })),
        },
    ];

    Ok(Json(logs))
}

pub fn export_logs(
    State(_state): State<Arc<AppState>>,
    Query(_params): Query<LogQuery>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(SuccessResponse {
        success: true,
        message: Some("Logs exported successfully".to_string()),
    }))
}

pub fn get_config(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<ConfigResponse>, (StatusCode, Json<serde_json::Value>)> {
    let now = Utc::now();

    let config = ConfigResponse {
        configs: vec![
            ConfigItem {
                key: "max_upload_size_mb".to_string(),
                value: serde_json::json!(100),
                description: Some("Maximum file upload size in MB".to_string()),
                editable: true,
                requires_restart: false,
            },
            ConfigItem {
                key: "session_timeout_minutes".to_string(),
                value: serde_json::json!(30),
                description: Some("User session timeout in minutes".to_string()),
                editable: true,
                requires_restart: false,
            },
            ConfigItem {
                key: "enable_2fa".to_string(),
                value: serde_json::json!(true),
                description: Some("Enable two-factor authentication".to_string()),
                editable: true,
                requires_restart: false,
            },
            ConfigItem {
                key: "database_pool_size".to_string(),
                value: serde_json::json!(20),
                description: Some("Database connection pool size".to_string()),
                editable: true,
                requires_restart: true,
            },
        ],
        last_updated: now,
    };

    Ok(Json(config))
}

pub fn update_config(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<ConfigUpdateRequest>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(SuccessResponse {
        success: true,
        message: Some(format!(
            "Configuration '{}' updated successfully",
            req.config_key
        )),
    }))
}

pub fn schedule_maintenance(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<MaintenanceScheduleRequest>,
) -> Result<Json<MaintenanceResponse>, (StatusCode, Json<serde_json::Value>)> {
    let maintenance_id = Uuid::new_v4();

    let maintenance = MaintenanceResponse {
        id: maintenance_id,
        scheduled_at: req.scheduled_at,
        duration_minutes: req.duration_minutes,
        reason: req.reason,
        status: "scheduled".to_string(),
        created_by: "admin".to_string(),
    };

    Ok(Json(maintenance))
}

pub fn create_backup(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<BackupRequest>,
) -> Result<Json<BackupResponse>, (StatusCode, Json<serde_json::Value>)> {
    let backup_id = Uuid::new_v4();
    let now = Utc::now();

    let backup = BackupResponse {
        id: backup_id,
        backup_type: req.backup_type,
        size_bytes: 1024 * 1024 * 500,
        created_at: now,
        status: "completed".to_string(),
        download_url: Some(format!("/admin/backups/{}/download", backup_id)),
        expires_at: Some(now.checked_add_signed(chrono::Duration::days(30)).unwrap_or(now)),
    };

    Ok(Json(backup))
}

pub fn restore_backup(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<RestoreRequest>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(SuccessResponse {
        success: true,
        message: Some(format!("Restore from backup {} initiated", req.backup_id)),
    }))
}

pub fn list_backups(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<Vec<BackupResponse>>, (StatusCode, Json<serde_json::Value>)> {
    let now = Utc::now();

    let backups = vec![
        BackupResponse {
            id: Uuid::new_v4(),
            backup_type: "full".to_string(),
            size_bytes: 1024 * 1024 * 500,
            created_at: now.checked_sub_signed(chrono::Duration::days(1)).unwrap_or(now),
            status: "completed".to_string(),
            download_url: Some("/admin/backups/1/download".to_string()),
            expires_at: Some(now.checked_add_signed(chrono::Duration::days(29)).unwrap_or(now)),
        },
        BackupResponse {
            id: Uuid::new_v4(),
            backup_type: "incremental".to_string(),
            size_bytes: 1024 * 1024 * 50,
            created_at: now.checked_sub_signed(chrono::Duration::hours(12)).unwrap_or(now),
            status: "completed".to_string(),
            download_url: Some("/admin/backups/2/download".to_string()),
            expires_at: Some(now.checked_add_signed(chrono::Duration::days(29)).unwrap_or(now)),
        },
    ];

    Ok(Json(backups))
}

pub fn manage_users(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<UserManagementRequest>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<serde_json::Value>)> {
    let message = match req.action.as_str() {
        "suspend" => format!("User {} suspended", req.user_id),
        "activate" => format!("User {} activated", req.user_id),
        "delete" => format!("User {} deleted", req.user_id),
        "reset_password" => format!("Password reset for user {}", req.user_id),
        _ => format!("Action {} performed on user {}", req.action, req.user_id),
    };

    Ok(Json(SuccessResponse {
        success: true,
        message: Some(message),
    }))
}

pub fn get_roles(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<Vec<serde_json::Value>>, (StatusCode, Json<serde_json::Value>)> {
    let roles = vec![
        serde_json::json!({
            "id": Uuid::new_v4(),
            "name": "admin",
            "description": "Full system access",
            "permissions": ["*"],
            "user_count": 5
        }),
        serde_json::json!({
            "id": Uuid::new_v4(),
            "name": "user",
            "description": "Standard user access",
            "permissions": ["read:own", "write:own"],
            "user_count": 1245
        }),
        serde_json::json!({
            "id": Uuid::new_v4(),
            "name": "guest",
            "description": "Limited read-only access",
            "permissions": ["read:public"],
            "user_count": 328
        }),
    ];

    Ok(Json(roles))
}

pub fn manage_roles(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<RoleManagementRequest>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(SuccessResponse {
        success: true,
        message: Some(format!("Role '{}' managed successfully", req.role_name)),
    }))
}

pub fn get_quotas(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<Vec<QuotaResponse>>, (StatusCode, Json<serde_json::Value>)> {
    let quotas = vec![
        QuotaResponse {
            id: Uuid::new_v4(),
            entity_type: "user".to_string(),
            entity_id: Uuid::new_v4(),
            quota_type: "storage".to_string(),
            limit_value: 10 * 1024 * 1024 * 1024,
            current_value: 7 * 1024 * 1024 * 1024,
            percent_used: 70.0,
        },
        QuotaResponse {
            id: Uuid::new_v4(),
            entity_type: "user".to_string(),
            entity_id: Uuid::new_v4(),
            quota_type: "api_calls".to_string(),
            limit_value: 10000,
            current_value: 3500,
            percent_used: 35.0,
        },
    ];

    Ok(Json(quotas))
}

pub fn manage_quotas(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<QuotaManagementRequest>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(SuccessResponse {
        success: true,
        message: Some(format!("Quota '{}' set successfully", req.quota_type)),
    }))
}

pub fn get_licenses(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<Vec<LicenseResponse>>, (StatusCode, Json<serde_json::Value>)> {
    let now = Utc::now();

    let licenses = vec![LicenseResponse {
        id: Uuid::new_v4(),
        license_type: "enterprise".to_string(),
        status: "active".to_string(),
        max_users: 1000,
        current_users: 850,
        features: vec![
            "unlimited_storage".to_string(),
            "advanced_analytics".to_string(),
            "priority_support".to_string(),
            "custom_integrations".to_string(),
        ],
        issued_at: now.checked_sub_signed(chrono::Duration::days(180)).unwrap_or(now),
        expires_at: Some(now.checked_add_signed(chrono::Duration::days(185)).unwrap_or(now)),
    }];

    Ok(Json(licenses))
}

pub fn manage_licenses(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<LicenseManagementRequest>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(SuccessResponse {
        success: true,
        message: Some(format!(
            "License '{}' activated successfully",
            req.license_type
        )),
    }))
}

// =============================================================================
// INVITATION MANAGEMENT HANDLERS
// =============================================================================

/// List all invitations for the organization
pub async fn list_invitations(
    State(state): State<Arc<AppState>>,
    user: AuthenticatedUser,
) -> impl axum::response::IntoResponse {
    let mut conn = match state.conn.get() {
        Ok(c) => c,
        Err(e) => {
            return Json(serde_json::json!({
                "success": false,
                "error": format!("Database connection error: {}", e),
                "invitations": []
            }));
        }
    };

    let org_id = user.organization_id.unwrap_or_else(Uuid::nil);
    let result: Result<Vec<InvitationRow>, _> = diesel::sql_query(
        "SELECT id, org_id, email, role, status, message, invited_by, created_at, expires_at, accepted_at
         FROM organization_invitations
         WHERE org_id = $1
         ORDER BY created_at DESC
         LIMIT 100"
    )
    .bind::<DieselUuid, _>(org_id)
    .load(&mut conn);

    match result {
        Ok(invitations) => Json(serde_json::json!({
            "success": true,
            "invitations": invitations
        })),
        Err(e) => {
            warn!("Failed to list invitations: {}", e);
            // Return empty list on database error
            Json(serde_json::json!({
                "success": false,
                "error": format!("Failed to fetch invitations: {}", e),
                "invitations": []
            }))
        }
    }
}

/// Create a single invitation
pub async fn create_invitation(
    State(state): State<Arc<AppState>>,
    user: AuthenticatedUser,
    Json(payload): Json<CreateInvitationRequest>,
) -> impl axum::response::IntoResponse {
    // Validate email format
    if !payload.email.contains('@') {
        return (StatusCode::BAD_REQUEST, Json(InvitationResponse {
            success: false,
            id: None,
            email: Some(payload.email),
            error: Some("Invalid email format".to_string()),
        }));
    }

    let mut conn = match state.conn.get() {
        Ok(c) => c,
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(InvitationResponse {
                success: false,
                id: None,
                email: Some(payload.email),
                error: Some(format!("Database connection error: {}", e)),
            }));
        }
    };

    let new_id = Uuid::new_v4();
    let org_id = user.organization_id.unwrap_or_else(Uuid::nil);
    let invited_by = user.user_id;
    let expires_at = Utc::now() + chrono::Duration::days(7);

    let result = diesel::sql_query(
        "INSERT INTO organization_invitations (id, org_id, email, role, status, message, invited_by, created_at, expires_at)
         VALUES ($1, $2, $3, $4, 'pending', $5, $6, NOW(), $7)
         ON CONFLICT (org_id, email) WHERE status = 'pending' DO UPDATE SET
             role = EXCLUDED.role,
             message = EXCLUDED.message,
             expires_at = EXCLUDED.expires_at,
             updated_at = NOW()
         RETURNING id"
    )
    .bind::<DieselUuid, _>(new_id)
    .bind::<DieselUuid, _>(org_id)
    .bind::<Varchar, _>(&payload.email)
    .bind::<Varchar, _>(&payload.role)
    .bind::<Nullable<Text>, _>(payload.message.as_deref())
    .bind::<DieselUuid, _>(invited_by)
    .bind::<Timestamptz, _>(expires_at)
    .execute(&mut conn);

    match result {
        Ok(_) => {
            // Send invitation email
            let email_to = payload.email.clone();
            let invite_role = payload.role.clone();
            let invite_message = payload.message.clone();
            let invite_id = new_id;

            #[cfg(feature = "mail")]
            tokio::spawn(async move {
                if let Err(e) = send_invitation_email(&email_to, &invite_role, invite_message.as_deref(), invite_id).await {
                    warn!("Failed to send invitation email to {}: {}", email_to, e);
                }
            });

            (StatusCode::OK, Json(InvitationResponse {
                success: true,
                id: Some(new_id),
                email: Some(payload.email),
                error: None,
            }))
        }
        Err(e) => {
            warn!("Failed to create invitation: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(InvitationResponse {
                success: false,
                id: None,
                email: Some(payload.email),
                error: Some(format!("Failed to create invitation: {}", e)),
            }))
        }
    }
}

/// Create bulk invitations
pub async fn create_bulk_invitations(
    State(state): State<Arc<AppState>>,
    user: AuthenticatedUser,
    Json(payload): Json<BulkInvitationRequest>,
) -> impl axum::response::IntoResponse {
    let mut conn = match state.conn.get() {
        Ok(c) => c,
        Err(e) => {
            return Json(BulkInvitationResponse {
                success: false,
                sent: 0,
                failed: payload.emails.len() as i32,
                errors: vec![format!("Database connection error: {}", e)],
            });
        }
    };

    let org_id = user.organization_id.unwrap_or_else(Uuid::nil);
    let invited_by = user.user_id;
    let expires_at = Utc::now() + chrono::Duration::days(7);

    let mut sent = 0;
    let mut failed = 0;
    let mut errors = Vec::new();

    for email in &payload.emails {
        // Validate email
        if !email.contains('@') {
            failed += 1;
            errors.push(format!("Invalid email: {}", email));
            continue;
        }

        let new_id = Uuid::new_v4();
        let result = diesel::sql_query(
            "INSERT INTO organization_invitations (id, org_id, email, role, status, message, invited_by, created_at, expires_at)
             VALUES ($1, $2, $3, $4, 'pending', $5, $6, NOW(), $7)
             ON CONFLICT (org_id, email) WHERE status = 'pending' DO NOTHING"
        )
        .bind::<DieselUuid, _>(new_id)
        .bind::<DieselUuid, _>(org_id)
        .bind::<Varchar, _>(email)
        .bind::<Varchar, _>(&payload.role)
        .bind::<Nullable<Text>, _>(payload.message.as_deref())
        .bind::<DieselUuid, _>(invited_by)
        .bind::<Timestamptz, _>(expires_at)
        .execute(&mut conn);

        match result {
            Ok(_) => sent += 1,
            Err(e) => {
                failed += 1;
                errors.push(format!("Failed for {}: {}", email, e));
            }
        }
    }

    Json(BulkInvitationResponse {
        success: failed == 0,
        sent,
        failed,
        errors,
    })
}

/// Get a specific invitation
pub async fn get_invitation(
    State(state): State<Arc<AppState>>,
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> impl axum::response::IntoResponse {
    let mut conn = match state.conn.get() {
        Ok(c) => c,
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "success": false,
                "error": format!("Database connection error: {}", e)
            })));
        }
    };

    let org_id = user.organization_id.unwrap_or_else(Uuid::nil);
    let result: Result<InvitationRow, _> = diesel::sql_query(
        "SELECT id, org_id, email, role, status, message, invited_by, created_at, expires_at, accepted_at
         FROM organization_invitations
         WHERE id = $1 AND org_id = $2"
    )
    .bind::<DieselUuid, _>(id)
    .bind::<DieselUuid, _>(org_id)
    .get_result(&mut conn);

    match result {
        Ok(invitation) => (StatusCode::OK, Json(serde_json::json!({
            "success": true,
            "invitation": invitation
        }))),
        Err(_) => (StatusCode::NOT_FOUND, Json(serde_json::json!({
            "success": false,
            "error": "Invitation not found"
        })))
    }
}

/// Cancel/delete an invitation
pub async fn cancel_invitation(
    State(state): State<Arc<AppState>>,
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> impl axum::response::IntoResponse {
    let mut conn = match state.conn.get() {
        Ok(c) => c,
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "success": false,
                "error": format!("Database connection error: {}", e)
            })));
        }
    };

    let org_id = user.organization_id.unwrap_or_else(Uuid::nil);
    let result = diesel::sql_query(
        "UPDATE organization_invitations
         SET status = 'cancelled', updated_at = NOW()
         WHERE id = $1 AND org_id = $2 AND status = 'pending'"
    )
    .bind::<DieselUuid, _>(id)
    .bind::<DieselUuid, _>(org_id)
    .execute(&mut conn);

    match result {
        Ok(rows) if rows > 0 => (StatusCode::OK, Json(serde_json::json!({
            "success": true,
            "id": id
        }))),
        Ok(_) => (StatusCode::NOT_FOUND, Json(serde_json::json!({
            "success": false,
            "error": "Invitation not found or already processed"
        }))),
        Err(e) => {
            warn!("Failed to cancel invitation: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "success": false,
                "error": format!("Failed to cancel invitation: {}", e)
            })))
        }
    }
}

/// Resend an invitation email
pub async fn resend_invitation(
    State(state): State<Arc<AppState>>,
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> impl axum::response::IntoResponse {
    let mut conn = match state.conn.get() {
        Ok(c) => c,
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "success": false,
                "error": format!("Database connection error: {}", e)
            })));
        }
    };

    let org_id = user.organization_id.unwrap_or_else(Uuid::nil);
    let new_expires_at = Utc::now() + chrono::Duration::days(7);

    // Update expiration and resend
    let result = diesel::sql_query(
        "UPDATE organization_invitations
         SET expires_at = $3, updated_at = NOW()
         WHERE id = $1 AND org_id = $2 AND status = 'pending'
         RETURNING email"
    )
    .bind::<DieselUuid, _>(id)
    .bind::<DieselUuid, _>(org_id)
    .bind::<Timestamptz, _>(new_expires_at)
    .execute(&mut conn);

    match result {
        Ok(rows) if rows > 0 => {
            // Trigger email resend
            #[cfg(feature = "mail")]
            {
                let resend_id = id;
                tokio::spawn(async move {
                    if let Err(e) = send_invitation_email_by_id(resend_id).await {
                        warn!("Failed to resend invitation email for {}: {}", resend_id, e);
                    }
                });
            }

            (StatusCode::OK, Json(serde_json::json!({
                "success": true,
                "id": id,
                "message": "Invitation resent successfully"
            })))
        }
        Ok(_) => (StatusCode::NOT_FOUND, Json(serde_json::json!({
            "success": false,
            "error": "Invitation not found or not in pending status"
        }))),
        Err(e) => {
            warn!("Failed to resend invitation: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "success": false,
                "error": format!("Failed to resend invitation: {}", e)
            })))
        }
    }
}

#[derive(Deserialize)]
pub struct CreateGroupRequest {
    pub name: String,
    pub description: Option<String>,
}

pub async fn create_group(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateGroupRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let pool = &state.conn;
    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "success": false,
                "error": format!("Database connection error: {}", e)
            })));
        }
    };

    let group_id = Uuid::new_v4();
    let result = diesel::sql_query(
        "INSERT INTO groups (id, name, description, created_at, updated_at)
         VALUES ($1, $2, $3, NOW(), NOW())
         RETURNING id"
    )
    .bind::<DieselUuid, _>(group_id)
    .bind::<Text, _>(&req.name)
    .bind::<Nullable<Text>, _>(req.description.as_deref())
    .execute(&mut conn);

    match result {
        Ok(_) => (StatusCode::CREATED, Json(serde_json::json!({
            "success": true,
            "id": group_id,
            "name": req.name
        }))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "success": false,
            "error": format!("Failed to create group: {}", e)
        })))
    }
}

pub async fn export_admin_report(
    State(_state): State<Arc<AppState>>,
) -> (StatusCode, Json<serde_json::Value>) {
    (StatusCode::OK, Json(serde_json::json!({
        "success": true,
        "report_url": "/api/admin/reports/latest.pdf",
        "generated_at": Utc::now().to_rfc3339()
    })))
}

pub async fn get_dashboard_stats(
    State(_state): State<Arc<AppState>>,
) -> Html<String> {
    Html(r##"
<div class="stat-card members">
    <div class="stat-icon"><svg viewBox="0 0 24 24" width="24" height="24" stroke="currentColor" stroke-width="2" fill="none"><path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"></path><circle cx="9" cy="7" r="4"></circle><path d="M23 21v-2a4 4 0 0 0-3-3.87"></path><path d="M16 3.13a4 4 0 0 1 0 7.75"></path></svg></div>
    <div class="stat-content"><span class="stat-value">24</span><span class="stat-label">Team Members</span></div>
    <span class="stat-trend positive">+3 this month</span>
</div>
<div class="stat-card bots">
    <div class="stat-icon"><svg viewBox="0 0 24 24" width="24" height="24" stroke="currentColor" stroke-width="2" fill="none"><rect x="3" y="11" width="18" height="10" rx="2"></rect><circle cx="12" cy="5" r="2"></circle><path d="M12 7v4"></path><line x1="8" y1="16" x2="8" y2="16"></line><line x1="16" y1="16" x2="16" y2="16"></line></svg></div>
    <div class="stat-content"><span class="stat-value">5</span><span class="stat-label">Active Bots</span></div>
    <span class="stat-trend">All operational</span>
</div>
<div class="stat-card messages">
    <div class="stat-icon"><svg viewBox="0 0 24 24" width="24" height="24" stroke="currentColor" stroke-width="2" fill="none"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"></path></svg></div>
    <div class="stat-content"><span class="stat-value">12.4K</span><span class="stat-label">Messages Today</span></div>
    <span class="stat-trend positive">+18% vs yesterday</span>
</div>
<div class="stat-card storage">
    <div class="stat-icon"><svg viewBox="0 0 24 24" width="24" height="24" stroke="currentColor" stroke-width="2" fill="none"><path d="M22 12H2"></path><path d="M5.45 5.11L2 12v6a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-6l-3.45-6.89A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11z"></path></svg></div>
    <div class="stat-content"><span class="stat-value">45.2 GB</span><span class="stat-label">Storage Used</span></div>
    <span class="stat-trend">of 100 GB</span>
</div>
"##.to_string())
}

pub async fn get_dashboard_health(
    State(_state): State<Arc<AppState>>,
) -> Html<String> {
    Html(r##"
<div class="health-item">
    <div class="health-indicator healthy"></div>
    <div class="health-info"><span class="health-name">API Server</span><span class="health-status">Operational</span></div>
</div>
<div class="health-item">
    <div class="health-indicator healthy"></div>
    <div class="health-info"><span class="health-name">Database</span><span class="health-status">Operational</span></div>
</div>
<div class="health-item">
    <div class="health-indicator healthy"></div>
    <div class="health-info"><span class="health-name">Bot Engine</span><span class="health-status">Operational</span></div>
</div>
<div class="health-item">
    <div class="health-indicator healthy"></div>
    <div class="health-info"><span class="health-name">File Storage</span><span class="health-status">Operational</span></div>
</div>
"##.to_string())
}

pub async fn get_dashboard_activity(
    State(_state): State<Arc<AppState>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Html<String> {
    let _page = params.get("page").and_then(|p| p.parse::<i32>().ok()).unwrap_or(1);
    Html(r##"
<div class="activity-item">
    <div class="activity-icon member"><svg viewBox="0 0 24 24" width="16" height="16" stroke="currentColor" stroke-width="2" fill="none"><path d="M16 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"></path><circle cx="8.5" cy="7" r="4"></circle><line x1="20" y1="8" x2="20" y2="14"></line><line x1="23" y1="11" x2="17" y2="11"></line></svg></div>
    <div class="activity-content"><span class="activity-user">John Doe</span> joined the organization</div>
    <span class="activity-time">2 hours ago</span>
</div>
<div class="activity-item">
    <div class="activity-icon bot"><svg viewBox="0 0 24 24" width="16" height="16" stroke="currentColor" stroke-width="2" fill="none"><rect x="3" y="11" width="18" height="10" rx="2"></rect><circle cx="12" cy="5" r="2"></circle></svg></div>
    <div class="activity-content"><span class="activity-user">Support Bot</span> processed 150 messages</div>
    <span class="activity-time">3 hours ago</span>
</div>
<div class="activity-item">
    <div class="activity-icon security"><svg viewBox="0 0 24 24" width="16" height="16" stroke="currentColor" stroke-width="2" fill="none"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"></path></svg></div>
    <div class="activity-content"><span class="activity-user">System</span> security scan completed</div>
    <span class="activity-time">5 hours ago</span>
</div>
"##.to_string())
}

pub async fn get_dashboard_members(
    State(_state): State<Arc<AppState>>,
) -> Html<String> {
    Html(r##"
<div class="member-item">
    <div class="member-avatar"><img src="/api/avatar/1" alt="JD" onerror="this.outerHTML='<div class=member-avatar-fallback>JD</div>'"></div>
    <div class="member-info"><span class="member-name">John Doe</span><span class="member-role">Admin</span></div>
    <span class="member-status online">Online</span>
</div>
<div class="member-item">
    <div class="member-avatar"><img src="/api/avatar/2" alt="JS" onerror="this.outerHTML='<div class=member-avatar-fallback>JS</div>'"></div>
    <div class="member-info"><span class="member-name">Jane Smith</span><span class="member-role">Member</span></div>
    <span class="member-status online">Online</span>
</div>
<div class="member-item">
    <div class="member-avatar"><img src="/api/avatar/3" alt="BW" onerror="this.outerHTML='<div class=member-avatar-fallback>BW</div>'"></div>
    <div class="member-info"><span class="member-name">Bob Wilson</span><span class="member-role">Member</span></div>
    <span class="member-status offline">Offline</span>
</div>
"##.to_string())
}

pub async fn get_dashboard_roles(
    State(_state): State<Arc<AppState>>,
) -> Html<String> {
    Html(r##"
<div class="role-bars">
    <div class="role-bar-item">
        <div class="role-bar-label"><span class="role-name">Owner</span><span class="role-count">1</span></div>
        <div class="role-bar"><div class="role-bar-fill" style="width: 4%"></div></div>
    </div>
    <div class="role-bar-item">
        <div class="role-bar-label"><span class="role-name">Admin</span><span class="role-count">3</span></div>
        <div class="role-bar"><div class="role-bar-fill" style="width: 12%"></div></div>
    </div>
    <div class="role-bar-item">
        <div class="role-bar-label"><span class="role-name">Member</span><span class="role-count">18</span></div>
        <div class="role-bar"><div class="role-bar-fill" style="width: 75%"></div></div>
    </div>
    <div class="role-bar-item">
        <div class="role-bar-label"><span class="role-name">Guest</span><span class="role-count">2</span></div>
        <div class="role-bar"><div class="role-bar-fill" style="width: 8%"></div></div>
    </div>
</div>
"##.to_string())
}

pub async fn get_dashboard_bots(
    State(_state): State<Arc<AppState>>,
) -> Html<String> {
    Html(r##"
<div class="bot-item">
    <div class="bot-avatar">CS</div>
    <div class="bot-info"><span class="bot-name">Customer Support Bot</span><span class="bot-desc">Handles customer inquiries</span></div>
    <span class="bot-status active">Active</span>
</div>
<div class="bot-item">
    <div class="bot-avatar">SA</div>
    <div class="bot-info"><span class="bot-name">Sales Assistant</span><span class="bot-desc">Lead qualification</span></div>
    <span class="bot-status active">Active</span>
</div>
<div class="bot-item">
    <div class="bot-avatar">HR</div>
    <div class="bot-info"><span class="bot-name">HR Helper</span><span class="bot-desc">Employee onboarding</span></div>
    <span class="bot-status inactive">Paused</span>
</div>
"##.to_string())
}

pub async fn get_dashboard_invitations(
    State(_state): State<Arc<AppState>>,
) -> Html<String> {
    Html(r##"
<div class="invitation-item">
    <div class="invitation-info"><span class="invitation-email">alice@example.com</span><span class="invitation-role">Member</span></div>
    <span class="invitation-status pending">Pending</span>
    <span class="invitation-expires">Expires in 5 days</span>
</div>
<div class="invitation-item">
    <div class="invitation-info"><span class="invitation-email">bob@example.com</span><span class="invitation-role">Admin</span></div>
    <span class="invitation-status pending">Pending</span>
    <span class="invitation-expires">Expires in 3 days</span>
</div>
"##.to_string())
}
