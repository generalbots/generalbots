use crate::sync_types::*;
use crate::google_client::{GoogleClient, GoogleConfig, GoogleContactsClient, TokenResponse as GoogleTokenResponse};
use crate::microsoft_client::{MicrosoftClient, MsTokenResponse};

use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

pub struct MicrosoftConfig {
    pub client_id: String,
    pub client_secret: String,
    pub tenant_id: String,
}

pub struct ExternalSyncService {
    google_client: GoogleClient,
    microsoft_client: Arc<MicrosoftClient>,
    accounts: Arc<RwLock<HashMap<Uuid, ExternalAccount>>>,
    mappings: Arc<RwLock<HashMap<Uuid, ContactMapping>>>,
    sync_history: Arc<RwLock<Vec<SyncHistory>>>,
    contacts: Arc<RwLock<HashMap<Uuid, ExternalContact>>>,
    google_config: Option<GoogleConfig>,
    microsoft_config: Option<MicrosoftConfig>,
}

fn convert_google_token(t: GoogleTokenResponse) -> TokenResponse {
    TokenResponse { access_token: t.access_token, refresh_token: t.refresh_token, expires_in: t.expires_in, expires_at: t.expires_at, scopes: t.scopes }
}

fn convert_ms_token(t: MsTokenResponse) -> TokenResponse {
    TokenResponse {
        access_token: t.access_token, refresh_token: t.refresh_token, expires_in: t.expires_in,
        expires_at: Some(Utc::now() + chrono::Duration::seconds(t.expires_in)),
        scopes: t.scope.map(|s| s.split(' ').map(String::from).collect()).unwrap_or_default(),
    }
}

impl ExternalSyncService {
    pub fn new(google_config: Option<GoogleConfig>, microsoft_config: Option<MicrosoftConfig>) -> Self {
        Self {
            google_client: GoogleClient::new(),
            microsoft_client: Arc::new(MicrosoftClient::new()),
            accounts: Arc::new(RwLock::new(HashMap::new())),
            mappings: Arc::new(RwLock::new(HashMap::new())),
            sync_history: Arc::new(RwLock::new(Vec::new())),
            contacts: Arc::new(RwLock::new(HashMap::new())),
            google_config,
            microsoft_config,
        }
    }

