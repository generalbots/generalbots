//! DLP detection tests, split out of `dlp/mod.rs` to keep that file inside
//! the project size limit.

use super::extract_context::{create_default_policies, validate_credit_card_checksum};
use super::*;

#[tokio::test]
async fn test_email_detection() {
    let manager = DlpManager::with_defaults().expect("Failed to create manager");

    let content = "Contact me at test@example.com for more info";
    let result = manager.scan(content).await;

    assert!(result.has_sensitive_data());
    assert!(result.data_types_found().contains(&SensitiveDataType::Email));
}

#[tokio::test]
async fn test_credit_card_detection() {
    let manager = DlpManager::with_defaults().expect("Failed to create manager");

    let content = "My card number is 4111-1111-1111-1111";
    let result = manager.scan(content).await;

    assert!(result.has_sensitive_data());
    assert!(result.data_types_found().contains(&SensitiveDataType::CreditCard));
}

#[tokio::test]
async fn test_ssn_detection() {
    let manager = DlpManager::with_defaults().expect("Failed to create manager");

    let content = "SSN: 123-45-6789";
    let result = manager.scan(content).await;

    assert!(result.has_sensitive_data());
    assert!(result.data_types_found().contains(&SensitiveDataType::Ssn));
}

#[tokio::test]
async fn test_aws_key_detection() {
    let manager = DlpManager::with_defaults().expect("Failed to create manager");

    let content = "AWS key: AKIAIOSFODNN7EXAMPLE";
    let result = manager.scan(content).await;

    assert!(result.has_sensitive_data());
    assert!(result.data_types_found().contains(&SensitiveDataType::AwsAccessKey));
}

#[tokio::test]
async fn test_jwt_detection() {
    let manager = DlpManager::with_defaults().expect("Failed to create manager");

    let content = "Token: eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U";
    let result = manager.scan(content).await;

    assert!(result.has_sensitive_data());
    assert!(result.data_types_found().contains(&SensitiveDataType::JwtToken));
}

#[tokio::test]
async fn test_clean_content() {
    let manager = DlpManager::with_defaults().expect("Failed to create manager");

    let content = "This is just a regular message with no sensitive data.";
    let result = manager.scan(content).await;

    assert!(!result.has_sensitive_data());
    assert_eq!(result.action, DlpAction::Allow);
}

#[tokio::test]
async fn test_redaction() {
    let manager = DlpManager::with_defaults().expect("Failed to create manager");

    let content = "Email me at user@example.com please";
    let result = manager.scan(content).await;

    assert!(result.redacted_content.is_some());
    let redacted = result.redacted_content.unwrap();
    assert!(redacted.contains("[EMAIL REDACTED]"));
    assert!(!redacted.contains("user@example.com"));
}

#[tokio::test]
async fn test_partial_redaction() {
    let manager = DlpManager::with_defaults().expect("Failed to create manager");

    let content = "Card: 4111111111111111";
    let result = manager.scan(content).await;

    let partial = manager.partial_redact_content(content, &result.matches);
    assert!(partial.contains("41"));
    assert!(partial.contains("11"));
    assert!(partial.contains("*"));
}

#[tokio::test]
async fn test_multiple_matches() {
    let manager = DlpManager::with_defaults().expect("Failed to create manager");

    let content = "Contact: test@example.com, phone: 555-123-4567, card: 4111111111111111";
    let result = manager.scan(content).await;

    assert!(result.match_count() >= 3);
    assert!(result.data_types_found().contains(&SensitiveDataType::Email));
    assert!(result.data_types_found().contains(&SensitiveDataType::PhoneNumber));
    assert!(result.data_types_found().contains(&SensitiveDataType::CreditCard));
}

#[tokio::test]
async fn test_policy_evaluation() {
    let manager = DlpManager::with_defaults().expect("Failed to create manager");

    let policy = DlpPolicy::new(
        "Block SSN",
        vec![SensitiveDataType::Ssn],
        DlpAction::Block,
    );
    manager.add_policy(policy).await;

    let content = "SSN: 123-45-6789";
    let result = manager.scan(content).await;

    assert!(result.blocked);
    assert_eq!(result.action, DlpAction::Block);
}

#[test]
fn test_credit_card_checksum_valid() {
    assert!(validate_credit_card_checksum("4111111111111111"));
    assert!(validate_credit_card_checksum("5500000000000004"));
}

#[test]
fn test_credit_card_checksum_invalid() {
    assert!(!validate_credit_card_checksum("4111111111111112"));
    assert!(!validate_credit_card_checksum("1234567890123456"));
}

#[test]
fn test_severity_levels() {
    assert_eq!(SensitiveDataType::CreditCard.severity(), DlpSeverity::Critical);
    assert_eq!(SensitiveDataType::Email.severity(), DlpSeverity::Medium);
    assert_eq!(SensitiveDataType::IpAddress.severity(), DlpSeverity::Low);
}

