//! Tests for the defensive LLM-output parser used by memory extraction.

use super::*;

#[test]
fn an_empty_or_prose_answer_yields_nothing() {
    for raw in ["", "   ", "I could not find anything durable.", "null"] {
        assert!(parse_extractions(raw).is_empty(), "{raw:?}");
    }
}

#[test]
fn at_most_eight_items_are_kept() {
    let items: Vec<String> = (0..12)
        .map(|i| format!("{{\"kind\":\"fact\",\"content\":\"memory {i}\"}}"))
        .collect();
    let raw = format!("[{}]", items.join(","));
    assert_eq!(parse_extractions(&raw).len(), MAX_EXTRACTED_ITEMS);
}

#[test]
fn content_is_clipped_to_the_documented_limit() {
    let long = "x".repeat(CONTENT_MAX_CHARS + 40);
    let raw = format!("[{{\"content\":\"{long}\"}}]");
    let parsed = parse_extractions(&raw);
    assert_eq!(parsed[0].content.chars().count(), CONTENT_MAX_CHARS);
}

#[test]
fn kind_is_lowercased_clipped_and_defaulted() {
    let long_kind = "K".repeat(KIND_MAX_CHARS + 10);
    let raw = format!(
        "[{{\"kind\":\"{long_kind}\",\"content\":\"a\"}},{{\"kind\":\"  \",\"content\":\"b\"}},\
          {{\"kind\":42,\"content\":\"c\"}}]"
    );
    let parsed = parse_extractions(&raw);
    assert_eq!(parsed[0].kind.chars().count(), KIND_MAX_CHARS);
    assert_eq!(parsed[0].kind, "k".repeat(KIND_MAX_CHARS));
    assert_eq!(parsed[1].kind, KIND_DEFAULT);
    assert_eq!(parsed[2].kind, KIND_DEFAULT);
}

#[test]
fn confidence_is_clamped_and_non_numeric_values_default() {
    let raw = "[{\"content\":\"a\",\"confidence\":-3},{\"content\":\"b\",\"confidence\":0.25},\
               {\"content\":\"c\",\"confidence\":\"high\"}]";
    let parsed = parse_extractions(raw);
    assert_eq!(parsed[0].confidence, 0.0);
    assert_eq!(parsed[1].confidence, 0.25);
    assert_eq!(parsed[2].confidence, CONFIDENCE_DEFAULT);
}

#[test]
fn entries_without_usable_content_are_dropped() {
    let raw = "[{\"kind\":\"fact\"},{\"content\":null},{\"content\":3},{\"content\":\"   \"},\
               {\"content\":\" kept \"}]";
    let parsed = parse_extractions(raw);
    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed[0].content, "kept");
}

#[test]
fn fences_are_stripped_before_parsing() {
    let cases = [
        "```json\n[{\"content\":\"a\"}]\n```",
        "```JSON\n[{\"content\":\"a\"}]\n```",
        "```\n[{\"content\":\"a\"}]\n```",
        "  [{\"content\":\"a\"}]  ",
    ];
    for raw in cases {
        assert_eq!(parse_extractions(raw).len(), 1, "{raw:?}");
    }
}

#[test]
fn an_array_embedded_in_prose_is_recovered() {
    let raw = "Here is what I found:\n[{\"kind\":\"goal\",\"content\":\"learns rust\"}]\nDone.";
    let parsed = parse_extractions(raw);
    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed[0].kind, "goal");
}

#[test]
fn a_wrapper_object_is_accepted_in_both_shapes() {
    let raw = "{\"memories\":[{\"content\":\"a\"},{\"content\":\"b\"}]}";
    assert_eq!(parse_extractions(raw).len(), 2);

    let empty = "{\"memories\":[]}";
    assert!(parse_extractions(empty).is_empty());

    // Any object that embeds an array is recovered by the bracket scan, not
    // only the documented `memories` wrapper.
    let no_key = "{\"items\":[{\"content\":\"a\"}]}";
    assert_eq!(parse_extractions(no_key).len(), 1);
}

#[test]
fn explicit_remember_matches_the_documented_substrings() {
    assert!(is_explicit_remember("remember that I am vegetarian"));
    assert!(is_explicit_remember("Please, REMEMBER THAT this is important"));
    assert!(is_explicit_remember("lembre que eu prefiro ligações"));
    assert!(!is_explicit_remember("lembra disso"));
    assert!(!is_explicit_remember("what do I prefer?"));
    // The probe is a substring test, not an imperative detector: a question
    // that quotes the phrase also samples the turn. Documented, not enforced.
    assert!(is_explicit_remember("do you remember that I hate crowds?"));
}
