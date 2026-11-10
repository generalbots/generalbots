//! Paragraph model and HTML↔text expectations.

use super::*;

fn paragraphs(html: &str) -> Vec<(String, String, bool, bool, bool)> {
    parse_html_to_paragraphs(html)
        .into_iter()
        .map(|p| (p.text, p.style, p.bold, p.italic, p.underline))
        .collect()
}

#[test]
fn block_tags_become_paragraphs_with_their_style() {
    let cases = [
        ("<p>Hello</p>", "p"),
        ("<h1>Title</h1>", "h1"),
        ("<h3>Sub</h3>", "h3"),
        ("<li>item</li>", "li"),
        ("<blockquote>quote</blockquote>", "blockquote"),
        ("<pre>code</pre>", "code"),
    ];
    for (html, style) in cases {
        let parsed = paragraphs(html);
        assert_eq!(parsed.len(), 1, "html {html:?}");
        assert_eq!(parsed[0].1, style, "html {html:?}");
    }
}

#[test]
fn consecutive_blocks_stay_separate_and_keep_their_order() {
    let parsed = paragraphs("<h1>One</h1><p>Two</p><p>Three</p>");
    let texts: Vec<&str> = parsed.iter().map(|p| p.0.as_str()).collect();
    assert_eq!(texts, ["One", "Two", "Three"]);
}

#[test]
fn line_breaks_and_entities_are_decoded_inside_a_paragraph() {
    assert_eq!(paragraphs("<p>a<br>b</p>")[0].0, "a\nb");
    assert_eq!(paragraphs("<p>a &amp; b</p>")[0].0, "a & b");
    assert_eq!(paragraphs("<p>&#233;t&#xE9;</p>")[0].0, "été");
    // A single decode: `&amp;lt;` is the text `&lt;`, never `<`.
    assert_eq!(paragraphs("<p>a&amp;lt;b</p>")[0].0, "a&lt;b");
}

#[test]
fn formatted_runs_mark_the_paragraph() {
    let cases = [
        ("<p><b>x</b>y</p>", (true, false, false)),
        ("<p><strong>x</strong></p>", (true, false, false)),
        ("<p><i>it</i></p>", (false, true, false)),
        ("<p><em>it</em></p>", (false, true, false)),
        ("<p><u>u</u></p>", (false, false, true)),
        ("<p>plain</p>", (false, false, false)),
    ];
    for (html, expected) in cases {
        let parsed = paragraphs(html);
        assert_eq!((parsed[0].2, parsed[0].3, parsed[0].4), expected, "html {html:?}");
    }
}

#[test]
fn empty_and_whitespace_only_input_yields_no_paragraphs() {
    for html in ["", "<p></p>", "<p>   </p>", "<div>\n</div>"] {
        assert!(paragraphs(html).is_empty(), "html {html:?}");
    }
}

#[test]
fn an_unclosed_block_still_keeps_its_text() {
    assert_eq!(paragraphs("<p>dangling")[0].0, "dangling");
    assert_eq!(html_to_paragraphs("<p>dangling"), vec!["dangling"]);
}

#[test]
fn paragraphs_to_html_escapes_each_paragraph() {
    let html = paragraphs_to_html(&["a < b".to_string(), "c & d".to_string()]);
    assert_eq!(html, "<p>a &lt; b</p><p>c &amp; d</p>");
    assert_eq!(paragraphs_to_html(&[]), "");
}

#[test]
fn entities_decode_once_and_unknown_ones_survive() {
    let cases = [
        ("&nbsp;", " "),
        ("&amp;", "&"),
        ("&lt;b&gt;", "<b>"),
        ("&quot;q&quot;", "\"q\""),
        ("&apos;a&apos;", "'a'"),
        ("&#233;", "é"),
        ("&#xE9;", "é"),
        ("&#39;", "'"),
        ("&unknown;", "&unknown;"),
        ("a & b", "a & b"),
        ("", ""),
    ];
    for (input, expected) in cases {
        assert_eq!(decode_html_entities(input), expected, "input {input:?}");
    }
}

#[test]
fn strip_html_drops_tags_but_keeps_text_and_entities() {
    assert_eq!(strip_html("<p>a <b>b</b></p>"), "a b");
    assert_eq!(strip_html("a &amp; b"), "a & b");
    assert_eq!(strip_html("&amp;lt;"), "&lt;");
    assert_eq!(strip_html(""), "");
}

#[test]
fn escape_html_covers_the_five_markup_characters() {
    assert_eq!(escape_html("<&>\"'"), "&lt;&amp;&gt;&quot;&#39;");
}

#[test]
fn word_count_ignores_markup_and_extra_whitespace() {
    let cases = [
        ("<p>hello world</p>", 2),
        ("  a   b  ", 2),
        ("", 0),
        ("<p></p>", 0),
        ("日本語 テスト", 2),
        ("one&nbsp;two", 2),
    ];
    for (input, expected) in cases {
        assert_eq!(count_words(input), expected, "input {input:?}");
    }
}

#[test]
fn docs_paths_are_lowercased_and_separators_neutralised() {
    assert_eq!(get_user_docs_path("Alice/Bob"), "users/alice_bob/docs");
    assert_eq!(
        get_user_docs_path("a\\b:c*d?e\"f<g>h|i"),
        "users/a_b_c_d_e_f_g_h_i/docs"
    );
    assert_eq!(get_user_docs_path(""), "users//docs");
}

#[tokio::test]
async fn document_bytes_cache_round_trips() {
    let id = generate_doc_id();
    assert!(get_cached_document_bytes(&id).await.is_none());
    cache_document_bytes(&id, vec![1, 2, 3]).await;
    assert_eq!(get_cached_document_bytes(&id).await, Some(vec![1, 2, 3]));
    remove_from_cache(&id).await;
    assert!(get_cached_document_bytes(&id).await.is_none());
}

#[test]
fn new_documents_start_empty_and_versioned() {
    let doc = create_new_document("user-1");
    assert_eq!(doc.title, "Untitled Document");
    assert_eq!(doc.content, "<p><br></p>");
    assert_eq!(doc.version, 1);
    assert_eq!(doc.owner_id, "user-1");
    assert_eq!(doc.id.len(), 36);
    assert_ne!(doc.id, create_new_document("user-1").id);
}
