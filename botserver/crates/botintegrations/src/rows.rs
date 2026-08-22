use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use crate::models::{ConnectionEvent, ConnectionRecord};

/// Database row of an integration connection. Contains `vault_path` - this
/// type never crosses the API boundary; convert through [`Self::into_record`].
#[derive(Debug, Clone, QueryableByName)]
pub struct ConnectionRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub provider_slug: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub display_name: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub auth_kind: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub status: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub vault_path: String,
    #[diesel(sql_type = diesel::sql_types::Jsonb)]
    pub granted_scopes: serde_json::Value,
    #[diesel(sql_type = diesel::sql_types::Jsonb)]
    pub configuration: serde_json::Value,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub provider_account_id: Option<String>,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub credential_version: i64,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
    pub expires_at: Option<DateTime<Utc>>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
    pub last_refreshed_at: Option<DateTime<Utc>>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
    pub last_tested_at: Option<DateTime<Utc>>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub last_test_status: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub last_error_code: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
    pub revoked_at: Option<DateTime<Utc>>,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    pub created_at: DateTime<Utc>,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    pub updated_at: DateTime<Utc>,
}

impl ConnectionRow {
    /// Converts the internal row into the public DTO, dropping `vault_path`
    /// and every other server-only field.
    pub fn into_record(self) -> ConnectionRecord {
        let granted_scopes = match self.granted_scopes {
            serde_json::Value::Array(items) => items
                .into_iter()
                .filter_map(|item| item.as_str().map(str::to_string))
                .collect(),
            _ => Vec::new(),
        };
        ConnectionRecord {
            id: self.id,
            provider_slug: self.provider_slug,
            display_name: self.display_name,
            auth_kind: self.auth_kind,
            status: self.status,
            granted_scopes,
            configuration: self.configuration,
            provider_account_id: self.provider_account_id,
            credential_version: self.credential_version,
            expires_at: self.expires_at,
            last_refreshed_at: self.last_refreshed_at,
            last_tested_at: self.last_tested_at,
            last_test_status: self.last_test_status,
            revoked_at: self.revoked_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

/// Internal database row of an audit event.
#[derive(Debug, Clone, QueryableByName)]
pub struct EventRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)]
    pub connection_id: Option<Uuid>,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub event_type: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub outcome: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub risk_level: String,
    #[diesel(sql_type = diesel::sql_types::Jsonb)]
    pub metadata: serde_json::Value,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    pub created_at: DateTime<Utc>,
}

impl From<EventRow> for ConnectionEvent {
    fn from(row: EventRow) -> Self {
        Self {
            id: row.id,
            connection_id: row.connection_id,
            event_type: row.event_type,
            outcome: row.outcome,
            risk_level: row.risk_level,
            metadata: row.metadata,
            created_at: row.created_at,
        }
    }
}
