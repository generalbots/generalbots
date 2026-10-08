use super::*;

/// RBAC middleware function for use with middleware::from_fn
/// This version takes the RbacManager as a parameter instead of State
pub async fn rbac_middleware_fn(
    request: Request<Body>,
    next: Next,
    rbac: Arc<RbacManager>,
) -> Response {
    let path = request.uri().path().to_string();
    let method = request.method().to_string();

    // If auth middleware already marked this path as public, skip RBAC check
    if request.extensions().get::<PublicPathAllowed>().is_some() {
        debug!("RBAC: path allowed as public/anonymous by auth middleware: {}", path);
        return next.run(request).await;
    }

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
