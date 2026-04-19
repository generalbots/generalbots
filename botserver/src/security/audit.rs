use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AuditEventCategory {
    Authentication,
    Authorization,
    DataAccess,
    DataModification,
    Administration,
    Security,
    System,
    Compliance,
}

impl AuditEventCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Authentication => "authentication",
            Self::Authorization => "authorization",
            Self::DataAccess => "data_access",
            Self::DataModification => "data_modification",
            Self::Administration => "administration",
            Self::Security => "security",
            Self::System => "system",
            Self::Compliance => "compliance",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AuditEventType {
    LoginSuccess,
    LoginFailure,
    Logout,
    MfaEnabled,
    MfaDisabled,
    MfaChallenge,
    PasswordChange,
    PasswordReset,
    SessionCreate,
    SessionRevoke,
    SessionExpire,

    PermissionGranted,
    PermissionDenied,
    RoleAssigned,
    RoleRemoved,
    GroupJoined,
    GroupLeft,

    DataRead,
    DataCreate,
    DataUpdate,
    DataDelete,
    DataShare,
    DataUnshare,
    DataDownload,
    DataExport,

    UserCreate,
    UserDelete,
    UserModify,
    UserDisable,
    UserEnable,
    ConfigChange,
    SettingChange,
    BotCreate,
    BotDelete,
    BotModify,

    ThreatDetected,
    PolicyViolation,
    RateLimitExceeded,
    SuspiciousActivity,
    BruteForceAttempt,
    InjectionAttempt,
    UnauthorizedAccess,

    ServiceStart,
    ServiceStop,
    BackupCreate,
    BackupRestore,
    MaintenanceStart,
    MaintenanceEnd,

    ConsentGiven,
    ConsentWithdrawn,
    DataExportRequest,
    DataDeletionRequest,
    PrivacyPolicyAccepted,
}

