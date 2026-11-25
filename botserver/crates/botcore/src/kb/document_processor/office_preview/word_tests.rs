//! DOCX preview tests. Kept in a sibling file so `word.rs` stays under the
//! repository size limit; `#[path]` keeps the module as `word::tests`.

use super::{docx_flow_html, word_html};
use std::collections::{HashMap, HashSet};
use std::io::{Cursor, Write};
use zip::write::FileOptions;
use zip::ZipWriter;

#[test]
fn letter_text_in_a_header_is_not_lost_when_the_body_is_empty() {
    let mut buffer = Cursor::new(Vec::new());
    {
        let mut zip = ZipWriter::new(&mut buffer);
        zip.start_file("word/document.xml", <FileOptions<()>>::default())
            .expect("document part");
        zip.write_all(
            br#"<w:document xmlns:w="w"><w:body></w:body></w:document>"#,
        )
        .expect("empty body");
        zip.start_file("word/header1.xml", <FileOptions<()>>::default())
            .expect("header part");
        zip.write_all(
            br#"<w:hdr xmlns:w="w"><w:p><w:r><w:t>Dear Sir or Madam</w:t></w:r></w:p></w:hdr>"#,
        )
        .expect("header text");
    }
    let html = word_html("letter.docx", &buffer.into_inner()).expect("preview");
    assert!(
        html.contains("Dear Sir or Madam"),
        "header-only letter must render its text: {html}"
    );
}

fn flow(xml: &str) -> String {
    docx_flow_html(xml, &HashMap::new(), &mut HashSet::new())
}

#[test]
fn docx_renders_tables_instead_of_loose_paragraphs() {
    let xml = r#"<w:document xmlns:w="w"><w:body><w:p><w:r><w:t>Quarterly Report</w:t></w:r></w:p><w:tbl><w:tr><w:tc><w:p><w:r><w:t>Region</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>Revenue</w:t></w:r></w:p></w:tc></w:tr><w:tr><w:tc><w:p><w:r><w:t>LATAM</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>120000</w:t></w:r></w:p></w:tc></w:tr></w:tbl></w:body></w:document>"#;
    let html = flow(xml);
    assert!(
        html.contains("<section>Quarterly Report</section>"),
        "{html}"
    );
    assert!(
        html.contains(
            "<table><tr><th>Region</th><th>Revenue</th></tr><tr><td>LATAM</td><td>120000</td></tr></table>"
        ),
        "{html}"
    );
    assert!(
        !html.contains("<section>Region</section>"),
        "table cells leaked as sections: {html}"
    );
}

#[test]
fn docx_heading_style_renders_a_heading() {
    let xml = r#"<w:document xmlns:w="w"><w:body><w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>Quarterly Report</w:t></w:r></w:p></w:body></w:document>"#;
    let html = flow(xml);
    assert!(html.contains("<h1>Quarterly Report</h1>"), "{html}");
}

#[test]
fn docx_drawings_are_rendered_where_they_appear() {
    let xml = r#"<w:document xmlns:w="w" xmlns:r="r" xmlns:a="a"><w:body><w:p><w:r><w:t>Before</w:t></w:r></w:p><w:p><w:r><w:drawing><a:blip r:embed="rId5"/></w:drawing></w:r></w:p><w:p><w:r><w:t>After</w:t></w:r></w:p></w:body></w:document>"#;
    let mut images = HashMap::new();
    images.insert(
        "rId5".to_string(),
        (
            "word/media/image1.png".to_string(),
            "data:image/png;base64,AAAA".to_string(),
        ),
    );
    let mut used = HashSet::new();
    let html = docx_flow_html(xml, &images, &mut used);
    let image = html.find("gb-doc-image").expect("image rendered inline");
    assert!(
        html.find("<section>Before</section>").unwrap() < image,
        "image must come after the preceding paragraph: {html}"
    );
    assert!(
        image < html.find("<section>After</section>").unwrap(),
        "image must come before the following paragraph: {html}"
    );
    assert!(used.contains("word/media/image1.png"));
}

#[test]
fn docx_page_break_becomes_a_separator() {
    let xml = r#"<w:document xmlns:w="w"><w:body><w:p><w:r><w:t>Page one</w:t></w:r></w:p><w:p><w:r><w:br w:type="page"/></w:r></w:p><w:p><w:r><w:t>Page two</w:t></w:r></w:p></w:body></w:document>"#;
    let html = flow(xml);
    assert!(html.contains("gb-page-break"), "{html}");
    assert!(
        html.find("Page one").unwrap() < html.find("gb-page-break").unwrap()
            && html.find("gb-page-break").unwrap() < html.find("Page two").unwrap(),
        "separator must sit between the pages: {html}"
    );
}
