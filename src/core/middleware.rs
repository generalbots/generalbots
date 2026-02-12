use axum::{
    body::Body,
    extract::{FromRequestParts, State},
    http::{header::AUTHORIZATION, request::Parts, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[cfg(any(feature = "research", feature = "llm"))]
use crate::core::kb::permissions::{build_qdrant_permission_filter, UserContext};
use crate::core::shared::utils::DbPool;

// ============================================================================
// Organization Context
// ============================================================================

/// Organization context extracted from request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationContext {
    pub organization_id: Uuid,
    pub organization_name: Option<String>,
    pub plan_id: Option<String>,
    pub is_owner: bool,
    pub permissions: Vec<String>,
}

impl OrganizationContext {
    pub fn new(organization_id: Uuid) -> Self {
        Self {
            organization_id,
            organization_name: None,
            plan_id: None,
            is_owner: false,
            permissions: Vec::new(),
        }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.organization_name = Some(name);
        self
    }

    pub fn with_plan(mut self, plan_id: String) -> Self {
        self.plan_id = Some(plan_id);
        self
    }

    pub fn as_owner(mut self) -> Self {
        self.is_owner = true;
        self
    }

    pub fn with_permissions(mut self, permissions: Vec<String>) -> Self {
        self.permissions = permissions;
        self
    }

    pub fn has_permission(&self, permission: &str) -> bool {
        self.is_owner || self.permissions.contains(&permission.to_string())
    }
}

// ============================================================================
// User Context (Authentication)
// ============================================================================

/// Authenticated user context extracted from request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticatedUser {
    pub user_id: Uuid,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub roles: Vec<String>,
    pub groups: Vec<String>,
    pub organization_id: Option<Uuid>,
    pub is_admin: bool,
    pub is_super_admin: bool,
    pub token_claims: Option<TokenClaims>,
}

impl AuthenticatedUser {
    pub fn new(user_id: Uuid) -> Self {
        Self {
            user_id,
            email: None,
            display_name: None,
            roles: Vec::new(),
            groups: Vec::new(),
            organization_id: None,
            is_admin: false,
            is_super_admin: false,
            token_claims: None,
        }
    }

    pub fn anonymous() -> Self {
        Self {
            user_id: Uuid::nil(),
            email: None,
            display_name: Some("Anonymous".to_string()),
            roles: vec!["anonymous".to_string()],
            groups: Vec::new(),
            organization_id: None,
            is_admin: false,
            is_super_admin: false,
            token_claims: None,
        }
    }

    pub fn with_email(mut self, email: String) -> Self {
        self.email = Some(email);
        self
    }

    pub fn with_roles(mut self, roles: Vec<String>) -> Self {
        self.roles = roles;
        self
    }

    pub fn with_groups(mut self, groups: Vec<String>) -> Self {
        self.groups = groups;
        self
    }

    pub fn with_organization(mut self, org_id: Uuid) -> Self {
        self.organization_id = Some(org_id);
        self
    }

    pub fn as_admin(mut self) -> Self {
        self.is_admin = true;
        self
    }

    pub fn has_role(&self, role: &str) -> bool {
        self.is_super_admin || self.roles.contains(&role.to_string())
    }

    pub fn has_any_role(&self, roles: &[&str]) -> bool {
        self.is_super_admin || roles.iter().any(|r| self.roles.contains(&r.to_string()))
    }

    pub fn has_group(&self, group: &str) -> bool {
        self.groups.contains(&group.to_string())
    }

    pub fn is_authenticated(&self) -> bool {
        !self.user_id.is_nil()
    }

    /// Convert to UserContext for KB permission checks
    #[cfg(any(feature = "research", feature = "llm"))]
    pub fn to_user_context(&self) -> UserContext {
        if self.is_authenticated() {
            UserContext::authenticated(self.user_id, self.email.clone(), self.organization_id)
                .with_roles(self.roles.clone())
                .with_groups(self.groups.clone())
        } else {
            UserContext::anonymous()
        }
    }

