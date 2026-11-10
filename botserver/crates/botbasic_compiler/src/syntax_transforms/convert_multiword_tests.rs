// Tests for `convert_multiword_keywords` — the multiword keyword → function
// call rewriter. Split out of `convert_multiword_keywords.rs` to keep that file
// under the AGENTS.md size limit.

use super::*;

fn convert(script: &str) -> String {
    convert_multiword_keywords(script)
}

#[test]
fn zero_argument_keywords_become_calls() {
    for (keyword, call) in [
        ("CLEAR KB", "clear_kb();"),
        ("CLEAR SUGGESTIONS", "clear_suggestions();"),
        ("CLEAR SWITCHERS", "clear_switchers();"),
        ("CLEAR TOOLS", "clear_tools();"),
        ("CLEAR WEBSITES", "clear_websites();"),
        ("CLEAR ERROR", "clear_error();"),
        ("VIBE TOOLS", "vibe_tools();"),
        ("ON ERROR RESUME NEXT", "on_error_resume_next();"),
        ("ON ERROR GOTO 0", "on_error_goto_0();"),
    ] {
        assert_eq!(convert(&format!("{keyword}\n")), format!("{call}\n"));
    }
}

#[test]
fn zero_argument_keywords_reject_extra_parameters() {
    let out = convert("CLEAR KB EXTRA\n");
    assert!(!out.contains("clear_kb()"), "got: {out}");
    assert_eq!(out, "CLEAR KB EXTRA;\n");
}

#[test]
fn single_argument_keywords_become_calls() {
    for (line, call) in [
        ("USE KB \"docs\"", "use_kb(\"docs\");"),
        ("USE TOOL \"t.bas\"", "use_tool(\"t.bas\");"),
        ("USE MODEL \"gpt\"", "use_model(\"gpt\");"),
        ("GET BOT MEMORY \"k\"", "get_bot_memory(\"k\");"),
        ("SET USER \"u\"", "set_user(\"u\");"),
        ("HAS ROLE \"admin\"", "has_role(\"admin\");"),
        ("CREATE TASK \"do it\"", "create_task(\"do it\");"),
        ("VIBE RUN \"build\"", "vibe_run(\"build\");"),
        ("DESCRIBE IMAGE src", "describe_image(src);"),
        ("DESCRIBE VIDEO src", "describe_video(src);"),
        ("SPEECH TO TEXT src", "speech_to_text(src);"),
        ("BOOK \"event\"", "book(\"event\");"),
    ] {
        assert_eq!(convert(&format!("{line}\n")), format!("{call}\n"));
    }
}

#[test]
fn two_argument_keywords_become_calls() {
    for (line, call) in [
        ("SET CONTEXT \"k\", v", "set_context(\"k\", v);"),
        ("SEND TO \"u\", \"m\"", "send_to(\"u\", \"m\");"),
        ("SEND TEAMS MESSAGE \"chat\", \"m\"", "send_teams_message(\"chat\", \"m\");"),
    ] {
        assert_eq!(convert(&format!("{line}\n")), format!("{call}\n"));
    }
}

#[test]
fn set_bot_memory_accepts_the_equals_separator() {
    assert_eq!(
        convert("SET BOT MEMORY \"k\" = v\n"),
        "set_bot_memory(\"k\", v);\n"
    );
    // The comma form still works.
    assert_eq!(
        convert("SET BOT MEMORY \"k\", v\n"),
        "set_bot_memory(\"k\", v);\n"
    );
}

#[test]
fn send_mail_requires_four_arguments() {
    assert_eq!(
        convert("SEND MAIL a, b, c, d\n"),
        "send_mail(a, b, c, d);\n"
    );
    let out = convert("SEND MAIL a, b, c\n");
    assert!(!out.contains("send_mail("), "got: {out}");
    assert_eq!(out, "SEND MAIL a, b, c;\n");
}

#[test]
fn create_draft_requires_four_arguments() {
    assert_eq!(
        convert("CREATE DRAFT a, b, c, d\n"),
        "create_draft(a, b, c, d);\n"
    );
    assert!(!convert("CREATE DRAFT a, b\n").contains("create_draft("));
}

#[test]
fn suggestion_keywords_keep_the_as_separator() {
    assert_eq!(
        convert("ADD SUGGESTION \"label\" AS \"text\"\n"),
        "ADD_SUGGESTION \"label\" as \"text\";\n"
    );
    assert_eq!(
        convert("ADD SUGGESTION \"label\", \"text\"\n"),
        "ADD_SUGGESTION \"label\" as \"text\";\n"
    );
    assert_eq!(
        convert("ADD SUGGESTION TOOL \"tool\" AS \"text\"\n"),
        "ADD_SUGGESTION_TOOL \"tool\" as \"text\";\n"
    );
    assert_eq!(
        convert("ADD SUGGESTION TEXT \"value\" AS \"text\"\n"),
        "ADD_SUGGESTION_TEXT \"value\" as \"text\";\n"
    );
}

