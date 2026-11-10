//! Offline unit tests for the quoted-text extractor (#2067).
//!
//! The chat assistant treats text between quotes as an explicit instruction
//! payload (a caption, a prompt, a title). `extract_quoted_text` accepts ASCII
//! and typographic quotes, and returns `None` when the message has no balanced
//! pair — an unterminated quote must not swallow the rest of the sentence.

use super::extract_quoted_text;

#[test]
fn straight_double_quotes_are_extracted() {
    assert_eq!(
        extract_quoted_text("render a clip titled \"Sunset\""),
        Some("Sunset".to_string())
    );
}

#[test]
fn straight_single_quotes_are_extracted() {
    assert_eq!(
        extract_quoted_text("use the caption 'hello world'"),
        Some("hello world".to_string())
    );
}

#[test]
fn typographic_quotes_are_extracted() {
    assert_eq!(
        extract_quoted_text("título \u{201c}Verão\u{201d}"),
        Some("Verão".to_string())
    );
}

#[test]
fn the_first_pair_wins() {
    assert_eq!(
        extract_quoted_text("\"one\" and \"two\""),
        Some("one".to_string())
    );
}

#[test]
fn mixed_quote_marks_still_pair() {
    // The scanner treats every quote mark as an opener/closer, so a message that
    // opens with `"` and closes with `'` yields the enclosed text.
    assert_eq!(
        extract_quoted_text("start \"middle' end"),
        Some("middle".to_string())
    );
}

#[test]
fn messages_without_a_pair_return_none() {
    assert_eq!(extract_quoted_text("no quotes here"), None);
    assert_eq!(extract_quoted_text(""), None);
    assert_eq!(extract_quoted_text("only one \" quote"), None);
}

#[test]
fn an_empty_pair_returns_none() {
    assert_eq!(extract_quoted_text("\"\""), None);
    assert_eq!(extract_quoted_text("''"), None);
}

#[test]
fn multibyte_text_is_sliced_on_character_boundaries() {
    // The scanner works on `chars`, so an emoji between the quotes is preserved
    // whole instead of being cut mid-codepoint.
    assert_eq!(
        extract_quoted_text("\"a 🚀 b\""),
        Some("a 🚀 b".to_string())
    );
}
