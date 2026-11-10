//! Offline unit tests for the plan AI suggestion parser (#2068).
//!
//! The planning assistant returns free-form model output; `parse_plan_suggestions`
//! is the only thing standing between that text and the task board. It must
//! accept a bare JSON array, tolerate the fences and prose models add around it,
//! and fail closed (empty vector) on anything else.

use super::{default_priority, default_status, parse_plan_suggestions};

#[test]
fn defaults_match_the_task_board_vocabulary() {
    assert_eq!(default_status(), "todo");
    assert_eq!(default_priority(), "medium");
}

#[test]
fn a_bare_array_is_parsed() {
    let text = r#"[
        {"title":"Write spec","status":"todo","priority":"high","assignee":"ada"},
        {"title":"Review","status":"review","priority":"low","assignee":null}
    ]"#;
    let parsed = parse_plan_suggestions(text);
    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed[0].title, "Write spec");
    assert_eq!(parsed[0].status, "todo");
    assert_eq!(parsed[0].priority, "high");
    assert_eq!(parsed[0].assignee.as_deref(), Some("ada"));
    assert_eq!(parsed[1].title, "Review");
    assert!(parsed[1].assignee.is_none());
}

#[test]
fn markdown_fences_and_prose_around_the_array_are_ignored() {
    let text = "Sure! Here are the tasks:\n```json\n[{\"title\":\"One\"}]\n```\nLet me know.";
    let parsed = parse_plan_suggestions(text);
    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed[0].title, "One");
}

#[test]
fn missing_fields_fall_back_to_the_defaults() {
    let parsed = parse_plan_suggestions(r#"[{"title":"Bare"}]"#);
    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed[0].title, "Bare");
    assert_eq!(parsed[0].status, default_status());
    assert_eq!(parsed[0].priority, default_priority());
    assert!(parsed[0].assignee.is_none());
}

#[test]
fn an_empty_array_is_an_empty_result() {
    assert!(parse_plan_suggestions("[]").is_empty());
}

#[test]
fn text_without_an_array_yields_nothing() {
    for text in [
        "",
        "I cannot help with that.",
        "no brackets at all",
        "[not json]",
        "{\"title\":\"object, not array\"}",
    ] {
        assert!(
            parse_plan_suggestions(text).is_empty(),
            "{text:?} must not produce suggestions"
        );
    }
}

#[test]
fn a_single_bracket_is_not_enough() {
    assert!(parse_plan_suggestions("[{\"title\":\"x\"}").is_empty());
    assert!(parse_plan_suggestions("{\"title\":\"x\"}]").is_empty());
    // `]` before `[` means the slice is empty/invalid.
    assert!(parse_plan_suggestions("] [").is_empty());
}

#[test]
fn the_outermost_brackets_are_used_when_the_array_is_nested() {
    // `rfind(']')` keeps a nested array inside the first object intact.
    let text = r#"prefix [{"title":"a","status":"todo","priority":"low","assignee":null}] suffix"#;
    let parsed = parse_plan_suggestions(text);
    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed[0].title, "a");
}

#[test]
fn invalid_entries_are_dropped_rather_than_panicking() {
    // A number where the object should be fails the whole parse: the caller
    // gets an empty vector, never a partial or panicking result.
    assert!(parse_plan_suggestions("[1,2,3]").is_empty());
    // An object missing `title` fails too.
    assert!(parse_plan_suggestions(r#"[{"status":"todo"}]"#).is_empty());
}
