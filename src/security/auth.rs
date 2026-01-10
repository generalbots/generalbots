use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tracing::{debug, warn};
use uuid::Uuid;

use crate::security::auth_provider::AuthProviderRegistry;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Permission {
    Read,
    Write,
    Delete,
    Admin,
    ManageUsers,
    ManageBots,
    ViewAnalytics,
    ManageSettings,
    ExecuteTasks,
    ViewLogs,
    ManageSecrets,
    AccessApi,
    ManageFiles,
    SendMessages,
    ViewConversations,
    ManageWebhooks,
    ManageIntegrations,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Role {
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

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "anonymous" => Self::Anonymous,
            "user" => Self::User,
            "moderator" | "mod" => Self::Moderator,
            "admin" => Self::Admin,
            "superadmin" | "super_admin" | "super" => Self::SuperAdmin,
            "service" | "svc" => Self::Service,
            "bot" => Self::Bot,
            "bot_owner" | "botowner" | "owner" => Self::BotOwner,
            "bot_operator" | "botoperator" | "operator" => Self::BotOperator,
            "bot_viewer" | "botviewer" | "viewer" => Self::BotViewer,
            _ => Self::Anonymous,
        }
    }

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

impl Default for Role {
    fn default() -> Self {
        Self::Anonymous
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
            username: format!("service:{}", name),
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
            username: format!("bot:{}", bot_name),
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

#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub require_auth: bool,
    pub jwt_secret: Option<String>,
    pub api_key_header: String,
    pub bearer_prefix: String,
    pub session_cookie_name: String,
    pub allow_anonymous_paths: Vec<String>,
    pub public_paths: Vec<String>,
    pub bot_id_header: String,
    pub org_id_header: String,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            require_auth: true,
            jwt_secret: None,
            api_key_header: "X-API-Key".to_string(),
            bearer_prefix: "Bearer ".to_string(),
            session_cookie_name: "session_id".to_string(),
            allow_anonymous_paths: vec![
                "/health".to_string(),
                "/healthz".to_string(),
                "/api/health".to_string(),
                "/api/v1/health".to_string(),
                "/.well-known".to_string(),
                "/metrics".to_string(),
                "/api/auth/login".to_string(),
                "/api/auth/logout".to_string(),
                "/api/auth/refresh".to_string(),
                "/api/auth/bootstrap".to_string(),
                "/api/auth/2fa/verify".to_string(),
                "/api/auth/2fa/resend".to_string(),
                "/oauth".to_string(),
                "/auth/callback".to_string(),
            ],
            public_paths: vec![
                "/".to_string(),
                "/static".to_string(),
                "/favicon.ico".to_string(),
                "/robots.txt".to_string(),
            ],
            bot_id_header: "X-Bot-ID".to_string(),
            org_id_header: "X-Organization-ID".to_string(),
        }
    }
}

impl AuthConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(secret) = std::env::var("JWT_SECRET") {
            config.jwt_secret = Some(secret);
        }

        if let Ok(require) = std::env::var("REQUIRE_AUTH") {
            config.require_auth = require == "true" || require == "1";
        }

        if let Ok(paths) = std::env::var("ANONYMOUS_PATHS") {
            config.allow_anonymous_paths = paths
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }

        config
    }

    pub fn with_jwt_secret(mut self, secret: impl Into<String>) -> Self {
        self.jwt_secret = Some(secret.into());
        self
    }

    pub fn with_require_auth(mut self, require: bool) -> Self {
        self.require_auth = require;
        self
    }

    pub fn add_anonymous_path(mut self, path: impl Into<String>) -> Self {
        self.allow_anonymous_paths.push(path.into());
        self
    }

    pub fn add_public_path(mut self, path: impl Into<String>) -> Self {
        self.public_paths.push(path.into());
        self
    }

    pub fn is_public_path(&self, path: &str) -> bool {
        for public_path in &self.public_paths {
            if path == public_path || path.starts_with(&format!("{}/", public_path)) {
                return true;
            }
        }
        false
    }

    pub fn is_anonymous_allowed(&self, path: &str) -> bool {
        for allowed_path in &self.allow_anonymous_paths {
            if path == allowed_path || path.starts_with(&format!("{}/", allowed_path)) {
                return true;
            }
        }
        false
    }
}