    /// Get Qdrant permission filter for this user
    #[cfg(any(feature = "research", feature = "llm"))]
    pub fn get_qdrant_filter(&self) -> serde_json::Value {
        build_qdrant_permission_filter(&self.to_user_context())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,
    pub exp: i64,
    pub iat: i64,
    pub iss: Option<String>,
    pub aud: Option<Vec<String>>,
    pub scope: Option<String>,
}

// ============================================================================
// Request Context (Combined)
// ============================================================================

/// Combined request context with organization and user information
#[derive(Debug, Clone)]
pub struct RequestContext {
    pub user: AuthenticatedUser,
    pub organization: Option<OrganizationContext>,
    pub request_id: Uuid,
    pub client_ip: Option<String>,
    pub user_agent: Option<String>,
    pub bot_id: Option<Uuid>,
    pub conversation_id: Option<Uuid>,
}

impl RequestContext {
    pub fn new(user: AuthenticatedUser) -> Self {
        Self {
            user,
            organization: None,
            request_id: Uuid::new_v4(),
            client_ip: None,
            user_agent: None,
            bot_id: None,
            conversation_id: None,
        }
    }

    pub fn with_organization(mut self, org: OrganizationContext) -> Self {
        self.organization = Some(org);
        self
    }

    pub fn with_client_info(mut self, ip: Option<String>, user_agent: Option<String>) -> Self {
        self.client_ip = ip;
        self.user_agent = user_agent;
        self
    }

    pub fn with_bot(mut self, bot_id: Uuid) -> Self {
        self.bot_id = Some(bot_id);
        self
    }

    pub fn with_conversation(mut self, conversation_id: Uuid) -> Self {
        self.conversation_id = Some(conversation_id);
        self
    }

    /// Check if user can access a specific organization
    pub fn can_access_organization(&self, org_id: Uuid) -> bool {
        if self.user.is_super_admin {
            return true;
        }

        self.organization
            .as_ref()
            .map(|o| o.organization_id == org_id)
            .unwrap_or(false)
    }

    /// Check if user has permission within current organization
    pub fn has_org_permission(&self, permission: &str) -> bool {
        if self.user.is_super_admin {
            return true;
        }

        self.organization
            .as_ref()
            .map(|o| o.has_permission(permission))
            .unwrap_or(false)
    }

    /// Get organization ID if available
    pub fn org_id(&self) -> Option<Uuid> {
        self.organization.as_ref().map(|o| o.organization_id)
    }
}

// ============================================================================
// Middleware State
// ============================================================================

#[derive(Clone)]
pub struct ContextMiddlewareState {
    pub db_pool: DbPool,
    pub jwt_secret: Arc<String>,
    pub org_cache: Arc<RwLock<std::collections::HashMap<Uuid, CachedOrganization>>>,
    pub user_cache: Arc<RwLock<std::collections::HashMap<Uuid, CachedUserData>>>,
    pub cache_ttl_seconds: u64,
}

#[derive(Clone)]
pub struct CachedOrganization {
    pub context: OrganizationContext,
    pub cached_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone)]
pub struct CachedUserData {
    pub roles: Vec<String>,
    pub groups: Vec<String>,
    pub cached_at: chrono::DateTime<chrono::Utc>,
}

impl ContextMiddlewareState {
    pub fn new(db_pool: DbPool, jwt_secret: String) -> Self {
        Self {
            db_pool,
            jwt_secret: Arc::new(jwt_secret),
            org_cache: Arc::new(RwLock::new(std::collections::HashMap::new())),
            user_cache: Arc::new(RwLock::new(std::collections::HashMap::new())),
            cache_ttl_seconds: 300,
        }
    }

    pub fn with_cache_ttl(mut self, ttl_seconds: u64) -> Self {
        self.cache_ttl_seconds = ttl_seconds;
        self
    }

    async fn get_organization_context(&self, org_id: Uuid) -> Option<OrganizationContext> {
        {
            let cache = self.org_cache.read().await;
            if let Some(cached) = cache.get(&org_id) {
                let age = chrono::Utc::now()
                    .signed_duration_since(cached.cached_at)
                    .num_seconds() as u64;
                if age < self.cache_ttl_seconds {
                    return Some(cached.context.clone());
                }
            }
        }

        let context = OrganizationContext::new(org_id)
            .with_name("Organization".to_string())
            .with_plan("free".to_string());

        {
            let mut cache = self.org_cache.write().await;
            cache.insert(
                org_id,
                CachedOrganization {
                    context: context.clone(),
                    cached_at: chrono::Utc::now(),
                },
            );
        }

        Some(context)
    }

