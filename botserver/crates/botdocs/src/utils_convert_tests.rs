//! Format-conversion expectations: detection, HTML↔Markdown↔RTF↔ODT.

use super::*;

const DOCX_PREFIX: &[u8] = b"PK\x03\x04";

fn zip_with(marker: &str) -> Vec<u8> {
    let mut bytes = DOCX_PREFIX.to_vec();
    bytes.extend(std::iter::repeat(b'0').take(30));
    bytes.extend_from_slice(marker.as_bytes());
    bytes
}

#[test]
fn format_detection_uses_magic_bytes_then_text_markers() {
    let cases: Vec<(&str, Vec<u8>, &str)> = vec![
        ("docx", zip_with("word/document.xml"), "docx"),
        ("odt", zip_with("content.xml"), "odt"),
        ("bare zip", zip_with("nothing-here"), "zip"),
        ("rtf", b"{\\rtf1\\ansi}".to_vec(), "rtf"),
        ("doc", vec![0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1], "doc"),
        ("doctype", b"<!DOCTYPE html><p>x</p>".to_vec(), "html"),
        ("html", b"  <html><body>x</body></html>".to_vec(), "html"),
        ("heading", b"# Title".to_vec(), "markdown"),
        ("embedded heading", b"intro\n# Title".to_vec(), "markdown"),
        ("plain", b"just words".to_vec(), "txt"),
        ("empty", Vec::new(), "txt"),
        ("three bytes", b"abc".to_vec(), "txt"),
    ];
    for (label, bytes, expected) in cases {
        assert_eq!(detect_document_format(&bytes), expected, "case {label}");
    }
}

#[test]
fn html_to_markdown_maps_the_supported_tags() {
    let cases = [
        ("<h1>Title</h1>", "# Title\n"),
        ("<h3>Sub</h3>", "### Sub\n"),
        ("<p>a <strong>b</strong></p>", "a **b**\n\n"),
        ("<em>i</em>", "*i*"),
        ("<ul><li>one</li><li>two</li></ul>", "- one\n- two\n\n"),
        ("a<br />b", "a\nb"),
        ("<code>x</code>", "`x`"),
        ("<hr>", "\n---\n"),
        ("", ""),
    ];
    for (html, expected) in cases {
        assert_eq!(html_to_markdown(html), expected, "html {html:?}");
    }
}

#[test]
fn markdown_to_html_covers_blocks_and_inline_runs() {
    let cases = [
        ("# Title", "<h1>Title</h1>"),
        ("###### Deep", "<h6>Deep</h6>"),
        ("####### Not a heading", "<p>####### Not a heading</p>"),
        ("plain words", "<p>plain words</p>"),
        ("**bold**", "<strong>bold</strong>"),
        ("*italic*", "<em>italic</em>"),
        ("`code`", "<code>code</code>"),
        ("[site](https://example.com)", "<a href=\"https://example.com\">site</a>"),
        ("> quoted", "<blockquote>quoted</blockquote>"),
        ("---", "<hr>"),
        ("- a\n- b", "<ul><li>a</li><li>b</li></ul>"),
        ("- a\nplain", "<ul><li>a</li></ul><p>plain</p>"),
        ("", ""),
    ];
    for (md, expected) in cases {
        assert_eq!(markdown_to_html(md), expected, "markdown {md:?}");
    }
}

#[test]
fn fenced_code_keeps_its_body_and_escapes_markup() {
    let html = markdown_to_html("```\nlet x = 1 < 2;\n```");
    assert_eq!(html, "<pre>let x = 1 &lt; 2;\n</pre>");
    // An unterminated fence is still closed.
    assert_eq!(markdown_to_html("```\nbody"), "<pre>body\n</pre>");
}

#[test]
fn rtf_to_html_tracks_runs_and_control_words() {
    let cases = [
        ("{\\rtf1\\ansi Hello}", "<div>Hello</div>"),
        ("{\\rtf1 Bold \\b on\\b0  off}", "<div>Bold <strong>on</strong> off</div>"),
        ("{\\rtf1 \\i it\\i0  x}", "<div><em>it</em> x</div>"),
        ("{\\rtf1 \\ul u\\ulnone  x}", "<div><u>u</u> x</div>"),
        ("{\\rtf1 a\\par b}", "<div>a<br>b</div>"),
        ("{\\rtf1 a\\tab b}", "<div>a&nbsp;&nbsp;&nbsp;&nbsp;b</div>"),
        ("{\\rtf1 \\u233?}", "<div>é</div>"),
        ("", "<div></div>"),
    ];
    for (rtf, expected) in cases {
        assert_eq!(rtf_to_html(rtf), expected, "rtf {rtf:?}");
    }
}

