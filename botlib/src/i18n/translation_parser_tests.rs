//! Pins the `.ftl`-subset parser every locale bundle is built from (#2064).
//!
//! The parser is pure text handling, so the expectations below are published
//! input/output pairs: a change in trimming, comment handling or multiline
//! accumulation shows up here instead of as a missing translation in the UI.

use super::TranslationFile;

fn value(file: &TranslationFile, key: &str) -> Option<String> {
    file.get(key).cloned()
}

#[test]
fn key_value_lines_are_parsed() {
    let file = TranslationFile::parse("greeting = Hello\nfarewell = Bye\n");
    assert_eq!(value(&file, "greeting").as_deref(), Some("Hello"));
    assert_eq!(value(&file, "farewell").as_deref(), Some("Bye"));
}

#[test]
fn comments_and_blank_lines_are_ignored() {
    let file = TranslationFile::parse("# a comment\n\n   \nkey = value\n");
    assert_eq!(value(&file, "key").as_deref(), Some("value"));
    assert_eq!(file.keys().count(), 1);
}

#[test]
fn spaces_around_the_separator_are_trimmed() {
    let file = TranslationFile::parse("  key   =   spaced value   \n");
    assert_eq!(value(&file, "key").as_deref(), Some("spaced value"));
}

#[test]
fn a_value_containing_equals_keeps_its_tail() {
    // Only the first `=` separates key from value: URLs and query strings
    // (`redirect = /login?next=a=b`) must survive verbatim.
    let file = TranslationFile::parse("redirect = /login?next=a=b\n");
    assert_eq!(value(&file, "redirect").as_deref(), Some("/login?next=a=b"));
}

#[test]
fn an_empty_value_is_an_empty_string() {
    let file = TranslationFile::parse("empty =\n");
    assert_eq!(value(&file, "empty").as_deref(), Some(""));
}

#[test]
fn a_braced_value_accumulates_following_lines() {
    let file = TranslationFile::parse("body = {\n  line one\n  line two\n}\n");
    assert_eq!(
        value(&file, "body").as_deref(),
        Some("{\nline one\nline two\n}")
    );
}

#[test]
fn an_unclosed_brace_at_eof_still_yields_the_accumulated_value() {
    let file = TranslationFile::parse("body = {\n  only line\n");
    assert_eq!(value(&file, "body").as_deref(), Some("{\nonly line"));
}

#[test]
fn a_following_key_closes_the_multiline_value() {
    let file = TranslationFile::parse("body = {\n  inside\n}\nafter = plain\n");
    assert_eq!(value(&file, "body").as_deref(), Some("{\ninside\n}"));
    assert_eq!(value(&file, "after").as_deref(), Some("plain"));
    assert_eq!(file.keys().count(), 2);
}

#[test]
fn a_balanced_brace_value_stays_on_one_line() {
    let file = TranslationFile::parse("hint = use {name} here\nnext = x\n");
    assert_eq!(value(&file, "hint").as_deref(), Some("use {name} here"));
    assert_eq!(value(&file, "next").as_deref(), Some("x"));
}

#[test]
fn merge_lets_the_right_hand_side_win() {
    let mut left = TranslationFile::parse("shared = left\nonly_left = a\n");
    let right = TranslationFile::parse("shared = right\nonly_right = b\n");
    left.merge(right);

    assert_eq!(value(&left, "shared").as_deref(), Some("right"));
    assert_eq!(value(&left, "only_left").as_deref(), Some("a"));
    assert_eq!(value(&left, "only_right").as_deref(), Some("b"));
}

#[test]
fn keys_is_the_union_after_a_merge() {
    let mut left = TranslationFile::parse("a = 1\n");
    left.merge(TranslationFile::parse("b = 2\n"));

    let mut keys: Vec<&str> = left.keys().collect();
    keys.sort_unstable();
    assert_eq!(keys, vec!["a", "b"]);
}

#[test]
fn an_absent_key_returns_none_without_panicking() {
    let file = TranslationFile::parse("a = 1\n");
    assert!(file.get("missing").is_none());
    assert!(file.get("").is_none());
    assert_eq!(file.keys().count(), 1);
}
