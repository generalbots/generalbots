//! #2760 — sandbox route catalogue tests (split for file-size).
use crate::rbac_middleware::{build_default_route_permissions, RbacManager};
use crate::{AuthenticatedUser, Role};
use uuid::Uuid;

/// Sandbox status is a read-only status of a published resource: it must be
/// in the authenticated catalogue with anonymous access disabled, or the
/// default-deny gate 403s the handler before it can report the truth.
#[test]
fn sandbox_status_prefix_is_in_authenticated_catalogue_without_anonymous() {
    let routes = super::routes();
    assert!(
        routes.iter().any(|r| {
            r.path_pattern == "/api/sandbox/status/**"
                && r.method == "GET"
                && !r.allow_anonymous
        }),
        "/api/sandbox/status/** GET must be in the authenticated catalogue with anonymous disabled"
    );
}

/// The mounted route is `/api/sandbox/status/:hash`, so a real hash path
/// must be admitted for an authenticated user and denied for an anonymous
/// caller through the full default-deny catalogue.
#[tokio::test]
async fn sandbox_status_admits_authenticated_and_denies_anonymous() {
    let manager = RbacManager::with_defaults();
    manager
        .register_routes(build_default_route_permissions())
        .await;
    let user = AuthenticatedUser::new(Uuid::new_v4(), "viewer@example.com".to_string())
        .with_role(Role::User);
    let anonymous = AuthenticatedUser::anonymous();
    let hash = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    let path = format!("/api/sandbox/status/{hash}");

    let allowed = manager.check_route_access(&path, "GET", &user).await;
    assert!(
        allowed.is_allowed(),
        "GET {path} must be allowed for an authenticated user: {}",
        allowed.reason
    );
    let denied = manager.check_route_access(&path, "GET", &anonymous).await;
    assert!(
        !denied.is_allowed(),
        "GET {path} must be denied for an anonymous caller: {}",
        denied.reason
    );
}

/// #2760 — sandbox config used to be in the anonymous catalogue, leaking
/// the configured bucket/origin to any local caller. It must now require a
/// suite session while staying reachable for authenticated users.
#[tokio::test]
async fn sandbox_config_denies_anonymous_and_allows_authenticated() {
    let manager = RbacManager::with_defaults();
    manager
        .register_routes(build_default_route_permissions())
        .await;
    let user = AuthenticatedUser::new(Uuid::new_v4(), "viewer@example.com".to_string())
        .with_role(Role::User);
    let anonymous = AuthenticatedUser::anonymous();

    let allowed = manager
        .check_route_access("/api/sandbox/config", "GET", &user)
        .await;
    assert!(
        allowed.is_allowed(),
        "GET /api/sandbox/config must be allowed for an authenticated user: {}",
        allowed.reason
    );
    let denied = manager
        .check_route_access("/api/sandbox/config", "GET", &anonymous)
        .await;
    assert!(
        !denied.is_allowed(),
        "GET /api/sandbox/config must be denied for an anonymous caller: {}",
        denied.reason
    );
}

/// #2760 — revoke is state-changing: it must require an authenticated
/// session and must never be anonymous.
#[tokio::test]
async fn sandbox_revoke_denies_anonymous_and_allows_authenticated() {
    let manager = RbacManager::with_defaults();
    manager
        .register_routes(build_default_route_permissions())
        .await;
    let user = AuthenticatedUser::new(Uuid::new_v4(), "viewer@example.com".to_string())
        .with_role(Role::User);
    let anonymous = AuthenticatedUser::anonymous();
    let path = format!("/api/sandbox/revoke/{}", "a".repeat(64));

    let allowed = manager.check_route_access(&path, "POST", &user).await;
    assert!(
        allowed.is_allowed(),
        "POST {path} must be allowed for an authenticated user: {}",
        allowed.reason
    );
    let denied = manager.check_route_access(&path, "POST", &anonymous).await;
    assert!(
        !denied.is_allowed(),
        "POST {path} must be denied for an anonymous caller: {}",
        denied.reason
    );
}
