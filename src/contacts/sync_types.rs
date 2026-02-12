// Sync types extracted from external_sync.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: i64,
    pub expires_at: Option<DateTime<Utc>>,
    pub scopes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportResult {
    Created,
    Updated,
    Skipped,
    Conflict,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExportResult {
    Created,
    Updated,
    Deleted,
    Skipped,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExternalProvider {
    Google,
    Microsoft,
    Apple,
    CardDav,
}

impl std::fmt::Display for ExternalProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExternalProvider::Google => write!(f, "google"),
            ExternalProvider::Microsoft => write!(f, "microsoft"),
            ExternalProvider::Apple => write!(f, "apple"),
            ExternalProvider::CardDav => write!(f, "carddav"),
        }
    }
}

impl std::str::FromStr for ExternalProvider {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "google" => Ok(ExternalProvider::Google),
            "microsoft" => Ok(ExternalProvider::Microsoft),
            "apple" => Ok(ExternalProvider::Apple),
            "carddav" => Ok(ExternalProvider::CardDav),
            _ => Err(format!("Unsupported provider: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum SyncDirection {
    #[default]
    TwoWay,
    ImportOnly,
    ExportOnly,
}

impl std::fmt::Display for SyncDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncDirection::TwoWay => write!(f, "two_way"),
            SyncDirection::ImportOnly => write!(f, "import_only"),
            SyncDirection::ExportOnly => write!(f, "export_only"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SyncStatus {
    Success,
    Synced,
    PartialSuccess,
    Failed,
    InProgress,
    Cancelled,
}

impl std::fmt::Display for SyncStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Success => write!(f, "success"),
            Self::Synced => write!(f, "synced"),
            Self::PartialSuccess => write!(f, "partial_success"),
            Self::Failed => write!(f, "failed"),
            Self::InProgress => write!(f, "in_progress"),
            Self::Cancelled => write!(f, "cancelled"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MappingSyncStatus {
    Synced,
    PendingUpload,
    PendingDownload,
    Conflict,
    Error,
    Deleted,
}

impl std::fmt::Display for MappingSyncStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MappingSyncStatus::Synced => write!(f, "synced"),
            MappingSyncStatus::PendingUpload => write!(f, "pending_upload"),
            MappingSyncStatus::PendingDownload => write!(f, "pending_download"),
            MappingSyncStatus::Conflict => write!(f, "conflict"),
            MappingSyncStatus::Error => write!(f, "error"),
            MappingSyncStatus::Deleted => write!(f, "deleted"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConflictResolution {
    KeepInternal,
    KeepExternal,
    KeepLocal,
    KeepRemote,
    Manual,
    Merge,
    Skip,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncTrigger {
    Manual,
    Scheduled,
    Webhook,
    ContactChange,
}

impl std::fmt::Display for SyncTrigger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncTrigger::Manual => write!(f, "manual"),
            SyncTrigger::Scheduled => write!(f, "scheduled"),
            SyncTrigger::Webhook => write!(f, "webhook"),
            SyncTrigger::ContactChange => write!(f, "contact_change"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactMapping {
    pub id: Uuid,
    pub account_id: Uuid,
    pub contact_id: Uuid,
    pub local_contact_id: Uuid,
    pub external_id: String,
    pub external_contact_id: String,
    pub external_etag: Option<String>,
    pub internal_version: i64,
    pub last_synced_at: DateTime<Utc>,
    pub sync_status: MappingSyncStatus,
    pub conflict_data: Option<ConflictData>,
    pub local_data: Option<serde_json::Value>,
    pub remote_data: Option<serde_json::Value>,
    pub conflict_detected_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictData {
    pub detected_at: DateTime<Utc>,
    pub internal_changes: Vec<String>,
    pub external_changes: Vec<String>,
    pub resolution: Option<ConflictResolution>,
    pub resolved_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncHistory {
    pub id: Uuid,
    pub account_id: Uuid,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: SyncStatus,
    pub direction: SyncDirection,
    pub contacts_created: u32,
    pub contacts_updated: u32,
    pub contacts_deleted: u32,
    pub contacts_skipped: u32,
    pub conflicts_detected: u32,
    pub errors: Vec<SyncError>,
    pub triggered_by: SyncTrigger,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncError {
    pub contact_id: Option<Uuid>,
    pub external_id: Option<String>,
    pub operation: String,
    pub error_code: String,
    pub error_message: String,
    pub retryable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalAccount {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub user_id: Uuid,
    pub provider: ExternalProvider,
    pub external_account_id: String,
    pub email: String,
    pub display_name: Option<String>,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_expires_at: Option<DateTime<Utc>>,
    pub scopes: Vec<String>,
    pub sync_enabled: bool,
    pub sync_direction: SyncDirection,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub last_sync_status: Option<String>,
    pub sync_cursor: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectAccountRequest {
    pub provider: ExternalProvider,
    pub authorization_code: String,
    pub redirect_uri: String,
    pub sync_direction: Option<SyncDirection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationUrlResponse {
    pub url: String,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartSyncRequest {
    pub full_sync: Option<bool>,
    pub direction: Option<SyncDirection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncProgressResponse {
    pub sync_id: Uuid,
    pub status: SyncStatus,
    pub progress_percent: u8,
    pub contacts_processed: u32,
    pub total_contacts: u32,
    pub current_operation: String,
    pub started_at: DateTime<Utc>,
    pub estimated_completion: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveConflictRequest {
    pub resolution: ConflictResolution,
    pub merged_data: Option<MergedContactData>,
    pub manual_data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergedContactData {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub company: Option<String>,
    pub job_title: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncSettings {
    pub sync_enabled: bool,
    pub sync_direction: SyncDirection,
    pub auto_sync_interval_minutes: u32,
    pub sync_contact_groups: bool,
    pub sync_photos: bool,
    pub conflict_resolution: ConflictResolution,
    pub field_mapping: HashMap<String, String>,
    pub exclude_tags: Vec<String>,
    pub include_only_tags: Vec<String>,
}

impl Default for SyncSettings {
    fn default() -> Self {
        Self {
            sync_enabled: true,
            sync_direction: SyncDirection::TwoWay,
            auto_sync_interval_minutes: 60,
            sync_contact_groups: true,
            sync_photos: true,
            conflict_resolution: ConflictResolution::KeepInternal,
            field_mapping: HashMap::new(),
            exclude_tags: vec![],
            include_only_tags: vec![],
        }
    }
}
