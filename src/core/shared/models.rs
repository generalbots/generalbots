use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use super::schema;

pub use super::schema::{
    basic_tools, bot_configuration, bot_memories, bots, clicks, distribution_lists,
    email_auto_responders, email_drafts, email_folders, email_label_assignments, email_labels,
    email_rules, email_signatures, email_templates, global_email_signatures, kb_collections,
    kb_documents, message_history, organizations, rbac_group_roles, rbac_groups,
    rbac_permissions, rbac_role_permissions, rbac_roles, rbac_user_groups, rbac_user_roles,
    scheduled_emails, session_tool_associations, shared_mailbox_members, shared_mailboxes,
    system_automations, tasks, user_email_accounts, user_kb_associations, user_login_tokens,
    user_preferences, user_sessions, users,
};

pub use botlib::message_types::MessageType;
pub use botlib::models::{ApiResponse, Attachment, BotResponse, Session, Suggestion, UserMessage};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerKind {
    Scheduled = 0,
    TableUpdate = 1,
    TableInsert = 2,
    TableDelete = 3,
    Webhook = 4,
    EmailReceived = 5,
    FolderChange = 6,
}

impl TriggerKind {
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::Scheduled),
            1 => Some(Self::TableUpdate),
            2 => Some(Self::TableInsert),
            3 => Some(Self::TableDelete),
            4 => Some(Self::Webhook),
            5 => Some(Self::EmailReceived),
            6 => Some(Self::FolderChange),
            _ => None,
        }
    }
}

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

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = organizations)]
#[diesel(primary_key(org_id))]
pub struct Organization {
    pub org_id: Uuid,
    pub name: String,
    pub slug: String,
    pub created_at: DateTime<Utc>,
}

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

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = clicks)]
pub struct Click {
    pub id: Uuid,
    pub campaign_id: String,
    pub email: String,
    pub updated_at: DateTime<Utc>,
}

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

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable, Selectable)]
#[diesel(table_name = rbac_roles)]
pub struct RbacRole {
    pub id: Uuid,
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub is_system: bool,
    pub is_active: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = rbac_roles)]
pub struct NewRbacRole {
    pub id: Uuid,
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub is_system: bool,
    pub is_active: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable, Selectable)]
#[diesel(table_name = rbac_groups)]
pub struct RbacGroup {
    pub id: Uuid,
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub parent_group_id: Option<Uuid>,
    pub is_active: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = rbac_groups)]
pub struct NewRbacGroup {
    pub id: Uuid,
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub parent_group_id: Option<Uuid>,
    pub is_active: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable, Selectable)]
#[diesel(table_name = rbac_permissions)]
pub struct RbacPermission {
    pub id: Uuid,
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub resource_type: String,
    pub action: String,
    pub category: String,
    pub is_system: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = rbac_permissions)]
pub struct NewRbacPermission {
    pub id: Uuid,
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub resource_type: String,
    pub action: String,
    pub category: String,
    pub is_system: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable, Associations)]
#[diesel(table_name = rbac_role_permissions)]
#[diesel(belongs_to(RbacRole, foreign_key = role_id))]
#[diesel(belongs_to(RbacPermission, foreign_key = permission_id))]
pub struct RbacRolePermission {
    pub id: Uuid,
    pub role_id: Uuid,
    pub permission_id: Uuid,
    pub granted_by: Option<Uuid>,
    pub granted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = rbac_role_permissions)]
pub struct NewRbacRolePermission {
    pub id: Uuid,
    pub role_id: Uuid,
    pub permission_id: Uuid,
    pub granted_by: Option<Uuid>,
    pub granted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable, Associations)]
#[diesel(table_name = rbac_user_roles)]
#[diesel(belongs_to(User, foreign_key = user_id))]
#[diesel(belongs_to(RbacRole, foreign_key = role_id))]
pub struct RbacUserRole {
    pub id: Uuid,
    pub user_id: Uuid,
    pub role_id: Uuid,
    pub granted_by: Option<Uuid>,
    pub granted_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = rbac_user_roles)]
pub struct NewRbacUserRole {
    pub id: Uuid,
    pub user_id: Uuid,
    pub role_id: Uuid,
    pub granted_by: Option<Uuid>,
    pub granted_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable, Associations)]
#[diesel(table_name = rbac_user_groups)]
#[diesel(belongs_to(User, foreign_key = user_id))]
#[diesel(belongs_to(RbacGroup, foreign_key = group_id))]
pub struct RbacUserGroup {
    pub id: Uuid,
    pub user_id: Uuid,
    pub group_id: Uuid,
    pub added_by: Option<Uuid>,
    pub added_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = rbac_user_groups)]
pub struct NewRbacUserGroup {
    pub id: Uuid,
    pub user_id: Uuid,
    pub group_id: Uuid,
    pub added_by: Option<Uuid>,
    pub added_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable, Associations)]
#[diesel(table_name = rbac_group_roles)]
#[diesel(belongs_to(RbacGroup, foreign_key = group_id))]
#[diesel(belongs_to(RbacRole, foreign_key = role_id))]
pub struct RbacGroupRole {
    pub id: Uuid,
    pub group_id: Uuid,
    pub role_id: Uuid,
    pub granted_by: Option<Uuid>,
    pub granted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = rbac_group_roles)]
pub struct NewRbacGroupRole {
    pub id: Uuid,
    pub group_id: Uuid,
    pub role_id: Uuid,
    pub granted_by: Option<Uuid>,
    pub granted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserWithRoles {
    pub user: User,
    pub roles: Vec<RbacRole>,
    pub groups: Vec<RbacGroup>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectivePermissions {
    pub user_id: Uuid,
    pub permissions: Vec<RbacPermission>,
    pub roles: Vec<RbacRole>,
    pub groups: Vec<RbacGroup>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleAssignment {
    pub role_id: Uuid,
    pub role_name: String,
    pub source: RoleSource,
    pub granted_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RoleSource {
    Direct,
    Group(Uuid),
}