    pub async fn connect_account(&self, organization_id: Uuid, user_id: Uuid, request: &ConnectAccountRequest) -> Result<ExternalAccount, ExternalSyncError> {
        let tokens = match request.provider {
            ExternalProvider::Google => {
                let gc = self.google_contacts_client().ok_or_else(|| ExternalSyncError { kind: ExternalSyncErrorKind::UnsupportedProvider, message: "Google not configured".into() })?;
                convert_google_token(gc.exchange_code(&request.authorization_code, &request.redirect_uri).await?)
            }
            ExternalProvider::Microsoft => {
                let mc = self.microsoft_contacts_client().ok_or_else(|| ExternalSyncError { kind: ExternalSyncErrorKind::UnsupportedProvider, message: "Microsoft not configured".into() })?;
                let cfg = self.microsoft_config.as_ref().ok_or_else(|| ExternalSyncError { kind: ExternalSyncErrorKind::UnsupportedProvider, message: "Microsoft not configured".into() })?;
                convert_ms_token(mc.exchange_code(&request.authorization_code, &request.redirect_uri, &cfg.client_id, &cfg.client_secret, &cfg.tenant_id).await?)
            }
            _ => return Err(ExternalSyncError { kind: ExternalSyncErrorKind::UnsupportedProvider, message: request.provider.to_string() }),
        };

        let user_info = match request.provider {
            ExternalProvider::Google => {
                let gc = self.google_contacts_client().ok_or_else(|| ExternalSyncError { kind: ExternalSyncErrorKind::UnsupportedProvider, message: "Google not configured".into() })?;
                let ui = gc.get_user_info(&tokens.access_token).await?;
                UserInfo { id: ui.id, email: ui.email, name: ui.name }
            }
            ExternalProvider::Microsoft => {
                let mc = self.microsoft_contacts_client().ok_or_else(|| ExternalSyncError { kind: ExternalSyncErrorKind::UnsupportedProvider, message: "Microsoft not configured".into() })?;
                let ui = mc.get_user_info(&tokens.access_token).await?;
                UserInfo { id: ui.id, email: ui.email.unwrap_or_default(), name: ui.display_name }
            }
            _ => return Err(ExternalSyncError { kind: ExternalSyncErrorKind::UnsupportedProvider, message: request.provider.to_string() }),
        };

        if let Some(existing) = self.find_existing_account(organization_id, &request.provider, &user_info.id).await? {
            let mut accounts = self.accounts.write().await;
            if let Some(account) = accounts.get_mut(&existing.id) {
                account.access_token = tokens.access_token;
                account.refresh_token = tokens.refresh_token;
                account.token_expires_at = tokens.expires_at;
                account.updated_at = Utc::now();
                return Ok(account.clone());
            }
        }

        let account_id = Uuid::new_v4();
        let now = Utc::now();
        let account = ExternalAccount {
            id: account_id, organization_id, user_id, provider: request.provider.clone(),
            external_account_id: user_info.id, email: user_info.email, display_name: user_info.name,
            access_token: tokens.access_token, refresh_token: tokens.refresh_token,
            token_expires_at: tokens.expires_at, scopes: tokens.scopes,
            sync_enabled: true, sync_direction: request.sync_direction.clone().unwrap_or_default(),
            last_sync_at: None, last_sync_status: None, sync_cursor: None,
            created_at: now, updated_at: now,
        };

        let mut accounts = self.accounts.write().await;
        accounts.insert(account.id, account.clone());
        Ok(account)
    }

    pub async fn disconnect_account(&self, organization_id: Uuid, account_id: Uuid) -> Result<(), ExternalSyncError> {
        let account = self.get_account(account_id).await?;
        if account.organization_id != organization_id {
            return Err(ExternalSyncError { kind: ExternalSyncErrorKind::Unauthorized, message: String::new() });
        }

        let access_token = account.access_token.clone();
        let provider = account.provider.clone();
        match provider {
            ExternalProvider::Google => {
                if let Some(gc) = self.google_contacts_client() {
                    let _ = gc.revoke_token(&access_token).await;
                }
            }
            ExternalProvider::Microsoft => { let _ = self.microsoft_client.revoke_token(&access_token).await; }
            _ => {}
        }

        let mut accounts = self.accounts.write().await;
        accounts.remove(&account_id);
        Ok(())
    }

    pub async fn start_sync(&self, organization_id: Uuid, account_id: Uuid, request: &StartSyncRequest, trigger: SyncTrigger) -> Result<SyncHistory, ExternalSyncError> {
        let account = self.get_account(account_id).await?;
        if account.organization_id != organization_id {
            return Err(ExternalSyncError { kind: ExternalSyncErrorKind::Unauthorized, message: String::new() });
        }
        if !account.sync_enabled {
            return Err(ExternalSyncError { kind: ExternalSyncErrorKind::SyncDisabled, message: String::new() });
        }
        if let Some(last_status) = &account.last_sync_status {
            if last_status == "in_progress" {
                return Err(ExternalSyncError { kind: ExternalSyncErrorKind::SyncInProgress, message: String::new() });
            }
        }

        let sync_id = Uuid::new_v4();
        let now = Utc::now();
        let direction = request.direction.clone().unwrap_or_else(|| account.sync_direction.clone());

        let mut history = SyncHistory {
            id: sync_id, account_id: account.id, started_at: now, completed_at: None,
            status: SyncStatus::InProgress, direction: direction.clone(),
            contacts_created: 0, contacts_updated: 0, contacts_deleted: 0,
            contacts_skipped: 0, conflicts_detected: 0, errors: vec![], triggered_by: trigger,
        };

        let result = match direction {
            SyncDirection::TwoWay | SyncDirection::ImportOnly => self.perform_import_sync(&account, request.full_sync.unwrap_or(false), &mut history).await,
            SyncDirection::ExportOnly => self.perform_export_sync(&account, &mut history).await,
        };

        history.completed_at = Some(Utc::now());
        history.status = if result.is_ok() {
            if history.errors.is_empty() { SyncStatus::Success } else { SyncStatus::PartialSuccess }
        } else { SyncStatus::Failed };

        let mut sync_history = self.sync_history.write().await;
        sync_history.push(history.clone());

        let mut accounts = self.accounts.write().await;
        if let Some(acc) = accounts.get_mut(&account_id) {
            acc.last_sync_status = Some(history.status.to_string());
            acc.last_sync_at = Some(Utc::now());
        }

        result?;
        Ok(history)
    }

