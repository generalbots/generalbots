//! Offline unit tests for the bot path/context helpers (#2064).
//!
//! `utils` builds the Drive path of every bot artifact
//! (`{org}.gborg/{bot}.gbai/{sub}`) and trims text before it reaches a model.
//! Both are pure formatting; only `get_work_path`/`get_stack_path` read the
//! environment and are left to the integration suite.

use super::{
    build_absolute_bot_path, build_bot_path, estimate_token_count, sanitize_utf16_surrogates,
    truncate_text_for_model,
};
use uuid::Uuid;

const ORG: &str = "00000000-0000-0000-0000-000000000000";

#[test]
fn bot_path_wraps_the_bucket_in_the_org_workspace() {
    assert_eq!(
        build_bot_path(ORG, "salesbot", "salesbot.gbdialog/start.bas"),
        "00000000-0000-0000-0000-000000000000.gborg/salesbot.gbai/salesbot.gbdialog/start.bas"
    );
}

#[test]
fn bot_path_accepts_any_displayable_org_id() {
    // The org id is formatted, not parsed: a non-UUID label stays as-is.
    assert_eq!(
        build_bot_path("acme", "b", "x.txt"),
        "acme.gborg/b.gbai/x.txt"
    );
    let nil = Uuid::nil();
    assert_eq!(build_bot_path(nil, "b", "x"), format!("{nil}.gborg/b.gbai/x"));
}

#[test]
fn absolute_bot_path_prefixes_the_work_root() {
    assert_eq!(
        build_absolute_bot_path("/opt/gbo/work", ORG, "salesbot", "salesbot.gbkb/docs/a.md"),
        "/opt/gbo/work/00000000-0000-0000-0000-000000000000.gborg/salesbot.gbai/salesbot.gbkb/docs/a.md"
    );
    // A relative root is not normalised, only joined with a single separator.
    assert_eq!(
        build_absolute_bot_path("work", "acme", "b", "c"),
        "work/acme.gborg/b.gbai/c"
    );
}

#[test]
fn bot_paths_keep_empty_segments_visible() {
    // An empty sub-path is a caller bug; the builder must not silently drop the
    // separator, so the malformed path shows up in logs instead of pointing at
    // the bot root.
    assert_eq!(build_bot_path("o", "b", ""), "o.gborg/b.gbai/");
    assert_eq!(build_absolute_bot_path("/w", "o", "b", ""), "/w/o.gborg/b.gbai/");
}

#[test]
fn token_estimate_is_one_token_per_four_bytes() {
    assert_eq!(estimate_token_count(""), 0);
    assert_eq!(estimate_token_count("abcd"), 1);
    assert_eq!(estimate_token_count("abcde"), 1);
    assert_eq!(estimate_token_count(&"x".repeat(400)), 100);
}

#[test]
fn short_text_is_returned_untouched() {
    let text = "hello world";
    assert_eq!(truncate_text_for_model(text, "any", 100), text);
    // Exactly at the limit is not truncated.
    assert_eq!(truncate_text_for_model("abcd", "any", 1), "abcd");
}

#[test]
fn long_text_is_cut_and_marked() {
    let text = "a".repeat(10);
    let cut = truncate_text_for_model(&text, "any", 1);
    assert_eq!(cut, format!("{}...[truncated]", "a".repeat(4)));
    assert!(cut.ends_with("...[truncated]"));
    assert!(cut.len() < text.len() + "...[truncated]".len());
}

#[test]
fn truncation_never_splits_a_character() {
    // `chars().take` keeps the output valid UTF-8 even when the byte limit
    // falls inside a multi-byte character.
    let text = "áéíóú".repeat(4);
    let cut = truncate_text_for_model(&text, "any", 1);
    assert!(cut.ends_with("...[truncated]"));
    let body = cut.trim_end_matches("...[truncated]");
    assert_eq!(body.chars().count(), 4);
    assert!(body.chars().all(|c| c == 'á' || c == 'é' || c == 'í' || c == 'ó' || c == 'ú'));
}

#[test]
fn sanitizing_surrogates_is_identity_for_valid_utf8() {
    // A Rust `&str` can never hold a lone surrogate, so the guard must not
    // damage legitimate text — including astral-plane characters.
    for text in ["plain", "acentuação", "emoji 🚀", "nul\u{0}inside", ""] {
        assert_eq!(sanitize_utf16_surrogates(text), text);
    }
}
