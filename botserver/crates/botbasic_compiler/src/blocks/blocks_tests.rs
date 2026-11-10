// Tests for the BEGIN TALK / BEGIN MAIL block rewriters. Split out of
// `blocks/mod.rs` to keep that file under the AGENTS.md size limit.

use super::mail::convert_mail_line_with_substitution;
use super::talk::convert_talk_line_with_substitution;
use super::*;

#[test]
fn talk_line_without_substitution_is_quoted() {
    assert_eq!(convert_talk_line_with_substitution("plain"), "TALK \"plain\"");
    assert_eq!(convert_talk_line_with_substitution(""), "TALK \"\"");
}

#[test]
fn talk_line_interpolates_a_variable() {
    assert_eq!(
        convert_talk_line_with_substitution("Hello ${name}!"),
        "TALK \"Hello \" + name + \"!\""
    );
    assert_eq!(
        convert_talk_line_with_substitution("${a} ${b}"),
        "TALK a + \" \" + b"
    );
}

#[test]
fn talk_line_keeps_a_function_call_together() {
    assert_eq!(
        convert_talk_line_with_substitution("len ${len(items)}"),
        "TALK \"len \" + len(items)"
    );
}

#[test]
fn talk_line_escapes_embedded_quotes() {
    assert_eq!(
        convert_talk_line_with_substitution("say \"hi\""),
        "TALK \"say \\\"hi\\\"\""
    );
}

#[test]
fn talk_block_of_one_line_uses_a_single_chunk() {
    assert_eq!(
        convert_talk_block(&["Hello ${name}!".to_string()]),
        "let __talk_chunk_0__ = \"Hello \" + name + \"!\";\nTALK __talk_chunk_0__;\n"
    );
}

#[test]
fn empty_talk_block_talks_nothing() {
    assert_eq!(convert_talk_block(&[]), "TALK \"\";\n");
}

#[test]
fn talk_block_joins_lines_with_newlines() {
    let lines = vec!["one".to_string(), "two".to_string()];
    assert_eq!(
        convert_talk_block(&lines),
        "let __talk_chunk_0__ = \"one\" + \"\\n\" + \"two\";\nTALK __talk_chunk_0__;\n"
    );
}

#[test]
fn talk_block_splits_into_five_line_chunks() {
    let lines: Vec<String> = (1..=6).map(|n| format!("line{n}")).collect();
    let out = convert_talk_block(&lines);
    assert!(out.contains("let __talk_chunk_0__ ="), "got: {out}");
    assert!(out.contains("let __talk_chunk_1__ = \"line6\";"), "got: {out}");
    assert!(
        out.contains("TALK __talk_chunk_0__ + \"\\n\" + __talk_chunk_1__;"),
        "got: {out}"
    );
}

#[test]
fn mail_line_without_substitution_is_quoted() {
    assert_eq!(convert_mail_line_with_substitution("plain"), "\"plain\"");
    assert_eq!(convert_mail_line_with_substitution(""), "");
}

#[test]
fn mail_line_interpolates_a_variable() {
    assert_eq!(
        convert_mail_line_with_substitution("Hi ${name}"),
        "\"Hi \" + name"
    );
    assert_eq!(convert_mail_line_with_substitution("${name}"), "name");
}

#[test]
fn mail_block_takes_the_subject_line_and_the_body() {
    let lines = vec![
        "SUBJECT: Hello".to_string(),
        String::new(),
        "Body ${x}".to_string(),
    ];
    assert_eq!(
        convert_mail_block("a@b.c", &lines),
        "let __mail_body_0__ = \"Body \" + x;\nsend_mail(\"a@b.c\", \"Hello\", __mail_body_0__, []);\n"
    );
}

#[test]
fn mail_block_quotes_a_literal_recipient_and_keeps_a_variable_one() {
    let out = convert_mail_block("\"a@b.c\"", &[]);
    assert_eq!(out, "send_mail(\"a@b.c\", \"\", \"\", []);\n");
    let out = convert_mail_block("recipient", &[]);
    assert_eq!(out, "send_mail(recipient, \"\", \"\", []);\n");
}

#[test]
fn mail_block_without_subject_or_body_sends_empty_strings() {
    let out = convert_mail_block("a@b.c", &[]);
    assert_eq!(out, "send_mail(\"a@b.c\", \"\", \"\", []);\n");
}

#[test]
fn begin_talk_block_is_converted_in_place() {
    assert_eq!(
        convert_begin_blocks("BEGIN TALK\nHello\nEND TALK\n"),
        "let __talk_chunk_0__ = \"Hello\";\nTALK __talk_chunk_0__;\n"
    );
}

#[test]
fn begin_mail_block_is_converted_in_place() {
    // The mail conversion already ends its statement; the block writer adds the
    // separating newline, so the block ends with a blank line.
    assert_eq!(
        convert_begin_blocks("BEGIN MAIL a@b.c\nSUBJECT: Hi\nBody\nEND MAIL\n"),
        "let __mail_body_0__ = \"Body\";\nsend_mail(\"a@b.c\", \"Hi\", __mail_body_0__, []);\n\n"
    );
}

#[test]
fn statements_outside_a_block_are_preserved() {
    assert_eq!(
        convert_begin_blocks("TALK \"before\"\nBEGIN TALK\nHi\nEND TALK\nTALK \"after\"\n"),
        "TALK \"before\"\nlet __talk_chunk_0__ = \"Hi\";\nTALK __talk_chunk_0__;\nTALK \"after\"\n"
    );
}

#[test]
fn comments_and_blank_lines_outside_a_block_are_dropped() {
    assert_eq!(
        convert_begin_blocks("' note\n\nTALK \"x\"\n"),
        "TALK \"x\"\n"
    );
}
