//! Sheet-scoped user identity (#789, #2127).
//!
//! The host crate (`botserver/src/sheet/user_middleware.rs`) converts the
//! verified platform `AuthenticatedUser` extension into a [`SheetUser`]
//! extension on every sheet request. Identity is a hard requirement: a request
//! that reaches a handler without a verified identity is answered with `401`
//! and nothing is read or written. There is deliberately no shared fallback
//! namespace — the legacy `default-user` string meant every anonymous caller
//! shared one set of workbooks (#2127).

use axum::http::StatusCode;
use axum::Json;

/// The error type every sheet handler returns: a status plus a JSON body.
pub type SheetError = (StatusCode, Json<serde_json::Value>);

/// Sheet-scoped identity, decoupled from the host security crate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SheetUser {
    /// Stable, server-derived owner id (a UUID for platform users).
    pub id: String,
    /// Display name, used for comments and collaboration labels.
    pub name: String,
    /// True only when the host middleware resolved a verified principal.
    pub authenticated: bool,
}

impl SheetUser {
    /// Identity of a principal the host has already verified.
    pub fn verified(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            authenticated: true,
        }
    }

    /// A user id that may own workbooks: verified, and not blank.
    pub fn is_verified(&self) -> bool {
        self.authenticated && !self.id.trim().is_empty()
    }
}

/// The `401` answer for a request without a verified identity (#2127).
///
/// Handlers and the host middleware share it so the rejection is identical on
/// both layers and the client can tell "log in" from "not found".
pub fn unauthenticated() -> SheetError {
    (
        StatusCode::UNAUTHORIZED,
        Json(serde_json::json!({
            "error": "Authentication required",
            "code": "sheet_unauthenticated",
        })),
    )
}

/// Resolves the effective owner id from a handler's optional extension.
///
/// Returns `401` instead of inventing an identity: a missing extension, an
/// unverified one or an empty id can never become a workbook owner.
pub fn resolve_user_id(user: Option<&SheetUser>) -> Result<String, SheetError> {
    match user {
        Some(user) if user.is_verified() => Ok(user.id.clone()),
        _ => Err(unauthenticated()),
    }
}

/// Resolves the display name of a verified user; `401` when absent.
pub fn resolve_user_name(user: Option<&SheetUser>) -> Result<String, SheetError> {
    match user {
        Some(user) if user.is_verified() => Ok(user.name.clone()),
        _ => Err(unauthenticated()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_extension_is_unauthorized() {
        let error = resolve_user_id(None).expect_err("no identity must be refused");
        assert_eq!(error.0, StatusCode::UNAUTHORIZED);
        assert_eq!(
            error.1 .0.get("code").and_then(|v| v.as_str()),
            Some("sheet_unauthenticated")
        );
        assert_eq!(resolve_user_name(None).expect_err("no name").0, StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn an_unverified_identity_is_unauthorized() {
        let spoofed = SheetUser {
            id: "alice".to_string(),
            name: "Alice".to_string(),
            authenticated: false,
        };
        assert_eq!(
            resolve_user_id(Some(&spoofed)).expect_err("unverified").0,
            StatusCode::UNAUTHORIZED
        );
    }

    #[test]
    fn a_blank_id_is_unauthorized() {
        let blank = SheetUser::verified("   ", "Ghost");
        assert_eq!(
            resolve_user_id(Some(&blank)).expect_err("blank id").0,
            StatusCode::UNAUTHORIZED
        );
    }

    #[test]
    fn a_verified_identity_is_the_owner() {
        let alice = SheetUser::verified("u-alice", "Alice");
        assert_eq!(resolve_user_id(Some(&alice)).expect("verified"), "u-alice");
        assert_eq!(resolve_user_name(Some(&alice)).expect("verified"), "Alice");
    }

    #[test]
    fn two_users_never_share_an_owner() {
        let alice = SheetUser::verified("u-alice", "Alice");
        let bob = SheetUser::verified("u-bob", "Bob");
        assert_ne!(
            resolve_user_id(Some(&alice)).expect("alice"),
            resolve_user_id(Some(&bob)).expect("bob")
        );
    }
}
