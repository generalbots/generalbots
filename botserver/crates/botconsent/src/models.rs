//! Data contracts for the per-app agent consent system.
//!
//! Action classes classify every application operation the agent may attempt.
//! Sensitive classes listed in [`ALWAYS_REPROMPT`] ignore stored "always"
//! grants once their cycle expires: the grant row carries an `expires_at`
//! set to end-of-month UTC at grant time and enforcement recomputes it.

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::schema::{app_permissions, consent_audit};

/// Every action class an application command can belong to.
pub const ACTION_CLASSES: &[&str] = &[
    "read", "create", "update", "delete", "payment", "share", "publish",
    "vm_command", "vm_file_read", "vm_file_write", "vm_file_delete", "vm_network", "vm_browser",
    "vm_all", "vm_drive_save", "vm_external_save",
];

/// Agent actions inside the person's virtual machine. `vm_all` is the
/// "allow everything" grant of one agent in one conversation; it covers these
/// classes only.
pub const VM_IN_CLASSES: &[&str] = &[
    "vm_command", "vm_file_read", "vm_file_write", "vm_file_delete", "vm_network", "vm_browser",
];

/// Agent actions that send data from the virtual machine to somewhere else
/// (the person's Drive, an external cloud drive). Never covered by `vm_all`.
pub const VM_OUT_CLASSES: &[&str] = &["vm_drive_save", "vm_external_save"];

/// Class of the conversation-wide "allow everything" grant.
pub const VM_ALL_CLASS: &str = "vm_all";

#[must_use]
pub fn is_vm_class(class: &str) -> bool {
    class == VM_ALL_CLASS || VM_IN_CLASSES.contains(&class) || VM_OUT_CLASSES.contains(&class)
}

#[must_use]
pub fn is_outside_vm_class(class: &str) -> bool {
    VM_OUT_CLASSES.contains(&class)
}

/// Classes that always re-prompt when the current grant cycle expires,
/// regardless of a stored "always" decision.
pub const ALWAYS_REPROMPT: &[&str] = &["payment"];

/// Classes that always re-prompt, regardless of a stored grant: they are
/// destructive enough that consent is per-call by design.
pub const DESTRUCTIVE_CLASSES: &[&str] = &["delete", "payment"];

/// Classes reserved for administrator pre-approval (placeholder for future use).
pub const ADMIN_PREAPPROVAL_ONLY: &[&str] = &[];

pub fn is_known_action_class(class: &str) -> bool {
    ACTION_CLASSES.contains(&class)
}

pub fn is_reprompt_class(class: &str) -> bool {
    ALWAYS_REPROMPT.contains(&class)
}

pub fn is_destructive_class(class: &str) -> bool {
    DESTRUCTIVE_CLASSES.contains(&class)
}

/// Risk tier shown on the consent card and used to decide whether a stored
/// grant may satisfy a call.
#[must_use]
pub fn risk_tier(class: &str) -> &'static str {
    match class {
        "read" | "vm_file_read" => "low",
        "create" | "update" | "share" => "medium",
        "delete" | "payment" | "publish" => "high",
        "vm_command" | "vm_file_write" | "vm_network" | "vm_browser" => "medium",
        "vm_file_delete" | "vm_drive_save" | "vm_external_save" | "vm_all" => "high",
        _ => "unknown",
    }
}

/// Lifetime of a grant. `Run` is scoped to one run id, `Workspace` to one
/// hour, `Always` until the sensitive-class cycle cap, `Conversation` to one
/// thread (the thread id is required) for at most a day.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsentScope {
    Once,
    Run,
    Workspace,
    Always,
    Conversation,
}

impl ConsentScope {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Once => "once",
            Self::Run => "run",
            Self::Workspace => "workspace",
            Self::Always => "always",
            Self::Conversation => "conversation",
        }
    }

    /// Seconds a persisted grant lives; `Once` is never persisted.
    #[must_use]
    pub fn lifetime_secs(self) -> Option<i64> {
        match self {
            Self::Once => None,
            Self::Run => Some(6 * 60 * 60),
            Self::Workspace => Some(60 * 60),
            Self::Always => None,
            Self::Conversation => Some(24 * 60 * 60),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Insertable)]