    async fn get_user_roles_groups(
        &self,
        user_id: Uuid,
        _org_id: Option<Uuid>,
    ) -> (Vec<String>, Vec<String>) {
        {
            let cache = self.user_cache.read().await;
            if let Some(cached) = cache.get(&user_id) {
                let age = chrono::Utc::now()
                    .signed_duration_since(cached.cached_at)
                    .num_seconds() as u64;
                if age < self.cache_ttl_seconds {
                    return (cached.roles.clone(), cached.groups.clone());
                }
            }
        }

        let roles = vec!["member".to_string()];
        let groups = Vec::new();

        {
            let mut cache = self.user_cache.write().await;
            cache.insert(
                user_id,
                CachedUserData {
                    roles: roles.clone(),
                    groups: groups.clone(),
                    cached_at: chrono::Utc::now(),
                },
            );
        }

        (roles, groups)
    }

    pub async fn clear_org_cache(&self, org_id: Uuid) {
        let mut cache = self.org_cache.write().await;
        cache.remove(&org_id);
    }

    pub async fn clear_user_cache(&self, user_id: Uuid) {
        let mut cache = self.user_cache.write().await;
        cache.remove(&user_id);
    }

    pub async fn clear_all_caches(&self) {
        {
            let mut cache = self.org_cache.write().await;
            cache.clear();
        }
        {
            let mut cache = self.user_cache.write().await;
            cache.clear();
        }
    }
}

pub async fn organization_context_middleware(
    State(state): State<Arc<ContextMiddlewareState>>,
    mut request: Request<Body>,
    next: Next,
) -> Response {
    let org_id = extract_organization_id(&request);

    if let Some(org_id) = org_id {
        if let Some(context) = state.get_organization_context(org_id).await {
            request.extensions_mut().insert(context);
        }
    }

    next.run(request).await
}

/// Extract and validate user authentication, adding context to extensions
pub async fn authentication_middleware(
    State(state): State<Arc<ContextMiddlewareState>>,
    mut request: Request<Body>,
    next: Next,
) -> Response {
    let user = match extract_and_validate_user(&request, &state).await {
        Ok(user) => user,
        Err(_) => AuthenticatedUser::anonymous(),
    };

    // If authenticated, fetch roles and groups
    let user = if user.is_authenticated() {
        let org_id = user.organization_id.or_else(|| {
            request
                .extensions()
                .get::<OrganizationContext>()
                .map(|o| o.organization_id)
        });

        let (roles, groups) = state.get_user_roles_groups(user.user_id, org_id).await;
        user.with_roles(roles).with_groups(groups)
    } else {
        user
    };

    request.extensions_mut().insert(user);
    next.run(request).await
}

/// Combine organization and user context into RequestContext
pub async fn request_context_middleware(mut request: Request<Body>, next: Next) -> Response {
    let user = request
        .extensions()
        .get::<AuthenticatedUser>()
        .cloned()
        .unwrap_or_else(AuthenticatedUser::anonymous);

    let organization = request.extensions().get::<OrganizationContext>().cloned();

    let client_ip = extract_client_ip(&request);
    let user_agent = request
        .headers()
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(String::from);

    let bot_id = extract_bot_id(&request);
    let conversation_id = extract_conversation_id(&request);

    let mut context = RequestContext::new(user).with_client_info(client_ip, user_agent);

    if let Some(org) = organization {
        context = context.with_organization(org);
    }

    if let Some(bot_id) = bot_id {
        context = context.with_bot(bot_id);
    }

    if let Some(conv_id) = conversation_id {
        context = context.with_conversation(conv_id);
    }

    request.extensions_mut().insert(context);
    next.run(request).await
}

/// Require authentication - returns 401 if not authenticated
pub async fn require_authentication_middleware(
    request: Request<Body>,
    next: Next,
) -> Result<Response, Response> {
    let user = request
        .extensions()
        .get::<AuthenticatedUser>()
        .cloned()
        .unwrap_or_else(AuthenticatedUser::anonymous);

    if !user.is_authenticated() {
        return Err(UnauthorizedResponse::new("Authentication required").into_response());
    }

    Ok(next.run(request).await)
}

