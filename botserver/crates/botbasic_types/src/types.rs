use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::user_sessions)]
pub struct UserSession {
    pub id: Uuid,
    pub user_id: String,
    pub bot_id: Uuid,
    pub title: String,
    pub answer_mode: i32,
    pub context_data: serde_json::Value,
    pub current_tool: Option<String>,
    pub message_count: i32,
    pub total_tokens: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub tenant_id: Option<Uuid>,
    pub active_email_account_id: Option<Uuid>,
    pub branch_id: Uuid,
    pub session_id: Option<String>,
    pub data: Option<serde_json::Value>,
    pub expires_at: Option<DateTime<Utc>>,
    /// #1682 — locked conversations (6.8.7-chat-lock).
    pub locked: bool,
    pub lock_pin_hash: Option<String>,
}

impl UserSession {
    /// #2547 — tenant scope for suggestion/context cache keys.
    ///
    /// This Diesel projection has no dedicated `org_id` column; the canonical
    /// `botlib::models::UserSession` owns the typed field and the `From` impl
    /// below carries it in the existing nullable `data` JSON column. Old rows
    /// have no `org_id` there and correctly degrade to `None` (legacy keys).
    pub fn org_id(&self) -> Option<Uuid> {
        self.data
            .as_ref()?
            .get("org_id")?
            .as_str()
            .and_then(|value| Uuid::parse_str(value).ok())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = crate::schema::bot_memories)]
pub struct BotMemory {
    pub id: Uuid,
    pub bot_id: Uuid,
    pub key: String,
    pub value: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// NOT NULL since `6.5.23-branch-scope-cleanup`; every insert must supply it.
    pub branch_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotSharedMemory {
    pub id: Uuid,
    pub bot_id: Uuid,
    pub key: String,
    pub value: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerKind(pub String);

impl TriggerKind {
    pub fn webhook() -> Self { Self("webhook".to_string()) }
    pub fn email_received() -> Self { Self("email_received".to_string()) }
    pub fn schedule() -> Self { Self("schedule".to_string()) }
    pub fn database_change() -> Self { Self("database_change".to_string()) }
    pub fn manual() -> Self { Self("manual".to_string()) }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExecution {
    pub id: Uuid,
    pub bot_id: Uuid,
    pub workflow_name: String,
    pub current_step: i32,
    pub state_json: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowEvent {
    pub id: Uuid,
    pub workflow_id: Option<Uuid>,
    pub event_name: String,
    pub event_data_json: Option<String>,
    pub processed: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub attachment_type: String,
    pub url: String,
    pub mime_type: Option<String>,
    pub filename: Option<String>,
    pub size: Option<u64>,
}

pub use botlib::db_pool::DbPool;

impl From<botlib::models::UserSession> for UserSession {
    fn from(u: botlib::models::UserSession) -> Self {
        Self {
            id: u.id,
            user_id: u.user_id.to_string(),
            bot_id: u.bot_id,
            title: u.title,
            answer_mode: 0,
            context_data: u.context_data,
            current_tool: u.current_tool,
            message_count: 0,
            total_tokens: 0,
            created_at: u.created_at,
            updated_at: u.updated_at,
            last_activity: u.updated_at,
            tenant_id: None,
            active_email_account_id: None,
            branch_id: u.branch_id,
            session_id: None,
            data: u.org_id.map(|org| serde_json::json!({ "org_id": org.to_string() })),
            expires_at: None,
            locked: false,
            lock_pin_hash: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session_with_data(data: Option<serde_json::Value>) -> UserSession {
        let now = Utc::now();
        UserSession {
            id: Uuid::new_v4(),
            user_id: "user".to_string(),
            bot_id: Uuid::new_v4(),
            title: String::new(),
            answer_mode: 0,
            context_data: serde_json::json!({}),
            current_tool: None,
            message_count: 0,
            total_tokens: 0,
            created_at: now,
            updated_at: now,
            last_activity: now,
            tenant_id: None,
            active_email_account_id: None,
            branch_id: Uuid::nil(),
            session_id: None,
            data,
            expires_at: None,
            locked: false,
            lock_pin_hash: None,
        }
    }

    /// #2547 — rows created before the org field existed have no `org_id` in
    /// `data`. They must degrade to `None` (legacy cache key), never panic.
    #[test]
    fn org_id_is_none_for_pre_field_sessions() {
        assert_eq!(session_with_data(None).org_id(), None);
        assert_eq!(session_with_data(Some(serde_json::json!({}))).org_id(), None);
        assert_eq!(
            session_with_data(Some(serde_json::json!({ "org_id": "not-a-uuid" }))).org_id(),
            None
        );
    }

    #[test]
    fn org_id_is_read_from_data_when_present() {
        let org = Uuid::new_v4();
        let session = session_with_data(Some(serde_json::json!({ "org_id": org.to_string() })));
        assert_eq!(session.org_id(), Some(org));
    }
}
