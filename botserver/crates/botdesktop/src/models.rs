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
    /// Owning organization, when known (extracted from the authenticated
    /// user). Kept for tenant scoping of session listings and audit.
    pub org_id: Option<Uuid>,
    /// Workspace branch, when known (extracted from the authenticated user).
    pub branch_id: Option<Uuid>,
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
        Self::with_scope(user_id, None, None, target_host, target_port, client_ip)
    }

    /// Creates a session attributed to a real user with tenant scope.
    pub fn with_scope(
        user_id: Uuid,
        org_id: Option<Uuid>,
        branch_id: Option<Uuid>,
        target_host: String,
        target_port: u16,
        client_ip: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id,
            org_id,
            branch_id,
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

    /// True when the session is owned by the given user (or the user is an
    /// admin operating on any session).
    pub fn is_owned_by(&self, user_id: Uuid) -> bool {
        self.user_id == user_id
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
            user_id: self.user_id,
            target_host: self.target_host.clone(),
            target_port: self.target_port,
            status: self.status.to_string(),
            created_at: self.created_at,
            last_active_at: self.last_active_at,
            bytes_sent: self.bytes_sent,
            bytes_received: self.bytes_received,
            // Client IP is masked so session listings never leak full
            // addresses through the API (audit tables keep the full value).
            client_ip: crate::types::mask_ip(&self.client_ip),
        }
    }
}

/// A single audit entry for a desktop proxy session. Written to the
/// `desktop_connection_log` table on session start and end.
#[derive(Debug, Clone)]
pub struct SessionAuditEvent {
    pub connection_id: Uuid,
    pub user_id: Uuid,
    pub session_id: Uuid,
    pub host: String,
    pub port: u16,
    pub protocol: String,
    pub connected_at: DateTime<Utc>,
    pub disconnected_at: Option<DateTime<Utc>>,
    pub bytes_transferred: i64,
    pub disconnect_reason: Option<String>,
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

impl ConnectionConfig {
    /// Target port allow-list. The desktop proxy exists to relay remote
    /// desktop protocols only (VNC 5900-5999 and RDP 3389); proxying to
    /// arbitrary service ports (databases, caches, control planes) is an
    /// SSRF vector and is rejected.
    pub fn is_target_port_allowed(port: u16) -> bool {
        port == RDP_PORT || (VNC_PORT_MIN..=VNC_PORT_MAX).contains(&port)
    }
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

/// RDP (Microsoft Remote Desktop Protocol).
pub const RDP_PORT: u16 = 3389;
/// VNC (RFB) default port range.
pub const VNC_PORT_MIN: u16 = 5900;
pub const VNC_PORT_MAX: u16 = 5999;

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
    fn test_target_port_allow_list() {
        // VNC range and RDP are allowed.
        assert!(ConnectionConfig::is_target_port_allowed(5900));
        assert!(ConnectionConfig::is_target_port_allowed(5999));
        assert!(ConnectionConfig::is_target_port_allowed(3389));
        // Arbitrary service ports are rejected (SSRF hardening).
        assert!(!ConnectionConfig::is_target_port_allowed(22));
        assert!(!ConnectionConfig::is_target_port_allowed(5432));
        assert!(!ConnectionConfig::is_target_port_allowed(6379));
        assert!(!ConnectionConfig::is_target_port_allowed(80));
        assert!(!ConnectionConfig::is_target_port_allowed(8080));
    }

    #[test]
    fn test_session_ownership() {
        let owner = Uuid::new_v4();
        let other = Uuid::new_v4();
        let s = DesktopSession::new(owner, "10.0.0.1".into(), 5900, "127.0.0.1".into());
        assert!(s.is_owned_by(owner));
        assert!(!s.is_owned_by(other));
    }

    #[test]
    fn test_summary_masks_client_ip() {
        let s = DesktopSession::new(
            Uuid::new_v4(),
            "10.0.0.1".into(),
            5900,
            "192.168.1.99".into(),
        );
        let sum = s.to_summary();
        assert_eq!(sum.user_id, s.user_id);
        assert_eq!(sum.client_ip, "192.x.x.xxx");
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
