use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnonymousSession {
    pub id: Uuid,
    pub fingerprint: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub message_count: u32,
    pub bot_id: Option<Uuid>,
    pub metadata: HashMap<String, String>,
    pub upgraded_to_user_id: Option<Uuid>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnonymousSessionConfig {
    pub session_ttl_minutes: i64,
    pub max_messages_per_session: u32,
    pub max_sessions_per_ip: u32,
    pub require_fingerprint: bool,
    pub allow_session_upgrade: bool,
    pub preserve_history_on_upgrade: bool,
}

impl Default for AnonymousSessionConfig {
    fn default() -> Self {
        Self {
            session_ttl_minutes: 60,
            max_messages_per_session: 20,
            max_sessions_per_ip: 5,
            require_fingerprint: false,
            allow_session_upgrade: true,
            preserve_history_on_upgrade: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMessage {
    pub id: Uuid,
    pub session_id: Uuid,
    pub role: MessageRole,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionUpgradeResult {
    pub success: bool,
    pub anonymous_session_id: Uuid,
    pub user_id: Uuid,
    pub messages_migrated: u32,
    pub upgraded_at: DateTime<Utc>,
}

pub struct AnonymousSessionManager {
    sessions: Arc<RwLock<HashMap<Uuid, AnonymousSession>>>,
    messages: Arc<RwLock<HashMap<Uuid, Vec<SessionMessage>>>>,
    ip_session_count: Arc<RwLock<HashMap<String, u32>>>,
    config: AnonymousSessionConfig,
}

impl AnonymousSessionManager {
    pub fn new(config: AnonymousSessionConfig) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            messages: Arc::new(RwLock::new(HashMap::new())),
            ip_session_count: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    pub async fn create_session(
        &self,
        ip_address: Option<String>,
        user_agent: Option<String>,
        fingerprint: Option<String>,
        bot_id: Option<Uuid>,
    ) -> Result<AnonymousSession, AnonymousSessionError> {
        if self.config.require_fingerprint && fingerprint.is_none() {
            return Err(AnonymousSessionError::FingerprintRequired);
        }

        if let Some(ref ip) = ip_address {
            let ip_counts = self.ip_session_count.read().await;
            let current_count = ip_counts.get(ip).copied().unwrap_or(0);
            if current_count >= self.config.max_sessions_per_ip {
                return Err(AnonymousSessionError::TooManySessions);
            }
        }

        let now = Utc::now();
        let session = AnonymousSession {
            id: Uuid::new_v4(),
            fingerprint,
            ip_address: ip_address.clone(),
            user_agent,
            created_at: now,
            last_activity: now,
            expires_at: now + Duration::minutes(self.config.session_ttl_minutes),
            message_count: 0,
            bot_id,
            metadata: HashMap::new(),
            upgraded_to_user_id: None,
            is_active: true,
        };

        let mut sessions = self.sessions.write().await;
        sessions.insert(session.id, session.clone());

        let mut messages = self.messages.write().await;
        messages.insert(session.id, Vec::new());

        if let Some(ip) = ip_address {
            let mut ip_counts = self.ip_session_count.write().await;
            *ip_counts.entry(ip).or_insert(0) += 1;
        }

        Ok(session)
    }

    pub async fn get_session(&self, session_id: Uuid) -> Option<AnonymousSession> {
        let sessions = self.sessions.read().await;
        let session = sessions.get(&session_id)?;

        if !session.is_active || session.expires_at < Utc::now() {
            return None;
        }

        Some(session.clone())
    }

    pub async fn get_or_create_session(
        &self,
        session_id: Option<Uuid>,
        ip_address: Option<String>,
        user_agent: Option<String>,
        fingerprint: Option<String>,
        bot_id: Option<Uuid>,
    ) -> Result<AnonymousSession, AnonymousSessionError> {
        if let Some(id) = session_id {
            if let Some(session) = self.get_session(id).await {
                return Ok(session);
            }
        }

        self.create_session(ip_address, user_agent, fingerprint, bot_id).await
    }

    pub async fn add_message(
        &self,
        session_id: Uuid,
        role: MessageRole,
        content: String,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<SessionMessage, AnonymousSessionError> {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(&session_id)
            .ok_or(AnonymousSessionError::SessionNotFound)?;

        if !session.is_active {
            return Err(AnonymousSessionError::SessionExpired);
        }

        if session.expires_at < Utc::now() {
            session.is_active = false;
            return Err(AnonymousSessionError::SessionExpired);
        }

        if role == MessageRole::User {
            if session.message_count >= self.config.max_messages_per_session {
                return Err(AnonymousSessionError::MessageLimitReached);
            }
            session.message_count += 1;
        }

        session.last_activity = Utc::now();
        session.expires_at = Utc::now() + Duration::minutes(self.config.session_ttl_minutes);

        let message = SessionMessage {
            id: Uuid::new_v4(),
            session_id,
            role,
            content,
            timestamp: Utc::now(),
            metadata,
        };

        drop(sessions);

        let mut messages = self.messages.write().await;
        messages
            .entry(session_id)
            .or_default()
            .push(message.clone());

        Ok(message)
    }

    pub async fn get_messages(&self, session_id: Uuid) -> Vec<SessionMessage> {
        let messages = self.messages.read().await;
        messages.get(&session_id).cloned().unwrap_or_default()
    }

    pub async fn upgrade_session(
        &self,
        session_id: Uuid,
        user_id: Uuid,
    ) -> Result<SessionUpgradeResult, AnonymousSessionError> {
        if !self.config.allow_session_upgrade {
            return Err(AnonymousSessionError::UpgradeNotAllowed);
        }

        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(&session_id)
            .ok_or(AnonymousSessionError::SessionNotFound)?;

        if session.upgraded_to_user_id.is_some() {
            return Err(AnonymousSessionError::AlreadyUpgraded);
        }

        session.upgraded_to_user_id = Some(user_id);
        session.is_active = false;

        let messages = self.messages.read().await;
        let message_count = messages
            .get(&session_id)
            .map(|m| m.len() as u32)
            .unwrap_or(0);

        if let Some(ref ip) = session.ip_address {
            drop(sessions);
            let mut ip_counts = self.ip_session_count.write().await;
            if let Some(count) = ip_counts.get_mut(ip) {
                *count = count.saturating_sub(1);
            }
        }

        Ok(SessionUpgradeResult {
            success: true,
            anonymous_session_id: session_id,
            user_id,
            messages_migrated: message_count,
            upgraded_at: Utc::now(),
        })
    }

    pub async fn get_messages_for_migration(&self, session_id: Uuid) -> Option<Vec<SessionMessage>> {
        if !self.config.preserve_history_on_upgrade {
            return None;
        }

        let sessions = self.sessions.read().await;
        let session = sessions.get(&session_id)?;

        if session.upgraded_to_user_id.is_none() {
            return None;
        }

        let messages = self.messages.read().await;
        messages.get(&session_id).cloned()
    }

    pub async fn invalidate_session(&self, session_id: Uuid) -> bool {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(&session_id) {
            session.is_active = false;

            if let Some(ref ip) = session.ip_address {
                let ip_clone = ip.clone();
                drop(sessions);
                let mut ip_counts = self.ip_session_count.write().await;
                if let Some(count) = ip_counts.get_mut(&ip_clone) {
                    *count = count.saturating_sub(1);
                }
            }

            return true;
        }
        false
    }

    pub async fn cleanup_expired_sessions(&self) -> u32 {
        let now = Utc::now();
        let mut cleaned = 0;

        let mut sessions = self.sessions.write().await;
        let expired_ids: Vec<(Uuid, Option<String>)> = sessions
            .iter()
            .filter(|(_, s)| s.expires_at < now || !s.is_active)
            .map(|(id, s)| (*id, s.ip_address.clone()))
            .collect();

        for (id, ip) in &expired_ids {
            sessions.remove(id);
            cleaned += 1;

            if let Some(ip_addr) = ip {
                let mut ip_counts = self.ip_session_count.write().await;
                if let Some(count) = ip_counts.get_mut(ip_addr) {
                    *count = count.saturating_sub(1);
                }
            }
        }

        drop(sessions);

        let mut messages = self.messages.write().await;
        for (id, _) in expired_ids {
            messages.remove(&id);
        }

        cleaned
    }

    pub async fn get_session_stats(&self) -> SessionStats {
        let sessions = self.sessions.read().await;
        let messages = self.messages.read().await;

        let active_sessions = sessions.values().filter(|s| s.is_active).count();
        let total_messages: usize = messages.values().map(|m| m.len()).sum();
        let upgraded_sessions = sessions
            .values()
            .filter(|s| s.upgraded_to_user_id.is_some())
            .count();

        SessionStats {
            total_sessions: sessions.len(),
            active_sessions,
            upgraded_sessions,
            total_messages,
            config: self.config.clone(),
        }
    }

    pub async fn get_remaining_messages(&self, session_id: Uuid) -> Option<u32> {
        let sessions = self.sessions.read().await;
        let session = sessions.get(&session_id)?;
        Some(self.config.max_messages_per_session.saturating_sub(session.message_count))
    }

    pub async fn extend_session(&self, session_id: Uuid, additional_minutes: i64) -> bool {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(&session_id) {
            if session.is_active {
                session.expires_at = session.expires_at + Duration::minutes(additional_minutes);
                return true;
            }
        }
        false
    }

    pub async fn set_metadata(&self, session_id: Uuid, key: String, value: String) -> bool {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(&session_id) {
            session.metadata.insert(key, value);
            return true;
        }
        false
    }

    pub fn config(&self) -> &AnonymousSessionConfig {
        &self.config
    }
}

impl Default for AnonymousSessionManager {
    fn default() -> Self {
        Self::new(AnonymousSessionConfig::default())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStats {
    pub total_sessions: usize,
    pub active_sessions: usize,
    pub upgraded_sessions: usize,
    pub total_messages: usize,
    pub config: AnonymousSessionConfig,
}

#[derive(Debug, Clone)]
pub enum AnonymousSessionError {
    SessionNotFound,
    SessionExpired,
    MessageLimitReached,
    TooManySessions,
    FingerprintRequired,
    UpgradeNotAllowed,
    AlreadyUpgraded,
    InvalidSession,
}

impl std::fmt::Display for AnonymousSessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SessionNotFound => write!(f, "Session not found"),
            Self::SessionExpired => write!(f, "Session has expired"),
            Self::MessageLimitReached => write!(f, "Message limit reached for anonymous session"),
            Self::TooManySessions => write!(f, "Too many sessions from this IP address"),
            Self::FingerprintRequired => write!(f, "Browser fingerprint is required"),
            Self::UpgradeNotAllowed => write!(f, "Session upgrade is not allowed"),
            Self::AlreadyUpgraded => write!(f, "Session has already been upgraded"),
            Self::InvalidSession => write!(f, "Invalid session"),
        }
    }
}

impl std::error::Error for AnonymousSessionError {}

pub async fn session_cleanup_job(manager: Arc<AnonymousSessionManager>, interval_seconds: u64) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(interval_seconds));

    loop {
        interval.tick().await;
        let cleaned = manager.cleanup_expired_sessions().await;
        if cleaned > 0 {
            tracing::info!("Cleaned up {} expired anonymous sessions", cleaned);
        }
    }
}
