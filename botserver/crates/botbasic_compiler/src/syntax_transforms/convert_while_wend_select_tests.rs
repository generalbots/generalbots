// Tests for `convert_while_wend_syntax`, `convert_select_case_syntax`,
// `convert_keywords_to_lowercase` and `rewrite_bare_datetime_tokens`. Split out
// of `convert_while_wend_syntax.rs` to keep that file under the AGENTS.md
// size limit.

use super::*;

#[test]
fn while_wend_becomes_a_braced_loop() {
    assert_eq!(
        convert_while_wend_syntax("WHILE i < 3\nTALK \"x\"\nWEND\n"),
        "while i < 3 {\nTALK \"x\"\n}\n"
    );
}

#[test]
fn while_wend_is_case_insensitive_and_keeps_the_condition_verbatim() {
    assert_eq!(
        convert_while_wend_syntax("while count(items) > 0\nWEND\n"),
        "while count(items) > 0 {\n}\n"
    );
}

#[test]
fn while_wend_leaves_other_lines_untouched() {
    assert_eq!(
        convert_while_wend_syntax("TALK \"WEND is not a terminator here\"\nWENDX\n\n"),
        "TALK \"WEND is not a terminator here\"\nWENDX\n\n"
    );
}

#[test]
fn select_case_becomes_an_if_chain() {
    let script = "SELECT choice\nCASE 1\nTALK \"a\"\nCASE 2\nTALK \"b\"\nEND SELECT\n";
    assert_eq!(
        convert_select_case_syntax(script),
        "if choice == 1 {\n  TALK \"a\";\n} else if choice == 2 {\n  TALK \"b\";\n}\n"
    );
}

#[test]
fn select_case_vba_spelling_uses_the_expression_as_subject() {
    // `SELECT CASE expr` is the spelling used by the shipped templates; the
    // subject is `expr`, never `CASE expr`.
    let script = "SELECT CASE UCASE(action)\nCASE 1\nTALK \"a\"\nEND SELECT\n";
    let out = convert_select_case_syntax(script);
    assert_eq!(out, "if UCASE(action) == 1 {\n  TALK \"a\";\n}\n");
    assert!(!out.contains("if CASE"), "subject kept the keyword: {out}");
}

#[test]
fn select_case_else_becomes_the_else_branch() {
    let script = "SELECT choice\nCASE 1\nTALK \"a\"\nCASE ELSE\nTALK \"b\"\nEND SELECT\n";
    assert_eq!(
        convert_select_case_syntax(script),
        "if choice == 1 {\n  TALK \"a\";\n} else {\n  TALK \"b\";\n}\n"
    );
}

#[test]
fn select_case_else_as_the_only_branch_keeps_a_condition() {
    let out = convert_select_case_syntax("SELECT choice\nCASE ELSE\nTALK \"b\"\nEND SELECT\n");
    assert_eq!(out, "if true {\n  TALK \"b\";\n}\n");
}

#[test]
fn select_case_keeps_numeric_literals_numeric() {
    // An integer subject compared with the string "1" never matches in Rhai.
    let out = convert_select_case_syntax("SELECT n\nCASE 1\nTALK \"a\"\nEND SELECT\n");
    assert!(out.contains("if n == 1 {"), "got: {out}");
    let out = convert_select_case_syntax("SELECT n\nCASE -1.5\nTALK \"a\"\nEND SELECT\n");
    assert!(out.contains("if n == -1.5 {"), "got: {out}");
}

#[test]
fn select_case_quotes_text_and_bare_words() {
    let out = convert_select_case_syntax("SELECT s\nCASE \"yes\"\nTALK \"a\"\nEND SELECT\n");
    assert!(out.contains("if s == \"yes\" {"), "got: {out}");
    let out = convert_select_case_syntax("SELECT s\nCASE pending\nTALK \"a\"\nEND SELECT\n");
    assert!(out.contains("if s == \"pending\" {"), "got: {out}");
}

