use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    pub idle_timeout_minutes: i64,
    pub absolute_timeout_hours: i64,
    pub max_concurrent_sessions: usize,
    pub session_id_length: usize,
    pub cookie_name: String,
    pub cookie_secure: bool,
    pub cookie_http_only: bool,
    pub cookie_same_site: SameSite,
    pub enable_device_tracking: bool,
    pub enable_ip_tracking: bool,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            idle_timeout_minutes: 30,
            absolute_timeout_hours: 24,
            max_concurrent_sessions: 5,
            session_id_length: 32,
            cookie_name: "gb_session".into(),
            cookie_secure: true,
            cookie_http_only: true,
            cookie_same_site: SameSite::Strict,
            enable_device_tracking: true,
            enable_ip_tracking: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SameSite {
    Strict,
    Lax,
    None,
}

impl SameSite {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Strict => "Strict",
            Self::Lax => "Lax",
            Self::None => "None",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionStatus {
    Active,
    Expired,
    Revoked,
    Invalidated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct DeviceInfo {
    pub user_agent: Option<String>,
    pub device_type: Option<String>,
    pub os: Option<String>,
    pub browser: Option<String>,
    pub fingerprint: Option<String>,
}


impl DeviceInfo {
    pub fn from_user_agent(user_agent: &str) -> Self {
        let ua_lower = user_agent.to_lowercase();

        let device_type = if ua_lower.contains("mobile") || ua_lower.contains("android") {
            Some("Mobile".into())
        } else if ua_lower.contains("tablet") || ua_lower.contains("ipad") {
            Some("Tablet".into())
        } else {
            Some("Desktop".into())
        };

        let os = if ua_lower.contains("windows") {
            Some("Windows".into())
        } else if ua_lower.contains("mac os") || ua_lower.contains("macos") {
            Some("macOS".into())
        } else if ua_lower.contains("linux") {
            Some("Linux".into())
        } else if ua_lower.contains("android") {
            Some("Android".into())
        } else if ua_lower.contains("iphone") || ua_lower.contains("ipad") {
            Some("iOS".into())
        } else {
            None
        };

        let browser = if ua_lower.contains("firefox") {
            Some("Firefox".into())
        } else if ua_lower.contains("chrome") && !ua_lower.contains("edg") {
            Some("Chrome".into())
        } else if ua_lower.contains("safari") && !ua_lower.contains("chrome") {
            Some("Safari".into())
        } else if ua_lower.contains("edg") {
            Some("Edge".into())
        } else {
            None
        };

        Self {
            user_agent: Some(user_agent.to_string()),
            device_type,
            os,
            browser,
            fingerprint: None,
        }
    }

    pub fn with_fingerprint(mut self, fingerprint: String) -> Self {
        self.fingerprint = Some(fingerprint);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub user_id: Uuid,
    pub status: SessionStatus,
    pub created_at: DateTime<Utc>,
    pub last_accessed_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub absolute_expires_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub device_info: Option<DeviceInfo>,
    pub remember_me: bool,
    pub metadata: HashMap<String, String>,
}

impl Session {
    pub fn new(user_id: Uuid, config: &SessionConfig) -> Self {
        let now = Utc::now();
        let idle_duration = Duration::minutes(config.idle_timeout_minutes);
        let absolute_duration = Duration::hours(config.absolute_timeout_hours);

        Self {
            id: generate_session_id(config.session_id_length),
            user_id,
            status: SessionStatus::Active,
            created_at: now,
            last_accessed_at: now,
            expires_at: now + idle_duration,
            absolute_expires_at: now + absolute_duration,
            ip_address: None,
            device_info: None,
            remember_me: false,
            metadata: HashMap::new(),
        }
    }

    pub fn with_ip(mut self, ip: String) -> Self {
        self.ip_address = Some(ip);
        self
    }

    pub fn with_device(mut self, device: DeviceInfo) -> Self {
        self.device_info = Some(device);
        self
    }

    pub fn with_remember_me(mut self, remember: bool) -> Self {
        self.remember_me = remember;
        if remember {
            self.absolute_expires_at = self.created_at + Duration::days(30);
        }
        self
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    pub fn is_valid(&self) -> bool {
        self.status == SessionStatus::Active && !self.is_expired()
    }

    pub fn is_expired(&self) -> bool {
        let now = Utc::now();
        now > self.expires_at || now > self.absolute_expires_at
    }

    pub fn touch(&mut self, idle_timeout_minutes: i64) {
        let now = Utc::now();
        self.last_accessed_at = now;
        self.expires_at = now + Duration::minutes(idle_timeout_minutes);
    }

    pub fn revoke(&mut self) {
        self.status = SessionStatus::Revoked;
    }

    pub fn invalidate(&mut self) {
        self.status = SessionStatus::Invalidated;
    }

    pub fn time_until_expiry(&self) -> Duration {
        let idle_remaining = self.expires_at - Utc::now();
        let absolute_remaining = self.absolute_expires_at - Utc::now();

        if idle_remaining < absolute_remaining {
            idle_remaining
        } else {
            absolute_remaining
        }
    }
}

pub trait SessionStore: Send + Sync {
    fn create(&self, session: Session) -> impl std::future::Future<Output = Result<()>> + Send;
    fn get(&self, session_id: &str) -> impl std::future::Future<Output = Result<Option<Session>>> + Send;
    fn update(&self, session: &Session) -> impl std::future::Future<Output = Result<()>> + Send;
    fn delete(&self, session_id: &str) -> impl std::future::Future<Output = Result<()>> + Send;
    fn get_user_sessions(&self, user_id: Uuid) -> impl std::future::Future<Output = Result<Vec<Session>>> + Send;
    fn delete_user_sessions(&self, user_id: Uuid) -> impl std::future::Future<Output = Result<usize>> + Send;
    fn cleanup_expired(&self) -> impl std::future::Future<Output = Result<usize>> + Send;
}

#[derive(Debug, Clone)]
pub struct InMemorySessionStore {
    sessions: Arc<RwLock<HashMap<String, Session>>>,
}

impl Default for InMemorySessionStore {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemorySessionStore {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl SessionStore for InMemorySessionStore {
    async fn create(&self, session: Session) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        sessions.insert(session.id.clone(), session);
        Ok(())
    }

    async fn get(&self, session_id: &str) -> Result<Option<Session>> {
        let sessions = self.sessions.read().await;
        Ok(sessions.get(session_id).cloned())
    }

    async fn update(&self, session: &Session) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        if sessions.contains_key(&session.id) {
            sessions.insert(session.id.clone(), session.clone());
            Ok(())
        } else {
            Err(anyhow!("Session not found: {}", session.id))
        }
    }

    async fn delete(&self, session_id: &str) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        sessions.remove(session_id);
        Ok(())
    }

    async fn get_user_sessions(&self, user_id: Uuid) -> Result<Vec<Session>> {
        let sessions = self.sessions.read().await;
        let user_sessions: Vec<Session> = sessions
            .values()
            .filter(|s| s.user_id == user_id)
            .cloned()
            .collect();
        Ok(user_sessions)
    }

    async fn delete_user_sessions(&self, user_id: Uuid) -> Result<usize> {
        let mut sessions = self.sessions.write().await;
        let initial_count = sessions.len();
        sessions.retain(|_, s| s.user_id != user_id);
        let deleted = initial_count - sessions.len();
        Ok(deleted)
    }

    async fn cleanup_expired(&self) -> Result<usize> {
        let mut sessions = self.sessions.write().await;
        let initial_count = sessions.len();
        sessions.retain(|_, s| !s.is_expired());
        let cleaned = initial_count - sessions.len();
        Ok(cleaned)
    }
}

pub struct SessionManager<S: SessionStore> {
    store: S,
    config: SessionConfig,
}

impl<S: SessionStore> SessionManager<S> {
    pub fn new(store: S, config: SessionConfig) -> Self {
        Self { store, config }
    }

    pub async fn create_session(
        &self,
        user_id: Uuid,
        ip_address: Option<String>,
        user_agent: Option<&str>,
        remember_me: bool,
    ) -> Result<Session> {
        let existing_sessions = self.store.get_user_sessions(user_id).await?;
        let active_count = existing_sessions.iter().filter(|s| s.is_valid()).count();

        if active_count >= self.config.max_concurrent_sessions {
            let mut oldest_sessions: Vec<_> = existing_sessions
                .into_iter()
                .filter(|s| s.is_valid())
                .collect();
            oldest_sessions.sort_by_key(|s| s.last_accessed_at);

            let sessions_to_remove = active_count - self.config.max_concurrent_sessions + 1;
            for session in oldest_sessions.iter().take(sessions_to_remove) {
                self.store.delete(&session.id).await?;
                debug!("Removed oldest session {} for user {user_id}", session.id);
            }
        }

        let mut session = Session::new(user_id, &self.config).with_remember_me(remember_me);

        if let Some(ip) = ip_address {
            session = session.with_ip(ip);
        }

        if self.config.enable_device_tracking {
            if let Some(ua) = user_agent {
                session = session.with_device(DeviceInfo::from_user_agent(ua));
            }
        }

        self.store.create(session.clone()).await?;
        info!("Created session {} for user {user_id}", session.id);

        Ok(session)
    }

    pub async fn validate_session(&self, session_id: &str) -> Result<Option<Session>> {
        let session = match self.store.get(session_id).await? {
            Some(s) => s,
            None => return Ok(None),
        };

        if !session.is_valid() {
            if session.is_expired() {
                self.store.delete(session_id).await?;
                debug!("Cleaned up expired session {session_id}");
            }
            return Ok(None);
        }

        Ok(Some(session))
    }

    pub async fn touch_session(&self, session_id: &str) -> Result<bool> {
        let mut session = match self.store.get(session_id).await? {
            Some(s) if s.is_valid() => s,
            _ => return Ok(false),
        };

        session.touch(self.config.idle_timeout_minutes);
        self.store.update(&session).await?;

        Ok(true)
    }

    pub async fn revoke_session(&self, session_id: &str) -> Result<bool> {
        let mut session = match self.store.get(session_id).await? {
            Some(s) => s,
            None => return Ok(false),
        };

        session.revoke();
        self.store.update(&session).await?;
        info!("Revoked session {session_id}");

        Ok(true)
    }

    pub async fn revoke_all_user_sessions(&self, user_id: Uuid) -> Result<usize> {
        let sessions = self.store.get_user_sessions(user_id).await?;
        let mut revoked = 0;

        for mut session in sessions {
            if session.status == SessionStatus::Active {
                session.revoke();
                self.store.update(&session).await?;
                revoked += 1;
            }
        }

        info!("Revoked {revoked} sessions for user {user_id}");
        Ok(revoked)
    }

    pub async fn revoke_all_except(&self, user_id: Uuid, keep_session_id: &str) -> Result<usize> {
        let sessions = self.store.get_user_sessions(user_id).await?;
        let mut revoked = 0;

        for mut session in sessions {
            if session.id != keep_session_id && session.status == SessionStatus::Active {
                session.revoke();
                self.store.update(&session).await?;
                revoked += 1;
            }
        }

        info!("Revoked {revoked} other sessions for user {user_id}");
        Ok(revoked)
    }

    pub async fn get_user_sessions(&self, user_id: Uuid) -> Result<Vec<Session>> {
        let sessions = self.store.get_user_sessions(user_id).await?;
        Ok(sessions.into_iter().filter(|s| s.is_valid()).collect())
    }

    pub async fn regenerate_session(&self, old_session_id: &str, ip_address: Option<String>, user_agent: Option<&str>) -> Result<Option<Session>> {
        let old_session = match self.store.get(old_session_id).await? {
            Some(s) if s.is_valid() => s,
            _ => return Ok(None),
        };

        let user_id = old_session.user_id;

        let mut new_session = Session::new(user_id, &self.config)
            .with_remember_me(old_session.remember_me)
            .with_metadata("regenerated_from".to_string(), old_session.id.clone());

        if let Some(ip) = ip_address {
            new_session = new_session.with_ip(ip);
        }

        if self.config.enable_device_tracking {
            if let Some(ua) = user_agent {
                new_session = new_session.with_device(DeviceInfo::from_user_agent(ua));
            }
        }

        for (key, value) in old_session.metadata {
            if key != "regenerated_from" {
                new_session = new_session.with_metadata(key, value);
            }
        }

        self.store.delete(old_session_id).await?;
        self.store.create(new_session.clone()).await?;

        info!("Regenerated session {} -> {} for user {user_id}", old_session_id, new_session.id);

        Ok(Some(new_session))
    }

    pub async fn invalidate_on_password_change(&self, user_id: Uuid) -> Result<usize> {
        let count = self.store.delete_user_sessions(user_id).await?;
        info!("Invalidated {count} sessions for user {user_id} due to password change");
        Ok(count)
    }

    pub async fn cleanup_expired_sessions(&self) -> Result<usize> {
        let cleaned = self.store.cleanup_expired().await?;
        if cleaned > 0 {
            info!("Cleaned up {cleaned} expired sessions");
        }
        Ok(cleaned)
    }

    pub fn build_cookie(&self, session: &Session) -> String {
        let max_age = session.time_until_expiry().num_seconds().max(0);
        let secure = if self.config.cookie_secure {
            "; Secure"
        } else {
            ""
        };
        let http_only = if self.config.cookie_http_only {
            "; HttpOnly"
        } else {
            ""
        };
        let same_site = format!("; SameSite={}", self.config.cookie_same_site.as_str());

        format!(
            "{}={}; Path=/; Max-Age={max_age}{secure}{http_only}{same_site}",
            self.config.cookie_name, session.id
        )
    }

    pub fn build_logout_cookie(&self) -> String {
        let secure = if self.config.cookie_secure {
            "; Secure"
        } else {
            ""
        };
        let http_only = if self.config.cookie_http_only {
            "; HttpOnly"
        } else {
            ""
        };
        let same_site = format!("; SameSite={}", self.config.cookie_same_site.as_str());

        format!(
            "{}=; Path=/; Max-Age=0{secure}{http_only}{same_site}",
            self.config.cookie_name
        )
    }

    pub fn config(&self) -> &SessionConfig {
        &self.config
    }
}

pub fn generate_session_id(length: usize) -> String {
    use rand::Rng;

    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::rng();

    (0..length)
        .map(|_| CHARSET[rng.random_range(0..CHARSET.len())] as char)
        .collect()
}

pub fn extract_session_id_from_cookie(cookie_header: &str, cookie_name: &str) -> Option<String> {
    for cookie in cookie_header.split(';') {
        let cookie = cookie.trim();
        if let Some((name, value)) = cookie.split_once('=') {
            if name.trim() == cookie_name {
                return Some(value.trim().to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;



    #[test]
    fn test_session_touch() {
        let config = SessionConfig::default();
        let user_id = Uuid::new_v4();
        let mut session = Session::new(user_id, &config);
        let original_expires = session.expires_at;

        std::thread::sleep(std::time::Duration::from_millis(10));
        session.touch(config.idle_timeout_minutes);

        assert!(session.expires_at > original_expires);
    }

    #[test]
    fn test_session_revoke() {
        let config = SessionConfig::default();
        let user_id = Uuid::new_v4();
        let mut session = Session::new(user_id, &config);

        assert!(session.is_valid());
        session.revoke();
        assert!(!session.is_valid());
        assert_eq!(session.status, SessionStatus::Revoked);
    }

    #[test]
    fn test_device_info_parsing() {
        let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/120.0.0.0";
        let device = DeviceInfo::from_user_agent(ua);

        assert_eq!(device.os, Some("Windows".into()));
        assert_eq!(device.browser, Some("Chrome".into()));
        assert_eq!(device.device_type, Some("Desktop".into()));
    }

    #[test]
    fn test_device_info_mobile() {
        let ua = "Mozilla/5.0 (Linux; Android 13) AppleWebKit/537.36 Mobile Safari/537.36";
        let device = DeviceInfo::from_user_agent(ua);

        assert_eq!(device.os, Some("Android".into()));
        assert_eq!(device.device_type, Some("Mobile".into()));
    }

    #[test]
    fn test_generate_session_id() {
        let id1 = generate_session_id(32);
        let id2 = generate_session_id(32);

        assert_eq!(id1.len(), 32);
        assert_eq!(id2.len(), 32);
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_extract_session_from_cookie() {
        let cookie = "gb_session=abc123xyz; other=value";
        let session_id = extract_session_id_from_cookie(cookie, "gb_session");

        assert_eq!(session_id, Some("abc123xyz".into()));
    }

    #[test]
    fn test_session_config_defaults() {
        let config = SessionConfig::default();

        assert_eq!(config.idle_timeout_minutes, 30);
        assert_eq!(config.absolute_timeout_hours, 24);
        assert_eq!(config.max_concurrent_sessions, 5);
        assert!(config.cookie_secure);
        assert!(config.cookie_http_only);
    }

    #[test]
    fn test_remember_me_extends_session() {
        let config = SessionConfig::default();
        let user_id = Uuid::new_v4();
        let session_normal = Session::new(user_id, &config);
        let session_remember = Session::new(user_id, &config).with_remember_me(true);

        assert!(session_remember.absolute_expires_at > session_normal.absolute_expires_at);
    }

    #[test]
    fn test_same_site_as_str() {
        assert_eq!(SameSite::Strict.as_str(), "Strict");
        assert_eq!(SameSite::Lax.as_str(), "Lax");
        assert_eq!(SameSite::None.as_str(), "None");
    }

    #[tokio::test]
    async fn test_in_memory_store() {
        let store = InMemorySessionStore::new();
        let config = SessionConfig::default();
        let user_id = Uuid::new_v4();
        let session = Session::new(user_id, &config);
        let session_id = session.id.clone();

        store.create(session.clone()).await.expect("Create failed");

        let retrieved = store.get(&session_id).await.expect("Get failed");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.as_ref().map(|s| &s.id), Some(&session_id));

        store.delete(&session_id).await.expect("Delete failed");
        let deleted = store.get(&session_id).await.expect("Get failed");
        assert!(deleted.is_none());
    }

    #[tokio::test]
    async fn test_session_manager_create() {
        let store = InMemorySessionStore::new();
        let config = SessionConfig::default();
        let manager = SessionManager::new(store, config);
        let user_id = Uuid::new_v4();

        let session = manager
            .create_session(user_id, Some("127.0.0.1".into()), Some("Test Agent"), false)
            .await
            .expect("Create failed");

        assert_eq!(session.user_id, user_id);
        assert!(session.is_valid());
    }

    #[tokio::test]
    async fn test_session_manager_validate() {
        let store = InMemorySessionStore::new();
        let config = SessionConfig::default();
        let manager = SessionManager::new(store, config);
        let user_id = Uuid::new_v4();

        let session = manager
            .create_session(user_id, None, None, false)
            .await
            .expect("Create failed");

        let validated = manager
            .validate_session(&session.id)
            .await
            .expect("Validate failed");
        assert!(validated.is_some());

        let invalid = manager
            .validate_session("nonexistent")
            .await
            .expect("Validate failed");
        assert!(invalid.is_none());
    }

    #[tokio::test]
    async fn test_session_manager_revoke() {
        let store = InMemorySessionStore::new();
        let config = SessionConfig::default();
        let manager = SessionManager::new(store, config);
        let user_id = Uuid::new_v4();

        let session = manager
            .create_session(user_id, None, None, false)
            .await
            .expect("Create failed");

        let revoked = manager
            .revoke_session(&session.id)
            .await
            .expect("Revoke failed");
        assert!(revoked);

        let validated = manager
            .validate_session(&session.id)
            .await
            .expect("Validate failed");
        assert!(validated.is_none());
    }

    #[tokio::test]
    async fn test_concurrent_session_limit() {
        let store = InMemorySessionStore::new();
        let mut config = SessionConfig::default();
        config.max_concurrent_sessions = 2;
        let manager = SessionManager::new(store, config);
        let user_id = Uuid::new_v4();

        let s1 = manager
            .create_session(user_id, None, None, false)
            .await
            .expect("Create failed");
        let s2 = manager
            .create_session(user_id, None, None, false)
            .await
            .expect("Create failed");
        let s3 = manager
            .create_session(user_id, None, None, false)
            .await
            .expect("Create failed");

        let sessions = manager
            .get_user_sessions(user_id)
            .await
            .expect("Get sessions failed");
        assert_eq!(sessions.len(), 2);

        let ids: Vec<_> = sessions.iter().map(|s| s.id.clone()).collect();
        assert!(!ids.contains(&s1.id));
        assert!(ids.contains(&s2.id));
        assert!(ids.contains(&s3.id));
    }
}