#[derive(Debug)]
pub enum AuthError {
    MissingToken,
    InvalidToken,
    ExpiredToken,
    InsufficientPermissions,
    InvalidApiKey,
    SessionExpired,
    UserNotFound,
    AccountDisabled,
    RateLimited,
    BotAccessDenied,
    BotNotFound,
    OrganizationAccessDenied,
    InternalError(String),
}

impl AuthError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::MissingToken => StatusCode::UNAUTHORIZED,
            Self::InvalidToken => StatusCode::UNAUTHORIZED,
            Self::ExpiredToken => StatusCode::UNAUTHORIZED,
            Self::InsufficientPermissions => StatusCode::FORBIDDEN,
            Self::InvalidApiKey => StatusCode::UNAUTHORIZED,
            Self::SessionExpired => StatusCode::UNAUTHORIZED,
            Self::UserNotFound => StatusCode::UNAUTHORIZED,
            Self::AccountDisabled => StatusCode::FORBIDDEN,
            Self::RateLimited => StatusCode::TOO_MANY_REQUESTS,
            Self::BotAccessDenied => StatusCode::FORBIDDEN,
            Self::BotNotFound => StatusCode::NOT_FOUND,
            Self::OrganizationAccessDenied => StatusCode::FORBIDDEN,
            Self::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn error_code(&self) -> &'static str {
        match self {
            Self::MissingToken => "missing_token",
            Self::InvalidToken => "invalid_token",
            Self::ExpiredToken => "expired_token",
            Self::InsufficientPermissions => "insufficient_permissions",
            Self::InvalidApiKey => "invalid_api_key",
            Self::SessionExpired => "session_expired",
            Self::UserNotFound => "user_not_found",
            Self::AccountDisabled => "account_disabled",
            Self::RateLimited => "rate_limited",
            Self::BotAccessDenied => "bot_access_denied",
            Self::BotNotFound => "bot_not_found",
            Self::OrganizationAccessDenied => "organization_access_denied",
            Self::InternalError(_) => "internal_error",
        }
    }

    pub fn message(&self) -> String {
        match self {
            Self::MissingToken => "Authentication token is required".to_string(),
            Self::InvalidToken => "Invalid authentication token".to_string(),
            Self::ExpiredToken => "Authentication token has expired".to_string(),
            Self::InsufficientPermissions => {
                "You don't have permission to access this resource".to_string()
            }
            Self::InvalidApiKey => "Invalid API key".to_string(),
            Self::SessionExpired => "Your session has expired".to_string(),
            Self::UserNotFound => "User not found".to_string(),
            Self::AccountDisabled => "Your account has been disabled".to_string(),
            Self::RateLimited => "Too many requests, please try again later".to_string(),
            Self::BotAccessDenied => "You don't have access to this bot".to_string(),
            Self::BotNotFound => "Bot not found".to_string(),
            Self::OrganizationAccessDenied => {
                "You don't have access to this organization".to_string()
            }
            Self::InternalError(_) => "An internal error occurred".to_string(),
        }
    }
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let body = Json(json!({
            "error": self.error_code(),
            "message": self.message()
        }));
        (status, body).into_response()
    }
}

pub fn extract_user_from_request(
    request: &Request<Body>,
    config: &AuthConfig,
) -> Result<AuthenticatedUser, AuthError> {
    if let Some(api_key) = request
        .headers()
        .get(&config.api_key_header)
        .and_then(|v| v.to_str().ok())
    {
        let mut user = validate_api_key_sync(api_key)?;

        if let Some(bot_id) = extract_bot_id_from_request(request, config) {
            user = user.with_current_bot(bot_id);
        }

        return Ok(user);
    }

    if let Some(auth_header) = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
    {
        if let Some(token) = auth_header.strip_prefix(&config.bearer_prefix) {
            let mut user = validate_bearer_token_sync(token)?;

            if let Some(bot_id) = extract_bot_id_from_request(request, config) {
                user = user.with_current_bot(bot_id);
            }

            return Ok(user);
        }
    }

    if let Some(session_id) =
        extract_session_from_cookies(request, &config.session_cookie_name)
    {
        let mut user = validate_session_sync(&session_id)?;

        if let Some(bot_id) = extract_bot_id_from_request(request, config) {
            user = user.with_current_bot(bot_id);
        }

        return Ok(user);
    }

    if let Some(user_id) = request
        .headers()
        .get("X-User-ID")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
    {
        let mut user = AuthenticatedUser::new(user_id, "header-user".to_string());

        if let Some(bot_id) = extract_bot_id_from_request(request, config) {
            user = user.with_current_bot(bot_id);
        }

        return Ok(user);
    }

    Err(AuthError::MissingToken)
}

