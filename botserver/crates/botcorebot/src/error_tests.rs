//! Offline unit tests for the bot-core error type (#2075).
//!
//! Every subsystem maps its own error into `BotCoreBotError`; the `From` impls
//! decide which bucket a failure lands in, and the `Display` text is what the
//! operator sees in the log. A mis-mapped error hides the real cause, so each
//! conversion is pinned.

use super::BotCoreBotError;

#[test]
fn io_errors_map_to_the_io_bucket() {
    let error = BotCoreBotError::from(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "missing.bas",
    ));
    assert!(matches!(error, BotCoreBotError::Io(_)));
    assert_eq!(error.to_string(), "IO error: missing.bas");
}

#[test]
fn serialization_errors_map_to_the_serialization_bucket() {
    let json_error = serde_json::from_str::<serde_json::Value>("{")
        .expect_err("invalid json");
    let error = BotCoreBotError::from(json_error);
    assert!(matches!(error, BotCoreBotError::Serialization(_)));
    assert!(error.to_string().starts_with("Serialization error: "));
}

#[test]
fn diesel_errors_map_to_the_database_bucket() {
    let error = BotCoreBotError::from(diesel::result::Error::NotFound);
    assert!(matches!(error, BotCoreBotError::Database(_)));
}

#[test]
fn every_variant_keeps_its_documented_prefix() {
    let cases = [
        (BotCoreBotError::Database("x".into()), "Database error: x"),
        (BotCoreBotError::SessionNotFound("s".into()), "Session not found: s"),
        (BotCoreBotError::BotNotFound("b".into()), "Bot not found: b"),
        (BotCoreBotError::AccessDenied("a".into()), "Access denied: a"),
        (BotCoreBotError::Llm("l".into()), "LLM error: l"),
        (BotCoreBotError::Script("s".into()), "Script error: s"),
        (BotCoreBotError::Tool("t".into()), "Tool error: t"),
        (BotCoreBotError::Io("i".into()), "IO error: i"),
        (
            BotCoreBotError::Serialization("j".into()),
            "Serialization error: j",
        ),
        (BotCoreBotError::Config("c".into()), "Configuration error: c"),
        (BotCoreBotError::Cache("k".into()), "Cache error: k"),
        (BotCoreBotError::Internal("n".into()), "Internal error: n"),
    ];
    for (error, expected) in cases {
        assert_eq!(error.to_string(), expected);
    }
}
