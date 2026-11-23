//! #2624 — one process-wide DLP gate shared by chat, streams and exports.
//!
//! The scanner used to exist only in tests, so live traffic was never checked.
//! This module owns a single lazily built `DlpManager` and exposes an async
//! redaction helper. Detection logic lives in `botsecurity-core` only — channels
//! and export handlers must not reimplement the regexes.

use super::*;
use std::sync::OnceLock;

static DLP: OnceLock<Option<Arc<DlpManager>>> = OnceLock::new();

fn manager() -> Option<&'static Arc<DlpManager>> {
    DLP.get_or_init(|| match DlpManager::with_defaults() {
        Ok(manager) => Some(Arc::new(manager)),
        Err(error) => {
            tracing::error!("DLP manager could not be initialized: {error}");
            None
        }
    })
    .as_ref()
}

/// Scan `content`, record a violation (hash only — never the secret) and return
/// the redacted text. A scanner that cannot be built fails open with a log, the
/// same posture as the rest of the library: DLP is defence in depth, not the
/// only gate in front of the model.
pub async fn redact_text(content: &str, direction: ScanDirection, user_id: Option<Uuid>) -> String {
    let Some(manager) = manager() else {
        return content.to_string();
    };
    let result = match user_id {
        Some(user_id) => manager.scan_with_user(content, user_id, direction).await,
        None => manager.scan_direction(content, direction).await,
    };
    if result.has_sensitive_data() || result.blocked {
        manager
            .record_violation(
                Uuid::nil(),
                "shared-dlp-gate",
                user_id,
                &result,
                content,
            )
            .await;
    }
    if result.blocked {
        return "[blocked by DLP policy]".to_string();
    }
    result
        .redacted_content
        .unwrap_or_else(|| content.to_string())
}

/// Count of recorded violations, for the admin view. Content is never stored.
pub async fn violation_count() -> usize {
    match manager() {
        Some(manager) => manager.get_violations(usize::MAX, None).await.len(),
        None => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn all_sensitive_data_types_are_detected_by_the_shared_gate() {
        let cases: [(SensitiveDataType, &str); 16] = [
            (SensitiveDataType::Email, "alice@example.com"),
            (SensitiveDataType::PhoneNumber, "415-555-0132"),
            (SensitiveDataType::CreditCard, "4111111111111111"),
            (SensitiveDataType::Ssn, "123-45-6789"),
            (SensitiveDataType::IpAddress, "192.168.1.44"),
            (SensitiveDataType::ApiKey, "api_key=abcdefghijklmnopqrstuvwxyz012345"),
            (SensitiveDataType::Password, "password=hunter2secret"),
            (SensitiveDataType::AwsAccessKey, "AKIAIOSFODNN7EXAMPLE"),
            (SensitiveDataType::AwsSecretKey, "aws_secret_access_key=wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"),
            (SensitiveDataType::PrivateKey, "-----BEGIN RSA PRIVATE KEY-----"),
            (SensitiveDataType::JwtToken, "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxIn0.abc123signature"),
            (SensitiveDataType::BankAccount, "account 123456789012"),
            (SensitiveDataType::PassportNumber, "passport X1234567"),
            (SensitiveDataType::DriversLicense, "driver's license D1234567"),
            (SensitiveDataType::HealthId, "medicaid id AB123456789"),
            (SensitiveDataType::Custom, "custom-placeholder-not-matched"),
        ];
        let Ok(manager) = DlpManager::with_defaults() else {
            return;
        };
        for (kind, sample) in cases {
            // Custom has no built-in pattern; the other variants must all fire.
            if kind == SensitiveDataType::Custom {
                continue;
            }
            let result = manager.scan(sample).await;
            assert!(
                result.matches.iter().any(|m| m.data_type == kind),
                "{kind:?} was not detected in {sample:?}"
            );
        }
        // Custom is caller-configured: verify a registered pattern is honoured.
        manager
            .add_custom_pattern("uat-custom", r"CUSTOM-[0-9]{6}", SensitiveDataType::Custom)
            .await
            .expect("valid custom pattern");
        let custom = manager.scan("token CUSTOM-123456").await;
        assert!(custom.matches.iter().any(|m| m.data_type == SensitiveDataType::Custom));
    }

    #[tokio::test]
    async fn redacted_text_never_contains_the_raw_secret() {
        let original = "my card is 4111111111111111 and my key is AKIAIOSFODNN7EXAMPLE";
        let redacted = redact_text(original, ScanDirection::Inbound, None).await;
        assert!(!redacted.contains("4111111111111111"));
        assert!(!redacted.contains("AKIAIOSFODNN7EXAMPLE"));
        assert!(redacted.contains("my card is"));
    }
}