/// Require specific role - returns 403 if role not present
pub fn require_role_middleware(
    required_role: &'static str,
) -> impl Fn(Request<Body>, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, Response>> + Send>>
       + Clone
       + Send {
    move |request: Request<Body>, next: Next| {
        Box::pin(async move {
            let user = request
                .extensions()
                .get::<AuthenticatedUser>()
                .cloned()
                .unwrap_or_else(AuthenticatedUser::anonymous);

            if !user.has_role(required_role) {
                return Err(ForbiddenResponse::new(&format!(
                    "Required role: {}",
                    required_role
                ))
                .into_response());
            }

            Ok(next.run(request).await)
        })
    }
}

/// Require organization context - returns 400 if no organization
pub async fn require_organization_middleware(
    request: Request<Body>,
    next: Next,
) -> Result<Response, Response> {
    let has_org = request.extensions().get::<OrganizationContext>().is_some();

    if !has_org {
        return Err(BadRequestResponse::new("Organization context required").into_response());
    }

    Ok(next.run(request).await)
}

/// Require admin role within organization
pub async fn require_org_admin_middleware(
    request: Request<Body>,
    next: Next,
) -> Result<Response, Response> {
    let context = request.extensions().get::<RequestContext>().cloned();

    let is_admin = context
        .as_ref()
        .map(|c| c.user.is_admin || c.user.is_super_admin || c.has_org_permission("admin"))
        .unwrap_or(false);

    if !is_admin {
        return Err(ForbiddenResponse::new("Organization admin access required").into_response());
    }

    Ok(next.run(request).await)
}

// ============================================================================
// Extractors
// ============================================================================

/// Extract organization ID from various sources
fn extract_organization_id(request: &Request<Body>) -> Option<Uuid> {
    // Try header first
    if let Some(org_header) = request.headers().get("X-Organization-Id") {
        if let Ok(org_str) = org_header.to_str() {
            if let Ok(org_id) = Uuid::parse_str(org_str) {
                return Some(org_id);
            }
        }
    }

    // Try query parameter
    if let Some(query) = request.uri().query() {
        for pair in query.split('&') {
            if let Some((key, value)) = pair.split_once('=') {
                if key == "org_id" || key == "organization_id" || key == "orgId" {
                    if let Ok(org_id) = Uuid::parse_str(value) {
                        return Some(org_id);
                    }
                }
            }
        }
    }

    // Try path parameter (for routes like /organizations/{id}/...)
    let path = request.uri().path();
    if path.contains("/organizations/") || path.contains("/orgs/") {
        let parts: Vec<&str> = path.split('/').collect();
        for (i, part) in parts.iter().enumerate() {
            if (*part == "organizations" || *part == "orgs") && i + 1 < parts.len() {
                if let Ok(org_id) = Uuid::parse_str(parts[i + 1]) {
                    return Some(org_id);
                }
            }
        }
    }

    None
}

/// Extract bot ID from request
fn extract_bot_id(request: &Request<Body>) -> Option<Uuid> {
    // Try header
    if let Some(header) = request.headers().get("X-Bot-Id") {
        if let Ok(s) = header.to_str() {
            if let Ok(id) = Uuid::parse_str(s) {
                return Some(id);
            }
        }
    }

    // Try path
    let path = request.uri().path();
    if path.contains("/bots/") {
        let parts: Vec<&str> = path.split('/').collect();
        for (i, part) in parts.iter().enumerate() {
            if *part == "bots" && i + 1 < parts.len() {
                if let Ok(id) = Uuid::parse_str(parts[i + 1]) {
                    return Some(id);
                }
            }
        }
    }

    None
}

/// Extract conversation ID from request
fn extract_conversation_id(request: &Request<Body>) -> Option<Uuid> {
    // Try header
    if let Some(header) = request.headers().get("X-Conversation-Id") {
        if let Ok(s) = header.to_str() {
            if let Ok(id) = Uuid::parse_str(s) {
                return Some(id);
            }
        }
    }

    // Try path
    let path = request.uri().path();
    if path.contains("/conversations/") {
        let parts: Vec<&str> = path.split('/').collect();
        for (i, part) in parts.iter().enumerate() {
            if *part == "conversations" && i + 1 < parts.len() {
                if let Ok(id) = Uuid::parse_str(parts[i + 1]) {
                    return Some(id);
                }
            }
        }
    }

    None
}

/// Extract client IP from request headers
fn extract_client_ip(request: &Request<Body>) -> Option<String> {
    // Try common proxy headers
    for header_name in &[
        "X-Forwarded-For",
        "X-Real-IP",
        "CF-Connecting-IP",
        "True-Client-IP",
    ] {
        if let Some(header) = request.headers().get(*header_name) {
            if let Ok(value) = header.to_str() {
                // X-Forwarded-For can contain multiple IPs
                let ip = value.split(',').next().map(|s| s.trim().to_string());
                if ip.is_some() {
                    return ip;
                }
            }
        }
    }

    None
}

/// Extract and validate user from authorization header
async fn extract_and_validate_user(
    request: &Request<Body>,
    state: &ContextMiddlewareState,
) -> Result<AuthenticatedUser, AuthError> {
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or(AuthError::MissingToken)?;

    let token = if let Some(stripped) = auth_header.strip_prefix("Bearer ") {
        stripped
    } else {
        return Err(AuthError::InvalidFormat);
    };

    // Validate JWT token
    let claims = validate_jwt(token, &state.jwt_secret)?;

    let user_id =
        Uuid::parse_str(&claims.sub).map_err(|_| AuthError::InvalidToken("Invalid user ID".to_string()))?;

    let user = AuthenticatedUser::new(user_id).with_email(claims.sub.clone());

    Ok(AuthenticatedUser {
        token_claims: Some(claims),
        ..user
    })
}

/// Validate JWT token and extract claims using jsonwebtoken crate
fn validate_jwt(token: &str, secret: &str) -> Result<TokenClaims, AuthError> {
    // Configure validation rules
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;
    validation.validate_nbf = false;
    validation.set_required_spec_claims(&["sub", "exp"]);

    // Also accept RS256 tokens (common with OIDC providers like Zitadel)
    // Try HS256 first, then RS256 if that fails
    let decoding_key = DecodingKey::from_secret(secret.as_bytes());

    match decode::<TokenClaims>(token, &decoding_key, &validation) {
        Ok(token_data) => Ok(token_data.claims),
        Err(e) => {
            // If HS256 fails, try decoding without signature verification
            // This handles cases where the token is from an external OIDC provider
            // and we just need to read the claims (signature already verified upstream)
            match e.kind() {
                jsonwebtoken::errors::ErrorKind::InvalidSignature => {
                    // Try RS256 with the secret as a PEM key
                    let mut rs_validation = Validation::new(Algorithm::RS256);
                    rs_validation.validate_exp = true;
                    rs_validation.validate_nbf = false;
                    rs_validation.set_required_spec_claims(&["sub", "exp"]);

                    // If secret looks like a PEM key, try to decode with it
                    if secret.contains("-----BEGIN") {
                        if let Ok(rsa_key) = DecodingKey::from_rsa_pem(secret.as_bytes()) {
                            if let Ok(token_data) = decode::<TokenClaims>(token, &rsa_key, &rs_validation) {
                                return Ok(token_data.claims);
                            }
                        }
                    }

                    // Fallback: decode without validation for trusted internal tokens
                    // Only do this if JWT_SKIP_VALIDATION env var is set
                    if std::env::var("JWT_SKIP_VALIDATION").is_ok() {
                        let mut insecure_validation = Validation::new(Algorithm::HS256);
                        insecure_validation.insecure_disable_signature_validation();
                        insecure_validation.validate_exp = true;
                        insecure_validation.set_required_spec_claims(&["sub", "exp"]);

                        if let Ok(token_data) = decode::<TokenClaims>(token, &DecodingKey::from_secret(&[]), &insecure_validation) {
                            return Ok(token_data.claims);
                        }
                    }

                    Err(AuthError::InvalidToken(format!("Invalid signature: {}", e)))
                }
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                    Err(AuthError::TokenExpired)
                }
                jsonwebtoken::errors::ErrorKind::InvalidToken => {
                    Err(AuthError::InvalidToken("Malformed token".to_string()))
                }
                jsonwebtoken::errors::ErrorKind::InvalidIssuer => {
                    Err(AuthError::InvalidToken("Invalid issuer".to_string()))
                }
                jsonwebtoken::errors::ErrorKind::InvalidAudience => {
                    Err(AuthError::InvalidToken("Invalid audience".to_string()))
                }
                jsonwebtoken::errors::ErrorKind::InvalidSubject => {
                    Err(AuthError::InvalidToken("Invalid subject".to_string()))
                }
                jsonwebtoken::errors::ErrorKind::MissingRequiredClaim(claim) => {
                    Err(AuthError::InvalidToken(format!("Missing required claim: {}", claim)))
                }
                _ => {
                    Err(AuthError::InvalidToken(format!("Token validation failed: {}", e)))
                }
            }
        }
    }
}