#[test]
fn single_argument_suggestion_uses_the_simple_form() {
    // `ADD SUGGESTION "Relatórios"` is the simple form used by the shipped
    // templates: one argument, no label.
    assert_eq!(
        convert("ADD SUGGESTION \"Relatórios\"\n"),
        "ADD_SUGGESTION \"Relatórios\";\n"
    );
}

#[test]
fn switcher_keywords_normalise_to_the_as_form() {
    assert_eq!(
        convert("ADD SWITCHER \"id\" AS \"label\"\n"),
        "ADD_SWITCHER \"id\" as \"label\";\n"
    );
    assert_eq!(
        convert("ADD SWITCHER \"id\", \"label\"\n"),
        "ADD_SWITCHER \"id\" as \"label\";\n"
    );
}

#[test]
fn already_underscored_keywords_are_passed_through_with_a_terminator() {
    assert_eq!(
        convert("ADD_MEMBER \"name\" AS \"admin\"\n"),
        "ADD_MEMBER \"name\" AS \"admin\";\n"
    );
    assert_eq!(
        convert("ADD_SUGGESTION_TOOL \"t\" AS \"x\";\n"),
        "ADD_SUGGESTION_TOOL \"t\" AS \"x\";\n"
    );
}

#[test]
fn keyword_in_expression_position_keeps_the_prefix() {
    assert_eq!(
        convert("result = SEND TO \"u\", \"m\"\n"),
        "result = send_to(\"u\", \"m\");\n"
    );
}

#[test]
fn indentation_is_preserved() {
    assert_eq!(convert("  VIBE RUN \"x\"\n"), "  vibe_run(\"x\");\n");
}

#[test]
fn trailing_statement_terminator_does_not_leak_into_the_last_argument() {
    assert_eq!(convert("VIBE RUN \"build\";\n"), "vibe_run(\"build\");\n");
    // A `;` inside the literal belongs to the literal.
    assert_eq!(
        convert("VIBE RUN \"build; app\"\n"),
        "vibe_run(\"build; app\");\n"
    );
}

#[test]
fn get_path_is_rewritten_to_get_file() {
    assert_eq!(
        convert("GET \"/gbdialog/config.txt\"\n"),
        "get_file(\"/gbdialog/config.txt\");\n"
    );
    assert_eq!(
        convert("x = GET path + \".txt\"\n"),
        "x = get_file(path + \".txt\");\n"
    );
}

#[test]
fn get_data_keywords_are_protected_from_the_file_rewrite() {
    for line in [
        "GET FROM customers WHERE id = 1",
        "GET HTTP \"https://example.com\"",
        "GET QUEUE \"jobs\"",
        "GET SHAREPOINT \"docs\"",
        "GET SUMMARY \"s\"",
    ] {
        let out = convert(&format!("{line}\n"));
        assert!(!out.contains("get_file("), "{line} → {out}");
    }
}

#[test]
fn create_file_with_becomes_a_call() {
    assert_eq!(
        convert("CREATE FILE \"x.txt\" WITH \"hello\"\n"),
        "create_file(\"x.txt\", \"hello\");\n"
    );
    assert_eq!(
        convert("CREATE FILE dest + \".meta\" WITH \"k=\" + v\n"),
        "create_file(dest + \".meta\", \"k=\" + v);\n"
    );
}

#[test]
fn on_event_declaration_becomes_a_noop_call() {
    assert_eq!(
        convert("ON EVENT \"media_uploaded\"\n"),
        "on_event(\"media_uploaded\");\n"
    );
}

#[test]
fn plain_statements_gain_a_terminator_once() {
    assert_eq!(convert("TALK \"hi\"\n"), "TALK \"hi\";\n");
    assert_eq!(convert("TALK \"hi\";\n"), "TALK \"hi\";\n");
    assert_eq!(convert("IF x {\n"), "IF x {\n");
    assert_eq!(convert("}\n"), "}\n");
    assert_eq!(convert("label:\n"), "label:\n");
}

#[test]
fn comments_become_blank_lines_and_blank_lines_survive() {
    // Comment content is dropped, but the line keeps its place in the script.
    let out = convert("' note\n\nTALK \"x\"\n");
    assert_eq!(out, "\n\nTALK \"x\";\n");
}

#[test]
fn for_each_becomes_the_rhai_for_loop() {
    assert_eq!(
        convert("FOR EACH item IN items\nNEXT\n"),
        "for item in (items) {\n}\n"
    );
    // An expression with a trailing `;` (the .ast shape) is unwrapped too.
    assert_eq!(
        convert("FOR_EACH x in list_of(x);\nNEXT;\n"),
        "for x in (list_of(x)) {\n}\n"
    );
}
