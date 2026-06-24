use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
use tracing::{debug, info};
use uuid::Uuid;

use crate::models::{ConnectionConfig, DesktopSession, SessionStatus};

// ---------------------------------------------------------------------------
// SessionManager
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<Uuid, DesktopSession>>>,
    config: ConnectionConfig,
}

impl SessionManager {
    pub fn new(config: ConnectionConfig) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    // -- Mutating operations ------------------------------------------------

    /// Register a new session. Returns an error if the user has exceeded the
    /// per-user rate limit.
    pub async fn register_session(
        &self,
        session: DesktopSession,
    ) -> Result<DesktopSession, SessionError> {
        let user_id = session.user_id;

        // Rate limit check
        let current_count = self.count_for_user(user_id).await;
        if current_count >= self.config.max_sessions_per_user {
            return Err(SessionError::RateLimitExceeded {
                user_id,
                max: self.config.max_sessions_per_user,
            });
        }

        let mut sessions = self.sessions.write().await;
        sessions.insert(session.id, session.clone());
        info!(
            "Registered session {} for user {} ({} total)",
            session.id,
            user_id,
            sessions
                .values()
                .filter(|s| s.user_id == user_id)
                .count()
        );
        Ok(session)
    }

    /// Remove a session by id. Returns the removed session if it existed.
    pub async fn remove_session(&self, id: Uuid) -> Option<DesktopSession> {
        let mut sessions = self.sessions.write().await;
        let removed = sessions.remove(&id);
        if let Some(ref s) = removed {
            debug!("Removed session {} for user {}", id, s.user_id);
        }
        removed
    }

    /// Mark a session as disconnected and schedule removal.
    pub async fn disconnect_session(&self, id: Uuid) {
        let mut sessions = self.sessions.write().await;
        if let Some(s) = sessions.get_mut(&id) {
            s.status = SessionStatus::Disconnected;
            debug!("Session {} marked disconnected", id);
        }
    }

    /// Touch a session to update last_active_at.
    pub async fn touch_session(&self, id: Uuid) {
        let mut sessions = self.sessions.write().await;
        if let Some(s) = sessions.get_mut(&id) {
            s.touch();
        }
    }

    /// Add bytes to session counters.
    pub async fn add_bytes(&self, id: Uuid, sent: i64, received: i64) {
        let mut sessions = self.sessions.write().await;
        if let Some(s) = sessions.get_mut(&id) {
            s.bytes_sent += sent;
            s.bytes_received += received;
        }
    }

    /// Mark a session as connected and touch it.
    pub async fn mark_connected(&self, id: Uuid) {
        let mut sessions = self.sessions.write().await;
        if let Some(s) = sessions.get_mut(&id) {
            s.status = SessionStatus::Connected;
            s.touch();
        }
    }

    /// Check whether a session is idle.
    pub async fn is_session_idle(&self, id: Uuid) -> bool {
        let sessions = self.sessions.read().await;
        sessions
            .get(&id)
            .map(|s| s.is_idle())
            .unwrap_or(true)
    }

    // -- Query operations ---------------------------------------------------

    pub async fn get_session(&self, id: Uuid) -> Option<DesktopSession> {
        let sessions = self.sessions.read().await;
        sessions.get(&id).cloned()
    }

    pub async fn get_sessions_for_user(&self, user_id: Uuid) -> Vec<DesktopSession> {
        let sessions = self.sessions.read().await;
        sessions
            .values()
            .filter(|s| s.user_id == user_id)
            .cloned()
            .collect()
    }

    pub async fn get_all_sessions(&self) -> Vec<DesktopSession> {
        let sessions = self.sessions.read().await;
        sessions.values().cloned().collect()
    }

    pub async fn count_for_user(&self, user_id: Uuid) -> usize {
        let sessions = self.sessions.read().await;
        sessions
            .values()
            .filter(|s| s.user_id == user_id && s.status != SessionStatus::Disconnected)
            .count()
    }

    /// Check whether a user is within the rate limit.
    pub async fn is_within_rate_limit(&self, user_id: Uuid) -> bool {
        self.count_for_user(user_id).await < self.config.max_sessions_per_user
    }

    pub async fn total_count(&self) -> usize {
        let sessions = self.sessions.read().await;
        sessions.len()
    }

    // -- Cleanup ------------------------------------------------------------