#[derive(Debug)]
enum AuthError {
    MissingToken,
    InvalidFormat,
    InvalidToken(String),
    TokenExpired,
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingToken => write!(f, "Missing authorization token"),
            Self::InvalidFormat => write!(f, "Invalid authorization format"),
            Self::InvalidToken(msg) => write!(f, "Invalid token: {msg}"),
            Self::TokenExpired => write!(f, "Token expired"),
        }
    }
}

// ============================================================================
// Response Types
// ============================================================================

struct UnauthorizedResponse {
    message: String,
}

impl UnauthorizedResponse {
    fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

impl IntoResponse for UnauthorizedResponse {
    fn into_response(self) -> Response {
        let body = serde_json::json!({
            "error": "unauthorized",
            "message": self.message,
            "code": "UNAUTHORIZED"
        });

        (
            StatusCode::UNAUTHORIZED,
            [
                ("Content-Type", "application/json"),
                ("WWW-Authenticate", "Bearer"),
            ],
            Json(body),
        )
            .into_response()
    }
}

struct ForbiddenResponse {
    message: String,
}

impl ForbiddenResponse {
    fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

impl IntoResponse for ForbiddenResponse {
    fn into_response(self) -> Response {
        let body = serde_json::json!({
            "error": "forbidden",
            "message": self.message,
            "code": "FORBIDDEN"
        });

        (StatusCode::FORBIDDEN, Json(body)).into_response()
    }
}

struct BadRequestResponse {
    message: String,
}

impl BadRequestResponse {
    fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

impl IntoResponse for BadRequestResponse {
    fn into_response(self) -> Response {
        let body = serde_json::json!({
            "error": "bad_request",
            "message": self.message,
            "code": "BAD_REQUEST"
        });

        (StatusCode::BAD_REQUEST, Json(body)).into_response()
    }
}

// ============================================================================
// Axum Extractors
// ============================================================================

/// Axum extractor for RequestContext
#[axum::async_trait]
impl<S> FromRequestParts<S> for RequestContext
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts.extensions.get::<RequestContext>().cloned().ok_or((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": "Request context not available"
            })),
        ))
    }
}

