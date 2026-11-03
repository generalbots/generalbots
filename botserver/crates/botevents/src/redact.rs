//! Redaction applied before an event is persisted or sent.
//!
//! Two layers:
//!
//! 1. known values (environment-derived credentials, seeded canaries) are
//!    replaced literally;
//! 2. the same credential shapes the security crate masks (`password=`,
//!    `token:`, bearer/basic headers, cloud keys, connection strings, private
//!    keys) are replaced by a stable marker.
//!
//! The production adapter additionally calls
//! `botsecurity-crypto::secrets::redact_sensitive_data`, so the rule set stays
//! in one place; this default implementation keeps the contract crate usable
//! by tests and by the eval harness without pulling the TLS stack in.

use std::sync::OnceLock;

use regex::Regex;
use serde_json::Value;

pub trait FrameRedactor: Send + Sync {
    fn redact(&self, value: &mut Value);
}

const REDACTED: &str = "[REDACTED]";
/// Values shorter than this are not treated as secrets: replacing a four-letter
/// word would destroy ordinary prose.
const MIN_SECRET_LEN: usize = 6;

#[derive(Debug, Clone, Default)]
pub struct DefaultRedactor {
    secrets: Vec<String>,
}

impl DefaultRedactor {
    #[must_use]
    pub fn new() -> Self {
        Self { secrets: Vec::new() }
    }

    /// Adds literal values to mask. Duplicates are removed and the longest
    /// value is applied first so a secret that contains another is masked as a
    /// whole.
    #[must_use]
    pub fn with_secrets<I, S>(mut self, secrets: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        for secret in secrets {
            let secret = secret.into();
            let trimmed = secret.trim();
            if trimmed.len() >= MIN_SECRET_LEN && !self.secrets.iter().any(|known| known == trimmed)
            {
                self.secrets.push(trimmed.to_string());
            }
        }
        self.secrets.sort_by_key(|secret| std::cmp::Reverse(secret.len()));
        self
    }

    /// Collects credential-looking environment values. A missing or malformed
    /// value never aborts: redaction degrades to the pattern layer.
    #[must_use]
    pub fn from_env() -> Self {
        let mut values = Vec::new();
        for (name, value) in std::env::vars() {
            let upper = name.to_uppercase();
            let credential_name = ["KEY", "TOKEN", "SECRET", "PASSWORD", "CREDENTIAL"]
                .iter()
                .any(|needle| upper.contains(needle));
            if credential_name {
                values.push(value);
            }
        }
        Self::new().with_secrets(values)
    }

    #[must_use]
    pub fn redact_text(&self, text: &str) -> String {
        let mut result = text.to_string();
        for secret in &self.secrets {
            if !secret.is_empty() {
                result = result.replace(secret.as_str(), REDACTED);
            }
        }
        for (pattern, replacement) in masking_rules() {
            result = pattern.replace_all(&result, *replacement).into_owned();
        }
        result
    }
}

impl FrameRedactor for DefaultRedactor {
    fn redact(&self, value: &mut Value) {
        scrub_value(value, self);
    }
}

fn scrub_value(value: &mut Value, redactor: &DefaultRedactor) {
    match value {
        Value::String(text) => *text = redactor.redact_text(text),
        Value::Array(items) => {
            for item in items {
                scrub_value(item, redactor);
            }
        }
        Value::Object(map) => {
            for item in map.values_mut() {
                scrub_value(item, redactor);
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) => {}
    }
}

fn masking_rules() -> &'static [(Regex, &'static str)] {
    static RULES: OnceLock<Vec<(Regex, &'static str)>> = OnceLock::new();
    RULES.get_or_init(|| {
        const PATTERNS: [(&str, &str); 12] = [
            (r"(?i)password\s*[=:]\s*\S+", "password=[REDACTED]"),
            (r"(?i)api[_-]?key\s*[=:]\s*\S+", "api_key=[REDACTED]"),
            (r"(?i)\btoken\s*[=:]\s*\S+", "token=[REDACTED]"),
            (r"(?i)\bsecret\s*[=:]\s*\S+", "secret=[REDACTED]"),
            (
                r"(?i)bearer\s+[A-Za-z0-9\-._~+/]+=*",
                "Bearer [REDACTED]",
            ),
            (r"(?i)basic\s+[A-Za-z0-9+/]{8,}=*", "Basic [REDACTED]"),
            (r"AKIA[0-9A-Z]{16}", "[REDACTED]"),
            (r"sk-[A-Za-z0-9_-]{20,}", "[REDACTED]"),
            (r"gh[pousr]_[A-Za-z0-9]{20,}", "[REDACTED]"),
            (r"xox[baprs]-[A-Za-z0-9-]{10,}", "[REDACTED]"),
            (
                r"-----BEGIN [A-Z ]*PRIVATE KEY-----[\s\S]*?-----END [A-Z ]*PRIVATE KEY-----",
                "[REDACTED PRIVATE KEY]",
            ),
            (
                r"(?i)://[^/\s:@]+:[^/\s@]+@",
                "://[REDACTED]@",
            ),
        ];
        PATTERNS
            .iter()
            .filter_map(|(pattern, replacement)| {
                Regex::new(pattern)
                    .ok()
                    .map(|regex| (regex, *replacement))
            })
            .collect()
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{DefaultRedactor, FrameRedactor};

    #[test]
    fn masks_known_values_everywhere() {
        let redactor = DefaultRedactor::new().with_secrets(["canary-value-123"]);
        let mut value = json!({
            "message": "deploy with canary-value-123 now",
            "nested": ["canary-value-123"],
            "count": 3
        });
        redactor.redact(&mut value);
        let text = value.to_string();
        assert!(!text.contains("canary-value-123"));
        assert!(text.contains("[REDACTED]"));
        assert_eq!(value.get("count").and_then(|v| v.as_u64()), Some(3));
    }

    #[test]
    fn masks_credential_shapes() {
        let redactor = DefaultRedactor::new();
        let mut value = json!({
            "log": "curl -H 'Authorization: Bearer abcdef0123456789' https://x",
            "dsn": "postgres://user:supersecret@db:5432/app",
            "key": "AKIAIOSFODNN7EXAMPLE"
        });
        redactor.redact(&mut value);
        let text = value.to_string();
        assert!(!text.contains("abcdef0123456789"));
        assert!(!text.contains("supersecret"));
        assert!(!text.contains("AKIAIOSFODNN7EXAMPLE"));
    }
}