#[test]
fn select_case_body_loses_its_let_and_gains_a_terminator() {
    let out = convert_select_case_syntax("SELECT s\nCASE 1\nlet y = 1\nEND SELECT\n");
    assert_eq!(out, "if s == 1 {\n  y = 1;\n}\n");
}

#[test]
fn select_without_cases_emits_nothing() {
    assert_eq!(convert_select_case_syntax("SELECT x\nEND SELECT\n"), "");
}

#[test]
fn select_case_leaves_sql_select_untouched() {
    let script = "SELECT name FROM users WHERE id = 1\n";
    assert_eq!(convert_select_case_syntax(script), script);
}

#[test]
fn select_case_preserves_surrounding_statements() {
    let script = "TALK \"before\"\nSELECT x\nCASE 1\nTALK \"a\"\nEND SELECT\nTALK \"after\"\n";
    let out = convert_select_case_syntax(script);
    assert!(out.starts_with("TALK \"before\"\n"), "got: {out}");
    assert!(out.trim_end().ends_with("TALK \"after\""), "got: {out}");
}

#[test]
fn rhai_builtins_are_lowercased() {
    assert_eq!(convert_keywords_to_lowercase("IF x THEN\n"), "if x THEN\n");
    assert_eq!(convert_keywords_to_lowercase("LET x = TRUE\n"), "let x = true\n");
    assert_eq!(convert_keywords_to_lowercase("FOR i IN xs\n"), "for i in xs\n");
    assert_eq!(convert_keywords_to_lowercase("DO WHILE\n"), "do while\n");
    assert_eq!(
        convert_keywords_to_lowercase("CONST a = 1\nRETURN a\n"),
        "const a = 1\nreturn a\n"
    );
}

#[test]
fn string_literals_are_never_lowercased() {
    // The text is user-facing data, not code.
    assert_eq!(
        convert_keywords_to_lowercase("TALK \"IF you DARE, LET it BE\"\n"),
        "TALK \"IF you DARE, LET it BE\"\n"
    );
    assert_eq!(
        convert_keywords_to_lowercase("TALK \"say \\\"IF\\\" now\"\n"),
        "TALK \"say \\\"IF\\\" now\"\n"
    );
}

#[test]
fn identifiers_containing_builtins_are_untouched() {
    assert_eq!(
        convert_keywords_to_lowercase("IF_ME = INPUT + FORMAT\n"),
        "IF_ME = INPUT + FORMAT\n"
    );
}

#[test]
fn bare_today_and_now_tokens_become_calls() {
    assert_eq!(
        rewrite_bare_datetime_tokens("let t = TODAY;"),
        "let t = TODAY();"
    );
    assert_eq!(
        rewrite_bare_datetime_tokens("x = NOW + 1"),
        "x = NOW() + 1"
    );
    assert_eq!(
        rewrite_bare_datetime_tokens("TALK \"a\" + TODAY"),
        "TALK \"a\" + TODAY()"
    );
}

#[test]
fn existing_datetime_calls_are_not_rewritten_twice() {
    assert_eq!(
        rewrite_bare_datetime_tokens("let t = TODAY();"),
        "let t = TODAY();"
    );
    assert_eq!(
        rewrite_bare_datetime_tokens("let t = NOW ( );"),
        "let t = NOW ( );"
    );
}

#[test]
fn datetime_tokens_inside_literals_and_longer_words_are_untouched() {
    assert_eq!(
        rewrite_bare_datetime_tokens("TALK \"TODAY is good\""),
        "TALK \"TODAY is good\""
    );
    assert_eq!(rewrite_bare_datetime_tokens("TODAYS = 1"), "TODAYS = 1");
    assert_eq!(
        rewrite_bare_datetime_tokens("TALK \"a\\\"TODAY\""),
        "TALK \"a\\\"TODAY\""
    );
}