/// Axum extractor for AuthenticatedUser
#[axum::async_trait]
impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthenticatedUser>()
            .cloned()
            .ok_or((
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": "Authentication required"
                })),
            ))
    }
}

/// Axum extractor for OrganizationContext
#[axum::async_trait]
impl<S> FromRequestParts<S> for OrganizationContext
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<OrganizationContext>()
            .cloned()
            .ok_or((
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "Organization context required"
                })),
            ))
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Create middleware state with database pool
pub fn create_context_middleware_state(
    db_pool: DbPool,
    jwt_secret: String,
) -> Arc<ContextMiddlewareState> {
    Arc::new(ContextMiddlewareState::new(db_pool, jwt_secret))
}

/// Check if user can access a specific resource
pub fn can_access_resource(
    context: &RequestContext,
    resource_org_id: Option<Uuid>,
    required_permission: Option<&str>,
) -> bool {
    // Super admin can access everything
    if context.user.is_super_admin {
        return true;
    }

    // Check organization match
    if let Some(res_org_id) = resource_org_id {
        if !context.can_access_organization(res_org_id) {
            return false;
        }
    }

    // Check permission if required
    if let Some(permission) = required_permission {
        if !context.has_org_permission(permission) {
            return false;
        }
    }

    true
}

/// Build permission filter for Qdrant searches based on user context
#[cfg(any(feature = "research", feature = "llm"))]
pub fn build_search_permission_filter(context: &RequestContext) -> serde_json::Value {
    context.user.get_qdrant_filter()
}

