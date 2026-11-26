//! #2965 — App Store install route catalogue tests (split for file-size).
use crate::rbac_middleware::{build_default_route_permissions, RbacManager};
use crate::{AuthenticatedUser, Role};
use uuid::Uuid;

/// Both verbs mounted by `apps::installs::register` must be present in the
/// authenticated catalogue with anonymous access disabled. Before #2965 only
/// `/api/apps/install/stats` was listed, so default-deny 403'd the store's
/// status probe and install sync before their handlers ran.
#[test]
fn install_verbs_are_in_authenticated_catalogue_without_anonymous() {
    let routes = super::routes();
    for method in ["GET", "POST"] {
        assert!(
            routes.iter().any(|r| {
                r.path_pattern == "/api/apps/install"
                    && r.method == method
                    && !r.allow_anonymous
            }),
            "/api/apps/install {method} must be in the authenticated catalogue with anonymous disabled"
        );
    }
}

/// Exercise the full default-deny catalogue with both an authenticated caller
/// and an anonymous one: the read-only GET status probe and the install POST
/// must both pass for a signed-in admin and both fail for an anonymous caller.
#[tokio::test]
async fn install_catalogue_admits_authenticated_and_denies_anonymous() {
    let manager = RbacManager::with_defaults();
    manager
        .register_routes(build_default_route_permissions())
        .await;
    let admin = AuthenticatedUser::new(Uuid::new_v4(), "admin@example.com".to_string())
        .with_role(Role::Admin);
    let anonymous = AuthenticatedUser::anonymous();

    for method in ["GET", "POST"] {
        let allowed = manager
            .check_route_access("/api/apps/install", method, &admin)
            .await;
        assert!(
            allowed.is_allowed(),
            "{method} /api/apps/install must be allowed for an authenticated admin: {}",
            allowed.reason
        );
        let denied = manager
            .check_route_access("/api/apps/install", method, &anonymous)
            .await;
        assert!(
            !denied.is_allowed(),
            "{method} /api/apps/install must be denied for an anonymous caller"
        );
    }
}
