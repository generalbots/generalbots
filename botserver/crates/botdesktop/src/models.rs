use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::ConnectionSummary;

// ---------------------------------------------------------------------------
// Session status
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    Connecting,
    Connected,
    Idle,
    Disconnected,
    Error,
}

impl std::fmt::Display for SessionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Connecting => write!(f, "connecting"),
            Self::Connected => write!(f, "connected"),
            Self::Idle => write!(f, "idle"),
            Self::Disconnected => write!(f, "disconnected"),
            Self::Error => write!(f, "error"),
        }
    }
}

// ---------------------------------------------------------------------------
// Desktop session (in-memory representation)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct DesktopSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub target_host: String,
    pub target_port: u16,
    pub status: SessionStatus,
    pub created_at: DateTime<Utc>,
    pub last_active_at: DateTime<Utc>,
    pub bytes_sent: i64,
    pub bytes_received: i64,
    pub client_ip: String,
}

impl DesktopSession {
    pub fn new(user_id: Uuid, target_host: String, target_port: u16, client_ip: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id,
            target_host,
            target_port,
            status: SessionStatus::Connecting,
            created_at: now,
            last_active_at: now,
            bytes_sent: 0,
            bytes_received: 0,
            client_ip,
        }
    }

    /// True if the session exceeds the maximum lifetime (4 hours).
    pub fn is_expired(&self) -> bool {
        Utc::now() - self.created_at > Duration::hours(MAX_LIFETIME_HOURS)
    }

    /// True if the session has been idle longer than the idle timeout (30 min).
    pub fn is_idle(&self) -> bool {
        Utc::now() - self.last_active_at > Duration::minutes(IDLE_TIMEOUT_MINUTES)
    }

    /// Touch last_active_at to current time.
    pub fn touch(&mut self) {
        self.last_active_at = Utc::now();
        if self.status == SessionStatus::Idle {
            self.status = SessionStatus::Connected;
        }
    }

    pub fn to_summary(&self) -> ConnectionSummary {
        ConnectionSummary {
            id: self.id,
            target_host: self.target_host.clone(),
            target_port: self.target_port,
            status: self.status.to_string(),
            created_at: self.created_at,
            last_active_at: self.last_active_at,
            bytes_sent: self.bytes_sent,
            bytes_received: self.bytes_received,
        }
    }
}

// ---------------------------------------------------------------------------
// Connection config (limits & timeouts)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ConnectionConfig {
    /// Maximum sessions per user.
    pub max_sessions_per_user: usize,
    /// Idle timeout in minutes (30).
    pub idle_timeout_minutes: i64,
    /// Maximum session lifetime in hours (4).
    pub max_lifetime_hours: i64,
    /// Cleanup interval in seconds (60).
    pub cleanup_interval_secs: u64,
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            max_sessions_per_user: MAX_SESSIONS_PER_USER,
            idle_timeout_minutes: IDLE_TIMEOUT_MINUTES,
            max_lifetime_hours: MAX_LIFETIME_HOURS,
            cleanup_interval_secs: CLEANUP_INTERVAL_SECS,
        }
    }
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

pub const MAX_SESSIONS_PER_USER: usize = 3;
pub const IDLE_TIMEOUT_MINUTES: i64 = 30;
pub const MAX_LIFETIME_HOURS: i64 = 4;
pub const CLEANUP_INTERVAL_SECS: u64 = 60;

#[cfg(test)]
mod tests {
    use super::*;

    fn make_session(minutes_ago: i64) -> DesktopSession {
        let mut s = DesktopSession::new(
            Uuid::new_v4(),
            "10.0.0.1".into(),
            5900,
            "192.168.1.100".into(),
        );
        s.last_active_at = Utc::now() - Duration::minutes(minutes_ago);
        s.status = SessionStatus::Connected;
        s
    }

    #[test]
    fn test_new_session_is_connecting() {
        let s = DesktopSession::new(Uuid::new_v4(), "host".into(), 22, "ip".into());
        assert_eq!(s.status, SessionStatus::Connecting);
        assert_eq!(s.bytes_sent, 0);
    }

    #[test]
    fn test_expired_after_4h() {
        let mut s = make_session(0);
        s.created_at = Utc::now() - Duration::hours(5);
        assert!(s.is_expired());
    }

    #[test]
    fn test_not_expired_within_4h() {
        let s = make_session(10);
        assert!(!s.is_expired());
    }

    #[test]
    fn test_idle_after_30m() {
        let s = make_session(35);
        assert!(s.is_idle());
    }

    #[test]
    fn test_not_idle_within_30m() {
        let s = make_session(10);
        assert!(!s.is_idle());
    }

    #[test]
    fn test_touch_resets_idle() {
        let mut s = make_session(35);
        s.status = SessionStatus::Idle;
        s.touch();
        assert!(!s.is_idle());
        assert_eq!(s.status, SessionStatus::Connected);
    }

    #[test]
    fn test_default_config() {
        let cfg = ConnectionConfig::default();
        assert_eq!(cfg.max_sessions_per_user, 3);
        assert_eq!(cfg.idle_timeout_minutes, 30);
        assert_eq!(cfg.max_lifetime_hours, 4);
        assert_eq!(cfg.cleanup_interval_secs, 60);
    }

    #[test]
    fn test_status_display() {
        assert_eq!(SessionStatus::Connected.to_string(), "connected");
        assert_eq!(SessionStatus::Error.to_string(), "error");
    }

    #[test]
    fn test_to_summary() {
        let s = make_session(5);
        let sum = s.to_summary();
        assert_eq!(sum.id, s.id);
        assert_eq!(sum.target_port, 5900);
    }
}
