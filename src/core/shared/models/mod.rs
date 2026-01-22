use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod core;
pub use self::core::*;

pub mod rbac;
pub use self::rbac::*;

#[cfg(feature = "tasks")]
pub mod tasks;
#[cfg(feature = "tasks")]
pub use self::tasks::*;

pub use super::schema;

// Re-export schema tables for convenience, as they were before
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
