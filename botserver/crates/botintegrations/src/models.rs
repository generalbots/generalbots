use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

pub use crate::storage::*;

/// Incoming payload for connection creation. The `secrets` member is a
/// transient carrier only: it is written to Vault and never persisted to
/// the database nor echoed back in any response.
#[derive(Debug, Clone)]
pub struct NewConnection {
    pub provider_slug: String,
    pub display_name: String,
    pub auth_kind: String,
    pub secrets: serde_json::Value,
    pub configuration: serde_json::Value,
    pub granted_scopes: Vec<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Public representation of an integration connection. Deliberately omits
/// `vault_path` and any credential material - those never leave the server.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ConnectionRecord {
    pub id: Uuid,
    pub provider_slug: String,
    pub display_name: String,
    pub auth_kind: String,
    pub status: String,
    pub granted_scopes: Vec<String>,
    pub configuration: serde_json::Value,
    pub provider_account_id: Option<String>,
    pub credential_version: i64,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_refreshed_at: Option<DateTime<Utc>>,
    pub last_tested_at: Option<DateTime<Utc>>,
    pub last_test_status: Option<String>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Public audit event for a connection.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ConnectionEvent {
    pub id: Uuid,
    pub connection_id: Option<Uuid>,
    pub event_type: String,
    pub outcome: String,
    pub risk_level: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_record() -> ConnectionRecord {
        ConnectionRecord {
            id: Uuid::from_u128(0x11),
            provider_slug: "github".to_string(),
            display_name: "GitHub".to_string(),
            auth_kind: "access_key".to_string(),
            status: "active".to_string(),
            granted_scopes: vec!["repo:read".to_string()],
            configuration: serde_json::json!({ "region": "us-east-1" }),
            provider_account_id: Some("acct-42".to_string()),
            credential_version: 1,
            expires_at: None,
            last_refreshed_at: None,
            last_tested_at: None,
            last_test_status: Some("unverified".to_string()),
            revoked_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn record_serialization_never_exposes_vault_or_secret_fields() {
        let serialized = serde_json::to_string(&sample_record()).expect("serialization in tests");
        assert!(!serialized.contains("vault_path"));
        assert!(!serialized.contains("secret"));
        assert!(!serialized.contains("token"));
    }

    #[test]
    fn event_serialization_never_exposes_sensitive_material() {
        let event = ConnectionEvent {
            id: Uuid::from_u128(0x22),
            connection_id: Some(Uuid::from_u128(0x33)),
            event_type: "connection.rotated".to_string(),
            outcome: "ok".to_string(),
            risk_level: "medium".to_string(),
            metadata: serde_json::json!({ "reason": "scheduled rotation" }),
            created_at: Utc::now(),
        };
        let serialized = serde_json::to_string(&event).expect("serialization in tests");
        assert!(!serialized.contains("secret"));
        assert!(!serialized.contains("password"));
        assert!(serialized.contains("connection.rotated"));
    }
}
