//! #2671 — regression tests for the billing/store/voucher/organization/product
//! surfaces added to the default-deny RBAC matrix.

use super::*;
use uuid::Uuid;

fn user() -> AuthenticatedUser {
    AuthenticatedUser::new(Uuid::new_v4(), "tenant-user@example.com".to_string()).with_role(Role::User)
}

fn admin() -> AuthenticatedUser {
    AuthenticatedUser::new(Uuid::new_v4(), "admin@example.com".to_string()).with_role(Role::Admin)
}

#[tokio::test]
async fn billing_store_state_changes_are_admin_only() {
    let manager = RbacManager::with_defaults();
    manager.register_routes(build_default_route_permissions()).await;

    // Only methods that are real state-changing endpoints are registered in
    // the matrix. Unregistered methods are default-denied for *everyone*,
    // which is safe but must not be asserted as "admin allowed".
    let cases: [(&str, &[&str]); 7] = [
        ("/api/billing/invoices", &["POST", "PUT", "DELETE"]),
        ("/api/billing/payments", &["POST", "PUT", "DELETE"]),
        ("/api/products/items", &["POST", "PUT", "DELETE"]),
        ("/api/cloud/store/purchase", &["POST"]),
        ("/api/cloud/vouchers/redeem", &["POST", "PUT", "DELETE"]),
        ("/api/cloud/organizations", &["POST", "PUT", "DELETE"]),
        ("/api/ui/billing/subscription/upgrade", &["POST", "PUT"]),
    ];
    for (path, methods) in cases {
        for method in methods {
            assert!(
                !manager.check_route_access(path, method, &user()).await.is_allowed(),
                "{method} {path} must be denied for a non-admin tenant user"
            );
            assert!(
                manager.check_route_access(path, method, &admin()).await.is_allowed(),
                "{method} {path} must stay allowed for an admin"
            );
        }
    }
}

#[tokio::test]
async fn billing_reads_stay_available_to_tenant_users() {
    let manager = RbacManager::with_defaults();
    manager.register_routes(build_default_route_permissions()).await;

    for path in ["/api/billing/invoices", "/api/products/items"] {
        assert!(
            manager.check_route_access(path, "GET", &user()).await.is_allowed(),
            "GET {path} must stay readable for an authenticated tenant user"
        );
    }
}
