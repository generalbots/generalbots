use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, warn};
use uuid::Uuid;

use super::auth::{AuthenticatedUser, Permission, Role};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RbacConfig {
    pub cache_ttl_seconds: u64,
    pub enable_permission_cache: bool,
    pub enable_group_inheritance: bool,
    pub default_deny: bool,
    pub audit_all_decisions: bool,
}

impl Default for RbacConfig {
    fn default() -> Self {
        Self {
            cache_ttl_seconds: 300,
            enable_permission_cache: true,
            enable_group_inheritance: true,
            default_deny: true,
            audit_all_decisions: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResourcePermission {
    pub resource_type: String,
    pub resource_id: String,
    pub permission: String,
}

impl ResourcePermission {
    pub fn new(resource_type: &str, resource_id: &str, permission: &str) -> Self {
        Self {
            resource_type: resource_type.to_string(),
            resource_id: resource_id.to_string(),
            permission: permission.to_string(),
        }
    }

    pub fn read(resource_type: &str, resource_id: &str) -> Self {
        Self::new(resource_type, resource_id, "read")
    }

    pub fn write(resource_type: &str, resource_id: &str) -> Self {
        Self::new(resource_type, resource_id, "write")
    }

    pub fn delete(resource_type: &str, resource_id: &str) -> Self {
        Self::new(resource_type, resource_id, "delete")
    }

    pub fn admin(resource_type: &str, resource_id: &str) -> Self {
        Self::new(resource_type, resource_id, "admin")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccessDecision {
    Allow,
    Deny,
    NotApplicable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessDecisionResult {
    pub decision: AccessDecision,
    pub reason: String,
    pub evaluated_at: DateTime<Utc>,
    pub cache_hit: bool,
    pub matched_rule: Option<String>,
}

impl AccessDecisionResult {
    pub fn allow(reason: &str) -> Self {
        Self {
            decision: AccessDecision::Allow,
            reason: reason.to_string(),
            evaluated_at: Utc::now(),
            cache_hit: false,
            matched_rule: None,
        }
    }

    pub fn deny(reason: &str) -> Self {
        Self {
            decision: AccessDecision::Deny,
            reason: reason.to_string(),
            evaluated_at: Utc::now(),
            cache_hit: false,
            matched_rule: None,
        }
    }

    pub fn with_cache_hit(mut self) -> Self {
        self.cache_hit = true;
        self
    }

    pub fn with_rule(mut self, rule: String) -> Self {
        self.matched_rule = Some(rule);
        self
    }

    pub fn is_allowed(&self) -> bool {
        self.decision == AccessDecision::Allow
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutePermission {
    pub path_pattern: String,
    pub method: String,
    pub required_permission: String,
    pub required_roles: Vec<String>,
    pub allow_anonymous: bool,
    pub description: Option<String>,
}

impl RoutePermission {
    pub fn new(path_pattern: &str, method: &str, permission: &str) -> Self {
        Self {
            path_pattern: path_pattern.to_string(),
            method: method.to_string(),
            required_permission: permission.to_string(),
            required_roles: Vec::new(),
            allow_anonymous: false,
            description: None,
        }
    }

    pub fn with_roles(mut self, roles: Vec<String>) -> Self {
        self.required_roles = roles;
        self
    }

    pub fn with_anonymous(mut self, allow: bool) -> Self {
        self.allow_anonymous = allow;
        self
    }

    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = Some(desc.to_string());
        self
    }

    pub fn matches_path(&self, path: &str) -> bool {
        if self.path_pattern.contains('*') {
            let pattern_parts: Vec<&str> = self.path_pattern.split('/').collect();
            let path_parts: Vec<&str> = path.split('/').collect();

            if pattern_parts.len() > path_parts.len() && !self.path_pattern.ends_with("*") {
                return false;
            }

            for (i, pattern_part) in pattern_parts.iter().enumerate() {
                if *pattern_part == "*" || *pattern_part == "**" {
                    if *pattern_part == "**" {
                        return true;
                    }
                    continue;
                }

                if pattern_part.starts_with(':') {
                    continue;
                }

                if i >= path_parts.len() || *pattern_part != path_parts[i] {
                    return false;
                }
            }

            pattern_parts.len() <= path_parts.len() || self.path_pattern.contains("**")
        } else if self.path_pattern.contains(':') {
            let pattern_parts: Vec<&str> = self.path_pattern.split('/').collect();
            let path_parts: Vec<&str> = path.split('/').collect();

            if pattern_parts.len() != path_parts.len() {
                return false;
            }

            for (pattern_part, path_part) in pattern_parts.iter().zip(path_parts.iter()) {
                if pattern_part.starts_with(':') {
                    continue;
                }
                if *pattern_part != *path_part {
                    return false;
                }
            }

            true
        } else {
            self.path_pattern == path
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAcl {
    pub resource_type: String,
    pub resource_id: String,
    pub owner_id: Option<Uuid>,
    pub permissions: HashMap<Uuid, HashSet<String>>,
    pub group_permissions: HashMap<String, HashSet<String>>,
    pub public_permissions: HashSet<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ResourceAcl {
    pub fn new(resource_type: &str, resource_id: &str) -> Self {
        let now = Utc::now();
        Self {
            resource_type: resource_type.to_string(),
            resource_id: resource_id.to_string(),
            owner_id: None,
            permissions: HashMap::new(),
            group_permissions: HashMap::new(),
            public_permissions: HashSet::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_owner(mut self, owner_id: Uuid) -> Self {
        self.owner_id = Some(owner_id);
        self
    }

    pub fn grant_user(&mut self, user_id: Uuid, permission: &str) {
        self.permissions
            .entry(user_id)
            .or_default()
            .insert(permission.to_string());
        self.updated_at = Utc::now();
    }

    pub fn revoke_user(&mut self, user_id: Uuid, permission: &str) {
        if let Some(perms) = self.permissions.get_mut(&user_id) {
            perms.remove(permission);
            if perms.is_empty() {
                self.permissions.remove(&user_id);
            }
        }
        self.updated_at = Utc::now();
    }

    pub fn grant_group(&mut self, group_name: &str, permission: &str) {
        self.group_permissions
            .entry(group_name.to_string())
            .or_default()
            .insert(permission.to_string());
        self.updated_at = Utc::now();
    }

    pub fn revoke_group(&mut self, group_name: &str, permission: &str) {
        if let Some(perms) = self.group_permissions.get_mut(group_name) {
            perms.remove(permission);
            if perms.is_empty() {
                self.group_permissions.remove(group_name);
            }
        }
        self.updated_at = Utc::now();
    }

    pub fn set_public(&mut self, permission: &str) {
        self.public_permissions.insert(permission.to_string());
        self.updated_at = Utc::now();
    }

    pub fn remove_public(&mut self, permission: &str) {
        self.public_permissions.remove(permission);
        self.updated_at = Utc::now();
    }

    pub fn check_access(&self, user_id: Option<Uuid>, groups: &[String], permission: &str) -> bool {
        if self.public_permissions.contains(permission) {
            return true;
        }

        if let Some(uid) = user_id {
            if self.owner_id == Some(uid) {
                return true;
            }

            if let Some(user_perms) = self.permissions.get(&uid) {
                if user_perms.contains(permission) || user_perms.contains("admin") {
                    return true;
                }
            }
        }

        for group in groups {
            if let Some(group_perms) = self.group_permissions.get(group) {
                if group_perms.contains(permission) || group_perms.contains("admin") {
                    return true;
                }
            }
        }

        false
    }
}

#[derive(Debug, Clone)]
struct CacheEntry<T> {
    value: T,
    expires_at: DateTime<Utc>,
}

impl<T: Clone> CacheEntry<T> {
    fn new(value: T, ttl_seconds: u64) -> Self {
        Self {
            value,
            expires_at: Utc::now() + chrono::Duration::seconds(ttl_seconds as i64),
        }
    }

    fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}

pub struct RbacManager {
    config: RbacConfig,
    route_permissions: Arc<RwLock<Vec<RoutePermission>>>,
    resource_acls: Arc<RwLock<HashMap<String, ResourceAcl>>>,
    permission_cache: Arc<RwLock<HashMap<String, CacheEntry<AccessDecisionResult>>>>,
    user_groups: Arc<RwLock<HashMap<Uuid, Vec<String>>>>,
}

impl RbacManager {
    pub fn new(config: RbacConfig) -> Self {
        Self {
            config,
            route_permissions: Arc::new(RwLock::new(Vec::new())),
            resource_acls: Arc::new(RwLock::new(HashMap::new())),
            permission_cache: Arc::new(RwLock::new(HashMap::new())),
            user_groups: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn with_defaults() -> Self {
        let manager = Self::new(RbacConfig::default());
        manager
    }

    pub async fn register_route(&self, permission: RoutePermission) {
        let mut routes = self.route_permissions.write().await;
        routes.push(permission);
    }

    pub async fn register_routes(&self, permissions: Vec<RoutePermission>) {
        let mut routes = self.route_permissions.write().await;
        routes.extend(permissions);
    }

    pub async fn check_route_access(
        &self,
        path: &str,
        method: &str,
        user: &AuthenticatedUser,
    ) -> AccessDecisionResult {
        let cache_key = format!("route:{}:{}:{}", path, method, user.user_id);

        if self.config.enable_permission_cache {
            let cache = self.permission_cache.read().await;
            if let Some(entry) = cache.get(&cache_key) {
                if !entry.is_expired() {
                    return entry.value.clone().with_cache_hit();
                }
            }
        }

        let routes = self.route_permissions.read().await;
        let method_upper = method.to_uppercase();

        for route in routes.iter() {
            if route.method.to_uppercase() != method_upper && route.method != "*" {
                continue;
            }

            if !route.matches_path(path) {
                continue;
            }

            if route.allow_anonymous {
                let result = AccessDecisionResult::allow("Anonymous access allowed")
                    .with_rule(route.path_pattern.clone());
                self.cache_result(&cache_key, &result).await;
                return result;
            }

            if !user.is_authenticated() {
                let result = AccessDecisionResult::deny("Authentication required");
                return result;
            }

            if !route.required_roles.is_empty() {
                let has_role = route.required_roles.iter().any(|r| {
                    let role = Role::from_str(r);
                    user.has_role(&role)
                });

                if !has_role {
                    let result = AccessDecisionResult::deny("Insufficient role")
                        .with_rule(route.path_pattern.clone());
                    return result;
                }
            }

            if !route.required_permission.is_empty() {
                let has_permission = self
                    .check_permission_string(user, &route.required_permission)
                    .await;

                if !has_permission {
                    let result = AccessDecisionResult::deny("Missing required permission")
                        .with_rule(route.path_pattern.clone());
                    return result;
                }
            }

            let result = AccessDecisionResult::allow("Access granted")
                .with_rule(route.path_pattern.clone());
            self.cache_result(&cache_key, &result).await;
            return result;
        }

        if self.config.default_deny {
            AccessDecisionResult::deny("No matching route permission found")
        } else {
            AccessDecisionResult::allow("Default allow - no matching rule")
        }
    }

    pub async fn check_resource_access(
        &self,
        user: &AuthenticatedUser,
        resource_type: &str,
        resource_id: &str,
        permission: &str,
    ) -> AccessDecisionResult {
        let cache_key = format!(
            "resource:{}:{}:{}:{}",
            resource_type, resource_id, permission, user.user_id
        );

        if self.config.enable_permission_cache {
            let cache = self.permission_cache.read().await;
            if let Some(entry) = cache.get(&cache_key) {
                if !entry.is_expired() {
                    return entry.value.clone().with_cache_hit();
                }
            }
        }

        if user.is_admin() {
            let result = AccessDecisionResult::allow("Admin access");
            self.cache_result(&cache_key, &result).await;
            return result;
        }

        let acl_key = format!("{}:{}", resource_type, resource_id);
        let acls = self.resource_acls.read().await;

        if let Some(acl) = acls.get(&acl_key) {
            let user_groups = self.get_user_groups(user.user_id).await;
            let user_id = if user.is_authenticated() {
                Some(user.user_id)
            } else {
                None
            };

            if acl.check_access(user_id, &user_groups, permission) {
                let result = AccessDecisionResult::allow("ACL permission granted");
                self.cache_result(&cache_key, &result).await;
                return result;
            }

            let result = AccessDecisionResult::deny("ACL permission denied");
            return result;
        }

        if self.config.default_deny {
            AccessDecisionResult::deny("No ACL found for resource")
        } else {
            AccessDecisionResult::allow("Default allow - no ACL defined")
        }
    }

    pub async fn set_resource_acl(&self, acl: ResourceAcl) {
        let key = format!("{}:{}", acl.resource_type, acl.resource_id);
        let mut acls = self.resource_acls.write().await;
        acls.insert(key, acl);

        self.invalidate_cache_prefix("resource:").await;
    }

    pub async fn get_resource_acl(
        &self,
        resource_type: &str,
        resource_id: &str,
    ) -> Option<ResourceAcl> {
        let key = format!("{resource_type}:{resource_id}");
        let acls = self.resource_acls.read().await;
        acls.get(&key).cloned()
    }

    pub async fn delete_resource_acl(&self, resource_type: &str, resource_id: &str) {
        let key = format!("{resource_type}:{resource_id}");
        let mut acls = self.resource_acls.write().await;
        acls.remove(&key);

        self.invalidate_cache_prefix("resource:").await;
    }

    pub async fn set_user_groups(&self, user_id: Uuid, groups: Vec<String>) {
        let mut user_groups = self.user_groups.write().await;
        user_groups.insert(user_id, groups);

        self.invalidate_cache_prefix(&format!("resource:")).await;
    }

    pub async fn add_user_to_group(&self, user_id: Uuid, group: &str) {
        let mut user_groups = self.user_groups.write().await;
        user_groups
            .entry(user_id)
            .or_default()
            .push(group.to_string());

        self.invalidate_cache_prefix("resource:").await;
    }

    pub async fn remove_user_from_group(&self, user_id: Uuid, group: &str) {
        let mut user_groups = self.user_groups.write().await;
        if let Some(groups) = user_groups.get_mut(&user_id) {
            groups.retain(|g| g != group);
        }

        self.invalidate_cache_prefix("resource:").await;
    }

    pub async fn get_user_groups(&self, user_id: Uuid) -> Vec<String> {
        let user_groups = self.user_groups.read().await;
        user_groups.get(&user_id).cloned().unwrap_or_default()
    }

    pub async fn invalidate_user_cache(&self, user_id: Uuid) {
        let prefix = format!(":{user_id}");
        let mut cache = self.permission_cache.write().await;
        cache.retain(|k, _| !k.ends_with(&prefix));
    }

    pub async fn clear_cache(&self) {
        let mut cache = self.permission_cache.write().await;
        cache.clear();
    }

    async fn cache_result(&self, key: &str, result: &AccessDecisionResult) {
        if !self.config.enable_permission_cache {
            return;
        }

        let mut cache = self.permission_cache.write().await;
        cache.insert(
            key.to_string(),
            CacheEntry::new(result.clone(), self.config.cache_ttl_seconds),
        );
    }

    async fn invalidate_cache_prefix(&self, prefix: &str) {
        let mut cache = self.permission_cache.write().await;
        cache.retain(|k, _| !k.starts_with(prefix));
    }

    pub async fn check_permission_string(&self, user: &AuthenticatedUser, permission_str: &str) -> bool {
        let permission = match permission_str.to_lowercase().as_str() {
            "read" => Permission::Read,
            "write" => Permission::Write,
            "delete" => Permission::Delete,
            "admin" => Permission::Admin,
            "manage_users" | "users.manage" => Permission::ManageUsers,
            "manage_bots" | "bots.manage" => Permission::ManageBots,
            "view_analytics" | "analytics.view" => Permission::ViewAnalytics,
            "manage_settings" | "settings.manage" => Permission::ManageSettings,
            "execute_tasks" | "tasks.execute" => Permission::ExecuteTasks,
            "view_logs" | "logs.view" => Permission::ViewLogs,
            "manage_secrets" | "secrets.manage" => Permission::ManageSecrets,
            "access_api" | "api.access" => Permission::AccessApi,
            "manage_files" | "files.manage" => Permission::ManageFiles,
            "send_messages" | "messages.send" => Permission::SendMessages,
            "view_conversations" | "conversations.view" => Permission::ViewConversations,
            "manage_webhooks" | "webhooks.manage" => Permission::ManageWebhooks,
            "manage_integrations" | "integrations.manage" => Permission::ManageIntegrations,
            _ => return false,
        };

        user.has_permission(&permission)
    }

    pub fn config(&self) -> &RbacConfig {
        &self.config
    }
}

/// RBAC middleware function for use with middleware::from_fn
/// This version takes the RbacManager as a parameter instead of State
pub async fn rbac_middleware_fn(
    request: Request<Body>,
    next: Next,
    rbac: Arc<RbacManager>,
) -> Response {
    let path = request.uri().path().to_string();
    let method = request.method().to_string();

    let user = request
        .extensions()
        .get::<AuthenticatedUser>()
        .cloned()
        .unwrap_or_else(AuthenticatedUser::anonymous);

    debug!(
        "RBAC check: {} {} | user_id={} authenticated={} roles={:?}",
        method, path, user.user_id, user.is_authenticated(), user.roles
    );

    let decision = rbac.check_route_access(&path, &method, &user).await;

    debug!(
        "RBAC decision for {} {}: {:?} - {}",
        method, path, decision.decision, decision.reason
    );

    if !decision.is_allowed() {
        if !user.is_authenticated() {
            warn!(
                "RBAC: Unauthorized access attempt to {} {} (no auth)",
                method, path
            );
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": "unauthorized",
                    "message": "Authentication required"
                })),
            )
                .into_response();
        }

        warn!(
            "RBAC: Forbidden access to {} {} for user {} with roles {:?}",
            method, path, user.user_id, user.roles
        );
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "error": "forbidden",
                "message": decision.reason
            })),
        )
            .into_response();
    }

    next.run(request).await
}

pub async fn rbac_middleware(
    State(rbac): State<Arc<RbacManager>>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let path = request.uri().path().to_string();
    let method = request.method().to_string();

    let user = request
        .extensions()
        .get::<AuthenticatedUser>()
        .cloned()
        .unwrap_or_else(AuthenticatedUser::anonymous);

    let decision = rbac.check_route_access(&path, &method, &user).await;

    if rbac.config.audit_all_decisions {
        debug!(
            "RBAC decision for {} {} by user {}: {:?} - {}",
            method, path, user.user_id, decision.decision, decision.reason
        );
    }

    if !decision.is_allowed() {
        if !user.is_authenticated() {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": "unauthorized",
                    "message": "Authentication required"
                })),
            )
                .into_response();
        }

        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "error": "forbidden",
                "message": decision.reason
            })),
        )
            .into_response();
    }

    next.run(request).await
}

