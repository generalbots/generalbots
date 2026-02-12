// External sync service with Google and Microsoft contacts integration
// Types and clients extracted to separate modules
use crate::contacts::sync_types::*;
use crate::contacts::google_client::GoogleClient;
use crate::contacts::microsoft_client::MicrosoftClient;

use chrono::{DateTime, Utc};
use log::{debug, error, warn};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::core::shared::state::AppState;

// External contact types - now in sync_types.rs
// Google/Microsoft clients - now in google_client.rs and microsoft_client.rs

#[derive(Debug, Clone)]
pub struct GoogleConfig {
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Debug, Clone)]
pub struct MicrosoftConfig {
    pub client_id: String,
    pub client_secret: String,
    pub tenant_id: String,
}

pub struct ExternalSyncService {
    google_client: GoogleClient,
    microsoft_client: MicrosoftClient,
    accounts: Arc<RwLock<HashMap<Uuid, ExternalAccount>>>,
    mappings: Arc<RwLock<HashMap<Uuid, ContactMapping>>>,
    sync_history: Arc<RwLock<Vec<SyncHistory>>>,
    contacts: Arc<RwLock<HashMap<Uuid, ExternalContact>>>,
}

impl ExternalSyncService {
    pub fn new(google_config: GoogleConfig, microsoft_config: MicrosoftConfig) -> Self {
        Self {
            google_client: GoogleClient::new(),
            microsoft_client: MicrosoftClient::new(),
            accounts: Arc::new(RwLock::new(HashMap::new())),
            mappings: Arc::new(RwLock::new(HashMap::new())),
            sync_history: Arc::new(RwLock::new(Vec::new())),
            contacts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    // Keep main sync methods - account management, sync operations, conflict resolution
    async fn find_existing_account(
        &self,
        organization_id: Uuid,
        provider: &ExternalProvider,
        external_id: &str,
    ) -> Result<Option<ExternalAccount>, ExternalSyncError> {
        let accounts = self.accounts.read().await;
        Ok(accounts.values().find(|a| {
            a.organization_id == organization_id
                && &a.provider == provider
                && a.external_account_id == external_id
        }).cloned())
    }

    async fn update_account_tokens(
        &self,
        account_id: Uuid,
        tokens: &TokenResponse,
    ) -> Result<ExternalAccount, ExternalSyncError> {
        let mut accounts = self.accounts.write().await;
        let account = accounts.get_mut(&account_id)
            .ok_or_else(|| ExternalSyncError::DatabaseError("Account not found".into()))?;
        account.access_token = tokens.access_token.clone();
        account.refresh_token = tokens.refresh_token.clone();
        account.token_expires_at = tokens.expires_at;
        account.updated_at = Utc::now();
        Ok(account.clone())
    }

    async fn save_account(&self, account: &ExternalAccount) -> Result<(), ExternalSyncError> {
        let mut accounts = self.accounts.write().await;
        accounts.insert(account.id, account.clone());
        Ok(())
    }

    async fn get_account(&self, account_id: Uuid) -> Result<ExternalAccount, ExternalSyncError> {
        let accounts = self.accounts.read().await;
        accounts.get(&account_id).cloned()
            .ok_or_else(|| ExternalSyncError::DatabaseError("Account not found".into()))
    }

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
                return Err(ExternalSyncError::UnsupportedProvider(request.provider.to_string()))
            }
        };

        // Get user info from provider
        let user_info = match request.provider {
            ExternalProvider::Google => {
                self.google_client.get_user_info(&tokens.access_token).await?
            }
            ExternalProvider::Microsoft => {
                self.microsoft_client.get_user_info(&tokens.access_token).await?
            }
            _ => return Err(ExternalSyncError::UnsupportedProvider(request.provider.to_string())),
        };

