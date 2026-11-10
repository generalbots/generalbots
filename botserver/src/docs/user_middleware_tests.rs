//! Identity resolution for the Docs middleware (#2135).

use super::*;
use crate::security::auth::AuthenticatedUser;
use uuid::Uuid;

const INSTALL: &str = "local-install-a";

fn verified(principal: &AuthenticatedUser) -> Option<DocUser> {
    resolve_doc_user(Some(principal), INSTALL)
}

#[test]
fn anonymous_gets_no_docs_namespace() {
    assert!(verified(&AuthenticatedUser::anonymous()).is_none());
}

#[test]
fn missing_principal_gets_no_docs_namespace() {
    assert!(resolve_doc_user(None, INSTALL).is_none());
}

#[test]
fn two_users_get_different_namespaces() {
    let first = verified(&AuthenticatedUser::new(Uuid::new_v4(), "first".to_string()))
        .expect("a directory user owns a namespace");
    let second = verified(&AuthenticatedUser::new(Uuid::new_v4(), "second".to_string()))
        .expect("a directory user owns a namespace");
    assert_ne!(first.docs_path(), second.docs_path());
}

#[test]
fn desktop_local_uses_the_per_install_id() {
    let user =
        verified(&AuthenticatedUser::desktop_local()).expect("the desktop owner owns a namespace");
    assert_eq!(user.docs_path(), "users/local-install-a/docs");
    assert_ne!(user.docs_path(), "users/default-user/docs");
}

#[test]
fn two_installs_get_different_namespaces() {
    let principal = AuthenticatedUser::desktop_local();
    let first = resolve_doc_user(Some(&principal), "local-install-a").expect("desktop owner");
    let second = resolve_doc_user(Some(&principal), "local-install-b").expect("desktop owner");
    assert_ne!(first.docs_path(), second.docs_path());
}

#[test]
fn the_legacy_owner_is_never_accepted_as_an_install_id() {
    let principal = AuthenticatedUser::desktop_local();
    assert!(resolve_doc_user(Some(&principal), "default-user").is_none());
    assert!(resolve_doc_user(Some(&principal), " Default-User ").is_none());
}