#[derive(Clone)]
pub struct RequirePermission {
    pub permission: String,
}

impl RequirePermission {
    pub fn new(permission: &str) -> Self {
        Self {
            permission: permission.to_string(),
        }
    }
}

#[derive(Clone)]
pub struct RequireRole {
    pub role: Role,
}

impl RequireRole {
    pub fn new(role: Role) -> Self {
        Self { role }
    }
}

#[derive(Clone)]
pub struct RequireResourceAccess {
    pub resource_type: String,
    pub permission: String,
}

impl RequireResourceAccess {
    pub fn new(resource_type: &str, permission: &str) -> Self {
        Self {
            resource_type: resource_type.to_string(),
            permission: permission.to_string(),
        }
    }
}

#[derive(Clone)]
pub struct RbacMiddlewareState {
    pub rbac_manager: Arc<RbacManager>,
    pub required_permission: Option<String>,
    pub required_roles: Vec<Role>,
    pub resource_type: Option<String>,
}

impl RbacMiddlewareState {
    pub fn new(rbac_manager: Arc<RbacManager>) -> Self {
        Self {
            rbac_manager,
            required_permission: None,
            required_roles: Vec::new(),
            resource_type: None,
        }
    }

    pub fn with_permission(mut self, permission: &str) -> Self {
        self.required_permission = Some(permission.to_string());
        self
    }