fn extract_bot_id_from_request(request: &Request<Body>, config: &AuthConfig) -> Option<Uuid> {
    request
        .headers()
        .get(&config.bot_id_header)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
}

fn extract_session_from_cookies(request: &Request<Body>, cookie_name: &str) -> Option<String> {
    request
        .headers()
        .get(header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .and_then(|cookies| {
            cookies.split(';').find_map(|cookie| {
                let mut parts = cookie.trim().splitn(2, '=');
                let name = parts.next()?;
                let value = parts.next()?;
                if name == cookie_name {
                    Some(value.to_string())
                } else {
                    None
                }
            })
        })
}

fn validate_api_key_sync(api_key: &str) -> Result<AuthenticatedUser, AuthError> {
    if api_key.is_empty() {
        return Err(AuthError::InvalidApiKey);
    }

    if api_key.len() < 16 {
        return Err(AuthError::InvalidApiKey);
    }

    Ok(AuthenticatedUser::service("api-client").with_metadata("api_key_prefix", &api_key[..8]))
}

fn validate_bearer_token_sync(token: &str) -> Result<AuthenticatedUser, AuthError> {
    if token.is_empty() {
        return Err(AuthError::InvalidToken);
    }

    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err(AuthError::InvalidToken);
    }

    Ok(AuthenticatedUser::new(
        Uuid::new_v4(),
        "jwt-user".to_string(),
    ))
}

fn validate_session_sync(session_id: &str) -> Result<AuthenticatedUser, AuthError> {
    if session_id.is_empty() {
        return Err(AuthError::SessionExpired);
    }

    if Uuid::parse_str(session_id).is_err() && session_id.len() < 32 {
        return Err(AuthError::InvalidToken);
    }

    Ok(
        AuthenticatedUser::new(Uuid::new_v4(), "session-user".to_string())
            .with_session(session_id),
    )
}

#[derive(Clone)]
pub struct AuthMiddlewareState {
    pub config: Arc<AuthConfig>,
    pub provider_registry: Arc<AuthProviderRegistry>,
}

impl AuthMiddlewareState {
    pub fn new(config: Arc<AuthConfig>, provider_registry: Arc<AuthProviderRegistry>) -> Self {
        Self {
            config,
            provider_registry,
        }
    }
}

pub async fn auth_middleware_with_providers(
    mut request: Request<Body>,
    next: Next,
    state: AuthMiddlewareState,
) -> Response {

    let path = request.uri().path().to_string();

    if state.config.is_public_path(&path) || state.config.is_anonymous_allowed(&path) {
        request
            .extensions_mut()
            .insert(AuthenticatedUser::anonymous());
        return next.run(request).await;
    }

    let extracted = ExtractedAuthData::from_request(&request, &state.config);
    let user = authenticate_with_extracted_data(extracted, &state.config, &state.provider_registry).await;

    match user {
        Ok(authenticated_user) => {
            debug!("Authenticated user: {} ({})", authenticated_user.username, authenticated_user.user_id);
            request.extensions_mut().insert(authenticated_user);
            next.run(request).await
        }
        Err(e) => {
            if !state.config.require_auth {
                warn!("Authentication failed but not required, allowing anonymous: {:?}", e);
                request
                    .extensions_mut()
                    .insert(AuthenticatedUser::anonymous());
                return next.run(request).await;
            }
            debug!("Authentication failed: {:?}", e);
            e.into_response()
        }
    }
}

struct ExtractedAuthData {
    api_key: Option<String>,
    bearer_token: Option<String>,
    session_id: Option<String>,
    user_id_header: Option<Uuid>,
    bot_id: Option<Uuid>,
}

impl ExtractedAuthData {
    fn from_request(request: &Request<Body>, config: &AuthConfig) -> Self {
        let api_key = request
            .headers()
            .get(&config.api_key_header)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let bearer_token = request
            .headers()
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.strip_prefix(&config.bearer_prefix))
            .map(|s| s.to_string());

