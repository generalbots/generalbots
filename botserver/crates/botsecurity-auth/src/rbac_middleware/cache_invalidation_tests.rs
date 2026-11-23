//! #2852 regression guards: cached allow/deny decisions must not outlive a
//! group membership or role change. Split from `tests.rs` to respect the
//! 450-line file limit.

use super::*;

/// A group grant must be usable immediately, and a revocation must revoke the
/// cached allow immediately — without waiting out the 300 s TTL.
#[tokio::test]
async fn cached_resource_decision_tracks_group_membership() {
    let manager = RbacManager::with_defaults();
    let user_id = Uuid::new_v4();
    let user = AuthenticatedUser::new(user_id, "member@example.com".to_string());

    let mut acl = ResourceAcl::new("document", "doc-1");
    acl.grant_group("editors", "read");
    manager.set_resource_acl(acl).await;

    let before = manager
        .check_resource_access(&user, "document", "doc-1", "read")
        .await;
    assert!(!before.is_allowed(), "non-member must be denied");

    manager.add_user_to_group(user_id, "editors").await;
    let allowed = manager
        .check_resource_access(&user, "document", "doc-1", "read")
        .await;
    assert!(allowed.is_allowed(), "new member must be allowed immediately: {}", allowed.reason);
    assert!(
        manager
            .permission_cache
            .read()
            .await
            .contains_key(&format!("resource:document:doc-1:read:{user_id}")),
        "the allow must have been cached"
    );

    manager.remove_user_from_group(user_id, "editors").await;
    let denied = manager
        .check_resource_access(&user, "document", "doc-1", "read")
        .await;
    assert!(
        !denied.is_allowed(),
        "revoked member must be denied immediately, not served the cached allow: {}",
        denied.reason
    );
    assert!(!denied.cache_hit, "revoked decision must be re-evaluated");
}

/// Directory/SCIM group handlers write membership with raw Diesel SQL and
/// cannot reach a manager instance. They call `invalidate_rbac_cache()`; the
/// next route check must see the new roles instead of the cached allow.
#[tokio::test]
async fn global_invalidation_expires_cached_route_allow() {
    let manager = RbacManager::with_defaults();
    manager
        .register_route(
            RoutePermission::new("/api/admin/groups", "GET", "")
                .with_roles(vec!["admin".to_string()]),
        )
        .await;
    let user_id = Uuid::new_v4();
    let admin = AuthenticatedUser::new(user_id, "admin@example.com".to_string())
        .with_roles(vec![Role::Admin]);
    let revoked = AuthenticatedUser::new(user_id, "admin@example.com".to_string())
        .with_roles(vec![Role::User]);

    let allowed = manager
        .check_route_access("/api/admin/groups", "GET", &admin)
        .await;
    assert!(allowed.is_allowed(), "admin must be allowed: {}", allowed.reason);
    assert!(!allowed.cache_hit);
    assert!(
        manager
            .permission_cache
            .read()
            .await
            .contains_key(&format!("route:/api/admin/groups:GET:{user_id}")),
        "the route allow must have been cached"
    );

    invalidate_rbac_cache();

    let denied = manager
        .check_route_access("/api/admin/groups", "GET", &revoked)
        .await;
    assert!(
        !denied.is_allowed(),
        "cached allow must become deny after invalidation: {}",
        denied.reason
    );
    assert!(!denied.cache_hit, "invalidated entry must be re-evaluated");
}
