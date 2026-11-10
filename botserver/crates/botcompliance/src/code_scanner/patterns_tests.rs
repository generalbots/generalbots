//! Offline unit tests for the compliance scan patterns (#2075).
//!
//! The scanner is the only thing that flags a hardcoded secret before it ships.
//! `build_patterns` is pure (regex compilation only), so each pattern is fed a
//! representative positive and negative sample here; the filesystem walk that
//! applies them is covered by the integration suite.

use super::*;

fn matches(title: &str, sample: &str) -> bool {
    let patterns = CodeScanner::build_patterns().expect("patterns compile");
    let pattern = patterns
        .iter()
        .find(|p| p.title == title)
        .unwrap_or_else(|| panic!("no pattern titled {title}"));
    pattern.regex.is_match(sample)
}

#[test]
fn every_pattern_is_complete_and_unique() {
    let patterns = CodeScanner::build_patterns().expect("patterns compile");
    assert!(patterns.len() >= 12, "expected the full catalog");

    let mut titles: Vec<&str> = patterns.iter().map(|p| p.title.as_str()).collect();
    let total = titles.len();
    titles.sort_unstable();
    titles.dedup();
    assert_eq!(titles.len(), total, "pattern titles must be unique");

    for pattern in &patterns {
        assert!(!pattern.description.is_empty(), "{} has no description", pattern.title);
        assert!(!pattern.remediation.is_empty(), "{} has no remediation", pattern.title);
        assert!(!pattern.category.is_empty(), "{} has no category", pattern.title);
        assert!(
            pattern.severity >= IssueSeverity::Low,
            "{} must carry a severity",
            pattern.title
        );
    }
}

#[test]
fn hardcoded_passwords_are_critical() {
    let patterns = CodeScanner::build_patterns().expect("patterns compile");
    let pattern = patterns
        .iter()
        .find(|p| p.title == "Hardcoded Password")
        .expect("password pattern");
    assert_eq!(pattern.severity, IssueSeverity::Critical);
    assert_eq!(pattern.issue_type, IssueType::PasswordInConfig);

    assert!(matches("Hardcoded Password", "PASSWORD = \"hunter2\""));
    assert!(matches("Hardcoded Password", "password='x'"));
    // A variable reference is not a literal.
    assert!(!matches("Hardcoded Password", "password = os.environ[\"PW\"]"));
    assert!(!matches("Hardcoded Password", "password = vault_get(\"pw\")"));
}

#[test]
fn api_keys_tokens_and_cloud_credentials_are_detected() {
    assert!(matches("Hardcoded API Key/Secret", "api_key = \"abcdefgh123\""));
    assert!(matches("Hardcoded API Key/Secret", "client_secret=\"s3cr3t-value\""));
    // Shorter than eight characters is too weak to distinguish from a variable.
    assert!(!matches("Hardcoded API Key/Secret", "api_key = \"short\""));
    // An identifier that merely contains "key" is not an assignment.
    assert!(!matches("Hardcoded API Key/Secret", "keyboard = layout"));

    assert!(matches(
        "Hardcoded Token",
        "token = \"abcdefghijklmnopqrstuvwx\""
    ));
    assert!(!matches("Hardcoded Token", "token = \"short\""));

    assert!(matches("AWS Access Key", "AKIAIOSFODNN7EXAMPLE"));
    assert!(matches(
        "Private Key in Code",
        "-----BEGIN RSA PRIVATE KEY-----"
    ));
    assert!(matches(
        "Private Key in Code",
        "-----BEGIN PRIVATE KEY-----"
    ));
}

#[test]
fn connection_strings_with_credentials_are_detected() {
    assert!(matches(
        "Database Credentials in Connection String",
        "postgres://user:secret@db.internal/app"
    ));
    assert!(matches(
        "Database Credentials in Connection String",
        "redis://default:pw@cache:6379"
    ));
    // A connection string without a password is fine.
    assert!(!matches(
        "Database Credentials in Connection String",
        "postgres://db.internal/app"
    ));
}

#[test]
fn dangerous_code_constructs_are_detected() {
    assert!(matches("Dynamic Code Execution", "result = EVAL(\"1+1\")"));
    assert!(matches("Dynamic Code Execution", "eval (payload)"));
    assert!(!matches("Dynamic Code Execution", "evaluate_score = 1"));

    assert!(matches(
        "Raw SQL Query",
        "SELECT * FROM customers WHERE id = 1"
    ));
    assert!(matches("Raw SQL Query", "delete from sessions "));

    assert!(matches("Deprecated IF...input Pattern", "IF input = \"yes\" THEN"));
    assert!(!matches("Deprecated IF...input Pattern", "IF answer = \"yes\" THEN"));
}

#[test]
fn underscore_keywords_are_flagged_as_low_severity_naming_issues() {
    let patterns = CodeScanner::build_patterns().expect("patterns compile");
    let pattern = patterns
        .iter()
        .find(|p| p.title == "Underscore in Keyword")
        .expect("keyword pattern");
    assert_eq!(pattern.severity, IssueSeverity::Low);
    assert_eq!(pattern.issue_type, IssueType::UnderscoreInKeyword);

    for keyword in [
        "GET_BOT_MEMORY",
        "SET_BOT_MEMORY",
        "GET_USER_MEMORY",
        "SET_USER_MEMORY",
        "USE_KB",
        "USE_TOOL",
        "SEND_MAIL",
        "CREATE_TASK",
    ] {
        assert!(matches("Underscore in Keyword", keyword), "{keyword}");
    }
    assert!(!matches("Underscore in Keyword", "GET BOT MEMORY"));
}

#[test]
fn ordinary_basic_code_produces_no_findings() {
    let patterns = CodeScanner::build_patterns().expect("patterns compile");
    let safe = [
        "TALK \"Hello from the bot\"",
        "ADD_SUGGESTION \"Check inventory\"",
        "response = GET VAULT SECRET \"api_key\"",
        "HEAR answer AS STRING",
        "SAVE customer",
    ];
    for line in safe {
        let hits: Vec<&str> = patterns
            .iter()
            .filter(|p| p.regex.is_match(line))
            .map(|p| p.title.as_str())
            .collect();
        assert!(hits.is_empty(), "{line:?} matched {hits:?}");
    }
}

#[test]
fn severity_ordering_matches_the_reported_ladder() {
    assert!(IssueSeverity::Info < IssueSeverity::Low);
    assert!(IssueSeverity::Low < IssueSeverity::Medium);
    assert!(IssueSeverity::Medium < IssueSeverity::High);
    assert!(IssueSeverity::High < IssueSeverity::Critical);
}