pub async fn validate_org_membership(
    _db_pool: &DbPool,
    _user_id: Uuid,
    _org_id: Uuid,
) -> Result<bool, String> {
    Ok(true)
}

pub async fn get_user_org_role(
    _db_pool: &DbPool,
    _user_id: Uuid,
    _org_id: Uuid,
) -> Result<Option<String>, String> {
    Ok(Some("member".to_string()))
}

/// Standard organization roles
pub struct OrgRoles;

impl OrgRoles {
    pub const OWNER: &'static str = "owner";
    pub const ADMIN: &'static str = "admin";
    pub const MEMBER: &'static str = "member";
    pub const VIEWER: &'static str = "viewer";
    pub const GUEST: &'static str = "guest";
}

/// Standard permissions
pub struct Permissions;

impl Permissions {
    // Organization permissions
    pub const ORG_MANAGE: &'static str = "org:manage";
    pub const ORG_BILLING: &'static str = "org:billing";
    pub const ORG_MEMBERS: &'static str = "org:members";
    pub const ORG_SETTINGS: &'static str = "org:settings";

    // Bot permissions
    pub const BOT_CREATE: &'static str = "bot:create";
    pub const BOT_EDIT: &'static str = "bot:edit";
    pub const BOT_DELETE: &'static str = "bot:delete";
    pub const BOT_PUBLISH: &'static str = "bot:publish";

    // KB permissions
    pub const KB_READ: &'static str = "kb:read";
    pub const KB_WRITE: &'static str = "kb:write";
    pub const KB_ADMIN: &'static str = "kb:admin";

    // App permissions
    pub const APP_CREATE: &'static str = "app:create";
    pub const APP_EDIT: &'static str = "app:edit";
    pub const APP_DELETE: &'static str = "app:delete";

    // Analytics permissions
    pub const ANALYTICS_VIEW: &'static str = "analytics:view";
    pub const ANALYTICS_EXPORT: &'static str = "analytics:export";
}

/// Default permissions for each role
pub fn default_permissions_for_role(role: &str) -> Vec<&'static str> {
    match role {
        "owner" => vec![
            Permissions::ORG_MANAGE,
            Permissions::ORG_BILLING,
            Permissions::ORG_MEMBERS,
            Permissions::ORG_SETTINGS,
            Permissions::BOT_CREATE,
            Permissions::BOT_EDIT,
            Permissions::BOT_DELETE,
            Permissions::BOT_PUBLISH,
            Permissions::KB_READ,
            Permissions::KB_WRITE,
            Permissions::KB_ADMIN,
            Permissions::APP_CREATE,
            Permissions::APP_EDIT,
            Permissions::APP_DELETE,
            Permissions::ANALYTICS_VIEW,
            Permissions::ANALYTICS_EXPORT,
        ],
        "admin" => vec![
            Permissions::ORG_MEMBERS,
            Permissions::ORG_SETTINGS,
            Permissions::BOT_CREATE,
            Permissions::BOT_EDIT,
            Permissions::BOT_DELETE,
            Permissions::BOT_PUBLISH,
            Permissions::KB_READ,
            Permissions::KB_WRITE,
            Permissions::KB_ADMIN,
            Permissions::APP_CREATE,
            Permissions::APP_EDIT,
            Permissions::APP_DELETE,
            Permissions::ANALYTICS_VIEW,
            Permissions::ANALYTICS_EXPORT,
        ],
        "member" => vec![
            Permissions::BOT_CREATE,
            Permissions::BOT_EDIT,
            Permissions::KB_READ,
            Permissions::KB_WRITE,
            Permissions::APP_CREATE,
            Permissions::APP_EDIT,
            Permissions::ANALYTICS_VIEW,
        ],
        "viewer" => vec![Permissions::KB_READ, Permissions::ANALYTICS_VIEW],
        "guest" => vec![Permissions::KB_READ],
        _ => vec![],
    }
}
