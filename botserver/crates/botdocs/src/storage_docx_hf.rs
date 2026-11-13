use docx_rs::{Docx, Footer, Header, NumPages, PageNum, Paragraph, Run};

use crate::storage_core::strip_html;

/// Extract `.docs-header-zone` / `.docs-footer-zone` text from editor HTML and
/// return body HTML without those zones plus their plain text (#2155 DOC-08).
pub fn extract_header_footer_zones(html: &str) -> (String, Option<String>, Option<String>) {
    let mut out = String::with_capacity(html.len());
    let mut header: Option<String> = None;
    let mut footer: Option<String> = None;
    let mut rest = html;
    while let Some(start) = rest.find("<div") {
        out.push_str(&rest[..start]);
        let after = &rest[start..];
        let Some(end_tag) = after.find('>') else {
            out.push_str(after);
            return (out, header, footer);
        };
        let tag = &after[..=end_tag];
        let kind = if tag.contains("docs-header-zone") {
            Some("header")
        } else if tag.contains("docs-footer-zone") {
            Some("footer")
        } else {
            None
        };
        if let Some(kind) = kind {
            if let Some(close_rel) = after.to_ascii_lowercase().find("</div>") {
                let inner = &after[end_tag + 1..close_rel];
                let text = strip_html(inner).trim().to_string();
                if !text.is_empty() {
                    match kind {
                        "header" => header = Some(text),
                        "footer" => footer = Some(text),
                        _ => {}
                    }
                }
                rest = &after[close_rel + "</div>".len()..];
                continue;
            }
        }
        out.push_str(tag);
        rest = &after[end_tag + 1..];
    }
    out.push_str(rest);
    (out, header, footer)
}

pub fn build_hf_paragraph(text: &str) -> Paragraph {
    let mut para = Paragraph::new();
    let mut buf = String::new();
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '{' {
            let token = if chars.peek() == Some(&'{') {
                chars.next();
                read_token(&mut chars, true)
            } else {
                read_token(&mut chars, false)
            };
            match token.as_deref() {
                Some("page") => {
                    flush_text(&mut para, &mut buf);
                    para = para.add_page_num(PageNum::new());
                    continue;
                }
                Some("pages") | Some("numpages") => {
                    flush_text(&mut para, &mut buf);
                    para = para.add_num_pages(NumPages::new());
                    continue;
                }
                Some(t) => {
                    buf.push('{');
                    if text.contains("{{") {
                        buf.push('{');
                        buf.push_str(t);
                        buf.push_str("}}");
                    } else {
                        buf.push_str(t);
                        buf.push('}');
                    }
                    continue;
                }
                None => buf.push('{'),
            }
        } else {
            buf.push(ch);
        }
    }
    if !buf.is_empty() || text.is_empty() {
        para = para.add_run(Run::new().add_text(if buf.is_empty() { " " } else { &buf }));
    }
    para
}

fn flush_text(para: &mut Paragraph, buf: &mut String) {
    if !buf.is_empty() {
        let text = std::mem::take(buf);
        *para = para.clone().add_run(Run::new().add_text(text));
    }
}

fn read_token(
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
    double: bool,
) -> Option<String> {
    let mut name = String::new();
    while let Some(&c) = chars.peek() {
        if c == '}' {
            chars.next();
            if double && chars.peek() == Some(&'}') {
                chars.next();
            }
            return Some(name.trim().to_ascii_lowercase());
        }
        name.push(c);
        chars.next();
        if name.len() > 32 {
            return None;
        }
    }
    None
}

pub fn apply_docx_header_footer(
    docx: Docx,
    header: Option<String>,
    footer: Option<String>,
) -> Docx {
    let mut out = docx;
    if let Some(text) = header {
        out = out.header(Header::new().add_paragraph(build_hf_paragraph(&text)));
    }
    if let Some(text) = footer {
        out = out.footer(Footer::new().add_paragraph(build_hf_paragraph(&text)));
    }
    out
}
