// Tests for the GOTO → state-machine rewriter, split out of
// `goto_transform.rs` to keep that file under the AGENTS.md size limit.

use super::*;

#[test]
fn detects_plain_goto_and_labels() {
    assert!(has_goto_constructs("GOTO done\ndone:\nTALK \"x\""));
    assert!(has_goto_constructs("loop:\nTALK \"x\""));
    assert!(!has_goto_constructs("TALK \"hello\""));
    assert!(!has_goto_constructs("ON ERROR GOTO handler"));
}

#[test]
fn label_lines_reject_comments_and_case_markers() {
    assert!(is_label_line("start:"));
    assert!(is_label_line("_inner:"));
    assert!(is_label_line("Label_2:"));
    assert!(!is_label_line("' start:"));
    assert!(!is_label_line("// start:"));
    assert!(!is_label_line("REM start:"));
    assert!(!is_label_line("CASE:"));
    assert!(!is_label_line("DEFAULT:"));
    assert!(!is_label_line("ELSE:"));
    assert!(!is_label_line("2start:"));
    assert!(!is_label_line("not a label"));
    assert!(!is_label_line(":"));
}

#[test]
fn extract_label_returns_the_bare_name() {
    assert_eq!(extract_label("start:"), Some("start".to_string()));
    assert_eq!(extract_label("  retry_1:  "), Some("retry_1".to_string()));
    assert_eq!(extract_label("TALK \"x\""), None);
}

#[test]
fn extract_goto_target_handles_quotes_and_whitespace() {
    assert_eq!(extract_goto_target("GOTO done"), Some("done".to_string()));
    assert_eq!(extract_goto_target("goto \"done\""), Some("done".to_string()));
    assert_eq!(extract_goto_target("IF x THEN GOTO retry"), Some("retry".to_string()));
    assert_eq!(extract_goto_target("TALK \"x\""), None);
}

#[test]
fn source_without_goto_is_returned_unchanged() {
    let source = "TALK \"hello\"\nTALK \"world\"";
    assert_eq!(transform_goto(source), source);
}

#[test]
fn goto_is_rewritten_into_a_state_machine() {
    let source = "GOTO second\nTALK \"skipped\"\nsecond:\nTALK \"done\"";
    let output = transform_goto(source);

    assert!(output.contains("let __goto_label = \"__start\";"));
    assert!(output.contains("while __goto_label != \"__exit\""));
    assert!(output.contains("if __goto_label == \"second\""));
    assert!(output.contains("__goto_label = \"second\"; continue;"));
    assert!(output.contains("TALK \"done\""));
    // Every source line stays inside some block; the GOTO is the last
    // reachable statement of the first block.
    let start_block = output
        .split("if __goto_label == \"second\"")
        .next()
        .expect("start block precedes the second block");
    assert!(start_block.contains("TALK \"skipped\""));
    assert!(start_block.contains("continue;"));
}

#[test]
fn goto_state_machine_guards_against_infinite_loops() {
    let output = transform_goto("loop:\nGOTO loop");
    assert!(output.contains("let __goto_max_iterations = 1000000;"));
    assert!(output.contains("throw \"GOTO loop exceeded maximum iterations. Possible infinite loop.\";"));
}

#[test]
fn conditional_goto_variants_become_if_blocks() {
    assert_eq!(
        transform_line("IF x = 1 THEN GOTO done"),
        "if x = 1 { __goto_label = \"done\"; continue; }"
    );
    assert_eq!(
        transform_line("IF x = 1 THEN GOTO done".replace(" THEN GOTO ", " GOTO ").as_str()),
        "if x = 1 { __goto_label = \"done\"; continue; }"
    );
    assert_eq!(
        transform_line("GOTO done"),
        "__goto_label = \"done\"; continue;"
    );
}

#[test]
fn on_error_goto_and_plain_lines_are_left_alone() {
    assert_eq!(
        transform_line("ON ERROR GOTO handler"),
        "ON ERROR GOTO handler"
    );
    assert_eq!(transform_line("TALK \"hi\""), "TALK \"hi\"");
    assert_eq!(transform_line("  TALK \"hi\"  "), "TALK \"hi\"");
}

#[test]
fn comments_are_preserved_as_line_comments() {
    let output = transform_goto("' a note\nGOTO end\nend:\nTALK \"x\"");
    assert!(output.contains("//"));
    assert!(output.contains("a note"));
}

#[test]
fn blocks_chain_to_the_next_label_in_source_order() {
    let lines = vec!["TALK \"a\"", "first:", "TALK \"b\"", "second:", "TALK \"c\""];
    let labels: HashSet<String> =
        ["first".to_string(), "second".to_string()].into_iter().collect();
    let blocks = split_into_blocks(&lines, &labels);

    assert_eq!(blocks.len(), 3);
    assert_eq!(blocks[0].name, "__start");
    assert_eq!(blocks[0].next_label.as_deref(), Some("first"));
    assert_eq!(blocks[1].name, "first");
    assert_eq!(blocks[1].next_label.as_deref(), Some("second"));
    assert_eq!(blocks[2].name, "second");
    assert_eq!(blocks[2].next_label, None);
}

#[test]
fn a_lone_label_without_goto_is_left_alone() {
    // A label alone is not a jump: the source is returned untouched.
    let source = "done:\nTALK \"x\"";
    assert_eq!(transform_goto(source), source);
}

#[test]
fn an_undefined_goto_target_still_emits_the_jump() {
    // The label does not exist, so the jump can never be taken; the transform
    // must still produce a valid state machine instead of panicking.
    let output = transform_goto("GOTO nowhere\nTALK \"x\"");
    assert!(output.contains("__goto_label = \"nowhere\"; continue;"), "got: {output}");
    assert!(output.contains("while __goto_label != \"__exit\""), "got: {output}");
}

#[test]
fn blocks_keep_comments_as_line_comments() {
    let lines = vec!["' a note", "TALK \"a\"", "next:", "TALK \"b\""];
    let labels: HashSet<String> = ["next".to_string()].into_iter().collect();
    let blocks = split_into_blocks(&lines, &labels);

    assert_eq!(blocks[0].lines[0], "//  a note");
    assert_eq!(blocks[0].lines[1], "TALK \"a\"");
    assert_eq!(blocks[1].name, "next");
}

#[test]
fn empty_source_yields_one_empty_start_block() {
    let blocks = split_into_blocks(&[], &HashSet::new());
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].name, "__start");
    assert!(blocks[0].lines.is_empty());
    assert_eq!(blocks[0].next_label, None);
}

#[test]
fn the_state_machine_guards_every_block() {
    let output = transform_goto("a:\nTALK \"a\"\nb:\nGOTO a");
    assert!(output.contains("if __goto_label == \"a\" {"), "got: {output}");
    assert!(output.contains("if __goto_label == \"b\" {"), "got: {output}");
    assert!(output.contains("let __goto_iterations = 0;"), "got: {output}");
    assert!(output.trim_end().ends_with('}'), "got: {output}");
}
