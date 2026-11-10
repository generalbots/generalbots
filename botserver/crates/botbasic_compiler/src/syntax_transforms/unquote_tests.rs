// Tests for the parameter/quoting helpers shared by every syntax rewriter
// (`unquote`, `strip_trailing_stmt_semicolon`, `parse_parameters`,
// `preprocess_llm_keyword`). Split out of `unquote.rs` to keep that file under
// the AGENTS.md size limit.

use super::*;

#[test]
fn unquote_strips_surrounding_double_quotes_and_space() {
    assert_eq!(unquote("\"docs\""), "docs");
    assert_eq!(unquote("  \"docs\"  "), "docs");
    assert_eq!(unquote("docs"), "docs");
    // Only double quotes are removed; single quotes are BASIC literals too but
    // this helper is used for MCP/event names which are always double quoted.
    assert_eq!(unquote("'docs'"), "'docs'");
    assert_eq!(unquote("\"\""), "");
}

#[test]
fn unquote_removes_repeated_quotes() {
    assert_eq!(unquote("\"\"docs\"\""), "docs");
}

#[test]
fn strip_semicolon_removes_only_the_statement_terminator() {
    assert_eq!(strip_trailing_stmt_semicolon("vibe_run(\"x\");"), "vibe_run(\"x\")");
    assert_eq!(strip_trailing_stmt_semicolon("plain"), "plain");
    assert_eq!(strip_trailing_stmt_semicolon("plain;"), "plain");
    assert_eq!(strip_trailing_stmt_semicolon("a;b"), "a;b");
    assert_eq!(strip_trailing_stmt_semicolon(""), "");
    assert_eq!(strip_trailing_stmt_semicolon("   "), "");
}

#[test]
fn strip_semicolon_ignores_terminators_inside_literals() {
    // The `;` belongs to the string, so it must survive untouched.
    assert_eq!(
        strip_trailing_stmt_semicolon("\"build; app\""),
        "\"build; app\""
    );
    // A real terminator after a literal that itself contains `;`.
    assert_eq!(
        strip_trailing_stmt_semicolon("\"build; app\";"),
        "\"build; app\""
    );
    assert_eq!(strip_trailing_stmt_semicolon("'a;';"), "'a;'");
    // Unterminated literal: the trailing `;` is inside the open string.
    assert_eq!(strip_trailing_stmt_semicolon("\"open;"), "\"open;");
}

#[test]
fn parse_parameters_splits_on_top_level_commas() {
    assert_eq!(parse_parameters("a, b"), vec!["a", "b"]);
    assert_eq!(parse_parameters("a,b,c"), vec!["a", "b", "c"]);
    assert_eq!(parse_parameters(""), Vec::<String>::new());
    assert_eq!(parse_parameters("   "), Vec::<String>::new());
    assert_eq!(parse_parameters("single"), vec!["single"]);
}

#[test]
fn parse_parameters_keeps_quoted_commas_together() {
    assert_eq!(parse_parameters("\"a,b\", c"), vec!["\"a,b\"", "c"]);
    assert_eq!(parse_parameters("'a,b', c"), vec!["'a,b'", "c"]);
    assert_eq!(
        parse_parameters("\"to\", \"subject\", \"body, with comma\""),
        vec!["\"to\"", "\"subject\"", "\"body, with comma\""]
    );
}

#[test]
fn parse_parameters_keeps_parenthesised_arguments_together() {
    assert_eq!(parse_parameters("f(1, 2), x"), vec!["f(1, 2)", "x"]);
    assert_eq!(parse_parameters("a[1, 2], b"), vec!["a[1, 2]", "b"]);
    assert_eq!(parse_parameters("g(f(1, 2), 3), 4"), vec!["g(f(1, 2), 3)", "4"]);
}

#[test]
fn parse_parameters_skips_empty_slots() {
    assert_eq!(parse_parameters("a,,b"), vec!["a", "b"]);
    assert_eq!(parse_parameters("a,"), vec!["a"]);
    assert_eq!(parse_parameters(",a"), vec!["a"]);
}

#[test]
fn parse_parameters_recovers_from_unbalanced_closing_paren() {
    // A stray `)` must not swallow the rest of the parameter list: the depth
    // counter saturates at zero instead of going negative.
    assert_eq!(parse_parameters("a), b"), vec!["a)", "b"]);
}

#[test]
fn llm_keyword_gains_the_default_optimize_clause() {
    let out = preprocess_llm_keyword("x = LLM \"summarise this\"\n");
    assert_eq!(out, "x = LLM \"summarise this\" WITH OPTIMIZE FOR \"speed\"\n");
}

#[test]
fn llm_keyword_keeps_the_prompt_literal_intact() {
    // The prompt must stay a single well-formed string literal: the closing
    // quote used to be dropped, yielding `LLM "summarise WITH OPTIMIZE FOR
    // "speed"` (unparsable).
    let out = preprocess_llm_keyword("LLM \"ask the model\"\n");
    assert!(out.contains("\"ask the model\""), "literal mangled: {out}");
    assert_eq!(out.matches('"').count(), 4, "expected two literals: {out}");
}

#[test]
fn llm_keyword_is_idempotent() {
    let once = preprocess_llm_keyword("LLM \"prompt\"\n");
    let twice = preprocess_llm_keyword(&once);
    assert_eq!(once, twice);
    assert_eq!(twice.matches("WITH OPTIMIZE").count(), 1, "got: {twice}");
}

#[test]
fn llm_keyword_keeps_an_explicit_clause_on_the_same_line() {
    let script = "LLM \"prompt\" WITH OPTIMIZE FOR \"cost\"\n";
    assert_eq!(preprocess_llm_keyword(script), script);
}

#[test]
fn llm_keyword_keeps_an_explicit_clause_on_the_next_line() {
    let script = "email = LLM \"write a subject\"\n  WITH OPTIMIZE FOR \"cost\"\n";
    assert_eq!(preprocess_llm_keyword(script), script);
}

#[test]
fn llm_keyword_accepts_lowercase_and_ignores_embedded_tokens() {
    assert!(preprocess_llm_keyword("llm \"p\"\n").starts_with("LLM \"p\""));
    // `XLLM` is an identifier, not the keyword.
    let out = preprocess_llm_keyword("XLLM \"p\"\n");
    assert_eq!(out, "XLLM \"p\"\n");
}

#[test]
fn llm_keyword_leaves_unquoted_expressions_alone() {
    let script = "answer = LLM prompt_variable\n";
    assert_eq!(preprocess_llm_keyword(script), script);
}

#[test]
fn llm_keyword_preserves_surrounding_lines() {
    let script = "a = 1\nLLM \"p\"\nb = 2\n";
    let out = preprocess_llm_keyword(script);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "a = 1");
    assert_eq!(lines[2], "b = 2");
}