    /// Spawn the periodic cleanup task. Runs every `cleanup_interval_secs`
    /// and removes sessions that are expired or idle.
    pub fn spawn_cleanup(self) -> tokio::task::JoinHandle<()> {
        let interval = self.config.cleanup_interval_secs;
        let idle_minutes = self.config.idle_timeout_minutes;
        let max_hours = self.config.max_lifetime_hours;

        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(tokio::time::Duration::from_secs(interval));
            loop {
                ticker.tick().await;
                let removed = self.run_cleanup(idle_minutes, max_hours).await;
                if removed > 0 {
                    info!("Cleanup: removed {} stale sessions", removed);
                }
            }
        })
    }

    /// Remove expired and idle sessions. Returns count of removed sessions.
    async fn run_cleanup(&self, idle_minutes: i64, max_hours: i64) -> usize {
        let mut sessions = self.sessions.write().await;
        let now = chrono::Utc::now();
        let idle_threshold = now - chrono::Duration::minutes(idle_minutes);
        let lifetime_threshold = now - chrono::Duration::hours(max_hours);

        let to_remove: Vec<Uuid> = sessions
            .iter()
            .filter(|(_, s)| {
                // Expired: older than max lifetime
                s.created_at < lifetime_threshold
                    // Idle: no activity within idle timeout (only if connected/idle)
                    || (s.status != SessionStatus::Disconnected
                        && s.last_active_at < idle_threshold)
            })
            .map(|(id, s)| {
                debug!(
                    "Stale session {} (user {}, status {}, age={:?}, last_active={:?})",
                    id,
                    s.user_id,
                    s.status,
                    now - s.created_at,
                    now - s.last_active_at,
                );
                *id
            })
            .collect();

        let count = to_remove.len();
        for id in &to_remove {
            sessions.remove(id);
        }

        count
    }
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("rate limit exceeded for user {user_id}: max {max} concurrent sessions")]
    RateLimitExceeded { user_id: Uuid, max: usize },

    #[error("session {0} not found")]
    NotFound(Uuid),
}

impl SessionError {
    pub fn status_code(&self) -> u16 {
        match self {
            Self::RateLimitExceeded { .. } => 429,
            Self::NotFound(_) => 404,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ConnectionConfig;

    fn test_config() -> ConnectionConfig {
        ConnectionConfig {
            max_sessions_per_user: 2,
            idle_timeout_minutes: 1,
            max_lifetime_hours: 1,
            cleanup_interval_secs: 10,
        }
    }

    fn make_user_session(user_id: Uuid) -> DesktopSession {
        DesktopSession::new(user_id, "10.0.0.1".into(), 5900, "192.168.1.1".into())
    }

    #[tokio::test]
    async fn test_register_and_get() {
        let mgr = SessionManager::new(test_config());
        let user = Uuid::new_v4();
        let session = make_user_session(user);
        let id = session.id;

        mgr.register_session(session).await.unwrap();
        let got = mgr.get_session(id).await;
        assert!(got.is_some());
        assert_eq!(got.unwrap().user_id, user);
    }

    #[tokio::test]
    async fn test_rate_limit() {
        let mgr = SessionManager::new(test_config());
        let user = Uuid::new_v4();

        mgr.register_session(make_user_session(user)).await.unwrap();
        mgr.register_session(make_user_session(user)).await.unwrap();

        let result = mgr.register_session(make_user_session(user)).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SessionError::RateLimitExceeded { .. }));
    }

    #[tokio::test]
    async fn test_remove_session() {
        let mgr = SessionManager::new(test_config());
        let session = make_user_session(Uuid::new_v4());
        let id = session.id;

        mgr.register_session(session).await.unwrap();
        let removed = mgr.remove_session(id).await;
        assert!(removed.is_some());
        assert!(mgr.get_session(id).await.is_none());
    }

    #[tokio::test]
    async fn test_is_within_rate_limit() {
        let mgr = SessionManager::new(test_config());
        let user = Uuid::new_v4();

        assert!(mgr.is_within_rate_limit(user).await);
        mgr.register_session(make_user_session(user)).await.unwrap();
        assert!(mgr.is_within_rate_limit(user).await);
        mgr.register_session(make_user_session(user)).await.unwrap();
        assert!(!mgr.is_within_rate_limit(user).await);
    }

    #[tokio::test]
    async fn test_get_sessions_for_user() {
        let mgr = SessionManager::new(test_config());
        let user = Uuid::new_v4();
        let other = Uuid::new_v4();

        mgr.register_session(make_user_session(user)).await.unwrap();
        mgr.register_session(make_user_session(other)).await.unwrap();

        let user_sessions = mgr.get_sessions_for_user(user).await;
        assert_eq!(user_sessions.len(), 1);
    }

    #[tokio::test]
    async fn test_disconnected_not_counted() {
        let mgr = SessionManager::new(test_config());
        let user = Uuid::new_v4();
        let s = make_user_session(user);
        let id = s.id;

        mgr.register_session(s).await.unwrap();
        mgr.disconnect_session(id).await;

        assert_eq!(mgr.count_for_user(user).await, 0);
        assert!(mgr.is_within_rate_limit(user).await);
    }

    #[tokio::test]
    async fn test_add_bytes() {
        let mgr = SessionManager::new(test_config());
        let session = make_user_session(Uuid::new_v4());
        let id = session.id;

        mgr.register_session(session).await.unwrap();
        mgr.add_bytes(id, 1024, 2048).await;

        let s = mgr.get_session(id).await.unwrap();
        assert_eq!(s.bytes_sent, 1024);
        assert_eq!(s.bytes_received, 2048);
    }
}
