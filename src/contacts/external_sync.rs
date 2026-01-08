//! External Address Book Synchronization Module
//!
//! This module provides synchronization between the internal Contacts app
//! and external address book providers like Google Contacts and Microsoft
//! People (Outlook/Office 365).

use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::shared::state::AppState;
use crate::shared::utils::DbPool;

/// Supported external providers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
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
    type Err = ExternalSyncError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "google" => Ok(ExternalProvider::Google),
            "microsoft" => Ok(ExternalProvider::Microsoft),
            "apple" => Ok(ExternalProvider::Apple),
            "carddav" => Ok(ExternalProvider::CardDav),
            _ => Err(ExternalSyncError::UnsupportedProvider(s.to_string())),
        }
    }
}

/// External account connection
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
    pub last_sync_status: Option<SyncStatus>,
    pub sync_cursor: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Sync direction configuration
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

/// Sync operation status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SyncStatus {
    Success,
    PartialSuccess,
    Failed,
    InProgress,
    Cancelled,
}

impl std::fmt::Display for SyncStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncStatus::Success => write!(f, "success"),
            SyncStatus::PartialSuccess => write!(f, "partial_success"),
            SyncStatus::Failed => write!(f, "failed"),
            SyncStatus::InProgress => write!(f, "in_progress"),
            SyncStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

/// Mapping between internal and external contact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactMapping {
    pub id: Uuid,
    pub account_id: Uuid,
    pub internal_contact_id: Uuid,
    pub external_contact_id: String,
    pub external_etag: Option<String>,
    pub internal_version: i64,
    pub last_synced_at: DateTime<Utc>,
    pub sync_status: MappingSyncStatus,
    pub conflict_data: Option<ConflictData>,
}

/// Sync status for individual contact mapping
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

/// Conflict information when sync encounters conflicting changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictData {
    pub detected_at: DateTime<Utc>,
    pub internal_changes: Vec<String>,
    pub external_changes: Vec<String>,
    pub resolution: Option<ConflictResolution>,
    pub resolved_at: Option<DateTime<Utc>>,
}

/// How to resolve a sync conflict
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConflictResolution {
    KeepInternal,
    KeepExternal,
    Merge,
    Skip,
}

/// Sync history record
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

/// What triggered the sync
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

/// Individual sync error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncError {
    pub contact_id: Option<Uuid>,
    pub external_id: Option<String>,
    pub operation: String,
    pub error_code: String,
    pub error_message: String,
    pub retryable: bool,
}

/// Request to connect an external account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectAccountRequest {
    pub provider: ExternalProvider,
    pub authorization_code: String,
    pub redirect_uri: String,
    pub sync_direction: Option<SyncDirection>,
}

/// Response with OAuth authorization URL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationUrlResponse {
    pub url: String,
    pub state: String,
}

/// Request to start manual sync
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartSyncRequest {
    pub full_sync: Option<bool>,
    pub direction: Option<SyncDirection>,
}

/// Sync progress response
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

/// Request to resolve a conflict
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveConflictRequest {
    pub resolution: ConflictResolution,
    pub merged_data: Option<MergedContactData>,
}

/// Merged contact data for manual conflict resolution
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

/// Sync settings for an account
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

/// Account status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountStatusResponse {
    pub account: ExternalAccount,
    pub sync_stats: SyncStats,
    pub pending_conflicts: u32,
    pub pending_errors: u32,
    pub next_scheduled_sync: Option<DateTime<Utc>>,
}

/// Sync statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStats {
    pub total_synced_contacts: u32,
    pub total_syncs: u32,
    pub successful_syncs: u32,
    pub failed_syncs: u32,
    pub last_successful_sync: Option<DateTime<Utc>>,
    pub average_sync_duration_seconds: u32,
}

