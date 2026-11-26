//! #2948 — RBAC decision table for the banking app's REST surface.
//!
//! The banking handlers' 401 was a tenant-scope bug, not a role gate: these
//! tests lock in that the catalogue admits an authenticated admin (and a
//! regular user) on every read the app issues, keeps mutations admin-only,
//! and never opens the prefix to an anonymous caller.

use crate::rbac_middleware::{RbacManager, build_default_route_permissions};
use crate::{AuthenticatedUser, Role};
use uuid::Uuid;

#[tokio::test]
async fn banking_reads_are_open_to_authenticated_users_and_closed_to_anonymous() {
    let manager = RbacManager::with_defaults();
    manager
        .register_routes(build_default_route_permissions())
        .await;
    let admin = AuthenticatedUser::new(Uuid::new_v4(), "admin@example.com".to_string())
        .with_role(Role::Admin);
    let user = AuthenticatedUser::new(Uuid::new_v4(), "member@example.com".to_string())
        .with_role(Role::User);
    let anonymous = AuthenticatedUser::anonymous();

    // The exact paths the banking suite app loads on open (#2948).
    for path in [
        "/api/banking/transactions",
        "/api/banking/platforms",
        "/api/banking/reconcile/pairs",
        "/api/banking/reports",
        "/api/banking/stats",
        "/api/banking/settings",
    ] {
        for principal in [&admin, &user] {
            let decision = manager.check_route_access(path, "GET", principal).await;
            assert!(
                decision.is_allowed(),
                "GET {path} must be allowed for {}: {}",
                principal.username,
                decision.reason
            );
        }
        let denied = manager.check_route_access(path, "GET", &anonymous).await;
        assert!(
            !denied.is_allowed(),
            "GET {path} must stay closed to anonymous callers"
        );
    }
}

#[tokio::test]
async fn banking_mutations_stay_admin_only() {
    let manager = RbacManager::with_defaults();
    manager
        .register_routes(build_default_route_permissions())
        .await;
    let admin = AuthenticatedUser::new(Uuid::new_v4(), "admin@example.com".to_string())
        .with_role(Role::Admin);
    let user = AuthenticatedUser::new(Uuid::new_v4(), "member@example.com".to_string())
        .with_role(Role::User);
    let anonymous = AuthenticatedUser::anonymous();

    for (method, path) in [
        ("POST", "/api/banking/transactions"),
        ("POST", "/api/banking/reconcile"),
        ("PUT", "/api/banking/settings"),
        ("DELETE", "/api/banking/transactions"),
    ] {
        assert!(
            manager
                .check_route_access(path, method, &admin)
                .await
                .is_allowed(),
            "{method} {path} must be allowed for an admin"
        );
        for denied_principal in [&user, &anonymous] {
            assert!(
                !manager
                    .check_route_access(path, method, denied_principal)
                    .await
                    .is_allowed(),
                "{method} {path} must be denied for {}",
                denied_principal.username
            );
        }
    }
}
