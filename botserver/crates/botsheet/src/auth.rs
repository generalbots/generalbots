//! Sheet-scoped user identity (#789).
//!
//! The host crate (`botserver/src/sheet/user_middleware.rs`) converts the
//! platform `AuthenticatedUser` extension into a [`SheetUser`] extension on
//! every sheet request. Handlers resolve the real user id through
//! [`resolve_user_id`]; anonymous or unauthenticated traffic keeps the legacy
//! `default-user` namespace so pre-auth clients keep working while real
//! bearer-token traffic is scoped to its owner.

/// Sheet-scoped identity, decoupled from the host security crate.
#[derive(Debug, Clone)]
pub struct SheetUser {
    pub id: String,
    pub name: String,
    pub authenticated: bool,
}

impl SheetUser {
    /// Anonymous fallback matching the legacy `default-user` namespace.
    pub fn anonymous() -> Self {
        Self {
            id: "default-user".to_string(),
            name: "Anonymous".to_string(),
            authenticated: false,
        }
    }
}

/// Resolves the effective user id from a handler's optional extension.
pub fn resolve_user_id(user: Option<&SheetUser>) -> String {
    user.map(|u| u.id.clone())
        .unwrap_or_else(|| "default-user".to_string())
}

/// Resolves the display name, falling back to `Anonymous`.
pub fn resolve_user_name(user: Option<&SheetUser>) -> String {
    user.map(|u| u.name.clone())
        .unwrap_or_else(|| "Anonymous".to_string())
}
