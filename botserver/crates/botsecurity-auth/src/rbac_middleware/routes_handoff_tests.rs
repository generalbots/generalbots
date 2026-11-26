//! #2959 — handoff suite route decisions.
//!
//! The app's 401s came from the frontend sending `Bearer null`: the live
//! session token is not stored in the legacy `gb_token` key the app used.
//! The RBAC catalogue already admits every mounted handoff path to any
//! authenticated caller (the handler scopes rows by verified branch claims)
//! and denies anonymous ones. These pure decision tests lock that contract in
//! so a future catalogue edit cannot turn the fixed frontend into a 403/401.

use crate::rbac_middleware::{build_default_route_permissions, RbacManager};
use crate::{AuthenticatedUser, Role};
use uuid::Uuid;

/// All verbs mounted by `bothandoff::routes::configure` must be present,
/// authenticated-only and free of extra role gates: tenant scoping is owned
/// by the handler, not by the route table.
#[test]
fn handoff_prefix_covers_every_mounted_verb_without_anonymous() {
    let routes = build_default_route_permissions();
    for method in ["GET", "POST", "PUT", "DELETE"] {
        let route = routes
            .iter()
            .find(|r| r.path_pattern == "/api/handoff/**" && r.method == method);
        assert!(
            route.is_some(),
            "/api/handoff/** {method} must be in the catalogue"
        );
        let route = route.expect("handoff route");
        assert!(route.required_roles.is_empty(), "{method} role gate");
        assert!(
            route.required_permission.is_empty(),
            "{method} permission gate"
        );
        assert!(!route.allow_anonymous, "{method} must require a session");
    }
}

/// An authenticated admin gets through default-deny for each endpoint the
/// handoff app polls or mutates; an anonymous caller is rejected by the same
/// rules. No running server is needed — this exercises the decision table.
#[tokio::test]
async fn handoff_endpoints_allow_admin_and_deny_anonymous() {
    let manager = RbacManager::with_defaults();
    manager
        .register_routes(build_default_route_permissions())
        .await;
    let admin = AuthenticatedUser::new(Uuid::new_v4(), "admin@example.com".to_string())
        .with_role(Role::Admin);
    let anonymous = AuthenticatedUser::anonymous();

    let cases = [
        ("GET", "/api/handoff/transcripts"),
        ("GET", "/api/handoff/deflection"),
        ("GET", "/api/handoff/csat"),
        ("GET", "/api/handoff/queue"),
        ("GET", "/api/handoff/analytics"),
        ("GET", "/api/handoff/channels"),
        ("GET", "/api/handoff/agents"),
        ("GET", "/api/handoff/sla"),
        (
            "POST",
            "/api/handoff/transfer/00000000-0000-0000-0000-000000000001",
        ),
    ];

    for (method, path) in cases {
        let allowed = manager.check_route_access(path, method, &admin).await;
        assert!(
            allowed.is_allowed(),
            "{method} {path} must be allowed for an admin: {}",
            allowed.reason
        );
        let denied = manager.check_route_access(path, method, &anonymous).await;
        assert!(
            !denied.is_allowed(),
            "{method} {path} must stay closed to anonymous callers"
        );
    }
}

/// The wildcard must not widen the grant to look-alike prefixes.
#[test]
fn handoff_prefix_does_not_match_lookalikes() {
    let routes = build_default_route_permissions();
    let route = routes
        .iter()
        .find(|r| r.path_pattern == "/api/handoff/**" && r.method == "GET")
        .expect("handoff GET route");
    assert!(route.matches_path("/api/handoff/transcripts"));
    assert!(!route.matches_path("/api/handoffx/transcripts"));
}
