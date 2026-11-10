//! Unit tests for the SQL identifier guards.
//!
//! Split out of `sql_guard.rs` to keep that file inside the project size
//! limit; the guards themselves stay beside their implementation.


use super::*;

#[test]
fn test_validate_table_name_allowed() {
    assert!(validate_table_name("users").is_ok());
    assert!(validate_table_name("bots").is_ok());
    assert!(validate_table_name("tasks").is_ok());
}

#[test]
fn test_validate_table_name_disallowed() {
    assert!(validate_table_name("evil_table").is_err());
    assert!(validate_table_name("users; DROP TABLE users;--").is_err());
    assert!(validate_table_name("").is_err());
}

#[test]
fn test_validate_order_direction() {
    assert_eq!(validate_order_direction("ASC").unwrap(), "ASC");
    assert_eq!(validate_order_direction("desc").unwrap(), "DESC");
    assert_eq!(validate_order_direction("Asc").unwrap(), "ASC");
    assert!(validate_order_direction("RANDOM").is_err());
}

#[test]
fn test_sanitize_identifier() {
    assert_eq!(sanitize_identifier("valid_name"), "valid_name");
    assert_eq!(sanitize_identifier("name123"), "name123");
    assert_eq!(sanitize_identifier("name; DROP--"), "nameDROP");
    assert_eq!(sanitize_identifier(""), "");
}

#[test]
fn test_check_for_injection_patterns() {
    assert!(check_for_injection_patterns("normal text").is_ok());
    assert!(check_for_injection_patterns("hello world").is_ok());

    assert!(check_for_injection_patterns("'; DROP TABLE users;--").is_err());
    assert!(check_for_injection_patterns("1 UNION SELECT * FROM passwords").is_err());
    assert!(check_for_injection_patterns("test/*comment*/").is_err());
    assert!(check_for_injection_patterns("WAITFOR DELAY '0:0:5'").is_err());
}

#[test]
fn test_escape_string_literal() {
    assert_eq!(escape_string_literal("hello"), "hello");
    assert_eq!(escape_string_literal("it's"), "it''s");
    assert_eq!(escape_string_literal("back\\slash"), "back\\\\slash");
    assert_eq!(escape_string_literal("O'Brien's"), "O''Brien''s");
}

#[test]
fn test_build_safe_select_query() {
    let query = build_safe_select_query("users", Some("created_at"), Some("DESC"), 10, 0);
    assert!(query.is_ok());
    let q = query.unwrap();
    assert!(q.contains("users"));
    assert!(q.contains("ORDER BY created_at DESC"));
    assert!(q.contains("LIMIT 10"));
    assert!(q.contains("OFFSET 0"));
}

#[test]
fn test_build_safe_select_query_invalid_table() {
    let query = build_safe_select_query("evil_table", None, None, 10, 0);
    assert!(query.is_err());
}

#[test]
fn test_build_safe_count_query() {
    let query = build_safe_count_query("users");
    assert!(query.is_ok());
    assert!(query.unwrap().contains("SELECT COUNT(*)"));
}

#[test]
fn test_build_safe_delete_query() {
    let query = build_safe_delete_query("tasks");
    assert!(query.is_ok());
    assert!(query.unwrap().contains("DELETE FROM tasks WHERE id = $1"));
}

#[test]
fn test_validate_identifier() {
    assert!(validate_identifier("valid_name").is_ok());
    assert!(validate_identifier("name123").is_ok());
    assert!(validate_identifier("").is_err());
    assert!(validate_identifier("123name").is_err());
}

#[test]
fn test_limit_clamping() {
    let query = build_safe_select_query("users", None, None, 10000, 0).unwrap();
    assert!(query.contains("LIMIT 1000"));

    let query2 = build_safe_select_query("users", None, None, -5, -10).unwrap();
    assert!(query2.contains("LIMIT 1"));
    assert!(query2.contains("OFFSET 0"));
}

#[test]
fn test_is_table_allowed() {
    assert!(is_table_allowed("users"));
    assert!(is_table_allowed("bots"));
    assert!(!is_table_allowed("hacked_table"));
}

#[test]
fn injection_payloads_never_survive_the_identifier_guard() {
    let hostile = [
        "users; DROP TABLE users",
        "users--",
        "users/*x*/",
        "users' OR '1'='1",
        "users\" OR 1=1",
        "users UNION SELECT password FROM users",
        "1users",
        "",
    ];
    for payload in hostile {
        assert!(
            validate_table_name(payload).is_err(),
            "{payload} must not pass validate_table_name"
        );
        assert!(
            validate_identifier(payload).is_err() || sanitize_identifier(payload) != payload,
            "{payload} must not pass validate_identifier unchanged"
        );
    }
}

#[test]
fn validate_table_name_is_deny_by_default() {
    // Only the system safelist passes without a database connection.
    assert!(validate_table_name("users").is_ok());
    assert!(validate_table_name("some_app_table").is_err());
    // The dynamic variant is syntax-only and must still reject injection.
    assert!(validate_dynamic_table_name("some_app_table").is_ok());
    assert!(validate_dynamic_table_name("users; DROP TABLE users").is_err());
}

#[test]
fn validate_identifier_rejects_leading_digits_and_overlong_names() {
    assert!(validate_identifier("9lives").is_err());
    assert!(validate_identifier("_ok").is_ok());
    assert!(validate_identifier(&"a".repeat(65)).is_err());
    assert!(validate_identifier(&"a".repeat(64)).is_ok());
}

#[test]
fn order_column_is_safelisted() {
    assert!(validate_order_column("created_at").is_ok());
    assert!(validate_order_column("created_at; DROP TABLE users").is_err());
    assert!(validate_order_column("password_hash").is_err());
}

#[test]
fn escape_string_literal_doubles_quotes() {
    assert_eq!(escape_string_literal("O'Brien"), "O''Brien");
    assert_eq!(escape_string_literal("plain"), "plain");
    assert_eq!(escape_string_literal("''"), "''''");
}

#[test]
fn select_builders_refuse_an_invalid_table() {
    assert!(
        build_safe_select_query("users; DROP TABLE users", None, None, 10, 0).is_err()
    );
    assert!(build_safe_count_query("users; DROP TABLE users").is_err());
    assert!(build_safe_delete_query("users; DROP TABLE users").is_err());
    assert!(build_safe_select_by_id_query("users; DROP TABLE users").is_err());
}

#[test]
fn system_tables_are_recognised() {
    assert!(is_system_table("users"));
    assert!(is_system_table("USERS"));
    assert!(!is_system_table("some_app_table"));
    assert!(is_table_allowed("bots"));
    assert!(!is_table_allowed("some_app_table"));
}

#[test]
fn injection_scanner_flags_every_documented_pattern() {
    for pattern in ["--", "/*", "*/", "xp_cmdshell", "waitfor delay", "union select"] {
        assert!(
            check_for_injection_patterns(pattern).is_err(),
            "{pattern} must be flagged"
        );
    }
    assert!(check_for_injection_patterns("plain text").is_ok());
}