    pub fn with_roles(mut self, roles: Vec<Role>) -> Self {
        self.required_roles = roles;
        self
    }

    pub fn with_resource_type(mut self, resource_type: &str) -> Self {
        self.resource_type = Some(resource_type.to_string());
        self
    }
}

pub async fn require_permission_middleware(
    State(state): State<RbacMiddlewareState>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, RbacError> {
    let user = request
        .extensions()
        .get::<AuthenticatedUser>()
        .cloned()
        .unwrap_or_else(AuthenticatedUser::anonymous);

    if let Some(ref required_perm) = state.required_permission {
        let has_permission = state
            .rbac_manager
            .check_permission_string(&user, required_perm)
            .await;

        if !has_permission {
            warn!(
                "Permission denied for user {}: missing permission {}",
                user.user_id, required_perm
            );
            return Err(RbacError::PermissionDenied(format!(
                "Missing required permission: {required_perm}"
            )));
        }
    }

    if !state.required_roles.is_empty() {
        let has_required_role = state
            .required_roles
            .iter()
            .any(|role| user.has_role(role));

        if !has_required_role {
            warn!(
                "Role check failed for user {}: required one of {:?}",
                user.user_id, state.required_roles
            );
            return Err(RbacError::InsufficientRole(format!(
                "Required role: {:?}",
                state.required_roles
            )));
        }
    }

    Ok(next.run(request).await)
}

pub async fn require_admin_middleware(
    request: Request<Body>,
    next: Next,
) -> Result<Response, RbacError> {
    let user = request
        .extensions()
        .get::<AuthenticatedUser>()
        .cloned()
        .unwrap_or_else(AuthenticatedUser::anonymous);

    if !user.is_admin() && !user.is_super_admin() {
        warn!("Admin access denied for user {}", user.user_id);
        return Err(RbacError::AdminRequired);
    }

    Ok(next.run(request).await)
}

pub async fn require_super_admin_middleware(
    request: Request<Body>,
    next: Next,
) -> Result<Response, RbacError> {
    let user = request
        .extensions()
        .get::<AuthenticatedUser>()
        .cloned()
        .unwrap_or_else(AuthenticatedUser::anonymous);

    if !user.is_super_admin() {
        warn!("Super admin access denied for user {}", user.user_id);
        return Err(RbacError::SuperAdminRequired);
    }

    Ok(next.run(request).await)
}

#[derive(Debug, Clone)]
pub enum RbacError {
    PermissionDenied(String),
    InsufficientRole(String),
    AdminRequired,
    SuperAdminRequired,
    ResourceAccessDenied(String),
}

impl IntoResponse for RbacError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::PermissionDenied(msg) => (StatusCode::FORBIDDEN, msg),
            Self::InsufficientRole(msg) => (StatusCode::FORBIDDEN, msg),
            Self::AdminRequired => (
                StatusCode::FORBIDDEN,
                "Administrator access required".to_string(),
            ),
            Self::SuperAdminRequired => (
                StatusCode::FORBIDDEN,
                "Super administrator access required".to_string(),
            ),
            Self::ResourceAccessDenied(msg) => (StatusCode::FORBIDDEN, msg),
        };

        let body = serde_json::json!({
            "error": "access_denied",
            "message": message,
            "code": "RBAC_DENIED"
        });

        (status, Json(body)).into_response()
    }
}

