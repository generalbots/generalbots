//! Database models and diesel-specific types
//!
//! This module contains diesel ORM models and database-specific types.
//! Common API types (BotResponse, UserMessage, etc.) are now in botlib.

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Re-export schema for backward compatibility (crate::shared::models::schema::*)
// This allows `use crate::shared::models::schema::table_name::dsl::*;` to work
pub use super::schema;

// Also re-export individual tables at this level for convenience
pub use super::schema::{
    basic_tools, bot_configuration, bot_memories, bots, clicks, email_drafts, email_folders,
    kb_collections, kb_documents, message_history, organizations, session_tool_associations,
    system_automations, tasks, user_email_accounts, user_kb_associations, user_login_tokens,
    user_preferences, user_sessions, users,
};

// Re-export common types from botlib for convenience
pub use botlib::message_types::MessageType;
pub use botlib::models::{ApiResponse, Attachment, BotResponse, Session, Suggestion, UserMessage};

/// Trigger kinds for automations
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TriggerKind {
    Scheduled = 0,
    TableUpdate = 1,
    TableInsert = 2,
    TableDelete = 3,
    Webhook = 4,
}

impl TriggerKind {
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::Scheduled),
            1 => Some(Self::TableUpdate),
            2 => Some(Self::TableInsert),
            3 => Some(Self::TableDelete),
            4 => Some(Self::Webhook),
            _ => None,
        }
    }
}

/// Automation database model
#[derive(Debug, Queryable, Serialize, Deserialize, Identifiable)]
#[diesel(table_name = system_automations)]
pub struct Automation {
    pub id: Uuid,
    pub bot_id: Uuid,
    pub kind: i32,
    pub target: Option<String>,
    pub schedule: Option<String>,
    pub param: String,
    pub is_active: bool,
    pub last_triggered: Option<DateTime<Utc>>,
}

/// User session database model
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable, Selectable)]
#[diesel(table_name = user_sessions)]
pub struct UserSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub bot_id: Uuid,
    pub title: String,
    pub context_data: serde_json::Value,
    pub current_tool: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Bot memory storage model
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable, Insertable)]
#[diesel(table_name = bot_memories)]
pub struct BotMemory {
    pub id: Uuid,
    pub bot_id: Uuid,
    pub key: String,
    pub value: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// User database model
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = users)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub is_active: bool,
    pub is_admin: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Bot database model
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = bots)]
pub struct Bot {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub llm_provider: String,
    pub llm_config: serde_json::Value,
    pub context_provider: String,
    pub context_config: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_active: Option<bool>,
    pub tenant_id: Option<Uuid>,
}

/// Organization database model
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = organizations)]
#[diesel(primary_key(org_id))]
pub struct Organization {
    pub org_id: Uuid,
    pub name: String,
    pub slug: String,
    pub created_at: DateTime<Utc>,
}

/// Message history database model
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = message_history)]
pub struct MessageHistory {
    pub id: Uuid,
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub role: i32,
    pub content_encrypted: String,
    pub message_type: i32,
    pub message_index: i64,
    pub created_at: DateTime<Utc>,
}

/// Bot configuration database model
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = bot_configuration)]
pub struct BotConfiguration {
    pub id: Uuid,
    pub bot_id: Uuid,
    pub config_key: String,
    pub config_value: String,
    pub is_encrypted: bool,
    pub config_type: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// User login token database model
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = user_login_tokens)]
pub struct UserLoginToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub last_used: DateTime<Utc>,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub is_active: bool,
}

/// User preferences database model
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = user_preferences)]
pub struct UserPreference {
    pub id: Uuid,
    pub user_id: Uuid,
    pub preference_key: String,
    pub preference_value: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Click tracking database model
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = clicks)]
pub struct Click {
    pub id: Uuid,
    pub campaign_id: String,
    pub email: String,
    pub updated_at: DateTime<Utc>,
}

/// Task database model
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = tasks)]
pub struct Task {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: String,
    pub assignee_id: Option<Uuid>,
    pub reporter_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub due_date: Option<DateTime<Utc>>,
    pub tags: Vec<String>,
    pub dependencies: Vec<Uuid>,
    pub estimated_hours: Option<f64>,
    pub actual_hours: Option<f64>,
    pub progress: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// New task for insertion
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = tasks)]
pub struct NewTask {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: String,
    pub assignee_id: Option<Uuid>,
    pub reporter_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub due_date: Option<DateTime<Utc>>,
    pub tags: Vec<String>,
    pub dependencies: Vec<Uuid>,
    pub estimated_hours: Option<f64>,
    pub progress: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trigger_kind_conversion() {
        assert_eq!(TriggerKind::from_i32(0), Some(TriggerKind::Scheduled));
        assert_eq!(TriggerKind::from_i32(4), Some(TriggerKind::Webhook));
        assert_eq!(TriggerKind::from_i32(99), None);
    }
}
