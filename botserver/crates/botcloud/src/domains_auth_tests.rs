//! Unit tests for the pure write-authorization predicate in `domains_auth.rs`.

use super::*;

#[test]
fn platform_admin_may_write_any_row() {
    let branch = Uuid::new_v4();
    assert!(may_write_domain(true, None, None));
    assert!(may_write_domain(true, None, Some(branch)));
    assert!(may_write_domain(true, Some(Uuid::new_v4()), Some(Uuid::new_v4())));
}

#[test]
fn tenant_user_may_only_write_own_branch_row() {
    let branch = Uuid::new_v4();
    assert!(may_write_domain(false, Some(branch), Some(branch)));
}

#[test]
fn tenant_user_cannot_write_null_branch_row() {
    // #2677: auto-provisioned domains have branch_id = NULL. Regardless of
    // which tenant the caller belongs to, that row is admin-only.
    assert!(!may_write_domain(false, Some(Uuid::new_v4()), None));
}

#[test]
fn tenant_user_cannot_write_another_tenants_branch() {
    let caller = Uuid::new_v4();
    let other = Uuid::new_v4();
    assert!(!may_write_domain(false, Some(caller), Some(other)));
}

#[test]
fn missing_caller_identity_fails_closed() {
    let branch = Uuid::new_v4();
    assert!(!may_write_domain(false, None, Some(branch)));
    assert!(!may_write_domain(false, None, None));
}

#[test]
fn production_source_has_no_null_branch_write_escape() {
    // Regression guard for the #2677 SQL: no UPDATE/DELETE may match a row
    // purely because its branch is unset.
    let source = include_str!("domains.rs");
    assert!(!source.contains("branch_id IS NULL"), "NULL-branch write escape reintroduced");
}