    pub async fn resolve_conflict(&self, organization_id: Uuid, mapping_id: Uuid, request: &ResolveConflictRequest) -> Result<ContactMapping, ExternalSyncError> {
        let mappings = self.mappings.read().await;
        let mapping = mappings.get(&mapping_id).cloned()
            .ok_or_else(|| ExternalSyncError { kind: ExternalSyncErrorKind::DatabaseError, message: "Mapping not found".into() })?;
        drop(mappings);

        let accounts = self.accounts.read().await;
        let account = accounts.get(&mapping.account_id).cloned()
            .ok_or_else(|| ExternalSyncError { kind: ExternalSyncErrorKind::DatabaseError, message: "Account not found".into() })?;
        drop(accounts);

        if account.organization_id != organization_id {
            return Err(ExternalSyncError { kind: ExternalSyncErrorKind::Unauthorized, message: String::new() });
        }

        let resolved_contact = match request.resolution {
            ConflictResolution::KeepInternal | ConflictResolution::KeepLocal => mapping.local_data.clone(),
            ConflictResolution::KeepExternal | ConflictResolution::KeepRemote => mapping.remote_data.clone(),
            ConflictResolution::Merge => {
                let mut merged = mapping.local_data.clone().unwrap_or_default();
                if let Some(remote) = &mapping.remote_data { merged = remote.clone(); }
                if let Some(m) = &request.merged_data {
                    merged.first_name = m.first_name.clone().or(merged.first_name.take());
                    merged.last_name = m.last_name.clone().or(merged.last_name.take());
                    merged.email = m.email.clone().or(merged.email.take());
                    merged.phone = m.phone.clone().or(merged.phone.take());
                    merged.company = m.company.clone().or(merged.company.take());
                    merged.notes = m.notes.clone().or(merged.notes.take());
                }
                Some(merged)
            }
            ConflictResolution::Manual => request.manual_data.as_ref().and_then(|v| serde_json::from_value(v.clone()).ok()),
            ConflictResolution::Skip => return Ok(mapping),
        };

        let now = Utc::now();
        let updated_mapping = ContactMapping {
            id: mapping.id, account_id: mapping.account_id, contact_id: mapping.contact_id,
            local_contact_id: mapping.local_contact_id, external_id: mapping.external_id.clone(),
            external_contact_id: mapping.external_contact_id.clone(), external_etag: mapping.external_etag.clone(),
            internal_version: mapping.internal_version + 1, last_synced_at: now,
            sync_status: MappingSyncStatus::Synced, conflict_data: None,
            local_data: resolved_contact, remote_data: mapping.remote_data.clone(),
            conflict_detected_at: None, created_at: mapping.created_at, updated_at: now,
        };

        let mut mappings = self.mappings.write().await;
        mappings.insert(updated_mapping.id, updated_mapping.clone());
        Ok(updated_mapping)
    }

