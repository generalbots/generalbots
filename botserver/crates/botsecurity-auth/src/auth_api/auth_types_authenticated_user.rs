use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use axum::http::StatusCode;
use axum::Json;

use super::{BotAccess, Permission, Role};
use super::auth_types_users::AuthRejection;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticatedUser {
    pub user_id: Uuid,
    pub username: String,
    pub email: Option<String>,
    pub roles: Vec<Role>,
    pub bot_access: HashMap<Uuid, BotAccess>,
    pub current_bot_id: Option<Uuid>,
    pub session_id: Option<String>,
    pub organization_id: Option<Uuid>,
    pub metadata: HashMap<String, String>,
}

impl Default for AuthenticatedUser {
    fn default() -> Self {
        Self::anonymous()
    }
}

impl AuthenticatedUser {
    pub fn new(user_id: Uuid, username: String) -> Self {
        Self {
            user_id,
            username,
            email: None,
            roles: vec![Role::User],
            bot_access: HashMap::new(),
            current_bot_id: None,
            session_id: None,
            organization_id: None,
            metadata: HashMap::new(),
        }
    }

    pub fn anonymous() -> Self {
        Self {
            user_id: Uuid::nil(),
            username: "anonymous".to_string(),
            email: None,
            roles: vec![Role::Anonymous],
            bot_access: HashMap::new(),
            current_bot_id: None,
            session_id: None,
            organization_id: None,
            metadata: HashMap::new(),
        }
    }

    pub fn service(name: &str) -> Self {
        Self {
            user_id: Uuid::nil(),
            username: format!("service:{name}"),
            email: None,
            roles: vec![Role::Service],
            bot_access: HashMap::new(),
            current_bot_id: None,
            session_id: None,
            organization_id: None,
            metadata: HashMap::new(),
        }
    }

    pub fn bot_user(bot_id: Uuid, bot_name: &str) -> Self {
        Self {
            user_id: bot_id,
            username: format!("bot:{bot_name}"),
            email: None,
            roles: vec![Role::Bot],
            bot_access: HashMap::new(),
            current_bot_id: Some(bot_id),
            session_id: None,
            organization_id: None,
            metadata: HashMap::new(),
        }
    }

    pub fn with_email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }

    pub fn with_role(mut self, role: Role) -> Self {
        if !self.roles.contains(&role) {
            self.roles.push(role);
        }
        self
    }

    pub fn with_roles(mut self, roles: Vec<Role>) -> Self {
        self.roles = roles;
        self
    }

    pub fn with_bot_access(mut self, access: BotAccess) -> Self {
        self.bot_access.insert(access.bot_id, access);
        self
    }

    pub fn with_current_bot(mut self, bot_id: Uuid) -> Self {
        self.current_bot_id = Some(bot_id);
        self
    }

    pub fn with_session(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    pub fn with_organization(mut self, org_id: Uuid) -> Self {
        self.organization_id = Some(org_id);
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    pub fn has_permission(&self, permission: &Permission) -> bool {
        self.roles.iter().any(|r| r.has_permission(permission))
    }

    pub fn has_any_permission(&self, permissions: &[Permission]) -> bool {
        permissions.iter().any(|p| self.has_permission(p))
    }

    pub fn has_all_permissions(&self, permissions: &[Permission]) -> bool {
        permissions.iter().all(|p| self.has_permission(p))
    }

    pub fn has_role(&self, role: &Role) -> bool {
        self.roles.contains(role)
    }

    pub fn has_any_role(&self, roles: &[Role]) -> bool {
        roles.iter().any(|r| self.roles.contains(r))
    }

    pub fn highest_role(&self) -> &Role {
        self.roles
            .iter()
            .max_by_key(|r| r.hierarchy_level())
            .unwrap_or(&Role::Anonymous)
    }

    pub fn is_admin(&self) -> bool {
        self.has_role(&Role::Admin) || self.has_role(&Role::SuperAdmin)
    }

    pub fn is_super_admin(&self) -> bool {
        self.has_role(&Role::SuperAdmin)
    }

    pub fn is_authenticated(&self) -> bool {
        !self.has_role(&Role::Anonymous) && self.user_id != Uuid::nil()
    }

    pub fn is_service(&self) -> bool {
        self.has_role(&Role::Service)
    }

    pub fn is_bot(&self) -> bool {
        self.has_role(&Role::Bot)
    }

    pub fn get_bot_access(&self, bot_id: &Uuid) -> Option<&BotAccess> {
        self.bot_access.get(bot_id).filter(|a| a.is_valid())
    }

    pub fn get_bot_role(&self, bot_id: &Uuid) -> Option<&Role> {
        self.get_bot_access(bot_id).map(|a| &a.role)
    }

    pub fn has_bot_permission(&self, bot_id: &Uuid, permission: &Permission) -> bool {
        if self.is_admin() {
            return true;
        }

        if let Some(access) = self.get_bot_access(bot_id) {
            access.role.has_permission(permission)
        } else {
            false
        }
    }

    pub fn can_access_bot(&self, bot_id: &Uuid) -> bool {
        if self.is_admin() || self.is_service() {
            return true;
        }

        if self.current_bot_id.as_ref() == Some(bot_id) && self.is_bot() {
            return true;
        }

        self.get_bot_access(bot_id).is_some()
    }

    pub fn can_manage_bot(&self, bot_id: &Uuid) -> bool {
        if self.is_admin() {
            return true;
        }

        if let Some(access) = self.get_bot_access(bot_id) {
            access.role == Role::BotOwner
        } else {
            false
        }
    }

    pub fn can_operate_bot(&self, bot_id: &Uuid) -> bool {
        if self.is_admin() {
            return true;
        }

        if let Some(access) = self.get_bot_access(bot_id) {
            access.role.is_at_least(&Role::BotOperator)
        } else {
            false
        }
    }

    pub fn can_view_bot(&self, bot_id: &Uuid) -> bool {
        if self.is_admin() || self.is_service() {
            return true;
        }

        if let Some(access) = self.get_bot_access(bot_id) {
            access.role.is_at_least(&Role::BotViewer)
        } else {
            false
        }
    }

    pub fn can_access_organization(&self, org_id: &Uuid) -> bool {
        if self.is_admin() {
            return true;
        }
        self.organization_id
            .as_ref()
            .map(|id| id == org_id)
            .unwrap_or(false)
    }

    pub fn accessible_bot_ids(&self) -> Vec<Uuid> {
        self.bot_access
            .iter()
            .filter(|(_, access)| access.is_valid())
            .map(|(id, _)| *id)
            .collect()
    }

    pub fn owned_bot_ids(&self) -> Vec<Uuid> {
        self.bot_access
            .iter()
            .filter(|(_, access)| access.is_valid() && access.role == Role::BotOwner)
            .map(|(id, _)| *id)
            .collect()
    }
}

#[axum::async_trait]
impl<S> axum::extract::FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = AuthRejection;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthenticatedUser>()
            .cloned()
            .ok_or(AuthRejection {
                status: StatusCode::UNAUTHORIZED,
                body: Json(serde_json::json!({"error": "Authentication required"})),
            })
    }
}

