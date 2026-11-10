//! Redaction rules: literal secrets first, then credential shapes.

use serde_json::json;

use super::{DefaultRedactor, FrameRedactor};

fn masked(text: &str) -> String {
    DefaultRedactor::new().redact_text(text)
}

#[test]
fn literal_secrets_are_masked_everywhere() {
    let redactor = DefaultRedactor::new().with_secrets(["canary-value-123"]);
    assert_eq!(redactor.redact_text("a canary-value-123 b"), "a [REDACTED] b");
    assert_eq!(
        redactor.redact_text("canary-value-123canary-value-123"),
        "[REDACTED][REDACTED]"
    );
}

#[test]
fn short_and_blank_secrets_are_ignored() {
    let redactor = DefaultRedactor::new().with_secrets(["abc", "", "   "]);
    assert_eq!(redactor.redact_text("abc stays readable"), "abc stays readable");
}

#[test]
fn secrets_are_trimmed_deduplicated_and_applied_longest_first() {
    let redactor = DefaultRedactor::new().with_secrets([
        "  abc123456  ",
        "abc123456",
        "abc123",
    ]);
    assert_eq!(redactor.redact_text("abc123456"), "[REDACTED]");
    assert_eq!(redactor.redact_text("abc123"), "[REDACTED]");
    assert_eq!(redactor.redact_text("abc123456"), "[REDACTED]");
}

#[test]
fn default_redactor_is_the_empty_constructor() {
    assert_eq!(
        DefaultRedactor::default().redact_text("nothing secret"),
        DefaultRedactor::new().redact_text("nothing secret")
    );
}

#[test]
fn assignment_shapes_are_masked() {
    assert_eq!(masked("password=hunter2"), "password=[REDACTED]");
    assert_eq!(masked("Password: hunter2"), "password=[REDACTED]");
    assert_eq!(masked("api-key=abc"), "api_key=[REDACTED]");
    assert_eq!(masked("API_KEY: abc"), "api_key=[REDACTED]");
    assert_eq!(masked("token=abc"), "token=[REDACTED]");
    assert_eq!(masked("secret: abc"), "secret=[REDACTED]");
}

#[test]
fn authorization_headers_are_masked() {
    assert_eq!(
        masked("Authorization: Bearer abc.def-123"),
        "Authorization: Bearer [REDACTED]"
    );
    assert_eq!(
        masked("Authorization: Basic dXNlcjpwYXNz"),
        "Authorization: Basic [REDACTED]"
    );
}

#[test]
fn published_key_shapes_are_masked() {
    assert_eq!(masked("AKIAIOSFODNN7EXAMPLE"), "[REDACTED]");
    assert_eq!(
        masked("sk-abcdefghijklmnopqrstuvwx"),
        "[REDACTED]"
    );
    assert_eq!(
        masked("ghp_abcdefghijklmnopqrstuvwxyz"),
        "[REDACTED]"
    );
    assert_eq!(
        masked("xoxb-1234567890-abcdef"),
        "[REDACTED]"
    );
}

#[test]
fn private_key_blocks_are_masked_as_a_whole() {
    let text = "-----BEGIN RSA PRIVATE KEY-----\nMIIabc\n-----END RSA PRIVATE KEY-----";
    assert_eq!(masked(text), "[REDACTED PRIVATE KEY]");
}

#[test]
fn connection_string_credentials_are_masked() {
    assert_eq!(
        masked("postgres://user:supersecret@db:5432/app"),
        "postgres://[REDACTED]@db:5432/app"
    );
}

#[test]
fn ordinary_prose_is_untouched() {
    let text = "The password policy is documented; tokens are short.";
    assert_eq!(masked(text), text);
}

#[test]
fn the_frame_walker_reaches_every_nested_string() {
    let redactor = DefaultRedactor::new().with_secrets(["canary-value-123"]);
    let mut value = json!({
        "top": "canary-value-123",
        "list": ["canary-value-123", 3, null, true],
        "object": { "deep": ["canary-value-123"] },
    });
    redactor.redact(&mut value);
    assert_eq!(value["top"], "[REDACTED]");
    assert_eq!(value["list"][0], "[REDACTED]");
    assert_eq!(value["list"][1], 3);
    assert_eq!(value["list"][2], serde_json::Value::Null);
    assert_eq!(value["list"][3], true);
    assert_eq!(value["object"]["deep"][0], "[REDACTED]");
}

#[test]
fn a_scalar_frame_is_redacted_in_place() {
    let redactor = DefaultRedactor::new().with_secrets(["canary-value-123"]);
    let mut value = json!("canary-value-123");
    redactor.redact(&mut value);
    assert_eq!(value, json!("[REDACTED]"));
}