    pub async fn get_account(&self, account_id: Uuid) -> Result<ExternalAccount, ExternalSyncError> {
        let accounts = self.accounts.read().await;
        accounts.get(&account_id).cloned()
            .ok_or_else(|| ExternalSyncError { kind: ExternalSyncErrorKind::DatabaseError, message: "Account not found".into() })
    }

    pub async fn list_accounts(&self, organization_id: Uuid) -> Result<Vec<ExternalAccount>, ExternalSyncError> {
        let accounts = self.accounts.read().await;
        Ok(accounts.values().filter(|a| a.organization_id == organization_id).cloned().collect())
    }

    pub async fn get_sync_history(&self, account_id: Uuid) -> Result<Vec<SyncHistory>, ExternalSyncError> {
        let sync_history = self.sync_history.read().await;
        Ok(sync_history.iter().filter(|h| h.account_id == account_id).cloned().collect())
    }

    pub async fn get_conflicts(&self, account_id: Uuid) -> Result<Vec<ContactMapping>, ExternalSyncError> {
        let mappings = self.mappings.read().await;
        Ok(mappings.values().filter(|m| m.account_id == account_id && m.sync_status == MappingSyncStatus::Conflict).cloned().collect())
    }

    async fn find_existing_account(&self, organization_id: Uuid, provider: &ExternalProvider, external_id: &str) -> Result<Option<ExternalAccount>, ExternalSyncError> {
        let accounts = self.accounts.read().await;
        Ok(accounts.values().find(|a| a.organization_id == organization_id && &a.provider == provider && a.external_account_id == external_id).cloned())
    }

    fn google_contacts_client(&self) -> Option<GoogleContactsClient> {
        self.google_config.as_ref().map(|c| GoogleContactsClient::new(c.clone()))
    }

    fn microsoft_contacts_client(&self) -> Option<Arc<MicrosoftClient>> {
        Some(self.microsoft_client.clone())
    }

    async fn perform_import_sync(&self, account: &ExternalAccount, _full_sync: bool, history: &mut SyncHistory) -> Result<(), ExternalSyncError> {
        let (external_contacts, new_cursor) = match account.provider {
            ExternalProvider::Google => self.google_client.fetch_contacts(&account.access_token).await?,
            ExternalProvider::Microsoft => {
                let (contacts, _) = self.microsoft_client.fetch_contacts(&account.access_token).await?;
                (contacts, None)
            }
            _ => return Err(ExternalSyncError { kind: ExternalSyncErrorKind::UnsupportedProvider, message: account.provider.to_string() }),
        };

        if let Some(cursor) = new_cursor {
            let mut accounts = self.accounts.write().await;
            if let Some(acc) = accounts.get_mut(&account.id) { acc.sync_cursor = Some(cursor); }
        }

        for external_contact in external_contacts {
            match self.import_contact(account, &external_contact).await {
                Ok(ImportResult::Created) => history.contacts_created += 1,
                Ok(ImportResult::Updated) => history.contacts_updated += 1,
                Ok(ImportResult::Skipped) => history.contacts_skipped += 1,
                Ok(ImportResult::Conflict) => history.conflicts_detected += 1,
                Err(e) => {
                    history.errors.push(SyncError {
                        contact_id: None, external_id: Some(external_contact.id.clone()),
                        operation: "import".to_string(), error_code: "import_failed".to_string(),
                        error_message: e.to_string(), retryable: true,
                    });
                }
            }
        }
        Ok(())
    }

