//! #2854 regression guards: the `/api/ops/**` observability surface must be
//! admin-only. Split from `tests.rs` to respect the 450-line file limit.

use super::*;

/// Every registered `/api/ops/**` route must require Admin or SuperAdmin.
/// `check_route_access` returns on the first matching route, so a roleless
/// entry would let any authenticated user read status, metrics, traces and
/// recent errors.
#[tokio::test]
async fn ops_routes_are_admin_only() {
    let routes = build_default_route_permissions();
    let manager = RbacManager::with_defaults();
    manager.register_routes(routes.clone()).await;

    let ops_routes: Vec<&RoutePermission> = routes
        .iter()
        .filter(|r| r.path_pattern == "/api/ops/**")
        .collect();
    assert!(
        ops_routes.len() >= 2,
        "expected at least GET and POST ops routes, found {}",
        ops_routes.len()
    );

    let user = AuthenticatedUser::new(Uuid::new_v4(), "user@example.com".to_string())
        .with_role(Role::User);
    let admin = AuthenticatedUser::new(Uuid::new_v4(), "admin@example.com".to_string())
        .with_role(Role::Admin);
    let super_admin = AuthenticatedUser::new(Uuid::new_v4(), "root@example.com".to_string())
        .with_role(Role::SuperAdmin);

    // Matrix: every registered ops method × a representative nested path.
    for route in &ops_routes {
        assert!(
            route.required_roles.iter().any(|r| r == "Admin")
                && route.required_roles.iter().any(|r| r == "SuperAdmin"),
            "{} {} must require Admin and SuperAdmin roles",
            route.method,
            route.path_pattern
        );
        for path in ["/api/ops/status", "/api/ops/metrics/summary"] {
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
}
