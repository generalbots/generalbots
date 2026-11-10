//! Bridges the platform auth extension into the Sheet handler scope (#789).
//!
//! The platform auth middleware inserts `AuthenticatedUser` into request
//! extensions. This middleware converts it into the sheet-scoped
//! [`SheetUser`] so the `botsheet` crate never depends on the security crate.
//!
//! #2127 — a request without a *verified* principal is refused with `401`
//! here, before any handler runs. The old code inserted an anonymous
//! `default-user` identity, which made every sheet endpoint an unauthenticated
//! shared namespace. The only exception is a single-user desktop install
//! (`no_security`/`desktop-local`), whose owner id is derived per install.

use axum::{body::Body, http::Request, middleware::Next, response::Response};
#[cfg(any(feature = "security", test))]
use axum::response::IntoResponse;
use botsheet::auth::SheetUser;
#[cfg(any(feature = "security", test))]
use botsheet::auth::unauthenticated;
use uuid::Uuid;

/// Rejects a sheet request that carries no verified identity (#2127).
#[cfg(any(feature = "security", test))]
fn refuse() -> Response {
    unauthenticated().into_response()
}

/// Stable owner id for a single-user desktop install (#2127).
///
/// The desktop edition has no directory account, so the workbook namespace is
/// derived from the installation itself: `GB_DESKTOP_LOCAL_USER_ID` wins when
/// set, otherwise a UUIDv5 over the install directory, the stack path and the
/// OS user gives one stable id per install. It is never the shared
/// `default-user` string, so two installs never see each other's workbooks.
pub fn desktop_local_owner_id() -> String {
    if let Ok(explicit) = std::env::var("GB_DESKTOP_LOCAL_USER_ID") {
        let trimmed = explicit.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    let install = std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|dir| dir.display().to_string()))
        .unwrap_or_default();
    let stack = std::env::var("BOTSERVER_STACK_PATH").unwrap_or_default();
    let os_user = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_default();
    let seed = format!("generalbots-desktop-local|{install}|{stack}|{os_user}");
    Uuid::new_v5(&Uuid::NAMESPACE_OID, seed.as_bytes()).to_string()
}

#[cfg(feature = "security")]
pub async fn sheet_user_middleware(mut request: Request<Body>, next: Next) -> Response {
    let user = request
        .extensions()
        .get::<crate::security::auth::AuthenticatedUser>()
        .cloned();
    let sheet_user = match user {
        Some(u) if u.is_authenticated() => {
            SheetUser::verified(u.user_id.to_string(), u.username.clone())
        }
        // Desktop edition: the auth middleware resolves the local OS owner
        // (nil user id by design) and marks it in `metadata`.
        Some(u) if u.metadata.get("desktop_local").map(String::as_str) == Some("true") => {
            SheetUser::verified(desktop_local_owner_id(), "local")
        }
        _ => return refuse(),
    };
    request.extensions_mut().insert(sheet_user);
    next.run(request).await
}

/// Builds without the `security` feature (no platform auth layer at all).
/// Such a build is a single-user install, so the per-install owner id is the
/// only principal — still never the shared `default-user` namespace.
#[cfg(not(feature = "security"))]
pub async fn sheet_user_middleware(mut request: Request<Body>, next: Next) -> Response {
    request
        .extensions_mut()
        .insert(SheetUser::verified(desktop_local_owner_id(), "local"));
    next.run(request).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    #[test]
    fn the_desktop_owner_id_is_stable_and_per_install() {
        let first = desktop_local_owner_id();
        let second = desktop_local_owner_id();
        assert_eq!(first, second, "the owner id must not change between calls");
        assert!(!first.trim().is_empty());
        assert_ne!(
            first, "default-user",
            "the desktop edition must never use the shared legacy namespace"
        );
    }

    #[test]
    fn an_unverified_principal_is_refused_with_401() {
        assert_eq!(refuse().status(), StatusCode::UNAUTHORIZED);
    }
}
