// Tests for `predeclare_variables` and `strip_string_literals`. Split out of
// `predeclare_variables.rs` to keep that file under the AGENTS.md size limit.

use super::*;

#[test]
fn assignments_get_a_declaration_block_sorted_by_name() {
    let out = predeclare_variables("z = 1\na = 2\n");
    assert_eq!(out, "let a;\nlet z;\n\nz = 1\na = 2\n");
}

#[test]
fn a_script_without_assignments_is_returned_unchanged() {
    assert_eq!(predeclare_variables("TALK \"hi\"\n"), "TALK \"hi\"\n");
    assert_eq!(predeclare_variables(""), "");
}

#[test]
fn each_variable_is_declared_once() {
    let out = predeclare_variables("x = 1\nx = 2\n");
    assert_eq!(out.matches("let x;").count(), 1, "got: {out}");
}

#[test]
fn comparison_and_compound_operators_are_not_declarations() {
    for line in ["x == 1", "x != 1", "x <= 1", "x >= 1", "x += 1", "x -= 1", "x *= 1", "x /= 1"] {
        assert_eq!(
            predeclare_variables(&format!("{line}\n")),
            format!("{line}\n"),
            "{line} must not declare a variable"
        );
    }
}

#[test]
fn reserved_words_are_not_declared() {
    for line in ["if = 1", "save = 1", "print = 1", "call = 1", "get = 1"] {
        assert_eq!(
            predeclare_variables(&format!("{line}\n")),
            format!("{line}\n"),
            "{line} must not declare a variable"
        );
    }
}

#[test]
fn non_identifier_left_hand_sides_are_skipped() {
    for line in [
        "obj.field = 1",
        "list[0] = 1",
        "f(x) = 1",
        "TALK \"a=b\"",
        "1x = 2",
    ] {
        assert_eq!(
            predeclare_variables(&format!("{line}\n")),
            format!("{line}\n"),
            "{line} must not declare a variable"
        );
    }
}

#[test]
fn comments_and_blank_lines_are_ignored() {
    let out = predeclare_variables("' x = 1\n// y = 1\n# z = 1\n\nx = 1\n");
    assert_eq!(out.matches("let ").count(), 1, "got: {out}");
    assert!(out.contains("let x;"), "got: {out}");
}

#[test]
fn underscored_identifiers_are_declarations() {
    let out = predeclare_variables("_tmp = 1\n");
    assert_eq!(out, "let _tmp;\n\n_tmp = 1\n");
}

#[test]
fn strip_string_literals_blanks_the_text_and_keeps_the_quotes() {
    assert_eq!(strip_string_literals("\"abc\""), "\"   \"");
    assert_eq!(strip_string_literals("a\"b\"c"), "a\" \"c");
    assert_eq!(strip_string_literals("a == 1"), "a == 1");
}

#[test]
fn strip_string_literals_honours_escapes() {
    // The escaped quote does not close the literal, so the whole body is blanked.
    assert_eq!(strip_string_literals("\"a\\\"b\""), "\"  \"");
    assert_eq!(
        strip_string_literals("\"unterminated"),
        format!("\"{}", " ".repeat("unterminated".len()))
    );
}