impl AuditEventType {
    pub fn category(&self) -> AuditEventCategory {
        match self {
            Self::LoginSuccess
            | Self::LoginFailure
            | Self::Logout
            | Self::MfaEnabled
            | Self::MfaDisabled
            | Self::MfaChallenge
            | Self::PasswordChange
            | Self::PasswordReset
            | Self::SessionCreate
            | Self::SessionRevoke
            | Self::SessionExpire => AuditEventCategory::Authentication,

            Self::PermissionGranted
            | Self::PermissionDenied
            | Self::RoleAssigned
            | Self::RoleRemoved
            | Self::GroupJoined
            | Self::GroupLeft => AuditEventCategory::Authorization,

            Self::DataRead | Self::DataDownload => AuditEventCategory::DataAccess,

            Self::DataCreate
            | Self::DataUpdate
            | Self::DataDelete
            | Self::DataShare
            | Self::DataUnshare
            | Self::DataExport => AuditEventCategory::DataModification,

            Self::UserCreate
            | Self::UserDelete
            | Self::UserModify
            | Self::UserDisable
            | Self::UserEnable
            | Self::ConfigChange
            | Self::SettingChange
            | Self::BotCreate
            | Self::BotDelete
            | Self::BotModify => AuditEventCategory::Administration,

            Self::ThreatDetected
            | Self::PolicyViolation
            | Self::RateLimitExceeded
            | Self::SuspiciousActivity
            | Self::BruteForceAttempt
            | Self::InjectionAttempt
            | Self::UnauthorizedAccess => AuditEventCategory::Security,

            Self::ServiceStart
            | Self::ServiceStop
            | Self::BackupCreate
            | Self::BackupRestore
            | Self::MaintenanceStart
            | Self::MaintenanceEnd => AuditEventCategory::System,

            Self::ConsentGiven
            | Self::ConsentWithdrawn
            | Self::DataExportRequest
            | Self::DataDeletionRequest
            | Self::PrivacyPolicyAccepted => AuditEventCategory::Compliance,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::LoginSuccess => "LOGIN_SUCCESS",
            Self::LoginFailure => "LOGIN_FAILURE",
            Self::Logout => "LOGOUT",
            Self::MfaEnabled => "MFA_ENABLED",
            Self::MfaDisabled => "MFA_DISABLED",
            Self::MfaChallenge => "MFA_CHALLENGE",
            Self::PasswordChange => "PASSWORD_CHANGE",
            Self::PasswordReset => "PASSWORD_RESET",
            Self::SessionCreate => "SESSION_CREATE",
            Self::SessionRevoke => "SESSION_REVOKE",
            Self::SessionExpire => "SESSION_EXPIRE",
            Self::PermissionGranted => "PERMISSION_GRANTED",
            Self::PermissionDenied => "PERMISSION_DENIED",
            Self::RoleAssigned => "ROLE_ASSIGNED",
            Self::RoleRemoved => "ROLE_REMOVED",
            Self::GroupJoined => "GROUP_JOINED",
            Self::GroupLeft => "GROUP_LEFT",
            Self::DataRead => "DATA_READ",
            Self::DataCreate => "DATA_CREATE",
            Self::DataUpdate => "DATA_UPDATE",
            Self::DataDelete => "DATA_DELETE",
            Self::DataShare => "DATA_SHARE",
            Self::DataUnshare => "DATA_UNSHARE",
            Self::DataDownload => "DATA_DOWNLOAD",
            Self::DataExport => "DATA_EXPORT",
            Self::UserCreate => "USER_CREATE",
            Self::UserDelete => "USER_DELETE",
            Self::UserModify => "USER_MODIFY",
            Self::UserDisable => "USER_DISABLE",
            Self::UserEnable => "USER_ENABLE",
            Self::ConfigChange => "CONFIG_CHANGE",
            Self::SettingChange => "SETTING_CHANGE",
            Self::BotCreate => "BOT_CREATE",
            Self::BotDelete => "BOT_DELETE",
            Self::BotModify => "BOT_MODIFY",
            Self::ThreatDetected => "THREAT_DETECTED",
            Self::PolicyViolation => "POLICY_VIOLATION",
            Self::RateLimitExceeded => "RATE_LIMIT_EXCEEDED",
            Self::SuspiciousActivity => "SUSPICIOUS_ACTIVITY",
            Self::BruteForceAttempt => "BRUTE_FORCE_ATTEMPT",
            Self::InjectionAttempt => "INJECTION_ATTEMPT",
            Self::UnauthorizedAccess => "UNAUTHORIZED_ACCESS",
            Self::ServiceStart => "SERVICE_START",
            Self::ServiceStop => "SERVICE_STOP",
            Self::BackupCreate => "BACKUP_CREATE",
            Self::BackupRestore => "BACKUP_RESTORE",
            Self::MaintenanceStart => "MAINTENANCE_START",
            Self::MaintenanceEnd => "MAINTENANCE_END",
            Self::ConsentGiven => "CONSENT_GIVEN",
            Self::ConsentWithdrawn => "CONSENT_WITHDRAWN",
            Self::DataExportRequest => "DATA_EXPORT_REQUEST",
            Self::DataDeletionRequest => "DATA_DELETION_REQUEST",
            Self::PrivacyPolicyAccepted => "PRIVACY_POLICY_ACCEPTED",
        }
    }

