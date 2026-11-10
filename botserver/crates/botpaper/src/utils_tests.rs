//! Offline unit tests for the Paper renderer (#2067).
//!
//! `utils` turns stored markdown into the HTMX fragments the editor swaps in.
//! The escaping and the markdown state machine are pure; `format_relative_time`
//! takes its input timestamp as an argument and only reads the clock to compute
//! the delta, so every branch is exercised deterministically.

use super::*;
use chrono::Duration;

#[test]
fn html_escape_covers_every_dangerous_character() {
    assert_eq!(
        html_escape("<script>alert('x')</script>"),
        "&lt;script&gt;alert(&#39;x&#39;)&lt;/script&gt;"
    );
    assert_eq!(html_escape("a & b"), "a &amp; b");
    assert_eq!(html_escape("\"quoted\""), "&quot;quoted&quot;");
    assert_eq!(html_escape("plain"), "plain");
    assert_eq!(html_escape(""), "");
    // The ampersand is escaped first, so a literal entity is not double-decoded
    // back into markup.
    assert_eq!(html_escape("&lt;"), "&amp;lt;");
}

#[test]
fn document_list_item_escapes_its_text_and_carries_the_template_path() {
    let html = format_document_list_item("id-1", "<b>Title</b>", "just now", false);
    assert!(html.contains("data-id=\"id-1\""));
    // KNOWN DEFECT: `PAPER_BY_ID` contains `{id}` but the builder replaces
    // `:id`, so `hx-get` keeps the literal placeholder and clicking an item
    // requests the template path instead of the document. Pinned as-is.
    assert!(html.contains("hx-get=\"/api/ui/paper/{id}\""));
    assert!(html.contains("&lt;b&gt;Title&lt;/b&gt;"));
    assert!(!html.contains("<b>Title</b>"));
    assert!(!html.contains("new-item"));

    let fresh = format_document_list_item("id-2", "Draft", "5m ago", true);
    assert!(fresh.contains("paper-item new-item"));
    assert!(fresh.contains("Draft"));
}

#[test]
fn document_list_item_escapes_the_id_used_in_the_url() {
    let html = format_document_list_item("a\"b", "T", "t", false);
    assert!(html.contains("data-id=\"a&quot;b\""));
    assert!(!html.contains("data-id=\"a\"b\""));
}

#[test]
fn document_content_renders_empty_and_filled_bodies() {
    let empty = format_document_content("My Doc", "");
    assert!(empty.contains("data-title=\"My Doc\""));
    assert!(empty.contains("<p data-placeholder=\"Start writing...\"></p>"));

    let filled = format_document_content("Doc", "# Heading\n\nbody");
    assert!(filled.contains("<h1>Heading</h1>"));
    assert!(filled.contains("<p>body</p>"));
    assert!(!filled.contains("data-placeholder=\"Start writing...\""));
}

#[test]
fn ai_response_and_error_blocks_escape_their_payload() {
    let ai = format_ai_response("<img src=x onerror=alert(1)>");
    assert!(ai.contains("ai-response-content"));
    assert!(ai.contains("&lt;img src=x onerror=alert(1)&gt;"));
    assert!(!ai.contains("<img"));
    assert!(ai.contains("btn-copy"));
    assert!(ai.contains("btn-insert"));
    assert!(ai.contains("btn-replace"));

    let err = format_error("boom & <bang>");
    assert!(err.contains("error-message"));
    assert!(err.contains("boom &amp; &lt;bang&gt;"));
    assert!(err.contains("error-icon"));
}

#[test]
fn markdown_headings_are_level_specific() {
    let html = markdown_to_html("# one\n## two\n### three");
    assert!(html.contains("<h1>one</h1>"));
    assert!(html.contains("<h2>two</h2>"));
    assert!(html.contains("<h3>three</h3>"));
    // `#### four` has no branch and falls back to a paragraph with the hashes
    // preserved (no silent promotion to h3).
    let deep = markdown_to_html("#### four");
    assert!(deep.contains("<p>#### four</p>"));
    assert!(!deep.contains("<h4>"));
}

#[test]
fn markdown_unordered_and_todo_lists_open_and_close_once() {
    let html = markdown_to_html("- a\n- b");
    assert_eq!(html, "<ul><li>a</li><li>b</li></ul>");

    let star = markdown_to_html("* a");
    assert_eq!(star, "<ul><li>a</li></ul>");

    let todo = markdown_to_html("- [ ] open\n- [x] done");
    assert!(todo.starts_with("<ul class=\"todo-list\">"));
    assert!(todo.contains("<li><input type=\"checkbox\"> open</li>"));
    assert!(todo.contains("<li><input type=\"checkbox\" checked> done</li>"));
    assert!(todo.ends_with("</ul>"));
}

