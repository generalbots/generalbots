//! Cell-validation rule expectations.

use super::*;
use botsheet_core::types::ValidationRule;

fn rule(validation_type: &str, value1: Option<&str>, value2: Option<&str>) -> ValidationRule {
    ValidationRule {
        validation_type: validation_type.to_string(),
        operator: None,
        value1: value1.map(str::to_string),
        value2: value2.map(str::to_string),
        allowed_values: None,
        error_title: None,
        error_message: None,
        input_title: None,
        input_message: None,
    }
}

#[test]
fn number_rules_check_parseability_and_bounds() {
    let unbounded = rule("number", None, None);
    assert!(validate_value("42", &unbounded).valid);
    assert!(validate_value("-0.5", &unbounded).valid);
    assert!(!validate_value("abc", &unbounded).valid);

    let bounded = rule("number", Some("10"), Some("20"));
    let cases = [("10", true), ("20", true), ("9.99", false), ("20.01", false)];
    for (value, expected) in cases {
        assert_eq!(validate_value(value, &bounded).valid, expected, "value {value}");
    }

    // A non-numeric bound is ignored rather than rejecting everything.
    assert!(validate_value("1000", &rule("number", Some("abc"), None)).valid);
}

#[test]
fn integer_rules_reject_decimals_and_text() {
    let integer = rule("integer", None, None);
    for value in ["0", "-7", "100"] {
        assert!(validate_value(value, &integer).valid, "value {value}");
    }
    for value in ["1.5", "abc", ""] {
        assert!(!validate_value(value, &integer).valid, "value {value}");
    }
}

#[test]
fn list_rules_compare_exactly_against_the_allowed_values() {
    let mut list = rule("list", None, None);
    assert!(validate_value("anything", &list).valid, "no list means no restriction");

    list.allowed_values = Some(vec!["red".to_string(), "green".to_string()]);
    assert!(validate_value("red", &list).valid);
    assert!(!validate_value("Red", &list).valid);
    assert!(!validate_value("", &list).valid);
}

#[test]
fn date_rules_parse_iso_dates_and_honour_bounds() {
    let unbounded = rule("date", None, None);
    assert!(validate_value("2024-02-29", &unbounded).valid);
    assert!(!validate_value("29/02/2026", &unbounded).valid);

    let bounded = rule("date", Some("2026-01-01"), Some("2026-12-31"));
    assert!(validate_value("2026-01-01", &bounded).valid);
    assert!(validate_value("2026-12-31", &bounded).valid);
    assert!(!validate_value("2025-12-31", &bounded).valid);
    assert!(!validate_value("2027-01-01", &bounded).valid);
}

#[test]
fn text_length_rules_count_characters_not_bytes() {
    let bounded = rule("text_length", Some("2"), Some("4"));
    for value in ["ab", "abcd", "éé", "ação"] {
        assert!(validate_value(value, &bounded).valid, "value {value}");
    }
    for value in ["a", "abcde"] {
        assert!(!validate_value(value, &bounded).valid, "value {value}");
    }
}

#[test]
fn unknown_rule_types_accept_everything_and_failures_carry_a_message() {
    assert!(validate_value("anything", &rule("any", None, None)).valid);
    assert!(validate_value("anything", &rule("", None, None)).valid);

    let result = validate_value("abc", &rule("number", None, None));
    assert!(!result.valid);
    assert_eq!(result.error_message.as_deref(), Some("Invalid value"));

    let mut custom = rule("number", None, None);
    custom.error_message = Some("Numbers only".to_string());
    assert_eq!(
        validate_value("abc", &custom).error_message.as_deref(),
        Some("Numbers only")
    );
    assert_eq!(validate_value("1", &custom).error_message, None);
}