        let session_id = extract_session_from_cookies(request, &config.session_cookie_name);

        let user_id_header = request
            .headers()
            .get("X-User-ID")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| Uuid::parse_str(s).ok());

        let bot_id = extract_bot_id_from_request(request, config);

        Self {
            api_key,
            bearer_token,
            session_id,
            user_id_header,
            bot_id,
        }
    }
}

async fn authenticate_with_extracted_data(
    data: ExtractedAuthData,
    config: &AuthConfig,
    registry: &AuthProviderRegistry,
) -> Result<AuthenticatedUser, AuthError> {
    if let Some(key) = data.api_key {
        let mut user = registry.authenticate_api_key(&key).await?;
        if let Some(bid) = data.bot_id {
            user = user.with_current_bot(bid);
        }
        return Ok(user);
    }

    if let Some(token) = data.bearer_token {
        let mut user = registry.authenticate_token(&token).await?;
        if let Some(bid) = data.bot_id {
            user = user.with_current_bot(bid);
        }
        return Ok(user);
    }

    if let Some(sid) = data.session_id {
        let mut user = validate_session_sync(&sid)?;
        if let Some(bid) = data.bot_id {
            user = user.with_current_bot(bid);
        }
        return Ok(user);
    }

    if let Some(uid) = data.user_id_header {
        let mut user = AuthenticatedUser::new(uid, "header-user".to_string());
        if let Some(bid) = data.bot_id {
            user = user.with_current_bot(bid);
        }
        return Ok(user);
    }

    if !config.require_auth {
        return Ok(AuthenticatedUser::anonymous());
    }

    Err(AuthError::MissingToken)
}

pub async fn extract_user_with_providers(
    request: &Request<Body>,
    config: &AuthConfig,
    registry: &AuthProviderRegistry,
) -> Result<AuthenticatedUser, AuthError> {
    let extracted = ExtractedAuthData::from_request(request, config);
    authenticate_with_extracted_data(extracted, config, registry).await
}

