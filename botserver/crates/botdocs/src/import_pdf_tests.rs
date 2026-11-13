//! PDF import expectations (#2254): a real written PDF converts to paragraphs,
//! and a broken file fails cleanly instead of panicking.

use super::*;

/// A two-paragraph PDF built by this crate's own writer — the inverse of the
/// export path — so the fixture never depends on an external file.
fn sample_pdf() -> Vec<u8> {
    crate::utils_pdf::html_to_pdf(
        "<h1>Quarterly Report</h1><p>The first paragraph stays intact.</p>\
         <p>The second paragraph follows it.</p>",
    )
}

#[test]
fn a_written_pdf_becomes_editor_paragraphs_in_order() {
    let html = pdf_to_editor_html(&sample_pdf()).expect("a written PDF must be readable");

    assert!(html.starts_with("<p>"), "paragraph markup expected: {html}");
    assert!(
        html.contains("Quarterly Report"),
        "heading text lost: {html}"
    );
    assert!(
        html.contains("first paragraph"),
        "first paragraph text lost: {html}"
    );
    assert!(
        html.contains("second paragraph"),
        "second paragraph text lost: {html}"
    );

    let first = html.find("Quarterly Report").unwrap_or(usize::MAX);
    let second = html.find("first paragraph").unwrap_or(usize::MAX);
    let third = html.find("second paragraph").unwrap_or(usize::MAX);
    assert!(
        first < second && second < third,
        "reading order changed: {html}"
    );
    assert!(
        !html.contains("Unsupported format"),
        "the PDF must not fall through to the unsupported branch"
    );
}

#[test]
fn a_non_pdf_payload_is_refused_by_the_signature() {
    let error = pdf_to_editor_html(b"just some text, not a PDF").expect_err("must be refused");
    assert!(error.contains("not a PDF"), "unexpected error: {error}");
}

#[test]
fn an_empty_payload_is_refused() {
    let error = pdf_to_editor_html(b"").expect_err("must be refused");
    assert!(error.contains("empty"), "unexpected error: {error}");
}

#[test]
fn a_corrupt_pdf_returns_a_clean_error() {
    // Correct signature, unusable body: the reader must report, not panic.
    let error = pdf_to_editor_html(b"%PDF-1.4\nthis is not a real pdf body\n%%EOF\n")
        .expect_err("a corrupt PDF must be refused");
    assert!(
        error.contains("could not be read") || error.contains("no extractable text"),
        "unexpected error: {error}"
    );

    // A truncated file (signature only) must also fail cleanly.
    let truncated = pdf_to_editor_html(b"%PDF-1.7\n");
    assert!(truncated.is_err(), "a truncated PDF must be refused");
}

#[test]
fn paragraphs_join_wrapped_lines_and_break_on_pages() {
    let text =
        "Wrapped line one\ncontinues here\n\nSecond paragraph\n\u{000C}Third page paragraph\n";
    let paragraphs = paragraphs_from_text(text);

    assert_eq!(
        paragraphs,
        vec![
            "Wrapped line one continues here".to_string(),
            "Second paragraph".to_string(),
            "Third page paragraph".to_string(),
        ]
    );
}

#[test]
fn signature_detection_uses_the_magic_bytes() {
    assert!(looks_like_pdf(b"%PDF-1.4\n"));
    assert!(!looks_like_pdf(b"PK\x03\x04"));
    assert!(!looks_like_pdf(b""));
}