#[test]
fn rtf_control_word_parameters_never_leak_into_the_text() {
    // `\fs24` carries a parameter; before the fix its digits printed as "24".
    let html = rtf_to_html("{\\rtf1\\fs24\\deff0 Sample}");
    assert_eq!(html, "<div>Sample</div>");
}

#[test]
fn html_to_rtf_emits_a_header_and_control_words() {
    let rtf = html_to_rtf("<p><strong>b</strong> plain</p>");
    assert!(rtf.starts_with("{\\rtf1\\ansi\\deff0\n"));
    assert!(rtf.contains("\\fonttbl"));
    assert!(rtf.contains("\\b b\\b0 "));
    assert!(rtf.contains("\\par\\par"));
    assert!(rtf.ends_with('}'));
}

#[test]
fn odt_content_becomes_html_with_matching_tags() {
    let cases = [
        ("<text:p>Hello</text:p>", "<div><p>Hello</p></div>"),
        (
            "<text:h text:outline-level=\"2\">Title</text:h>",
            "<div><h2>Title</h2></div>",
        ),
        (
            "<text:p>a<text:span text:style-name=\"Bold\">b</text:span>c</text:p>",
            "<div><p>a<strong>b</strong>c</p></div>",
        ),
        (
            "<text:p>a<text:span text:style-name=\"Italic\">b</text:span>c</text:p>",
            "<div><p>a<em>b</em>c</p></div>",
        ),
        ("<text:p>a<text:line-break/>b</text:p>", "<div><p>a<br>b</p></div>"),
        ("<text:p>a<text:tab/>b</text:p>", "<div><p>a&nbsp;&nbsp;&nbsp;&nbsp;b</p></div>"),
        ("", "<div></div>"),
    ];
    for (odt, expected) in cases {
        assert_eq!(odt_content_to_html(odt), expected, "odt {odt:?}");
    }
}

#[test]
fn html_to_odt_content_keeps_styles_and_wraps_paragraphs_once() {
    let odt = html_to_odt_content("<p>Hi</p>");
    assert!(odt.starts_with("<?xml version=\"1.0\""));
    assert!(odt.contains("<text:p>Hi</text:p>"));
    assert!(!odt.contains("<text:p><text:p>"), "paragraph wrapped twice: {odt}");
    assert!(odt.ends_with("</office:document-content>"));

    let styled = html_to_odt_content("<p><strong>b</strong> <em>i</em></p>");
    assert!(styled.contains("<text:span text:style-name=\"Bold\">b</text:span>"));
    assert!(styled.contains("<text:span text:style-name=\"Italic\">i</text:span>"));

    let heading = html_to_odt_content("<h1>Title</h1>");
    assert!(heading.contains("<text:h text:outline-level=\"1\">Title</text:h>"));

    let unsupported = html_to_odt_content("<p>a<span class=\"x\">b</span></p>");
    assert!(unsupported.contains("<text:p>ab</text:p>"), "{unsupported}");

    let breaks = html_to_odt_content("<p>a<br>b</p>");
    assert!(breaks.contains("<text:line-break/>"));
}

#[test]
fn convert_to_html_dispatches_on_the_detected_format() {
    assert_eq!(
        convert_to_html(b"plain\ntext").expect("txt"),
        "<p>plain</p><p>text</p>"
    );
    assert_eq!(
        convert_to_html(b"a < b").expect("txt escapes"),
        "<p>a &lt; b</p>"
    );
    assert_eq!(convert_to_html(b"# Title").expect("markdown"), "<h1>Title</h1>");
    assert_eq!(
        convert_to_html(b"{\\rtf1\\ansi Hi}").expect("rtf"),
        "<div>Hi</div>"
    );
    assert_eq!(
        convert_to_html(b"<html><body>hi</body></html>").expect("html"),
        "<html><body>hi</body></html>"
    );
    assert_eq!(
        convert_to_html(&zip_with("word/document.xml")).expect_err("docx unsupported"),
        "Unsupported format: docx"
    );
    assert_eq!(
        convert_to_html(&zip_with("other")).expect_err("zip unsupported"),
        "Unsupported format: zip"
    );
}
