//! #2964 — RBAC decision table for the POS suite app's REST surface.
//!
//! The products/orders 401s were caused by the frontend sending `Bearer null`
//! (the live session token is not in the legacy `gb_token` key). The handler
//! now resolves its tenant from verified claims: an authenticated admin with
//! a nil/absent workspace claim gets the global scope, anonymous callers are
//! refused. These tests lock the catalogue side of that contract: every POS
//! path the app calls is authenticated-only and free of extra role gates.

use crate::rbac_middleware::{build_default_route_permissions, RbacManager};
use crate::{AuthenticatedUser, Role};
use uuid::Uuid;

/// Every verb mounted by `botpos::routes::configure` must be present with no
/// role/permission gate and no anonymous access. Tenant scoping is owned by
/// the handler, not by the route table.
#[test]
fn pos_prefix_covers_every_mounted_verb_without_anonymous() {
    let routes = build_default_route_permissions();
    for method in ["GET", "POST", "PUT", "DELETE"] {
        let route = routes
            .iter()
            .find(|r| r.path_pattern == "/api/pos/**" && r.method == method);
        assert!(
            route.is_some(),
            "/api/pos/** {method} must be in the RBAC catalogue"
        );
        let route = route.expect("pos route");
        assert!(route.required_roles.is_empty(), "{method} role gate");
        assert!(
            route.required_permission.is_empty(),
            "{method} permission gate"
        );
        assert!(!route.allow_anonymous, "{method} must require a session");
    }
}

/// Offline decision table: an authenticated admin (and a regular user) reach
/// each endpoint the POS app loads or mutates; an anonymous caller does not.
#[tokio::test]
async fn pos_endpoints_allow_admin_and_deny_anonymous() {
    let manager = RbacManager::with_defaults();
    manager
        .register_routes(build_default_route_permissions())
        .await;
    let admin = AuthenticatedUser::new(Uuid::new_v4(), "admin@example.com".to_string())
        .with_role(Role::Admin);
    let user = AuthenticatedUser::new(Uuid::new_v4(), "member@example.com".to_string())
        .with_role(Role::User);
    let anonymous = AuthenticatedUser::anonymous();

    let cases = [
        ("GET", "/api/pos/products"),
        ("POST", "/api/pos/products"),
        ("GET", "/api/pos/orders"),
        ("POST", "/api/pos/orders"),
        ("GET", "/api/pos/orders/00000000-0000-0000-0000-000000000001"),
    ];

    for (method, path) in cases {
        for principal in [&admin, &user] {
            let allowed = manager.check_route_access(path, method, principal).await;
            assert!(
                allowed.is_allowed(),
                "{method} {path} must be allowed for {}: {}",
                principal.username,
                allowed.reason
            );
        }
        let denied = manager.check_route_access(path, method, &anonymous).await;
        assert!(
            !denied.is_allowed(),
            "{method} {path} must stay closed to anonymous callers"
        );
    }
}

/// The wildcard must cover the exact paths the app polls, but never widen the
/// grant to a look-alike prefix.
#[test]
fn pos_prefix_matches_registered_paths_not_lookalikes() {
    let routes = build_default_route_permissions();
    let route = routes
        .iter()
        .find(|r| r.path_pattern == "/api/pos/**" && r.method == "GET")
        .expect("pos GET catalogue entry");
    for path in [
        "/api/pos/products",
        "/api/pos/orders",
        "/api/pos/orders/00000000-0000-0000-0000-000000000001",
    ] {
        assert!(route.matches_path(path), "GET {path} must match");
    }
    assert!(!route.matches_path("/api/position/products"));
}
