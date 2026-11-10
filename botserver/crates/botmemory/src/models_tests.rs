//! Table-driven tests for the validation rules applied to every memory write.
//!
//! The rules are the crate's contract with the HTTP surface: a rejected value
//! must never reach the table, and an accepted value must already be in the
//! normalized form the dedupe engine compares.

use super::*;

#[test]
fn scope_accepts_only_private_and_branch() {
    let accepted = [
        ("private", "private"),
        ("branch", "branch"),
        ("  PRIVATE ", "private"),
        ("\tbranch\n", "branch"),
    ];
    for (input, expected) in accepted {
        assert_eq!(ensure_scope(input).as_deref(), Ok(expected), "scope {input:?}");
    }

    let rejected = ["", "   ", "public", "shared", "branchx", "private-cloud"];
    for input in rejected {
        let error = ensure_scope(input).expect_err("must reject scope");
        assert!(
            error.contains("invalid scope") && error.contains("private or branch"),
            "scope {input:?} produced {error:?}"
        );
    }
}

#[test]
fn scope_error_quotes_the_offending_value() {
    let error = ensure_scope("tenant").expect_err("must reject");
    assert!(error.contains("'tenant'"), "{error:?}");
}

#[test]
fn kind_normalizes_to_lowercase_and_defaults_to_fact() {
    assert_eq!(ensure_kind("FACT").as_deref(), Ok("fact"));
    assert_eq!(ensure_kind("  Preference ").as_deref(), Ok("preference"));
    assert_eq!(ensure_kind("").as_deref(), Ok("fact"));
    assert_eq!(ensure_kind("   ").as_deref(), Ok("fact"));
}

#[test]
fn kind_is_bounded_by_characters_not_bytes() {
    let at_limit = "k".repeat(KIND_MAX_CHARS);
    assert_eq!(ensure_kind(&at_limit).as_deref(), Ok(at_limit.as_str()));

    let over_limit = "k".repeat(KIND_MAX_CHARS + 1);
    let error = ensure_kind(&over_limit).expect_err("must reject long kind");
    assert!(error.contains("characters"), "{error:?}");

    // Multi-byte characters count once each: 32 accents are 32 chars.
    let multibyte = "é".repeat(KIND_MAX_CHARS);
    assert!(ensure_kind(&multibyte).is_ok());
}

#[test]
fn content_is_trimmed_and_must_not_be_blank() {
    assert_eq!(ensure_content("  likes tea  ").as_deref(), Ok("likes tea"));
    assert_eq!(ensure_content("line one\nline two").as_deref(), Ok("line one\nline two"));

    for blank in ["", "   ", "\n\t"] {
        let error = ensure_content(blank).expect_err("must reject blank content");
        assert_eq!(error, "content must not be empty");
    }
}

#[test]
fn content_bound_is_measured_after_trimming() {
    let body = "a".repeat(CONTENT_MAX_CHARS);
    let padded = format!("   {body}   ");
    assert_eq!(ensure_content(&padded).as_deref(), Ok(body.as_str()));

    let over = "a".repeat(CONTENT_MAX_CHARS + 1);
    let error = ensure_content(&over).expect_err("must reject oversized content");
    assert!(error.contains("characters"), "{error:?}");
}

#[test]
fn source_defaults_to_manual_and_is_bounded() {
    assert_eq!(ensure_source("").as_deref(), Ok("manual"));
    assert_eq!(ensure_source("   ").as_deref(), Ok("manual"));
    assert_eq!(ensure_source(" conversation:s1 ").as_deref(), Ok("conversation:s1"));

    let at_limit = "s".repeat(SOURCE_MAX_CHARS);
    assert!(ensure_source(&at_limit).is_ok());

    let over_limit = "s".repeat(SOURCE_MAX_CHARS + 1);
    let error = ensure_source(&over_limit).expect_err("must reject long source");
    assert!(error.contains("characters"), "{error:?}");
}

#[test]
fn import_body_deserialization_applies_the_documented_defaults() {
    let body: ImportBody =
        serde_json::from_str(r#"{"items":[{"content":"likes tea"}]}"#).expect("body parses");
    assert!(!body.dry_run);
    let item = &body.items[0];
    assert_eq!(item.kind, "fact");
    assert_eq!(item.scope, SCOPE_PRIVATE);
    assert_eq!(item.confidence, 0.8);
    assert!(!item.pinned);
    assert_eq!(item.source, None);
}

#[test]
fn memory_body_requires_content() {
    let missing = serde_json::from_str::<MemoryBody>(r#"{"kind":"fact"}"#);
    assert!(missing.is_err());

    let body: MemoryBody = serde_json::from_str(r#"{"content":"likes tea"}"#).expect("body parses");
    assert_eq!(body.kind, "fact");
    assert_eq!(body.scope, SCOPE_PRIVATE);
    assert!(!body.pinned);
}