/// External contact representation (provider-agnostic)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalContact {
    pub id: String,
    pub etag: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub display_name: Option<String>,
    pub email_addresses: Vec<ExternalEmail>,
    pub phone_numbers: Vec<ExternalPhone>,
    pub addresses: Vec<ExternalAddress>,
    pub company: Option<String>,
    pub job_title: Option<String>,
    pub department: Option<String>,
    pub notes: Option<String>,
    pub birthday: Option<String>,
    pub photo_url: Option<String>,
    pub groups: Vec<String>,
    pub custom_fields: HashMap<String, String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalEmail {
    pub address: String,
    pub label: Option<String>,
    pub primary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalPhone {
    pub number: String,
    pub label: Option<String>,
    pub primary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalAddress {
    pub street: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub label: Option<String>,
    pub primary: bool,
}

/// External sync service
pub struct ExternalSyncService {
    pool: DbPool,
    google_client: GoogleContactsClient,
    microsoft_client: MicrosoftPeopleClient,
}

impl ExternalSyncService {
    pub fn new(pool: DbPool, google_config: GoogleConfig, microsoft_config: MicrosoftConfig) -> Self {
        Self {
            pool,
            google_client: GoogleContactsClient::new(google_config),
            microsoft_client: MicrosoftPeopleClient::new(microsoft_config),
        }
    }

    /// Get OAuth authorization URL for a provider
    pub fn get_authorization_url(
        &self,
        provider: &ExternalProvider,
        redirect_uri: &str,
        state: &str,
    ) -> Result<AuthorizationUrlResponse, ExternalSyncError> {
        let url = match provider {
            ExternalProvider::Google => self.google_client.get_auth_url(redirect_uri, state),
            ExternalProvider::Microsoft => self.microsoft_client.get_auth_url(redirect_uri, state),
            ExternalProvider::Apple => {
                return Err(ExternalSyncError::UnsupportedProvider("Apple".to_string()))
            }
            ExternalProvider::CardDav => {
                return Err(ExternalSyncError::UnsupportedProvider(
                    "CardDAV requires direct configuration".to_string(),
                ))
            }
        };

        Ok(AuthorizationUrlResponse {
            url,
            state: state.to_string(),
        })
    }

    /// Connect an external account using OAuth authorization code
    pub async fn connect_account(
        &self,
        organization_id: Uuid,
        user_id: Uuid,
        request: &ConnectAccountRequest,
    ) -> Result<ExternalAccount, ExternalSyncError> {
        // Exchange authorization code for tokens
        let tokens = match request.provider {
            ExternalProvider::Google => {
                self.google_client
                    .exchange_code(&request.authorization_code, &request.redirect_uri)
                    .await?
            }
            ExternalProvider::Microsoft => {
                self.microsoft_client
                    .exchange_code(&request.authorization_code, &request.redirect_uri)
                    .await?
            }
            _ => {
                return Err(ExternalSyncError::UnsupportedProvider(
                    request.provider.to_string(),
                ))
            }
        };

        // Get user info from provider
        let user_info = match request.provider {
            ExternalProvider::Google => {
                self.google_client.get_user_info(&tokens.access_token).await?
            }
            ExternalProvider::Microsoft => {
                self.microsoft_client
                    .get_user_info(&tokens.access_token)
                    .await?
            }
            _ => unreachable!(),
        };

        // Check if account already exists
        if let Some(existing) = self
            .find_existing_account(organization_id, &request.provider, &user_info.id)
            .await?
        {
            // Update tokens
            return self
                .update_account_tokens(existing.id, &tokens)
                .await;
        }

        // Create new account
        let account_id = Uuid::new_v4();
        let now = Utc::now();

        let account = ExternalAccount {
            id: account_id,
            organization_id,
            user_id,
            provider: request.provider.clone(),
            external_account_id: user_info.id,
            email: user_info.email,
            display_name: user_info.name,
            access_token: tokens.access_token,
            refresh_token: tokens.refresh_token,
            token_expires_at: tokens.expires_at,
            scopes: tokens.scopes,
            sync_enabled: true,
            sync_direction: request.sync_direction.clone().unwrap_or_default(),
            last_sync_at: None,
            last_sync_status: None,
            sync_cursor: None,
            created_at: now,
            updated_at: now,
        };

        self.save_account(&account).await?;

        Ok(account)
    }

    /// Disconnect an external account
    pub async fn disconnect_account(
        &self,
        organization_id: Uuid,
        account_id: Uuid,
    ) -> Result<(), ExternalSyncError> {
        let account = self.get_account(account_id).await?;

        if account.organization_id != organization_id {
            return Err(ExternalSyncError::Unauthorized);
        }

        // Revoke tokens with provider
        match account.provider {
            ExternalProvider::Google => {
                let _ = self.google_client.revoke_token(&account.access_token).await;
            }
            ExternalProvider::Microsoft => {
                let _ = self
                    .microsoft_client
                    .revoke_token(&account.access_token)
                    .await;
            }
            _ => {}
        }

        // Delete account and mappings
        self.delete_account(account_id).await?;

        Ok(())
    }

    /// Start a sync operation
    pub async fn start_sync(
        &self,
        organization_id: Uuid,
        account_id: Uuid,
        request: &StartSyncRequest,
        trigger: SyncTrigger,
    ) -> Result<SyncHistory, ExternalSyncError> {
        let account = self.get_account(account_id).await?;

        if account.organization_id != organization_id {
            return Err(ExternalSyncError::Unauthorized);
        }

        if !account.sync_enabled {
            return Err(ExternalSyncError::SyncDisabled);
        }

        // Check if sync is already in progress
        if let Some(last_status) = &account.last_sync_status {
            if *last_status == SyncStatus::InProgress {
                return Err(ExternalSyncError::SyncInProgress);
            }
        }

        // Refresh token if needed
        let account = self.ensure_valid_token(account).await?;

        let sync_id = Uuid::new_v4();
        let now = Utc::now();
        let direction = request.direction.clone().unwrap_or(account.sync_direction.clone());

        let mut history = SyncHistory {
            id: sync_id,
            account_id,
            started_at: now,
            completed_at: None,
            status: SyncStatus::InProgress,
            direction: direction.clone(),
            contacts_created: 0,
            contacts_updated: 0,
            contacts_deleted: 0,
            contacts_skipped: 0,
            conflicts_detected: 0,
            errors: vec![],
            triggered_by: trigger,
        };

        self.save_sync_history(&history).await?;
        self.update_account_sync_status(account_id, SyncStatus::InProgress)
            .await?;

        // Perform sync based on direction
        let result = match direction {
            SyncDirection::TwoWay => {
                self.perform_two_way_sync(&account, request.full_sync.unwrap_or(false), &mut history)
                    .await
            }
            SyncDirection::ImportOnly => {
                self.perform_import_sync(&account, request.full_sync.unwrap_or(false), &mut history)
                    .await
            }
            SyncDirection::ExportOnly => {
                self.perform_export_sync(&account, &mut history).await
            }
        };

        // Update history with results
        history.completed_at = Some(Utc::now());
        history.status = match &result {
            Ok(_) if history.errors.is_empty() => SyncStatus::Success,
            Ok(_) => SyncStatus::PartialSuccess,
            Err(_) => SyncStatus::Failed,
        };

        self.save_sync_history(&history).await?;
        self.update_account_sync_status(account_id, history.status.clone())
            .await?;

        if let Err(e) = result {
            return Err(e);
        }

        Ok(history)
    }

    /// Perform two-way sync
    async fn perform_two_way_sync(
        &self,
        account: &ExternalAccount,
        full_sync: bool,
        history: &mut SyncHistory,
    ) -> Result<(), ExternalSyncError> {
        // First import from external
        self.perform_import_sync(account, full_sync, history).await?;

        // Then export to external
        self.perform_export_sync(account, history).await?;

        Ok(())
    }

    /// Import contacts from external provider
    async fn perform_import_sync(
        &self,
        account: &ExternalAccount,
        full_sync: bool,
        history: &mut SyncHistory,
    ) -> Result<(), ExternalSyncError> {
        let sync_cursor = if full_sync {
            None
        } else {
            account.sync_cursor.clone()
        };

        // Fetch contacts from provider
        let (external_contacts, new_cursor) = match account.provider {
            ExternalProvider::Google => {
                self.google_client
                    .list_contacts(&account.access_token, sync_cursor.as_deref())
                    .await?
            }
            ExternalProvider::Microsoft => {
                self.microsoft_client
                    .list_contacts(&account.access_token, sync_cursor.as_deref())
                    .await?
            }
            _ => return Err(ExternalSyncError::UnsupportedProvider(account.provider.to_string())),
        };

        // Process each contact
        for external_contact in external_contacts {
            match self
                .import_contact(account, &external_contact, history)
                .await
            {
                Ok(ImportResult::Created) => history.contacts_created += 1,
                Ok(ImportResult::Updated) => history.contacts_updated += 1,
                Ok(ImportResult::Skipped) => history.contacts_skipped += 1,
                Ok(ImportResult::Conflict) => history.conflicts_detected += 1,
                Err(e) => {
                    history.errors.push(SyncError {
                        contact_id: None,
                        external_id: Some(external_contact.id.clone()),
                        operation: "import".to_string(),
                        error_code: "import_failed".to_string(),
                        error_message: e.to_string(),
                        retryable: true,
                    });
                }
            }
        }

        // Update sync cursor
        if let Some(cursor) = new_cursor {
            self.update_account_sync_cursor(account.id, &cursor).await?;
        }

        Ok(())
    }

    /// Export contacts to external provider
    async fn perform_export_sync(
        &self,
        account: &ExternalAccount,
        history: &mut SyncHistory,
    ) -> Result<(), ExternalSyncError> {
        // Get pending uploads
        let pending_contacts = self.get_pending_uploads(account.id).await?;

        for mapping in pending_contacts {
            match self.export_contact(account, &mapping, history).await {
                Ok(ExportResult::Created) => history.contacts_created += 1,
                Ok(ExportResult::Updated) => history.contacts_updated += 1,
                Ok(ExportResult::Deleted) => history.contacts_deleted += 1,
                Ok(ExportResult::Skipped) => history.contacts_skipped += 1,
                Err(e) => {
                    history.errors.push(SyncError {
                        contact_id: Some(mapping.internal_contact_id),
                        external_id: Some(mapping.external_contact_id.clone()),
                        operation: "export".to_string(),
                        error_code: "export_failed".to_string(),
                        error_message: e.to_string(),
                        retryable: true,
                    });
                }
            }
        }

        Ok(())
    }

    /// Import a single contact
    async fn import_contact(
        &self,
        account: &ExternalAccount,
        external: &ExternalContact,
        _history: &mut SyncHistory,
    ) -> Result<ImportResult, ExternalSyncError> {
        // Check if mapping exists
        let existing_mapping = self
            .get_mapping_by_external_id(account.id, &external.id)
            .await?;

        if let Some(mapping) = existing_mapping {
            // Check for conflicts
            if mapping.external_etag.as_ref() != external.etag.as_ref() {
                // External changed
                let internal_changed = self
                    .has_internal_changes(mapping.internal_contact_id, mapping.internal_version)
                    .await?;

                if internal_changed {
                    // Conflict detected
                    self.mark_conflict(
                        &mapping,
                        vec!["external_updated".to_string()],
                        vec!["internal_updated".to_string()],
                    )
                    .await?;
                    return Ok(ImportResult::Conflict);
                }

                // Update internal contact
                self.update_internal_contact(mapping.internal_contact_id, external)
                    .await?;
                self.update_mapping_after_sync(&mapping, external.etag.as_deref())
                    .await?;
                return Ok(ImportResult::Updated);
            }

            // No changes
            return Ok(ImportResult::Skipped);
        }

        // Create new internal contact
        let contact_id = self
            .create_internal_contact(account.organization_id, account.user_id, external)
            .await?;

        // Create mapping
        self.create_mapping(account.id, contact_id, &external.id, external.etag.as_deref())
            .await?;

        Ok(ImportResult::Created)
    }

    /// Export a single contact
    async fn export_contact(
        &self,
        account: &ExternalAccount,
        mapping: &ContactMapping,
        _history: &mut SyncHistory,
    ) -> Result<ExportResult, ExternalSyncError> {
        // Get internal contact
        let internal = self.get_internal_contact(mapping.internal_contact_id).await?;

        // Convert to external format
        let external = self.convert_to_external(&internal);

        // Check if this is a new contact or update
        if mapping.external_contact_id.is_empty() {
            // Create new external contact
            let (external_id, etag) = match account.provider {
                ExternalProvider::Google => {
                    self.google_client
                        .create_contact(&account.access_token, &external)
                        .await?
                }
                ExternalProvider::Microsoft => {
                    self.microsoft_client
                        .create_contact(&account.access_token, &external)
                        .await?
                }
                _ => return Err(ExternalSyncError::UnsupportedProvider(account.provider.to_string())),
            };

            self.update_mapping_external_id(mapping.id, &external_id, etag.as_deref())
                .await?;
            return Ok(ExportResult::Created);
        }

        // Update existing external contact
        let etag = match account.provider {
            ExternalProvider::Google => {
                self.google_client
                    .update_contact(
                        &account.access_token,
                        &mapping.external_contact_id,
                        &external,
                    )
                    .await?
            }
            ExternalProvider::Microsoft => {
                self.microsoft_client
                    .update_contact(
                        &account.access_token,
                        &mapping.external_contact_id,
                        &external,
                    )
                    .await?
            }
            _ => return Err(ExternalSyncError::UnsupportedProvider(account.provider.to_string())),
        };

        self.update_mapping_after_sync(mapping, etag.as_deref()).await?;

        Ok(ExportResult::Updated)
    }

    /// Get list of connected accounts
    pub async fn list_accounts(
        &self,
        organization_id: Uuid,
        user_id: Option<Uuid>,
    ) -> Result<Vec<AccountStatusResponse>, ExternalSyncError> {
        let accounts = self.fetch_accounts(organization_id, user_id).await?;
        let mut results = Vec::new();

        for account in accounts {
            let sync_stats = self.get_sync_stats(account.id).await?;
            let pending_conflicts = self.count_pending_conflicts(account.id).await?;
            let pending_errors = self.count_pending_errors(account.id).await?;
            let next_sync = self.get_next_scheduled_sync(account.id).await?;

            results.push(AccountStatusResponse {
                account,
                sync_stats,
                pending_conflicts,
                pending_errors,
                next_scheduled_sync: next_sync,
            });
        }

        Ok(results)
    }

    /// Get sync history for an account
    pub async fn get_sync_history(
        &self,
        organization_id: Uuid,
        account_id: Uuid,
        limit: Option<u32>,
    ) -> Result<Vec<SyncHistory>, ExternalSyncError> {
        let account = self.get_account(account_id).await?;

        if account.organization_id != organization_id {
            return Err(ExternalSyncError::Unauthorized);
        }

        self.fetch_sync_history(account_id, limit.unwrap_or(20)).await
    }

    /// Get pending conflicts for an account
    pub async fn get_conflicts(
        &self,
        organization_id: Uuid,
        account_id: Uuid,
    ) -> Result<Vec<ContactMapping>, ExternalSyncError> {
        let account = self.get_account(account_id).await?;

        if account.organization_id != organization_id {
            return Err(ExternalSyncError::Unauthorized);
        }

        self.fetch_conflicts(account_id).await
    }

    /// Resolve a sync conflict
    pub async fn resolve_conflict(
        &self,
        organization_id: Uuid,
        mapping_id: Uuid,
        request: &ResolveConflictRequest,
    ) -> Result<ContactMapping, ExternalSyncError> {
        let mapping = self.get_mapping(mapping_id).await?;
        let account = self.get_account(mapping.account_id).await?;

        if account.organization_id != organization_id {
            return Err(ExternalSyncError::Unauthorized(
                "Access denied to this mapping".to_string(),
            ));
        }

        // Apply the resolution based on strategy
        let resolved_contact = match request.resolution {
            ConflictResolution::KeepLocal => mapping.local_data.clone(),
            ConflictResolution::KeepRemote => mapping.remote_data.clone(),
            ConflictResolution::Merge => {
                // Merge logic: prefer remote for non-null fields
                let mut merged = mapping.local_data.clone().unwrap_or_default();
                if let Some(remote) = &mapping.remote_data {
                    merged = remote.clone();
                }
                Some(merged)
            }
            ConflictResolution::Manual => request.manual_data.clone(),
        };

        // Update the mapping with resolved data
        let updated_mapping = ContactMapping {
            id: mapping.id,
            account_id: mapping.account_id,
            local_contact_id: mapping.local_contact_id,
            external_id: mapping.external_id,
            local_data: resolved_contact.clone(),
            remote_data: mapping.remote_data,
            sync_status: SyncStatus::Synced,
            last_synced_at: Some(Utc::now()),
            conflict_detected_at: None,
            created_at: mapping.created_at,
            updated_at: Utc::now(),
        };

        Ok(updated_mapping)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_status_display() {
        assert_eq!(format!("{:?}", SyncStatus::Pending), "Pending");
        assert_eq!(format!("{:?}", SyncStatus::Synced), "Synced");
        assert_eq!(format!("{:?}", SyncStatus::Conflict), "Conflict");
    }

    #[test]
    fn test_conflict_resolution_variants() {
        let _keep_local = ConflictResolution::KeepLocal;
        let _keep_remote = ConflictResolution::KeepRemote;
        let _merge = ConflictResolution::Merge;
        let _manual = ConflictResolution::Manual;
    }
}
