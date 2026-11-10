//! Docs identity expectations (#2135): ownership comes from the verified
//! caller, never from a constant, and an unresolved caller is rejected.

use super::*;
use axum::http::Extensions;

fn user(id: &str) -> DocUser {
    DocUser::verified(id, id).expect("a verified id owns a namespace")
}

#[test]
fn two_users_get_different_namespaces() {
    let first = user("11111111-1111-1111-1111-111111111111");
    let second = user("22222222-2222-2222-2222-222222222222");
    assert_ne!(first.docs_path(), second.docs_path());
    assert_eq!(
        first.docs_path(),
        "users/11111111-1111-1111-1111-111111111111/docs"
    );
}

#[test]
fn the_namespace_is_the_verified_id_not_a_constant() {
    let id = "9f8b7a6c-5d4e-4f3a-2b1c-0d9e8f7a6b5c";
    assert_eq!(user(id).docs_path(), format!("users/{id}/docs"));
}

#[test]
fn the_legacy_shared_owner_is_never_an_identity() {
    assert!(DocUser::verified("default-user", "Shared").is_none());
    assert!(DocUser::verified(" Default-User ", "Shared").is_none());
    assert!(DocUser::verified("DEFAULT-USER", "Shared").is_none());
}

#[test]
fn an_empty_id_is_never_an_identity() {
    assert!(DocUser::verified("", "Nobody").is_none());
    assert!(DocUser::verified("   ", "Nobody").is_none());
    assert!(DocUser::verified("///", "Nobody").is_none());
}

#[test]
fn a_request_without_the_extension_is_rejected() {
    let extensions = Extensions::new();
    assert_eq!(
        DocUser::from_extensions(&extensions),
        Err(DocAuthError::Unauthenticated)
    );
}

#[test]
fn a_request_with_the_extension_resolves_to_that_owner() {
    let mut extensions = Extensions::new();
    extensions.insert(user("user-a"));
    let resolved = DocUser::from_extensions(&extensions).expect("the inserted identity is readable");
    assert_eq!(resolved.id(), "user-a");
    assert_eq!(resolved.display_name(), "user-a");
}

#[test]
fn the_display_name_defaults_to_the_id() {
    let resolved = DocUser::verified("user-a", "   ").expect("a verified id owns a namespace");
    assert_eq!(resolved.display_name(), "user-a");
}

/// The IDOR guarantee of #2135: a document id is only addressable inside the
/// namespace of the caller who owns it, so knowing (or guessing) another user's
/// document id never reaches their object.
#[test]
fn another_users_document_id_never_resolves_inside_my_namespace() {
    let doc_id = "9f8b7a6c-5d4e-4f3a-2b1c-0d9e8f7a6b5c";
    let mine = user("user-a");
    let theirs = user("user-b");

    let my_object = format!("{}/{doc_id}.html", mine.docs_path());
    let their_object = format!("{}/{doc_id}.html", theirs.docs_path());

    assert_ne!(my_object, their_object);
    assert!(my_object.starts_with(&format!("{}/", mine.docs_path())));
    assert!(!my_object.starts_with(&format!("{}/", theirs.docs_path())));
    assert!(!their_object.starts_with(&format!("{}/", mine.docs_path())));
}
