
pub mod core;
pub use self::core::*;

pub mod rbac;
pub use self::rbac::*;

#[cfg(feature = "tasks")]
pub mod task_models;
#[cfg(feature = "tasks")]
pub use self::task_models::*;

pub use super::schema;

// Re-export core schema tables
pub use super::schema::{
    basic_tools, bot_configuration, bot_memories, bots, clicks,
    message_history, organizations, rbac_group_roles, rbac_groups,
    rbac_permissions, rbac_role_permissions, rbac_roles, rbac_user_groups, rbac_user_roles,
    session_tool_associations, system_automations, user_login_tokens,
    user_preferences, user_sessions, users,
};

// Re-export feature-gated schema tables
#[cfg(feature = "tasks")]
pub use super::schema::tasks;

#[cfg(feature = "mail")]
pub use super::schema::{
    distribution_lists, email_auto_responders, email_drafts, email_folders,
    email_label_assignments, email_labels, email_rules, email_signatures,
    email_templates, global_email_signatures, scheduled_emails,
    shared_mailbox_members, shared_mailboxes, user_email_accounts,
};

#[cfg(feature = "people")]
pub use super::schema::{
    crm_accounts, crm_activities, crm_contacts, crm_leads, crm_notes,
    crm_opportunities, crm_pipeline_stages, people, people_departments,
    people_org_chart, people_person_skills, people_skills, people_team_members,
    people_teams, people_time_off,
};

#[cfg(feature = "vectordb")]
pub use super::schema::{
    kb_collections, kb_documents, user_kb_associations,
};

pub use botlib::message_types::MessageType;
pub use botlib::models::{ApiResponse, Attachment, BotResponse, Session, Suggestion, UserMessage};
