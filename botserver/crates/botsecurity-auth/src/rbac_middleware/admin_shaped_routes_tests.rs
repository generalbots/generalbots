//! #2775 regression guards: admin-shaped stub handlers must be gated in the
//! RBAC matrix even while their handlers are stubs. Split from `tests.rs` to
//! respect the 450-line file limit.

use super::*;

fn admin_shaped_roles(route: &RoutePermission) -> bool {
    route.required_roles.iter().any(|r| r == "Admin")
        && route.required_roles.iter().any(|r| r == "SuperAdmin")
}

/// Every registered `/api/settings/security/**` entry must require Admin or
/// SuperAdmin. Those rules are placed before the broad `/api/settings/**`
/// self-service entries because `check_route_access` returns on the first
/// match — a roleless shadow would let any authenticated user reach TLS,
/// MFA, API-key, session and password-policy stubs.
#[tokio::test]
async fn settings_security_routes_are_admin_only() {
    let routes = build_default_route_permissions();
    let manager = RbacManager::with_defaults();
    manager.register_routes(routes.clone()).await;

    let security_routes: Vec<&RoutePermission> = routes
        .iter()
        .filter(|r| r.path_pattern == "/api/settings/security/**")
        .collect();
    assert!(
        security_routes.len() >= 4,
        "expected GET/POST/PUT/DELETE security settings routes, found {}",
        security_routes.len()
    );

    let user = AuthenticatedUser::new(Uuid::new_v4(), "user@example.com".to_string())
        .with_role(Role::User);
    let admin = AuthenticatedUser::new(Uuid::new_v4(), "admin@example.com".to_string())
        .with_role(Role::Admin);
    let super_admin = AuthenticatedUser::new(Uuid::new_v4(), "root@example.com".to_string())
        .with_role(Role::SuperAdmin);

    let paths = [
        "/api/settings/security/overview",
        "/api/settings/security/tls",
        "/api/settings/security/api-keys/123",
    ];
    for route in &security_routes {
        assert!(
            admin_shaped_roles(route),
            "{} {} must require Admin and SuperAdmin roles",
            route.method,
            route.path_pattern
        );
        for path in paths {
            let denied = manager.check_route_access(path, &route.method, &user).await;
            assert!(
                !denied.is_allowed(),
                "{} {} must be denied for a non-admin: {}",
                route.method,
                path,
                denied.reason
            );
            for privileged in [&admin, &super_admin] {
                let allowed = manager
                    .check_route_access(path, &route.method, privileged)
                    .await;
                assert!(
                    allowed.is_allowed(),
                    "{} {} must be allowed for an admin: {}",
                    route.method,
                    path,
                    allowed.reason
                );
            }
        }
    }

    // The self-service scope must stay intact: a plain user still reaches
    // their own settings, only the security sub-tree is privileged.
    let own_settings = manager.check_route_access("/api/settings/profile", "GET", &user).await;
    assert!(
        own_settings.is_allowed(),
        "GET /api/settings/profile must stay self-service: {}",
        own_settings.reason
    );
}

/// `POST /api/governance/**` must be admin-only: the `/sessions/kill` handler
/// has no role check of its own, so the matrix is the only guard. Reads stay
/// reachable for normal users (the governance dashboard is part of the
/// Compliance suite).
#[tokio::test]
async fn governance_mutations_are_admin_only() {
    let routes = build_default_route_permissions();
    let manager = RbacManager::with_defaults();
    manager.register_routes(routes.clone()).await;

    let governance_posts: Vec<&RoutePermission> = routes
        .iter()
        .filter(|r| r.path_pattern == "/api/governance/**" && r.method == "POST")
        .collect();
    assert_eq!(
        governance_posts.len(),
        1,
        "expected exactly one POST /api/governance/** rule"
    );
    assert!(
        admin_shaped_roles(governance_posts[0]),
        "POST /api/governance/** must require Admin and SuperAdmin roles"
    );

    let user = AuthenticatedUser::new(Uuid::new_v4(), "user@example.com".to_string())
        .with_role(Role::User);
    let admin = AuthenticatedUser::new(Uuid::new_v4(), "admin@example.com".to_string())
        .with_role(Role::Admin);
    let super_admin = AuthenticatedUser::new(Uuid::new_v4(), "root@example.com".to_string())
        .with_role(Role::SuperAdmin);

    let path = "/api/governance/sessions/kill";
    let denied = manager.check_route_access(path, "POST", &user).await;
    assert!(
        !denied.is_allowed(),
        "POST {path} must be denied for a non-admin: {}",
        denied.reason
    );
    for privileged in [&admin, &super_admin] {
        let allowed = manager.check_route_access(path, "POST", privileged).await;
        assert!(
            allowed.is_allowed(),
            "POST {path} must be allowed for an admin: {}",
            allowed.reason
        );
    }

    let read = manager.check_route_access("/api/governance/dashboard", "GET", &user).await;
    assert!(
        read.is_allowed(),
        "GET /api/governance/dashboard must stay readable: {}",
        read.reason
    );
}