#[diesel(table_name = app_permissions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AppPermissionRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub org_id: Option<Uuid>,
    pub branch_id: Option<Uuid>,
    pub app_id: String,
    pub action_class: String,
    pub scope: serde_json::Value,
    pub granted: bool,
    pub granted_via: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub granted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Insertable)]
#[diesel(table_name = consent_audit)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ConsentAuditRow {
    pub id: Uuid,
    pub permission_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub request: serde_json::Value,
    pub outcome: String,
    pub decided_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

/// Body requesting a stored grant for an application action class.
#[derive(Debug, Clone, Deserialize)]
pub struct GrantBody {
    pub app_id: String,
    pub action_class: String,
    #[serde(default)]
    pub scope: serde_json::Value,
    pub expires_at: Option<DateTime<Utc>>,
}

/// User decision over one pending consent request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Decision {
    #[serde(rename = "allow_once")]
    AllowOnce,
    #[serde(rename = "allow_run")]
    AllowRun,
    #[serde(rename = "allow_workspace")]
    AllowWorkspace,
    #[serde(rename = "always")]
    Always,
    /// This kind of action, for this agent, in this conversation.
    #[serde(rename = "allow_conversation")]
    AllowConversation,
    /// Every in-VM action of this agent in this conversation (`vm_all`).
    #[serde(rename = "allow_all")]
    AllowAll,
    #[serde(rename = "deny")]
    Deny,
}

/// Body posted by the consent card / settings UI to resolve a request.
#[derive(Debug, Clone, Deserialize)]
pub struct ResolveBody {
    pub request_id: String,
    pub decision: Decision,
    /// The conversation the card is answered in; when present, a
    /// conversation-scoped decision must come from the conversation that asked.
    #[serde(default)]
    pub thread_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decision_serde_roundtrip_lowercase() {
        assert_eq!(
            serde_json::from_str::<Decision>("\"allow_once\"").expect("parse allow_once"),
            Decision::AllowOnce
        );
        assert_eq!(
            serde_json::from_str::<Decision>("\"always\"").expect("parse always"),
            Decision::Always
        );
        assert_eq!(
            serde_json::from_str::<Decision>("\"deny\"").expect("parse deny"),
            Decision::Deny
        );
        assert_eq!(
            serde_json::to_string(&Decision::AllowOnce).expect("serialize allow_once"),
            "\"allow_once\""
        );
    }

    #[test]
    fn reprompt_classes_are_known() {
        assert!(is_reprompt_class("payment"));
        assert!(!is_reprompt_class("read"));
        assert!(is_known_action_class("publish"));
        assert!(!is_known_action_class("sudo"));
    }

    #[test]
    fn vm_classes_are_known_and_split_by_boundary() {
        for class in VM_IN_CLASSES.iter().chain(VM_OUT_CLASSES) {
            assert!(is_known_action_class(class), "{class}");
            assert!(is_vm_class(class), "{class}");
            assert!(!is_destructive_class(class), "{class}");
        }
        assert!(is_outside_vm_class("vm_drive_save"));
        assert!(!is_outside_vm_class("vm_command"));
        assert!(VM_IN_CLASSES.contains(&"vm_file_read"), "vm_all covers file reads");
        assert!(!is_outside_vm_class("vm_file_read"));
        assert_eq!(risk_tier("vm_file_read"), "low");
        assert!(!is_vm_class("delete"));
        assert_eq!(
            serde_json::from_str::<Decision>("\"allow_all\"").expect("parse allow_all"),
            Decision::AllowAll
        );
        assert_eq!(ConsentScope::Conversation.as_str(), "conversation");
    }
}
