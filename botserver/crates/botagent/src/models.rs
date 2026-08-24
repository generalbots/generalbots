//! Row models and request bodies for the agent endpoints.

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::agent_sessions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AgentSessionRow {
    pub id: Uuid,
    pub session_id: String,
    pub user_id: Uuid,
    pub org_id: Option<Uuid>,
    pub branch_id: Option<Uuid>,
    pub bot_id: Uuid,
    pub vm_name: String,
    pub status: String,
    pub last_active_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::agent_snapshots)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AgentSnapshotRow {
    pub id: Uuid,
    pub agent_session_id: Uuid,
    pub label: Option<String>,
    pub incus_snapshot: String,
    pub size_bytes: Option<i64>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::org_api_keys)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct OrgApiKeyRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub name: String,
    pub key_hash: String,
    pub key_prefix: String,
    pub scopes: serde_json::Value,
    pub last_used_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::sandbox_runs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SandboxRunRow {
    pub id: Uuid,
    pub org_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub language: String,
    pub status: String,
    pub exit_code: Option<i32>,
    pub stdout_ref: Option<String>,
    pub stderr_ref: Option<String>,
    pub duration_ms: Option<i32>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct ModeBody {
    pub session_id: String,
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct SnapshotCreateBody {
    pub label: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ExecBody {
    pub language: String,
    pub code: String,
    pub timeout_ms: Option<u64>,
    pub files: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
pub struct KeyCreateBody {
    pub name: String,
    pub scopes: Vec<String>,
    /// Required when an admin JWT (instead of an org key) creates a key.
    pub org_id: Option<Uuid>,
}
