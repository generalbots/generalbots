//! Pure unit tests for the #2668 share-owner guard.

use super::delete_bot::share_owner_allowed;

#[test]
fn share_owner_allowed_no_requested_id_uses_caller() {
    assert!(share_owner_allowed("victim", None, false));
}

#[test]
fn share_owner_allowed_same_user() {
    assert!(share_owner_allowed("alice", Some("alice"), false));
}

#[test]
fn share_owner_allowed_rejects_forged_owner() {
    // The #2668 attack: attacker names a victim as owner.
    assert!(!share_owner_allowed("attacker", Some("victim"), false));
    assert!(!share_owner_allowed("alice", Some(""), false));
}

#[test]
fn share_owner_allowed_platform_admin_delegation() {
    // Platform admins may act for another user (support tooling).
    assert!(share_owner_allowed("admin", Some("victim"), true));
}