        // Check if account already exists
        if let Some(existing) = self
            .find_existing_account(organization_id, &request.provider, &user_info.id)
            .await?
        {
            // Update tokens
            return self.update_account_tokens(existing.id, &tokens).await;
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
                let _ = self.microsoft_client.revoke_token(&account.access_token).await;
            }
            _ => {}
        }

        // Delete account and mappings
        self.delete_account(account_id).await?;
        Ok(())
    }

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

        // Check if sync already in progress
        if let Some(last_status) = &account.last_sync_status {
            if last_status == "in_progress" {
                return Err(ExternalSyncError::SyncInProgress);
            }
        }

        let sync_id = Uuid::new_v4();
        let now = Utc::now();
        let direction = request.direction.clone().unwrap_or(account.sync_direction);

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
        history.status = if result.is_ok() {
            if history.errors.is_empty() {
                SyncStatus::Success
            } else {
                SyncStatus::PartialSuccess
            }
        } else {
            SyncStatus::Failed
        };

        self.save_sync_history(&history).await?;
        self.update_account_sync_status(account_id, history.status.clone())
            .await?;

        if let Err(e) = result {
            return Err(e);
        }

        Ok(history)
    }

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
                self.google_client.fetch_contacts(&account.access_token).await?
            }
            ExternalProvider::Microsoft => {
                self.microsoft_client.fetch_contacts(&account.access_token).await?
            }
            _ => return Err(ExternalSyncError::UnsupportedProvider(account.provider.to_string())),
        };

        // Update sync cursor
        self.update_account_sync_cursor(account.id, new_cursor).await?;

        // Process each contact
        for external_contact in external_contacts {
            match self.import_contact(account, &external_contact, history).await {
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

        Ok(())
    }

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
                        contact_id: Some(mapping.local_contact_id),
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

    async fn import_contact(
        &self,
        account: &ExternalAccount,
        external: &ExternalContact,
        history: &mut SyncHistory,
    ) -> Result<ImportResult, ExternalSyncError> {
        let existing_mapping = self
            .get_mapping_by_external_id(account.id, &external.id)
            .await?;

        if let Some(mapping) = existing_mapping {
            // Check for conflicts
            let internal_changed = self.has_internal_changes(&mapping).await?;
            if internal_changed {
                return Ok(ImportResult::Conflict);
            }

            self.update_mapping_after_sync(mapping.id, external.etag).await?;
            return Ok(ImportResult::Updated);
        }

        // Create new mapping and internal contact
        let contact_id = self.create_internal_contact(account.organization_id, external).await?;

        let now = Utc::now();
        let new_mapping = ContactMapping {
            id: Uuid::new_v4(),
            account_id: account.id,
            contact_id,
            local_contact_id: contact_id,
            external_id: external.id.clone(),
            external_contact_id: external.id.clone(),
            external_etag: external.etag.clone(),
            internal_version: 1,
            last_synced_at: now,
            sync_status: MappingSyncStatus::Synced,
            conflict_data: None,
            local_data: None,
            remote_data: None,
            conflict_detected_at: None,
            created_at: now,
            updated_at: now,
        };

        self.create_mapping(&new_mapping).await?;
        Ok(ImportResult::Created)
    }

    async fn export_contact(
        &self,
        account: &ExternalAccount,
        mapping: &ContactMapping,
        history: &mut SyncHistory,
    ) -> Result<ExportResult, ExternalSyncError> {
        let internal = self.get_internal_contact(mapping.local_contact_id).await?;
        let external = self.convert_to_external(&internal).await?;

        match account.provider {
            ExternalProvider::Google => {
                self.google_client
                    .update_contact(&account.access_token, &mapping.external_contact_id, &external)
                    .await?;
            }
            ExternalProvider::Microsoft => {
                self.microsoft_client
                    .update_contact(&account.access_token, &mapping.external_contact_id, &external)
                    .await?;
            }
            _ => return Err(ExternalSyncError::UnsupportedProvider(account.provider.to_string())),
        }

        self.update_mapping_after_sync(mapping.id, external.etag).await?;
        Ok(ExportResult::Updated)
    }

    async fn resolve_conflict(
        &self,
        organization_id: Uuid,
        mapping_id: Uuid,
        request: &ResolveConflictRequest,
    ) -> Result<ContactMapping, ExternalSyncError> {
        let mapping = self.get_mapping(mapping_id).await?;
        let account = self.get_account(mapping.account_id).await?;

        if account.organization_id != organization_id {
            return Err(ExternalSyncError::Unauthorized);
        }

        let resolved_contact = match request.resolution {
            ConflictResolution::KeepInternal => {
                mapping.local_data.clone()
            }
            ConflictResolution::KeepExternal => {
                mapping.remote_data.clone()
            }
            ConflictResolution::KeepLocal => {
                mapping.local_data.clone()
            }
            ConflictResolution::KeepRemote => {
                mapping.remote_data.clone()
            }
            ConflictResolution::Merge => {
                let mut merged = mapping.local_data.clone().unwrap_or_default();
                if let Some(remote) = &mapping.remote_data {
                    merged = remote.clone();
                }
                request.merged_data.as_ref().map(|m| {
                    merged.first_name = m.first_name.clone().or(merged.first_name);
                    merged.last_name = m.last_name.clone().or(merged.last_name);
                    merged.email = m.email.clone().or(merged.email);
                    merged.phone = m.phone.clone().or(merged.phone);
                    merged.company = m.company.clone().or(merged.company);
                    merged.notes = m.notes.clone().or(merged.notes);
                });
                Some(merged)
            }
            ConflictResolution::Manual => {
                request.manual_data.clone()
            }
            ConflictResolution::Skip => {
                return Ok(mapping.clone());
            }
        };

        let now = Utc::now();
        let updated_mapping = ContactMapping {
            id: mapping.id,
            account_id: mapping.account_id,
            contact_id: mapping.contact_id,
            local_contact_id: mapping.local_contact_id,
            external_id: mapping.external_id.clone(),
            external_contact_id: mapping.external_contact_id.clone(),
            external_etag: mapping.external_etag.clone(),
            internal_version: mapping.internal_version + 1,
            last_synced_at: now,
            sync_status: MappingSyncStatus::Synced,
            conflict_data: None,
            local_data: resolved_contact.clone(),
            remote_data: mapping.remote_data.clone(),
            conflict_detected_at: None,
            created_at: mapping.created_at,
            updated_at: now,
        };

        let mut mappings = self.mappings.write().await;
        mappings.insert(updated_mapping.id, updated_mapping.clone());
        Ok(updated_mapping)
    }

    // Helper methods
    async fn create_internal_contact(
        &self,
        _organization_id: Uuid,
        external: &ExternalContact,
    ) -> Result<Uuid, ExternalSyncError> {
        let contact_id = Uuid::new_v4();
        let mut contacts = self.contacts.write().await;
        let mut contact = external.clone();
        contact.id = contact_id.to_string();
        contacts.insert(contact_id, contact);
        Ok(contact_id)
    }

    async fn get_internal_contact(&self, contact_id: Uuid) -> Result<ExternalContact, ExternalSyncError> {
        let contacts = self.contacts.read().await;
        contacts.get(&contact_id).cloned()
            .ok_or_else(|| ExternalSyncError::DatabaseError("Contact not found".into()))
    }

    async fn convert_to_external(&self, contact: &ExternalContact) -> Result<ExternalContact, ExternalSyncError> {
        Ok(contact.clone())
    }

    async fn has_internal_changes(&self, _mapping: &ContactMapping) -> Result<bool, ExternalSyncError> {
        Ok(false)
    }

    async fn create_mapping(&self, mapping: &ContactMapping) -> Result<(), ExternalSyncError> {
        let mut mappings = self.mappings.write().await;
        mappings.insert(mapping.id, mapping.clone());
        Ok(())
    }

    async fn save_sync_history(&self, history: &SyncHistory) -> Result<(), ExternalSyncError> {
        let mut sync_history = self.sync_history.write().await;
        sync_history.push(history.clone());
        Ok(())
    }

    async fn update_account_sync_status(
        &self,
        account_id: Uuid,
        status: SyncStatus,
    ) -> Result<(), ExternalSyncError> {
        let mut accounts = self.accounts.write().await;
        if let Some(account) = accounts.get_mut(&account_id) {
            account.last_sync_status = Some(status.to_string());
            account.last_sync_at = Some(Utc::now());
        }
        Ok(())
    }

    async fn update_account_sync_cursor(
        &self,
        account_id: Uuid,
        cursor: Option<String>,
    ) -> Result<(), ExternalSyncError> {
        let mut accounts = self.accounts.write().await;
        if let Some(account) = accounts.get_mut(&account_id) {
            account.sync_cursor = cursor;
        }
        Ok(())
    }

    async fn get_pending_uploads(&self, account_id: Uuid) -> Result<Vec<ContactMapping>, ExternalSyncError> {
        let mappings = self.mappings.read().await;
        Ok(mappings.values()
            .filter(|m| m.account_id == account_id && m.sync_status == MappingSyncStatus::PendingUpload)
            .cloned()
            .collect())
    }

    async fn get_mapping_by_external_id(
        &self,
        account_id: Uuid,
        external_id: &str,
    ) -> Result<Option<ContactMapping>, ExternalSyncError> {
        let mappings = self.mappings.read().await;
        Ok(mappings.values()
            .find(|m| m.account_id == account_id && m.external_id == external_id)
            .cloned())
    }

    async fn get_mapping(&self, mapping_id: Uuid) -> Result<ContactMapping, ExternalSyncError> {
        let mappings = self.mappings.read().await;
        mappings.get(&mapping_id).cloned()
            .ok_or_else(|| ExternalSyncError::DatabaseError("Mapping not found".into()))
    }
}

