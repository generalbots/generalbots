//! Contract tests for the ADD BOT trigger normalisation (#2064).
//!
//! `TriggerType` is parsed from a user-supplied string and persisted; an
//! unknown value silently becoming `keyword` is the documented fallback, so it
//! is pinned here.

use super::TriggerType;

#[test]
fn trigger_types_are_parsed_case_insensitively() {
    let cases = [
        ("keyword", TriggerType::Keyword),
        ("TOOL", TriggerType::Tool),
        ("Schedule", TriggerType::Schedule),
        ("event", TriggerType::Event),
        ("always", TriggerType::Always),
    ];
    for (input, expected) in cases {
        assert_eq!(TriggerType::from(input.to_string()), expected, "{input}");
    }
}

#[test]
fn an_unknown_trigger_falls_back_to_keyword() {
    for input in ["", "cron", "web hook", "тool"] {
        assert_eq!(
            TriggerType::from(input.to_string()),
            TriggerType::Keyword,
            "{input:?} must not invent a trigger kind"
        );
    }
}

#[test]
fn trigger_types_render_their_wire_names() {
    let cases = [
        (TriggerType::Keyword, "keyword"),
        (TriggerType::Tool, "tool"),
        (TriggerType::Schedule, "schedule"),
        (TriggerType::Event, "event"),
        (TriggerType::Always, "always"),
    ];
    for (trigger, expected) in cases {
        assert_eq!(trigger.to_string(), expected);
    }
}
