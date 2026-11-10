// Tests for `convert_if_then_syntax` — the BASIC → Rhai control-flow rewriter.
// Split out of `convert_if_then_syntax.rs` to keep that file under the
// AGENTS.md size limit.

use super::*;

fn convert(script: &str) -> String {
    convert_if_then_syntax(script)
}

#[test]
fn simple_if_block_becomes_a_braced_block() {
    assert_eq!(
        convert("IF x = 1 THEN\nTALK \"a\"\nEND IF\n"),
        "if x == 1 {\n  TALK \"a\";\n}\n"
    );
}

#[test]
fn comparison_operators_are_translated() {
    assert_eq!(convert("IF a = 1 THEN\nEND IF\n"), "if a == 1 {\n}\n");
    assert_eq!(convert("IF a <> 1 THEN\nEND IF\n"), "if a != 1 {\n}\n");
    // An already-comparative condition is never rewritten twice.
    assert_eq!(convert("IF a >= 1 THEN\nEND IF\n"), "if a >= 1 {\n}\n");
    assert_eq!(convert("IF a != 1 THEN\nEND IF\n"), "if a != 1 {\n}\n");
}

#[test]
fn boolean_operators_are_translated() {
    assert_eq!(
        convert("IF a = 1 AND b = 2 THEN\nEND IF\n"),
        "if a == 1 && b == 2 {\n}\n"
    );
    assert_eq!(
        convert("IF a = 1 OR b = 2 THEN\nEND IF\n"),
        "if a == 1 || b == 2 {\n}\n"
    );
    assert_eq!(
        convert("IF a = 1 and b = 2 THEN\nEND IF\n"),
        "if a == 1 && b == 2 {\n}\n"
    );
}

#[test]
fn not_in_becomes_the_rhai_operator() {
    assert_eq!(
        convert("IF x NOT IN list THEN\nEND IF\n"),
        "if x !in list {\n}\n"
    );
    assert_eq!(
        convert("IF x not in list THEN\nEND IF\n"),
        "if x !in list {\n}\n"
    );
}

#[test]
fn bare_error_condition_uses_the_error_flag_function() {
    // `ERROR` is a function in the engine, not a variable.
    assert_eq!(convert("IF ERROR THEN\nEND IF\n"), "if error_flag() {\n}\n");
}

#[test]
fn else_and_else_if_chain() {
    let script = "IF a = 1 THEN\nTALK \"one\"\nELSE IF a = 2 THEN\nTALK \"two\"\nELSE\nTALK \"other\"\nEND IF\n";
    let out = convert(script);
    assert!(out.contains("} else if a == 2 {"), "got: {out}");
    assert!(out.contains("} else {"), "got: {out}");
    assert_eq!(out.matches("if ").count(), 2, "got: {out}");
}

#[test]
fn elseif_spelling_is_equivalent() {
    assert_eq!(
        convert("IF a = 1 THEN\nELSEIF a = 2 THEN\nEND IF\n"),
        "if a == 1 {\n} else if a == 2 {\n}\n"
    );
}

#[test]
fn inline_if_with_end_if_on_the_same_line() {
    assert_eq!(
        convert("IF x THEN TALK \"a\" END IF\n"),
        "if x {\n  TALK \"a\";\n}\n"
    );
}

#[test]
fn inline_if_else_on_the_same_line() {
    assert_eq!(
        convert("IF x THEN TALK \"a\" ELSE TALK \"b\" END IF\n"),
        "if x {\n  TALK \"a\";\n} else {\n  TALK \"b\";\n}\n"
    );
}

#[test]
fn nested_if_blocks_keep_the_stack_balanced() {
    let script = "IF a = 1 THEN\nIF b = 2 THEN\nTALK \"x\"\nEND IF\nEND IF\n";
    assert_eq!(
        convert(script),
        "if a == 1 {\nif b == 2 {\n  TALK \"x\";\n}\n}\n"
    );
}

#[test]
fn unmatched_end_if_emits_nothing() {
    assert_eq!(convert("END IF\n"), "");
}