    pub fn severity(&self) -> AuditSeverity {
        match self {
            Self::LoginFailure
            | Self::PermissionDenied
            | Self::UnauthorizedAccess
            | Self::RateLimitExceeded => AuditSeverity::Warning,

            Self::ThreatDetected
            | Self::PolicyViolation
            | Self::BruteForceAttempt
            | Self::InjectionAttempt
            | Self::SuspiciousActivity => AuditSeverity::Critical,

            Self::UserDelete
            | Self::DataDelete
            | Self::PasswordChange
            | Self::PasswordReset
            | Self::MfaDisabled
            | Self::ConfigChange
            | Self::BotDelete => AuditSeverity::High,

            _ => AuditSeverity::Info,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum AuditSeverity {
    Debug,
    Info,
    Warning,
    High,
    Critical,
}

impl AuditSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Debug => "debug",
            Self::Info => "info",
            Self::Warning => "warning",
            Self::High => "high",
            Self::Critical => "critical",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditOutcome {
    Success,
    Failure,
    Partial,
    Unknown,
}

impl AuditOutcome {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Failure => "failure",
            Self::Partial => "partial",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditActor {
    pub user_id: Option<Uuid>,
    pub username: Option<String>,
    pub email: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub session_id: Option<String>,
    pub actor_type: ActorType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActorType {
    User,
    Service,
    System,
    Bot,
    Anonymous,
}

impl Default for AuditActor {
    fn default() -> Self {
        Self {
            user_id: None,
            username: None,
            email: None,
            ip_address: None,
            user_agent: None,
            session_id: None,
            actor_type: ActorType::Anonymous,
        }
    }
}

impl AuditActor {
    pub fn user(user_id: Uuid) -> Self {
        Self {
            user_id: Some(user_id),
            actor_type: ActorType::User,
            ..Default::default()
        }
    }

    pub fn system() -> Self {
        Self {
            actor_type: ActorType::System,
            ..Default::default()
        }
    }

    pub fn service(name: &str) -> Self {
        Self {
            username: Some(name.to_string()),
            actor_type: ActorType::Service,
            ..Default::default()
        }
    }

    pub fn bot(bot_id: Uuid) -> Self {
        Self {
            user_id: Some(bot_id),
            actor_type: ActorType::Bot,
            ..Default::default()
        }
    }

    pub fn anonymous() -> Self {
        Self::default()
    }

    pub fn with_username(mut self, username: String) -> Self {
        self.username = Some(username);
        self
    }

    pub fn with_email(mut self, email: String) -> Self {
        self.email = Some(email);
        self
    }

    pub fn with_ip(mut self, ip: String) -> Self {
        self.ip_address = Some(ip);
        self
    }

    pub fn with_user_agent(mut self, ua: String) -> Self {
        self.user_agent = Some(ua);
        self
    }

    pub fn with_session(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditResource {
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub resource_name: Option<String>,
    pub parent_resource: Option<Box<AuditResource>>,
}

impl AuditResource {
    pub fn new(resource_type: &str) -> Self {
        Self {
            resource_type: resource_type.to_string(),
            resource_id: None,
            resource_name: None,
            parent_resource: None,
        }
    }

    pub fn with_id(mut self, id: &str) -> Self {
        self.resource_id = Some(id.to_string());
        self
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.resource_name = Some(name.to_string());
        self
    }

    pub fn with_parent(mut self, parent: AuditResource) -> Self {
        self.parent_resource = Some(Box::new(parent));
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub event_type: AuditEventType,
    pub category: AuditEventCategory,
    pub severity: AuditSeverity,
    pub outcome: AuditOutcome,
    pub actor: AuditActor,
    pub resource: Option<AuditResource>,
    pub action: String,
    pub description: String,
    pub metadata: HashMap<String, serde_json::Value>,
    pub request_id: Option<String>,
    pub organization_id: Option<Uuid>,
    pub previous_hash: Option<String>,
    pub hash: String,
}

impl AuditEvent {
    pub fn new(event_type: AuditEventType, actor: AuditActor) -> Self {
        let id = Uuid::new_v4();
        let timestamp = Utc::now();
        let category = event_type.category();
        let severity = event_type.severity();
        let action = event_type.as_str().to_string();

        let mut event = Self {
            id,
            timestamp,
            event_type,
            category,
            severity,
            outcome: AuditOutcome::Success,
            actor,
            resource: None,
            action,
            description: String::new(),
            metadata: HashMap::new(),
            request_id: None,
            organization_id: None,
            previous_hash: None,
            hash: String::new(),
        };

        event.hash = event.compute_hash();
        event
    }

    pub fn with_outcome(mut self, outcome: AuditOutcome) -> Self {
        self.outcome = outcome;
        self.hash = self.compute_hash();
        self
    }

    pub fn with_resource(mut self, resource: AuditResource) -> Self {
        self.resource = Some(resource);
        self.hash = self.compute_hash();
        self
    }

    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self.hash = self.compute_hash();
        self
    }

    pub fn with_metadata(mut self, key: &str, value: serde_json::Value) -> Self {
        self.metadata.insert(key.to_string(), value);
        self.hash = self.compute_hash();
        self
    }

    pub fn with_request_id(mut self, request_id: String) -> Self {
        self.request_id = Some(request_id);
        self.hash = self.compute_hash();
        self
    }

    pub fn with_organization(mut self, org_id: Uuid) -> Self {
        self.organization_id = Some(org_id);
        self.hash = self.compute_hash();
        self
    }

    pub fn with_previous_hash(mut self, hash: String) -> Self {
        self.previous_hash = Some(hash);
        self.hash = self.compute_hash();
        self
    }

    fn compute_hash(&self) -> String {
        let mut hasher = Sha256::new();

        hasher.update(self.id.as_bytes());
        hasher.update(self.timestamp.to_rfc3339().as_bytes());
        hasher.update(self.event_type.as_str().as_bytes());
        hasher.update(self.outcome.as_str().as_bytes());
        hasher.update(self.action.as_bytes());
        hasher.update(self.description.as_bytes());

        if let Some(ref prev) = self.previous_hash {
            hasher.update(prev.as_bytes());
        }

        if let Some(ref actor_id) = self.actor.user_id {
            hasher.update(actor_id.as_bytes());
        }

        let result = hasher.finalize();
        hex::encode(result)
    }

    pub fn verify_hash(&self) -> bool {
        self.hash == self.compute_hash()
    }

    pub fn is_security_event(&self) -> bool {
        self.category == AuditEventCategory::Security
    }

    pub fn is_critical(&self) -> bool {
        self.severity == AuditSeverity::Critical
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    pub enabled: bool,
    pub async_logging: bool,
    pub buffer_size: usize,
    pub flush_interval_seconds: u64,
    pub retention_days: u32,
    pub min_severity: AuditSeverity,
    pub categories_enabled: Vec<AuditEventCategory>,
    pub tamper_evident: bool,
    pub compress_old_logs: bool,
    pub encrypt_sensitive_data: bool,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            async_logging: true,
            buffer_size: 1000,
            flush_interval_seconds: 5,
            retention_days: 365,
            min_severity: AuditSeverity::Info,
            categories_enabled: vec![
                AuditEventCategory::Authentication,
                AuditEventCategory::Authorization,
                AuditEventCategory::DataAccess,
                AuditEventCategory::DataModification,
                AuditEventCategory::Administration,
                AuditEventCategory::Security,
                AuditEventCategory::Compliance,
            ],
            tamper_evident: true,
            compress_old_logs: true,
            encrypt_sensitive_data: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditQuery {
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub event_types: Option<Vec<AuditEventType>>,
    pub categories: Option<Vec<AuditEventCategory>>,
    pub severities: Option<Vec<AuditSeverity>>,
    pub outcomes: Option<Vec<AuditOutcome>>,
    pub actor_id: Option<Uuid>,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub organization_id: Option<Uuid>,
    pub request_id: Option<String>,
    pub search_text: Option<String>,
    pub limit: usize,
    pub offset: usize,
}

impl Default for AuditQuery {
    fn default() -> Self {
        Self {
            start_time: None,
            end_time: None,
            event_types: None,
            categories: None,
            severities: None,
            outcomes: None,
            actor_id: None,
            resource_type: None,
            resource_id: None,
            organization_id: None,
            request_id: None,
            search_text: None,
            limit: 100,
            offset: 0,
        }
    }
}

impl AuditQuery {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_time_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.start_time = Some(start);
        self.end_time = Some(end);
        self
    }

    pub fn with_event_types(mut self, types: Vec<AuditEventType>) -> Self {
        self.event_types = Some(types);
        self
    }

    pub fn with_categories(mut self, categories: Vec<AuditEventCategory>) -> Self {
        self.categories = Some(categories);
        self
    }

    pub fn with_actor(mut self, actor_id: Uuid) -> Self {
        self.actor_id = Some(actor_id);
        self
    }

    pub fn with_resource(mut self, resource_type: &str, resource_id: &str) -> Self {
        self.resource_type = Some(resource_type.to_string());
        self.resource_id = Some(resource_id.to_string());
        self
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    pub fn with_offset(mut self, offset: usize) -> Self {
        self.offset = offset;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditQueryResult {
    pub events: Vec<AuditEvent>,
    pub total_count: usize,
    pub has_more: bool,
}

pub trait AuditStore: Send + Sync {
    fn store(&self, event: AuditEvent) -> impl std::future::Future<Output = Result<()>> + Send;
    fn store_batch(&self, events: Vec<AuditEvent>) -> impl std::future::Future<Output = Result<()>> + Send;
    fn query(&self, query: AuditQuery) -> impl std::future::Future<Output = Result<AuditQueryResult>> + Send;
    fn get_by_id(&self, id: Uuid) -> impl std::future::Future<Output = Result<Option<AuditEvent>>> + Send;
    fn get_chain(&self, start_id: Uuid, count: usize) -> impl std::future::Future<Output = Result<Vec<AuditEvent>>> + Send;
    fn verify_chain(&self, start_id: Uuid, end_id: Uuid) -> impl std::future::Future<Output = Result<bool>> + Send;
    fn cleanup_old_events(&self, before: DateTime<Utc>) -> impl std::future::Future<Output = Result<usize>> + Send;
}

#[derive(Debug, Clone)]
pub struct InMemoryAuditStore {
    events: Arc<RwLock<Vec<AuditEvent>>>,
    max_events: usize,
}

impl Default for InMemoryAuditStore {
    fn default() -> Self {
        Self::new(100_000)
    }
}

impl InMemoryAuditStore {
    pub fn new(max_events: usize) -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
            max_events,
        }
    }
}

impl AuditStore for InMemoryAuditStore {
    async fn store(&self, event: AuditEvent) -> Result<()> {
        let mut events = self.events.write().await;

        if events.len() >= self.max_events {
            events.remove(0);
        }

        events.push(event);
        Ok(())
    }

    async fn store_batch(&self, new_events: Vec<AuditEvent>) -> Result<()> {
        let mut events = self.events.write().await;

        for event in new_events {
            if events.len() >= self.max_events {
                events.remove(0);
            }
            events.push(event);
        }

        Ok(())
    }

    async fn query(&self, query: AuditQuery) -> Result<AuditQueryResult> {
        let events = self.events.read().await;

        let filtered: Vec<AuditEvent> = events
            .iter()
            .filter(|e| {
                if let Some(ref start) = query.start_time {
                    if e.timestamp < *start {
                        return false;
                    }
                }
                if let Some(ref end) = query.end_time {
                    if e.timestamp > *end {
                        return false;
                    }
                }
                if let Some(ref types) = query.event_types {
                    if !types.contains(&e.event_type) {
                        return false;
                    }
                }
                if let Some(ref categories) = query.categories {
                    if !categories.contains(&e.category) {
                        return false;
                    }
                }
                if let Some(ref severities) = query.severities {
                    if !severities.contains(&e.severity) {
                        return false;
                    }
                }
                if let Some(ref outcomes) = query.outcomes {
                    if !outcomes.contains(&e.outcome) {
                        return false;
                    }
                }
                if let Some(actor_id) = query.actor_id {
                    if e.actor.user_id != Some(actor_id) {
                        return false;
                    }
                }
                if let Some(ref org_id) = query.organization_id {
                    if e.organization_id.as_ref() != Some(org_id) {
                        return false;
                    }
                }
                if let Some(ref resource_type) = query.resource_type {
                    if let Some(ref resource) = e.resource {
                        if &resource.resource_type != resource_type {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                if let Some(ref search) = query.search_text {
                    let search_lower = search.to_lowercase();
                    if !e.description.to_lowercase().contains(&search_lower)
                        && !e.action.to_lowercase().contains(&search_lower)
                    {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();

        let total_count = filtered.len();
        let has_more = query.offset + query.limit < total_count;

        let page: Vec<AuditEvent> = filtered
            .into_iter()
            .rev()
            .skip(query.offset)
            .take(query.limit)
            .collect();

        Ok(AuditQueryResult {
            events: page,
            total_count,
            has_more,
        })
    }

    async fn get_by_id(&self, id: Uuid) -> Result<Option<AuditEvent>> {
        let events = self.events.read().await;
        Ok(events.iter().find(|e| e.id == id).cloned())
    }

    async fn get_chain(&self, start_id: Uuid, count: usize) -> Result<Vec<AuditEvent>> {
        let events = self.events.read().await;

        let start_idx = events.iter().position(|e| e.id == start_id);
        if let Some(idx) = start_idx {
            let end_idx = (idx + count).min(events.len());
            Ok(events[idx..end_idx].to_vec())
        } else {
            Ok(Vec::new())
        }
    }

    async fn verify_chain(&self, start_id: Uuid, end_id: Uuid) -> Result<bool> {
        let events = self.events.read().await;

        let start_idx = events.iter().position(|e| e.id == start_id);
        let end_idx = events.iter().position(|e| e.id == end_id);

        match (start_idx, end_idx) {
            (Some(start), Some(end)) if start <= end => {
                let chain = &events[start..=end];

                for i in 1..chain.len() {
                    if chain[i].previous_hash.as_ref() != Some(&chain[i - 1].hash) {
                        return Ok(false);
                    }
                    if !chain[i].verify_hash() {
                        return Ok(false);
                    }
                }

                Ok(true)
            }
            _ => Ok(false),
        }
    }

    async fn cleanup_old_events(&self, before: DateTime<Utc>) -> Result<usize> {
        let mut events = self.events.write().await;
        let initial_count = events.len();
        events.retain(|e| e.timestamp >= before);
        Ok(initial_count - events.len())
    }
}

pub struct AuditLogger<S: AuditStore> {
    config: AuditConfig,
    store: S,
    buffer: Arc<RwLock<Vec<AuditEvent>>>,
    last_hash: Arc<RwLock<Option<String>>>,
}

impl<S: AuditStore> AuditLogger<S> {
    pub fn new(config: AuditConfig, store: S) -> Self {
        Self {
            config,
            store,
            buffer: Arc::new(RwLock::new(Vec::new())),
            last_hash: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn log(&self, mut event: AuditEvent) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        if event.severity < self.config.min_severity {
            return Ok(());
        }

        if !self.config.categories_enabled.contains(&event.category) {
            return Ok(());
        }

        if self.config.tamper_evident {
            let mut last_hash = self.last_hash.write().await;
            if let Some(ref hash) = *last_hash {
                event = event.with_previous_hash(hash.clone());
            }
            *last_hash = Some(event.hash.clone());
        }

        if event.is_critical() {
            info!(
                "CRITICAL AUDIT: {} - {} - {}",
                event.event_type.as_str(),
                event.action,
                event.description
            );
        }

        if self.config.async_logging {
            let mut buffer = self.buffer.write().await;
            buffer.push(event);

            if buffer.len() >= self.config.buffer_size {
                let events: Vec<AuditEvent> = buffer.drain(..).collect();
                drop(buffer);
                self.store.store_batch(events).await?;
            }
        } else {
            self.store.store(event).await?;
        }

        Ok(())
    }

    pub async fn log_auth_success(&self, actor: AuditActor, method: &str) -> Result<()> {
        let event = AuditEvent::new(AuditEventType::LoginSuccess, actor)
            .with_description(&format!("Successful authentication via {method}"))
            .with_metadata("auth_method", serde_json::json!(method));

        self.log(event).await
    }

    pub async fn log_auth_failure(&self, actor: AuditActor, reason: &str) -> Result<()> {
        let event = AuditEvent::new(AuditEventType::LoginFailure, actor)
            .with_outcome(AuditOutcome::Failure)
            .with_description(&format!("Authentication failed: {reason}"))
            .with_metadata("failure_reason", serde_json::json!(reason));

        self.log(event).await
    }

    pub async fn log_permission_denied(
        &self,
        actor: AuditActor,
        resource: AuditResource,
        permission: &str,
    ) -> Result<()> {
        let event = AuditEvent::new(AuditEventType::PermissionDenied, actor)
            .with_outcome(AuditOutcome::Failure)
            .with_resource(resource)
            .with_description(&format!("Permission denied: {permission}"))
            .with_metadata("required_permission", serde_json::json!(permission));

        self.log(event).await
    }

    pub async fn log_data_access(
        &self,
        actor: AuditActor,
        resource: AuditResource,
        action: &str,
    ) -> Result<()> {
        let event = AuditEvent::new(AuditEventType::DataRead, actor)
            .with_resource(resource)
            .with_description(&format!("Data accessed: {action}"));

        self.log(event).await
    }

    pub async fn log_data_modification(
        &self,
        actor: AuditActor,
        resource: AuditResource,
        event_type: AuditEventType,
        description: &str,
    ) -> Result<()> {
        let event = AuditEvent::new(event_type, actor)
            .with_resource(resource)
            .with_description(description);

        self.log(event).await
    }

    pub async fn log_security_event(
        &self,
        actor: AuditActor,
        event_type: AuditEventType,
        description: &str,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Result<()> {
        let mut event = AuditEvent::new(event_type, actor).with_description(description);

        for (key, value) in metadata {
            event = event.with_metadata(&key, value);
        }

        self.log(event).await
    }

    pub async fn flush(&self) -> Result<()> {
        let events: Vec<AuditEvent> = {
            let mut buffer = self.buffer.write().await;
            buffer.drain(..).collect()
        };

        if !events.is_empty() {
            self.store.store_batch(events).await?;
        }

        Ok(())
    }

    pub async fn query(&self, query: AuditQuery) -> Result<AuditQueryResult> {
        self.store.query(query).await
    }

    pub fn config(&self) -> &AuditConfig {
        &self.config
    }
}

pub fn create_audit_logger() -> AuditLogger<InMemoryAuditStore> {
    AuditLogger::new(AuditConfig::default(), InMemoryAuditStore::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_event_creation() {
        let actor = AuditActor::user(Uuid::new_v4());
        let event = AuditEvent::new(AuditEventType::LoginSuccess, actor);

        assert_eq!(event.event_type, AuditEventType::LoginSuccess);
        assert_eq!(event.category, AuditEventCategory::Authentication);
        assert!(event.verify_hash());
    }

    #[test]
    fn test_audit_event_hash_verification() {
        let actor = AuditActor::user(Uuid::new_v4());
        let event = AuditEvent::new(AuditEventType::DataCreate, actor)
            .with_description("Created new document");

        assert!(event.verify_hash());
    }

    #[test]
    fn test_audit_severity_levels() {
        assert_eq!(
            AuditEventType::LoginSuccess.severity(),
            AuditSeverity::Info
        );
        assert_eq!(
            AuditEventType::LoginFailure.severity(),
            AuditSeverity::Warning
        );
        assert_eq!(
            AuditEventType::ThreatDetected.severity(),
            AuditSeverity::Critical
        );
    }

    #[test]
    fn test_audit_actor_builders() {
        let user_actor = AuditActor::user(Uuid::new_v4())
            .with_username("testuser".into())
            .with_ip("192.168.1.1".into());

        assert!(user_actor.user_id.is_some());
        assert_eq!(user_actor.username, Some("testuser".into()));
        assert_eq!(user_actor.actor_type, ActorType::User);
    }

    #[test]
    fn test_audit_resource_builder() {
        let resource = AuditResource::new("file")
            .with_id("123")
            .with_name("document.pdf");

        assert_eq!(resource.resource_type, "file");
        assert_eq!(resource.resource_id, Some("123".into()));
        assert_eq!(resource.resource_name, Some("document.pdf".into()));
    }

    #[tokio::test]
    async fn test_in_memory_store() {
        let store = InMemoryAuditStore::new(1000);
        let actor = AuditActor::user(Uuid::new_v4());
        let event = AuditEvent::new(AuditEventType::LoginSuccess, actor);
        let event_id = event.id;

        store.store(event).await.expect("Store failed");

        let retrieved = store.get_by_id(event_id).await.expect("Get failed");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.as_ref().map(|e| e.id), Some(event_id));
    }

    #[tokio::test]
    async fn test_audit_query() {
        let store = InMemoryAuditStore::new(1000);

        for _ in 0..5 {
            let actor = AuditActor::user(Uuid::new_v4());
            let event = AuditEvent::new(AuditEventType::LoginSuccess, actor);
            store.store(event).await.expect("Store failed");
        }

        let query = AuditQuery::new()
            .with_event_types(vec![AuditEventType::LoginSuccess])
            .with_limit(10);

        let result = store.query(query).await.expect("Query failed");
        assert_eq!(result.events.len(), 5);
        assert_eq!(result.total_count, 5);
    }

    #[tokio::test]
    async fn test_audit_logger() {
        let logger = create_audit_logger();
        let actor = AuditActor::user(Uuid::new_v4());

        logger
            .log_auth_success(actor.clone(), "password")
            .await
            .expect("Log failed");

        logger
            .log_auth_failure(actor, "invalid_password")
            .await
            .expect("Log failed");

        logger.flush().await.expect("Flush failed");
    }

    #[test]
    fn test_event_category_mapping() {
        assert_eq!(
            AuditEventType::LoginSuccess.category(),
            AuditEventCategory::Authentication
        );
        assert_eq!(
            AuditEventType::PermissionDenied.category(),
            AuditEventCategory::Authorization
        );
        assert_eq!(
            AuditEventType::DataRead.category(),
            AuditEventCategory::DataAccess
        );
        assert_eq!(
            AuditEventType::UserCreate.category(),
            AuditEventCategory::Administration
        );
        assert_eq!(
            AuditEventType::ThreatDetected.category(),
            AuditEventCategory::Security
        );
    }
}
