//! Docs identity resolution (#2135): the owner comes from the verified
//! principal, and an unverified one has no namespace at all.

use super::*;
use crate::security::auth::AuthenticatedUser;
use uuid::Uuid;

#[test]
fn anonymous_gets_no_docs_namespace() {
    assert!(doc_user_for(&AuthenticatedUser::anonymous()).is_none());
}

#[test]
fn two_users_get_different_namespaces() {
    let first = doc_user_for(&AuthenticatedUser::new(Uuid::new_v4(), "first".to_string()))
        .expect("a directory user owns a namespace");
    let second = doc_user_for(&AuthenticatedUser::new(Uuid::new_v4(), "second".to_string()))
        .expect("a directory user owns a namespace");
    assert_ne!(first.docs_path(), second.docs_path());
}

#[test]
fn the_namespace_is_the_verified_user_id() {
    let user_id = Uuid::new_v4();
    let user = doc_user_for(&AuthenticatedUser::new(user_id, "first".to_string()))
        .expect("a directory user owns a namespace");
    assert_eq!(user.docs_path(), format!("users/{user_id}/docs"));
}

/// A single-user install has no directory account: the owner is the install id,
/// never the shared legacy namespace.
#[cfg(feature = "desktop-local")]
#[test]
fn desktop_local_uses_the_per_install_owner() {
    let first = doc_user_for(&AuthenticatedUser::desktop_local())
        .expect("the desktop owner owns a namespace");
    let second = doc_user_for(&AuthenticatedUser::desktop_local())
        .expect("the desktop owner owns a namespace");
    assert_eq!(first.docs_path(), second.docs_path());
    assert_ne!(first.docs_path(), "users/default-user/docs");
    assert!(!first.docs_path().starts_with("users/default-user/"));
}