// Error type - now uses types from sync_types
#[derive(Debug, Clone)]
pub enum ExternalSyncError {
    DatabaseError(String),
    UnsupportedProvider(String),
    Unauthorized,
    SyncDisabled,
    SyncInProgress,
    ApiError(String),
    InvalidData(String),
    NetworkError(String),
    AuthError(String),
    ParseError(String),
}

impl std::fmt::Display for ExternalSyncError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DatabaseError(e) => write!(f, "Database error: {e}"),
            Self::UnsupportedProvider(p) => write!(f, "Unsupported provider: {p}"),
            Self::Unauthorized => write!(f, "Unauthorized"),
            Self::SyncDisabled => write!(f, "Sync is disabled"),
            Self::SyncInProgress => write!(f, "Sync already in progress"),
            Self::ApiError(e) => write!(f, "API error: {e}"),
            Self::InvalidData(e) => write!(f, "Invalid data: {e}"),
            Self::NetworkError(e) => write!(f, "Network error: {e}"),
            Self::AuthError(e) => write!(f, "Auth error: {e}"),
            Self::ParseError(e) => write!(f, "Parse error: {e}"),
        }
    }
}

impl std::error::Error for ExternalSyncError {}

impl axum::response::IntoResponse for ExternalSyncError {
    fn into_response(self) -> axum::response::Response {
        let status = match self {
            Self::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::UnsupportedProvider(_) => StatusCode::BAD_REQUEST,
            Self::SyncDisabled => StatusCode::FORBIDDEN,
            Self::SyncInProgress => StatusCode::CONFLICT,
            Self::InvalidData(_) => StatusCode::BAD_REQUEST,
            Self::ApiError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::NetworkError(_) => StatusCode::SERVICE_UNAVAILABLE,
            Self::AuthError(_) => StatusCode::UNAUTHORIZED,
            Self::ParseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, self.to_string()).into_response()
    }
}

// External contact and related types
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserEmail {
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPhone {
    pub phone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserName {
    pub name: String,
}
