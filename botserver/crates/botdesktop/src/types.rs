use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Requests
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct ConnectionCreateRequest {
    pub target_host: String,
    pub target_port: u16,
    /// Remote-desktop protocol (`vnc` or `rdp`); defaults to `vnc`.
    #[serde(default = "default_protocol")]
    pub protocol: String,
    /// Optional RDP target password, vaulted server-side and never persisted
    /// in the database or returned to the client.
    #[serde(default)]
    pub password: Option<String>,
    /// Optional RDP domain for NLA authentication.
    #[serde(default)]
    pub domain: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HealthCheckRequest {
    pub host: String,
    pub port: u16,
}

fn default_protocol() -> String {
    "vnc".to_string()
}

// ---------------------------------------------------------------------------
// Responses
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionSummary {
    pub id: Uuid,
    /// Owning user (never the nil UUID in authenticated flows).
    pub user_id: Uuid,
    pub target_host: String,
    pub target_port: u16,
    /// Remote-desktop protocol (`vnc` or `rdp`).
    pub protocol: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub last_active_at: DateTime<Utc>,
    pub bytes_sent: i64,
    pub bytes_received: i64,
    /// Client IP (masked for logs; full value only for the owning user).
    pub client_ip: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    pub reachable: bool,
    pub latency_ms: Option<u64>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn err(msg: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(msg.into()),
        }
    }
}

// ---------------------------------------------------------------------------
// WebSocket proxy messages (client ↔ server)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum WsProxyMessage {
    /// Binary payload relayed to TCP target
    #[serde(rename = "data")]
    Data(String),

    /// Client signals the proxy should close
    #[serde(rename = "close")]
    Close,

    /// Server notifies client of an error
    #[serde(rename = "error")]
    Error(String),

    /// Server sends connection metadata after handshake
    #[serde(rename = "connected")]
    Connected(ConnectionMeta),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionMeta {
    pub connection_id: Uuid,
    pub target_host: String,
    pub target_port: u16,
    pub connected_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Mask an IP address for logging — only first octet exposed.
/// 10.16.164.222 → 10.x.x.xxx
pub fn mask_ip(ip: &str) -> String {
    let parts: Vec<&str> = ip.split('.').collect();
    if parts.len() == 4 {
        format!("{}.x.x.xxx", parts[0])
    } else {
        "x.x.x.x".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_ip_v4() {
        assert_eq!(mask_ip("10.16.164.222"), "10.x.x.xxx");
        assert_eq!(mask_ip("192.168.1.1"), "192.x.x.xxx");
        assert_eq!(mask_ip("127.0.0.1"), "127.x.x.xxx");
    }

    #[test]
    fn test_mask_ip_short() {
        assert_eq!(mask_ip("not-an-ip"), "x.x.x.x");
    }

    #[test]
    fn test_api_response_ok() {
        let r = ApiResponse::ok(42i32);
        assert!(r.success);
        assert_eq!(r.data, Some(42));
        assert!(r.error.is_none());
    }

    #[test]
    fn test_api_response_err() {
        let r: ApiResponse<i32> = ApiResponse::err("boom");
        assert!(!r.success);
        assert!(r.data.is_none());
        assert_eq!(r.error.as_deref(), Some("boom"));
    }

    #[test]
    fn test_ws_proxy_message_roundtrip() {
        let msg = WsProxyMessage::Connected(ConnectionMeta {
            connection_id: Uuid::nil(),
            target_host: "localhost".into(),
            target_port: 5900,
            connected_at: Utc::now(),
        });
        let json = serde_json::to_string(&msg).unwrap();
        let back: WsProxyMessage = serde_json::from_str(&json).unwrap();
        assert!(matches!(back, WsProxyMessage::Connected(_)));
    }
}