    async fn perform_export_sync(&self, account: &ExternalAccount, history: &mut SyncHistory) -> Result<(), ExternalSyncError> {
        let pending_contacts = self.get_pending_uploads(account.id).await?;
        for mapping in pending_contacts {
            match self.export_contact(account, &mapping).await {
                Ok(ExportResult::Created) => history.contacts_created += 1,
                Ok(ExportResult::Updated) => history.contacts_updated += 1,
                Ok(ExportResult::Deleted) => history.contacts_deleted += 1,
                Ok(ExportResult::Skipped) => history.contacts_skipped += 1,
                Err(e) => {
                    history.errors.push(SyncError {
                        contact_id: Some(mapping.local_contact_id), external_id: Some(mapping.external_contact_id.clone()),
                        operation: "export".to_string(), error_code: "export_failed".to_string(),
                        error_message: e.to_string(), retryable: true,
                    });
                }
            }
        }
        Ok(())
    }

    async fn import_contact(&self, account: &ExternalAccount, external: &ExternalContact) -> Result<ImportResult, ExternalSyncError> {
        let existing_mapping = self.get_mapping_by_external_id(account.id, &external.id).await?;
        if let Some(mapping) = existing_mapping {
            self.update_mapping_after_sync(mapping.id, external.etag.clone()).await?;
            return Ok(ImportResult::Updated);
        }

        let contact_id = self.create_internal_contact(account.organization_id, external).await?;
        let now = Utc::now();
        let new_mapping = ContactMapping {
            id: Uuid::new_v4(), account_id: account.id, contact_id, local_contact_id: contact_id,
            external_id: external.id.clone(), external_contact_id: external.id.clone(),
            external_etag: external.etag.clone(), internal_version: 1, last_synced_at: now,
            sync_status: MappingSyncStatus::Synced, conflict_data: None, local_data: None, remote_data: None,
            conflict_detected_at: None, created_at: now, updated_at: now,
        };
        let mut mappings = self.mappings.write().await;
        mappings.insert(new_mapping.id, new_mapping);
        Ok(ImportResult::Created)
    }

    async fn export_contact(&self, account: &ExternalAccount, mapping: &ContactMapping) -> Result<ExportResult, ExternalSyncError> {
        let internal = self.get_internal_contact(mapping.local_contact_id).await?;
        match account.provider {
            ExternalProvider::Google => {
                self.google_client.update_contact(&account.access_token, &mapping.external_contact_id, &internal).await?;
            }
            ExternalProvider::Microsoft => {
                self.microsoft_client.update_contact(&account.access_token, &mapping.external_contact_id, &internal).await?;
            }
            _ => return Err(ExternalSyncError { kind: ExternalSyncErrorKind::UnsupportedProvider, message: account.provider.to_string() }),
        }
        self.update_mapping_after_sync(mapping.id, internal.etag.clone()).await?;
        Ok(ExportResult::Updated)
    }

    async fn create_internal_contact(&self, _organization_id: Uuid, external: &ExternalContact) -> Result<Uuid, ExternalSyncError> {
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
            .ok_or_else(|| ExternalSyncError { kind: ExternalSyncErrorKind::DatabaseError, message: "Contact not found".into() })
    }

    async fn get_mapping_by_external_id(&self, account_id: Uuid, external_id: &str) -> Result<Option<ContactMapping>, ExternalSyncError> {
        let mappings = self.mappings.read().await;
        Ok(mappings.values().find(|m| m.account_id == account_id && m.external_id == external_id).cloned())
    }

    async fn get_pending_uploads(&self, account_id: Uuid) -> Result<Vec<ContactMapping>, ExternalSyncError> {
        let mappings = self.mappings.read().await;
        Ok(mappings.values().filter(|m| m.account_id == account_id && m.sync_status == MappingSyncStatus::PendingUpload).cloned().collect())
    }

    async fn update_mapping_after_sync(&self, mapping_id: Uuid, etag: Option<String>) -> Result<(), ExternalSyncError> {
        let mut mappings = self.mappings.write().await;
        if let Some(mapping) = mappings.get_mut(&mapping_id) {
            mapping.sync_status = MappingSyncStatus::Synced;
            mapping.last_synced_at = Utc::now();
            mapping.external_etag = etag;
            mapping.updated_at = Utc::now();
        }
        Ok(())
    }
}