#[test]
fn top_level_assignment_gains_let() {
    assert_eq!(convert("x = 1\n"), "let x = 1;\n");
    assert_eq!(convert("  x = 1;\n"), "let x = 1;\n");
}

#[test]
fn assignment_inside_a_block_does_not_gain_let() {
    assert_eq!(
        convert("IF x = 1 THEN\ny = 2\nEND IF\n"),
        "if x == 1 {\n  y = 2;\n}\n"
    );
}

#[test]
fn existing_let_is_not_duplicated() {
    assert_eq!(convert("let x = 1\n"), "let x = 1;\n");
    assert_eq!(convert("LET x = 1\n"), "LET x = 1;\n");
}

#[test]
fn statement_keywords_are_not_treated_as_assignments() {
    for line in [
        "merge = 1",
        "update = 1",
        "save = 1",
        "insert = 1",
        "talk = 1",
        "print = 1",
        "return = 1",
    ] {
        let out = convert(&format!("{line}\n"));
        assert!(!out.starts_with("let "), "{line} → {out}");
    }
}

#[test]
fn equals_inside_a_string_literal_is_not_an_assignment() {
    assert_eq!(
        convert("create_file(p, \"a=b\")\n"),
        "create_file(p, \"a=b\");\n"
    );
    assert_eq!(convert("TALK \"x = y\"\n"), "TALK \"x = y\";\n");
}

#[test]
fn comparison_lines_are_not_treated_as_assignments() {
    for line in ["x == 1", "x != 1", "x <= 1", "x >= 1", "x += 1", "x -= 1"] {
        let out = convert(&format!("{line}\n"));
        assert!(!out.starts_with("let "), "{line} → {out}");
    }
}

#[test]
fn with_block_becomes_a_map_literal() {
    assert_eq!(
        convert("WITH obj\na = 1\nb = \"two\"\nEND WITH\n"),
        "let obj = #{\n a: 1,\n b: \"two\",\n};\n"
    );
}

#[test]
fn save_statement_keeps_the_table_quoted() {
    let out = convert("SAVE \"customers\", a, b\n");
    assert!(out.starts_with("SAVE \"customers\","), "got: {out}");
    assert!(out.trim_end().ends_with(';'), "got: {out}");
    assert!(!out.contains("let SAVE"), "got: {out}");
}

#[test]
fn send_email_becomes_send_mail() {
    assert_eq!(
        convert("SEND EMAIL a@b.c, Hi, Body\n"),
        "send_mail(a@b.c, Hi, Body, []);\n"
    );
}

#[test]
fn comments_and_blank_lines_are_dropped() {
    let out = convert("' a comment\n\n// another\n# hash\nTALK \"x\"\n");
    assert_eq!(out, "TALK \"x\";\n");
}

#[test]
fn while_loops_keep_their_braces() {
    let out = convert("while i < 3 {\nTALK \"x\"\n}\n");
    assert!(out.starts_with("while i < 3 {\n"), "got: {out}");
    assert!(out.trim_end().ends_with('}'), "got: {out}");
    assert!(out.contains("TALK \"x\";"), "got: {out}");
}

#[test]
fn statement_terminators_are_added_only_once() {
    assert_eq!(convert("TALK \"x\"\n"), "TALK \"x\";\n");
    assert_eq!(convert("TALK \"x\";\n"), "TALK \"x\";\n");
    // A closing brace already terminates the statement.
    assert_eq!(convert("if x {\n"), "if x {\n");
    assert_eq!(convert("}\n"), "}\n");
}

#[test]
fn select_lines_are_left_terminator_free() {
    // SELECT/CASE is converted by its own pass; this one must not append `;`.
    let out = convert("SELECT x\nCASE 1\nEND SELECT\n");
    assert!(!out.contains("SELECT x;"), "got: {out}");
}

#[test]
fn trailing_continuation_comma_suppresses_the_terminator() {
    let out = convert("x = 1,\n");
    assert_eq!(out, "let x = 1,\n");
}