#[test]
fn test_dlp_match_masking() {
    let dlp_match = DlpMatch::new(
        SensitiveDataType::CreditCard,
        "4111111111111111",
        0,
        16,
    );

    let masked = dlp_match.masked_value();
    assert!(masked.starts_with("41"));
    assert!(masked.ends_with("11"));
    assert!(masked.contains("*"));
}

#[test]
fn test_default_policies() {
    let policies = create_default_policies();

    assert!(!policies.is_empty());
    assert!(policies.iter().any(|p| p.name == "Block Credit Cards"));
    assert!(policies.iter().any(|p| p.name == "Block SSN"));
}

#[tokio::test]
async fn test_custom_pattern() {
    let manager = DlpManager::with_defaults().expect("Failed to create manager");

    manager
        .add_custom_pattern("Custom ID", r"CUSTOM-\d{6}", SensitiveDataType::Custom)
        .await
        .expect("Failed to add pattern");

    let content = "ID: CUSTOM-123456";
    let result = manager.scan(content).await;

    assert!(result.has_sensitive_data());
    assert!(result.data_types_found().contains(&SensitiveDataType::Custom));
}

#[test]
fn test_policy_exception() {
    let mut policy = DlpPolicy::new(
        "Test Policy",
        vec![SensitiveDataType::Email],
        DlpAction::Block,
    );

    policy.add_exception(DlpException::Domain("@internal.com".into()));

    assert!(policy.is_exception("user@internal.com", None));
    assert!(!policy.is_exception("user@external.com", None));
}

#[test]
fn test_extract_context() {
    let content = "This is a test email@example.com in the middle";
    let context = super::extract_context::extract_context(content, 15, 33, 10);

    assert!(context.contains("email@example.com"));
    assert!(context.len() < content.len() + 10);
}

#[tokio::test]
async fn test_scan_direction() {
    let config = DlpConfig {
        scan_inbound: false,
        ..Default::default()
    };
    let manager = DlpManager::new(config).expect("Failed to create manager");

    let content = "Email: test@example.com";
    let user_id = Uuid::new_v4();

    let inbound_result = manager
        .scan_with_user(content, user_id, ScanDirection::Inbound)
        .await;
    assert!(!inbound_result.has_sensitive_data());

    let outbound_result = manager
        .scan_with_user(content, user_id, ScanDirection::Outbound)
        .await;
    assert!(outbound_result.has_sensitive_data());
}

#[tokio::test]
async fn ssn_api_keys_and_jwt_are_detected() {
    let manager = DlpManager::with_defaults().expect("manager");

    let ssn = manager.scan("SSN 123-45-6789").await;
    assert!(ssn.data_types_found().contains(&SensitiveDataType::Ssn));

    // The detector keys off an `api_key=`-style label, not a bare token.
    let key = manager
        .scan("api_key=abcdefghijklmnopqrstuvwxyz012345")
        .await;
    assert!(key.data_types_found().contains(&SensitiveDataType::ApiKey));

    let jwt = manager
        .scan("Authorization: Bearer eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxIn0.abc123")
        .await;
    assert!(jwt.has_sensitive_data());
}

#[tokio::test]
async fn clean_content_reports_nothing() {
    let manager = DlpManager::with_defaults().expect("manager");
    let result = manager.scan("The quarterly report is ready.").await;
    assert!(!result.has_sensitive_data());
    assert_eq!(result.match_count(), 0);
    assert!(!result.blocked);
    assert!(result.redacted_content.is_none());
}

#[tokio::test]
async fn a_disabled_manager_scans_nothing() {
    let mut config = DlpConfig::default();
    config.enabled = false;
    let manager = DlpManager::new(config).expect("manager");

    let result = manager.scan("card 4111-1111-1111-1111 test@example.com").await;
    assert!(!result.has_sensitive_data());
    assert!(!result.blocked);
}

#[tokio::test]
async fn oversized_content_is_blocked_before_matching() {
    let mut config = DlpConfig::default();
    config.max_content_length = 16;
    let manager = DlpManager::new(config).expect("manager");

    let result = manager.scan(&"x".repeat(64)).await;
    assert!(result.blocked);
    assert!(result.matches.is_empty());
    assert!(!result.policy_violations.is_empty());
}

#[tokio::test]
async fn redaction_removes_the_sensitive_value() {
    let mut config = DlpConfig::default();
    config.redact_sensitive_data = true;
    let manager = DlpManager::new(config).expect("manager");

    let result = manager.scan("mail me at test@example.com please").await;
    let redacted = result.redacted_content.expect("redacted content");
    assert!(!redacted.contains("test@example.com"));
    assert!(redacted.contains("please"));
}

#[tokio::test]
async fn the_credit_card_checksum_accepts_only_valid_numbers() {
    assert!(validate_credit_card_checksum("4111111111111111"));
    assert!(validate_credit_card_checksum("5555 5555 5555 4444"));
    assert!(!validate_credit_card_checksum("4111111111111112"));
    assert!(!validate_credit_card_checksum("1234"));
}
