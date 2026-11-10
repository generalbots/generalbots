//! Spreadsheet access rules (#789) exercised offline (#2074).
//!
//! `botsheet_core::state` decides who may read or mutate a workbook. A bug
//! here is a tenancy bug, and it needs no database to reproduce: the rule is a
//! pure function of the owner, the ACL map and the requesting user.

use std::collections::HashMap;

use botsheet_core::state::{
    can_read_sheet, can_write_sheet, ensure_write_allowed, get_user_sheets_path,
};
use botsheet_core::types::Spreadsheet;

/// A fixed timestamp: the ACL rule never reads the clock, and the workspace
/// builds chrono without its `clock` feature in most crates.
fn epoch() -> chrono::DateTime<chrono::Utc> {
    chrono::DateTime::from_timestamp(0, 0).expect("the unix epoch is a valid timestamp")
}

/// Builds a workbook with the given owner and `(user, permission)` grants.
fn sheet(owner: &str, grants: &[(&str, &str)]) -> Spreadsheet {
    let acl: HashMap<String, String> = grants
        .iter()
        .map(|(user, permission)| ((*user).to_string(), (*permission).to_string()))
        .collect();
    Spreadsheet {
        id: "sheet-1".to_string(),
        name: "Budget".to_string(),
        owner_id: owner.to_string(),
        worksheets: Vec::new(),
        created_at: epoch(),
        updated_at: epoch(),
        named_ranges: None,
        external_links: None,
        source_bucket: None,
        source_path: None,
        source_bytes: None,
        acl,
    }
}

#[test]
fn the_owner_can_read_and_write() {
    let owned = sheet("alice", &[]);
    assert!(can_read_sheet("alice", &owned));
    assert!(can_write_sheet("alice", &owned));
    assert_eq!(ensure_write_allowed("alice", &owned), Ok(()));
}

#[test]
fn a_stranger_is_denied_read_and_write() {
    let owned = sheet("alice", &[]);
    assert!(!can_read_sheet("mallory", &owned));
    assert!(!can_write_sheet("mallory", &owned));
    assert_eq!(
        ensure_write_allowed("mallory", &owned),
        Err("Access denied".to_string()),
        "a denied write must report the 403-style message, not a panic"
    );
}

#[test]
fn a_view_grant_reads_but_never_writes() {
    let shared = sheet("alice", &[("bob", "view")]);
    assert!(can_read_sheet("bob", &shared));
    assert!(!can_write_sheet("bob", &shared));
    assert_eq!(
        ensure_write_allowed("bob", &shared),
        Err("Access denied".to_string())
    );
}

#[test]
fn an_edit_grant_reads_and_writes() {
    let shared = sheet("alice", &[("carol", "edit")]);
    assert!(can_read_sheet("carol", &shared));
    assert!(can_write_sheet("carol", &shared));
    assert_eq!(ensure_write_allowed("carol", &shared), Ok(()));
}

#[test]
fn legacy_placeholder_sheets_stay_open() {
    // Pre-identity workbooks have no owner: they must remain usable, otherwise
    // every migrated sheet would lock its users out (#789).
    for owner in ["", "default-user"] {
        let legacy = sheet(owner, &[]);
        assert!(
            can_read_sheet("anyone", &legacy),
            "legacy owner `{owner}` must stay readable"
        );
        assert!(
            can_write_sheet("anyone", &legacy),
            "legacy owner `{owner}` must stay writable"
        );
    }
}

#[test]
fn an_unknown_permission_grants_read_only() {
    let shared = sheet("alice", &[("dave", "comment")]);
    assert!(can_read_sheet("dave", &shared));
    assert!(
        !can_write_sheet("dave", &shared),
        "only an explicit `edit` grant may write"
    );
}

#[test]
fn the_user_sheets_path_is_namespaced_per_user() {
    assert_eq!(get_user_sheets_path("alice"), "users/alice/sheets");
    assert_ne!(
        get_user_sheets_path("alice"),
        get_user_sheets_path("bob"),
        "two users must never share a sheets prefix"
    );
}
