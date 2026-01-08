use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub organization_id: Uuid,
    pub actor_id: Uuid,
    pub actor_email: Option<String>,
    pub actor_ip: Option<String>,
    pub action: AuditAction,
    pub resource_type: ResourceType,
    pub resource_id: Option<Uuid>,
    pub resource_name: Option<String>,
    pub details: AuditDetails,
    pub result: AuditResult,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuditAction {
    Create,
    Read,
    Update,
    Delete,
    Login,
    Logout,
    PasswordChange,
    PasswordReset,
    RoleAssign,
    RoleRevoke,
    GroupAdd,
    GroupRemove,
    PermissionGrant,
    PermissionRevoke,
    AccessAttempt,
    AccessDenied,
    Export,
    Import,
    Invite,
    InviteAccept,
    InviteRevoke,
    SettingsChange,
    BillingChange,
    ApiKeyCreate,
    ApiKeyRevoke,
    MfaEnable,
    MfaDisable,
    SessionTerminate,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ResourceType {
    User,
    Organization,
    Role,
    Group,
    Permission,
    Bot,
    KnowledgeBase,
    Document,
    App,
    Form,
    Site,
    ApiKey,
    Session,
    Subscription,
    Invoice,
    Settings,
    Channel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditDetails {
    pub description: String,
    pub before_state: Option<serde_json::Value>,
    pub after_state: Option<serde_json::Value>,
    pub changes: Option<Vec<FieldChange>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldChange {
    pub field: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuditResult {
    Success,
    Failure,
    Denied,
    Error,
}

impl AuditLogEntry {
    pub fn new(
        organization_id: Uuid,
        actor_id: Uuid,
        action: AuditAction,
        resource_type: ResourceType,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            organization_id,
            actor_id,
            actor_email: None,
            actor_ip: None,
            action,
            resource_type,
            resource_id: None,
            resource_name: None,
            details: AuditDetails {
                description: String::new(),
                before_state: None,
                after_state: None,
                changes: None,
            },
            result: AuditResult::Success,
            metadata: None,
        }
    }

    pub fn with_actor_email(mut self, email: impl Into<String>) -> Self {
        self.actor_email = Some(email.into());
        self
    }

    pub fn with_actor_ip(mut self, ip: impl Into<String>) -> Self {
        self.actor_ip = Some(ip.into());
        self
    }

    pub fn with_resource(mut self, id: Uuid, name: impl Into<String>) -> Self {
        self.resource_id = Some(id);
        self.resource_name = Some(name.into());
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.details.description = description.into();
        self
    }

    pub fn with_before_state(mut self, state: serde_json::Value) -> Self {
        self.details.before_state = Some(state);
        self
    }

    pub fn with_after_state(mut self, state: serde_json::Value) -> Self {
        self.details.after_state = Some(state);
        self
    }

    pub fn with_changes(mut self, changes: Vec<FieldChange>) -> Self {
        self.details.changes = Some(changes);
        self
    }

    pub fn with_result(mut self, result: AuditResult) -> Self {
        self.result = result;
        self
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    pub fn success(mut self) -> Self {
        self.result = AuditResult::Success;
        self
    }

    pub fn failure(mut self) -> Self {
        self.result = AuditResult::Failure;
        self
    }

    pub fn denied(mut self) -> Self {
        self.result = AuditResult::Denied;
        self
    }
}

pub struct AuditLogger {
    entries: Arc<RwLock<VecDeque<AuditLogEntry>>>,
    max_entries: usize,
    retention_days: u32,
}

impl AuditLogger {
    pub fn new(max_entries: usize, retention_days: u32) -> Self {
        Self {
            entries: Arc::new(RwLock::new(VecDeque::with_capacity(max_entries))),
            max_entries,
            retention_days,
        }
    }

    pub async fn log(&self, entry: AuditLogEntry) {
        let mut entries = self.entries.write().await;

        if entries.len() >= self.max_entries {
            entries.pop_front();
        }

        log::info!(
            "AUDIT: action={:?} resource={:?} actor={} result={:?} - {}",
            entry.action,
            entry.resource_type,
            entry.actor_id,
            entry.result,
            entry.details.description
        );

        entries.push_back(entry);
    }

    pub async fn log_role_assignment(
        &self,
        organization_id: Uuid,
        actor_id: Uuid,
        target_user_id: Uuid,
        role_name: &str,
        actor_email: Option<&str>,
    ) {
        let entry = AuditLogEntry::new(
            organization_id,
            actor_id,
            AuditAction::RoleAssign,
            ResourceType::Role,
        )
        .with_resource(target_user_id, role_name)
        .with_description(format!("Assigned role '{role_name}' to user {target_user_id}"));

        let entry = match actor_email {
            Some(email) => entry.with_actor_email(email),
            None => entry,
        };

        self.log(entry).await;
    }

    pub async fn log_role_revocation(
        &self,
        organization_id: Uuid,
        actor_id: Uuid,
        target_user_id: Uuid,
        role_name: &str,
        actor_email: Option<&str>,
    ) {
        let entry = AuditLogEntry::new(
            organization_id,
            actor_id,
            AuditAction::RoleRevoke,
            ResourceType::Role,
        )
        .with_resource(target_user_id, role_name)
        .with_description(format!("Revoked role '{role_name}' from user {target_user_id}"));

        let entry = match actor_email {
            Some(email) => entry.with_actor_email(email),
            None => entry,
        };

        self.log(entry).await;
    }

    pub async fn log_group_membership(
        &self,
        organization_id: Uuid,
        actor_id: Uuid,
        target_user_id: Uuid,
        group_name: &str,
        is_addition: bool,
        actor_email: Option<&str>,
    ) {
        let action = if is_addition {
            AuditAction::GroupAdd
        } else {
            AuditAction::GroupRemove
        };

        let description = if is_addition {
            format!("Added user {target_user_id} to group '{group_name}'")
        } else {
            format!("Removed user {target_user_id} from group '{group_name}'")
        };

        let entry = AuditLogEntry::new(organization_id, actor_id, action, ResourceType::Group)
            .with_resource(target_user_id, group_name)
            .with_description(description);

        let entry = match actor_email {
            Some(email) => entry.with_actor_email(email),
            None => entry,
        };

        self.log(entry).await;
    }

    pub async fn log_permission_change(
        &self,
        organization_id: Uuid,
        actor_id: Uuid,
        target_id: Uuid,
        permission: &str,
        is_grant: bool,
        actor_email: Option<&str>,
    ) {
        let action = if is_grant {
            AuditAction::PermissionGrant
        } else {
            AuditAction::PermissionRevoke
        };

        let description = if is_grant {
            format!("Granted permission '{permission}' to {target_id}")
        } else {
            format!("Revoked permission '{permission}' from {target_id}")
        };

        let entry =
            AuditLogEntry::new(organization_id, actor_id, action, ResourceType::Permission)
                .with_resource(target_id, permission)
                .with_description(description);

        let entry = match actor_email {
            Some(email) => entry.with_actor_email(email),
            None => entry,
        };

        self.log(entry).await;
    }

    pub async fn log_access_attempt(
        &self,
        organization_id: Uuid,
        actor_id: Uuid,
        resource_type: ResourceType,
        resource_id: Uuid,
        resource_name: &str,
        permission_required: &str,
        allowed: bool,
        actor_ip: Option<&str>,
    ) {
        let action = if allowed {
            AuditAction::AccessAttempt
        } else {
            AuditAction::AccessDenied
        };

        let result = if allowed {
            AuditResult::Success
        } else {
            AuditResult::Denied
        };

        let description = if allowed {
            format!(
                "Access granted to {resource_name} (required: {permission_required})"
            )
        } else {
            format!(
                "Access denied to {resource_name} (required: {permission_required})"
            )
        };

        let entry = AuditLogEntry::new(organization_id, actor_id, action, resource_type)
            .with_resource(resource_id, resource_name)
            .with_description(description)
            .with_result(result);

        let entry = match actor_ip {
            Some(ip) => entry.with_actor_ip(ip),
            None => entry,
        };

        self.log(entry).await;
    }

    pub async fn log_settings_change(
        &self,
        organization_id: Uuid,
        actor_id: Uuid,
        setting_name: &str,
        old_value: Option<&str>,
        new_value: Option<&str>,
        actor_email: Option<&str>,
    ) {
        let changes = vec![FieldChange {
            field: setting_name.to_string(),
            old_value: old_value.map(String::from),
            new_value: new_value.map(String::from),
        }];

        let entry = AuditLogEntry::new(
            organization_id,
            actor_id,
            AuditAction::SettingsChange,
            ResourceType::Settings,
        )
        .with_description(format!("Changed setting '{setting_name}'"))
        .with_changes(changes);

        let entry = match actor_email {
            Some(email) => entry.with_actor_email(email),
            None => entry,
        };

        self.log(entry).await;
    }

    pub async fn query(
        &self,
        filter: AuditLogFilter,
    ) -> Vec<AuditLogEntry> {
        let entries = self.entries.read().await;
        let cutoff = Utc::now() - chrono::Duration::days(i64::from(self.retention_days));

        entries
            .iter()
            .filter(|e| e.timestamp > cutoff)
            .filter(|e| filter.matches(e))
            .cloned()
            .collect()
    }

    pub async fn query_paginated(
        &self,
        filter: AuditLogFilter,
        page: usize,
        per_page: usize,
    ) -> AuditLogPage {
        let all_entries = self.query(filter).await;
        let total = all_entries.len();
        let total_pages = (total + per_page - 1) / per_page;

        let start = page.saturating_sub(1) * per_page;
        let entries: Vec<_> = all_entries.into_iter().skip(start).take(per_page).collect();

        AuditLogPage {
            entries,
            page,
            per_page,
            total,
            total_pages,
        }
    }

    pub async fn export_for_compliance(
        &self,
        organization_id: Uuid,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> ComplianceExport {
        let entries = self.entries.read().await;

        let filtered: Vec<_> = entries
            .iter()
            .filter(|e| e.organization_id == organization_id)
            .filter(|e| e.timestamp >= start_date && e.timestamp <= end_date)
            .cloned()
            .collect();

        let mut action_counts: std::collections::HashMap<String, u64> =
            std::collections::HashMap::new();

        for entry in &filtered {
            let key = format!("{:?}", entry.action);
            *action_counts.entry(key).or_insert(0) += 1;
        }

        let denied_count = filtered
            .iter()
            .filter(|e| e.result == AuditResult::Denied)
            .count();

        let unique_actors: std::collections::HashSet<_> =
            filtered.iter().map(|e| e.actor_id).collect();

        ComplianceExport {
            organization_id,
            start_date,
            end_date,
            total_events: filtered.len(),
            action_summary: action_counts,
            access_denied_count: denied_count,
            unique_actors: unique_actors.len(),
            entries: filtered,
            generated_at: Utc::now(),
        }
    }

    pub async fn cleanup_old_entries(&self) {
        let mut entries = self.entries.write().await;
        let cutoff = Utc::now() - chrono::Duration::days(i64::from(self.retention_days));

        entries.retain(|e| e.timestamp > cutoff);
    }

    pub fn retention_days(&self) -> u32 {
        self.retention_days
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new(100_000, 90)
    }
}

#[derive(Debug, Clone, Default)]
pub struct AuditLogFilter {
    pub organization_id: Option<Uuid>,
    pub actor_id: Option<Uuid>,
    pub actions: Option<Vec<AuditAction>>,
    pub resource_types: Option<Vec<ResourceType>>,
    pub resource_id: Option<Uuid>,
    pub results: Option<Vec<AuditResult>>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub search_term: Option<String>,
}

impl AuditLogFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn organization(mut self, org_id: Uuid) -> Self {
        self.organization_id = Some(org_id);
        self
    }

    pub fn actor(mut self, actor_id: Uuid) -> Self {
        self.actor_id = Some(actor_id);
        self
    }

    pub fn actions(mut self, actions: Vec<AuditAction>) -> Self {
        self.actions = Some(actions);
        self
    }

    pub fn resource_types(mut self, types: Vec<ResourceType>) -> Self {
        self.resource_types = Some(types);
        self
    }

    pub fn resource(mut self, resource_id: Uuid) -> Self {
        self.resource_id = Some(resource_id);
        self
    }

    pub fn results(mut self, results: Vec<AuditResult>) -> Self {
        self.results = Some(results);
        self
    }

    pub fn date_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.start_date = Some(start);
        self.end_date = Some(end);
        self
    }

    pub fn search(mut self, term: impl Into<String>) -> Self {
        self.search_term = Some(term.into());
        self
    }

    fn matches(&self, entry: &AuditLogEntry) -> bool {
        if let Some(org_id) = self.organization_id {
            if entry.organization_id != org_id {
                return false;
            }
        }

        if let Some(actor_id) = self.actor_id {
            if entry.actor_id != actor_id {
                return false;
            }
        }

        if let Some(ref actions) = self.actions {
            if !actions.contains(&entry.action) {
                return false;
            }
        }

        if let Some(ref types) = self.resource_types {
            if !types.contains(&entry.resource_type) {
                return false;
            }
        }

        if let Some(resource_id) = self.resource_id {
            if entry.resource_id != Some(resource_id) {
                return false;
            }
        }

        if let Some(ref results) = self.results {
            if !results.contains(&entry.result) {
                return false;
            }
        }

        if let Some(start) = self.start_date {
            if entry.timestamp < start {
                return false;
            }
        }

        if let Some(end) = self.end_date {
            if entry.timestamp > end {
                return false;
            }
        }

        if let Some(ref term) = self.search_term {
            let term_lower = term.to_lowercase();
            let matches_description = entry.details.description.to_lowercase().contains(&term_lower);
            let matches_name = entry
                .resource_name
                .as_ref()
                .map(|n| n.to_lowercase().contains(&term_lower))
                .unwrap_or(false);
            let matches_email = entry
                .actor_email
                .as_ref()
                .map(|e| e.to_lowercase().contains(&term_lower))
                .unwrap_or(false);

            if !matches_description && !matches_name && !matches_email {
                return false;
            }
        }

        true
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogPage {
    pub entries: Vec<AuditLogEntry>,
    pub page: usize,
    pub per_page: usize,
    pub total: usize,
    pub total_pages: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceExport {
    pub organization_id: Uuid,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub total_events: usize,
    pub action_summary: std::collections::HashMap<String, u64>,
    pub access_denied_count: usize,
    pub unique_actors: usize,
    pub entries: Vec<AuditLogEntry>,
    pub generated_at: DateTime<Utc>,
}

pub fn create_audit_logger() -> AuditLogger {
    AuditLogger::default()
}

pub fn create_audit_logger_with_config(max_entries: usize, retention_days: u32) -> AuditLogger {
    AuditLogger::new(max_entries, retention_days)
}
