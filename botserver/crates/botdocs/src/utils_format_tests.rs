//! Formatting helper expectations.

use super::*;
use chrono::{Duration, TimeZone};

fn at(offset: Duration) -> (DateTime<Utc>, DateTime<Utc>) {
    let now = Utc
        .with_ymd_and_hms(2026, 3, 10, 12, 0, 0)
        .single()
        .expect("fixed now");
    (now - offset, now)
}

#[test]
fn relative_time_buckets() {
    let cases = [
        (Duration::seconds(30), "just now"),
        (Duration::minutes(1), "1 minute ago"),
        (Duration::minutes(5), "5 minutes ago"),
        (Duration::hours(1), "1 hour ago"),
        (Duration::hours(23), "23 hours ago"),
        (Duration::days(1), "1 day ago"),
        (Duration::days(6), "6 days ago"),
        (Duration::days(7), "1 week ago"),
        (Duration::days(29), "4 weeks ago"),
        (Duration::days(60), "Jan 09, 2026"),
    ];
    for (offset, expected) in cases {
        let (then, now) = at(offset);
        assert_eq!(
            format_relative_time_at(then, now),
            expected,
            "offset {offset:?}"
        );
    }
}

#[test]
fn future_timestamps_read_as_just_now() {
    let (then, now) = at(-Duration::minutes(5));
    assert_eq!(format_relative_time_at(then, now), "just now");
}

#[test]
fn truncation_counts_characters_not_bytes() {
    let cases = [
        ("hello", 5, "hello"),
        ("hello", 6, "hello"),
        ("hello world", 8, "hello..."),
        ("hello world", 5, "hello..."),
        ("abcdef", 3, "abc..."),
        ("ééé", 5, "ééé"),
        ("ééééééé", 4, "éééé..."),
    ];
    for (text, max, expected) in cases {
        assert_eq!(
            truncate_text(text, max),
            expected,
            "text {text:?} max {max}"
        );
    }
}

#[test]
fn html_escape_and_strip_html_agree_with_the_storage_module() {
    assert_eq!(html_escape("<&>\"'"), "&lt;&amp;&gt;&quot;&#x27;");
    assert_eq!(strip_html("<p>a</p>"), "a");
    assert_eq!(strip_html("a &amp; b"), "a & b");
    assert_eq!(strip_html("&amp;lt;"), "&lt;");
}

#[test]
fn word_count_ignores_markup() {
    assert_eq!(count_words("<p>one two</p>"), 2);
    assert_eq!(count_words(""), 0);
}

#[test]
fn filenames_keep_only_safe_characters() {
    let cases = [
        ("report.docx", "report.docx"),
        ("../../etc/passwd", ".._.._etc_passwd"),
        ("a b", "a_b"),
        ("_leading_", "leading"),
        ("relatório-2026.pdf", "relatório-2026.pdf"),
        ("", ""),
    ];
    for (input, expected) in cases {
        assert_eq!(sanitize_filename(input), expected, "input {input:?}");
    }
}

#[test]
fn error_payloads_are_json_objects() {
    assert_eq!(
        format_error("boom"),
        serde_json::json!({ "error": "boom", "success": false })
    );
}

#[test]
fn document_list_items_carry_a_relative_stamp_and_word_count() {
    let now = Utc
        .with_ymd_and_hms(2026, 3, 10, 12, 0, 0)
        .single()
        .expect("fixed now");
    let item = format_document_list_item("doc-1", "Notes", now - Duration::days(60), 12);
    assert_eq!(item["id"], "doc-1");
    assert_eq!(item["title"], "Notes");
    assert_eq!(item["word_count"], 12);
    assert_eq!(item["updated_relative"], "Jan 09, 2026");
    assert_eq!(item["updated_at"], (now - Duration::days(60)).to_rfc3339());
}

#[test]
fn document_content_payload_counts_words_from_markup() {
    let now = Utc
        .with_ymd_and_hms(2026, 3, 10, 12, 0, 0)
        .single()
        .expect("fixed now");
    let doc = format_document_content("doc-1", "Notes", "<p>alpha beta</p>", now, now);
    assert_eq!(doc["word_count"], 2);
    assert_eq!(doc["content"], "<p>alpha beta</p>");
    assert_eq!(doc["created_at"], now.to_rfc3339());
}

#[test]
fn generated_ids_are_unique_uuids() {
    let first = generate_document_id();
    let second = generate_document_id();
    assert_eq!(first.len(), 36);
    assert_ne!(first, second);
}
