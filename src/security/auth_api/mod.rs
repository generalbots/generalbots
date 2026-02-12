//! Authentication and authorization API module
//!
//! This module provides a comprehensive authentication and authorization system
//! with support for roles, permissions, bot access control, and multiple
//! authentication methods (API keys, JWT tokens, sessions).

pub mod config;
pub mod error;
pub mod middleware;
pub mod tests;
pub mod types;
pub mod utils;

// Re-export commonly used types at the module level
pub use config::AuthConfig;
pub use error::AuthError;
pub use middleware::{
    admin_only_middleware, auth_middleware, auth_middleware_with_providers,
    bot_operator_middleware, bot_owner_middleware, bot_scope_middleware,
    require_admin, require_auth_middleware, require_bot_access,
    require_bot_permission, require_permission, require_permission_middleware,
    require_role, require_role_middleware, AuthMiddlewareState,
};
pub use types::{
    AuthenticatedUser, BotAccess, Permission, Role,
};
pub use utils::{
    extract_bot_id_from_request, extract_session_from_cookies,
    extract_user_from_request, extract_user_with_providers, is_jwt_format,
    validate_session_sync,
};