pub fn create_permission_layer(
    rbac_manager: Arc<RbacManager>,
    permission: &str,
) -> RbacMiddlewareState {
    RbacMiddlewareState::new(rbac_manager).with_permission(permission)
}

pub fn create_role_layer(rbac_manager: Arc<RbacManager>, roles: Vec<Role>) -> RbacMiddlewareState {
    RbacMiddlewareState::new(rbac_manager).with_roles(roles)
}

pub fn create_admin_layer(rbac_manager: Arc<RbacManager>) -> RbacMiddlewareState {
    RbacMiddlewareState::new(rbac_manager).with_roles(vec![Role::Admin, Role::SuperAdmin])
}

pub fn build_default_route_permissions() -> Vec<RoutePermission> {
    vec![
        // =====================================================================
        // PUBLIC / ANONYMOUS ROUTES (no auth required)
        // =====================================================================
        RoutePermission::new("/health", "GET", "").with_anonymous(true),
        RoutePermission::new("/healthz", "GET", "").with_anonymous(true),
        RoutePermission::new("/api/health", "GET", "").with_anonymous(true),
        RoutePermission::new("/api/version", "GET", "").with_anonymous(true),
        RoutePermission::new("/api/product", "GET", "").with_anonymous(true),
        RoutePermission::new("/api/i18n/**", "GET", "").with_anonymous(true),

        // Auth routes - login must be anonymous
        RoutePermission::new("/api/auth", "GET", "").with_anonymous(true),

        RoutePermission::new("/api/auth/login", "POST", "").with_anonymous(true),
        RoutePermission::new("/api/auth/bootstrap", "POST", "").with_anonymous(true),
        RoutePermission::new("/api/auth/refresh", "POST", "").with_anonymous(true),
        RoutePermission::new("/api/auth/logout", "POST", ""),
        RoutePermission::new("/api/auth/me", "GET", ""),
        RoutePermission::new("/api/auth/**", "GET", ""),
        RoutePermission::new("/api/auth/**", "POST", ""),

        // WebSocket - anonymous for chat support
        RoutePermission::new("/ws", "GET", "").with_anonymous(true),
        RoutePermission::new("/ws/**", "GET", "").with_anonymous(true),

        // Chat - ANONYMOUS for customer support
        RoutePermission::new("/api/chat/**", "GET", "").with_anonymous(true),
        RoutePermission::new("/api/chat/**", "POST", "").with_anonymous(true),

        // Sessions - anonymous can create sessions for chat
        RoutePermission::new("/api/sessions", "POST", "").with_anonymous(true),
        RoutePermission::new("/api/sessions", "GET", ""),
        RoutePermission::new("/api/sessions/**", "GET", ""),

        // =====================================================================
        // AUTHENTICATED USER ROUTES (any logged-in user)
        // =====================================================================

        // Drive / Files
        RoutePermission::new("/api/drive/**", "GET", ""),
        RoutePermission::new("/api/drive/**", "POST", ""),
        RoutePermission::new("/api/drive/**", "PUT", ""),
        RoutePermission::new("/api/drive/**", "DELETE", ""),
        RoutePermission::new("/api/files/**", "GET", ""),
        RoutePermission::new("/api/files/**", "POST", ""),
        RoutePermission::new("/api/files/**", "PUT", ""),
        RoutePermission::new("/api/files/**", "DELETE", ""),

        // Mail
        RoutePermission::new("/api/mail/**", "GET", ""),
        RoutePermission::new("/api/mail/**", "POST", ""),
        RoutePermission::new("/api/mail/**", "PUT", ""),
        RoutePermission::new("/api/mail/**", "DELETE", ""),

        // Calendar
        RoutePermission::new("/api/calendar/**", "GET", ""),
        RoutePermission::new("/api/calendar/**", "POST", ""),
        RoutePermission::new("/api/calendar/**", "PUT", ""),
        RoutePermission::new("/api/calendar/**", "DELETE", ""),

        // Tasks
        RoutePermission::new("/api/tasks/**", "GET", ""),
        RoutePermission::new("/api/tasks/**", "POST", ""),
        RoutePermission::new("/api/tasks/**", "PUT", ""),
        RoutePermission::new("/api/tasks/**", "PATCH", ""),
        RoutePermission::new("/api/tasks/**", "DELETE", ""),

        // Docs / Paper
        RoutePermission::new("/api/docs/**", "GET", ""),
        RoutePermission::new("/api/docs/**", "POST", ""),
        RoutePermission::new("/api/docs/**", "PUT", ""),
        RoutePermission::new("/api/docs/**", "DELETE", ""),
        RoutePermission::new("/api/paper/**", "GET", ""),
        RoutePermission::new("/api/paper/**", "POST", ""),
        RoutePermission::new("/api/paper/**", "PUT", ""),
        RoutePermission::new("/api/paper/**", "DELETE", ""),

        // Sheet
        RoutePermission::new("/api/sheet/**", "GET", ""),
        RoutePermission::new("/api/sheet/**", "POST", ""),
        RoutePermission::new("/api/sheet/**", "PUT", ""),
        RoutePermission::new("/api/sheet/**", "DELETE", ""),

        // Slides
        RoutePermission::new("/api/slides/**", "GET", ""),
        RoutePermission::new("/api/slides/**", "POST", ""),
        RoutePermission::new("/api/slides/**", "PUT", ""),
        RoutePermission::new("/api/slides/**", "DELETE", ""),

        // Meet
        RoutePermission::new("/api/meet/**", "GET", ""),
        RoutePermission::new("/api/meet/**", "POST", ""),
        RoutePermission::new("/api/meet/**", "PUT", ""),
        RoutePermission::new("/api/meet/**", "DELETE", ""),

        // Research
        RoutePermission::new("/api/research/**", "GET", ""),
        RoutePermission::new("/api/research/**", "POST", ""),
        RoutePermission::new("/api/research/**", "PUT", ""),
        RoutePermission::new("/api/research/**", "DELETE", ""),

        // Sources
        RoutePermission::new("/api/sources/**", "GET", ""),
        RoutePermission::new("/api/sources/**", "POST", ""),
        RoutePermission::new("/api/sources/**", "PUT", ""),
        RoutePermission::new("/api/sources/**", "DELETE", ""),

        // Canvas
        RoutePermission::new("/api/canvas/**", "GET", ""),
        RoutePermission::new("/api/canvas/**", "POST", ""),
        RoutePermission::new("/api/canvas/**", "PUT", ""),
        RoutePermission::new("/api/canvas/**", "DELETE", ""),

        // Video / Player
        RoutePermission::new("/api/video/**", "GET", ""),
        RoutePermission::new("/api/video/**", "POST", ""),
        RoutePermission::new("/api/player/**", "GET", ""),
        RoutePermission::new("/api/player/**", "POST", ""),

        // Workspaces
        RoutePermission::new("/api/workspaces/**", "GET", ""),
        RoutePermission::new("/api/workspaces/**", "POST", ""),
        RoutePermission::new("/api/workspaces/**", "PUT", ""),
        RoutePermission::new("/api/workspaces/**", "DELETE", ""),

        // Projects
        RoutePermission::new("/api/projects/**", "GET", ""),
        RoutePermission::new("/api/projects/**", "POST", ""),
        RoutePermission::new("/api/projects/**", "PUT", ""),
        RoutePermission::new("/api/projects/**", "DELETE", ""),

        // Goals
        RoutePermission::new("/api/goals/**", "GET", ""),
        RoutePermission::new("/api/goals/**", "POST", ""),
        RoutePermission::new("/api/goals/**", "PUT", ""),
        RoutePermission::new("/api/goals/**", "DELETE", ""),

        // Settings (user's own settings)
        RoutePermission::new("/api/settings/**", "GET", ""),
        RoutePermission::new("/api/settings/**", "POST", ""),
        RoutePermission::new("/api/settings/**", "PUT", ""),

        // Bots (read for all authenticated users)
        RoutePermission::new("/api/bots", "GET", ""),
        RoutePermission::new("/api/bots/:id", "GET", ""),
        RoutePermission::new("/api/bots/:id/**", "GET", ""),

        // Autotask
        RoutePermission::new("/api/autotask/**", "GET", ""),
        RoutePermission::new("/api/autotask/**", "POST", ""),
        RoutePermission::new("/api/autotask/**", "PUT", ""),
        RoutePermission::new("/api/autotask/**", "DELETE", ""),

        // Designer
        RoutePermission::new("/api/designer/**", "GET", ""),
        RoutePermission::new("/api/designer/**", "POST", ""),
        RoutePermission::new("/api/designer/**", "PUT", ""),
        RoutePermission::new("/api/designer/**", "DELETE", ""),

        // Dashboards
        RoutePermission::new("/api/dashboards/**", "GET", ""),
        RoutePermission::new("/api/dashboards/**", "POST", ""),
        RoutePermission::new("/api/dashboards/**", "PUT", ""),
        RoutePermission::new("/api/dashboards/**", "DELETE", ""),

        // DB/Table access
        RoutePermission::new("/api/db/**", "GET", ""),
        RoutePermission::new("/api/db/**", "POST", ""),
        RoutePermission::new("/api/db/**", "PUT", ""),
        RoutePermission::new("/api/db/**", "DELETE", ""),

        // CRM / Contacts
        RoutePermission::new("/api/crm/**", "GET", ""),
        RoutePermission::new("/api/crm/**", "POST", ""),
        RoutePermission::new("/api/crm/**", "PUT", ""),
        RoutePermission::new("/api/crm/**", "DELETE", ""),
        RoutePermission::new("/api/contacts/**", "GET", ""),
        RoutePermission::new("/api/contacts/**", "POST", ""),
        RoutePermission::new("/api/contacts/**", "PUT", ""),
        RoutePermission::new("/api/contacts/**", "DELETE", ""),

        // Billing / Products
        RoutePermission::new("/api/billing/**", "GET", ""),
        RoutePermission::new("/api/billing/**", "POST", ""),
        RoutePermission::new("/api/products/**", "GET", ""),
        RoutePermission::new("/api/products/**", "POST", ""),
        RoutePermission::new("/api/products/**", "PUT", ""),
        RoutePermission::new("/api/products/**", "DELETE", ""),

        // Tickets
        RoutePermission::new("/api/tickets/**", "GET", ""),
        RoutePermission::new("/api/tickets/**", "POST", ""),
        RoutePermission::new("/api/tickets/**", "PUT", ""),
        RoutePermission::new("/api/tickets/**", "DELETE", ""),

        // Learn
        RoutePermission::new("/api/learn/**", "GET", ""),
        RoutePermission::new("/api/learn/**", "POST", ""),

        // Social
        RoutePermission::new("/api/social/**", "GET", ""),
        RoutePermission::new("/api/social/**", "POST", ""),

        // LLM
        RoutePermission::new("/api/llm/**", "GET", ""),
        RoutePermission::new("/api/llm/**", "POST", ""),

        // Email
        RoutePermission::new("/api/email/**", "GET", ""),
        RoutePermission::new("/api/email/**", "POST", ""),
        RoutePermission::new("/api/email/**", "PUT", ""),
        RoutePermission::new("/api/email/**", "DELETE", ""),

        // Messaging channels
        RoutePermission::new("/api/telegram/**", "GET", ""),
        RoutePermission::new("/api/telegram/**", "POST", ""),
        RoutePermission::new("/api/whatsapp/**", "GET", ""),
        RoutePermission::new("/api/whatsapp/**", "POST", ""),
        RoutePermission::new("/api/msteams/**", "GET", ""),
        RoutePermission::new("/api/msteams/**", "POST", ""),
        RoutePermission::new("/api/instagram/**", "GET", ""),
        RoutePermission::new("/api/instagram/**", "POST", ""),

        // Pages
        RoutePermission::new("/api/pages/**", "GET", ""),
        RoutePermission::new("/api/pages/**", "POST", ""),
        RoutePermission::new("/api/pages/**", "PUT", ""),
        RoutePermission::new("/api/pages/**", "DELETE", ""),

        // Insights
        RoutePermission::new("/api/insights/**", "GET", ""),
        RoutePermission::new("/api/insights/**", "POST", ""),

        // App logs
        RoutePermission::new("/api/app-logs/**", "GET", ""),
        RoutePermission::new("/api/app-logs/**", "POST", ""),

        // User profile (own user)
        RoutePermission::new("/api/user/**", "GET", ""),
        RoutePermission::new("/api/user/**", "PUT", ""),

        // =====================================================================
        // UI ROUTES (HTMX endpoints) - authenticated users
        // =====================================================================
        RoutePermission::new("/api/ui/tasks/**", "GET", ""),
        RoutePermission::new("/api/ui/tasks/**", "POST", ""),
        RoutePermission::new("/api/ui/tasks/**", "PUT", ""),
        RoutePermission::new("/api/ui/tasks/**", "PATCH", ""),
        RoutePermission::new("/api/ui/tasks/**", "DELETE", ""),
        RoutePermission::new("/api/ui/calendar/**", "GET", ""),
        RoutePermission::new("/api/ui/calendar/**", "POST", ""),
        RoutePermission::new("/api/ui/drive/**", "GET", ""),
        RoutePermission::new("/api/ui/drive/**", "POST", ""),
        RoutePermission::new("/api/ui/mail/**", "GET", ""),
        RoutePermission::new("/api/ui/mail/**", "POST", ""),
        RoutePermission::new("/api/ui/docs/**", "GET", ""),
        RoutePermission::new("/api/ui/docs/**", "POST", ""),
        RoutePermission::new("/api/ui/paper/**", "GET", ""),
        RoutePermission::new("/api/ui/paper/**", "POST", ""),
        RoutePermission::new("/api/ui/sheet/**", "GET", ""),
        RoutePermission::new("/api/ui/sheet/**", "POST", ""),
        RoutePermission::new("/api/ui/slides/**", "GET", ""),
        RoutePermission::new("/api/ui/slides/**", "POST", ""),
        RoutePermission::new("/api/ui/meet/**", "GET", ""),
        RoutePermission::new("/api/ui/meet/**", "POST", ""),
        RoutePermission::new("/api/ui/research/**", "GET", ""),
        RoutePermission::new("/api/ui/research/**", "POST", ""),
        RoutePermission::new("/api/ui/sources/**", "GET", ""),
        RoutePermission::new("/api/ui/sources/**", "POST", ""),
        RoutePermission::new("/api/ui/canvas/**", "GET", ""),
        RoutePermission::new("/api/ui/video/**", "GET", ""),
        RoutePermission::new("/api/ui/player/**", "GET", ""),
        RoutePermission::new("/api/ui/workspaces/**", "GET", ""),
        RoutePermission::new("/api/ui/projects/**", "GET", ""),
        RoutePermission::new("/api/ui/goals/**", "GET", ""),
        RoutePermission::new("/api/ui/designer/**", "GET", ""),
        RoutePermission::new("/api/ui/dashboards/**", "GET", ""),
        RoutePermission::new("/api/ui/crm/**", "GET", ""),
        RoutePermission::new("/api/ui/billing/**", "GET", ""),
        RoutePermission::new("/api/ui/products/**", "GET", ""),
        RoutePermission::new("/api/ui/tickets/**", "GET", ""),
        RoutePermission::new("/api/ui/learn/**", "GET", ""),
        RoutePermission::new("/api/ui/social/**", "GET", ""),
        RoutePermission::new("/api/ui/settings/**", "GET", ""),
        RoutePermission::new("/api/ui/autotask/**", "GET", ""),
        RoutePermission::new("/api/ui/email/**", "GET", ""),
        RoutePermission::new("/api/ui/email/**", "POST", ""),

        // =====================================================================
        // ADMIN ROUTES (requires Admin or SuperAdmin role)
        // =====================================================================
        RoutePermission::new("/api/users", "GET", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/users", "POST", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/users/:id", "GET", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/users/:id", "PUT", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/users/:id", "DELETE", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/users/**", "GET", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/users/**", "POST", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/users/**", "PUT", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/users/**", "DELETE", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),

        // Groups management
        RoutePermission::new("/api/groups/**", "GET", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/groups/**", "POST", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/groups/**", "PUT", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/groups/**", "DELETE", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),

        // Bot management (create/delete)
        RoutePermission::new("/api/bots", "POST", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/bots/:id", "PUT", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/bots/:id", "DELETE", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/bots/:id/**", "PUT", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/bots/:id/**", "DELETE", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),

        // Analytics (admin view)
        RoutePermission::new("/api/analytics/**", "GET", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into(), "Moderator".into()]),
        RoutePermission::new("/api/ui/analytics/**", "GET", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into(), "Moderator".into()]),

        // Monitoring
        RoutePermission::new("/api/monitoring/**", "GET", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/ui/monitoring/**", "GET", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),

        // Audit logs
        RoutePermission::new("/api/audit/**", "GET", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/ui/audit/**", "GET", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),

        // Security settings
        RoutePermission::new("/api/security/**", "GET", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/security/**", "POST", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/security/**", "PUT", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/ui/security/**", "GET", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),

        // Admin panel
        RoutePermission::new("/api/admin/**", "GET", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/admin/**", "POST", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/admin/**", "PUT", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/admin/**", "DELETE", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/ui/admin/**", "GET", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),

        // Attendant (customer service)
        RoutePermission::new("/api/attendant/**", "GET", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into(), "Moderator".into()]),
        RoutePermission::new("/api/attendant/**", "POST", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into(), "Moderator".into()]),
        RoutePermission::new("/api/ui/attendant/**", "GET", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into(), "Moderator".into()]),

        // =====================================================================
        // SUPER ADMIN ONLY ROUTES
        // =====================================================================
        RoutePermission::new("/api/rbac/**", "GET", "")
            .with_roles(vec!["SuperAdmin".into()]),
        RoutePermission::new("/api/rbac/**", "POST", "")
            .with_roles(vec!["SuperAdmin".into()]),
        RoutePermission::new("/api/rbac/**", "PUT", "")
            .with_roles(vec!["SuperAdmin".into()]),
        RoutePermission::new("/api/rbac/**", "DELETE", "")
            .with_roles(vec!["SuperAdmin".into()]),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_permission_exact_match() {
        let route = RoutePermission::new("/api/users", "GET", "users.read");

        assert!(route.matches_path("/api/users"));
        assert!(!route.matches_path("/api/users/123"));
        assert!(!route.matches_path("/api/user"));
    }

    #[test]
    fn test_route_permission_param_match() {
        let route = RoutePermission::new("/api/users/:id", "GET", "users.read");

        assert!(route.matches_path("/api/users/123"));
        assert!(route.matches_path("/api/users/abc"));
        assert!(!route.matches_path("/api/users"));
        assert!(!route.matches_path("/api/users/123/profile"));
    }

    #[test]
    fn test_route_permission_wildcard_match() {
        let route = RoutePermission::new("/api/drive/**", "GET", "drive.read");

        assert!(route.matches_path("/api/drive"));
        assert!(route.matches_path("/api/drive/files"));
        assert!(route.matches_path("/api/drive/files/123"));
        assert!(route.matches_path("/api/drive/a/b/c/d"));
        assert!(!route.matches_path("/api/mail"));
    }

    #[test]
    fn test_route_permission_single_wildcard() {
        let route = RoutePermission::new("/api/*/info", "GET", "info.read");

        assert!(route.matches_path("/api/users/info"));
        assert!(route.matches_path("/api/bots/info"));
    }

    #[test]
    fn test_resource_acl_owner_access() {
        let owner_id = Uuid::new_v4();
        let other_id = Uuid::new_v4();

        let acl = ResourceAcl::new("file", "123").with_owner(owner_id);

        assert!(acl.check_access(Some(owner_id), &[], "read"));
        assert!(acl.check_access(Some(owner_id), &[], "write"));
        assert!(acl.check_access(Some(owner_id), &[], "delete"));
        assert!(!acl.check_access(Some(other_id), &[], "read"));
    }

    #[test]
    fn test_resource_acl_user_permissions() {
        let user_id = Uuid::new_v4();
        let mut acl = ResourceAcl::new("file", "123");

        acl.grant_user(user_id, "read");

        assert!(acl.check_access(Some(user_id), &[], "read"));
        assert!(!acl.check_access(Some(user_id), &[], "write"));
    }

    #[test]
    fn test_resource_acl_group_permissions() {
        let user_id = Uuid::new_v4();
        let mut acl = ResourceAcl::new("file", "123");

        acl.grant_group("editors", "write");

        assert!(acl.check_access(Some(user_id), &["editors".into()], "write"));
        assert!(!acl.check_access(Some(user_id), &["viewers".into()], "write"));
    }

    #[test]
    fn test_resource_acl_public_permissions() {
        let mut acl = ResourceAcl::new("file", "123");

        acl.set_public("read");

        assert!(acl.check_access(None, &[], "read"));
        assert!(!acl.check_access(None, &[], "write"));
    }

    #[test]
    fn test_resource_acl_admin_access() {
        let user_id = Uuid::new_v4();
        let mut acl = ResourceAcl::new("file", "123");

        acl.grant_user(user_id, "admin");

        assert!(acl.check_access(Some(user_id), &[], "read"));
        assert!(acl.check_access(Some(user_id), &[], "write"));
        assert!(acl.check_access(Some(user_id), &[], "delete"));
    }

    #[test]
    fn test_access_decision_result() {
        let allow = AccessDecisionResult::allow("Test allow");
        assert!(allow.is_allowed());

        let deny = AccessDecisionResult::deny("Test deny");
        assert!(!deny.is_allowed());
    }

    #[test]
    fn test_resource_permission_builders() {
        let read = ResourcePermission::read("file", "123");
        assert_eq!(read.permission, "read");

        let write = ResourcePermission::write("file", "123");
        assert_eq!(write.permission, "write");

        let delete = ResourcePermission::delete("file", "123");
        assert_eq!(delete.permission, "delete");
    }

    #[tokio::test]
    async fn test_rbac_manager_creation() {
        let manager = RbacManager::with_defaults();
        let routes = build_default_route_permissions();

        manager.register_routes(routes).await;

        let user = AuthenticatedUser::anonymous();
        let decision = manager
            .check_route_access("/api/health", "GET", &user)
            .await;

        assert!(decision.is_allowed());
    }

    #[tokio::test]
    async fn test_user_groups() {
        let manager = RbacManager::with_defaults();
        let user_id = Uuid::new_v4();

        manager.set_user_groups(user_id, vec!["editors".into(), "viewers".into()]).await;

        let groups = manager.get_user_groups(user_id).await;
        assert_eq!(groups.len(), 2);
        assert!(groups.contains(&"editors".into()));
    }

    #[tokio::test]
    async fn test_resource_acl_management() {
        let manager = RbacManager::with_defaults();
        let owner_id = Uuid::new_v4();

        let acl = ResourceAcl::new("document", "doc-123").with_owner(owner_id);
        manager.set_resource_acl(acl).await;

        let retrieved = manager.get_resource_acl("document", "doc-123").await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.as_ref().and_then(|a| a.owner_id), Some(owner_id));
    }
}