pub async fn auth_middleware(
    State(config): State<std::sync::Arc<AuthConfig>>,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, AuthError> {
    let path = request.uri().path().to_string();

    if config.is_public_path(&path) || config.is_anonymous_allowed(&path) {
        request
            .extensions_mut()
            .insert(AuthenticatedUser::anonymous());
        return Ok(next.run(request).await);
    }

    match extract_user_from_request(&request, &config) {
        Ok(user) => {
            request.extensions_mut().insert(user);
            Ok(next.run(request).await)
        }
        Err(e) => {
            if !config.require_auth {
                request
                    .extensions_mut()
                    .insert(AuthenticatedUser::anonymous());
                return Ok(next.run(request).await);
            }
            Err(e)
        }
    }
}

pub async fn require_auth_middleware(
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, AuthError> {
    let user = request
        .extensions()
        .get::<AuthenticatedUser>()
        .cloned()
        .unwrap_or_else(AuthenticatedUser::anonymous);

    if !user.is_authenticated() {
        return Err(AuthError::MissingToken);
    }

    request.extensions_mut().insert(user);
    Ok(next.run(request).await)
}

pub fn require_permission(
    permission: Permission,
) -> impl Fn(Request<Body>) -> Result<Request<Body>, AuthError> + Clone {
    move |request: Request<Body>| {
        let user = request
            .extensions()
            .get::<AuthenticatedUser>()
            .cloned()
            .unwrap_or_else(AuthenticatedUser::anonymous);

        if !user.has_permission(&permission) {
            return Err(AuthError::InsufficientPermissions);
        }

        Ok(request)
    }
}

pub fn require_role(
    role: Role,
) -> impl Fn(Request<Body>) -> Result<Request<Body>, AuthError> + Clone {
    move |request: Request<Body>| {
        let user = request
            .extensions()
            .get::<AuthenticatedUser>()
            .cloned()
            .unwrap_or_else(AuthenticatedUser::anonymous);

        if !user.has_role(&role) {
            return Err(AuthError::InsufficientPermissions);
        }

        Ok(request)
    }
}

pub fn require_admin() -> impl Fn(Request<Body>) -> Result<Request<Body>, AuthError> + Clone {
    move |request: Request<Body>| {
        let user = request
            .extensions()
            .get::<AuthenticatedUser>()
            .cloned()
            .unwrap_or_else(AuthenticatedUser::anonymous);

        if !user.is_admin() {
            return Err(AuthError::InsufficientPermissions);
        }

        Ok(request)
    }
}

pub fn require_bot_access(
    bot_id: Uuid,
) -> impl Fn(Request<Body>) -> Result<Request<Body>, AuthError> + Clone {
    move |request: Request<Body>| {
        let user = request
            .extensions()
            .get::<AuthenticatedUser>()
            .cloned()
            .unwrap_or_else(AuthenticatedUser::anonymous);

        if !user.can_access_bot(&bot_id) {
            return Err(AuthError::BotAccessDenied);
        }

        Ok(request)
    }
}

pub fn require_bot_permission(
    bot_id: Uuid,
    permission: Permission,
) -> impl Fn(Request<Body>) -> Result<Request<Body>, AuthError> + Clone {
    move |request: Request<Body>| {
        let user = request
            .extensions()
            .get::<AuthenticatedUser>()
            .cloned()
            .unwrap_or_else(AuthenticatedUser::anonymous);

        if !user.has_bot_permission(&bot_id, &permission) {
            return Err(AuthError::InsufficientPermissions);
        }

        Ok(request)
    }
}

pub async fn require_permission_middleware(
    permission: Permission,
    request: Request<Body>,
    next: Next,
) -> Result<Response, AuthError> {
    let user = request
        .extensions()
        .get::<AuthenticatedUser>()
        .cloned()
        .unwrap_or_else(AuthenticatedUser::anonymous);

    if !user.has_permission(&permission) {
        return Err(AuthError::InsufficientPermissions);
    }

    Ok(next.run(request).await)
}

pub async fn require_role_middleware(
    role: Role,
    request: Request<Body>,
    next: Next,
) -> Result<Response, AuthError> {
    let user = request
        .extensions()
        .get::<AuthenticatedUser>()
        .cloned()
        .unwrap_or_else(AuthenticatedUser::anonymous);

    if !user.has_role(&role) {
        return Err(AuthError::InsufficientPermissions);
    }

    Ok(next.run(request).await)
}

pub async fn admin_only_middleware(
    request: Request<Body>,
    next: Next,
) -> Result<Response, AuthError> {
    let user = request
        .extensions()
        .get::<AuthenticatedUser>()
        .cloned()
        .unwrap_or_else(AuthenticatedUser::anonymous);

    if !user.is_admin() {
        return Err(AuthError::InsufficientPermissions);
    }

    Ok(next.run(request).await)
}

pub async fn bot_scope_middleware(
    Path(bot_id): Path<Uuid>,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, AuthError> {
    let user = request
        .extensions()
        .get::<AuthenticatedUser>()
        .cloned()
        .unwrap_or_else(AuthenticatedUser::anonymous);

    if !user.can_access_bot(&bot_id) {
        return Err(AuthError::BotAccessDenied);
    }

    let user = user.with_current_bot(bot_id);
    request.extensions_mut().insert(user);

    Ok(next.run(request).await)
}

pub async fn bot_owner_middleware(
    Path(bot_id): Path<Uuid>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, AuthError> {
    let user = request
        .extensions()
        .get::<AuthenticatedUser>()
        .cloned()
        .unwrap_or_else(AuthenticatedUser::anonymous);

    if !user.can_manage_bot(&bot_id) {
        return Err(AuthError::InsufficientPermissions);
    }

    Ok(next.run(request).await)
}

pub async fn bot_operator_middleware(
    Path(bot_id): Path<Uuid>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, AuthError> {
    let user = request
        .extensions()
        .get::<AuthenticatedUser>()
        .cloned()
        .unwrap_or_else(AuthenticatedUser::anonymous);

    if !user.can_operate_bot(&bot_id) {
        return Err(AuthError::InsufficientPermissions);
    }

    Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_permissions() {
        assert!(!Role::Anonymous.has_permission(&Permission::Read));
        assert!(Role::User.has_permission(&Permission::Read));
        assert!(Role::User.has_permission(&Permission::AccessApi));
        assert!(!Role::User.has_permission(&Permission::Write));

        assert!(Role::Admin.has_permission(&Permission::ManageUsers));
        assert!(Role::Admin.has_permission(&Permission::Delete));

        assert!(Role::SuperAdmin.has_permission(&Permission::ManageSecrets));
    }

    #[test]
    fn test_role_from_str() {
        assert_eq!(Role::from_str("admin"), Role::Admin);
        assert_eq!(Role::from_str("ADMIN"), Role::Admin);
        assert_eq!(Role::from_str("user"), Role::User);
        assert_eq!(Role::from_str("superadmin"), Role::SuperAdmin);
        assert_eq!(Role::from_str("bot_owner"), Role::BotOwner);
        assert_eq!(Role::from_str("unknown"), Role::Anonymous);
    }

    #[test]
    fn test_role_hierarchy() {
        assert!(Role::SuperAdmin.is_at_least(&Role::Admin));
        assert!(Role::Admin.is_at_least(&Role::Moderator));
        assert!(Role::BotOwner.is_at_least(&Role::BotOperator));
        assert!(Role::BotOperator.is_at_least(&Role::BotViewer));
        assert!(!Role::User.is_at_least(&Role::Admin));
    }

    #[test]
    fn test_authenticated_user_builder() {
        let user = AuthenticatedUser::new(Uuid::new_v4(), "testuser".to_string())
            .with_email("test@example.com")
            .with_role(Role::Admin)
            .with_metadata("key", "value");

        assert_eq!(user.email, Some("test@example.com".to_string()));
        assert!(user.has_role(&Role::Admin));
        assert_eq!(user.metadata.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_user_permissions() {
        let admin = AuthenticatedUser::new(Uuid::new_v4(), "admin".to_string())
            .with_role(Role::Admin);

        assert!(admin.has_permission(&Permission::ManageUsers));
        assert!(admin.has_permission(&Permission::Delete));
        assert!(admin.is_admin());

        let user = AuthenticatedUser::new(Uuid::new_v4(), "user".to_string());
        assert!(user.has_permission(&Permission::Read));
        assert!(!user.has_permission(&Permission::ManageUsers));
        assert!(!user.is_admin());
    }

    #[test]
    fn test_anonymous_user() {
        let anon = AuthenticatedUser::anonymous();
        assert!(!anon.is_authenticated());
        assert!(anon.has_role(&Role::Anonymous));
        assert!(!anon.has_permission(&Permission::Read));
    }

    #[test]
    fn test_service_user() {
        let service = AuthenticatedUser::service("scheduler");
        assert!(service.has_role(&Role::Service));
        assert!(service.has_permission(&Permission::ExecuteTasks));
    }

    #[test]
    fn test_bot_user() {
        let bot_id = Uuid::new_v4();
        let bot = AuthenticatedUser::bot_user(bot_id, "test-bot");
        assert!(bot.is_bot());
        assert!(bot.has_permission(&Permission::SendMessages));
        assert_eq!(bot.current_bot_id, Some(bot_id));
    }

    #[test]
    fn test_auth_config_paths() {
        let config = AuthConfig::default();

        assert!(config.is_anonymous_allowed("/health"));
        assert!(config.is_anonymous_allowed("/api/health"));
        assert!(!config.is_anonymous_allowed("/api/users"));

        assert!(config.is_public_path("/static"));
        assert!(config.is_public_path("/static/css/style.css"));
        assert!(!config.is_public_path("/api/private"));
    }

    #[test]
    fn test_auth_error_responses() {
        assert_eq!(AuthError::MissingToken.status_code(), StatusCode::UNAUTHORIZED);
        assert_eq!(AuthError::InsufficientPermissions.status_code(), StatusCode::FORBIDDEN);
        assert_eq!(AuthError::RateLimited.status_code(), StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(AuthError::BotAccessDenied.status_code(), StatusCode::FORBIDDEN);
    }

    #[test]
    fn test_bot_access() {
        let bot_id = Uuid::new_v4();
        let other_bot_id = Uuid::new_v4();

        let user = AuthenticatedUser::new(Uuid::new_v4(), "user".to_string())
            .with_bot_access(BotAccess::viewer(bot_id));

        assert!(user.can_access_bot(&bot_id));
        assert!(user.can_view_bot(&bot_id));
        assert!(!user.can_operate_bot(&bot_id));
        assert!(!user.can_manage_bot(&bot_id));
        assert!(!user.can_access_bot(&other_bot_id));

        let admin = AuthenticatedUser::new(Uuid::new_v4(), "admin".to_string())
            .with_role(Role::Admin);

        assert!(admin.can_access_bot(&bot_id));
        assert!(admin.can_access_bot(&other_bot_id));
    }

    #[test]
    fn test_bot_owner_access() {
        let bot_id = Uuid::new_v4();

        let owner = AuthenticatedUser::new(Uuid::new_v4(), "owner".to_string())
            .with_bot_access(BotAccess::owner(bot_id));

        assert!(owner.can_access_bot(&bot_id));
        assert!(owner.can_view_bot(&bot_id));
        assert!(owner.can_operate_bot(&bot_id));
        assert!(owner.can_manage_bot(&bot_id));
    }

    #[test]
    fn test_bot_operator_access() {
        let bot_id = Uuid::new_v4();

        let operator = AuthenticatedUser::new(Uuid::new_v4(), "operator".to_string())
            .with_bot_access(BotAccess::operator(bot_id));

        assert!(operator.can_access_bot(&bot_id));
        assert!(operator.can_view_bot(&bot_id));
        assert!(operator.can_operate_bot(&bot_id));
        assert!(!operator.can_manage_bot(&bot_id));
    }

    #[test]
    fn test_bot_permission_check() {
        let bot_id = Uuid::new_v4();

        let operator = AuthenticatedUser::new(Uuid::new_v4(), "operator".to_string())
            .with_bot_access(BotAccess::operator(bot_id));

        assert!(operator.has_bot_permission(&bot_id, &Permission::SendMessages));
        assert!(operator.has_bot_permission(&bot_id, &Permission::ViewAnalytics));
        assert!(!operator.has_bot_permission(&bot_id, &Permission::ManageBots));
    }

    #[test]
    fn test_bot_access_expiry() {
        let bot_id = Uuid::new_v4();
        let past_time = chrono::Utc::now().timestamp() - 3600;

        let expired_access = BotAccess::viewer(bot_id).with_expiry(past_time);
        assert!(expired_access.is_expired());
        assert!(!expired_access.is_valid());

        let future_time = chrono::Utc::now().timestamp() + 3600;
        let valid_access = BotAccess::viewer(bot_id).with_expiry(future_time);
        assert!(!valid_access.is_expired());
        assert!(valid_access.is_valid());
    }

    #[test]
    fn test_accessible_bot_ids() {
        let bot1 = Uuid::new_v4();
        let bot2 = Uuid::new_v4();

        let user = AuthenticatedUser::new(Uuid::new_v4(), "user".to_string())
            .with_bot_access(BotAccess::owner(bot1))
            .with_bot_access(BotAccess::viewer(bot2));

        let accessible = user.accessible_bot_ids();
        assert_eq!(accessible.len(), 2);
        assert!(accessible.contains(&bot1));
        assert!(accessible.contains(&bot2));

        let owned = user.owned_bot_ids();
        assert_eq!(owned.len(), 1);
        assert!(owned.contains(&bot1));
    }

    #[test]
    fn test_organization_access() {
        let org_id = Uuid::new_v4();
        let other_org_id = Uuid::new_v4();

        let user = AuthenticatedUser::new(Uuid::new_v4(), "user".to_string())
            .with_organization(org_id);

        assert!(user.can_access_organization(&org_id));
        assert!(!user.can_access_organization(&other_org_id));
    }

    #[test]
    fn test_has_any_permission() {
        let user = AuthenticatedUser::new(Uuid::new_v4(), "user".to_string());

        assert!(user.has_any_permission(&[Permission::Read, Permission::Write]));
        assert!(!user.has_any_permission(&[Permission::Delete, Permission::Admin]));
    }

    #[test]
    fn test_has_all_permissions() {
        let admin = AuthenticatedUser::new(Uuid::new_v4(), "admin".to_string())
            .with_role(Role::Admin);

        assert!(admin.has_all_permissions(&[Permission::Read, Permission::Write, Permission::Delete]));
        assert!(!admin.has_all_permissions(&[Permission::ManageSecrets]));
    }

    #[test]
    fn test_highest_role() {
        let user = AuthenticatedUser::new(Uuid::new_v4(), "user".to_string())
            .with_role(Role::Admin)
            .with_role(Role::Moderator);

        assert_eq!(user.highest_role(), &Role::Admin);
    }
}
