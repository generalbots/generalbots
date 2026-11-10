use super::*;

#[test]
fn keywords_route_to_the_documented_category_and_priority() {
    let cases = [
        ("login fails after payment", "Authentication", "High"),
        ("forgot my password", "Authentication", "High"),
        ("payment refund requested", "Billing", "High"),
        ("the invoice is wrong", "Billing", "High"),
        ("email not receiving messages", "Email", "Medium"),
        ("the app is slow and lags", "Performance", "High"),
        ("crash on startup", "Bug", "Medium"),
        ("how do I install the agent", "Onboarding", "Low"),
        ("virus on the machine", "Security", "Critical"),
        ("the chat bot answers wrong", "Chat Bot", "Medium"),
    ];
    for (text, category, priority) in cases {
        let (found_category, found_priority, hint) = suggest_category(text);
        assert_eq!(found_category, category, "text={text}");
        assert_eq!(found_priority, priority, "text={text}");
        assert!(!hint.is_empty(), "text={text}");
    }
}

#[test]
fn the_first_matching_rule_wins() {
    // "login" (Authentication, rule 1) appears before "payment" (Billing, rule 2).
    assert_eq!(suggest_category("login fails after payment").0, "Authentication");
    // "error" (Bug, rule 5) appears before "setup" (Onboarding, rule 6).
    assert_eq!(suggest_category("setup error").0, "Bug");
}

#[test]
fn unmatched_text_falls_back_to_general_low() {
    let (category, priority, hint) = suggest_category("something entirely unrelated");
    assert_eq!(category, "General");
    assert_eq!(priority, "Low");
    assert!(hint.contains("triage"));
    assert_eq!(suggest_category("").0, "General");
}

#[test]
fn matching_is_case_sensitive_so_the_caller_lowercases_first() {
    // The handler lowercases the draft before calling this helper; direct
    // mixed-case input does not match and falls back to General.
    assert_eq!(suggest_category("LOGIN FAILS").0, "General");
    assert_eq!(suggest_category("Login is broken").0, "General");
    assert_eq!(suggest_category("login fails").0, "Authentication");
}

#[test]
fn multi_word_keywords_match_across_the_joined_subject_and_description() {
    // The handler joins subject + description with a space, so "sign in" and
    // "not receiving" still match when split across the two fields.
    assert_eq!(suggest_category("cannot sign in today").0, "Authentication");
    assert_eq!(suggest_category("not receiving any mail").0, "Email");
}
