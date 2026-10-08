use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

// Permission is defined in the parent module (types.rs); bring it into scope so
// the impl blocks and Role::permissions() can name it directly.
use super::Permission;

// Re-export the AuthenticatedUser type from its dedicated module so the parent
// `types` re-export (`pub use super::auth_types_users::{..., AuthenticatedUser}`) keeps
// resolving.
use axum::http::StatusCode;
use axum::Json;
pub use super::auth_types_authenticated_user::AuthenticatedUser;

/// Marker extension inserted by auth middleware when a path is allowed as public/anonymous.
/// RBAC middleware checks for this marker and skips route permission checks when present.
#[derive(Debug, Clone)]
pub struct PublicPathAllowed;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Role {
    #[default]
    Anonymous,
    User,
    Moderator,
    Admin,
    SuperAdmin,
    Service,
    Bot,
    BotOwner,
    BotOperator,
    BotViewer,
}

impl Role {
    pub fn permissions(&self) -> HashSet<Permission> {
        match self {
            Self::Anonymous => HashSet::new(),
            Self::User => {
                let mut perms = HashSet::new();
                perms.insert(Permission::Read);
                perms.insert(Permission::AccessApi);
                perms
            }
            Self::Moderator => {
                let mut perms = Self::User.permissions();
                perms.insert(Permission::Write);
                perms.insert(Permission::ViewLogs);
                perms.insert(Permission::ViewAnalytics);
                perms.insert(Permission::ViewConversations);
                perms
            }
            Self::Admin => {
                let mut perms = Self::Moderator.permissions();
                perms.insert(Permission::Delete);
                perms.insert(Permission::ManageUsers);
                perms.insert(Permission::ManageBots);
                perms.insert(Permission::ManageSettings);
                perms.insert(Permission::ExecuteTasks);
                perms.insert(Permission::ManageFiles);
                perms.insert(Permission::ManageWebhooks);
                perms
            }
            Self::SuperAdmin => {
                let mut perms = Self::Admin.permissions();
                perms.insert(Permission::Admin);
                perms.insert(Permission::ManageSecrets);
                perms.insert(Permission::ManageIntegrations);
                perms
            }
            Self::Service => {
                let mut perms = HashSet::new();
                perms.insert(Permission::Read);
                perms.insert(Permission::Write);
                perms.insert(Permission::AccessApi);
                perms.insert(Permission::ExecuteTasks);
                perms.insert(Permission::SendMessages);
                perms
            }
            Self::Bot => {
                let mut perms = HashSet::new();
                perms.insert(Permission::Read);
                perms.insert(Permission::Write);
                perms.insert(Permission::AccessApi);
                perms.insert(Permission::SendMessages);
                perms
            }
            Self::BotOwner => {
                let mut perms = HashSet::new();
                perms.insert(Permission::Read);
                perms.insert(Permission::Write);
                perms.insert(Permission::Delete);
                perms.insert(Permission::AccessApi);
                perms.insert(Permission::ManageBots);
                perms.insert(Permission::ManageSettings);
                perms.insert(Permission::ViewAnalytics);
                perms.insert(Permission::ViewLogs);
                perms.insert(Permission::ManageFiles);
                perms.insert(Permission::SendMessages);
                perms.insert(Permission::ViewConversations);
                perms.insert(Permission::ManageWebhooks);
                perms
            }
            Self::BotOperator => {
                let mut perms = HashSet::new();
                perms.insert(Permission::Read);
                perms.insert(Permission::Write);
                perms.insert(Permission::AccessApi);
                perms.insert(Permission::ViewAnalytics);
                perms.insert(Permission::ViewLogs);
                perms.insert(Permission::SendMessages);
                perms.insert(Permission::ViewConversations);
                perms
            }
            Self::BotViewer => {
                let mut perms = HashSet::new();
                perms.insert(Permission::Read);
                perms.insert(Permission::AccessApi);
                perms.insert(Permission::ViewAnalytics);
                perms.insert(Permission::ViewConversations);
                perms
            }
        }
    }

    pub fn has_permission(&self, permission: &Permission) -> bool {
        self.permissions().contains(permission)
    }
}

impl std::str::FromStr for Role {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "anonymous" => Ok(Self::Anonymous),
            "user" => Ok(Self::User),
            "moderator" | "mod" => Ok(Self::Moderator),
            "admin" => Ok(Self::Admin),
            "superadmin" | "super_admin" | "super" => Ok(Self::SuperAdmin),
            "service" | "svc" => Ok(Self::Service),
            "bot" => Ok(Self::Bot),
            "bot_owner" | "botowner" | "owner" => Ok(Self::BotOwner),
            "bot_operator" | "botoperator" | "operator" => Ok(Self::BotOperator),
            "bot_viewer" | "botviewer" | "viewer" => Ok(Self::BotViewer),
            _ => Ok(Self::Anonymous),
        }
    }
}

impl Role {
    pub fn hierarchy_level(&self) -> u8 {
        match self {
            Self::Anonymous => 0,
            Self::User => 1,
            Self::BotViewer => 2,
            Self::BotOperator => 3,
            Self::BotOwner => 4,
            Self::Bot => 4,
            Self::Moderator => 5,
            Self::Service => 6,
            Self::Admin => 7,
            Self::SuperAdmin => 8,
        }
    }

    pub fn is_at_least(&self, other: &Role) -> bool {
        self.hierarchy_level() >= other.hierarchy_level()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotAccess {
    pub bot_id: Uuid,
    pub role: Role,
    pub granted_at: Option<i64>,
    pub granted_by: Option<Uuid>,
    pub expires_at: Option<i64>,
}

impl BotAccess {
    pub fn new(bot_id: Uuid, role: Role) -> Self {
        Self {
            bot_id,
            role,
            granted_at: Some(chrono::Utc::now().timestamp()),
            granted_by: None,
            expires_at: None,
        }
    }

    pub fn owner(bot_id: Uuid) -> Self {
        Self::new(bot_id, Role::BotOwner)
    }

    pub fn operator(bot_id: Uuid) -> Self {
        Self::new(bot_id, Role::BotOperator)
    }

    pub fn viewer(bot_id: Uuid) -> Self {
        Self::new(bot_id, Role::BotViewer)
    }

    pub fn with_expiry(mut self, expires_at: i64) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    pub fn with_grantor(mut self, granted_by: Uuid) -> Self {
        self.granted_by = Some(granted_by);
        self
    }

    pub fn is_expired(&self) -> bool {
        if let Some(expires) = self.expires_at {
            chrono::Utc::now().timestamp() > expires
        } else {
            false
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.is_expired()
    }
}

#[derive(Debug, Clone)]
pub struct AuthRejection {
    pub status: StatusCode,
    pub body: Json<serde_json::Value>,
}

impl axum::response::IntoResponse for AuthRejection {
    fn into_response(self) -> axum::response::Response {
        (self.status, self.body).into_response()
    }
}

