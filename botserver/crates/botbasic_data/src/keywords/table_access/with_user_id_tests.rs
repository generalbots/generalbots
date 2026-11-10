//! Contract tests for `UserRoles` derivation from a chat session (#2064).
//!
//! `from_user_session` is the only bridge between the session's `context_data`
//! and the table/field permission checks: getting the key precedence or the
//! lowercasing wrong silently grants or denies access to every table.

use super::*;
use chrono::Utc;
use uuid::Uuid;

fn session_with(context_data: Value) -> UserSession {
    let now = Utc::now();
    UserSession {
        id: Uuid::new_v4(),
        user_id: Uuid::new_v4().to_string(),
        bot_id: Uuid::new_v4(),
        title: "t".to_string(),
        answer_mode: 0,
        context_data,
        current_tool: None,
        message_count: 0,
        total_tokens: 0,
        created_at: now,
        updated_at: now,
        last_activity: now,
        tenant_id: None,
        active_email_account_id: None,
        branch_id: Uuid::nil(),
        session_id: None,
        data: None,
        expires_at: None,
        locked: false,
        lock_pin_hash: None,
    }
}

#[test]
fn roles_are_read_from_an_array() {
    let session = session_with(serde_json::json!({"roles": ["Admin", " Manager "]}));
    let roles = UserRoles::from_user_session(&session);

    assert_eq!(roles.roles, vec!["admin", "manager"]);
    assert!(roles.has_access(&["admin".to_string()]));
}

#[test]
fn roles_are_read_from_a_semicolon_string() {
    let session = session_with(serde_json::json!({"roles": "admin; hr ;;Manager"}));
    let roles = UserRoles::from_user_session(&session);

    assert_eq!(roles.roles, vec!["admin", "hr", "manager"]);
}

#[test]
fn the_first_populated_role_key_wins() {
    let session = session_with(serde_json::json!({
        "roles": ["admin"],
        "user_roles": ["guest"],
        "zitadel_roles": ["nobody"],
    }));
    assert_eq!(UserRoles::from_user_session(&session).roles, vec!["admin"]);

    // An empty first key falls through to the next one.
    let session = session_with(serde_json::json!({
        "roles": [],
        "user_roles": ["guest"],
    }));
    assert_eq!(UserRoles::from_user_session(&session).roles, vec!["guest"]);
}

#[test]
fn a_session_without_role_keys_has_no_roles() {
    let roles = UserRoles::from_user_session(&session_with(serde_json::json!({})));
    assert!(roles.roles.is_empty());
    assert!(roles.has_access(&[]), "no required role means everyone");
    assert!(!roles.has_access(&["admin".to_string()]));
}

#[test]
fn the_admin_flag_promotes_the_user_to_admin() {
    let session = session_with(serde_json::json!({"is_admin": true, "roles": ["user"]}));
    let roles = UserRoles::from_user_session(&session);

    assert_eq!(roles.roles, vec!["user", "admin"]);
    assert!(roles.is_admin());
}

#[test]
fn the_admin_flag_does_not_duplicate_an_existing_admin_role() {
    let session = session_with(serde_json::json!({"is_admin": true, "roles": ["admin"]}));
    assert_eq!(UserRoles::from_user_session(&session).roles, vec!["admin"]);
}

#[test]
fn a_false_admin_flag_adds_nothing() {
    let session = session_with(serde_json::json!({"is_admin": false, "roles": ["user"]}));
    assert_eq!(UserRoles::from_user_session(&session).roles, vec!["user"]);
    assert!(!UserRoles::from_user_session(&session).is_admin());
}

#[test]
fn the_user_id_is_parsed_when_it_is_a_uuid() {
    let session = session_with(serde_json::json!({}));
    let expected = Uuid::parse_str(&session.user_id).expect("uuid");
    assert_eq!(UserRoles::from_user_session(&session).user_id, Some(expected));

    let mut external = session_with(serde_json::json!({}));
    external.user_id = "wa:5511999999999".to_string();
    assert_eq!(UserRoles::from_user_session(&external).user_id, None);
}

#[test]
fn has_role_is_case_insensitive_on_the_argument() {
    let roles = UserRoles::new(vec!["Admin".to_string()]);
    assert!(roles.has_role("admin"));
    assert!(roles.has_role("ADMIN"));
    assert!(!roles.has_role("guest"));
    assert!(!roles.has_role(""));
}

#[test]
fn restricted_fields_list_only_the_denied_ones() {
    let mut info = TableAccessInfo::default();
    info.field_read_roles
        .insert("ssn".to_string(), vec!["admin".to_string()]);
    info.field_read_roles
        .insert("salary".to_string(), vec!["hr".to_string()]);
    info.field_write_roles
        .insert("ssn".to_string(), vec!["admin".to_string()]);

    let user = UserRoles::new(vec!["hr".to_string()]);
    assert_eq!(info.get_restricted_read_fields(&user), vec!["ssn"]);
    assert_eq!(info.get_restricted_write_fields(&user), vec!["ssn"]);

    let admin = UserRoles::new(vec!["admin".to_string()]);
    let mut allowed = admin.roles.clone();
    allowed.extend(["hr".to_string()]);
    let both = UserRoles::new(allowed);
    assert!(info.get_restricted_read_fields(&both).is_empty());
    assert!(info.get_restricted_write_fields(&both).is_empty());
}

#[test]
fn table_level_read_and_write_roles_are_enforced() {
    let info = TableAccessInfo {
        table_name: "contacts".to_string(),
        read_roles: vec!["admin".to_string(), "sales".to_string()],
        write_roles: vec!["admin".to_string()],
        ..TableAccessInfo::default()
    };

    let sales = UserRoles::new(vec!["sales".to_string()]);
    assert!(info.can_read(&sales));
    assert!(!info.can_write(&sales));

    let admin = UserRoles::new(vec!["admin".to_string()]);
    assert!(info.can_read(&admin));
    assert!(info.can_write(&admin));
}