#[test]
fn markdown_ordered_list_uses_the_text_after_the_marker() {
    // KNOWN DEFECT: the list closer is hardcoded to `</ul>`, so an ordered list
    // is closed with the wrong tag (browsers still render it, validators do not).
    let html = markdown_to_html("1. first\n2. second");
    assert_eq!(html, "<ol><li>first</li><li>second</li></ul>");
    // A digit without a `. ` separator stays a paragraph.
    let plain = markdown_to_html("2026 was a year");
    assert!(plain.contains("<p>2026 was a year</p>"));
}

#[test]
fn markdown_blank_line_closes_a_list() {
    let html = markdown_to_html("- a\n\nparagraph");
    assert!(html.contains("</ul><br><p>paragraph</p>"));
}

#[test]
fn markdown_code_block_escapes_and_closes() {
    let html = markdown_to_html("```\n<b>raw</b>\n```");
    assert_eq!(html, "<pre><code>&lt;b&gt;raw&lt;/b&gt;\n</code></pre>");
}

#[test]
fn unterminated_code_block_and_list_are_closed_at_eof() {
    let code = markdown_to_html("```\nunclosed");
    assert!(code.starts_with("<pre><code>"));
    assert!(code.ends_with("</code></pre>"));

    let list = markdown_to_html("- item");
    assert!(list.ends_with("</ul>"));
}

#[test]
fn inline_markdown_emits_an_unbalanced_bold_tag() {
    // KNOWN DEFECT: both `**` delimiters are rewritten to `<b>`, so the closing
    // tag is missing. The text is still escaped; only the pairing is wrong.
    let html = markdown_to_html("a **bold** with `code`");
    assert!(html.contains("<b>bold<b>"));
    assert!(html.contains("<code>code</code>"));

    let underscore = markdown_to_html("__b__");
    assert!(underscore.contains("<b>b<b>"));
}

#[test]
fn inline_markdown_pairs_code_fences_and_leaves_text_untouched() {
    let html = markdown_to_html("a `x` b");
    assert_eq!(html.matches("<code>").count(), 1);
    assert_eq!(html.matches("</code>").count(), 1);
    // Text outside the backticks is not rewritten.
    assert!(html.contains("a <code>x</code> b"));
}

#[test]
fn inline_markdown_escapes_before_formatting() {
    // The security property holds even though the bold tags are unbalanced:
    // the payload is escaped before any tag is emitted.
    let html = markdown_to_html("**<script>**");
    assert!(html.contains("<b>&lt;script&gt;<b>"));
    assert!(!html.contains("<script>"));
}

#[test]
fn strip_markdown_removes_markers_and_keeps_text() {
    let stripped = strip_markdown("# Title\n\n- **bold** item\n- [x] done\n`code`");
    assert!(stripped.contains("Title\n"));
    assert!(stripped.contains("bold item\n"));
    assert!(stripped.contains("done\n"));
    assert!(stripped.contains("code\n"));
    assert!(!stripped.contains('#'));
    assert!(!stripped.contains('*'));
    assert!(!stripped.contains('`'));
    assert!(!stripped.contains("- [x]"));
}

#[test]
fn strip_markdown_drops_fence_markers_but_keeps_body() {
    let stripped = strip_markdown("```\nlet x = 1;\n```");
    assert_eq!(stripped, "let x = 1;\n");
}

#[test]
fn relative_time_covers_every_bucket() {
    let now = Utc::now();
    assert_eq!(format_relative_time(now), "just now");
    assert_eq!(format_relative_time(now - Duration::seconds(90)), "1m ago");
    assert_eq!(format_relative_time(now - Duration::minutes(59)), "59m ago");
    assert_eq!(format_relative_time(now - Duration::hours(2)), "2h ago");
    assert_eq!(format_relative_time(now - Duration::days(3)), "3d ago");
    // Older than a week falls back to an absolute month/day label.
    let old = now - Duration::days(30);
    assert_eq!(format_relative_time(old), old.format("%b %d").to_string());
    // A future timestamp is not "in the past": the negative delta is < 60 s.
    assert_eq!(format_relative_time(now + Duration::minutes(5)), "just now");
}
