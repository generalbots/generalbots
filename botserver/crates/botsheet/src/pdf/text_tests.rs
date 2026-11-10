//! Cell text and content-stream metric expectations.

use super::*;

#[test]
fn pdf_specials_are_escaped_and_non_winansi_becomes_question_mark() {
    let cases = [
        ("plain", "plain"),
        ("a(b)c", "a\\(b\\)c"),
        ("back\\slash", "back\\\\slash"),
        ("café", "caf?"),
        ("a~z", "a~z"),
        ("tab\there", "tab?here"),
    ];
    for (input, expected) in cases {
        assert_eq!(escape_text(input), expected, "input {input:?}");
    }
}

#[test]
fn long_values_truncate_by_characters_with_an_ellipsis() {
    assert_eq!(cell_to_text("short"), "short");
    assert_eq!(cell_to_text("0123456789abcdef"), "0123456789abcdef");
    // 17 chars → 15 kept + ellipsis (char-counted, never a byte slice).
    assert_eq!(cell_to_text("0123456789abcdefg"), "0123456789abcde…");
    assert_eq!(cell_to_text("ééééééééééééééééé"), "ééééééééééééééé…");
}

#[test]
fn column_letters_stop_at_the_last_valid_column() {
    let cases = [(0, "A"), (25, "Z"), (26, "AA"), (27, "AB"), (16383, "XFD")];
    for (col, expected) in cases {
        assert_eq!(column_letter(col), expected, "column {col}");
    }
    assert_eq!(column_letter(16_384), "?");
}

#[test]
fn keys_parse_only_when_both_parts_are_numbers() {
    assert_eq!(parse_key("1,2"), Some((1, 2)));
    assert_eq!(parse_key("0,0"), Some((0, 0)));
    assert_eq!(parse_key("1,a"), None);
    assert_eq!(parse_key("a,1"), None);
    assert_eq!(parse_key("1"), None);
    assert_eq!(parse_key(""), None);
}

#[test]
fn text_commands_position_and_escape_the_run() {
    assert_eq!(text_cmd(48.0, 780.5, "a(b)"), "1 0 0 1 48.0 780.5 Tm (a\\(b\\)) Tj\n");
}
